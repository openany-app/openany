//! Wie ein fremdes Programm eine Sache nennt -- für den CalDAV-/CardDAV-
//! Dienst am Desktop (docs/plan-kalender-kontakte-im-system.md, K5).
//!
//! Legt Thunderbird oder Apple Kalender einen Termin an, wählt es selbst
//! den Namen der Ressource (`…/<name>.ics`) und die `UID` darin. openany
//! vergibt seine eigene uuid; damit das Programm den Termin danach unter
//! seinem Namen wiederfindet, steht hier, wie er heißt. Sachen ohne
//! Eintrag heißen wie ihre uuid.

use crate::{Ergebnis, Speicher};
use rusqlite::{params, OptionalExtension};
use std::collections::HashMap;

/// Name der Ressource und `UID`, wie das fremde Programm sie gewählt hat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Davname {
    pub name: String,
    pub uid: String,
}

impl Speicher {
    pub fn davname_merken(&self, art: &str, uuid: &str, n: &Davname) -> Ergebnis<()> {
        self.db().execute(
            "INSERT INTO davnamen (art, uuid, name, uid) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT (art, uuid) DO UPDATE SET name = excluded.name, uid = excluded.uid",
            params![art, uuid, n.name, n.uid],
        )?;
        Ok(())
    }

    /// uuid → Name, für eine ganze Art.
    pub fn davnamen(&self, art: &str) -> Ergebnis<HashMap<String, Davname>> {
        let db = self.db();
        let mut abfrage = db.prepare("SELECT uuid, name, uid FROM davnamen WHERE art = ?1")?;
        let liste = abfrage
            .query_map(params![art], |z| {
                Ok((
                    z.get::<_, String>(0)?,
                    Davname {
                        name: z.get(1)?,
                        uid: z.get(2)?,
                    },
                ))
            })?
            .collect::<Result<_, _>>()?;
        Ok(liste)
    }

    /// Die uuid hinter einem Namen.
    pub fn davname_zu_uuid(&self, art: &str, name: &str) -> Ergebnis<Option<String>> {
        Ok(self
            .db()
            .query_row(
                "SELECT uuid FROM davnamen WHERE art = ?1 AND name = ?2",
                params![art, name],
                |z| z.get(0),
            )
            .optional()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ein_fremder_name_wird_wiedergefunden() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        let n = Davname {
            name: "abc-123".into(),
            uid: "abc-123@thunderbird".into(),
        };
        s.davname_merken("termin", "u-1", &n).unwrap();
        assert_eq!(
            s.davname_zu_uuid("termin", "abc-123").unwrap().as_deref(),
            Some("u-1")
        );
        assert_eq!(s.davnamen("termin").unwrap()["u-1"], n);
        assert_eq!(s.davname_zu_uuid("kontakt", "abc-123").unwrap(), None);
    }
}
