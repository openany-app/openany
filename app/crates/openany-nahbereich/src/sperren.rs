//! Sperren beim Bearbeiten freigegebener Notizen (Tiffy, 01.10.2026: die
//! strenge Regel -- keine Konfliktkopien).
//!
//! **Die Sperre vergibt das Geraet, das die Notiz fuehrt** -- das der Person,
//! die freigegeben hat. Wer bearbeiten will, holt sie dort; gespeichert wird
//! auch dort. Ist das Geraet nicht erreichbar, bleibt die Notiz nur lesbar.
//! Deshalb kann es zwei gleichzeitige Fassungen gar nicht geben.
//!
//! **Die Sperre laeuft ab**, nach [`SPERRE_GILT`] ohne Tippen (jedes
//! Speichern verlaengert sie) -- oder sofort, wenn man schliesst.
//!
//! **Auch die eigene Person zaehlt:** Wer auf dem fuehrenden Geraet selbst an
//! der Notiz schreibt, haelt sie damit ebenso, und ein Mitglied bekommt
//! solange keine Sperre.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Wie lange eine Sperre ohne Tippen gilt.
pub const SPERRE_GILT: Duration = Duration::from_secs(10 * 60);

/// Wer eine Notiz gerade haelt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Halter {
    pub personen_id: String,
    pub name: String,
    /// Das Geraet des Mitglieds; leer, wenn es hier auf dem fuehrenden
    /// Geraet selbst geschieht.
    pub geraet: String,
}

#[derive(Debug, Default)]
pub struct Sperren {
    gehalten: HashMap<String, (Halter, Instant)>,
}

pub type GemeinsameSperren = Arc<Mutex<Sperren>>;

impl Sperren {
    fn aufraeumen(&mut self) {
        self.gehalten
            .retain(|_, (_, seit)| seit.elapsed() < SPERRE_GILT);
    }

    /// Wer haelt die Notiz gerade (wenn jemand)?
    pub fn halter(&mut self, notiz: &str) -> Option<Halter> {
        self.aufraeumen();
        self.gehalten.get(notiz).map(|(h, _)| h.clone())
    }

    /// Die Sperre nehmen oder verlaengern. `Err` nennt, wer sie hat.
    pub fn nehmen(&mut self, notiz: &str, wer: Halter) -> Result<(), Halter> {
        self.aufraeumen();
        match self.gehalten.get(notiz) {
            Some((h, _)) if *h != wer => Err(h.clone()),
            _ => {
                self.gehalten
                    .insert(notiz.to_string(), (wer, Instant::now()));
                Ok(())
            }
        }
    }

    /// Hat `wer` die Sperre (noch)?
    pub fn haelt(&mut self, notiz: &str, wer: &Halter) -> bool {
        self.halter(notiz).is_some_and(|h| h == *wer)
    }

    /// Freigeben -- nur, wer sie hat.
    pub fn loslassen(&mut self, notiz: &str, wer: &Halter) {
        if self.gehalten.get(notiz).is_some_and(|(h, _)| h == wer) {
            self.gehalten.remove(notiz);
        }
    }

    #[cfg(test)]
    fn altern(&mut self, notiz: &str, um: Duration) {
        if let Some((_, seit)) = self.gehalten.get_mut(notiz) {
            *seit -= um;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wer(name: &str) -> Halter {
        Halter {
            personen_id: name.into(),
            name: name.into(),
            geraet: format!("fp-{name}"),
        }
    }

    #[test]
    fn eine_sperre_haelt_bis_sie_losgelassen_wird_oder_ablaeuft() {
        let mut s = Sperren::default();
        assert!(s.nehmen("n1", wer("Ben")).is_ok());
        assert!(s.nehmen("n1", wer("Ben")).is_ok(), "verlaengern geht");
        assert_eq!(s.nehmen("n1", wer("Carla")).unwrap_err().name, "Ben");
        assert!(
            s.nehmen("n2", wer("Carla")).is_ok(),
            "andere Notiz, andere Sperre"
        );

        // Carla laesst Bens Sperre nicht los.
        s.loslassen("n1", &wer("Carla"));
        assert!(s.haelt("n1", &wer("Ben")));
        s.loslassen("n1", &wer("Ben"));
        assert!(s.nehmen("n1", wer("Carla")).is_ok());

        // Zehn Minuten ohne Tippen: weg.
        s.altern("n1", SPERRE_GILT + Duration::from_secs(1));
        assert!(s.halter("n1").is_none());
        assert!(s.nehmen("n1", wer("Ben")).is_ok());
    }
}
