//! Ein Konto in der App anlegen (docs/plan-registrierung-app.md in openany,
//! A1/A4) -- und der Ausweis-Tausch fuer die Sicherungsdatei (A2).
//!
//! **Das Raetsel** ist ein Arbeitsnachweis: Gesucht ist ein Versuch, dessen
//! Argon2id-Abdruck `bits` fuehrende Nullbits hat. Im Mittel kostet das
//! 2^bits Abdruecke; anyid prueft mit einem. Die Werte muessen genau die von
//! anyid sein (libsodium, `crypto_pwhash` mit Argon2id v1.3, eine Spur,
//! 32 Bytes) -- der Test unten haelt einen Abdruck von libsodium fest.

use argon2::{Algorithm, Argon2, Params, Version};
use base64::Engine;
use serde::Deserialize;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crate::Geraet;

/// Was anyid fuer das Anlegen ausgibt.
#[derive(Debug, Clone, Deserialize)]
pub struct Raetsel {
    pub raetsel: String,
    /// base64
    pub salz: String,
    pub bits: u32,
    pub argon: Argonwerte,
    /// Wie lange es gilt, in Sekunden.
    pub gueltig: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Argonwerte {
    pub kib: u32,
    pub durchgaenge: u32,
}

/// Das angelegte Konto und sein erstes Geraet.
#[derive(Debug, Clone, Deserialize)]
pub struct Angelegt {
    /// Der Ausweis dieses Geraets -- genau einmal zu sehen.
    pub token: String,
    pub geraet: Geraet,
    pub konto: Konto,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Konto {
    pub sub: String,
    pub name: String,
}

/// Ein neuer Ausweis nach dem Tausch.
#[derive(Debug, Clone, Deserialize)]
pub struct Getauscht {
    pub token: String,
    pub geraet: Geraet,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Raetselfehler {
    #[error("Das Rätsel wurde abgebrochen.")]
    Abgebrochen,
    #[error("Das Rätsel ist nicht lesbar.")]
    Unlesbar,
}

/// Wie viele Versuche bisher -- fuer einen Fortschritt in der Oberflaeche.
/// Erwartet sind `2^bits`; mehr als das Doppelte ist Pech, kein Fehler.
#[derive(Debug, Default)]
pub struct Raetselstand {
    pub versuche: AtomicU64,
    pub abbrechen: AtomicBool,
}

/// Sucht die Loesung. Rechnet lange (Sekunden bis Minuten) und blockiert --
/// in einem eigenen Faden aufrufen. Gibt die Loesung als base64 zurueck.
pub fn loesen(raetsel: &Raetsel, stand: &Raetselstand) -> Result<String, Raetselfehler> {
    let salz = base64::engine::general_purpose::STANDARD
        .decode(&raetsel.salz)
        .map_err(|_| Raetselfehler::Unlesbar)?;
    let argon = argon_fuer(&raetsel.argon)?;
    let mut abdruck = [0u8; 32];

    for i in 0u64.. {
        if stand.abbrechen.load(Ordering::Relaxed) {
            return Err(Raetselfehler::Abgebrochen);
        }
        let versuch = i.to_be_bytes();
        argon
            .hash_password_into(&versuch, &salz, &mut abdruck)
            .map_err(|_| Raetselfehler::Unlesbar)?;
        stand.versuche.store(i + 1, Ordering::Relaxed);
        if fuehrende_nullbits(&abdruck) >= raetsel.bits {
            return Ok(base64::engine::general_purpose::STANDARD.encode(versuch));
        }
    }
    unreachable!("u64 reicht")
}

fn argon_fuer(werte: &Argonwerte) -> Result<Argon2<'static>, Raetselfehler> {
    let params = Params::new(werte.kib, werte.durchgaenge, 1, Some(32))
        .map_err(|_| Raetselfehler::Unlesbar)?;
    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
}

pub fn fuehrende_nullbits(bytes: &[u8]) -> u32 {
    let mut bits = 0;
    for b in bytes {
        if *b == 0 {
            bits += 8;
            continue;
        }
        return bits + b.leading_zeros();
    }
    bits
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }

    /// Derselbe Abdruck wie libsodium auf anyids Seite:
    /// `sodium_crypto_pwhash(32, pack("J", 5), str_repeat("\x01", 16), 2, 19456*1024, ARGON2ID13)`.
    #[test]
    fn rechnet_wie_libsodium() {
        let argon = argon_fuer(&Argonwerte {
            kib: 19456,
            durchgaenge: 2,
        })
        .unwrap();
        let mut out = [0u8; 32];
        argon
            .hash_password_into(&5u64.to_be_bytes(), &[1u8; 16], &mut out)
            .unwrap();
        assert_eq!(
            hex(&out),
            "457c0af6ac949b49995faab7b81f72bddcc5e32087d23c4ceb0d77f6024e60b4"
        );

        let leicht = argon_fuer(&Argonwerte {
            kib: 1024,
            durchgaenge: 1,
        })
        .unwrap();
        leicht
            .hash_password_into(&5u64.to_be_bytes(), &[1u8; 16], &mut out)
            .unwrap();
        assert_eq!(
            hex(&out),
            "04ede9e2824dd3d5aaea980817146b23af9bc960b4ee7771aa5404ec5363f15a"
        );
    }

    #[test]
    fn findet_eine_loesung_und_laesst_sich_abbrechen() {
        let raetsel = Raetsel {
            raetsel: "r".into(),
            salz: base64::engine::general_purpose::STANDARD.encode([7u8; 16]),
            bits: 3,
            argon: Argonwerte {
                kib: 1024,
                durchgaenge: 1,
            },
            gueltig: 600,
        };
        let stand = Raetselstand::default();
        let loesung = loesen(&raetsel, &stand).unwrap();
        let versuch = base64::engine::general_purpose::STANDARD
            .decode(loesung)
            .unwrap();
        let mut out = [0u8; 32];
        argon_fuer(&raetsel.argon)
            .unwrap()
            .hash_password_into(&versuch, &[7u8; 16], &mut out)
            .unwrap();
        assert!(fuehrende_nullbits(&out) >= 3);
        assert!(stand.versuche.load(Ordering::Relaxed) >= 1);

        let weg = Raetselstand::default();
        weg.abbrechen.store(true, Ordering::Relaxed);
        assert_eq!(loesen(&raetsel, &weg), Err(Raetselfehler::Abgebrochen));
    }

    #[test]
    fn nullbits() {
        assert_eq!(fuehrende_nullbits(&[0x80]), 0);
        assert_eq!(fuehrende_nullbits(&[0x01]), 7);
        assert_eq!(fuehrende_nullbits(&[0x00, 0x0f]), 12);
    }
}
