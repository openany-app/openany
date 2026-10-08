//! Wo das Gerätetoken liegt.
//!
//! **Nicht neben der Warteschlange.** Die SQLite-Datei ist eine Arbeitsdatei;
//! ein Ausweis gehoert dorthin, wo das Betriebssystem ihn schuetzt - auf
//! Android in den Keystore, auf dem Schreibtisch in den Schluesselbund. Beide
//! liegen hinter einer Plattformschnittstelle, die es hier noch nicht gibt.
//!
//! Diese Datei zieht deshalb die Grenze und nicht mehr: ein Trait, an dem die
//! Schale haengt, und eine Umsetzung als Datei, mit der sich auf dem
//! Entwicklungsrechner arbeiten laesst. **Die Dateifassung ist ausdruecklich
//! keine sichere Ablage** - sie sagt das selbst, damit sie niemand fuer eine
//! haelt.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Der Ausweis eines gekoppelten Geraets.
pub trait Tokenspeicher {
    fn lesen(&self) -> io::Result<Option<String>>;

    fn schreiben(&self, token: &str) -> io::Result<()>;

    /// Beim Widerruf: Ein Token, das der Server nicht mehr kennt, hat auf dem
    /// Geraet nichts mehr zu suchen.
    fn vergessen(&self) -> io::Result<()>;
}

/// Eine Ablage als Datei - **fuer die Entwicklung**.
///
/// Wer sie in einem ausgelieferten Programm benutzt, legt einen dauerhaft
/// gueltigen Ausweis unverschluesselt auf ein fremdes Geraet. Die Datei
/// bekommt immerhin Rechte 0600, damit es nicht auch noch beilaeufig
/// passiert.
#[derive(Debug, Clone)]
pub struct Dateispeicher {
    pfad: PathBuf,
}

impl Dateispeicher {
    pub fn neu(pfad: impl AsRef<Path>) -> Self {
        Self {
            pfad: pfad.as_ref().to_path_buf(),
        }
    }
}

impl Tokenspeicher for Dateispeicher {
    fn lesen(&self) -> io::Result<Option<String>> {
        match fs::read_to_string(&self.pfad) {
            Ok(inhalt) => {
                let token = inhalt.trim().to_string();

                Ok(if token.is_empty() { None } else { Some(token) })
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn schreiben(&self, token: &str) -> io::Result<()> {
        if let Some(ordner) = self.pfad.parent() {
            fs::create_dir_all(ordner)?;
        }

        fs::write(&self.pfad, token)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            fs::set_permissions(&self.pfad, fs::Permissions::from_mode(0o600))?;
        }

        Ok(())
    }

    fn vergessen(&self) -> io::Result<()> {
        match fs::remove_file(&self.pfad) {
            // Nicht da ist der gewuenschte Zustand, kein Fehler.
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            anderes => anderes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ohne_datei_gibt_es_kein_token() {
        let ordner = tempfile::tempdir().unwrap();
        let speicher = Dateispeicher::neu(ordner.path().join("ausweis"));

        assert_eq!(speicher.lesen().unwrap(), None);
    }

    #[test]
    fn schreibt_und_liest_wieder() {
        let ordner = tempfile::tempdir().unwrap();
        let speicher = Dateispeicher::neu(ordner.path().join("tief/ausweis"));

        speicher.schreiben("geheim").unwrap();

        assert_eq!(speicher.lesen().unwrap().as_deref(), Some("geheim"));
    }

    #[test]
    fn vergisst_und_beschwert_sich_nicht_ueber_das_zweite_mal() {
        let ordner = tempfile::tempdir().unwrap();
        let speicher = Dateispeicher::neu(ordner.path().join("ausweis"));

        speicher.schreiben("geheim").unwrap();
        speicher.vergessen().unwrap();

        assert_eq!(speicher.lesen().unwrap(), None);
        // Nicht da ist der gewuenschte Zustand.
        speicher.vergessen().unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn legt_die_datei_nicht_fuer_alle_lesbar_an() {
        use std::os::unix::fs::PermissionsExt;

        let ordner = tempfile::tempdir().unwrap();
        let pfad = ordner.path().join("ausweis");
        Dateispeicher::neu(&pfad).schreiben("geheim").unwrap();

        let modus = fs::metadata(&pfad).unwrap().permissions().mode() & 0o777;

        assert_eq!(modus, 0o600);
    }
}
