//! Was an einer Notiz-Mappe haengt: die Angaben der Druckfassung und welche
//! Datei ihre Quellenliste ist (15.10.2026).
//!
//! Drueben stehen beide an `note_folders` (`druckfassung`,
//! `library_file_node_id`). Ueber die Leitung geht die Mappe als `note_folder`
//! mit ihrem PFAD als Schluessel -- dieselbe Regel wie beim Anhang: Die
//! Mappen-Id gilt nur auf einer Seite.
//!
//! **Das letzte Schreiben gewinnt** (Tiffy, 15.10.2026). Die Angaben sind
//! ein Titelblatt, keine Arbeit, die man verlieren koennte; die Quellenliste
//! selbst ist eine Datei und reist als solche.
//!
//! **Die Angaben bleiben JSON-Text**, wie sie aus der Oberflaeche kommen. Der
//! Speicher muss sie nicht verstehen -- nur aufheben und weiterreichen.

use crate::{Ergebnis, Protokoll, Speicher};
use openany_client::{Art, Was};
use rusqlite::{params, OptionalExtension, Row};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Mappenangaben {
    /// Der Pfad der Mappe (`Uni/Hausarbeit`).
    pub pfad: String,
    /// Die Angaben der Druckfassung als JSON-Objekt, oder `None`.
    pub druckfassung: Option<String>,
    /// Die uuid der Datei mit der Quellenliste (CSL-JSON), oder `None`.
    pub bibliothek: Option<String>,
    pub geaendert_at: String,
}

const SPALTEN: &str = "pfad, druckfassung, bibliothek, geaendert_at";

fn aus(z: &Row<'_>) -> rusqlite::Result<Mappenangaben> {
    Ok(Mappenangaben {
        pfad: z.get(0)?,
        druckfassung: z.get(1)?,
        bibliothek: z.get(2)?,
        geaendert_at: z.get(3)?,
    })
}

fn liegt_in(pfad: &str, wurzel: &str) -> bool {
    pfad == wurzel || pfad.starts_with(&format!("{wurzel}/"))
}

impl Speicher {
    pub fn mappenangaben_schreiben(&self, m: &Mappenangaben, protokoll: Protokoll) -> Ergebnis<()> {
        let geaendert = if m.geaendert_at.is_empty() {
            chrono::Utc::now().to_rfc3339()
        } else {
            m.geaendert_at.clone()
        };
        self.db().execute(
            &format!(
                "INSERT INTO mappen_angaben ({SPALTEN}) VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT (pfad) DO UPDATE SET
                    druckfassung = excluded.druckfassung,
                    bibliothek = excluded.bibliothek,
                    geaendert_at = excluded.geaendert_at"
            ),
            params![m.pfad, m.druckfassung, m.bibliothek, geaendert],
        )?;

        match protokoll {
            Protokoll::Merken => self.merken(&Art::Mappe, &m.pfad, Was::Da),
            Protokoll::Still => Ok(()),
            Protokoll::Von(h) => self.merken_von(&Art::Mappe, &m.pfad, Was::Da, Some(h)),
        }
    }

    pub fn mappenangaben(&self, pfad: &str) -> Ergebnis<Option<Mappenangaben>> {
        Ok(self
            .db()
            .query_row(
                &format!("SELECT {SPALTEN} FROM mappen_angaben WHERE pfad = ?1"),
                params![pfad],
                aus,
            )
            .optional()?)
    }

    pub fn mappenangaben_alle(&self) -> Ergebnis<Vec<Mappenangaben>> {
        let db = self.db();
        let mut abfrage = db.prepare(&format!("SELECT {SPALTEN} FROM mappen_angaben"))?;
        let liste = abfrage.query_map([], aus)?.collect::<Result<_, _>>()?;
        Ok(liste)
    }

    /// Eine Mappe wurde umbenannt: Ihre Angaben und die ihrer Untermappen
    /// ziehen mit -- und gehen unter dem neuen Pfad hinaus, damit drueben die
    /// umbenannte Mappe sie bekommt.
    pub fn mappenangaben_umziehen(&self, alt: &str, neu: &str) -> Ergebnis<()> {
        for mut m in self.mappenangaben_alle()? {
            if !liegt_in(&m.pfad, alt) {
                continue;
            }
            let vorher = m.pfad.clone();
            m.pfad = format!("{neu}{}", &vorher[alt.len()..]);
            m.geaendert_at = String::new();
            self.db()
                .execute("DELETE FROM mappen_angaben WHERE pfad = ?1", [&vorher])?;
            self.mappenangaben_schreiben(&m, Protokoll::Merken)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s() -> Speicher {
        Speicher::im_arbeitsspeicher().unwrap()
    }

    #[test]
    fn schreiben_lesen_und_merken() {
        let s = s();
        let m = Mappenangaben {
            pfad: "Uni/Arbeit".into(),
            druckfassung: Some(r#"{"titel":"T"}"#.into()),
            bibliothek: None,
            geaendert_at: String::new(),
        };
        s.mappenangaben_schreiben(&m, Protokoll::Merken).unwrap();

        let da = s.mappenangaben("Uni/Arbeit").unwrap().unwrap();
        assert_eq!(da.druckfassung, m.druckfassung);
        assert!(!da.geaendert_at.is_empty());

        let (aenderungen, _, _) = s.aenderungen_seit(0).unwrap();
        assert!(aenderungen
            .iter()
            .any(|a| a.art == Art::Mappe && a.schluessel == "Uni/Arbeit"));
    }

    #[test]
    fn umbenennen_nimmt_untermappen_mit() {
        let s = s();
        for p in ["Uni", "Uni/Arbeit", "Unikat"] {
            s.mappenangaben_schreiben(
                &Mappenangaben {
                    pfad: p.into(),
                    druckfassung: Some("{}".into()),
                    ..Default::default()
                },
                Protokoll::Still,
            )
            .unwrap();
        }
        s.mappenangaben_umziehen("Uni", "Studium").unwrap();

        let mut pfade: Vec<_> = s
            .mappenangaben_alle()
            .unwrap()
            .into_iter()
            .map(|m| m.pfad)
            .collect();
        pfade.sort();
        assert_eq!(pfade, ["Studium", "Studium/Arbeit", "Unikat"]);
    }
}
