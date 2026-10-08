//! Paaren -- die Zustandsmaschine, ohne Netz.
//!
//! **Zwei eigene Geraete erkennen sich an ihrem Fingerabdruck.** Wer den
//! Fingerabdruck des anderen einmal bestaetigt hat, ist gepaart; ab dann
//! prueft jede Verbindung, dass am anderen Ende genau dieses Zertifikat steht.
//!
//! **Der Vergleichscode schuetzt das erste Mal.** Beim ersten Kontakt koennte
//! ein Dritter im selben WLAN sich dazwischenschieben und jedem Geraet sein
//! eigenes Zertifikat zeigen. Beide Geraete rechnen deshalb aus BEIDEN
//! Fingerabdruecken und zwei Zufallszahlen einen 6-stelligen Code. Nur wenn
//! kein Dritter dazwischen sitzt, zeigen beide denselben -- und erst wenn der
//! Mensch das auf beiden Seiten bestaetigt, gilt die Paarung.
//!
//! **Beide Seiten muessen bestaetigen.** Das anfragende Geraet meldet seine
//! Bestaetigung hinueber; das gefragte Geraet wird danach gefragt. Jedes merkt
//! sich das andere erst, wenn es von beiden Bestaetigungen weiss.

use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Wie lange eine offene Paarung gilt.
pub const PAARUNG_GILT: Duration = Duration::from_secs(120);

/// Der Vergleichscode aus beiden Fingerabdruecken und beiden Zufallszahlen.
///
/// Die Fingerabdruecke gehen SORTIERT hinein, damit beide Seiten dasselbe
/// rechnen, ohne sich zu einigen, wer "erster" ist. Die Zufallszahlen in fester
/// Rolle: erst die des Anfragenden, dann die des Gefragten.
pub fn vergleichscode(
    fp_a: &str,
    fp_b: &str,
    zufall_anfrage: &str,
    zufall_antwort: &str,
) -> String {
    let (klein, gross) = if fp_a <= fp_b {
        (fp_a, fp_b)
    } else {
        (fp_b, fp_a)
    };
    let mut h = Sha256::new();
    for teil in [klein, gross, zufall_anfrage, zufall_antwort] {
        h.update(teil.as_bytes());
        h.update([0x1f]);
    }
    let d = h.finalize();
    let zahl = u32::from_be_bytes([d[0], d[1], d[2], d[3]]) % 1_000_000;
    format!("{zahl:06}")
}

/// Welche Seite eine Paarung angestossen hat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Rolle {
    Anfragend,
    Gefragt,
}

/// Eine offene Paarung mit einem anderen Geraet.
#[derive(Debug, Clone, serde::Serialize)]
pub struct OffenePaarung {
    pub fingerabdruck: String,
    pub name: String,
    pub code: String,
    pub rolle: Rolle,
    pub hier_bestaetigt: bool,
    pub dort_bestaetigt: bool,
    #[serde(skip)]
    seit: Instant,
}

impl OffenePaarung {
    pub fn fertig(&self) -> bool {
        self.hier_bestaetigt && self.dort_bestaetigt
    }
}

/// Alle offenen Paarungen dieses Geraets, nach Fingerabdruck des anderen.
#[derive(Debug, Default)]
pub struct Paarungen {
    offen: HashMap<String, OffenePaarung>,
}

impl Paarungen {
    /// Eine Anfrage kam herein (wir sind gefragt).
    pub fn eingang(
        &mut self,
        fp_anderer: &str,
        name: &str,
        fp_ich: &str,
        zufall_anfrage: &str,
        zufall_antwort: &str,
    ) -> String {
        let code = vergleichscode(fp_anderer, fp_ich, zufall_anfrage, zufall_antwort);
        self.offen.insert(
            fp_anderer.to_string(),
            OffenePaarung {
                fingerabdruck: fp_anderer.to_string(),
                name: name.to_string(),
                code: code.clone(),
                rolle: Rolle::Gefragt,
                hier_bestaetigt: false,
                dort_bestaetigt: false,
                seit: Instant::now(),
            },
        );
        code
    }

    /// Wir haben angefragt und die Antwort bekommen.
    pub fn ausgang(
        &mut self,
        fp_anderer: &str,
        name: &str,
        fp_ich: &str,
        zufall_anfrage: &str,
        zufall_antwort: &str,
    ) -> String {
        let code = vergleichscode(fp_ich, fp_anderer, zufall_anfrage, zufall_antwort);
        self.offen.insert(
            fp_anderer.to_string(),
            OffenePaarung {
                fingerabdruck: fp_anderer.to_string(),
                name: name.to_string(),
                code: code.clone(),
                rolle: Rolle::Anfragend,
                hier_bestaetigt: false,
                dort_bestaetigt: false,
                seit: Instant::now(),
            },
        );
        code
    }

    pub fn hier_bestaetigen(&mut self, fp: &str) -> Option<&OffenePaarung> {
        self.aufraeumen();
        let p = self.offen.get_mut(fp)?;
        p.hier_bestaetigt = true;
        Some(p)
    }

    pub fn dort_bestaetigt(&mut self, fp: &str) -> Option<&OffenePaarung> {
        self.aufraeumen();
        let p = self.offen.get_mut(fp)?;
        p.dort_bestaetigt = true;
        Some(p)
    }

    /// Den QR-Weg: Das Geheimnis im Code beweist die Naehe, ein Vergleich ist
    /// ueberfluessig -- beide Bestaetigungen gelten als gegeben.
    pub fn durch_geheimnis(&mut self, fp: &str, name: &str) -> OffenePaarung {
        let p = OffenePaarung {
            fingerabdruck: fp.to_string(),
            name: name.to_string(),
            code: String::new(),
            rolle: Rolle::Gefragt,
            hier_bestaetigt: true,
            dort_bestaetigt: true,
            seit: Instant::now(),
        };
        self.offen.insert(fp.to_string(), p.clone());
        p
    }

    pub fn abbrechen(&mut self, fp: &str) {
        self.offen.remove(fp);
    }

    pub fn entfernen_wenn_fertig(&mut self, fp: &str) -> Option<OffenePaarung> {
        if self.offen.get(fp).is_some_and(OffenePaarung::fertig) {
            return self.offen.remove(fp);
        }
        None
    }

    pub fn get(&mut self, fp: &str) -> Option<&OffenePaarung> {
        self.aufraeumen();
        self.offen.get(fp)
    }

    pub fn alle(&mut self) -> Vec<OffenePaarung> {
        self.aufraeumen();
        self.offen.values().cloned().collect()
    }

    fn aufraeumen(&mut self) {
        self.offen.retain(|_, p| p.seit.elapsed() < PAARUNG_GILT);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beide_seiten_rechnen_denselben_code() {
        let a = vergleichscode("AAAA", "BBBB", "za", "zb");
        let b = vergleichscode("BBBB", "AAAA", "za", "zb");
        assert_eq!(a, b);
        assert_eq!(a.len(), 6);
    }

    #[test]
    fn ein_dritter_dazwischen_ergibt_einen_anderen_code() {
        // A spricht mit M (glaubt, es sei B), M spricht mit B.
        let bei_a = vergleichscode("AAAA", "MMMM", "za", "zm");
        let bei_b = vergleichscode("MMMM", "BBBB", "zm2", "zb");
        assert_ne!(bei_a, bei_b);
    }

    #[test]
    fn fertig_erst_mit_beiden_bestaetigungen() {
        let mut gefragt = Paarungen::default();
        let mut anfragend = Paarungen::default();

        let code_b = gefragt.eingang("AAAA", "Telefon", "BBBB", "za", "zb");
        let code_a = anfragend.ausgang("BBBB", "Tablet", "AAAA", "za", "zb");
        assert_eq!(code_a, code_b);

        anfragend.hier_bestaetigen("BBBB");
        gefragt.dort_bestaetigt("AAAA");
        assert!(
            gefragt.entfernen_wenn_fertig("AAAA").is_none(),
            "gefragt hat noch nicht bestaetigt"
        );

        gefragt.hier_bestaetigen("AAAA");
        assert!(gefragt.entfernen_wenn_fertig("AAAA").is_some());

        anfragend.dort_bestaetigt("BBBB");
        assert!(anfragend.entfernen_wenn_fertig("BBBB").is_some());
    }
}
