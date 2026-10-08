package de.openany.app

import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyPermanentlyInvalidatedException
import android.security.keystore.KeyProperties
import android.util.Log
import java.security.KeyStore
import java.security.UnrecoverableKeyException
import javax.crypto.AEADBadTagException
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

/**
 * Der Tresor fuer die Ausweise -- ein Schluessel im Android-Keystore.
 *
 * Die drei Ausweise dieses Programms (anyid-Geraetetoken, openany-
 * Geraeteschluessel, Matrix-Sitzung samt der Passphrase fuer den Speicher des
 * SDK) lagen bis zum 22.09.2026 als Klartext im App-Ordner. Der ist zwar nur
 * fuer diese App lesbar, aber er reist mit der Sicherung des Geraets, und auf
 * einem gerooteten Telefon liegt er offen.
 *
 * **Der Schluessel verlaesst den Keystore nie.** Er entsteht auf dem Geraet,
 * ist nicht exportierbar, und auf Geraeten mit Sicherheitschip liegt er dort.
 * Rust gibt Bytes hinein und bekommt Bytes heraus; der Schluessel selbst ist
 * fuer niemanden zu haben, auch nicht fuer dieses Programm.
 *
 * **Keine Nutzer-Bestaetigung.** Der Auffrischer im Hintergrund muss den
 * Ausweis lesen, waehrend das Geraet gesperrt in der Tasche liegt.
 *
 * **Ist der Schluessel fort** -- eine Sicherung auf ein anderes Geraet
 * zurueckgespielt, der Keystore geleert --, gibt [oeffnen] `null`. Der Rust-
 * Teil liest das als "nicht gekoppelt", und der Mensch koppelt neu. Ein
 * Ausweis, den niemand mehr oeffnen kann, ist kein Ausweis.
 *
 * Gerufen wird von Rust ueber JNI (`src-tauri/src/tresor.rs`). Damit das von
 * jedem Faden aus geht, meldet sich die Klasse beim Laden selbst an: Nur auf
 * einem Java-Faden findet JNI die Klassen dieser App.
 */
object Tresor {
  private const val TAG = "openany"
  private const val ALIAS = "openany-ausweise"
  private const val IV_LAENGE = 12
  private const val TAG_BITS = 128

  init {
    System.loadLibrary("openany_app_lib")
    anmelden()
  }

  @JvmStatic private external fun anmelden()

  /** Beruehren genuegt: Das laedt die Klasse und meldet sie bei Rust an. */
  fun bereit() {}

  /** Klartext hinein, `IV ‖ Geheimtext ‖ Tag` heraus. */
  @JvmStatic
  fun verschliessen(klar: ByteArray): ByteArray {
    val chiffre = Cipher.getInstance("AES/GCM/NoPadding")
    chiffre.init(Cipher.ENCRYPT_MODE, schluessel(anlegen = true))
    return chiffre.iv + chiffre.doFinal(klar)
  }

  /** Das Gegenstueck -- `null`, wenn es sich nicht mehr oeffnen laesst. */
  @JvmStatic
  fun oeffnen(zu: ByteArray): ByteArray? {
    if (zu.size <= IV_LAENGE) return null
    val schluessel = schluessel(anlegen = false) ?: return null

    return try {
      val chiffre = Cipher.getInstance("AES/GCM/NoPadding")
      chiffre.init(Cipher.DECRYPT_MODE, schluessel, GCMParameterSpec(TAG_BITS, zu, 0, IV_LAENGE))
      chiffre.doFinal(zu, IV_LAENGE, zu.size - IV_LAENGE)
    } catch (e: AEADBadTagException) {
      Log.w(TAG, "Tresor: Ausweis passt nicht zum Schluessel", e)
      null
    } catch (e: KeyPermanentlyInvalidatedException) {
      Log.w(TAG, "Tresor: Schluessel ungueltig geworden", e)
      null
    }
  }

  @Synchronized
  private fun schluessel(anlegen: Boolean): SecretKey? {
    val ablage = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }

    try {
      (ablage.getKey(ALIAS, null) as? SecretKey)?.let { return it }
    } catch (e: UnrecoverableKeyException) {
      Log.w(TAG, "Tresor: Schluessel nicht lesbar", e)
      if (!anlegen) return null
      ablage.deleteEntry(ALIAS)
    }

    if (!anlegen) return null

    val erzeuger = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, "AndroidKeyStore")
    erzeuger.init(
      KeyGenParameterSpec.Builder(ALIAS, KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
        .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
        .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
        .setKeySize(256)
        .build()
    )
    // Einmal je Geraet -- die einzige Spur, dass der Tresor wirklich
    // verschlossen hat (eine Release-Fassung laesst niemanden in ihren
    // Ordner schauen, auch adb nicht).
    Log.i(TAG, "Tresor: Schluessel im Keystore angelegt")
    return erzeuger.generateKey()
  }
}
