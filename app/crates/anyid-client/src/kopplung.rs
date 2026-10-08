//! Der Nachweis, den nur das Programm fuehren kann.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::Rng;
use sha2::{Digest, Sha256};

/// Das Geheimnis einer Kopplung - erzeugt, bevor irgendetwas verschickt wird.
///
/// Der `challenge` geht mit dem ersten Aufruf hinaus, der Verifier bleibt hier.
/// Wer die Kopplung mitliest, hat damit die Haelfte, aus der sich die andere
/// nicht errechnen laesst - genau dafuer ist ein Hash da.
#[derive(Debug, Clone)]
pub struct Verifier(String);

impl Verifier {
    /// 64 Zeichen aus dem Zufallsgenerator des Betriebssystems.
    pub fn neu() -> Self {
        const ZEICHEN: &[u8] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";

        let mut rng = rand::thread_rng();

        Self(
            (0..64)
                .map(|_| ZEICHEN[rng.gen_range(0..ZEICHEN.len())] as char)
                .collect(),
        )
    }

    /// Nur fuer Tests und fuer das Wiederaufnehmen einer laufenden Kopplung.
    pub fn aus(wert: impl Into<String>) -> Self {
        Self(wert.into())
    }

    pub fn als_str(&self) -> &str {
        &self.0
    }

    /// Der SHA-256 des Verifiers, base64url ohne Polster - 43 Zeichen.
    ///
    /// Genau die Form, die anyid erwartet; sie prueft die Laenge, weil kuerzer
    /// kein SHA-256 sein koennte.
    pub fn challenge(&self) -> String {
        URL_SAFE_NO_PAD.encode(Sha256::digest(self.0.as_bytes()))
    }
}

/// Eine begonnene Kopplung: was dem Menschen zu zeigen ist und womit das
/// Programm abholt.
#[derive(Debug, Clone)]
pub struct Kopplungsstart {
    /// Anzeigen, nicht verschicken.
    pub code: String,
    pub geraetecode: String,
    pub verifier: Verifier,
    pub intervall: u64,
    pub ablauf_in: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn der_challenge_hat_die_form_die_anyid_verlangt() {
        // anyid prueft `size:43` - kuerzer koennte kein SHA-256 sein, laenger
        // nur ein anderes Verfahren.
        let challenge = Verifier::neu().challenge();

        assert_eq!(challenge.len(), 43);
        assert!(!challenge.contains('='), "kein Polster");
        assert!(
            !challenge.contains('+') && !challenge.contains('/'),
            "base64url"
        );
    }

    #[test]
    fn derselbe_verifier_ergibt_denselben_challenge() {
        let v = Verifier::aus("immer-derselbe");

        assert_eq!(v.challenge(), Verifier::aus("immer-derselbe").challenge());
    }

    #[test]
    fn zwei_verifier_sind_verschieden() {
        // Waeren sie es nicht, koennte ein zweites Programm die Kopplung eines
        // ersten abholen.
        assert_ne!(Verifier::neu().als_str(), Verifier::neu().als_str());
    }

    #[test]
    fn der_challenge_stimmt_mit_dem_ueberein_was_anyid_rechnet() {
        // Derselbe Wert, den anyids Geraetekopplung::challenge() erzeugt -
        // hier als fester Vergleich, damit ein Wechsel des Verfahrens auf
        // einer der beiden Seiten auffaellt.
        assert_eq!(
            Verifier::aus("abc").challenge(),
            "ungWv48Bz-pBQUDeXa4iI7ADYaOWF3qctBD_YfIAFa0",
        );
    }
}
