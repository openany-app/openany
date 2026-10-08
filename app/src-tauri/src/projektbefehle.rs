//! Projekte im Programm -- die Verdrahtung.
//!
//! Ein Projekt ist eine weitere Gegenstelle (`<basis>#projekt:<uuid>`), und
//! der Laeufer dafuer steht in `openany-sync`. Hier steht nur, wie ein Klick
//! dorthin kommt und was die Oberflaeche zu sehen bekommt.
//!
//! **Welche Projekte es gibt, sagt der persoenliche Strom.** Der Laeufer
//! traegt sie als Art `project` ein; endet eine Mitgliedschaft, raeumt er sie
//! weg. Ein Projekt vom Server legt man in der Webapp an.
//!
//! **Lokale Projekte (01.10.2026, docs/konzept-lokale-mitgliedschaften.md)**
//! entstehen dagegen HIER, ohne Konto: mit einer unterschriebenen
//! Mitgliederliste (`openany_nahbereich::Mitgliederliste`) und ohne Weg zum
//! Server. Eingeladen wird spaeter vor Ort.

use crate::Zustand;
use openany_client::{Delta, Eintrag, Ergebnis, OpenanyClient, OpenanyError};
use openany_store::Projekt;
use openany_sync::{projekt_lauf, Projektbericht, Projektgegenstelle};
use serde::Serialize;
use std::sync::Arc;

fn fehler(e: impl ToString) -> String {
    e.to_string()
}

/// Der Strom eines Projekts auf openany.de.
pub struct ProjektGegenstelle {
    client: OpenanyClient,
    projekt: String,
    basis: String,
}

impl ProjektGegenstelle {
    pub fn neu(client: OpenanyClient, projekt: &str) -> Self {
        let basis = Projekt::gegenstelle(client.basis(), projekt);

        Self {
            client,
            projekt: projekt.to_string(),
            basis,
        }
    }
}

#[async_trait::async_trait]
impl Projektgegenstelle for ProjektGegenstelle {
    fn basis(&self) -> &str {
        &self.basis
    }

    fn projekt(&self) -> &str {
        &self.projekt
    }

    async fn delta(&self, seit: i64) -> Result<Delta, OpenanyError> {
        self.client.projekt_delta(&self.projekt, seit).await
    }

    async fn anwenden(&self, eintraege: &[Eintrag]) -> Result<Vec<Ergebnis>, OpenanyError> {
        self.client.projekt_anwenden(&self.projekt, eintraege).await
    }
}

/* ── Was die Oberflaeche sieht ─────────────────────────────────────────── */

#[derive(Serialize)]
pub struct ProjektAnzeige {
    id: String,
    name: String,
    /// `owner` oder `member` -- die Rolle kommt aus dem persoenlichen Strom.
    rolle: String,
    /// Auf diesem Geraet entstanden, ohne Server (lokales Projekt).
    lokal: bool,
    /// Wie viele Boards in diesem Projekt liegen.
    boards: usize,
    roadmaps: usize,
    ortsgruppen: usize,
    abstimmungen: usize,
}

#[derive(Serialize)]
pub struct Projektlauf {
    projekt: String,
    name: String,
    gezogen: usize,
    geschoben: usize,
    uebersprungen: usize,
    fehler: Vec<String>,
}

fn lauf_anzeige(projekt: &Projekt, bericht: Projektbericht) -> Projektlauf {
    Projektlauf {
        projekt: projekt.uuid.clone(),
        name: projekt.name.clone(),
        gezogen: bericht.gezogen,
        geschoben: bericht.geschoben,
        uebersprungen: bericht.uebersprungen,
        fehler: bericht.fehler,
    }
}

/* ── Befehle ───────────────────────────────────────────────────────────── */

/// Die Projekte, von denen dieses Geraet weiss.
#[tauri::command]
pub async fn projekte_liste(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Vec<ProjektAnzeige>, String> {
    let speicher = zustand.speicher.lock().await;
    let mut liste = Vec::new();

    for p in speicher.projekte().map_err(fehler)? {
        let zaehlen = |art: &str| -> Result<usize, String> {
            Ok(speicher.projektsachen(&p.uuid, art).map_err(fehler)?.len())
        };
        let (boards, roadmaps, ortsgruppen, abstimmungen) = (
            zaehlen("board")?,
            zaehlen("roadmap")?,
            zaehlen("place_group")?,
            zaehlen("poll")?,
        );

        liste.push(ProjektAnzeige {
            lokal: p.mitgliederliste.is_some(),
            id: p.uuid,
            name: p.name,
            rolle: p.rolle,
            boards,
            roadmaps,
            ortsgruppen,
            abstimmungen,
        });
    }

    Ok(liste)
}

/// Die eigene Antwort auf eine Option -- `None` nimmt sie zurueck.
///
/// **Entschieden am 22.09.2026 (Tiffy):** Ein Geraet stimmt fuer sein eigenes
/// Konto ab, und der Server erzwingt die Regeln der Webapp (Kapazitaet,
/// Einfach- oder Mehrfachauswahl, nur die eigene Einladung). Hier wird nur
/// vorgemerkt; was die Regeln nicht erlauben, kommt als `rejected` zurueck,
/// und der naechste Zug holt den wirklichen Stand.
#[tauri::command]
pub async fn projekt_abstimmen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    option: String,
    antwort: Option<String>,
) -> Result<(), String> {
    let speicher = zustand.speicher.lock().await;

    abstimmen(&speicher, &projekt, &option, antwort.as_deref())
}

fn abstimmen(
    speicher: &openany_store::Speicher,
    projekt: &str,
    option: &str,
    antwort: Option<&str>,
) -> Result<(), String> {
    if let Some(a) = antwort {
        if !matches!(a, "yes" | "no" | "maybe") {
            return Err(format!("Unbekannte Antwort: {a}"));
        }
    }

    let mut sache = speicher
        .projektsache(projekt, option)
        .map_err(fehler)?
        .filter(|o| o.art == "poll_option")
        .ok_or("This option no longer exists.")?;

    // Eine geschlossene Abstimmung nimmt drueben nichts an -- dann lieber
    // gleich hier sagen als nach dem naechsten Abgleich still zuruecksetzen.
    let offen = sache
        .eltern
        .as_deref()
        .and_then(|u| speicher.projektsache(projekt, u).ok().flatten())
        .map(|u| u.text("status") == "open")
        .unwrap_or(false);

    if !offen {
        return Err("This poll is closed.".into());
    }

    if let Some(felder) = sache.felder.as_object_mut() {
        felder.insert("my_answer".into(), serde_json::json!(antwort));
    }
    sache.geaendert_at = String::new();

    speicher
        .projektsache_schreiben(&sache, openany_store::Protokoll::Merken)
        .map_err(fehler)
}

/// Mit allen bekannten Projekten abgleichen.
///
/// **Ein Lauf je Projekt** -- elf Projekte sind elf Delta-Abfragen. Das
/// traegt die Drossel (`throttle:abgleich`), aber es ist der Grund, warum der
/// Auffrischer im Hintergrund Projekte seltener fragen sollte als den
/// persoenlichen Strom.
#[tauri::command]
pub async fn projekte_abgleichen(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Vec<Projektlauf>, String> {
    let Some(client) = crate::server_client(&zustand).await else {
        return Err("Not connected to openany.de.".into());
    };

    let projekte = {
        let speicher = zustand.speicher.lock().await;
        speicher.projekte().map_err(fehler)?
    };

    let mut laeufe = Vec::new();

    // Lokale Projekte kennt der Server nicht -- sie gleichen sich vor Ort ab.
    for p in projekte.into_iter().filter(|p| p.mitgliederliste.is_none()) {
        let gegenstelle = ProjektGegenstelle::neu(client.clone(), &p.uuid);
        let bericht = {
            let speicher = zustand.speicher.lock().await;
            projekt_lauf(&speicher, &gegenstelle).await
        };

        laeufe.push(lauf_anzeige(&p, bericht));

        // Die Freigaben gleich mit: Eigene muessen auf dem Geraet bleiben,
        // auch wenn niemand das Projekt oeffnet. Ohne Netz bleibt der letzte
        // Stand stehen.
        let _ = freigaben_aktualisieren(&zustand, &client, &p.uuid).await;
    }

    // Projekte, in denen dieses Konto nicht mehr ist, halten nichts mehr fest.
    {
        let lebende: Vec<String> = {
            let speicher = zustand.speicher.lock().await;
            speicher
                .projekte()
                .map_err(fehler)?
                .into_iter()
                .map(|p| p.uuid)
                .collect()
        };
        let mut e = zustand.einstellungen.lock().await;
        let vorher = e.projekt_behalten.len();
        e.projekt_behalten.retain(|p, _| lebende.contains(p));
        if e.projekt_behalten.len() != vorher {
            e.schreiben(&zustand.einstellungspfad()).map_err(fehler)?;
        }
    }

    Ok(laeufe)
}

/* ── Projektsachen allgemein (01.10.2026) ──────────────────────────────── */
//
// Die Projekt-Ansichten der Webapp laufen seit dem 01.10.2026 auch hier
// (packages/oberflaeche/projekte). Ihre `api`-Methoden beantwortet die App
// selbst -- aus dem lokalen Speicher, in der Form, die der Server im Abgleich
// erklaert (app/src/quellen/projektApi.js). Dafuer braucht es nur diese
// allgemeinen Wege: Sachen einer Art lesen, anlegen, aendern, entfernen.
// Alles wird GEMERKT: Ein Server-Projekt gleicht es mit openany.de ab.

#[derive(Serialize)]
pub struct SacheAnzeige {
    uuid: String,
    art: String,
    eltern: Option<String>,
    felder: serde_json::Value,
    geaendert_at: String,
}

fn sache_anzeige(s: openany_store::Projektsache) -> SacheAnzeige {
    SacheAnzeige {
        uuid: s.uuid,
        art: s.art,
        eltern: s.eltern,
        felder: s.felder,
        geaendert_at: s.geaendert_at,
    }
}

/// Die Sachen einer Art -- alle, oder nur die unter einem Elternteil.
#[tauri::command]
pub async fn projekt_sachen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    art: String,
    eltern: Option<String>,
) -> Result<Vec<SacheAnzeige>, String> {
    let speicher = zustand.speicher.lock().await;
    let liste = match eltern {
        Some(e) => speicher
            .projektsachen_unter(&projekt, &e)
            .map_err(fehler)?
            .into_iter()
            .filter(|s| s.art == art)
            .collect(),
        None => speicher.projektsachen(&projekt, &art).map_err(fehler)?,
    };
    Ok(liste.into_iter().map(sache_anzeige).collect())
}

#[tauri::command]
pub async fn projekt_sache(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    uuid: String,
) -> Result<Option<SacheAnzeige>, String> {
    let speicher = zustand.speicher.lock().await;
    Ok(speicher
        .projektsache(&projekt, &uuid)
        .map_err(fehler)?
        .map(sache_anzeige))
}

/// Eine Sache anlegen; gibt ihre uuid zurueck.
#[tauri::command]
pub async fn projekt_sache_anlegen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    art: String,
    eltern: Option<String>,
    felder: serde_json::Value,
) -> Result<String, String> {
    let speicher = zustand.speicher.lock().await;
    speicher
        .projekt(&projekt)
        .map_err(fehler)?
        .ok_or("This project does not exist here.")?;
    let uuid = uuid::Uuid::new_v4().to_string();
    speicher
        .projektsache_schreiben(
            &openany_store::Projektsache {
                projekt: projekt.clone(),
                art,
                uuid: uuid.clone(),
                eltern,
                felder,
                geaendert_at: String::new(),
            },
            openany_store::Protokoll::Merken,
        )
        .map_err(fehler)?;
    Ok(uuid)
}

/// Felder aendern (zusammengelegt: nur die genannten), wahlweise auch das
/// Elternteil (`eltern_setzen`).
#[tauri::command]
pub async fn projekt_sache_aendern(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    uuid: String,
    felder: serde_json::Value,
    eltern: Option<String>,
    eltern_setzen: Option<bool>,
) -> Result<(), String> {
    let speicher = zustand.speicher.lock().await;
    let mut sache = speicher
        .projektsache(&projekt, &uuid)
        .map_err(fehler)?
        .ok_or("This no longer exists.")?;
    if let (Some(alt), Some(neu)) = (sache.felder.as_object_mut(), felder.as_object()) {
        for (k, v) in neu {
            alt.insert(k.clone(), v.clone());
        }
    }
    if eltern_setzen.unwrap_or(false) {
        sache.eltern = eltern;
    }
    sache.geaendert_at = String::new();
    speicher
        .projektsache_schreiben(&sache, openany_store::Protokoll::Merken)
        .map_err(fehler)
}

/// Eine Sache entfernen -- mit allem darunter.
#[tauri::command]
pub async fn projekt_sache_entfernen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    uuid: String,
) -> Result<bool, String> {
    let speicher = zustand.speicher.lock().await;
    speicher
        .projektsache_entfernen(&projekt, &uuid, openany_store::Protokoll::Merken)
        .map_err(fehler)
}

/* ── Lokale Projekte (01.10.2026) ──────────────────────────────────────── */

async fn lokal(zustand: &Zustand, projekt: &str) -> bool {
    zustand
        .speicher
        .lock()
        .await
        .projekt(projekt)
        .ok()
        .flatten()
        .is_some_and(|p| p.mitgliederliste.is_some())
}

/// Ein Projekt nur auf diesem Geraet anlegen -- ohne Konto, ohne Server.
/// Die eigene Person ist Eigentuemerin.
#[tauri::command]
pub async fn projekt_lokal_anlegen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    name: String,
) -> Result<String, String> {
    use openany_nahbereich::Gastgeber;
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("A project needs a name.".into());
    }
    let ident = crate::geraet_identitaet(&zustand)?;
    let person = zustand
        .gastgeber
        .person()
        .ok_or("This device has no person yet -- has \"Nearby devices\" started?")?;
    let uuid = uuid::Uuid::new_v4().to_string();
    let liste =
        openany_nahbereich::Mitgliederliste::gruenden(&ident, &person, &uuid).map_err(fehler)?;
    let projekt = Projekt {
        uuid: uuid.clone(),
        name,
        rolle: openany_nahbereich::mitglieder::EIGENTUEMER.into(),
        geaendert_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        mitgliederliste: None,
    };
    zustand
        .speicher
        .lock()
        .await
        .projekt_lokal_schreiben(&projekt, &serde_json::to_string(&liste).map_err(fehler)?)
        .map_err(fehler)?;
    Ok(uuid)
}

#[derive(Serialize)]
pub struct MitgliedAnzeige {
    id: String,
    name: String,
    rolle: String,
    /// Das bin ich (meine Person).
    ich: bool,
    /// Darf ich es entfernen? (Ich bin Eigentuemer, und es bin nicht ich.)
    entfernbar: bool,
}

#[derive(Serialize)]
pub struct MitgliederAnzeige {
    mitglieder: Vec<MitgliedAnzeige>,
    /// Warum die Liste nicht gilt -- dann zeigt die Oberflaeche nichts an.
    fehler: Option<String>,
    /// Die Eigentuemerin hat nur dieses eine Geraet und keine Instanz: Geht
    /// es verloren, ist das Projekt nicht mehr zu verwalten (§11).
    einziges_geraet: bool,
    /// Darf ich austreten? (Ich bin Mitglied, und nicht die letzte
    /// Eigentuemerin.)
    kann_austreten: bool,
    /// Stehe ich (noch) in der Liste? Wer entfernt wurde, sieht es hier --
    /// auch ohne Abgleich, sobald die Liste einmal angekommen ist.
    bin_mitglied: bool,
}

/// Die Mitglieder eines LOKALEN Projekts, wie die unterschriebene Liste sie
/// ergibt. `None` bei einem Projekt vom Server.
#[tauri::command]
pub async fn projekt_mitglieder(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
) -> Result<Option<MitgliederAnzeige>, String> {
    use openany_nahbereich::Gastgeber;
    let Some(json) = zustand
        .speicher
        .lock()
        .await
        .projekt(&projekt)
        .map_err(fehler)?
        .and_then(|p| p.mitgliederliste)
    else {
        return Ok(None);
    };
    let liste: openany_nahbereich::Mitgliederliste = serde_json::from_str(&json).map_err(fehler)?;
    let ich = zustand.gastgeber.person();
    let ident = crate::geraet_identitaet(&zustand).ok();
    // Bekannte Personen: die eigene und die anderer Mitglieder; fuer
    // Unbekannte gilt ihr Anker (das Geraet, das beim Beitritt dabei war).
    let g = zustand.gastgeber.clone();
    let kennt = move |id: &str| g.bekannte_person(id);
    let einziges_geraet = match (&ich, &ident) {
        (Some(p), Some(i)) => {
            p.geraete(&[&i.fingerabdruck]).len() < 2
                && crate::server_client(&zustand).await.is_none()
        }
        _ => true,
    };
    Ok(Some(match liste.mitglieder(&kennt) {
        Ok(m) => {
            let bin = |id: &str| {
                ich.as_ref()
                    .is_some_and(|p| p.personen_id == id || p.frueher.iter().any(|f| f == id))
            };
            let eigentuemer = |x: &openany_nahbereich::Mitglied| {
                x.rolle == openany_nahbereich::mitglieder::EIGENTUEMER
            };
            let bin_eigentuemer = m.iter().any(|x| bin(&x.personen_id) && eigentuemer(x));
            let zahl_eigentuemer = m.iter().filter(|x| eigentuemer(x)).count();
            let kann_austreten = m
                .iter()
                .any(|x| bin(&x.personen_id) && !(eigentuemer(x) && zahl_eigentuemer == 1));
            MitgliederAnzeige {
                mitglieder: m
                    .iter()
                    .map(|x| MitgliedAnzeige {
                        id: x.personen_id.clone(),
                        name: x.name.clone(),
                        rolle: x.rolle.clone(),
                        ich: bin(&x.personen_id),
                        entfernbar: bin_eigentuemer && !bin(&x.personen_id),
                    })
                    .collect(),
                fehler: None,
                einziges_geraet,
                kann_austreten,
                bin_mitglied: m.iter().any(|x| bin(&x.personen_id)),
            }
        }
        Err(e) => MitgliederAnzeige {
            mitglieder: Vec::new(),
            fehler: Some(e.to_string()),
            einziges_geraet,
            kann_austreten: false,
            bin_mitglied: false,
        },
    }))
}

/// Ein Mitglied entfernen (nur die Eigentuemerin). Die anderen erfahren es
/// beim naechsten Abgleich; das entfernte Geraet auch -- es bekommt noch die
/// Liste, aber nichts mehr sonst, und raeumt das Projekt weg. Was es schon
/// hatte, bleibt auf seinen Geraeten.
#[tauri::command]
pub async fn projekt_mitglied_entfernen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    personen_id: String,
) -> Result<(), String> {
    use openany_nahbereich::Gastgeber;
    let g = zustand.gastgeber.clone();
    let ident = crate::geraet_identitaet(&zustand)?;
    let speicher = zustand.speicher.lock().await;
    let (p, mut liste, _) = openany_nahbereich::projektnah::lesen(&speicher, &projekt)?;
    let m = liste.mitglieder(&|id| g.person_von(id)).map_err(fehler)?;
    let wer = m
        .iter()
        .find(|x| x.personen_id == personen_id)
        .ok_or("Not a member.")?
        .clone();
    liste
        .eintragen(
            &ident,
            "austritt",
            &wer.personen_id,
            &wer.name,
            "",
            &wer.geraet,
        )
        .map_err(fehler)?;
    // Gilt das? (Nur die Eigentuemerin entfernt; die letzte bleibt.)
    let neu = liste.mitglieder(&|id| g.person_von(id)).map_err(fehler)?;
    speicher
        .projekt_lokal_schreiben(&p, &serde_json::to_string(&liste).map_err(fehler)?)
        .map_err(fehler)?;
    for art in [
        openany_nahbereich::projektnah::GETEILTE_NOTIZ,
        openany_nahbereich::projektnah::GETEILTE_DATEI,
        openany_nahbereich::projektnah::GETEILTES_BILD,
    ] {
        for s in speicher.projektsachen(&projekt, art).map_err(fehler)? {
            if !neu.iter().any(|x| x.personen_id == s.text("von")) {
                speicher
                    .projektsache_entfernen(&projekt, &s.uuid, openany_store::Protokoll::Still)
                    .map_err(fehler)?;
            }
        }
    }
    Ok(())
}

/// Selbst austreten -- NUR VOR ORT: Der unterschriebene Austritt geht erst an
/// ein Mitgliedsgeraet in der Naehe, dann raeumt dieses Geraet das Projekt
/// weg. Sonst erfuehre niemand davon.
#[tauri::command]
pub async fn projekt_austreten(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
) -> Result<String, String> {
    use openany_nahbereich::Gastgeber;
    let g = zustand.gastgeber.clone();
    let ich = g.person().ok_or("This device has no person yet.")?;
    let ident = crate::geraet_identitaet(&zustand)?;
    let (mut liste, m) = {
        let speicher = zustand.speicher.lock().await;
        let (_, liste, _) = openany_nahbereich::projektnah::lesen(&speicher, &projekt)?;
        let m = liste.mitglieder(&|id| g.person_von(id)).map_err(fehler)?;
        (liste, m)
    };
    let selbst = m
        .iter()
        .find(|x| x.personen_id == ich.personen_id || ich.frueher.contains(&x.personen_id))
        .ok_or("You are not a member of this project.")?
        .clone();
    liste
        .eintragen(
            &ident,
            "austritt",
            &selbst.personen_id,
            &selbst.name,
            "",
            &selbst.geraet,
        )
        .map_err(fehler)?;
    liste.mitglieder(&|id| g.person_von(id)).map_err(fehler)?;

    // Wer von den anderen ist in der Naehe?
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
            m.iter().any(|x| {
                x.personen_id != selbst.personen_id
                    && openany_nahbereich::freigaben::geraet_von(x, &d.fingerabdruck, &|id| {
                        g.person_von(id)
                    })
            })
        })
        .collect();
    let mut bei = None;
    let mut letzter = "No other member is nearby -- leaving only works in person.".to_string();
    for d in geraete {
        match openany_nahbereich::anruf::liste_senden(
            &ident,
            &d.adresse,
            openany_nahbereich::DIENST_PORT,
            &d.fingerabdruck,
            &liste,
        )
        .await
        {
            Ok(()) => {
                bei = Some(d.name.clone());
                break;
            }
            Err(e) => letzter = format!("{}: {e}", d.name),
        }
    }
    let bei = bei.ok_or(letzter)?;
    zustand
        .speicher
        .lock()
        .await
        .projekt_vergessen("lokal", &projekt)
        .map_err(fehler)?;
    Ok(bei)
}

/* ── Fremde Notizen bearbeiten (01.10.2026, strenge Sperre) ────────────── */
//
// Die Sperre vergibt das Geraet der Person, die freigegeben hat; gespeichert
// wird dort (openany-nahbereich/src/sperren.rs). Ist es nicht in der Naehe,
// bleibt die Notiz nur lesbar.

/// Wer hat diese fremde Notiz freigegeben, und wo ist sie dort?
async fn notiz_herkunft(
    zustand: &Zustand,
    projekt: &str,
    id: &str,
) -> Result<(String, String, String), String> {
    use openany_nahbereich::Gastgeber;
    let g = zustand.gastgeber.clone();
    let speicher = zustand.speicher.lock().await;
    let s = speicher
        .projektsache(projekt, id)
        .map_err(fehler)?
        .ok_or("This note does not exist here.")?;
    let von = s.text("von").to_string();
    let zk_id = id
        .strip_prefix(&format!("{von}:"))
        .ok_or("This note does not exist here.")?
        .to_string();
    let (_, liste, _) = openany_nahbereich::projektnah::lesen(&speicher, projekt)?;
    let anker = liste
        .mitglieder(&|x| g.person_von(x))
        .map_err(fehler)?
        .into_iter()
        .find(|m| m.personen_id == von)
        .map(|m| m.geraet)
        .ok_or("Whoever shared this is no longer a member.")?;
    Ok((von, anker, zk_id))
}

#[derive(Serialize)]
pub struct GesperrteNotiz {
    titel: String,
    inhalt: String,
    /// Das fuehrende Geraet -- dort wird gespeichert und losgelassen.
    geraet: String,
}

/// Eine fremde Notiz zum Bearbeiten sperren -- beim Geraet dessen, der sie
/// freigegeben hat. Gibt den frischen Inhalt zurueck.
#[tauri::command]
pub async fn projekt_notiz_sperren(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    id: String,
) -> Result<GesperrteNotiz, String> {
    let (von, anker, zk_id) = notiz_herkunft(&zustand, &projekt, &id).await?;
    let geraete = geraete_von(&zustand, &von, &anker).await;
    if geraete.is_empty() {
        return Err("Read only: the device that shared this note is not nearby right now.".into());
    }
    let ident = crate::geraet_identitaet(&zustand)?;
    let mut letzter = String::new();
    for d in geraete {
        match openany_nahbereich::anruf::notiz_sperren(
            &ident,
            &d.adresse,
            openany_nahbereich::DIENST_PORT,
            &d.fingerabdruck,
            &projekt,
            &zk_id,
        )
        .await
        {
            Ok(n) => {
                return Ok(GesperrteNotiz {
                    titel: n.titel,
                    inhalt: n.inhalt,
                    geraet: d.fingerabdruck,
                })
            }
            Err(e) => letzter = e,
        }
    }
    Err(letzter)
}

/// Speichern -- auf dem fuehrenden Geraet; die Kopie hier zieht gleich mit.
#[tauri::command]
pub async fn projekt_notiz_speichern(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    id: String,
    geraet: String,
    titel: String,
    inhalt: String,
) -> Result<(), String> {
    let (_, _, zk_id) = notiz_herkunft(&zustand, &projekt, &id).await?;
    let adresse = crate::adresse_von_geraet(&zustand, &geraet).await?;
    let ident = crate::geraet_identitaet(&zustand)?;
    openany_nahbereich::anruf::notiz_speichern(
        &ident,
        &adresse,
        openany_nahbereich::DIENST_PORT,
        &geraet,
        &projekt,
        &zk_id,
        &titel,
        &inhalt,
    )
    .await?;
    let speicher = zustand.speicher.lock().await;
    if let Some(mut s) = speicher.projektsache(&projekt, &id).map_err(fehler)? {
        s.felder["titel"] = serde_json::json!(titel);
        s.felder["inhalt"] = serde_json::json!(inhalt);
        s.geaendert_at = String::new();
        speicher
            .projektsache_schreiben(&s, openany_store::Protokoll::Still)
            .map_err(fehler)?;
    }
    Ok(())
}

/// Fertig: die Sperre loslassen.
#[tauri::command]
pub async fn projekt_notiz_entsperren(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    id: String,
    geraet: String,
) -> Result<(), String> {
    let (_, _, zk_id) = notiz_herkunft(&zustand, &projekt, &id).await?;
    let adresse = crate::adresse_von_geraet(&zustand, &geraet).await?;
    let ident = crate::geraet_identitaet(&zustand)?;
    openany_nahbereich::anruf::notiz_entsperren(
        &ident,
        &adresse,
        openany_nahbereich::DIENST_PORT,
        &geraet,
        &projekt,
        &zk_id,
    )
    .await
}

/* ── Chat vor Ort (01.10.2026) ─────────────────────────────────────────── */

#[derive(Serialize)]
pub struct ChatZeile {
    id: String,
    name: String,
    text: String,
    at: String,
    /// Von mir (meiner Person).
    ich: bool,
}

fn chat_zeile(
    n: openany_nahbereich::chat::Nachricht,
    ich: &openany_nahbereich::Person,
) -> ChatZeile {
    ChatZeile {
        ich: n.personen_id == ich.personen_id || ich.frueher.contains(&n.personen_id),
        id: n.id,
        name: n.name,
        text: n.text,
        at: n.at,
    }
}

/// Die Nachrichten eines lokalen Projekts, aelteste zuerst.
#[tauri::command]
pub async fn projekt_chat(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
) -> Result<Vec<ChatZeile>, String> {
    use openany_nahbereich::Gastgeber;
    let ich = zustand
        .gastgeber
        .person()
        .ok_or("This device has no person yet.")?;
    let alle =
        openany_nahbereich::projektnah::chat_lesen(&*zustand.speicher.lock().await, &projekt)?;
    Ok(alle.into_iter().map(|n| chat_zeile(n, &ich)).collect())
}

/*
 * UNGELESEN IM CHAT (Tiffy, 02.10.2026: der Zaehler am Reiter blieb leer).
 * Bis wohin gelesen ist, merkt sich dieses Geraet als Projektsache
 * `chat_gelesen` -- still, sie reist weder zum Server noch zu Mitgliedern
 * (projektnah::PLANUNG nennt sie nicht): Gelesen ist, was HIER gelesen ist.
 */
const CHAT_GELESEN: &str = "chat_gelesen";

fn chat_gelesen_bis(speicher: &openany_store::Speicher, projekt: &str) -> Option<String> {
    speicher
        .projektsache(projekt, CHAT_GELESEN)
        .ok()
        .flatten()
        .map(|s| s.text("bis").to_string())
        .filter(|b| !b.is_empty())
}

fn spaeter_als(at: &str, bis: Option<&str>) -> bool {
    let Some(bis) = bis else { return true };
    match (
        chrono::DateTime::parse_from_rfc3339(at),
        chrono::DateTime::parse_from_rfc3339(bis),
    ) {
        (Ok(a), Ok(b)) => a > b,
        _ => at > bis,
    }
}

/// Wie viele Chat-Nachrichten anderer hier noch niemand gelesen hat.
#[tauri::command]
pub async fn projekt_chat_ungelesen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
) -> Result<usize, String> {
    use openany_nahbereich::Gastgeber;
    let Some(ich) = zustand.gastgeber.person() else {
        return Ok(0);
    };
    let speicher = zustand.speicher.lock().await;
    let bis = chat_gelesen_bis(&speicher, &projekt);
    let alle = openany_nahbereich::projektnah::chat_lesen(&speicher, &projekt)?;
    Ok(alle
        .into_iter()
        .map(|n| chat_zeile(n, &ich))
        .filter(|z| !z.ich && spaeter_als(&z.at, bis.as_deref()))
        .count())
}

/// Der Chat ist gelesen -- bis zur juengsten Nachricht.
#[tauri::command]
pub async fn projekt_chat_gelesen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
) -> Result<(), String> {
    let speicher = zustand.speicher.lock().await;
    let alle = openany_nahbereich::projektnah::chat_lesen(&speicher, &projekt)?;
    let Some(juengste) = alle.iter().map(|n| n.at.clone()).max() else {
        return Ok(());
    };
    if !spaeter_als(&juengste, chat_gelesen_bis(&speicher, &projekt).as_deref()) {
        return Ok(());
    }
    speicher
        .projektsache_schreiben(
            &openany_store::Projektsache {
                projekt: projekt.clone(),
                art: CHAT_GELESEN.into(),
                uuid: CHAT_GELESEN.into(),
                eltern: None,
                felder: serde_json::json!({ "bis": juengste }),
                geaendert_at: String::new(),
            },
            openany_store::Protokoll::Still,
        )
        .map_err(fehler)
}

/// Eine Nachricht schreiben: hier ablegen und SOFORT an jedes Mitgliedsgeraet
/// in der Naehe schicken (im Hintergrund -- ein Geraet, das nicht antwortet,
/// haelt das Schreiben nicht auf). Wer nicht da ist, bekommt sie beim
/// naechsten Abgleich.
#[tauri::command]
pub async fn projekt_chat_senden(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    text: String,
) -> Result<ChatZeile, String> {
    use openany_nahbereich::Gastgeber;
    let text = text.trim().to_string();
    if text.is_empty() {
        return Err("Empty message.".into());
    }
    let g = zustand.gastgeber.clone();
    let ich = g.person().ok_or("This device has no person yet.")?;
    let ident = crate::geraet_identitaet(&zustand)?;
    let (nachricht, m) = {
        let speicher = zustand.speicher.lock().await;
        let (_, liste, _) = openany_nahbereich::projektnah::lesen(&speicher, &projekt)?;
        let m = liste.mitglieder(&|id| g.person_von(id)).map_err(fehler)?;
        let selbst = m
            .iter()
            .find(|x| x.personen_id == ich.personen_id || ich.frueher.contains(&x.personen_id))
            .ok_or("You are not a member of this project.")?;
        let n = openany_nahbereich::chat::Nachricht::schreiben(
            &ident,
            &projekt,
            &selbst.personen_id,
            &selbst.name,
            &text,
        )
        .map_err(fehler)?;
        openany_nahbereich::projektnah::chat_aufnehmen(
            &speicher,
            g.as_ref(),
            &projekt,
            std::slice::from_ref(&n),
        )?;
        (n, m)
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
            openany_nahbereich::freigaben::mitglied_von(&m, &d.fingerabdruck, &|id| {
                g.person_von(id)
            })
            .is_some()
        })
        .collect();
    let raus = nachricht.clone();
    let projekt_raus = projekt.clone();
    tokio::spawn(async move {
        for d in geraete {
            let _ = openany_nahbereich::anruf::chat_senden(
                &ident,
                &d.adresse,
                openany_nahbereich::DIENST_PORT,
                &d.fingerabdruck,
                &projekt_raus,
                std::slice::from_ref(&raus),
            )
            .await;
        }
    });
    Ok(chat_zeile(nachricht, &ich))
}

/// Ein lokales Projekt loeschen -- auf diesem Geraet. Nur fuer Projekte, die
/// es nur hier gibt; eines vom Server verlaesst man in der Webapp.
#[tauri::command]
pub async fn projekt_lokal_loeschen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
) -> Result<(), String> {
    let speicher = zustand.speicher.lock().await;
    let ist_lokal = speicher
        .projekt(&projekt)
        .map_err(fehler)?
        .is_some_and(|p| p.mitgliederliste.is_some());
    if !ist_lokal {
        return Err("Only a local project can be deleted here.".into());
    }
    speicher
        .projekt_vergessen("lokal", &projekt)
        .map_err(fehler)?;
    Ok(())
}

/* ── Freigaben in lokale Projekte (01.10.2026) ─────────────────────────── */
//
// Notiz-Mappen, Ordner (Dateien, Dokumente) und Alben, erst nur lesen; was
// andere freigeben, liegt im Projekt (Tiffy, 01.10.2026). Die Regeln stehen
// in openany-nahbereich (freigaben.rs, projektnah.rs). Bytes kommen beim
// Oeffnen vom Geraet dessen, der freigegeben hat, und bleiben dann hier.

#[derive(Serialize)]
pub struct GeteiltesAnzeige {
    id: String,
    name: String,
    /// Der Weg unterhalb der Freigabe (Unterordner, Unteralben, Mappe).
    pfad: String,
    mime: String,
    groesse: u64,
    /// Liegen die Bytes auf diesem Geraet?
    da: bool,
    /// Nur bei Bildern: Liegt das Vorschaubild hier?
    vorschau_da: bool,
}

#[derive(Serialize)]
pub struct LokaleFreigabeAnzeige {
    /// `note_folder`, `file_folder`, `album`.
    art: String,
    /// `read` oder `edit` (nur Notiz-Mappen).
    stufe: String,
    schluessel: String,
    name: String,
    /// Bei Ordnern: `files` oder `documents` (soweit bekannt).
    zone: String,
    von: String,
    eigen: bool,
    eintraege: Vec<GeteiltesAnzeige>,
}

#[derive(Serialize)]
pub struct Wahl {
    art: String,
    schluessel: String,
    name: String,
    /// Wo es liegt, zum Unterscheiden gleicher Namen.
    ort: String,
}

#[derive(Serialize)]
pub struct LokaleFreigaben {
    freigaben: Vec<LokaleFreigabeAnzeige>,
    /// Was sich von dieser Person noch freigeben laesst.
    waehlbar: Vec<Wahl>,
}

fn zonenname(zone: &str) -> &'static str {
    if zone == "documents" {
        "Dokumente"
    } else {
        "Dateien"
    }
}

#[tauri::command]
pub async fn projekt_lokale_freigaben(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
) -> Result<LokaleFreigaben, String> {
    use openany_nahbereich::freigaben::{geltende, ALBUM, NOTIZ_MAPPE, ORDNER};
    use openany_nahbereich::projektnah::{GETEILTES_BILD, GETEILTE_DATEI, GETEILTE_NOTIZ};
    use openany_nahbereich::Gastgeber;
    let g = zustand.gastgeber.clone();
    let ich = g.person().ok_or("This device has no person yet.")?;
    let speicher = zustand.speicher.lock().await;
    let (_, liste, freigaben) = openany_nahbereich::projektnah::lesen(&speicher, &projekt)?;
    let m = liste.mitglieder(&|id| g.person_von(id)).map_err(fehler)?;
    let (n, d, b) = openany_nahbereich::projektnah::meine_geteilten(
        &speicher,
        g.as_ref(),
        &projekt,
        &m,
        &freigaben,
    )?;
    let hat = |a: &Option<String>| a.as_deref().is_some_and(|a| zustand.inhalte.hat(a));

    let mut aus = Vec::new();
    for f in geltende(&freigaben, &projekt, &m, &|id| g.person_von(id)) {
        let eigen = f.personen_id == ich.personen_id || ich.frueher.contains(&f.personen_id);
        let von = m
            .iter()
            .find(|x| x.personen_id == f.personen_id)
            .map(|x| x.name.clone())
            .unwrap_or_default();
        let mut eintraege: Vec<GeteiltesAnzeige> = if eigen {
            match f.art.as_str() {
                NOTIZ_MAPPE => n
                    .iter()
                    .filter(|x| {
                        openany_nahbereich::freigaben::in_mappe(x.mappe.as_deref(), &f.schluessel)
                    })
                    .map(|x| GeteiltesAnzeige {
                        id: x.zk_id.clone(),
                        name: x.titel.clone(),
                        pfad: String::new(),
                        mime: "text/markdown".into(),
                        groesse: 0,
                        da: true,
                        vorschau_da: false,
                    })
                    .collect(),
                art => (if art == ALBUM { &b } else { &d })
                    .iter()
                    .filter(|x| x.freigabe == f.schluessel)
                    .map(|x| GeteiltesAnzeige {
                        id: x.uuid.clone(),
                        name: x.name.clone(),
                        pfad: x.pfad.clone(),
                        mime: x.mime.clone(),
                        groesse: x.groesse,
                        da: hat(&x.abdruck),
                        vorschau_da: hat(&x.vorschau),
                    })
                    .collect(),
            }
        } else {
            let art = match f.art.as_str() {
                NOTIZ_MAPPE => GETEILTE_NOTIZ,
                ALBUM => GETEILTES_BILD,
                _ => GETEILTE_DATEI,
            };
            speicher
                .projektsachen(&projekt, art)
                .map_err(fehler)?
                .into_iter()
                .filter(|s| s.text("von") == f.personen_id)
                .filter(|s| {
                    if art == GETEILTE_NOTIZ {
                        openany_nahbereich::freigaben::in_mappe(
                            s.felder.get("mappe").and_then(|v| v.as_str()),
                            &f.schluessel,
                        )
                    } else {
                        s.text("freigabe") == f.schluessel
                    }
                })
                .map(|s| {
                    let feld =
                        |k: &str| s.felder.get(k).and_then(|v| v.as_str()).map(str::to_string);
                    GeteiltesAnzeige {
                        id: s.uuid.clone(),
                        name: if art == GETEILTE_NOTIZ {
                            s.text("titel")
                        } else {
                            s.text("name")
                        }
                        .to_string(),
                        pfad: s.text("pfad").to_string(),
                        mime: s.text("mime").to_string(),
                        groesse: s.zahl("groesse") as u64,
                        da: art == GETEILTE_NOTIZ || hat(&feld("abdruck")),
                        vorschau_da: hat(&feld("vorschau")),
                    }
                })
                .collect()
        };
        eintraege.sort_by_key(|e| (e.pfad.to_lowercase(), e.name.to_lowercase()));
        let zone = if f.art == ORDNER {
            if eigen {
                speicher
                    .datei(&f.schluessel)
                    .ok()
                    .flatten()
                    .map(|o| o.zone)
                    .unwrap_or_default()
            } else {
                speicher
                    .projektsachen(&projekt, GETEILTE_DATEI)
                    .ok()
                    .and_then(|l| l.into_iter().find(|s| s.text("freigabe") == f.schluessel))
                    .map(|s| s.text("zone").to_string())
                    .unwrap_or_default()
            }
        } else {
            String::new()
        };
        aus.push(LokaleFreigabeAnzeige {
            stufe: f.stufe.clone(),
            art: f.art.clone(),
            schluessel: f.schluessel.clone(),
            name: f.name.clone(),
            zone,
            von,
            eigen,
            eintraege,
        });
    }

    // Was sich noch freigeben laesst: eigene Mappen, Ordner, Alben.
    let schon: Vec<(String, String)> = aus
        .iter()
        .filter(|f| f.eigen)
        .map(|f| (f.art.clone(), f.schluessel.clone()))
        .collect();
    let frei = |art: &str, k: &str| !schon.iter().any(|(a, s)| a == art && s == k);
    let mut waehlbar: Vec<Wahl> = speicher
        .mappen()
        .map_err(fehler)?
        .into_iter()
        .filter(|x| frei(NOTIZ_MAPPE, &x.pfad))
        .map(|x| Wahl {
            art: NOTIZ_MAPPE.into(),
            ort: format!("Notizen/{}", x.pfad),
            schluessel: x.pfad,
            name: x.name,
        })
        .collect();
    for zone in ["files", "documents"] {
        for o in speicher.ordner_der_zone(zone).map_err(fehler)? {
            if o.papierkorb_at.is_some() || !frei(ORDNER, &o.uuid) {
                continue;
            }
            let weg: Vec<String> = speicher
                .ordnerweg(Some(&o.uuid))
                .map_err(fehler)?
                .into_iter()
                .map(|x| x.name)
                .collect();
            waehlbar.push(Wahl {
                art: ORDNER.into(),
                schluessel: o.uuid,
                name: o.name,
                ort: format!("{}/{}", zonenname(zone), weg.join("/")),
            });
        }
    }
    for a in speicher.alle_alben().map_err(fehler)? {
        if a.papierkorb_at.is_some() || !frei(ALBUM, &a.uuid) {
            continue;
        }
        let mut weg: Vec<String> = speicher
            .albumweg(&a.uuid)
            .map_err(fehler)?
            .into_iter()
            .map(|x| x.name)
            .collect();
        weg.push(a.name.clone());
        waehlbar.push(Wahl {
            art: ALBUM.into(),
            schluessel: a.uuid,
            name: a.name,
            ort: format!("Galerie/{}", weg.join("/")),
        });
    }
    Ok(LokaleFreigaben {
        freigaben: aus,
        waehlbar,
    })
}

/// Etwas Eigenes in ein lokales Projekt freigeben -- oder die Freigabe
/// zuruecknehmen (`freigeben` = false).
#[tauri::command]
pub async fn projekt_lokal_freigeben(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    art: String,
    schluessel: String,
    name: String,
    freigeben: bool,
    bearbeiten: Option<bool>,
) -> Result<(), String> {
    use openany_nahbereich::freigaben::{Freigabe, ALBUM, BEARBEITEN, LESEN, NOTIZ_MAPPE, ORDNER};
    use openany_nahbereich::Gastgeber;
    if ![NOTIZ_MAPPE, ORDNER, ALBUM].contains(&art.as_str()) {
        return Err("Unknown kind.".into());
    }
    let g = zustand.gastgeber.clone();
    let ich = g.person().ok_or("This device has no person yet.")?;
    let ident = crate::geraet_identitaet(&zustand)?;
    let speicher = zustand.speicher.lock().await;
    let (_, liste, mut freigaben) = openany_nahbereich::projektnah::lesen(&speicher, &projekt)?;
    let m = liste.mitglieder(&|id| g.person_von(id)).map_err(fehler)?;
    let meine_id = m
        .iter()
        .find(|x| x.personen_id == ich.personen_id || ich.frueher.contains(&x.personen_id))
        .map(|x| x.personen_id.clone())
        .ok_or("You are not a member of this project.")?;
    // Freigeben laesst sich nur, was hier liegt -- es ist ja das Eigene.
    let da = match art.as_str() {
        NOTIZ_MAPPE => true,
        ORDNER => speicher
            .datei(&schluessel)
            .map_err(fehler)?
            .is_some_and(|d| d.ist_ordner),
        _ => speicher.album(&schluessel).map_err(fehler)?.is_some(),
    };
    if freigeben && !da {
        return Err("This does not exist on this device.".into());
    }
    let f = Freigabe::unterschreiben(
        &ident,
        &projekt,
        &meine_id,
        &art,
        &schluessel,
        &name,
        match (freigeben, art == NOTIZ_MAPPE && bearbeiten.unwrap_or(false)) {
            (false, _) => "",
            (true, true) => BEARBEITEN,
            (true, false) => LESEN,
        },
    )
    .map_err(fehler)?;
    if !f.gilt(&projekt, &m, &|id| g.person_von(id)) {
        return Err("This share would not be valid.".into());
    }
    freigaben.push(f);
    speicher
        .projekt_freigaben_schreiben(
            &projekt,
            &serde_json::to_string(&freigaben).map_err(fehler)?,
        )
        .map_err(fehler)
}

#[derive(Serialize)]
pub struct GeteilteNotizInhalt {
    titel: String,
    inhalt: String,
    von: String,
}

/// Eine Notiz aus einer Freigabe lesen: eine eigene aus dem Notizbuch, eine
/// fremde aus dem Projekt.
#[tauri::command]
pub async fn projekt_geteilte_notiz(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    id: String,
) -> Result<GeteilteNotizInhalt, String> {
    let speicher = zustand.speicher.lock().await;
    if let Some(s) = speicher.projektsache(&projekt, &id).map_err(fehler)? {
        return Ok(GeteilteNotizInhalt {
            titel: s.text("titel").to_string(),
            inhalt: s.text("inhalt").to_string(),
            von: s.text("von_name").to_string(),
        });
    }
    let n = speicher
        .notiz(&id)
        .map_err(fehler)?
        .ok_or("This note does not exist here.")?;
    Ok(GeteilteNotizInhalt {
        titel: n.titel,
        inhalt: n.inhalt,
        von: String::new(),
    })
}

/// Die Geraete in der Naehe, die zu dieser Person gehoeren (Anker und Kette).
async fn geraete_von(
    zustand: &Zustand,
    personen_id: &str,
    anker: &str,
) -> Vec<openany_nahbereich::GeraetInDerNaehe> {
    use openany_nahbereich::Gastgeber;
    let gueltig: Vec<String> = zustand
        .gastgeber
        .person_von(personen_id)
        .map(|p| p.gueltige(&[anker]).into_keys().collect())
        .unwrap_or_default();
    zustand
        .nah
        .lock()
        .await
        .as_ref()
        .map(|n| n.geraete())
        .unwrap_or_default()
        .into_iter()
        .filter(|d| d.fingerabdruck == anker || gueltig.contains(&d.fingerabdruck))
        .collect()
}

/// Die Bytes einer freigegebenen Datei oder eines Bilds (oder seines
/// Vorschaubilds), als Base64. Eigene kommen aus der eigenen Ablage; fremde
/// aus dem Projekt -- und liegen sie noch nicht hier, vom Geraet dessen, der
/// sie freigegeben hat, wenn es in der Naehe ist.
#[tauri::command]
pub async fn projekt_geteilter_inhalt(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    id: String,
    vorschau: bool,
) -> Result<String, String> {
    use base64::Engine;
    use openany_nahbereich::Gastgeber;
    let (abdruck, groesse, von) = {
        let speicher = zustand.speicher.lock().await;
        if let Some(s) = speicher.projektsache(&projekt, &id).map_err(fehler)? {
            let feld = if vorschau { "vorschau" } else { "abdruck" };
            let a = s
                .felder
                .get(feld)
                .and_then(|v| v.as_str())
                .ok_or("There is no content for this.")?
                .to_string();
            let g = if vorschau {
                u64::MAX
            } else {
                s.zahl("groesse") as u64
            };
            (a, g, Some(s.text("von").to_string()))
        } else if let Some(d) = speicher.datei(&id).map_err(fehler)? {
            (
                d.abdruck.ok_or("There is no content for this.")?,
                d.groesse,
                None,
            )
        } else if let Some(b) = speicher.bild(&id).map_err(fehler)? {
            let a = if vorschau { b.vorschau } else { b.abdruck };
            (a.ok_or("There is no content for this.")?, b.groesse, None)
        } else {
            return Err("This does not exist here.".into());
        }
    };

    if !zustand.inhalte.hat(&abdruck) {
        let Some(von) = von else {
            // Eigenes, das hier nur „bei Bedarf" liegt: holen wie beim
            // eigenen Oeffnen (Server oder gepaartes Geraet).
            crate::dateibefehle::von_geraeten_holen(zustand.inner(), &abdruck, groesse).await?;
            let bytes = zustand
                .inhalte
                .lesen(&abdruck, 0, usize::MAX)
                .map_err(fehler)?;
            return Ok(base64::engine::general_purpose::STANDARD.encode(bytes));
        };
        // Wer hat freigegeben, und welches seiner Geraete ist in der Naehe?
        let g = zustand.gastgeber.clone();
        let anker = {
            let speicher = zustand.speicher.lock().await;
            let (_, liste, _) = openany_nahbereich::projektnah::lesen(&speicher, &projekt)?;
            liste
                .mitglieder(&|id| g.person_von(id))
                .map_err(fehler)?
                .into_iter()
                .find(|m| m.personen_id == von)
                .map(|m| m.geraet)
                .ok_or("Whoever shared this is no longer a member.")?
        };
        let geraete = geraete_von(&zustand, &von, &anker).await;
        if geraete.is_empty() {
            return Err(
                "Not on this device yet -- the device that shared it is not nearby right now."
                    .into(),
            );
        }
        let ident = crate::geraet_identitaet(&zustand)?;
        let mut letzter = String::new();
        for d in geraete {
            match openany_nahbereich::anruf::projekt_inhalt_holen(
                &ident,
                &d.adresse,
                openany_nahbereich::DIENST_PORT,
                &d.fingerabdruck,
                &projekt,
                &abdruck,
                groesse,
                &zustand.inhalte,
            )
            .await
            {
                Ok(()) => break,
                Err(e) => letzter = e,
            }
        }
        if !zustand.inhalte.hat(&abdruck) {
            return Err(letzter);
        }
    }
    let bytes = zustand
        .inhalte
        .lesen(&abdruck, 0, usize::MAX)
        .map_err(fehler)?;
    Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
}

#[derive(Serialize)]
pub struct NahProjektlauf {
    projekt: String,
    name: String,
    mit: String,
    neue_mitglieder: usize,
    notizen: usize,
    dateien: usize,
    bilder: usize,
    /// Dieses Geraet wurde aus dem Projekt entfernt -- es ist jetzt fort.
    entfernt: bool,
    /// Neue Chat-Nachrichten.
    chat: usize,
    /// Sachen der Planung, die kamen, sich aenderten oder gingen.
    planung: usize,
    fehler: Option<String>,
}

/// Lokale Projekte mit den Geraeten in der Naehe abgleichen, die zu einem
/// Mitglied gehoeren (projektnah.rs). Vorschaubilder kommen gleich mit.
#[tauri::command]
pub async fn projekte_nah_abgleichen(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Vec<NahProjektlauf>, String> {
    use openany_nahbereich::Gastgeber;
    let g = zustand.gastgeber.clone();
    let ident = crate::geraet_identitaet(&zustand)?;
    let geraete: Vec<openany_nahbereich::GeraetInDerNaehe> = zustand
        .nah
        .lock()
        .await
        .as_ref()
        .map(|n| n.geraete())
        .unwrap_or_default()
        .into_iter()
        .filter(|d| d.openany && d.fingerabdruck != ident.fingerabdruck)
        .collect();
    let lokale: Vec<Projekt> = zustand
        .speicher
        .lock()
        .await
        .projekte()
        .map_err(fehler)?
        .into_iter()
        .filter(|p| p.mitgliederliste.is_some())
        .collect();

    let mut laeufe = Vec::new();
    for p in lokale {
        for d in &geraete {
            // Gehoert das Geraet zu einem Mitglied? Sonst gar nicht fragen.
            let mitglied = {
                let speicher = zustand.speicher.lock().await;
                openany_nahbereich::projektnah::lesen(&speicher, &p.uuid)
                    .ok()
                    .and_then(|(_, liste, _)| liste.mitglieder(&|id| g.person_von(id)).ok())
                    .is_some_and(|m| {
                        openany_nahbereich::freigaben::mitglied_von(&m, &d.fingerabdruck, &|id| {
                            g.person_von(id)
                        })
                        .is_some()
                    })
            };
            if !mitglied {
                continue;
            }
            let ergebnis = match openany_nahbereich::anruf::projekt_stand(
                &ident,
                &d.adresse,
                openany_nahbereich::DIENST_PORT,
                &d.fingerabdruck,
                &p.uuid,
            )
            .await
            {
                Ok(stand) => {
                    let bericht = {
                        let speicher = zustand.speicher.lock().await;
                        openany_nahbereich::projektnah::stand_uebernehmen(
                            &speicher,
                            g.as_ref(),
                            &p.uuid,
                            &d.fingerabdruck,
                            &stand,
                        )
                    };
                    // Vorschaubilder gleich mit: klein, und ohne sie ist ein
                    // Album nur eine Liste von Namen.
                    if bericht.is_ok() {
                        for b in &stand.bilder {
                            if let Some(v) = &b.vorschau {
                                let _ = openany_nahbereich::anruf::projekt_inhalt_holen(
                                    &ident,
                                    &d.adresse,
                                    openany_nahbereich::DIENST_PORT,
                                    &d.fingerabdruck,
                                    &p.uuid,
                                    v,
                                    u64::MAX,
                                    &zustand.inhalte,
                                )
                                .await;
                            }
                        }
                    }
                    bericht
                }
                Err(e) => Err(e),
            };
            laeufe.push(match ergebnis {
                Ok(b) => NahProjektlauf {
                    projekt: p.uuid.clone(),
                    name: p.name.clone(),
                    mit: d.name.clone(),
                    neue_mitglieder: b.neue_mitglieder,
                    notizen: b.notizen,
                    dateien: b.dateien,
                    bilder: b.bilder,
                    entfernt: b.nicht_mehr_mitglied,
                    chat: b.chat,
                    planung: b.planung,
                    fehler: None,
                },
                Err(e) => NahProjektlauf {
                    projekt: p.uuid.clone(),
                    name: p.name.clone(),
                    mit: d.name.clone(),
                    neue_mitglieder: 0,
                    notizen: 0,
                    dateien: 0,
                    bilder: 0,
                    entfernt: false,
                    chat: 0,
                    planung: 0,
                    fehler: Some(e),
                },
            });
            // Entfernt: Das Projekt ist hier fort (was schon da war, inklusive).
            if laeufe.last().is_some_and(|l| l.entfernt) {
                zustand
                    .speicher
                    .lock()
                    .await
                    .projekt_vergessen("lokal", &p.uuid)
                    .map_err(fehler)?;
                break;
            }
        }
    }
    Ok(laeufe)
}

/* ── Freigaben (Weg 1, 30.09.2026) ─────────────────────────────────────── */
//
// Was Mitglieder in ein Projekt freigeben: Ordner, Akten, Alben, Notiz-
// Mappen. EIGENE behaelt dieses Geraet immer ganz (`projekt_behalten` in den
// Einstellungen) und zeigt sie aus dem eigenen Speicher, auch ohne Netz.
// FREMDE kommen beim Oeffnen vom Server, nur lesend -- Bearbeiten bleibt der
// Webapp (Tiffy, 30.09.2026).

#[derive(Serialize)]
pub struct FreigabeAnzeige {
    /// Die Zahl auf dem Server -- fuer fremde Freigaben der Schluessel zum
    /// Inhalt.
    id: i64,
    /// Die uuid -- fuer eigene der Schluessel im eigenen Speicher. Notiz-
    /// Mappen haben keine.
    uuid: Option<String>,
    /// `album`, `file_folder`, `note_folder`.
    art: String,
    /// Bei Ordnern: `files` (Dateien) oder `documents` (Akte).
    zone: Option<String>,
    name: String,
    eigen: bool,
    /// `read` oder `edit` -- im Programm wird in beiden Faellen nur gelesen.
    stufe: String,
    von: String,
    im_papierkorb: bool,
}

fn freigabe_anzeige(z: &serde_json::Value) -> Option<FreigabeAnzeige> {
    let text = |k: &str| z.get(k).and_then(|v| v.as_str()).map(str::to_string);
    Some(FreigabeAnzeige {
        id: z.get("shareable_id")?.as_i64()?,
        uuid: text("uuid"),
        art: text("type")?,
        zone: text("zone"),
        name: text("name").unwrap_or_default(),
        eigen: z.get("own").and_then(|v| v.as_bool()).unwrap_or(false),
        stufe: text("permission").unwrap_or_else(|| "read".into()),
        von: z
            .pointer("/shared_by/name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        im_papierkorb: z.get("trashed").and_then(|v| v.as_bool()).unwrap_or(false),
    })
}

/// Die Freigaben holen und die eigenen Ordner und Alben unter „immer
/// behalten" eintragen. Kam etwas dazu, gleich nachholen.
async fn freigaben_aktualisieren(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    projekt: &str,
) -> Result<Vec<FreigabeAnzeige>, String> {
    let roh = client
        .projekt_freigaben(projekt)
        .await
        .map_err(|e| format!("{e}"))?;
    let liste: Vec<FreigabeAnzeige> = roh
        .as_array()
        .map(|a| a.iter().filter_map(freigabe_anzeige).collect())
        .unwrap_or_default();

    let mut eigene: Vec<String> = liste
        .iter()
        .filter(|f| f.eigen && !f.im_papierkorb && f.art != "note_folder")
        .filter_map(|f| f.uuid.clone())
        .collect();
    eigene.sort();

    let neu_dabei = {
        let mut e = zustand.einstellungen.lock().await;
        let vorher = e.projekt_behalten.get(projekt).cloned().unwrap_or_default();
        if vorher == eigene {
            false
        } else {
            let dazu = eigene.iter().any(|u| !vorher.contains(u));
            if eigene.is_empty() {
                e.projekt_behalten.remove(projekt);
            } else {
                e.projekt_behalten.insert(projekt.to_string(), eigene);
            }
            e.schreiben(&zustand.einstellungspfad()).map_err(fehler)?;
            dazu
        }
    };
    if neu_dabei {
        crate::dateibefehle::nachholen_im_hintergrund(zustand.clone());
    }
    Ok(liste)
}

#[tauri::command]
pub async fn projekt_freigaben(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
) -> Result<Vec<FreigabeAnzeige>, String> {
    // Freigaben in lokale Projekte kommen mit einem spaeteren Schritt.
    if lokal(&zustand, &projekt).await {
        return Ok(Vec::new());
    }
    let Some(client) = crate::server_client(&zustand).await else {
        return Ok(eigene_aus_dem_speicher(&zustand, &projekt).await);
    };
    match freigaben_aktualisieren(zustand.inner(), &client, &projekt).await {
        Ok(liste) => Ok(liste),
        // OHNE NETZ die eigenen aus dem letzten Stand: Sie liegen ohnehin
        // hier, und genau dafuer behaelt das Geraet sie.
        Err(_) => Ok(eigene_aus_dem_speicher(&zustand, &projekt).await),
    }
}

async fn eigene_aus_dem_speicher(zustand: &Zustand, projekt: &str) -> Vec<FreigabeAnzeige> {
    let uuids = zustand
        .einstellungen
        .lock()
        .await
        .projekt_behalten
        .get(projekt)
        .cloned()
        .unwrap_or_default();
    let speicher = zustand.speicher.lock().await;
    uuids
        .into_iter()
        .filter_map(|u| {
            let (art, name, zone) = if let Ok(Some(d)) = speicher.datei(&u) {
                ("file_folder", d.name, Some(d.zone.to_string()))
            } else if let Ok(Some(a)) = speicher.album(&u) {
                ("album", a.name, None)
            } else {
                return None;
            };
            Some(FreigabeAnzeige {
                id: 0,
                uuid: Some(u),
                art: art.into(),
                zone,
                name,
                eigen: true,
                stufe: String::new(),
                von: String::new(),
                im_papierkorb: false,
            })
        })
        .collect()
}

/// Den Inhalt einer fremden Freigabe lesen: `art` `file_folder` (Ordner oder
/// Akte), `album` oder `note_folder`, `id` die Zahl auf dem Server. Die
/// Antwort ist die des Servers, unveraendert -- dieselbe, die die Webapp
/// zeigt.
#[tauri::command]
pub async fn projekt_freigabe_inhalt(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    art: String,
    id: i64,
) -> Result<serde_json::Value, String> {
    let client = crate::server_client(&zustand)
        .await
        .ok_or("Not connected to openany.de.")?;
    let (pfad, abfrage) = match art.as_str() {
        "file_folder" => (
            format!("/projects/{projekt}/files"),
            vec![("folder_id", id.to_string())],
        ),
        "album" => (format!("/projects/{projekt}/albums/{id}"), vec![]),
        "note_folder" => (format!("/projects/{projekt}/note-folders/{id}"), vec![]),
        _ => return Err(format!("Unknown share: {art}")),
    };
    client
        .projekt_json(&pfad, &abfrage)
        .await
        .map_err(|e| format!("{e}"))
}

/// Die Bytes einer Datei oder eines Bilds aus einer fremden Freigabe, als
/// Base64 (auf Android kaemen Bytes sonst als Zahlenliste an). `pfad` ist
/// ein Link aus der Antwort des Servers (`url`, `download_url`).
#[tauri::command]
pub async fn projekt_freigabe_datei(
    zustand: tauri::State<'_, Arc<Zustand>>,
    pfad: String,
) -> Result<String, String> {
    use base64::Engine;
    let client = crate::server_client(&zustand)
        .await
        .ok_or("Not connected to openany.de.")?;
    let bytes = client
        .projekt_bytes(&pfad)
        .await
        .map_err(|e| format!("{e}"))?;
    Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use openany_store::Projektsache;
    use openany_store::{Protokoll, Speicher};

    fn sache(
        art: &str,
        uuid: &str,
        eltern: Option<&str>,
        felder: serde_json::Value,
    ) -> Projektsache {
        Projektsache {
            projekt: "p1".into(),
            art: art.into(),
            uuid: uuid.into(),
            eltern: eltern.map(Into::into),
            felder,
            geaendert_at: String::new(),
        }
    }

    fn speicher_mit_board() -> Speicher {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        s.projekt_schreiben(&Projekt {
            uuid: "p1".into(),
            name: "Haushalt".into(),
            rolle: "owner".into(),
            geaendert_at: String::new(),
            mitgliederliste: None,
        })
        .unwrap();

        for sa in [
            sache(
                "board",
                "b1",
                None,
                serde_json::json!({ "name": "Einkauf" }),
            ),
            sache(
                "column",
                "s2",
                Some("b1"),
                serde_json::json!({ "name": "Fertig", "position": 2 }),
            ),
            sache(
                "column",
                "s1",
                Some("b1"),
                serde_json::json!({ "name": "Offen", "position": 1 }),
            ),
            sache(
                "card",
                "k2",
                Some("s1"),
                serde_json::json!({ "title": "Brot", "position": 2 }),
            ),
            sache(
                "card",
                "k1",
                Some("s1"),
                serde_json::json!({ "title": "Milch", "position": 1 }),
            ),
        ] {
            s.projektsache_schreiben(&sa, Protokoll::Still).unwrap();
        }

        s
    }

    fn speicher_mit_abstimmung(status: &str) -> Speicher {
        let s = speicher_mit_board();

        for sa in [
            sache(
                "poll",
                "u1",
                None,
                serde_json::json!({ "title": "Farbe", "kind": "survey", "status": status,
                    "settings": { "multiple": true }, "voted": 2 }),
            ),
            sache(
                "poll_option",
                "o2",
                Some("u1"),
                serde_json::json!({ "label": "Rot", "position": 1, "count": 0, "my_answer": null }),
            ),
            sache(
                "poll_option",
                "o1",
                Some("u1"),
                serde_json::json!({ "label": "Blau", "position": 0, "count": 2, "my_answer": "yes" }),
            ),
        ] {
            s.projektsache_schreiben(&sa, Protokoll::Still).unwrap();
        }

        s
    }

    /// Abstimmen merkt nur die eigene Antwort vor -- der Server entscheidet.
    #[test]
    fn abstimmen_merkt_die_antwort_vor() {
        let s = speicher_mit_abstimmung("open");
        let marke = s.projekt_aenderungen_seit("p1", 0).unwrap().1;

        abstimmen(&s, "p1", "o2", Some("yes")).unwrap();

        assert_eq!(
            s.projektsache("p1", "o2")
                .unwrap()
                .unwrap()
                .text("my_answer"),
            "yes"
        );
        assert_eq!(s.projekt_aenderungen_seit("p1", marke).unwrap().0.len(), 1);
    }

    /// Zuruecknehmen schickt `null` -- das versteht der Server als "nicht mehr".
    #[test]
    fn eine_antwort_laesst_sich_zuruecknehmen() {
        let s = speicher_mit_abstimmung("open");

        abstimmen(&s, "p1", "o1", None).unwrap();

        assert!(s.projektsache("p1", "o1").unwrap().unwrap().felder["my_answer"].is_null());
    }

    /// Eine geschlossene Abstimmung nimmt schon hier nichts an.
    #[test]
    fn geschlossen_ist_geschlossen() {
        let s = speicher_mit_abstimmung("closed");

        assert!(abstimmen(&s, "p1", "o2", Some("yes")).is_err());
        assert!(abstimmen(&s, "p1", "o2", Some("vielleicht")).is_err());
    }
}
