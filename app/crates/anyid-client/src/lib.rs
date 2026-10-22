//! Die Anmeldung des Programms: Kopplung und Ticket.
//!
//! **Das Problem.** Ein Programm auf einem fremden Geraet kann kein Geheimnis
//! halten -- jeder, der das Geraet in die Hand nimmt, sieht hinein. Der Weg,
//! den die Anwendungen im Browser gehen (Weiterleitung, Ticket, Cookie),
//! setzt aber genau das voraus.
//!
//! **Der Weg stattdessen.** Das Programm faengt an, der Mensch bestaetigt im
//! Browser, das Programm holt ab. Was es dabei schuetzt, ist kein Geheimnis,
//! sondern zweierlei: die Bestaetigung eines angemeldeten Menschen -- und der
//! `verifier`, den das Programm bei sich behaelt und aus dem sich der zuvor
//! gesendete `challenge` nicht zurueckrechnen laesst.
//!
//! **Was das Geraetetoken kann.** Genau eines: fuer den Menschen, an den es
//! gebunden ist, ein Ticket fuer *eine* Anwendung erbitten. Es kann kein
//! Passwort aendern und nichts, wofuer ein zweiter Faktor verlangt wird --
//! Tickets daraus tragen `zweiter_faktor: false`, weil ein Token, das seit
//! Wochen auf einem Tablet liegt, keine Anmeldung ist.
//!
//! **Ein Geraet schaltet ein Geraet frei** (anyid seit Z4). Mit Anmeldenamen
//! begonnen, geht die Kopplung als Anfrage an die Geraete dieses Kontos statt
//! an einen Browser. Beide Seiten tauschen ueber einen Briefkasten bei anyid
//! Nachrichten aus, die anyid nicht lesen kann; das alte Geraet bestaetigt
//! danach mit seinem Geraetetoken.

mod client;
mod kopplung;
mod speicher;

pub use client::{AnyidClient, AnyidError};
pub use kopplung::{Kopplungsstart, Verifier};
pub use speicher::{Dateispeicher, Tokenspeicher};

use serde::Deserialize;

/// Was anyid auf den Beginn einer Kopplung antwortet.
#[derive(Debug, Clone, Deserialize)]
pub struct Kopplungscodes {
    /// Der kurze Code, den der Mensch abtippt. Er wird angezeigt, nicht
    /// verschickt - abgeholt wird nie mit ihm.
    pub code: String,
    /// Der lange Code zum Abholen. Er bleibt im Programm.
    pub geraetecode: String,
    pub ablauf_in: u64,
    /// Wie oft das Programm nachfragen darf, in Sekunden.
    pub intervall: u64,
    /// Nur bei einer Anfrage an andere Geraete.
    #[serde(default)]
    pub anfrage: Option<String>,
}

/// Eine offene Anfrage an das Konto dieses Geraets: Ein neues Geraet will
/// sich anmelden (`anmelden`) oder braucht nur den Kontoschluessel
/// (`schluessel`).
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Anfrage {
    pub anfrage: String,
    pub geraet: String,
    pub art: String,
    pub seit: String,
}

/// Der Ausgang eines Abhol-Versuchs.
#[derive(Debug, Clone)]
pub enum Abholung {
    /// Der Mensch hat noch nicht bestaetigt. Nach `intervall` erneut fragen.
    Wartet { intervall: u64 },
    /// Das Token - genau einmal zu sehen.
    Fertig { token: String, geraet: Geraet },
    /// Abgelaufen, erfunden, falscher Verifier oder schon abgeholt. Die vier
    /// sind von aussen nicht zu unterscheiden, und das ist Absicht.
    Ungueltig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Geraet {
    pub id: String,
    pub name: String,
    pub anwendung: String,
}

/// Ein Ticket, wie es auch eine Anmeldung im Browser erzeugt. Die Anwendung
/// loest es auf ihrem gewohnten Weg ein.
#[derive(Debug, Clone, Deserialize)]
pub struct Ticket {
    pub ticket: String,
    pub anwendung: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn die_anfrage_ist_nur_bei_einer_anfrage_dabei() {
        let ohne: Kopplungscodes = serde_json::from_str(
            r#"{"code":"ABCD-EFGH","geraetecode":"x","ablauf_in":600,"intervall":5}"#,
        )
        .unwrap();
        assert_eq!(ohne.anfrage, None);

        let mit: Kopplungscodes = serde_json::from_str(
            r#"{"code":"ABCD-EFGH","geraetecode":"x","ablauf_in":600,"intervall":5,"anfrage":"a1"}"#,
        )
        .unwrap();
        assert_eq!(mit.anfrage.as_deref(), Some("a1"));
    }

    #[test]
    fn eine_anfrage_liest_sich_wie_anyid_sie_schickt() {
        let a: Anfrage = serde_json::from_str(
            r#"{"anfrage":"a1","geraet":"Telefon","art":"anmelden","seit":"2026-10-21T10:00:00+00:00"}"#,
        )
        .unwrap();
        assert_eq!(a.art, "anmelden");
    }
}
