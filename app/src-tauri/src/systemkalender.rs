//! Kalender im Kalender des Telefons -- wie DAVx5, nur Ende-zu-Ende
//! (docs/plan-kalender-kontakte-im-system.md, K1). Nur unter Android.
//!
//! Der Server kann Termine nicht lesen; also trägt die App sie selbst ein:
//! als Konto „openany“ in den Kalender des Systems (`CalendarContract`),
//! entschlüsselt, nur auf diesem Gerät. Danach zeigt jede Kalender-App sie.
//!
//! **Hier der Soll-Stand, in Kotlin das Eintragen.** [`stand`] stellt
//! zusammen, was im System stehen soll -- eigene Kalender, in Hubs geteilte
//! (nur ansehbare nur lesend) und Abos nur auf Wunsch (Entscheidung 2) --,
//! `Systemkalender.kt` gleicht den Systemkalender damit ab. Was jemand dort
//! schon geändert hat (`DIRTY`), lässt es stehen: Das holt K2 zurück.
//!
//! **Wann:** beim Einschalten, beim Start, wenn sich etwas ändert (ein
//! leiser Takt vergleicht den Abdruck des Stands), im Auffrischer und wenn
//! Android den SyncAdapter anstößt -- das tut es, sobald jemand im Kalender
//! des Telefons einen openany-Termin ändert.
//!
//! **Zurück (K2):** Vor jedem Eintragen holt [`zurueckholen`], was eine
//! andere Kalender-App geändert, angelegt oder gelöscht hat, und schreibt es
//! in openany -- verschlüsselt und abgeglichen wie jede Änderung. Treffen
//! sich zwei Änderungen, gewinnt die aus dem Telefon: Sie ist die, die eben
//! bemerkt wurde. Felder, die openany nicht kennt, gehen verloren.

#![cfg_attr(not(target_os = "android"), allow(dead_code))]

use crate::Zustand;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::sync::{Arc, Mutex};

fn fehler(e: impl ToString) -> String {
    e.to_string()
}

/// Ein Kalender, wie er ins System kommt.
#[derive(Serialize, Debug, PartialEq)]
struct Syskalender {
    /// `_SYNC_ID`: die uuid des Kalenders, bei geteilten `hub:<uuid>`.
    schluessel: String,
    name: String,
    /// `#rrggbb`
    farbe: String,
    schreibbar: bool,
    termine: Vec<Systermin>,
}

/// Ein Termin, wie er ins System kommt. Zeiten als Wanduhrzeit
/// (`Y-m-dTH:i:s`) -- die Zone setzt Kotlin, sie ist die des Geräts.
#[derive(Serialize, Debug, PartialEq)]
struct Systermin {
    schluessel: String,
    titel: String,
    beschreibung: String,
    ort: String,
    beginn: String,
    ende: String,
    ganztags: bool,
    /// RFC 5545, leer = einmalig.
    rrule: String,
    exdate: String,
    /// Minuten vor Beginn, leer = keine (K3) -- Android meldet sie selbst.
    erinnerung: String,
}

/// openanys Wiederholung (`DAILY` … `YEARLY`, bis `rrule_bis`, dazu der
/// Rest der Regel) als RRULE.
fn rrule(frequenz: &str, bis: &str, regel: &str) -> String {
    let f = frequenz.trim().to_ascii_uppercase();
    if !matches!(f.as_str(), "DAILY" | "WEEKLY" | "MONTHLY" | "YEARLY") {
        return String::new();
    }
    // Aus dem Abgleich, bei geteilten Kalendern von anderen: nur, was eine
    // Regel sein kann (`ics_regel_sauber`).
    let regel = openany_sync::ics_regel_sauber(regel);
    if !regel.is_empty() {
        // COUNT und UNTIL schliessen sich aus (RFC 5545).
        let bis = if regel.contains("COUNT=") { "" } else { bis };
        let kern = rrule(&f, bis, "");
        return format!("{kern};{regel}");
    }
    let datum: String = bis
        .trim()
        .chars()
        .take(10)
        .filter(char::is_ascii_digit)
        .collect();
    if datum.len() == 8 {
        format!("FREQ={f};UNTIL={datum}T235959Z")
    } else {
        format!("FREQ={f}")
    }
}

fn termin_aus(schluessel: String, f: &openany_client::Terminfelder) -> Systermin {
    Systermin {
        schluessel,
        titel: f.titel.clone(),
        beschreibung: f.beschreibung.clone(),
        ort: f.ort.clone(),
        beginn: f.beginn.clone(),
        ende: f.ende.clone(),
        ganztags: f.ganztags,
        rrule: rrule(&f.rrule, &f.rrule_bis, &f.regel),
        exdate: openany_sync::ics_zeiten_sauber(&f.exdates),
        erinnerung: f.erinnerung.clone(),
    }
}

/// Ein Termin eines geteilten Kalenders -- die Felder mit ihren Namen über
/// der Leitung.
fn geteilter_termin(schluessel: &str, f: &Map<String, Value>) -> Systermin {
    let text = |k: &str| {
        f.get(k)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    };
    let ganz = f
        .get("all_day")
        .map(|v| v.as_bool().unwrap_or_else(|| v.as_i64() == Some(1)))
        .unwrap_or(false);
    Systermin {
        schluessel: schluessel.to_string(),
        titel: text("title"),
        beschreibung: text("description"),
        ort: text("location"),
        beginn: openany_store::kalenderbuch::zeit(&text("start")),
        ende: openany_store::kalenderbuch::zeit(&text("end")),
        ganztags: ganz,
        rrule: rrule(&text("rrule"), &text("rrule_until"), &text("rrule_parts")),
        exdate: text("exdates"),
        erinnerung: text("reminder"),
    }
}

/// Was im System stehen soll.
async fn stand(zustand: &Zustand, abos: bool) -> Result<Vec<Syskalender>, String> {
    let mut aus = Vec::new();
    {
        let speicher = zustand.speicher.lock().await;
        let termine = speicher.termine_alle().map_err(fehler)?;
        for k in speicher.kalender_alle().map_err(fehler)? {
            if k.papierkorb_at.is_some() || (k.ist_abo() && !abos) {
                continue;
            }
            let hier: Vec<_> = termine
                .iter()
                .filter(|t| t.kalender_uuid == k.uuid && t.papierkorb_at.is_none())
                .collect();
            let termine = hier
                .iter()
                .map(|t| {
                    let mut s = termin_aus(t.uuid.clone(), &t.felder);
                    // Ein verschobenes Vorkommen steht im Telefon als eigener
                    // Termin; die Serie lässt seinen alten Platz aus.
                    for v in hier.iter().filter(|v| v.felder.serie == t.uuid) {
                        if !v.felder.vorkommen.is_empty()
                            && !s.exdate.split(',').any(|e| e == v.felder.vorkommen)
                        {
                            if !s.exdate.is_empty() {
                                s.exdate.push(',');
                            }
                            s.exdate.push_str(&v.felder.vorkommen);
                        }
                    }
                    s
                })
                .collect();
            aus.push(Syskalender {
                schreibbar: !k.ist_abo(),
                schluessel: k.uuid,
                name: k.name,
                farbe: k.farbe,
                termine,
            });
        }
    }
    // In Hubs geteilt (Entscheidung 2): nur ansehbare nur lesend.
    for (projekt, g) in crate::projektkalender::fremde(zustand).await {
        if g.eigen {
            continue;
        }
        aus.push(Syskalender {
            schluessel: format!("hub:{}", g.uuid),
            name: format!("{} ({projekt})", g.name),
            farbe: g.color.clone(),
            schreibbar: hub_schreibbar(&g),
            termine: g
                .termine
                .iter()
                .map(|(u, f)| geteilter_termin(u, f))
                .collect(),
        });
    }
    Ok(aus)
}

/// Ein geteilter Kalender, in den Mitglieder schreiben dürfen -- über den
/// Hub; vor Ort Geteiltes wird nur gelesen.
fn hub_schreibbar(g: &crate::projektkalender::Geteilt) -> bool {
    g.stufe == "edit"
        && crate::projektkalender::freigabe_von_kalender(&format!("p:{}", g.uuid)).is_some()
}

/// Eine Änderung aus dem Kalender des Telefons (`Systemkalender.aenderungen`).
#[derive(Deserialize, Debug)]
struct Aenderung {
    id: i64,
    schluessel: Option<String>,
    kalender: String,
    geloescht: bool,
    titel: String,
    beschreibung: String,
    ort: String,
    beginn: String,
    ende: String,
    ganztags: bool,
    frequenz: String,
    bis: String,
    exdate: String,
    /// Die erste Erinnerung im Telefon, Minuten; leer = keine.
    #[serde(default)]
    erinnerung: String,
    /// Die ganze RRULE -- für den Rest über die Frequenz hinaus.
    #[serde(default)]
    rrule: String,
    /// Ein einzeln geändertes Vorkommen (Ausnahme im Telefon): der
    /// Schlüssel der Serie und ihr ursprünglicher Beginn (`YYYYMMDDTHHMMSS`).
    #[serde(default)]
    serie: String,
    #[serde(default)]
    vorkommen: String,
    /// Im Telefon abgesagt: Bei einem Vorkommen heißt das „fällt aus“.
    #[serde(default)]
    abgesagt: bool,
}

/// Was übernommen ist (`Systemkalender.bestaetigen`).
#[derive(Serialize)]
struct Erledigt {
    id: i64,
    schluessel: Option<String>,
    loeschen: bool,
}

fn felder_setzen(f: &mut openany_client::Terminfelder, a: &Aenderung) {
    f.titel = a.titel.clone();
    f.beschreibung = a.beschreibung.clone();
    f.ort = a.ort.clone();
    f.beginn = openany_store::kalenderbuch::zeit(&a.beginn);
    f.ende = openany_store::kalenderbuch::zeit(&a.ende);
    f.ganztags = a.ganztags;
    f.rrule = a.frequenz.clone();
    f.rrule_bis = if a.bis.is_empty() {
        String::new()
    } else {
        openany_store::kalenderbuch::zeit(&a.bis)
    };
    f.exdates = a.exdate.clone();
    f.erinnerung = a.erinnerung.clone();
    f.regel = if a.rrule.is_empty() || f.rrule == "NONE" {
        String::new()
    } else {
        openany_sync::ics_regel_aus(&a.rrule)
    };
    if !a.serie.is_empty() && !a.vorkommen.is_empty() {
        f.serie = a.serie.clone();
        f.vorkommen = a.vorkommen.clone();
    }
}

/// Eine Änderung in openany übernehmen. `None`: nicht übernehmbar (nur
/// lesend) -- dann setzt der nächste Lauf den Termin im Telefon zurück.
async fn uebernehmen(zustand: &Arc<Zustand>, a: &Aenderung) -> Result<Erledigt, String> {
    let zurueck = Erledigt {
        id: a.id,
        schluessel: a.schluessel.clone(),
        loeschen: false,
    };
    // In einem Hub geteilt: nur, wer bearbeiten darf, und über den Hub.
    if let Some(freigabe) = a.kalender.strip_prefix("hub:") {
        let darf = crate::projektkalender::fremde(zustand)
            .await
            .into_iter()
            .any(|(_, g)| g.uuid == freigabe && hub_schreibbar(&g));
        if !darf {
            return Ok(zurueck);
        }
        if a.geloescht {
            if let Some(u) = a.schluessel.as_deref() {
                crate::projektkalender::termin_loeschen(zustand, freigabe, u).await?;
            }
            return Ok(Erledigt {
                loeschen: true,
                ..zurueck
            });
        }
        let (t, _) = crate::projektkalender::termin_speichern(
            zustand,
            freigabe,
            a.schluessel.clone(),
            |f| felder_setzen(f, a),
        )
        .await?;
        return Ok(Erledigt {
            schluessel: Some(t.uuid),
            ..zurueck
        });
    }

    let speicher = zustand.speicher.lock().await;
    let kalender = speicher.kalender(&a.kalender).map_err(fehler)?;
    let Some(kalender) = kalender.filter(|k| k.papierkorb_at.is_none() && !k.ist_abo()) else {
        return Ok(zurueck);
    };
    if a.geloescht {
        if let Some(u) = a.schluessel.as_deref() {
            crate::serie_wegwerfen(&speicher, u)?;
        }
        return Ok(Erledigt {
            loeschen: true,
            ..zurueck
        });
    }
    // Ein Vorkommen im Telefon abgesagt: In der Serie fällt es aus. Die
    // Ausnahme dort geht weg; der nächste Lauf trägt die Serie mit dem
    // ausgelassenen Tag ein.
    if a.abgesagt && !a.serie.is_empty() && !a.vorkommen.is_empty() {
        if let Some(mut serie) = speicher.termin(&a.serie).map_err(fehler)? {
            if !serie.felder.exdates.split(',').any(|e| e == a.vorkommen) {
                if !serie.felder.exdates.is_empty() {
                    serie.felder.exdates.push(',');
                }
                serie.felder.exdates.push_str(&a.vorkommen);
                serie.geaendert_at = String::new();
                speicher
                    .termin_schreiben(&serie, openany_store::Protokoll::Merken)
                    .map_err(fehler)?;
            }
        }
        if let Some(u) = a.schluessel.as_deref() {
            speicher
                .papierkorb(
                    &openany_client::Art::Termin,
                    u,
                    openany_store::Protokoll::Merken,
                )
                .map_err(fehler)?;
        }
        return Ok(Erledigt {
            loeschen: true,
            ..zurueck
        });
    }
    let uuid = a
        .schluessel
        .clone()
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    // Was der Kalender des Telefons nicht kennt, bleibt stehen.
    let mut felder = speicher
        .termin(&uuid)
        .map_err(fehler)?
        .map(|t| t.felder)
        .unwrap_or_default();
    felder_setzen(&mut felder, a);
    speicher
        .termin_schreiben(
            &openany_store::Termin {
                uuid: uuid.clone(),
                kalender_uuid: kalender.uuid,
                felder,
                papierkorb_at: None,
                geaendert_at: String::new(),
            },
            openany_store::Protokoll::Merken,
        )
        .map_err(fehler)?;
    Ok(Erledigt {
        schluessel: Some(uuid),
        ..zurueck
    })
}

/// K2: holen, was im Kalender des Telefons geändert wurde. Gibt die Zahl
/// der übernommenen Änderungen zurück.
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
            Err(e) => eprintln!("Phone calendar, change {}: {e}", a.id),
        }
    }
    let text = serde_json::to_string(&erledigt).map_err(fehler)?;
    tauri::async_runtime::spawn_blocking(move || android::bestaetigen(&text))
        .await
        .map_err(fehler)?;
    // Der nächste Lauf schreibt alles in openanys Form neu.
    *ZULETZT.lock().unwrap_or_else(|e| e.into_inner()) = None;
    Ok(erledigt.len())
}

/// Der Abdruck des zuletzt eingetragenen Stands -- gleich, dann nichts tun.
static ZULETZT: Mutex<Option<u64>> = Mutex::new(None);

fn abdruck(text: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut h);
    h.finish()
}

/// Eintragen, wenn eingeschaltet und anders als zuletzt. `immer`: auch wenn
/// gleich (beim Einschalten, aus dem SyncAdapter).
/// Ein Lauf zur Zeit: SyncAdapter, Takt und Einschalten kommen sonst
/// gleichzeitig, lesen dieselbe Änderung und legen je einen Termin an.
static SPERRE: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

pub(crate) async fn spiegeln(zustand: &Arc<Zustand>, immer: bool) -> Result<(), String> {
    let _sperre = SPERRE.lock().await;
    let wahl = zustand.einstellungen.lock().await.systemkalender.clone();
    let Some(wahl) = wahl else {
        return Ok(());
    };
    // Erst zurück, was im Telefon geändert wurde -- sonst stünde es gleich
    // wieder in der alten Fassung da.
    if let Err(e) = zurueckholen(zustand).await {
        eprintln!("Phone calendar, changes: {e}");
    }
    let soll = stand(zustand, wahl.abos).await?;
    let text = serde_json::to_string(&soll).map_err(fehler)?;
    let a = abdruck(&text);
    if !immer && *ZULETZT.lock().unwrap_or_else(|e| e.into_inner()) == Some(a) {
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
        None => Err("The phone calendar is not reachable.".into()),
    }
}

#[derive(Serialize)]
pub struct Systemkalenderlage {
    an: bool,
    abos: bool,
    /// Gibt es den Kalender des Telefons hier (Android)?
    moeglich: bool,
}

#[tauri::command]
pub async fn systemkalender_stand(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Systemkalenderlage, String> {
    let w = zustand.einstellungen.lock().await.systemkalender.clone();
    Ok(Systemkalenderlage {
        an: w.is_some(),
        abos: w.is_some_and(|w| w.abos),
        moeglich: cfg!(target_os = "android"),
    })
}

/// Ein- oder ausschalten. Aus: das Konto fort -- Android nimmt seine
/// Kalender und Termine mit; in openany bleibt alles.
#[tauri::command]
pub async fn systemkalender_setzen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    an: bool,
    abos: bool,
) -> Result<(), String> {
    {
        let mut e = zustand.einstellungen.lock().await;
        e.systemkalender = an.then_some(crate::einstellungen::Systemkalender { abos });
        e.schreiben(&zustand.einstellungspfad()).map_err(fehler)?;
    }
    *ZULETZT.lock().unwrap_or_else(|e| e.into_inner()) = None;
    if an {
        spiegeln(&zustand, true).await
    } else {
        // Das Konto trägt auch die Kontakte (K4): dann nur die Kalender fort.
        let kontakte = zustand.einstellungen.lock().await.systemkontakte;
        tauri::async_runtime::spawn_blocking(move || {
            if kontakte {
                android::leeren();
            } else {
                android::abmelden();
            }
        })
        .await
        .map_err(fehler)?;
        Ok(())
    }
}

/// Das Konto „openany“ fort -- Android nimmt Kalender und Kontakte mit.
pub(crate) fn konto_fort() {
    android::abmelden();
}

/// Beim Start: einmal eintragen, dann leise im Takt nachsehen.
pub(crate) fn beim_start(zustand: &Arc<Zustand>) {
    let z = zustand.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            if let Err(e) = spiegeln(&z, false).await {
                eprintln!("Phone calendar: {e}");
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
    pub(super) fn abmelden() {}
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
    pub extern "system" fn Java_de_openany_app_Systemkalender_anmelden(
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

    /// `""` = gelungen, sonst der Grund.
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

    pub(super) fn abmelden() {
        let _ = rufen(|env, k| env.call_static_method(k, "abmelden", "()V", &[])?.v());
    }

    /// Nur die Kalender des Kontos fort (das Konto bleibt für die Kontakte).
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

    /// Der SyncAdapter (`Systemkalender.kt`): eintragen, auch ohne Fenster.
    #[no_mangle]
    pub extern "system" fn Java_de_openany_app_Systemkalender_laufen(
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
        .unwrap_or_else(|_| "Crash in the phone calendar".into());
        env.new_string(ergebnis)
            .map(|s| s.into_raw())
            .unwrap_or(std::ptr::null_mut())
    }
}

#[cfg(test)]
mod tests {
    use super::{felder_setzen, rrule, Aenderung};

    #[test]
    fn eine_aenderung_aus_dem_telefon_in_openanys_form() {
        let a = Aenderung {
            id: 7,
            schluessel: None,
            kalender: "k".into(),
            geloescht: false,
            titel: "Lauftreff".into(),
            beschreibung: String::new(),
            ort: "Park".into(),
            beginn: "2026-11-11T00:00:00".into(),
            ende: "2026-11-11T00:00:00".into(),
            ganztags: true,
            frequenz: "WEEKLY".into(),
            bis: "2027-03-31".into(),
            exdate: String::new(),
            erinnerung: "15".into(),
            rrule: "FREQ=WEEKLY;UNTIL=20270331T235959Z;BYDAY=WE,FR;INTERVAL=2".into(),
            serie: String::new(),
            vorkommen: String::new(),
            abgesagt: false,
        };
        let mut f = openany_client::Terminfelder {
            exdates: "alt".into(),
            ..Default::default()
        };
        felder_setzen(&mut f, &a);
        assert_eq!(f.titel, "Lauftreff");
        assert_eq!(f.beginn, "2026-11-11T00:00:00");
        assert_eq!(f.rrule, "WEEKLY");
        assert_eq!(f.rrule_bis, "2027-03-31T00:00:00");
        // Hin und zurück: dieselbe Regel.
        assert_eq!(f.regel, "INTERVAL=2;BYDAY=WE,FR");
        assert_eq!(
            rrule(&f.rrule, &f.rrule_bis, &f.regel),
            "FREQ=WEEKLY;UNTIL=20270331T235959Z;INTERVAL=2;BYDAY=WE,FR"
        );
        assert_eq!(f.exdates, "");
        assert_eq!(f.erinnerung, "15");
    }

    #[test]
    fn wiederholung_als_rrule() {
        assert_eq!(rrule("WEEKLY", "", ""), "FREQ=WEEKLY");
        assert_eq!(
            rrule("monthly", "2027-03-31", ""),
            "FREQ=MONTHLY;UNTIL=20270331T235959Z"
        );
        assert_eq!(
            rrule("YEARLY", "2027-03-31T00:00:00", ""),
            "FREQ=YEARLY;UNTIL=20270331T235959Z"
        );
        assert_eq!(rrule("NONE", "2027-03-31", ""), "");
        assert_eq!(rrule("", "", ""), "");
        // Mit COUNT kein UNTIL dazu.
        assert_eq!(
            rrule("WEEKLY", "2027-03-31", "COUNT=3"),
            "FREQ=WEEKLY;COUNT=3"
        );
    }
}
