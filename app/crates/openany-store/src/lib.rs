//! Der lokale Speicher: eine SQLite-Datei, die ohne Netz vollstaendig ist.
//!
//! **Offline-first heisst hier woertlich: Das Netz ist die Ausnahme.** Was
//! der Mensch sieht, kommt aus dieser Datei; der Abgleich fuellt sie und
//! traegt sie hinaus, aber nichts wartet auf ihn. Ein Programm, das erst nach
//! einer Antwort etwas anzeigen kann, ist in der U-Bahn kein Programm.
//!
//! ## Vier Dinge liegen hier, und sie sind bewusst getrennt
//!
//! | | |
//! |---|---|
//! | **Die Sachen** | Notizen, Kalender, Termine, Kontakte |
//! | **Das Protokoll** | was sich hier geaendert hat, seit wann |
//! | **Die Marken** | wie weit der Abgleich mit einer Gegenstelle ist |
//! | **Die Urspruenge** | der zuletzt beidseitig bekannte Stand je Sache |
//!
//! ## Die Gegenstelle steht im Schluessel -- von Anfang an
//!
//! Heute spricht dieses Programm mit genau einer Instanz. Es verspricht
//! *"wahlweise gegen die eigene oder gegen gar keine"* -- und was wahlweise
//! gegen eine spricht, spricht eines Tages mit zweien.
//!
//! Deshalb tragen **Marken und Urspruenge die Gegenstelle im
//! Primaerschluessel**, obwohl heute nur eine eingetragen wird. Sie spaeter
//! nachzuruesten hiesse, jeden vorhandenen Ursprungsstand zu entwerten: Ohne
//! Ursprung ist jeder Vergleich zweiseitig, und zwei abweichende Staende
//! sehen genauso aus, ob nun eine Seite geaendert hat oder beide. Der erste
//! Abgleich nach so einer Umstellung machte aus jeder Notiz eine
//! Konfliktkopie.
//!
//! Eine Spalte, die heute immer denselben Wert traegt, kostet nichts. Sie
//! nachtraeglich einzuziehen kostet die Ordnung aller Nutzer auf einmal.
//!
//! ## Der Server haelt nichts davon
//!
//! Kein serverseitiger Fortschrittszustand je Geraet -- und das ist kein
//! Mangel, sondern die Bauart. Der Server entscheidet Konflikte mit dem
//! `base_hash`, den **wir** mitschicken, und merkt sich nichts. Deshalb
//! kommen mehrere Geraete eines Kontos sich nicht in die Quere, und deshalb liegt die
//! Buchfuehrung vollstaendig in dieser Datei.

mod abgleichstand;
mod anhaenge;
mod dateien;
mod galerie;
mod inhalt_dort;
mod inhalte;
pub mod kalenderbuch;
mod kontaktfotos;
mod mails;
mod nachrichten;
mod notizbuch;
mod papierkorb;
mod pgp;
mod projekte;
mod protokoll;
mod sachen;
mod schema;
mod verlaufsfilter;
pub mod verweise;

pub use abgleichstand::Marke;
pub use anhaenge::Anhang;
pub use dateien::Datei;
pub use galerie::{Album, Bild};
pub use inhalte::{Inhalte, Ladung};
pub use kontaktfotos::{fotoabdruck, FotoHinaus};
pub use mails::{Mail, MailStand, MAILS_SEITE};
pub use nachrichten::{Nachricht, NACHRICHTEN_SEITE};
pub use notizbuch::{mappen_id, Filter, Graphknoten, Mappe, Notizgraph, Seite, PRO_SEITE};
pub use papierkorb::{Weggelegt, PAPIERKORB_SEITE, PAPIERKORB_TAGE};
pub use pgp::{Abgelegt as PgpAbgelegt, PgpSchluessel};
pub use projekte::{Projekt, Projektaenderung, Projektsache};
pub use protokoll::{Aenderung, Protokoll};
pub use sachen::{Kalender, Kontakt, Notiz, Termin, Weg};
pub use verlaufsfilter::Verlaufsfilter;

use rusqlite::Connection;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SpeicherFehler {
    #[error("SQLite: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// Die Datei stammt aus einer neueren Fassung des Programms.
    ///
    /// **Ein eigener Fehler und kein Abbruch mit einer SQL-Meldung**: Wer sein
    /// Telefon zurueckspielt und dabei eine aeltere Programmfassung erwischt,
    /// soll den Grund lesen koennen. Und vor allem: Die Datei bleibt
    /// unberuehrt. Ein Programm, das eine unbekannte Fassung "repariert",
    /// loescht Daten, die eine neuere Fassung noch verstuende.
    #[error(
        "This storage belongs to a newer version (schema {gefunden}, expected at most {bekannt})."
    )]
    ZuNeu { gefunden: i64, bekannt: i64 },
}

pub type Ergebnis<T> = Result<T, SpeicherFehler>;

/// Die geoeffnete Datei.
///
/// **Die Verbindung liegt hinter einem Mutex, und das ist keine Vorsicht,
/// sondern eine Notwendigkeit.** SQLites Verbindung ist `Send`, aber nicht
/// `Sync` -- sie darf wandern, aber nicht geteilt werden. Ohne diesen Mutex
/// waere `&Speicher` nicht ueber Threadgrenzen reichbar, und damit waere der
/// Laeufer nicht auf einem Nebenlaeufer zu starten: Eine Oberflaeche, die
/// waehrend des Abgleichs stehenbleibt, ist genau das, was offline-first
/// vermeiden soll.
///
/// Der Mutex ist **nicht wiedereintrittsfaehig**. Deshalb gilt hier
/// durchgehend: eine Sperre je Anweisung, und nie eine gehalten, waehrend eine
/// andere Methode dieses Typs gerufen wird. Wer das bricht, bekommt kein
/// Fehlerbild, sondern ein Programm, das steht.
#[derive(Debug)]
pub struct Speicher {
    db: Mutex<Connection>,
}

impl Speicher {
    /// Oeffnen und, falls noetig, aufbauen beziehungsweise nachziehen.
    pub fn oeffnen(pfad: impl AsRef<Path>) -> Ergebnis<Self> {
        let db = Connection::open(pfad)?;

        // WAL: Lesen blockiert nicht mehr, waehrend der Abgleich schreibt.
        // Auf einem Telefon ist das der Unterschied zwischen einer Liste, die
        // scrollt, und einer, die haengt, sobald im Hintergrund etwas laeuft.
        db.pragma_update(None, "journal_mode", "WAL")?;

        Self::einrichten(db)
    }

    /// Ein in sich stimmiges Abbild der ganzen Datei nach `ziel` -- fuer die
    /// Sicherung. `VACUUM INTO` liest unter einer Lesesperre und nimmt das
    /// WAL mit; ein blosses Kopieren der Datei saehe halbe Schreibvorgaenge.
    pub fn abbild_schreiben(&self, ziel: impl AsRef<Path>) -> Ergebnis<()> {
        let ziel = ziel.as_ref().to_string_lossy().to_string();
        self.db().execute("VACUUM INTO ?1", [ziel])?;
        Ok(())
    }

    /// Nur fuer Tests -- und ausdruecklich benannt, damit niemand
    /// versehentlich ein Programm baut, dessen Speicher beim Beenden weg ist.
    pub fn im_arbeitsspeicher() -> Ergebnis<Self> {
        Self::einrichten(Connection::open_in_memory()?)
    }

    fn einrichten(db: Connection) -> Ergebnis<Self> {
        // Fremdschluessel sind in SQLite je Verbindung abgeschaltet, nicht je
        // Datei. Wer das vergisst, hat Fremdschluessel, die nichts tun -- und
        // merkt es erst an verwaisten Zeilen.
        db.pragma_update(None, "foreign_keys", "ON")?;

        schema::nachziehen(&db)?;

        Ok(Self { db: Mutex::new(db) })
    }

    /// Der Stand des Schemas in dieser Datei.
    pub fn schemastand(&self) -> Ergebnis<i64> {
        schema::stand(&self.db())
    }

    /// Die Verbindung, gesperrt.
    ///
    /// Ein vergifteter Mutex heisst: In einer anderen Sperre ist ein Panik
    /// aufgetreten. Weiterzuarbeiten ist dann richtiger als mitzupaniken --
    /// SQLite selbst ist unversehrt, und ein Programm, das wegen eines
    /// fremden Fehlers keine Notizen mehr anzeigt, hilft niemandem.
    pub(crate) fn db(&self) -> MutexGuard<'_, Connection> {
        self.db
            .lock()
            .unwrap_or_else(|vergiftet| vergiftet.into_inner())
    }

    /// Fuer alles, was hier noch keinen Namen hat.
    ///
    /// Als Rueckruf und nicht als `&Connection`: Die Sperre muss enden, wenn
    /// die Abfrage endet, und ein herausgereichter Verweis liesse offen,
    /// wann das ist.
    ///
    /// Oeffentlich, weil dieses Crate die Oberflaeche nicht kennt und ihr
    /// nicht vorschreiben will, welche Abfrage sie braucht. Wer damit
    /// **schreibt**, umgeht allerdings das Protokoll -- und was nicht im
    /// Protokoll steht, wird nie abgeglichen.
    pub fn mit_verbindung<T>(&self, tun: impl FnOnce(&Connection) -> T) -> T {
        tun(&self.db())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ein_frischer_speicher_traegt_den_heutigen_stand() {
        let s = Speicher::im_arbeitsspeicher().unwrap();

        assert_eq!(s.schemastand().unwrap(), schema::STAND);
    }

    #[test]
    fn zweimal_oeffnen_baut_nicht_zweimal_auf() {
        let ordner = tempfile::tempdir().unwrap();
        let pfad = ordner.path().join("openany.sqlite");

        {
            let s = Speicher::oeffnen(&pfad).unwrap();
            s.notiz_schreiben(
                &Notiz {
                    zk_id: "zk-1".into(),
                    titel: "Bleibt".into(),
                    ..Default::default()
                },
                Protokoll::Merken,
            )
            .unwrap();
        }

        let wieder = Speicher::oeffnen(&pfad).unwrap();

        assert_eq!(wieder.schemastand().unwrap(), schema::STAND);
        assert_eq!(
            wieder.notiz("zk-1").unwrap().unwrap().titel,
            "Bleibt",
            "der zweite Aufbau haette sie geloescht"
        );
    }

    #[test]
    fn das_abbild_ist_eine_vollstaendige_datei() {
        let ordner = tempfile::tempdir().unwrap();
        let s = Speicher::oeffnen(ordner.path().join("openany.sqlite")).unwrap();
        s.notiz_schreiben(
            &Notiz {
                zk_id: "zk-1".into(),
                titel: "Gesichert".into(),
                ..Default::default()
            },
            Protokoll::Merken,
        )
        .unwrap();

        let abbild = ordner.path().join("abbild.sqlite");
        s.abbild_schreiben(&abbild).unwrap();

        let wieder = Speicher::oeffnen(&abbild).unwrap();
        assert_eq!(wieder.notiz("zk-1").unwrap().unwrap().titel, "Gesichert");
    }

    #[test]
    fn eine_neuere_datei_wird_nicht_angefasst() {
        let ordner = tempfile::tempdir().unwrap();
        let pfad = ordner.path().join("openany.sqlite");

        {
            let db = Connection::open(&pfad).unwrap();
            db.pragma_update(None, "user_version", schema::STAND + 7)
                .unwrap();
        }

        let fehler = Speicher::oeffnen(&pfad).unwrap_err();

        assert!(
            matches!(fehler, SpeicherFehler::ZuNeu { .. }),
            "statt {fehler:?}"
        );
    }
}
