//! Der Postausgang fuer den Weg „openany" (Tiffy, 02.10.2026: local-first).
//!
//! **Ohne Netz wird nicht abgelehnt, sondern gewartet.** Eine Nachricht, die
//! den Server nicht erreicht -- kein Netz, Zeitueberschreitung, der Server
//! bremst oder hat gerade einen Aussetzer (5xx) --, liegt hier und geht beim
//! naechsten Mal hinaus: beim Oeffnen der Nachrichten und im
//! Hintergrundlauf (hintergrund.rs). Im Verlauf steht sie solange mit
//! „wartet".
//!
//! **Was der Server ablehnt, wartet nicht.** Ein unbekannter Empfaenger wird
//! durch Wiederholen nicht bekannt. Beim Senden kommt der Fehler sofort; trifft
//! er eine Nachricht, die schon wartete, bleibt sie mit dem Grund stehen und
//! wird nicht weiter versucht -- loeschen kann man sie wie jede andere.
//!
//! Abgelegt neben den Einstellungen (`postausgang-openany.json`).

use crate::Zustand;
use openany_client::OpenanyError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

/// Kennungen im Verlauf beginnen hiermit -- so kennt `nachricht_loeschen`
/// eine wartende Nachricht.
pub const VORSILBE: &str = "warte:";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Wartend {
    pub id: String,
    pub ziel: String,
    pub text: String,
    pub at: String,
    /// Der Server hat abgelehnt -- wird nicht mehr versucht.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fehler: Option<String>,
}

/// Lohnt es, es spaeter noch einmal zu versuchen?
pub fn spaeter_nochmal(e: &OpenanyError) -> bool {
    match e {
        OpenanyError::Transport(_) | OpenanyError::Gebremst { .. } => true,
        OpenanyError::Abgelehnt { status, .. } => status.is_server_error(),
        _ => false,
    }
}

// Lesen und Schreiben der Datei nie gleichzeitig.
static SPERRE: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn pfad(zustand: &Zustand) -> PathBuf {
    zustand.ordner.join("postausgang-openany.json")
}

fn lesen(zustand: &Zustand) -> Vec<Wartend> {
    std::fs::read_to_string(pfad(zustand))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

fn schreiben(zustand: &Zustand, liste: &[Wartend]) {
    let _ = std::fs::write(
        pfad(zustand),
        serde_json::to_string_pretty(liste).unwrap_or_default(),
    );
}

fn aendern<T>(zustand: &Zustand, tun: impl FnOnce(&mut Vec<Wartend>) -> T) -> T {
    let _sperre = SPERRE.lock().unwrap_or_else(|e| e.into_inner());
    let mut liste = lesen(zustand);
    let ergebnis = tun(&mut liste);
    schreiben(zustand, &liste);
    ergebnis
}

pub fn alle(zustand: &Zustand) -> Vec<Wartend> {
    let _sperre = SPERRE.lock().unwrap_or_else(|e| e.into_inner());
    lesen(zustand)
}

pub fn einreihen(zustand: &Zustand, ziel: &str, text: &str) -> Wartend {
    let w = Wartend {
        id: uuid::Uuid::new_v4().to_string(),
        ziel: ziel.to_string(),
        text: text.to_string(),
        at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        fehler: None,
    };
    aendern(zustand, |l| l.push(w.clone()));
    w
}

/// Eine wartende Nachricht zuruecknehmen. `true`, wenn es sie gab.
pub fn entfernen(zustand: &Zustand, id: &str) -> bool {
    aendern(zustand, |l| {
        let vorher = l.len();
        l.retain(|w| w.id != id);
        l.len() != vorher
    })
}

/// Was wartet, hinausschicken -- der Reihe nach, damit die Reihenfolge beim
/// Empfaenger stimmt. Beim ersten Fehler, der „spaeter nochmal" heisst, hoert
/// der Gang auf: Dann ist das Netz weg, und der Rest scheiterte genauso.
/// Gibt zurueck, wie viele sich geaendert haben (hinaus oder abgelehnt) --
/// dann laedt die Ansicht neu.
pub async fn leeren(zustand: &Arc<Zustand>) -> usize {
    let offen: Vec<Wartend> = alle(zustand)
        .into_iter()
        .filter(|w| w.fehler.is_none())
        .collect();
    if offen.is_empty() {
        return 0;
    }
    let Some(client) = crate::server_client(zustand).await else {
        return 0;
    };
    let mut hinaus = 0;
    for w in offen {
        match client.nachricht_senden(&w.ziel, &w.text).await {
            Ok(_) => {
                entfernen(zustand, &w.id);
                hinaus += 1;
            }
            Err(e) if spaeter_nochmal(&e) => break,
            Err(e) => {
                aendern(zustand, |l| {
                    if let Some(x) = l.iter_mut().find(|x| x.id == w.id) {
                        x.fehler = Some(e.to_string());
                    }
                });
                hinaus += 1;
            }
        }
    }
    hinaus
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nur_was_vorbeigeht_wartet() {
        assert!(spaeter_nochmal(&OpenanyError::Gebremst { sekunden: 5 }));
        // Ein widerrufener oder falsch ausgestellter Schluessel wird durch
        // Warten nicht gueltig.
        assert!(!spaeter_nochmal(&OpenanyError::SchluesselUngueltig));
        assert!(!spaeter_nochmal(&OpenanyError::NichtErlaubt("x".into())));
        assert!(!spaeter_nochmal(&OpenanyError::Unlesbar("x".into())));
    }
}
