package de.openany.app

import android.content.Context
import android.net.ConnectivityManager
import android.util.Log
import androidx.work.BackoffPolicy
import androidx.work.Constraints
import androidx.work.ExistingPeriodicWorkPolicy
import androidx.work.NetworkType
import androidx.work.PeriodicWorkRequestBuilder
import androidx.work.WorkManager
import androidx.work.Worker
import androidx.work.WorkerParameters
import java.util.concurrent.TimeUnit

/*
 * DER AUFFRISCHER: ein Abgleich mit openany.de, ohne dass jemand die App
 * oeffnet und drueckt.
 *
 * WorkManager weckt ihn etwa stuendlich (das ist das Mindeste, was Android
 * fuer regelmaessige Arbeit zulaesst, ist 15 Minuten; wir brauchen weniger),
 * nur mit Netz und nicht bei leerem Akku. Wann genau, entscheidet Android --
 * Doze und Akkusparen schieben ihn, und das ist in Ordnung: Es geht um den
 * Stand vom Abend am Morgen, nicht um Sekunden.
 *
 * ER LAEUFT IM SELBEN PROZESS WIE DAS FENSTER, wenn es eines gibt -- sonst
 * in einem frischen, ohne Tauri. Die Rust-Seite (`hintergrund.rs`) oeffnet
 * den Speicher oder nimmt den schon offenen; Datenordner und Cache muessen
 * dabei DIESELBEN sein, die Tauri dem Fenster nennt (dataDir, cacheDir --
 * siehe `Zustand::holen`). Sonst gaebe es zwei Datenbanken.
 *
 * Auf einer gemessenen Leitung (Mobilfunk) holt er nur die Liste; die Bytes
 * der Dateien und Bilder erst im WLAN. Das kann nur Kotlin wissen, deshalb
 * reist es als Argument mit.
 */
class Auffrischer(ctx: Context, params: WorkerParameters) : Worker(ctx, params) {
  override fun doWork(): Result {
    val ctx = applicationContext
    val netz = ctx.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager
    val nurListe = netz.isActiveNetworkMetered

    val ergebnis = try {
      // Der Auffrischer laeuft auch, wenn das Programm nie geoeffnet wurde --
      // dann hat noch niemand den Tresor angemeldet.
      Tresor.bereit()
      Standbild.bereit()
      laufen(ctx.dataDir.absolutePath, ctx.cacheDir.absolutePath, nurListe)
    } catch (e: Throwable) {
      // Die Bibliothek fehlt oder der Aufruf brach -- spaeter noch einmal,
      // mit wachsendem Abstand (siehe einplanen).
      Log.w(TAG, "Auffrischer: Aufruf gescheitert", e)
      return Result.retry()
    }

    Log.i(TAG, "Auffrischer: $ergebnis")
    // Ist das Fenster offen, soll es den neuen Stand zeigen, statt ihn erst
    // beim naechsten Oeffnen zu bemerken.
    MainActivity.aufgefrischt()
    return Result.success()
  }

  companion object {
    private const val TAG = "openany"
    private const val NAME = "auffrischer"
    private const val EINSTELLUNG = "auffrischer"

    init {
      // Dieselbe Bibliothek, die auch `Rust` laedt. Zweimal laden ist
      // erlaubt und kostet nichts; hier fehlt sie sonst, wenn der Prozess
      // vom WorkManager statt vom Fenster gestartet wurde.
      System.loadLibrary("openany_app_lib")
    }

    /** `hintergrund.rs`: Java_de_openany_app_Auffrischer_laufen. */
    @JvmStatic external fun laufen(daten: String, zwischen: String, nurListe: Boolean): String

    /** Ob der Mensch ihn will. Vorgabe: ja -- ohne Instanz tut er ohnehin nichts. */
    fun gewuenscht(ctx: Context): Boolean =
      ctx.getSharedPreferences("openany", Context.MODE_PRIVATE).getBoolean(EINSTELLUNG, true)

    fun setzen(ctx: Context, an: Boolean) {
      ctx.getSharedPreferences("openany", Context.MODE_PRIVATE).edit().putBoolean(EINSTELLUNG, an).apply()
      if (an) einplanen(ctx) else WorkManager.getInstance(ctx).cancelUniqueWork(NAME)
    }

    /**
     * Einplanen -- bei jedem Start der App noch einmal, mit KEEP: Ein schon
     * laufender Plan bleibt, samt seinem Takt. Ohne den Aufruf beim Start
     * verschwaende der Plan mit einer Neuinstallation, und niemand saehe es.
     */
    fun einplanen(ctx: Context) {
      val bedingungen = Constraints.Builder()
        .setRequiredNetworkType(NetworkType.CONNECTED)
        .setRequiresBatteryNotLow(true)
        .build()
      val anfrage = PeriodicWorkRequestBuilder<Auffrischer>(1, TimeUnit.HOURS)
        .setConstraints(bedingungen)
        .setBackoffCriteria(BackoffPolicy.EXPONENTIAL, 15, TimeUnit.MINUTES)
        .build()
      WorkManager.getInstance(ctx).enqueueUniquePeriodicWork(NAME, ExistingPeriodicWorkPolicy.KEEP, anfrage)
    }
  }
}
