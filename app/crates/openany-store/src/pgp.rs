//! Öffentliche OpenPGP-Schlüssel der Gegenüber (Schema-Stand 17,
//! docs/plan-email-pgp.md, Schritt 3a). Nur auf diesem Gerät.
//!
//! Was ein Schlüssel ist, weiß die App (`openany-post`); hier liegt er nur,
//! ASCII-armored, mit Fingerabdruck und woher er kam.

use crate::{Ergebnis, Speicher};
use rusqlite::{params, OptionalExtension, Row};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PgpSchluessel {
    /// Klein geschrieben.
    pub adresse: String,
    pub fingerabdruck: String,
    pub oeffentlich: String,
    /// `autocrypt`, `wkd`, `keys.openpgp.org` oder `hand`.
    pub quelle: String,
    pub zuerst_at: String,
    pub aktualisiert_at: String,
    /// Der Fingerabdruck davor, wenn sich der Schlüssel geändert hat.
    pub vorher: Option<String>,
    pub geaendert_at: Option<String>,
}

const SPALTEN: &str =
    "adresse, fingerabdruck, oeffentlich, quelle, zuerst_at, aktualisiert_at, vorher, geaendert_at";

fn aus(z: &Row<'_>) -> rusqlite::Result<PgpSchluessel> {
    Ok(PgpSchluessel {
        adresse: z.get(0)?,
        fingerabdruck: z.get(1)?,
        oeffentlich: z.get(2)?,
        quelle: z.get(3)?,
        zuerst_at: z.get(4)?,
        aktualisiert_at: z.get(5)?,
        vorher: z.get(6)?,
        geaendert_at: z.get(7)?,
    })
}

/// Was beim Ablegen geschah.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Abgelegt {
    Neu,
    Bekannt,
    /// Ein anderer Schlüssel als bisher -- die Oberfläche soll es sagen.
    Geaendert,
}

impl Speicher {
    /// Einen Schlüssel für `adresse` ablegen.
    pub fn pgp_ablegen(
        &self,
        adresse: &str,
        fingerabdruck: &str,
        oeffentlich: &str,
        quelle: &str,
    ) -> Ergebnis<Abgelegt> {
        let adresse = adresse.trim().to_lowercase();
        let jetzt = chrono::Utc::now().to_rfc3339();
        let db = self.db();
        let alt: Option<String> = db
            .query_row(
                "SELECT fingerabdruck FROM pgp_schluessel WHERE adresse = ?1",
                params![adresse],
                |z| z.get(0),
            )
            .optional()?;
        Ok(match alt {
            None => {
                db.execute(
                    &format!(
                        "INSERT INTO pgp_schluessel ({SPALTEN})
                         VALUES (?1, ?2, ?3, ?4, ?5, ?5, NULL, NULL)"
                    ),
                    params![adresse, fingerabdruck, oeffentlich, quelle, jetzt],
                )?;
                Abgelegt::Neu
            }
            Some(f) if f == fingerabdruck => {
                // Derselbe Schlüssel, vielleicht mit neuen Unterschlüsseln
                // oder verlängerter Laufzeit: den neuesten Stand behalten.
                db.execute(
                    "UPDATE pgp_schluessel SET oeffentlich = ?2, aktualisiert_at = ?3
                     WHERE adresse = ?1",
                    params![adresse, oeffentlich, jetzt],
                )?;
                Abgelegt::Bekannt
            }
            Some(f) => {
                db.execute(
                    "UPDATE pgp_schluessel SET fingerabdruck = ?2, oeffentlich = ?3, quelle = ?4,
                        aktualisiert_at = ?5, vorher = ?6, geaendert_at = ?5
                     WHERE adresse = ?1",
                    params![adresse, fingerabdruck, oeffentlich, quelle, jetzt, f],
                )?;
                Abgelegt::Geaendert
            }
        })
    }

    pub fn pgp_schluessel(&self, adresse: &str) -> Ergebnis<Option<PgpSchluessel>> {
        Ok(self
            .db()
            .query_row(
                &format!("SELECT {SPALTEN} FROM pgp_schluessel WHERE adresse = ?1"),
                params![adresse.trim().to_lowercase()],
                aus,
            )
            .optional()?)
    }

    pub fn pgp_liste(&self) -> Ergebnis<Vec<PgpSchluessel>> {
        let db = self.db();
        let mut a = db.prepare(&format!(
            "SELECT {SPALTEN} FROM pgp_schluessel ORDER BY adresse"
        ))?;
        let liste = a.query_map([], aus)?.collect::<Result<_, _>>()?;
        Ok(liste)
    }

    pub fn pgp_loeschen(&self, adresse: &str) -> Ergebnis<bool> {
        Ok(self.db().execute(
            "DELETE FROM pgp_schluessel WHERE adresse = ?1",
            params![adresse.trim().to_lowercase()],
        )? > 0)
    }

    /// Den Hinweis auf einen Wechsel wegnehmen, wenn der Mensch ihn gesehen
    /// und den neuen Fingerabdruck verglichen hat.
    pub fn pgp_wechsel_gesehen(&self, adresse: &str) -> Ergebnis<()> {
        self.db().execute(
            "UPDATE pgp_schluessel SET vorher = NULL, geaendert_at = NULL WHERE adresse = ?1",
            params![adresse.trim().to_lowercase()],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neu_bekannt_geaendert() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        assert_eq!(
            s.pgp_ablegen("F@B.test", "AAA", "k1", "autocrypt").unwrap(),
            Abgelegt::Neu
        );
        assert_eq!(
            s.pgp_ablegen("f@b.test", "AAA", "k1b", "wkd").unwrap(),
            Abgelegt::Bekannt
        );
        let k = s.pgp_schluessel("f@b.test").unwrap().unwrap();
        assert_eq!(
            (k.oeffentlich.as_str(), k.quelle.as_str()),
            ("k1b", "autocrypt")
        );

        assert_eq!(
            s.pgp_ablegen("f@b.test", "BBB", "k2", "hand").unwrap(),
            Abgelegt::Geaendert
        );
        let k = s.pgp_schluessel("f@b.test").unwrap().unwrap();
        assert_eq!(k.vorher.as_deref(), Some("AAA"));
        s.pgp_wechsel_gesehen("f@b.test").unwrap();
        assert!(s
            .pgp_schluessel("f@b.test")
            .unwrap()
            .unwrap()
            .vorher
            .is_none());

        assert!(s.pgp_loeschen("f@b.test").unwrap());
        assert!(s.pgp_liste().unwrap().is_empty());
    }
}
