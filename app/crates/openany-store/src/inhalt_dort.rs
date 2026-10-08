//! Was bei welcher Gegenstelle liegt -- ein Gedaechtnis, kein Stammdatum.
//!
//! **Warum es das gibt.** Zwei Geraete benennen Inhalte nach ihrem Abdruck
//! und koennen einander fragen: „welche dieser sha256 hast du?" ([`crate::Inhalte`],
//! `Gegenstelle::inhalte_da`). Der Server kennt diese Sicht nicht -- bei ihm
//! haengen Bytes an einer Datei oder einem Bild, einen Abdruck fuehrt er
//! nirgends. Fragen kann man ihn also nicht.
//!
//! Also merkt sich diese Seite, was sie selbst weiss: Ein Inhalt liegt
//! drueben, wenn er von dort kam oder erfolgreich dorthin ging.
//!
//! **Mehr behauptet die Tabelle nicht, und das genuegt.** Geht sie verloren
//! oder irrt sie sich zugunsten von „liegt nicht dort", wird hoechstens etwas
//! ein zweites Mal geschickt -- der Server erkennt dieselbe Datei am Abdruck
//! wieder und nimmt sie nicht doppelt an. Irrt sie sich andersherum, fehlt
//! drueben etwas, bis es jemand erneut anfasst. Deshalb wird ein Vermerk
//! **erst nach dem Gelingen** gesetzt: Ein abgebrochener Upload darf nicht als
//! erledigt gelten.
//!
//! **Sie reist nicht mit.** Was Geraet A ueber den Server weiss, gilt nicht
//! fuer Geraet B -- und stuende es im Abgleich, glaubte B es trotzdem.

use crate::{Ergebnis, Speicher};
use chrono::Utc;
use rusqlite::params;
use std::collections::BTreeSet;

impl Speicher {
    /// Vermerken: Diese Gegenstelle hat diesen Inhalt.
    pub fn inhalt_dort_merken(&self, basis: &str, abdruck: &str) -> Ergebnis<()> {
        self.db().execute(
            "INSERT INTO inhalt_dort (basis, abdruck, seit) VALUES (?1, ?2, ?3)
             ON CONFLICT (basis, abdruck) DO NOTHING",
            params![basis, abdruck, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    /// Welche dieser Abdruecke liegen bei der Gegenstelle?
    pub fn inhalte_dort(&self, basis: &str, abdruecke: &[String]) -> Ergebnis<BTreeSet<String>> {
        if abdruecke.is_empty() {
            return Ok(BTreeSet::new());
        }

        let db = self.db();
        let mut gefunden = BTreeSet::new();

        // In Haeppchen, damit die Abfrage nicht ueber SQLites Grenze fuer
        // Platzhalter laeuft (999). Wer 5000 Dateien hat, soll keine
        // Fehlermeldung bekommen, sondern eine Antwort.
        for teil in abdruecke.chunks(500) {
            let platzhalter = std::iter::repeat_n("?", teil.len())
                .collect::<Vec<_>>()
                .join(",");
            let mut abfrage = db.prepare(&format!(
                "SELECT abdruck FROM inhalt_dort
                  WHERE basis = ? AND abdruck IN ({platzhalter})"
            ))?;

            let mut werte: Vec<&dyn rusqlite::ToSql> = Vec::with_capacity(teil.len() + 1);
            werte.push(&basis);
            for a in teil {
                werte.push(a);
            }

            let treffer = abfrage.query_map(werte.as_slice(), |z| z.get::<_, String>(0))?;
            for t in treffer {
                gefunden.insert(t?);
            }
        }

        Ok(gefunden)
    }

    /// Einen Vermerk zuruecknehmen -- drueben gibt es den Inhalt nicht (mehr).
    pub fn inhalt_dort_vergessen(&self, basis: &str, abdruck: &str) -> Ergebnis<()> {
        self.db().execute(
            "DELETE FROM inhalt_dort WHERE basis = ?1 AND abdruck = ?2",
            params![basis, abdruck],
        )?;
        Ok(())
    }

    /// Alles vergessen, was zu einer Gegenstelle gehoert.
    ///
    /// Beim Entkoppeln: Was drueben liegt, geht diese Seite dann nichts mehr
    /// an -- und beim naechsten Koppeln (womoeglich mit einem anderen Konto)
    /// waere jeder alte Vermerk eine Behauptung ueber eine fremde Ablage.
    pub fn inhalt_dort_leeren(&self, basis: &str) -> Ergebnis<usize> {
        Ok(self
            .db()
            .execute("DELETE FROM inhalt_dort WHERE basis = ?1", params![basis])?)
    }
}

#[cfg(test)]
mod tests {
    use crate::Speicher;

    fn speicher() -> Speicher {
        Speicher::im_arbeitsspeicher().unwrap()
    }

    #[test]
    fn gemerktes_wird_wiedergefunden() {
        let s = speicher();
        s.inhalt_dort_merken("https://openany.de", "aa").unwrap();
        s.inhalt_dort_merken("https://openany.de", "bb").unwrap();

        let da = s
            .inhalte_dort(
                "https://openany.de",
                &["aa".into(), "cc".into(), "bb".into()],
            )
            .unwrap();

        assert_eq!(da.len(), 2);
        assert!(da.contains("aa") && da.contains("bb"));
        assert!(!da.contains("cc"));
    }

    /// Jede Gegenstelle hat ihre eigene Ablage. Was auf dem Server liegt,
    /// sagt nichts darueber, was das Tablet hat.
    #[test]
    fn jede_gegenstelle_fuer_sich() {
        let s = speicher();
        s.inhalt_dort_merken("https://openany.de", "aa").unwrap();

        assert!(s
            .inhalte_dort("nah:abc", &["aa".into()])
            .unwrap()
            .is_empty());
    }

    #[test]
    fn zweimal_merken_stoert_nicht() {
        let s = speicher();
        s.inhalt_dort_merken("x", "aa").unwrap();
        s.inhalt_dort_merken("x", "aa").unwrap();

        assert_eq!(s.inhalte_dort("x", &["aa".into()]).unwrap().len(), 1);
    }

    #[test]
    fn vergessen_nimmt_den_vermerk_zurueck() {
        let s = speicher();
        s.inhalt_dort_merken("x", "aa").unwrap();
        s.inhalt_dort_vergessen("x", "aa").unwrap();

        assert!(s.inhalte_dort("x", &["aa".into()]).unwrap().is_empty());
    }

    #[test]
    fn leeren_trifft_nur_die_eine_gegenstelle() {
        let s = speicher();
        s.inhalt_dort_merken("x", "aa").unwrap();
        s.inhalt_dort_merken("y", "aa").unwrap();

        assert_eq!(s.inhalt_dort_leeren("x").unwrap(), 1);
        assert!(s.inhalte_dort("x", &["aa".into()]).unwrap().is_empty());
        assert_eq!(s.inhalte_dort("y", &["aa".into()]).unwrap().len(), 1);
    }

    /// Ueber SQLites Grenze fuer Platzhalter hinaus (999) -- wer viele
    /// Dateien hat, soll eine Antwort bekommen und keine Fehlermeldung.
    #[test]
    fn auch_sehr_viele_abdruecke_gehen() {
        let s = speicher();
        let viele: Vec<String> = (0..1500).map(|i| format!("abdruck{i}")).collect();
        s.inhalt_dort_merken("x", "abdruck1200").unwrap();

        let da = s.inhalte_dort("x", &viele).unwrap();

        assert_eq!(da.len(), 1);
        assert!(da.contains("abdruck1200"));
    }

    #[test]
    fn ohne_frage_keine_antwort() {
        assert!(speicher().inhalte_dort("x", &[]).unwrap().is_empty());
    }
}
