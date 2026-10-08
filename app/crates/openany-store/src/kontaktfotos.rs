//! Die Fotos der Kontakte -- getrennt vom Kontakt, weil sie gross sind.
//!
//! **Der Abdruck steht am Kontakt, die Bytes stehen hier.** Eine Liste von
//! dreihundert Kontakten soll nicht dreihundert Bilder mitlesen, und der
//! Abgleich vergleicht ohnehin nur den Abdruck: Stimmt er, wird nichts geholt.
//!
//! **Wer hat es geaendert?** `hinaus` merkt sich, dass das Foto HIER gesetzt
//! oder entfernt wurde und beim naechsten Abgleich hinueber muss. Ein Foto,
//! das der Abgleich geholt hat, traegt 0 -- sonst schickte der naechste Lauf
//! das Bild, das eben erst hereinkam, gleich wieder zurueck.

use crate::{Ergebnis, Protokoll, Speicher};
use rusqlite::{params, OptionalExtension};
use sha2::{Digest, Sha256};

/// Was beim Hinausschicken mit dem Foto zu tun ist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FotoHinaus {
    Setzen(Vec<u8>),
    Entfernen,
}

/// sha256 ueber die Bytes, klein und hexadezimal.
///
/// Oeffentlich, weil ein anderes Geraet ein Foto als Bytes schickt und hier
/// derselbe Abdruck entstehen muss, den es selbst dafuer fuehrt -- sonst
/// holte jeder Lauf das Bild neu.
pub fn fotoabdruck(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

impl Speicher {
    /// Die Bytes des Fotos, falls es hier welche gibt.
    pub fn kontaktfoto(&self, uuid: &str) -> Ergebnis<Option<Vec<u8>>> {
        Ok(self
            .db()
            .query_row(
                "SELECT bytes FROM kontaktfotos WHERE kontakt_uuid = ?1 AND bytes IS NOT NULL",
                params![uuid],
                |z| z.get(0),
            )
            .optional()?)
    }

    /// Ein Foto HIER setzen: Bytes ablegen, Abdruck an den Kontakt, fuer den
    /// naechsten Abgleich vormerken.
    pub fn kontaktfoto_setzen(&self, uuid: &str, bytes: &[u8]) -> Ergebnis<()> {
        let abdruck = fotoabdruck(bytes);
        self.db().execute(
            "INSERT INTO kontaktfotos (kontakt_uuid, abdruck, bytes, hinaus) VALUES (?1, ?2, ?3, 1)
             ON CONFLICT (kontakt_uuid) DO UPDATE SET abdruck = ?2, bytes = ?3, hinaus = 1",
            params![uuid, abdruck, bytes],
        )?;
        self.foto_am_kontakt(uuid, Some(abdruck), Protokoll::Merken)
    }

    /// Ein Foto HIER entfernen.
    pub fn kontaktfoto_entfernen(&self, uuid: &str) -> Ergebnis<()> {
        self.db().execute(
            "INSERT INTO kontaktfotos (kontakt_uuid, abdruck, bytes, hinaus) VALUES (?1, NULL, NULL, 1)
             ON CONFLICT (kontakt_uuid) DO UPDATE SET abdruck = NULL, bytes = NULL, hinaus = 1",
            params![uuid],
        )?;
        self.foto_am_kontakt(uuid, None, Protokoll::Merken)
    }

    /// Ein Foto, das der Abgleich geholt hat -- mit dem Abdruck von DRUEBEN,
    /// nicht einem hier gerechneten: Der Server verkleinert und speichert
    /// neu, sein Abdruck ist der, den er beim naechsten Mal wieder nennt.
    pub fn kontaktfoto_uebernehmen(
        &self,
        uuid: &str,
        abdruck: Option<&str>,
        bytes: Option<&[u8]>,
        protokoll: Protokoll,
    ) -> Ergebnis<()> {
        match (abdruck, bytes) {
            (Some(a), Some(b)) => {
                self.db().execute(
                    "INSERT INTO kontaktfotos (kontakt_uuid, abdruck, bytes, hinaus) VALUES (?1, ?2, ?3, 0)
                     ON CONFLICT (kontakt_uuid) DO UPDATE SET abdruck = ?2, bytes = ?3, hinaus = 0",
                    params![uuid, a, b],
                )?;
            }
            _ => {
                self.db().execute(
                    "DELETE FROM kontaktfotos WHERE kontakt_uuid = ?1",
                    params![uuid],
                )?;
            }
        }
        self.foto_am_kontakt(uuid, abdruck.map(str::to_string), protokoll)
    }

    /// Muss das Foto hinueber?
    pub fn kontaktfoto_hinaus(&self, uuid: &str) -> Ergebnis<Option<FotoHinaus>> {
        let zeile: Option<(Option<Vec<u8>>, i64)> = self
            .db()
            .query_row(
                "SELECT bytes, hinaus FROM kontaktfotos WHERE kontakt_uuid = ?1",
                params![uuid],
                |z| Ok((z.get(0)?, z.get(1)?)),
            )
            .optional()?;

        Ok(match zeile {
            Some((Some(b), 1)) => Some(FotoHinaus::Setzen(b)),
            Some((None, 1)) => Some(FotoHinaus::Entfernen),
            _ => None,
        })
    }

    /// Hinuebergeschickt: nicht noch einmal.
    pub fn kontaktfoto_hinaus_erledigt(&self, uuid: &str) -> Ergebnis<()> {
        self.db().execute(
            "UPDATE kontaktfotos SET hinaus = 0 WHERE kontakt_uuid = ?1",
            params![uuid],
        )?;
        self.db().execute(
            "DELETE FROM kontaktfotos WHERE kontakt_uuid = ?1 AND bytes IS NULL",
            params![uuid],
        )?;
        Ok(())
    }

    fn foto_am_kontakt(
        &self,
        uuid: &str,
        abdruck: Option<String>,
        protokoll: Protokoll,
    ) -> Ergebnis<()> {
        let Some(mut k) = self.kontakt(uuid)? else {
            return Ok(());
        };
        k.foto = abdruck;
        if protokoll == Protokoll::Merken {
            k.geaendert_at = String::new();
        }
        self.kontakt_schreiben(&k, protokoll)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Kontakt;

    fn mit_kontakt() -> Speicher {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        s.kontakt_schreiben(
            &Kontakt {
                uuid: "c".into(),
                anzeigename: "H".into(),
                ..Default::default()
            },
            Protokoll::Still,
        )
        .unwrap();
        s
    }

    #[test]
    fn hier_gesetzt_muss_hinaus_und_steht_am_kontakt() {
        let s = mit_kontakt();
        s.kontaktfoto_setzen("c", b"bild").unwrap();

        assert_eq!(s.kontaktfoto("c").unwrap().as_deref(), Some(&b"bild"[..]));
        assert_eq!(
            s.kontaktfoto_hinaus("c").unwrap(),
            Some(FotoHinaus::Setzen(b"bild".to_vec()))
        );
        assert_eq!(
            s.kontakt("c")
                .unwrap()
                .unwrap()
                .foto
                .as_deref()
                .map(str::len),
            Some(64)
        );

        s.kontaktfoto_hinaus_erledigt("c").unwrap();
        assert_eq!(s.kontaktfoto_hinaus("c").unwrap(), None);
        assert!(s.kontaktfoto("c").unwrap().is_some());
    }

    #[test]
    fn geholt_muss_nicht_zurueck() {
        let s = mit_kontakt();
        s.kontaktfoto_uebernehmen("c", Some("abc"), Some(b"bild"), Protokoll::Still)
            .unwrap();

        assert_eq!(s.kontaktfoto_hinaus("c").unwrap(), None);
        assert_eq!(
            s.kontakt("c").unwrap().unwrap().foto.as_deref(),
            Some("abc")
        );
    }

    #[test]
    fn hier_entfernt_geht_als_entfernen_hinaus() {
        let s = mit_kontakt();
        s.kontaktfoto_uebernehmen("c", Some("abc"), Some(b"bild"), Protokoll::Still)
            .unwrap();
        s.kontaktfoto_entfernen("c").unwrap();

        assert_eq!(
            s.kontaktfoto_hinaus("c").unwrap(),
            Some(FotoHinaus::Entfernen)
        );
        assert!(s.kontaktfoto("c").unwrap().is_none());
        assert_eq!(s.kontakt("c").unwrap().unwrap().foto, None);
    }
}
