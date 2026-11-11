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
//! **Nur openany.de.** Eine eigene Instanz kann ohne den (nicht
//! oeffentlichen) Server niemand betreiben. „Mit openany.de verbinden" setzt
//! die Adresse
//! [`OPENANY`], „Verbindung trennen" leert sie wieder. Leer bleibt der
//! Zustand „nur auf diesem Geraet".

use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};

/// Die einzige Instanz, mit der sich die App verbindet.
pub const OPENANY: &str = "https://openany.de";

/// Der CalDAV-Dienst am Desktop (K5): Anmeldung und Anschluss. Das Kennwort
/// schützt vor anderen Benutzern dieses Rechners; neu erzeugt sperrt es
/// eingerichtete Programme aus.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Kalenderdienst {
    pub benutzer: String,
    pub kennwort: String,
    pub port: u16,
}

/// Wie die Kalender ins System kommen (K1).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Systemkalender {
    /// Abos (ICS) auch -- nur auf Wunsch (Entscheidung 2), nur lesend.
    #[serde(default)]
    pub abos: bool,
}

/// Was der Fotowächter beobachtet (D5).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FotosAutomatisch {
    /// Gerätealben, deren neue Fotos und Videos kommen.
    pub alben: Vec<String>,
    /// Nur, was ab dann aufgenommen wurde (ms seit 1970) -- beim Einschalten
    /// gesetzt, damit nicht der ganze Bestand nachkommt.
    pub seit: i64,
    #[serde(default)]
    pub nur_wlan: bool,
    #[serde(default)]
    pub nur_laden: bool,
}

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

    /// Der Name des verbundenen Kontos, gemerkt beim Koppeln und nur zum
    /// Anzeigen. Leer heisst: vor dieser Anzeige gekoppelt. Eine Umbenennung
    /// drueben kommt erst mit dem naechsten Koppeln an.
    #[serde(default)]
    pub kontoname: String,

    /// Eine zusaetzliche Wurzel-CA als Dateipfad. Nicht in der Oberflaeche;
    /// gilt nur, wenn sie von Hand in der Datei steht (Entwicklung).
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

    /// Fotos vom Gerät (docs/plan-viele-dateien.md, Frage 6), einmal gefragt:
    /// `Some(true)` ohne Kopie -- die Alben gehen auf openany.de, hier bleibt
    /// die Vorschau; `Some(false)` als Kopie auf diesem Gerät; `None` noch
    /// nicht gefragt.
    #[serde(default)]
    pub fotos_ohne_kopie: Option<bool>,

    /// Neue Fotos automatisch übernehmen (docs/plan-viele-dateien.md, D5);
    /// `None` = aus. Gilt nur auf diesem Gerät.
    #[serde(default)]
    pub fotos_automatisch: Option<FotosAutomatisch>,

    /// Kalender im Kalender des Telefons (docs/plan-kalender-kontakte-im-system.md,
    /// K1); `None` = aus. Gilt nur auf diesem Gerät.
    #[serde(default)]
    pub systemkalender: Option<Systemkalender>,

    /// Kontakte im Adressbuch des Telefons (K4, nur Android).
    #[serde(default)]
    pub systemkontakte: bool,

    /// Der Kalenderdienst für andere Programme auf diesem Rechner (K5);
    /// `None` = aus.
    #[serde(default)]
    pub kalenderdienst: Option<Kalenderdienst>,

    /// Bei Regel `ausgewaehlt`: Ordner und Alben (uuids), deren Inhalt dieses
    /// Geraet immer behaelt -- samt allem, was darin liegt.
    #[serde(default)]
    pub behalten: Vec<String>,

    /// Eigene Ordner und Alben, die in ein Projekt freigegeben sind (uuids,
    /// je Projekt-uuid) -- sie behaelt dieses Geraet IMMER, gleich welche
    /// Regel gilt. Getrennt von `behalten`: Endet die
    /// Freigabe, faellt der Ordner zurueck auf die Regel, ausser er ist
    /// selbst angeheftet. Gesetzt beim Abgleich der Projekte.
    #[serde(default)]
    pub projekt_behalten: std::collections::BTreeMap<String, Vec<String>>,

    /// Nur auf dem Schreibtisch (docs/plan-desktop.md, 1c): etwa stuendlich
    /// abgleichen, solange das Programm laeuft. Unter Android haelt den
    /// Wunsch das System (`Auffrischer.kt`).
    #[serde(default)]
    pub auffrischen: bool,

    /// Nur auf dem Schreibtisch: der Wachdienst („Sofort benachrichtigen"),
    /// solange das Programm laeuft. Unter Android `Wachdienst.kt`.
    #[serde(default)]
    pub sofort: bool,

    /// Nur auf dem Schreibtisch: beim Anmelden starten und beim Schliessen des
    /// Fensters im Menue-Symbol weiterlaufen.
    #[serde(default)]
    pub weiterlaufen: bool,

    /// Beim letzten Abgleich war das Kontingent auf dem Server voll. Hier
    /// gemerkt, damit der Hinweis auch ohne Netz und nach einem Neustart steht.
    #[serde(default)]
    pub kontingent_voll: bool,
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
            _ => "ausgewaehlt",
        }
    }

    /// Ein neuer Geraetename -- alles andere bleibt, wie es war.
    ///
    /// **Warum Methoden und keine Zeilen am Aufrufer.** Holten sich die
    /// Felder, die das Formular nicht traegt, ihren Wert je mit einem eigenen
    /// `lock().await` im selben Ausdruck, laege der erste Waechter noch, wenn
    /// der zweite genommen wird -- und das Speichern hinge. Hier haelt der
    /// Aufrufer die Sperre schon und ruft `self` auf.
    pub fn mit_namen(&self, geraetename: &str) -> Self {
        Self {
            geraetename: geraetename.trim().to_string(),
            ..self.clone()
        }
    }

    /// Der Name des verbundenen Kontos -- alles andere bleibt, wie es war.
    pub fn mit_kontoname(&self, kontoname: &str) -> Self {
        Self {
            kontoname: kontoname.trim().to_string(),
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
            kontoname: String::new(),
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
            kontoname: "Beispiel".into(),
            ca_pfad: "/tmp/alt.pem".into(),
            inhalte_regel: "ausgewaehlt".into(),
            behalten: vec!["ordner-a".into(), "album-b".into()],
            projekt_behalten: [(
                "p1".to_string(),
                vec!["album-b".to_string(), "ordner-c".to_string()],
            )]
            .into_iter()
            .collect(),
            auffrischen: false,
            sofort: false,
            weiterlaufen: false,
            kontingent_voll: false,
            fotos_ohne_kopie: Some(true),
            fotos_automatisch: None,
            systemkalender: None,
            kalenderdienst: None,
            systemkontakte: false,
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

    /// Als Verhalten festgehalten:
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

    /// Nach dem Trennen darf die App nicht mehr behaupten, jemand sei
    /// angemeldet. Umbenennen und Verbinden lassen den Namen dagegen stehen.
    #[test]
    fn kontoname_geht_erst_mit_dem_trennen() {
        assert_eq!(bestand().ohne_openany().kontoname, "");
        assert_eq!(bestand().mit_namen("Tablet").kontoname, "Beispiel");
        assert_eq!(
            bestand().mit_openany("https://id.anytail.de").kontoname,
            "Beispiel"
        );
        assert_eq!(bestand().mit_kontoname("  Neu  ").kontoname, "Neu");
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
