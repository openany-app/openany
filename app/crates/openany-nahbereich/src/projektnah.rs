//! Abgleich eines LOKALEN Projekts zwischen Mitgliedern -- vor Ort, ohne
//! Server (docs/konzept-lokale-mitgliedschaften.md, §6; Tiffy, 01.10.2026:
//! freigegebene Notiz-Mappen, erst nur lesen, angezeigt im Projekt).
//!
//! **Ein Geraet fragt, das andere liefert seinen Stand:** Mitgliederliste,
//! Freigaben, die Personen der Mitglieder -- und die Notizen, die SEINE Person
//! freigegeben hat. Kein Delta: Ein Projekt traegt wenig, und ein ganzer Stand
//! ist leichter richtig als ein Strom.
//!
//! **Jedes Geraet prueft zweimal** (§5):
//! - Beim Liefern: nur an ein Geraet eines Mitglieds.
//! - Beim Annehmen: Die Mitgliederliste ersetzt die eigene nur, wenn sie gilt
//!   und die eigene fortsetzt. Freigaben nur, wenn sie gelten. Notizen nur
//!   die der Person, die gerade liefert, und nur aus ihren geltenden
//!   Freigaben.
//!
//! **Warum Notizen nur von ihrer Autorin:** Sie sind nicht unterschrieben.
//! Reichte ein drittes Geraet sie weiter, koennte es sie veraendern. Bei drei
//! Mitgliedern bekommt C die Notizen von A also erst, wenn A und C sich
//! treffen. Mitgliederliste, Freigaben und Personen sind dagegen
//! unterschrieben und reisen ueber jeden.

use crate::dienst::Gastgeber;
use crate::freigaben::{self, Freigabe, ALBUM, NOTIZ_MAPPE, ORDNER};
use crate::mitglieder::{Mitglied, Mitgliederliste};
use crate::Person;
use openany_store::{Projektsache, Protokoll, Speicher};
use serde::{Deserialize, Serialize};

/// Was an Planung zwischen Mitgliedern reist (Schritt 6, 02.10.2026):
/// dieselben Arten wie im Strom eines Server-Projekts, ohne Abstimmungen
/// (die rechnet nur ein Server aus). Eine feste Liste und keine offene: Ein
/// Mitglied soll hier nichts ablegen koennen, was keine Ansicht kennt.
pub const PLANUNG: &[&str] = &[
    "board",
    "column",
    "card",
    "roadmap",
    "milestone",
    "place_group",
    "place",
    "school_year",
    "school_year_break",
    "subject",
    "week_plan",
    "week_plan_slot",
    "week_plan_period",
];

/// Eine Sache der Planung, wie sie reist.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Planungssache {
    pub art: String,
    pub uuid: String,
    pub eltern: Option<String>,
    pub felder: serde_json::Value,
    pub geaendert_at: String,
}

/// Etwas, das jemand geloescht hat -- damit es nicht vom naechsten Mitglied
/// wiederkommt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Grabstein {
    pub art: String,
    pub uuid: String,
    pub at: String,
}

/// Die Art, unter der fremde, freigegebene Notizen im Projekt liegen.
pub const GETEILTE_NOTIZ: &str = "geteilte_notiz";
/// ... Dateien aus Ordnern (Dateien, Dokumente).
pub const GETEILTE_DATEI: &str = "geteilte_datei";
/// ... Bilder und Videos aus Alben.
pub const GETEILTES_BILD: &str = "geteiltes_bild";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frage {
    pub projekt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeteilteNotiz {
    pub zk_id: String,
    pub titel: String,
    pub inhalt: String,
    pub mappe: Option<String>,
    pub geaendert_at: String,
}

/// Eine Datei oder ein Bild aus einer Freigabe -- ohne Bytes. Die holt das
/// andere Geraet beim Oeffnen (`/openany/v1/projekt/inhalt`), Vorschaubilder
/// gleich beim Abgleich.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GeteilteDatei {
    pub uuid: String,
    /// Der Schluessel der Freigabe, in der sie liegt.
    pub freigabe: String,
    /// Der Weg darunter (Unterordner bzw. Unteralben), mit `/`; leer = oben.
    pub pfad: String,
    pub name: String,
    pub mime: String,
    pub groesse: u64,
    pub abdruck: Option<String>,
    /// Nur bei Bildern: das Vorschaubild.
    #[serde(default)]
    pub vorschau: Option<String>,
    /// Nur bei Dateien: `files` oder `documents`.
    #[serde(default)]
    pub zone: String,
    pub geaendert_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stand {
    pub name: String,
    pub mitgliederliste: Mitgliederliste,
    pub freigaben: Vec<Freigabe>,
    /// Die bekannten Personen der Mitglieder -- fuer ihre weiteren Geraete.
    pub personen: Vec<Person>,
    /// Nur die Notizen der Person, die liefert, aus ihren Freigaben.
    pub notizen: Vec<GeteilteNotiz>,
    /// Ihre Dateien aus freigegebenen Ordnern (ohne Bytes).
    #[serde(default)]
    pub dateien: Vec<GeteilteDatei>,
    /// Ihre Bilder aus freigegebenen Alben (ohne Bytes).
    #[serde(default)]
    pub bilder: Vec<GeteilteDatei>,
    /// Die Chat-Nachrichten (unterschrieben, reisen ueber jeden).
    #[serde(default)]
    pub chat: Vec<crate::chat::Nachricht>,
    /// Die Planung des Projekts -- Boards, Roadmaps, Orte, Schulplanung.
    #[serde(default)]
    pub planung: Vec<Planungssache>,
    #[serde(default)]
    pub grabsteine: Vec<Grabstein>,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct Bericht {
    pub projekt: String,
    pub name: String,
    pub neue_mitglieder: usize,
    pub freigaben: usize,
    pub notizen: usize,
    pub dateien: usize,
    pub bilder: usize,
    /// Wie viele Sachen der Planung neu kamen, sich aenderten oder gingen.
    pub planung: usize,
    /// Dieses Geraet ist nicht mehr Mitglied (entfernt) -- der Aufrufer
    /// raeumt das Projekt weg.
    pub nicht_mehr_mitglied: bool,
    /// Neue Chat-Nachrichten.
    pub chat: usize,
}

/// Die Chat-Nachrichten eines Projekts, aelteste zuerst.
pub fn chat_lesen(
    speicher: &Speicher,
    projekt: &str,
) -> Result<Vec<crate::chat::Nachricht>, String> {
    let mut alle: Vec<crate::chat::Nachricht> = speicher
        .projektsachen(projekt, crate::chat::CHAT)
        .map_err(|e| e.to_string())?
        .into_iter()
        .filter_map(|s| serde_json::from_value(s.felder).ok())
        .collect();
    alle.sort_by(|a, b| a.at.cmp(&b.at).then(a.id.cmp(&b.id)));
    Ok(alle)
}

/// Nachrichten aufnehmen, die gelten und noch nicht da sind. Gibt zurueck,
/// wie viele neu sind.
pub fn chat_aufnehmen(
    speicher: &Speicher,
    g: &dyn Gastgeber,
    projekt: &str,
    nachrichten: &[crate::chat::Nachricht],
) -> Result<usize, String> {
    let (_, liste, _) = lesen(speicher, projekt)?;
    let mut neu = 0;
    for n in nachrichten {
        if !n.gilt(projekt, &liste, &|id| g.person_von(id)) {
            continue;
        }
        if speicher
            .projektsache(projekt, &n.id)
            .map_err(|e| e.to_string())?
            .is_some()
        {
            continue;
        }
        speicher
            .projektsache_schreiben(
                &Projektsache {
                    projekt: projekt.to_string(),
                    art: crate::chat::CHAT.into(),
                    uuid: n.id.clone(),
                    eltern: None,
                    felder: serde_json::to_value(n).map_err(|e| e.to_string())?,
                    geaendert_at: n.at.clone(),
                },
                Protokoll::Still,
            )
            .map_err(|e| e.to_string())?;
        neu += 1;
    }
    Ok(neu)
}

/// Ist das Geraet `fp` gerade das eines Mitglieds?
pub fn ist_mitgliedsgeraet(
    speicher: &Speicher,
    g: &dyn Gastgeber,
    projekt: &str,
    fp: &str,
) -> Result<bool, String> {
    let (_, liste, _) = lesen(speicher, projekt)?;
    let m = mitglieder(&liste, g)?;
    Ok(freigaben::mitglied_von(&m, fp, &|id| g.person_von(id)).is_some())
}

/// Gehoert(e) das Geraet `fp` zu irgendwem, der je in der Liste stand? Wer
/// entfernt wurde, soll das noch erfahren koennen -- aber nichts mehr
/// bekommen als die Liste.
fn je_dabei(liste: &Mitgliederliste, fp: &str, g: &dyn Gastgeber) -> bool {
    liste.eintraege.iter().any(|e| {
        (e.art == "gruendung" || e.art == "beitritt")
            && freigaben::geraet_von(
                &Mitglied {
                    personen_id: e.personen_id.clone(),
                    name: e.name.clone(),
                    rolle: e.rolle.clone(),
                    geraet: e.geraet.clone(),
                },
                fp,
                &|id| g.person_von(id),
            )
    })
}

/// Bin ich (diese Person) Mitglied?
fn bin_dabei(m: &[Mitglied], ich: &Person) -> bool {
    m.iter()
        .any(|x| x.personen_id == ich.personen_id || ich.frueher.contains(&x.personen_id))
}

/// Was Ausgeschiedene geteilt haben, verschwindet hier.
fn ausgeschiedene_raeumen(
    speicher: &Speicher,
    projekt: &str,
    m: &[Mitglied],
) -> Result<(), String> {
    for art in [GETEILTE_NOTIZ, GETEILTE_DATEI, GETEILTES_BILD] {
        for s in speicher
            .projektsachen(projekt, art)
            .map_err(|e| e.to_string())?
        {
            if !m.iter().any(|x| x.personen_id == s.text("von")) {
                speicher
                    .projektsache_entfernen(projekt, &s.uuid, Protokoll::Still)
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}

/// Eine gueltige Fortsetzung der eigenen Liste? Dann gilt sie.
fn setzt_fort(eigene: &Mitgliederliste, neu: &Mitgliederliste, g: &dyn Gastgeber) -> bool {
    neu.projekt == eigene.projekt
        && neu.eintraege.len() > eigene.eintraege.len()
        && neu.eintraege[..eigene.eintraege.len()] == eigene.eintraege[..]
        && mitglieder(neu, g).is_ok()
}

/// Eine Liste annehmen, die ein Geraet herueberschickt (Austritt vor Ort,
/// `/openany/v1/projekt/liste`). Nur von einem Geraet, das je dabei war, und
/// nur als gueltige Fortsetzung der eigenen.
pub fn liste_annehmen(
    speicher: &Speicher,
    g: &dyn Gastgeber,
    projekt: &str,
    fp_anderer: &str,
    neu: &Mitgliederliste,
) -> Result<(), String> {
    let (p, liste, _) = lesen(speicher, projekt)?;
    if !je_dabei(&liste, fp_anderer, g) {
        return Err("Not a member of this project.".into());
    }
    if !setzt_fort(&liste, neu, g) {
        return Err("This list does not continue the one here.".into());
    }
    let m = mitglieder(neu, g)?;
    speicher
        .projekt_lokal_schreiben(&p, &serde_json::to_string(neu).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    ausgeschiedene_raeumen(speicher, projekt, &m)
}

/// Mitgliederliste und Freigaben eines lokalen Projekts aus dem Speicher.
pub fn lesen(
    speicher: &Speicher,
    projekt: &str,
) -> Result<(openany_store::Projekt, Mitgliederliste, Vec<Freigabe>), String> {
    let p = speicher
        .projekt(projekt)
        .map_err(|e| e.to_string())?
        .ok_or("This project does not exist here.")?;
    let liste: Mitgliederliste =
        serde_json::from_str(p.mitgliederliste.as_deref().ok_or("Not a local project.")?)
            .map_err(|e| e.to_string())?;
    let freigaben: Vec<Freigabe> = speicher
        .projekt_freigaben(projekt)
        .map_err(|e| e.to_string())?
        .and_then(|j| serde_json::from_str(&j).ok())
        .unwrap_or_default();
    Ok((p, liste, freigaben))
}

fn mitglieder(liste: &Mitgliederliste, g: &dyn Gastgeber) -> Result<Vec<Mitglied>, String> {
    liste
        .mitglieder(&|id| g.person_von(id))
        .map_err(|e| e.to_string())
}

/// Liefern: den Stand dieses Geraets fuer `anrufer` -- nur, wenn er zu einem
/// Mitglied gehoert.
pub fn stand_bauen(
    speicher: &Speicher,
    g: &dyn Gastgeber,
    projekt: &str,
    anrufer: &str,
) -> Result<Stand, String> {
    let (p, liste, freigaben) = lesen(speicher, projekt)?;
    let m = mitglieder(&liste, g)?;
    if freigaben::mitglied_von(&m, anrufer, &|id| g.person_von(id)).is_none() {
        // Wer entfernt wurde, erfaehrt es noch -- mit der Liste und nichts
        // sonst.
        if je_dabei(&liste, anrufer, g) {
            return Ok(Stand {
                name: p.name,
                mitgliederliste: liste,
                freigaben: Vec::new(),
                personen: Vec::new(),
                notizen: Vec::new(),
                dateien: Vec::new(),
                bilder: Vec::new(),
                chat: Vec::new(),
                planung: Vec::new(),
                grabsteine: Vec::new(),
            });
        }
        return Err("Not a member of this project.".into());
    }
    let (notizen, dateien, bilder) = meine_geteilten(speicher, g, projekt, &m, &freigaben)?;
    let personen = m
        .iter()
        .filter_map(|m| g.person_von(&m.personen_id))
        .collect();
    Ok(Stand {
        name: p.name,
        mitgliederliste: liste,
        freigaben,
        personen,
        notizen,
        dateien,
        bilder,
        chat: {
            let alle = chat_lesen(speicher, projekt)?;
            let ab = alle.len().saturating_sub(crate::chat::IM_STAND);
            alle[ab..].to_vec()
        },
        planung: planung_lesen(speicher, projekt)?,
        grabsteine: grabsteine_lesen(speicher, projekt)?,
    })
}

/// Die Planung dieses Projekts, wie sie reist.
pub fn planung_lesen(speicher: &Speicher, projekt: &str) -> Result<Vec<Planungssache>, String> {
    let mut liste = Vec::new();
    for art in PLANUNG {
        for s in speicher
            .projektsachen(projekt, art)
            .map_err(|e| e.to_string())?
        {
            liste.push(Planungssache {
                art: s.art,
                uuid: s.uuid,
                eltern: s.eltern,
                felder: s.felder,
                geaendert_at: s.geaendert_at,
            });
        }
    }
    Ok(liste)
}

fn grabsteine_lesen(speicher: &Speicher, projekt: &str) -> Result<Vec<Grabstein>, String> {
    Ok(speicher
        .projekt_grabsteine(projekt)
        .map_err(|e| e.to_string())?
        .into_iter()
        .filter(|(art, _, _)| PLANUNG.contains(&art.as_str()))
        .map(|(art, uuid, at)| Grabstein { art, uuid, at })
        .collect())
}

/// Ein Zeitpunkt zum Vergleichen. RFC 3339 ist nur in EINER Schreibweise
/// als Text vergleichbar; Geraete schreiben Bruchteile verschieden lang.
fn zeit(t: &str) -> Option<chrono::DateTime<chrono::FixedOffset>> {
    chrono::DateTime::parse_from_rfc3339(t).ok()
}

/// Ist `a` juenger als `b`? Unlesbar gilt als aelter.
fn juenger(a: &str, b: &str) -> bool {
    match (zeit(a), zeit(b)) {
        (Some(a), Some(b)) => a > b,
        (Some(_), None) => true,
        _ => false,
    }
}

/// Die Planung eines anderen Mitglieds aufnehmen.
///
/// **Die juengste Aenderung gewinnt**, je Sache -- dieselbe Regel wie auf
/// dem Server (`entscheidung: letztes`). Ein Grabstein zaehlt mit: Wer
/// loeschte, nachdem der andere zuletzt schrieb, hat das letzte Wort.
///
/// **Planung reist ueber jeden**, anders als Notizen: Sie ist nicht
/// unterschrieben, aber jedes Mitglied darf sie ohnehin bearbeiten. Ein
/// Geraet, das sie weiterreicht, koennte also nichts, was es nicht auch
/// selbst duerfte. Angenommen wird sie nur von einem Mitgliedsgeraet (das
/// prueft [`stand_uebernehmen`] vorher).
pub fn planung_aufnehmen(
    speicher: &Speicher,
    projekt: &str,
    planung: &[Planungssache],
    grabsteine: &[Grabstein],
) -> Result<usize, String> {
    let fehler = |e: openany_store::SpeicherFehler| e.to_string();
    let hier_tot: std::collections::HashMap<String, String> = speicher
        .projekt_grabsteine(projekt)
        .map_err(fehler)?
        .into_iter()
        .map(|(_, uuid, at)| (uuid, at))
        .collect();
    let mut geaendert = 0;

    for s in planung {
        let lesbar = PLANUNG.contains(&s.art.as_str())
            && !s.uuid.is_empty()
            && s.uuid.len() <= 64
            && s.felder.is_object()
            && s.felder.to_string().len() <= 100_000
            && zeit(&s.geaendert_at).is_some();
        if !lesbar {
            continue;
        }
        if hier_tot
            .get(&s.uuid)
            .is_some_and(|tot| !juenger(&s.geaendert_at, tot))
        {
            continue;
        }
        match speicher.projektsache(projekt, &s.uuid).map_err(fehler)? {
            // Dieselbe uuid als andere Art waere ein Fehler drueben -- dann
            // lieber die eigene behalten.
            Some(alt) if alt.art != s.art || !juenger(&s.geaendert_at, &alt.geaendert_at) => {
                continue
            }
            _ => {}
        }
        speicher
            .projektsache_schreiben(
                &Projektsache {
                    projekt: projekt.to_string(),
                    art: s.art.clone(),
                    uuid: s.uuid.clone(),
                    eltern: s.eltern.clone(),
                    felder: s.felder.clone(),
                    geaendert_at: s.geaendert_at.clone(),
                },
                Protokoll::Still,
            )
            .map_err(fehler)?;
        geaendert += 1;
    }

    // Grabsteine nach den Sachen: Ein Behaelter, der drueben geloescht ist,
    // nimmt auch Kinder mit, die eben erst ankamen.
    for t in grabsteine {
        if !PLANUNG.contains(&t.art.as_str()) {
            continue;
        }
        let Some(alt) = speicher.projektsache(projekt, &t.uuid).map_err(fehler)? else {
            continue;
        };
        if alt.art == t.art && !juenger(&alt.geaendert_at, &t.at) {
            // Gemerkt, damit dieses Geraet den Grabstein weiterreicht.
            speicher
                .projektsache_entfernen(projekt, &t.uuid, Protokoll::Merken)
                .map_err(fehler)?;
            geaendert += 1;
        }
    }

    Ok(geaendert)
}

type Geteiltes = (Vec<GeteilteNotiz>, Vec<GeteilteDatei>, Vec<GeteilteDatei>);

/// Was DIESES Geraet in ein Projekt freigibt -- Notizen, Dateien und Bilder
/// der eigenen Person aus ihren geltenden Freigaben.
pub fn meine_geteilten(
    speicher: &Speicher,
    g: &dyn Gastgeber,
    projekt: &str,
    m: &[Mitglied],
    freigaben: &[Freigabe],
) -> Result<Geteiltes, String> {
    let fehler = |e: openany_store::SpeicherFehler| e.to_string();
    let ich = g.person().ok_or("This device has no person yet.")?;
    let meine: Vec<&Freigabe> = freigaben::geltende(freigaben, projekt, m, &|id| g.person_von(id))
        .into_iter()
        .filter(|f| f.personen_id == ich.personen_id || ich.frueher.contains(&f.personen_id))
        .collect();

    let mappen: Vec<&&Freigabe> = meine.iter().filter(|f| f.art == NOTIZ_MAPPE).collect();
    let notizen = if mappen.is_empty() {
        Vec::new()
    } else {
        speicher
            .notizen()
            .map_err(fehler)?
            .into_iter()
            .filter(|n| !n.im_papierkorb())
            .filter(|n| {
                mappen
                    .iter()
                    .any(|f| freigaben::in_mappe(n.mappe.as_deref(), &f.schluessel))
            })
            .map(|n| GeteilteNotiz {
                zk_id: n.zk_id,
                titel: n.titel,
                inhalt: n.inhalt,
                mappe: n.mappe,
                geaendert_at: n.geaendert_at,
            })
            .collect()
    };

    let mut dateien = Vec::new();
    for f in meine.iter().filter(|f| f.art == ORDNER) {
        let lebt = speicher
            .datei(&f.schluessel)
            .map_err(fehler)?
            .is_some_and(|w| w.ist_ordner && w.papierkorb_at.is_none());
        if !lebt {
            continue;
        }
        for uuid in speicher.ordner_mit_inhalt(&f.schluessel).map_err(fehler)? {
            let Some(d) = speicher.datei(&uuid).map_err(fehler)? else {
                continue;
            };
            if d.ist_ordner || d.papierkorb_at.is_some() {
                continue;
            }
            // Der Weg unterhalb der Freigabe.
            let weg = speicher.ordnerweg(d.eltern.as_deref()).map_err(fehler)?;
            let ab = weg
                .iter()
                .position(|o| o.uuid == f.schluessel)
                .map_or(weg.len(), |i| i + 1);
            let pfad = weg[ab..]
                .iter()
                .map(|o| o.name.as_str())
                .collect::<Vec<_>>()
                .join("/");
            dateien.push(GeteilteDatei {
                uuid: d.uuid,
                freigabe: f.schluessel.clone(),
                pfad,
                name: d.name,
                mime: d.mime,
                groesse: d.groesse,
                abdruck: d.abdruck,
                vorschau: None,
                zone: d.zone,
                geaendert_at: d.geaendert_at,
            });
        }
    }

    let mut bilder = Vec::new();
    for f in meine.iter().filter(|f| f.art == ALBUM) {
        let lebt = speicher
            .album(&f.schluessel)
            .map_err(fehler)?
            .is_some_and(|a| a.papierkorb_at.is_none());
        if !lebt {
            continue;
        }
        for album in speicher
            .album_mit_unteralben(&f.schluessel)
            .map_err(fehler)?
        {
            // Der Weg unterhalb der Freigabe: die Alben nach ihr, und das
            // Album selbst, wenn es nicht die Freigabe ist.
            let mut weg = speicher.albumweg(&album).map_err(fehler)?;
            if let Some(a) = speicher.album(&album).map_err(fehler)? {
                weg.push(a);
            }
            let ab = weg
                .iter()
                .position(|a| a.uuid == f.schluessel)
                .map_or(weg.len(), |i| i + 1);
            let pfad = weg[ab..]
                .iter()
                .map(|a| a.name.as_str())
                .collect::<Vec<_>>()
                .join("/");
            for b in speicher.bilder_im_album(Some(&album)).map_err(fehler)? {
                if b.papierkorb_at.is_some() {
                    continue;
                }
                bilder.push(GeteilteDatei {
                    uuid: b.uuid,
                    freigabe: f.schluessel.clone(),
                    pfad: pfad.clone(),
                    name: b.name,
                    mime: b.mime,
                    groesse: b.groesse,
                    abdruck: b.abdruck,
                    vorschau: b.vorschau,
                    zone: String::new(),
                    geaendert_at: b.geaendert_at,
                });
            }
        }
    }
    Ok((notizen, dateien, bilder))
}

/// Darf `anrufer` diesen Inhalt aus diesem Projekt bekommen? Nur ein
/// Mitglied, und nur ein Abdruck aus einer EIGENEN geltenden Freigabe dieses
/// Geraets (Original oder Vorschau). `Some(groesse)`: ja (bei einer Vorschau
/// unbekannt, dann `u64::MAX`).
pub fn inhalt_erlaubt(
    speicher: &Speicher,
    g: &dyn Gastgeber,
    projekt: &str,
    anrufer: &str,
    abdruck: &str,
) -> Result<Option<u64>, String> {
    let (_, liste, freigaben) = lesen(speicher, projekt)?;
    let m = mitglieder(&liste, g)?;
    if freigaben::mitglied_von(&m, anrufer, &|id| g.person_von(id)).is_none() {
        return Ok(None);
    }
    let (_, dateien, bilder) = meine_geteilten(speicher, g, projekt, &m, &freigaben)?;
    Ok(dateien.iter().chain(bilder.iter()).find_map(|d| {
        if d.abdruck.as_deref() == Some(abdruck) {
            Some(d.groesse)
        } else if d.vorschau.as_deref() == Some(abdruck) {
            Some(u64::MAX)
        } else {
            None
        }
    }))
}

/// Darf das Geraet `anrufer` diese Notiz bearbeiten? Nur ein Mitglied, und
/// nur eine Notiz aus einer EIGENEN geltenden Freigabe dieses Geraets mit
/// der Stufe „bearbeiten". Gibt das Mitglied und die Notiz zurueck.
pub fn bearbeitbare_notiz(
    speicher: &Speicher,
    g: &dyn Gastgeber,
    projekt: &str,
    anrufer: &str,
    zk_id: &str,
) -> Result<(Mitglied, openany_store::Notiz), String> {
    let (_, liste, freigaben) = lesen(speicher, projekt)?;
    let m = mitglieder(&liste, g)?;
    let wer = freigaben::mitglied_von(&m, anrufer, &|id| g.person_von(id))
        .ok_or("Not a member of this project.")?
        .clone();
    let ich = g.person().ok_or("This device has no person yet.")?;
    let notiz = speicher
        .notiz(zk_id)
        .map_err(|e| e.to_string())?
        .filter(|n| !n.im_papierkorb())
        .ok_or("This note no longer exists here.")?;
    let erlaubt = freigaben::geltende(&freigaben, projekt, &m, &|id| g.person_von(id))
        .into_iter()
        .any(|f| {
            (f.personen_id == ich.personen_id || ich.frueher.contains(&f.personen_id))
                && f.art == NOTIZ_MAPPE
                && f.stufe == freigaben::BEARBEITEN
                && freigaben::in_mappe(notiz.mappe.as_deref(), &f.schluessel)
        });
    if !erlaubt {
        return Err("This note is not shared for editing.".into());
    }
    Ok((wer, notiz))
}

/// Annehmen: den Stand eines anderen Geraets (`fp_anderer`) pruefen und
/// uebernehmen, was gilt.
pub fn stand_uebernehmen(
    speicher: &Speicher,
    g: &dyn Gastgeber,
    projekt: &str,
    fp_anderer: &str,
    stand: &Stand,
) -> Result<Bericht, String> {
    let fehler = |e: openany_store::SpeicherFehler| e.to_string();
    let (p, mut liste, mut freigaben) = lesen(speicher, projekt)?;
    let ich = g.person().ok_or("This device has no person yet.")?;
    let vorher = mitglieder(&liste, g)?.len();

    // Personen zuerst: Sie entscheiden, welche Geraete zu wem gehoeren. Nur
    // die von Mitgliedern, und nie die eigene (die kommt nur vom Paaren).
    let ids: Vec<String> = stand
        .mitgliederliste
        .mitglieder(&|id| stand.personen.iter().find(|p| p.personen_id == id).cloned())
        .map(|m| m.into_iter().map(|m| m.personen_id).collect())
        .unwrap_or_default();
    for person in &stand.personen {
        if ids.contains(&person.personen_id)
            && person.personen_id != ich.personen_id
            && !ich.frueher.contains(&person.personen_id)
        {
            g.person_fremd_merken(person.clone());
        }
    }

    // Die Mitgliederliste: nur eine gueltige Fortsetzung der eigenen.
    if setzt_fort(&liste, &stand.mitgliederliste, g) {
        liste = stand.mitgliederliste.clone();
    }
    let m = mitglieder(&liste, g)?;

    // Entfernt? Dann nimmt dieses Geraet nichts mehr an; der Aufrufer raeumt
    // das Projekt weg.
    if !bin_dabei(&m, &ich) {
        return Ok(Bericht {
            projekt: projekt.to_string(),
            name: p.name.clone(),
            nicht_mehr_mitglied: true,
            ..Default::default()
        });
    }
    ausgeschiedene_raeumen(speicher, projekt, &m)?;

    // Wer liefert? Ohne Mitgliedschaft nimmt dieses Geraet nichts an.
    let liefernd = freigaben::mitglied_von(&m, fp_anderer, &|id| g.person_von(id))
        .ok_or("The other device belongs to no member.")?
        .clone();

    // Freigaben: nur, was gilt.
    let gueltige: Vec<Freigabe> = stand
        .freigaben
        .iter()
        .filter(|f| f.gilt(projekt, &m, &|id| g.person_von(id)))
        .cloned()
        .collect();
    freigaben::zusammenlegen(&mut freigaben, &gueltige);

    // Inhalte: nur die der liefernden Person, aus ihren geltenden Freigaben.
    // Was sie nicht mehr freigibt, verschwindet hier. Die eigene Person
    // liefert keine -- ihre Sachen hat dieses Geraet selbst.
    let mut bericht = Bericht {
        projekt: projekt.to_string(),
        name: p.name.clone(),
        ..Default::default()
    };
    if liefernd.personen_id != ich.personen_id && !ich.frueher.contains(&liefernd.personen_id) {
        let ihre: Vec<Freigabe> =
            freigaben::geltende(&freigaben, projekt, &m, &|id| g.person_von(id))
                .into_iter()
                .filter(|f| f.personen_id == liefernd.personen_id)
                .cloned()
                .collect();
        for art in [GETEILTE_NOTIZ, GETEILTE_DATEI, GETEILTES_BILD] {
            for alt in speicher.projektsachen(projekt, art).map_err(fehler)? {
                if alt.text("von") == liefernd.personen_id {
                    speicher
                        .projektsache_entfernen(projekt, &alt.uuid, Protokoll::Still)
                        .map_err(fehler)?;
                }
            }
        }
        let ablegen = |art: &str, id: &str, felder: serde_json::Value, at: &str| {
            speicher
                .projektsache_schreiben(
                    &Projektsache {
                        projekt: projekt.to_string(),
                        art: art.into(),
                        uuid: format!("{}:{}", liefernd.personen_id, id),
                        eltern: None,
                        felder,
                        geaendert_at: at.to_string(),
                    },
                    Protokoll::Still,
                )
                .map_err(fehler)
        };

        for n in &stand.notizen {
            let passt = ihre.iter().any(|f| {
                f.art == NOTIZ_MAPPE && freigaben::in_mappe(n.mappe.as_deref(), &f.schluessel)
            });
            if passt {
                ablegen(
                    GETEILTE_NOTIZ,
                    &n.zk_id,
                    serde_json::json!({
                        "titel": n.titel,
                        "inhalt": n.inhalt,
                        "mappe": n.mappe,
                        "von": liefernd.personen_id,
                        "von_name": liefernd.name,
                    }),
                    &n.geaendert_at,
                )?;
                bericht.notizen += 1;
            }
        }
        for (liste, art, freigabe_art) in [
            (&stand.dateien, GETEILTE_DATEI, ORDNER),
            (&stand.bilder, GETEILTES_BILD, ALBUM),
        ] {
            for d in liste {
                if !ihre
                    .iter()
                    .any(|f| f.art == freigabe_art && f.schluessel == d.freigabe)
                {
                    continue;
                }
                ablegen(
                    art,
                    &d.uuid,
                    serde_json::json!({
                        "freigabe": d.freigabe,
                        "pfad": d.pfad,
                        "name": d.name,
                        "mime": d.mime,
                        "groesse": d.groesse,
                        "abdruck": d.abdruck,
                        "vorschau": d.vorschau,
                        "zone": d.zone,
                        "von": liefernd.personen_id,
                        "von_name": liefernd.name,
                    }),
                    &d.geaendert_at,
                )?;
                if art == GETEILTE_DATEI {
                    bericht.dateien += 1;
                } else {
                    bericht.bilder += 1;
                }
            }
        }
    }

    let json = serde_json::to_string(&liste).map_err(|e| e.to_string())?;
    speicher
        .projekt_lokal_schreiben(&p, &json)
        .map_err(fehler)?;
    speicher
        .projekt_freigaben_schreiben(
            projekt,
            &serde_json::to_string(&freigaben).map_err(|e| e.to_string())?,
        )
        .map_err(fehler)?;

    // Der Chat: nachdem die Liste steht -- sie entscheidet, wer schreiben darf.
    bericht.chat = chat_aufnehmen(speicher, g, projekt, &stand.chat)?;
    bericht.planung = planung_aufnehmen(speicher, projekt, &stand.planung, &stand.grabsteine)?;
    bericht.neue_mitglieder = m.len().saturating_sub(vorher);
    bericht.freigaben = freigaben::geltende(&freigaben, projekt, &m, &|id| g.person_von(id)).len();
    Ok(bericht)
}

#[cfg(test)]
mod planung_tests {
    use super::*;

    fn sache(uuid: &str, name: &str, at: &str) -> Planungssache {
        Planungssache {
            art: "board".into(),
            uuid: uuid.into(),
            eltern: None,
            felder: serde_json::json!({ "name": name }),
            geaendert_at: at.into(),
        }
    }

    fn name(s: &Speicher, uuid: &str) -> Option<String> {
        s.projektsache("p1", uuid)
            .unwrap()
            .map(|x| x.text("name").to_string())
    }

    #[test]
    fn die_juengere_aenderung_gewinnt_auch_bei_anderer_schreibweise() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        let mal =
            |at: &str, n: &str| planung_aufnehmen(&s, "p1", &[sache("b1", n, at)], &[]).unwrap();

        assert_eq!(mal("2026-10-02T10:00:00+00:00", "Eins"), 1);
        // Juenger, mit Bruchteilen geschrieben.
        assert_eq!(mal("2026-10-02T10:00:00.5+00:00", "Zwei"), 1);
        assert_eq!(name(&s, "b1").as_deref(), Some("Zwei"));
        // Aelter, in anderer Zone geschrieben: verliert.
        assert_eq!(mal("2026-10-02T12:00:00+02:00", "Alt"), 0);
        assert_eq!(name(&s, "b1").as_deref(), Some("Zwei"));
    }

    #[test]
    fn fremdes_und_unlesbares_bleibt_draussen() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        let mut chat = sache("x1", "?", "2026-10-02T10:00:00+00:00");
        chat.art = "chat".into();
        let ohne_zeit = sache("b2", "?", "");
        assert_eq!(
            planung_aufnehmen(&s, "p1", &[chat, ohne_zeit], &[]).unwrap(),
            0
        );
    }

    #[test]
    fn ein_grabstein_schlaegt_nur_aeltere_fassungen() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        planung_aufnehmen(
            &s,
            "p1",
            &[sache("b1", "Beete", "2020-10-02T10:00:00+00:00")],
            &[],
        )
        .unwrap();
        let stein = |at: &str| Grabstein {
            art: "board".into(),
            uuid: "b1".into(),
            at: at.into(),
        };

        // Geloescht VOR der letzten Aenderung: bleibt.
        assert_eq!(
            planung_aufnehmen(&s, "p1", &[], &[stein("2020-10-02T09:00:00+00:00")]).unwrap(),
            0
        );
        assert!(name(&s, "b1").is_some());
        // Danach: fort -- und kommt mit dem alten Stand nicht wieder.
        assert_eq!(
            planung_aufnehmen(&s, "p1", &[], &[stein("2020-10-02T11:00:00+00:00")]).unwrap(),
            1
        );
        assert_eq!(
            planung_aufnehmen(
                &s,
                "p1",
                &[sache("b1", "Beete", "2020-10-02T10:00:00+00:00")],
                &[]
            )
            .unwrap(),
            0
        );
        assert!(name(&s, "b1").is_none());
    }
}
