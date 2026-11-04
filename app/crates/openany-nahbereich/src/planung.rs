//! Planung auf beiden Wegen (docs/plan-projekte-eine-art.md).
//!
//! **Alles unter Planung folgt dem Konzept des Whiteboards**: Eine Sache (Abstimmung, Liste, Whiteboard) gehoert der
//! Person, die sie anlegt; was andere beitragen (eine Stimme, „ich bringe
//! Salat"), ist ihr EIGENER Eintrag. Jeder Eintrag ist vom Geraet seiner
//! Schreiberin unterschrieben und darf deshalb ueber jeden reisen -- vor Ort
//! und, verschluesselt, ueber openany.de.
//!
//! **Ein Schreiber je Eintrag.** Eine neuere Fassung desselben Eintrags gilt
//! nur von derselben Person. Loeschen (eine Fassung mit `geloescht`) darf
//! ausserdem, wer die Sache angelegt hat, und jede Eigentuemerin des
//! Projekts -- Ordnung halten, wie auf dem Server bisher.
//!
//! **Die Anlage.** Jede Sache beginnt mit einem Eintrag, dessen id die id
//! der Sache ist (`inhalt: {"art":"anlage","sorte":…}`). Wer ihn schrieb,
//! hat die Sache angelegt; ist er geloescht, ist es die ganze Sache.
//!
//! Was ein Eintrag BEDEUTET (Kopf, Option, Stimme, Zeichnung) und welche
//! Regeln gelten (Plaetze, eine Stimme, Einladungen), entscheidet die
//! Oberflaeche -- fuer alle Geraete gleich. Hier zaehlt nur, wer schreiben
//! durfte.

use crate::mitglieder::{Mitgliederliste, EIGENTUEMER};
use crate::person::{der, pruefen, unterschreiben, PersonFehler};
use crate::tls::fingerabdruck;
use crate::{Identitaet, Person};
use serde::{Deserialize, Serialize};

/// Die Art, unter der Planungseintraege im Projekt liegen.
pub const EINTRAG: &str = "planung_eintrag";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Eintrag {
    pub id: String,
    pub projekt: String,
    /// Die Sache (Abstimmung, Whiteboard), zu der er gehoert.
    pub sache: String,
    /// `umfrage` oder `whiteboard`.
    pub sorte: String,
    pub personen_id: String,
    /// Der Inhalt als JSON-Text -- unterschrieben wird genau dieser Text.
    /// Leer, wenn geloescht.
    pub inhalt: String,
    pub geloescht: bool,
    pub at: String,
    pub unterzeichner: String,
    pub zertifikat_pem: String,
    pub signatur: String,
}

fn text(e: &Eintrag) -> Vec<u8> {
    let mut b = b"openany-planung-v1".to_vec();
    let geloescht = if e.geloescht { "1" } else { "0" };
    for teil in [
        e.id.as_str(),
        &e.projekt,
        &e.sache,
        &e.sorte,
        &e.personen_id,
        &e.inhalt,
        geloescht,
        &e.at,
        &e.unterzeichner,
    ] {
        b.push(0x1f);
        b.extend_from_slice(teil.as_bytes());
    }
    b
}

impl Eintrag {
    #[allow(clippy::too_many_arguments)]
    pub fn schreiben(
        ident: &Identitaet,
        projekt: &str,
        id: &str,
        sache: &str,
        sorte: &str,
        personen_id: &str,
        inhalt: Option<&str>,
    ) -> Result<Self, PersonFehler> {
        let mut e = Self {
            id: id.to_string(),
            projekt: projekt.to_string(),
            sache: sache.to_string(),
            sorte: sorte.to_string(),
            personen_id: personen_id.to_string(),
            inhalt: inhalt.unwrap_or_default().to_string(),
            geloescht: inhalt.is_none(),
            at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            unterzeichner: ident.fingerabdruck.clone(),
            zertifikat_pem: ident.zertifikat_pem.clone(),
            signatur: String::new(),
        };
        e.signatur = unterschreiben(ident, &text(&e))?;
        Ok(e)
    }

    /// Echt, fuer dieses Projekt, und von einem Geraet einer Person, die
    /// (je) Mitglied war. Ob die Fassung eine vorhandene ersetzen darf,
    /// entscheidet [`ersetzt`].
    pub fn gilt(
        &self,
        projekt: &str,
        liste: &Mitgliederliste,
        person: &dyn Fn(&str) -> Option<Person>,
    ) -> bool {
        if self.projekt != projekt
            || !["umfrage", "whiteboard"].contains(&self.sorte.as_str())
            || self.inhalt.len() > 8_000_000
        {
            return false;
        }
        let echt = der(&self.zertifikat_pem)
            .is_some_and(|d| fingerabdruck(&d) == self.unterzeichner)
            && pruefen(&self.zertifikat_pem, &text(self), &self.signatur);
        echt && crate::chat::von_mitglied(liste, &self.personen_id, &self.unterzeichner, person)
    }

    /// Die Inhalt als JSON, oder `None` (geloescht, unlesbar).
    pub fn wert(&self) -> Option<serde_json::Value> {
        if self.geloescht {
            return None;
        }
        serde_json::from_str(&self.inhalt).ok()
    }
}

/// Darf `neu` die Fassung `alt` desselben Eintrags ersetzen?
///
/// - Nur, wenn sie juenger ist (bei gleicher Zeit entscheidet die Unterschrift
///   -- alle Geraete kommen so auf dieselbe).
/// - Von derselben Person immer; von einer anderen nur als Loeschen, und nur
///   von der, die die Sache angelegt hat (`anlegerin`), oder einer
///   Eigentuemerin.
pub fn ersetzt(
    alt: &Eintrag,
    neu: &Eintrag,
    anlegerin: Option<&str>,
    eigentuemer: &[String],
) -> bool {
    if (neu.at.as_str(), neu.signatur.as_str()) <= (alt.at.as_str(), alt.signatur.as_str()) {
        return false;
    }
    if neu.personen_id == alt.personen_id {
        return true;
    }
    neu.geloescht
        && (anlegerin == Some(neu.personen_id.as_str()) || eigentuemer.contains(&neu.personen_id))
}

/// Die Personen-Ids der Eigentuemerinnen nach der Liste von jetzt.
pub fn eigentuemer(
    liste: &Mitgliederliste,
    person: &dyn Fn(&str) -> Option<Person>,
) -> Vec<String> {
    liste
        .mitglieder(person)
        .map(|m| {
            m.into_iter()
                .filter(|m| m.rolle == EIGENTUEMER)
                .map(|m| m.personen_id)
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mitglieder::MITGLIED;

    struct Lage {
        anna_tel: Identitaet,
        ben_tel: Identitaet,
        cara_tel: Identitaet,
        anna: Person,
        ben: Person,
        cara: Person,
        liste: Mitgliederliste,
    }

    fn lage() -> Lage {
        let anna_tel = Identitaet::erzeugen().unwrap();
        let ben_tel = Identitaet::erzeugen().unwrap();
        let cara_tel = Identitaet::erzeugen().unwrap();
        let anna = Person::neu(&anna_tel, "Telefon").unwrap();
        let ben = Person::neu(&ben_tel, "Telefon").unwrap();
        let cara = Person::neu(&cara_tel, "Telefon").unwrap();
        let mut liste = Mitgliederliste::gruenden(&anna_tel, &anna, "p1").unwrap();
        for (p, t) in [(&ben, &ben_tel), (&cara, &cara_tel)] {
            liste
                .eintragen(
                    &anna_tel,
                    "beitritt",
                    &p.personen_id,
                    "x",
                    MITGLIED,
                    &t.fingerabdruck,
                )
                .unwrap();
        }
        Lage {
            anna_tel,
            ben_tel,
            cara_tel,
            anna,
            ben,
            cara,
            liste,
        }
    }

    #[test]
    fn ein_schreiber_je_eintrag_loeschen_auch_anlegerin_und_eigentuemerin() {
        let l = lage();
        let keiner = |_: &str| None;
        let eigen = eigentuemer(&l.liste, &keiner);
        assert_eq!(eigen, vec![l.anna.personen_id.clone()]);

        // Ben legt eine Abstimmung an; Cara stimmt ab.
        let stimme = Eintrag::schreiben(
            &l.cara_tel,
            "p1",
            "s1",
            "u1",
            "umfrage",
            &l.cara.personen_id,
            Some(r#"{"art":"stimme"}"#),
        )
        .unwrap();
        assert!(stimme.gilt("p1", &l.liste, &keiner));
        std::thread::sleep(std::time::Duration::from_millis(5));

        // Ben aendert Caras Stimme: nein. Ben loescht sie (er hat angelegt): ja.
        let geaendert = Eintrag::schreiben(
            &l.ben_tel,
            "p1",
            "s1",
            "u1",
            "umfrage",
            &l.ben.personen_id,
            Some(r#"{"art":"stimme","x":1}"#),
        )
        .unwrap();
        assert!(!ersetzt(
            &stimme,
            &geaendert,
            Some(&l.ben.personen_id),
            &eigen
        ));
        let weg = Eintrag::schreiben(
            &l.ben_tel,
            "p1",
            "s1",
            "u1",
            "umfrage",
            &l.ben.personen_id,
            None,
        )
        .unwrap();
        assert!(ersetzt(&stimme, &weg, Some(&l.ben.personen_id), &eigen));
        // Waere Ben nicht Anleger: nein. Anna als Eigentuemerin: ja.
        assert!(!ersetzt(&stimme, &weg, Some(&l.anna.personen_id), &[]));
        let von_anna = Eintrag::schreiben(
            &l.anna_tel,
            "p1",
            "s1",
            "u1",
            "umfrage",
            &l.anna.personen_id,
            None,
        )
        .unwrap();
        assert!(ersetzt(&stimme, &von_anna, None, &eigen));

        // Cara selbst darf aendern -- aber nicht mit einer aelteren Fassung.
        let neuer = Eintrag::schreiben(
            &l.cara_tel,
            "p1",
            "s1",
            "u1",
            "umfrage",
            &l.cara.personen_id,
            Some(r#"{"art":"stimme","y":2}"#),
        )
        .unwrap();
        assert!(ersetzt(&stimme, &neuer, None, &[]));
        assert!(!ersetzt(&neuer, &stimme, None, &[]));
    }

    #[test]
    fn nur_echte_eintraege_von_mitgliedern() {
        let l = lage();
        let keiner = |_: &str| None;
        let fremd = Identitaet::erzeugen().unwrap();
        let e = Eintrag::schreiben(
            &l.ben_tel,
            "p1",
            "e1",
            "u1",
            "umfrage",
            &l.ben.personen_id,
            Some("{}"),
        )
        .unwrap();
        assert!(e.gilt("p1", &l.liste, &keiner));
        assert!(!e.gilt("p2", &l.liste, &keiner));
        let mut veraendert = e.clone();
        veraendert.inhalt = r#"{"boese":1}"#.into();
        assert!(!veraendert.gilt("p1", &l.liste, &keiner));
        // Im Namen von Anna von Bens Geraet: nein.
        let untergeschoben = Eintrag::schreiben(
            &l.ben_tel,
            "p1",
            "e2",
            "u1",
            "umfrage",
            &l.anna.personen_id,
            Some("{}"),
        )
        .unwrap();
        assert!(!untergeschoben.gilt("p1", &l.liste, &keiner));
        let von_fremd =
            Eintrag::schreiben(&fremd, "p1", "e3", "u1", "umfrage", "x", Some("{}")).unwrap();
        assert!(!von_fremd.gilt("p1", &l.liste, &keiner));
        let falsche_sorte = Eintrag::schreiben(
            &l.ben_tel,
            "p1",
            "e4",
            "u1",
            "kanban",
            &l.ben.personen_id,
            Some("{}"),
        )
        .unwrap();
        assert!(!falsche_sorte.gilt("p1", &l.liste, &keiner));
    }
}
