//! Kontakte im Adressbuch des Telefons -- wie DAVx5, nur Ende-zu-Ende
//! (docs/plan-kalender-kontakte-im-system.md, K4). Nur unter Android.
//!
//! Dasselbe Muster wie der Kalender (`systemkalender.rs`): Hier steht der
//! Soll-Stand, `Systemkontakte.kt` trägt ihn unter dem Konto „openany“ ein.
//! Jeder Kontakt wird ein `RawContact`; Android führt ihn mit gleichnamigen
//! Kontakten anderer Konten zusammen. Danach zeigt jede App den Namen --
//! auch beim Anruf.
//!
//! **Was hinübergeht:** Name, Telefon, E-Mail, Anschrift, Firma, Geburtstag,
//! Webseite, Spitzname und das Foto. Kennungen zum Funken (Matrix,
//! Meshtastic, openany) kennt das Adressbuch des Telefons nicht; sie bleiben
//! nur in openany -- und unberührt, wenn der Kontakt dort geändert wird.
//!
//! **Zurück:** wie K2 (`DIRTY`/`DELETED`). Das Foto geht nur hinüber: Ein im
//! Telefon geändertes Foto setzt der nächste Lauf zurück.

#![cfg_attr(not(target_os = "android"), allow(dead_code))]

use crate::Zustand;
use openany_store::{Kontakt, Protokoll, Weg};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

fn fehler(e: impl ToString) -> String {
    e.to_string()
}

/// Die Arten, die das Adressbuch des Telefons kennt.
const ARTEN: [&str; 7] = [
    "phone",
    "email",
    "address",
    "company",
    "birthday",
    "vcard:url",
    "vcard:nickname",
];

/// Ein Weg, wie er ins Telefon kommt: die Art, eine der Sorten
/// `mobile`/`home`/`work`/`other`/`custom` und bei `custom` die Beschriftung.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct Sysweg {
    art: String,
    sorte: String,
    #[serde(default)]
    beschriftung: String,
    wert: String,
}

#[derive(Serialize, Debug, PartialEq)]
struct Syskontakt {
    /// `SOURCE_ID`: die uuid des Kontakts.
    schluessel: String,
    name: String,
    wege: Vec<Sysweg>,
    /// Pfad zu den Bytes des Fotos (im Zwischenspeicher); leer = keines.
    foto: String,
    /// Gleich wie im Telefon (`SYNC1`) = nichts zu tun.
    abdruck: String,
}

/// Die Beschriftung als Sorte. Was nicht passt, bleibt eigene Beschriftung.
fn sorte(beschriftung: &str) -> (&'static str, String) {
    let b = beschriftung.trim();
    match b.to_lowercase().as_str() {
        "" => ("other", String::new()),
        "cell" | "mobile" | "mobil" | "handy" => ("mobile", String::new()),
        "home" | "privat" | "zuhause" => ("home", String::new()),
        "work" | "arbeit" | "dienstlich" | "büro" => ("work", String::new()),
        "other" | "sonstige" | "andere" => ("other", String::new()),
        _ => ("custom", b.to_string()),
    }
}

/// Eine Sorte aus dem Telefon als Beschriftung -- so, wie sie aus einer
/// vCard käme (`TYPE=CELL`).
fn beschriftung_von(sorte: &str, eigene: &str) -> String {
    match sorte {
        "mobile" => "CELL".into(),
        "home" => "HOME".into(),
        "work" => "WORK".into(),
        "custom" => eigene.trim().to_string(),
        _ => String::new(),
    }
}

fn wege_ins_telefon(k: &Kontakt) -> Vec<Sysweg> {
    k.wege
        .iter()
        .filter(|w| ARTEN.contains(&w.art.as_str()) && !w.wert.trim().is_empty())
        .map(|w| {
            let (s, b) = sorte(w.beschriftung.as_deref().unwrap_or_default());
            Sysweg {
                art: w.art.clone(),
                sorte: s.into(),
                beschriftung: b,
                wert: w.wert.trim().to_string(),
            }
        })
        .collect()
}

fn abdruck_von(text: &str) -> String {
    openany_store::fotoabdruck(text.as_bytes())[..16].to_string()
}

/// Was im Adressbuch des Telefons stehen soll. Fotos legt es als Dateien in
/// den Zwischenspeicher -- über JNI reisen nur Pfade.
async fn stand(zustand: &Zustand) -> Result<Vec<Syskontakt>, String> {
    let ordner = zustand.zwischenspeicher.join("kontaktfotos");
    let speicher = zustand.speicher.lock().await;
    let mut aus = Vec::new();
    for k in speicher.kontakte().map_err(fehler)? {
        let wege = wege_ins_telefon(&k);
        let foto = match &k.foto {
            Some(a) => {
                let pfad = ordner.join(format!("{a}.bild"));
                if !pfad.exists() {
                    if let Some(bytes) = speicher.kontaktfoto(&k.uuid).map_err(fehler)? {
                        let _ = std::fs::create_dir_all(&ordner);
                        let _ = std::fs::write(&pfad, bytes);
                    }
                }
                if pfad.exists() {
                    pfad.to_string_lossy().into_owned()
                } else {
                    String::new()
                }
            }
            None => String::new(),
        };
        let abdruck = abdruck_von(&format!(
            "{}\n{}\n{}",
            k.anzeigename,
            serde_json::to_string(&wege).unwrap_or_default(),
            k.foto.as_deref().unwrap_or_default()
        ));
        aus.push(Syskontakt {
            schluessel: k.uuid,
            name: k.anzeigename,
            wege,
            foto,
            abdruck,
        });
    }
    Ok(aus)
}

/// Eine Änderung aus dem Adressbuch des Telefons (`Systemkontakte.aenderungen`).
#[derive(Deserialize, Debug)]
struct Aenderung {
    id: i64,
    schluessel: Option<String>,
    geloescht: bool,
    #[serde(default)]
    name: String,
    #[serde(default)]
    wege: Vec<Sysweg>,
}

#[derive(Serialize)]
struct Erledigt {
    id: i64,
    schluessel: Option<String>,
    loeschen: bool,
}

/// Die Wege nach einer Änderung im Telefon: Was das Telefon nicht kennt,
/// bleibt; die übrigen kommen aus dem Telefon. Eine Beschriftung, die zur
/// selben Sorte gehört, bleibt, wie sie war -- sonst würde aus „Handy“
/// beim ersten Hin und Her „CELL“.
fn wege_zusammen(alt: &[Weg], neu: &[Sysweg]) -> Vec<Weg> {
    let mut aus: Vec<Weg> = alt
        .iter()
        .filter(|w| !ARTEN.contains(&w.art.as_str()))
        .cloned()
        .collect();
    for n in neu {
        if !ARTEN.contains(&n.art.as_str()) || n.wert.trim().is_empty() {
            continue;
        }
        let bisher = alt.iter().find(|w| {
            w.art == n.art
                && w.wert.trim() == n.wert.trim()
                && sorte(w.beschriftung.as_deref().unwrap_or_default()).0 == n.sorte
                && (n.sorte != "custom"
                    || w.beschriftung.as_deref().unwrap_or_default().trim()
                        == n.beschriftung.trim())
        });
        let beschriftung = match bisher {
            Some(w) => w.beschriftung.clone(),
            None => Some(beschriftung_von(&n.sorte, &n.beschriftung)).filter(|b| !b.is_empty()),
        };
        aus.push(Weg {
            art: n.art.clone(),
            beschriftung,
            wert: n.wert.trim().to_string(),
        });
    }
    aus
}

/// Gleich bis auf die Reihenfolge? Dann nichts schreiben.
fn gleich(a: &[Weg], b: &[Weg]) -> bool {
    let mut x: Vec<_> = a
        .iter()
        .map(|w| (&w.art, &w.beschriftung, &w.wert))
        .collect();
    let mut y: Vec<_> = b
        .iter()
        .map(|w| (&w.art, &w.beschriftung, &w.wert))
        .collect();
    x.sort();
    y.sort();
    x == y
}

async fn uebernehmen(zustand: &Arc<Zustand>, a: &Aenderung) -> Result<Erledigt, String> {
    let speicher = zustand.speicher.lock().await;
    if a.geloescht {
        if let Some(u) = a.schluessel.as_deref() {
            speicher
                .papierkorb(&openany_client::Art::Kontakt, u, Protokoll::Merken)
                .map_err(fehler)?;
        }
        return Ok(Erledigt {
            id: a.id,
            schluessel: a.schluessel.clone(),
            loeschen: true,
        });
    }
    let vorhanden = match a.schluessel.as_deref() {
        Some(u) => speicher.kontakt(u).map_err(fehler)?,
        None => None,
    };
    let mut k = vorhanden.clone().unwrap_or_else(|| Kontakt {
        uuid: uuid::Uuid::new_v4().to_string(),
        ..Default::default()
    });
    let name = a.name.trim();
    if !name.is_empty() {
        k.anzeigename = name.to_string();
    }
    let wege = wege_zusammen(&k.wege, &a.wege);
    let geaendert = vorhanden.is_none()
        || k.anzeigename
            != vorhanden
                .as_ref()
                .map(|v| v.anzeigename.as_str())
                .unwrap_or_default()
        || !gleich(&k.wege, &wege);
    if geaendert && (!k.anzeigename.is_empty() || !wege.is_empty()) {
        k.wege = wege;
        k.papierkorb_at = None;
        k.geaendert_at = String::new();
        speicher
            .kontakt_schreiben(&k, Protokoll::Merken)
            .map_err(fehler)?;
    }
    Ok(Erledigt {
        id: a.id,
        schluessel: Some(k.uuid),
        loeschen: false,
    })
}

async fn zurueckholen(zustand: &Arc<Zustand>) -> Result<usize, String> {
    let roh = tauri::async_runtime::spawn_blocking(android::aenderungen)
        .await
        .map_err(fehler)?
        .unwrap_or_default();
    let liste: Vec<Aenderung> = serde_json::from_str(&roh).unwrap_or_default();
    if liste.is_empty() {
        return Ok(0);
    }
    let mut erledigt = Vec::new();
    for a in &liste {
        match uebernehmen(zustand, a).await {
            Ok(e) => erledigt.push(e),
            Err(e) => eprintln!("Phone contacts, change {}: {e}", a.id),
        }
    }
    let text = serde_json::to_string(&erledigt).map_err(fehler)?;
    tauri::async_runtime::spawn_blocking(move || android::bestaetigen(&text))
        .await
        .map_err(fehler)?;
    *ZULETZT.lock().unwrap_or_else(|e| e.into_inner()) = None;
    Ok(erledigt.len())
}

static ZULETZT: Mutex<Option<String>> = Mutex::new(None);
static SPERRE: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Eintragen, wenn eingeschaltet und anders als zuletzt (`immer`: auch wenn
/// gleich). Ein Lauf zur Zeit, wie beim Kalender.
pub(crate) async fn spiegeln(zustand: &Arc<Zustand>, immer: bool) -> Result<(), String> {
    let _sperre = SPERRE.lock().await;
    if !zustand.einstellungen.lock().await.systemkontakte {
        return Ok(());
    }
    if let Err(e) = zurueckholen(zustand).await {
        eprintln!("Phone contacts, changes: {e}");
    }
    let soll = stand(zustand).await?;
    let text = serde_json::to_string(&soll).map_err(fehler)?;
    let a = abdruck_von(&text);
    if !immer && ZULETZT.lock().unwrap_or_else(|e| e.into_inner()).as_deref() == Some(&a) {
        return Ok(());
    }
    let ergebnis = tauri::async_runtime::spawn_blocking(move || android::spiegeln(&text))
        .await
        .map_err(fehler)?;
    match ergebnis {
        Some(e) if e.is_empty() => {
            *ZULETZT.lock().unwrap_or_else(|e| e.into_inner()) = Some(a);
            Ok(())
        }
        Some(e) => Err(e),
        None => Err("The phone's address book is not reachable.".into()),
    }
}

#[derive(Serialize)]
pub struct Systemkontaktelage {
    an: bool,
    moeglich: bool,
}

#[tauri::command]
pub async fn systemkontakte_stand(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Systemkontaktelage, String> {
    Ok(Systemkontaktelage {
        an: zustand.einstellungen.lock().await.systemkontakte,
        moeglich: cfg!(target_os = "android"),
    })
}

/// Ein- oder ausschalten. Aus: die Kontakte des Kontos fort -- das Konto
/// selbst nur, wenn auch der Kalender aus ist.
#[tauri::command]
pub async fn systemkontakte_setzen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    an: bool,
) -> Result<(), String> {
    let kalender = {
        let mut e = zustand.einstellungen.lock().await;
        e.systemkontakte = an;
        e.schreiben(&zustand.einstellungspfad()).map_err(fehler)?;
        e.systemkalender.is_some()
    };
    *ZULETZT.lock().unwrap_or_else(|e| e.into_inner()) = None;
    if an {
        spiegeln(&zustand, true).await
    } else {
        tauri::async_runtime::spawn_blocking(move || {
            if kalender {
                android::leeren();
            } else {
                crate::systemkalender::konto_fort();
            }
        })
        .await
        .map_err(fehler)?;
        Ok(())
    }
}

/// Beim Start: eintragen, dann leise im Takt nachsehen.
pub(crate) fn beim_start(zustand: &Arc<Zustand>) {
    let z = zustand.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            if let Err(e) = spiegeln(&z, false).await {
                eprintln!("Phone contacts: {e}");
            }
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        }
    });
}

#[cfg(not(target_os = "android"))]
mod android {
    pub(super) fn spiegeln(_text: &str) -> Option<String> {
        None
    }
    pub(super) fn leeren() {}
    pub(super) fn aenderungen() -> Option<String> {
        None
    }
    pub(super) fn bestaetigen(_text: &str) {}
}

#[cfg(target_os = "android")]
mod android {
    use jni::objects::{GlobalRef, JClass, JObject, JString, JValue};
    use jni::sys::jstring;
    use jni::{JNIEnv, JavaVM};
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::sync::OnceLock;

    static KLASSE: OnceLock<(JavaVM, GlobalRef)> = OnceLock::new();

    #[no_mangle]
    pub extern "system" fn Java_de_openany_app_Systemkontakte_anmelden(
        env: JNIEnv,
        klasse: JClass,
    ) {
        if KLASSE.get().is_some() {
            return;
        }
        if let (Ok(vm), Ok(global)) = (env.get_java_vm(), env.new_global_ref(&klasse)) {
            let _ = KLASSE.set((vm, global));
        }
    }

    fn rufen<T>(tun: impl FnOnce(&mut JNIEnv, &JClass) -> jni::errors::Result<T>) -> Option<T> {
        let (vm, klasse) = KLASSE.get()?;
        let mut env = vm.attach_current_thread_permanently().ok()?;
        match env.with_local_frame(16, |env| tun(env, klasse.as_obj().into())) {
            Ok(r) => Some(r),
            Err(_) => {
                if env.exception_check().unwrap_or(false) {
                    let _ = env.exception_clear();
                }
                None
            }
        }
    }

    pub(super) fn spiegeln(text: &str) -> Option<String> {
        rufen(|env, k| {
            let t = env.new_string(text)?;
            let o = env
                .call_static_method(
                    k,
                    "spiegeln",
                    "(Ljava/lang/String;)Ljava/lang/String;",
                    &[JValue::Object(&JObject::from(t))],
                )?
                .l()?;
            Ok(env.get_string(&JString::from(o))?.into())
        })
    }

    pub(super) fn leeren() {
        let _ = rufen(|env, k| env.call_static_method(k, "leeren", "()V", &[])?.v());
    }

    pub(super) fn aenderungen() -> Option<String> {
        rufen(|env, k| {
            let o = env
                .call_static_method(k, "aenderungen", "()Ljava/lang/String;", &[])?
                .l()?;
            Ok(env.get_string(&JString::from(o))?.into())
        })
    }

    pub(super) fn bestaetigen(text: &str) {
        let _ = rufen(|env, k| {
            let t = env.new_string(text)?;
            env.call_static_method(
                k,
                "bestaetigen",
                "(Ljava/lang/String;)V",
                &[JValue::Object(&JObject::from(t))],
            )?
            .v()
        });
    }

    /// Der SyncAdapter (`Systemkontakte.kt`): eintragen, auch ohne Fenster.
    #[no_mangle]
    pub extern "system" fn Java_de_openany_app_Systemkontakte_laufen(
        mut env: JNIEnv,
        _k: JClass,
        daten: JString,
        zwischen: JString,
    ) -> jstring {
        let daten: String = env.get_string(&daten).map(Into::into).unwrap_or_default();
        let zwischen: String = env
            .get_string(&zwischen)
            .map(Into::into)
            .unwrap_or_default();
        let ergebnis = catch_unwind(AssertUnwindSafe(|| {
            match crate::Zustand::holen(daten.into(), zwischen.into()) {
                Ok(z) => tauri::async_runtime::block_on(super::spiegeln(&z, true))
                    .err()
                    .unwrap_or_default(),
                Err(e) => e,
            }
        }))
        .unwrap_or_else(|_| "Crash in the phone contacts".into());
        env.new_string(ergebnis)
            .map(|s| s.into_raw())
            .unwrap_or(std::ptr::null_mut())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn weg(art: &str, b: Option<&str>, wert: &str) -> Weg {
        Weg {
            art: art.into(),
            beschriftung: b.map(Into::into),
            wert: wert.into(),
        }
    }

    #[test]
    fn nur_was_das_telefon_kennt_geht_hinueber() {
        let k = Kontakt {
            uuid: "u".into(),
            anzeigename: "Beispiel Probe".into(),
            wege: vec![
                weg("matrix", None, "@probe:beispiel.test"),
                weg("phone", Some("Handy"), "+49 30 1234567"),
                weg("email", Some("Arbeit"), "probe@beispiel.test"),
                weg("phone", Some("Ferienhaus"), "+49 40 7654321"),
            ],
            ..Default::default()
        };
        let w = wege_ins_telefon(&k);
        assert_eq!(w.len(), 3);
        assert_eq!(
            (w[0].sorte.as_str(), w[0].beschriftung.as_str()),
            ("mobile", "")
        );
        assert_eq!(w[1].sorte, "work");
        assert_eq!(
            (w[2].sorte.as_str(), w[2].beschriftung.as_str()),
            ("custom", "Ferienhaus")
        );
    }

    #[test]
    fn zurueck_bleiben_kennungen_und_beschriftungen() {
        let alt = vec![
            weg("matrix", None, "@probe:beispiel.test"),
            weg("phone", Some("Handy"), "+49 30 1234567"),
        ];
        let neu = vec![
            Sysweg {
                art: "phone".into(),
                sorte: "mobile".into(),
                beschriftung: String::new(),
                wert: "+49 30 1234567".into(),
            },
            Sysweg {
                art: "email".into(),
                sorte: "home".into(),
                beschriftung: String::new(),
                wert: "probe@beispiel.test".into(),
            },
        ];
        let w = wege_zusammen(&alt, &neu);
        assert_eq!(w[0], alt[0]);
        // „Handy“ bleibt „Handy“, nicht „CELL“.
        assert_eq!(w[1], alt[1]);
        assert_eq!(w[2], weg("email", Some("HOME"), "probe@beispiel.test"));
        // Nichts geändert: dieselben Wege.
        assert!(gleich(&wege_zusammen(&alt, &neu[..1]), &alt));
    }
}
