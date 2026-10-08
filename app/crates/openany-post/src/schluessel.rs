//! OpenPGP-Schlüssel (docs/plan-email-pgp.md, Schritt 3a).
//!
//! **Der eigene Schlüssel** entsteht hier oder wird eingelesen (etwa der
//! Export aus Thunderbird). Er wird ENTSPERRT zurückgegeben: Die App legt ihn
//! in den Tresor (Android-Keystore), eine zweite Passphrase davor brächte
//! nichts, außer dass man sie bei jeder Mail eingeben müsste. Nur die Ausfuhr
//! als Datei bekommt eine Passphrase.
//!
//! **Fremde Schlüssel** kommen aus Autocrypt-Kopfzeilen empfangener Mails,
//! vom Mailanbieter des Gegenübers (WKD) oder -- nur auf Knopfdruck -- von
//! keys.openpgp.org (Tiffy, 29.09.2026: Der Schlüsselserver soll nicht
//! nebenbei erfahren, wem man schreibt).
//!
//! **Erzeugt wird ein v4-Schlüssel mit Ed25519 und Cv25519**, wie Thunderbird
//! und GnuPG ihn lesen. v6 (RFC 9580) können viele Gegenüber noch nicht.

use crate::{Ergebnis, PostFehler};
use pgp::composed::{
    ArmorOptions, Deserializable, EncryptionCaps, KeyType, SecretKeyParamsBuilder, SignedPublicKey,
    SignedSecretKey, SubkeyParamsBuilder,
};
use pgp::crypto::ecc_curve::ECCCurve;
use pgp::ser::Serialize as _;
use pgp::types::Password;
use serde::{Deserialize, Serialize};

fn fehler(e: impl std::fmt::Display) -> PostFehler {
    PostFehler::Schluessel(e.to_string())
}

/// Das eigene Schlüsselpaar eines Postfachs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EigenerSchluessel {
    /// ASCII-armored, ENTSPERRT -- gehört nur in den Tresor.
    pub geheim: String,
    /// ASCII-armored.
    pub oeffentlich: String,
    /// 40 Hex-Zeichen, groß.
    pub fingerabdruck: String,
}

/// Ein öffentlicher Schlüssel eines Gegenübers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FremderSchluessel {
    /// ASCII-armored.
    pub oeffentlich: String,
    pub fingerabdruck: String,
    /// Die Adressen aus den Benutzerkennungen, klein geschrieben.
    pub adressen: Vec<String>,
}

fn fingerabdruck(k: &impl pgp::types::KeyDetails) -> String {
    format!("{:X}", k.fingerprint())
}

/// Die Adressen aus den Benutzerkennungen („Name <a@b>" oder nur „a@b").
fn adressen(details: &pgp::composed::SignedKeyDetails) -> Vec<String> {
    details
        .users
        .iter()
        .filter_map(|u| {
            let id = String::from_utf8_lossy(u.id.id()).to_string();
            let adresse = match (id.rfind('<'), id.rfind('>')) {
                (Some(a), Some(b)) if a < b => id[a + 1..b].to_string(),
                _ => id,
            };
            let adresse = adresse.trim().to_lowercase();
            adresse.contains('@').then_some(adresse)
        })
        .collect()
}

fn armored_geheim(k: &SignedSecretKey) -> Ergebnis<String> {
    k.to_armored_string(ArmorOptions::default()).map_err(fehler)
}

fn armored_oeffentlich(k: &SignedPublicKey) -> Ergebnis<String> {
    k.to_armored_string(ArmorOptions::default()).map_err(fehler)
}

fn eigener(k: &SignedSecretKey) -> Ergebnis<EigenerSchluessel> {
    Ok(EigenerSchluessel {
        geheim: armored_geheim(k)?,
        oeffentlich: armored_oeffentlich(&k.to_public_key())?,
        fingerabdruck: fingerabdruck(&k.primary_key),
    })
}

/// Ein neues Schlüsselpaar für `adresse` (mit `name`, wenn einer da ist).
pub fn erzeugen(adresse: &str, name: &str) -> Ergebnis<EigenerSchluessel> {
    let uid = if name.trim().is_empty() {
        format!("<{}>", adresse.trim())
    } else {
        format!("{} <{}>", name.trim(), adresse.trim())
    };
    let mut verschluesseln = SubkeyParamsBuilder::default();
    verschluesseln
        .key_type(KeyType::ECDH(ECCCurve::Curve25519Legacy))
        .can_sign(false)
        .can_encrypt(EncryptionCaps::All)
        .can_authenticate(false);
    let mut params = SecretKeyParamsBuilder::default();
    params
        .key_type(KeyType::Ed25519Legacy)
        .can_certify(true)
        .can_sign(true)
        .can_encrypt(EncryptionCaps::None)
        .primary_user_id(uid)
        .subkeys(vec![verschluesseln.build().map_err(fehler)?]);
    let schluessel = params
        .build()
        .map_err(fehler)?
        .generate(rand::thread_rng())
        .map_err(fehler)?;
    eigener(&schluessel)
}

/// Einen geheimen Schlüssel einlesen (armored oder binär) und entsperren.
/// `passphrase` darf leer sein, wenn er keine hat.
pub fn einlesen(daten: &[u8], passphrase: &str) -> Ergebnis<EigenerSchluessel> {
    let (mut k, _) = SignedSecretKey::from_reader_single(daten)
        .map_err(|_| PostFehler::Schluessel("This is not a secret OpenPGP key.".into()))?;
    k.verify_bindings().map_err(fehler)?;
    let pw = Password::from(passphrase);
    k.primary_key
        .remove_password(&pw)
        .map_err(|_| PostFehler::Schluessel("The passphrase does not match.".into()))?;
    for sub in &mut k.secret_subkeys {
        sub.key
            .remove_password(&pw)
            .map_err(|_| PostFehler::Schluessel("The passphrase does not match.".into()))?;
    }
    eigener(&k)
}

/// Den geheimen Schlüssel als Datei ausführen, gesperrt mit `passphrase`.
pub fn ausfuhr(geheim: &str, passphrase: &str) -> Ergebnis<String> {
    if passphrase.chars().count() < 8 {
        return Err(PostFehler::Schluessel(
            "The passphrase needs at least 8 characters.".into(),
        ));
    }
    let (mut k, _) = SignedSecretKey::from_string(geheim).map_err(fehler)?;
    let pw = Password::from(passphrase);
    k.primary_key
        .set_password(rand::thread_rng(), &pw)
        .map_err(fehler)?;
    for sub in &mut k.secret_subkeys {
        sub.key
            .set_password(rand::thread_rng(), &pw)
            .map_err(fehler)?;
    }
    armored_geheim(&k)
}

/// Einen öffentlichen Schlüssel lesen (armored oder binär) und prüfen.
pub fn oeffentlich_lesen(daten: &[u8]) -> Ergebnis<FremderSchluessel> {
    let (k, _) = SignedPublicKey::from_reader_single(daten)
        .map_err(|_| PostFehler::Schluessel("This is not a public OpenPGP key.".into()))?;
    k.verify_bindings().map_err(fehler)?;
    Ok(FremderSchluessel {
        oeffentlich: armored_oeffentlich(&k)?,
        fingerabdruck: fingerabdruck(&k.primary_key),
        adressen: adressen(&k.details),
    })
}

/* ── Autocrypt (https://autocrypt.org/level1.html) ──────────────────────── */

/// Die Kopfzeile `Autocrypt:` für ausgehende Mails, schon gefaltet (78
/// Zeichen je Zeile, Fortsetzung mit Leerzeichen).
pub fn autocrypt_kopf(adresse: &str, oeffentlich: &str) -> Ergebnis<String> {
    use base64::Engine;
    let (k, _) = SignedPublicKey::from_string(oeffentlich).map_err(fehler)?;
    let daten = base64::engine::general_purpose::STANDARD.encode(k.to_bytes().map_err(fehler)?);
    let mut kopf = format!("addr={}; keydata=", adresse.trim().to_lowercase());
    for stueck in daten.as_bytes().chunks(72) {
        kopf.push_str("\r\n ");
        kopf.push_str(std::str::from_utf8(stueck).unwrap_or_default());
    }
    Ok(kopf)
}

/// Eine empfangene Autocrypt-Kopfzeile lesen. Nur gültig, wenn `addr=` der
/// Absender ist -- sonst könnte jede Mail einen Schlüssel für eine fremde
/// Adresse unterschieben.
pub fn autocrypt_lesen(kopf: &str, absender: &str) -> Option<FremderSchluessel> {
    use base64::Engine;
    let mut addr = None;
    let mut keydata = None;
    for teil in kopf.split(';') {
        let (k, v) = teil.split_once('=')?;
        match k.trim() {
            "addr" => addr = Some(v.trim().to_lowercase()),
            "keydata" => keydata = Some(v.split_whitespace().collect::<String>()),
            // Unbekannte kritische Attribute (ohne `_`) machen die Kopfzeile
            // ungültig, sagt Autocrypt.
            "prefer-encrypt" => {}
            andere if !andere.starts_with('_') => return None,
            _ => {}
        }
    }
    if addr? != absender.trim().to_lowercase() {
        return None;
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(keydata?)
        .ok()?;
    oeffentlich_lesen(&bytes).ok()
}

/* ── WKD und keys.openpgp.org ──────────────────────────────────────────── */

/// z-base-32 (RFC 6189), wie WKD es für den gehashten Lokalteil nimmt.
fn zbase32(daten: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ybndrfg8ejkmcpqxot1uwisza345h769";
    let mut aus = String::new();
    let mut puffer: u32 = 0;
    let mut bits = 0;
    for &b in daten {
        puffer = (puffer << 8) | b as u32;
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            aus.push(ALPHABET[((puffer >> bits) & 31) as usize] as char);
        }
    }
    if bits > 0 {
        aus.push(ALPHABET[((puffer << (5 - bits)) & 31) as usize] as char);
    }
    aus
}

fn wkd_adressen(adresse: &str) -> Option<[String; 2]> {
    use sha1::{Digest, Sha1};
    let (lokal, domain) = adresse.trim().rsplit_once('@')?;
    let domain = domain.to_lowercase();
    let hash = zbase32(&Sha1::digest(lokal.to_lowercase().as_bytes()));
    let l = url_teil(lokal);
    Some([
        format!("https://openpgpkey.{domain}/.well-known/openpgpkey/{domain}/hu/{hash}?l={l}"),
        format!("https://{domain}/.well-known/openpgpkey/hu/{hash}?l={l}"),
    ])
}

fn url_teil(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

async fn holen(url: &str) -> Option<Vec<u8>> {
    let c = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()?;
    let r = c.get(url).send().await.ok()?;
    if !r.status().is_success() {
        return None;
    }
    r.bytes().await.ok().map(|b| b.to_vec())
}

/// Nur einen Schlüssel, der die gesuchte Adresse auch trägt.
fn passend(daten: &[u8], adresse: &str) -> Option<FremderSchluessel> {
    let k = oeffentlich_lesen(daten).ok()?;
    k.adressen
        .contains(&adresse.trim().to_lowercase())
        .then_some(k)
}

/// Beim Mailanbieter des Gegenübers suchen (Web Key Directory): erst die
/// „fortgeschrittene", dann die „direkte" Adresse.
pub async fn wkd_suchen(adresse: &str) -> Option<FremderSchluessel> {
    for url in wkd_adressen(adresse)? {
        if let Some(k) = holen(&url).await.and_then(|d| passend(&d, adresse)) {
            return Some(k);
        }
    }
    None
}

/// Bei keys.openpgp.org suchen -- dort stehen nur bestätigte Adressen.
pub async fn schluesselserver_suchen(adresse: &str) -> Option<FremderSchluessel> {
    let url = format!(
        "https://keys.openpgp.org/vks/v1/by-email/{}",
        url_teil(&adresse.trim().to_lowercase())
    );
    holen(&url).await.and_then(|d| passend(&d, adresse))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erzeugen_ausfuhr_und_wieder_einlesen() {
        let k = erzeugen("tiffy@beispiel.test", "Tiffy").unwrap();
        assert_eq!(k.fingerabdruck.len(), 40);
        let fremd = oeffentlich_lesen(k.oeffentlich.as_bytes()).unwrap();
        assert_eq!(fremd.fingerabdruck, k.fingerabdruck);
        assert_eq!(fremd.adressen, vec!["tiffy@beispiel.test"]);

        let datei = ausfuhr(&k.geheim, "lange passphrase").unwrap();
        assert!(einlesen(datei.as_bytes(), "falsch!!").is_err());
        let wieder = einlesen(datei.as_bytes(), "lange passphrase").unwrap();
        assert_eq!(wieder.fingerabdruck, k.fingerabdruck);
        // Entsperrt: ohne Passphrase noch einmal lesbar.
        assert!(einlesen(wieder.geheim.as_bytes(), "").is_ok());
    }

    #[test]
    fn autocrypt_hin_und_zurueck_nur_fuer_den_absender() {
        let k = erzeugen("tiffy@beispiel.test", "").unwrap();
        let kopf = autocrypt_kopf("tiffy@beispiel.test", &k.oeffentlich).unwrap();
        assert!(kopf.lines().all(|z| z.len() <= 78));
        let gelesen = autocrypt_lesen(&kopf, "Tiffy@Beispiel.test").unwrap();
        assert_eq!(gelesen.fingerabdruck, k.fingerabdruck);
        assert!(autocrypt_lesen(&kopf, "mallory@beispiel.test").is_none());
    }

    /// Das Beispiel aus dem WKD-Entwurf: „Joe.Doe@Example.ORG".
    #[test]
    fn wkd_adresse_wie_im_entwurf() {
        let [fortgeschritten, direkt] = wkd_adressen("Joe.Doe@Example.ORG").unwrap();
        assert_eq!(
            direkt,
            "https://example.org/.well-known/openpgpkey/hu/iy9q119eutrkn8s1mk4r39qejnbu3n5q?l=Joe.Doe"
        );
        assert!(fortgeschritten
            .starts_with("https://openpgpkey.example.org/.well-known/openpgpkey/example.org/hu/"));
    }
}
