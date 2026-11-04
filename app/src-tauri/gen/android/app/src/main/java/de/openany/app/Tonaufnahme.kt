package de.openany.app

import android.annotation.SuppressLint
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.media.AudioAttributes
import android.media.AudioFormat
import android.media.AudioPlaybackCaptureConfiguration
import android.media.AudioRecord
import android.media.MediaCodec
import android.media.MediaCodecInfo
import android.media.MediaFormat
import android.media.MediaMuxer
import android.media.MediaRecorder
import android.media.projection.MediaProjection
import android.media.projection.MediaProjectionManager
import android.os.Build
import android.os.Handler
import android.os.IBinder
import android.os.Looper
import android.os.PowerManager
import android.util.Log
import androidx.core.app.NotificationCompat
import androidx.core.content.ContextCompat
import java.io.File
import kotlin.math.abs
import kotlin.math.max
import kotlin.math.min
import org.json.JSONObject

/*
 * DIE TONAUFNAHME DER MEDIATHEK: Mikrofon oder Systemaudio, auch bei
 * ausgeschaltetem Bildschirm und mit der App im Hintergrund.
 *
 * EIN WEG FUER BEIDE QUELLEN: AudioRecord -> AAC -> .m4a. `MediaRecorder` kann
 * kein Systemaudio; das Mikrofon geht deshalb denselben Weg.
 *
 * DIE ZEIT KOMMT AUS DEN GELESENEN SAMPLES, nicht von der Uhr: Waehrend der
 * Pause wird weitergelesen, aber nichts geschrieben. So entsteht keine Luecke
 * in der Datei, und die Anzeige stimmt mit der Laenge ueberein.
 *
 * DIE FERTIGE DATEI bleibt unter cache/aufnahme/fertig.m4a liegen, bis die
 * Oberflaeche sie in die Mediathek holt (`galerie_aufnahme_ablegen`). So
 * ueberlebt sie auch das Beenden ueber die Benachrichtigung bei geschlossener
 * App. Wird der Prozess hart beendet, ist die laufende Datei verloren: Eine
 * MP4-Datei ist erst nach `muxer.stop` lesbar.
 */
class Tonaufnahme : Service() {
  private var faden: Thread? = null
  private var projektion: MediaProjection? = null

  override fun onBind(intent: Intent?): IBinder? = null

  override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
    if (intent?.action == AKTION_BEENDEN) {
      beendenGewuenscht = true
      return START_NOT_STICKY
    }
    if (faden != null) return START_NOT_STICKY

    val art = intent?.getStringExtra(EXTRA_ART) ?: ART_MIKROFON
    vordergrund(art)
    try {
      if (art == ART_SYSTEM) {
        val manager = getSystemService(Context.MEDIA_PROJECTION_SERVICE) as MediaProjectionManager
        @Suppress("DEPRECATION")
        val daten = intent?.getParcelableExtra<Intent>(EXTRA_DATEN)
          ?: throw IllegalStateException("keine Einwilligung")
        projektion = manager.getMediaProjection(intent.getIntExtra(EXTRA_CODE, 0), daten)?.apply {
          registerCallback(object : MediaProjection.Callback() {
            override fun onStop() { beendenGewuenscht = true }
          }, Handler(Looper.getMainLooper()))
        }
      }
    } catch (e: Exception) {
      Log.w(TAG, "Tonaufnahme: keine Projektion", e)
      fehler = FEHLER_TECHNIK
      laeuft = false
      stopSelf()
      return START_NOT_STICKY
    }

    faden = Thread({ aufnehmen(art) }, "openany-tonaufnahme").apply { start() }
    return START_NOT_STICKY
  }

  override fun onDestroy() {
    beendenGewuenscht = true
    faden?.join(5000)
    projektion?.stop()
    super.onDestroy()
  }

  private fun vordergrund(art: String) {
    val kanaele = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
    kanaele.createNotificationChannel(
      NotificationChannel(KANAL, getString(R.string.kanal_tonaufnahme), NotificationManager.IMPORTANCE_LOW).apply {
        description = getString(R.string.kanal_tonaufnahme_beschreibung)
      }
    )
    val oeffnen = packageManager.getLaunchIntentForPackage(packageName)?.let {
      PendingIntent.getActivity(this, 0, it, PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT)
    }
    val beenden = PendingIntent.getService(
      this, 1, Intent(this, Tonaufnahme::class.java).setAction(AKTION_BEENDEN),
      PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
    )
    val hinweis = NotificationCompat.Builder(this, KANAL)
      .setSmallIcon(R.drawable.ic_stat_openany)
      .setContentTitle(getString(R.string.tonaufnahme_hinweis))
      .setOngoing(true)
      .setContentIntent(oeffnen)
      .addAction(0, getString(R.string.tonaufnahme_beenden), beenden)
      .build()
    if (Build.VERSION.SDK_INT >= 30) {
      // AUCH BEI SYSTEMAUDIO DER MIKROFON-TYP: Ohne ihn fehlt dem Dienst das Recht
      // RECORD_AUDIO, sobald die App nicht mehr vorn ist, und Android liefert
      // stillschweigend Nullen -- genau dann, wenn eine andere App spielt.
      var typ = ServiceInfo.FOREGROUND_SERVICE_TYPE_MICROPHONE
      if (art == ART_SYSTEM) typ = typ or ServiceInfo.FOREGROUND_SERVICE_TYPE_MEDIA_PROJECTION
      startForeground(ID, hinweis, typ)
    } else {
      startForeground(ID, hinweis)
    }
  }

  @SuppressLint("MissingPermission") // MainActivity holt das Recht vor dem Start.
  private fun aufnahmegeraet(art: String, puffer: Int): AudioRecord {
    if (art != ART_SYSTEM || Build.VERSION.SDK_INT < 29) {
      return AudioRecord(MediaRecorder.AudioSource.MIC, RATE, AudioFormat.CHANNEL_IN_MONO, AudioFormat.ENCODING_PCM_16BIT, puffer)
    }
    val auswahl = AudioPlaybackCaptureConfiguration.Builder(projektion!!)
      .addMatchingUsage(AudioAttributes.USAGE_MEDIA)
      .addMatchingUsage(AudioAttributes.USAGE_GAME)
      .addMatchingUsage(AudioAttributes.USAGE_UNKNOWN)
      .build()
    return AudioRecord.Builder()
      .setAudioFormat(
        AudioFormat.Builder()
          .setEncoding(AudioFormat.ENCODING_PCM_16BIT)
          .setSampleRate(RATE)
          .setChannelMask(AudioFormat.CHANNEL_IN_STEREO)
          .build()
      )
      .setBufferSizeInBytes(puffer)
      .setAudioPlaybackCaptureConfig(auswahl)
      .build()
  }

  private fun aufnehmen(art: String) {
    val ordner = File(cacheDir, "aufnahme").apply { mkdirs() }
    val laufend = File(ordner, "laufend.m4a").apply { delete() }
    val sperre = (getSystemService(Context.POWER_SERVICE) as PowerManager)
      .newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, "openany:tonaufnahme")
    sperre.setReferenceCounted(false)
    sperre.acquire(ZWOELF_STUNDEN_MS)

    val kanaele = if (art == ART_SYSTEM) 2 else 1
    val maske = if (kanaele == 2) AudioFormat.CHANNEL_IN_STEREO else AudioFormat.CHANNEL_IN_MONO
    val puffer = max(AudioRecord.getMinBufferSize(RATE, maske, AudioFormat.ENCODING_PCM_16BIT) * 2, RATE * kanaele)

    var geraet: AudioRecord? = null
    var codec: MediaCodec? = null
    var muxer: MediaMuxer? = null
    var fertigOhneFehler = false
    var gesamplet = 0L
    try {
      geraet = aufnahmegeraet(art, puffer)
      if (geraet.state != AudioRecord.STATE_INITIALIZED) throw IllegalStateException("AudioRecord nicht bereit")

      val format = MediaFormat.createAudioFormat(MediaFormat.MIMETYPE_AUDIO_AAC, RATE, kanaele).apply {
        setInteger(MediaFormat.KEY_AAC_PROFILE, MediaCodecInfo.CodecProfileLevel.AACObjectLC)
        setInteger(MediaFormat.KEY_BIT_RATE, if (kanaele == 2) 160_000 else 96_000)
        setInteger(MediaFormat.KEY_MAX_INPUT_SIZE, puffer)
      }
      codec = MediaCodec.createEncoderByType(MediaFormat.MIMETYPE_AUDIO_AAC).apply {
        configure(format, null, null, MediaCodec.CONFIGURE_FLAG_ENCODE)
        start()
      }
      muxer = MediaMuxer(laufend.path, MediaMuxer.OutputFormat.MUXER_OUTPUT_MPEG_4)
      val schreiber = Schreiber(codec, muxer, kanaele)

      geraet.startRecording()
      val bytes = ByteArray(min(puffer, RATE * kanaele))
      while (!beendenGewuenscht) {
        val n = geraet.read(bytes, 0, bytes.size, AudioRecord.READ_NON_BLOCKING)
        if (n < 0) throw IllegalStateException("AudioRecord.read: $n")
        if (n == 0) {
          // Systemaudio liefert nichts, solange nichts spielt.
          schreiber.leeren(false)
          Thread.sleep(10)
          continue
        }
        pegel = spitze(bytes, n)
        if (!pausiert) {
          schreiber.fuellen(bytes, n, zeitstempel(gesamplet))
          gesamplet += n / (2L * kanaele)
          millisekunden = gesamplet * 1000 / RATE
        }
        schreiber.leeren(false)
      }
      geraet.stop()
      schreiber.abschliessen(zeitstempel(gesamplet))
      fertigOhneFehler = schreiber.hatDaten
      if (!fertigOhneFehler) fehler = FEHLER_LEER
    } catch (e: Exception) {
      Log.w(TAG, "Tonaufnahme gescheitert", e)
      fehler = FEHLER_TECHNIK
    } finally {
      try { codec?.stop() } catch (_: Exception) {}
      codec?.release()
      // Ohne eine einzige Spur wirft `stop`; dann ist ohnehin nichts zu retten.
      try { if (fertigOhneFehler) muxer?.stop() } catch (e: Exception) {
        Log.w(TAG, "Tonaufnahme: Datei nicht abgeschlossen", e)
        fertigOhneFehler = false
        fehler = FEHLER_TECHNIK
      }
      try { muxer?.release() } catch (_: Exception) {}
      geraet?.release()
      sperre.release()
    }

    if (fertigOhneFehler && !wegwerfen) {
      laufend.renameTo(File(ordner, DATEI_FERTIG))
      File(ordner, DATEI_DAUER).writeText((gesamplet.toDouble() / RATE).toString())
    } else {
      laufend.delete()
    }
    wegwerfen = false
    laeuft = false
    stopForeground(STOP_FOREGROUND_REMOVE)
    stopSelf()
  }

  private fun zeitstempel(samples: Long) = samples * 1_000_000L / RATE

  private fun spitze(bytes: ByteArray, n: Int): Float {
    var hoechst = 0
    var i = 0
    while (i + 1 < n) {
      val wert = (bytes[i].toInt() and 0xff) or (bytes[i + 1].toInt() shl 8)
      hoechst = max(hoechst, abs(wert))
      i += 2
    }
    return hoechst / 32768f
  }

  /** Reicht PCM an den Kodierer und dessen Ausgabe in die Datei. */
  private class Schreiber(private val codec: MediaCodec, private val muxer: MediaMuxer, private val kanaele: Int) {
    private val info = MediaCodec.BufferInfo()
    private var spur = -1
    var hatDaten = false
      private set

    fun fuellen(bytes: ByteArray, n: Int, zeit: Long) {
      var erledigt = 0
      while (erledigt < n) {
        val i = codec.dequeueInputBuffer(WARTEN_US)
        if (i < 0) {
          leeren(false)
          continue
        }
        val eingang = codec.getInputBuffer(i)!!.apply { clear() }
        val stueck = min(eingang.remaining(), n - erledigt)
        eingang.put(bytes, erledigt, stueck)
        codec.queueInputBuffer(i, 0, stueck, zeit + erledigt * 1_000_000L / (RATE * 2 * kanaele), 0)
        erledigt += stueck
      }
    }

    fun abschliessen(zeit: Long) {
      var i = codec.dequeueInputBuffer(WARTEN_US)
      while (i < 0) {
        leeren(false)
        i = codec.dequeueInputBuffer(WARTEN_US)
      }
      codec.queueInputBuffer(i, 0, 0, zeit, MediaCodec.BUFFER_FLAG_END_OF_STREAM)
      leeren(true)
    }

    fun leeren(bisZumEnde: Boolean) {
      while (true) {
        val i = codec.dequeueOutputBuffer(info, if (bisZumEnde) WARTEN_US else 0)
        when {
          i == MediaCodec.INFO_TRY_AGAIN_LATER -> if (!bisZumEnde) return
          i == MediaCodec.INFO_OUTPUT_FORMAT_CHANGED -> {
            spur = muxer.addTrack(codec.outputFormat)
            muxer.start()
          }
          i >= 0 -> {
            val ende = info.flags and MediaCodec.BUFFER_FLAG_END_OF_STREAM != 0
            val nurKopf = info.flags and MediaCodec.BUFFER_FLAG_CODEC_CONFIG != 0
            if (info.size > 0 && !nurKopf && spur >= 0) {
              muxer.writeSampleData(spur, codec.getOutputBuffer(i)!!, info)
              hatDaten = true
            }
            codec.releaseOutputBuffer(i, false)
            if (ende) return
          }
        }
      }
    }
  }

  companion object {
    private const val TAG = "openany"
    private const val KANAL = "tonaufnahme"
    private const val ID = 4711
    private const val RATE = 44_100
    private const val WARTEN_US = 10_000L
    private const val ZWOELF_STUNDEN_MS = 12L * 60 * 60 * 1000
    private const val AKTION_BEENDEN = "de.openany.app.TONAUFNAHME_BEENDEN"
    private const val EXTRA_ART = "art"
    private const val EXTRA_CODE = "code"
    private const val EXTRA_DATEN = "daten"
    private const val DATEI_FERTIG = "fertig.m4a"
    private const val DATEI_DAUER = "fertig.dauer"

    const val ART_MIKROFON = "mikrofon"
    const val ART_SYSTEM = "system"
    const val FEHLER_RECHT = "recht"
    const val FEHLER_ABGEBROCHEN = "abgebrochen"
    const val FEHLER_TECHNIK = "technik"
    const val FEHLER_LEER = "leer"

    @Volatile var fehler: String? = null
    @Volatile var pausiert = false
    @Volatile private var laeuft = false
    @Volatile private var aktuelleArt = ART_MIKROFON
    @Volatile private var beendenGewuenscht = false
    @Volatile private var wegwerfen = false
    @Volatile private var millisekunden = 0L
    @Volatile private var pegel = 0f

    fun starten(kontext: Context, art: String, code: Int, daten: Intent?) {
      if (laeuft) return
      laeuft = true
      aktuelleArt = art
      beendenGewuenscht = false
      wegwerfen = false
      pausiert = false
      millisekunden = 0
      pegel = 0f
      fehler = null
      val absicht = Intent(kontext, Tonaufnahme::class.java)
        .putExtra(EXTRA_ART, art)
        .putExtra(EXTRA_CODE, code)
      if (daten != null) absicht.putExtra(EXTRA_DATEN, daten)
      ContextCompat.startForegroundService(kontext, absicht)
    }

    /** `verwerfen` raeumt auch eine fertige Datei weg, die noch niemand geholt hat. */
    fun beenden(kontext: Context, verwerfen: Boolean) {
      if (verwerfen) {
        wegwerfen = true
        val ordner = File(kontext.cacheDir, "aufnahme")
        File(ordner, DATEI_FERTIG).delete()
        File(ordner, DATEI_DAUER).delete()
      }
      beendenGewuenscht = true
    }

    fun stand(kontext: Context): String {
      val ordner = File(kontext.cacheDir, "aufnahme")
      val fertig = !laeuft && File(ordner, DATEI_FERTIG).isFile
      val dauer = if (fertig) File(ordner, DATEI_DAUER).takeIf { it.isFile }?.readText()?.toDoubleOrNull() else null
      return JSONObject()
        .put("laeuft", laeuft)
        .put("pausiert", pausiert)
        .put("art", aktuelleArt)
        .put("ms", millisekunden)
        .put("pegel", pegel.toDouble())
        .put("fehler", fehler ?: JSONObject.NULL)
        .put("fertig", fertig)
        .put("dauer", dauer ?: JSONObject.NULL)
        .toString()
    }
  }
}
