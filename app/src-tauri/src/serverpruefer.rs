//! Mitglieder von Server-Projekten vor Ort erkennen (docs/plan-whiteboard.md,
//! W3a).
//!
//! Der Nahbereich kennt Geraete nur an ihrem TLS-Fingerabdruck. Wer in einem
//! SERVER-Projekt mitschreiben will, zeigt deshalb einen Ausweis: Name und
//! Geraet im Schluesselverzeichnis, die Zeit, unterschrieben mit dem
//! Geraeteschluessel -- gebunden an genau diese Leitung (beide
//! Fingerabdruecke, `openany_direktpost::ausweis_unterschreiben`). Dieses
//! Geraet, der Knoten, prueft:
//!
//! 1. die Zeit (fuenf Minuten),
//! 2. die Notiz liegt in einer EIGENEN Mappe, die in dem Projekt zum
//!    Bearbeiten geteilt ist -- bzw. das Whiteboard gehoert diesem Konto
//!    (W3b; mitzeichnen darf jedes Mitglied),
//! 3. der Name steht in der Mitgliederliste des Projekts (Server),
//! 4. die Unterschrift passt zum aktiven Geraet im Verzeichnis.
//!
//! Was der Server dabei erfaehrt: nichts Neues -- nur, dass dieses Geraet
//! Mitgliederliste und Verzeichnis liest, wie bei jedem Oeffnen.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use openany_nahbereich::sitzungen::{ist_whiteboard, Ausweis, ServerAnfrage, ServerPruefer};
use serde_json::Value;

use crate::projektchat;
use crate::{direktpost, mappenteilen, GETEILT};

/// Wie alt ein Ausweis hoechstens sein darf (Uhren gehen nicht gleich).
const AUSWEIS_GILT: u64 = 300;
/// Wie lange eine Antwort auf „ist diese Mappe hier geteilt?" gilt -- `hat`
/// fragt jeder in der Naehe, ohne Ausweis.
const MAPPE_GILT: Duration = Duration::from_secs(60);

pub(crate) fn jetzt() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// (Projekt, Freigabe) → (wann gefragt, Pfad der Mappe, wenn sie dort zum
/// Bearbeiten geteilt ist).
type Merker = HashMap<(String, String), (Instant, Option<String>)>;
static GEMERKT: Mutex<Option<Merker>> = Mutex::new(None);

/// Der Pfad einer eigenen Mappe, die in `projekt` zum Bearbeiten geteilt ist.
async fn geteilte_mappe(projekt: &str, freigabe: &str) -> Option<String> {
    let schluessel = (projekt.to_string(), freigabe.to_string());
    if let Some((wann, pfad)) = GEMERKT
        .lock()
        .ok()
        .and_then(|m| m.as_ref().and_then(|m| m.get(&schluessel).cloned()))
    {
        if wann.elapsed() < MAPPE_GILT {
            return pfad;
        }
    }
    let pfad = async {
        let zustand = GETEILT.get().cloned()?;
        let client = projektchat::verbunden(&zustand).await.ok()?;
        let roh = client.projekt_json("/mappen-freigaben", &[]).await.ok()?;
        let zum_bearbeiten = roh
            .get("freigaben")?
            .as_array()?
            .iter()
            .find(|f| f.get("uuid").and_then(Value::as_str) == Some(freigabe))?
            .get("projekte")?
            .as_array()?
            .iter()
            .any(|p| {
                p.get("projekt").and_then(Value::as_str) == Some(projekt)
                    && p.get("stufe").and_then(Value::as_str) == Some("edit")
            });
        if !zum_bearbeiten {
            return None;
        }
        mappenteilen::eigene(&zustand, &client)
            .await
            .ok()?
            .into_iter()
            .find(|f| f.uuid == freigabe && f.art == "mappe")
            .map(|f| f.pfad)
    }
    .await;
    if let Ok(mut m) = GEMERKT.lock() {
        m.get_or_insert_with(HashMap::new)
            .insert(schluessel, (Instant::now(), pfad.clone()));
    }
    pfad
}

/// Liegt die Notiz in dieser Mappe (oder darunter)?
async fn in_mappe(pfad: &str, notiz: &str) -> bool {
    let Some(zustand) = GETEILT.get().cloned() else {
        return false;
    };
    let mappe = zustand
        .speicher
        .lock()
        .await
        .notiz(notiz)
        .ok()
        .flatten()
        .filter(|n| !n.im_papierkorb())
        .and_then(|n| n.mappe);
    mappe.is_some_and(|m| m == pfad || m.starts_with(&format!("{pfad}/")))
}

/// Die Zeit eines Ausweises: nicht zu alt, nicht aus der Zukunft.
fn zeit_gilt(a: &Ausweis, jetzt: u64) -> bool {
    a.zeit.abs_diff(jetzt) <= AUSWEIS_GILT
}

pub(crate) struct ServerPruefen;

#[async_trait::async_trait]
impl ServerPruefer for ServerPruefen {
    async fn hat(&self, projekt: &str, freigabe: &str, notiz: &str) -> bool {
        match geteilte_mappe(projekt, freigabe).await {
            Some(pfad) => in_mappe(&pfad, notiz).await,
            None => false,
        }
    }

    async fn pruefen(&self, a: ServerAnfrage) -> Result<(String, String), String> {
        let w = &a.ausweis;
        if !zeit_gilt(w, jetzt()) {
            return Err("This identification has expired.".into());
        }
        let zustand = GETEILT.get().cloned().ok_or("Not started yet.")?;
        match ist_whiteboard(&a.notiz) {
            Some(wb) => {
                if crate::projektwhiteboard::eigenes(&zustand, &a.projekt, wb)
                    .await
                    .is_none()
                {
                    return Err("This whiteboard is not here.".into());
                }
            }
            None => {
                if !self.hat(&a.projekt, &a.freigabe, &a.notiz).await {
                    return Err("This note is not shared for editing.".into());
                }
            }
        }
        let client = projektchat::verbunden(&zustand).await?;
        let p = client
            .projekt_json(&format!("/projects/{}", a.projekt), &[])
            .await
            .map_err(|e| e.to_string())?;
        let mitglied = p.get("members").and_then(Value::as_array).is_some_and(|l| {
            l.iter()
                .any(|m| m.get("name").and_then(Value::as_str) == Some(w.name.as_str()))
        });
        if !mitglied {
            return Err("Not a member of this project.".into());
        }
        let ed = direktpost::aktives_geraet(&client, &w.name, w.geraet)
            .await
            .ok_or("This device is not in the key directory.")?;
        let ich = crate::geraet_identitaet(&zustand)?.fingerabdruck;
        openany_direktpost::ausweis_pruefen(
            &ed,
            &ich,
            &a.anrufer,
            &w.name,
            w.geraet,
            w.zeit,
            &w.signatur,
        )
        .map_err(|_| "The identification does not match.".to_string())?;
        Ok((format!("konto:{}", w.name), w.name.clone()))
    }

    async fn whiteboard(&self, projekt: &str, whiteboard: &str) -> Option<(String, String)> {
        let zustand = GETEILT.get().cloned()?;
        crate::projektwhiteboard::eigenes(&zustand, projekt, whiteboard).await
    }

    async fn whiteboard_speichern(
        &self,
        projekt: &str,
        whiteboard: &str,
        titel: Option<String>,
        inhalt: String,
    ) -> Result<(), String> {
        let zustand = GETEILT.get().cloned().ok_or("Not started yet.")?;
        crate::projektwhiteboard::fuer_sitzung_speichern(
            &zustand, projekt, whiteboard, titel, &inhalt,
        )
        .await
    }
}

/// Der eigene Ausweis fuer eine Leitung zu `ziel` (Fingerabdruck).
pub(crate) fn ausweis(zustand: &crate::Zustand, ziel: &str) -> Result<Ausweis, String> {
    let ich = direktpost::schluessel(zustand)?;
    let name = direktpost::mein_name(zustand).ok_or("This device is not set up yet.")?;
    let geraet = direktpost::mein_geraet(zustand).ok_or("This device is not set up yet.")?;
    let anrufer = crate::geraet_identitaet(zustand)?.fingerabdruck;
    let zeit = jetzt();
    Ok(Ausweis {
        signatur: openany_direktpost::ausweis_unterschreiben(
            &ich, ziel, &anrufer, &name, geraet, zeit,
        ),
        name,
        geraet,
        zeit,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ein_ausweis_gilt_fuenf_minuten_in_beide_richtungen() {
        let a = |zeit| Ausweis {
            zeit,
            ..Default::default()
        };
        assert!(zeit_gilt(&a(1000), 1000));
        assert!(zeit_gilt(&a(700), 1000));
        assert!(zeit_gilt(&a(1300), 1000));
        assert!(!zeit_gilt(&a(699), 1000));
        assert!(!zeit_gilt(&a(1301), 1000));
    }
}
