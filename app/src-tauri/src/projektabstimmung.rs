//! Abstimmungen Ende-zu-Ende (docs/plan-abstimmungen-e2e.md, 5b).
//!
//! Eine Abstimmung ist beim Server eine Liste verschluesselter Eintraege
//! (Kopf, Optionen, Stimmen, Erledigt). Hier wird nur geholt, geprueft,
//! entschluesselt und verschluesselt -- **was** ein Eintrag bedeutet und
//! welche Regeln gelten (Kapazitaet, eine Stimme, Einladungen, Ablauf),
//! entscheidet die Oberflaeche (`abstimmungBauen.js`), fuer alle Geraete
//! gleich.
//!
//! Jeder Eintrag ist ein Projektfeld `umfrage:<abstimmung>:<eintrag>`: Er
//! laesst sich weder in eine andere Abstimmung noch in ein anderes Projekt
//! schieben. Wer ihn schrieb, sagt der Server (`von`); dass das stimmt,
//! zeigt die Unterschrift eines Geraets genau dieses Kontos.
//!
//! **Feste uuids** fuer Eintraege, die es je Person nur einmal gibt (die
//! Stimme zu einer Option, der Kopf): Die Oberflaeche nennt einen
//! `schluessel`, die uuid folgt daraus und aus dem eigenen Namen. So
//! schreiben zwei eigene Geraete denselben Eintrag statt zwei.

use std::collections::HashMap;
use std::sync::Arc;

use openany_client::{Method, OpenanyClient};
use openany_direktpost::{Kontoschluessel, Verschluesselt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::direktpost;
use crate::projektchat::{self, Verzeichnisse};
use crate::Zustand;

fn text(e: impl ToString) -> String {
    e.to_string()
}

fn feld(umfrage: &str, eintrag: &str) -> String {
    format!("umfrage:{umfrage}:{eintrag}")
}

fn gueltig(uuid: &str) -> Result<&str, String> {
    uuid::Uuid::parse_str(uuid)
        .map(|_| uuid)
        .map_err(|_| "Not a valid id.".to_string())
}

/// Die uuid eines Eintrags, den es je Person nur einmal gibt.
fn feste_uuid(umfrage: &str, schluessel: &str, ich: &str) -> String {
    uuid::Uuid::new_v5(
        &uuid::Uuid::NAMESPACE_URL,
        format!("openany-umfrage:{umfrage}|{schluessel}|{ich}").as_bytes(),
    )
    .to_string()
}

/// Was die Ablage von einer Abstimmung behaelt: geoeffnet, mit Stand.
#[derive(Serialize, Deserialize, Clone)]
struct Geoeffnet {
    stand: String,
    abstimmung: Value,
}

#[derive(Serialize, Deserialize, Default)]
struct Ablage {
    /// Projekt → (ich, mitglieder, Abstimmungen nach uuid).
    projekte: HashMap<String, AblageProjekt>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
struct AblageProjekt {
    ich: Value,
    mitglieder: Value,
    abstimmungen: Vec<(String, Geoeffnet)>,
}

fn ablage_pfad(zustand: &Zustand) -> std::path::PathBuf {
    zustand.ordner.join("projektabstimmungen.json")
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

/// Wie viele Abstimmungen dieses Geraet zuletzt in einem Projekt gesehen hat
/// -- fuer den Zaehler „in der Planung" in der Projektliste.
pub(crate) fn anzahl(zustand: &Zustand, projekt: &str) -> usize {
    ablage_lesen(zustand)
        .projekte
        .get(projekt)
        .map_or(0, |p| p.abstimmungen.len())
}

/// Eine Abstimmung vom Server oeffnen: jeder Eintrag geprueft und
/// entschluesselt; was nicht aufgeht, faellt weg.
async fn oeffnen(
    p: &Kontoschluessel,
    projekt: &str,
    verz: &mut Verzeichnisse<'_>,
    voll: &Value,
) -> Value {
    let umfrage = voll
        .get("uuid")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let mut eintraege = Vec::new();
    for e in voll
        .get("eintraege")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let (Some(uuid), Some(von), Some(geraet), Some(version)) = (
            e.get("uuid").and_then(Value::as_str),
            e.get("von_name").and_then(Value::as_str),
            e.get("geraet").and_then(Value::as_i64),
            e.get("version").and_then(Value::as_u64),
        ) else {
            continue;
        };
        let v = Verschluesselt {
            inhalt: e
                .get("inhalt")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            signatur: e
                .get("signatur")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
        };
        let Some(ed) = verz.ed25519(von, geraet).await else {
            continue;
        };
        let Ok(roh) = openany_direktpost::projektfeld_oeffnen(
            p,
            projekt,
            &feld(&umfrage, uuid),
            version as u32,
            &v,
            &ed,
        ) else {
            continue;
        };
        let Ok(inhalt) = serde_json::from_str::<Value>(&roh) else {
            continue;
        };
        eintraege.push(json!({
            "nr": e.get("nr"),
            "uuid": uuid,
            "von": e.get("von"),
            "von_name": von,
            "zeit": e.get("zeit"),
            "geaendert": e.get("geaendert"),
            "inhalt": inhalt,
        }));
    }
    json!({
        "uuid": umfrage,
        "created_by": voll.get("created_by"),
        "created_at": voll.get("created_at"),
        "stand": voll.get("stand"),
        "eintraege": eintraege,
    })
}

/// Alle Abstimmungen eines Projekts, frisch -- oder aus der Ablage, wenn das
/// Netz fehlt. Was sich nicht geaendert hat (`stand`), wird nicht neu geholt.
#[tauri::command]
pub async fn projekt_abstimmungen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
) -> Result<Value, String> {
    let projekt = gueltig(&projekt)?.to_string();
    let mut ablage = ablage_lesen(&zustand);
    let alt = ablage.projekte.remove(&projekt).unwrap_or_default();

    let frisch = async {
        let client = projektchat::verbunden(&zustand).await?;
        let liste = client
            .projekt_json(&format!("/projects/{projekt}/polls"), &[])
            .await
            .map_err(text)?;
        let p = projektchat::schluessel(&zustand, &client, &projekt).await?;
        let mut verz = Verzeichnisse::neu(&client);
        let vorher: HashMap<&str, &Geoeffnet> = alt
            .abstimmungen
            .iter()
            .map(|(u, g)| (u.as_str(), g))
            .collect();
        let mut neu = Vec::new();
        for k in liste
            .get("polls")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let (Some(uuid), Some(stand)) = (
                k.get("uuid").and_then(Value::as_str),
                k.get("stand").and_then(Value::as_str),
            ) else {
                continue;
            };
            if let Some(g) = vorher.get(uuid).filter(|g| g.stand == stand) {
                neu.push((uuid.to_string(), (*g).clone()));
                continue;
            }
            let Some(p) = &p else { continue };
            let voll = client
                .projekt_json(&format!("/projects/{projekt}/polls/{uuid}"), &[])
                .await
                .map_err(text)?;
            neu.push((
                uuid.to_string(),
                Geoeffnet {
                    stand: stand.to_string(),
                    abstimmung: oeffnen(p, &projekt, &mut verz, &voll).await,
                },
            ));
        }
        Ok::<_, String>(AblageProjekt {
            ich: liste.get("ich").cloned().unwrap_or(Value::Null),
            mitglieder: liste.get("mitglieder").cloned().unwrap_or(json!([])),
            abstimmungen: neu,
        })
    }
    .await;

    let stand = match frisch {
        Ok(s) => {
            ablage.projekte.insert(projekt.clone(), s.clone());
            ablage_schreiben(&zustand, &ablage);
            s
        }
        // Ohne Netz: was zuletzt da war. Ohne Ablage: der Fehler.
        Err(e) if alt.abstimmungen.is_empty() => return Err(e),
        Err(_) => alt,
    };
    Ok(json!({
        "ich": stand.ich,
        "mitglieder": stand.mitglieder,
        "polls": stand.abstimmungen.into_iter().map(|(_, g)| g.abstimmung).collect::<Vec<_>>(),
    }))
}

/// Eine Abstimmung, frisch vom Server.
#[tauri::command]
pub async fn projekt_abstimmung(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    umfrage: String,
) -> Result<Value, String> {
    gueltig(&projekt)?;
    gueltig(&umfrage)?;
    let client = projektchat::verbunden(&zustand).await?;
    let voll = client
        .projekt_json(&format!("/projects/{projekt}/polls/{umfrage}"), &[])
        .await
        .map_err(text)?;
    let p = projektchat::schluessel(&zustand, &client, &projekt)
        .await?
        .ok_or("This device has no key for this project yet.")?;
    let mut verz = Verzeichnisse::neu(&client);
    let mut a = oeffnen(&p, &projekt, &mut verz, &voll).await;
    a["ich"] = voll.get("ich").cloned().unwrap_or(Value::Null);
    a["mitglieder"] = voll.get("mitglieder").cloned().unwrap_or(json!([]));
    Ok(a)
}

/// Ein Eintrag zum Schreiben: Inhalt (JSON) und wahlweise ein fester
/// Schluessel (Stimme zu einer Option, Kopf ...).
#[derive(Deserialize)]
pub struct Neu {
    #[serde(default)]
    schluessel: Option<String>,
    inhalt: Value,
}

struct Schreiber {
    ich: openany_direktpost::Schluessel,
    p: Kontoschluessel,
    name: String,
}

impl Schreiber {
    fn uuid(&self, umfrage: &str, schluessel: Option<&str>) -> String {
        match schluessel {
            Some(s) => feste_uuid(umfrage, s, &self.name),
            None => uuid::Uuid::new_v4().to_string(),
        }
    }

    fn eintrag(
        &self,
        projekt: &str,
        umfrage: &str,
        uuid: &str,
        inhalt: &Value,
    ) -> Result<Value, String> {
        let roh = serde_json::to_string(inhalt).map_err(text)?;
        let (v, version) = openany_direktpost::projektfeld_verschluesseln(
            &self.ich,
            &self.p,
            projekt,
            &feld(umfrage, uuid),
            &roh,
        )
        .map_err(text)?;
        Ok(json!({ "uuid": uuid, "inhalt": v.inhalt, "signatur": v.signatur, "version": version }))
    }
}

async fn schreiber(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    projekt: &str,
) -> Result<Schreiber, String> {
    Ok(Schreiber {
        ich: direktpost::schluessel(zustand)?,
        p: projektchat::schluessel(zustand, client, projekt)
            .await?
            .ok_or("This device has no key for this project yet.")?,
        name: direktpost::mein_name(zustand).ok_or("This device is not set up yet.")?,
    })
}

/// Anlegen: Kopf und Optionen in einem. `einladen`: Mitglieder, die einen
/// Hinweis bekommen (ohne Inhalt). Antwort: die uuid.
#[tauri::command]
pub async fn projekt_abstimmung_anlegen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    eintraege: Vec<Neu>,
    benachrichtigen: bool,
    einladen: Vec<i64>,
) -> Result<String, String> {
    gueltig(&projekt)?;
    let client = projektchat::verbunden(&zustand).await?;
    let s = schreiber(&zustand, &client, &projekt).await?;
    let umfrage = uuid::Uuid::new_v4().to_string();
    let liste = eintraege
        .iter()
        .map(|e| {
            let uuid = s.uuid(&umfrage, e.schluessel.as_deref());
            s.eintrag(&projekt, &umfrage, &uuid, &e.inhalt)
        })
        .collect::<Result<Vec<_>, _>>()?;
    client
        .projekt_senden(
            Method::POST,
            &format!("/projects/{projekt}/polls"),
            Some(json!({ "uuid": umfrage, "eintraege": liste, "notify_members": benachrichtigen, "einladen": einladen })),
        )
        .await
        .map_err(text)?;
    Ok(umfrage)
}

/// Einen Eintrag schreiben oder aendern (`uuid` oder fester `schluessel`).
#[tauri::command]
pub async fn projekt_abstimmung_schreiben(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    umfrage: String,
    uuid: Option<String>,
    schluessel: Option<String>,
    inhalt: Value,
) -> Result<(), String> {
    gueltig(&projekt)?;
    gueltig(&umfrage)?;
    let client = projektchat::verbunden(&zustand).await?;
    let s = schreiber(&zustand, &client, &projekt).await?;
    let uuid = match uuid {
        Some(u) => gueltig(&u)?.to_string(),
        None => s.uuid(&umfrage, schluessel.as_deref()),
    };
    let e = s.eintrag(&projekt, &umfrage, &uuid, &inhalt)?;
    client
        .projekt_senden(
            Method::PUT,
            &format!("/projects/{projekt}/polls/{umfrage}/eintraege/{uuid}"),
            Some(e),
        )
        .await
        .map_err(text)?;
    Ok(())
}

/// Einen Eintrag loeschen -- per uuid oder ueber den festen Schluessel.
#[tauri::command]
pub async fn projekt_abstimmung_eintrag_loeschen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    umfrage: String,
    uuid: Option<String>,
    schluessel: Option<String>,
) -> Result<(), String> {
    gueltig(&projekt)?;
    gueltig(&umfrage)?;
    let client = projektchat::verbunden(&zustand).await?;
    let uuid = match (uuid, schluessel) {
        (Some(u), _) => gueltig(&u)?.to_string(),
        (None, Some(k)) => feste_uuid(
            &umfrage,
            &k,
            &direktpost::mein_name(&zustand).ok_or("This device is not set up yet.")?,
        ),
        (None, None) => return Err("No entry given.".into()),
    };
    client
        .projekt_senden(
            Method::DELETE,
            &format!("/projects/{projekt}/polls/{umfrage}/eintraege/{uuid}"),
            None,
        )
        .await
        .map_err(text)?;
    Ok(())
}

/// Die ganze Abstimmung loeschen.
#[tauri::command]
pub async fn projekt_abstimmung_loeschen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    umfrage: String,
) -> Result<(), String> {
    gueltig(&projekt)?;
    gueltig(&umfrage)?;
    let client = projektchat::verbunden(&zustand).await?;
    client
        .projekt_senden(
            Method::DELETE,
            &format!("/projects/{projekt}/polls/{umfrage}"),
            None,
        )
        .await
        .map_err(text)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feste_uuid_je_person_und_schluessel() {
        let a = feste_uuid("u-1", "stimme|o-1", "anna");
        assert_eq!(a, feste_uuid("u-1", "stimme|o-1", "anna"));
        assert_ne!(a, feste_uuid("u-1", "stimme|o-1", "ben"));
        assert_ne!(a, feste_uuid("u-1", "stimme|o-2", "anna"));
        assert_ne!(a, feste_uuid("u-2", "stimme|o-1", "anna"));
        assert!(uuid::Uuid::parse_str(&a).is_ok());
    }

    #[test]
    fn das_feld_bindet_abstimmung_und_eintrag() {
        assert_eq!(feld("u-1", "e-1"), "umfrage:u-1:e-1");
    }
}
