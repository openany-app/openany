//! Neue Fotos automatisch übernehmen -- openany als Foto-Sicherung
//! (docs/plan-viele-dateien.md, D5). Nur unter Android.
//!
//! **Bemerken statt suchen.** `Fotowaechter.kt` lässt sich vom WorkManager
//! wecken, wenn sich in der Medienbibliothek etwas tut (`ContentUriTrigger`),
//! und ruft dann hier herein -- auch ohne Fenster, in einem frischen Prozess
//! wie der Auffrischer (`hintergrund.rs`). Dazu beim Start der App einmal
//! nachsehen, falls Android etwas verschluckt hat.
//!
//! **Nur Neues.** Beobachtet werden die gewählten Gerätealben, und nur, was
//! ab dem Einschalten aufgenommen wurde. Doppeltes erkennt die Schlange aus
//! D2 (gelöscht bleibt gelöscht); ohne Kopie gilt wie in D3 die gemerkte
//! Wahl aus Frage 6.

#![cfg_attr(not(target_os = "android"), allow(dead_code))]

use crate::einstellungen::FotosAutomatisch;
use crate::Zustand;
use serde::Serialize;
use std::sync::Arc;

fn fehler(e: impl ToString) -> String {
    e.to_string()
}

#[derive(Serialize)]
pub struct Waechterlage {
    an: bool,
    alben: Vec<String>,
    nur_wlan: bool,
    nur_laden: bool,
    verbunden: bool,
    /// Die gemerkte Wahl aus Frage 6; `None` = noch nicht gefragt.
    ohne_kopie: Option<bool>,
}

#[tauri::command]
pub async fn fotos_automatisch_stand(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Waechterlage, String> {
    let e = zustand.einstellungen.lock().await;
    let a = e.fotos_automatisch.clone().unwrap_or_default();
    Ok(Waechterlage {
        an: e.fotos_automatisch.is_some(),
        alben: a.alben,
        nur_wlan: a.nur_wlan,
        nur_laden: a.nur_laden,
        verbunden: e.verbunden(),
        ohne_kopie: e.fotos_ohne_kopie.filter(|_| e.verbunden()),
    })
}

/// Ein- oder ausschalten. Beim Einschalten zählt ab jetzt; wer nur die Alben
/// oder Bedingungen ändert, behält seinen Zeitpunkt.
#[tauri::command]
pub async fn fotos_automatisch_setzen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    an: bool,
    alben: Vec<String>,
    nur_wlan: bool,
    nur_laden: bool,
    ohne_kopie: bool,
) -> Result<(), String> {
    if an && alben.is_empty() {
        return Err("Please choose at least one album.".into());
    }
    {
        let mut e = zustand.einstellungen.lock().await;
        e.fotos_automatisch = an.then(|| FotosAutomatisch {
            seit: e
                .fotos_automatisch
                .as_ref()
                .map_or_else(|| chrono::Utc::now().timestamp_millis(), |a| a.seit),
            alben,
            nur_wlan,
            nur_laden,
        });
        if an && e.verbunden() {
            e.fotos_ohne_kopie = Some(ohne_kopie);
        }
        e.schreiben(&zustand.einstellungspfad()).map_err(fehler)?;
    }
    crate::medien::waechter_einplanen(an, nur_wlan, nur_laden);
    Ok(())
}

/// Was seit dem Einschalten in den gewählten Alben neu ist -- in die
/// Schlange. Gibt die Zahl der neuen Einträge zurück; aus: 0.
async fn neue_einreihen(zustand: &Zustand) -> Result<Vec<openany_store::NeueUebernahme>, String> {
    let (auto, ohne_kopie) = {
        let e = zustand.einstellungen.lock().await;
        let Some(a) = e.fotos_automatisch.clone() else {
            return Ok(Vec::new());
        };
        (a, e.fotos_ohne_kopie.unwrap_or(false) && e.verbunden())
    };
    crate::medien::vorbereiten(zustand, &auto.alben, auto.seit, ohne_kopie, auto.nur_wlan).await
}

/// Beim Start der App: den Wächter (neu) einplanen und nachsehen, ob etwas
/// liegen geblieben ist.
pub(crate) fn beim_start(app: &tauri::AppHandle, zustand: &Arc<Zustand>) {
    let (app, z) = (app.clone(), zustand.clone());
    tauri::async_runtime::spawn(async move {
        let auto = z.einstellungen.lock().await.fotos_automatisch.clone();
        let Some(a) = auto else {
            return;
        };
        crate::medien::waechter_einplanen(true, a.nur_wlan, a.nur_laden);
        match neue_einreihen(&z).await {
            Ok(neue) if !neue.is_empty() => {
                let _ = crate::uebernahme::einreihen_neue(&app, &z, &neue).await;
            }
            Ok(_) => {}
            Err(e) => eprintln!("Photo watcher: {e}"),
        }
    });
}

/// Für den stündlichen Auffrischer (`hintergrund.rs`): nachholen, was der
/// Wächter verpasst hat -- etwa wenn die App aus der Liste der letzten Apps
/// gewischt wurde und der WorkManager dabei ein Ereignis verwarf. Hält sich
/// an „nur im WLAN“; mit „nur beim Laden“ bleibt es beim Wächter, der den
/// Ladezustand kennt.
pub(crate) async fn nachholen(zustand: &Arc<Zustand>, gemessen: bool) -> usize {
    let auto = zustand.einstellungen.lock().await.fotos_automatisch.clone();
    let Some(a) = auto else {
        return 0;
    };
    if a.nur_laden || (a.nur_wlan && gemessen) {
        return 0;
    }
    let Ok(neue) = neue_einreihen(zustand).await else {
        return 0;
    };
    if neue.is_empty()
        || zustand
            .speicher
            .lock()
            .await
            .uebernahmen_einreihen(&neue)
            .is_err()
    {
        return 0;
    }
    crate::uebernahme::ohne_fenster_abarbeiten(zustand).await
}

/// Ein Lauf ohne Fenster: einreihen, abarbeiten, danach abgleichen, damit
/// neue Fotos gleich auf openany.de kommen.
fn im_hintergrund(ordner: std::path::PathBuf, zwischenspeicher: std::path::PathBuf) -> String {
    let zustand = match Zustand::holen(ordner, zwischenspeicher) {
        Ok(z) => z,
        Err(e) => return serde_json::json!({ "fehler": e }).to_string(),
    };
    tauri::async_runtime::block_on(async {
        let neue = match neue_einreihen(&zustand).await {
            Ok(n) => n,
            Err(e) => return serde_json::json!({ "fehler": e }).to_string(),
        };
        let eingereiht = match zustand.speicher.lock().await.uebernahmen_einreihen(&neue) {
            Ok(e) => e.neu,
            Err(e) => return serde_json::json!({ "fehler": e.to_string() }).to_string(),
        };
        let getan = crate::uebernahme::ohne_fenster_abarbeiten(&zustand).await;
        let abgleich = if getan > 0 {
            crate::hintergrund::auffrischen(&zustand, crate::medien::gemessen()).await
        } else {
            String::new()
        };
        serde_json::json!({ "eingereiht": eingereiht, "uebernommen": getan, "abgleich": abgleich })
            .to_string()
    })
}

#[cfg(target_os = "android")]
mod android {
    use jni::objects::{JClass, JString};
    use jni::sys::jstring;
    use jni::JNIEnv;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    fn text(env: &mut JNIEnv, s: &JString) -> String {
        env.get_string(s).map(Into::into).unwrap_or_default()
    }

    /// `Fotowaechter.laufen(daten, zwischen)` -- kein Absturz darf über die
    /// JNI-Grenze (siehe `hintergrund.rs`).
    #[no_mangle]
    pub extern "system" fn Java_de_openany_app_Fotowaechter_laufen(
        mut env: JNIEnv,
        _klasse: JClass,
        daten: JString,
        zwischen: JString,
    ) -> jstring {
        let daten = text(&mut env, &daten);
        let zwischen = text(&mut env, &zwischen);
        let ergebnis = catch_unwind(AssertUnwindSafe(|| {
            super::im_hintergrund(daten.into(), zwischen.into())
        }))
        .unwrap_or_else(|_| r#"{"fehler":"Crash in the photo watcher"}"#.to_string());
        env.new_string(ergebnis)
            .map(|s| s.into_raw())
            .unwrap_or(std::ptr::null_mut())
    }
}
