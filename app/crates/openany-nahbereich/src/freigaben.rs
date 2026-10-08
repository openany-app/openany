//! Freigaben in ein lokales Projekt (docs/konzept-lokale-mitgliedschaften.md,
//! §2 und §5) -- zunaechst Notiz-Mappen, nur zum Lesen (Tiffy, 01.10.2026).
//!
//! **Jedes Mitglied gibt Eigenes frei**, wie auf dem Server. Eine Freigabe ist
//! ein unterschriebener Eintrag ("ich gebe Mappe X in Projekt P frei" oder
//! "... nicht mehr"), und gilt nur, wenn ein Geraet DIESER Person ihn
//! unterschrieben hat und die Person Mitglied ist. Je Person und Mappe gilt
//! der juengste Eintrag.
//!
//! **Keine Kette wie bei den Mitgliedern:** Freigaben verschiedener Personen
//! haengen nicht voneinander ab, und eine Person entscheidet nur ueber ihre
//! eigenen. Eine Menge unterschriebener Eintraege reicht.

use crate::mitglieder::Mitglied;
use crate::person::{der, pruefen, unterschreiben, PersonFehler};
use crate::tls::fingerabdruck;
use crate::{Identitaet, Person};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const NOTIZ_MAPPE: &str = "note_folder";
/// Ein Ordner aus Dateien oder Dokumente (Schluessel: seine uuid).
pub const ORDNER: &str = "file_folder";
/// Ein Album der Galerie (Schluessel: seine uuid).
pub const ALBUM: &str = "album";
pub const LESEN: &str = "read";
/// Notiz-Mappen: lesen UND bearbeiten (mit Sperre beim fuehrenden Geraet).
pub const BEARBEITEN: &str = "edit";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Freigabe {
    pub projekt: String,
    /// `note_folder` (spaeter auch Ordner und Alben).
    pub art: String,
    /// Bei Notiz-Mappen: der Mappenpfad.
    pub schluessel: String,
    /// Wie die Freigabe heisst (der Mappenname).
    pub name: String,
    /// Wer freigibt.
    pub personen_id: String,
    /// `read`; leer, wenn die Freigabe zurueckgenommen ist.
    pub stufe: String,
    pub at: String,
    pub unterzeichner: String,
    pub zertifikat_pem: String,
    pub signatur: String,
}

fn nachricht(f: &Freigabe) -> Vec<u8> {
    let mut n = b"openany-freigabe-v1".to_vec();
    for teil in [
        &f.projekt,
        &f.art,
        &f.schluessel,
        &f.name,
        &f.personen_id,
        &f.stufe,
        &f.at,
        &f.unterzeichner,
    ] {
        n.push(0x1f);
        n.extend_from_slice(teil.as_bytes());
    }
    n
}

impl Freigabe {
    /// Freigeben (`stufe` = `read`) oder zuruecknehmen (`stufe` leer).
    pub fn unterschreiben(
        ident: &Identitaet,
        projekt: &str,
        personen_id: &str,
        art: &str,
        schluessel: &str,
        name: &str,
        stufe: &str,
    ) -> Result<Self, PersonFehler> {
        let mut f = Self {
            projekt: projekt.to_string(),
            art: art.to_string(),
            schluessel: schluessel.to_string(),
            name: name.to_string(),
            personen_id: personen_id.to_string(),
            stufe: stufe.to_string(),
            at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            unterzeichner: ident.fingerabdruck.clone(),
            zertifikat_pem: ident.zertifikat_pem.clone(),
            signatur: String::new(),
        };
        f.signatur = unterschreiben(ident, &nachricht(&f))?;
        Ok(f)
    }

    /// Echt, fuer dieses Projekt, von einem Geraet der genannten Person, und
    /// die Person ist Mitglied.
    pub fn gilt(
        &self,
        projekt: &str,
        mitglieder: &[Mitglied],
        person: &dyn Fn(&str) -> Option<Person>,
    ) -> bool {
        if self.projekt != projekt {
            return false;
        }
        let echt = der(&self.zertifikat_pem)
            .is_some_and(|d| fingerabdruck(&d) == self.unterzeichner)
            && pruefen(&self.zertifikat_pem, &nachricht(self), &self.signatur);
        echt && mitglieder.iter().any(|m| {
            m.personen_id == self.personen_id && geraet_von(m, &self.unterzeichner, person)
        })
    }
}

/// Gehoert das Geraet `fp` zu diesem Mitglied? Der Anker sicher; weitere
/// Geraete, wenn die Person bekannt ist und ihre Kette vom Anker aus traegt.
pub fn geraet_von(m: &Mitglied, fp: &str, person: &dyn Fn(&str) -> Option<Person>) -> bool {
    fp == m.geraet
        || person(&m.personen_id).is_some_and(|p| p.gueltige(&[&m.geraet]).contains_key(fp))
}

/// Zu welchem Mitglied gehoert das Geraet `fp`? `None`: zu keinem.
pub fn mitglied_von<'a>(
    mitglieder: &'a [Mitglied],
    fp: &str,
    person: &dyn Fn(&str) -> Option<Person>,
) -> Option<&'a Mitglied> {
    mitglieder.iter().find(|m| geraet_von(m, fp, person))
}

/// Die geltenden Freigaben: je Person und Mappe der juengste gueltige
/// Eintrag, und nur, wenn er nicht zuruecknimmt.
pub fn geltende<'a>(
    eintraege: &'a [Freigabe],
    projekt: &str,
    mitglieder: &[Mitglied],
    person: &dyn Fn(&str) -> Option<Person>,
) -> Vec<&'a Freigabe> {
    let mut juengste: BTreeMap<(&str, &str, &str), &Freigabe> = BTreeMap::new();
    for f in eintraege
        .iter()
        .filter(|f| f.gilt(projekt, mitglieder, person))
    {
        let k = (
            f.personen_id.as_str(),
            f.art.as_str(),
            f.schluessel.as_str(),
        );
        if juengste.get(&k).is_none_or(|v| f.at > v.at) {
            juengste.insert(k, f);
        }
    }
    juengste
        .into_values()
        .filter(|f| !f.stufe.is_empty())
        .collect()
}

/// Zwei Mengen zusammenlegen (ohne Doppelte).
pub fn zusammenlegen(eigene: &mut Vec<Freigabe>, andere: &[Freigabe]) {
    for f in andere {
        if !eigene.contains(f) {
            eigene.push(f.clone());
        }
    }
}

/// Liegt eine Notiz mit diesem Mappenpfad in der freigegebenen Mappe (oder
/// darunter)?
pub fn in_mappe(notiz_mappe: Option<&str>, freigegeben: &str) -> bool {
    notiz_mappe.is_some_and(|m| m == freigegeben || m.starts_with(&format!("{freigegeben}/")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mitglieder::{Mitgliederliste, MITGLIED};

    #[test]
    fn nur_eigenes_von_mitgliedern_und_der_juengste_eintrag_gilt() {
        let anna_tel = Identitaet::erzeugen().unwrap();
        let ben_tel = Identitaet::erzeugen().unwrap();
        let fremd = Identitaet::erzeugen().unwrap();
        let anna = Person::neu(&anna_tel, "Telefon").unwrap();
        let ben = Person::neu(&ben_tel, "Telefon").unwrap();
        let mut liste = Mitgliederliste::gruenden(&anna_tel, &anna, "p1").unwrap();
        liste
            .eintragen(
                &anna_tel,
                "beitritt",
                &ben.personen_id,
                "Ben",
                MITGLIED,
                &ben_tel.fingerabdruck,
            )
            .unwrap();
        let m = liste.mitglieder(&|_| None).unwrap();
        let keiner = |_: &str| None;

        let annas = Freigabe::unterschreiben(
            &anna_tel,
            "p1",
            &anna.personen_id,
            NOTIZ_MAPPE,
            "Garten",
            "Garten",
            LESEN,
        )
        .unwrap();
        // Ben gibt im Namen von Anna frei: gilt nicht.
        let untergeschoben = Freigabe::unterschreiben(
            &ben_tel,
            "p1",
            &anna.personen_id,
            NOTIZ_MAPPE,
            "Privat",
            "Privat",
            LESEN,
        )
        .unwrap();
        // Jemand, der kein Mitglied ist: gilt nicht.
        let von_fremd =
            Freigabe::unterschreiben(&fremd, "p1", "x", NOTIZ_MAPPE, "Spam", "Spam", LESEN)
                .unwrap();
        let alle = vec![annas.clone(), untergeschoben, von_fremd];
        let g = geltende(&alle, "p1", &m, &keiner);
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].schluessel, "Garten");

        // Anna nimmt zurueck: der juengere Eintrag gilt.
        std::thread::sleep(std::time::Duration::from_millis(5));
        let zurueck = Freigabe::unterschreiben(
            &anna_tel,
            "p1",
            &anna.personen_id,
            NOTIZ_MAPPE,
            "Garten",
            "Garten",
            "",
        )
        .unwrap();
        let mut alle = alle;
        zusammenlegen(&mut alle, &[zurueck, annas]);
        assert!(geltende(&alle, "p1", &m, &keiner).is_empty());

        // Fuer ein anderes Projekt unterschrieben: gilt hier nicht.
        let anderes = Freigabe::unterschreiben(
            &anna_tel,
            "p2",
            &anna.personen_id,
            NOTIZ_MAPPE,
            "A",
            "A",
            LESEN,
        )
        .unwrap();
        assert!(!anderes.gilt("p1", &m, &keiner));
    }

    #[test]
    fn unter_mappen_gehoeren_dazu() {
        assert!(in_mappe(Some("Garten"), "Garten"));
        assert!(in_mappe(Some("Garten/Beete"), "Garten"));
        assert!(!in_mappe(Some("Gartenzaun"), "Garten"));
        assert!(!in_mappe(None, "Garten"));
    }
}
