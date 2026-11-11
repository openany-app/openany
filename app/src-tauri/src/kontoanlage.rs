//! Ein Konto in der App anlegen, seine Fristen und der Ausweis-Tausch nach
//! einer eingespielten Sicherung (docs/plan-registrierung-app.md, A2-A4).
//!
//! **Anlegen:** Name nennen, anyid gibt ein Raetsel aus, dieses Geraet
//! rechnet es in einem eigenen Faden -- je nach Geraet Sekunden bis eine
//! halbe Minute -- und legt mit der Loesung an. Die Oberflaeche fragt im
//! Takt nach dem Stand ([`konto_anlegen_stand`]) und kann abbrechen.
//!
//! **Danach** ist das Geraet gekoppelt wie nach einer Freischaltung. Die App
//! bietet sofort eine Sicherung an: Ohne Passwort ist sie der einzige Weg
//! zurueck, falls das Geraet verloren geht.

use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::{Arc, LazyLock, Mutex};

use openany_anmeldung::{loesen, Anmeldefehler, Raetselstand};
use serde::Serialize;

use crate::{direktpost, Zustand};

/// Ein laufendes Anlegen. Hoechstens eins je Programm.
struct Lauf {
    stand: Arc<Raetselstand>,
    bits: u32,
    ergebnis: Arc<Mutex<Option<Result<String, String>>>>,
}

static LAUF: LazyLock<Mutex<Option<Lauf>>> = LazyLock::new(|| Mutex::new(None));

/// Aus einer Ablehnung von anyid das Wort (`name_vergeben`, `geschlossen`
/// ...), das die Oberflaeche uebersetzt; sonst der Fehler, wie er ist.
fn wort(e: Anmeldefehler) -> String {
    match e {
        // Die Drossel von anyid antwortet englisch und ohne Wort.
        Anmeldefehler::Anyid(anyid_client::AnyidError::Abgelehnt { status, .. })
            if status.as_u16() == 429 =>
        {
            "zu_viele".into()
        }
        Anmeldefehler::Anyid(anyid_client::AnyidError::Abgelehnt { meldung, .. }) => meldung,
        anderes => anderes.to_string(),
    }
}

/// Name pruefen, Raetsel holen und im Hintergrund loesen und anlegen.
#[tauri::command]
pub async fn konto_anlegen_beginnen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    name: String,
) -> Result<(), String> {
    let name = name.trim().to_string();
    crate::openany_vorbereiten(&zustand).await?;
    let raetsel = zustand
        .anmeldung()
        .await?
        .konto_raetsel(&name)
        .await
        .map_err(wort)?;

    let stand = Arc::new(Raetselstand::default());
    let ergebnis = Arc::new(Mutex::new(None));
    if let Some(alt) = LAUF.lock().unwrap().replace(Lauf {
        stand: stand.clone(),
        bits: raetsel.bits,
        ergebnis: ergebnis.clone(),
    }) {
        alt.stand.abbrechen.store(true, Ordering::Relaxed);
    }

    let zustand = zustand.inner().clone();
    tauri::async_runtime::spawn(async move {
        let geloest = tauri::async_runtime::spawn_blocking({
            let stand = stand.clone();
            let raetsel = raetsel.clone();
            move || loesen(&raetsel, &stand)
        })
        .await;
        let fertig = match geloest {
            Ok(Ok(loesung)) => anlegen(&zustand, &raetsel.raetsel, &loesung).await,
            Ok(Err(e)) => Err(e.to_string()),
            Err(e) => Err(e.to_string()),
        };
        *ergebnis.lock().unwrap() = Some(fertig);
    });
    Ok(())
}

async fn anlegen(zustand: &Arc<Zustand>, raetsel: &str, loesung: &str) -> Result<String, String> {
    let name = zustand
        .anmeldung()
        .await?
        .konto_anlegen(raetsel, loesung)
        .await
        .map_err(wort)?;
    crate::kontoname_merken(zustand, &name).await;
    sicherung_anbieten_setzen(zustand, true);
    // Die Schluessel dieses Geraets melden und den Konto-Schluessel der
    // Nachrichten anlegen -- es ist das erste Geraet des Kontos.
    if let Some(client) = crate::server_client(zustand).await {
        let _ = direktpost::bereit(zustand, &client).await;
    }
    Ok(name)
}

#[derive(Serialize)]
pub struct Anlegestand {
    laeuft: bool,
    /// Zwischen 0 und 1, geschaetzt aus den Versuchen. Erwartet sind 2^bits;
    /// wer Pech hat, braucht mehr -- dann bleibt der Balken kurz vor dem Ende.
    anteil: f64,
    /// Der Kontoname, sobald angelegt.
    fertig: Option<String>,
    fehler: Option<String>,
}

#[tauri::command]
pub async fn konto_anlegen_stand() -> Anlegestand {
    let mut halter = LAUF.lock().unwrap();
    let Some(lauf) = halter.as_ref() else {
        return Anlegestand {
            laeuft: false,
            anteil: 0.0,
            fertig: None,
            fehler: None,
        };
    };
    let erwartet = 2f64.powi(lauf.bits as i32);
    let anteil = (lauf.stand.versuche.load(Ordering::Relaxed) as f64 / erwartet).min(0.95);
    let ergebnis = lauf.ergebnis.lock().unwrap().clone();
    match ergebnis {
        None => Anlegestand {
            laeuft: true,
            anteil,
            fertig: None,
            fehler: None,
        },
        Some(e) => {
            *halter = None;
            match e {
                Ok(name) => Anlegestand {
                    laeuft: false,
                    anteil: 1.0,
                    fertig: Some(name),
                    fehler: None,
                },
                Err(f) => Anlegestand {
                    laeuft: false,
                    anteil,
                    fertig: None,
                    fehler: Some(f),
                },
            }
        }
    }
}

#[tauri::command]
pub async fn konto_anlegen_abbrechen() {
    if let Some(lauf) = LAUF.lock().unwrap().take() {
        lauf.stand.abbrechen.store(true, Ordering::Relaxed);
    }
}

// --- Fristen am Konto (A3) --------------------------------------------------

/// Der Stand vom Server -- `None` ohne Verbindung oder Netz. Maßgeblich ist
/// immer der Server; die App merkt sich nichts davon.
#[tauri::command]
pub async fn konto_fristen(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Option<openany_client::Kontofristen>, String> {
    let Some(client) = crate::server_client(&zustand).await else {
        return Ok(None);
    };
    Ok(client.konto_fristen().await.ok())
}

/// Warum das Konto geloescht wurde -- nur sinnvoll, wenn der Schluessel
/// abgewiesen wird. `None` ohne Verbindung oder ohne Abschied.
#[tauri::command]
pub async fn konto_abschied(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Option<openany_client::Abschied>, String> {
    let Some(client) = crate::server_client(&zustand).await else {
        return Ok(None);
    };
    Ok(client.konto_abschied().await.ok().flatten())
}

/// `erste` (Nutzungsbedingungen), `zweite` oder `aktiv` (Konto behalten).
#[tauri::command]
pub async fn konto_bestaetigen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    was: String,
) -> Result<openany_client::Kontofristen, String> {
    let client = crate::server_client(&zustand)
        .await
        .ok_or("This device is not connected.")?;
    client
        .konto_bestaetigen(&was)
        .await
        .map_err(|e| e.to_string())
}

/// Die App ist im Vordergrund. Fehler sind egal -- beim naechsten Mal.
#[tauri::command]
pub async fn konto_aktiv(
    zustand: tauri::State<'_, Arc<Zustand>>,
    sprache: String,
) -> Result<(), String> {
    if let Some(client) = crate::server_client(&zustand).await {
        let _ = client.konto_aktiv(&sprache).await;
    }
    Ok(())
}

// --- Sicherung anbieten -----------------------------------------------------

fn anbieten_pfad(zustand: &Zustand) -> PathBuf {
    zustand.ordner.join("sicherung-anbieten")
}

/// Nach dem Anlegen eines Kontos und nach einem Ausweis-Tausch: Eine
/// Sicherung ist faellig, bis eine angelegt ist.
pub(crate) fn sicherung_anbieten_setzen(zustand: &Zustand, an: bool) {
    let p = anbieten_pfad(zustand);
    if an {
        let _ = std::fs::write(p, b"");
    } else {
        let _ = std::fs::remove_file(p);
    }
}

#[tauri::command]
pub async fn sicherung_anbieten(zustand: tauri::State<'_, Arc<Zustand>>) -> Result<bool, String> {
    Ok(anbieten_pfad(&zustand).is_file())
}

// --- Ausweis tauschen nach dem Einspielen (A2) ------------------------------

/// Die Merkdatei, die das Einspielen hinterlaesst (sicherungsbefehle.rs).
pub(crate) const TAUSCH_FAELLIG: &str = "ausweis-tauschen";

/// Beim Start: Ist eine Sicherung mit Ausweis eingespielt worden, den
/// Ausweis tauschen und einen frischen openany-Schluessel holen. Der alte
/// Ausweis gilt danach nicht mehr -- auch nicht auf dem Geraet, von dem die
/// Datei stammt.
///
/// Ohne Netz bleibt die Merkdatei und es geht beim naechsten Start weiter.
/// Taugt der Ausweis nicht mehr (widerrufen, schon getauscht), ist das Geraet
/// danach getrennt und verbindet sich ueber die 6 Ziffern neu.
pub(crate) async fn tauschen_falls_faellig(zustand: Arc<Zustand>) {
    let merk = zustand.ordner.join(TAUSCH_FAELLIG);
    if !merk.is_file() {
        return;
    }
    let anmeldung = match zustand.anmeldung().await {
        Ok(a) => a,
        Err(_) => return,
    };
    match anmeldung.ausweis_tauschen().await {
        Ok(name) => {
            let _ = std::fs::remove_file(&merk);
            crate::kontoname_merken(&zustand, &name).await;
            sicherung_anbieten_setzen(&zustand, true);
            if let Some(client) = crate::server_client(&zustand).await {
                let _ = direktpost::bereit(&zustand, &client).await;
            }
        }
        Err(Anmeldefehler::Anyid(anyid_client::AnyidError::GeraetWiderrufen))
        | Err(Anmeldefehler::NichtGekoppelt) => {
            let _ = std::fs::remove_file(&merk);
            let _ = crate::verbindung_vergessen(&zustand).await;
        }
        Err(_) => {}
    }
}
