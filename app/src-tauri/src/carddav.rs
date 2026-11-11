//! Das Adressbuch für andere Programme auf diesem Rechner -- CardDAV
//! (docs/plan-kalender-kontakte-im-system.md, K5). Derselbe Dienst wie der
//! Kalender (`caldav.rs`): selber Anschluss, selbe Anmeldung, nur Desktop.
//!
//! Ein Adressbuch „openany“ unter `/dav/adressbuch/openany/`, je Kontakt eine
//! vCard. **Die vCard-Regeln sind die von `@oberflaeche/shared/vcard`** (das
//! Ein- und Ausgeben als Datei): 3.0 schreiben, 2.1/3.0/4.0 lesen; Matrix,
//! Meshtastic und der openany-Name als `IMPP` mit eigenem Schema; was
//! openany nicht kennt, reist durchgereicht als `vcard:<feld>`.
//!
//! Anders als das Einlesen einer Datei **ersetzt** ein PUT den Kontakt: Das
//! Programm schickt die ganze Karte, wie es sie von hier bekam.

use crate::caldav::{
    antwort_eintrag, hrefs, percent_decode, percent_encode, rechte, xml, Anfrage, Antwort, FUSS,
    KOPF,
};
use base64::Engine;
use openany_store::{Davname, Kontakt, Protokoll, Speicher, Weg};
use std::collections::HashMap;

const ART: &str = "kontakt";
pub(crate) const ZUHAUSE: &str = "/dav/adressbuch/";
const BUCH: &str = "/dav/adressbuch/openany/";
const VCARD: &str = "vcard:";
/// Ein Foto über dieser Größe nimmt der Dienst nicht an.
const FOTO_HOECHSTENS: usize = 2 * 1024 * 1024;

// --- vCard -------------------------------------------------------------------

/// Art des Wegs → Feld (wie `FELDER` in vcard.js).
const FELDER: [(&str, &str); 6] = [
    ("phone", "TEL"),
    ("email", "EMAIL"),
    ("address", "ADR"),
    ("company", "ORG"),
    ("birthday", "BDAY"),
    ("other", "NOTE"),
];
const IMPP: [&str; 3] = ["matrix", "meshtastic", "openany"];
const NUR_LESEN: [(&str, &str); 4] = [
    ("X-OPENANY-MATRIX", "matrix"),
    ("X-OPENANY-MESHTASTIC", "meshtastic"),
    ("X-OPENANY-NAME", "openany"),
    ("X-OPENANY-OTHER", "other"),
];
const NICHT_DURCHREICHEN: [&str; 15] = [
    "BEGIN",
    "END",
    "VERSION",
    "PRODID",
    "REV",
    "SOURCE",
    "KIND",
    "XML",
    "CLIENTPIDMAP",
    "FN",
    "N",
    "UID",
    "PHOTO",
    "LOGO",
    "SOUND",
];

fn maskieren(w: &str) -> String {
    w.replace('\\', "\\\\")
        .replace("\r\n", "\\n")
        .replace(['\n', '\r'], "\\n")
        .replace(';', "\\;")
        .replace(',', "\\,")
}

fn zeilenfest(w: &str) -> String {
    w.replace("\r\n", "\\n").replace(['\n', '\r'], "\\n")
}

fn typ(label: Option<&str>) -> String {
    let sauber: String = label
        .unwrap_or_default()
        .chars()
        .map(|c| if ":;,\r\n".contains(c) { ' ' } else { c })
        .collect();
    let sauber = sauber.trim();
    if sauber.is_empty() {
        String::new()
    } else {
        format!(";TYPE={sauber}")
    }
}

fn uri(art: &str, wert: &str) -> String {
    if art == "matrix" {
        if let Some((name, server)) = wert.strip_prefix('@').and_then(|r| r.split_once(':')) {
            return format!("matrix:u/{name}:{server}");
        }
    }
    format!("{art}:{wert}")
}

fn falten(zeile: &str) -> String {
    if zeile.len() <= 75 {
        return zeile.to_string();
    }
    let mut aus = String::new();
    let mut laenge = 0;
    let mut erste = true;
    for c in zeile.chars() {
        let grenze = if erste { 75 } else { 74 };
        if laenge + c.len_utf8() > grenze {
            aus.push_str("\r\n ");
            laenge = 0;
            erste = false;
        }
        aus.push(c);
        laenge += c.len_utf8();
    }
    aus
}

fn weg_zeile(w: &Weg) -> String {
    let marke = typ(w.beschriftung.as_deref());
    if let Some(feld) = w.art.strip_prefix(VCARD) {
        return format!("{}{marke}:{}", feld.to_uppercase(), zeilenfest(&w.wert));
    }
    if IMPP.contains(&w.art.as_str()) {
        let mut dienst = w.art.clone();
        dienst[..1].make_ascii_uppercase();
        return format!(
            "IMPP{marke};X-SERVICE-TYPE={dienst}:{}",
            maskieren(&uri(&w.art, &w.wert))
        );
    }
    let feld = FELDER
        .iter()
        .find(|(a, _)| *a == w.art)
        .map_or("NOTE", |(_, f)| *f);
    let wert = if feld == "ADR" {
        format!(";;{};;;;", maskieren(&w.wert))
    } else {
        maskieren(&w.wert)
    };
    format!("{feld}{marke}:{wert}")
}

/// Ein Kontakt als vCard 3.0. `uid`: die UID, unter der das Programm ihn
/// kennt; `foto`: die Bytes, falls es eines gibt.
pub(crate) fn schreiben(k: &Kontakt, uid: &str, foto: Option<&[u8]>) -> String {
    let mut z = vec!["BEGIN:VCARD".to_string(), "VERSION:3.0".to_string()];
    z.push("PRODID:-//openany//Adressbuch//DE".into());
    z.push(format!("FN:{}", maskieren(&k.anzeigename)));
    z.push(format!("N:{};;;;", maskieren(&k.anzeigename)));
    if uid == k.uuid {
        z.push(format!("UID:urn:uuid:{uid}"));
    } else {
        z.push(format!("UID:{uid}"));
    }
    if let Some(b) = foto {
        z.push(format!(
            "PHOTO;ENCODING=b;TYPE=JPEG:{}",
            base64::engine::general_purpose::STANDARD.encode(b)
        ));
    }
    for w in &k.wege {
        z.push(weg_zeile(w));
    }
    z.push("END:VCARD".into());
    z.iter().map(|l| falten(l) + "\r\n").collect()
}

fn entmaskieren(w: &str) -> String {
    let mut aus = String::new();
    let mut it = w.chars().peekable();
    while let Some(c) = it.next() {
        if c == '\\' {
            match it.peek() {
                Some('n' | 'N') => {
                    aus.push('\n');
                    it.next();
                }
                Some(&z @ (';' | ',' | '\\')) => {
                    aus.push(z);
                    it.next();
                }
                _ => aus.push(c),
            }
        } else {
            aus.push(c);
        }
    }
    aus
}

fn qp(w: &str) -> String {
    let b = w.as_bytes();
    let mut aus = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'=' && i + 2 < b.len() {
            if let Ok(v) = u8::from_str_radix(&w[i + 1..i + 3], 16) {
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

fn entfalten(inhalt: &str) -> Vec<String> {
    let mut zeilen: Vec<String> = Vec::new();
    let text = inhalt.trim_start_matches('\u{feff}');
    for zeile in text.split("\r\n").flat_map(|z| z.split(['\n', '\r'])) {
        if let Some(letzte) = zeilen.last_mut() {
            if letzte.ends_with('=') && letzte.to_ascii_uppercase().contains("QUOTED-PRINTABLE") {
                letzte.pop();
                letzte.push_str(zeile);
                continue;
            }
            if zeile.starts_with(' ') || zeile.starts_with('\t') {
                letzte.push_str(&zeile[1..]);
                continue;
            }
        }
        zeilen.push(zeile.to_string());
    }
    zeilen
}

fn parameter(teile: &[&str]) -> HashMap<String, String> {
    let mut p = HashMap::new();
    for t in teile {
        match t.split_once('=') {
            None => {
                p.insert("TYPE".into(), t.trim().to_uppercase());
            }
            Some((n, w)) => {
                p.insert(
                    n.trim().to_uppercase(),
                    w.trim().trim_matches('"').to_string(),
                );
            }
        }
    }
    p
}

fn beschriftung(p: &HashMap<String, String>) -> Option<String> {
    let m = p
        .get("X-ABLABEL")
        .or_else(|| p.get("TYPE"))
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    if m.is_empty() || ["PREF", "INTERNET", "VOICE"].contains(&m.to_uppercase().as_str()) {
        return None;
    }
    Some(m.chars().take(255).collect())
}

fn dienst(wert: &str, p: &HashMap<String, String>) -> String {
    let schema = wert
        .trim()
        .split(':')
        .next()
        .unwrap_or_default()
        .to_lowercase();
    if IMPP.contains(&schema.as_str()) {
        return schema;
    }
    let marke = p
        .get("X-SERVICE-TYPE")
        .map(|s| s.trim().to_lowercase())
        .unwrap_or_default();
    if IMPP.contains(&marke.as_str()) {
        marke
    } else {
        format!("{VCARD}impp")
    }
}

fn aus_uri(art: &str, wert: &str) -> String {
    let w = wert.trim();
    if !IMPP.contains(&art) {
        return w.to_string();
    }
    let vorne = format!("{art}:");
    let ohne = if w.to_lowercase().starts_with(&vorne) {
        &w[vorne.len()..]
    } else {
        w
    };
    if art == "matrix" {
        if let Some((n, s)) = ohne.strip_prefix("u/").and_then(|r| r.split_once(':')) {
            return format!("@{n}:{s}");
        }
    }
    ohne.to_string()
}

/// An `;` teilen, die trennen -- nicht an maskierten.
fn teilen(w: &str) -> Vec<String> {
    let mut aus = vec![String::new()];
    let mut vorher = '\0';
    for c in w.chars() {
        if c == ';' && vorher != '\\' {
            aus.push(String::new());
        } else {
            aus.last_mut().unwrap().push(c);
        }
        vorher = c;
    }
    aus
}

fn feldname_gut(n: &str) -> bool {
    !n.is_empty()
        && n.len() <= 51
        && n.chars().next().is_some_and(|c| c.is_ascii_alphanumeric())
        && n.chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-')
}

#[derive(Debug, Default, PartialEq)]
pub(crate) struct Gelesen {
    pub uid: String,
    pub name: String,
    pub foto: Option<Vec<u8>>,
    pub wege: Vec<Weg>,
}

/// Die erste vCard eines Texts (eine Ressource trägt eine).
pub(crate) fn lesen(inhalt: &str) -> Option<Gelesen> {
    let mut k: Option<Gelesen> = None;
    let mut nachname = String::new();
    for zeile in entfalten(inhalt) {
        let oben = zeile.trim().to_uppercase();
        if oben == "BEGIN:VCARD" {
            k = Some(Gelesen::default());
            continue;
        }
        if oben == "END:VCARD" {
            let mut g = k.take()?;
            if g.name.is_empty() {
                g.name = nachname;
            }
            return (!g.name.is_empty()).then_some(g);
        }
        let Some(g) = k.as_mut() else { continue };
        let Some((kopf, wert)) = zeile.split_once(':') else {
            continue;
        };
        let teile: Vec<&str> = kopf.split(';').collect();
        let mut name = teile[0].trim().to_uppercase();
        if let Some(i) = name.rfind('.') {
            name = name[i + 1..].to_string();
        }
        let p = parameter(&teile[1..]);
        let wert = if p
            .get("ENCODING")
            .is_some_and(|e| e.eq_ignore_ascii_case("QUOTED-PRINTABLE"))
        {
            qp(wert)
        } else {
            wert.to_string()
        };
        match name.as_str() {
            "PHOTO" => {
                let mut w = wert.trim().to_string();
                if w.to_lowercase().starts_with("data:") {
                    w = w
                        .split_once(',')
                        .map(|(_, r)| r.to_string())
                        .unwrap_or_default();
                } else if !p.get("ENCODING").is_some_and(|e| {
                    e.eq_ignore_ascii_case("b") || e.eq_ignore_ascii_case("base64")
                }) {
                    continue;
                }
                let w: String = w.chars().filter(|c| !c.is_whitespace()).collect();
                g.foto = base64::engine::general_purpose::STANDARD.decode(w).ok();
                continue;
            }
            "UID" => {
                g.uid = wert.trim().to_string();
                continue;
            }
            "FN" => {
                g.name = entmaskieren(&wert).trim().to_string();
                continue;
            }
            "N" => {
                let s: Vec<String> = teilen(&wert)
                    .iter()
                    .map(|x| entmaskieren(x).trim().to_string())
                    .collect();
                nachname = [
                    s.get(1).cloned().unwrap_or_default(),
                    s.first().cloned().unwrap_or_default(),
                ]
                .into_iter()
                .filter(|x| !x.is_empty())
                .collect::<Vec<_>>()
                .join(" ");
                continue;
            }
            _ => {}
        }
        let art = if name == "IMPP" {
            Some(dienst(&wert, &p))
        } else {
            FELDER
                .iter()
                .find(|(_, f)| *f == name)
                .map(|(a, _)| a.to_string())
                .or_else(|| {
                    NUR_LESEN
                        .iter()
                        .find(|(f, _)| *f == name)
                        .map(|(_, a)| a.to_string())
                })
        };
        let Some(art) = art else {
            if name == "KEY" || NICHT_DURCHREICHEN.contains(&name.as_str()) || !feldname_gut(&name)
            {
                continue;
            }
            g.wege.push(Weg {
                art: format!("{VCARD}{}", name.to_lowercase()),
                beschriftung: beschriftung(&p),
                wert: wert.trim().to_string(),
            });
            continue;
        };
        let wert = if name == "IMPP" {
            aus_uri(&art, &entmaskieren(&wert))
        } else {
            wert
        };
        let fertig = if art == "address" {
            teilen(&wert)
                .iter()
                .map(|x| entmaskieren(x).trim().to_string())
                .filter(|x| !x.is_empty())
                .collect::<Vec<_>>()
                .join(", ")
        } else if name == "IMPP" {
            wert.trim().to_string()
        } else {
            entmaskieren(&wert).trim().to_string()
        };
        if fertig.is_empty() {
            continue;
        }
        g.wege.push(Weg {
            art,
            beschriftung: beschriftung(&p),
            wert: fertig,
        });
    }
    None
}

// --- CardDAV ------------------------------------------------------------------

fn etag(k: &Kontakt) -> String {
    format!("\"{}\"", k.abdruck())
}

fn ctag(kontakte: &[Kontakt]) -> String {
    let mut text = String::new();
    for k in kontakte {
        text.push_str(&k.uuid);
        text.push_str(&k.abdruck());
    }
    openany_store::fotoabdruck(text.as_bytes())[..24].to_string()
}

fn name_von(uuid: &str, namen: &HashMap<String, Davname>) -> (String, String) {
    namen
        .get(uuid)
        .map(|n| (n.name.clone(), n.uid.clone()))
        .unwrap_or_else(|| (uuid.to_string(), uuid.to_string()))
}

fn href(name: &str) -> String {
    format!("{BUCH}{}.vcf", percent_encode(name))
}

fn buch_props(kontakte: &[Kontakt]) -> String {
    format!(
        "<d:resourcetype><d:collection/><card:addressbook/></d:resourcetype>\
         <d:displayname>openany</d:displayname>\
         <card:supported-address-data><card:address-data-type content-type=\"text/vcard\" version=\"3.0\"/></card:supported-address-data>\
         <cs:getctag>{}</cs:getctag>\
         <d:supported-report-set><d:supported-report><d:report><card:addressbook-query/></d:report></d:supported-report><d:supported-report><d:report><card:addressbook-multiget/></d:report></d:supported-report></d:supported-report-set>{}",
        ctag(kontakte),
        rechte(true)
    )
}

fn karte_props(k: &Kontakt, daten: Option<&str>) -> String {
    let d = daten
        .map(|d| format!("<card:address-data>{}</card:address-data>", xml(d)))
        .unwrap_or_default();
    format!(
        "<d:resourcetype/><d:getcontenttype>text/vcard; charset=utf-8</d:getcontenttype><d:getetag>{}</d:getetag>{d}",
        xml(&etag(k))
    )
}

fn karte(speicher: &Speicher, k: &Kontakt, namen: &HashMap<String, Davname>) -> String {
    let (_, uid) = name_von(&k.uuid, namen);
    let foto = k
        .foto
        .as_ref()
        .and_then(|_| speicher.kontaktfoto(&k.uuid).ok().flatten());
    schreiben(k, &uid, foto.as_deref())
}

/// Eine Anfrage unter `/dav/adressbuch/` beantworten.
pub(crate) fn bearbeiten(speicher: &Speicher, a: &Anfrage) -> Result<Antwort, String> {
    let methode = a.methode.to_ascii_uppercase();
    let pfad = a.pfad.split('?').next().unwrap_or_default();
    let kontakte = speicher.kontakte().map_err(|e| e.to_string())?;
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
            k.push_str(&antwort_eintrag(BUCH, &buch_props(&kontakte)));
        }
        return Ok(Antwort::xml(format!("{KOPF}{k}{FUSS}")));
    }
    let Some(rest) = pfad.strip_prefix(BUCH.trim_end_matches('/')) else {
        return Ok(Antwort::leer(404));
    };
    let rest = rest.trim_start_matches('/');
    let ressource =
        (!rest.is_empty()).then(|| percent_decode(rest.strip_suffix(".vcf").unwrap_or(rest)));
    let uuid_von = |name: &str| -> Option<String> {
        if let Ok(Some(u)) = speicher.davname_zu_uuid(ART, name) {
            return Some(u);
        }
        kontakte
            .iter()
            .find(|k| k.uuid == name && !namen.contains_key(&k.uuid))
            .map(|k| k.uuid.clone())
    };

    match (methode.as_str(), ressource) {
        ("PROPFIND", None) => {
            let mut k = antwort_eintrag(BUCH, &buch_props(&kontakte));
            if a.tiefe >= 1 {
                for kt in &kontakte {
                    let (name, _) = name_von(&kt.uuid, &namen);
                    k.push_str(&antwort_eintrag(&href(&name), &karte_props(kt, None)));
                }
            }
            Ok(Antwort::xml(format!("{KOPF}{k}{FUSS}")))
        }
        ("REPORT", None) => {
            let gewuenscht: Option<Vec<String>> = a
                .koerper
                .contains("addressbook-multiget")
                .then(|| hrefs(&a.koerper));
            let mut k = String::new();
            for kt in &kontakte {
                let (name, _) = name_von(&kt.uuid, &namen);
                let h = href(&name);
                if let Some(g) = &gewuenscht {
                    if !g.iter().any(|x| percent_decode(x) == percent_decode(&h)) {
                        continue;
                    }
                }
                let v = karte(speicher, kt, &namen);
                k.push_str(&antwort_eintrag(&h, &karte_props(kt, Some(&v))));
            }
            Ok(Antwort::xml(format!("{KOPF}{k}{FUSS}")))
        }
        ("PROPFIND", Some(name)) => {
            let Some(kt) = uuid_von(&name).and_then(|u| kontakte.iter().find(|k| k.uuid == u))
            else {
                return Ok(Antwort::leer(404));
            };
            Ok(Antwort::xml(format!(
                "{KOPF}{}{FUSS}",
                antwort_eintrag(&href(&name), &karte_props(kt, None))
            )))
        }
        ("GET" | "HEAD", Some(name)) => {
            let Some(kt) = uuid_von(&name).and_then(|u| kontakte.iter().find(|k| k.uuid == u))
            else {
                return Ok(Antwort::leer(404));
            };
            Ok(Antwort {
                status: 200,
                kopf: vec![
                    ("Content-Type", "text/vcard; charset=utf-8".into()),
                    ("ETag", etag(kt)),
                ],
                koerper: if methode == "HEAD" {
                    String::new()
                } else {
                    karte(speicher, kt, &namen)
                },
            })
        }
        ("PUT", Some(name)) => {
            let vorhanden = uuid_von(&name).and_then(|u| kontakte.iter().find(|k| k.uuid == u));
            if a.if_none_match.as_deref() == Some("*") && vorhanden.is_some() {
                return Ok(Antwort::leer(412));
            }
            if let Some(m) = a.if_match.as_deref() {
                match vorhanden {
                    Some(k) if m == "*" || m == etag(k) => {}
                    _ => return Ok(Antwort::leer(412)),
                }
            }
            let Some(g) = lesen(&a.koerper) else {
                return Ok(Antwort::leer(400));
            };
            let neu_angelegt = vorhanden.is_none();
            let mut k = vorhanden.cloned().unwrap_or_else(|| Kontakt {
                uuid: uuid::Uuid::new_v4().to_string(),
                ..Default::default()
            });
            k.anzeigename = g.name;
            k.wege = g.wege;
            k.papierkorb_at = None;
            k.geaendert_at = String::new();
            speicher
                .kontakt_schreiben(&k, Protokoll::Merken)
                .map_err(|e| e.to_string())?;
            // Ein neues Foto übernehmen; fehlt es in der Karte, bleibt das alte.
            if let Some(bytes) = g
                .foto
                .filter(|b| !b.is_empty() && b.len() <= FOTO_HOECHSTENS)
            {
                if k.foto.as_deref() != Some(openany_store::fotoabdruck(&bytes).as_str()) {
                    speicher
                        .kontaktfoto_setzen(&k.uuid, &bytes)
                        .map_err(|e| e.to_string())?;
                }
            }
            let uid = g.uid.trim_start_matches("urn:uuid:").to_string();
            if name != k.uuid || (!uid.is_empty() && uid != k.uuid) {
                speicher
                    .davname_merken(
                        ART,
                        &k.uuid,
                        &Davname {
                            name: name.clone(),
                            uid: if g.uid.is_empty() {
                                name.clone()
                            } else {
                                g.uid.clone()
                            },
                        },
                    )
                    .map_err(|e| e.to_string())?;
            }
            let neu = speicher
                .kontakt(&k.uuid)
                .map_err(|e| e.to_string())?
                .unwrap_or(k);
            Ok(Antwort {
                status: if neu_angelegt { 201 } else { 204 },
                kopf: vec![("ETag", etag(&neu))],
                koerper: String::new(),
            })
        }
        ("DELETE", Some(name)) => {
            let Some(u) = uuid_von(&name) else {
                return Ok(Antwort::leer(404));
            };
            if let Some(m) = a.if_match.as_deref() {
                if let Some(k) = kontakte.iter().find(|k| k.uuid == u) {
                    if m != "*" && m != etag(k) {
                        return Ok(Antwort::leer(412));
                    }
                }
            }
            speicher
                .papierkorb(&openany_client::Art::Kontakt, &u, Protokoll::Merken)
                .map_err(|e| e.to_string())?;
            Ok(Antwort::leer(204))
        }
        _ => Ok(Antwort::leer(405)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn weg(art: &str, b: Option<&str>, wert: &str) -> Weg {
        Weg {
            art: art.into(),
            beschriftung: b.map(Into::into),
            wert: wert.into(),
        }
    }

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

    #[test]
    fn eine_karte_geht_hin_und_zurueck() {
        let k = Kontakt {
            uuid: "1b637a58-9f50-4c7a-9b81-b3da53918dac".into(),
            anzeigename: "Beispiel; Probe".into(),
            wege: vec![
                weg("matrix", None, "@probe:beispiel.test"),
                weg("phone", Some("CELL"), "+49 30 5550100"),
                weg("address", Some("HOME"), "Musterweg 1, 12345 Beispielstadt"),
                weg("vcard:nickname", None, "Probi"),
            ],
            ..Default::default()
        };
        let text = schreiben(&k, &k.uuid, Some(b"bild"));
        assert!(text.contains("IMPP;X-SERVICE-TYPE=Matrix:matrix:u/probe:beispiel.test\r\n"));
        assert!(text.contains("UID:urn:uuid:1b637a58"));
        let g = lesen(&text).unwrap();
        assert_eq!(g.name, "Beispiel; Probe");
        assert_eq!(g.foto.as_deref(), Some(&b"bild"[..]));
        // Die Anschrift kommt als eine Zeile, aus den Teilen der ADR.
        assert_eq!(g.wege[2].wert, "Musterweg 1, 12345 Beispielstadt");
        assert_eq!(g.wege, k.wege);
    }

    #[test]
    fn eine_fremde_karte_mit_apple_gruppen_und_qp() {
        let g = lesen(
            "BEGIN:VCARD\r\nVERSION:2.1\r\nN:Probe;Beispiel;;;\r\n\
             item1.TEL;CELL:+49 30 1\r\nitem1.X-ABLabel:Handy\r\n\
             NOTE;ENCODING=QUOTED-PRINTABLE:Gr=C3=BC=C3=9Fe\r\nEND:VCARD\r\n",
        )
        .unwrap();
        assert_eq!(g.name, "Beispiel Probe");
        assert_eq!(g.wege[0], weg("phone", Some("CELL"), "+49 30 1"));
        assert!(g.wege.iter().any(|w| w.art == "other" && w.wert == "Grüße"));
    }

    #[test]
    fn ein_programm_legt_an_aendert_und_loescht() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        let z = bearbeiten(&s, &anfrage("PROPFIND", ZUHAUSE, 1, "")).unwrap();
        assert!(z.koerper.contains("<card:addressbook/>"));
        let karte = "BEGIN:VCARD\r\nVERSION:3.0\r\nUID:abc@beispiel.test\r\nFN:Beispiel Probe\r\n\
                     TEL;TYPE=WORK:+49 30 2\r\nEND:VCARD\r\n";
        let p = bearbeiten(
            &s,
            &anfrage("PUT", "/dav/adressbuch/openany/abc.vcf", 0, karte),
        )
        .unwrap();
        assert_eq!(p.status, 201);
        let k = &s.kontakte().unwrap()[0];
        assert_eq!(k.wege, vec![weg("phone", Some("WORK"), "+49 30 2")]);
        let g = bearbeiten(
            &s,
            &anfrage("GET", "/dav/adressbuch/openany/abc.vcf", 0, ""),
        )
        .unwrap();
        assert!(g.koerper.contains("UID:abc@beispiel.test"));
        let l = bearbeiten(&s, &anfrage("PROPFIND", BUCH, 1, "")).unwrap();
        assert!(l.koerper.contains("/dav/adressbuch/openany/abc.vcf"));
        let d = bearbeiten(
            &s,
            &anfrage("DELETE", "/dav/adressbuch/openany/abc.vcf", 0, ""),
        )
        .unwrap();
        assert_eq!(d.status, 204);
        assert!(s.kontakte().unwrap().is_empty());
    }
}
