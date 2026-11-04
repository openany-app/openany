//! Der Chat eines lokalen Projekts -- vor Ort.
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

/* ── Anheften, Loesen, Loeschen (E4, docs/plan-projekte-eine-art.md) ─────── */

/// Die Art, unter der Aktionen im Projekt liegen.
pub const AKTION: &str = "chat_aktion";

/// Was eine Aktion tun kann.
pub const ANHEFTEN: &str = "anheften";
pub const LOESEN: &str = "loesen";
pub const LOESCHEN: &str = "loeschen";

/// Eine Aktion an einer Nachricht -- wie eine Nachricht vom Geraet
/// unterschrieben, und deshalb darf sie wie eine Nachricht ueber Dritte und
/// ueber openany.de reisen.
///
/// **Die Regeln wie auf dem Server:** Anheften und Loesen darf jedes
/// Mitglied, Loeschen die Person, die geschrieben hat, oder eine
/// Eigentuemerin. Geprueft wird beim Anzeigen ([`ansicht`]), denn ob jemand
/// Eigentuemerin ist, sagt die Liste von jetzt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Aktion {
    pub id: String,
    pub projekt: String,
    /// Die Nachricht, um die es geht.
    pub nachricht: String,
    /// [`ANHEFTEN`], [`LOESEN`] oder [`LOESCHEN`].
    pub art: String,
    pub personen_id: String,
    pub at: String,
    pub unterzeichner: String,
    pub zertifikat_pem: String,
    pub signatur: String,
}

fn aktion_text(a: &Aktion) -> Vec<u8> {
    let mut b = b"openany-chat-aktion-v1".to_vec();
    for teil in [
        &a.id,
        &a.projekt,
        &a.nachricht,
        &a.art,
        &a.personen_id,
        &a.at,
        &a.unterzeichner,
    ] {
        b.push(0x1f);
        b.extend_from_slice(teil.as_bytes());
    }
    b
}

/// Kommt das Geraet `fp` von einer Person, die (je) Mitglied war?
pub(crate) fn von_mitglied(
    liste: &Mitgliederliste,
    personen_id: &str,
    fp: &str,
    person: &dyn Fn(&str) -> Option<Person>,
) -> bool {
    liste.eintraege.iter().any(|e| {
        (e.art == "gruendung" || e.art == "beitritt")
            && e.personen_id == personen_id
            && crate::freigaben::geraet_von(
                &Mitglied {
                    personen_id: e.personen_id.clone(),
                    name: e.name.clone(),
                    rolle: e.rolle.clone(),
                    geraet: e.geraet.clone(),
                },
                fp,
                person,
            )
    })
}

impl Aktion {
    pub fn schreiben(
        ident: &Identitaet,
        projekt: &str,
        nachricht: &str,
        art: &str,
        personen_id: &str,
    ) -> Result<Self, PersonFehler> {
        let mut a = Self {
            id: uuid::Uuid::new_v4().to_string(),
            projekt: projekt.to_string(),
            nachricht: nachricht.to_string(),
            art: art.to_string(),
            personen_id: personen_id.to_string(),
            at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            unterzeichner: ident.fingerabdruck.clone(),
            zertifikat_pem: ident.zertifikat_pem.clone(),
            signatur: String::new(),
        };
        a.signatur = unterschreiben(ident, &aktion_text(&a))?;
        Ok(a)
    }

    /// Echt, fuer dieses Projekt, eine bekannte Art, und von einem Geraet
    /// einer Person, die (je) Mitglied war. Ob sie WIRKT, entscheidet
    /// [`ansicht`].
    pub fn gilt(
        &self,
        projekt: &str,
        liste: &Mitgliederliste,
        person: &dyn Fn(&str) -> Option<Person>,
    ) -> bool {
        if self.projekt != projekt || ![ANHEFTEN, LOESEN, LOESCHEN].contains(&self.art.as_str()) {
            return false;
        }
        let echt = der(&self.zertifikat_pem)
            .is_some_and(|d| fingerabdruck(&d) == self.unterzeichner)
            && pruefen(&self.zertifikat_pem, &aktion_text(self), &self.signatur);
        echt && von_mitglied(liste, &self.personen_id, &self.unterzeichner, person)
    }
}

/// Eine Nachricht, wie der Chat sie zeigt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Angezeigt {
    pub nachricht: Nachricht,
    pub angeheftet: bool,
}

/// Was der Chat zeigt: die Nachrichten ohne die geloeschten, mit dem Stand
/// des Anheftens (die juengste Aktion gilt). `eigentuemer`: die
/// Personen-Ids der Eigentuemerinnen nach der Liste von jetzt.
pub fn ansicht(
    nachrichten: &[Nachricht],
    aktionen: &[Aktion],
    eigentuemer: &[String],
) -> Vec<Angezeigt> {
    let mut sortiert: Vec<&Aktion> = aktionen.iter().collect();
    sortiert.sort_by(|a, b| a.at.cmp(&b.at).then(a.id.cmp(&b.id)));
    nachrichten
        .iter()
        .filter(|n| {
            !sortiert.iter().any(|a| {
                a.nachricht == n.id
                    && a.art == LOESCHEN
                    && (a.personen_id == n.personen_id || eigentuemer.contains(&a.personen_id))
            })
        })
        .map(|n| Angezeigt {
            nachricht: n.clone(),
            angeheftet: sortiert
                .iter()
                .rev()
                .find(|a| a.nachricht == n.id && (a.art == ANHEFTEN || a.art == LOESEN))
                .is_some_and(|a| a.art == ANHEFTEN),
        })
        .collect()
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

    #[test]
    fn anheften_und_loeschen_nach_den_regeln_des_servers() {
        let anna_tel = Identitaet::erzeugen().unwrap();
        let ben_tel = Identitaet::erzeugen().unwrap();
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
        let eigentuemer = vec![anna.personen_id.clone()];

        let von_anna =
            Nachricht::schreiben(&anna_tel, "p1", &anna.personen_id, "Anna", "eins").unwrap();
        let von_ben =
            Nachricht::schreiben(&ben_tel, "p1", &ben.personen_id, "Ben", "zwei").unwrap();
        let alle = vec![von_anna.clone(), von_ben.clone()];

        // Ben heftet Annas Nachricht an (darf jedes Mitglied).
        let anheften =
            Aktion::schreiben(&ben_tel, "p1", &von_anna.id, ANHEFTEN, &ben.personen_id).unwrap();
        assert!(anheften.gilt("p1", &liste, &keiner));
        let v = ansicht(&alle, std::slice::from_ref(&anheften), &eigentuemer);
        assert!(v[0].angeheftet && !v[1].angeheftet);

        // Ben loescht Annas Nachricht: wirkt nicht (nicht seine, nicht Eigentuemer).
        let fremd_geloescht =
            Aktion::schreiben(&ben_tel, "p1", &von_anna.id, LOESCHEN, &ben.personen_id).unwrap();
        assert_eq!(ansicht(&alle, &[fremd_geloescht], &eigentuemer).len(), 2);

        // Ben loescht seine eigene, Anna (Eigentuemerin) jede.
        let eigene =
            Aktion::schreiben(&ben_tel, "p1", &von_ben.id, LOESCHEN, &ben.personen_id).unwrap();
        assert_eq!(ansicht(&alle, &[eigene], &eigentuemer).len(), 1);
        let von_eigentuemerin =
            Aktion::schreiben(&anna_tel, "p1", &von_ben.id, LOESCHEN, &anna.personen_id).unwrap();
        let v = ansicht(&alle, &[von_eigentuemerin], &eigentuemer);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].nachricht.id, von_anna.id);

        // Im Namen von Anna von Bens Geraet: gilt nicht.
        let untergeschoben =
            Aktion::schreiben(&ben_tel, "p1", &von_ben.id, LOESCHEN, &anna.personen_id).unwrap();
        assert!(!untergeschoben.gilt("p1", &liste, &keiner));
        // Unbekannte Art: gilt nicht.
        let quatsch =
            Aktion::schreiben(&ben_tel, "p1", &von_ben.id, "verstecken", &ben.personen_id).unwrap();
        assert!(!quatsch.gilt("p1", &liste, &keiner));
    }
}
