//! Verschlüsselte Mails: PGP/MIME (RFC 3156) mit geschütztem Betreff
//! (docs/plan-email-pgp.md, Schritt 3b).
//!
//! **Senden.** Die ganze Mail -- Text, Anhänge, der echte Betreff -- wird zu
//! einem inneren MIME-Teil, der verschlüsselt und signiert wird. Außen steht
//! nur `multipart/encrypted` mit dem Betreff „...", wie bei Thunderbird: Der
//! Mailanbieter sieht Absender, Empfänger und Zeit, aber nicht, worum es
//! geht.
//!
//! **Verschlüsselt wird an den Empfänger UND an sich selbst.** Sonst ließe
//! sich die eigene Mail in „Gesendet" -- in Thunderbird, oder hier nach einem
//! Neuaufsetzen -- nicht mehr lesen.
//!
//! **Empfangen.** Entschlüsseln mit dem eigenen Schlüssel, dann die Signatur
//! gegen den bekannten Schlüssel des Absenders prüfen. Auch „inline"
//! verschlüsselte Mails (der Text selbst ist ein `BEGIN PGP MESSAGE`) werden
//! gelesen.

use crate::mime::AnhangDaten;
use crate::{Ergebnis, PostFehler};
use base64::Engine;
use pgp::composed::{Deserializable, Message, MessageBuilder, SignedPublicKey, SignedSecretKey};
use pgp::crypto::hash::HashAlgorithm;
use pgp::crypto::sym::SymmetricKeyAlgorithm;
use pgp::types::{KeyDetails as _, Password};

fn fehler(e: impl std::fmt::Display) -> PostFehler {
    PostFehler::Schluessel(e.to_string())
}

/// Was die Signatur einer entschlüsselten Mail sagt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signatur {
    /// Nicht signiert.
    Keine,
    /// Signiert, und der bekannte Schlüssel des Absenders bestätigt es.
    Gueltig,
    /// Signiert, aber nicht vom bekannten Schlüssel des Absenders -- oder
    /// die Mail wurde verändert.
    Ungueltig,
    /// Signiert, aber der Schlüssel des Absenders ist hier nicht bekannt.
    Unbekannt,
}

impl Signatur {
    pub fn als_text(self) -> Option<&'static str> {
        match self {
            Signatur::Keine => None,
            Signatur::Gueltig => Some("gueltig"),
            Signatur::Ungueltig => Some("ungueltig"),
            Signatur::Unbekannt => Some("unbekannt"),
        }
    }
}

/* ── Der innere Teil ───────────────────────────────────────────────────── */

/// RFC 2047 für Kopfzeilen mit Nicht-ASCII.
fn kopfwort(s: &str) -> String {
    if s.is_ascii() && !s.contains(['\r', '\n']) {
        s.to_string()
    } else {
        format!(
            "=?UTF-8?B?{}?=",
            base64::engine::general_purpose::STANDARD.encode(s.replace(['\r', '\n'], " "))
        )
    }
}

fn base64_zeilen(daten: &[u8]) -> String {
    let b = base64::engine::general_purpose::STANDARD.encode(daten);
    b.as_bytes()
        .chunks(76)
        .map(|z| std::str::from_utf8(z).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\r\n")
}

fn grenze() -> String {
    let mut z = [0u8; 12];
    let _ = rustls::crypto::ring::default_provider()
        .secure_random
        .fill(&mut z);
    format!(
        "oa-{}",
        z.iter().map(|b| format!("{b:02x}")).collect::<String>()
    )
}

/// Der innere MIME-Teil: Text und Anhänge, mit dem echten Betreff als
/// „protected header" (draft-autocrypt-lamps-protected-headers).
pub fn inneres(
    von: &str,
    an: &[String],
    betreff: &str,
    text: &str,
    anhaenge: &[AnhangDaten],
) -> Vec<u8> {
    let g = grenze();
    let mut s = String::new();
    s.push_str(&format!(
        "Content-Type: multipart/mixed; boundary=\"{g}\"; protected-headers=\"v1\"\r\n"
    ));
    s.push_str(&format!("Subject: {}\r\n", kopfwort(betreff)));
    s.push_str(&format!("From: {}\r\n", kopfwort(von)));
    s.push_str(&format!("To: {}\r\n", kopfwort(&an.join(", "))));
    s.push_str("\r\n");
    s.push_str(&format!("--{g}\r\n"));
    s.push_str("Content-Type: text/plain; charset=utf-8\r\n");
    s.push_str("Content-Transfer-Encoding: base64\r\n\r\n");
    s.push_str(&base64_zeilen(text.as_bytes()));
    s.push_str("\r\n");
    for a in anhaenge {
        let name = kopfwort(&a.name).replace('"', "'");
        s.push_str(&format!("--{g}\r\n"));
        s.push_str(&format!("Content-Type: {}; name=\"{name}\"\r\n", a.mime));
        s.push_str(&format!(
            "Content-Disposition: attachment; filename=\"{name}\"\r\n"
        ));
        s.push_str("Content-Transfer-Encoding: base64\r\n\r\n");
        s.push_str(&base64_zeilen(&a.daten));
        s.push_str("\r\n");
    }
    s.push_str(&format!("--{g}--\r\n"));
    s.into_bytes()
}

/* ── Verschlüsseln und entschlüsseln ───────────────────────────────────── */

fn oeffentlich(armored: &str) -> Ergebnis<SignedPublicKey> {
    Ok(SignedPublicKey::from_string(armored).map_err(fehler)?.0)
}

fn geheim(armored: &str) -> Ergebnis<SignedSecretKey> {
    Ok(SignedSecretKey::from_string(armored).map_err(fehler)?.0)
}

/// `daten` verschlüsseln an alle `empfaenger` und an den eigenen Schlüssel,
/// signiert mit dem eigenen. Zurück kommt eine armored Nachricht.
pub fn verschluesseln(
    daten: Vec<u8>,
    empfaenger: &[String],
    eigener_geheim: &str,
) -> Ergebnis<String> {
    let eigen = geheim(eigener_geheim)?;
    let eigen_oeffentlich = eigen.to_public_key();
    let fremde: Vec<SignedPublicKey> = empfaenger
        .iter()
        .map(|k| oeffentlich(k))
        .collect::<Ergebnis<_>>()?;

    let mut rng = rand::thread_rng();
    let mut b =
        MessageBuilder::from_bytes("", daten).seipd_v1(&mut rng, SymmetricKeyAlgorithm::AES256);
    for k in fremde.iter().chain(std::iter::once(&eigen_oeffentlich)) {
        let unter = k
            .public_subkeys
            .iter()
            .find(|s| s.key.algorithm().can_encrypt())
            .ok_or_else(|| {
                PostFehler::Schluessel(format!("The key {:X} cannot encrypt.", k.fingerprint()))
            })?;
        b.encrypt_to_key(&mut rng, &unter.key).map_err(fehler)?;
    }
    b.sign(&eigen.primary_key, Password::empty(), HashAlgorithm::Sha256);
    b.to_armored_string(&mut rng, Default::default())
        .map_err(fehler)
}

/// Eine armored (oder binäre) Nachricht mit dem eigenen Schlüssel
/// entschlüsseln und, wenn sie signiert ist, gegen `absender` prüfen.
pub fn entschluesseln(
    nachricht: &[u8],
    eigener_geheim: &str,
    absender: Option<&str>,
) -> Ergebnis<(Vec<u8>, Signatur)> {
    let eigen = geheim(eigener_geheim)?;
    let (msg, _) = Message::from_armor(nachricht)
        .or_else(|_| Message::from_bytes(nachricht).map(|m| (m, Default::default())))
        .map_err(|_| PostFehler::Schluessel("Not a readable OpenPGP message.".into()))?;
    let mut klar = msg
        .decrypt(&Password::empty(), &eigen)
        .map_err(|_| PostFehler::Schluessel("This mail is not encrypted to your key.".into()))?;
    if klar.is_compressed() {
        klar = klar.decompress().map_err(fehler)?;
    }
    let daten = klar.as_data_vec().map_err(fehler)?;

    let signatur = if !klar.is_signed() {
        Signatur::Keine
    } else {
        match absender.and_then(|a| oeffentlich(a).ok()) {
            None => Signatur::Unbekannt,
            Some(k) => {
                // Signiert wird mit dem Hauptschlüssel oder einem
                // Unterschlüssel zum Signieren -- beide versuchen.
                let passt = klar.verify(&k.primary_key).is_ok()
                    || k.public_subkeys.iter().any(|s| klar.verify(&s.key).is_ok());
                if passt {
                    Signatur::Gueltig
                } else {
                    Signatur::Ungueltig
                }
            }
        }
    };
    Ok((daten, signatur))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schluessel::erzeugen;

    #[test]
    fn hin_und_zurueck_mit_signatur() {
        let tiffy = erzeugen("tiffy@beispiel.test", "Tiffy").unwrap();
        let ferdinand = erzeugen("ferdinand@beispiel.test", "").unwrap();
        let innen = inneres(
            "tiffy@beispiel.test",
            &["ferdinand@beispiel.test".into()],
            "Geheime Grüße",
            "Hallo Ferdinand!",
            &[AnhangDaten {
                name: "plan.pdf".into(),
                mime: "application/pdf".into(),
                daten: b"%PDF-1.4\n".to_vec(),
            }],
        );
        let zu = verschluesseln(
            innen.clone(),
            std::slice::from_ref(&ferdinand.oeffentlich),
            &tiffy.geheim,
        )
        .unwrap();
        assert!(zu.starts_with("-----BEGIN PGP MESSAGE-----"));

        // Ferdinand liest und prüft gegen Tiffys Schlüssel.
        let (klar, sig) =
            entschluesseln(zu.as_bytes(), &ferdinand.geheim, Some(&tiffy.oeffentlich)).unwrap();
        assert_eq!(klar, innen);
        assert_eq!(sig, Signatur::Gueltig);
        let m = crate::mime::lesen(&klar).unwrap();
        assert_eq!(m.betreff, "Geheime Grüße");
        assert_eq!(m.text, "Hallo Ferdinand!");
        assert_eq!(m.anhaenge[0].daten, b"%PDF-1.4\n");

        // Tiffy liest ihre eigene (Gesendet).
        assert!(entschluesseln(zu.as_bytes(), &tiffy.geheim, None).is_ok());

        // Mit fremdem Schlüssel geprüft: ungültig; ohne: unbekannt.
        let (_, sig) = entschluesseln(
            zu.as_bytes(),
            &ferdinand.geheim,
            Some(&ferdinand.oeffentlich),
        )
        .unwrap();
        assert_eq!(sig, Signatur::Ungueltig);
        let (_, sig) = entschluesseln(zu.as_bytes(), &ferdinand.geheim, None).unwrap();
        assert_eq!(sig, Signatur::Unbekannt);

        // Ein Dritter kann nichts lesen.
        let dritter = erzeugen("mallory@beispiel.test", "").unwrap();
        assert!(entschluesseln(zu.as_bytes(), &dritter.geheim, None).is_err());
    }
}
