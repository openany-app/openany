//! Die Bytes der Dateien -- abgelegt nach ihrem Abdruck.
//!
//! **Ein Inhalt liegt einmal**, wie viele Zeilen ihn auch nennen: eine Kopie
//! in zwei Ordnern, dieselbe Datei in Dateien und Dokumenten. Und der Name
//! ist zugleich die Pruefung: Was unter `ab/abcdef…` liegt, hat genau diesen
//! sha256, sonst waere es nicht dorthin gekommen.
//!
//! **Atomar.** Geschrieben wird in `.laden/`, erst nach dem letzten Byte wird
//! gerechnet, synchronisiert und umbenannt. Ein Programm, das mittendrin
//! stirbt (Akku, Wischen), hinterlaesst eine halbe Datei in `.laden/` und nie
//! eine halbe unter einem Abdruck, der Vollstaendigkeit behauptet.
//!
//! Getrennt von der SQLite-Datei, weil Bytes dort nicht hingehoeren: Eine
//! 2-GB-Aufnahme in einer Tabelle machte jede Sicherung und jedes
//! Aufraeumen der Datenbank zu einer Sache von Minuten.

use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

pub struct Inhalte {
    wurzel: PathBuf,
}

/// Ein Inhalt, der gerade ankommt -- stueckweise, womoeglich ueber viele
/// Aufrufe hinweg.
pub struct Ladung {
    pfad: PathBuf,
    datei: File,
    rechner: Sha256,
    groesse: u64,
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn ist_abdruck(a: &str) -> bool {
    a.len() == 64 && a.bytes().all(|b| b.is_ascii_hexdigit())
}

impl Inhalte {
    pub fn oeffnen(wurzel: impl Into<PathBuf>) -> io::Result<Self> {
        let wurzel = wurzel.into();
        fs::create_dir_all(wurzel.join(".laden"))?;
        Ok(Self { wurzel })
    }

    /// Wo ein Inhalt liegt -- ob es ihn gibt oder nicht. `None` bei einem
    /// Abdruck, der keiner ist (sonst liesse sich mit `../` hinauslaufen).
    pub fn pfad(&self, abdruck: &str) -> Option<PathBuf> {
        ist_abdruck(abdruck).then(|| {
            let a = abdruck.to_ascii_lowercase();
            self.wurzel.join(&a[..2]).join(&a)
        })
    }

    pub fn hat(&self, abdruck: &str) -> bool {
        self.pfad(abdruck).is_some_and(|p| p.is_file())
    }

    /// Einen Ausschnitt lesen -- fuer Vorschau und fuer ein anderes Geraet,
    /// das stueckweise holt.
    pub fn lesen(&self, abdruck: &str, von: u64, hoechstens: usize) -> io::Result<Vec<u8>> {
        let pfad = self
            .pfad(abdruck)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "no fingerprint"))?;
        let mut datei = File::open(pfad)?;
        datei.seek(SeekFrom::Start(von))?;
        let mut puffer = Vec::with_capacity(hoechstens.min(8 * 1024 * 1024));
        datei.take(hoechstens as u64).read_to_end(&mut puffer)?;
        Ok(puffer)
    }

    pub fn entfernen(&self, abdruck: &str) -> io::Result<()> {
        match self.pfad(abdruck) {
            Some(p) if p.exists() => fs::remove_file(p),
            _ => Ok(()),
        }
    }

    /// Eine neue Ladung beginnen.
    pub fn ladung(&self) -> io::Result<Ladung> {
        let pfad = self
            .wurzel
            .join(".laden")
            .join(uuid::Uuid::new_v4().simple().to_string());
        Ok(Ladung {
            datei: File::create(&pfad)?,
            pfad,
            rechner: Sha256::new(),
            groesse: 0,
        })
    }

    /// Die Ladung ablegen: Gibt Abdruck und Groesse zurueck. Liegt derselbe
    /// Inhalt schon da, bleibt der vorhandene und die Ladung verschwindet.
    pub fn ablegen(&self, ladung: Ladung) -> io::Result<(String, u64)> {
        let Ladung {
            pfad,
            datei,
            rechner,
            groesse,
        } = ladung;
        datei.sync_all()?;
        drop(datei);

        let abdruck = hex(&rechner.finalize());
        let ziel = self.pfad(&abdruck).expect("computed here");
        if ziel.is_file() {
            fs::remove_file(&pfad)?;
        } else {
            fs::create_dir_all(ziel.parent().expect("has a folder"))?;
            fs::rename(&pfad, &ziel)?;
        }
        Ok((abdruck, groesse))
    }

    /// Eine Ladung verwerfen.
    pub fn verwerfen(&self, ladung: Ladung) {
        let _ = fs::remove_file(&ladung.pfad);
    }

    /// Wie viel die Ablage belegt, in Bytes.
    pub fn belegt(&self) -> u64 {
        fn summe(p: &Path) -> u64 {
            fs::read_dir(p)
                .map(|e| {
                    e.flatten()
                        .map(|e| match e.metadata() {
                            Ok(m) if m.is_dir() => summe(&e.path()),
                            Ok(m) => m.len(),
                            Err(_) => 0,
                        })
                        .sum()
                })
                .unwrap_or(0)
        }
        summe(&self.wurzel)
    }

    /// Alles wegraeumen, was keine Zeile mehr nennt, und liegengebliebene
    /// Ladungen. Gibt die Zahl der entfernten Inhalte zurueck.
    pub fn aufraeumen(&self, benutzt: &std::collections::BTreeSet<String>) -> io::Result<usize> {
        let mut weg = 0;
        for eintrag in fs::read_dir(&self.wurzel)?.flatten() {
            let pfad = eintrag.path();
            let name = eintrag.file_name().to_string_lossy().to_string();
            if name == ".laden" {
                for l in fs::read_dir(&pfad)?.flatten() {
                    let _ = fs::remove_file(l.path());
                }
                continue;
            }
            if !pfad.is_dir() {
                continue;
            }
            for datei in fs::read_dir(&pfad)?.flatten() {
                let a = datei.file_name().to_string_lossy().to_string();
                if ist_abdruck(&a) && !benutzt.contains(&a) {
                    fs::remove_file(datei.path())?;
                    weg += 1;
                }
            }
        }
        Ok(weg)
    }
}

impl Ladung {
    pub fn schreiben(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.datei.write_all(bytes)?;
        self.rechner.update(bytes);
        self.groesse += bytes.len() as u64;
        Ok(())
    }

    pub fn groesse(&self) -> u64 {
        self.groesse
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stueckweise_geladen_liegt_unter_seinem_abdruck() {
        let ordner = tempfile::tempdir().unwrap();
        let inhalte = Inhalte::oeffnen(ordner.path()).unwrap();

        let mut l = inhalte.ladung().unwrap();
        l.schreiben(b"Hallo ").unwrap();
        l.schreiben(b"Welt").unwrap();
        let (abdruck, groesse) = inhalte.ablegen(l).unwrap();

        assert_eq!(abdruck, hex(&Sha256::digest(b"Hallo Welt")));
        assert_eq!(groesse, 10);
        assert!(inhalte.hat(&abdruck));
        assert_eq!(inhalte.lesen(&abdruck, 6, 100).unwrap(), b"Welt");
    }

    #[test]
    fn derselbe_inhalt_liegt_nur_einmal() {
        let ordner = tempfile::tempdir().unwrap();
        let inhalte = Inhalte::oeffnen(ordner.path()).unwrap();
        for _ in 0..2 {
            let mut l = inhalte.ladung().unwrap();
            l.schreiben(b"doppelt").unwrap();
            inhalte.ablegen(l).unwrap();
        }
        assert_eq!(inhalte.belegt(), 7);
    }

    #[test]
    fn ein_falscher_abdruck_fuehrt_nirgendwohin() {
        let ordner = tempfile::tempdir().unwrap();
        let inhalte = Inhalte::oeffnen(ordner.path()).unwrap();
        assert!(inhalte.pfad("../../etc/passwd").is_none());
        assert!(inhalte.lesen("abc", 0, 10).is_err());
    }

    #[test]
    fn aufraeumen_laesst_stehen_was_eine_zeile_nennt() {
        let ordner = tempfile::tempdir().unwrap();
        let inhalte = Inhalte::oeffnen(ordner.path()).unwrap();
        let mut abdruecke = Vec::new();
        for text in [&b"bleibt"[..], b"geht"] {
            let mut l = inhalte.ladung().unwrap();
            l.schreiben(text).unwrap();
            abdruecke.push(inhalte.ablegen(l).unwrap().0);
        }
        let halb = inhalte.ladung().unwrap(); // liegengeblieben
        drop(halb);

        let benutzt = [abdruecke[0].clone()].into_iter().collect();
        assert_eq!(inhalte.aufraeumen(&benutzt).unwrap(), 1);
        assert!(inhalte.hat(&abdruecke[0]));
        assert!(!inhalte.hat(&abdruecke[1]));
        assert_eq!(
            fs::read_dir(ordner.path().join(".laden")).unwrap().count(),
            0
        );
    }
}
