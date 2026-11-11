package de.openany.app

import android.content.Context
import android.provider.MediaStore
import android.util.Log
import androidx.work.Constraints
import androidx.work.ExistingWorkPolicy
import androidx.work.NetworkType
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.WorkManager
import androidx.work.Worker
import androidx.work.WorkerParameters
import java.util.concurrent.TimeUnit

/*
 * DER FOTOWAECHTER (docs/plan-viele-dateien.md, D5): Was die Kamera neu
 * aufnimmt, kommt von selbst in die Mediathek -- und, ist der Abgleich an,
 * verschluesselt zu openany.de.
 *
 * BEMERKEN STATT SUCHEN: Der WorkManager weckt ihn, wenn sich in der
 * Medienbibliothek etwas tut (`ContentUriTrigger`). Ein solcher Auftrag
 * gilt fuer EIN Ereignis; danach plant er sich selbst neu ein. Wann genau
 * er laeuft, entscheidet Android -- neue Fotos kommen mit Verzug, nicht
 * sofort, und das sagt die Oberflaeche.
 *
 * NUR DIE HUELLE, wie der Auffrischer: Was neu ist und was damit geschieht,
 * entscheidet Rust (`fotowaechter.rs`), im selben Prozess wie das Fenster,
 * wenn es eines gibt, sonst in einem frischen.
 */
class Fotowaechter(ctx: Context, params: WorkerParameters) : Worker(ctx, params) {
  override fun doWork(): Result {
    val ctx = applicationContext
    try {
      Tresor.bereit()
      Standbild.bereit()
      Medien.bereit()
      Medien.kontext = ctx
      Uebernahmedienst.kontext = ctx
      val ergebnis = laufen(ctx.dataDir.absolutePath, ctx.cacheDir.absolutePath)
      Log.i(TAG, "Fotowaechter: $ergebnis")
      MainActivity.aufgefrischt()
    } catch (e: Throwable) {
      Log.w(TAG, "Fotowaechter: Aufruf gescheitert", e)
    }
    // Fuer das naechste Ereignis -- mit denselben Bedingungen.
    val p = einstellungen(ctx)
    if (p.getBoolean(AN, false)) planen(ctx, p.getBoolean(NUR_WLAN, false), p.getBoolean(NUR_LADEN, false))
    return Result.success()
  }

  companion object {
    private const val TAG = "openany"
    private const val NAME = "fotowaechter"
    private const val AN = "fotowaechter_an"
    private const val NUR_WLAN = "fotowaechter_nur_wlan"
    private const val NUR_LADEN = "fotowaechter_nur_laden"

    init {
      System.loadLibrary("openany_app_lib")
    }

    /** `fotowaechter.rs`: Java_de_openany_app_Fotowaechter_laufen. */
    @JvmStatic external fun laufen(daten: String, zwischen: String): String

    private fun einstellungen(ctx: Context) = ctx.getSharedPreferences("openany", Context.MODE_PRIVATE)

    /** Von Rust (ueber Medien), wenn der Mensch ihn ein- oder ausschaltet -- und beim Start. */
    fun einplanen(ctx: Context, an: Boolean, nurWlan: Boolean, nurLaden: Boolean) {
      einstellungen(ctx).edit()
        .putBoolean(AN, an).putBoolean(NUR_WLAN, nurWlan).putBoolean(NUR_LADEN, nurLaden)
        .apply()
      if (an) planen(ctx, nurWlan, nurLaden) else WorkManager.getInstance(ctx).cancelUniqueWork(NAME)
    }

    private fun planen(ctx: Context, nurWlan: Boolean, nurLaden: Boolean) {
      val bedingungen = Constraints.Builder()
        .addContentUriTrigger(MediaStore.Images.Media.EXTERNAL_CONTENT_URI, true)
        .addContentUriTrigger(MediaStore.Video.Media.EXTERNAL_CONTENT_URI, true)
        // Eine Serie von Fotos weckt ihn einmal, nicht zwanzigmal.
        .setTriggerContentUpdateDelay(10, TimeUnit.SECONDS)
        .setTriggerContentMaxDelay(2, TimeUnit.MINUTES)
        .setRequiredNetworkType(if (nurWlan) NetworkType.UNMETERED else NetworkType.NOT_REQUIRED)
        .setRequiresCharging(nurLaden)
        .build()
      val auftrag = OneTimeWorkRequestBuilder<Fotowaechter>().setConstraints(bedingungen).build()
      WorkManager.getInstance(ctx).enqueueUniqueWork(NAME, ExistingWorkPolicy.REPLACE, auftrag)
    }
  }
}
