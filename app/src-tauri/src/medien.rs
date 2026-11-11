//! Fotos vom Gerät übernehmen (docs/plan-viele-dateien.md, D3) -- nur unter
//! Android.
//!
//! Die Medienbibliothek liest `Medien.kt`; hier wird über JNI gefragt, was es
//! gibt, ausgewählt und eingereiht. Die Oberfläche nennt nur die Auswahl
//! (alle, diese Gerätealben, ab einem Datum) -- nie eine Adresse. Geöffnet
//! wird jedes Foto erst im Lauf der Schlange (`uebernahme.rs`), als
//! Dateideskriptor.
//!
//! **Gerätealben werden gleichnamige Alben** in der Mediathek
//! (Entscheidung 2): „Kamera" nach „Kamera", „WhatsApp Images" nach
//! „WhatsApp Images". Was schon da ist, wird weiterverwendet.
//!
//! **Ohne Kopie** (Entscheidung 1, Frage 6): Ist das Gerät mit openany.de
//! verbunden und so gewählt, gehen die Alben auf den Server; hier bleiben
//! Zeile und Vorschau, das Original liest der Abgleich aus der Quelle
//! (`openany_store::Aussen`). Die Wahl wird einmal getroffen und gemerkt.

#![cfg_attr(not(target_os = "android"), allow(dead_code))]

use crate::Zustand;
use openany_store::{Eingereiht, NeueUebernahme, Protokoll};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Default)]
pub struct Album {
    name: String,
    anzahl: u64,
    bytes: u64,
}

/// Was die Medienbibliothek hat, für die Auswahl (Skizze B).
#[derive(Serialize, Deserialize, Default)]
pub struct Uebersicht {
    /// `alle`, `auswahl` (Android 14: nur ausgewählte Fotos) oder `keine`.
    #[serde(default)]
    erlaubt: String,
    #[serde(default)]
    anzahl: u64,
    #[serde(default)]
    bytes: u64,
    #[serde(default)]
    alben: Vec<Album>,
    /// Freier Platz auf dem Gerät, abzüglich der Reserve.
    #[serde(default)]
    frei: u64,
    /// Mit openany.de verbunden -- sonst gibt es nur die Kopie.
    #[serde(default)]
    verbunden: bool,
    /// Die gemerkte Wahl (Frage 6); `None` = noch nicht gefragt.
    #[serde(default)]
    ohne_kopie: Option<bool>,
    /// Freier Platz im Konto auf openany.de, wenn er sich erfragen ließ.
    #[serde(default)]
    server_frei: Option<u64>,
}

/// Ein Foto oder Video, wie `Medien.liste` es nennt.
#[derive(Deserialize)]
struct Eintrag {
    uri: String,
    kennung: String,
    name: String,
    mime: String,
    groesse: u64,
    mtime: i64,
    album: String,
}

/// Was übernommen werden soll.
#[derive(Deserialize)]
pub struct Auswahl {
    /// Leer = alle Gerätealben.
    #[serde(default)]
    alben: Vec<String>,
    /// Nur, was ab diesem Zeitpunkt aufgenommen wurde (ms seit 1970).
    #[serde(default)]
    ab: Option<i64>,
    /// Ohne Kopie (Frage 6) -- wird gemerkt.
    #[serde(default)]
    ohne_kopie: bool,
    /// Ohne Kopie: nur im WLAN hinaustragen.
    #[serde(default)]
    nur_wlan: bool,
}

fn fehler(e: impl ToString) -> String {
    e.to_string()
}

/// Die Übersicht der Medienbibliothek.
#[tauri::command]
pub async fn medien_uebersicht(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Uebersicht, String> {
    let roh = tauri::async_runtime::spawn_blocking(android::uebersicht)
        .await
        .map_err(fehler)?
        .ok_or("Only available in the Android app.")?;
    let mut u: Uebersicht = serde_json::from_str(&roh).map_err(fehler)?;
    u.frei = crate::uebernahme::frei_fuer_uebernahme(&zustand);
    let e = zustand.einstellungen.lock().await.clone();
    u.verbunden = e.verbunden();
    u.ohne_kopie = e.fotos_ohne_kopie.filter(|_| u.verbunden);
    if u.verbunden {
        u.server_frei = server_frei(&zustand).await;
    }
    Ok(u)
}

/// Was im Konto auf openany.de noch frei ist -- höchstens ein paar Sekunden
/// gefragt; ohne Netz eben nicht.
async fn server_frei(zustand: &Arc<Zustand>) -> Option<u64> {
    let g = crate::server_gegenstelle(zustand).await?;
    let b = tokio::time::timeout(std::time::Duration::from_secs(5), g.client().belegung())
        .await
        .ok()?
        .ok()?;
    b.kontingent.map(|k| k.saturating_sub(b.kontingent_belegt))
}

/// Die Auswahl in die Schlange: je Gerätealbum ein gleichnamiges Album.
#[tauri::command]
pub async fn medien_uebernehmen(
    app: tauri::AppHandle,
    zustand: tauri::State<'_, Arc<Zustand>>,
    auswahl: Auswahl,
) -> Result<Eingereiht, String> {
    // Die Wahl merken (Frage 6). Ohne Verbindung zu openany.de gibt es nur
    // die Kopie.
    let ohne_kopie = {
        let mut e = zustand.einstellungen.lock().await;
        let ohne = auswahl.ohne_kopie && e.verbunden();
        if e.verbunden() && e.fotos_ohne_kopie != Some(ohne) {
            e.fotos_ohne_kopie = Some(ohne);
            e.schreiben(&zustand.einstellungspfad()).map_err(fehler)?;
        }
        ohne
    };
    let neue = vorbereiten(
        &zustand,
        &auswahl.alben,
        auswahl.ab.unwrap_or(0),
        ohne_kopie,
        auswahl.nur_wlan,
    )
    .await?;
    crate::uebernahme::einreihen_neue(&app, &zustand, &neue).await
}

/// Die Fotos und Videos der Gerätealben `alben` (leer = alle) ab `ab` (ms)
/// als Einträge für die Schlange -- samt gleichnamigen Alben (Entscheidung 2),
/// ohne Kopie mit dem Schalter „auf den Server“, und nach der Prüfung, ob der
/// Platz für die Kopien reicht. Für den Dialog und den Fotowächter (D5).
pub(crate) async fn vorbereiten(
    zustand: &Zustand,
    alben: &[String],
    ab: i64,
    ohne_kopie: bool,
    nur_wlan: bool,
) -> Result<Vec<NeueUebernahme>, String> {
    let alben = serde_json::to_string(alben).map_err(fehler)?;
    let roh = tauri::async_runtime::spawn_blocking(move || android::liste(&alben, ab))
        .await
        .map_err(fehler)?
        .ok_or("Only available in the Android app.")?;
    let eintraege: Vec<Eintrag> = serde_json::from_str(&roh).map_err(fehler)?;

    let mut neue: Vec<NeueUebernahme> = eintraege
        .iter()
        .map(|e| NeueUebernahme {
            quelle: e.uri.clone(),
            quelle_schluessel: format!("medien|{}|{}", e.kennung, e.mtime),
            zone: "galerie".into(),
            ziel: None,
            name: e.name.clone(),
            mime: e.mime.clone(),
            groesse: e.groesse,
            // Was in keinem Album liegt, kommt in keinen Abgleich -- das
            // bleibt als Kopie hier.
            ohne_kopie: ohne_kopie && !e.album.is_empty(),
            nur_wlan,
        })
        .collect();
    // Ohne Kopie braucht es hier nur Platz für die Datei, die gerade dran
    // ist -- das prüft der Lauf je Datei.
    let kopien: Vec<NeueUebernahme> = neue.iter().filter(|n| !n.ohne_kopie).cloned().collect();
    crate::uebernahme::platz_pruefen(zustand, &kopien)?;

    // Erst nach der Platzprüfung die Alben: Reicht er nicht, bleiben keine
    // leeren zurück.
    let mut album_von: HashMap<&str, String> = HashMap::new();
    let mut neu_angelegt = Vec::new();
    {
        let speicher = zustand.speicher.lock().await;
        for e in &eintraege {
            if e.album.is_empty() || album_von.contains_key(e.album.as_str()) {
                continue;
            }
            let (uuid, neu) = crate::ablegen::behaelter(&speicher, "galerie", None, &e.album)?;
            if neu {
                neu_angelegt.push(uuid.clone());
            }
            // Ohne Kopie gehört das Album auf den Server, sonst bliebe das
            // Original nur in der Quelle.
            if ohne_kopie {
                speicher
                    .auf_server_setzen(&openany_client::Art::Album, &uuid, true, Protokoll::Merken)
                    .map_err(fehler)?;
            }
            album_von.insert(&e.album, uuid);
        }
    }
    for uuid in &neu_angelegt {
        crate::dateibefehle::neuen_behaelter_waehlen(zustand, uuid, None).await;
    }
    for (n, e) in neue.iter_mut().zip(&eintraege) {
        n.ziel = album_von.get(e.album.as_str()).cloned();
    }
    Ok(neue)
}

/// Ein Foto aus der Medienbibliothek zum Lesen öffnen.
pub(crate) fn oeffnen(adresse: &str) -> Option<std::fs::File> {
    android::oeffnen(adresse)
}

/// Ist die Verbindung gemessen (Mobilfunk, Hotspot)? Dann wartet, was nur im
/// WLAN hinaus soll. Auf dem Schreibtisch nie.
pub(crate) fn gemessen() -> bool {
    android::gemessen()
}

/// Ein ohne Kopie übernommenes Original aus seiner Quelle in eine
/// Zwischendatei neben den Inhalten -- geprüft gegen den Abdruck. Wer sie
/// bekommt, räumt sie weg (oder übernimmt sie in den Inhaltsspeicher).
/// `None`, wenn es keine Quelle gibt oder sie nicht mehr dieselbe ist; dann
/// ist sie auch vergessen.
pub(crate) async fn aussen_zwischenkopie(
    zustand: &Zustand,
    abdruck: &str,
) -> Result<Option<std::path::PathBuf>, String> {
    let Some(a) = zustand
        .speicher
        .lock()
        .await
        .aussen(abdruck)
        .map_err(fehler)?
    else {
        return Ok(None);
    };
    let ziel = zustand.inhalte.zwischenpfad().map_err(fehler)?;
    let quelle = a.quelle.clone();
    let z = ziel.clone();
    let gelesen = tauri::async_runtime::spawn_blocking(move || -> Option<String> {
        use sha2::{Digest, Sha256};
        use std::io::{Read, Write};
        let mut ein = if quelle.starts_with("content://") {
            oeffnen(&quelle)?
        } else {
            std::fs::File::open(&quelle).ok()?
        };
        let mut aus = std::fs::File::create(&z).ok()?;
        let mut rechner = Sha256::new();
        let mut puffer = vec![0u8; 1024 * 1024];
        loop {
            let n = ein.read(&mut puffer).ok()?;
            if n == 0 {
                break;
            }
            rechner.update(&puffer[..n]);
            aus.write_all(&puffer[..n]).ok()?;
        }
        aus.sync_all().ok()?;
        Some(
            rechner
                .finalize()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect(),
        )
    })
    .await
    .map_err(fehler)?;
    if gelesen.as_deref() == Some(abdruck) {
        return Ok(Some(ziel));
    }
    let _ = std::fs::remove_file(&ziel);
    zustand
        .speicher
        .lock()
        .await
        .aussen_vergessen(abdruck)
        .map_err(fehler)?;
    Ok(None)
}

/// Den Fotowächter (D5, `Fotowaechter.kt`) einplanen oder abbestellen.
pub(crate) fn waechter_einplanen(an: bool, nur_wlan: bool, nur_laden: bool) {
    android::waechter_einplanen(an, nur_wlan, nur_laden);
}

/// Den Vordergrunddienst der Übernahme starten (`Uebernahmedienst.kt`).
pub(crate) fn dienst_starten() {
    android::dienst_starten();
}

#[cfg(not(target_os = "android"))]
mod android {
    pub(super) fn uebersicht() -> Option<String> {
        None
    }
    pub(super) fn liste(_alben: &str, _ab: i64) -> Option<String> {
        None
    }
    pub(super) fn oeffnen(_adresse: &str) -> Option<std::fs::File> {
        None
    }
    pub(super) fn dienst_starten() {}
    pub(super) fn waechter_einplanen(_an: bool, _nur_wlan: bool, _nur_laden: bool) {}
    pub(super) fn gemessen() -> bool {
        false
    }
}

#[cfg(target_os = "android")]
mod android {
    use jni::objects::{GlobalRef, JClass, JObject, JString, JValue};
    use jni::{JNIEnv, JavaVM};
    use std::os::fd::FromRawFd;
    use std::sync::OnceLock;

    static MEDIEN: OnceLock<(JavaVM, GlobalRef)> = OnceLock::new();

    #[no_mangle]
    pub extern "system" fn Java_de_openany_app_Medien_anmelden(env: JNIEnv, klasse: JClass) {
        if MEDIEN.get().is_some() {
            return;
        }
        if let (Ok(vm), Ok(global)) = (env.get_java_vm(), env.new_global_ref(&klasse)) {
            let _ = MEDIEN.set((vm, global));
        }
    }

    /// Eine statische Methode von `Medien` rufen; räumt eine Ausnahme ab.
    fn rufen<T>(tun: impl FnOnce(&mut JNIEnv, &JClass) -> jni::errors::Result<T>) -> Option<T> {
        let (vm, klasse) = MEDIEN.get()?;
        let mut env = vm.attach_current_thread_permanently().ok()?;
        let ergebnis = env.with_local_frame(16, |env| tun(env, klasse.as_obj().into()));
        match ergebnis {
            Ok(r) => Some(r),
            Err(_) => {
                if env.exception_check().unwrap_or(false) {
                    let _ = env.exception_clear();
                }
                None
            }
        }
    }

    fn text(env: &mut JNIEnv, o: JObject) -> jni::errors::Result<String> {
        Ok(env.get_string(&JString::from(o))?.into())
    }

    pub(super) fn uebersicht() -> Option<String> {
        rufen(|env, k| {
            let o = env
                .call_static_method(k, "uebersicht", "()Ljava/lang/String;", &[])?
                .l()?;
            text(env, o)
        })
    }

    pub(super) fn liste(alben: &str, ab: i64) -> Option<String> {
        rufen(|env, k| {
            let a = env.new_string(alben)?;
            let o = env
                .call_static_method(
                    k,
                    "liste",
                    "(Ljava/lang/String;J)Ljava/lang/String;",
                    &[JValue::Object(&JObject::from(a)), JValue::Long(ab)],
                )?
                .l()?;
            text(env, o)
        })
    }

    pub(super) fn gemessen() -> bool {
        rufen(|env, k| env.call_static_method(k, "gemessen", "()Z", &[])?.z()).unwrap_or(false)
    }

    pub(super) fn dienst_starten() {
        let _ = rufen(|env, k| env.call_static_method(k, "dienstStarten", "()V", &[])?.v());
    }

    pub(super) fn waechter_einplanen(an: bool, nur_wlan: bool, nur_laden: bool) {
        let _ = rufen(|env, k| {
            env.call_static_method(
                k,
                "waechterEinplanen",
                "(ZZZ)V",
                &[
                    JValue::Bool(an as u8),
                    JValue::Bool(nur_wlan as u8),
                    JValue::Bool(nur_laden as u8),
                ],
            )?
            .v()
        });
    }

    pub(super) fn oeffnen(adresse: &str) -> Option<std::fs::File> {
        let fd = rufen(|env, k| {
            let a = env.new_string(adresse)?;
            env.call_static_method(
                k,
                "oeffnen",
                "(Ljava/lang/String;)I",
                &[JValue::Object(&JObject::from(a))],
            )?
            .i()
        })?;
        // SAFETY: `detachFd` hat den Deskriptor abgegeben; ab hier gehört er
        // diesem File, das ihn beim Fallenlassen schließt.
        (fd >= 0).then(|| unsafe { std::fs::File::from_raw_fd(fd) })
    }
}
