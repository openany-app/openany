//! Dateien, Bilder, Alben und Ordner Ende-zu-Ende (docs/plan-dateien-e2e.md, 7b).
//!
//! Dasselbe Muster wie [`crate::notizschloss`]: **uebersetzt wird an der
//! Grenze zum Server.** Speicher, Laeufer und der Abgleich vor Ort kennen
//! alles im Klartext.
//!
//! **Der Kopf** traegt genau die Felder, die der Abgleich sonst offen
//! schickte (Name, Eltern bzw. Album, Zone, Typ, Groesse, Abdruck, Vorschau,
//! EXIF ...) -- dazu bei Dateien und Bildern den Schluessel des Inhalts und
//! die Abdruecke der Chiffrate. Wer den Kopf oeffnet, kann die Bytes holen,
//! pruefen und entschluesseln; der Server sieht davon nichts.
//!
//! Hinaus geht ein verdeckter Abdruck des Kopfs (wie bei Notizen), damit der
//! Server Gleiches erkennt, ohne zu lesen.
//!
//! **In einer Freigabe** (geteiltes Album, geteilter Ordner, Bild einer
//! geteilten Mappe; 7c) ist der Kopf mit dem Freigabeschluessel statt dem
//! Kontoschluessel verschlossen und nennt die Freigabe offen -- so koennen
//! Mitglieder ihn oeffnen. Die Bytes bleiben, wie sie sind.

use openany_client::{Art, Eintrag, Was};
use openany_direktpost::{Kontoschluessel, Schluessel, Verschluesselt};
use openany_store::Inhaltsschluessel;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use crate::adressbuch::Adressbuch;
use crate::mappenteilen::Freigabe;

/// Felder, die nicht in den Kopf gehoeren: Zeitpunkt und Abgleichs-Beiwerk.
const NICHT_IM_KOPF: [&str; 4] = ["updated_at", "base_hash", "origin", "deleted_at"];

#[derive(Serialize, Deserialize)]
struct Kopf {
    v: u8,
    uuid: String,
    art: String,
    felder: Map<String, Value>,
    /// Bei Dateien und Bildern: age-Identitaet des Inhalts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    schluessel: Option<String>,
    /// Abdruck und Groesse des Chiffrats (so heisst es beim Server).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    chiffrat: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    chiffrat_groesse: Option<u64>,
    /// Abdruck des Vorschau-Chiffrats (Bilder), mit demselben Schluessel.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    vorschau_chiffrat: Option<String>,
}

/// Eine Art, die als Kopf reist?
pub(crate) fn ist_dateiart(art: &Art) -> bool {
    matches!(art, Art::Album | Art::Datei | Art::Medium)
}

/// Ein Eintrag, den der Server verschluesselt liefert?
pub(crate) fn ist_verschluesselt(e: &Eintrag) -> bool {
    ist_dateiart(&e.art) && e.was == Was::Da && e.felder.contains_key("inhalt")
}

/// Ist das ein Ordner (nicht eine Datei)?
fn ist_ordner(e: &Eintrag) -> bool {
    e.felder.get("is_folder").and_then(Value::as_bool) == Some(true)
}

/// Braucht dieser Eintrag einen Inhaltsschluessel (Datei oder Bild)?
pub(crate) fn traegt_inhalt(e: &Eintrag) -> bool {
    e.art == Art::Medium || (e.art == Art::Datei && !ist_ordner(e))
}

fn hex(d: impl AsRef<[u8]>) -> String {
    d.as_ref().iter().map(|b| format!("{b:02x}")).collect()
}

/// Womit ein Kopf verschlossen wird: der Kontoschluessel oder der einer
/// Freigabe (dann mit deren uuid).
pub(crate) struct Verschluss<'a> {
    pub ich: &'a Schluessel,
    pub geraet: i64,
    pub schluessel: &'a Kontoschluessel,
    pub freigabe: Option<&'a str>,
}

/// Einen Eintrag als Kopf verschliessen -- siehe [`Adressbuch::datei_kopf_verschliessen`].
pub(crate) fn kopf_zu(
    v: &Verschluss<'_>,
    e: &Eintrag,
    inhalt: Option<&Inhaltsschluessel>,
    vorschau_chiffrat: Option<&str>,
) -> Result<Eintrag, String> {
    let mut felder = e.felder.clone();
    for f in NICHT_IM_KOPF {
        felder.remove(f);
    }
    let kopf_abdruck = hex(Sha256::digest(
        serde_json::to_vec(&felder).map_err(|e| e.to_string())?,
    ));
    let roh = serde_json::to_vec(&Kopf {
        v: 1,
        uuid: e.key.clone(),
        art: e.art.ueber_die_leitung().to_string(),
        felder,
        schluessel: inhalt.map(|k| k.schluessel.clone()),
        chiffrat: inhalt.map(|k| k.chiffrat.clone()),
        chiffrat_groesse: inhalt.map(|k| k.groesse),
        vorschau_chiffrat: vorschau_chiffrat.map(str::to_string),
    })
    .map_err(|e| e.to_string())?;
    let (c, version) = openany_direktpost::sache_verschluesseln(
        v.ich,
        v.schluessel,
        e.art.ueber_die_leitung(),
        &e.key,
        &roh,
    )
    .map_err(|e| e.to_string())?;
    let mut aus = Eintrag::neu(e.art.clone(), e.key.clone(), Was::Da)
        .mit("inhalt", c.inhalt)
        .mit("signatur", c.signatur)
        .mit("geraet", v.geraet)
        .mit("version", version)
        .mit(
            "hash",
            v.schluessel.abdruck_verdecken(&e.key, &kopf_abdruck),
        );
    if let Some(f) = v.freigabe {
        aus = aus.mit("freigabe", f);
    }
    if e.art == Art::Datei && ist_ordner(e) {
        aus = aus.mit("ordner", true);
    }
    if let Some(t) = e.felder.get("updated_at") {
        aus = aus.mit("updated_at", t.clone());
    }
    Ok(aus)
}

/// Einen Kopf oeffnen -- mit dem Schluessel, mit dem er verschlossen ist,
/// unterschrieben von dem Geraet mit `ed25519`.
pub(crate) fn kopf_auf(
    schluessel: &Kontoschluessel,
    ed25519: &str,
    e: &Eintrag,
) -> Option<Geoeffnet> {
    let v = Verschluesselt {
        inhalt: e.text("inhalt")?.to_string(),
        signatur: e.text("signatur")?.to_string(),
    };
    let version = e.felder.get("version").and_then(Value::as_u64)? as u32;
    let roh = openany_direktpost::sache_oeffnen(
        schluessel,
        e.art.ueber_die_leitung(),
        &e.key,
        version,
        &v,
        ed25519,
    )
    .ok()?;
    let k: Kopf = serde_json::from_slice(&roh).ok()?;
    // Der Server darf einen Kopf nicht unter fremder Kennung ausgeben.
    if k.uuid != e.key || k.art != e.art.ueber_die_leitung() {
        return None;
    }
    let mut eintrag = Eintrag::neu(e.art.clone(), e.key.clone(), Was::Da);
    eintrag.felder = k.felder;
    if let Some(t) = e.felder.get("updated_at") {
        eintrag = eintrag.mit("updated_at", t.clone());
    }
    let schluessel = k.schluessel.clone();
    let inhalt = match (&schluessel, eintrag.text("hash"), &k.chiffrat) {
        (Some(s), Some(a), Some(c)) => Some(Inhaltsschluessel {
            abdruck: a.to_string(),
            schluessel: s.clone(),
            chiffrat: c.clone(),
            groesse: k.chiffrat_groesse.unwrap_or(0),
        }),
        _ => None,
    };
    // Die Vorschau nennt der Kopf -- oder, wenn sie erst nach dem Kopf
    // entstand, der Server (`thumb_hash` des Chiffrats). Sie ist mit
    // demselben Schluessel zu; aufgehen kann sie nur mit ihm.
    let vorschau_chiffrat = k
        .vorschau_chiffrat
        .clone()
        .or_else(|| e.text("thumb_hash").map(str::to_string));
    let vorschau = match (&schluessel, eintrag.text("thumb_hash"), &vorschau_chiffrat) {
        (Some(s), Some(a), Some(c)) => Some(Inhaltsschluessel {
            abdruck: a.to_string(),
            schluessel: s.clone(),
            chiffrat: c.clone(),
            groesse: 0,
        }),
        _ => None,
    };
    Some(Geoeffnet {
        eintrag,
        inhalt,
        vorschau,
    })
}

/// Was ein geoeffneter Kopf ueber Schluessel mitbringt.
pub(crate) struct Geoeffnet {
    pub eintrag: Eintrag,
    pub inhalt: Option<Inhaltsschluessel>,
    pub vorschau: Option<Inhaltsschluessel>,
}

impl Adressbuch {
    /// Mit dem Kontoschluessel -- oder dem Schluessel einer Freigabe (7c).
    pub(crate) fn verschluss<'a>(&'a self, f: Option<&'a Freigabe>) -> Verschluss<'a> {
        Verschluss {
            ich: &self.ich,
            geraet: self.geraet,
            schluessel: f.map_or(&self.konto, |f| &f.schluessel),
            freigabe: f.map(|f| f.uuid.as_str()),
        }
    }

    /// Einen Eintrag als Kopf verschliessen. `inhalt` und `vorschau_chiffrat`
    /// nur bei Dateien und Bildern -- ohne Inhaltsschluessel gibt es dort
    /// keinen Kopf (der kaeme mit dem Hochladen).
    pub fn datei_kopf_verschliessen(
        &self,
        e: &Eintrag,
        inhalt: Option<&Inhaltsschluessel>,
        vorschau_chiffrat: Option<&str>,
        f: Option<&Freigabe>,
    ) -> Result<Eintrag, String> {
        kopf_zu(&self.verschluss(f), e, inhalt, vorschau_chiffrat)
    }

    /// Den Kopf, wie er mit einem Upload reist (`sync/uploads`, Feld `kopf`).
    pub fn upload_kopf(
        &self,
        e: &Eintrag,
        inhalt: &Inhaltsschluessel,
        vorschau_chiffrat: Option<&str>,
        f: Option<&Freigabe>,
    ) -> Result<Value, String> {
        let k = self.datei_kopf_verschliessen(e, Some(inhalt), vorschau_chiffrat, f)?;
        let mut aus = serde_json::json!({
            "inhalt": k.felder["inhalt"],
            "signatur": k.felder["signatur"],
            "geraet": k.felder["geraet"],
            "version": k.felder["version"],
            "hash": k.felder["hash"],
        });
        if let Some(f) = f {
            aus["freigabe"] = f.uuid.clone().into();
        }
        Ok(aus)
    }

    /// Einen gezogenen Kopf oeffnen: der Eintrag im Klartext und, bei
    /// Dateien und Bildern, die Schluessel fuer Inhalt und Vorschau. Liegt er
    /// in einer eigenen Freigabe, mit deren Schluessel (`f`).
    pub(crate) fn datei_kopf_oeffnen(
        &self,
        e: &Eintrag,
        f: Option<&Freigabe>,
    ) -> Option<Geoeffnet> {
        let von = e.felder.get("geraet").and_then(Value::as_i64)?;
        let ed = self.geraete.get(&von)?;
        kopf_auf(f.map_or(&self.konto, |f| &f.schluessel), ed, e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn buch() -> Adressbuch {
        let ich = Schluessel::erzeugen().unwrap();
        let ed = ich.oeffentlich().1;
        Adressbuch {
            ich,
            konto: Kontoschluessel::neu(),
            geraet: 7,
            geraete: HashMap::from([(7, ed)]),
        }
    }

    fn bild() -> Eintrag {
        Eintrag::neu(Art::Medium, "u-1".to_string(), Was::Da)
            .mit("name", "Strand.jpg")
            .mit("album", "a-1")
            .mit("mime_type", "image/jpeg")
            .mit("size", 1234u64)
            .mit("hash", "a".repeat(64))
            .mit("thumb_hash", "b".repeat(64))
            .mit("updated_at", "2026-10-20T10:00:00Z")
    }

    #[test]
    fn ein_bild_geht_als_kopf_und_kommt_zurueck() {
        let b = buch();
        let k = Inhaltsschluessel {
            abdruck: "a".repeat(64),
            schluessel: openany_direktpost::dateischloss::neuer_schluessel(),
            chiffrat: "c".repeat(64),
            groesse: 1300,
        };
        let zu = b
            .datei_kopf_verschliessen(&bild(), Some(&k), Some(&"d".repeat(64)), None)
            .unwrap();
        assert!(zu.text("name").is_none());
        assert!(zu.text("album").is_none());
        assert!(ist_verschluesselt(&zu));

        let auf = b.datei_kopf_oeffnen(&zu, None).unwrap();
        assert_eq!(auf.eintrag.text("name"), Some("Strand.jpg"));
        assert_eq!(auf.eintrag.text("album"), Some("a-1"));
        assert_eq!(auf.eintrag.text("updated_at"), Some("2026-10-20T10:00:00Z"));
        assert_eq!(auf.inhalt, Some(k.clone()));
        let v = auf.vorschau.unwrap();
        assert_eq!(v.abdruck, "b".repeat(64));
        assert_eq!(v.chiffrat, "d".repeat(64));
        assert_eq!(v.schluessel, k.schluessel);
    }

    #[test]
    fn gleicher_kopf_gleicher_abdruck() {
        let b = buch();
        let ordner = Eintrag::neu(Art::Datei, "o-1".to_string(), Was::Da)
            .mit("name", "Akte")
            .mit("is_folder", true);
        let x = b
            .datei_kopf_verschliessen(&ordner, None, None, None)
            .unwrap();
        let y = b
            .datei_kopf_verschliessen(
                &ordner.clone().mit("updated_at", "spaeter"),
                None,
                None,
                None,
            )
            .unwrap();
        assert_eq!(x.text("hash"), y.text("hash"));
        assert_eq!(x.felder.get("ordner"), Some(&Value::Bool(true)));
    }

    #[test]
    fn in_einer_freigabe_mit_deren_schluessel() {
        let b = buch();
        let f = Freigabe {
            uuid: "f-1".into(),
            art: "album".into(),
            pfad: String::new(),
            wurzel: Some("a-1".into()),
            schluessel: Kontoschluessel::neu(),
        };
        let k = Inhaltsschluessel {
            abdruck: "a".repeat(64),
            schluessel: openany_direktpost::dateischloss::neuer_schluessel(),
            chiffrat: "c".repeat(64),
            groesse: 9,
        };
        let zu = b
            .datei_kopf_verschliessen(&bild(), Some(&k), None, Some(&f))
            .unwrap();
        assert_eq!(zu.text("freigabe"), Some("f-1"));
        // Mit dem Kontoschluessel geht er nicht auf, mit dem der Freigabe schon.
        assert!(b.datei_kopf_oeffnen(&zu, None).is_none());
        let auf = b.datei_kopf_oeffnen(&zu, Some(&f)).unwrap();
        assert_eq!(auf.eintrag.text("name"), Some("Strand.jpg"));
        // Ein Mitglied: nur der Freigabeschluessel und das Geraet des Besitzers.
        let ed = b.ich.oeffentlich().1;
        let mitglied = kopf_auf(&f.schluessel, &ed, &zu).unwrap();
        assert_eq!(mitglied.inhalt, Some(k));

        let mit = b
            .upload_kopf(&bild(), &mitglied.inhalt.unwrap(), None, Some(&f))
            .unwrap();
        assert_eq!(mit["freigabe"], "f-1");
    }

    #[test]
    fn ein_kopf_unter_fremder_kennung_geht_nicht_auf() {
        let b = buch();
        let mut zu = b
            .datei_kopf_verschliessen(&bild(), None, None, None)
            .unwrap();
        zu.key = "u-2".to_string();
        assert!(b.datei_kopf_oeffnen(&zu, None).is_none());
    }
}
