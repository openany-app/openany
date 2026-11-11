//! Ein Kalenderdienst nur auf diesem Rechner -- CalDAV für Thunderbird,
//! Apple Kalender, Evolution & Co. (docs/plan-kalender-kontakte-im-system.md,
//! K5). Nur in der Desktop-App.
//!
//! Auf dem Desktop gibt es keinen gemeinsamen Kalender, den alle Programme
//! lesen wie unter Android. Was sie aber alle sprechen, ist CalDAV. Also
//! öffnet die App einen kleinen Dienst auf `127.0.0.1` (nie im Netz), mit
//! Anmeldung -- auf einem Rechner mit mehreren Benutzern läse sonst jeder
//! mit. Die Termine bleiben Ende-zu-Ende verschlüsselt auf openany.de; hier
//! sind sie nur für diesen Rechner entschlüsselt, wie in der App selbst.
//!
//! **Nur die Teile, die diese Programme benutzen:** `OPTIONS`, `PROPFIND`
//! (Depth 0/1), `REPORT` (`calendar-query`, `calendar-multiget`), `GET`,
//! `PUT` (mit `If-Match`/`If-None-Match`), `DELETE`. Die Termine reisen als
//! ICS über `openany_sync::ics` -- dieselbe Abbildung wie Ein- und Ausgabe in
//! der App. ETag eines Termins ist sein Abdruck, `getctag` eines Kalenders
//! der Abdruck über alle seine Termine.
//!
//! **Aufbau:** `/dav/` ist der einzige Benutzer, `/dav/kalender/` sein
//! Zuhause, darunter je Kalender eine Sammlung und je Termin `<name>.ics`.
//! Eigene Kalender sind schreibbar, in Hubs geteilte nur lesbar.

#![cfg_attr(any(target_os = "android", target_os = "ios"), allow(dead_code))]

use openany_client::Terminfelder;
use openany_store::{Davname, Protokoll, Speicher, Termin};
use std::collections::HashMap;

const ART: &str = "termin";
const WURZEL: &str = "/dav/";
const ZUHAUSE: &str = "/dav/kalender/";

/// Eine Anfrage, schon ohne HTTP drumherum.
pub(crate) struct Anfrage {
    pub methode: String,
    pub pfad: String,
    pub tiefe: u8,
    pub koerper: String,
    pub if_match: Option<String>,
    pub if_none_match: Option<String>,
}

#[derive(Debug)]
pub(crate) struct Antwort {
    pub status: u16,
    pub kopf: Vec<(&'static str, String)>,
    pub koerper: String,
}

impl Antwort {
    pub(crate) fn leer(status: u16) -> Self {
        Self {
            status,
            kopf: Vec::new(),
            koerper: String::new(),
        }
    }
    pub(crate) fn xml(koerper: String) -> Self {
        Self {
            status: 207,
            kopf: vec![("Content-Type", "application/xml; charset=utf-8".into())],
            koerper,
        }
    }
}

/// Ein in einem Hub geteilter Kalender, schon gelesen (nur lesend).
pub(crate) struct Fremd {
    pub schluessel: String,
    pub name: String,
    pub farbe: String,
    pub termine: Vec<(String, Terminfelder)>,
}

struct Kal {
    schluessel: String,
    name: String,
    farbe: String,
    schreibbar: bool,
    /// (uuid, Felder)
    termine: Vec<(String, Terminfelder)>,
}

fn kalender(speicher: &Speicher, fremde: &[Fremd]) -> Result<Vec<Kal>, String> {
    let termine = speicher.termine_alle().map_err(|e| e.to_string())?;
    let mut aus: Vec<Kal> = speicher
        .kalender_alle()
        .map_err(|e| e.to_string())?
        .into_iter()
        .filter(|k| k.papierkorb_at.is_none() && !k.ist_abo())
        .map(|k| Kal {
            termine: termine
                .iter()
                .filter(|t| t.kalender_uuid == k.uuid && t.papierkorb_at.is_none())
                .map(|t| (t.uuid.clone(), t.felder.clone()))
                .collect(),
            schluessel: k.uuid,
            name: k.name,
            farbe: k.farbe,
            schreibbar: true,
        })
        .collect();
    aus.extend(fremde.iter().map(|f| Kal {
        schluessel: f.schluessel.clone(),
        name: f.name.clone(),
        farbe: f.farbe.clone(),
        schreibbar: false,
        termine: f.termine.clone(),
    }));
    Ok(aus)
}

pub(crate) fn xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Eine Ressource: ein Termin -- oder eine Serie samt ihren verschobenen
/// Vorkommen, die ein Programm als eine Datei kennt (dieselbe UID).
struct Res<'a> {
    uuid: &'a str,
    felder: &'a Terminfelder,
    /// (uuid, Felder), nach Vorkommen geordnet.
    vorkommen: Vec<(&'a str, &'a Terminfelder)>,
}

fn ressourcen(k: &Kal) -> Vec<Res<'_>> {
    let serien: std::collections::HashSet<&str> = k
        .termine
        .iter()
        .filter(|(_, f)| !f.ist_vorkommen())
        .map(|(u, _)| u.as_str())
        .collect();
    let mut aus: Vec<Res> = k
        .termine
        .iter()
        // Ein Vorkommen ohne seine Serie (weggeworfen) steht für sich.
        .filter(|(_, f)| !f.ist_vorkommen() || !serien.contains(f.serie.as_str()))
        .map(|(u, f)| Res {
            uuid: u,
            felder: f,
            vorkommen: Vec::new(),
        })
        .collect();
    for (u, f) in k.termine.iter().filter(|(_, f)| f.ist_vorkommen()) {
        if let Some(r) = aus.iter_mut().find(|r| r.uuid == f.serie) {
            r.vorkommen.push((u, f));
        }
    }
    for r in &mut aus {
        r.vorkommen
            .sort_by(|a, b| a.1.vorkommen.cmp(&b.1.vorkommen));
    }
    aus
}

fn etag(r: &Res) -> String {
    etag_aus(r.felder, r.vorkommen.iter().map(|(_, f)| *f))
}

/// Ohne Vorkommen der Abdruck des Termins, sonst einer über alle.
fn etag_aus<'a>(f: &Terminfelder, vorkommen: impl Iterator<Item = &'a Terminfelder>) -> String {
    use sha2::{Digest, Sha256};
    let mut abdruck = f.abdruck();
    let mut h = Sha256::new();
    let mut mehr = false;
    for v in vorkommen {
        if !mehr {
            h.update(abdruck.as_bytes());
            mehr = true;
        }
        h.update(v.abdruck().as_bytes());
    }
    if mehr {
        abdruck = h.finalize().iter().map(|b| format!("{b:02x}")).collect();
    }
    format!("\"{abdruck}\"")
}

fn ctag(k: &Kal) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    let mut liste: Vec<_> = k.termine.iter().collect();
    liste.sort_by(|a, b| a.0.cmp(&b.0));
    for (u, f) in liste {
        h.update(u.as_bytes());
        h.update(f.abdruck().as_bytes());
    }
    h.update(k.name.as_bytes());
    h.update(k.farbe.as_bytes());
    h.finalize()[..12]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

pub(crate) const KOPF: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<d:multistatus xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav" xmlns:cs="http://calendarserver.org/ns/" xmlns:ic="http://apple.com/ns/ical/" xmlns:card="urn:ietf:params:xml:ns:carddav">"#;
pub(crate) const FUSS: &str = "</d:multistatus>";

pub(crate) fn antwort_eintrag(href: &str, props: &str) -> String {
    format!(
        "<d:response><d:href>{}</d:href><d:propstat><d:prop>{props}</d:prop><d:status>HTTP/1.1 200 OK</d:status></d:propstat></d:response>",
        xml(href)
    )
}

fn benutzer_props() -> String {
    format!(
        "<d:resourcetype><d:collection/><d:principal/></d:resourcetype>\
         <d:displayname>openany</d:displayname>\
         <d:current-user-principal><d:href>{WURZEL}</d:href></d:current-user-principal>\
         <d:principal-URL><d:href>{WURZEL}</d:href></d:principal-URL>\
         <c:calendar-home-set><d:href>{ZUHAUSE}</d:href></c:calendar-home-set>\
         <c:calendar-user-address-set><d:href>mailto:openany@localhost</d:href></c:calendar-user-address-set>\
         <card:addressbook-home-set><d:href>{}</d:href></card:addressbook-home-set>",
        crate::carddav::ZUHAUSE
    )
}

pub(crate) fn rechte(schreibbar: bool) -> &'static str {
    if schreibbar {
        "<d:current-user-privilege-set><d:privilege><d:read/></d:privilege><d:privilege><d:write/></d:privilege><d:privilege><d:write-content/></d:privilege><d:privilege><d:bind/></d:privilege><d:privilege><d:unbind/></d:privilege></d:current-user-privilege-set>"
    } else {
        "<d:current-user-privilege-set><d:privilege><d:read/></d:privilege></d:current-user-privilege-set>"
    }
}

fn kalender_props(k: &Kal) -> String {
    let farbe = if k.farbe.starts_with('#') && k.farbe.len() == 7 {
        format!("{}FF", k.farbe.to_uppercase())
    } else {
        "#7C5CB0FF".into()
    };
    format!(
        "<d:resourcetype><d:collection/><c:calendar/></d:resourcetype>\
         <d:displayname>{}</d:displayname>\
         <ic:calendar-color>{farbe}</ic:calendar-color>\
         <c:supported-calendar-component-set><c:comp name=\"VEVENT\"/></c:supported-calendar-component-set>\
         <cs:getctag>{}</cs:getctag>\
         <d:owner><d:href>{WURZEL}</d:href></d:owner>\
         <d:current-user-principal><d:href>{WURZEL}</d:href></d:current-user-principal>\
         <d:supported-report-set><d:supported-report><d:report><c:calendar-query/></d:report></d:supported-report><d:supported-report><d:report><c:calendar-multiget/></d:report></d:supported-report></d:supported-report-set>{}",
        xml(&k.name),
        ctag(k),
        rechte(k.schreibbar)
    )
}

/// Name der Ressource eines Termins (ohne `.ics`) und die `UID` darin.
fn name_von(uuid: &str, namen: &HashMap<String, Davname>) -> (String, String) {
    namen
        .get(uuid)
        .map(|n| (n.name.clone(), n.uid.clone()))
        .unwrap_or_else(|| (uuid.to_string(), uuid.to_string()))
}

fn ics_von(k: &Kal, r: &Res, namen: &HashMap<String, Davname>) -> String {
    let (_, uid) = name_von(r.uuid, namen);
    let liste = std::iter::once((r.uuid.to_string(), r.felder.clone()))
        .chain(
            r.vorkommen
                .iter()
                .map(|(u, f)| (u.to_string(), (*f).clone())),
        )
        .collect();
    let mut liste = openany_sync::ics_ordnen(liste);
    if r.vorkommen.is_empty() {
        // Für sich stehend (auch ein Vorkommen ohne Serie).
        for (_, f) in &mut liste {
            f.serie.clear();
            f.vorkommen.clear();
        }
    }
    for (u, _) in &mut liste {
        u.clone_from(&uid);
    }
    openany_sync::ics_schreiben(&k.name, &liste)
}

fn termin_props(r: &Res, mit_daten: Option<&str>) -> String {
    let daten = mit_daten
        .map(|d| format!("<c:calendar-data>{}</c:calendar-data>", xml(d)))
        .unwrap_or_default();
    format!(
        "<d:resourcetype/><d:getcontenttype>text/calendar; charset=utf-8</d:getcontenttype><d:getetag>{}</d:getetag>{daten}",
        xml(&etag(r))
    )
}

/// Pfadteile unter `/dav/kalender/`: (Kalender, Ressource ohne `.ics`).
fn teile(pfad: &str) -> Option<(String, Option<String>)> {
    let rest = pfad.strip_prefix(ZUHAUSE)?;
    let rest = rest.trim_end_matches('/');
    if rest.is_empty() {
        return None;
    }
    match rest.split_once('/') {
        None => Some((percent_decode(rest), None)),
        Some((k, r)) => Some((
            percent_decode(k),
            Some(percent_decode(r.strip_suffix(".ics").unwrap_or(r))),
        )),
    }
}

pub(crate) fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut aus = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                aus.push(v);
                i += 3;
                continue;
            }
        }
        aus.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&aus).into_owned()
}

pub(crate) fn percent_encode(s: &str) -> String {
    s.bytes()
        .map(|c| match c {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'@' => {
                (c as char).to_string()
            }
            _ => format!("%{c:02X}"),
        })
        .collect()
}

fn href_kalender(k: &Kal) -> String {
    format!("{ZUHAUSE}{}/", percent_encode(&k.schluessel))
}

fn href_termin(k: &Kal, name: &str) -> String {
    format!("{}{}.ics", href_kalender(k), percent_encode(name))
}

/// Die `href`s, nach denen ein `calendar-multiget` fragt.
pub(crate) fn hrefs(koerper: &str) -> Vec<String> {
    let mut aus = Vec::new();
    let mut rest = koerper;
    while let Some(i) = rest.find("href>") {
        rest = &rest[i + 5..];
        if let Some(j) = rest.find('<') {
            let h = rest[..j].trim();
            if !h.is_empty() {
                aus.push(h.to_string());
            }
            rest = &rest[j..];
        }
    }
    aus
}

/// Eine Anfrage beantworten.
pub(crate) fn bearbeiten(
    speicher: &Speicher,
    fremde: &[Fremd],
    a: &Anfrage,
) -> Result<Antwort, String> {
    let methode = a.methode.to_ascii_uppercase();
    if methode == "OPTIONS" {
        let mut x = Antwort::leer(200);
        x.kopf
            .push(("DAV", "1, 3, calendar-access, addressbook".into()));
        x.kopf.push((
            "Allow",
            "OPTIONS, GET, HEAD, PUT, DELETE, PROPFIND, REPORT".into(),
        ));
        return Ok(x);
    }

    // Die Wurzel, `.well-known` und der Benutzer: dieselbe Auskunft.
    let pfad = a.pfad.split('?').next().unwrap_or_default();
    if pfad == "/"
        || pfad.starts_with("/.well-known/caldav")
        || pfad.starts_with("/.well-known/carddav")
        || pfad == WURZEL
        || pfad == "/dav"
    {
        if methode != "PROPFIND" {
            return Ok(Antwort::leer(405));
        }
        return Ok(Antwort::xml(format!(
            "{KOPF}{}{FUSS}",
            antwort_eintrag(WURZEL, &benutzer_props())
        )));
    }

    // Das Adressbuch (CardDAV) hat seinen eigenen Teil.
    if pfad.starts_with(crate::carddav::ZUHAUSE.trim_end_matches('/')) {
        return crate::carddav::bearbeiten(speicher, a);
    }

    let alle = kalender(speicher, fremde)?;
    let namen = speicher.davnamen(ART).map_err(|e| e.to_string())?;

    if pfad == ZUHAUSE || pfad == ZUHAUSE.trim_end_matches('/') {
        if methode != "PROPFIND" {
            return Ok(Antwort::leer(405));
        }
        let mut k = antwort_eintrag(
            ZUHAUSE,
            &format!(
                "<d:resourcetype><d:collection/></d:resourcetype><d:displayname>openany</d:displayname>{}",
                rechte(false)
            ),
        );
        if a.tiefe >= 1 {
            for kal in &alle {
                k.push_str(&antwort_eintrag(&href_kalender(kal), &kalender_props(kal)));
            }
        }
        return Ok(Antwort::xml(format!("{KOPF}{k}{FUSS}")));
    }

    let Some((kal_schluessel, ressource)) = teile(pfad) else {
        return Ok(Antwort::leer(404));
    };
    let Some(kal) = alle.iter().find(|k| k.schluessel == kal_schluessel) else {
        return Ok(Antwort::leer(404));
    };
    let res = ressourcen(kal);
    // uuid eines Termins nach seinem Namen.
    let uuid_von = |name: &str| -> Option<String> {
        if let Ok(Some(u)) = speicher.davname_zu_uuid(ART, name) {
            return Some(u);
        }
        kal.termine
            .iter()
            .find(|(u, _)| u == name && !namen.contains_key(u))
            .map(|(u, _)| u.clone())
    };

    match (methode.as_str(), ressource) {
        ("PROPFIND", None) => {
            let mut k = antwort_eintrag(&href_kalender(kal), &kalender_props(kal));
            if a.tiefe >= 1 {
                for r in &res {
                    let (name, _) = name_von(r.uuid, &namen);
                    k.push_str(&antwort_eintrag(
                        &href_termin(kal, &name),
                        &termin_props(r, None),
                    ));
                }
            }
            Ok(Antwort::xml(format!("{KOPF}{k}{FUSS}")))
        }
        ("REPORT", None) => {
            // `calendar-multiget` nennt die gewünschten Termine; eine
            // `calendar-query` bekommt alle -- filtern können die Programme.
            let gewuenscht: Option<Vec<String>> = a
                .koerper
                .contains("calendar-multiget")
                .then(|| hrefs(&a.koerper));
            let mut k = String::new();
            for r in &res {
                let (name, _) = name_von(r.uuid, &namen);
                let href = href_termin(kal, &name);
                if let Some(g) = &gewuenscht {
                    if !g.iter().any(|h| percent_decode(h) == percent_decode(&href)) {
                        continue;
                    }
                }
                let ics = ics_von(kal, r, &namen);
                k.push_str(&antwort_eintrag(&href, &termin_props(r, Some(&ics))));
            }
            Ok(Antwort::xml(format!("{KOPF}{k}{FUSS}")))
        }
        ("PROPFIND", Some(name)) => {
            let Some(u) = uuid_von(&name) else {
                return Ok(Antwort::leer(404));
            };
            let Some(r) = res.iter().find(|r| r.uuid == u) else {
                return Ok(Antwort::leer(404));
            };
            Ok(Antwort::xml(format!(
                "{KOPF}{}{FUSS}",
                antwort_eintrag(&href_termin(kal, &name), &termin_props(r, None))
            )))
        }
        ("GET" | "HEAD", Some(name)) => {
            let Some(u) = uuid_von(&name) else {
                return Ok(Antwort::leer(404));
            };
            let Some(r) = res.iter().find(|r| r.uuid == u) else {
                return Ok(Antwort::leer(404));
            };
            Ok(Antwort {
                status: 200,
                kopf: vec![
                    ("Content-Type", "text/calendar; charset=utf-8".into()),
                    ("ETag", etag(r)),
                ],
                koerper: if methode == "HEAD" {
                    String::new()
                } else {
                    ics_von(kal, r, &namen)
                },
            })
        }
        ("PUT", Some(name)) => {
            if !kal.schreibbar {
                return Ok(Antwort::leer(403));
            }
            let vorhanden = uuid_von(&name).and_then(|u| res.iter().find(|r| r.uuid == u));
            if a.if_none_match.as_deref() == Some("*") && vorhanden.is_some() {
                return Ok(Antwort::leer(412));
            }
            if let Some(m) = a.if_match.as_deref() {
                match vorhanden {
                    Some(r) if m == "*" || m == etag(r) => {}
                    _ => return Ok(Antwort::leer(412)),
                }
            }
            let gelesen = openany_sync::ics_lesen(&a.koerper);
            // Die Serie (ohne RECURRENCE-ID) und ihre verschobenen Vorkommen.
            let (mut serie, mut verschoben): (Vec<_>, Vec<_>) = gelesen
                .termine
                .into_iter()
                .partition(|t| t.felder.vorkommen.is_empty());
            let neu = if serie.is_empty() {
                // Nur ein Vorkommen, ohne Serie: als eigener Termin.
                if verschoben.is_empty() {
                    return Ok(Antwort::leer(400));
                }
                let mut t = verschoben.remove(0);
                t.felder.vorkommen.clear();
                t
            } else {
                serie.remove(0)
            };
            let neu_angelegt = vorhanden.is_none();
            let uuid = vorhanden
                .map(|r| r.uuid.to_string())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
            let schreiben = |uuid: &str, felder: &Terminfelder| {
                speicher
                    .termin_schreiben(
                        &Termin {
                            uuid: uuid.to_string(),
                            kalender_uuid: kal.schluessel.clone(),
                            felder: felder.clone(),
                            papierkorb_at: None,
                            geaendert_at: String::new(),
                        },
                        Protokoll::Merken,
                    )
                    .map_err(|e| e.to_string())
            };
            // Was ICS nicht trägt, bleibt stehen (wie in der App).
            let mut felder = vorhanden.map(|r| r.felder.clone()).unwrap_or_default();
            let n = neu.felder;
            felder.titel = n.titel;
            felder.beschreibung = n.beschreibung;
            felder.ort = n.ort;
            felder.beginn = n.beginn;
            felder.ende = n.ende;
            felder.ganztags = n.ganztags;
            felder.rrule = n.rrule;
            felder.rrule_bis = n.rrule_bis;
            felder.regel = n.regel;
            felder.exdates = n.exdates;
            felder.erinnerung = n.erinnerung;
            felder.serie.clear();
            felder.vorkommen.clear();
            schreiben(&uuid, &felder)?;
            // Die Vorkommen: vorhandene nach ihrem ursprünglichen Beginn
            // wiederfinden, fehlende wegwerfen.
            let bisher: Vec<(&str, &Terminfelder)> =
                vorhanden.map(|r| r.vorkommen.clone()).unwrap_or_default();
            let mut neue_vorkommen = Vec::new();
            for v in verschoben {
                let alt = bisher
                    .iter()
                    .find(|(_, f)| f.vorkommen == v.felder.vorkommen);
                let v_uuid = alt
                    .map(|(u, _)| u.to_string())
                    .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
                let mut f = alt.map(|(_, f)| (*f).clone()).unwrap_or_default();
                let n = v.felder;
                f.titel = n.titel;
                f.beschreibung = n.beschreibung;
                f.ort = n.ort;
                f.beginn = n.beginn;
                f.ende = n.ende;
                f.ganztags = n.ganztags;
                f.erinnerung = n.erinnerung;
                f.rrule = "NONE".into();
                f.rrule_bis.clear();
                f.regel.clear();
                f.exdates.clear();
                f.serie.clone_from(&uuid);
                f.vorkommen = n.vorkommen;
                schreiben(&v_uuid, &f)?;
                neue_vorkommen.push(f);
            }
            for (u, f) in &bisher {
                if !neue_vorkommen.iter().any(|n| n.vorkommen == f.vorkommen) {
                    speicher
                        .papierkorb(&openany_client::Art::Termin, u, Protokoll::Merken)
                        .map_err(|e| e.to_string())?;
                }
            }
            neue_vorkommen.sort_by(|a, b| a.vorkommen.cmp(&b.vorkommen));
            // Unter dem Namen und der UID des Programms wiederfinden.
            if name != uuid || (!neu.uid.is_empty() && neu.uid != uuid) {
                speicher
                    .davname_merken(
                        ART,
                        &uuid,
                        &Davname {
                            name: name.clone(),
                            uid: if neu.uid.is_empty() {
                                name.clone()
                            } else {
                                neu.uid
                            },
                        },
                    )
                    .map_err(|e| e.to_string())?;
            }
            Ok(Antwort {
                status: if neu_angelegt { 201 } else { 204 },
                kopf: vec![("ETag", etag_aus(&felder, neue_vorkommen.iter()))],
                koerper: String::new(),
            })
        }
        ("DELETE", Some(name)) => {
            if !kal.schreibbar {
                return Ok(Antwort::leer(403));
            }
            let Some(u) = uuid_von(&name) else {
                return Ok(Antwort::leer(404));
            };
            if let Some(m) = a.if_match.as_deref() {
                if let Some(r) = res.iter().find(|r| r.uuid == u) {
                    if m != "*" && m != etag(r) {
                        return Ok(Antwort::leer(412));
                    }
                }
            }
            crate::serie_wegwerfen(speicher, &u)?;
            Ok(Antwort::leer(204))
        }
        _ => Ok(Antwort::leer(405)),
    }
}

/// Der feste Anschluss -- damit eine eingerichtete Verbindung nach einem
/// Neustart weiter stimmt. Ist er belegt, die nächsten.
const ANSCHLUSS: u16 = 5232;

#[derive(serde::Serialize)]
pub struct Dienstlage {
    an: bool,
    /// Gibt es den Dienst hier (Desktop)?
    moeglich: bool,
    adresse: String,
    benutzer: String,
    kennwort: String,
    /// Läuft er gerade? (Aus, wenn der Anschluss nicht frei war.)
    laeuft: bool,
}

#[tauri::command]
pub async fn kalenderdienst_stand(
    zustand: tauri::State<'_, std::sync::Arc<crate::Zustand>>,
) -> Result<Dienstlage, String> {
    let d = zustand.einstellungen.lock().await.kalenderdienst.clone();
    Ok(Dienstlage {
        an: d.is_some(),
        moeglich: cfg!(not(any(target_os = "android", target_os = "ios"))),
        adresse: d
            .as_ref()
            .map(|d| format!("http://127.0.0.1:{}/", d.port))
            .unwrap_or_default(),
        benutzer: d.as_ref().map(|d| d.benutzer.clone()).unwrap_or_default(),
        kennwort: d.map(|d| d.kennwort).unwrap_or_default(),
        laeuft: dienst::laeuft(),
    })
}

fn neues_kennwort() -> String {
    uuid::Uuid::new_v4().simple().to_string()[..20].to_string()
}

/// Ein- oder ausschalten.
#[tauri::command]
pub async fn kalenderdienst_setzen(
    zustand: tauri::State<'_, std::sync::Arc<crate::Zustand>>,
    an: bool,
) -> Result<(), String> {
    if cfg!(any(target_os = "android", target_os = "ios")) {
        return Err("Only available in the desktop app.".into());
    }
    {
        let mut e = zustand.einstellungen.lock().await;
        e.kalenderdienst = an.then(|| {
            e.kalenderdienst
                .clone()
                .unwrap_or(crate::einstellungen::Kalenderdienst {
                    benutzer: "openany".into(),
                    kennwort: neues_kennwort(),
                    port: ANSCHLUSS,
                })
        });
        e.schreiben(&zustand.einstellungspfad())
            .map_err(|e| e.to_string())?;
    }
    if an {
        dienst::starten(zustand.inner()).await
    } else {
        dienst::anhalten();
        Ok(())
    }
}

/// Ein neues Kennwort -- eingerichtete Programme müssen es neu bekommen.
#[tauri::command]
pub async fn kalenderdienst_neues_kennwort(
    zustand: tauri::State<'_, std::sync::Arc<crate::Zustand>>,
) -> Result<(), String> {
    let mut e = zustand.einstellungen.lock().await;
    if let Some(d) = e.kalenderdienst.as_mut() {
        d.kennwort = neues_kennwort();
    }
    e.schreiben(&zustand.einstellungspfad())
        .map_err(|e| e.to_string())
}

/// Beim Start: Ist er eingeschaltet, gleich öffnen.
pub(crate) fn beim_start(zustand: &std::sync::Arc<crate::Zustand>) {
    let z = zustand.clone();
    tauri::async_runtime::spawn(async move {
        if z.einstellungen.lock().await.kalenderdienst.is_some() {
            if let Err(e) = dienst::starten(&z).await {
                eprintln!("Calendar service: {e}");
            }
        }
    });
}

mod dienst {
    use super::{bearbeiten, Anfrage, Fremd, ANSCHLUSS};
    use crate::Zustand;
    use base64::Engine;
    use bytes::Bytes;
    use http_body_util::{BodyExt, Full, Limited};
    use hyper::{Request, Response};
    use hyper_util::rt::TokioIo;
    use std::sync::{Arc, Mutex};

    static LAUF: Mutex<Option<tauri::async_runtime::JoinHandle<()>>> = Mutex::new(None);

    pub(super) fn laeuft() -> bool {
        LAUF.lock().unwrap_or_else(|e| e.into_inner()).is_some()
    }

    pub(super) fn anhalten() {
        if let Some(h) = LAUF.lock().unwrap_or_else(|e| e.into_inner()).take() {
            h.abort();
        }
    }

    /// Öffnen -- nur auf 127.0.0.1. Ein belegter Anschluss: die nächsten,
    /// und der gefundene wird gemerkt.
    pub(super) async fn starten(zustand: &Arc<Zustand>) -> Result<(), String> {
        anhalten();
        let gewuenscht = zustand
            .einstellungen
            .lock()
            .await
            .kalenderdienst
            .as_ref()
            .map_or(ANSCHLUSS, |d| d.port);
        let mut lauscher = None;
        for p in gewuenscht..gewuenscht.saturating_add(10) {
            if let Ok(l) = tokio::net::TcpListener::bind(("127.0.0.1", p)).await {
                lauscher = Some((l, p));
                break;
            }
        }
        let Some((lauscher, port)) = lauscher else {
            return Err("No free port for the calendar service.".into());
        };
        if port != gewuenscht {
            let mut e = zustand.einstellungen.lock().await;
            if let Some(d) = e.kalenderdienst.as_mut() {
                d.port = port;
            }
            let _ = e.schreiben(&zustand.einstellungspfad());
        }
        let z = zustand.clone();
        let h = tauri::async_runtime::spawn(async move {
            loop {
                let Ok((strom, _)) = lauscher.accept().await else {
                    continue;
                };
                let z = z.clone();
                tokio::spawn(async move {
                    let dienst = hyper::service::service_fn(move |a| {
                        let z = z.clone();
                        async move { Ok::<_, std::convert::Infallible>(beantworten(&z, a).await) }
                    });
                    let _ = hyper::server::conn::http1::Builder::new()
                        .serve_connection(TokioIo::new(strom), dienst)
                        .await;
                });
            }
        });
        *LAUF.lock().unwrap_or_else(|e| e.into_inner()) = Some(h);
        Ok(())
    }

    fn antwort(
        status: u16,
        kopf: Vec<(&'static str, String)>,
        koerper: String,
    ) -> Response<Full<Bytes>> {
        let mut r = Response::builder().status(status);
        for (n, w) in kopf {
            r = r.header(n, w);
        }
        r.body(Full::new(Bytes::from(koerper)))
            .unwrap_or_else(|_| Response::new(Full::new(Bytes::new())))
    }

    /// Basic-Anmeldung gegen die gemerkten Zugangsdaten.
    fn angemeldet(kopf: Option<&str>, benutzer: &str, kennwort: &str) -> bool {
        let Some(roh) = kopf.and_then(|k| k.strip_prefix("Basic ")) else {
            return false;
        };
        let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(roh.trim()) else {
            return false;
        };
        let erwartet = format!("{benutzer}:{kennwort}");
        // Gleich lange Vergleiche, damit die Antwortzeit nichts verrät.
        bytes.len() == erwartet.len()
            && bytes
                .iter()
                .zip(erwartet.as_bytes())
                .fold(0u8, |acc, (a, b)| acc | (a ^ b))
                == 0
    }

    async fn beantworten(
        z: &Arc<Zustand>,
        a: Request<hyper::body::Incoming>,
    ) -> Response<Full<Bytes>> {
        let Some(d) = z.einstellungen.lock().await.kalenderdienst.clone() else {
            return antwort(503, vec![], String::new());
        };
        let kopf = |n: &str| {
            a.headers()
                .get(n)
                .and_then(|v| v.to_str().ok())
                .map(str::to_string)
        };
        if !angemeldet(kopf("authorization").as_deref(), &d.benutzer, &d.kennwort) {
            return antwort(
                401,
                vec![("WWW-Authenticate", "Basic realm=\"openany\"".into())],
                String::new(),
            );
        }
        let methode = a.method().as_str().to_string();
        let pfad = a.uri().path().to_string();
        let tiefe = match kopf("depth").as_deref() {
            Some("0") => 0,
            _ => 1,
        };
        let if_match = kopf("if-match");
        let if_none_match = kopf("if-none-match");
        // Höchstens 10 MB -- ein Termin ist klein.
        let koerper = match Limited::new(a.into_body(), 10 * 1024 * 1024)
            .collect()
            .await
        {
            Ok(k) => String::from_utf8_lossy(&k.to_bytes()).into_owned(),
            Err(_) => return antwort(413, vec![], String::new()),
        };
        let fremde: Vec<Fremd> = crate::projektkalender::fremde(z)
            .await
            .into_iter()
            .filter(|(_, g)| !g.eigen)
            .map(|(projekt, g)| Fremd {
                schluessel: format!("hub-{}", g.uuid.replace(':', "-")),
                name: format!("{} ({projekt})", g.name),
                farbe: g.color.clone(),
                termine: g
                    .termine
                    .iter()
                    .map(|(u, f)| (u.clone(), openany_client::Terminfelder::aus_eintrag(f)))
                    .collect(),
            })
            .collect();
        let anfrage = Anfrage {
            methode,
            pfad,
            tiefe,
            koerper,
            if_match,
            if_none_match,
        };
        let ergebnis = {
            let speicher = z.speicher.lock().await;
            bearbeiten(&speicher, &fremde, &anfrage)
        };
        match ergebnis {
            Ok(x) => antwort(x.status, x.kopf, x.koerper),
            Err(e) => antwort(500, vec![], e),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::angemeldet;

        #[test]
        fn nur_mit_den_richtigen_zugangsdaten() {
            // "openany:geheim"
            assert!(angemeldet(
                Some("Basic b3BlbmFueTpnZWhlaW0="),
                "openany",
                "geheim"
            ));
            assert!(!angemeldet(
                Some("Basic b3BlbmFueTpmYWxzY2g="),
                "openany",
                "geheim"
            ));
            assert!(!angemeldet(None, "openany", "geheim"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use openany_store::Kalender;

    fn anfrage(m: &str, p: &str, tiefe: u8, koerper: &str) -> Anfrage {
        Anfrage {
            methode: m.into(),
            pfad: p.into(),
            tiefe,
            koerper: koerper.into(),
            if_match: None,
            if_none_match: None,
        }
    }

    fn speicher() -> Speicher {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        s.kalender_schreiben(
            &Kalender {
                uuid: "k1".into(),
                name: "Mein Kalender".into(),
                farbe: "#3366cc".into(),
                ..Default::default()
            },
            Protokoll::Still,
        )
        .unwrap();
        s.termin_schreiben(
            &Termin {
                uuid: "t1".into(),
                kalender_uuid: "k1".into(),
                felder: Terminfelder {
                    titel: "Zahnarzt".into(),
                    beginn: "2026-11-20T09:00:00".into(),
                    ende: "2026-11-20T09:30:00".into(),
                    erinnerung: "15".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            Protokoll::Still,
        )
        .unwrap();
        s
    }

    #[test]
    fn eine_serie_mit_verschobenem_vorkommen_ist_eine_datei() {
        let s = speicher();
        let ics = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:serie@beispiel.test\r\nSUMMARY:Sport\r\n\
                   DTSTART:20261103T180000\r\nDTEND:20261103T190000\r\n\
                   RRULE:FREQ=WEEKLY;INTERVAL=2;BYDAY=TU,TH;COUNT=6\r\nEND:VEVENT\r\n\
                   BEGIN:VEVENT\r\nUID:serie@beispiel.test\r\nSUMMARY:Sport, später\r\n\
                   RECURRENCE-ID:20261105T180000\r\nDTSTART:20261105T200000\r\n\
                   DTEND:20261105T210000\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
        let w = bearbeiten(
            &s,
            &[],
            &anfrage("PUT", "/dav/kalender/k1/serie.ics", 0, ics),
        )
        .unwrap();
        assert_eq!(w.status, 201);
        let etag = w.kopf.iter().find(|(n, _)| *n == "ETag").unwrap().1.clone();

        // In openany: zwei Termine, das Vorkommen verweist auf die Serie.
        let termine = s.termine_im_kalender("k1").unwrap();
        let serie = termine.iter().find(|t| t.felder.titel == "Sport").unwrap();
        assert_eq!(serie.felder.regel, "INTERVAL=2;COUNT=6;BYDAY=TU,TH");
        let v = termine.iter().find(|t| t.felder.ist_vorkommen()).unwrap();
        assert_eq!(v.felder.serie, serie.uuid);
        assert_eq!(v.felder.beginn, "2026-11-05T20:00:00");

        // Nach außen: eine Datei mit beiden, unter derselben UID.
        let k = bearbeiten(&s, &[], &anfrage("PROPFIND", "/dav/kalender/k1/", 1, "")).unwrap();
        assert_eq!(k.koerper.matches("<d:response>").count(), 3);
        let g = bearbeiten(
            &s,
            &[],
            &anfrage("GET", "/dav/kalender/k1/serie.ics", 0, ""),
        )
        .unwrap();
        assert_eq!(g.koerper.matches("UID:serie@beispiel.test").count(), 2);
        assert!(g.koerper.contains("RECURRENCE-ID:20261105T180000"));
        assert!(g
            .koerper
            .contains("RRULE:FREQ=WEEKLY;INTERVAL=2;COUNT=6;BYDAY=TU,TH"));
        assert_eq!(g.kopf.iter().find(|(n, _)| *n == "ETag").unwrap().1, etag);

        // Das Vorkommen wieder zurückgenommen: Es geht in den Papierkorb.
        let nur_serie = ics
            .split("BEGIN:VEVENT")
            .take(2)
            .collect::<Vec<_>>()
            .join("BEGIN:VEVENT")
            + "END:VCALENDAR\r\n";
        let mut a = anfrage("PUT", "/dav/kalender/k1/serie.ics", 0, &nur_serie);
        a.if_match = Some(etag);
        assert_eq!(bearbeiten(&s, &[], &a).unwrap().status, 204);
        assert!(s
            .termine_im_kalender("k1")
            .unwrap()
            .iter()
            .all(|t| !t.felder.ist_vorkommen()));

        // Die Serie gelöscht: alles fort.
        let d = bearbeiten(
            &s,
            &[],
            &anfrage("DELETE", "/dav/kalender/k1/serie.ics", 0, ""),
        )
        .unwrap();
        assert_eq!(d.status, 204);
        assert!(s
            .termine_im_kalender("k1")
            .unwrap()
            .iter()
            .all(|t| t.felder.titel != "Sport"));
    }

    #[test]
    fn entdecken_bis_zum_termin() {
        let s = speicher();
        let w = bearbeiten(&s, &[], &anfrage("PROPFIND", "/.well-known/caldav", 0, "")).unwrap();
        assert!(w
            .koerper
            .contains("<c:calendar-home-set><d:href>/dav/kalender/</d:href>"));

        let z = bearbeiten(&s, &[], &anfrage("PROPFIND", "/dav/kalender/", 1, "")).unwrap();
        assert!(z.koerper.contains("<d:href>/dav/kalender/k1/</d:href>"));
        assert!(z
            .koerper
            .contains("<d:displayname>Mein Kalender</d:displayname>"));
        assert!(z.koerper.contains("#3366CCFF"));

        let k = bearbeiten(&s, &[], &anfrage("PROPFIND", "/dav/kalender/k1/", 1, "")).unwrap();
        assert!(k
            .koerper
            .contains("<d:href>/dav/kalender/k1/t1.ics</d:href>"));

        let g = bearbeiten(&s, &[], &anfrage("GET", "/dav/kalender/k1/t1.ics", 0, "")).unwrap();
        assert_eq!(g.status, 200);
        assert!(g.koerper.contains("SUMMARY:Zahnarzt"));
        assert!(g.koerper.contains("UID:t1"));
        assert!(g.koerper.contains("TRIGGER:-PT15M"));

        let r = bearbeiten(
            &s,
            &[],
            &anfrage("REPORT", "/dav/kalender/k1/", 1, "<c:calendar-multiget><d:href>/dav/kalender/k1/t1.ics</d:href></c:calendar-multiget>"),
        )
        .unwrap();
        assert!(r.koerper.contains("<c:calendar-data>"));
    }

    #[test]
    fn ein_programm_legt_an_aendert_und_loescht() {
        let s = speicher();
        let ics = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:abc@thunderbird\r\nSUMMARY:Neu aus Thunderbird\r\nDTSTART:20261121T100000\r\nDTEND:20261121T110000\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
        let mut a = anfrage("PUT", "/dav/kalender/k1/abc.ics", 0, ics);
        a.if_none_match = Some("*".into());
        let p = bearbeiten(&s, &[], &a).unwrap();
        assert_eq!(p.status, 201);
        let etag1 = p.kopf.iter().find(|(n, _)| *n == "ETag").unwrap().1.clone();

        // Unter seinem Namen und seiner UID wieder da.
        let k = bearbeiten(&s, &[], &anfrage("PROPFIND", "/dav/kalender/k1/", 1, "")).unwrap();
        assert!(k.koerper.contains("/dav/kalender/k1/abc.ics"));
        let g = bearbeiten(&s, &[], &anfrage("GET", "/dav/kalender/k1/abc.ics", 0, "")).unwrap();
        assert!(g.koerper.contains("UID:abc@thunderbird"));

        // Ändern nur mit dem richtigen ETag.
        let neu = ics.replace("Neu aus Thunderbird", "Geändert");
        let mut falsch = anfrage("PUT", "/dav/kalender/k1/abc.ics", 0, &neu);
        falsch.if_match = Some("\"alt\"".into());
        assert_eq!(bearbeiten(&s, &[], &falsch).unwrap().status, 412);
        let mut richtig = anfrage("PUT", "/dav/kalender/k1/abc.ics", 0, &neu);
        richtig.if_match = Some(etag1);
        assert_eq!(bearbeiten(&s, &[], &richtig).unwrap().status, 204);
        assert_eq!(
            s.termine_alle()
                .unwrap()
                .iter()
                .filter(|t| t.felder.titel == "Geändert")
                .count(),
            1
        );

        // Löschen: in den Papierkorb.
        let d = bearbeiten(
            &s,
            &[],
            &anfrage("DELETE", "/dav/kalender/k1/abc.ics", 0, ""),
        )
        .unwrap();
        assert_eq!(d.status, 204);
        let uuid = s.davname_zu_uuid(ART, "abc").unwrap().unwrap();
        assert!(s.termin(&uuid).unwrap().unwrap().papierkorb_at.is_some());
        let k = bearbeiten(&s, &[], &anfrage("PROPFIND", "/dav/kalender/k1/", 1, "")).unwrap();
        assert!(!k.koerper.contains("abc.ics"));
    }

    #[test]
    fn geteilte_kalender_sind_nur_lesbar() {
        let s = speicher();
        let fremd = Fremd {
            schluessel: "hub-1".into(),
            name: "Garten (Hub)".into(),
            farbe: "#00aa00".into(),
            termine: vec![],
        };
        let mut a = anfrage(
            "PUT",
            "/dav/kalender/hub-1/x.ics",
            0,
            "BEGIN:VCALENDAR\r\nEND:VCALENDAR",
        );
        a.if_none_match = Some("*".into());
        assert_eq!(bearbeiten(&s, &[fremd], &a).unwrap().status, 403);
    }
}
