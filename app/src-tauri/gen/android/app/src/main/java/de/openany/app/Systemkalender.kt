package de.openany.app

import android.Manifest
import android.accounts.AbstractAccountAuthenticator
import android.accounts.Account
import android.accounts.AccountAuthenticatorResponse
import android.accounts.AccountManager
import android.app.Service
import android.content.AbstractThreadedSyncAdapter
import android.content.ContentProviderClient
import android.content.ContentProviderOperation
import android.content.ContentResolver
import android.content.ContentValues
import android.content.Context
import android.content.Intent
import android.content.SyncResult
import android.content.pm.PackageManager
import android.graphics.Color
import android.net.Uri
import android.os.Bundle
import android.os.IBinder
import android.provider.CalendarContract
import android.provider.CalendarContract.Calendars
import android.provider.CalendarContract.Events
import android.provider.CalendarContract.Reminders
import android.util.Log
import org.json.JSONArray
import org.json.JSONObject
import java.text.SimpleDateFormat
import java.util.Locale
import java.util.TimeZone

/**
 * Kalender im Kalender des Telefons -- wie DAVx5, nur Ende-zu-Ende
 * (docs/plan-kalender-kontakte-im-system.md, K1).
 *
 * Rust stellt zusammen, was im System stehen soll (`systemkalender.rs`), und
 * reicht es als JSON hierher; hier wird unter dem Konto „openany“
 * eingetragen, als „SyncAdapter“ -- nur so darf die App Kalender besitzen.
 * Je Termin merkt sich `SYNC_DATA1` den Abdruck: Gleiches wird nicht neu
 * geschrieben. Zeilen, die jemand in einer anderen Kalender-App geaendert
 * hat (`DIRTY`), bleiben stehen -- die holt K2 zurueck.
 *
 * Wanduhrzeit (openany kennt keine Zone) wird in der Zone des Geraets
 * gelesen; ganztaegige Termine liegen, wie Android es verlangt, in UTC.
 */
object Systemkalender {
  private const val TAG = "openany"
  const val KONTOTYP = "de.openany.app"

  @Volatile var kontext: Context? = null

  init {
    System.loadLibrary("openany_app_lib")
    anmelden()
  }

  @JvmStatic private external fun anmelden()

  /** `systemkalender.rs`: eintragen ohne Fenster (SyncAdapter). */
  @JvmStatic external fun laufen(daten: String, zwischen: String): String

  fun bereit() {}

  fun erlaubt(k: Context): Boolean =
    k.checkSelfPermission(Manifest.permission.READ_CALENDAR) == PackageManager.PERMISSION_GRANTED &&
      k.checkSelfPermission(Manifest.permission.WRITE_CALENDAR) == PackageManager.PERMISSION_GRANTED

  private fun konto(k: Context): Account {
    val name = k.getString(R.string.app_name)
    val konto = Account(name, KONTOTYP)
    val am = AccountManager.get(k)
    if (am.getAccountsByType(KONTOTYP).none { it.name == name }) {
      am.addAccountExplicitly(konto, null, null)
    }
    // Seit Android 8 sieht eine andere App ein Konto nur, wenn es fuer sie
    // sichtbar ist -- sonst zeigt etwa Google Kalender die Kalender nicht.
    if (android.os.Build.VERSION.SDK_INT >= 26) {
      am.setAccountVisibility(konto, AccountManager.PACKAGE_NAME_KEY_LEGACY_VISIBLE, AccountManager.VISIBILITY_VISIBLE)
    }
    ContentResolver.setIsSyncable(konto, CalendarContract.AUTHORITY, 1)
    ContentResolver.setSyncAutomatically(konto, CalendarContract.AUTHORITY, true)
    return konto
  }

  private fun alsSync(uri: Uri, konto: Account): Uri = uri.buildUpon()
    .appendQueryParameter(CalendarContract.CALLER_IS_SYNCADAPTER, "true")
    .appendQueryParameter(Calendars.ACCOUNT_NAME, konto.name)
    .appendQueryParameter(Calendars.ACCOUNT_TYPE, konto.type)
    .build()

  /** Nur die Kalender des Kontos fort -- das Konto bleibt fuer die Kontakte (K4). */
  @JvmStatic
  fun leeren() {
    val k = kontext ?: return
    if (!erlaubt(k)) return
    val konto = Account(k.getString(R.string.app_name), KONTOTYP)
    try {
      k.contentResolver.delete(
        alsSync(Calendars.CONTENT_URI, konto),
        "${Calendars.ACCOUNT_NAME}=? AND ${Calendars.ACCOUNT_TYPE}=?", arrayOf(konto.name, konto.type),
      )
      ContentResolver.setIsSyncable(konto, CalendarContract.AUTHORITY, 0)
    } catch (e: Exception) {
      Log.w(TAG, "Systemkalender: Leeren gescheitert", e)
    }
  }

  /** Aus: das Konto fort -- Android nimmt Kalender, Termine und Kontakte mit. */
  @JvmStatic
  fun abmelden() {
    val k = kontext ?: return
    val am = AccountManager.get(k)
    for (a in am.getAccountsByType(KONTOTYP)) am.removeAccountExplicitly(a)
  }

  /**
   * Den Stand eintragen. `""` = gelungen, sonst der Grund.
   *
   * NACHEINANDER (`@Synchronized`): Einschalten, Takt und SyncAdapter
   * koennen gleichzeitig kommen; zwei Laeufe, die beide noch nichts
   * vorfinden, legten jeden Kalender doppelt an.
   */
  @JvmStatic
  @Synchronized
  fun spiegeln(json: String): String {
    val k = kontext ?: return "No context."
    if (!erlaubt(k)) return "Calendar permission missing."
    return try {
      val konto = konto(k)
      val r = k.contentResolver
      val vorhanden = mutableMapOf<String, Long>()
      r.query(
        alsSync(Calendars.CONTENT_URI, konto), arrayOf(Calendars._ID, Calendars._SYNC_ID),
        "${Calendars.ACCOUNT_NAME}=? AND ${Calendars.ACCOUNT_TYPE}=?", arrayOf(konto.name, konto.type), null,
      )?.use { c ->
        while (c.moveToNext()) {
          val sid = c.getString(1) ?: continue
          // Doppelt (aus einem frueheren Wettlauf): nur einer bleibt.
          if (sid in vorhanden) {
            r.delete(alsSync(Calendars.CONTENT_URI, konto), "${Calendars._ID}=?", arrayOf(c.getLong(0).toString()))
          } else {
            vorhanden[sid] = c.getLong(0)
          }
        }
      }

      val kalender = JSONArray(json)
      val gesehen = mutableSetOf<String>()
      for (i in 0 until kalender.length()) {
        val kal = kalender.getJSONObject(i)
        val schluessel = kal.getString("schluessel")
        gesehen += schluessel
        val werte = ContentValues().apply {
          put(Calendars.ACCOUNT_NAME, konto.name)
          put(Calendars.ACCOUNT_TYPE, konto.type)
          put(Calendars._SYNC_ID, schluessel)
          put(Calendars.NAME, kal.getString("name"))
          put(Calendars.CALENDAR_DISPLAY_NAME, kal.getString("name"))
          put(Calendars.CALENDAR_COLOR, farbe(kal.optString("farbe")))
          put(
            Calendars.CALENDAR_ACCESS_LEVEL,
            if (kal.getBoolean("schreibbar")) Calendars.CAL_ACCESS_OWNER else Calendars.CAL_ACCESS_READ,
          )
          put(Calendars.OWNER_ACCOUNT, konto.name)
          put(Calendars.VISIBLE, 1)
          put(Calendars.SYNC_EVENTS, 1)
          put(Calendars.CALENDAR_TIME_ZONE, TimeZone.getDefault().id)
        }
        val id = vorhanden[schluessel]?.also { id ->
          r.update(alsSync(Calendars.CONTENT_URI, konto), werte, "${Calendars._ID}=?", arrayOf(id.toString()))
        } ?: r.insert(alsSync(Calendars.CONTENT_URI, konto), werte)?.lastPathSegment?.toLong() ?: continue
        termineAbgleichen(k, konto, id, kal.getJSONArray("termine"))
      }
      // Kalender, die es in openany nicht mehr gibt (oder nicht mehr im System
      // geben soll), samt ihren Terminen fort.
      for ((schluessel, id) in vorhanden) {
        if (schluessel !in gesehen) {
          r.delete(alsSync(Calendars.CONTENT_URI, konto), "${Calendars._ID}=?", arrayOf(id.toString()))
        }
      }
      ""
    } catch (e: Exception) {
      Log.w(TAG, "Systemkalender: Eintragen gescheitert", e)
      e.message ?: "Unknown error"
    }
  }

  /*
   * K2 -- ZURUECK: Was eine andere Kalender-App an openany-Terminen geaendert,
   * angelegt oder geloescht hat (`DIRTY`, `DELETED`). Zeiten kommen in
   * openanys Form zurueck: Wanduhrzeit in der Zone des Geraets, ganztaegig das
   * Datum mit dem letzten Tag einschliesslich. Was openany nicht kennt
   * (Teilnehmer …), geht dabei verloren.
   */
  @JvmStatic
  @Synchronized
  fun aenderungen(): String {
    val k = kontext ?: return "[]"
    if (!erlaubt(k)) return "[]"
    val aus = JSONArray()
    try {
      val konto = Account(k.getString(R.string.app_name), KONTOTYP)
      val kalender = mutableMapOf<Long, String>()
      k.contentResolver.query(
        alsSync(Calendars.CONTENT_URI, konto), arrayOf(Calendars._ID, Calendars._SYNC_ID),
        "${Calendars.ACCOUNT_NAME}=? AND ${Calendars.ACCOUNT_TYPE}=?", arrayOf(konto.name, konto.type), null,
      )?.use { c -> while (c.moveToNext()) c.getString(1)?.let { kalender[c.getLong(0)] = it } }
      if (kalender.isEmpty()) return "[]"
      val spalten = arrayOf(
        Events._ID, Events._SYNC_ID, Events.CALENDAR_ID, Events.DELETED, Events.TITLE, Events.DESCRIPTION,
        Events.EVENT_LOCATION, Events.DTSTART, Events.DTEND, Events.DURATION, Events.ALL_DAY, Events.RRULE, Events.EXDATE,
        Events.ORIGINAL_SYNC_ID, Events.ORIGINAL_INSTANCE_TIME, Events.ORIGINAL_ALL_DAY, Events.STATUS,
      )
      k.contentResolver.query(
        alsSync(Events.CONTENT_URI, konto), spalten,
        "${Events.CALENDAR_ID} IN (${kalender.keys.joinToString(",")}) AND (${Events.DIRTY}=1 OR ${Events.DELETED}=1)",
        null, null,
      )?.use { c ->
        while (c.moveToNext()) {
          val ganztags = c.getInt(10) == 1
          val zone = if (ganztags) TimeZone.getTimeZone("UTC") else TimeZone.getDefault()
          val beginn = c.getLong(7)
          val ende = if (!c.isNull(8)) c.getLong(8) else beginn + dauer(c.getString(9), ganztags)
          val (frequenz, bis) = wiederholung(c.getString(11))
          val tag = 24L * 60 * 60 * 1000
          aus.put(
            JSONObject()
              .put("id", c.getLong(0))
              .put("schluessel", c.getString(1) ?: JSONObject.NULL)
              .put("kalender", kalender[c.getLong(2)])
              .put("geloescht", c.getInt(3) == 1)
              .put("titel", c.getString(4) ?: "")
              .put("beschreibung", c.getString(5) ?: "")
              .put("ort", c.getString(6) ?: "")
              .put("beginn", wand(beginn, zone))
              // Ganztaegig: Android endet am Tag danach um 0 Uhr, openany am letzten Tag.
              .put("ende", wand(if (ganztags) maxOf(beginn, ende - tag) else ende, zone))
              .put("ganztags", ganztags)
              .put("frequenz", frequenz)
              .put("bis", bis)
              .put("exdate", c.getString(12) ?: "")
              .put("erinnerung", erinnerungLesen(k, konto, c.getLong(0)))
              .put("rrule", c.getString(11) ?: "")
              // Eine Ausnahme einer Serie: welches Vorkommen sie ersetzt.
              .put("serie", c.getString(13) ?: "")
              .put("vorkommen", if (c.isNull(14)) "" else vorkommen(c.getLong(14), c.getInt(15) == 1))
              .put("abgesagt", !c.isNull(16) && c.getInt(16) == Events.STATUS_CANCELED)
          )
        }
      }
    } catch (e: Exception) {
      Log.w(TAG, "Systemkalender: Aenderungen nicht lesbar", e)
    }
    return aus.toString()
  }

  /**
   * Was Rust uebernommen hat: `[{ id, schluessel, loeschen }]`. Geloeschte
   * Zeilen gehen fort; die anderen bekommen ihre uuid und gelten als sauber.
   * Ohne Abdruck schreibt der naechste Lauf sie in openanys Form neu.
   */
  @JvmStatic
  @Synchronized
  fun bestaetigen(json: String) {
    val k = kontext ?: return
    val konto = Account(k.getString(R.string.app_name), KONTOTYP)
    val ops = ArrayList<ContentProviderOperation>()
    val liste = JSONArray(json)
    for (i in 0 until liste.length()) {
      val e = liste.getJSONObject(i)
      val auswahl = "${Events._ID}=?"
      val id = arrayOf(e.getLong("id").toString())
      ops += if (e.getBoolean("loeschen")) {
        ContentProviderOperation.newDelete(alsSync(Events.CONTENT_URI, konto)).withSelection(auswahl, id).build()
      } else {
        val werte = ContentValues().apply {
          if (!e.isNull("schluessel")) put(Events._SYNC_ID, e.getString("schluessel"))
          put(Events.DIRTY, 0)
          put(Events.DELETED, 0)
          putNull(Events.SYNC_DATA1)
        }
        ContentProviderOperation.newUpdate(alsSync(Events.CONTENT_URI, konto)).withSelection(auswahl, id).withValues(werte).build()
      }
    }
    try {
      for (stueck in ops.chunked(200)) k.contentResolver.applyBatch(CalendarContract.AUTHORITY, ArrayList(stueck))
    } catch (e: Exception) {
      Log.w(TAG, "Systemkalender: Bestaetigen gescheitert", e)
    }
  }

  /** Der ursprüngliche Beginn eines Vorkommens wie in openanys `exdates`. */
  private fun vorkommen(ms: Long, ganztags: Boolean): String =
    SimpleDateFormat("yyyyMMdd'T'HHmmss", Locale.ROOT)
      .apply { timeZone = if (ganztags) TimeZone.getTimeZone("UTC") else TimeZone.getDefault() }
      .format(java.util.Date(ms))

  private fun wand(ms: Long, zone: TimeZone): String =
    SimpleDateFormat("yyyy-MM-dd'T'HH:mm:ss", Locale.ROOT).apply { timeZone = zone }.format(java.util.Date(ms))

  /** RFC 5545: `P1D`, `PT1H30M`, `P3600S`, `P1W` … in Millisekunden. */
  private fun dauer(text: String?, ganztags: Boolean): Long {
    val stunde = 60L * 60 * 1000
    val fallback = if (ganztags) 24 * stunde else stunde
    val t = text?.trim()?.removePrefix("+") ?: return fallback
    val m = Regex("""P(?:(\d+)W)?(?:(\d+)D)?(?:T(?:(\d+)H)?(?:(\d+)M)?(?:(\d+)S)?)?""").matchEntire(t) ?: return fallback
    val (w, d, h, mi, s) = m.destructured
    val ms = ((w.toLongOrNull() ?: 0) * 7 * 24 * stunde) + ((d.toLongOrNull() ?: 0) * 24 * stunde) +
      ((h.toLongOrNull() ?: 0) * stunde) + ((mi.toLongOrNull() ?: 0) * 60_000) + ((s.toLongOrNull() ?: 0) * 1000)
    return if (ms > 0) ms else fallback
  }

  /** `FREQ=WEEKLY;UNTIL=20270331T235959Z` → (`WEEKLY`, `2027-03-31`). Anderes wird `NONE`. */
  private fun wiederholung(rrule: String?): Pair<String, String> {
    if (rrule.isNullOrBlank()) return "NONE" to ""
    val teile = rrule.removePrefix("RRULE:").split(';').mapNotNull { it.split('=', limit = 2).takeIf { p -> p.size == 2 } }
      .associate { it[0].uppercase() to it[1] }
    val f = teile["FREQ"]?.uppercase()
    if (f !in setOf("DAILY", "WEEKLY", "MONTHLY", "YEARLY")) return "NONE" to ""
    val u = teile["UNTIL"]?.take(8)?.takeIf { it.length == 8 && it.all(Char::isDigit) }
    return f!! to (u?.let { "${it.substring(0, 4)}-${it.substring(4, 6)}-${it.substring(6, 8)}" } ?: "")
  }

  private fun termineAbgleichen(k: Context, konto: Account, kalender: Long, termine: JSONArray) {
    val r = k.contentResolver
    class Zeile(val id: Long, val schmutzig: Boolean, val abdruck: String?)
    val vorhanden = mutableMapOf<String, Zeile>()
    r.query(
      alsSync(Events.CONTENT_URI, konto), arrayOf(Events._ID, Events._SYNC_ID, Events.DIRTY, Events.DELETED, Events.SYNC_DATA1),
      "${Events.CALENDAR_ID}=?", arrayOf(kalender.toString()), null,
    )?.use { c ->
      while (c.moveToNext()) {
        val sid = c.getString(1) ?: continue
        vorhanden[sid] = Zeile(c.getLong(0), c.getInt(2) == 1 || c.getInt(3) == 1, c.getString(4))
      }
    }
    val ops = ArrayList<ContentProviderOperation>()
    val gesehen = mutableSetOf<String>()
    // Geschriebene Termine und ihre Erinnerung (K3) -- die Erinnerungen
    // folgen in einem zweiten Durchgang, wenn jeder Termin seine Zeile hat.
    val erinnerungen = mutableMapOf<String, Int?>()
    for (i in 0 until termine.length()) {
      val t = termine.getJSONObject(i)
      val schluessel = t.getString("schluessel")
      gesehen += schluessel
      val abdruck = t.toString().hashCode().toString()
      val alt = vorhanden[schluessel]
      // Anderswo geaendert: steht, bis K2 es zurueckholt.
      if (alt != null && (alt.schmutzig || alt.abdruck == abdruck)) continue
      val werte = terminWerte(t, kalender, abdruck) ?: continue
      erinnerungen[schluessel] = t.optString("erinnerung").toIntOrNull()
      ops += if (alt == null) {
        ContentProviderOperation.newInsert(alsSync(Events.CONTENT_URI, konto)).withValues(werte).build()
      } else {
        ContentProviderOperation.newUpdate(alsSync(Events.CONTENT_URI, konto))
          .withSelection("${Events._ID}=?", arrayOf(alt.id.toString())).withValues(werte).build()
      }
    }
    for ((schluessel, z) in vorhanden) {
      if (schluessel !in gesehen && !z.schmutzig) {
        ops += ContentProviderOperation.newDelete(alsSync(Events.CONTENT_URI, konto))
          .withSelection("${Events._ID}=?", arrayOf(z.id.toString())).build()
      }
    }
    // In Stuecken: Ein Stapel mit tausend Zeilen faellt durch Binder-Grenzen.
    for (stueck in ops.chunked(200)) r.applyBatch(CalendarContract.AUTHORITY, ArrayList(stueck))
    erinnerungenSchreiben(k, konto, kalender, erinnerungen)
  }

  /** Je geschriebenem Termin: alte Erinnerungen fort, die eine aus openany hinein. */
  private fun erinnerungenSchreiben(k: Context, konto: Account, kalender: Long, soll: Map<String, Int?>) {
    if (soll.isEmpty()) return
    val r = k.contentResolver
    val ids = mutableMapOf<String, Long>()
    r.query(
      alsSync(Events.CONTENT_URI, konto), arrayOf(Events._ID, Events._SYNC_ID),
      "${Events.CALENDAR_ID}=?", arrayOf(kalender.toString()), null,
    )?.use { c -> while (c.moveToNext()) c.getString(1)?.let { ids[it] = c.getLong(0) } }
    val ops = ArrayList<ContentProviderOperation>()
    for ((schluessel, minuten) in soll) {
      val id = ids[schluessel] ?: continue
      ops += ContentProviderOperation.newDelete(alsSync(Reminders.CONTENT_URI, konto))
        .withSelection("${Reminders.EVENT_ID}=?", arrayOf(id.toString())).build()
      if (minuten != null) {
        ops += ContentProviderOperation.newInsert(alsSync(Reminders.CONTENT_URI, konto))
          .withValue(Reminders.EVENT_ID, id)
          .withValue(Reminders.MINUTES, minuten)
          .withValue(Reminders.METHOD, Reminders.METHOD_ALERT)
          .build()
      }
    }
    for (stueck in ops.chunked(200)) r.applyBatch(CalendarContract.AUTHORITY, ArrayList(stueck))
  }

  /** Die erste Erinnerung eines Termins im Telefon, in Minuten; leer = keine. */
  private fun erinnerungLesen(k: Context, konto: Account, termin: Long): String {
    k.contentResolver.query(
      alsSync(Reminders.CONTENT_URI, konto), arrayOf(Reminders.MINUTES),
      "${Reminders.EVENT_ID}=?", arrayOf(termin.toString()), "${Reminders.MINUTES} ASC",
    )?.use { c -> if (c.moveToFirst()) return maxOf(0, c.getInt(0)).toString() }
    return ""
  }

  private fun farbe(text: String): Int = try {
    Color.parseColor(if (text.startsWith("#")) text else "#$text")
  } catch (_: Exception) {
    Color.parseColor("#7C5CB0")
  }

  private fun lesen(text: String, zone: TimeZone): Long? {
    if (text.isBlank()) return null
    val f = SimpleDateFormat("yyyy-MM-dd'T'HH:mm:ss", Locale.ROOT).apply { timeZone = zone; isLenient = false }
    return try {
      f.parse(if (text.length == 10) "${text}T00:00:00" else text.take(19))?.time
    } catch (_: Exception) {
      null
    }
  }

  private fun terminWerte(t: JSONObject, kalender: Long, abdruck: String): ContentValues? {
    val ganztags = t.getBoolean("ganztags")
    val zone = if (ganztags) TimeZone.getTimeZone("UTC") else TimeZone.getDefault()
    val tag = 24L * 60 * 60 * 1000
    val beginnText = t.getString("beginn")
    val beginn = lesen(if (ganztags) beginnText.take(10) else beginnText, zone) ?: return null
    val endeText = t.getString("ende")
    var ende = if (ganztags) {
      // Ganztaegig endet ein Termin bei Android am Tag DANACH um 0 Uhr.
      val e = lesen(endeText.take(10), zone) ?: beginn
      if (endeText.length > 10 && endeText.substring(11) != "00:00:00" || e <= beginn) e + tag else e
    } else {
      lesen(endeText, zone) ?: (beginn + 60 * 60 * 1000)
    }
    if (ende <= beginn) ende = beginn + if (ganztags) tag else 60 * 60 * 1000
    val rrule = t.optString("rrule")
    return ContentValues().apply {
      put(Events.CALENDAR_ID, kalender)
      put(Events._SYNC_ID, t.getString("schluessel"))
      put(Events.SYNC_DATA1, abdruck)
      put(Events.TITLE, t.optString("titel"))
      put(Events.DESCRIPTION, t.optString("beschreibung"))
      put(Events.EVENT_LOCATION, t.optString("ort"))
      put(Events.ALL_DAY, if (ganztags) 1 else 0)
      put(Events.HAS_ALARM, if (t.optString("erinnerung").toIntOrNull() != null) 1 else 0)
      put(Events.DTSTART, beginn)
      put(Events.EVENT_TIMEZONE, zone.id)
      if (rrule.isBlank()) {
        put(Events.DTEND, ende)
        putNull(Events.DURATION)
        putNull(Events.RRULE)
      } else {
        // Eine Serie hat bei Android eine Dauer statt eines Endes.
        putNull(Events.DTEND)
        put(Events.DURATION, if (ganztags) "P${(ende - beginn) / tag}D" else "P${(ende - beginn) / 1000}S")
        put(Events.RRULE, rrule)
      }
      val exdate = t.optString("exdate")
      if (exdate.isBlank()) putNull(Events.EXDATE) else put(Events.EXDATE, exdate)
      put(Events.DIRTY, 0)
    }
  }
}

/** Das Konto braucht einen Authenticator -- anmelden muss sich niemand. */
class KontoDienst : Service() {
  private val waechter by lazy {
    object : AbstractAccountAuthenticator(this) {
      override fun editProperties(r: AccountAuthenticatorResponse?, t: String?): Bundle? = null
      override fun addAccount(r: AccountAuthenticatorResponse?, t: String?, a: String?, f: Array<out String>?, o: Bundle?): Bundle? = null
      override fun confirmCredentials(r: AccountAuthenticatorResponse?, a: Account?, o: Bundle?): Bundle? = null
      override fun getAuthToken(r: AccountAuthenticatorResponse?, a: Account?, t: String?, o: Bundle?): Bundle? = null
      override fun getAuthTokenLabel(t: String?): String? = null
      override fun updateCredentials(r: AccountAuthenticatorResponse?, a: Account?, t: String?, o: Bundle?): Bundle? = null
      override fun hasFeatures(r: AccountAuthenticatorResponse?, a: Account?, f: Array<out String>?): Bundle =
        Bundle().apply { putBoolean(AccountManager.KEY_BOOLEAN_RESULT, false) }
    }
  }

  override fun onBind(intent: Intent?): IBinder? = waechter.iBinder
}

/** Android stoesst den Abgleich an (etwa „Jetzt synchronisieren“ in den Konten). */
class SystemkalenderSync : Service() {
  private val adapter by lazy {
    object : AbstractThreadedSyncAdapter(applicationContext, true) {
      override fun onPerformSync(a: Account?, e: Bundle?, au: String?, p: ContentProviderClient?, s: SyncResult?) {
        try {
          Tresor.bereit()
          Systemkalender.bereit()
          Systemkalender.kontext = context
          val ergebnis = Systemkalender.laufen(context.dataDir.absolutePath, context.cacheDir.absolutePath)
          if (ergebnis.isNotEmpty()) Log.w("openany", "Systemkalender: $ergebnis")
        } catch (ex: Throwable) {
          Log.w("openany", "Systemkalender: Abgleich gescheitert", ex)
        }
      }
    }
  }

  override fun onBind(intent: Intent?): IBinder? = adapter.syncAdapterBinder
}
