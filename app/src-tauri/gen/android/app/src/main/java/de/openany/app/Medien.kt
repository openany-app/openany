package de.openany.app

import android.Manifest
import android.content.ContentUris
import android.content.Context
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import android.provider.MediaStore
import android.util.Log
import org.json.JSONArray
import org.json.JSONObject

/**
 * Die Fotos und Videos des Geräts -- aus der Medienbibliothek (`MediaStore`),
 * für „Fotos vom Gerät übernehmen" (docs/plan-viele-dateien.md, D3).
 *
 * Gerufen wird von Rust über JNI (`src-tauri/src/medien.rs`): Rust fragt,
 * was es gibt, wählt aus und öffnet jedes Foto als Dateideskriptor. Die
 * Webansicht nennt nur die Auswahl („alle", „diese Alben", „ab Datum") --
 * nie eine Adresse, die geöffnet würde.
 *
 * Wie Standbild und Tresor meldet sich die Klasse beim Laden selbst an.
 */
object Medien {
  private const val TAG = "openany"

  /** Gesetzt in MainActivity.onCreate -- für die Medienbibliothek. */
  @Volatile var kontext: Context? = null

  init {
    System.loadLibrary("openany_app_lib")
    anmelden()
  }

  @JvmStatic private external fun anmelden()

  fun bereit() {}

  /** Was erlaubt ist: `alle`, `auswahl` (ab Android 14 „nur ausgewählte") oder `keine`. */
  fun stand(k: Context): String {
    fun hat(recht: String) = k.checkSelfPermission(recht) == PackageManager.PERMISSION_GRANTED
    return when {
      Build.VERSION.SDK_INT >= 33 && hat(Manifest.permission.READ_MEDIA_IMAGES) -> "alle"
      Build.VERSION.SDK_INT >= 34 && hat(Manifest.permission.READ_MEDIA_VISUAL_USER_SELECTED) -> "auswahl"
      Build.VERSION.SDK_INT < 33 && hat(Manifest.permission.READ_EXTERNAL_STORAGE) -> "alle"
      else -> "keine"
    }
  }

  /** Die Rechte, nach denen „Fotos vom Gerät übernehmen" fragt. */
  fun rechte(): Array<String> {
    val r = mutableListOf<String>()
    if (Build.VERSION.SDK_INT >= 33) {
      r += Manifest.permission.READ_MEDIA_IMAGES
      r += Manifest.permission.READ_MEDIA_VIDEO
      r += Manifest.permission.POST_NOTIFICATIONS
    } else {
      r += Manifest.permission.READ_EXTERNAL_STORAGE
    }
    if (Build.VERSION.SDK_INT >= 34) r += Manifest.permission.READ_MEDIA_VISUAL_USER_SELECTED
    // Sonst schwärzt Android den Ort in den EXIF-Daten.
    if (Build.VERSION.SDK_INT >= 29) r += Manifest.permission.ACCESS_MEDIA_LOCATION
    return r.toTypedArray()
  }

  private fun sammlungen(): List<Pair<String, Uri>> =
    if (Build.VERSION.SDK_INT >= 29) {
      listOf(
        "bild" to MediaStore.Images.Media.getContentUri(MediaStore.VOLUME_EXTERNAL),
        "video" to MediaStore.Video.Media.getContentUri(MediaStore.VOLUME_EXTERNAL),
      )
    } else {
      listOf(
        "bild" to MediaStore.Images.Media.EXTERNAL_CONTENT_URI,
        "video" to MediaStore.Video.Media.EXTERNAL_CONTENT_URI,
      )
    }

  private class Eintrag(
    val uri: String,
    val kennung: String,
    val name: String,
    val mime: String,
    val groesse: Long,
    val mtime: Long,
    val album: String,
    val datum: Long,
  )

  private fun alle(k: Context): List<Eintrag> {
    val liste = mutableListOf<Eintrag>()
    val spalten = arrayOf(
      MediaStore.MediaColumns._ID,
      MediaStore.MediaColumns.DISPLAY_NAME,
      MediaStore.MediaColumns.MIME_TYPE,
      MediaStore.MediaColumns.SIZE,
      MediaStore.MediaColumns.DATE_MODIFIED,
      MediaStore.Images.ImageColumns.BUCKET_DISPLAY_NAME,
      MediaStore.Images.ImageColumns.DATE_TAKEN,
    )
    for ((art, sammlung) in sammlungen()) {
      try {
        k.contentResolver.query(sammlung, spalten, null, null, null)?.use { c ->
          while (c.moveToNext()) {
            val id = c.getLong(0)
            val mtime = c.getLong(4)
            val datum = c.getLong(6).takeIf { it > 0 } ?: (mtime * 1000)
            liste += Eintrag(
              uri = ContentUris.withAppendedId(sammlung, id).toString(),
              kennung = "$art:$id",
              name = c.getString(1) ?: "$id",
              mime = c.getString(2) ?: "",
              groesse = c.getLong(3),
              mtime = mtime,
              album = c.getString(5) ?: "",
              datum = datum,
            )
          }
        }
      } catch (e: Exception) {
        Log.w(TAG, "Medien: $art nicht lesbar", e)
      }
    }
    // Die neuesten zuerst: Reicht der Platz nicht, kommen sie zuerst.
    liste.sortByDescending { it.datum }
    return liste
  }

  /** `{ erlaubt, anzahl, bytes, alben: [{ name, anzahl, bytes }] }` */
  @JvmStatic
  fun uebersicht(): String {
    val k = kontext ?: return "{}"
    val erlaubt = stand(k)
    val eintraege = if (erlaubt == "keine") emptyList() else alle(k)
    val alben = JSONArray()
    eintraege.groupBy { it.album }.toSortedMap(String.CASE_INSENSITIVE_ORDER).forEach { (name, e) ->
      alben.put(JSONObject().put("name", name).put("anzahl", e.size).put("bytes", e.sumOf { it.groesse }))
    }
    return JSONObject()
      .put("erlaubt", erlaubt)
      .put("anzahl", eintraege.size)
      .put("bytes", eintraege.sumOf { it.groesse })
      .put("alben", alben)
      .toString()
  }

  /**
   * Die Fotos und Videos der Auswahl, die neuesten zuerst. `alben` als
   * JSON-Liste von Namen, leer = alle; `ab` in ms seit 1970, 0 = alle.
   */
  @JvmStatic
  fun liste(alben: String, ab: Long): String {
    val k = kontext ?: return "[]"
    if (stand(k) == "keine") return "[]"
    val nur = try {
      JSONArray(alben).let { a -> (0 until a.length()).map { a.getString(it) }.toSet() }
    } catch (_: Exception) {
      emptySet()
    }
    val aus = JSONArray()
    for (e in alle(k)) {
      if (nur.isNotEmpty() && e.album !in nur) continue
      if (ab > 0 && e.datum < ab) continue
      aus.put(
        JSONObject()
          .put("uri", e.uri).put("kennung", e.kennung).put("name", e.name).put("mime", e.mime)
          .put("groesse", e.groesse).put("mtime", e.mtime).put("album", e.album)
      )
    }
    return aus.toString()
  }

  /** Mobilfunk oder Hotspot: Was nur im WLAN hinaus soll, wartet dann. */
  @JvmStatic
  fun gemessen(): Boolean {
    val k = kontext ?: return false
    val netz = k.getSystemService(Context.CONNECTIVITY_SERVICE) as android.net.ConnectivityManager
    return netz.isActiveNetworkMetered
  }

  /** Von Rust (fotowaechter.rs): den Fotowächter (D5) ein- oder ausplanen. */
  @JvmStatic
  fun waechterEinplanen(an: Boolean, nurWlan: Boolean, nurLaden: Boolean) {
    val k = kontext ?: return
    Fotowaechter.einplanen(k, an, nurWlan, nurLaden)
  }

  /** Von Rust, wenn die Schlange anläuft -- über diese Klasse, weil Rust sie kennt. */
  @JvmStatic
  fun dienstStarten() = Uebernahmedienst.starten()

  /**
   * Ein Foto zum Lesen öffnen: ein Dateideskriptor, den Rust übernimmt und
   * schließt -- oder -1. NUR aus der Medienbibliothek.
   */
  @JvmStatic
  fun oeffnen(adresse: String): Int {
    val k = kontext ?: return -1
    return try {
      var uri = Uri.parse(adresse)
      if (uri.scheme != "content" || uri.authority != MediaStore.AUTHORITY) return -1
      // Mit dem Ort in den EXIF-Daten, wenn erlaubt.
      if (Build.VERSION.SDK_INT >= 29 &&
        k.checkSelfPermission(Manifest.permission.ACCESS_MEDIA_LOCATION) == PackageManager.PERMISSION_GRANTED
      ) {
        uri = MediaStore.setRequireOriginal(uri)
      }
      k.contentResolver.openFileDescriptor(uri, "r")?.detachFd() ?: -1
    } catch (e: Exception) {
      Log.w(TAG, "Medien: nicht zu öffnen", e)
      -1
    }
  }
}
