//! Notiz-Anhaenge -- eine ZUORDNUNG, keine Datei.
//!
//! Der Notiztext bleibt Standard-Markdown: `![](bild.png)`. Was dieser
//! relative Pfad bedeutet, steht hier -- er zeigt auf ein Bild der Galerie
//! oder auf eine Datei. **Die Bytes gehoeren dem Ziel** und reisen einmal,
//! ueber dessen eigenen Eintrag; ein Anhang traegt keine.
//!
//! **Regel 1 auf der Leitung: jede Datei genau einmal.** Drueben laesst der
//! `DeltaFeed` ein Bild, das an einer Notiz haengt, aus dem `media`-Strom
//! ausdruecklich weg und meldet es nur als `note_asset` -- mit
//! `target_type` und `target_uuid`. Wer die Bytes will, fragt das Ziel.
//!
//! **Geschluesselt nach MAPPENPFAD.** So kommt er ueber die Leitung (dort
//! ausdruecklich portabel gemacht: die lokale Mappen-Id gilt nur auf einer
//! Seite), und so heisst die Mappe hier ohnehin -- `mappen.pfad` ist der
//! Primaerschluessel.
//!
//! **Ohne Papierkorb.** Drueben hat `note_assets` keinen: Die Zuordnung ist
//! da oder sie ist weg. Ein weggelegter Anhang waere ein Bild, das im Text
//! steht und trotzdem nicht gelten soll -- das gibt es nicht.

use crate::{Ergebnis, Protokoll, Speicher};
use openany_client::{Art, Was};
use rusqlite::{params, OptionalExtension, Row};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Anhang {
    /// Der Pfad der Mappe, in der die Notiz liegt.
    pub mappe: String,
    /// Der relative Pfad, wie er im Notiztext steht.
    pub pfad: String,
    /// `media` oder `file`.
    pub ziel_art: String,
    pub ziel_uuid: String,
    pub groesse: u64,
    /// Abdruck des Originals -- `None`, wenn das Ziel drueben keinen hat.
    pub abdruck: Option<String>,
    /// Abdruck der Vorschau; nur bei Bildern. **Das ist der, der im
    /// Flusstext gebraucht wird** -- klein, und auf einem Geraet "bei Bedarf"
    /// immer da.
    pub vorschau: Option<String>,
}

impl Anhang {
    /// Der Schluessel, unter dem er im Protokoll und auf der Leitung steht.
    ///
    /// Zusammengesetzt und nicht gespeichert: Zwei Spalten und ein Feld, das
    /// aus beiden entsteht, liefen irgendwann auseinander.
    pub fn schluessel(&self) -> String {
        format!("{}|{}", self.mappe, self.pfad)
    }
}

const SPALTEN: &str = "mappe, pfad, ziel_art, ziel_uuid, groesse, abdruck, vorschau";

fn anhang_aus(z: &Row<'_>) -> rusqlite::Result<Anhang> {
    Ok(Anhang {
        mappe: z.get(0)?,
        pfad: z.get(1)?,
        ziel_art: z.get(2)?,
        ziel_uuid: z.get(3)?,
        groesse: z.get::<_, i64>(4)?.max(0) as u64,
        abdruck: z.get(5)?,
        vorschau: z.get(6)?,
    })
}

impl Speicher {
    pub fn anhang_schreiben(&self, a: &Anhang, protokoll: Protokoll) -> Ergebnis<()> {
        self.db().execute(
            &format!(
                "INSERT INTO anhaenge ({SPALTEN}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT (mappe, pfad) DO UPDATE SET
                    ziel_art = excluded.ziel_art,
                    ziel_uuid = excluded.ziel_uuid,
                    groesse = excluded.groesse,
                    abdruck = excluded.abdruck,
                    vorschau = excluded.vorschau"
            ),
            params![
                a.mappe,
                a.pfad,
                a.ziel_art,
                a.ziel_uuid,
                a.groesse as i64,
                a.abdruck,
                a.vorschau
            ],
        )?;

        match protokoll {
            Protokoll::Merken => self.merken(&Art::Notizanhang, &a.schluessel(), Was::Da),
            Protokoll::Still => Ok(()),
            Protokoll::Von(h) => {
                self.merken_von(&Art::Notizanhang, &a.schluessel(), Was::Da, Some(h))
            }
        }
    }

    pub fn anhang(&self, mappe: &str, pfad: &str) -> Ergebnis<Option<Anhang>> {
        Ok(self
            .db()
            .query_row(
                &format!("SELECT {SPALTEN} FROM anhaenge WHERE mappe = ?1 AND pfad = ?2"),
                params![mappe, pfad],
                anhang_aus,
            )
            .optional()?)
    }

    /// Alle Anhaenge einer Mappe -- das, was die Oberflaeche zum Aufloesen
    /// eines Notiztexts braucht.
    pub fn anhaenge_der_mappe(&self, mappe: &str) -> Ergebnis<Vec<Anhang>> {
        let db = self.db();
        let mut abfrage = db.prepare(&format!(
            "SELECT {SPALTEN} FROM anhaenge WHERE mappe = ?1 ORDER BY pfad"
        ))?;
        let liste = abfrage
            .query_map(params![mappe], anhang_aus)?
            .collect::<Result<_, _>>()?;
        Ok(liste)
    }

    /// Alle Anhaenge, ueber alle Mappen -- fuer den Abgleich der Inhalte.
    pub fn anhaenge_alle(&self) -> Ergebnis<Vec<Anhang>> {
        let db = self.db();
        let mut abfrage = db.prepare(&format!("SELECT {SPALTEN} FROM anhaenge"))?;
        let liste = abfrage
            .query_map([], anhang_aus)?
            .collect::<Result<_, _>>()?;
        Ok(liste)
    }

    /// Eine Zuordnung faellt weg. `false`, wenn es sie gar nicht gab.
    pub fn anhang_entfernen(
        &self,
        mappe: &str,
        pfad: &str,
        protokoll: Protokoll,
    ) -> Ergebnis<bool> {
        let n = self.db().execute(
            "DELETE FROM anhaenge WHERE mappe = ?1 AND pfad = ?2",
            params![mappe, pfad],
        )?;

        if n == 0 {
            return Ok(false);
        }

        let schluessel = format!("{mappe}|{pfad}");

        match protokoll {
            Protokoll::Merken => self.merken(&Art::Notizanhang, &schluessel, Was::Fort)?,
            Protokoll::Still => {}
            Protokoll::Von(h) => {
                self.merken_von(&Art::Notizanhang, &schluessel, Was::Fort, Some(h))?
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s() -> Speicher {
        Speicher::im_arbeitsspeicher().unwrap()
    }

    fn anhang(mappe: &str, pfad: &str) -> Anhang {
        Anhang {
            mappe: mappe.into(),
            pfad: pfad.into(),
            ziel_art: "media".into(),
            ziel_uuid: "ab16147a".into(),
            groesse: 732_444,
            abdruck: Some("original".into()),
            vorschau: Some("klein".into()),
        }
    }

    #[test]
    fn ein_anhang_kommt_an_und_wird_wiedergefunden() {
        let speicher = s();
        speicher
            .anhang_schreiben(&anhang("UniStuff/BWL", "bild.png"), Protokoll::Still)
            .unwrap();

        let gefunden = speicher
            .anhang("UniStuff/BWL", "bild.png")
            .unwrap()
            .unwrap();

        assert_eq!(gefunden.ziel_uuid, "ab16147a");
        assert_eq!(gefunden.ziel_art, "media");
    }

    /// DER SCHLUESSEL IST DER DER LEITUNG. Mappe und Pfad zusammen -- genau so
    /// steht er im Protokoll drueben, und nur so findet ihn `nachschlagen`
    /// wieder.
    #[test]
    fn der_schluessel_setzt_sich_aus_mappe_und_pfad_zusammen() {
        assert_eq!(
            anhang("UniStuff/BWL", "Screenshot 2025.png").schluessel(),
            "UniStuff/BWL|Screenshot 2025.png"
        );
    }

    /// EIN PFAD JE MAPPE, und ein zweiter Eintrag ersetzt den ersten. Sonst
    /// zeigte derselbe `![](bild.png)` je nach Zeile woandershin.
    #[test]
    fn derselbe_pfad_zeigt_immer_nur_auf_eines() {
        let speicher = s();
        speicher
            .anhang_schreiben(&anhang("Mappe", "bild.png"), Protokoll::Still)
            .unwrap();

        let mut neu = anhang("Mappe", "bild.png");
        neu.ziel_uuid = "anders".into();
        speicher.anhang_schreiben(&neu, Protokoll::Still).unwrap();

        let alle = speicher.anhaenge_der_mappe("Mappe").unwrap();

        assert_eq!(alle.len(), 1);
        assert_eq!(alle[0].ziel_uuid, "anders");
    }

    /// Derselbe Dateiname in zwei Mappen sind zwei Anhaenge. Wer nur nach dem
    /// Pfad schluesselte, zeigte in der einen Notiz das Bild der anderen.
    #[test]
    fn zwei_mappen_teilen_sich_keinen_pfad() {
        let speicher = s();
        speicher
            .anhang_schreiben(&anhang("Eine", "bild.png"), Protokoll::Still)
            .unwrap();
        let mut andere = anhang("Andere", "bild.png");
        andere.ziel_uuid = "zweites".into();
        speicher
            .anhang_schreiben(&andere, Protokoll::Still)
            .unwrap();

        assert_eq!(speicher.anhaenge_der_mappe("Eine").unwrap().len(), 1);
        assert_eq!(
            speicher.anhaenge_der_mappe("Andere").unwrap()[0].ziel_uuid,
            "zweites"
        );
    }

    #[test]
    fn eine_zuordnung_faellt_weg_und_sagt_ob_es_sie_gab() {
        let speicher = s();
        speicher
            .anhang_schreiben(&anhang("Mappe", "bild.png"), Protokoll::Still)
            .unwrap();

        assert!(speicher
            .anhang_entfernen("Mappe", "bild.png", Protokoll::Still)
            .unwrap());
        assert!(!speicher
            .anhang_entfernen("Mappe", "bild.png", Protokoll::Still)
            .unwrap());
        assert!(speicher.anhaenge_der_mappe("Mappe").unwrap().is_empty());
    }

    /// WAS VON DRUEBEN KAM, TRAEGT SEINE HERKUNFT.
    ///
    /// Der Eintrag entsteht trotzdem -- ein drittes Geraet soll ihn ja
    /// bekommen. Was ihn davor bewahrt, im Kreis zu laufen, ist die Herkunft:
    /// Der Laeufer schiebt nichts dorthin zurueck, woher es kam.
    #[test]
    fn was_von_drueben_kam_traegt_seine_herkunft() {
        let speicher = s();
        speicher
            .anhang_schreiben(
                &anhang("Mappe", "bild.png"),
                Protokoll::Von("https://openany.de"),
            )
            .unwrap();

        let (liste, _, _) = speicher.aenderungen_seit(0).unwrap();

        assert_eq!(liste.len(), 1);
        assert_eq!(liste[0].herkunft.as_deref(), Some("https://openany.de"));
        assert_eq!(liste[0].schluessel, "Mappe|bild.png");
    }
}
