//! Abgleich eines LOKALEN Projekts zwischen Mitgliedern -- vor Ort, ohne
//! Server (docs/konzept-lokale-mitgliedschaften.md, §6).
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
use crate::freigaben::{self, Freigabe, ALBUM, KALENDER, NOTIZ_MAPPE, ORDNER};
use crate::mitglieder::{Mitglied, Mitgliederliste};
use crate::Person;
use openany_store::{Projektsache, Protokoll, Speicher};
use serde::{Deserialize, Serialize};

/// Die Art, unter der fremde, freigegebene Notizen im Projekt liegen.
pub const GETEILTE_NOTIZ: &str = "geteilte_notiz";
/// ... Dateien aus Ordnern (Dateien, Dokumente).
pub const GETEILTE_DATEI: &str = "geteilte_datei";
/// ... Bilder und Videos aus Alben.
pub const GETEILTES_BILD: &str = "geteiltes_bild";
/// Ein Kalender, den ein anderes Mitglied vor Ort teilt (E7) -- samt Terminen.
pub const GETEILTER_KALENDER: &str = "geteilter_kalender";

/// Ein geteilter Kalender, wie er im Stand reist: nur von seiner Besitzerin.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeteilterKalender {
    /// Die uuid des Kalenders bei der Besitzerin.
    pub kalender: String,
    pub name: String,
    pub farbe: String,
    /// Je Termin: uuid und Felder, wie der Abgleich sie kennt.
    pub termine: Vec<(String, serde_json::Map<String, serde_json::Value>)>,
}

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
    /// Anheften, Loesen, Loeschen (E4) -- unterschrieben wie die Nachrichten.
    /// Aeltere Geraete kennen das Feld nicht und lassen es aus.
    #[serde(default)]
    pub aktionen: Vec<crate::chat::Aktion>,
    /// Die Planung als unterschriebene Eintraege (E5, planung.rs). Aeltere
    /// Geraete kennen das Feld nicht und lassen es aus.
    #[serde(default)]
    pub planung_eintraege: Vec<crate::planung::Eintrag>,
    /// Die eigenen geteilten Kalender der liefernden Person (E7). Aeltere
    /// Geraete kennen das Feld nicht und lassen es aus.
    #[serde(default)]
    pub kalender: Vec<GeteilterKalender>,
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

/// Die Planungseintraege eines Projekts (E5), je Eintrag die geltende
/// Fassung, aelteste zuerst (nach Zeit, dann id -- diese Reihenfolge zaehlt
/// fuer Plaetze, auf allen Geraeten gleich).
pub fn planung_eintraege_lesen(
    speicher: &Speicher,
    projekt: &str,
) -> Result<Vec<crate::planung::Eintrag>, String> {
    let mut alle: Vec<crate::planung::Eintrag> = speicher
        .projektsachen(projekt, crate::planung::EINTRAG)
        .map_err(|e| e.to_string())?
        .into_iter()
        .filter_map(|s| serde_json::from_value(s.felder).ok())
        .collect();
    alle.sort_by(|a, b| a.at.cmp(&b.at).then(a.id.cmp(&b.id)));
    Ok(alle)
}

/// Planungseintraege aufnehmen: echt, von Mitgliedern, und nur, wo sie die
/// vorhandene Fassung ersetzen duerfen (`planung::ersetzt`). Gibt zurueck,
/// wie viele etwas aenderten.
pub fn planung_eintraege_aufnehmen(
    speicher: &Speicher,
    g: &dyn Gastgeber,
    projekt: &str,
    eintraege: &[crate::planung::Eintrag],
) -> Result<usize, String> {
    let (_, liste, _) = lesen(speicher, projekt)?;
    let person = |id: &str| g.person_von(id);
    let eigentuemer = crate::planung::eigentuemer(&liste, &person);
    let mut sortiert: Vec<&crate::planung::Eintrag> = eintraege.iter().collect();
    // Die Anlagen zuerst: Sie sagen, wer eine Sache angelegt hat.
    sortiert.sort_by_key(|e| (e.id != e.sache, e.at.clone()));
    let mut neu = 0;
    for e in sortiert {
        if !e.gilt(projekt, &liste, &person) {
            eprintln!(
                "Board {projekt}: Eintrag {} verworfen (Geraet {} gilt nicht fuer Person {})",
                e.id, e.unterzeichner, e.personen_id
            );
            continue;
        }
        let lesen_eins = |id: &str| -> Result<Option<crate::planung::Eintrag>, String> {
            Ok(speicher
                .projektsache(projekt, id)
                .map_err(|e| e.to_string())?
                .filter(|s| s.art == crate::planung::EINTRAG)
                .and_then(|s| serde_json::from_value(s.felder).ok()))
        };
        if let Some(alt) = lesen_eins(&e.id)? {
            let anlegerin = lesen_eins(&e.sache)?.map(|a| a.personen_id);
            if !crate::planung::ersetzt(&alt, e, anlegerin.as_deref(), &eigentuemer) {
                continue;
            }
        }
        speicher
            .projektsache_schreiben(
                &Projektsache {
                    projekt: projekt.to_string(),
                    art: crate::planung::EINTRAG.into(),
                    uuid: e.id.clone(),
                    eltern: Some(e.sache.clone()),
                    felder: serde_json::to_value(e).map_err(|e| e.to_string())?,
                    geaendert_at: e.at.clone(),
                },
                Protokoll::Still,
            )
            .map_err(|e| e.to_string())?;
        neu += 1;
    }
    Ok(neu)
}

/// Die Aktionen am Chat eines Projekts (E4), aelteste zuerst.
pub fn aktionen_lesen(
    speicher: &Speicher,
    projekt: &str,
) -> Result<Vec<crate::chat::Aktion>, String> {
    let mut alle: Vec<crate::chat::Aktion> = speicher
        .projektsachen(projekt, crate::chat::AKTION)
        .map_err(|e| e.to_string())?
        .into_iter()
        .filter_map(|s| serde_json::from_value(s.felder).ok())
        .collect();
    alle.sort_by(|a, b| a.at.cmp(&b.at).then(a.id.cmp(&b.id)));
    Ok(alle)
}

/// Aktionen aufnehmen, die gelten und noch nicht da sind. Gibt zurueck, wie
/// viele neu sind.
pub fn aktionen_aufnehmen(
    speicher: &Speicher,
    g: &dyn Gastgeber,
    projekt: &str,
    aktionen: &[crate::chat::Aktion],
) -> Result<usize, String> {
    let (_, liste, _) = lesen(speicher, projekt)?;
    let mut neu = 0;
    for a in aktionen {
        if !a.gilt(projekt, &liste, &|id| g.person_von(id))
            || speicher
                .projektsache(projekt, &a.id)
                .map_err(|e| e.to_string())?
                .is_some()
        {
            continue;
        }
        speicher
            .projektsache_schreiben(
                &Projektsache {
                    projekt: projekt.to_string(),
                    art: crate::chat::AKTION.into(),
                    uuid: a.id.clone(),
                    eltern: None,
                    felder: serde_json::to_value(a).map_err(|e| e.to_string())?,
                    geaendert_at: a.at.clone(),
                },
                Protokoll::Still,
            )
            .map_err(|e| e.to_string())?;
        neu += 1;
    }
    Ok(neu)
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
        if let Some(grund) = n.warum_nicht(projekt, &liste, &|id| g.person_von(id)) {
            eprintln!("Hub-Chat {projekt}: Nachricht {} verworfen: {grund}", n.id);
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
                kalender: Vec::new(),
                chat: Vec::new(),
                aktionen: Vec::new(),
                planung_eintraege: Vec::new(),
            });
        }
        return Err("Not a member of this project.".into());
    }
    let (notizen, dateien, bilder) = meine_geteilten(speicher, g, projekt, &m, &freigaben)?;
    let kalender = meine_kalender(speicher, g, projekt, &m, &freigaben)?;
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
        kalender,
        chat: {
            let alle = chat_lesen(speicher, projekt)?;
            let ab = alle.len().saturating_sub(crate::chat::IM_STAND);
            alle[ab..].to_vec()
        },
        aktionen: aktionen_lesen(speicher, projekt)?,
        planung_eintraege: planung_eintraege_lesen(speicher, projekt)?,
    })
}

type Geteiltes = (Vec<GeteilteNotiz>, Vec<GeteilteDatei>, Vec<GeteilteDatei>);

/// Was DIESES Geraet in ein Projekt freigibt -- Notizen, Dateien und Bilder
/// der eigenen Person aus ihren geltenden Freigaben.
/// Die eigenen Kalender, die diese Person vor Ort geteilt hat -- mit allen
/// Terminen (E7). Was im Papierkorb liegt, reist nicht.
pub fn meine_kalender(
    speicher: &Speicher,
    g: &dyn Gastgeber,
    projekt: &str,
    m: &[Mitglied],
    freigaben: &[Freigabe],
) -> Result<Vec<GeteilterKalender>, String> {
    let fehler = |e: openany_store::SpeicherFehler| e.to_string();
    let ich = g.person().ok_or("This device has no person yet.")?;
    let mut aus = Vec::new();
    for f in freigaben::geltende(freigaben, projekt, m, &|id| g.person_von(id)) {
        if f.art != KALENDER
            || !(f.personen_id == ich.personen_id || ich.frueher.contains(&f.personen_id))
        {
            continue;
        }
        let Some(k) = speicher.kalender(&f.schluessel).map_err(fehler)? else {
            continue;
        };
        if k.papierkorb_at.is_some() {
            continue;
        }
        aus.push(GeteilterKalender {
            kalender: k.uuid.clone(),
            name: k.name.clone(),
            farbe: k.farbe.clone(),
            termine: speicher
                .termine_im_kalender(&k.uuid)
                .map_err(fehler)?
                .into_iter()
                .filter(|t| t.papierkorb_at.is_none())
                .map(|t| (t.uuid.clone(), t.felder.als_eintrag()))
                .collect(),
        });
    }
    Ok(aus)
}

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
        for art in [
            GETEILTE_NOTIZ,
            GETEILTE_DATEI,
            GETEILTES_BILD,
            GETEILTER_KALENDER,
        ] {
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

        for k in &stand.kalender {
            if ihre
                .iter()
                .any(|f| f.art == KALENDER && f.schluessel == k.kalender)
            {
                ablegen(
                    GETEILTER_KALENDER,
                    &k.kalender,
                    serde_json::json!({
                        "kalender": k.kalender,
                        "name": k.name,
                        "farbe": k.farbe,
                        "termine": k.termine,
                        "von": liefernd.personen_id,
                        "von_name": liefernd.name,
                    }),
                    "",
                )?;
            }
        }
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
    bericht.chat = chat_aufnehmen(speicher, g, projekt, &stand.chat)?
        + aktionen_aufnehmen(speicher, g, projekt, &stand.aktionen)?;
    bericht.planung = planung_eintraege_aufnehmen(speicher, g, projekt, &stand.planung_eintraege)?;
    bericht.neue_mitglieder = m.len().saturating_sub(vorher);
    bericht.freigaben = freigaben::geltende(&freigaben, projekt, &m, &|id| g.person_von(id)).len();
    Ok(bericht)
}

#[cfg(test)]
mod kalender_tests {
    use super::*;

    /// So legt `stand_uebernehmen` einen Kalender ab; die App liest ihn mit
    /// demselben Typ zurueck (E7).
    #[test]
    fn ein_abgelegter_kalender_liest_sich_zurueck() {
        let mut termin = serde_json::Map::new();
        termin.insert("title".into(), serde_json::json!("Beispiel"));
        let abgelegt = serde_json::json!({
            "kalender": "k-1",
            "name": "Arbeit",
            "farbe": "#123456",
            "termine": [["t-1", termin]],
            "von": "person-1",
            "von_name": "Beispiel Probe",
        });
        let k: GeteilterKalender = serde_json::from_value(abgelegt).unwrap();
        assert_eq!(k.kalender, "k-1");
        assert_eq!(k.termine.len(), 1);
        assert_eq!(k.termine[0].1["title"], "Beispiel");
    }
}
