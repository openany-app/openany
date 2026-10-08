package de.openany.app

import android.Manifest
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.content.pm.ServiceInfo
import android.net.ConnectivityManager
import android.net.Network
import android.os.Build
import android.os.IBinder
import android.util.Log
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import androidx.core.content.ContextCompat

/*
 * DER WACHDIENST: haelt eine dauerhafte, sparsame Leitung zu openanys ntfy
 * und zeigt eine Benachrichtigung, wenn eine Nachricht kommt -- ohne
 * Google, ohne Zusatz-App. Plan: docs/plan-app-neuaufsatz.md, Phase 6.
 *
 * DIESE KLASSE IST NUR DIE HUELLE. Die Leitung, das Deuten der Signale, der
 * Abgleich und die Entscheidung, WAS angezeigt wird, stehen in Rust
 * (`src-tauri/src/wachdienst.rs` und das Crate `openany-meldungen`). Hier
 * steht nur, was Android verlangt: ein Vordergrunddienst mit sichtbarem
 * Hinweis, die Netzmeldungen des Systems und das Anzeigen selbst.
 *
 * `remoteMessaging` UND NICHT `dataSync`: Ein dataSync-Dienst darf seit
 * Android 15 nur sechs Stunden am Tag laufen und nach einem Neustart nicht
 * von selbst starten. Genau fuer „haelt eine Verbindung fuer Nachrichten"
 * gibt es remoteMessaging.
 *
 * KEINE WACHSPERRE. Eine offene TCP-Verbindung braucht keine: Kommen Daten,
 * weckt das Netz das Geraet. Eine dauerhafte Sperre hielte die CPU wach und
 * kostete genau den Akku, den der ganze Aufbau sparen soll.
 *
 * Rust ruft `laufen` auf einem eigenen Faden und kehrt erst zurueck, wenn
 * `anhalten` kommt. Auf diesem Faden ruft Rust auch `zeige` und `protokoll`
 * zurueck -- dort findet JNI die Klassen der App.
 */
class Wachdienst : Service() {
  private var faden: Thread? = null
  private var netzRueckruf: ConnectivityManager.NetworkCallback? = null
  @Volatile private var angehalten = false

  override fun onBind(intent: Intent?): IBinder? = null

  override fun onCreate() {
    super.onCreate()
    kontext = applicationContext
    kanaeleAnlegen(this)

    val hinweis = NotificationCompat.Builder(this, KANAL_DIENST)
      .setSmallIcon(R.drawable.ic_stat_openany)
      .setContentTitle("openany")
      .setContentText(getString(R.string.wachdienst_hinweis))
      .setPriority(NotificationCompat.PRIORITY_MIN)
      .setOngoing(true)
      .setShowWhen(false)
      .setContentIntent(oeffnen(this))
      .build()
    if (Build.VERSION.SDK_INT >= 34) {
      startForeground(ID_DIENST, hinweis, ServiceInfo.FOREGROUND_SERVICE_TYPE_REMOTE_MESSAGING)
    } else {
      startForeground(ID_DIENST, hinweis)
    }

    // Der Rust-Teil liest als Erstes den Ausweis -- dafuer muss der Tresor
    // angemeldet sein, auch wenn das Fenster nie offen war (Neustart).
    Tresor.bereit()
    Standbild.bereit()

    // NETZWECHSEL SOFORT MELDEN. Nach WLAN -> Mobilfunk ist die alte
    // Verbindung tot, ohne dass es jemand merkt; ohne diese Meldung
    // wartete Rust die volle Lese-Frist von vier Minuten ab.
    val netz = getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager
    val rueckruf = object : ConnectivityManager.NetworkCallback() {
      override fun onAvailable(network: Network) = netz(true)
      override fun onLost(network: Network) {
        // Verloren ist nur „dieses" Netz; ein anderes kann schon da sein.
        netz(netz.activeNetwork != null)
      }
    }
    netz.registerDefaultNetworkCallback(rueckruf)
    netzRueckruf = rueckruf

    faden = Thread({
      val grund = try {
        laufen(dataDir.absolutePath, cacheDir.absolutePath)
      } catch (e: Throwable) {
        Log.w(TAG, "Wachdienst: Aufruf gescheitert", e)
        "Aufruf gescheitert"
      }
      Log.i(TAG, "Wachdienst endete: $grund")
      // Von selbst geendet (nicht gekoppelt, keine Instanz): Ein Dienst
      // ohne Leitung soll nicht mit Hinweis weiterstehen.
      if (!angehalten) stopSelf()
    }, "openany-wachdienst").apply { start() }
  }

  override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int = START_STICKY

  override fun onDestroy() {
    angehalten = true
    anhalten()
    netzRueckruf?.let {
      (getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager).unregisterNetworkCallback(it)
    }
    super.onDestroy()
  }

  companion object {
    private const val TAG = "openany"
    private const val KANAL_DIENST = "wachdienst"
    private const val KANAL_NACHRICHTEN = "nachrichten"
    private const val ID_DIENST = 1
    private const val ID_NACHRICHT = 2
    private const val EINSTELLUNG = "wachdienst"

    @Volatile private var kontext: Context? = null

    init {
      System.loadLibrary("openany_app_lib")
    }

    /** `wachdienst.rs`: kehrt erst nach [anhalten] zurueck. */
    @JvmStatic external fun laufen(daten: String, zwischen: String): String
    @JvmStatic external fun anhalten()
    @JvmStatic external fun netz(da: Boolean)
    /** Der Server vergisst das Thema dieses Geraets. Blockiert -- nicht auf dem UI-Faden. */
    @JvmStatic external fun abmelden(daten: String, zwischen: String): String

    /** Von Rust gerufen, wenn eine Nachricht angezeigt werden soll. */
    @JvmStatic
    fun zeige(titel: String, text: String) {
      val ctx = kontext ?: return
      // Leerer Text heisst „Nachricht ohne Vorschau": Rust kennt die Sprache
      // des Systems nicht, die Uebersetzung liegt hier (res/values*/wachdienst.xml).
      val anzeige = text.ifBlank { ctx.getString(R.string.neue_nachricht) }
      if (Build.VERSION.SDK_INT >= 33 &&
        ContextCompat.checkSelfPermission(ctx, Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED
      ) {
        Log.i(TAG, "Wachdienst: Benachrichtigung nicht erlaubt")
        return
      }
      val n = NotificationCompat.Builder(ctx, KANAL_NACHRICHTEN)
        .setSmallIcon(R.drawable.ic_stat_openany)
        .setContentTitle(titel)
        .setContentText(anzeige)
        .setStyle(NotificationCompat.BigTextStyle().bigText(anzeige))
        .setCategory(NotificationCompat.CATEGORY_MESSAGE)
        .setPriority(NotificationCompat.PRIORITY_HIGH)
        .setAutoCancel(true)
        .setContentIntent(oeffnen(ctx))
        .build()
      // EINE Kennung: Die neueste ersetzt die vorige, statt dass sich zehn
      // Hinweise auf „Neue Nachricht" stapeln.
      NotificationManagerCompat.from(ctx).notify(ID_NACHRICHT, n)
    }

    /** Von Rust gerufen -- `eprintln!` landet auf Android im Nichts. */
    @JvmStatic
    fun protokoll(text: String) {
      Log.i(TAG, text)
    }

    /** Ob der Mensch ihn will. Vorgabe: nein -- er traegt einen Dauerhinweis. */
    fun gewuenscht(ctx: Context): Boolean =
      ctx.getSharedPreferences("openany", Context.MODE_PRIVATE).getBoolean(EINSTELLUNG, false)

    fun setzen(ctx: Context, an: Boolean) {
      ctx.getSharedPreferences("openany", Context.MODE_PRIVATE).edit().putBoolean(EINSTELLUNG, an).apply()
      if (an) {
        starten(ctx)
      } else {
        ctx.stopService(Intent(ctx, Wachdienst::class.java))
        Thread({
          Tresor.bereit()
          Standbild.bereit()
          Log.i(TAG, "Wachdienst: " + abmelden(ctx.dataDir.absolutePath, ctx.cacheDir.absolutePath))
        }, "openany-abmelden").start()
      }
    }

    fun starten(ctx: Context) {
      ContextCompat.startForegroundService(ctx, Intent(ctx, Wachdienst::class.java))
    }

    private fun oeffnen(ctx: Context): PendingIntent =
      PendingIntent.getActivity(
        ctx, 0,
        Intent(ctx, MainActivity::class.java).addFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP),
        PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
      )

    /*
     * ZWEI KANAELE, weil der Mensch sie getrennt einstellen koennen soll:
     * den Dauerhinweis ganz still (oder aus), die Nachrichten laut.
     */
    private fun kanaeleAnlegen(ctx: Context) {
      if (Build.VERSION.SDK_INT < 26) return
      val nm = ctx.getSystemService(NotificationManager::class.java)
      nm.createNotificationChannel(
        NotificationChannel(KANAL_DIENST, ctx.getString(R.string.kanal_wachdienst), NotificationManager.IMPORTANCE_MIN).apply {
          description = ctx.getString(R.string.kanal_wachdienst_beschreibung)
          setShowBadge(false)
        }
      )
      nm.createNotificationChannel(
        NotificationChannel(KANAL_NACHRICHTEN, ctx.getString(R.string.kanal_nachrichten), NotificationManager.IMPORTANCE_HIGH).apply {
          description = ctx.getString(R.string.kanal_nachrichten_beschreibung)
        }
      )
    }
  }
}

/*
 * NACH DEM NEUSTART (und nach einem Update) wieder anlaufen -- sonst waere
 * der Schalter nach dem ersten Ausschalten des Geraets eine Luege. Beide
 * Anlaesse gehoeren zu den Ausnahmen, in denen Android einen
 * Vordergrunddienst aus dem Hintergrund starten laesst.
 */
class Startschuss : BroadcastReceiver() {
  override fun onReceive(ctx: Context, intent: Intent) {
    val anlass = intent.action
    if (anlass != Intent.ACTION_BOOT_COMPLETED && anlass != Intent.ACTION_MY_PACKAGE_REPLACED) return
    if (Wachdienst.gewuenscht(ctx)) Wachdienst.starten(ctx)
  }
}
