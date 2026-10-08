//! Die Sicherungsdatei: alles von openany auf diesem Geraet in einer
//! verschluesselten Datei, die auf einen Stick passt und auf einem anderen
//! Geraet wieder geoeffnet wird.
//!
//! Sie ersetzt den alten Stick (08.10.2026): Statt eines lauffaehigen
//! openany auf dem Stick liegt dort nur noch eine Datei.
//!
//! ## Das Format
//!
//! Eine **age-Datei** mit Passphrase (scrypt), darin ein **tar**. Beides ist
//! offen beschrieben, und das ist Absicht: Wer openany nicht mehr hat, kommt
//! mit `age -d datei.age | tar x` trotzdem an seine Daten. Der erste Eintrag
//! im tar ist immer `openany-sicherung.json` (der [`Kopf`]); was danach kommt,
//! bestimmt die Schale -- diese Crate weiss nicht, was eine Notiz ist.
//!
//! ## Die Passphrase
//!
//! Vorgeschlagen werden sechs Woerter aus der langen Liste der EFF (7776
//! Woerter, rund 77 Bit). Eingegebenes wird vor dem Verschluesseln
//! [geglaettet](passphrase_glaetten) -- klein, einfache Leerzeichen --, damit
//! die Grossschreibung einer Handytastatur am Satzanfang nicht zur
//! verlorenen Sicherung wird.
//!
//! Wortliste: EFF, „Large Wordlist for Passphrases", CC BY 3.0 US,
//! <https://www.eff.org/dice>.

use age::secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

/// Was diese Fassung schreibt. Liest sie eine hoehere, sagt sie das, statt
/// halb zu verstehen.
pub const FORMAT: u32 = 1;

/// Der erste Eintrag jeder Sicherung.
pub const KOPF: &str = "openany-sicherung.json";

/// Wie viele Woerter ein Vorschlag hat.
pub const WOERTER: usize = 6;

/// Kuerzer darf eine eigene Passphrase nicht sein (nach dem Glaetten).
pub const MINDESTLAENGE: usize = 12;

/// scrypt mit N = 2^17: 128 MiB und auf einem Telefon um eine Sekunde.
/// FEST und nicht je Geraet gemessen -- sonst schriebe ein schneller
/// Rechner eine Datei, die ein altes Telefon nicht mehr aufbekommt.
const AUFWAND: u8 = 17;

/// Was ein Oeffnen hoechstens hinnimmt. Eine fremde Datei mit riesigem
/// Aufwand soll das Telefon nicht stundenlang beschaeftigen.
const AUFWAND_HOECHSTENS: u8 = 20;

#[derive(Debug, thiserror::Error)]
pub enum SicherungFehler {
    #[error("Wrong passphrase.")]
    FalschePassphrase,
    #[error("This is not an openany backup.")]
    KeineSicherung,
    #[error(
        "This backup comes from a newer version of openany (format {0}). Please update the app."
    )]
    NeueresFormat(u32),
    #[error("The backup is damaged: {0}")]
    Beschaedigt(String),
    #[error("The passphrase is too short (at least {MINDESTLAENGE} characters).")]
    ZuKurz,
    #[error(transparent)]
    Io(#[from] io::Error),
}

pub type Ergebnis<T> = Result<T, SicherungFehler>;

/// Was ueber die Sicherung selbst gesagt wird.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Kopf {
    pub format: u32,
    /// ISO-8601, wie ueberall in openany.
    pub erstellt: String,
    /// Welches Programm in welcher Fassung sie geschrieben hat.
    pub programm: String,
    /// Der Name des Geraets, von dem sie stammt -- zum Wiedererkennen.
    #[serde(default)]
    pub geraet: String,
}

impl Kopf {
    pub fn jetzt(programm: &str, geraet: &str) -> Self {
        Self {
            format: FORMAT,
            erstellt: chrono::Utc::now().to_rfc3339(),
            programm: programm.to_string(),
            geraet: geraet.to_string(),
        }
    }
}

/* ── Passphrase ─────────────────────────────────────────────────────────── */

fn woerter() -> impl Iterator<Item = &'static str> {
    include_str!("woerter.txt").lines()
}

/// Sechs Woerter, durch Leerzeichen getrennt.
pub fn passphrase_vorschlagen() -> String {
    let liste: Vec<&str> = woerter().collect();
    let n = liste.len() as u32;
    // Gleichverteilt: Was ueber dem letzten vollen Vielfachen liegt, wird
    // verworfen, statt mit `%` die ersten Woerter zu bevorzugen.
    let grenze = u32::MAX - u32::MAX % n;
    let mut gewaehlt = Vec::with_capacity(WOERTER);
    while gewaehlt.len() < WOERTER {
        let mut b = [0u8; 4];
        getrandom::fill(&mut b).expect("no random source");
        let z = u32::from_le_bytes(b);
        if z < grenze {
            gewaehlt.push(liste[(z % n) as usize]);
        }
    }
    gewaehlt.join(" ")
}

/// Klein und mit einfachen Leerzeichen -- so wird verschluesselt und so
/// wird geoeffnet.
pub fn passphrase_glaetten(p: &str) -> String {
    p.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn geheim(p: &str) -> Ergebnis<SecretString> {
    let p = passphrase_glaetten(p);
    if p.chars().count() < MINDESTLAENGE {
        return Err(SicherungFehler::ZuKurz);
    }
    Ok(SecretString::from(p))
}

/* ── Schreiben ──────────────────────────────────────────────────────────── */

/// Eine Sicherung schreiben.
///
/// `beilagen` sind kleine Teile aus dem Speicher (etwa ein entschluesselter
/// Ausweis), `dateien` liegen auf der Platte und werden gestreamt -- eine
/// Galerie passt nicht in den Arbeitsspeicher eines Telefons. Beide tragen
/// ihren Namen im Archiv mit, und zwar mit `/`.
pub fn schreiben<W: Write>(
    ziel: W,
    passphrase: &str,
    kopf: &Kopf,
    beilagen: &[(String, Vec<u8>)],
    dateien: &[(String, PathBuf)],
) -> Ergebnis<W> {
    schreiben_mit(ziel, passphrase, AUFWAND, kopf, beilagen, dateien)
}

fn schreiben_mit<W: Write>(
    ziel: W,
    passphrase: &str,
    aufwand: u8,
    kopf: &Kopf,
    beilagen: &[(String, Vec<u8>)],
    dateien: &[(String, PathBuf)],
) -> Ergebnis<W> {
    let mut empfaenger = age::scrypt::Recipient::new(geheim(passphrase)?);
    empfaenger.set_work_factor(aufwand);
    let verschluesselt =
        age::Encryptor::with_recipients(std::iter::once(&empfaenger as &dyn age::Recipient))
            .map_err(|e| SicherungFehler::Beschaedigt(e.to_string()))?
            .wrap_output(ziel)?;

    let mut tar = tar::Builder::new(verschluesselt);
    tar.mode(tar::HeaderMode::Deterministic);

    let kopf = serde_json::to_vec_pretty(kopf).map_err(io::Error::other)?;
    beilage(&mut tar, KOPF, &kopf)?;
    for (name, daten) in beilagen {
        beilage(&mut tar, name, daten)?;
    }
    for (name, pfad) in dateien {
        let mut f = std::fs::File::open(pfad)?;
        tar.append_file(name, &mut f)?;
    }

    Ok(tar.into_inner()?.finish()?)
}

fn beilage<W: Write>(tar: &mut tar::Builder<W>, name: &str, daten: &[u8]) -> io::Result<()> {
    let mut h = tar::Header::new_gnu();
    h.set_size(daten.len() as u64);
    h.set_mode(0o600);
    h.set_entry_type(tar::EntryType::Regular);
    tar.append_data(&mut h, name, daten)
}

/* ── Oeffnen ────────────────────────────────────────────────────────────── */

/// Eine Sicherung in `ziel` auspacken. `ziel` sollte leer sein; was die
/// Schale daraus macht, entscheidet sie.
///
/// Nur gewoehnliche Dateien und Ordner kommen heraus, und nur unterhalb von
/// `ziel` -- eine Datei, die jemand untergeschoben hat, schreibt keine
/// Verknuepfung und nicht nach `../`.
pub fn lesen<R: Read>(quelle: R, passphrase: &str, ziel: &Path) -> Ergebnis<Kopf> {
    let entschluesselt = entschluesseln(quelle, passphrase)?;
    let mut tar = tar::Archive::new(entschluesselt);
    let mut kopf: Option<Kopf> = None;

    for eintrag in tar.entries().map_err(kaputt)? {
        let mut eintrag = eintrag.map_err(kaputt)?;
        let name = eintrag
            .path()
            .map_err(kaputt)?
            .to_string_lossy()
            .to_string();

        if kopf.is_none() {
            if name != KOPF {
                return Err(SicherungFehler::KeineSicherung);
            }
            let mut text = String::new();
            eintrag.read_to_string(&mut text).map_err(kaputt)?;
            let k: Kopf =
                serde_json::from_str(&text).map_err(|_| SicherungFehler::KeineSicherung)?;
            if k.format > FORMAT {
                return Err(SicherungFehler::NeueresFormat(k.format));
            }
            kopf = Some(k);
            continue;
        }

        match eintrag.header().entry_type() {
            tar::EntryType::Regular | tar::EntryType::Directory => {}
            _ => {
                return Err(SicherungFehler::Beschaedigt(format!(
                    "unexpected entry {name}"
                )))
            }
        }
        if !eintrag.unpack_in(ziel).map_err(kaputt)? {
            return Err(SicherungFehler::Beschaedigt(format!(
                "path outside: {name}"
            )));
        }
    }

    kopf.ok_or(SicherungFehler::KeineSicherung)
}

fn entschluesseln<R: Read>(quelle: R, passphrase: &str) -> Ergebnis<impl Read> {
    let d = age::Decryptor::new(quelle).map_err(|_| SicherungFehler::KeineSicherung)?;
    if !d.is_scrypt() {
        return Err(SicherungFehler::KeineSicherung);
    }
    let mut ich = age::scrypt::Identity::new(SecretString::from(passphrase_glaetten(passphrase)));
    ich.set_max_work_factor(AUFWAND_HOECHSTENS);
    d.decrypt(std::iter::once(&ich as &dyn age::Identity))
        .map_err(|e| match e {
            age::DecryptError::DecryptionFailed
            | age::DecryptError::KeyDecryptionFailed
            | age::DecryptError::NoMatchingKeys => SicherungFehler::FalschePassphrase,
            _ => SicherungFehler::KeineSicherung,
        })
}

/// Ein Lesefehler mitten im Strom heisst: Die Datei ist abgeschnitten oder
/// veraendert (age prueft jedes Stueck).
fn kaputt(e: io::Error) -> SicherungFehler {
    SicherungFehler::Beschaedigt(e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Im Test reicht ein kleiner Aufwand; das Format ist dasselbe.
    fn sichern(p: &str, beilagen: &[(String, Vec<u8>)], dateien: &[(String, PathBuf)]) -> Vec<u8> {
        let kopf = Kopf::jetzt("test 1", "Tablet");
        schreiben_mit(Vec::new(), p, 10, &kopf, beilagen, dateien).unwrap()
    }

    const P: &str = "abacus abdomen abdominal abide";

    #[test]
    fn hin_und_zurueck() {
        let quelle = tempfile::tempdir().unwrap();
        let bild = quelle.path().join("bild");
        std::fs::write(&bild, vec![7u8; 100_000]).unwrap();

        let datei = sichern(
            P,
            &[("geheim/postfach.json".into(), b"{\"a\":1}".to_vec())],
            &[("inhalte/ab/cdef".into(), bild)],
        );

        let ziel = tempfile::tempdir().unwrap();
        let kopf = lesen(&datei[..], P, ziel.path()).unwrap();
        assert_eq!(kopf.format, FORMAT);
        assert_eq!(kopf.geraet, "Tablet");
        assert_eq!(
            std::fs::read(ziel.path().join("geheim/postfach.json")).unwrap(),
            b"{\"a\":1}"
        );
        assert_eq!(
            std::fs::read(ziel.path().join("inhalte/ab/cdef")).unwrap(),
            vec![7u8; 100_000]
        );
    }

    #[test]
    fn grossschreibung_und_leerzeichen_zaehlen_nicht() {
        let datei = sichern(P, &[], &[]);
        let ziel = tempfile::tempdir().unwrap();
        assert!(lesen(
            &datei[..],
            "  Abacus  abdomen ABDOMINAL\nabide ",
            ziel.path()
        )
        .is_ok());
    }

    #[test]
    fn falsche_passphrase_wird_so_genannt() {
        let datei = sichern(P, &[], &[]);
        let ziel = tempfile::tempdir().unwrap();
        assert!(matches!(
            lesen(&datei[..], "abacus abdomen abdominal abider", ziel.path()),
            Err(SicherungFehler::FalschePassphrase)
        ));
    }

    #[test]
    fn fremdes_ist_keine_sicherung() {
        let ziel = tempfile::tempdir().unwrap();
        assert!(matches!(
            lesen(&b"PK\x03\x04 irgendwas"[..], P, ziel.path()),
            Err(SicherungFehler::KeineSicherung)
        ));
    }

    #[test]
    fn abgeschnitten_ist_beschaedigt() {
        let quelle = tempfile::tempdir().unwrap();
        let gross = quelle.path().join("gross");
        std::fs::write(&gross, vec![1u8; 300_000]).unwrap();
        let datei = sichern(P, &[], &[("inhalte/x".into(), gross)]);

        let ziel = tempfile::tempdir().unwrap();
        let halb = &datei[..datei.len() / 2];
        assert!(matches!(
            lesen(halb, P, ziel.path()),
            Err(SicherungFehler::Beschaedigt(_))
        ));
    }

    #[test]
    fn neueres_format_sagt_es() {
        let mut kopf = Kopf::jetzt("test 2", "");
        kopf.format = FORMAT + 1;
        let datei = schreiben_mit(Vec::new(), P, 10, &kopf, &[], &[]).unwrap();
        let ziel = tempfile::tempdir().unwrap();
        assert!(matches!(
            lesen(&datei[..], P, ziel.path()),
            Err(SicherungFehler::NeueresFormat(f)) if f == FORMAT + 1
        ));
    }

    #[test]
    fn nichts_ausserhalb_des_ziels() {
        // Von Hand gebaut, wie es ein Angreifer taete: Kopf richtig, danach
        // ein Eintrag mit `../`.
        let mut empfaenger = age::scrypt::Recipient::new(SecretString::from(P.to_string()));
        empfaenger.set_work_factor(10);
        let w =
            age::Encryptor::with_recipients(std::iter::once(&empfaenger as &dyn age::Recipient))
                .unwrap()
                .wrap_output(Vec::new())
                .unwrap();
        let mut tar = tar::Builder::new(w);
        let kopf = serde_json::to_vec(&Kopf::jetzt("x", "")).unwrap();
        beilage(&mut tar, KOPF, &kopf).unwrap();
        let mut h = tar::Header::new_gnu();
        let boese = b"boese";
        h.set_size(boese.len() as u64);
        h.set_entry_type(tar::EntryType::Regular);
        // `append_data` wehrt `..` selbst ab; der Name geht deshalb roh in
        // den Kopf.
        h.as_old_mut().name[..9].copy_from_slice(b"../boese\0");
        h.set_cksum();
        tar.append(&h, &boese[..]).unwrap();
        let datei = tar.into_inner().unwrap().finish().unwrap();

        let aussen = tempfile::tempdir().unwrap();
        let ziel = aussen.path().join("ziel");
        std::fs::create_dir(&ziel).unwrap();
        assert!(lesen(&datei[..], P, &ziel).is_err());
        assert!(!aussen.path().join("boese").exists());
    }

    #[test]
    fn zu_kurz_wird_nicht_verschluesselt() {
        let kopf = Kopf::jetzt("x", "");
        assert!(matches!(
            schreiben_mit(Vec::new(), "kurz", 10, &kopf, &[], &[]),
            Err(SicherungFehler::ZuKurz)
        ));
    }

    #[test]
    fn vorschlag_hat_sechs_woerter_aus_der_liste() {
        let liste: std::collections::HashSet<&str> = woerter().collect();
        assert_eq!(liste.len(), 7776);
        let v = passphrase_vorschlagen();
        let teile: Vec<&str> = v.split(' ').collect();
        assert_eq!(teile.len(), WOERTER);
        assert!(teile.iter().all(|w| liste.contains(w)));
        assert_eq!(passphrase_glaetten(&v), v);
        assert_ne!(v, passphrase_vorschlagen());
    }
}
