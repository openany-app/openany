//! Ein Chat auf beiden Wegen (docs/plan-projekte-eine-art.md).
//!
//! **Eine Form.** Jede Nachricht ist die vom Geraet unterschriebene
//! Vor-Ort-Nachricht (`openany_nahbereich::chat::Nachricht`), jedes
//! Anheften, Loesen, Loeschen eine ebenso unterschriebene Aktion. Beide
//! liegen immer hier, als Projektsachen -- auch in Projekten mit Wolke. Der
//! Chat ueberlebt damit jedes Umschalten.
//!
//! **Zwei Wege.** Vor Ort gehen sie sofort an Mitgliedsgeraete in der Naehe
//! und reisen im Stand. Mit Wolke gehen sie zusaetzlich verschluesselt zu
//! openany.de: mit dem Projektschluessel, und mit ihrer uuid als Kennung,
//! damit dieselbe Nachricht nur einmal hochkommt.
//!
//! **Die Bruecke.** Hochgeladen wird ALLES, was hier liegt und dort fehlt --
//! auch Nachrichten von Mitgliedern ohne Konto, die dieses Geraet vor Ort
//! bekam. Was vom Server kommt, reicht dieses Geraet vor Ort weiter. Weil
//! jede Nachricht vom Geraet ihrer Absenderin unterschrieben ist, kann
//! dabei niemand etwas faelschen.
//!
//! **Nachrichten in der alten Form** (nur Text) werden hier nicht gezeigt.

use std::collections::BTreeSet;
use std::sync::Arc;

use openany_client::{Method, OpenanyClient};
use openany_nahbereich::chat::{Aktion, Nachricht, ANHEFTEN, LOESCHEN, LOESEN};
use openany_nahbereich::mitglieder::EIGENTUEMER;
use openany_nahbereich::{projektnah, Gastgeber};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{direktpost, projektchat, Zustand};

fn text(e: impl ToString) -> String {
    e.to_string()
}

/* ── Was die Ansicht bekommt ────────────────────────────────────────────── */

#[derive(Debug, Serialize)]
pub struct ChatZeile {
    id: String,
    name: String,
    text: String,
    at: String,
    /// Von mir (meiner Person).
    ich: bool,
    angeheftet: bool,
}

/// Was im Chat steht -- ohne Geloeschtes, mit dem Stand des Anheftens.
fn ansicht(
    zustand: &Zustand,
    speicher: &openany_store::Speicher,
    projekt: &str,
) -> Result<Vec<ChatZeile>, String> {
    let g = zustand.gastgeber.clone();
    let ich = g.person().ok_or("This device has no person yet.")?;
    let (_, liste, _) = projektnah::lesen(speicher, projekt)?;
    let eigentuemer: Vec<String> = liste
        .mitglieder(&|id| g.person_von(id))
        .map_err(text)?
        .into_iter()
        .filter(|m| m.rolle == EIGENTUEMER)
        .map(|m| m.personen_id)
        .collect();
    let nachrichten = projektnah::chat_lesen(speicher, projekt)?;
    let aktionen = projektnah::aktionen_lesen(speicher, projekt)?;
    Ok(
        openany_nahbereich::chat::ansicht(&nachrichten, &aktionen, &eigentuemer)
            .into_iter()
            .map(|a| ChatZeile {
                ich: a.nachricht.personen_id == ich.personen_id
                    || ich.frueher.contains(&a.nachricht.personen_id),
                id: a.nachricht.id,
                name: a.nachricht.name,
                text: a.nachricht.text,
                at: a.nachricht.at,
                angeheftet: a.angeheftet,
            })
            .collect(),
    )
}

async fn auf_server(zustand: &Zustand, projekt: &str) -> bool {
    zustand
        .speicher
        .lock()
        .await
        .projekt(projekt)
        .ok()
        .flatten()
        .is_some_and(|p| p.auf_server)
}

/// Der Chat eines Projekts. Mit Wolke vorher ein Abgleich mit openany.de --
/// geht der nicht (kein Netz), zeigt er, was hier liegt.
#[tauri::command]
pub async fn projekt_chat(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
) -> Result<Vec<ChatZeile>, String> {
    if auf_server(&zustand, &projekt).await {
        if let Some(client) = crate::server_client(&zustand).await {
            if let Err(e) = server_abgleichen(&zustand, &client, &projekt).await {
                eprintln!("Chat nicht mit openany.de abgeglichen ({projekt}): {e}");
            }
        }
    }
    let speicher = zustand.speicher.lock().await;
    // Ein Projekt von vor E2 hat keine Liste -- und ohne sie weiss der Chat
    // nicht, wessen Nachrichten echt sind.
    if speicher
        .projekt(&projekt)
        .map_err(text)?
        .is_some_and(|p| p.mitgliederliste.is_none())
    {
        return Err("This project is older than the signed member list and has no chat any more. Projects like this are removed with the cleanup (E8).".into());
    }
    ansicht(&zustand, &speicher, &projekt)
}

/* ── Schreiben ──────────────────────────────────────────────────────────── */

/// Die eigene Person, wie sie in der Liste steht.
fn selbst(
    zustand: &Zustand,
    speicher: &openany_store::Speicher,
    projekt: &str,
) -> Result<openany_nahbereich::Mitglied, String> {
    let g = zustand.gastgeber.clone();
    let ich = g.person().ok_or("This device has no person yet.")?;
    let (_, liste, _) = projektnah::lesen(speicher, projekt)?;
    liste
        .mitglieder(&|id| g.person_von(id))
        .map_err(text)?
        .into_iter()
        .find(|x| x.personen_id == ich.personen_id || ich.frueher.contains(&x.personen_id))
        .ok_or_else(|| "You are not a member of this project.".to_string())
}

/// Sofort an jedes Mitgliedsgeraet in der Naehe -- im Hintergrund: Ein
/// Geraet, das nicht antwortet, haelt das Schreiben nicht auf.
async fn vor_ort_senden(
    zustand: &Arc<Zustand>,
    projekt: &str,
    nachrichten: Vec<Nachricht>,
    aktionen: Vec<Aktion>,
) {
    let Ok(ident) = crate::geraet_identitaet(zustand) else {
        return;
    };
    let g = zustand.gastgeber.clone();
    let mitglieder = {
        let speicher = zustand.speicher.lock().await;
        match projektnah::lesen(&speicher, projekt) {
            Ok((_, liste, _)) => liste.mitglieder(&|id| g.person_von(id)).unwrap_or_default(),
            Err(_) => return,
        }
    };
    let geraete: Vec<openany_nahbereich::GeraetInDerNaehe> = zustand
        .nah
        .lock()
        .await
        .as_ref()
        .map(|n| n.geraete())
        .unwrap_or_default()
        .into_iter()
        .filter(|d| d.openany && d.fingerabdruck != ident.fingerabdruck)
        .filter(|d| {
            openany_nahbereich::freigaben::mitglied_von(&mitglieder, &d.fingerabdruck, &|id| {
                g.person_von(id)
            })
            .is_some()
        })
        .collect();
    let projekt = projekt.to_string();
    tokio::spawn(async move {
        for d in geraete {
            let _ = openany_nahbereich::anruf::chat_senden(
                &ident,
                &d.adresse,
                openany_nahbereich::DIENST_PORT,
                &d.fingerabdruck,
                &projekt,
                &nachrichten,
                &aktionen,
            )
            .await;
        }
    });
}

/// Eine Nachricht schreiben: hier ablegen, vor Ort sofort weitergeben, mit
/// Wolke hochladen. `benachrichtigen`: Hinweis an die Mitglieder (nur mit
/// Wolke, der Server schickt ihn -- ohne Inhalt).
#[tauri::command]
pub async fn projekt_chat_senden(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    text: String,
    benachrichtigen: Option<bool>,
) -> Result<Vec<ChatZeile>, String> {
    let inhalt = text.trim().to_string();
    if inhalt.is_empty() {
        return Err("Empty message.".into());
    }
    let ident = crate::geraet_identitaet(&zustand)?;
    let nachricht = {
        let speicher = zustand.speicher.lock().await;
        let ich = selbst(&zustand, &speicher, &projekt)?;
        let n = Nachricht::schreiben(&ident, &projekt, &ich.personen_id, &ich.name, &inhalt)
            .map_err(|e| e.to_string())?;
        projektnah::chat_aufnehmen(
            &speicher,
            zustand.gastgeber.as_ref(),
            &projekt,
            std::slice::from_ref(&n),
        )?;
        n
    };
    vor_ort_senden(&zustand, &projekt, vec![nachricht.clone()], Vec::new()).await;
    if auf_server(&zustand, &projekt).await {
        if let Some(client) = crate::server_client(&zustand).await {
            // Diese mit dem Hinweis, falls gewuenscht; den Rest bringt der
            // Abgleich -- mit demselben Schluessel.
            if let Ok(Some(zeug)) = schreibzeug(&zustand, &client, &projekt).await {
                let _ = hochladen_eine(
                    &zustand,
                    &client,
                    &projekt,
                    &zeug,
                    &Inhalt::Nachricht(nachricht),
                    benachrichtigen.unwrap_or(false),
                )
                .await;
                let _ = abgleichen_mit(&zustand, &client, &projekt, &zeug).await;
            }
        }
    }
    let speicher = zustand.speicher.lock().await;
    ansicht(&zustand, &speicher, &projekt)
}

/// Anheften, Loesen oder Loeschen. Die Regeln wie auf dem Server: Anheften
/// darf jedes Mitglied, Loeschen die Person, die geschrieben hat, oder eine
/// Eigentuemerin.
#[tauri::command]
pub async fn projekt_chat_aktion(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    nachricht: String,
    art: String,
) -> Result<Vec<ChatZeile>, String> {
    let art = match art.as_str() {
        ANHEFTEN => ANHEFTEN,
        LOESEN => LOESEN,
        LOESCHEN => LOESCHEN,
        _ => return Err("Unknown action.".into()),
    };
    let ident = crate::geraet_identitaet(&zustand)?;
    let aktion = {
        let speicher = zustand.speicher.lock().await;
        let ich = selbst(&zustand, &speicher, &projekt)?;
        let ziel = projektnah::chat_lesen(&speicher, &projekt)?
            .into_iter()
            .find(|n| n.id == nachricht)
            .ok_or("This message does not exist here.")?;
        if art == LOESCHEN && ziel.personen_id != ich.personen_id && ich.rolle != EIGENTUEMER {
            return Err("Only the author or an owner can delete a message.".into());
        }
        let a =
            Aktion::schreiben(&ident, &projekt, &nachricht, art, &ich.personen_id).map_err(text)?;
        projektnah::aktionen_aufnehmen(
            &speicher,
            zustand.gastgeber.as_ref(),
            &projekt,
            std::slice::from_ref(&a),
        )?;
        a
    };
    vor_ort_senden(&zustand, &projekt, Vec::new(), vec![aktion]).await;
    if auf_server(&zustand, &projekt).await {
        if let Some(client) = crate::server_client(&zustand).await {
            let _ = server_abgleichen(&zustand, &client, &projekt).await;
        }
    }
    let speicher = zustand.speicher.lock().await;
    ansicht(&zustand, &speicher, &projekt)
}

/* ── Der Weg ueber openany.de ───────────────────────────────────────────── */

/// Was verschluesselt reist: eine Nachricht oder eine Aktion, ganz.
#[derive(Debug, Clone, Serialize, Deserialize)]
// Die Art steht NEBEN dem Inhalt, nicht darin: Eine Aktion hat selbst ein
// Feld `art` (anheften, loesen, loeschen).
#[serde(tag = "typ", content = "daten", rename_all = "lowercase")]
enum Inhalt {
    Nachricht(Nachricht),
    Aktion(Aktion),
}

impl Inhalt {
    fn id(&self) -> &str {
        match self {
            Inhalt::Nachricht(n) => &n.id,
            Inhalt::Aktion(a) => &a.id,
        }
    }
}

/// Merker, die nur hier gelten und nicht reisen (Protokoll::Still).
const HOCHGELADEN: &str = "chat_hochgeladen";
const MARKE: &str = "chat_marke";

fn merker(speicher: &openany_store::Speicher, projekt: &str, art: &str) -> Value {
    speicher
        .projektsache(projekt, art)
        .ok()
        .flatten()
        .map(|s| s.felder)
        .unwrap_or_else(|| json!({}))
}

fn merken(
    speicher: &openany_store::Speicher,
    projekt: &str,
    art: &str,
    felder: Value,
) -> Result<(), String> {
    speicher
        .projektsache_schreiben(
            &openany_store::Projektsache {
                projekt: projekt.to_string(),
                art: art.into(),
                uuid: art.into(),
                eltern: None,
                felder,
                geaendert_at: String::new(),
            },
            openany_store::Protokoll::Still,
        )
        .map_err(text)
}

fn hochgeladene(speicher: &openany_store::Speicher, projekt: &str) -> BTreeSet<String> {
    merker(speicher, projekt, HOCHGELADEN)
        .get("ids")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect()
}

fn als_hochgeladen(
    speicher: &openany_store::Speicher,
    projekt: &str,
    neu: &[String],
) -> Result<(), String> {
    if neu.is_empty() {
        return Ok(());
    }
    let mut ids = hochgeladene(speicher, projekt);
    ids.extend(neu.iter().cloned());
    merken(speicher, projekt, HOCHGELADEN, json!({ "ids": ids }))
}

/// Eine Nachricht oder Aktion verschluesselt hochladen.
/// Was zum Hochladen gebraucht wird -- EINMAL je Lauf (siehe
/// planungseinheit::Schreibzeug: je Nachricht geholt, stiess es an die
/// Drossel des Servers).
struct Schreibzeug {
    p: openany_direktpost::Kontoschluessel,
    ich: openany_direktpost::Schluessel,
    von: String,
}

async fn schreibzeug(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    projekt: &str,
) -> Result<Option<Schreibzeug>, String> {
    let Some(p) = projektchat::schluessel(zustand, client, projekt).await? else {
        return Ok(None);
    };
    Ok(Some(Schreibzeug {
        p,
        ich: direktpost::schluessel(zustand)?,
        von: direktpost::mein_name(zustand).ok_or("This device is not set up for messages yet.")?,
    }))
}

async fn hochladen_eine(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    projekt: &str,
    s: &Schreibzeug,
    inhalt: &Inhalt,
    benachrichtigen: bool,
) -> Result<(), String> {
    let (p, ich, von) = (&s.p, &s.ich, &s.von);
    let klartext = serde_json::to_string(inhalt).map_err(text)?;
    let (v, version) = openany_direktpost::projekt_verschluesseln(ich, p, projekt, von, &klartext)
        .map_err(text)?;
    let art = match inhalt {
        Inhalt::Nachricht(_) => "nachricht",
        Inhalt::Aktion(_) => "aktion",
    };
    client
        .projekt_senden(
            Method::POST,
            &format!("/projects/{projekt}/messages"),
            Some(json!({
                "inhalt": v.inhalt, "signatur": v.signatur, "version": version,
                "kennung": inhalt.id(), "art": art, "notify_members": benachrichtigen,
            })),
        )
        .await
        .map_err(text)?;
    als_hochgeladen(
        &*zustand.speicher.lock().await,
        projekt,
        &[inhalt.id().to_string()],
    )
}

/// Mit openany.de abgleichen: erst holen, was dort neu ist, dann hochladen,
/// was dort fehlt -- auch fremde Nachrichten (die Bruecke).
pub(crate) async fn server_abgleichen(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    projekt: &str,
) -> Result<(), String> {
    let Some(zeug) = schreibzeug(zustand, client, projekt).await? else {
        // Ohne Schluessel (wartet auf seine Kiste) laesst sich nichts lesen
        // und nichts schreiben.
        return Ok(());
    };
    abgleichen_mit(zustand, client, projekt, &zeug).await
}

async fn abgleichen_mit(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    projekt: &str,
    zeug: &Schreibzeug,
) -> Result<(), String> {
    let p = &zeug.p;

    // 1. Holen, seit der Marke -- seitenweise.
    let mut marke = merker(&*zustand.speicher.lock().await, projekt, MARKE)
        .get("id")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let mut verz = projektchat::Verzeichnisse::neu(client);
    loop {
        let antwort = client
            .projekt_json(
                &format!("/projects/{projekt}/messages"),
                &[("after_id", marke.to_string())],
            )
            .await
            .map_err(text)?;
        let seite: Vec<Value> = antwort
            .get("messages")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if seite.is_empty() {
            break;
        }
        let mut nachrichten = Vec::new();
        let mut aktionen = Vec::new();
        for mut m in seite.iter().cloned() {
            marke = marke.max(m.get("id").and_then(Value::as_i64).unwrap_or(0));
            projektchat::oeffnen(Some(p), projekt, &mut verz, &mut m).await;
            let body = m.get("body").and_then(Value::as_str).unwrap_or_default();
            // Alte Nachrichten (nur Text) sind kein JSON in dieser Form: fort.
            match serde_json::from_str::<Inhalt>(body) {
                Ok(Inhalt::Nachricht(n)) => nachrichten.push(n),
                Ok(Inhalt::Aktion(a)) => aktionen.push(a),
                Err(_) => {}
            }
        }
        let neu = {
            let speicher = zustand.speicher.lock().await;
            let g = zustand.gastgeber.as_ref();
            let neu = projektnah::chat_aufnehmen(&speicher, g, projekt, &nachrichten)?
                + projektnah::aktionen_aufnehmen(&speicher, g, projekt, &aktionen)?;
            // Was vom Server kam, liegt dort -- nicht noch einmal hochladen.
            let dort: Vec<String> = nachrichten
                .iter()
                .map(|n| n.id.clone())
                .chain(aktionen.iter().map(|a| a.id.clone()))
                .collect();
            als_hochgeladen(&speicher, projekt, &dort)?;
            merken(&speicher, projekt, MARKE, json!({ "id": marke }))?;
            neu
        };
        if neu > 0 {
            zustand.gastgeber.chat_angekommen();
            // Was vom Server kam, gleich an Mitglieder vor Ort (die Bruecke).
            vor_ort_senden(zustand, projekt, nachrichten, aktionen).await;
        }
        if seite.len() < 200 {
            break;
        }
    }

    // 2. Hochladen, was dort fehlt.
    let fehlen: Vec<Inhalt> = {
        let speicher = zustand.speicher.lock().await;
        let schon = hochgeladene(&speicher, projekt);
        projektnah::chat_lesen(&speicher, projekt)?
            .into_iter()
            .map(Inhalt::Nachricht)
            .chain(
                projektnah::aktionen_lesen(&speicher, projekt)?
                    .into_iter()
                    .map(Inhalt::Aktion),
            )
            .filter(|i| !schon.contains(i.id()))
            .collect()
    };
    for i in fehlen {
        hochladen_eine(zustand, client, projekt, zeug, &i, false).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use openany_nahbereich::{Identitaet, Person};

    /// Was verschluesselt reist, kommt ganz zurueck; eine alte Nachricht (nur
    /// Text) ist keine und faellt heraus.
    #[test]
    fn inhalt_hin_und_zurueck_und_alte_fallen_heraus() {
        let tel = Identitaet::erzeugen().unwrap();
        let anna = Person::neu(&tel, "Telefon").unwrap();
        let n = Nachricht::schreiben(&tel, "p1", &anna.personen_id, "Anna", "Hallo").unwrap();
        let a = Aktion::schreiben(&tel, "p1", &n.id, ANHEFTEN, &anna.personen_id).unwrap();

        for i in [Inhalt::Nachricht(n.clone()), Inhalt::Aktion(a.clone())] {
            let text = serde_json::to_string(&i).unwrap();
            match (serde_json::from_str::<Inhalt>(&text).unwrap(), &i) {
                (Inhalt::Nachricht(x), Inhalt::Nachricht(y)) => assert_eq!(&x, y),
                (Inhalt::Aktion(x), Inhalt::Aktion(y)) => assert_eq!(&x, y),
                _ => panic!("Art vertauscht"),
            }
        }
        assert!(serde_json::from_str::<Inhalt>("Hallo zusammen").is_err());
        assert!(serde_json::from_str::<Inhalt>(r#"{"text":"alt"}"#).is_err());
    }
}
