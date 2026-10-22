//! Direktnachrichten Ende-zu-Ende -- Konto-Modell (docs/plan-direktnachrichten-e2e.md,
//! Tiffy 16.10.2026).
//!
//! **Nur die Kryptographie, ohne Netz und ohne Tauri** -- damit sie sich
//! ohne Geraet pruefen laesst.
//!
//! - **Konto-Schluessel** ([`Kontoschluessel`], age/X25519): Verschluesselt
//!   wird an ihn -- an den des Empfaengers und den eigenen. Er liegt auf ALLEN
//!   freigegebenen Geraeten eines Kontos, samt allen frueheren Fassungen (nach
//!   einem Wechsel bleibt der alte Verlauf lesbar). Egal also, auf welchem
//!   Geraet man liest oder schreibt.
//! - **Geraeteschluessel** ([`Schluessel`]): age zum Empfangen der Kiste,
//!   Ed25519 zum Unterschreiben. Jede Nachricht und jede Kiste unterschreibt
//!   das Geraet, das sie macht.
//! - **Kiste** ([`kiste_packen`], [`kiste_oeffnen`]): der Konto-Schluessel,
//!   verschlossen fuer EIN Geraet. So kommt er zu einem neuen Geraet (nach
//!   dem Code-Vergleich, [`code`]) und nach einem Wechsel zu allen.
//! - **Anhang** ([`anhang_verschluesseln`], [`anhang_oeffnen`]): die Datei
//!   mit einem eigenen Zufallsschluessel; Name, Typ, Groesse, Schluessel und
//!   Pruefsumme stehen im verschluesselten, unterschriebenen [`Inhalt`]. So
//!   kann der Server die Datei weder lesen noch unbemerkt austauschen.

pub mod dateischloss;
pub mod freischalten;

use std::io::{Read, Write};

use base64::{engine::general_purpose::STANDARD as B64, Engine};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha512};

#[derive(Debug, thiserror::Error)]
pub enum Fehler {
    #[error("invalid key: {0}")]
    Schluessel(String),
    #[error("could not encrypt: {0}")]
    Verschluesseln(String),
    #[error("not readable on this device")]
    NichtFuerMich,
    #[error("signature does not match")]
    Unterschrift,
    #[error("content damaged: {0}")]
    Inhalt(String),
}

fn schluesselfehler(e: impl ToString) -> Fehler {
    Fehler::Schluessel(e.to_string())
}

// --- age-Hilfen ------------------------------------------------------------

fn age_verschluesseln(empfaenger: &[String], klartext: &[u8]) -> Result<Vec<u8>, Fehler> {
    let mut ziele: Vec<age::x25519::Recipient> = Vec::new();
    for r in empfaenger {
        let r = r
            .parse::<age::x25519::Recipient>()
            .map_err(schluesselfehler)?;
        if !ziele.iter().any(|z| z.to_string() == r.to_string()) {
            ziele.push(r);
        }
    }
    let verschluesseler =
        age::Encryptor::with_recipients(ziele.iter().map(|z| z as &dyn age::Recipient))
            .map_err(|e| Fehler::Verschluesseln(e.to_string()))?;
    let mut chiffrat = Vec::new();
    let mut schreiber = verschluesseler
        .wrap_output(&mut chiffrat)
        .map_err(|e| Fehler::Verschluesseln(e.to_string()))?;
    schreiber
        .write_all(klartext)
        .map_err(|e| Fehler::Verschluesseln(e.to_string()))?;
    schreiber
        .finish()
        .map_err(|e| Fehler::Verschluesseln(e.to_string()))?;
    Ok(chiffrat)
}

fn age_entschluesseln(
    identitaeten: &[&age::x25519::Identity],
    chiffrat: &[u8],
) -> Result<Vec<u8>, Fehler> {
    let entschluesseler =
        age::Decryptor::new(chiffrat).map_err(|e| Fehler::Inhalt(e.to_string()))?;
    let mut leser = entschluesseler
        .decrypt(identitaeten.iter().map(|i| *i as &dyn age::Identity))
        .map_err(|_| Fehler::NichtFuerMich)?;
    let mut klartext = Vec::new();
    leser
        .read_to_end(&mut klartext)
        .map_err(|e| Fehler::Inhalt(e.to_string()))?;
    Ok(klartext)
}

fn pruefen(ed25519_b64: &str, nachricht: &[u8], signatur_b64: &str) -> Result<(), Fehler> {
    let ed: [u8; 32] = B64
        .decode(ed25519_b64)
        .map_err(schluesselfehler)?
        .try_into()
        .map_err(|_| Fehler::Schluessel("ed25519 length".into()))?;
    let pruefer = VerifyingKey::from_bytes(&ed).map_err(schluesselfehler)?;
    let sig: [u8; 64] = B64
        .decode(signatur_b64)
        .map_err(|_| Fehler::Unterschrift)?
        .try_into()
        .map_err(|_| Fehler::Unterschrift)?;
    pruefer
        .verify(nachricht, &Signature::from_bytes(&sig))
        .map_err(|_| Fehler::Unterschrift)
}

// --- Geraeteschluessel -----------------------------------------------------

/// Die geheimen Schluessel EINES Geraets.
pub struct Schluessel {
    age: age::x25519::Identity,
    unterschrift: SigningKey,
}

#[derive(Serialize, Deserialize)]
struct GeraetGespeichert {
    v: u8,
    age: String,
    ed25519: String,
}

impl Schluessel {
    pub fn erzeugen() -> Result<Self, Fehler> {
        let mut saat = [0u8; 32];
        getrandom::fill(&mut saat).map_err(schluesselfehler)?;
        Ok(Self {
            age: age::x25519::Identity::generate(),
            unterschrift: SigningKey::from_bytes(&saat),
        })
    }

    /// Als Text -- fuer Tresor und Sicherungsdatei. GEHEIM.
    pub fn als_text(&self) -> String {
        use age::secrecy::ExposeSecret;
        serde_json::to_string(&GeraetGespeichert {
            v: 1,
            age: self.age.to_string().expose_secret().to_string(),
            ed25519: B64.encode(self.unterschrift.to_bytes()),
        })
        .expect("serialisierbar")
    }

    pub fn aus_text(text: &str) -> Result<Self, Fehler> {
        let g: GeraetGespeichert = serde_json::from_str(text).map_err(schluesselfehler)?;
        let age = g
            .age
            .parse::<age::x25519::Identity>()
            .map_err(schluesselfehler)?;
        let bytes: [u8; 32] = B64
            .decode(g.ed25519)
            .map_err(schluesselfehler)?
            .try_into()
            .map_err(|_| Fehler::Schluessel("ed25519 length".into()))?;
        Ok(Self {
            age,
            unterschrift: SigningKey::from_bytes(&bytes),
        })
    }

    /// Die oeffentlichen Teile, wie der Server sie nimmt: (`age1…`, Ed25519 Base64).
    pub fn oeffentlich(&self) -> (String, String) {
        (
            self.age.to_public().to_string(),
            B64.encode(self.unterschrift.verifying_key().to_bytes()),
        )
    }

    fn unterschreiben(&self, nachricht: &[u8]) -> String {
        B64.encode(self.unterschrift.sign(nachricht).to_bytes())
    }
}

// `code` (6 Ziffern aus den Schluesseln des neuen Geraets) ist seit Z4 fort:
// Ein luegender Server konnte sich passende Schluessel erzeugen. Freigegeben
// wird per SPAKE2, siehe [`freischalten`].

// --- Ausweis vor Ort (docs/plan-whiteboard.md, W3a) ---------------------------

/// Was ein Ausweis unterschreibt: das Konto (`name`, `geraet` im
/// Verzeichnis), beide TLS-Fingerabdruecke (wer ruft, wen) und die Zeit. So
/// gilt er nur auf genau dieser Leitung und nur kurz -- ein mitgeschnittener
/// Ausweis taugt bei keinem anderen Geraet und nicht spaeter.
fn ausweis_text(ziel: &str, anrufer: &str, name: &str, geraet: i64, zeit: u64) -> Vec<u8> {
    format!("openany-ausweis-v1\n{ziel}\n{anrufer}\n{name}\n{geraet}\n{zeit}").into_bytes()
}

/// Ein Mitglied weist sich vor Ort aus: mit dem Unterschriftsschluessel
/// dieses Geraets, den das Verzeichnis unter `name`/`geraet` fuehrt.
pub fn ausweis_unterschreiben(
    ich: &Schluessel,
    ziel: &str,
    anrufer: &str,
    name: &str,
    geraet: i64,
    zeit: u64,
) -> String {
    ich.unterschreiben(&ausweis_text(ziel, anrufer, name, geraet, zeit))
}

/// Einen Ausweis pruefen -- `ed25519` aus dem Verzeichnis.
pub fn ausweis_pruefen(
    ed25519: &str,
    ziel: &str,
    anrufer: &str,
    name: &str,
    geraet: i64,
    zeit: u64,
    signatur: &str,
) -> Result<(), Fehler> {
    pruefen(
        ed25519,
        &ausweis_text(ziel, anrufer, name, geraet, zeit),
        signatur,
    )
}

// --- Konto-Schluessel ------------------------------------------------------

/// Der Konto-Schluessel samt allen frueheren Fassungen. GEHEIM.
pub struct Kontoschluessel {
    fassungen: Vec<(u32, age::x25519::Identity)>,
}

#[derive(Serialize, Deserialize)]
struct FassungGespeichert {
    v: u32,
    age: String,
}

impl Clone for Kontoschluessel {
    fn clone(&self) -> Self {
        Self {
            fassungen: self.fassungen.clone(),
        }
    }
}

impl Kontoschluessel {
    /// Ein neuer Konto-Schluessel, Fassung 1 (das erste Geraet).
    pub fn neu() -> Self {
        Self {
            fassungen: vec![(1, age::x25519::Identity::generate())],
        }
    }

    /// Wechseln: eine neue Fassung dazu (Geraet verloren). Die alten bleiben
    /// -- sonst waere der bisherige Verlauf unlesbar.
    pub fn wechseln(&mut self) -> u32 {
        let neu = self.aktuell_fassung() + 1;
        self.fassungen
            .push((neu, age::x25519::Identity::generate()));
        neu
    }

    pub fn aktuell_fassung(&self) -> u32 {
        self.fassungen.iter().map(|(v, _)| *v).max().unwrap_or(0)
    }

    /// Der oeffentliche Teil der gueltigen Fassung (`age1…`).
    pub fn oeffentlich(&self) -> String {
        self.fassungen
            .iter()
            .max_by_key(|(v, _)| *v)
            .map(|(_, i)| i.to_public().to_string())
            .unwrap_or_default()
    }

    /// Als Text -- fuer Tresor, Sicherungsdatei und Kiste. GEHEIM.
    pub fn als_text(&self) -> String {
        use age::secrecy::ExposeSecret;
        let liste: Vec<FassungGespeichert> = self
            .fassungen
            .iter()
            .map(|(v, i)| FassungGespeichert {
                v: *v,
                age: i.to_string().expose_secret().to_string(),
            })
            .collect();
        serde_json::to_string(&liste).expect("serialisierbar")
    }

    pub fn aus_text(text: &str) -> Result<Self, Fehler> {
        let liste: Vec<FassungGespeichert> =
            serde_json::from_str(text).map_err(schluesselfehler)?;
        let mut fassungen = Vec::new();
        for f in liste {
            fassungen.push((
                f.v,
                f.age
                    .parse::<age::x25519::Identity>()
                    .map_err(schluesselfehler)?,
            ));
        }
        if fassungen.is_empty() {
            return Err(Fehler::Schluessel("empty".into()));
        }
        Ok(Self { fassungen })
    }

    fn identitaeten(&self) -> Vec<&age::x25519::Identity> {
        self.fassungen.iter().map(|(_, i)| i).collect()
    }

    /// Ein Abdruck, den der Server vergleichen, aber nicht nachrechnen kann
    /// (docs/plan-kalender-e2e.md): `abdruck` ist der Klartext-Abdruck eines
    /// Termins, verdeckt mit einem Geheimnis aus der ERSTEN Fassung -- so
    /// bleibt er ueber Schluesselwechsel gleich, und ein Wechsel macht keine
    /// Konflikte. Ohne das Geheimnis liessen sich kurze Termine (Titel, Zeit)
    /// am Abdruck durchprobieren.
    pub fn abdruck_verdecken(&self, uuid: &str, abdruck: &str) -> String {
        use age::secrecy::ExposeSecret;
        let erste = self
            .fassungen
            .iter()
            .min_by_key(|(v, _)| *v)
            .map(|(_, i)| i.to_string().expose_secret().to_string())
            .unwrap_or_default();
        let geheim = Sha256::digest(format!("openany-abdruck-v1\n{erste}").as_bytes());
        let mut h = Sha256::new();
        h.update(geheim);
        h.update(format!("\n{uuid}\n{abdruck}").as_bytes());
        h.finalize().iter().map(|b| format!("{b:02x}")).collect()
    }
}

// --- Kisten ------------------------------------------------------------------

fn kiste_zu_unterschreiben(ziel_geraet: i64, version: u32, chiffrat_b64: &str) -> Vec<u8> {
    format!("openany-kiste-v1\n{ziel_geraet}\n{version}\n{chiffrat_b64}").into_bytes()
}

/// Den Konto-Schluessel fuer ein Geraet verschliessen. `ziel_age` ist dessen
/// Geraeteschluessel. Antwort: (Chiffrat, Unterschrift), beides Base64.
pub fn kiste_packen(
    ich: &Schluessel,
    konto: &Kontoschluessel,
    ziel_geraet: i64,
    ziel_age: &str,
) -> Result<(String, String), Fehler> {
    let chiffrat = B64.encode(age_verschluesseln(
        &[ziel_age.to_string()],
        konto.als_text().as_bytes(),
    )?);
    let signatur = ich.unterschreiben(&kiste_zu_unterschreiben(
        ziel_geraet,
        konto.aktuell_fassung(),
        &chiffrat,
    ));
    Ok((chiffrat, signatur))
}

/// Eine Kiste oeffnen -- gepackt fuer `mein_geraet`, unterschrieben vom
/// Geraet mit `von_ed25519` (eines des eigenen Kontos).
pub fn kiste_oeffnen(
    ich: &Schluessel,
    mein_geraet: i64,
    version: u32,
    chiffrat: &str,
    signatur: &str,
    von_ed25519: &str,
) -> Result<Kontoschluessel, Fehler> {
    pruefen(
        von_ed25519,
        &kiste_zu_unterschreiben(mein_geraet, version, chiffrat),
        signatur,
    )?;
    let roh = B64
        .decode(chiffrat)
        .map_err(|e| Fehler::Inhalt(e.to_string()))?;
    let klartext = age_entschluesseln(&[&ich.age], &roh)?;
    let text = String::from_utf8(klartext).map_err(|e| Fehler::Inhalt(e.to_string()))?;
    Kontoschluessel::aus_text(&text)
}

// --- Nachrichten -------------------------------------------------------------

/// Ein Geraet aus dem Verzeichnis des Servers (zum Pruefen von Unterschriften).
#[derive(Debug, Clone, Deserialize)]
pub struct Geraet {
    pub geraet: i64,
    pub ed25519: String,
    #[serde(default = "ja")]
    pub aktiv: bool,
}

fn ja() -> bool {
    true
}

/// Was verschluesselt wird.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Inhalt {
    pub v: u8,
    /// Absender und Empfaenger (Konto-Namen) -- auch drinnen: Der Klartext
    /// bindet sich selbst an die Unterhaltung.
    pub von: String,
    pub an: String,
    pub text: String,
    /// Ein Anhang (Schritt 4). Fehlt in aelteren Nachrichten.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anhang: Option<AnhangInfo>,
}

/// Was der Empfaenger zum Oeffnen eines Anhangs braucht -- nur im
/// verschluesselten Inhalt, nie beim Server.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnhangInfo {
    pub name: String,
    pub mime: String,
    /// Bytes der Datei (unverschluesselt).
    pub groesse: u64,
    /// age-Identitaet (`AGE-SECRET-KEY-1…`) nur fuer diese eine Datei.
    pub schluessel: String,
    /// SHA-256 des Chiffrats, hex -- bindet die Datei an die Nachricht.
    pub pruefsumme: String,
}

fn pruefsumme(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Eine Datei verschluesseln: mit einem frischen Schluessel nur fuer sie.
/// Zurueck kommt das Chiffrat (das geht zum Server) und, was in den Inhalt
/// der Nachricht gehoert.
pub fn anhang_verschluesseln(
    name: &str,
    mime: &str,
    bytes: &[u8],
) -> Result<(Vec<u8>, AnhangInfo), Fehler> {
    use age::secrecy::ExposeSecret;
    let identitaet = age::x25519::Identity::generate();
    let chiffrat = age_verschluesseln(&[identitaet.to_public().to_string()], bytes)?;
    let info = AnhangInfo {
        name: name.into(),
        mime: mime.into(),
        groesse: bytes.len() as u64,
        schluessel: identitaet.to_string().expose_secret().to_string(),
        pruefsumme: pruefsumme(&chiffrat),
    };
    Ok((chiffrat, info))
}

/// Eine Datei oeffnen -- erst die Pruefsumme, dann entschluesseln.
pub fn anhang_oeffnen(chiffrat: &[u8], info: &AnhangInfo) -> Result<Vec<u8>, Fehler> {
    if pruefsumme(chiffrat) != info.pruefsumme {
        return Err(Fehler::Unterschrift);
    }
    let identitaet = info
        .schluessel
        .parse::<age::x25519::Identity>()
        .map_err(schluesselfehler)?;
    age_entschluesseln(&[&identitaet], chiffrat)
}

/// Was an den Server geht.
#[derive(Debug, Clone, PartialEq)]
pub struct Verschluesselt {
    /// age, Base64.
    pub inhalt: String,
    /// Ed25519 ueber Chiffrat, Absender, Empfaenger -- Base64.
    pub signatur: String,
}

fn nachricht_zu_unterschreiben(inhalt_b64: &str, von: &str, an: &str) -> Vec<u8> {
    format!("openany-direkt-v1\n{von}\n{an}\n{inhalt_b64}").into_bytes()
}

/// `text` von `von` an `an` -- verschluesselt an die Konto-Schluessel in
/// `empfaenger` (der des Empfaengers und der eigene), unterschrieben von
/// diesem Geraet.
pub fn verschluesseln(
    ich: &Schluessel,
    von: &str,
    an: &str,
    text: &str,
    anhang: Option<AnhangInfo>,
    empfaenger: &[String],
) -> Result<Verschluesselt, Fehler> {
    let klartext = serde_json::to_vec(&Inhalt {
        v: 1,
        von: von.into(),
        an: an.into(),
        text: text.into(),
        anhang,
    })
    .map_err(|e| Fehler::Verschluesseln(e.to_string()))?;
    let inhalt = B64.encode(age_verschluesseln(empfaenger, &klartext)?);
    let signatur = ich.unterschreiben(&nachricht_zu_unterschreiben(&inhalt, von, an));
    Ok(Verschluesselt { inhalt, signatur })
}

/// Pruefen (gegen das unterschreibende Geraet des Absenders) und mit dem
/// eigenen Konto-Schluessel -- irgendeiner Fassung -- entschluesseln.
pub fn oeffnen(
    konto: &Kontoschluessel,
    von: &str,
    an: &str,
    v: &Verschluesselt,
    absender: &Geraet,
) -> Result<Inhalt, Fehler> {
    pruefen(
        &absender.ed25519,
        &nachricht_zu_unterschreiben(&v.inhalt, von, an),
        &v.signatur,
    )?;
    let roh = B64
        .decode(&v.inhalt)
        .map_err(|e| Fehler::Inhalt(e.to_string()))?;
    let klartext = age_entschluesseln(&konto.identitaeten(), &roh)?;
    let inhalt: Inhalt =
        serde_json::from_slice(&klartext).map_err(|e| Fehler::Inhalt(e.to_string()))?;
    if inhalt.von != von || inhalt.an != an {
        return Err(Fehler::Unterschrift);
    }
    Ok(inhalt)
}

// --- Adressbuch (docs/plan-adressbuch-e2e.md) -------------------------------

fn kontakt_zu_unterschreiben(uuid: &str, version: u32, inhalt_b64: &str) -> Vec<u8> {
    format!("openany-kontakt-v1\n{uuid}\n{version}\n{inhalt_b64}").into_bytes()
}

/// Einen Kontakt (`klartext`, JSON) an den eigenen Konto-Schluessel in seiner
/// gueltigen Fassung verschluesseln -- nur das eigene Konto liest ihn.
/// Unterschrieben samt uuid und Fassung: Der Server kann Kontakte weder lesen
/// noch untereinander vertauschen. Zurueck kommt auch die Fassung.
pub fn kontakt_verschluesseln(
    ich: &Schluessel,
    konto: &Kontoschluessel,
    uuid: &str,
    klartext: &[u8],
) -> Result<(Verschluesselt, u32), Fehler> {
    let version = konto.aktuell_fassung();
    let inhalt = B64.encode(age_verschluesseln(&[konto.oeffentlich()], klartext)?);
    let signatur = ich.unterschreiben(&kontakt_zu_unterschreiben(uuid, version, &inhalt));
    Ok((Verschluesselt { inhalt, signatur }, version))
}

/// Pruefen (gegen das eigene Geraet mit `von_ed25519`) und mit irgendeiner
/// Fassung des Konto-Schluessels oeffnen.
pub fn kontakt_oeffnen(
    konto: &Kontoschluessel,
    uuid: &str,
    version: u32,
    v: &Verschluesselt,
    von_ed25519: &str,
) -> Result<Vec<u8>, Fehler> {
    pruefen(
        von_ed25519,
        &kontakt_zu_unterschreiben(uuid, version, &v.inhalt),
        &v.signatur,
    )?;
    let roh = B64
        .decode(&v.inhalt)
        .map_err(|e| Fehler::Inhalt(e.to_string()))?;
    age_entschluesseln(&konto.identitaeten(), &roh)
}

// --- Eigene Sachen: Kalender, Termine (docs/plan-kalender-e2e.md) ----------

fn sache_zu_unterschreiben(art: &str, uuid: &str, version: u32, inhalt_b64: &str) -> Vec<u8> {
    format!("openany-sache-v1\n{art}\n{uuid}\n{version}\n{inhalt_b64}").into_bytes()
}

/// Wie [`kontakt_verschluesseln`], fuer eine Sache der Art `art`
/// (`calendar`, `event` ...): Art und uuid stehen in der Unterschrift -- der
/// Server kann keinen Termin als Kalender oder als einen anderen Termin
/// ausgeben.
pub fn sache_verschluesseln(
    ich: &Schluessel,
    konto: &Kontoschluessel,
    art: &str,
    uuid: &str,
    klartext: &[u8],
) -> Result<(Verschluesselt, u32), Fehler> {
    let version = konto.aktuell_fassung();
    let inhalt = B64.encode(age_verschluesseln(&[konto.oeffentlich()], klartext)?);
    let signatur = ich.unterschreiben(&sache_zu_unterschreiben(art, uuid, version, &inhalt));
    Ok((Verschluesselt { inhalt, signatur }, version))
}

pub fn sache_oeffnen(
    konto: &Kontoschluessel,
    art: &str,
    uuid: &str,
    version: u32,
    v: &Verschluesselt,
    von_ed25519: &str,
) -> Result<Vec<u8>, Fehler> {
    pruefen(
        von_ed25519,
        &sache_zu_unterschreiben(art, uuid, version, &v.inhalt),
        &v.signatur,
    )?;
    let roh = B64
        .decode(&v.inhalt)
        .map_err(|e| Fehler::Inhalt(e.to_string()))?;
    age_entschluesseln(&konto.identitaeten(), &roh)
}

// --- Sicherheitsnummer (docs/plan-adressbuch-e2e.md, 2c) --------------------

/// Die Haelfte EINES Kontos: 30 Ziffern aus Name und Konto-Schluessel,
/// mit vielen Runden gehasht wie bei Signal -- damit sich kein zweiter
/// Schluessel mit derselben Nummer durchprobieren laesst.
fn sicherheitshaelfte(name: &str, age: &str) -> String {
    let mut h = Sha512::digest(format!("openany-sicherheit-v1\n{name}\n{age}").as_bytes()).to_vec();
    for _ in 0..5200 {
        let mut d = Sha512::new();
        d.update(&h);
        d.update(age.as_bytes());
        h = d.finalize().to_vec();
    }
    h[..30]
        .chunks(5)
        .map(|c| {
            let zahl = c.iter().fold(0u64, |z, b| (z << 8) | u64::from(*b));
            format!("{:05}", zahl % 100_000)
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Die Sicherheitsnummer zweier Konten: zwoelf Fuenfergruppen, auf beiden
/// Seiten gleich (die Haelften nach Namen sortiert). Stimmt sie, hat der
/// Server keinem von beiden einen falschen Schluessel untergeschoben.
pub fn sicherheitsnummer(ich: (&str, &str), du: (&str, &str)) -> String {
    let mut haelften = [
        (ich.0.trim().to_lowercase(), ich.1.trim()),
        (du.0.trim().to_lowercase(), du.1.trim()),
    ];
    haelften.sort();
    format!(
        "{} {}",
        sicherheitshaelfte(&haelften[0].0, haelften[0].1),
        sicherheitshaelfte(&haelften[1].0, haelften[1].1)
    )
}

// --- Projektschluessel (docs/plan-projektchat-e2e.md) -------------------------
//
// Derselbe Typ wie der Konto-Schluessel ([`Kontoschluessel`]: age mit
// Fassungen), nur geteilt mit den Mitgliedern eines Projekts statt mit den
// eigenen Geraeten. Die Kiste enthaelt ALLE Fassungen -- ein neues Mitglied
// liest den bisherigen Verlauf, ein ausgetretenes nach dem Wechsel nichts
// Neues mehr.

fn projektkiste_zu_unterschreiben(
    projekt: &str,
    nutzer: i64,
    version: u32,
    chiffrat_b64: &str,
) -> Vec<u8> {
    format!("openany-projektkiste-v1\n{projekt}\n{nutzer}\n{version}\n{chiffrat_b64}").into_bytes()
}

/// Den Projektschluessel fuer ein Mitglied verschliessen -- an dessen
/// Konto-Schluessel (`ziel_konto_age`). Antwort: (Chiffrat, Unterschrift).
pub fn projektkiste_packen(
    ich: &Schluessel,
    projektschluessel: &Kontoschluessel,
    projekt: &str,
    nutzer: i64,
    ziel_konto_age: &str,
) -> Result<(String, String), Fehler> {
    let chiffrat = B64.encode(age_verschluesseln(
        &[ziel_konto_age.to_string()],
        projektschluessel.als_text().as_bytes(),
    )?);
    let signatur = ich.unterschreiben(&projektkiste_zu_unterschreiben(
        projekt,
        nutzer,
        projektschluessel.aktuell_fassung(),
        &chiffrat,
    ));
    Ok((chiffrat, signatur))
}

/// Die eigene Kiste oeffnen -- mit dem eigenen Konto-Schluessel, unterschrieben
/// vom Geraet mit `von_ed25519`.
pub fn projektkiste_oeffnen(
    konto: &Kontoschluessel,
    projekt: &str,
    nutzer: i64,
    version: u32,
    chiffrat: &str,
    signatur: &str,
    von_ed25519: &str,
) -> Result<Kontoschluessel, Fehler> {
    pruefen(
        von_ed25519,
        &projektkiste_zu_unterschreiben(projekt, nutzer, version, chiffrat),
        signatur,
    )?;
    let roh = B64
        .decode(chiffrat)
        .map_err(|e| Fehler::Inhalt(e.to_string()))?;
    let klartext = age_entschluesseln(&konto.identitaeten(), &roh)?;
    let text = String::from_utf8(klartext).map_err(|e| Fehler::Inhalt(e.to_string()))?;
    Kontoschluessel::aus_text(&text)
}

/// Was im Chiffrat einer Chat-Nachricht steht.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Projektinhalt {
    pub v: u8,
    pub projekt: String,
    pub von: String,
    pub text: String,
}

fn projekt_zu_unterschreiben(projekt: &str, von: &str, version: u32, inhalt_b64: &str) -> Vec<u8> {
    format!("openany-projekt-v1\n{projekt}\n{von}\n{version}\n{inhalt_b64}").into_bytes()
}

/// Eine Chat-Nachricht an die gueltige Fassung des Projektschluessels.
pub fn projekt_verschluesseln(
    ich: &Schluessel,
    projektschluessel: &Kontoschluessel,
    projekt: &str,
    von: &str,
    text: &str,
) -> Result<(Verschluesselt, u32), Fehler> {
    let version = projektschluessel.aktuell_fassung();
    let klartext = serde_json::to_vec(&Projektinhalt {
        v: 1,
        projekt: projekt.into(),
        von: von.into(),
        text: text.into(),
    })
    .map_err(|e| Fehler::Verschluesseln(e.to_string()))?;
    let inhalt = B64.encode(age_verschluesseln(
        &[projektschluessel.oeffentlich()],
        &klartext,
    )?);
    let signatur = ich.unterschreiben(&projekt_zu_unterschreiben(projekt, von, version, &inhalt));
    Ok((Verschluesselt { inhalt, signatur }, version))
}

/// Pruefen (gegen das Geraet des Absenders) und oeffnen.
pub fn projekt_oeffnen(
    projektschluessel: &Kontoschluessel,
    projekt: &str,
    von: &str,
    version: u32,
    v: &Verschluesselt,
    absender_ed25519: &str,
) -> Result<Projektinhalt, Fehler> {
    pruefen(
        absender_ed25519,
        &projekt_zu_unterschreiben(projekt, von, version, &v.inhalt),
        &v.signatur,
    )?;
    let roh = B64
        .decode(&v.inhalt)
        .map_err(|e| Fehler::Inhalt(e.to_string()))?;
    let klartext = age_entschluesseln(&projektschluessel.identitaeten(), &roh)?;
    let inhalt: Projektinhalt =
        serde_json::from_slice(&klartext).map_err(|e| Fehler::Inhalt(e.to_string()))?;
    if inhalt.projekt != projekt || inhalt.von != von {
        return Err(Fehler::Unterschrift);
    }
    Ok(inhalt)
}

/// Ein Feld des Projekts selbst -- der Name (3c). Eigene Domaene in der
/// Unterschrift: Ein Name laesst sich nicht als Chat-Nachricht ausgeben und
/// umgekehrt.
fn projektfeld_zu_unterschreiben(
    projekt: &str,
    feld: &str,
    version: u32,
    inhalt_b64: &str,
) -> Vec<u8> {
    format!("openany-projektfeld-v1\n{projekt}\n{feld}\n{version}\n{inhalt_b64}").into_bytes()
}

pub fn projektfeld_verschluesseln(
    ich: &Schluessel,
    projektschluessel: &Kontoschluessel,
    projekt: &str,
    feld: &str,
    wert: &str,
) -> Result<(Verschluesselt, u32), Fehler> {
    let version = projektschluessel.aktuell_fassung();
    let inhalt = B64.encode(age_verschluesseln(
        &[projektschluessel.oeffentlich()],
        wert.as_bytes(),
    )?);
    let signatur = ich.unterschreiben(&projektfeld_zu_unterschreiben(
        projekt, feld, version, &inhalt,
    ));
    Ok((Verschluesselt { inhalt, signatur }, version))
}

pub fn projektfeld_oeffnen(
    projektschluessel: &Kontoschluessel,
    projekt: &str,
    feld: &str,
    version: u32,
    v: &Verschluesselt,
    von_ed25519: &str,
) -> Result<String, Fehler> {
    pruefen(
        von_ed25519,
        &projektfeld_zu_unterschreiben(projekt, feld, version, &v.inhalt),
        &v.signatur,
    )?;
    let roh = B64
        .decode(&v.inhalt)
        .map_err(|e| Fehler::Inhalt(e.to_string()))?;
    String::from_utf8(age_entschluesseln(&projektschluessel.identitaeten(), &roh)?)
        .map_err(|e| Fehler::Inhalt(e.to_string()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn ausweis_gilt_nur_auf_seiner_leitung() {
        let ich = Schluessel::erzeugen().unwrap();
        let (_, ed) = ich.oeffentlich();
        let sig = ausweis_unterschreiben(&ich, "ziel", "anrufer", "anna", 7, 1000);
        assert!(ausweis_pruefen(&ed, "ziel", "anrufer", "anna", 7, 1000, &sig).is_ok());
        // Anderes Ziel, anderer Anrufer, anderes Konto, andere Zeit: nichts.
        assert!(ausweis_pruefen(&ed, "anderes", "anrufer", "anna", 7, 1000, &sig).is_err());
        assert!(ausweis_pruefen(&ed, "ziel", "fremd", "anna", 7, 1000, &sig).is_err());
        assert!(ausweis_pruefen(&ed, "ziel", "anrufer", "ben", 7, 1000, &sig).is_err());
        assert!(ausweis_pruefen(&ed, "ziel", "anrufer", "anna", 8, 1000, &sig).is_err());
        assert!(ausweis_pruefen(&ed, "ziel", "anrufer", "anna", 7, 1001, &sig).is_err());
        let fremd = Schluessel::erzeugen().unwrap().oeffentlich().1;
        assert!(ausweis_pruefen(&fremd, "ziel", "anrufer", "anna", 7, 1000, &sig).is_err());
    }

    use super::*;

    #[test]
    fn sache_hin_und_zurueck_und_nicht_vertauschbar() {
        let ich = Schluessel::erzeugen().unwrap();
        let ed = ich.oeffentlich().1;
        let konto = Kontoschluessel::neu();
        let (v, version) = sache_verschluesseln(&ich, &konto, "event", "t-1", b"termin").unwrap();
        assert_eq!(
            sache_oeffnen(&konto, "event", "t-1", version, &v, &ed).unwrap(),
            b"termin"
        );
        assert!(sache_oeffnen(&konto, "calendar", "t-1", version, &v, &ed).is_err());
        assert!(sache_oeffnen(&konto, "event", "t-2", version, &v, &ed).is_err());
    }

    #[test]
    fn verdeckter_abdruck_bleibt_ueber_den_wechsel() {
        let mut konto = Kontoschluessel::neu();
        let vorher = konto.abdruck_verdecken("t-1", "abc");
        konto.wechseln();
        assert_eq!(konto.abdruck_verdecken("t-1", "abc"), vorher);
        assert_ne!(konto.abdruck_verdecken("t-2", "abc"), vorher);
        assert_ne!(
            Kontoschluessel::neu().abdruck_verdecken("t-1", "abc"),
            vorher
        );
        assert_eq!(vorher.len(), 64);
    }

    fn geraet(id: i64, s: &Schluessel) -> Geraet {
        Geraet {
            geraet: id,
            ed25519: s.oeffentlich().1,
            aktiv: true,
        }
    }

    #[test]
    fn der_projektname_geht_nur_als_name_auf() {
        let tablet = Schluessel::erzeugen().unwrap();
        let ed = tablet.oeffentlich().1;
        let p = Kontoschluessel::neu();
        let (v, version) =
            projektfeld_verschluesseln(&tablet, &p, "p-1", "name", "Beispiel").unwrap();
        assert_eq!(
            projektfeld_oeffnen(&p, "p-1", "name", version, &v, &ed).unwrap(),
            "Beispiel"
        );
        assert!(projektfeld_oeffnen(&p, "p-2", "name", version, &v, &ed).is_err());
        // Nicht als Chat-Nachricht auszugeben.
        assert!(projekt_oeffnen(&p, "p-1", "name", version, &v, &ed).is_err());
    }

    /// Kiste fuer ein Mitglied, Chat hin und zurueck, nach dem Wechsel: Der
    /// Ausgetretene liest Neues nicht mehr, die Verbliebenen auch Altes.
    #[test]
    fn projektschluessel_kiste_chat_und_wechsel() {
        let tablet = Schluessel::erzeugen().unwrap();
        let ed = tablet.oeffentlich().1;
        let du = Kontoschluessel::neu();
        let geht = Kontoschluessel::neu();
        let mut p = Kontoschluessel::neu();

        let (c, sig) = projektkiste_packen(&tablet, &p, "p-1", 7, &du.oeffentlich()).unwrap();
        let bei_dir = projektkiste_oeffnen(&du, "p-1", 7, 1, &c, &sig, &ed).unwrap();
        // Nicht fuer ein anderes Projekt oder Mitglied.
        assert!(projektkiste_oeffnen(&du, "p-2", 7, 1, &c, &sig, &ed).is_err());
        assert!(projektkiste_oeffnen(&du, "p-1", 8, 1, &c, &sig, &ed).is_err());

        let (alt, v1) = projekt_verschluesseln(&tablet, &p, "p-1", "tiffy", "Hallo").unwrap();
        assert_eq!(v1, 1);
        assert_eq!(
            projekt_oeffnen(&bei_dir, "p-1", "tiffy", 1, &alt, &ed)
                .unwrap()
                .text,
            "Hallo"
        );
        assert!(projekt_oeffnen(&bei_dir, "p-1", "jemand", 1, &alt, &ed).is_err());
        assert!(projekt_oeffnen(&bei_dir, "p-2", "tiffy", 1, &alt, &ed).is_err());

        let (cg, sg) = projektkiste_packen(&tablet, &p, "p-1", 9, &geht.oeffentlich()).unwrap();
        let beim_gehenden = projektkiste_oeffnen(&geht, "p-1", 9, 1, &cg, &sg, &ed).unwrap();

        p.wechseln();
        let (c2, s2) = projektkiste_packen(&tablet, &p, "p-1", 7, &du.oeffentlich()).unwrap();
        let bei_dir2 = projektkiste_oeffnen(&du, "p-1", 7, 2, &c2, &s2, &ed).unwrap();
        let (neu, v2) = projekt_verschluesseln(&tablet, &p, "p-1", "tiffy", "Neu").unwrap();
        assert_eq!(v2, 2);
        assert_eq!(
            projekt_oeffnen(&bei_dir2, "p-1", "tiffy", 2, &neu, &ed)
                .unwrap()
                .text,
            "Neu"
        );
        assert_eq!(
            projekt_oeffnen(&bei_dir2, "p-1", "tiffy", 1, &alt, &ed)
                .unwrap()
                .text,
            "Hallo"
        );
        assert!(projekt_oeffnen(&beim_gehenden, "p-1", "tiffy", 2, &neu, &ed).is_err());
    }

    #[test]
    fn die_sicherheitsnummer_ist_auf_beiden_seiten_gleich() {
        let a = Kontoschluessel::neu().oeffentlich();
        let b = Kontoschluessel::neu().oeffentlich();
        let n = sicherheitsnummer(("tiffy", &a), ("beispiel", &b));
        assert_eq!(n, sicherheitsnummer(("Beispiel", &b), ("tiffy", &a)));
        assert_eq!(n.split(' ').count(), 12);
        assert!(n
            .split(' ')
            .all(|g| g.len() == 5 && g.chars().all(|c| c.is_ascii_digit())));
        // Ein anderer Schluessel, eine andere Nummer.
        let c = Kontoschluessel::neu().oeffentlich();
        assert_ne!(n, sicherheitsnummer(("tiffy", &a), ("beispiel", &c)));
    }

    /// Ein Kontakt: jedes eigene Geraet liest ihn, ein fremdes Konto nicht,
    /// und unter einer anderen uuid gilt die Unterschrift nicht.
    #[test]
    fn kontakt_liest_nur_das_eigene_konto() {
        let tablet = Schluessel::erzeugen().unwrap();
        let ich = Kontoschluessel::neu();
        let du = Kontoschluessel::neu();
        let (v, version) = kontakt_verschluesseln(&tablet, &ich, "k-1", b"{\"n\":1}").unwrap();
        assert_eq!(version, 1);
        let ed = tablet.oeffentlich().1;
        assert_eq!(
            kontakt_oeffnen(&ich, "k-1", 1, &v, &ed).unwrap(),
            b"{\"n\":1}"
        );
        assert!(kontakt_oeffnen(&du, "k-1", 1, &v, &ed).is_err());
        assert!(kontakt_oeffnen(&ich, "k-2", 1, &v, &ed).is_err());
        assert!(kontakt_oeffnen(&ich, "k-1", 2, &v, &ed).is_err());

        // Nach einem Wechsel: neue Fassung, die alte bleibt lesbar.
        let mut neu = Kontoschluessel::aus_text(&ich.als_text()).unwrap();
        neu.wechseln();
        let (v2, version2) = kontakt_verschluesseln(&tablet, &neu, "k-1", b"x").unwrap();
        assert_eq!(version2, 2);
        assert!(kontakt_oeffnen(&ich, "k-1", 2, &v2, &ed).is_err());
        assert_eq!(
            kontakt_oeffnen(&neu, "k-1", 1, &v, &ed).unwrap(),
            b"{\"n\":1}"
        );
    }

    /// Tablet schreibt; Handy (dasselbe Konto, freigegeben) und Empfaenger lesen.
    #[test]
    fn jedes_geraet_des_kontos_liest() {
        let tablet = Schluessel::erzeugen().unwrap();
        let handy = Schluessel::erzeugen().unwrap();
        let ich = Kontoschluessel::neu();
        let du = Kontoschluessel::neu();

        // Freigabe: Tablet packt dem Handy die Kiste.
        let (chiffrat, signatur) = kiste_packen(&tablet, &ich, 2, &handy.oeffentlich().0).unwrap();
        let am_handy =
            kiste_oeffnen(&handy, 2, 1, &chiffrat, &signatur, &tablet.oeffentlich().1).unwrap();
        assert_eq!(am_handy.oeffentlich(), ich.oeffentlich());

        let v = verschluesseln(
            &tablet,
            "ich",
            "du",
            "Hallo",
            None,
            &[du.oeffentlich(), ich.oeffentlich()],
        )
        .unwrap();
        for leser in [&ich, &am_handy, &du] {
            assert_eq!(
                oeffnen(leser, "ich", "du", &v, &geraet(1, &tablet))
                    .unwrap()
                    .text,
                "Hallo"
            );
        }
        let fremd = Kontoschluessel::neu();
        assert!(matches!(
            oeffnen(&fremd, "ich", "du", &v, &geraet(1, &tablet)),
            Err(Fehler::NichtFuerMich)
        ));
    }

    #[test]
    fn eine_kiste_fuer_ein_anderes_geraet_oder_gefaelscht_geht_nicht_auf() {
        let tablet = Schluessel::erzeugen().unwrap();
        let handy = Schluessel::erzeugen().unwrap();
        let boese = Schluessel::erzeugen().unwrap();
        let konto = Kontoschluessel::neu();
        let (chiffrat, signatur) =
            kiste_packen(&tablet, &konto, 2, &handy.oeffentlich().0).unwrap();

        // Als waere sie fuer Geraet 3: Unterschrift passt nicht.
        assert!(matches!(
            kiste_oeffnen(&handy, 3, 1, &chiffrat, &signatur, &tablet.oeffentlich().1),
            Err(Fehler::Unterschrift)
        ));
        // Unterschrieben angeblich von einem anderen Geraet: passt nicht.
        assert!(matches!(
            kiste_oeffnen(&handy, 2, 1, &chiffrat, &signatur, &boese.oeffentlich().1),
            Err(Fehler::Unterschrift)
        ));
        // Ein drittes Geraet kann sie nicht oeffnen.
        assert!(matches!(
            kiste_oeffnen(&boese, 2, 1, &chiffrat, &signatur, &tablet.oeffentlich().1),
            Err(Fehler::NichtFuerMich)
        ));
    }

    /// Nach einem Wechsel bleibt der alte Verlauf lesbar.
    #[test]
    fn nach_dem_wechsel_bleibt_alles_lesbar() {
        let tablet = Schluessel::erzeugen().unwrap();
        let mut ich = Kontoschluessel::neu();
        let du = Kontoschluessel::neu();
        let alt = verschluesseln(&tablet, "du", "ich", "Alt", None, &[ich.oeffentlich()]).unwrap();

        assert_eq!(ich.wechseln(), 2);
        let neu = verschluesseln(&tablet, "du", "ich", "Neu", None, &[ich.oeffentlich()]).unwrap();
        let wieder = Kontoschluessel::aus_text(&ich.als_text()).unwrap();
        for (v, t) in [(&alt, "Alt"), (&neu, "Neu")] {
            assert_eq!(
                oeffnen(&wieder, "du", "ich", v, &geraet(1, &tablet))
                    .unwrap()
                    .text,
                t
            );
        }
        let _ = du;
    }

    #[test]
    fn umgeleitet_oder_gefaelscht_scheitert() {
        let tablet = Schluessel::erzeugen().unwrap();
        let boese = Schluessel::erzeugen().unwrap();
        let du = Kontoschluessel::neu();
        let v = verschluesseln(&tablet, "ich", "du", "Geheim", None, &[du.oeffentlich()]).unwrap();

        assert!(matches!(
            oeffnen(&du, "ich", "dritter", &v, &geraet(1, &tablet)),
            Err(Fehler::Unterschrift)
        ));
        assert!(matches!(
            oeffnen(&du, "ich", "du", &v, &geraet(1, &boese)),
            Err(Fehler::Unterschrift)
        ));
    }

    #[test]
    fn schluessel_ueberstehen_den_tresor() {
        let s = Schluessel::erzeugen().unwrap();
        assert_eq!(
            Schluessel::aus_text(&s.als_text()).unwrap().oeffentlich(),
            s.oeffentlich()
        );
    }

    #[test]
    fn ein_anhang_reist_mit_und_laesst_sich_nicht_tauschen() {
        let tablet = Schluessel::erzeugen().unwrap();
        let ich = Kontoschluessel::neu();
        let du = Kontoschluessel::neu();
        let datei = b"%PDF-1.7 beispiel".to_vec();

        let (chiffrat, info) =
            anhang_verschluesseln("plan.pdf", "application/pdf", &datei).unwrap();
        assert_ne!(chiffrat, datei);
        let v = verschluesseln(
            &tablet,
            "ich",
            "du",
            "Anbei",
            Some(info),
            &[du.oeffentlich(), ich.oeffentlich()],
        )
        .unwrap();

        let drinnen = oeffnen(&du, "ich", "du", &v, &geraet(1, &tablet)).unwrap();
        let info = drinnen.anhang.expect("Anhang im Inhalt");
        assert_eq!(
            (info.name.as_str(), info.groesse),
            ("plan.pdf", datei.len() as u64)
        );
        assert_eq!(anhang_oeffnen(&chiffrat, &info).unwrap(), datei);

        // Eine andere Datei an derselben Stelle: Die Pruefsumme passt nicht.
        let (fremd, _) = anhang_verschluesseln("x", "x", b"anderes").unwrap();
        assert!(matches!(
            anhang_oeffnen(&fremd, &info),
            Err(Fehler::Unterschrift)
        ));
    }
}
