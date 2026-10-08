//! Einladung vor Ort (docs/konzept-lokale-mitgliedschaften.md, §4).
//!
//! **Derselbe Ablauf wie beim Paaren, aber nie dieselbe Frage.** Beide
//! Geraete zeigen 6 Ziffern, beide bestaetigen. In die Ziffern gehen ausser
//! den Fingerabdruecken und zwei Zufallszahlen auch die Projekt-uuid und ein
//! eigenes Vorwort ein: Ein Code aus einer Paarung passt nie zu einer
//! Einladung, und eine Einladung in Projekt A nie zu Projekt B. Der Dialog
//! sagt ausdruecklich "laedt dich in das Projekt ... ein" -- ein
//! versehentliches "Passt" gaebe sonst ein ganzes Geraet frei, wo nur ein
//! Projekt gemeint war.
//!
//! **Der Ablauf** (A laedt ein, B wird eingeladen):
//! 1. A fragt B an (`/openany/v1/einladung`): Projekt, Name, Zufall.
//! 2. Beide zeigen den Code; beide bestaetigen am Geraet.
//! 3. A fragt nach (`/einladung/stand`); hat B bestaetigt, kommt B's Person
//!    mit.
//! 4. A unterschreibt den Beitritt (mitglieder.rs) und schickt die Liste samt
//!    der eigenen Person (`/einladung/aufnahme`).
//! 5. B prueft die Liste SELBST -- gilt sie, steht B darin, kam sie von dem
//!    Geraet, das B bestaetigt hat? -- und legt das Projekt an.

use crate::mitglieder::{Mitglied, MitgliedFehler, Mitgliederliste, MITGLIED};
use crate::paaren::Rolle;
use crate::Person;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// KONTAKT BESTAETIGEN (Tiffy, 06.10.2026): derselbe Ablauf, aber statt
/// eines Projekts steht hier dieses Kennwort. Am Ende tauschen beide ihre
/// Person, und jede Seite merkt sich die andere als bestaetigt -- fuer
/// Direktnachrichten vor Ort (direkt.rs). Weil das Kennwort in den Code
/// eingeht, passt ein Kontaktcode nie zu einer Projekteinladung.
pub const KONTAKT: &str = "openany-kontakt";

/// Wie lange eine offene Einladung gilt.
pub const EINLADUNG_GILT: Duration = Duration::from_secs(180);

/// Der Vergleichscode einer Einladung -- nie gleich einem Paarungscode.
pub fn einladungscode(
    fp_a: &str,
    fp_b: &str,
    zufall_anfrage: &str,
    zufall_antwort: &str,
    projekt: &str,
) -> String {
    let (klein, gross) = if fp_a <= fp_b {
        (fp_a, fp_b)
    } else {
        (fp_b, fp_a)
    };
    let mut h = Sha256::new();
    for teil in [
        "openany-einladung-v1",
        klein,
        gross,
        zufall_anfrage,
        zufall_antwort,
        projekt,
    ] {
        h.update(teil.as_bytes());
        h.update([0x1f]);
    }
    let d = h.finalize();
    let zahl = u32::from_be_bytes([d[0], d[1], d[2], d[3]]) % 1_000_000;
    format!("{zahl:06}")
}

/// Eine offene Einladung, auf welcher Seite auch immer.
#[derive(Debug, Clone, Serialize)]
pub struct OffeneEinladung {
    /// Das andere Geraet.
    pub fingerabdruck: String,
    /// Wie das andere Geraet heisst (A sieht B's Geraet, B sieht A's).
    pub name: String,
    pub projekt: String,
    pub projekt_name: String,
    /// Wer einlaedt (der Personenname der Eigentuemerin) -- fuer B's Dialog.
    pub von: String,
    pub code: String,
    /// `anfragend` = hier wird eingeladen; `gefragt` = hier wird man eingeladen.
    pub rolle: Rolle,
    pub hier_bestaetigt: bool,
    /// A: B hat bestaetigt (und seine Person geschickt).
    pub dort_bestaetigt: bool,
    /// A: B's Person, sobald B bestaetigt hat.
    #[serde(skip)]
    pub person: Option<Person>,
    #[serde(skip)]
    seit: Instant,
}

#[derive(Debug, Default)]
pub struct Einladungen {
    offen: HashMap<String, OffeneEinladung>,
}

pub type GemeinsameEinladungen = Arc<Mutex<Einladungen>>;

impl Einladungen {
    #[allow(clippy::too_many_arguments)]
    fn neu(
        &mut self,
        fp_anderer: &str,
        name: &str,
        projekt: &str,
        projekt_name: &str,
        von: &str,
        code: String,
        rolle: Rolle,
    ) -> OffeneEinladung {
        let e = OffeneEinladung {
            fingerabdruck: fp_anderer.to_string(),
            name: name.to_string(),
            projekt: projekt.to_string(),
            projekt_name: projekt_name.to_string(),
            von: von.to_string(),
            code,
            rolle,
            hier_bestaetigt: false,
            dort_bestaetigt: false,
            person: None,
            seit: Instant::now(),
        };
        self.offen.insert(fp_anderer.to_string(), e.clone());
        e
    }

    /// B: Eine Einladung kam herein.
    #[allow(clippy::too_many_arguments)]
    pub fn eingang(
        &mut self,
        fp_anderer: &str,
        name: &str,
        fp_ich: &str,
        anfrage: &Anfrage,
        zufall_antwort: &str,
    ) -> OffeneEinladung {
        let code = einladungscode(
            fp_anderer,
            fp_ich,
            &anfrage.zufall,
            zufall_antwort,
            &anfrage.projekt,
        );
        self.neu(
            fp_anderer,
            name,
            &anfrage.projekt,
            &anfrage.projekt_name,
            &anfrage.von,
            code,
            Rolle::Gefragt,
        )
    }

    /// A: B hat geantwortet.
    #[allow(clippy::too_many_arguments)]
    pub fn ausgang(
        &mut self,
        fp_anderer: &str,
        name: &str,
        fp_ich: &str,
        anfrage: &Anfrage,
        zufall_antwort: &str,
    ) -> OffeneEinladung {
        let code = einladungscode(
            fp_ich,
            fp_anderer,
            &anfrage.zufall,
            zufall_antwort,
            &anfrage.projekt,
        );
        self.neu(
            fp_anderer,
            name,
            &anfrage.projekt,
            &anfrage.projekt_name,
            &anfrage.von,
            code,
            Rolle::Anfragend,
        )
    }

    pub fn hier_bestaetigen(&mut self, fp: &str) -> Option<&OffeneEinladung> {
        self.aufraeumen();
        let e = self.offen.get_mut(fp)?;
        e.hier_bestaetigt = true;
        Some(e)
    }

    /// A: B hat bestaetigt und seine Person geschickt.
    pub fn dort_bestaetigt(&mut self, fp: &str, person: Person) -> Option<&OffeneEinladung> {
        self.aufraeumen();
        let e = self.offen.get_mut(fp)?;
        e.dort_bestaetigt = true;
        e.person = Some(person);
        Some(e)
    }

    pub fn get(&mut self, fp: &str) -> Option<&OffeneEinladung> {
        self.aufraeumen();
        self.offen.get(fp)
    }

    pub fn alle(&mut self) -> Vec<OffeneEinladung> {
        self.aufraeumen();
        self.offen.values().cloned().collect()
    }

    pub fn entfernen(&mut self, fp: &str) -> Option<OffeneEinladung> {
        self.offen.remove(fp)
    }

    fn aufraeumen(&mut self) {
        self.offen.retain(|_, e| e.seit.elapsed() < EINLADUNG_GILT);
    }
}

/* ── Was ueber die Leitung geht ──────────────────────────────────────── */

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anfrage {
    pub projekt: String,
    pub projekt_name: String,
    /// Der Personenname der Einladenden.
    pub von: String,
    /// Der Name des einladenden Geraets.
    pub geraet: String,
    pub zufall: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Antwort {
    pub geraet: String,
    pub zufall: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Stand {
    pub bestaetigt: bool,
    /// B's Person -- erst, wenn B bestaetigt hat.
    pub person: Option<Person>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aufnahme {
    pub projekt_name: String,
    pub mitgliederliste: Mitgliederliste,
    /// Die Person der Einladenden -- damit B ihre weiteren Geraete erkennt.
    pub eigentuemer: Person,
}

/// B prueft, was A zur Aufnahme schickt -- bevor irgendetwas gespeichert
/// wird. Gibt die Mitglieder zurueck.
///
/// - Die Liste gehoert zu genau dem Projekt, zu dem B "Passt" gesagt hat.
/// - Sie gilt (jede Unterschrift, die Kette), mit der Person der
///   Einladenden fuer deren Geraete.
/// - Der juengste Eintrag ist B's Beitritt, als Mitglied, mit B's Geraet als
///   Anker, und unterschrieben von dem Geraet, mit dem B gerade spricht.
pub fn aufnahme_pruefen(
    aufnahme: &Aufnahme,
    einladung: &OffeneEinladung,
    ich: &Person,
    fp_ich: &str,
) -> Result<Vec<Mitglied>, MitgliedFehler> {
    let liste = &aufnahme.mitgliederliste;
    let falsch = |grund| Err(MitgliedFehler::Ungueltig(liste.eintraege.len(), grund));
    if liste.projekt != einladung.projekt {
        return falsch("a different project");
    }
    if einladung.rolle != Rolle::Gefragt || !einladung.hier_bestaetigt {
        return falsch("not confirmed here");
    }
    let eigentuemer = aufnahme.eigentuemer.clone();
    let kennt = move |id: &str| (id == eigentuemer.personen_id).then(|| eigentuemer.clone());
    let mitglieder = liste.mitglieder(&kennt)?;
    let Some(letzter) = liste.eintraege.last() else {
        return falsch("leer");
    };
    if letzter.art != "beitritt"
        || letzter.personen_id != ich.personen_id
        || letzter.geraet != fp_ich
        || letzter.rolle != MITGLIED
        || letzter.unterzeichner != einladung.fingerabdruck
    {
        return falsch("not this joining");
    }
    Ok(mitglieder)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paaren::vergleichscode;
    use crate::Identitaet;

    #[test]
    fn ein_einladungscode_ist_nie_ein_paarungscode_und_haengt_am_projekt() {
        let a = einladungscode("AAAA", "BBBB", "za", "zb", "p1");
        assert_eq!(a, einladungscode("BBBB", "AAAA", "za", "zb", "p1"));
        assert_ne!(a, einladungscode("AAAA", "BBBB", "za", "zb", "p2"));
        assert_ne!(a, vergleichscode("AAAA", "BBBB", "za", "zb"));
    }

    #[test]
    fn b_nimmt_nur_den_eigenen_beitritt_von_der_einladenden_an() {
        let anna_tel = Identitaet::erzeugen().unwrap();
        let ben_tel = Identitaet::erzeugen().unwrap();
        let mut anna = Person::neu(&anna_tel, "Telefon").unwrap();
        anna.name = "Anna".into();
        let ben = Person::neu(&ben_tel, "Telefon").unwrap();

        let anfrage = Anfrage {
            projekt: "p1".into(),
            projekt_name: "Garten".into(),
            von: "Anna".into(),
            geraet: "Telefon".into(),
            zufall: "za".into(),
        };
        let mut bei_ben = Einladungen::default();
        bei_ben.eingang(
            &anna_tel.fingerabdruck,
            "Telefon",
            &ben_tel.fingerabdruck,
            &anfrage,
            "zb",
        );

        let mut liste = Mitgliederliste::gruenden(&anna_tel, &anna, "p1").unwrap();
        liste
            .eintragen(
                &anna_tel,
                "beitritt",
                &ben.personen_id,
                "",
                MITGLIED,
                &ben_tel.fingerabdruck,
            )
            .unwrap();
        let aufnahme = Aufnahme {
            projekt_name: "Garten".into(),
            mitgliederliste: liste.clone(),
            eigentuemer: anna.clone(),
        };

        // Noch nicht bestaetigt: abgewiesen.
        let e = bei_ben.get(&anna_tel.fingerabdruck).unwrap().clone();
        assert!(aufnahme_pruefen(&aufnahme, &e, &ben, &ben_tel.fingerabdruck).is_err());

        let e = bei_ben
            .hier_bestaetigen(&anna_tel.fingerabdruck)
            .unwrap()
            .clone();
        let m = aufnahme_pruefen(&aufnahme, &e, &ben, &ben_tel.fingerabdruck).unwrap();
        assert_eq!(m.len(), 2);

        // Ein anderes Projekt, oder ein Beitritt fuer jemand anderen: nein.
        let mut anders = aufnahme.clone();
        anders.mitgliederliste = Mitgliederliste::gruenden(&anna_tel, &anna, "p2").unwrap();
        assert!(aufnahme_pruefen(&anders, &e, &ben, &ben_tel.fingerabdruck).is_err());
        let carla_tel = Identitaet::erzeugen().unwrap();
        let mut fremd = Mitgliederliste::gruenden(&anna_tel, &anna, "p1").unwrap();
        fremd
            .eintragen(
                &anna_tel,
                "beitritt",
                "carla",
                "",
                MITGLIED,
                &carla_tel.fingerabdruck,
            )
            .unwrap();
        let fremd = Aufnahme {
            mitgliederliste: fremd,
            ..aufnahme.clone()
        };
        assert!(aufnahme_pruefen(&fremd, &e, &ben, &ben_tel.fingerabdruck).is_err());
    }
}
