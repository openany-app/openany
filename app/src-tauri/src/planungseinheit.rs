//! Planung auf beiden Wegen (docs/plan-projekte-eine-art.md, E5).
//!
//! Das Whiteboard eines Hubs ist eine
//! Menge unterschriebener Eintraege (`openany_nahbereich::planung`), die
//! immer HIER liegen, als Projektsachen. Vor Ort reisen sie im Stand; mit
//! Wolke zusaetzlich verschluesselt ueber openany.de (`/planung`), mit ihrer
//! id als Kennung -- eine neuere Fassung ersetzt dort die vorige.
//!
//! Hochgeladen wird jede Fassung, die dort fehlt -- auch Eintraege anderer,
//! die dieses Geraet vor Ort bekam (die Bruecke). Was vom Server kommt,
//! landet hier und reist beim naechsten Treffen vor Ort weiter.
//!
//! `projektwhiteboard.rs` baut daraus, was die Oberflaeche braucht.

use std::collections::BTreeMap;
use std::sync::Arc;

use openany_client::{Method, OpenanyClient};
use openany_nahbereich::planung::Eintrag;
use openany_nahbereich::{projektnah, Gastgeber, Mitglied};
use serde_json::{json, Value};

use crate::{direktpost, projektchat, Zustand};

fn text(e: impl ToString) -> String {
    e.to_string()
}

/// Wer ich in diesem Projekt bin, und wer dabei ist.
pub(crate) struct Lage {
    pub ich: Mitglied,
}

pub(crate) fn lage(
    zustand: &Zustand,
    speicher: &openany_store::Speicher,
    projekt: &str,
) -> Result<Lage, String> {
    let g = zustand.gastgeber.clone();
    let person = g.person().ok_or("This device has no person yet.")?;
    let (_, liste, _) = projektnah::lesen(speicher, projekt)?;
    let mitglieder = liste.mitglieder(&|id| g.person_von(id)).map_err(text)?;
    let ich = mitglieder
        .iter()
        .find(|m| m.personen_id == person.personen_id || person.frueher.contains(&m.personen_id))
        .cloned()
        .ok_or("You are not a member of this project.")?;
    Ok(Lage { ich })
}

/// Einen Eintrag schreiben (oder mit `inhalt: None` loeschen): hier
/// unterschreiben und ablegen, mit Wolke gleich hochladen. `hinweis`: Die
/// Mitglieder bekommen einen Hinweis (ohne Inhalt) -- nur mit Wolke.
pub(crate) async fn schreiben(
    zustand: &Arc<Zustand>,
    projekt: &str,
    sorte: &str,
    sache: &str,
    id: &str,
    inhalt: Option<&Value>,
    hinweis: bool,
) -> Result<Eintrag, String> {
    let ident = crate::geraet_identitaet(zustand)?;
    let roh = inhalt
        .map(serde_json::to_string)
        .transpose()
        .map_err(text)?;
    let e = {
        let speicher = zustand.speicher.lock().await;
        let l = lage(zustand, &speicher, projekt)?;
        let e = Eintrag::schreiben(
            &ident,
            projekt,
            id,
            sache,
            sorte,
            &l.ich.personen_id,
            roh.as_deref(),
        )
        .map_err(text)?;
        let neu = projektnah::planung_eintraege_aufnehmen(
            &speicher,
            zustand.gastgeber.as_ref(),
            projekt,
            std::slice::from_ref(&e),
        )?;
        if neu == 0 {
            return Err("You may not change this entry.".into());
        }
        e
    };
    if auf_server(zustand, projekt).await {
        if let Some(client) = crate::server_client(zustand).await {
            let hoch = async {
                match schreibzeug(zustand, &client, projekt).await? {
                    Some(s) => hochladen_eine(zustand, &client, projekt, &s, &e, hinweis).await,
                    None => Ok(()),
                }
            };
            if let Err(err) = hoch.await {
                eprintln!("Planung nicht hochgeladen ({projekt}): {err}");
            }
        }
    }
    Ok(e)
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

/// Mit Wolke: vor dem Lesen abgleichen. Geht das nicht (kein Netz), gilt,
/// was hier liegt.
pub(crate) async fn vorher_abgleichen(zustand: &Arc<Zustand>, projekt: &str) {
    if !auf_server(zustand, projekt).await || eben_erst(projekt) {
        return;
    }
    if let Some(client) = crate::server_client(zustand).await {
        if let Err(e) = abgleichen(zustand, &client, projekt).await {
            eprintln!("Planung nicht mit openany.de abgeglichen ({projekt}): {e}");
        }
    }
}

/// Eben erst abgeglichen? Die Planung oeffnet Abstimmungen UND Whiteboards
/// -- zweimal kurz hintereinander abzugleichen kostet nur Anfragen.
fn eben_erst(projekt: &str) -> bool {
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::time::{Duration, Instant};
    static ZULETZT: Mutex<Option<HashMap<String, Instant>>> = Mutex::new(None);
    let Ok(mut z) = ZULETZT.lock() else {
        return false;
    };
    let karte = z.get_or_insert_with(HashMap::new);
    if karte
        .get(projekt)
        .is_some_and(|t| t.elapsed() < Duration::from_secs(10))
    {
        return true;
    }
    karte.insert(projekt.to_string(), Instant::now());
    false
}

/* ── Der Weg ueber openany.de ───────────────────────────────────────────── */

/// Merker, die nur hier gelten und nicht reisen.
const HOCHGELADEN: &str = "planung_hochgeladen";
const MARKE: &str = "planung_marke";

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

/// Welche Fassung (Zeit) je Eintrag schon dort liegt.
fn hochgeladene(speicher: &openany_store::Speicher, projekt: &str) -> BTreeMap<String, String> {
    serde_json::from_value(merker(speicher, projekt, HOCHGELADEN)).unwrap_or_default()
}

fn als_hochgeladen(
    speicher: &openany_store::Speicher,
    projekt: &str,
    neu: &[&Eintrag],
) -> Result<(), String> {
    if neu.is_empty() {
        return Ok(());
    }
    let mut dort = hochgeladene(speicher, projekt);
    for e in neu {
        dort.insert(e.id.clone(), e.at.clone());
    }
    merken(
        speicher,
        projekt,
        HOCHGELADEN,
        serde_json::to_value(dort).map_err(text)?,
    )
}

fn feld(id: &str) -> String {
    format!("planung:{id}")
}

/// Was zum Hochladen gebraucht wird -- EINMAL je Lauf geholt. Den
/// Projektschluessel zu holen kostet mehrere Anfragen; je Eintrag geholt,
/// stiesse ein Abgleich mit ein paar Eintraegen an die Drossel des Servers.
pub(crate) struct Schreibzeug {
    p: openany_direktpost::Kontoschluessel,
    ich: openany_direktpost::Schluessel,
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
    }))
}

async fn hochladen_eine(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    projekt: &str,
    s: &Schreibzeug,
    e: &Eintrag,
    hinweis: bool,
) -> Result<(), String> {
    let (p, ich) = (&s.p, &s.ich);
    let roh = serde_json::to_string(e).map_err(text)?;
    let (v, version) =
        openany_direktpost::projektfeld_verschluesseln(ich, p, projekt, &feld(&e.id), &roh)
            .map_err(text)?;
    let mut koerper = json!({ "inhalt": v.inhalt, "signatur": v.signatur, "version": version });
    if hinweis {
        koerper["hinweis"] = json!("abstimmung");
    }
    client
        .projekt_senden(
            Method::PUT,
            &format!("/projects/{projekt}/planung/{}", e.id),
            Some(koerper),
        )
        .await
        .map_err(text)?;
    als_hochgeladen(&*zustand.speicher.lock().await, projekt, &[e])
}

/// Holen, was dort neu ist; dann hochladen, was dort fehlt oder aelter ist.
pub(crate) async fn abgleichen(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    projekt: &str,
) -> Result<(), String> {
    let Some(zeug) = schreibzeug(zustand, client, projekt).await? else {
        return Ok(());
    };
    let p = &zeug.p;

    let mut marke = merker(&*zustand.speicher.lock().await, projekt, MARKE)
        .get("folge")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let mut verz = projektchat::Verzeichnisse::neu(zustand, client);
    loop {
        let antwort = client
            .projekt_json(
                &format!("/projects/{projekt}/planung"),
                &[("seit", marke.to_string())],
            )
            .await
            .map_err(text)?;
        let seite: Vec<Value> = antwort
            .get("eintraege")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if seite.is_empty() {
            break;
        }
        let mut gekommen = Vec::new();
        for w in &seite {
            marke = marke.max(w.get("folge").and_then(Value::as_i64).unwrap_or(0));
            let (Some(kennung), Some(von), Some(geraet), Some(version)) = (
                w.get("kennung").and_then(Value::as_str),
                w.get("von_name").and_then(Value::as_str),
                w.get("geraet").and_then(Value::as_i64),
                w.get("version").and_then(Value::as_u64),
            ) else {
                continue;
            };
            let Some(ed) = verz.ed25519(von, geraet).await else {
                continue;
            };
            let v = openany_direktpost::Verschluesselt {
                inhalt: w
                    .get("inhalt")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                signatur: w
                    .get("signatur")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
            };
            let Ok(roh) = openany_direktpost::projektfeld_oeffnen(
                p,
                projekt,
                &feld(kennung),
                version as u32,
                &v,
                &ed,
            ) else {
                continue;
            };
            if let Ok(e) = serde_json::from_str::<Eintrag>(&roh) {
                if e.id == kennung {
                    gekommen.push(e);
                }
            }
        }
        // Wie im Chat: Traegt die Liste einen Eintrag nicht, erst den Buergen
        // fragen (docs/plan-buergen.md).
        let liste_offen = {
            let speicher = zustand.speicher.lock().await;
            let g = zustand.gastgeber.as_ref();
            projektnah::lesen(&speicher, projekt)
                .ok()
                .and_then(|(_, liste, _)| {
                    let person = |id: &str| g.person_von(id);
                    gekommen
                        .iter()
                        .any(|e| !e.gilt(projekt, &liste, &person))
                        .then_some(liste)
                })
        };
        if let Some(liste) = liste_offen {
            crate::buergen::nachfragen(zustand, client, &liste).await;
        }
        {
            let speicher = zustand.speicher.lock().await;
            projektnah::planung_eintraege_aufnehmen(
                &speicher,
                zustand.gastgeber.as_ref(),
                projekt,
                &gekommen,
            )?;
            // Was vom Server kam, liegt dort -- in dieser Fassung.
            als_hochgeladen(&speicher, projekt, &gekommen.iter().collect::<Vec<_>>())?;
            merken(&speicher, projekt, MARKE, json!({ "folge": marke }))?;
        }
        if seite.len() < 200 {
            break;
        }
    }

    let fehlen: Vec<Eintrag> = {
        let speicher = zustand.speicher.lock().await;
        let dort = hochgeladene(&speicher, projekt);
        projektnah::planung_eintraege_lesen(&speicher, projekt)?
            .into_iter()
            .filter(|e| dort.get(&e.id).is_none_or(|at| at < &e.at))
            .collect()
    };
    for e in fehlen {
        hochladen_eine(zustand, client, projekt, &zeug, &e, false).await?;
    }
    Ok(())
}
