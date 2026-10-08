//! Die Galerie -- Alben und Bilder, das Gegenstueck zu openanys `albums` und
//! `media`.
//!
//! Wie bei den Dateien steht hier nur, WAS es gibt; die Bytes liegen nach
//! Abdruck in der [`crate::Inhalte`]-Ablage, die Vorschaubilder daneben.
//!
//! **Bilder ohne Album** liegen auf der obersten Ebene der Galerie
//! (`album` NULL), wie drueben die Wurzel-Bilder.
//!
//! **Papierkorb mit Teilbaum**, wie drueben (`trashed_via_parent`): Ein
//! Album nimmt Unteralben und Bilder mit, erkannt am selben Zeitpunkt.

use crate::{Ergebnis, Protokoll, Speicher};
use chrono::Utc;
use openany_client::{Art, Was};
use rusqlite::{params, OptionalExtension, Row};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Album {
    pub uuid: String,
    pub name: String,
    pub beschreibung: String,
    pub eltern: Option<String>,
    pub papierkorb_at: Option<String>,
    pub geaendert_at: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Bild {
    pub uuid: String,
    /// `None` = oberste Ebene der Galerie.
    pub album: Option<String>,
    pub name: String,
    pub mime: String,
    pub groesse: u64,
    pub abdruck: Option<String>,
    /// Abdruck des Vorschaubilds (JPEG, lange Seite 480 px). Reist immer
    /// mit -- auch ein Geraet "bei Bedarf" soll die Galerie sehen.
    pub vorschau: Option<String>,
    /// EXIF wie drueben in `custom_properties.exif`: `{date, gps: {lat, lng}}`
    /// als JSON-Text, leer wenn nichts gelesen wurde.
    pub exif: String,
    pub papierkorb_at: Option<String>,
    pub geaendert_at: String,
}

const ALBUM_SPALTEN: &str = "uuid, name, beschreibung, eltern, papierkorb_at, geaendert_at";
const BILD_SPALTEN: &str =
    "uuid, album, name, mime, groesse, abdruck, vorschau, exif, papierkorb_at, geaendert_at";

fn album_aus(z: &Row<'_>) -> rusqlite::Result<Album> {
    Ok(Album {
        uuid: z.get(0)?,
        name: z.get(1)?,
        beschreibung: z.get(2)?,
        eltern: z.get(3)?,
        papierkorb_at: z.get(4)?,
        geaendert_at: z.get(5)?,
    })
}

fn bild_aus(z: &Row<'_>) -> rusqlite::Result<Bild> {
    Ok(Bild {
        uuid: z.get(0)?,
        album: z.get(1)?,
        name: z.get(2)?,
        mime: z.get(3)?,
        groesse: z.get::<_, i64>(4)?.max(0) as u64,
        abdruck: z.get(5)?,
        vorschau: z.get(6)?,
        exif: z.get(7)?,
        papierkorb_at: z.get(8)?,
        geaendert_at: z.get(9)?,
    })
}

fn jetzt_wenn_leer(wert: &str) -> String {
    if wert.is_empty() {
        Utc::now().to_rfc3339()
    } else {
        wert.to_string()
    }
}

impl Speicher {
    fn protokoll_art(&self, protokoll: Protokoll, art: Art, key: &str, was: Was) -> Ergebnis<()> {
        match protokoll {
            Protokoll::Merken => self.merken(&art, key, was),
            Protokoll::Still => Ok(()),
            Protokoll::Von(h) => self.merken_von(&art, key, was, Some(h)),
        }
    }

    // --- Alben -----------------------------------------------------------

    pub fn album_schreiben(&self, a: &Album, protokoll: Protokoll) -> Ergebnis<()> {
        self.db().execute(
            &format!(
                "INSERT INTO alben ({ALBUM_SPALTEN}) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT (uuid) DO UPDATE SET
                    name = excluded.name, beschreibung = excluded.beschreibung,
                    eltern = excluded.eltern, papierkorb_at = excluded.papierkorb_at,
                    geaendert_at = excluded.geaendert_at"
            ),
            params![
                a.uuid,
                a.name,
                a.beschreibung,
                a.eltern,
                a.papierkorb_at,
                jetzt_wenn_leer(&a.geaendert_at)
            ],
        )?;
        self.protokoll_art(protokoll, Art::Album, &a.uuid, Was::Da)
    }

    pub fn album(&self, uuid: &str) -> Ergebnis<Option<Album>> {
        Ok(self
            .db()
            .query_row(
                &format!("SELECT {ALBUM_SPALTEN} FROM alben WHERE uuid = ?1"),
                params![uuid],
                album_aus,
            )
            .optional()?)
    }

    /// Lebendige Alben unter einem Album (`None` = oberste Ebene), nach Namen.
    pub fn alben_unter(&self, eltern: Option<&str>) -> Ergebnis<Vec<Album>> {
        let db = self.db();
        let mut abfrage = db.prepare(&format!(
            "SELECT {ALBUM_SPALTEN} FROM alben
             WHERE eltern IS ?1 AND papierkorb_at IS NULL
             ORDER BY name COLLATE NOCASE"
        ))?;
        let liste = abfrage
            .query_map(params![eltern], album_aus)?
            .collect::<Result<_, _>>()?;
        Ok(liste)
    }

    pub fn alle_alben(&self) -> Ergebnis<Vec<Album>> {
        let db = self.db();
        let mut abfrage = db.prepare(&format!(
            "SELECT {ALBUM_SPALTEN} FROM alben WHERE papierkorb_at IS NULL ORDER BY name COLLATE NOCASE"
        ))?;
        let liste = abfrage
            .query_map([], album_aus)?
            .collect::<Result<_, _>>()?;
        Ok(liste)
    }

    /// Der Weg von oben bis zu diesem Album, ohne es selbst.
    pub fn albumweg(&self, uuid: &str) -> Ergebnis<Vec<Album>> {
        let mut weg: Vec<Album> = Vec::new();
        let mut naechster = self.album(uuid)?.and_then(|a| a.eltern);
        while let Some(u) = naechster {
            if weg.iter().any(|a| a.uuid == u) || weg.len() > 64 {
                break;
            }
            let Some(a) = self.album(&u)? else { break };
            naechster = a.eltern.clone();
            weg.push(a);
        }
        weg.reverse();
        Ok(weg)
    }

    fn unteralben(&self, uuid: &str) -> Ergebnis<Vec<Album>> {
        let db = self.db();
        let mut abfrage = db.prepare(&format!(
            "WITH RECURSIVE baum(u) AS (
                SELECT uuid FROM alben WHERE eltern = ?1
                UNION SELECT a.uuid FROM alben a JOIN baum ON a.eltern = baum.u
             )
             SELECT {ALBUM_SPALTEN} FROM alben WHERE uuid IN (SELECT u FROM baum)"
        ))?;
        let liste = abfrage
            .query_map(params![uuid], album_aus)?
            .collect::<Result<_, _>>()?;
        Ok(liste)
    }

    /// Die uuids eines Albums und seiner Unteralben.
    pub fn album_mit_unteralben(&self, uuid: &str) -> Ergebnis<Vec<String>> {
        let mut alle = vec![uuid.to_string()];
        alle.extend(self.unteralben(uuid)?.into_iter().map(|a| a.uuid));
        Ok(alle)
    }

    pub fn album_liegt_in(&self, ziel: &str, album: &str) -> Ergebnis<bool> {
        Ok(ziel == album || self.unteralben(album)?.iter().any(|a| a.uuid == ziel))
    }

    /// Ein Album mit Unteralben und allen Bildern darin in den Papierkorb.
    pub fn album_papierkorb(&self, uuid: &str, protokoll: Protokoll) -> Ergebnis<bool> {
        let jetzt = Utc::now().to_rfc3339();
        let mut alben = vec![uuid.to_string()];
        alben.extend(
            self.unteralben(uuid)?
                .into_iter()
                .filter(|a| a.papierkorb_at.is_none())
                .map(|a| a.uuid),
        );
        let mut getan = false;
        for a in &alben {
            let n = self.db().execute(
                "UPDATE alben SET papierkorb_at = ?1, geaendert_at = ?1
                 WHERE uuid = ?2 AND papierkorb_at IS NULL",
                params![jetzt, a],
            )?;
            if n > 0 {
                getan = true;
                self.protokoll_art(protokoll, Art::Album, a, Was::Papierkorb)?;
            }
            for b in self.bilder_im_album(Some(a))? {
                self.bild_papierkorb_um(&b.uuid, &jetzt, protokoll)?;
            }
        }
        Ok(getan)
    }

    /// Zurueck, mit allem, was im selben Moment mitging, und den Alben
    /// darueber, falls die auch im Papierkorb liegen.
    pub fn album_wiederherstellen(&self, uuid: &str, protokoll: Protokoll) -> Ergebnis<bool> {
        let Some(a) = self.album(uuid)? else {
            return Ok(false);
        };
        let Some(seit) = a.papierkorb_at.clone() else {
            return Ok(false);
        };
        let mut alben = vec![a.uuid.clone()];
        alben.extend(
            self.unteralben(uuid)?
                .into_iter()
                .filter(|u| u.papierkorb_at.as_deref() == Some(seit.as_str()))
                .map(|u| u.uuid),
        );
        for oben in self.albumweg(uuid)? {
            if oben.papierkorb_at.is_some() {
                alben.push(oben.uuid);
            }
        }
        let jetzt = Utc::now().to_rfc3339();
        for u in &alben {
            let n = self.db().execute(
                "UPDATE alben SET papierkorb_at = NULL, geaendert_at = ?1
                 WHERE uuid = ?2 AND papierkorb_at IS NOT NULL",
                params![jetzt, u],
            )?;
            if n > 0 {
                self.protokoll_art(protokoll, Art::Album, u, Was::Da)?;
            }
            let bilder: Vec<Bild> = self.bilder_alle_im_album(u)?;
            for b in bilder
                .into_iter()
                .filter(|b| b.papierkorb_at.as_deref() == Some(seit.as_str()))
            {
                self.db().execute(
                    "UPDATE medien SET papierkorb_at = NULL, geaendert_at = ?1 WHERE uuid = ?2",
                    params![jetzt, b.uuid],
                )?;
                self.protokoll_art(protokoll, Art::Medium, &b.uuid, Was::Da)?;
            }
        }
        Ok(true)
    }

    /// Endgueltig, mit allem darin. Gibt die frei gewordenen Abdruecke zurueck.
    pub fn album_loeschen(&self, uuid: &str, protokoll: Protokoll) -> Ergebnis<Vec<String>> {
        let mut alben = self.unteralben(uuid)?;
        if let Some(a) = self.album(uuid)? {
            alben.push(a);
        }
        let mut frei = Vec::new();
        for a in &alben {
            for b in self.bilder_alle_im_album(&a.uuid)? {
                frei.extend(self.bild_loeschen(&b.uuid, protokoll)?);
            }
            self.db()
                .execute("DELETE FROM alben WHERE uuid = ?1", params![a.uuid])?;
            self.protokoll_art(protokoll, Art::Album, &a.uuid, Was::Fort)?;
        }
        Ok(frei)
    }

    // --- Bilder ----------------------------------------------------------

    pub fn bild_schreiben(&self, b: &Bild, protokoll: Protokoll) -> Ergebnis<()> {
        self.db().execute(
            &format!(
                "INSERT INTO medien ({BILD_SPALTEN}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                 ON CONFLICT (uuid) DO UPDATE SET
                    album = excluded.album, name = excluded.name, mime = excluded.mime,
                    groesse = excluded.groesse, abdruck = excluded.abdruck,
                    vorschau = excluded.vorschau,
                    exif = excluded.exif, papierkorb_at = excluded.papierkorb_at,
                    geaendert_at = excluded.geaendert_at"
            ),
            params![
                b.uuid,
                b.album,
                b.name,
                b.mime,
                b.groesse as i64,
                b.abdruck,
                b.vorschau,
                b.exif,
                b.papierkorb_at,
                jetzt_wenn_leer(&b.geaendert_at)
            ],
        )?;
        self.protokoll_art(protokoll, Art::Medium, &b.uuid, Was::Da)
    }

    pub fn bild(&self, uuid: &str) -> Ergebnis<Option<Bild>> {
        Ok(self
            .db()
            .query_row(
                &format!("SELECT {BILD_SPALTEN} FROM medien WHERE uuid = ?1"),
                params![uuid],
                bild_aus,
            )
            .optional()?)
    }

    /// Lebendige Bilder eines Albums (`None` = oberste Ebene), neueste zuerst.
    pub fn bilder_im_album(&self, album: Option<&str>) -> Ergebnis<Vec<Bild>> {
        let db = self.db();
        let mut abfrage = db.prepare(&format!(
            "SELECT {BILD_SPALTEN} FROM medien
             WHERE album IS ?1 AND papierkorb_at IS NULL
             ORDER BY geaendert_at DESC"
        ))?;
        let liste = abfrage
            .query_map(params![album], bild_aus)?
            .collect::<Result<_, _>>()?;
        Ok(liste)
    }

    fn bilder_alle_im_album(&self, album: &str) -> Ergebnis<Vec<Bild>> {
        let db = self.db();
        let mut abfrage = db.prepare(&format!(
            "SELECT {BILD_SPALTEN} FROM medien WHERE album = ?1"
        ))?;
        let liste = abfrage
            .query_map(params![album], bild_aus)?
            .collect::<Result<_, _>>()?;
        Ok(liste)
    }

    /// Alle lebendigen Bilder mit Inhalt -- fuer "was fehlt hier noch?".
    /// Alle Bilder, zu denen es ueberhaupt Bytes geben kann -- Original ODER
    /// Vorschau.
    ///
    /// **HIER STAND `AND abdruck IS NOT NULL`, UND DAS HAT EINEN TAG GEKOSTET.**
    /// Die Bedingung stammt aus der Zeit, in der Bilder nur von GERAETEN kamen;
    /// die schicken immer einen Abdruck des Originals mit.
    ///
    /// Ein Bild vom SERVER hat keinen. Das ist Absicht und steht so im
    /// `DeltaFeed`: "Hashes nur, wo sie umsonst zu haben sind" -- fuer ein Bild
    /// laege er erst nach einem vollstaendigen Lesen ueber SFTP vor, bei
    /// tausend Bildern ein Vielfaches der eigentlichen Uebertragung. Seit dem
    /// 16.09.2026 schickt der Server aber den Abdruck der VORSCHAU mit, und die
    /// ist billig zu haben.
    ///
    /// Mit der alten Bedingung fiel jedes Server-Bild aus dieser Abfrage --
    /// also aus der Liste der fehlenden Inhalte und aus dem Wegweiser. Die
    /// Vorschauen wurden nie angefragt (im Zugriffsprotokoll: null Anfragen
    /// nach `media_vorschau`, jemals), und in der Galerie stand "Leer" ueber
    /// Alben, die Bilder hatten.
    ///
    /// Alle Aufrufer greifen auf das Feld zu, das sie brauchen (`b.abdruck?`
    /// oder `b.vorschau`) -- eine Zeile ohne Original faellt dort von selbst
    /// heraus. Was hier zu eng war, muss dort nicht noch einmal eng sein.
    pub fn lebendige_bilder(&self) -> Ergebnis<Vec<Bild>> {
        let db = self.db();
        let mut abfrage = db.prepare(&format!(
            "SELECT {BILD_SPALTEN} FROM medien
             WHERE papierkorb_at IS NULL AND (abdruck IS NOT NULL OR vorschau IS NOT NULL)"
        ))?;
        let liste = abfrage.query_map([], bild_aus)?.collect::<Result<_, _>>()?;
        Ok(liste)
    }

    /// Wie viele lebendige Bilder ein Album hat, und das neueste als Titelbild.
    pub fn album_kennzahlen(&self, album: &str) -> Ergebnis<(usize, Option<Bild>)> {
        let bilder = self.bilder_im_album(Some(album))?;
        Ok((bilder.len(), bilder.into_iter().next()))
    }

    fn bild_papierkorb_um(&self, uuid: &str, zeit: &str, protokoll: Protokoll) -> Ergebnis<bool> {
        let n = self.db().execute(
            "UPDATE medien SET papierkorb_at = ?1, geaendert_at = ?1
             WHERE uuid = ?2 AND papierkorb_at IS NULL",
            params![zeit, uuid],
        )?;
        if n > 0 {
            self.protokoll_art(protokoll, Art::Medium, uuid, Was::Papierkorb)?;
        }
        Ok(n > 0)
    }

    pub fn bild_papierkorb(&self, uuid: &str, protokoll: Protokoll) -> Ergebnis<bool> {
        self.bild_papierkorb_um(uuid, &Utc::now().to_rfc3339(), protokoll)
    }

    pub fn bild_wiederherstellen(&self, uuid: &str, protokoll: Protokoll) -> Ergebnis<bool> {
        let Some(b) = self.bild(uuid)? else {
            return Ok(false);
        };
        // Liegt sein Album im Papierkorb, kommt es mit -- sonst kaeme das
        // Bild unsichtbar zurueck.
        if let Some(a) = b.album.as_deref() {
            if self.album(a)?.is_some_and(|a| a.papierkorb_at.is_some()) {
                self.album_wiederherstellen(a, protokoll)?;
            }
        }
        let n = self.db().execute(
            "UPDATE medien SET papierkorb_at = NULL, geaendert_at = ?1
             WHERE uuid = ?2 AND papierkorb_at IS NOT NULL",
            params![Utc::now().to_rfc3339(), uuid],
        )?;
        if n > 0 {
            self.protokoll_art(protokoll, Art::Medium, uuid, Was::Da)?;
        }
        Ok(true)
    }

    pub fn bild_loeschen(&self, uuid: &str, protokoll: Protokoll) -> Ergebnis<Vec<String>> {
        let abdruecke: Vec<String> = self
            .bild(uuid)?
            .map(|b| b.abdruck.into_iter().chain(b.vorschau).collect())
            .unwrap_or_default();
        let n = self
            .db()
            .execute("DELETE FROM medien WHERE uuid = ?1", params![uuid])?;
        if n > 0 || !matches!(protokoll, Protokoll::Von(_)) {
            self.protokoll_art(protokoll, Art::Medium, uuid, Was::Fort)?;
        }
        Ok(abdruecke)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s() -> Speicher {
        Speicher::im_arbeitsspeicher().unwrap()
    }

    fn album(uuid: &str, eltern: Option<&str>) -> Album {
        Album {
            uuid: uuid.into(),
            name: uuid.into(),
            eltern: eltern.map(Into::into),
            ..Default::default()
        }
    }

    fn bild(uuid: &str, album: Option<&str>, abdruck: &str) -> Bild {
        Bild {
            uuid: uuid.into(),
            album: album.map(Into::into),
            name: format!("{uuid}.jpg"),
            mime: "image/jpeg".into(),
            groesse: 3,
            abdruck: Some(abdruck.into()),
            ..Default::default()
        }
    }

    #[test]
    fn ein_album_nimmt_unteralben_und_bilder_mit_und_bringt_sie_zurueck() {
        let s = s();
        s.album_schreiben(&album("a", None), Protokoll::Merken)
            .unwrap();
        s.album_schreiben(&album("b", Some("a")), Protokoll::Merken)
            .unwrap();
        s.bild_schreiben(&bild("x", Some("b"), "h1"), Protokoll::Merken)
            .unwrap();
        s.bild_schreiben(&bild("y", None, "h2"), Protokoll::Merken)
            .unwrap();

        assert!(s.album_papierkorb("a", Protokoll::Merken).unwrap());
        assert!(s.bild("x").unwrap().unwrap().papierkorb_at.is_some());
        assert!(
            s.bild("y").unwrap().unwrap().papierkorb_at.is_none(),
            "das Wurzelbild bleibt"
        );
        assert!(s.alben_unter(None).unwrap().is_empty());

        s.album_wiederherstellen("a", Protokoll::Merken).unwrap();
        assert!(s.bild("x").unwrap().unwrap().papierkorb_at.is_none());
        assert_eq!(s.alben_unter(Some("a")).unwrap().len(), 1);
    }

    #[test]
    fn ein_bild_bringt_sein_album_zurueck() {
        let s = s();
        s.album_schreiben(&album("a", None), Protokoll::Merken)
            .unwrap();
        s.bild_schreiben(&bild("x", Some("a"), "h1"), Protokoll::Merken)
            .unwrap();
        s.album_papierkorb("a", Protokoll::Merken).unwrap();

        s.bild_wiederherstellen("x", Protokoll::Merken).unwrap();
        assert!(s.album("a").unwrap().unwrap().papierkorb_at.is_none());
    }

    #[test]
    fn endgueltig_loeschen_nennt_die_abdruecke() {
        let s = s();
        s.album_schreiben(&album("a", None), Protokoll::Merken)
            .unwrap();
        s.album_schreiben(&album("b", Some("a")), Protokoll::Merken)
            .unwrap();
        s.bild_schreiben(&bild("x", Some("b"), "h1"), Protokoll::Merken)
            .unwrap();
        let mut frei = s.album_loeschen("a", Protokoll::Merken).unwrap();
        frei.sort();
        assert_eq!(frei, ["h1"]);
        assert!(s.album("b").unwrap().is_none());
        assert!(s.bild("x").unwrap().is_none());
    }

    #[test]
    fn ein_album_mit_seinen_unteralben() {
        let s = s();
        s.album_schreiben(&album("a", None), Protokoll::Merken)
            .unwrap();
        s.album_schreiben(&album("b", Some("a")), Protokoll::Merken)
            .unwrap();
        s.album_schreiben(&album("c", Some("b")), Protokoll::Merken)
            .unwrap();
        s.album_schreiben(&album("d", None), Protokoll::Merken)
            .unwrap();
        let mut alle = s.album_mit_unteralben("a").unwrap();
        alle.sort();
        assert_eq!(alle, ["a", "b", "c"]);
    }

    #[test]
    fn kennzahlen_und_weg() {
        let s = s();
        s.album_schreiben(&album("a", None), Protokoll::Merken)
            .unwrap();
        s.album_schreiben(&album("b", Some("a")), Protokoll::Merken)
            .unwrap();
        s.bild_schreiben(&bild("x", Some("b"), "h1"), Protokoll::Merken)
            .unwrap();
        let (anzahl, titel) = s.album_kennzahlen("b").unwrap();
        assert_eq!(anzahl, 1);
        assert_eq!(titel.unwrap().uuid, "x");
        assert_eq!(s.albumweg("b").unwrap()[0].uuid, "a");
        assert!(s.album_liegt_in("b", "a").unwrap());
    }
}

#[cfg(test)]
mod bilder_vom_server {
    use super::*;

    fn s() -> Speicher {
        Speicher::im_arbeitsspeicher().unwrap()
    }

    /// EIN BILD VOM SERVER HAT KEINEN ORIGINALABDRUCK -- nur den der Vorschau.
    ///
    /// Der `DeltaFeed` schickt fuer ein Bild bewusst keinen Inhalts-Hash: Er
    /// laege erst nach einem vollstaendigen Lesen ueber SFTP vor. Den Abdruck
    /// der VORSCHAU schickt er seit dem 16.09.2026, weil der beim Erzeugen
    /// ohnehin anfaellt.
    ///
    /// `lebendige_bilder` verlangte trotzdem einen Originalabdruck. Damit fiel
    /// jedes Server-Bild aus der Liste der fehlenden Inhalte UND aus dem
    /// Wegweiser -- die Vorschau wurde nie angefragt, und in der Galerie stand
    /// "Leer" ueber Alben, die Bilder hatten.
    #[test]
    fn ein_bild_mit_nur_einer_vorschau_faellt_nicht_heraus() {
        let speicher = s();

        speicher
            .bild_schreiben(
                &Bild {
                    uuid: "vom-server".into(),
                    name: "strand.jpg".into(),
                    mime: "image/jpeg".into(),
                    groesse: 763_296,
                    abdruck: None,
                    vorschau: Some("ae53".into()),
                    ..Default::default()
                },
                Protokoll::Von("https://openany.de"),
            )
            .unwrap();

        let bilder = speicher.lebendige_bilder().unwrap();

        assert_eq!(bilder.len(), 1, "das Bild vom Server fehlt");
        assert_eq!(bilder[0].vorschau.as_deref(), Some("ae53"));
        assert!(bilder[0].abdruck.is_none());
    }

    /// Und eines ohne beides bleibt draussen: Zu ihm gibt es nichts zu holen
    /// und nichts anzubieten -- es waere eine Zeile, die jede Liste verlaengert
    /// und in keiner etwas beitraegt.
    #[test]
    fn ein_bild_ohne_alles_bleibt_draussen() {
        let speicher = s();

        speicher
            .bild_schreiben(
                &Bild {
                    uuid: "leer".into(),
                    name: "leer.jpg".into(),
                    ..Default::default()
                },
                Protokoll::Von("https://openany.de"),
            )
            .unwrap();

        assert!(speicher.lebendige_bilder().unwrap().is_empty());
    }
}
