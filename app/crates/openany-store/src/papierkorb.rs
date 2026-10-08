//! Was im Papierkorb liegt -- ueber alle Arten, die dieses Programm traegt.
//!
//! Eine gemischte, chronologische Liste wie drueben (`TrashController`):
//! zuletzt Geloeschtes zuerst. Wiederherstellen und endgueltig Loeschen stehen
//! schon in `sachen.rs`; hier steht nur die Liste.

use crate::{Ergebnis, Speicher};
use rusqlite::params;

/// Ein Eintrag der Liste, mit den Namen der Webapp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Weggelegt {
    /// `note`, `calendar`, `event`, `contact` -- wie ueber der Leitung.
    pub art: &'static str,
    pub schluessel: String,
    pub name: String,
    /// Wo es lag: Mappe einer Notiz, Kalender eines Termins.
    pub kontext: Option<String>,
    pub seit: String,
}

pub const PAPIERKORB_SEITE: usize = 50;

/// Wie lange etwas im Papierkorb liegt -- dieselbe Zahl wie drueben
/// (`php artisan trash:prune --days=30`). Die Ansicht sagt sie auch so an.
pub const PAPIERKORB_TAGE: i64 = 30;

impl Speicher {
    /// Eine Seite des Papierkorbs und ob es weitere gibt.
    pub fn papierkorb_seite(&self, seite: usize) -> Ergebnis<(Vec<Weggelegt>, bool)> {
        let mut alle = Vec::new();
        let db = self.db();

        let mut abfrage = db.prepare(
            "SELECT zk_id, titel, mappe, papierkorb_at FROM notizen WHERE papierkorb_at IS NOT NULL",
        )?;
        for zeile in abfrage.query_map(params![], |z| {
            Ok(Weggelegt {
                art: "note",
                schluessel: z.get(0)?,
                name: z.get(1)?,
                kontext: z.get::<_, Option<String>>(2)?.filter(|m| !m.is_empty()),
                seit: z.get(3)?,
            })
        })? {
            alle.push(zeile?);
        }

        let mut abfrage = db.prepare(
            "SELECT uuid, name, papierkorb_at FROM kalender WHERE papierkorb_at IS NOT NULL",
        )?;
        for zeile in abfrage.query_map(params![], |z| {
            Ok(Weggelegt {
                art: "calendar",
                schluessel: z.get(0)?,
                name: z.get(1)?,
                kontext: None,
                seit: z.get(2)?,
            })
        })? {
            alle.push(zeile?);
        }

        // Der Kalendername als Kontext -- auch wenn der Kalender selbst im
        // Papierkorb liegt, sonst stuende da nur "Termin".
        let mut abfrage = db.prepare(
            "SELECT t.uuid, t.titel, k.name, t.papierkorb_at
             FROM termine t LEFT JOIN kalender k ON k.uuid = t.kalender_uuid
             WHERE t.papierkorb_at IS NOT NULL",
        )?;
        for zeile in abfrage.query_map(params![], |z| {
            Ok(Weggelegt {
                art: "event",
                schluessel: z.get(0)?,
                name: z.get(1)?,
                kontext: z.get(2)?,
                seit: z.get(3)?,
            })
        })? {
            alle.push(zeile?);
        }

        let mut abfrage = db.prepare(
            "SELECT uuid, anzeigename, papierkorb_at FROM kontakte WHERE papierkorb_at IS NOT NULL",
        )?;
        for zeile in abfrage.query_map(params![], |z| {
            Ok(Weggelegt {
                art: "contact",
                schluessel: z.get(0)?,
                name: z.get(1)?,
                kontext: None,
                seit: z.get(2)?,
            })
        })? {
            alle.push(zeile?);
        }

        // Dateien und Ordner: nur, was nicht MIT seinem Ordner hineinging --
        // sonst stuende jede Datei eines weggelegten Ordners einzeln da. Die
        // Art heisst wie drueben (`TrashRegistry`): `file` in der Zone
        // Dateien, `document` in den Akten; Kontext ist der Ordner.
        let mut abfrage = db.prepare(
            "SELECT d.uuid, d.name, d.zone, d.papierkorb_at, e.name
             FROM dateien d LEFT JOIN dateien e ON e.uuid = d.eltern
             WHERE d.papierkorb_at IS NOT NULL
               AND (e.uuid IS NULL OR e.papierkorb_at IS NOT d.papierkorb_at)",
        )?;
        for zeile in abfrage.query_map(params![], |z| {
            Ok(Weggelegt {
                art: if z.get::<_, String>(2)? == "documents" {
                    "document"
                } else {
                    "file"
                },
                schluessel: z.get(0)?,
                name: z.get(1)?,
                kontext: z.get(4)?,
                seit: z.get(3)?,
            })
        })? {
            alle.push(zeile?);
        }

        // Galerie wie drueben (`TrashRegistry`): `album` nur, was nicht mit
        // seinem Elternalbum hineinging; Bilder als `album_image` bzw. auf
        // der obersten Ebene `gallery_image`, und nicht die, die mit ihrem
        // Album hineingingen.
        let mut abfrage = db.prepare(
            "SELECT a.uuid, a.name, a.papierkorb_at, e.name
             FROM alben a LEFT JOIN alben e ON e.uuid = a.eltern
             WHERE a.papierkorb_at IS NOT NULL
               AND (e.uuid IS NULL OR e.papierkorb_at IS NOT a.papierkorb_at)",
        )?;
        for zeile in abfrage.query_map(params![], |z| {
            Ok(Weggelegt {
                art: "album",
                schluessel: z.get(0)?,
                name: z.get(1)?,
                kontext: z.get(3)?,
                seit: z.get(2)?,
            })
        })? {
            alle.push(zeile?);
        }
        let mut abfrage = db.prepare(
            "SELECT m.uuid, m.name, m.papierkorb_at, a.name, m.album
             FROM medien m LEFT JOIN alben a ON a.uuid = m.album
             WHERE m.papierkorb_at IS NOT NULL
               AND (a.uuid IS NULL OR a.papierkorb_at IS NOT m.papierkorb_at)",
        )?;
        for zeile in abfrage.query_map(params![], |z| {
            Ok(Weggelegt {
                art: if z.get::<_, Option<String>>(4)?.is_some() {
                    "album_image"
                } else {
                    "gallery_image"
                },
                schluessel: z.get(0)?,
                name: z.get(1)?,
                kontext: z.get(3)?,
                seit: z.get(2)?,
            })
        })? {
            alle.push(zeile?);
        }

        alle.sort_by(|a, b| {
            b.seit
                .cmp(&a.seit)
                .then_with(|| b.schluessel.cmp(&a.schluessel))
        });

        let anfang = (seite.max(1) - 1) * PAPIERKORB_SEITE;
        let weitere = alle.len() > anfang + PAPIERKORB_SEITE;
        Ok((
            alle.into_iter()
                .skip(anfang)
                .take(PAPIERKORB_SEITE)
                .collect(),
            weitere,
        ))
    }
}

impl Speicher {
    /// Was laenger als [`PAPIERKORB_TAGE`] im Papierkorb liegt, endgueltig
    /// entfernen -- mit Grabstein im Protokoll, damit die Gegenseite es nicht
    /// zurueckspielt.
    ///
    /// Laeuft beim Start des Programms. Ohne das stuende in der Ansicht eine
    /// Zusage ("nach 30 Tagen automatisch entfernt"), die hier niemand haelt.
    ///
    /// @return wie viele Sachen fort sind
    pub fn papierkorb_aufraeumen(&self) -> Ergebnis<usize> {
        let grenze = (chrono::Utc::now() - chrono::Duration::days(PAPIERKORB_TAGE)).to_rfc3339();
        let mut fort = 0;
        let mut seite = 1;

        loop {
            let (liste, weitere) = self.papierkorb_seite(seite)?;
            let mut hier = 0;
            for w in liste.iter().filter(|w| w.seit.as_str() < grenze.as_str()) {
                if matches!(w.art, "file" | "document") {
                    self.datei_loeschen(&w.schluessel, crate::Protokoll::Merken)?;
                    hier += 1;
                    continue;
                }
                if w.art == "album" {
                    self.album_loeschen(&w.schluessel, crate::Protokoll::Merken)?;
                    hier += 1;
                    continue;
                }
                if matches!(w.art, "album_image" | "gallery_image") {
                    self.bild_loeschen(&w.schluessel, crate::Protokoll::Merken)?;
                    hier += 1;
                    continue;
                }
                let art = openany_client::Art::aus(w.art);
                if art == openany_client::Art::Kontakt {
                    self.kontaktfoto_uebernehmen(
                        &w.schluessel,
                        None,
                        None,
                        crate::Protokoll::Still,
                    )?;
                }
                self.loeschen(&art, &w.schluessel, crate::Protokoll::Merken)?;
                hier += 1;
            }
            fort += hier;
            // Die Liste ist neueste zuerst: Wurde auf dieser Seite etwas
            // entfernt, rueckt der Rest nach -- dieselbe Seite noch einmal.
            if hier == 0 {
                if !weitere {
                    return Ok(fort);
                }
                seite += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Kalender, Kontakt, Notiz, Protokoll, Termin};
    use openany_client::{Art, Terminfelder};

    #[test]
    fn alle_arten_neueste_zuerst_und_zurueckholbar() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        s.notiz_schreiben(
            &Notiz {
                zk_id: "n".into(),
                titel: "Einkauf".into(),
                mappe: Some("Haus".into()),
                ..Default::default()
            },
            Protokoll::Still,
        )
        .unwrap();
        s.kalender_schreiben(
            &Kalender {
                uuid: "k".into(),
                name: "Familie".into(),
                ..Default::default()
            },
            Protokoll::Still,
        )
        .unwrap();
        s.termin_schreiben(
            &Termin {
                uuid: "t".into(),
                kalender_uuid: "k".into(),
                felder: Terminfelder {
                    titel: "Elternabend".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            Protokoll::Still,
        )
        .unwrap();
        s.kontakt_schreiben(
            &Kontakt {
                uuid: "c".into(),
                anzeigename: "Hannah".into(),
                ..Default::default()
            },
            Protokoll::Still,
        )
        .unwrap();

        for (art, key) in [(Art::Notiz, "n"), (Art::Termin, "t"), (Art::Kontakt, "c")] {
            s.papierkorb(&art, key, Protokoll::Merken).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        let (liste, weitere) = s.papierkorb_seite(1).unwrap();
        let arten: Vec<_> = liste.iter().map(|w| w.art).collect();
        assert_eq!(arten, vec!["contact", "event", "note"]);
        assert!(!weitere);
        assert_eq!(liste[1].kontext.as_deref(), Some("Familie"));
        assert_eq!(liste[2].kontext.as_deref(), Some("Haus"));

        s.wiederherstellen(&Art::Notiz, "n", Protokoll::Merken)
            .unwrap();
        assert_eq!(s.papierkorb_seite(1).unwrap().0.len(), 2);
    }

    #[test]
    fn nach_dreissig_tagen_ist_es_fort_und_hinterlaesst_einen_grabstein() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        for id in ["alt", "neu"] {
            s.notiz_schreiben(
                &Notiz {
                    zk_id: id.into(),
                    titel: id.into(),
                    ..Default::default()
                },
                Protokoll::Still,
            )
            .unwrap();
            s.papierkorb(&Art::Notiz, id, Protokoll::Still).unwrap();
        }
        let vor_31_tagen = (chrono::Utc::now() - chrono::Duration::days(31)).to_rfc3339();
        s.mit_verbindung(|db| {
            db.execute(
                "UPDATE notizen SET papierkorb_at = ?1 WHERE zk_id = 'alt'",
                [&vor_31_tagen],
            )
            .unwrap()
        });
        let marke = s.eigene_marke_jetzt().unwrap();

        assert_eq!(s.papierkorb_aufraeumen().unwrap(), 1);

        assert!(s.notiz("alt").unwrap().is_none());
        assert!(s.notiz("neu").unwrap().is_some());
        let (aenderungen, _, _) = s.aenderungen_seit(marke).unwrap();
        assert_eq!(aenderungen.len(), 1, "der Grabstein fuer die alte Notiz");
    }

    #[test]
    fn ein_alter_ordner_im_papierkorb_wird_mit_inhalt_entfernt() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        for (uuid, eltern, ordner) in [("o", None, true), ("d", Some("o"), false)] {
            s.datei_schreiben(
                &crate::Datei {
                    uuid: uuid.into(),
                    zone: "files".into(),
                    ist_ordner: ordner,
                    eltern: eltern.map(Into::into),
                    name: uuid.into(),
                    ..Default::default()
                },
                Protokoll::Still,
            )
            .unwrap();
        }
        s.datei_papierkorb("o", Protokoll::Still).unwrap();
        s.mit_verbindung(|db| {
            db.execute(
                "UPDATE dateien SET papierkorb_at = '2000-01-01T00:00:00+00:00'",
                [],
            )
            .unwrap()
        });

        let (liste, _) = s.papierkorb_seite(1).unwrap();
        assert_eq!(liste.len(), 1, "nur der Ordner, nicht jede Datei darin");
        assert_eq!(s.papierkorb_aufraeumen().unwrap(), 1);
        assert!(s.datei("d").unwrap().is_none());
    }
}
