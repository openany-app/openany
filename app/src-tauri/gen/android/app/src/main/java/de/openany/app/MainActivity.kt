package de.openany.app

import android.Manifest
import android.content.ActivityNotFoundException
import android.content.ContentValues
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.media.projection.MediaProjectionConfig
import android.media.projection.MediaProjectionManager
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
import androidx.core.view.WindowCompat
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
   * nicht verlaesslich (unten 0, oben laege der Inhalt unter der
   * Statusleiste). Deshalb liest die
   * Oberflaeche die Werte hier ab (in CSS-Pixeln) -- `src/raender.js`.
   */
  private val raender = Raender()

  /*
   * GERAETE IN DER NAEHE BRAUCHEN MULTICAST.
   *
   * Android wirft eingehende Multicast-Pakete weg, solange keine App eine
   * Sperre haelt -- Akku (ein `adb shell` braucht sie nicht, der
   * App-Prozess schon). Gehalten wird sie nur,
   * solange das Programm vorn ist: Im Hintergrund sucht es ohnehin nicht.
   */
  private var multicast: WifiManager.MulticastLock? = null

  /*
   * DIE SICHERUNGSDATEI: Wohin sie geht und woher sie kommt,
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

  /*
   * TONAUFNAHME: erst die Rechte, bei Systemaudio dann die Einwilligung des
   * Systems, dann der Dienst. Die Seite erfaehrt den Ausgang aus
   * `Tonaufnahme.stand`, das sie ohnehin laufend liest.
   */
  private val tonEinwilligung = registerForActivityResult(
    ActivityResultContracts.StartActivityForResult()
  ) { antwort ->
    val daten = antwort.data
    if (antwort.resultCode == RESULT_OK && daten != null) {
      Tonaufnahme.starten(this, Tonaufnahme.ART_SYSTEM, antwort.resultCode, daten)
    } else {
      Tonaufnahme.fehler = Tonaufnahme.FEHLER_ABGEBROCHEN
    }
  }

  private var tonArt = Tonaufnahme.ART_MIKROFON

  private val tonRechte = registerForActivityResult(
    ActivityResultContracts.RequestMultiplePermissions()
  ) { ergebnis ->
    when {
      ergebnis[Manifest.permission.RECORD_AUDIO] != true -> Tonaufnahme.fehler = Tonaufnahme.FEHLER_RECHT
      tonArt == Tonaufnahme.ART_SYSTEM -> {
        val projektion = getSystemService(Context.MEDIA_PROJECTION_SERVICE) as MediaProjectionManager
        // Ab Android 14 bietet das System sonst „Einzelne App" an; aufgenommen
        // wuerde dann nur deren Ton.
        tonEinwilligung.launch(
          if (Build.VERSION.SDK_INT >= 34) {
            projektion.createScreenCaptureIntent(MediaProjectionConfig.createConfigForDefaultDisplay())
          } else {
            projektion.createScreenCaptureIntent()
          }
        )
      }
      else -> Tonaufnahme.starten(this, Tonaufnahme.ART_MIKROFON, 0, null)
    }
  }

  /*
   * FOTOS VOM GERAET: die Rechte fuer die Medienbibliothek. Die Antwort geht
   * als Ereignis `openany-medien` mit dem neuen Stand an die Seite.
   */
  private val medienRechte = registerForActivityResult(
    ActivityResultContracts.RequestMultiplePermissions()
  ) { _ ->
    val webView = offenes?.get() ?: return@registerForActivityResult
    val stand = JSONObject.quote(Medien.stand(this))
    webView.post {
      webView.evaluateJavascript(
        "window.dispatchEvent(new CustomEvent('openany-medien', { detail: $stand }))",
        null
      )
    }
  }

  /*
   * KALENDER IM TELEFON (K1): die Kalenderrechte. Die Antwort geht als
   * Ereignis `openany-systemkalender` (erlaubt ja/nein) an die Seite.
   */
  private val kalenderRechte = registerForActivityResult(
    ActivityResultContracts.RequestMultiplePermissions()
  ) { _ ->
    val webView = offenes?.get() ?: return@registerForActivityResult
    val erlaubt = Systemkalender.erlaubt(this)
    webView.post {
      webView.evaluateJavascript(
        "window.dispatchEvent(new CustomEvent('openany-systemkalender', { detail: $erlaubt }))",
        null
      )
    }
  }

  /*
   * KONTAKTE IM TELEFON (K4): die Kontaktrechte. Die Antwort geht als
   * Ereignis `openany-systemkontakte` (erlaubt ja/nein) an die Seite.
   */
  private val kontaktRechte = registerForActivityResult(
    ActivityResultContracts.RequestMultiplePermissions()
  ) { _ ->
    val webView = offenes?.get() ?: return@registerForActivityResult
    val erlaubt = Systemkontakte.erlaubt(this)
    webView.post {
      webView.evaluateJavascript(
        "window.dispatchEvent(new CustomEvent('openany-systemkontakte', { detail: $erlaubt }))",
        null
      )
    }
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
    Medien.bereit()
    Medien.kontext = applicationContext
    Systemkalender.bereit()
    Systemkalender.kontext = applicationContext
    Systemkontakte.bereit()
    Systemkontakte.kontext = applicationContext
    Uebernahmedienst.kontext = applicationContext
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

    teilenAufnehmen(intent)
  }

  override fun onNewIntent(intent: Intent) {
    super.onNewIntent(intent)
    teilenAufnehmen(intent)
  }

  /*
   * OPENANY ALS TEILEN-ZIEL (docs/plan-viele-dateien.md, Skizze C).
   *
   * Die Freigabe einer geteilten Adresse gilt nur, solange diese Aktivitaet
   * lebt -- deshalb sofort kopieren, nach cache/teilen/<stapel>/<Name>. Dort
   * holt Rust sie ab (`teilen.rs`), sobald der Mensch das Ziel gewaehlt hat;
   * die Seite fragt beim Start und nach `openany-teilen` nach. Pfade gehen
   * dabei nie durch die Webansicht.
   *
   * Jede App kann hierher teilen, und kopiert wird, bevor der Mensch etwas
   * bestaetigt. Darum: nur `content://` einer ANDEREN App (kein `file://`,
   * nichts aus dem eigenen Speicher), und nur, solange auf dem Geraet Platz
   * bleibt -- sonst fuellte eine fremde App den Speicher.
   */
  private fun teilenAufnehmen(absicht: Intent?) {
    val a = absicht ?: return
    val adressen: List<Uri> = when (a.action) {
      Intent.ACTION_SEND -> listOfNotNull(stream(a))
      Intent.ACTION_SEND_MULTIPLE -> streams(a)
      else -> return
    }
    // Nur einmal: Beim Drehen kaeme dieselbe Absicht sonst noch einmal.
    a.action = null
    if (adressen.isEmpty()) return
    Thread {
      val stapel = File(File(cacheDir, "teilen"), System.currentTimeMillis().toString()).apply { mkdirs() }
      val vergeben = mutableSetOf<String>()
      for (uri in adressen) {
        if (!teilbar(uri)) {
          android.util.Log.w("openany", "Teilen: Adresse nicht angenommen")
          continue
        }
        var ziel: File? = null
        try {
          val name = freierName(geteilterName(uri), vergeben)
          val datei = File(stapel, name)
          ziel = datei
          val ganz = contentResolver.openInputStream(uri)?.use { ein ->
            datei.outputStream().use { aus -> kopierenMitPlatz(ein, aus) }
          } ?: false
          if (!ganz) {
            android.util.Log.w("openany", "Teilen: kein Platz mehr, abgebrochen")
            datei.delete()
            break
          }
        } catch (e: Exception) {
          android.util.Log.w("openany", "Teilen: nicht lesbar", e)
          ziel?.delete()
        }
      }
      if (stapel.list().isNullOrEmpty()) {
        stapel.delete()
        return@Thread
      }
      val webView = offenes?.get() ?: return@Thread
      webView.post {
        webView.evaluateJavascript("window.dispatchEvent(new Event('openany-teilen'))", null)
      }
    }.start()
  }

  /** Nur Inhalte einer anderen App: kein Dateipfad, nichts vom eigenen Anbieter. */
  private fun teilbar(uri: Uri): Boolean =
    uri.scheme == "content" && uri.authority?.startsWith(packageName) != true

  /**
   * Kopiert, solange dem Geraet [TEILEN_PLATZ_FREI] bleiben. `false`: Der
   * Platz ging aus, die Datei ist unvollstaendig.
   */
  private fun kopierenMitPlatz(ein: java.io.InputStream, aus: java.io.OutputStream): Boolean {
    val puffer = ByteArray(1 shl 16)
    var seitPruefung = 0L
    while (true) {
      val n = ein.read(puffer)
      if (n < 0) return true
      aus.write(puffer, 0, n)
      seitPruefung += n
      if (seitPruefung >= 8L shl 20) {
        seitPruefung = 0
        if (cacheDir.usableSpace < TEILEN_PLATZ_FREI) return false
      }
    }
  }

  @Suppress("DEPRECATION")
  private fun stream(a: Intent): Uri? =
    if (Build.VERSION.SDK_INT >= 33) a.getParcelableExtra(Intent.EXTRA_STREAM, Uri::class.java)
    else a.getParcelableExtra(Intent.EXTRA_STREAM)

  @Suppress("DEPRECATION")
  private fun streams(a: Intent): List<Uri> =
    (if (Build.VERSION.SDK_INT >= 33) a.getParcelableArrayListExtra(Intent.EXTRA_STREAM, Uri::class.java)
    else a.getParcelableArrayListExtra(Intent.EXTRA_STREAM)) ?: emptyList()

  /** Der Name, den die teilende App nennt -- mit Endung, sonst nach dem Typ. */
  private fun geteilterName(uri: Uri): String {
    var name: String? = null
    try {
      contentResolver.query(uri, arrayOf(android.provider.OpenableColumns.DISPLAY_NAME), null, null, null)?.use { c ->
        if (c.moveToFirst()) name = c.getString(0)
      }
    } catch (_: Exception) {
    }
    var n = (name ?: uri.lastPathSegment ?: "Datei").substringAfterLast('/').replace(Regex("[\\\\/:*?\"<>|]"), "_").ifBlank { "Datei" }
    if (!n.contains('.')) {
      val endung = contentResolver.getType(uri)?.let { android.webkit.MimeTypeMap.getSingleton().getExtensionFromMimeType(it) }
      if (endung != null) n = "$n.$endung"
    }
    return n
  }

  private fun freierName(name: String, vergeben: MutableSet<String>): String {
    var kandidat = name
    var i = 2
    while (!vergeben.add(kandidat.lowercase())) {
      val punkt = name.lastIndexOf('.')
      kandidat = if (punkt > 0) "${name.substring(0, punkt)} ($i)${name.substring(punkt)}" else "$name ($i)"
      i++
    }
    return kandidat
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
    webView.addJavascriptInterface(Leisten(), "openanyLeisten")
    webView.addJavascriptInterface(TonBruecke(), "openanyTon")
    webView.addJavascriptInterface(MedienBruecke(), "openanyMedien")
    webView.addJavascriptInterface(TeilenBruecke(), "openanyTeilen")
    webView.addJavascriptInterface(SystemkalenderBruecke(), "openanySystemkalender")
    webView.addJavascriptInterface(SystemkontakteBruecke(), "openanySystemkontakte")
    offenes = WeakReference(webView)

    // AM FENSTER, NICHT AN DER WEBANSICHT. Bis Android 10 reicht das System
    // die Raender von oben nach unten durch, und eine Ansicht dazwischen kann
    // sie verbrauchen -- dann kam bei der Webansicht nichts an, `oben` blieb 0
    // und die Seite lag unter der Statusleiste (Android 8). Das Fenster sieht
    // sie immer zuerst; was es sonst damit tut, laeuft unten weiter.
    val fenster = window.decorView
    ViewCompat.setOnApplyWindowInsetsListener(fenster) { v, insets ->
      val r = insets.getInsets(
        WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout()
      )
      val dichte = resources.displayMetrics.density

      // Die Tastatur zaehlt NICHT zum Rand: Sie ueberdeckt die Seite, statt
      // sie zu verkleinern (edge-to-edge), und die Webansicht meldet sie auch
      // nicht ueber `visualViewport`. Der Editor braucht ihre Hoehe, um seine Leiste darueber zu
      // setzen.
      //
      // DIE VOLLE HOEHE, samt Gestenleiste darunter: Die Webansicht reicht bis
      // an den unteren Bildschirmrand. Mit abgezogener Gestenleiste laege die
      // Leiste des Editors um genau deren Hoehe hinter der Tastatur.
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

      ViewCompat.onApplyWindowInsets(v, insets)
    }
    // Die ersten Raender kamen womoeglich schon vor dem Anmelden.
    ViewCompat.requestApplyInsets(fenster)
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
  /**
   * Die Symbole in Status- und Gestenleiste passend zum Theme der APP.
   * `enableEdgeToEdge` richtet sie nach dem System: Ein Geraet im Dunkelmodus
   * mit der App auf „hell" zeigte weisse Uhrzeit auf weissem Grund. Die Oberflaeche meldet jeden Wechsel (app/src/raender.js).
   */
  inner class Leisten {
    @JavascriptInterface
    fun dunkel(dunkel: Boolean) {
      runOnUiThread {
        val steuerung = WindowCompat.getInsetsController(window, window.decorView)
        steuerung.isAppearanceLightStatusBars = !dunkel
        steuerung.isAppearanceLightNavigationBars = !dunkel
      }
    }
  }

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

    /**
     * Ein Original in den Download-Ordner, vom Pfad aus gestreamt: Als Base64
     * ueber die Bruecke passte ein grosses Foto nicht in den Java-Speicher.
     * NUR aus der Ablage -- sonst koennte die Seite jede Datei hinausreichen.
     */
    @JavascriptInterface
    fun inDownloadsDatei(pfad: String, name: String, mime: String): String {
      val erlaubt = File(dataDir, "inhalte").canonicalFile
      val datei = File(pfad).canonicalFile
      if (!datei.path.startsWith(erlaubt.path + File.separator) || !datei.isFile) return "Die Datei ist nicht erreichbar."
      return schreiben(name, mime) { aus -> datei.inputStream().use { it.copyTo(aus) } }
    }

    private fun schreiben(name: String, mime: String, daten: ByteArray): String =
      schreiben(name, mime) { aus -> aus.write(daten) }

    private fun schreiben(name: String, mime: String, inhalt: (java.io.OutputStream) -> Unit): String {
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
        r.openOutputStream(ort)?.use(inhalt)
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

  inner class TonBruecke {
    @JavascriptInterface fun kannSystem(): Boolean = Build.VERSION.SDK_INT >= 29
    @JavascriptInterface fun stand(): String = Tonaufnahme.stand(this@MainActivity)
    @JavascriptInterface fun pausieren(an: Boolean) { Tonaufnahme.pausiert = an }
    @JavascriptInterface fun beenden() { Tonaufnahme.beenden(this@MainActivity, false) }
    @JavascriptInterface fun verwerfen() { Tonaufnahme.beenden(this@MainActivity, true) }

    @JavascriptInterface
    fun starten(art: String) {
      Tonaufnahme.fehler = null
      tonArt = if (art == Tonaufnahme.ART_SYSTEM && kannSystem()) Tonaufnahme.ART_SYSTEM else Tonaufnahme.ART_MIKROFON
      val rechte = mutableListOf(Manifest.permission.RECORD_AUDIO)
      if (Build.VERSION.SDK_INT >= 33) rechte += Manifest.permission.POST_NOTIFICATIONS
      runOnUiThread { tonRechte.launch(rechte.toTypedArray()) }
    }
  }

  /*
   * Nur die Rechte. Was es gibt und was uebernommen wird, fragt die Seite
   * bei Rust (medien.rs) -- dort wird auch geoeffnet.
   */
  inner class MedienBruecke {
    @JavascriptInterface fun stand(): String = Medien.stand(this@MainActivity)

    @JavascriptInterface
    fun erlauben() {
      runOnUiThread { medienRechte.launch(Medien.rechte()) }
    }

    /** Ab Android 14, bei „nur ausgewählte": die Auswahl ändern -- dieselbe Frage. */
    @JavascriptInterface
    fun einstellungen() {
      runOnUiThread {
        try {
          startActivity(Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS, Uri.parse("package:$packageName")))
        } catch (_: ActivityNotFoundException) {
        }
      }
    }
  }

  /** Nach dem Teilen: zurueck zu der App, aus der geteilt wurde (Skizze C). */
  inner class TeilenBruecke {
    @JavascriptInterface
    fun fertig() {
      runOnUiThread { moveTaskToBack(true) }
    }
  }

  /** Nur die Rechte; ein- und ausschalten geht ueber Rust (systemkalender.rs). */
  inner class SystemkalenderBruecke {
    @JavascriptInterface fun erlaubt(): Boolean = Systemkalender.erlaubt(this@MainActivity)

    @JavascriptInterface
    fun erlauben() {
      runOnUiThread {
        kalenderRechte.launch(arrayOf(Manifest.permission.READ_CALENDAR, Manifest.permission.WRITE_CALENDAR))
      }
    }
  }

  /** Nur die Rechte; ein- und ausschalten geht ueber Rust (systemkontakte.rs). */
  inner class SystemkontakteBruecke {
    @JavascriptInterface fun erlaubt(): Boolean = Systemkontakte.erlaubt(this@MainActivity)

    @JavascriptInterface
    fun erlauben() {
      runOnUiThread {
        kontaktRechte.launch(arrayOf(Manifest.permission.READ_CONTACTS, Manifest.permission.WRITE_CONTACTS))
      }
    }
  }

  inner class WachdienstSchalter {
    @JavascriptInterface fun an(): Boolean = Wachdienst.gewuenscht(this@MainActivity)

    /** Kurz vibrieren, wenn eine Nachricht ankommt und die App offen ist (App.vue). */
    @JavascriptInterface
    fun vibrieren() {
      val v = if (Build.VERSION.SDK_INT >= 31) {
        (getSystemService(Context.VIBRATOR_MANAGER_SERVICE) as android.os.VibratorManager).defaultVibrator
      } else {
        @Suppress("DEPRECATION")
        getSystemService(Context.VIBRATOR_SERVICE) as android.os.Vibrator
      }
      if (!v.hasVibrator()) return
      if (Build.VERSION.SDK_INT >= 26) {
        v.vibrate(android.os.VibrationEffect.createOneShot(180, android.os.VibrationEffect.DEFAULT_AMPLITUDE))
      } else {
        @Suppress("DEPRECATION")
        v.vibrate(180)
      }
    }

    /** Ton und Vibration bei geschlossener App bestimmt Android: dorthin. */
    @JavascriptInterface
    fun systemEinstellungen() {
      runOnUiThread {
        try {
          val absicht = if (Build.VERSION.SDK_INT >= 26) {
            Intent(Settings.ACTION_APP_NOTIFICATION_SETTINGS).putExtra(Settings.EXTRA_APP_PACKAGE, packageName)
          } else {
            Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS, Uri.parse("package:$packageName"))
          }
          startActivity(absicht)
        } catch (_: ActivityNotFoundException) {
        }
      }
    }

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
    /** So viel bleibt dem Geraet frei, wenn eine andere App hierher teilt. */
    private const val TEILEN_PLATZ_FREI = 512L shl 20

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
