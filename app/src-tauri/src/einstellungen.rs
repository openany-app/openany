//! Wohin dieses Programm spricht -- und ob ueberhaupt.
//!
//! **Der wichtigste Zustand ist der leere.** openany-app funktioniert ohne
//! Anmeldung und ohne Netz: Notizen und Termine liegen in der lokalen
//! SQLite, und wer nie eine Instanz eintraegt, hat trotzdem ein vollstaendiges
//! Programm. Das ist der Unterschied zu anytail-app, dessen Schale ohne
//! Kopplung nichts zeigt -- die beiden Saetze passen nicht in ein Programm,
//! und hier gilt dieser.
//!
//! Deshalb ist [`Einstellungen`] durchweg mit leeren Feldern gueltig, und
//! [`Einstellungen::verbunden`] ist eine Frage, keine Voraussetzung.
//!
//! **Nur openany.de (Tiffy, 08.10.2026).** Bis dahin liess sich unter
//! „Abgleich" eine beliebige Instanz eintragen, samt anyid-Adresse und
//! eigener Wurzel-CA. Das brachte niemandem etwas -- eine eigene Instanz
//! kann ohne den (nicht oeffentlichen) Server ohnehin niemand betreiben --
//! und verwirrte. Jetzt setzt „Mit openany.de verbinden" die Adresse
//! [`OPENANY`], „Verbindung trennen" leert sie wieder. Leer bleibt der
//! Zustand „nur auf diesem Geraet".

use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};

/// Die einzige Instanz, mit der sich die App verbindet.
pub const OPENANY: &str = "https://openany.de";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Einstellungen {
    /// Die Wurzel der openany-Instanz, etwa `https://openany.de`. Leer heisst:
    /// gar keine.
    #[serde(default)]
    pub openany_basis: String,

    /// Die Wurzel von anyid. Nur zum Koppeln noetig.
    #[serde(default)]
    pub anyid_basis: String,

    /// Wie dieses Geraet in beiden Listen heissen soll.
    #[serde(default)]
    pub geraetename: String,

    /// Eine zusaetzliche Wurzel-CA als Dateipfad. Seit dem 08.10.2026 nicht
    /// mehr in der Oberflaeche; gilt nur, wenn sie von Hand in der Datei
    /// steht (Entwicklung).
    ///
    /// Gebraucht in der Entwicklung -- dort steht ein Caddy mit eigener CA
    /// davor -- und in Haeusern, die ihren Verkehr ueber eine eigene fuehren.
    /// **Kein Abschalten der Pruefung**: eine Wurzel mehr, nicht eine Wurzel
    /// weniger. Ein `danger_accept_invalid_certs` waere die Gewohnheit, die
    /// irgendwann in einem ausgelieferten Programm landet -- und dann traegt
    /// ein Geraeteschluessel, der auf alle Notizen reicht, ueber eine Leitung,
    /// der niemand mehr ansieht, wer mithoert.
    #[serde(default)]
    pub ca_pfad: String,

    /// Wie viel Dateiinhalt dieses Geraet behaelt (Plan, Phase 3):
    /// `alles` holt nach jedem Abgleich, was fehlt; alles andere heisst
    /// "bei Bedarf" -- geholt wird beim Oeffnen. Reist nicht mit: Das Tablet
    /// mit 6 GB frei entscheidet anders als der PC.
    #[serde(default)]
    pub inhalte_regel: String,

    /// Bei Regel `ausgewaehlt`: Ordner und Alben (uuids), deren Inhalt dieses
    /// Geraet immer behaelt -- samt allem, was darin liegt.
    #[serde(default)]
    pub behalten: Vec<String>,

    /// Eigene Ordner und Alben, die in ein Projekt freigegeben sind (uuids,
    /// je Projekt-uuid) -- sie behaelt dieses Geraet IMMER, gleich welche
    /// Regel gilt (Tiffy, 30.09.2026). Getrennt von `behalten`: Endet die
    /// Freigabe, faellt der Ordner zurueck auf die Regel, ausser er ist
    /// selbst angeheftet. Gesetzt beim Abgleich der Projekte.
    #[serde(default)]
    pub projekt_behalten: std::collections::BTreeMap<String, Vec<String>>,
}

impl Einstellungen {
    /// Ist eine Instanz eingetragen? Sagt **nichts** darueber, ob dieses
    /// Geraet gekoppelt ist -- das steht in den Ausweisen.
    pub fn verbunden(&self) -> bool {
        !self.openany_basis.trim().is_empty()
    }

    pub fn ca(&self) -> Option<Vec<u8>> {
        if self.ca_pfad.trim().is_empty() {
            return None;
        }

        std::fs::read(self.ca_pfad.trim()).ok()
    }

    /// Ein Geraetename, der nie leer ist: Er steht drueben in zwei Listen, und
    /// ein namenloser Eintrag ist einer, den niemand widerrufen mag, weil
    /// niemand weiss, was er ist.
    pub fn name(&self) -> String {
        let name = self.geraetename.trim();

        if name.is_empty() {
            "Dieses Gerät".to_string()
        } else {
            name.to_string()
        }
    }

    /// Was ueber Projekt-Freigaben immer bleibt, ohne Doppel.
    pub fn freigaben_behalten(&self) -> Vec<String> {
        let mut alle: Vec<String> = self.projekt_behalten.values().flatten().cloned().collect();
        alle.sort();
        alle.dedup();
        alle
    }

    /// Alles, was dieses Geraet behaelt: die eigenen Markierungen und die
    /// eigenen Freigaben in Projekten.
    pub fn alles_behalten(&self) -> Vec<String> {
        let mut alle = self.behalten.clone();
        for f in self.freigaben_behalten() {
            if !alle.contains(&f) {
                alle.push(f);
            }
        }
        alle
    }

    /// `alles`, `ausgewaehlt` oder `bei_bedarf` -- nie etwas anderes.
    pub fn regel(&self) -> &'static str {
        match self.inhalte_regel.as_str() {
            "alles" => "alles",
            "ausgewaehlt" => "ausgewaehlt",
            _ => "bei_bedarf",
        }
    }

    /// Ein neuer Geraetename -- alles andere bleibt, wie es war.
    ///
    /// **Warum Methoden und keine Zeilen am Aufrufer.** Dort stand einmal die
    /// Struktur ausgeschrieben, und die Felder, die das Formular nicht trug,
    /// holten sich ihren Wert je mit einem eigenen `lock().await` -- im
    /// selben Ausdruck. Der erste Waechter lag noch, als der zweite genommen
    /// wurde, und das Speichern hing (15.-16.09.2026). Hier haelt der
    /// Aufrufer die Sperre schon und ruft `self` auf.
    pub fn mit_namen(&self, geraetename: &str) -> Self {
        Self {
            geraetename: geraetename.trim().to_string(),
            ..self.clone()
        }
    }

    /// Mit openany.de verbunden: die feste Adresse und das anyid, das die
    /// Instanz nennt.
    pub fn mit_openany(&self, anyid_basis: &str) -> Self {
        Self {
            openany_basis: OPENANY.to_string(),
            anyid_basis: anyid_basis.trim().trim_end_matches('/').to_string(),
            ..self.clone()
        }
    }

    /// Verbindung getrennt: wieder nur auf diesem Geraet.
    pub fn ohne_openany(&self) -> Self {
        Self {
            openany_basis: String::new(),
            anyid_basis: String::new(),
            ..self.clone()
        }
    }

    pub fn lesen(pfad: &Path) -> Self {
        std::fs::read_to_string(pfad)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            // Eine kaputte Datei ergibt die Vorgabe und keinen Absturz: Die
            // Einstellungen sind Beiwerk, die Notizen sind es nicht.
            .unwrap_or_default()
    }

    pub fn schreiben(&self, pfad: &PathBuf) -> io::Result<()> {
        if let Some(ordner) = pfad.parent() {
            std::fs::create_dir_all(ordner)?;
        }

        std::fs::write(pfad, serde_json::to_string_pretty(self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bestand() -> Einstellungen {
        Einstellungen {
            openany_basis: "https://alt.example".into(),
            anyid_basis: "https://id.alt.example".into(),
            geraetename: "Altes Tablet".into(),
            ca_pfad: "/tmp/alt.pem".into(),
            inhalte_regel: "ausgewaehlt".into(),
            behalten: vec!["ordner-a".into(), "album-b".into()],
            projekt_behalten: [(
                "p1".to_string(),
                vec!["album-b".to_string(), "ordner-c".to_string()],
            )]
            .into_iter()
            .collect(),
        }
    }

    /// Eigene Freigaben kommen zu den Markierungen dazu, ohne Doppel, und
    /// ueberleben das Verbinden.
    #[test]
    fn freigaben_werden_mit_behalten() {
        let e = bestand();
        assert_eq!(e.freigaben_behalten(), vec!["album-b", "ordner-c"]);
        assert_eq!(e.alles_behalten(), vec!["ordner-a", "album-b", "ordner-c"]);
        assert_eq!(
            e.mit_openany("https://id.anytail.de").projekt_behalten,
            e.projekt_behalten
        );
    }

    /// DER KERN DES FEHLERS VOM 15.09.2026, als Verhalten festgehalten:
    /// Speicherregel und Behalten-Liste stehen in einer anderen Kachel. Wer
    /// verbindet, trennt oder umbenennt, darf sie nicht verlieren.
    #[test]
    fn was_der_schritt_nicht_betrifft_ueberlebt() {
        for neu in [
            bestand().mit_openany("https://id.anytail.de"),
            bestand().ohne_openany(),
            bestand().mit_namen("Tablet"),
        ] {
            assert_eq!(neu.inhalte_regel, "ausgewaehlt");
            assert_eq!(neu.behalten, vec!["ordner-a", "album-b"]);
        }
    }

    #[test]
    fn verbinden_heisst_openany_de_und_trennen_heisst_nichts() {
        let neu = bestand().mit_openany("https://id.anytail.de/");
        assert_eq!(neu.openany_basis, OPENANY);
        assert_eq!(neu.anyid_basis, "https://id.anytail.de");
        assert!(neu.verbunden());
        assert_eq!(neu.name(), "Altes Tablet");

        let weg = neu.ohne_openany();
        assert!(!weg.verbunden());
        assert_eq!(weg.anyid_basis, "");
    }

    /// Die Bildschirmtastatur haengt gern ein Leerzeichen an. In zwei
    /// Geraetelisten sieht das wie ein Tippfehler des Menschen aus.
    #[test]
    fn leerzeichen_fallen_weg() {
        assert_eq!(bestand().mit_namen("  Tablet  ").geraetename, "Tablet");
    }
}
