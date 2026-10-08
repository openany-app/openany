//! Wie lange nach einem Abriss gewartet wird.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Wachsende Wartezeit mit Zufall: 2 s, 4 s, 8 s ... bis 5 min, jeweils
/// zwischen der Haelfte und dem Ganzen davon.
///
/// Der Zufall ist der Punkt. Faellt ntfy fuer eine Minute aus, haben alle
/// Geraete im selben Moment ihre Leitung verloren -- ohne Zufall kaemen sie
/// auch im selben Moment wieder, jedes Mal, bis ganz oben.
#[derive(Debug, Clone)]
pub struct Wartezeit {
    versuch: u32,
    anfang: Duration,
    hoechstens: Duration,
}

impl Default for Wartezeit {
    fn default() -> Self {
        Self::new(Duration::from_secs(2), Duration::from_secs(300))
    }
}

impl Wartezeit {
    pub fn new(anfang: Duration, hoechstens: Duration) -> Self {
        Self {
            versuch: 0,
            anfang,
            hoechstens,
        }
    }

    /// Die obere Grenze fuer den naechsten Versuch, ohne Zufall.
    pub fn grenze(&self) -> Duration {
        let faktor = 1u32.checked_shl(self.versuch.min(20)).unwrap_or(u32::MAX);
        self.anfang.saturating_mul(faktor).min(self.hoechstens)
    }

    /// Die naechste Wartezeit; zaehlt den Versuch mit.
    pub fn naechste(&mut self) -> Duration {
        let grenze = self.grenze();
        self.versuch = self.versuch.saturating_add(1);
        let haelfte = grenze / 2;
        haelfte + haelfte.mul_f64(zufall())
    }

    /// Nach einer Leitung, die wirklich stand (`open` kam an).
    pub fn zuruecksetzen(&mut self) {
        self.versuch = 0;
    }
}

/// Eine Zahl in [0, 1). Fuer Streuung reicht die Uhr; eine Zufallskiste
/// waere fuer diesen Zweck eine Abhaengigkeit ohne Gegenwert.
fn zufall() -> f64 {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    // Die unteren Bits der Nanosekunden sind auf manchen Uhren grob; einmal
    // durchmischen (splitmix-artig) verteilt sie.
    let mut x = u64::from(n).wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    (x >> 11) as f64 / (1u64 << 53) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waechst_bis_zur_grenze_und_streut_darunter() {
        let mut w = Wartezeit::default();
        let mut grenzen = Vec::new();
        for _ in 0..12 {
            let g = w.grenze();
            let t = w.naechste();
            assert!(t >= g / 2 && t <= g, "{t:?} ausserhalb von {g:?}");
            grenzen.push(g.as_secs());
        }
        assert_eq!(&grenzen[..9], &[2, 4, 8, 16, 32, 64, 128, 256, 300]);
        assert_eq!(grenzen[11], 300);
    }

    #[test]
    fn zuruecksetzen_faengt_vorne_an() {
        let mut w = Wartezeit::default();
        for _ in 0..5 {
            w.naechste();
        }
        w.zuruecksetzen();
        assert_eq!(w.grenze(), Duration::from_secs(2));
    }

    #[test]
    fn viele_versuche_laufen_nicht_ueber() {
        let mut w = Wartezeit::default();
        for _ in 0..1000 {
            w.naechste();
        }
        assert_eq!(w.grenze(), Duration::from_secs(300));
    }
}
