//! Der Chat eines lokalen Projekts -- vor Ort (Tiffy, 01.10.2026: erst nur
//! vor Ort; ueber Entfernung spaeter, etwa ueber Matrix).
//!
//! **Beim Senden geht eine Nachricht sofort an jedes Mitgliedsgeraet in der
//! Naehe** (`/openany/v1/projekt/chat`) -- dort erscheint sie gleich. Wer
//! gerade nicht da ist, bekommt sie beim naechsten Abgleich: Der Stand eines
//! Projekts traegt die Nachrichten mit.
//!
//! **Jede Nachricht ist unterschrieben.** Deshalb darf sie ueber Dritte
//! reisen: Ben bekommt Annas Nachricht auch von Carla, und kann trotzdem
//! pruefen, dass sie von Anna ist und unveraendert. Gilt nur, wer (je)
//! Mitglied war -- auch eine alte Nachricht eines Ausgetretenen bleibt
//! lesbar.

use crate::mitglieder::{Mitglied, Mitgliederliste};
use crate::person::{der, pruefen, unterschreiben, PersonFehler};
use crate::tls::fingerabdruck;
use crate::{Identitaet, Person};
use serde::{Deserialize, Serialize};

/// Die Art, unter der Nachrichten im Projekt liegen.
pub const CHAT: &str = "chat";

/// Wie viele Nachrichten ein Stand hoechstens mitbringt (die juengsten).
pub const IM_STAND: usize = 500;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Nachricht {
    pub id: String,
    pub projekt: String,
    pub personen_id: String,
    /// Der Name, unter dem die Person Mitglied ist.
    pub name: String,
    pub text: String,
    pub at: String,
    pub unterzeichner: String,
    pub zertifikat_pem: String,
    pub signatur: String,
}

fn nachricht(n: &Nachricht) -> Vec<u8> {
    let mut b = b"openany-chat-v1".to_vec();
    for teil in [
        &n.id,
        &n.projekt,
        &n.personen_id,
        &n.name,
        &n.text,
        &n.at,
        &n.unterzeichner,
    ] {
        b.push(0x1f);
        b.extend_from_slice(teil.as_bytes());
    }
    b
}

impl Nachricht {
    pub fn schreiben(
        ident: &Identitaet,
        projekt: &str,
        personen_id: &str,
        name: &str,
        text: &str,
    ) -> Result<Self, PersonFehler> {
        let mut n = Self {
            id: uuid::Uuid::new_v4().to_string(),
            projekt: projekt.to_string(),
            personen_id: personen_id.to_string(),
            name: name.to_string(),
            text: text.to_string(),
            at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            unterzeichner: ident.fingerabdruck.clone(),
            zertifikat_pem: ident.zertifikat_pem.clone(),
            signatur: String::new(),
        };
        n.signatur = unterschreiben(ident, &nachricht(&n))?;
        Ok(n)
    }

    /// Echt, fuer dieses Projekt, und von einem Geraet einer Person, die
    /// (je) Mitglied war.
    pub fn gilt(
        &self,
        projekt: &str,
        liste: &Mitgliederliste,
        person: &dyn Fn(&str) -> Option<Person>,
    ) -> bool {
        if self.projekt != projekt || self.text.trim().is_empty() || self.text.len() > 20_000 {
            return false;
        }
        let echt = der(&self.zertifikat_pem)
            .is_some_and(|d| fingerabdruck(&d) == self.unterzeichner)
            && pruefen(&self.zertifikat_pem, &nachricht(self), &self.signatur);
        echt && liste.eintraege.iter().any(|e| {
            (e.art == "gruendung" || e.art == "beitritt")
                && e.personen_id == self.personen_id
                && crate::freigaben::geraet_von(
                    &Mitglied {
                        personen_id: e.personen_id.clone(),
                        name: e.name.clone(),
                        rolle: e.rolle.clone(),
                        geraet: e.geraet.clone(),
                    },
                    &self.unterzeichner,
                    person,
                )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mitglieder::MITGLIED;

    #[test]
    fn nur_echte_nachrichten_von_mitgliedern_gelten() {
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
        let keiner = |_: &str| None;

        let hallo =
            Nachricht::schreiben(&ben_tel, "p1", &ben.personen_id, "Ben", "Hallo!").unwrap();
        assert!(hallo.gilt("p1", &liste, &keiner));
        assert!(!hallo.gilt("p2", &liste, &keiner));

        // Veraendert: gilt nicht mehr.
        let mut falsch = hallo.clone();
        falsch.text = "Tschuess!".into();
        assert!(!falsch.gilt("p1", &liste, &keiner));

        // Im Namen von Anna, aber von Bens Geraet: nein.
        let untergeschoben =
            Nachricht::schreiben(&ben_tel, "p1", &anna.personen_id, "Anna", "Ich bin Anna")
                .unwrap();
        assert!(!untergeschoben.gilt("p1", &liste, &keiner));

        // Kein Mitglied: nein.
        let von_fremd = Nachricht::schreiben(&fremd, "p1", "x", "X", "Spam").unwrap();
        assert!(!von_fremd.gilt("p1", &liste, &keiner));
    }
}
