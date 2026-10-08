//! Dateiinhalte von einer Gegenstelle holen -- stueckweise, geprueft, mit
//! Platzreserve.
//!
//! **Getrennt vom Laeufer, und das mit Absicht.** Der Laeufer gleicht ab,
//! WAS es gibt; das geht schnell und haelt den Speicher nur kurz. Inhalte
//! koennen Gigabytes sein. Wer sie holt, braucht den Speicher nicht --
//! nur die Liste, was fehlt, und die Ablage. So bleibt die Oberflaeche
//! bedienbar, waehrend im Hintergrund ein Video ankommt.
//!
//! **Die Regel je Geraet** (Plan, Phase 3): *alles behalten* holt nach jedem
//! Abgleich, was fehlt; *bei Bedarf* holt erst beim Oeffnen. Beide halten
//! eine Reserve frei ([`reserve`]).

use crate::gegenstelle::Gegenstelle;
use openany_client::OpenanyError;
use openany_store::Inhalte;
use std::time::Duration;

/// Wie oft ein Stueck wiederholt wird, wenn die Gegenstelle bremst.
///
/// Vier Versuche, dann gilt der Inhalt als nicht geholt. Unendlich zu
/// wiederholen hiesse, dass ein Abgleich nie fertig wird und niemand erfaehrt,
/// woran es liegt.
const VERSUCHE: u32 = 4;

/// Laenger als zwei Minuten wird nicht gewartet, was der Kopf auch sagt.
///
/// Der Wert kommt von der Gegenstelle und ist damit fremde Eingabe. Ein
/// `Retry-After: 86400` legte den Abgleich sonst fuer einen Tag schlafen --
/// in einem Programm, das der Mensch gerade offen hat und ansieht.
const HOECHSTENS_WARTEN: u64 = 120;

/// Wie gross ein Stueck ist -- ueber ein wackliges WLAN lieber vier Anfragen
/// zu 4 MB, die einzeln scheitern duerfen, als eine zu 16.
pub const STUECK: u64 = 4 * 1024 * 1024;

/// Wie viel Platz frei bleiben muss: 10 % des Datentraegers, hoechstens 1 GB.
///
/// Ein Tablet mit 23 GB haelt so 1 GB frei, eines mit 8 GB 800 MB. Ein
/// Telefon, dessen Speicher durch openany volllaeuft, macht keine Fotos
/// mehr und nimmt keine Nachrichten an -- das darf ein Abgleich nicht
/// verursachen.
pub fn reserve(gesamt: u64) -> u64 {
    (gesamt / 10).min(1024 * 1024 * 1024)
}

#[derive(Debug, thiserror::Error)]
pub enum InhaltFehler {
    #[error(transparent)]
    Gegenstelle(#[from] openany_client::OpenanyError),
    #[error("Storage: {0}")]
    Ablage(#[from] std::io::Error),
    #[error("The other device delivered less than the file size.")]
    Unvollstaendig,
    #[error("The content does not match its fingerprint -- discarded.")]
    FalscherAbdruck,
    #[error("Not enough storage on this device.")]
    ZuWenigPlatz,
}

/// Ein Stueck holen -- und warten, wenn die Gegenstelle bremst.
///
/// **WARUM ES DAS BRAUCHT.** `OpenanyError::Gebremst` gab es von Anfang an,
/// samt der Wartezeit aus dem Kopf `Retry-After`. Ausgewertet hat sie nie
/// jemand: Ein 429 fiel in denselben Zweig wie jeder andere Fehler, der
/// Inhalt galt als nicht geholt, und gefragt wurde nie wieder.
///
/// Am 16.09.2026 im Zugriffsprotokoll gesehen -- ein Erstabgleich gegen
/// openany.de: 59 Inhalte kamen durch, **39 bekamen 429**, 7 brachen ab. Die
/// Vorschauen, die hinten in der Reihe standen, kamen nie an die Reihe. In
/// der Galerie stand daraufhin "Leer" ueber Alben, die Bilder hatten -- und
/// im Bericht stand eine Zahl bei "uebersprungen", die niemand deuten konnte.
///
/// Die Drossel drueben ist inzwischen eine eigene und grosszuegigere
/// (`throttle:abgleich`). Das macht diese Schleife nicht ueberfluessig,
/// sondern beide noetig: Eine Grenze, die nie greift, ist keine Grenze -- und
/// ein Programm, das Bremsen ignoriert, faellt beim naechsten Mal wieder um,
/// nur spaeter.
async fn mit_geduld<G: Gegenstelle + ?Sized>(
    gegenstelle: &G,
    abdruck: &str,
    von: u64,
    laenge: u64,
) -> Result<Vec<u8>, OpenanyError> {
    let mut versuch = 1;

    loop {
        match gegenstelle.inhalt(abdruck, von, laenge).await {
            Err(OpenanyError::Gebremst { sekunden }) if versuch < VERSUCHE => {
                tokio::time::sleep(Duration::from_secs(sekunden.min(HOECHSTENS_WARTEN))).await;
                versuch += 1;
            }
            // Auch ein `Gebremst` im letzten Versuch kommt hier heraus und
            // wird ein Fehler mit Namen. Der Bericht sagt dann "zu viele
            // Anfragen" statt einer stummen Zahl.
            ergebnis => return ergebnis,
        }
    }
}

/// Einen Inhalt holen und ablegen. Liegt er schon da, passiert nichts.
pub async fn inhalt_holen<G: Gegenstelle + ?Sized>(
    inhalte: &Inhalte,
    gegenstelle: &G,
    abdruck: &str,
    groesse: u64,
) -> Result<(), InhaltFehler> {
    if inhalte.hat(abdruck) {
        return Ok(());
    }

    let mut ladung = inhalte.ladung()?;
    let mut von = 0;
    while von < groesse {
        let will = STUECK.min(groesse - von);
        let stueck = match mit_geduld(gegenstelle, abdruck, von, will).await {
            Ok(s) => s,
            Err(e) => {
                inhalte.verwerfen(ladung);
                return Err(e.into());
            }
        };
        if stueck.is_empty() {
            /*
             * DER STROM IST ZU ENDE -- ob zu frueh, sagt der ABDRUCK.
             *
             * Hier stand ein sofortiges `Unvollstaendig`. Das setzte voraus,
             * dass `groesse` stimmt; fuer Dateien und Originale tut sie das
             * (sie steht im Eintrag). Fuer VORSCHAUEN steht sie nicht darin --
             * dort geht eine Schaetzung mit (`VORSCHAU_GROESSE`), und die ist
             * fast immer zu gross.
             *
             * Am 16.09.2026 kamen darum zwoelf Vorschauen vollstaendig an und
             * wurden verworfen: "Das andere Geraet lieferte weniger, als die
             * Datei gross ist." Das andere Geraet hatte alles geliefert.
             *
             * Die Groesse ist ein Hinweis, der Abdruck ist die Zusicherung.
             * Also abbrechen und unten pruefen -- passt der Abdruck, war es
             * vollstaendig; passt er nicht, faellt es dort auf.
             */
            break;
        }
        ladung.schreiben(&stueck)?;
        let kam = stueck.len() as u64;
        von += kam;

        /*
         * WENIGER ALS GEFRAGT HEISST: DAS WAR ALLES.
         *
         * Die Gegenstelle deckelt nur nach oben; was sie schickt, geht bis
         * ans Ende der Datei. Kommt also weniger als gefragt, ist die Datei
         * zu Ende -- und die naechste Frage ginge hinter ihr Ende.
         *
         * Genau das tat der Laeufer bei jeder VORSCHAU: Ihre Groesse steht
         * in keinem Eintrag, es geht eine Schaetzung mit (256 KiB), und die
         * ist fast immer zu gross. Das erste Stueck brachte die ganze
         * Vorschau, das zweite beantwortete der Server mit 416 (am
         * 21.09.2026 im Protokoll: sechs Vorschauen, sechs 416). Es ging
         * nichts verloren -- der Abdruck stimmte, die Vorschau kam an --,
         * aber jede kostete eine Anfrage, die von vornherein ins Leere ging.
         */
        if kam < will {
            break;
        }
    }

    // Erst jetzt steht fest, was angekommen ist. Passt es nicht, war es nie
    // da -- ein falscher Inhalt unter einem richtigen Namen waere schlimmer
    // als gar keiner.
    let (angekommen, _) = inhalte.ablegen(ladung)?;
    if !angekommen.eq_ignore_ascii_case(abdruck) {
        inhalte.entfernen(&angekommen)?;

        // ZWEI NAMEN FUER ZWEI FAELLE. Kam weniger, als angekuendigt war, und
        // passt der Abdruck nicht, dann brach die Uebertragung ab -- das ist
        // etwas anderes als vollstaendig angekommene Bytes, die nicht zu
        // ihrem Namen passen. Der erste Fall heilt beim naechsten Lauf, der
        // zweite nicht, und wer den Bericht liest, soll das unterscheiden.
        return Err(if von < groesse {
            InhaltFehler::Unvollstaendig
        } else {
            InhaltFehler::FalscherAbdruck
        });
    }
    Ok(())
}

/// Was ein Durchgang geholt hat.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize)]
pub struct InhalteBericht {
    pub geholt: usize,
    /// Liegt bei der Gegenstelle auch nicht.
    pub nicht_da: usize,
    pub zu_wenig_platz: usize,
    pub fehler: Vec<String>,
}

/// Alles holen, was fehlt und die Gegenstelle hat -- solange Platz ist.
///
/// * `fehlend` -- (Abdruck, Groesse) je Inhalt, der hier fehlt.
/// * `frei` -- wie viel gerade frei ist (wird vor jeder Datei neu gefragt).
/// * `gesamt` -- Groesse des Datentraegers, fuer die Reserve.
pub async fn fehlende_inhalte_holen<G: Gegenstelle + ?Sized>(
    inhalte: &Inhalte,
    gegenstelle: &G,
    fehlend: &[(String, u64)],
    frei: impl Fn() -> u64,
    gesamt: u64,
) -> InhalteBericht {
    let mut bericht = InhalteBericht::default();
    let rest = reserve(gesamt);

    let abdruecke: Vec<String> = fehlend.iter().map(|(a, _)| a.clone()).collect();
    let mut dort = std::collections::BTreeSet::new();
    for teil in abdruecke.chunks(500) {
        match gegenstelle.inhalte_da(teil).await {
            Ok(liste) => dort.extend(liste),
            Err(e) => {
                bericht.fehler.push(e.to_string());
                return bericht;
            }
        }
    }

    // Kleine zuerst: Bei knappem Platz passen so viele Dateien wie moeglich,
    // statt dass ein einzelnes Video alles blockiert.
    let mut reihe: Vec<&(String, u64)> = fehlend.iter().collect();
    reihe.sort_by_key(|(_, g)| *g);

    for (abdruck, groesse) in reihe {
        if !dort.contains(abdruck) {
            bericht.nicht_da += 1;
            continue;
        }
        if frei() < groesse.saturating_add(rest) {
            bericht.zu_wenig_platz += 1;
            continue;
        }
        match inhalt_holen(inhalte, gegenstelle, abdruck, *groesse).await {
            Ok(()) => bericht.geholt += 1,
            Err(e) => bericht.fehler.push(e.to_string()),
        }
    }
    bericht
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn die_reserve_ist_zehn_prozent_hoechstens_ein_gigabyte() {
        assert_eq!(reserve(8 * 1024 * 1024 * 1024), 858_993_459);
        assert_eq!(reserve(224 * 1024 * 1024 * 1024), 1024 * 1024 * 1024);
    }

    /* ── Die Geduld ──────────────────────────────────────────────────── */

    use openany_client::{Delta, Eintrag, Ergebnis};
    use std::sync::atomic::{AtomicU32, Ordering};

    /// Bremst die ersten `bremst` Anfragen und liefert dann.
    struct Bremser {
        bremst: AtomicU32,
        gefragt: AtomicU32,
    }

    impl Bremser {
        fn neu(bremst: u32) -> Self {
            Self {
                bremst: AtomicU32::new(bremst),
                gefragt: AtomicU32::new(0),
            }
        }
    }

    #[async_trait::async_trait]
    impl Gegenstelle for Bremser {
        fn basis(&self) -> &str {
            "https://pappe.test"
        }

        async fn delta(&self, _seit: i64) -> Result<Delta, OpenanyError> {
            unreachable!("hier wird nur geholt")
        }

        async fn notiztext(&self, _zk_id: &str) -> Result<String, OpenanyError> {
            unreachable!("hier wird nur geholt")
        }

        async fn anwenden(&self, _e: &[Eintrag]) -> Result<Vec<Ergebnis>, OpenanyError> {
            unreachable!("hier wird nur geholt")
        }

        async fn inhalt(
            &self,
            _abdruck: &str,
            _von: u64,
            _laenge: u64,
        ) -> Result<Vec<u8>, OpenanyError> {
            self.gefragt.fetch_add(1, Ordering::SeqCst);

            // `Retry-After: 0` -- der Test soll die Geduld pruefen, nicht die
            // Uhr. Eine echte Wartezeit machte aus einer Zusicherung eine
            // Sanduhr, und die erste hektische Woche stellte sie wieder ab.
            if self.bremst.fetch_sub(1, Ordering::SeqCst) > 0 {
                return Err(OpenanyError::Gebremst { sekunden: 0 });
            }

            Ok(b"hier".to_vec())
        }
    }

    /// EIN 429 IST KEIN NEIN, sondern ein "gleich wieder".
    ///
    /// Bis zum 16.09.2026 fiel er in denselben Zweig wie jeder andere Fehler:
    /// Der Inhalt galt als nicht geholt, und gefragt wurde nie wieder.
    #[tokio::test]
    async fn eine_bremse_wird_abgewartet_und_nicht_als_fehler_gezaehlt() {
        let server = Bremser::neu(2);

        let ergebnis = mit_geduld(&server, "abdruck", 0, 4).await;

        assert_eq!(ergebnis.unwrap(), b"hier".to_vec());
        assert_eq!(
            server.gefragt.load(Ordering::SeqCst),
            3,
            "zweimal gebremst, beim dritten Mal geliefert"
        );
    }

    /* ── Die Stuecke ─────────────────────────────────────────────────── */

    /// Liefert `inhalt` in Stuecken aus einem festen Vorrat und zaehlt mit.
    struct Vorrat {
        bytes: Vec<u8>,
        gefragt: AtomicU32,
    }

    #[async_trait::async_trait]
    impl Gegenstelle for Vorrat {
        fn basis(&self) -> &str {
            "https://pappe.test"
        }

        async fn delta(&self, _seit: i64) -> Result<Delta, OpenanyError> {
            unreachable!("hier wird nur geholt")
        }

        async fn notiztext(&self, _zk_id: &str) -> Result<String, OpenanyError> {
            unreachable!("hier wird nur geholt")
        }

        async fn anwenden(&self, _e: &[Eintrag]) -> Result<Vec<Ergebnis>, OpenanyError> {
            unreachable!("hier wird nur geholt")
        }

        async fn inhalt(
            &self,
            _abdruck: &str,
            von: u64,
            laenge: u64,
        ) -> Result<Vec<u8>, OpenanyError> {
            self.gefragt.fetch_add(1, Ordering::SeqCst);

            // Wie der Server: hinter dem Ende gibt es nichts, und was kommt,
            // geht hoechstens bis dorthin.
            let von = von.min(self.bytes.len() as u64) as usize;
            let bis = (von + laenge as usize).min(self.bytes.len());

            Ok(self.bytes[von..bis].to_vec())
        }
    }

    fn abdruck_von(bytes: &[u8]) -> String {
        use sha2::{Digest, Sha256};

        format!("{:x}", Sha256::digest(bytes))
    }

    /// EINE VORSCHAU KOSTET EINE ANFRAGE, NICHT ZWEI.
    ///
    /// Ihre Groesse steht in keinem Eintrag; es geht eine Schaetzung mit, und
    /// die ist zu gross. Bis zum 21.09.2026 fragte der Laeufer deshalb hinter
    /// dem Ende noch einmal nach und bekam 416.
    #[tokio::test]
    async fn ein_kurzes_stueck_beendet_das_holen() {
        let ordner = tempfile::tempdir().unwrap();
        let inhalte = Inhalte::oeffnen(ordner.path()).unwrap();
        let bytes = vec![7u8; 38_274];
        let server = Vorrat {
            bytes: bytes.clone(),
            gefragt: AtomicU32::new(0),
        };

        inhalt_holen(&inhalte, &server, &abdruck_von(&bytes), 262_144)
            .await
            .expect("die Vorschau kommt vollstaendig an");

        assert_eq!(
            server.gefragt.load(Ordering::SeqCst),
            1,
            "keine Frage hinter das Ende"
        );
        assert!(inhalte.hat(&abdruck_von(&bytes)));
    }

    /// Und was wirklich mehrere Stuecke braucht, bekommt sie auch.
    #[tokio::test]
    async fn ein_grosser_inhalt_kommt_in_stuecken() {
        let ordner = tempfile::tempdir().unwrap();
        let inhalte = Inhalte::oeffnen(ordner.path()).unwrap();
        let bytes = vec![3u8; (STUECK + 1000) as usize];
        let server = Vorrat {
            bytes: bytes.clone(),
            gefragt: AtomicU32::new(0),
        };

        inhalt_holen(&inhalte, &server, &abdruck_von(&bytes), bytes.len() as u64)
            .await
            .unwrap();

        assert_eq!(server.gefragt.load(Ordering::SeqCst), 2);
        assert!(inhalte.hat(&abdruck_von(&bytes)));
    }

    /// Aber nicht unendlich: Ein Abgleich, der nie fertig wird, sagt niemandem,
    /// woran es liegt. Nach `VERSUCHE` kommt der Fehler MIT NAMEN heraus.
    #[tokio::test]
    async fn irgendwann_ist_auch_die_geduld_zu_ende() {
        let server = Bremser::neu(u32::MAX);

        let fehler = mit_geduld(&server, "abdruck", 0, 4).await.unwrap_err();

        assert!(matches!(fehler, OpenanyError::Gebremst { .. }), "{fehler}");
        assert_eq!(server.gefragt.load(Ordering::SeqCst), VERSUCHE);
    }
}
