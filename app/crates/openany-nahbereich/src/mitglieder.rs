//! Die Mitgliederliste eines lokalen Projekts
//! (docs/konzept-lokale-mitgliedschaften.md, §5).
//!
//! **Ohne Server prueft jedes Geraet selbst.** Sonst koennte ein Mitglied sich
//! zum Eigentuemer machen, und alle anderen Geraete glaubten es. Deshalb ist
//! die Liste kein Zustand, sondern eine Kette unterschriebener Eintraege
//! (Gruendung, Beitritt, Rolle, Austritt), und jedes Geraet rechnet den
//! Zustand selbst aus -- und prueft dabei jede Unterschrift.
//!
//! **Die Regeln:**
//! - Der erste Eintrag ist die Gruendung: Die gruendende Person ist
//!   Eigentuemerin, unterschrieben von ihrem Geraet.
//! - Jeder weitere Eintrag nennt den Abdruck seines Vorgaengers. Ein
//!   Eintrag, der herausfaellt oder umsortiert wird, bricht die Kette.
//! - Aendern darf nur ein Geraet einer Eigentuemer-Person -- "nur die
//!   Eigentuemerin laedt ein" (§11). Austreten darf jede Person selbst.
//! - Die letzte Eigentuemerin kann nicht gehen und nicht herabgestuft werden.
//!
//! **Welche Geraete zu einer Person gehoeren**, sagt ihre Person (person.rs),
//! gerechnet ab dem Geraet, das der Eintrag als ihren Anker nennt (bei einem
//! Beitritt: das Geraet, das vor Ort dabei war). Kennt dieses Geraet die
//! Person nicht, gilt nur der Anker selbst.
//!
//! **Zwei Eigentuemer, die gleichzeitig aendern**, ergaeben zwei Ketten. Das
//! ist heute ausgeschlossen (eine Eigentuemerin), und wird es geloest, dann
//! hier.

use crate::person::{der, pruefen, unterschreiben, PersonFehler};
use crate::tls::fingerabdruck;
use crate::{Identitaet, Person};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const EIGENTUEMER: &str = "owner";
pub const MITGLIED: &str = "member";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Mitgliedereintrag {
    /// `gruendung`, `beitritt`, `rolle`, `austritt`.
    pub art: String,
    pub personen_id: String,
    pub name: String,
    /// `owner` oder `member`; leer beim Austritt.
    pub rolle: String,
    /// Der Anker dieser Person: ein Geraet, das sicher zu ihr gehoert.
    pub geraet: String,
    pub at: String,
    /// Abdruck (SHA-256) der Unterschrift des Vorgaengers; leer beim ersten.
    pub vorher: String,
    pub unterzeichner: String,
    /// Das Zertifikat des Unterzeichners -- so prueft man ohne Nachschlagen.
    pub zertifikat_pem: String,
    pub signatur: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Mitgliederliste {
    pub projekt: String,
    pub eintraege: Vec<Mitgliedereintrag>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Mitglied {
    pub personen_id: String,
    pub name: String,
    pub rolle: String,
    pub geraet: String,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum MitgliedFehler {
    #[error("Entry {0}: {1}")]
    Ungueltig(usize, &'static str),
    #[error("{0}")]
    Schluessel(String),
}

impl From<PersonFehler> for MitgliedFehler {
    fn from(e: PersonFehler) -> Self {
        Self::Schluessel(e.to_string())
    }
}

fn nachricht(projekt: &str, e: &Mitgliedereintrag) -> Vec<u8> {
    let mut n = b"openany-mitglied-v1".to_vec();
    for teil in [
        projekt,
        &e.art,
        &e.personen_id,
        &e.name,
        &e.rolle,
        &e.geraet,
        &e.at,
        &e.vorher,
        &e.unterzeichner,
    ] {
        n.push(0x1f);
        n.extend_from_slice(teil.as_bytes());
    }
    n
}

fn abdruck(signatur: &str) -> String {
    Sha256::digest(signatur.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

impl Mitgliederliste {
    /// Ein neues Projekt: `person` ist Eigentuemerin, `ident` ihr Geraet.
    pub fn gruenden(
        ident: &Identitaet,
        person: &Person,
        projekt: &str,
    ) -> Result<Self, MitgliedFehler> {
        let mut liste = Self {
            projekt: projekt.to_string(),
            eintraege: Vec::new(),
        };
        liste.eintragen(
            ident,
            "gruendung",
            &person.personen_id,
            &person.name,
            EIGENTUEMER,
            &ident.fingerabdruck,
        )?;
        Ok(liste)
    }

    /// Einen Eintrag anhaengen, unterschrieben von `ident`. Ob er gilt,
    /// prueft [`Self::mitglieder`] -- hier wird nur gebaut.
    pub fn eintragen(
        &mut self,
        ident: &Identitaet,
        art: &str,
        personen_id: &str,
        name: &str,
        rolle: &str,
        geraet: &str,
    ) -> Result<(), MitgliedFehler> {
        let mut e = Mitgliedereintrag {
            art: art.to_string(),
            personen_id: personen_id.to_string(),
            name: name.to_string(),
            rolle: rolle.to_string(),
            geraet: geraet.to_string(),
            at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            vorher: self
                .eintraege
                .last()
                .map(|v| abdruck(&v.signatur))
                .unwrap_or_default(),
            unterzeichner: ident.fingerabdruck.clone(),
            zertifikat_pem: ident.zertifikat_pem.clone(),
            signatur: String::new(),
        };
        e.signatur = unterschreiben(ident, &nachricht(&self.projekt, &e))?;
        self.eintraege.push(e);
        Ok(())
    }

    /// Die Mitglieder, wie die Kette sie ergibt -- oder warum sie nicht gilt.
    ///
    /// `person` liefert die bekannte Person zu einer Personen-Id (fuer ihre
    /// Geraete jenseits des Ankers); `None` ist erlaubt.
    pub fn mitglieder(
        &self,
        person: &dyn Fn(&str) -> Option<Person>,
    ) -> Result<Vec<Mitglied>, MitgliedFehler> {
        let mut mitglieder: Vec<Mitglied> = Vec::new();
        let geraet_von = |m: &Mitglied, fp: &str| -> bool {
            fp == m.geraet
                || person(&m.personen_id).is_some_and(|p| p.gueltige(&[&m.geraet]).contains_key(fp))
        };

        for (i, e) in self.eintraege.iter().enumerate() {
            let falsch = |grund| Err(MitgliedFehler::Ungueltig(i, grund));

            // Die Unterschrift: echt, und vom genannten Unterzeichner.
            let passt =
                der(&e.zertifikat_pem).is_some_and(|d| fingerabdruck(&d) == e.unterzeichner);
            if !passt || !pruefen(&e.zertifikat_pem, &nachricht(&self.projekt, e), &e.signatur) {
                return falsch("invalid signature");
            }
            // Die Kette.
            let erwartet = if i == 0 {
                String::new()
            } else {
                abdruck(&self.eintraege[i - 1].signatur)
            };
            if e.vorher != erwartet {
                return falsch("chain broken");
            }

            if i == 0 {
                if e.art != "gruendung" || e.rolle != EIGENTUEMER || e.geraet != e.unterzeichner {
                    return falsch("no valid founding");
                }
                mitglieder.push(Mitglied {
                    personen_id: e.personen_id.clone(),
                    name: e.name.clone(),
                    rolle: EIGENTUEMER.into(),
                    geraet: e.geraet.clone(),
                });
                continue;
            }

            let von_eigentuemer = mitglieder
                .iter()
                .any(|m| m.rolle == EIGENTUEMER && geraet_von(m, &e.unterzeichner));
            let stelle = mitglieder
                .iter()
                .position(|m| m.personen_id == e.personen_id);
            let eigentuemer = mitglieder.iter().filter(|m| m.rolle == EIGENTUEMER).count();

            match e.art.as_str() {
                "beitritt" => {
                    if !von_eigentuemer {
                        return falsch("only the owner can invite");
                    }
                    if ![EIGENTUEMER, MITGLIED].contains(&e.rolle.as_str()) {
                        return falsch("unknown role");
                    }
                    let neu = Mitglied {
                        personen_id: e.personen_id.clone(),
                        name: e.name.clone(),
                        rolle: e.rolle.clone(),
                        geraet: e.geraet.clone(),
                    };
                    match stelle {
                        Some(s) => mitglieder[s] = neu,
                        None => mitglieder.push(neu),
                    }
                }
                "rolle" => {
                    let Some(s) = stelle else {
                        return falsch("not a member");
                    };
                    if !von_eigentuemer {
                        return falsch("only the owner can change roles");
                    }
                    if ![EIGENTUEMER, MITGLIED].contains(&e.rolle.as_str()) {
                        return falsch("unknown role");
                    }
                    if mitglieder[s].rolle == EIGENTUEMER
                        && e.rolle != EIGENTUEMER
                        && eigentuemer == 1
                    {
                        return falsch("the last owner stays");
                    }
                    mitglieder[s].rolle = e.rolle.clone();
                }
                "austritt" => {
                    let Some(s) = stelle else {
                        return falsch("not a member");
                    };
                    if !von_eigentuemer && !geraet_von(&mitglieder[s], &e.unterzeichner) {
                        return falsch("only the person leaving or the owner may record leaving");
                    }
                    if mitglieder[s].rolle == EIGENTUEMER && eigentuemer == 1 {
                        return falsch("the last owner stays");
                    }
                    mitglieder.remove(s);
                }
                _ => return falsch("unknown kind"),
            }
        }

        if mitglieder.is_empty() {
            return Err(MitgliedFehler::Ungueltig(0, "leer"));
        }
        Ok(mitglieder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn geraet() -> Identitaet {
        Identitaet::erzeugen().unwrap()
    }

    fn person(ident: &Identitaet, name: &str) -> Person {
        let mut p = Person::neu(ident, "Geraet").unwrap();
        p.name = name.into();
        p
    }

    #[test]
    fn gruenden_einladen_austreten() {
        let anna_tel = geraet();
        let ben_tel = geraet();
        let anna = person(&anna_tel, "Anna");
        let ben = person(&ben_tel, "Ben");

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
        assert_eq!(m.len(), 2);
        assert_eq!(
            (m[0].name.as_str(), m[0].rolle.as_str()),
            ("Anna", EIGENTUEMER)
        );
        assert_eq!((m[1].name.as_str(), m[1].rolle.as_str()), ("Ben", MITGLIED));

        // Ben geht selbst.
        liste
            .eintragen(
                &ben_tel,
                "austritt",
                &ben.personen_id,
                "Ben",
                "",
                &ben_tel.fingerabdruck,
            )
            .unwrap();
        assert_eq!(liste.mitglieder(&|_| None).unwrap().len(), 1);
    }

    #[test]
    fn ein_mitglied_laedt_nicht_ein_und_macht_sich_nicht_zur_eigentuemerin() {
        let anna_tel = geraet();
        let ben_tel = geraet();
        let carla_tel = geraet();
        let anna = person(&anna_tel, "Anna");
        let ben = person(&ben_tel, "Ben");

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

        let mut einladung = liste.clone();
        einladung
            .eintragen(
                &ben_tel,
                "beitritt",
                "carla",
                "Carla",
                MITGLIED,
                &carla_tel.fingerabdruck,
            )
            .unwrap();
        assert_eq!(
            einladung.mitglieder(&|_| None),
            Err(MitgliedFehler::Ungueltig(2, "only the owner can invite"))
        );

        let mut putsch = liste.clone();
        putsch
            .eintragen(
                &ben_tel,
                "rolle",
                &ben.personen_id,
                "Ben",
                EIGENTUEMER,
                &ben_tel.fingerabdruck,
            )
            .unwrap();
        assert!(putsch.mitglieder(&|_| None).is_err());
    }

    #[test]
    fn faelschung_und_luecke_brechen_die_kette() {
        let anna_tel = geraet();
        let ben_tel = geraet();
        let anna = person(&anna_tel, "Anna");
        let mut liste = Mitgliederliste::gruenden(&anna_tel, &anna, "p1").unwrap();
        liste
            .eintragen(
                &anna_tel,
                "beitritt",
                "ben",
                "Ben",
                MITGLIED,
                &ben_tel.fingerabdruck,
            )
            .unwrap();
        liste
            .eintragen(
                &anna_tel,
                "rolle",
                "ben",
                "Ben",
                EIGENTUEMER,
                &ben_tel.fingerabdruck,
            )
            .unwrap();

        // Rolle im Nachhinein geaendert: Unterschrift passt nicht mehr.
        let mut gefaelscht = liste.clone();
        gefaelscht.eintraege[1].rolle = EIGENTUEMER.into();
        assert_eq!(
            gefaelscht.mitglieder(&|_| None),
            Err(MitgliedFehler::Ungueltig(1, "invalid signature"))
        );

        // Ein Eintrag herausgenommen: Die Kette bricht.
        let mut luecke = liste.clone();
        luecke.eintraege.remove(1);
        assert_eq!(
            luecke.mitglieder(&|_| None),
            Err(MitgliedFehler::Ungueltig(1, "chain broken"))
        );

        // Fuer ein anderes Projekt unterschrieben: gilt hier nicht.
        let mut anderes = liste.clone();
        anderes.projekt = "p2".into();
        assert!(anderes.mitglieder(&|_| None).is_err());
    }

    #[test]
    fn ein_zweites_geraet_der_eigentuemerin_darf_einladen() {
        let anna_tel = geraet();
        let anna_tab = geraet();
        let ben_tel = geraet();
        let mut anna = person(&anna_tel, "Anna");
        // Gepaart: beide buergen fuereinander.
        anna.eintraege.push(
            crate::Geraeteeintrag::buergen(&anna_tel, &anna_tab.zertifikat_pem, "Tablet").unwrap(),
        );

        let mut liste = Mitgliederliste::gruenden(&anna_tel, &anna, "p1").unwrap();
        liste
            .eintragen(
                &anna_tab,
                "beitritt",
                "ben",
                "Ben",
                MITGLIED,
                &ben_tel.fingerabdruck,
            )
            .unwrap();

        // Ohne Annas Person kennt man nur ihr Telefon -- dann gilt es nicht.
        assert!(liste.mitglieder(&|_| None).is_err());
        let a = anna.clone();
        let kennt = move |id: &str| (id == a.personen_id).then(|| a.clone());
        assert_eq!(liste.mitglieder(&kennt).unwrap().len(), 2);

        // Die letzte Eigentuemerin bleibt.
        let mut weg = liste.clone();
        weg.eintragen(
            &anna_tel,
            "austritt",
            &anna.personen_id,
            "Anna",
            "",
            &anna_tel.fingerabdruck,
        )
        .unwrap();
        assert!(weg.mitglieder(&kennt).is_err());
    }
}
