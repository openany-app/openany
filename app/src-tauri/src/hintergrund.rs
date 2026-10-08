//! Der Auffrischer: ein Abgleich mit openany.de, ohne dass jemand die App
//! oeffnet und drueckt.
//!
//! **Bis zum 17.09.2026 musste ein Mensch die App oeffnen und druecken.**
//! Ein Tablet, das nachts auf dem Tisch liegt, hatte am Morgen den Stand
//! vom Vorabend -- und zeigte ihn wie einen frischen.
//!
//! Auf Android laeuft das ueber den `WorkManager` (`Auffrischer.kt`, etwa
//! stuendlich, nur mit Netz und nicht bei leerem Akku). Der ruft ueber JNI
//! hier herein -- **in denselben Prozess wie das Fenster, wenn es eines
//! gibt**, sonst in einen frischen ohne Tauri. Deshalb haengt hier nichts an
//! einem `AppHandle`: [`crate::Zustand::holen`] oeffnet den Speicher oder gibt
//! den schon offenen zurueck, und `tauri::async_runtime` hat auch ohne
//! Fenster einen Laufzeitkern.
//!
//! **Nur der Server, nicht die Geraete in der Naehe.** Fuer die braeuchte es
//! die Multicast-Sperre und den Dienst auf 53318, und beides gehoert dem
//! Fenster (siehe `MainActivity.kt`: gehalten, solange die App vorn ist).
//!
//! **Auf einer Mobilfunkleitung nur die Liste.** Notizen, Termine, Kontakte,
//! Zuordnungen -- klein. Die Bytes der Dateien und Bilder (`inhalte_auch`)
//! nur im WLAN; das entscheidet Kotlin (`isActiveNetworkMetered`) und sagt es
//! mit.

use crate::Zustand;
use std::path::PathBuf;
use std::sync::Arc;

/// Ein Lauf im Hintergrund. Gibt einen kurzen JSON-Text zurueck, der in
/// Androids Log landet -- mehr sieht dort niemand, aber das soll man sehen.
pub(crate) fn im_hintergrund(
    ordner: PathBuf,
    zwischenspeicher: PathBuf,
    nur_liste: bool,
) -> String {
    let zustand = match Zustand::holen(ordner, zwischenspeicher) {
        Ok(z) => z,
        Err(e) => return serde_json::json!({ "fehler": e }).to_string(),
    };
    tauri::async_runtime::block_on(auffrischen(&zustand, nur_liste))
}

/// Der Abgleich selbst -- ohne Instanz oder Kopplung ist er "nichts zu tun"
/// und kein Fehler: Ein Geraet, das nur fuer sich lebt, hat den Auffrischer
/// trotzdem eingeplant, und der soll dann still sein.
pub(crate) async fn auffrischen(zustand: &Arc<Zustand>, nur_liste: bool) -> String {
    let (verbunden, gekoppelt) = {
        let e = zustand.einstellungen.lock().await;
        let gekoppelt = zustand.schluesselablage().lesen().ok().flatten().is_some();
        (e.verbunden(), gekoppelt)
    };
    if !verbunden || !gekoppelt {
        return serde_json::json!({ "getan": false, "grund": "not paired" }).to_string();
    }

    // Wartende openany-Nachrichten gehen mit hinaus (postausgang.rs).
    crate::postausgang::leeren(zustand).await;

    match crate::server_abgleich(zustand, !nur_liste).await {
        Ok(anzeige) => serde_json::json!({
            "getan": true,
            "nur_liste": nur_liste,
            "bericht": anzeige,
        })
        .to_string(),
        Err(e) => serde_json::json!({ "getan": false, "fehler": e }).to_string(),
    }
}

/// Der JNI-Einstieg -- `Auffrischer.laufen(daten, zwischen, nurListe)` in
/// Kotlin. Der Name ist die Adresse: Paket, Klasse, Methode.
///
/// **Kein Absturz darf hier hindurch.** Ein Panic ueber die JNI-Grenze
/// bringt den ganzen Prozess um, und WorkManager plante ihn dann als
/// "gescheitert" neu -- stuendlich, still, mit derselben Ursache.
#[cfg(target_os = "android")]
mod android {
    use jni::objects::{JClass, JString};
    use jni::sys::{jboolean, jstring, JNI_TRUE};
    use jni::JNIEnv;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    fn text(env: &mut JNIEnv, s: &JString) -> String {
        env.get_string(s).map(Into::into).unwrap_or_default()
    }

    #[no_mangle]
    pub extern "system" fn Java_de_openany_app_Auffrischer_laufen(
        mut env: JNIEnv,
        _klasse: JClass,
        daten: JString,
        zwischen: JString,
        nur_liste: jboolean,
    ) -> jstring {
        let daten = text(&mut env, &daten);
        let zwischen = text(&mut env, &zwischen);
        let ergebnis = catch_unwind(AssertUnwindSafe(|| {
            super::im_hintergrund(daten.into(), zwischen.into(), nur_liste == JNI_TRUE)
        }))
        .unwrap_or_else(|_| r#"{"getan":false,"fehler":"Crash in the refresher"}"#.to_string());

        env.new_string(ergebnis)
            .map(|s| s.into_raw())
            .unwrap_or(std::ptr::null_mut())
    }
}
