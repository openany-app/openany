//! Das Board eines Hubs (docs/plan-whiteboard.md; eines je Hub).
//!
//! **Eines, ohne Anlegen.** Jeder Hub hat genau ein Board; seine Kennung
//! folgt aus der des Hubs ([`board_id`]). Es entsteht mit dem Hub, ohne dass
//! jemand es anlegt, und laesst sich nicht loeschen.
//!
//! **Alle zeichnen.** Jedes Mitglied speichert seine Fassung der Zeichnung
//! als eigenen Eintrag unter Planung (`planungseinheit.rs`) -- je Person
//! einer, mit fester Kennung, damit eigene Geraete denselben schreiben. Die
//! Fassungen reisen wie alle Eintraege, vor Ort und mit Wolke.
//!
//! **Zusammengefuehrt wird je Element** ([`zusammenfuehren`]), wie Excalidraw
//! es selbst tut: Die hoehere `version` gewinnt, bei Gleichstand die
//! kleinere `versionNonce`. Geloeschte Elemente bleiben als Grabstein
//! (`isDeleted`) in der Fassung, sonst kaeme ein geloeschtes Element aus
//! einer aelteren Fassung zurueck.

use std::collections::HashMap;
use std::sync::Arc;

use openany_nahbereich::planung::Eintrag;
use serde_json::{json, Map, Value};

use crate::planungseinheit;
use crate::Zustand;

/// Die Sorte der Eintraege -- `whiteboard`, damit auch Geraete mit einer
/// aelteren App sie vor Ort annehmen. Ein Board hat keine Anlage.
const SORTE: &str = "whiteboard";

fn gueltig(uuid: &str) -> Result<&str, String> {
    uuid::Uuid::parse_str(uuid)
        .map(|_| uuid)
        .map_err(|_| "Not a valid id.".to_string())
}

/// Die Kennung des Boards eines Hubs.
pub(crate) fn board_id(projekt: &str) -> String {
    uuid::Uuid::new_v5(
        &uuid::Uuid::NAMESPACE_URL,
        format!("openany-board:{projekt}").as_bytes(),
    )
    .to_string()
}

/// Die Kennung der Fassung einer Person.
fn fassung_id(board: &str, person: &str) -> String {
    uuid::Uuid::new_v5(
        &uuid::Uuid::NAMESPACE_URL,
        format!("openany-board:{board}|{person}").as_bytes(),
    )
    .to_string()
}

fn zahl(e: &Value, feld: &str) -> i64 {
    e.get(feld).and_then(Value::as_i64).unwrap_or(0)
}

/// Gewinnt `neu` gegen `alt`? Wie Excalidraw: hoehere Version, bei
/// Gleichstand die kleinere Nonce.
fn gewinnt(neu: &Value, alt: &Value) -> bool {
    let (vn, va) = (zahl(neu, "version"), zahl(alt, "version"));
    vn > va || (vn == va && zahl(neu, "versionNonce") < zahl(alt, "versionNonce"))
}

/// Mehrere Fassungen zu einer: je Element-id das gewinnende, in der
/// Reihenfolge der ersten Fassung, die es nennt (die Reihenfolge ist die
/// Ebene). Grabsteine bleiben drin.
pub(crate) fn zusammenfuehren<'a>(fassungen: impl IntoIterator<Item = &'a [Value]>) -> Vec<Value> {
    let mut aus: Vec<Value> = Vec::new();
    let mut wo: HashMap<String, usize> = HashMap::new();
    for fassung in fassungen {
        for e in fassung {
            let Some(id) = e.get("id").and_then(Value::as_str) else {
                continue;
            };
            match wo.get(id) {
                Some(&i) => {
                    if gewinnt(e, &aus[i]) {
                        aus[i] = e.clone();
                    }
                }
                None => {
                    wo.insert(id.to_string(), aus.len());
                    aus.push(e.clone());
                }
            }
        }
    }
    aus
}

/// Ein Grabstein traegt nur, was zum Vergleichen noetig ist.
fn grabstein(e: &Value) -> Value {
    let mut g = Map::new();
    for feld in ["id", "type", "version", "versionNonce", "updated"] {
        if let Some(v) = e.get(feld) {
            g.insert(feld.into(), v.clone());
        }
    }
    g.insert("isDeleted".into(), Value::Bool(true));
    Value::Object(g)
}

fn geloescht(e: &Value) -> bool {
    e.get("isDeleted").and_then(Value::as_bool) == Some(true)
}

fn elemente(inhalt: &Value) -> Vec<Value> {
    inhalt
        .get("elements")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

/// Die Fassungen aller Mitglieder, die neueste zuerst -- und wann zuletzt
/// gespeichert wurde.
fn fassungen(
    speicher: &openany_store::Speicher,
    projekt: &str,
) -> Result<(Vec<Vec<Value>>, String), String> {
    let board = board_id(projekt);
    let mut alle: Vec<Eintrag> =
        openany_nahbereich::projektnah::planung_eintraege_lesen(speicher, projekt)?
            .into_iter()
            .filter(|e| e.sorte == SORTE && e.sache == board && !e.geloescht)
            .collect();
    alle.sort_by(|a, b| b.at.cmp(&a.at));
    let zuletzt = alle.first().map(|e| e.at.clone()).unwrap_or_default();
    Ok((
        alle.iter()
            .filter_map(|e| e.wert())
            .map(|w| elemente(&w))
            .collect(),
        zuletzt,
    ))
}

/// Das Board eines Hubs: `{ uuid, inhalt: { elements }, geaendert }` --
/// ohne Grabsteine.
#[tauri::command]
pub async fn projekt_board(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
) -> Result<Value, String> {
    gueltig(&projekt)?;
    planungseinheit::vorher_abgleichen(&zustand, &projekt).await;
    oeffnen(&zustand, &projekt).await
}

pub(crate) async fn oeffnen(zustand: &Arc<Zustand>, projekt: &str) -> Result<Value, String> {
    gueltig(projekt)?;
    let speicher = zustand.speicher.lock().await;
    // Nur Mitglieder sehen das Board.
    planungseinheit::lage(zustand, &speicher, projekt)?;
    let (f, geaendert) = fassungen(&speicher, projekt)?;
    let sichtbar: Vec<Value> = zusammenfuehren(f.iter().map(Vec::as_slice))
        .into_iter()
        .filter(|e| !geloescht(e))
        .collect();
    Ok(json!({
        "uuid": board_id(projekt),
        "inhalt": { "elements": sichtbar },
        "geaendert": geaendert,
    }))
}

/// Speichern: die eigene Fassung -- das, was hier gezeichnet ist, mit allen
/// anderen Fassungen zusammengefuehrt.
#[tauri::command]
pub async fn projekt_board_speichern(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    inhalt: Value,
) -> Result<(), String> {
    speichern(&zustand, &projekt, inhalt).await
}

pub(crate) async fn speichern(
    zustand: &Arc<Zustand>,
    projekt: &str,
    inhalt: Value,
) -> Result<(), String> {
    gueltig(projekt)?;
    let board = board_id(projekt);
    let (zusammen, ich) = {
        let speicher = zustand.speicher.lock().await;
        let l = planungseinheit::lage(zustand, &speicher, projekt)?;
        let (mut f, _) = fassungen(&speicher, projekt)?;
        f.insert(0, elemente(&inhalt));
        (
            zusammenfuehren(f.iter().map(Vec::as_slice)),
            l.ich.personen_id,
        )
    };
    let fassung: Vec<Value> = zusammen
        .iter()
        .map(|e| {
            if geloescht(e) {
                grabstein(e)
            } else {
                e.clone()
            }
        })
        .collect();
    planungseinheit::schreiben(
        zustand,
        projekt,
        SORTE,
        &board,
        &fassung_id(&board, &ich),
        Some(&json!({ "elements": fassung })),
        false,
    )
    .await
    .map(|_| ())
}

/// Das Board fuer eine Sitzung (W3b): (Titel, Zeichnung als JSON). Jedes
/// Mitglied darf -- `None`, wenn es keines ist oder `board` nicht das Board
/// dieses Hubs.
pub(crate) async fn eigenes(
    zustand: &Arc<Zustand>,
    projekt: &str,
    board: &str,
) -> Option<(String, String)> {
    if board != board_id(projekt) {
        return None;
    }
    let b = oeffnen(zustand, projekt).await.ok()?;
    Some((
        String::new(),
        serde_json::to_string(b.get("inhalt").unwrap_or(&json!({ "elements": [] }))).ok()?,
    ))
}

/// Speichern fuer eine Sitzung (W3b).
pub(crate) async fn fuer_sitzung_speichern(
    zustand: &Arc<Zustand>,
    projekt: &str,
    board: &str,
    _titel: Option<String>,
    inhalt: &str,
) -> Result<(), String> {
    if board != board_id(projekt) {
        return Err("This is not the board of this hub.".into());
    }
    let inhalt: Value = serde_json::from_str(inhalt).map_err(|e| e.to_string())?;
    speichern(zustand, projekt, inhalt).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn el(id: &str, version: i64, nonce: i64) -> Value {
        json!({ "id": id, "type": "rectangle", "version": version, "versionNonce": nonce, "x": version })
    }

    #[test]
    fn jeder_hub_hat_ein_festes_board_und_jede_person_eine_fassung() {
        assert_eq!(board_id("p-1"), board_id("p-1"));
        assert_ne!(board_id("p-1"), board_id("p-2"));
        let b = board_id("p-1");
        assert_ne!(fassung_id(&b, "anna"), fassung_id(&b, "ben"));
        assert!(uuid::Uuid::parse_str(&fassung_id(&b, "anna")).is_ok());
    }

    #[test]
    fn die_hoehere_version_gewinnt_bei_gleichstand_die_kleinere_nonce() {
        let a = vec![el("x", 2, 50), el("y", 1, 1)];
        let b = vec![el("x", 3, 99), el("y", 1, 0), el("z", 1, 1)];
        let z = zusammenfuehren([a.as_slice(), b.as_slice()]);
        let ids: Vec<&str> = z.iter().map(|e| e["id"].as_str().unwrap()).collect();
        assert_eq!(ids, ["x", "y", "z"]);
        assert_eq!(z[0]["version"], 3);
        assert_eq!(z[1]["versionNonce"], 0);
    }

    #[test]
    fn ein_grabstein_haelt_ein_geloeschtes_element_fern() {
        let alt = vec![el("x", 1, 1)];
        let geloescht_hier = vec![grabstein(
            &json!({ "id": "x", "type": "rectangle", "version": 2, "versionNonce": 7 }),
        )];
        let z = zusammenfuehren([alt.as_slice(), geloescht_hier.as_slice()]);
        assert!(geloescht(&z[0]));
        assert_eq!(z[0].as_object().unwrap().len(), 5);
    }
}
