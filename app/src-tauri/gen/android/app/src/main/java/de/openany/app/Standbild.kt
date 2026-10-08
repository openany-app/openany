package de.openany.app

import android.graphics.Bitmap
import android.media.MediaMetadataRetriever
import android.os.Build
import android.util.Log
import java.io.ByteArrayOutputStream
import java.nio.ByteBuffer
import kotlin.math.min
import kotlin.math.roundToInt

/**
 * Das Standbild eines Videos -- mit dem Werkzeug, das Android dafür hat.
 *
 * WARUM HIER UND NICHT IN DER WEBANSICHT (27.09.2026). Firefox auf Android gab
 * aus einem Video nur schwarze Bilder heraus, und in der Webansicht dieses
 * Programms spielen Videos aus der Ablage nicht zuverlässig (abgefangene
 * Anfragen). `MediaMetadataRetriever` liest direkt aus der Datei, mit dem
 * Hardware-Decoder, ohne dass etwas abgespielt wird -- und ohne Fenster, also
 * auch im Hintergrund nach dem Abgleich.
 *
 * Gerufen wird von Rust über JNI (`src-tauri/src/standbild.rs`). Wie beim
 * Tresor meldet sich die Klasse beim Laden selbst an, damit Rust sie von
 * jedem Faden aus findet.
 */
object Standbild {
  private const val TAG = "openany"

  /** Lange Seite höchstens so groß -- dieselbe Vorschau wie bei Fotos. */
  private const val KANTE = 480

  init {
    System.loadLibrary("openany_app_lib")
    anmelden()
  }

  @JvmStatic private external fun anmelden()

  /** Berühren genügt: Das lädt die Klasse und meldet sie bei Rust an. */
  fun bereit() {}

  /**
   * `[Dauer in ms, 8 Bytes groß-endian] ‖ JPEG` -- oder `null`, wenn die Datei
   * kein lesbares Video ist. Ohne Bild, aber mit Dauer: nur die 8 Bytes.
   */
  @JvmStatic
  fun ziehen(pfad: String): ByteArray? {
    val leser = MediaMetadataRetriever()
    return try {
      leser.setDataSource(pfad)
      val dauerMs = leser.extractMetadata(MediaMetadataRetriever.METADATA_KEY_DURATION)?.toLongOrNull() ?: 0L
      // Eine Sekunde hinein, damit es nicht das schwarze Anfangsbild ist;
      // bei kurzen Clips die Mitte. In Mikrosekunden.
      val zeit = min(1_000_000L, dauerMs * 1000 / 2)
      val bild: Bitmap? = if (Build.VERSION.SDK_INT >= 27) {
        leser.getScaledFrameAtTime(zeit, MediaMetadataRetriever.OPTION_CLOSEST_SYNC, KANTE, KANTE)
      } else {
        leser.getFrameAtTime(zeit, MediaMetadataRetriever.OPTION_CLOSEST_SYNC)?.let(::verkleinern)
      }
      val jpeg = bild?.let {
        val aus = ByteArrayOutputStream()
        it.compress(Bitmap.CompressFormat.JPEG, 80, aus)
        it.recycle()
        aus.toByteArray()
      } ?: ByteArray(0)
      ByteBuffer.allocate(8 + jpeg.size).putLong(dauerMs).put(jpeg).array()
    } catch (e: Exception) {
      Log.w(TAG, "Standbild: nicht lesbar", e)
      null
    } finally {
      try { leser.release() } catch (_: Exception) {}
    }
  }

  private fun verkleinern(bild: Bitmap): Bitmap {
    val lang = maxOf(bild.width, bild.height)
    if (lang <= KANTE) return bild
    val faktor = KANTE.toFloat() / lang
    val klein = Bitmap.createScaledBitmap(bild, (bild.width * faktor).roundToInt(), (bild.height * faktor).roundToInt(), true)
    bild.recycle()
    return klein
  }
}
