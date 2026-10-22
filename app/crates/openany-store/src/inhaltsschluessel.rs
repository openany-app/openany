//! Die Schluessel verschluesselter Inhalte (docs/plan-dateien-e2e.md, 7b).
//!
//! **Nach dem Klartext-Abdruck, weil Inhalte hier so heissen.** Die Ablage
//! ([`crate::Inhalte`]) fuehrt Bytes unter ihrem sha256; derselbe Inhalt
//! liegt genau einmal da -- und hat damit auch genau einen Schluessel. Der
//! Server dagegen kennt nur das Chiffrat; sein Abdruck steht daneben.
//!
//! Was hier fehlt, steht im Kopf der Datei: Ein Geraet, das die Datei nicht
//! selbst verschluesselt hat, traegt die Zeile beim Holen nach.

use crate::{Ergebnis, Speicher};
use rusqlite::{params, OptionalExtension};

/// Was zum Holen und Oeffnen eines Inhalts beim Server gehoert.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inhaltsschluessel {
    /// sha256 des Klartexts.
    pub abdruck: String,
    /// age-Identitaet nur fuer diesen Inhalt.
    pub schluessel: String,
    /// sha256 des Chiffrats -- so heisst er beim Server.
    pub chiffrat: String,
    /// Groesse des Chiffrats.
    pub groesse: u64,
}

impl Speicher {
    pub fn inhaltsschluessel_merken(&self, s: &Inhaltsschluessel) -> Ergebnis<()> {
        self.db().execute(
            "INSERT INTO inhaltsschluessel (abdruck, schluessel, chiffrat, groesse) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT (abdruck) DO UPDATE SET schluessel = excluded.schluessel,
                chiffrat = excluded.chiffrat, groesse = excluded.groesse",
            params![s.abdruck, s.schluessel, s.chiffrat, s.groesse as i64],
        )?;
        Ok(())
    }

    /// Alle -- fuer die Gegenstelle, die sie VOR dem Lauf braucht.
    pub fn alle_inhaltsschluessel(&self) -> Ergebnis<Vec<Inhaltsschluessel>> {
        let db = self.db();
        let mut a =
            db.prepare("SELECT abdruck, schluessel, chiffrat, groesse FROM inhaltsschluessel")?;
        let liste = a
            .query_map([], |z| {
                Ok(Inhaltsschluessel {
                    abdruck: z.get(0)?,
                    schluessel: z.get(1)?,
                    chiffrat: z.get(2)?,
                    groesse: z.get::<_, i64>(3)? as u64,
                })
            })?
            .collect::<Result<_, _>>()?;
        Ok(liste)
    }

    pub fn inhaltsschluessel(&self, abdruck: &str) -> Ergebnis<Option<Inhaltsschluessel>> {
        Ok(self
            .db()
            .query_row(
                "SELECT abdruck, schluessel, chiffrat, groesse FROM inhaltsschluessel WHERE abdruck = ?1",
                params![abdruck],
                |z| {
                    Ok(Inhaltsschluessel {
                        abdruck: z.get(0)?,
                        schluessel: z.get(1)?,
                        chiffrat: z.get(2)?,
                        groesse: z.get::<_, i64>(3)? as u64,
                    })
                },
            )
            .optional()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merken_und_wiederfinden() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        let k = Inhaltsschluessel {
            abdruck: "a".repeat(64),
            schluessel: "AGE-SECRET-KEY-1X".into(),
            chiffrat: "c".repeat(64),
            groesse: 42,
        };
        s.inhaltsschluessel_merken(&k).unwrap();
        assert_eq!(s.inhaltsschluessel(&k.abdruck).unwrap(), Some(k.clone()));
        assert_eq!(s.inhaltsschluessel("fehlt").unwrap(), None);
    }
}
