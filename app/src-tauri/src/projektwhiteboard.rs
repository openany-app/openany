//! Whiteboards in der Planung (docs/plan-whiteboard.md, W1).
//!
//! Beim Server liegt ein Whiteboard als zwei Chiffrate mit dem
//! Projektschluessel: Kopf (`{ titel }`) und Inhalt (die Zeichnung, wie
//! Excalidraw sie beschreibt). Beide sind Projektfelder
//! `whiteboard:<uuid>:kopf` bzw. `…:inhalt` und vom Geraet dessen
//! unterschrieben, der es angelegt hat -- nur er speichert.
//!
//! Was zuletzt da war, behaelt die Ablage: die Koepfe je Projekt (fuer die
//! Liste und den Zaehler ohne Netz) und jede geoeffnete Zeichnung.

use std::collections::HashMap;
use std::sync::Arc;

use openany_client::{Method, OpenanyClient, OpenanyError};
use openany_direktpost::{Kontoschluessel, Verschluesselt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::direktpost;
use crate::projektchat::{self, Verzeichnisse};
use crate::Zustand;

/// Was die Oberflaeche erkennt, wenn ein anderes eigenes Geraet zwischendurch
/// gespeichert hat.
const VERALTET: &str = "whiteboard-veraltet";

fn text(e: impl ToString) -> String {
    e.to_string()
}

fn feld(whiteboard: &str, teil: &str) -> String {
    format!("whiteboard:{whiteboard}:{teil}")
}

fn gueltig(uuid: &str) -> Result<&str, String> {
    uuid::Uuid::parse_str(uuid)
        .map(|_| uuid)
        .map_err(|_| "Not a valid id.".to_string())
}

#[derive(Serialize, Deserialize, Default)]
struct Ablage {
    /// Projekt → Koepfe, wie die Liste sie zeigt.
    projekte: HashMap<String, Vec<Value>>,
    /// Whiteboard → zuletzt geoeffnet (Kopf samt `inhalt`).
    offen: HashMap<String, Value>,
}

fn ablage_pfad(zustand: &Zustand) -> std::path::PathBuf {
    zustand.ordner.join("projektwhiteboards.json")
}

fn ablage_lesen(zustand: &Zustand) -> Ablage {
    std::fs::read(ablage_pfad(zustand))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

fn ablage_schreiben(zustand: &Zustand, a: &Ablage) {
    if let Ok(b) = serde_json::to_vec(a) {
        let _ = std::fs::write(ablage_pfad(zustand), b);
    }
}

/// Wie viele Whiteboards dieses Geraet zuletzt in einem Projekt gesehen hat
/// -- fuer den Zaehler „in der Planung" in der Projektliste.
pub(crate) fn anzahl(zustand: &Zustand, projekt: &str) -> usize {
    ablage_lesen(zustand)
        .projekte
        .get(projekt)
        .map_or(0, Vec::len)
}

/// Ein Teil (Kopf oder Inhalt) pruefen und oeffnen.
async fn teil_oeffnen(
    p: &Kontoschluessel,
    projekt: &str,
    verz: &mut Verzeichnisse<'_>,
    w: &Value,
    teil: &str,
) -> Option<Value> {
    let uuid = w.get("uuid")?.as_str()?;
    let von = w.get("von_name")?.as_str()?;
    let geraet = w.get("geraet")?.as_i64()?;
    let version = w.get("version")?.as_u64()?;
    let v = Verschluesselt {
        inhalt: w.get(teil)?.as_str()?.to_string(),
        signatur: w.get(format!("{teil}_signatur"))?.as_str()?.to_string(),
    };
    let ed = verz.ed25519(von, geraet).await?;
    let roh = openany_direktpost::projektfeld_oeffnen(
        p,
        projekt,
        &feld(uuid, teil),
        version as u32,
        &v,
        &ed,
    )
    .ok()?;
    serde_json::from_str(&roh).ok()
}

/// Die Form, die die Oberflaeche bekommt -- ohne Chiffrate.
fn anzeige(w: &Value, ich: &Value, kopf: &Value) -> Value {
    json!({
        "uuid": w.get("uuid"),
        "eigen": w.get("created_by").is_some() && w.get("created_by") == Some(ich),
        "von_name": w.get("von_name"),
        "titel": kopf.get("titel").and_then(Value::as_str).unwrap_or_default(),
        "stand": w.get("stand"),
        "created_at": w.get("created_at"),
        "geaendert": w.get("geaendert"),
    })
}

/// Alle Whiteboards eines Projekts -- frisch, oder ohne Netz aus der Ablage.
#[tauri::command]
pub async fn projekt_whiteboards(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
) -> Result<Vec<Value>, String> {
    let projekt = gueltig(&projekt)?.to_string();
    let frisch = async {
        let client = projektchat::verbunden(&zustand).await?;
        let liste = client
            .projekt_json(&format!("/projects/{projekt}/whiteboards"), &[])
            .await
            .map_err(text)?;
        let Some(p) = projektchat::schluessel(&zustand, &client, &projekt).await? else {
            return Ok(Vec::new());
        };
        let ich = liste.get("ich").cloned().unwrap_or(Value::Null);
        let mut verz = Verzeichnisse::neu(&client);
        let mut neu = Vec::new();
        for w in liste
            .get("whiteboards")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if let Some(kopf) = teil_oeffnen(&p, &projekt, &mut verz, w, "kopf").await {
                neu.push(anzeige(w, &ich, &kopf));
            }
        }
        Ok::<_, String>(neu)
    }
    .await;

    let mut ablage = ablage_lesen(&zustand);
    match frisch {
        Ok(liste) => {
            ablage.projekte.insert(projekt, liste.clone());
            ablage_schreiben(&zustand, &ablage);
            Ok(liste)
        }
        Err(e) => ablage.projekte.remove(&projekt).ok_or(e),
    }
}

/// Ein Whiteboard samt Zeichnung -- frisch, oder ohne Netz das zuletzt
/// geoeffnete (dann `offline: true`).
#[tauri::command]
pub async fn projekt_whiteboard(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    whiteboard: String,
) -> Result<Value, String> {
    oeffnen(&zustand, &projekt, &whiteboard).await
}

/// Ein Whiteboard oeffnen -- auch fuer den Knoten einer Sitzung (W3b).
pub(crate) async fn oeffnen(
    zustand: &Arc<Zustand>,
    projekt: &str,
    whiteboard: &str,
) -> Result<Value, String> {
    gueltig(projekt)?;
    gueltig(whiteboard)?;
    let frisch = async {
        let client = projektchat::verbunden(zustand).await?;
        let w = client
            .projekt_json(
                &format!("/projects/{projekt}/whiteboards/{whiteboard}"),
                &[],
            )
            .await
            .map_err(text)?;
        let p = projektchat::schluessel(zustand, &client, projekt)
            .await?
            .ok_or("This device has no key for this project yet.")?;
        let mut verz = Verzeichnisse::neu(&client);
        let kopf = teil_oeffnen(&p, projekt, &mut verz, &w, "kopf")
            .await
            .ok_or("This whiteboard could not be opened.")?;
        let inhalt = teil_oeffnen(&p, projekt, &mut verz, &w, "inhalt")
            .await
            .ok_or("This whiteboard could not be opened.")?;
        let ich = w.get("ich").cloned().unwrap_or(Value::Null);
        let mut a = anzeige(&w, &ich, &kopf);
        a["inhalt"] = inhalt;
        Ok::<_, String>(a)
    }
    .await;

    let mut ablage = ablage_lesen(zustand);
    match frisch {
        Ok(a) => {
            ablage.offen.insert(whiteboard.to_string(), a.clone());
            ablage_schreiben(zustand, &ablage);
            Ok(a)
        }
        Err(e) => {
            let mut a = ablage.offen.remove(whiteboard).ok_or(e)?;
            a["offline"] = json!(true);
            Ok(a)
        }
    }
}

/// Kopf und Inhalt verschluesseln -- in der Form, die der Server erwartet.
fn verschluesseln(
    ich: &openany_direktpost::Schluessel,
    p: &Kontoschluessel,
    projekt: &str,
    whiteboard: &str,
    titel: &str,
    inhalt: &Value,
) -> Result<Value, String> {
    let kopf = serde_json::to_string(&json!({ "titel": titel.trim() })).map_err(text)?;
    let (k, version) = openany_direktpost::projektfeld_verschluesseln(
        ich,
        p,
        projekt,
        &feld(whiteboard, "kopf"),
        &kopf,
    )
    .map_err(text)?;
    let roh = serde_json::to_string(inhalt).map_err(text)?;
    let (i, version_i) = openany_direktpost::projektfeld_verschluesseln(
        ich,
        p,
        projekt,
        &feld(whiteboard, "inhalt"),
        &roh,
    )
    .map_err(text)?;
    debug_assert_eq!(version, version_i);
    Ok(json!({
        "kopf": k.inhalt,
        "kopf_signatur": k.signatur,
        "inhalt": i.inhalt,
        "inhalt_signatur": i.signatur,
        "version": version,
    }))
}

async fn schluessel(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    projekt: &str,
) -> Result<(openany_direktpost::Schluessel, Kontoschluessel), String> {
    Ok((
        direktpost::schluessel(zustand)?,
        projektchat::schluessel(zustand, client, projekt)
            .await?
            .ok_or("This device has no key for this project yet.")?,
    ))
}

/// Anlegen: leer, mit Titel. Antwort: die uuid.
#[tauri::command]
pub async fn projekt_whiteboard_anlegen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    titel: String,
) -> Result<String, String> {
    gueltig(&projekt)?;
    let client = projektchat::verbunden(&zustand).await?;
    let (ich, p) = schluessel(&zustand, &client, &projekt).await?;
    let uuid = uuid::Uuid::new_v4().to_string();
    let mut koerper = verschluesseln(
        &ich,
        &p,
        &projekt,
        &uuid,
        &titel,
        &json!({ "elements": [] }),
    )?;
    koerper["uuid"] = json!(uuid);
    client
        .projekt_senden(
            Method::POST,
            &format!("/projects/{projekt}/whiteboards"),
            Some(koerper),
        )
        .await
        .map_err(text)?;
    Ok(uuid)
}

/// Speichern auf dem Stand, der gelesen wurde. Antwort: der neue Stand.
/// Hat ein anderes eigenes Geraet zwischendurch gespeichert: `VERALTET`.
#[tauri::command]
pub async fn projekt_whiteboard_speichern(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    whiteboard: String,
    titel: String,
    inhalt: Value,
    stand: u32,
) -> Result<u32, String> {
    speichern(&zustand, &projekt, &whiteboard, &titel, inhalt, stand).await
}

/// Ein EIGENES Whiteboard fuer eine Sitzung (W3b): (Titel, Zeichnung als
/// JSON). `None`, wenn es nicht diesem Konto gehoert.
pub(crate) async fn eigenes(
    zustand: &Arc<Zustand>,
    projekt: &str,
    whiteboard: &str,
) -> Option<(String, String)> {
    let w = oeffnen(zustand, projekt, whiteboard).await.ok()?;
    if w.get("eigen").and_then(Value::as_bool) != Some(true) {
        return None;
    }
    Some((
        w.get("titel")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        serde_json::to_string(w.get("inhalt").unwrap_or(&json!({ "elements": [] }))).ok()?,
    ))
}

/// Speichern fuer eine Sitzung (W3b): Was gezeichnet wurde, gilt -- also auf
/// dem neuesten Stand, den der Server kennt (einmal nachgelesen, falls ein
/// anderes eigenes Geraet dazwischen gespeichert hat).
pub(crate) async fn fuer_sitzung_speichern(
    zustand: &Arc<Zustand>,
    projekt: &str,
    whiteboard: &str,
    titel: Option<String>,
    inhalt: &str,
) -> Result<(), String> {
    let inhalt: Value = serde_json::from_str(inhalt).map_err(text)?;
    let mut w = ablage_lesen(zustand).offen.remove(whiteboard);
    if w.is_none() {
        w = Some(oeffnen(zustand, projekt, whiteboard).await?);
    }
    for versuch in 0..2 {
        let a = w.as_ref().ok_or("This whiteboard could not be opened.")?;
        let titel = titel.clone().unwrap_or_else(|| {
            a.get("titel")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string()
        });
        let stand = a.get("stand").and_then(Value::as_u64).unwrap_or(1) as u32;
        match speichern(zustand, projekt, whiteboard, &titel, inhalt.clone(), stand).await {
            Err(e) if e == VERALTET && versuch == 0 => {
                w = Some(oeffnen(zustand, projekt, whiteboard).await?);
            }
            r => return r.map(|_| ()),
        }
    }
    Err(VERALTET.into())
}

/// Speichern auf dem Stand, der gelesen wurde (siehe den Befehl).
pub(crate) async fn speichern(
    zustand: &Arc<Zustand>,
    projekt: &str,
    whiteboard: &str,
    titel: &str,
    inhalt: Value,
    stand: u32,
) -> Result<u32, String> {
    gueltig(projekt)?;
    gueltig(whiteboard)?;
    let client = projektchat::verbunden(zustand).await?;
    let (ich, p) = schluessel(zustand, &client, projekt).await?;
    let mut koerper = verschluesseln(&ich, &p, projekt, whiteboard, titel, &inhalt)?;
    koerper["stand"] = json!(stand);
    let antwort = client
        .projekt_senden(
            Method::PUT,
            &format!("/projects/{projekt}/whiteboards/{whiteboard}"),
            Some(koerper),
        )
        .await
        .map_err(|e| match e {
            OpenanyError::Abgelehnt { status, .. } if status.as_u16() == 409 => {
                VERALTET.to_string()
            }
            e => text(e),
        })?;
    let neu = antwort
        .get("stand")
        .and_then(Value::as_u64)
        .map_or(stand + 1, |s| s as u32);

    // Die Ablage gleich mitziehen: wer offline wieder oeffnet, sieht das
    // Gespeicherte.
    let mut ablage = ablage_lesen(zustand);
    if let Some(a) = ablage.offen.get_mut(whiteboard) {
        a["titel"] = json!(titel.trim());
        a["inhalt"] = inhalt;
        a["stand"] = json!(neu);
        ablage_schreiben(zustand, &ablage);
    }
    Ok(neu)
}

/// Das Whiteboard loeschen.
#[tauri::command]
pub async fn projekt_whiteboard_loeschen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    whiteboard: String,
) -> Result<(), String> {
    gueltig(&projekt)?;
    gueltig(&whiteboard)?;
    let client = projektchat::verbunden(&zustand).await?;
    client
        .projekt_senden(
            Method::DELETE,
            &format!("/projects/{projekt}/whiteboards/{whiteboard}"),
            None,
        )
        .await
        .map_err(text)?;
    let mut ablage = ablage_lesen(&zustand);
    ablage.offen.remove(&whiteboard);
    if let Some(l) = ablage.projekte.get_mut(&projekt) {
        l.retain(|w| w.get("uuid").and_then(Value::as_str) != Some(whiteboard.as_str()));
    }
    ablage_schreiben(&zustand, &ablage);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn das_feld_bindet_whiteboard_und_teil() {
        assert_eq!(feld("w-1", "kopf"), "whiteboard:w-1:kopf");
        assert_ne!(feld("w-1", "kopf"), feld("w-1", "inhalt"));
    }

    #[test]
    fn eigen_nur_bei_gleicher_kennung() {
        let w = json!({ "uuid": "u", "created_by": 7, "stand": 3 });
        let kopf = json!({ "titel": "Skizze" });
        assert_eq!(anzeige(&w, &json!(7), &kopf)["eigen"], json!(true));
        assert_eq!(anzeige(&w, &json!(8), &kopf)["eigen"], json!(false));
        assert_eq!(
            anzeige(&json!({}), &Value::Null, &kopf)["eigen"],
            json!(false)
        );
        assert_eq!(anzeige(&w, &json!(7), &kopf)["titel"], json!("Skizze"));
    }
}
