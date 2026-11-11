package de.openany.app

import android.Manifest
import android.accounts.Account
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
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.net.Uri
import android.os.Bundle
import android.os.IBinder
import android.provider.ContactsContract
import android.provider.ContactsContract.CommonDataKinds.Email
import android.provider.ContactsContract.CommonDataKinds.Event
import android.provider.ContactsContract.CommonDataKinds.Nickname
import android.provider.ContactsContract.CommonDataKinds.Organization
import android.provider.ContactsContract.CommonDataKinds.Phone
import android.provider.ContactsContract.CommonDataKinds.Photo
import android.provider.ContactsContract.CommonDataKinds.StructuredName
import android.provider.ContactsContract.CommonDataKinds.StructuredPostal
import android.provider.ContactsContract.CommonDataKinds.Website
import android.provider.ContactsContract.Data
import android.provider.ContactsContract.RawContacts
import android.util.Log
import org.json.JSONArray
import org.json.JSONObject
import java.io.ByteArrayOutputStream
import java.io.File

/**
 * Kontakte im Adressbuch des Telefons -- wie DAVx5, nur Ende-zu-Ende
 * (docs/plan-kalender-kontakte-im-system.md, K4).
 *
 * Rust stellt zusammen, was im Adressbuch stehen soll (`systemkontakte.rs`);
 * hier wird es unter dem Konto „openany“ eingetragen -- je Kontakt ein
 * `RawContact` mit `SOURCE_ID` = uuid und `SYNC1` = Abdruck: Gleiches wird
 * nicht neu geschrieben. Was jemand im Telefon geaendert hat (`DIRTY`),
 * bleibt stehen, bis Rust es zurueckgeholt hat.
 */
object Systemkontakte {
  private const val TAG = "openany"

  @Volatile var kontext: Context? = null

  init {
    System.loadLibrary("openany_app_lib")
    anmelden()
  }

  @JvmStatic private external fun anmelden()

  /** `systemkontakte.rs`: eintragen ohne Fenster (SyncAdapter). */
  @JvmStatic external fun laufen(daten: String, zwischen: String): String

  fun bereit() {}

  fun erlaubt(k: Context): Boolean =
    k.checkSelfPermission(Manifest.permission.READ_CONTACTS) == PackageManager.PERMISSION_GRANTED &&
      k.checkSelfPermission(Manifest.permission.WRITE_CONTACTS) == PackageManager.PERMISSION_GRANTED

  private fun konto(k: Context): Account {
    val name = k.getString(R.string.app_name)
    val konto = Account(name, Systemkalender.KONTOTYP)
    val am = AccountManager.get(k)
    if (am.getAccountsByType(Systemkalender.KONTOTYP).none { it.name == name }) {
      am.addAccountExplicitly(konto, null, null)
    }
    if (android.os.Build.VERSION.SDK_INT >= 26) {
      am.setAccountVisibility(konto, AccountManager.PACKAGE_NAME_KEY_LEGACY_VISIBLE, AccountManager.VISIBILITY_VISIBLE)
    }
    ContentResolver.setIsSyncable(konto, ContactsContract.AUTHORITY, 1)
    ContentResolver.setSyncAutomatically(konto, ContactsContract.AUTHORITY, true)
    // Ohne Gruppe sichtbar -- sonst blendet das Adressbuch die Kontakte aus.
    val werte = ContentValues().apply {
      put(ContactsContract.Settings.ACCOUNT_NAME, konto.name)
      put(ContactsContract.Settings.ACCOUNT_TYPE, konto.type)
      put(ContactsContract.Settings.UNGROUPED_VISIBLE, 1)
      put(ContactsContract.Settings.SHOULD_SYNC, 1)
    }
    k.contentResolver.insert(alsSync(ContactsContract.Settings.CONTENT_URI, konto), werte)
    return konto
  }

  private fun alsSync(uri: Uri, konto: Account): Uri = uri.buildUpon()
    .appendQueryParameter(ContactsContract.CALLER_IS_SYNCADAPTER, "true")
    .appendQueryParameter(RawContacts.ACCOUNT_NAME, konto.name)
    .appendQueryParameter(RawContacts.ACCOUNT_TYPE, konto.type)
    .build()

  private fun kontoName(k: Context) = Account(k.getString(R.string.app_name), Systemkalender.KONTOTYP)

  /** Nur die Kontakte des Kontos fort (das Konto bleibt fuer den Kalender). */
  @JvmStatic
  fun leeren() {
    val k = kontext ?: return
    if (!erlaubt(k)) return
    val konto = kontoName(k)
    try {
      k.contentResolver.delete(
        alsSync(RawContacts.CONTENT_URI, konto),
        "${RawContacts.ACCOUNT_NAME}=? AND ${RawContacts.ACCOUNT_TYPE}=?", arrayOf(konto.name, konto.type),
      )
      ContentResolver.setIsSyncable(konto, ContactsContract.AUTHORITY, 0)
    } catch (e: Exception) {
      Log.w(TAG, "Systemkontakte: Leeren gescheitert", e)
    }
  }

  /** Den Stand eintragen. `""` = gelungen, sonst der Grund. */
  @JvmStatic
  @Synchronized
  fun spiegeln(json: String): String {
    val k = kontext ?: return "No context."
    if (!erlaubt(k)) return "Contacts permission missing."
    return try {
      val konto = konto(k)
      val r = k.contentResolver
      class Zeile(val id: Long, val abdruck: String?, val schmutzig: Boolean)
      val vorhanden = mutableMapOf<String, Zeile>()
      r.query(
        alsSync(RawContacts.CONTENT_URI, konto),
        arrayOf(RawContacts._ID, RawContacts.SOURCE_ID, RawContacts.SYNC1, RawContacts.DIRTY),
        "${RawContacts.ACCOUNT_NAME}=? AND ${RawContacts.ACCOUNT_TYPE}=? AND ${RawContacts.DELETED}=0",
        arrayOf(konto.name, konto.type), null,
      )?.use { c ->
        while (c.moveToNext()) {
          val sid = c.getString(1)
          // Ohne Schluessel: neu im Telefon, das holt `aenderungen` -- oder
          // ein Rest von frueher: der naechste Lauf nach dem Zurueckholen
          // hat ihn mit Schluessel.
          if (sid == null) continue
          if (sid in vorhanden) {
            r.delete(alsSync(RawContacts.CONTENT_URI, konto), "${RawContacts._ID}=?", arrayOf(c.getLong(0).toString()))
          } else {
            vorhanden[sid] = Zeile(c.getLong(0), c.getString(2), c.getInt(3) == 1)
          }
        }
      }

      val kontakte = JSONArray(json)
      val gesehen = mutableSetOf<String>()
      for (i in 0 until kontakte.length()) {
        val kt = kontakte.getJSONObject(i)
        val schluessel = kt.getString("schluessel")
        gesehen += schluessel
        val alt = vorhanden[schluessel]
        if (alt != null && (alt.schmutzig || alt.abdruck == kt.getString("abdruck"))) continue
        val ops = ArrayList<ContentProviderOperation>()
        val rawUri = alsSync(RawContacts.CONTENT_URI, konto)
        val dataUri = alsSync(Data.CONTENT_URI, konto)
        if (alt == null) {
          ops += ContentProviderOperation.newInsert(rawUri)
            .withValue(RawContacts.ACCOUNT_NAME, konto.name)
            .withValue(RawContacts.ACCOUNT_TYPE, konto.type)
            .withValue(RawContacts.SOURCE_ID, schluessel)
            .withValue(RawContacts.SYNC1, kt.getString("abdruck"))
            .build()
        } else {
          ops += ContentProviderOperation.newUpdate(rawUri)
            .withSelection("${RawContacts._ID}=?", arrayOf(alt.id.toString()))
            .withValue(RawContacts.SYNC1, kt.getString("abdruck"))
            .withValue(RawContacts.DIRTY, 0)
            .build()
          ops += ContentProviderOperation.newDelete(dataUri)
            .withSelection("${Data.RAW_CONTACT_ID}=?", arrayOf(alt.id.toString()))
            .build()
        }
        fun daten(mime: String): ContentProviderOperation.Builder {
          val b = ContentProviderOperation.newInsert(dataUri).withValue(Data.MIMETYPE, mime)
          return if (alt == null) b.withValueBackReference(Data.RAW_CONTACT_ID, 0) else b.withValue(Data.RAW_CONTACT_ID, alt.id)
        }
        ops += daten(StructuredName.CONTENT_ITEM_TYPE)
          .withValue(StructuredName.DISPLAY_NAME, kt.getString("name"))
          .build()
        val wege = kt.getJSONArray("wege")
        for (j in 0 until wege.length()) {
          weg(wege.getJSONObject(j), ::daten)?.let { ops += it }
        }
        foto(kt.optString("foto"))?.let { bytes ->
          ops += daten(Photo.CONTENT_ITEM_TYPE).withValue(Photo.PHOTO, bytes).build()
        }
        r.applyBatch(ContactsContract.AUTHORITY, ops)
      }
      // Kontakte, die es in openany nicht mehr gibt.
      for ((schluessel, z) in vorhanden) {
        if (schluessel !in gesehen && !z.schmutzig) {
          r.delete(alsSync(RawContacts.CONTENT_URI, konto), "${RawContacts._ID}=?", arrayOf(z.id.toString()))
        }
      }
      ""
    } catch (e: Exception) {
      Log.w(TAG, "Systemkontakte: Eintragen gescheitert", e)
      e.message ?: "Unknown error"
    }
  }

  /** Die Sorte als Android-Typ, je Art; `custom` traegt die Beschriftung. */
  private fun typ(art: String, sorte: String): Int = when (art) {
    "phone" -> when (sorte) {
      "mobile" -> Phone.TYPE_MOBILE; "home" -> Phone.TYPE_HOME; "work" -> Phone.TYPE_WORK
      "custom" -> Phone.TYPE_CUSTOM; else -> Phone.TYPE_OTHER
    }
    "email" -> when (sorte) {
      "mobile" -> Email.TYPE_MOBILE; "home" -> Email.TYPE_HOME; "work" -> Email.TYPE_WORK
      "custom" -> Email.TYPE_CUSTOM; else -> Email.TYPE_OTHER
    }
    "address" -> when (sorte) {
      "home" -> StructuredPostal.TYPE_HOME; "work" -> StructuredPostal.TYPE_WORK
      "custom" -> StructuredPostal.TYPE_CUSTOM; else -> StructuredPostal.TYPE_OTHER
    }
    else -> 0
  }

  private fun sorte(art: String, typ: Int): String = when (art) {
    "phone" -> when (typ) {
      Phone.TYPE_MOBILE -> "mobile"; Phone.TYPE_HOME -> "home"; Phone.TYPE_WORK -> "work"
      Phone.TYPE_CUSTOM -> "custom"; else -> "other"
    }
    "email" -> when (typ) {
      Email.TYPE_MOBILE -> "mobile"; Email.TYPE_HOME -> "home"; Email.TYPE_WORK -> "work"
      Email.TYPE_CUSTOM -> "custom"; else -> "other"
    }
    "address" -> when (typ) {
      StructuredPostal.TYPE_HOME -> "home"; StructuredPostal.TYPE_WORK -> "work"
      StructuredPostal.TYPE_CUSTOM -> "custom"; else -> "other"
    }
    else -> "other"
  }

  private fun weg(w: JSONObject, daten: (String) -> ContentProviderOperation.Builder): ContentProviderOperation? {
    val art = w.getString("art")
    val wert = w.getString("wert")
    val sorte = w.optString("sorte")
    val eigene = w.optString("beschriftung")
    return when (art) {
      "phone" -> daten(Phone.CONTENT_ITEM_TYPE).withValue(Phone.NUMBER, wert)
        .withValue(Phone.TYPE, typ(art, sorte)).withValue(Phone.LABEL, eigene.ifEmpty { null }).build()
      "email" -> daten(Email.CONTENT_ITEM_TYPE).withValue(Email.ADDRESS, wert)
        .withValue(Email.TYPE, typ(art, sorte)).withValue(Email.LABEL, eigene.ifEmpty { null }).build()
      "address" -> daten(StructuredPostal.CONTENT_ITEM_TYPE).withValue(StructuredPostal.FORMATTED_ADDRESS, wert)
        .withValue(StructuredPostal.STREET, wert)
        .withValue(StructuredPostal.TYPE, typ(art, sorte)).withValue(StructuredPostal.LABEL, eigene.ifEmpty { null }).build()
      "company" -> daten(Organization.CONTENT_ITEM_TYPE).withValue(Organization.COMPANY, wert)
        .withValue(Organization.TYPE, Organization.TYPE_WORK).build()
      "birthday" -> daten(Event.CONTENT_ITEM_TYPE).withValue(Event.START_DATE, wert)
        .withValue(Event.TYPE, Event.TYPE_BIRTHDAY).build()
      "vcard:url" -> daten(Website.CONTENT_ITEM_TYPE).withValue(Website.URL, wert)
        .withValue(Website.TYPE, Website.TYPE_OTHER).build()
      "vcard:nickname" -> daten(Nickname.CONTENT_ITEM_TYPE).withValue(Nickname.NAME, wert)
        .withValue(Nickname.TYPE, Nickname.TYPE_DEFAULT).build()
      else -> null
    }
  }

  /** Die Bytes des Fotos, verkleinert -- das Adressbuch will ein kleines. */
  private fun foto(pfad: String): ByteArray? {
    if (pfad.isEmpty()) return null
    val datei = File(pfad)
    if (!datei.exists()) return null
    val bild = BitmapFactory.decodeFile(datei.absolutePath) ?: return null
    val seite = 512
    val klein = if (bild.width > seite || bild.height > seite) {
      val f = seite.toFloat() / maxOf(bild.width, bild.height)
      Bitmap.createScaledBitmap(bild, (bild.width * f).toInt().coerceAtLeast(1), (bild.height * f).toInt().coerceAtLeast(1), true)
    } else bild
    return ByteArrayOutputStream().use { aus ->
      klein.compress(Bitmap.CompressFormat.JPEG, 85, aus)
      aus.toByteArray()
    }
  }

  /**
   * Was im Adressbuch des Telefons geaendert, angelegt oder geloescht wurde
   * (`DIRTY`/`DELETED`), als JSON fuer `systemkontakte.rs`.
   */
  @JvmStatic
  @Synchronized
  fun aenderungen(): String {
    val k = kontext ?: return "[]"
    if (!erlaubt(k)) return "[]"
    val aus = JSONArray()
    try {
      val konto = kontoName(k)
      val r = k.contentResolver
      r.query(
        alsSync(RawContacts.CONTENT_URI, konto),
        arrayOf(RawContacts._ID, RawContacts.SOURCE_ID, RawContacts.DELETED),
        "${RawContacts.ACCOUNT_NAME}=? AND ${RawContacts.ACCOUNT_TYPE}=? AND (${RawContacts.DIRTY}=1 OR ${RawContacts.DELETED}=1)",
        arrayOf(konto.name, konto.type), null,
      )?.use { c ->
        while (c.moveToNext()) {
          val id = c.getLong(0)
          val eintrag = JSONObject()
            .put("id", id)
            .put("schluessel", c.getString(1) ?: JSONObject.NULL)
            .put("geloescht", c.getInt(2) == 1)
          if (c.getInt(2) != 1) datenLesen(k, konto, id, eintrag)
          aus.put(eintrag)
        }
      }
    } catch (e: Exception) {
      Log.w(TAG, "Systemkontakte: Aenderungen nicht lesbar", e)
    }
    return aus.toString()
  }

  private fun datenLesen(k: Context, konto: Account, id: Long, eintrag: JSONObject) {
    val wege = JSONArray()
    var name = ""
    k.contentResolver.query(
      alsSync(Data.CONTENT_URI, konto),
      arrayOf(Data.MIMETYPE, Data.DATA1, Data.DATA2, Data.DATA3, Data.DATA4, Data.DATA7, Data.DATA9, Data.DATA10),
      "${Data.RAW_CONTACT_ID}=?", arrayOf(id.toString()), null,
    )?.use { c ->
      while (c.moveToNext()) {
        val wert = c.getString(1)?.trim().orEmpty()
        val typ = if (c.isNull(2)) 0 else c.getInt(2)
        val eigene = c.getString(3).orEmpty()
        fun weg(art: String, w: String = wert, s: String = sorte(art, typ)) {
          if (w.isNotEmpty()) wege.put(JSONObject().put("art", art).put("sorte", s).put("beschriftung", if (s == "custom") eigene else "").put("wert", w))
        }
        when (c.getString(0)) {
          StructuredName.CONTENT_ITEM_TYPE -> name = wert
          Phone.CONTENT_ITEM_TYPE -> weg("phone")
          Email.CONTENT_ITEM_TYPE -> weg("email")
          StructuredPostal.CONTENT_ITEM_TYPE -> {
            // Eine Zeile wie in openany; ohne fertige Zeile aus den Teilen.
            val zeile = wert.ifEmpty {
              // DATA4 Strasse, DATA9 Postleitzahl, DATA7 Ort, DATA10 Land.
              listOf(c.getString(4), listOfNotNull(c.getString(6), c.getString(5)).joinToString(" "), c.getString(7))
                .filterNotNull().map { it.trim() }.filter { it.isNotEmpty() }.joinToString(", ")
            }
            weg("address", zeile.replace("\n", ", "))
          }
          Organization.CONTENT_ITEM_TYPE -> weg("company", s = "other")
          Event.CONTENT_ITEM_TYPE -> if (typ == Event.TYPE_BIRTHDAY) weg("birthday", s = "other")
          Website.CONTENT_ITEM_TYPE -> weg("vcard:url", s = "other")
          Nickname.CONTENT_ITEM_TYPE -> weg("vcard:nickname", s = "other")
        }
      }
    }
    eintrag.put("name", name).put("wege", wege)
  }

  /**
   * Was uebernommen ist: neue bekommen ihren Schluessel, geaenderte werden
   * sauber (und ihr Abdruck geleert, damit der naechste Lauf openanys
   * Fassung eintraegt), geloeschte verschwinden endgueltig.
   */
  @JvmStatic
  @Synchronized
  fun bestaetigen(json: String) {
    val k = kontext ?: return
    if (!erlaubt(k)) return
    try {
      val konto = kontoName(k)
      val r = k.contentResolver
      val liste = JSONArray(json)
      for (i in 0 until liste.length()) {
        val e = liste.getJSONObject(i)
        val id = e.getLong("id").toString()
        if (e.getBoolean("loeschen")) {
          r.delete(alsSync(RawContacts.CONTENT_URI, konto), "${RawContacts._ID}=?", arrayOf(id))
          continue
        }
        val werte = ContentValues().apply {
          put(RawContacts.DIRTY, 0)
          put(RawContacts.SYNC1, "")
          if (!e.isNull("schluessel")) put(RawContacts.SOURCE_ID, e.getString("schluessel"))
        }
        r.update(alsSync(RawContacts.CONTENT_URI, konto), werte, "${RawContacts._ID}=?", arrayOf(id))
      }
    } catch (ex: Exception) {
      Log.w(TAG, "Systemkontakte: Bestaetigen gescheitert", ex)
    }
  }
}

/** Android stoesst den Abgleich an (etwa nach einer Aenderung im Adressbuch). */
class SystemkontakteSync : Service() {
  private val adapter by lazy {
    object : AbstractThreadedSyncAdapter(applicationContext, true) {
      override fun onPerformSync(a: Account?, e: Bundle?, au: String?, p: ContentProviderClient?, s: SyncResult?) {
        try {
          Tresor.bereit()
          Systemkontakte.bereit()
          Systemkontakte.kontext = context
          val ergebnis = Systemkontakte.laufen(context.dataDir.absolutePath, context.cacheDir.absolutePath)
          if (ergebnis.isNotEmpty()) Log.w("openany", "Systemkontakte: $ergebnis")
        } catch (ex: Throwable) {
          Log.w("openany", "Systemkontakte: Abgleich gescheitert", ex)
        }
      }
    }
  }

  override fun onBind(intent: Intent?): IBinder? = adapter.syncAdapterBinder
}
