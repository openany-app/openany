//! Die Identitaet dieses Geraets: ein selbst erzeugtes Zertifikat, dessen
//! SHA-256-Fingerabdruck das Geraet ausweist.
//!
//! **Einmal erzeugt, dann immer dasselbe.** Der Fingerabdruck ist der Name,
//! unter dem ein anderes Geraet sich dieses merkt (Paaren, Schritt 4). Ein
//! neues Zertifikat bei jedem Start machte aus jedem Neustart ein fremdes
//! Geraet.
//!
//! **Vorlaeufig als Datei** im App-Ordner, mit Rechten nur fuer das Programm.
//! Der private Schluessel gehoert in den Keystore bzw. Schluesselbund; das
//! kommt vor jeder Weitergabe (APK 0.1, "bewusst nicht in 0.1").

use localsend::crypto::cert::generate_self_signed;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum IdentitaetFehler {
    #[error("The device identity could not be read or written: {0}")]
    Datei(#[from] std::io::Error),
    #[error("The device identity is unreadable: {0}")]
    Unlesbar(#[from] serde_json::Error),
    #[error("The certificate could not be created: {0}")]
    Erzeugen(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Identitaet {
    pub zertifikat_pem: String,
    pub schluessel_pem: String,
    /// SHA-256 ueber das Zertifikat, gross und hexadezimal -- so, wie
    /// LocalSend ihn im Protokoll fuehrt.
    pub fingerabdruck: String,
}

impl Identitaet {
    /// Aus der Datei lesen oder, wenn es keine gibt, erzeugen und ablegen.
    pub fn laden_oder_erzeugen(pfad: &Path) -> Result<Self, IdentitaetFehler> {
        if pfad.exists() {
            return Ok(serde_json::from_str(&std::fs::read_to_string(pfad)?)?);
        }

        let neu = Self::erzeugen()?;
        if let Some(ordner) = pfad.parent() {
            std::fs::create_dir_all(ordner)?;
        }
        std::fs::write(pfad, serde_json::to_string_pretty(&neu)?)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(pfad, std::fs::Permissions::from_mode(0o600))?;
        }
        Ok(neu)
    }

    /// Eine neue Identitaet, ohne sie abzulegen -- wohin, entscheidet der
    /// Aufrufer (im Programm: der Tresor).
    pub fn erzeugen() -> Result<Self, IdentitaetFehler> {
        let c = generate_self_signed().map_err(|e| IdentitaetFehler::Erzeugen(e.to_string()))?;
        Ok(Self {
            zertifikat_pem: c.certificate_pem,
            schluessel_pem: c.private_key_pem,
            fingerabdruck: c.fingerprint,
        })
    }

    /// Die ersten zwoelf Zeichen, in Vierergruppen -- zum Vorlesen und
    /// Vergleichen, nicht zum Pruefen.
    pub fn kurz(&self) -> String {
        self.fingerabdruck
            .chars()
            .take(12)
            .collect::<Vec<_>>()
            .chunks(4)
            .map(|g| g.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn einmal_erzeugt_dann_immer_dieselbe() {
        let ordner = tempfile::tempdir().unwrap();
        let pfad = ordner.path().join("nah/identitaet.json");

        let erste = Identitaet::laden_oder_erzeugen(&pfad).unwrap();
        let zweite = Identitaet::laden_oder_erzeugen(&pfad).unwrap();

        assert_eq!(erste, zweite);
        assert_eq!(erste.fingerabdruck.len(), 64);
        assert_eq!(erste.kurz().len(), 14);
    }
}
