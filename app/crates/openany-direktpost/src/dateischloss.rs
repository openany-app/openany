//! Dateien, Bilder und ihre Vorschauen verschluesseln (docs/plan-dateien-e2e.md).
//!
//! **Ein Schluessel je Inhalt.** Jeder Inhalt bekommt eine eigene
//! age-Identitaet; sie steht im Kopf der Datei (der mit dem Konto- bzw.
//! Freigabeschluessel verschluesselt ist). Wer eine Datei in ein geteiltes
//! Album schiebt, verpackt nur den Kopf neu -- die Bytes bleiben.
//!
//! **Stroemend, von Datei zu Datei.** Ein Tablet haelt keine 400 MB im
//! Speicher. age verschluesselt ohnehin in 64-KB-Stuecken; hier wird nur
//! nie mehr als ein Puffer gelesen. Unterwegs entsteht der Abdruck (sha256)
//! der jeweils anderen Seite -- er benennt das Chiffrat beim Server und den
//! Klartext auf dem Geraet.
//!
//! Die Vorschau eines Bilds geht mit DEMSELBEN Schluessel zu (klein, im
//! Speicher): ein Schluessel im Kopf genuegt fuer beides.

use crate::Fehler;
use age::secrecy::ExposeSecret;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::Path;

/// Was ueber einen verschluesselten Inhalt in den Kopf und in die eigene
/// Buchfuehrung gehoert.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dateischluessel {
    /// age-Identitaet (`AGE-SECRET-KEY-1…`) nur fuer diesen Inhalt.
    pub schluessel: String,
    /// sha256 des Chiffrats, hex -- so heisst er beim Server.
    pub chiffrat_abdruck: String,
    pub chiffrat_groesse: u64,
}

fn hex(d: impl AsRef<[u8]>) -> String {
    d.as_ref().iter().map(|b| format!("{b:02x}")).collect()
}

/// Ein Schreiber, der unterwegs mitzaehlt und mitrechnet.
struct Mitschrift<W: Write> {
    innen: W,
    hash: Sha256,
    groesse: u64,
}

impl<W: Write> Write for Mitschrift<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.innen.write(buf)?;
        self.hash.update(&buf[..n]);
        self.groesse += n as u64;
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.innen.flush()
    }
}

/// Einen neuen Schluessel erzeugen.
pub fn neuer_schluessel() -> String {
    age::x25519::Identity::generate()
        .to_string()
        .expose_secret()
        .to_string()
}

fn identitaet(schluessel: &str) -> Result<age::x25519::Identity, Fehler> {
    schluessel
        .parse::<age::x25519::Identity>()
        .map_err(|e| Fehler::Schluessel(e.to_string()))
}

fn verschluesseler(schluessel: &str) -> Result<age::Encryptor, Fehler> {
    let r = identitaet(schluessel)?.to_public();
    age::Encryptor::with_recipients(std::iter::once(&r as &dyn age::Recipient))
        .map_err(|e| Fehler::Verschluesseln(e.to_string()))
}

/// Eine Datei verschluesseln, `quelle` → `ziel`, mit `schluessel`
/// (meist frisch aus [`neuer_schluessel`]; derselbe Inhalt darf seinen
/// alten behalten).
pub fn datei_verschluesseln(
    quelle: &Path,
    ziel: &Path,
    schluessel: &str,
) -> Result<Dateischluessel, Fehler> {
    let fehler = |e: io::Error| Fehler::Verschluesseln(e.to_string());
    let mut lesen = BufReader::new(File::open(quelle).map_err(fehler)?);
    let mitschrift = Mitschrift {
        innen: BufWriter::new(File::create(ziel).map_err(fehler)?),
        hash: Sha256::new(),
        groesse: 0,
    };
    let mut schreiber = verschluesseler(schluessel)?
        .wrap_output(mitschrift)
        .map_err(fehler)?;
    io::copy(&mut lesen, &mut schreiber).map_err(fehler)?;
    let mut mitschrift = schreiber.finish().map_err(fehler)?;
    mitschrift.flush().map_err(fehler)?;
    Ok(Dateischluessel {
        schluessel: schluessel.to_string(),
        chiffrat_abdruck: hex(mitschrift.hash.finalize()),
        chiffrat_groesse: mitschrift.groesse,
    })
}

/// Eine Datei oeffnen, `quelle` (Chiffrat) → `ziel` (Klartext). Ist der
/// Abdruck des Chiffrats bekannt, wird er zuerst geprueft, dann
/// entschluesselt. Zurueck kommt der Abdruck des Klartexts -- der Aufrufer
/// vergleicht ihn mit dem Kopf (das ist die eigentliche Pruefung; age ist
/// ohnehin authentisiert).
pub fn datei_oeffnen(
    quelle: &Path,
    ziel: &Path,
    schluessel: &str,
    chiffrat_abdruck: Option<&str>,
) -> Result<String, Fehler> {
    let fehler = |e: io::Error| Fehler::Inhalt(e.to_string());
    if let Some(chiffrat_abdruck) = chiffrat_abdruck {
        let mut h = Sha256::new();
        let mut f = BufReader::new(File::open(quelle).map_err(fehler)?);
        io::copy(&mut f, &mut HashSchreiber(&mut h)).map_err(fehler)?;
        if hex(h.finalize()) != chiffrat_abdruck {
            return Err(Fehler::Unterschrift);
        }
    }
    let id = identitaet(schluessel)?;
    let entschluesseler = age::Decryptor::new(BufReader::new(File::open(quelle).map_err(fehler)?))
        .map_err(|e| Fehler::Inhalt(e.to_string()))?;
    let mut leser = entschluesseler
        .decrypt(std::iter::once(&id as &dyn age::Identity))
        .map_err(|_| Fehler::NichtFuerMich)?;
    let mut mitschrift = Mitschrift {
        innen: BufWriter::new(File::create(ziel).map_err(fehler)?),
        hash: Sha256::new(),
        groesse: 0,
    };
    io::copy(&mut leser, &mut mitschrift).map_err(fehler)?;
    mitschrift.flush().map_err(fehler)?;
    Ok(hex(mitschrift.hash.finalize()))
}

struct HashSchreiber<'a>(&'a mut Sha256);

impl Write for HashSchreiber<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.update(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Kleine Bytes (die Vorschau) mit demselben Schluessel -- im Speicher.
pub fn bytes_verschluesseln(schluessel: &str, klartext: &[u8]) -> Result<Vec<u8>, Fehler> {
    let fehler = |e: io::Error| Fehler::Verschluesseln(e.to_string());
    let mut chiffrat = Vec::new();
    let mut w = verschluesseler(schluessel)?
        .wrap_output(&mut chiffrat)
        .map_err(fehler)?;
    w.write_all(klartext).map_err(fehler)?;
    w.finish().map_err(fehler)?;
    Ok(chiffrat)
}

pub fn bytes_oeffnen(schluessel: &str, chiffrat: &[u8]) -> Result<Vec<u8>, Fehler> {
    let id = identitaet(schluessel)?;
    let d = age::Decryptor::new(chiffrat).map_err(|e| Fehler::Inhalt(e.to_string()))?;
    let mut r = d
        .decrypt(std::iter::once(&id as &dyn age::Identity))
        .map_err(|_| Fehler::NichtFuerMich)?;
    let mut klar = Vec::new();
    r.read_to_end(&mut klar)
        .map_err(|e| Fehler::Inhalt(e.to_string()))?;
    Ok(klar)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eine_grosse_datei_geht_hin_und_zurueck() {
        let d = tempfile::tempdir().unwrap();
        let (klar, zu, auf) = (d.path().join("k"), d.path().join("z"), d.path().join("a"));
        // Mehr als ein age-Stueck (64 KB) und ein Puffer.
        let inhalt: Vec<u8> = (0..300_000u32).map(|i| (i % 251) as u8).collect();
        std::fs::write(&klar, &inhalt).unwrap();

        let s = datei_verschluesseln(&klar, &zu, &neuer_schluessel()).unwrap();
        assert_eq!(s.chiffrat_groesse, std::fs::metadata(&zu).unwrap().len());
        assert_ne!(std::fs::read(&zu).unwrap()[..64], inhalt[..64]);

        let abdruck = datei_oeffnen(&zu, &auf, &s.schluessel, Some(&s.chiffrat_abdruck)).unwrap();
        assert_eq!(std::fs::read(&auf).unwrap(), inhalt);
        assert_eq!(abdruck, hex(Sha256::digest(&inhalt)));
    }

    #[test]
    fn ein_veraendertes_chiffrat_geht_nicht_auf() {
        let d = tempfile::tempdir().unwrap();
        let (klar, zu, auf) = (d.path().join("k"), d.path().join("z"), d.path().join("a"));
        std::fs::write(&klar, b"Beispiel").unwrap();
        let s = datei_verschluesseln(&klar, &zu, &neuer_schluessel()).unwrap();
        let mut c = std::fs::read(&zu).unwrap();
        let n = c.len() - 1;
        c[n] ^= 1;
        std::fs::write(&zu, &c).unwrap();

        assert!(matches!(
            datei_oeffnen(&zu, &auf, &s.schluessel, Some(&s.chiffrat_abdruck)),
            Err(Fehler::Unterschrift)
        ));
    }

    #[test]
    fn ein_fremder_schluessel_oeffnet_nichts() {
        let d = tempfile::tempdir().unwrap();
        let (klar, zu, auf) = (d.path().join("k"), d.path().join("z"), d.path().join("a"));
        std::fs::write(&klar, b"Beispiel").unwrap();
        let s = datei_verschluesseln(&klar, &zu, &neuer_schluessel()).unwrap();

        assert!(datei_oeffnen(&zu, &auf, &neuer_schluessel(), Some(&s.chiffrat_abdruck)).is_err());
    }

    #[test]
    fn die_vorschau_nimmt_denselben_schluessel() {
        let s = neuer_schluessel();
        let c = bytes_verschluesseln(&s, b"vorschau").unwrap();
        assert_eq!(bytes_oeffnen(&s, &c).unwrap(), b"vorschau");
    }
}
