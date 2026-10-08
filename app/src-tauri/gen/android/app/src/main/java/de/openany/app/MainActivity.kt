package de.openany.app

import android.Manifest
import android.content.ActivityNotFoundException
import android.content.ContentValues
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.net.wifi.WifiManager
import android.os.Build
import android.os.Bundle
import android.os.PowerManager
import android.provider.MediaStore
import android.provider.Settings
import android.webkit.JavascriptInterface
import android.webkit.WebView
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.core.content.FileProvider
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat
import java.io.File
import java.lang.ref.WeakReference
import kotlin.math.roundToInt
import org.json.JSONObject

class MainActivity : TauriActivity() {
  /*
   * DIE RAENDER KOMMEN VON ANDROID, NICHT AUS CSS.
   *
   * Seit Android 15 zeichnet jede App bis unter Status- und Gestenleiste
   * (edge-to-edge). Die Webansicht meldet das ueber `env(safe-area-inset-*)`
   * nicht verlaesslich: unten kam am 06.09.2026 auf dem Tablet 0 an, oben lag
   * am 15.09.2026 der Inhalt unter der Statusleiste. Deshalb liest die
   * Oberflaeche die Werte hier ab (in CSS-Pixeln) -- `src/raender.js`.
   */
  private val raender = Raender()

  /*
   * GERAETE IN DER NAEHE BRAUCHEN MULTICAST.
   *
   * Android wirft eingehende Multicast-Pakete weg, solange keine App eine
   * Sperre haelt -- Akku. Die Suchprobe am 15.09.2026 lief als `adb shell` und
   * brauchte sie nicht; im App-Prozess gilt die Regel. Gehalten wird sie nur,
   * solange das Programm vorn ist: Im Hintergrund sucht es ohnehin nicht.
   */
  private var multicast: WifiManager.MulticastLock? = null

  /*
   * DIE SICHERUNGSDATEI (08.10.2026): Wohin sie geht und woher sie kommt,
   * waehlt der Mensch in der Auswahl des Systems -- auch ein USB-Stick steht
   * dort. Rust schreibt und liest nur im Zwischenspeicher (cache/sicherung);
   * hier wird zwischen ihm und dem gewaehlten Ort kopiert. Die Antwort geht
   * als Ereignis `openany-sicherung` an die Seite.
   *
   * Angemeldet beim Bauen der Aktivitaet -- spaeter erlaubt Android es nicht.
   */
  private var abzulegen: File? = null

  private val ablegen = registerForActivityResult(
    ActivityResultContracts.CreateDocument("application/octet-stream")
  ) { ort ->
    val quelle = abzulegen
    abzulegen = null
    if (ort == null || quelle == null) {
      sicherungMelden(JSONObject().put("art", "abgelegt").put("abgebrochen", true))
      return@registerForActivityResult
    }
    Thread {
      val antwort = JSONObject().put("art", "abgelegt")
      try {
        contentResolver.openOutputStream(ort, "w")?.use { aus ->
          quelle.inputStream().use { it.copyTo(aus, 1 shl 16) }
        } ?: throw java.io.IOException("Die Datei ließ sich nicht schreiben.")
        antwort.put("ok", true)
      } catch (e: Exception) {
        antwort.put("fehler", e.message ?: "Unbekannter Fehler")
      } finally {
        quelle.delete()
      }
      sicherungMelden(antwort)
    }.start()
  }

  private val waehlen = registerForActivityResult(ActivityResultContracts.OpenDocument()) { ort ->
    if (ort == null) {
      sicherungMelden(JSONObject().put("art", "gewaehlt").put("abgebrochen", true))
      return@registerForActivityResult
    }
    Thread {
      val antwort = JSONObject().put("art", "gewaehlt")
      try {
        val ordner = File(cacheDir, "sicherung").apply { mkdirs() }
        val ziel = File(ordner, "eingang.age")
        contentResolver.openInputStream(ort)?.use { ein ->
          ziel.outputStream().use { ein.copyTo(it, 1 shl 16) }
        } ?: throw java.io.IOException("Die Datei ließ sich nicht lesen.")
        antwort.put("pfad", ziel.path)
      } catch (e: Exception) {
        antwort.put("fehler", e.message ?: "Unbekannter Fehler")
      }
      sicherungMelden(antwort)
    }.start()
  }

  private fun sicherungMelden(antwort: JSONObject) {
    val webView = offenes?.get() ?: return
    val detail = JSONObject.quote(antwort.toString())
    webView.post {
      webView.evaluateJavascript(
        "window.dispatchEvent(new CustomEvent('openany-sicherung', { detail: JSON.parse($detail) }))",
        null
      )
    }
  }

  override fun onCreate(savedInstanceState: Bundle?) {
    // VOR super.onCreate: Dort startet der Rust-Teil, und der liest als
    // Erstes die Ausweise -- dafuer muss der Tresor schon angemeldet sein.
    Tresor.bereit()
    Standbild.bereit()
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
    val wlan = applicationContext.getSystemService(Context.WIFI_SERVICE) as WifiManager
    multicast = wlan.createMulticastLock("openany-nahbereich").apply { setReferenceCounted(false) }

    // Der Auffrischer (Abgleich im Hintergrund, etwa stuendlich): bei jedem
    // Start noch einmal einplanen, falls der Mensch ihn will. KEEP haelt
    // einen laufenden Plan, siehe Auffrischer.einplanen.
    if (Auffrischer.gewuenscht(this)) Auffrischer.einplanen(applicationContext)

    // Der Wachdienst, falls gewuenscht: Wurde er beendet (Update, ein
    // Hersteller, der aufraeumt), holt ihn das Oeffnen der App zurueck.
    if (Wachdienst.gewuenscht(this)) Wachdienst.starten(applicationContext)
  }

  override fun onResume() {
    super.onResume()
    multicast?.acquire()
  }

  override fun onPause() {
    multicast?.release()
    super.onPause()
  }

  override fun onWebViewCreate(webView: WebView) {
    webView.addJavascriptInterface(raender, "openanyRaender")
    webView.addJavascriptInterface(Oeffner(), "openanyOeffnen")
    webView.addJavascriptInterface(AuffrischerSchalter(), "openanyAuffrischer")
    webView.addJavascriptInterface(WachdienstSchalter(), "openanyWachdienst")
    webView.addJavascriptInterface(Ablage(), "openanyAblage")
    offenes = WeakReference(webView)

    ViewCompat.setOnApplyWindowInsetsListener(webView) { _, insets ->
      val r = insets.getInsets(
        WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout()
      )
      val dichte = resources.displayMetrics.density

      // Die Tastatur zaehlt NICHT zum Rand: Sie ueberdeckt die Seite, statt
      // sie zu verkleinern (edge-to-edge), und die Webansicht meldet sie auch
      // nicht ueber `visualViewport` (15.09.2026, Tablet: 1006 px mit und ohne
      // Tastatur). Der Editor braucht ihre Hoehe, um seine Leiste darueber zu
      // setzen.
      //
      // DIE VOLLE HOEHE, samt Gestenleiste darunter: Die Webansicht reicht bis
      // an den unteren Bildschirmrand. Mit abgezogener Gestenleiste lag die
      // Leiste des Editors um genau diese 24 px hinter der Tastatur.
      raender.tastatur = (insets.getInsets(WindowInsetsCompat.Type.ime()).bottom / dichte).roundToInt()

      raender.oben = (r.top / dichte).roundToInt()
      raender.unten = (r.bottom / dichte).roundToInt()
      raender.links = (r.left / dichte).roundToInt()
      raender.rechts = (r.right / dichte).roundToInt()

      // Drehen, Tastatur, Wechsel der Gestensteuerung: Die Oberflaeche soll
      // neu lesen, statt auf den naechsten Start zu warten.
      webView.post {
        webView.evaluateJavascript("window.dispatchEvent(new Event('openany-raender'))", null)
      }

      insets
    }
  }

  /*
   * EINE DATEI MIT EINEM ANDEREN PROGRAMM OEFFNEN.
   *
   * Die Schale legt sie unter cache/oeffnen/<zufall>/<Name> ab
   * (`datei_oeffnen` in dateibefehle.rs); hier geht sie ueber den
   * FileProvider hinaus -- mit Leserecht nur fuer diese eine Datei.
   *
   * NUR AUS DIESEM ORDNER. Die Seite koennte sonst jede Datei des Programms
   * weitergeben, die Datenbank eingeschlossen.
   */
  inner class Oeffner {
    @JavascriptInterface
    fun oeffnen(pfad: String, mime: String): Boolean {
      val erlaubt = File(cacheDir, "oeffnen").canonicalFile
      val datei = File(pfad).canonicalFile
      if (!datei.path.startsWith(erlaubt.path + File.separator) || !datei.isFile) return false

      val uri = FileProvider.getUriForFile(this@MainActivity, "$packageName.fileprovider", datei)
      val absicht = Intent(Intent.ACTION_VIEW).apply {
        setDataAndType(uri, mime.ifBlank { "application/octet-stream" })
        addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_ACTIVITY_NEW_TASK)
      }
      // Die Auswahl zeigt Android selbst -- auch dann, wenn kein Programm das
      // Format kann ("Keine Apps"). Das ist ehrlicher als eine eigene Meldung.
      runOnUiThread {
        try {
          startActivity(Intent.createChooser(absicht, datei.name).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
        } catch (_: ActivityNotFoundException) {
        }
      }
      return true
    }
  }

  /*
   * DER SCHALTER FUER DEN AUFFRISCHER, fuer die Kachel „Abgleich".
   *
   * Der Plan lebt beim WorkManager und der Wunsch in den Einstellungen des
   * Systems -- nicht in einstellungen.json, weil Rust ihn nie braucht: Ein
   * Lauf, den niemand will, wird gar nicht erst geweckt.
   */
  inner class AuffrischerSchalter {
    @JavascriptInterface fun an(): Boolean = Auffrischer.gewuenscht(this@MainActivity)
    @JavascriptInterface fun setzen(an: Boolean) = Auffrischer.setzen(applicationContext, an)
  }

  /*
   * DER SCHALTER FUER DEN WACHDIENST („Sofort benachrichtigen").
   *
   * Beim Einschalten fragt er einmal nach zwei Dingen, die nur eine
   * Aktivitaet erfragen kann: Benachrichtigungen zeigen zu duerfen (ab
   * Android 13) und von der Akku-Optimierung ausgenommen zu werden. Ohne das
   * Zweite beenden manche Hersteller den Dienst trotz aller Regeln. Beides
   * ist eine Bitte; sagt der Mensch nein, laeuft der Dienst trotzdem -- er
   * zeigt dann eben nichts an oder wird oefter beendet.
   */
  /**
   * Eine Datei in den Ordner „Download" des Geraets legen -- AUSSERHALB von
   * openany und damit ausserhalb des Abgleichs. Gebraucht fuer die
   * Sicherungskopie des geheimen PGP-Schluessels, die nicht in „Dateien"
   * und damit nicht auf den Server gehoert (docs/plan-email-pgp.md).
   *
   * Ueber MediaStore, ohne Speicher-Berechtigung -- das geht ab Android 10.
   * Zurueck kommt "" oder der Grund, warum es nicht ging.
   */
  inner class Ablage {
    @JavascriptInterface
    fun inDownloads(name: String, mime: String, text: String): String =
      schreiben(name, mime, text.toByteArray(Charsets.UTF_8))

    /**
     * Die fertige Sicherung an einen Ort nach Wahl. NUR aus cache/sicherung
     * -- sonst koennte die Seite jede Datei des Programms hinausreichen.
     */
    @JavascriptInterface
    fun sicherungAblegen(pfad: String, name: String): Boolean {
      val erlaubt = File(cacheDir, "sicherung").canonicalFile
      val datei = File(pfad).canonicalFile
      if (!datei.path.startsWith(erlaubt.path + File.separator) || !datei.isFile) return false
      abzulegen = datei
      runOnUiThread { ablegen.launch(name) }
      return true
    }

    @JavascriptInterface
    fun sicherungWaehlen() {
      runOnUiThread { waehlen.launch(arrayOf("*/*")) }
    }

    /**
     * Nach dem Einspielen: ganz neu anfangen. Der Rust-Teil tauscht die
     * Daten beim Start, bevor er die Datenbank oeffnet.
     */
    @JavascriptInterface
    fun neuStarten() {
      val start = packageManager.getLaunchIntentForPackage(packageName) ?: return
      startActivity(Intent.makeRestartActivityTask(start.component))
      Runtime.getRuntime().exit(0)
    }

    /** Dasselbe fuer Binaeres (Anhaenge), als Base64 ueber die Bruecke. */
    @JavascriptInterface
    fun inDownloadsBase64(name: String, mime: String, base64: String): String =
      try {
        schreiben(name, mime, android.util.Base64.decode(base64, android.util.Base64.DEFAULT))
      } catch (e: IllegalArgumentException) {
        "Die Datei kam unlesbar an."
      }

    private fun schreiben(name: String, mime: String, daten: ByteArray): String {
      if (Build.VERSION.SDK_INT < 29) return "Dafür braucht es Android 10 oder neuer."
      return try {
        val r = contentResolver
        val werte = ContentValues().apply {
          put(MediaStore.Downloads.DISPLAY_NAME, name)
          put(MediaStore.Downloads.MIME_TYPE, mime)
          put(MediaStore.Downloads.IS_PENDING, 1)
        }
        val ort = r.insert(MediaStore.Downloads.EXTERNAL_CONTENT_URI, werte)
          ?: return "Die Datei ließ sich nicht anlegen."
        r.openOutputStream(ort)?.use { it.write(daten) }
          ?: return "Die Datei ließ sich nicht schreiben."
        werte.clear()
        werte.put(MediaStore.Downloads.IS_PENDING, 0)
        r.update(ort, werte, null, null)
        ""
      } catch (e: Exception) {
        e.message ?: "Unbekannter Fehler"
      }
    }
  }

  inner class WachdienstSchalter {
    @JavascriptInterface fun an(): Boolean = Wachdienst.gewuenscht(this@MainActivity)

    @JavascriptInterface
    fun setzen(an: Boolean) {
      Wachdienst.setzen(applicationContext, an)
      if (!an) return
      runOnUiThread {
        if (Build.VERSION.SDK_INT >= 33 &&
          checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED
        ) {
          requestPermissions(arrayOf(Manifest.permission.POST_NOTIFICATIONS), 1)
        }
        val strom = getSystemService(Context.POWER_SERVICE) as PowerManager
        if (!strom.isIgnoringBatteryOptimizations(packageName)) {
          try {
            startActivity(
              Intent(Settings.ACTION_REQUEST_IGNORE_BATTERY_OPTIMIZATIONS, Uri.parse("package:$packageName"))
            )
          } catch (_: ActivityNotFoundException) {
          }
        }
      }
    }
  }

  companion object {
    /*
     * DAS OFFENE FENSTER, falls es eines gibt -- damit der Auffrischer ihm
     * sagen kann, dass sich etwas geaendert hat. Schwach gehalten: Ein
     * Worker darf eine geschlossene Aktivitaet nicht am Leben halten.
     */
    @Volatile private var offenes: WeakReference<WebView>? = null

    // @JvmStatic: Auch Rust ruft das (wachdienst.rs), als statische Methode.
    @JvmStatic
    fun aufgefrischt() {
      val webView = offenes?.get() ?: return
      webView.post {
        webView.evaluateJavascript("window.dispatchEvent(new Event('openany-aufgefrischt'))", null)
      }
    }
  }

  /** Nur Zahlen, keine Methode mit Wirkung -- mehr soll die Seite hier nicht erreichen. */
  class Raender {
    @Volatile var oben = 0
    @Volatile var unten = 0
    @Volatile var links = 0
    @Volatile var rechts = 0
    @Volatile var tastatur = 0

    @JavascriptInterface fun oben(): Int = oben
    @JavascriptInterface fun unten(): Int = unten
    @JavascriptInterface fun links(): Int = links
    @JavascriptInterface fun rechts(): Int = rechts
    @JavascriptInterface fun tastatur(): Int = tastatur
  }
}
