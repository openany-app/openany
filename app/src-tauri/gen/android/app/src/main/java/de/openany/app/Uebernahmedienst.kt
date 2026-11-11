package de.openany.app

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import androidx.core.content.ContextCompat

/*
 * DER DIENST WAEHREND EINER UEBERNAHME (docs/plan-viele-dateien.md, D3):
 * haelt das Programm am Leben, solange die Schlange aus D2 Fotos uebernimmt,
 * und zeigt den Stand -- „openany übernimmt Fotos · 237 von 4.515" mit
 * „Pausieren".
 *
 * NUR DIE HUELLE. Die Schlange laeuft in Rust (`src-tauri/src/uebernahme.rs`);
 * hier wird ihr Stand gelesen (`stand`) und „Pausieren" weitergegeben.
 * Endet die Schlange, endet der Dienst.
 *
 * `dataSync`: Seit Android 15 darf ein solcher Dienst 6 Stunden am Tag
 * laufen. Danach ruft Android `onTimeout`; die Schlange pausiert, und der
 * Hinweis sagt ehrlich, warum. Beim naechsten Oeffnen geht es weiter.
 */
class Uebernahmedienst : Service() {
  @Volatile private var faden: Thread? = null

  override fun onBind(intent: Intent?): IBinder? = null

  override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
    if (intent?.action == AKTION_PAUSIEREN) {
      pausieren(true)
      beenden()
      return START_NOT_STICKY
    }
    kanalAnlegen(this)
    val hinweis = hinweis(this, 0, 0)
    if (Build.VERSION.SDK_INT >= 29) {
      startForeground(ID, hinweis, ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC)
    } else {
      startForeground(ID, hinweis)
    }
    if (faden == null) {
      faden = Thread {
        // `erledigt|gesamt|laeuft` -- siehe uebernahme.rs.
        while (true) {
          val teile = stand().split('|')
          val erledigt = teile.getOrNull(0)?.toIntOrNull() ?: 0
          val gesamt = teile.getOrNull(1)?.toIntOrNull() ?: 0
          val laeuft = teile.getOrNull(2) == "1"
          // Erst der Lauf entscheidet, wann Schluss ist: Er steht schon auf
          // „laeuft", bevor der Dienst startet.
          if (!laeuft) break
          try {
            NotificationManagerCompat.from(this).notify(ID, hinweis(this, erledigt, gesamt))
          } catch (_: SecurityException) {
          }
          try { Thread.sleep(1500) } catch (_: InterruptedException) { break }
        }
        beenden()
      }.apply { start() }
    }
    return START_NOT_STICKY
  }

  // Android 15: Die Frist fuer dataSync ist um.
  override fun onTimeout(startId: Int, fgsType: Int) {
    pausieren(true)
    try {
      NotificationManagerCompat.from(this).notify(
        ID_FRIST,
        NotificationCompat.Builder(this, KANAL)
          .setSmallIcon(R.drawable.ic_stat_openany)
          .setContentTitle(getString(R.string.uebernahme_titel))
          .setContentText(getString(R.string.uebernahme_frist))
          .setContentIntent(oeffnen(this))
          .setAutoCancel(true)
          .build()
      )
    } catch (_: SecurityException) {
    }
    beenden()
  }

  private fun beenden() {
    faden = null
    stopForeground(STOP_FOREGROUND_REMOVE)
    stopSelf()
  }

  companion object {
    private const val ID = 7301
    private const val ID_FRIST = 7302
    private const val KANAL = "uebernahme"
    private const val AKTION_PAUSIEREN = "de.openany.app.UEBERNAHME_PAUSIEREN"

    /** Gesetzt in MainActivity.onCreate. */
    @Volatile var kontext: Context? = null

    // Von Rust (uebernahme.rs) ueber JNI.
    @JvmStatic external fun stand(): String
    @JvmStatic external fun pausieren(an: Boolean)

    /** Von Rust, wenn die Schlange anlaeuft. */
    @JvmStatic
    fun starten() {
      val k = kontext ?: return
      try {
        ContextCompat.startForegroundService(k, Intent(k, Uebernahmedienst::class.java))
      } catch (_: Exception) {
        // Aus dem Hintergrund darf ein Dienst ab Android 12 nicht starten --
        // die Schlange laeuft dann, solange das Programm laeuft.
      }
    }

    private fun oeffnen(ctx: Context): PendingIntent =
      PendingIntent.getActivity(
        ctx, 0,
        Intent(ctx, MainActivity::class.java).addFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP),
        PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
      )

    private fun hinweis(ctx: Context, erledigt: Int, gesamt: Int) =
      NotificationCompat.Builder(ctx, KANAL)
        .setSmallIcon(R.drawable.ic_stat_openany)
        .setContentTitle(ctx.getString(R.string.uebernahme_titel))
        .setContentText(if (gesamt > 0) ctx.getString(R.string.uebernahme_stand, erledigt, gesamt) else null)
        .setProgress(gesamt, erledigt, gesamt == 0)
        .setOngoing(true)
        .setOnlyAlertOnce(true)
        .setContentIntent(oeffnen(ctx))
        .addAction(
          0,
          ctx.getString(R.string.uebernahme_pausieren),
          PendingIntent.getService(
            ctx, 1,
            Intent(ctx, Uebernahmedienst::class.java).setAction(AKTION_PAUSIEREN),
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
          ),
        )
        .build()

    private fun kanalAnlegen(ctx: Context) {
      if (Build.VERSION.SDK_INT < 26) return
      val nm = ctx.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
      nm.createNotificationChannel(
        NotificationChannel(KANAL, ctx.getString(R.string.kanal_uebernahme), NotificationManager.IMPORTANCE_LOW).apply {
          description = ctx.getString(R.string.kanal_uebernahme_beschreibung)
        }
      )
    }
  }
}
