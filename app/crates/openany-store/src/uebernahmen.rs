//! Viele Dateien auf einmal übernehmen -- die Warteschlange
//! (docs/plan-viele-dateien.md, D2).
//!
//! **Die erste Stufe, nicht die zweite.** Eine Übernahme bringt eine Datei in
//! dieses Gerät (Inhaltsspeicher, Zeile in Dateien oder Mediathek). Zum Server
//! trägt sie danach der Abgleich wie jede andere Änderung -- den gibt es schon,
//! und er setzt nach einem Abbruch fort.
//!
//! **Was die Tabelle trägt, übersteht einen Neustart.** Ein Lauf arbeitet die
//! Zeilen mit `stand = 'wartet'` der Reihe nach ab; wer die App mitten drin
//! schließt, findet beim nächsten Start dieselben Zeilen wieder.
//!
//! **Doppelte überspringen, billig zuerst.** `quelle_schluessel` erkennt eine
//! Quelle wieder, ohne sie zu lesen. Was in `uebernommen` steht, kommt nicht
//! noch einmal in die Schlange -- auch wenn die entstandene Sache inzwischen
//! gelöscht ist (gelöscht bleibt gelöscht, Entscheidung 9). Den teureren
//! Vergleich über den Inhalt macht der Lauf (gleicher Abdruck im selben Ziel).

use crate::{Ergebnis, Speicher};
use rusqlite::{params, OptionalExtension};

/// Eine Datei, die übernommen werden soll.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NeueUebernahme {
    /// Wo die Bytes liegen: ein Pfad auf diesem Computer oder (Android) ein
    /// Foto der Medienbibliothek.
    pub quelle: String,
    /// Woran die Quelle wiederzuerkennen ist, ohne sie zu lesen.
    pub quelle_schluessel: String,
    /// `files`, `documents` oder `galerie`.
    pub zone: String,
    /// Ordner bzw. Album (uuid); `None` = oberste Ebene.
    pub ziel: Option<String>,
    pub name: String,
    pub mime: String,
    pub groesse: u64,
    /// Ohne Kopie (Entscheidung 1): Hier bleibt nur die Vorschau, das
    /// Original liest der Abgleich beim Hinaustragen aus der Quelle
    /// ([`crate::Aussen`]).
    pub ohne_kopie: bool,
    /// Ohne Kopie: nur im WLAN hinaustragen.
    pub nur_wlan: bool,
}

/// Eine Zeile der Warteschlange.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Uebernahme {
    pub id: i64,
    pub quelle: String,
    pub quelle_schluessel: String,
    pub zone: String,
    pub ziel: Option<String>,
    pub name: String,
    pub mime: String,
    pub groesse: u64,
    pub ohne_kopie: bool,
    pub nur_wlan: bool,
}

/// Wie weit die Übernahme ist -- für die Zeile „237 von 1.204“.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct Uebernahmestand {
    pub wartet: u64,
    pub fertig: u64,
    pub uebersprungen: u64,
    pub fehler: u64,
    /// Noch zu übernehmende Bytes.
    pub bytes_offen: u64,
}

impl Uebernahmestand {
    pub fn gesamt(&self) -> u64 {
        self.wartet + self.fertig + self.uebersprungen + self.fehler
    }
}

/// Eine Datei, die nicht übernommen wurde, samt Grund.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct NichtUebernommen {
    pub name: String,
    pub quelle: String,
    pub grund: String,
}

/// Was beim Einreihen geschah.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct Eingereiht {
    pub neu: u64,
    /// Schon einmal übernommen oder schon in der Schlange.
    pub bekannt: u64,
}

fn jetzt() -> String {
    chrono::Utc::now().to_rfc3339()
}

impl Speicher {
    /// Dateien in die Schlange stellen. Was schon einmal übernommen wurde oder
    /// gerade wartet, kommt nicht doppelt hinein.
    pub fn uebernahmen_einreihen(&self, neue: &[NeueUebernahme]) -> Ergebnis<Eingereiht> {
        let mut db = self.db();
        let tx = db.transaction()?;
        let mut aus = Eingereiht::default();
        {
            let mut bekannt = tx.prepare(
                "SELECT 1 FROM uebernommen WHERE quelle_schluessel = ?1
                 UNION ALL
                 SELECT 1 FROM uebernahmen WHERE quelle_schluessel = ?1 AND stand = 'wartet'
                 LIMIT 1",
            )?;
            let mut rein = tx.prepare(
                "INSERT INTO uebernahmen
                     (quelle, quelle_schluessel, zone, ziel, name, mime, groesse, angelegt_at,
                      ohne_kopie, nur_wlan)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            )?;
            let at = jetzt();
            for n in neue {
                if bekannt.exists(params![n.quelle_schluessel])? {
                    aus.bekannt += 1;
                    continue;
                }
                rein.execute(params![
                    n.quelle,
                    n.quelle_schluessel,
                    n.zone,
                    n.ziel,
                    n.name,
                    n.mime,
                    n.groesse as i64,
                    at,
                    n.ohne_kopie,
                    n.nur_wlan
                ])?;
                aus.neu += 1;
            }
        }
        tx.commit()?;
        Ok(aus)
    }

    /// Die nächste wartende Datei, in der Reihenfolge des Einreihens.
    pub fn uebernahme_naechste(&self) -> Ergebnis<Option<Uebernahme>> {
        Ok(self
            .db()
            .query_row(
                "SELECT id, quelle, quelle_schluessel, zone, ziel, name, mime, groesse,
                        ohne_kopie, nur_wlan
                 FROM uebernahmen WHERE stand = 'wartet' ORDER BY id LIMIT 1",
                [],
                |z| {
                    Ok(Uebernahme {
                        id: z.get(0)?,
                        quelle: z.get(1)?,
                        quelle_schluessel: z.get(2)?,
                        zone: z.get(3)?,
                        ziel: z.get(4)?,
                        name: z.get(5)?,
                        mime: z.get(6)?,
                        groesse: z.get::<_, i64>(7)? as u64,
                        ohne_kopie: z.get(8)?,
                        nur_wlan: z.get(9)?,
                    })
                },
            )
            .optional()?)
    }

    /// Übernommen: die entstandene Sache merken -- und die Quelle für immer.
    pub fn uebernahme_fertig(&self, u: &Uebernahme, uuid: &str) -> Ergebnis<()> {
        let mut db = self.db();
        let tx = db.transaction()?;
        tx.execute(
            "UPDATE uebernahmen SET stand = 'fertig', ergebnis = ?2 WHERE id = ?1",
            params![u.id, uuid],
        )?;
        tx.execute(
            "INSERT OR REPLACE INTO uebernommen (quelle_schluessel, zone, uuid, uebernommen_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![u.quelle_schluessel, u.zone, uuid, jetzt()],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Übersprungen, weil derselbe Inhalt im Ziel schon liegt. Gilt danach
    /// ebenfalls als übernommen.
    pub fn uebernahme_uebersprungen(&self, u: &Uebernahme, vorhanden: &str) -> Ergebnis<()> {
        let mut db = self.db();
        let tx = db.transaction()?;
        tx.execute(
            "UPDATE uebernahmen SET stand = 'uebersprungen', ergebnis = ?2 WHERE id = ?1",
            params![u.id, vorhanden],
        )?;
        tx.execute(
            "INSERT OR REPLACE INTO uebernommen (quelle_schluessel, zone, uuid, uebernommen_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![u.quelle_schluessel, u.zone, vorhanden, jetzt()],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Liegt derselbe Inhalt (Klartext-Abdruck) im Ziel schon -- außerhalb des
    /// Papierkorbs? Dann die Kennung der vorhandenen Sache.
    pub fn uebernahme_schon_da(
        &self,
        zone: &str,
        ziel: Option<&str>,
        abdruck: &str,
    ) -> Ergebnis<Option<String>> {
        let db = self.db();
        Ok(if zone == "galerie" {
            db.query_row(
                "SELECT uuid FROM medien WHERE abdruck = ?1 AND album IS ?2
                 AND papierkorb_at IS NULL LIMIT 1",
                params![abdruck, ziel],
                |z| z.get(0),
            )
            .optional()?
        } else {
            db.query_row(
                "SELECT uuid FROM dateien WHERE abdruck = ?1 AND zone = ?2 AND eltern IS ?3
                 AND papierkorb_at IS NULL LIMIT 1",
                params![abdruck, zone, ziel],
                |z| z.get(0),
            )
            .optional()?
        })
    }

    /// Nicht übernommen, mit Grund. Hält den Lauf nicht an.
    pub fn uebernahme_fehler(&self, id: i64, grund: &str) -> Ergebnis<()> {
        self.db().execute(
            "UPDATE uebernahmen SET stand = 'fehler', grund = ?2 WHERE id = ?1",
            params![id, grund],
        )?;
        Ok(())
    }

    pub fn uebernahmestand(&self) -> Ergebnis<Uebernahmestand> {
        let db = self.db();
        let mut a = db.prepare(
            "SELECT stand, COUNT(*), COALESCE(SUM(groesse), 0) FROM uebernahmen GROUP BY stand",
        )?;
        let mut s = Uebernahmestand::default();
        let zeilen = a.query_map([], |z| {
            Ok((
                z.get::<_, String>(0)?,
                z.get::<_, i64>(1)? as u64,
                z.get::<_, i64>(2)? as u64,
            ))
        })?;
        for z in zeilen {
            let (stand, anzahl, bytes) = z?;
            match stand.as_str() {
                "wartet" => {
                    s.wartet = anzahl;
                    s.bytes_offen = bytes;
                }
                "fertig" => s.fertig = anzahl,
                "uebersprungen" => s.uebersprungen = anzahl,
                "fehler" => s.fehler = anzahl,
                _ => {}
            }
        }
        Ok(s)
    }

    /// Die Liste „nicht übernommen“.
    pub fn uebernahmen_fehlgeschlagen(&self) -> Ergebnis<Vec<NichtUebernommen>> {
        let db = self.db();
        let mut a = db.prepare(
            "SELECT name, quelle, COALESCE(grund, '') FROM uebernahmen
             WHERE stand = 'fehler' ORDER BY id",
        )?;
        let liste = a
            .query_map([], |z| {
                Ok(NichtUebernommen {
                    name: z.get(0)?,
                    quelle: z.get(1)?,
                    grund: z.get(2)?,
                })
            })?
            .collect::<Result<_, _>>()?;
        Ok(liste)
    }

    /// Ein Lauf ist zu Ende gesehen: Erledigtes und Fehler aus der Anzeige
    /// nehmen. Was übernommen wurde, bleibt in `uebernommen`.
    pub fn uebernahmen_abschliessen(&self) -> Ergebnis<()> {
        self.db()
            .execute("DELETE FROM uebernahmen WHERE stand <> 'wartet'", [])?;
        Ok(())
    }

    /// Alle wartenden verwerfen (der Mensch hat den Lauf abgebrochen).
    pub fn uebernahmen_verwerfen(&self) -> Ergebnis<u64> {
        Ok(self
            .db()
            .execute("DELETE FROM uebernahmen WHERE stand = 'wartet'", [])? as u64)
    }

    /// „Liste der Übernommenen vergessen“: Danach darf alles noch einmal
    /// übernommen werden (Entscheidung 9, für einen Neuanfang).
    pub fn uebernommen_vergessen(&self) -> Ergebnis<u64> {
        Ok(self.db().execute("DELETE FROM uebernommen", [])? as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn speicher() -> Speicher {
        Speicher::im_arbeitsspeicher().unwrap()
    }

    fn neu(n: u32) -> NeueUebernahme {
        NeueUebernahme {
            quelle: format!("/Bilder/{n}.jpg"),
            quelle_schluessel: format!("/Bilder/{n}.jpg|100|1"),
            zone: "galerie".into(),
            ziel: Some("album-1".into()),
            name: format!("{n}.jpg"),
            mime: "image/jpeg".into(),
            groesse: 100,
            ..Default::default()
        }
    }

    #[test]
    fn eingereiht_wird_der_reihe_nach_und_nichts_doppelt() {
        let s = speicher();
        let e = s.uebernahmen_einreihen(&[neu(1), neu(2), neu(1)]).unwrap();
        assert_eq!(e, Eingereiht { neu: 2, bekannt: 1 });

        let a = s.uebernahme_naechste().unwrap().unwrap();
        assert_eq!(a.name, "1.jpg");
        let st = s.uebernahmestand().unwrap();
        assert_eq!((st.wartet, st.bytes_offen, st.gesamt()), (2, 200, 2));
    }

    #[test]
    fn uebernommenes_kommt_nicht_wieder_auch_nicht_nach_dem_abschliessen() {
        let s = speicher();
        s.uebernahmen_einreihen(&[neu(1)]).unwrap();
        let a = s.uebernahme_naechste().unwrap().unwrap();
        s.uebernahme_fertig(&a, "bild-1").unwrap();
        s.uebernahmen_abschliessen().unwrap();

        // Ein zweiter Lauf über dieselben Dateien legt nichts an.
        let e = s.uebernahmen_einreihen(&[neu(1), neu(2)]).unwrap();
        assert_eq!(e, Eingereiht { neu: 1, bekannt: 1 });

        // Nach dem Vergessen darf es wieder hinein.
        assert_eq!(s.uebernommen_vergessen().unwrap(), 1);
        s.uebernahmen_verwerfen().unwrap();
        let e = s.uebernahmen_einreihen(&[neu(1)]).unwrap();
        assert_eq!(e.neu, 1);
    }

    #[test]
    fn fehler_halten_nicht_an_und_stehen_in_der_liste() {
        let s = speicher();
        s.uebernahmen_einreihen(&[neu(1), neu(2)]).unwrap();
        let a = s.uebernahme_naechste().unwrap().unwrap();
        s.uebernahme_fehler(a.id, "Quelle fort").unwrap();

        assert_eq!(s.uebernahme_naechste().unwrap().unwrap().name, "2.jpg");
        let liste = s.uebernahmen_fehlgeschlagen().unwrap();
        assert_eq!(liste.len(), 1);
        assert_eq!(liste[0].grund, "Quelle fort");
        // Ein fehlgeschlagener gilt nicht als übernommen.
        s.uebernahmen_abschliessen().unwrap();
        assert_eq!(s.uebernahmen_einreihen(&[neu(1)]).unwrap().neu, 1);
    }

    #[test]
    fn uebersprungenes_gilt_als_uebernommen() {
        let s = speicher();
        s.uebernahmen_einreihen(&[neu(1)]).unwrap();
        let a = s.uebernahme_naechste().unwrap().unwrap();
        s.uebernahme_uebersprungen(&a, "bild-alt").unwrap();
        assert_eq!(s.uebernahmestand().unwrap().uebersprungen, 1);
        assert_eq!(s.uebernahmen_einreihen(&[neu(1)]).unwrap().bekannt, 1);
    }

    #[test]
    fn derselbe_inhalt_im_selben_album_ist_schon_da() {
        let s = speicher();
        let b = crate::Bild {
            uuid: "bild-1".into(),
            album: Some("album-1".into()),
            abdruck: Some("abc".into()),
            ..Default::default()
        };
        s.bild_schreiben(&b, crate::Protokoll::Merken).unwrap();

        assert_eq!(
            s.uebernahme_schon_da("galerie", Some("album-1"), "abc")
                .unwrap()
                .as_deref(),
            Some("bild-1")
        );
        // Anderes Album, anderer Inhalt: nicht da.
        assert!(s
            .uebernahme_schon_da("galerie", Some("album-2"), "abc")
            .unwrap()
            .is_none());
        assert!(s
            .uebernahme_schon_da("galerie", Some("album-1"), "xyz")
            .unwrap()
            .is_none());
    }
}
