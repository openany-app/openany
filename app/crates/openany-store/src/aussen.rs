//! Inhalte, die nicht im Inhaltsspeicher liegen, sondern außerhalb der App
//! -- die Übernahme ohne Kopie (docs/plan-viele-dateien.md, Entscheidung 1).
//!
//! Ein Foto aus der Medienbibliothek des Telefons wird nicht kopiert: Hier
//! bleiben Zeile und Vorschau, und dieser Eintrag sagt, wo das Original
//! liegt. Der Abgleich liest es beim Hinaustragen von dort, und wer es öffnen
//! will, bekommt es ebenfalls von dort (oder später vom Server).
//!
//! **Der Abdruck entscheidet.** Wird das Foto in der System-Galerie
//! verändert, stimmt die Quelle nicht mehr mit dem Abdruck überein -- dann
//! gilt sie als fort, nie als „derselbe Inhalt“.

use crate::{Ergebnis, Speicher};
use rusqlite::{params, OptionalExtension};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Aussen {
    pub abdruck: String,
    /// Wo die Bytes liegen (`content://media/…`).
    pub quelle: String,
    pub groesse: u64,
    /// Nur im WLAN hinaustragen.
    pub nur_wlan: bool,
}

fn aus(z: &rusqlite::Row<'_>) -> rusqlite::Result<Aussen> {
    Ok(Aussen {
        abdruck: z.get(0)?,
        quelle: z.get(1)?,
        groesse: z.get::<_, i64>(2)?.max(0) as u64,
        nur_wlan: z.get(3)?,
    })
}

impl Speicher {
    pub fn aussen_merken(&self, a: &Aussen) -> Ergebnis<()> {
        self.db().execute(
            "INSERT INTO aussen (abdruck, quelle, groesse, nur_wlan) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT (abdruck) DO UPDATE SET
                quelle = excluded.quelle, groesse = excluded.groesse, nur_wlan = excluded.nur_wlan",
            params![a.abdruck, a.quelle, a.groesse as i64, a.nur_wlan],
        )?;
        Ok(())
    }

    pub fn aussen(&self, abdruck: &str) -> Ergebnis<Option<Aussen>> {
        Ok(self
            .db()
            .query_row(
                "SELECT abdruck, quelle, groesse, nur_wlan FROM aussen WHERE abdruck = ?1",
                params![abdruck],
                aus,
            )
            .optional()?)
    }

    pub fn aussen_alle(&self) -> Ergebnis<HashMap<String, Aussen>> {
        let db = self.db();
        let mut abfrage = db.prepare("SELECT abdruck, quelle, groesse, nur_wlan FROM aussen")?;
        let liste = abfrage
            .query_map([], aus)?
            .map(|z| z.map(|a| (a.abdruck.clone(), a)))
            .collect::<Result<_, _>>()?;
        Ok(liste)
    }

    /// Die Quelle ist fort oder nicht mehr dieselbe.
    pub fn aussen_vergessen(&self, abdruck: &str) -> Ergebnis<()> {
        self.db()
            .execute("DELETE FROM aussen WHERE abdruck = ?1", params![abdruck])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merken_lesen_vergessen() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        let a = Aussen {
            abdruck: "ab".repeat(32),
            quelle: "content://media/external/images/media/7".into(),
            groesse: 1234,
            nur_wlan: true,
        };
        s.aussen_merken(&a).unwrap();
        assert_eq!(s.aussen(&a.abdruck).unwrap(), Some(a.clone()));
        assert_eq!(s.aussen_alle().unwrap().len(), 1);
        s.aussen_vergessen(&a.abdruck).unwrap();
        assert_eq!(s.aussen(&a.abdruck).unwrap(), None);
    }
}
