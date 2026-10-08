//! Dateien und Ordner -- das Gegenstueck zu openanys `file_nodes`.
//!
//! **Hier steht, WAS es gibt, nicht der Inhalt.** Die Bytes liegen in der
//! [`crate::Inhalte`]-Ablage, nach ihrem Abdruck; eine Zeile hier nennt den
//! Abdruck. Das trennt, was jedes Geraet vollstaendig kennen soll (Namen,
//! Ordner, Groessen), von dem, was ein Geraet mit wenig Platz nur bei Bedarf
//! holt (siehe Plan, Phase 3).
//!
//! **Zwei Zonen wie in der Webapp:** `files` (Ordner) und `documents`
//! (Akten). Verschoben wird nur innerhalb einer Zone.
//!
//! **Papierkorb mit Teilbaum.** Ein Ordner geht mit allem darin in den
//! Papierkorb, und alles kommt zusammen zurueck -- erkannt an demselben
//! Zeitpunkt, wie drueben (`FileNode::restoreWithDescendants`).

use crate::{Ergebnis, Protokoll, Speicher};
use chrono::Utc;
use openany_client::{Art, Was};
use rusqlite::{params, OptionalExtension, Row};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Datei {
    pub uuid: String,
    /// `files` oder `documents`.
    pub zone: String,
    pub ist_ordner: bool,
    /// uuid des Ordners; `None` = oberste Ebene der Zone.
    pub eltern: Option<String>,
    pub name: String,
    pub groesse: u64,
    pub mime: String,
    /// sha256 des Inhalts; bei Ordnern `None`.
    pub abdruck: Option<String>,
    pub papierkorb_at: Option<String>,
    pub geaendert_at: String,
}

impl Datei {
    /// Der Anzeigetyp wie drueben (`FileNode::displayType`).
    pub fn anzeigetyp(&self) -> &'static str {
        if self.ist_ordner {
            return "folder";
        }
        let mime = self.mime.to_lowercase();
        for (teil, typ) in [
            ("pdf", "pdf"),
            ("image", "image"),
            ("audio", "audio"),
            ("video", "video"),
            ("text", "text"),
        ] {
            if mime.contains(teil) {
                return typ;
            }
        }
        "file"
    }
}

const SPALTEN: &str =
    "uuid, zone, ist_ordner, eltern, name, groesse, mime, abdruck, papierkorb_at, geaendert_at";

fn datei_aus(z: &Row<'_>) -> rusqlite::Result<Datei> {
    Ok(Datei {
        uuid: z.get(0)?,
        zone: z.get(1)?,
        ist_ordner: z.get::<_, i64>(2)? != 0,
        eltern: z.get(3)?,
        name: z.get(4)?,
        groesse: z.get::<_, i64>(5)?.max(0) as u64,
        mime: z.get(6)?,
        abdruck: z.get(7)?,
        papierkorb_at: z.get(8)?,
        geaendert_at: z.get(9)?,
    })
}

impl Speicher {
    pub fn datei_schreiben(&self, d: &Datei, protokoll: Protokoll) -> Ergebnis<()> {
        let geaendert = if d.geaendert_at.is_empty() {
            Utc::now().to_rfc3339()
        } else {
            d.geaendert_at.clone()
        };
        self.db().execute(
            &format!(
                "INSERT INTO dateien ({SPALTEN}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                 ON CONFLICT (uuid) DO UPDATE SET
                    zone = excluded.zone, ist_ordner = excluded.ist_ordner,
                    eltern = excluded.eltern, name = excluded.name,
                    groesse = excluded.groesse, mime = excluded.mime,
                    abdruck = excluded.abdruck, papierkorb_at = excluded.papierkorb_at,
                    geaendert_at = excluded.geaendert_at"
            ),
            params![
                d.uuid,
                d.zone,
                d.ist_ordner as i64,
                d.eltern,
                d.name,
                d.groesse as i64,
                d.mime,
                d.abdruck,
                d.papierkorb_at,
                geaendert,
            ],
        )?;

        match protokoll {
            Protokoll::Merken => self.merken(&Art::Datei, &d.uuid, Was::Da),
            Protokoll::Still => Ok(()),
            Protokoll::Von(h) => self.merken_von(&Art::Datei, &d.uuid, Was::Da, Some(h)),
        }
    }

    pub fn datei(&self, uuid: &str) -> Ergebnis<Option<Datei>> {
        Ok(self
            .db()
            .query_row(
                &format!("SELECT {SPALTEN} FROM dateien WHERE uuid = ?1"),
                params![uuid],
                datei_aus,
            )
            .optional()?)
    }

    /// Was lebendig in einem Ordner liegt: Ordner zuerst, dann nach Namen
    /// (ohne Gross/klein) -- dieselbe Reihenfolge wie drueben.
    pub fn dateien_im_ordner(&self, zone: &str, eltern: Option<&str>) -> Ergebnis<Vec<Datei>> {
        let db = self.db();
        let mut abfrage = db.prepare(&format!(
            "SELECT {SPALTEN} FROM dateien
             WHERE zone = ?1 AND eltern IS ?2 AND papierkorb_at IS NULL
             ORDER BY ist_ordner DESC, name COLLATE NOCASE, name"
        ))?;
        let liste = abfrage
            .query_map(params![zone, eltern], datei_aus)?
            .collect::<Result<_, _>>()?;
        Ok(liste)
    }

    /// Die zuletzt geaenderten lebendigen Dateien (ohne Ordner) -- fuer die
    /// Kachel auf der Startseite.
    pub fn zuletzt_geaenderte_dateien(&self, wie_viele: usize) -> Ergebnis<Vec<Datei>> {
        let db = self.db();
        let mut abfrage = db.prepare(&format!(
            "SELECT {SPALTEN} FROM dateien
             WHERE ist_ordner = 0 AND papierkorb_at IS NULL
             ORDER BY geaendert_at DESC LIMIT ?1"
        ))?;
        let liste = abfrage
            .query_map(params![wie_viele as i64], datei_aus)?
            .collect::<Result<_, _>>()?;
        Ok(liste)
    }

    /// Alle lebendigen Dateien mit Inhalt -- fuer "was fehlt hier noch?".
    pub fn lebendige_dateien(&self) -> Ergebnis<Vec<Datei>> {
        let db = self.db();
        let mut abfrage = db.prepare(&format!(
            "SELECT {SPALTEN} FROM dateien
             WHERE ist_ordner = 0 AND papierkorb_at IS NULL AND abdruck IS NOT NULL"
        ))?;
        let liste = abfrage
            .query_map([], datei_aus)?
            .collect::<Result<_, _>>()?;
        Ok(liste)
    }

    /// Alle lebendigen Ordner einer Zone -- fuer "Verschieben nach".
    pub fn ordner_der_zone(&self, zone: &str) -> Ergebnis<Vec<Datei>> {
        let db = self.db();
        let mut abfrage = db.prepare(&format!(
            "SELECT {SPALTEN} FROM dateien
             WHERE zone = ?1 AND ist_ordner = 1 AND papierkorb_at IS NULL
             ORDER BY name COLLATE NOCASE"
        ))?;
        let liste = abfrage
            .query_map(params![zone], datei_aus)?
            .collect::<Result<_, _>>()?;
        Ok(liste)
    }

    /// Alles Lebendige einer Zone, Ordner zuerst -- fuer die Suche nach
    /// Namen (die Auswahl trifft der Aufrufer: SQLite kennt Gross und klein
    /// nur fuer ASCII, und „Ärztliche Bescheinigung" soll „ärzt" finden).
    pub fn dateien_der_zone(&self, zone: &str) -> Ergebnis<Vec<Datei>> {
        let db = self.db();
        let mut abfrage = db.prepare(&format!(
            "SELECT {SPALTEN} FROM dateien
             WHERE zone = ?1 AND papierkorb_at IS NULL
             ORDER BY ist_ordner DESC, name COLLATE NOCASE, name"
        ))?;
        let liste = abfrage
            .query_map(params![zone], datei_aus)?
            .collect::<Result<_, _>>()?;
        Ok(liste)
    }

    /// Dateien einer Zone, deren Text noch nicht (oder nicht mehr, weil sie
    /// sich geaendert haben) gelesen ist -- fuer den Inhaltsleser. Ob der
    /// Inhalt hier liegt, prueft der Aufrufer.
    pub fn dateien_ohne_text(&self, zone: &str, version: i64) -> Ergebnis<Vec<Datei>> {
        let spalten = SPALTEN
            .split(',')
            .map(|s| format!("d.{}", s.trim()))
            .collect::<Vec<_>>()
            .join(", ");
        let db = self.db();
        let mut abfrage = db.prepare(&format!(
            "SELECT {spalten} FROM dateien d
             LEFT JOIN dateitexte t ON t.uuid = d.uuid
             WHERE d.zone = ?1 AND d.ist_ordner = 0 AND d.papierkorb_at IS NULL
               AND d.abdruck IS NOT NULL
               AND (t.uuid IS NULL OR t.abdruck <> d.abdruck
                    OR (t.stand = 'unlesbar' AND t.version < ?2))
             ORDER BY d.geaendert_at DESC"
        ))?;
        let liste = abfrage
            .query_map(params![zone, version], datei_aus)?
            .collect::<Result<_, _>>()?;
        Ok(liste)
    }

    /// Den gelesenen Text ablegen -- nur zum Abdruck, aus dem er stammt.
    /// `false`: Die Datei hat sich inzwischen geaendert (oder ist fort).
    pub fn dateitext_setzen(
        &self,
        uuid: &str,
        abdruck: &str,
        stand: &str,
        text: Option<&str>,
        version: i64,
    ) -> Ergebnis<bool> {
        let passt = self
            .datei(uuid)?
            .is_some_and(|d| d.abdruck.as_deref() == Some(abdruck));
        if !passt {
            return Ok(false);
        }
        self.db().execute(
            "INSERT INTO dateitexte (uuid, abdruck, stand, text, version)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT (uuid) DO UPDATE SET
                abdruck = excluded.abdruck, stand = excluded.stand,
                text = excluded.text, version = excluded.version",
            params![uuid, abdruck, stand, text, version],
        )?;
        Ok(true)
    }

    /// Die gueltigen Texte einer Zone: uuid -> Text (nur zum aktuellen
    /// Abdruck, nur gelesene).
    pub fn dateitexte_der_zone(
        &self,
        zone: &str,
    ) -> Ergebnis<std::collections::HashMap<String, String>> {
        let db = self.db();
        let mut abfrage = db.prepare(
            "SELECT d.uuid, t.text FROM dateien d
             JOIN dateitexte t ON t.uuid = d.uuid AND t.abdruck = d.abdruck
             WHERE d.zone = ?1 AND d.papierkorb_at IS NULL AND t.stand = 'ok'
               AND t.text IS NOT NULL",
        )?;
        let liste = abfrage
            .query_map(params![zone], |z| Ok((z.get(0)?, z.get(1)?)))?
            .collect::<Result<_, _>>()?;
        Ok(liste)
    }

    /// Der Weg von oben bis zu diesem Ordner, ihn eingeschlossen.
    ///
    /// Bricht bei einem Kreis ab, statt ewig zu laufen -- entstehen kann er
    /// nur durch zwei Geraete, die gleichzeitig ueber Kreuz verschieben.
    pub fn ordnerweg(&self, uuid: Option<&str>) -> Ergebnis<Vec<Datei>> {
        let mut weg = Vec::new();
        let mut naechster = uuid.map(str::to_string);
        while let Some(u) = naechster {
            if weg.iter().any(|d: &Datei| d.uuid == u) || weg.len() > 64 {
                break;
            }
            let Some(d) = self.datei(&u)? else { break };
            naechster = d.eltern.clone();
            weg.push(d);
        }
        weg.reverse();
        Ok(weg)
    }

    /// Liegt schon etwas Lebendiges mit diesem Namen in dem Ordner?
    pub fn dateiname_vergeben(
        &self,
        zone: &str,
        eltern: Option<&str>,
        name: &str,
        ausser: Option<&str>,
    ) -> Ergebnis<bool> {
        Ok(self.db().query_row(
            "SELECT EXISTS (SELECT 1 FROM dateien
             WHERE zone = ?1 AND eltern IS ?2 AND papierkorb_at IS NULL
               AND lower(name) = lower(?3) AND uuid IS NOT ?4)",
            params![zone, eltern, name, ausser],
            |z| z.get::<_, i64>(0),
        )? != 0)
    }

    /// Alle Nachfahren eines Ordners, lebendig oder nicht.
    fn nachfahren(&self, uuid: &str) -> Ergebnis<Vec<Datei>> {
        let db = self.db();
        let mut abfrage = db.prepare(&format!(
            "WITH RECURSIVE baum(u) AS (
                SELECT uuid FROM dateien WHERE eltern = ?1
                UNION SELECT d.uuid FROM dateien d JOIN baum ON d.eltern = baum.u
             )
             SELECT {SPALTEN} FROM dateien WHERE uuid IN (SELECT u FROM baum)"
        ))?;
        let liste = abfrage
            .query_map(params![uuid], datei_aus)?
            .collect::<Result<_, _>>()?;
        Ok(liste)
    }

    /// Die uuids eines Ordners und von allem darin -- fuer "auf diesem Geraet
    /// behalten".
    pub fn ordner_mit_inhalt(&self, uuid: &str) -> Ergebnis<Vec<String>> {
        let mut alle = vec![uuid.to_string()];
        alle.extend(self.nachfahren(uuid)?.into_iter().map(|d| d.uuid));
        Ok(alle)
    }

    /// Ist `ziel` dieser Ordner selbst oder liegt es darin?
    pub fn liegt_in(&self, ziel: &str, ordner: &str) -> Ergebnis<bool> {
        Ok(ziel == ordner || self.nachfahren(ordner)?.iter().any(|d| d.uuid == ziel))
    }

    /// Eine Datei oder einen Ordner mit allem darin in den Papierkorb.
    pub fn datei_papierkorb(&self, uuid: &str, protokoll: Protokoll) -> Ergebnis<bool> {
        let jetzt = Utc::now().to_rfc3339();
        let mut betroffen = vec![uuid.to_string()];
        betroffen.extend(
            self.nachfahren(uuid)?
                .into_iter()
                .filter(|d| d.papierkorb_at.is_none())
                .map(|d| d.uuid),
        );

        let mut getan = false;
        for u in &betroffen {
            let getroffen = self.db().execute(
                "UPDATE dateien SET papierkorb_at = ?1, geaendert_at = ?1
                 WHERE uuid = ?2 AND papierkorb_at IS NULL",
                params![jetzt, u],
            )?;
            if getroffen > 0 {
                getan = true;
                self.protokoll_datei(protokoll, u, Was::Papierkorb)?;
            }
        }
        Ok(getan)
    }

    /// Zurueck aus dem Papierkorb, mit allem, was im selben Moment mitging --
    /// und mit den Ordnern darueber, falls die auch dort liegen.
    pub fn datei_wiederherstellen(&self, uuid: &str, protokoll: Protokoll) -> Ergebnis<bool> {
        let Some(d) = self.datei(uuid)? else {
            return Ok(false);
        };
        let Some(seit) = d.papierkorb_at.clone() else {
            return Ok(false);
        };

        let mut betroffen = vec![d.uuid.clone()];
        betroffen.extend(
            self.nachfahren(uuid)?
                .into_iter()
                .filter(|n| n.papierkorb_at.as_deref() == Some(seit.as_str()))
                .map(|n| n.uuid),
        );
        // Ein Ordner darueber im Papierkorb liesse die Datei unsichtbar
        // zurueckkommen. Er kommt mit, aber nur er, nicht seine Geschwister.
        for oben in self.ordnerweg(d.eltern.as_deref())? {
            if oben.papierkorb_at.is_some() {
                betroffen.push(oben.uuid);
            }
        }

        let jetzt = Utc::now().to_rfc3339();
        for u in &betroffen {
            let getroffen = self.db().execute(
                "UPDATE dateien SET papierkorb_at = NULL, geaendert_at = ?1
                 WHERE uuid = ?2 AND papierkorb_at IS NOT NULL",
                params![jetzt, u],
            )?;
            if getroffen > 0 {
                self.protokoll_datei(protokoll, u, Was::Da)?;
            }
        }
        Ok(true)
    }

    /// Endgueltig fort, mit allem darin. Gibt die Abdruecke zurueck, die dabei
    /// frei wurden -- ob ihr Inhalt geloescht werden darf, entscheidet
    /// [`Speicher::abdruck_benutzt`].
    pub fn datei_loeschen(&self, uuid: &str, protokoll: Protokoll) -> Ergebnis<Vec<String>> {
        let mut betroffen = self.nachfahren(uuid)?;
        if let Some(d) = self.datei(uuid)? {
            betroffen.push(d);
        }
        let mut frei = Vec::new();
        for d in &betroffen {
            self.db()
                .execute("DELETE FROM dateien WHERE uuid = ?1", params![d.uuid])?;
            self.protokoll_datei(protokoll, &d.uuid, Was::Fort)?;
            if let Some(a) = &d.abdruck {
                frei.push(a.clone());
            }
        }
        Ok(frei)
    }

    /// Nennt noch irgendeine Zeile -- auch eine im Papierkorb -- diesen Inhalt?
    pub fn abdruck_benutzt(&self, abdruck: &str) -> Ergebnis<bool> {
        Ok(self.db().query_row(
            // Auch Nachrichten nennen Inhalte: einen selbst gesendeten Anhang
            // (Stand 13). Ohne diese Zeile räumte das Aufräumen ihn weg.
            "SELECT EXISTS (SELECT 1 FROM dateien WHERE abdruck = ?1)
                 OR EXISTS (SELECT 1 FROM medien WHERE abdruck = ?1 OR vorschau = ?1)
                 OR EXISTS (SELECT 1 FROM nachrichten WHERE json_extract(anhang, '$.abdruck') = ?1)
                 OR EXISTS (SELECT 1 FROM mails, json_each(mails.anhaenge) a
                            WHERE json_extract(a.value, '$.abdruck') = ?1)",
            params![abdruck],
            |z| z.get::<_, i64>(0),
        )? != 0)
    }

    /// Welche Sache traegt diesen Inhalt? -- `(Art, uuid)`.
    ///
    /// **Die Bruecke zum Server, und nur dorthin.** Zwischen zwei Geraeten
    /// heisst ein Inhalt nach seinem Abdruck: Sie legen ihn unter `sha256` ab,
    /// und derselbe Inhalt existiert dort genau einmal, gleich wie viele
    /// Dateien ihn nennen. Der Server kennt diese Sicht gar nicht -- bei ihm
    /// haengen Bytes an einer Datei oder einem Bild, und `GET /sync/content`
    /// fragt nach `type` und `key`. Einer von beiden muss uebersetzen, und nur
    /// hier liegt das Wissen dafuer.
    ///
    /// Nennen mehrere Dateien denselben Inhalt, ist jede eine richtige
    /// Antwort: Die Bytes sind dieselben. Genommen wird die erste, die nicht
    /// im Papierkorb liegt -- Geloeschtes gibt der Server nicht mehr her.
    ///
    /// Das Vorschaubild eines Mediums bleibt aussen vor: Es entsteht auf
    /// jeder Seite selbst und reist nie ueber diesen Weg.
    pub fn sache_zu_abdruck(&self, abdruck: &str) -> Ergebnis<Option<(Art, String)>> {
        let db = self.db();

        let datei: Option<String> = db
            .query_row(
                "SELECT uuid FROM dateien
                  WHERE abdruck = ?1 AND papierkorb_at IS NULL
                  LIMIT 1",
                params![abdruck],
                |z| z.get(0),
            )
            .optional()?;

        if let Some(uuid) = datei {
            return Ok(Some((Art::Datei, uuid)));
        }

        let bild: Option<String> = db
            .query_row(
                "SELECT uuid FROM medien
                  WHERE abdruck = ?1 AND papierkorb_at IS NULL
                  LIMIT 1",
                params![abdruck],
                |z| z.get(0),
            )
            .optional()?;

        Ok(bild.map(|uuid| (Art::Medium, uuid)))
    }

    /// Alle Abdruecke, die eine Zeile nennt.
    pub fn benutzte_abdruecke(&self) -> Ergebnis<std::collections::BTreeSet<String>> {
        let db = self.db();
        let mut abfrage = db.prepare(
            "SELECT abdruck FROM dateien WHERE abdruck IS NOT NULL
                 UNION SELECT abdruck FROM medien WHERE abdruck IS NOT NULL
                 UNION SELECT vorschau FROM medien WHERE vorschau IS NOT NULL
                 UNION SELECT json_extract(anhang, '$.abdruck') FROM nachrichten
                       WHERE json_extract(anhang, '$.abdruck') IS NOT NULL
                 UNION SELECT json_extract(a.value, '$.abdruck') FROM mails, json_each(mails.anhaenge) a
                       WHERE json_extract(a.value, '$.abdruck') IS NOT NULL
                 UNION SELECT json_extract(felder, '$.abdruck') FROM projektsachen
                       WHERE art IN ('geteilte_datei', 'geteiltes_bild')
                         AND json_extract(felder, '$.abdruck') IS NOT NULL
                 UNION SELECT json_extract(felder, '$.vorschau') FROM projektsachen
                       WHERE art = 'geteiltes_bild' AND json_extract(felder, '$.vorschau') IS NOT NULL"
        )?;
        let menge = abfrage
            .query_map([], |z| z.get::<_, String>(0))?
            .collect::<Result<_, _>>()?;
        Ok(menge)
    }

    fn protokoll_datei(&self, protokoll: Protokoll, uuid: &str, was: Was) -> Ergebnis<()> {
        match protokoll {
            Protokoll::Merken => self.merken(&Art::Datei, uuid, was),
            Protokoll::Still => Ok(()),
            Protokoll::Von(h) => self.merken_von(&Art::Datei, uuid, was, Some(h)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s() -> Speicher {
        Speicher::im_arbeitsspeicher().unwrap()
    }

    fn ordner(uuid: &str, eltern: Option<&str>, name: &str) -> Datei {
        Datei {
            uuid: uuid.into(),
            zone: "files".into(),
            ist_ordner: true,
            eltern: eltern.map(Into::into),
            name: name.into(),
            ..Default::default()
        }
    }

    fn datei(uuid: &str, eltern: Option<&str>, name: &str, abdruck: &str) -> Datei {
        Datei {
            uuid: uuid.into(),
            zone: "files".into(),
            eltern: eltern.map(Into::into),
            name: name.into(),
            groesse: 3,
            mime: "application/pdf".into(),
            abdruck: Some(abdruck.into()),
            ..Default::default()
        }
    }

    #[test]
    fn ordner_zuerst_dann_nach_namen() {
        let s = s();
        s.datei_schreiben(&datei("d1", None, "zebra.pdf", "a"), Protokoll::Merken)
            .unwrap();
        s.datei_schreiben(&ordner("o1", None, "Belege"), Protokoll::Merken)
            .unwrap();
        s.datei_schreiben(&datei("d2", None, "Apfel.pdf", "b"), Protokoll::Merken)
            .unwrap();

        let namen: Vec<_> = s
            .dateien_im_ordner("files", None)
            .unwrap()
            .into_iter()
            .map(|d| d.name)
            .collect();
        assert_eq!(namen, ["Belege", "Apfel.pdf", "zebra.pdf"]);
    }

    #[test]
    fn ein_ordner_geht_mit_allem_darin_und_kommt_mit_allem_zurueck() {
        let s = s();
        s.datei_schreiben(&ordner("o1", None, "Steuer"), Protokoll::Merken)
            .unwrap();
        s.datei_schreiben(&ordner("o2", Some("o1"), "2026"), Protokoll::Merken)
            .unwrap();
        s.datei_schreiben(
            &datei("d1", Some("o2"), "Beleg.pdf", "a"),
            Protokoll::Merken,
        )
        .unwrap();

        assert!(s.datei_papierkorb("o1", Protokoll::Merken).unwrap());
        assert!(s.datei("d1").unwrap().unwrap().papierkorb_at.is_some());
        assert!(s.dateien_im_ordner("files", None).unwrap().is_empty());

        s.datei_wiederherstellen("o1", Protokoll::Merken).unwrap();
        assert!(s.datei("d1").unwrap().unwrap().papierkorb_at.is_none());
    }

    #[test]
    fn eine_datei_bringt_ihren_ordner_mit_zurueck() {
        let s = s();
        s.datei_schreiben(&ordner("o1", None, "Steuer"), Protokoll::Merken)
            .unwrap();
        s.datei_schreiben(
            &datei("d1", Some("o1"), "Beleg.pdf", "a"),
            Protokoll::Merken,
        )
        .unwrap();
        s.datei_papierkorb("o1", Protokoll::Merken).unwrap();

        s.datei_wiederherstellen("d1", Protokoll::Merken).unwrap();
        assert!(s.datei("o1").unwrap().unwrap().papierkorb_at.is_none());
    }

    #[test]
    fn endgueltig_loeschen_nennt_die_frei_gewordenen_abdruecke() {
        let s = s();
        s.datei_schreiben(&ordner("o1", None, "Alt"), Protokoll::Merken)
            .unwrap();
        s.datei_schreiben(&datei("d1", Some("o1"), "a.pdf", "abc"), Protokoll::Merken)
            .unwrap();
        s.datei_schreiben(&datei("d2", None, "kopie.pdf", "abc"), Protokoll::Merken)
            .unwrap();

        let frei = s.datei_loeschen("o1", Protokoll::Merken).unwrap();
        assert_eq!(frei, ["abc"]);
        assert!(
            s.abdruck_benutzt("abc").unwrap(),
            "die Kopie nennt denselben Inhalt noch"
        );
    }

    #[test]
    fn namen_sind_je_ordner_eindeutig_ohne_gross_und_klein() {
        let s = s();
        s.datei_schreiben(&datei("d1", None, "Brief.pdf", "a"), Protokoll::Merken)
            .unwrap();
        assert!(s
            .dateiname_vergeben("files", None, "brief.PDF", None)
            .unwrap());
        assert!(!s
            .dateiname_vergeben("files", None, "brief.pdf", Some("d1"))
            .unwrap());
        assert!(!s
            .dateiname_vergeben("documents", None, "Brief.pdf", None)
            .unwrap());
    }

    #[test]
    fn verschieben_in_sich_selbst_ist_erkennbar() {
        let s = s();
        s.datei_schreiben(&ordner("o1", None, "A"), Protokoll::Merken)
            .unwrap();
        s.datei_schreiben(&ordner("o2", Some("o1"), "B"), Protokoll::Merken)
            .unwrap();
        assert!(s.liegt_in("o2", "o1").unwrap());
        assert!(!s.liegt_in("o1", "o2").unwrap());
    }

    #[test]
    fn ein_text_gilt_nur_zum_abdruck_seiner_datei() {
        let s = s();
        s.datei_schreiben(&datei("d1", None, "Vertrag.pdf", "h1"), Protokoll::Still)
            .unwrap();
        s.datei_schreiben(&datei("d2", None, "Brief.pdf", "h2"), Protokoll::Still)
            .unwrap();
        let offen = |s: &Speicher| {
            s.dateien_ohne_text("files", 2)
                .unwrap()
                .into_iter()
                .map(|d| d.uuid)
                .collect::<Vec<_>>()
        };
        assert_eq!(offen(&s).len(), 2);

        // Zu einem anderen Abdruck: gilt nicht.
        assert!(!s.dateitext_setzen("d1", "alt", "ok", Some("x"), 2).unwrap());
        assert!(s
            .dateitext_setzen("d1", "h1", "ok", Some("Kuendigungsfrist"), 2)
            .unwrap());
        assert!(s.dateitext_setzen("d2", "h2", "keinText", None, 2).unwrap());
        assert!(offen(&s).is_empty());
        assert_eq!(s.dateitexte_der_zone("files").unwrap().len(), 1);

        // Die Datei aendert sich: wieder offen, der alte Text zaehlt nicht mehr.
        s.datei_schreiben(&datei("d1", None, "Vertrag.pdf", "h1b"), Protokoll::Still)
            .unwrap();
        assert_eq!(offen(&s), ["d1"]);
        assert!(s.dateitexte_der_zone("files").unwrap().is_empty());

        // Was ein aelterer Leser nicht lesen konnte, versucht ein neuerer.
        assert!(s
            .dateitext_setzen("d1", "h1b", "unlesbar", None, 1)
            .unwrap());
        assert!(s.dateien_ohne_text("files", 1).unwrap().is_empty());
        assert_eq!(s.dateien_ohne_text("files", 2).unwrap().len(), 1);
    }
}
