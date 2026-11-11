//! Meldungen ohne Konto, fuer Verwalter (docs/plan-registrierung-app.md, A5).
//!
//! Die Meldungen kommen von der Seite openany.de/melden, verschluesselt an
//! die Konto-Schluessel aller Verwalter und an die meldende Person. Hier
//! werden sie mit dem Konto-Schluessel dieses Geraets gelesen; Antworten
//! gehen an dieselben Empfaenger.

use std::sync::Arc;

use serde::Serialize;
use serde_json::Value;

use crate::{direktpost, Zustand};

#[derive(Serialize)]
pub struct Meldungsnachricht {
    von: String,
    at: String,
    /// Der entschluesselte Inhalt (JSON), oder `None`, wenn es nicht geht.
    inhalt: Option<Value>,
}

#[derive(Serialize)]
pub struct Meldungsanzeige {
    kennung: String,
    stand: String,
    eingegangen_at: String,
    letzte_nachricht_at: String,
    nachrichten: Vec<Meldungsnachricht>,
}

fn text(v: &Value, feld: &str) -> String {
    v.get(feld)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn anzeigen(konto: &openany_direktpost::Kontoschluessel, m: &Value) -> Meldungsanzeige {
    Meldungsanzeige {
        kennung: text(m, "kennung"),
        stand: text(m, "stand"),
        eingegangen_at: text(m, "eingegangen_at"),
        letzte_nachricht_at: text(m, "letzte_nachricht_at"),
        nachrichten: m
            .get("nachrichten")
            .and_then(Value::as_array)
            .map(|liste| {
                liste
                    .iter()
                    .map(|n| Meldungsnachricht {
                        von: text(n, "von"),
                        at: text(n, "at"),
                        inhalt: konto
                            .entschluesseln_b64(&text(n, "chiffrat"))
                            .ok()
                            .and_then(|b| serde_json::from_slice(&b).ok()),
                    })
                    .collect()
            })
            .unwrap_or_default(),
    }
}

async fn verbunden(zustand: &Arc<Zustand>) -> Result<openany_client::OpenanyClient, String> {
    crate::server_client(zustand)
        .await
        .ok_or_else(|| "This device is not connected.".to_string())
}

/// Die Meldungen -- `None`, wenn dieses Konto keine sehen darf (kein
/// Verwalter) oder keine Verbindung besteht. Dann zeigt die App nichts.
#[tauri::command]
pub async fn meldungen_lesen(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Option<Vec<Meldungsanzeige>>, String> {
    let Some(client) = crate::server_client(&zustand).await else {
        return Ok(None);
    };
    let Ok(antwort) = client.meldungen().await else {
        return Ok(None);
    };
    let konto = direktpost::konto(&zustand).ok_or("This device has no account key yet.")?;
    Ok(Some(
        antwort
            .get("meldungen")
            .and_then(Value::as_array)
            .map(|l| l.iter().map(|m| anzeigen(&konto, m)).collect())
            .unwrap_or_default(),
    ))
}

/// Antworten: an die meldende Person und alle Verwalter.
#[tauri::command]
pub async fn meldung_antworten(
    zustand: tauri::State<'_, Arc<Zustand>>,
    kennung: String,
    antwort: String,
) -> Result<Meldungsanzeige, String> {
    let client = verbunden(&zustand).await?;
    let konto = direktpost::konto(&zustand).ok_or("This device has no account key yet.")?;
    let liste = client.meldungen().await.map_err(|e| e.to_string())?;
    let meldung = liste
        .get("meldungen")
        .and_then(Value::as_array)
        .and_then(|l| l.iter().find(|m| text(m, "kennung") == kennung))
        .ok_or("Unknown report.")?;
    let mut empfaenger: Vec<String> = liste
        .get("empfaenger")
        .and_then(Value::as_array)
        .map(|l| {
            l.iter()
                .filter_map(|e| e.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    empfaenger.push(text(meldung, "antwort_pub"));
    empfaenger.push(konto.oeffentlich());

    let klartext = serde_json::to_vec(&serde_json::json!({ "text": antwort.trim() }))
        .map_err(|e| e.to_string())?;
    let chiffrat =
        openany_direktpost::an_empfaenger_b64(&empfaenger, &klartext).map_err(|e| e.to_string())?;
    let neu = client
        .meldung_antworten(&kennung, &chiffrat)
        .await
        .map_err(|e| e.to_string())?;
    Ok(anzeigen(&konto, &neu))
}

#[tauri::command]
pub async fn meldung_stand_setzen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    kennung: String,
    stand: String,
) -> Result<Meldungsanzeige, String> {
    let client = verbunden(&zustand).await?;
    let konto = direktpost::konto(&zustand).ok_or("This device has no account key yet.")?;
    let neu = client
        .meldung_stand(&kennung, &stand)
        .await
        .map_err(|e| e.to_string())?;
    Ok(anzeigen(&konto, &neu))
}
