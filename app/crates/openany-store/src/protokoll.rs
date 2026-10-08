//! Was sich HIER geaendert hat -- das Gegenstueck zu openanys `change_log`.
//!
//! **Ohne Protokoll gibt es keinen Abgleich**, und zwar aus einem Grund, der
//! erst beim zweiten Hinsehen zwingt: Ein geloeschter Datensatz kann nicht
//! sagen, dass es ihn gab. Ein Vergleich der Bestaende faende ihn schlicht
//! nicht -- und die Gegenseite spielte ihn zurueck.
//!
//! Geschrieben wird beim Speichern (siehe [`crate::Speicher`]), gelesen beim
//! Schieben. Die Marke ist die `id` des zuletzt gelieferten Eintrags.

use crate::{Ergebnis, Speicher};
use chrono::Utc;
use openany_client::{Art, Was};
use rusqlite::params;
use std::collections::BTreeMap;

/// Soll diese Aenderung ins Protokoll?
///
/// **Der Schalter ist der Unterschied zwischen einem Abgleich und einem
/// Pingpong.** Was der Laeufer gerade von drueben gezogen hat, ist keine
/// hiesige Aenderung -- traegt er es trotzdem ein, schiebt der naechste
/// Durchgang es unveraendert zurueck. Der Server antwortet dann artig
/// `unchanged`, es geht also nichts kaputt; es kostet nur bei jedem Lauf eine
/// Runde ueber die Mobilfunkleitung fuer jede Sache, die ohnehin schon
/// stimmt.
///
/// Was der Laeufer **selbst** neu erzeugt -- eine Konfliktkopie etwa -- geht
/// sehr wohl ins Protokoll: Die kennt drueben niemand.
///
/// **Seit mehr als zwei Geraeten: [`Protokoll::Von`].** Mit `Still` kam bei
/// einem dritten Geraet nie an, was A an B geschickt hat, solange A und C
/// sich nicht treffen. `Von` traegt das Angenommene ein, mit der
/// Gegenstelle, von der es kam -- und an genau die geht es nicht zurueck.
/// Dass ein Kreis A→B→C→A trotzdem zur Ruhe kommt, liegt an der zweiten
/// Regel: Wer annimmt, schreibt nur, was sich wirklich aendert. Beim dritten
/// Geraet ist der Stand schon da, also entsteht dort kein Eintrag mehr.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protokoll<'a> {
    /// Eine Aenderung dieses Geraets. Sie wird beim naechsten Lauf geschoben.
    Merken,
    /// Angewendet, was von drueben kam. Nichts einzutragen.
    Still,
    /// Angewendet, was von dieser Gegenstelle kam: eintragen, aber nicht an
    /// sie zurueckschicken.
    Von(&'a str),
}

/// Ein Eintrag im Protokoll.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Aenderung {
    pub id: i64,
    pub art: Art,
    pub schluessel: String,
    pub was: Was,
    pub at: String,
    /// Von welcher Gegenstelle die Aenderung kam; `None` = hier entstanden.
    pub herkunft: Option<String>,
}

/// Wie viele Rohzeilen eine Seite hoechstens liest -- dieselbe Zahl wie
/// drueben in `ChangeLog::PAGE`.
pub const SEITE: usize = 500;

impl Speicher {
    /// Eine Aenderung festhalten.
    ///
    /// Oeffentlich, weil die Oberflaeche eigene Wege haben darf, etwas zu
    /// schreiben. Wer das tut, muss selbst daran denken -- deshalb nehmen
    /// alle Schreibwege in [`crate::Speicher`] das [`Protokoll`] als
    /// Pflichtangabe entgegen und nicht als Voreinstellung: Vergessen soll
    /// nicht die bequemere Haelfte sein.
    pub fn merken(&self, art: &Art, schluessel: &str, was: Was) -> Ergebnis<()> {
        self.merken_von(art, schluessel, was, None)
    }

    /// Wie [`Speicher::merken`], mit der Gegenstelle, von der es kam.
    pub fn merken_von(
        &self,
        art: &Art,
        schluessel: &str,
        was: Was,
        herkunft: Option<&str>,
    ) -> Ergebnis<()> {
        self.db().execute(
            "INSERT INTO aenderungen (art, schluessel, was, at, herkunft) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                art.ueber_die_leitung(),
                schluessel,
                was.ueber_die_leitung(),
                Utc::now().to_rfc3339(),
                herkunft,
            ],
        )?;

        Ok(())
    }

    /// Die eigene Marke, ohne etwas zu holen.
    pub fn eigene_marke_jetzt(&self) -> Ergebnis<i64> {
        Ok(self.db().query_row(
            "SELECT COALESCE(MAX(id), 0) FROM aenderungen WHERE projekt IS NULL",
            [],
            |z| z.get(0),
        )?)
    }

    /// Aenderungen seit der Marke -- hoechstens [`SEITE`] Rohzeilen.
    ///
    /// **Je Sache nur der letzte Stand** innerhalb dieses Ausschnitts: Wer
    /// eine Notiz zwanzigmal gespeichert hat, braucht sie nicht zwanzigmal
    /// abzugleichen.
    ///
    /// **Die Marke wandert trotzdem ueber alle gelesenen Rohzeilen** und
    /// nicht nur ueber die zurueckgegebenen. Sonst bliebe sie bei einer Seite
    /// aus zwanzig Speicherungen derselben Notiz auf der ersten stehen, und
    /// der Abgleich liefe in einer Schleife -- immer dieselbe Seite, immer
    /// dieselbe eine Notiz.
    ///
    /// Die Reihenfolge ist die der ersten Erwaehnung je Sache; sie ist
    /// stabil, damit ein Lauf zweimal dasselbe schickt und sich Fehler
    /// nachvollziehen lassen.
    pub fn aenderungen_seit(&self, marke: i64) -> Ergebnis<(Vec<Aenderung>, i64, bool)> {
        let db = self.db();
        let mut abfrage = db.prepare(
            // `projekt IS NULL`: Was zu einem Projekt gehoert, reist in
            // dessen eigenem Strom (siehe `projekte.rs`). Ohne diese
            // Bedingung schoebe der persoenliche Lauf Karten an openany.de --
            // unter einer Art, die der persoenliche Endpunkt nicht kennt.
            "SELECT id, art, schluessel, was, at, herkunft FROM aenderungen
             WHERE id > ?1 AND projekt IS NULL ORDER BY id LIMIT ?2",
        )?;

        let rohe: Vec<Aenderung> = abfrage
            .query_map(params![marke, SEITE as i64], |z| {
                Ok(Aenderung {
                    id: z.get(0)?,
                    art: Art::aus(&z.get::<_, String>(1)?),
                    schluessel: z.get(2)?,
                    // Eine Zeile mit unbekanntem `was` kann es nur geben, wenn
                    // eine neuere Fassung sie geschrieben hat -- und die faengt
                    // `nachziehen` schon ab. `Da` ist hier der harmloseste
                    // Rueckfall: Er schickt einen Stand, er loescht nichts.
                    was: Was::aus(&z.get::<_, String>(3)?).unwrap_or(Was::Da),
                    at: z.get(4)?,
                    herkunft: z.get(5)?,
                })
            })?
            .collect::<Result<_, _>>()?;

        let gelesen = rohe.len();
        let marke_danach = rohe.last().map(|a| a.id).unwrap_or(marke);

        // BTreeMap nach der ersten Erwaehnung: der spaetere Eintrag gewinnt,
        // der Platz in der Liste bleibt der erste.
        let mut platz: BTreeMap<String, usize> = BTreeMap::new();
        let mut letzte: Vec<Aenderung> = Vec::new();

        for a in rohe {
            let schluessel = format!("{}|{}", a.art.ueber_die_leitung(), a.schluessel);

            match platz.get(&schluessel) {
                Some(&i) => letzte[i] = a,
                None => {
                    platz.insert(schluessel, letzte.len());
                    letzte.push(a);
                }
            }
        }

        Ok((letzte, marke_danach, gelesen == SEITE))
    }

    /// Altes wegraeumen -- **aber nie den letzten Eintrag je Sache**.
    ///
    /// Ohne diese Ausnahme verloere der Abgleich seine Grabsteine: Eine vor
    /// Monaten geloeschte Notiz stuende in keinem Protokoll mehr, die
    /// Gegenseite hielte sie fuer nie dagewesen und spielte sie zurueck.
    ///
    /// @return wie viele Zeilen fort sind
    pub fn protokoll_kuerzen(&self, tage: i64) -> Ergebnis<usize> {
        let grenze = (Utc::now() - chrono::Duration::days(tage)).to_rfc3339();

        Ok(self.db().execute(
            "DELETE FROM aenderungen
             WHERE at < ?1
               AND id NOT IN (SELECT MAX(id) FROM aenderungen GROUP BY art, schluessel)",
            params![grenze],
        )?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Notiz, Speicher};

    fn speicher() -> Speicher {
        Speicher::im_arbeitsspeicher().unwrap()
    }

    #[test]
    fn ohne_aenderungen_steht_die_marke_still() {
        let s = speicher();
        let (liste, marke, mehr) = s.aenderungen_seit(0).unwrap();

        assert!(liste.is_empty());
        assert_eq!(marke, 0);
        assert!(!mehr);
    }

    #[test]
    fn zwanzig_speicherungen_ergeben_eine_aenderung() {
        let s = speicher();

        for i in 0..20 {
            s.notiz_schreiben(
                &Notiz {
                    zk_id: "zk-1".into(),
                    titel: format!("Fassung {i}"),
                    ..Default::default()
                },
                Protokoll::Merken,
            )
            .unwrap();
        }

        let (liste, marke, _) = s.aenderungen_seit(0).unwrap();

        assert_eq!(liste.len(), 1);
        assert_eq!(
            marke, 20,
            "die Marke wandert ueber alle Rohzeilen, nicht nur ueber die gelieferte"
        );
    }

    #[test]
    fn der_spaetere_stand_gewinnt_und_behaelt_den_platz() {
        let s = speicher();

        s.notiz_schreiben(&Notiz::mit_id("zk-1"), Protokoll::Merken)
            .unwrap();
        s.notiz_schreiben(&Notiz::mit_id("zk-2"), Protokoll::Merken)
            .unwrap();
        s.papierkorb(&Art::Notiz, "zk-1", Protokoll::Merken)
            .unwrap();

        let (liste, _, _) = s.aenderungen_seit(0).unwrap();

        assert_eq!(liste.len(), 2);
        assert_eq!(
            liste[0].schluessel, "zk-1",
            "der Platz der ersten Erwaehnung"
        );
        assert_eq!(liste[0].was, Was::Papierkorb, "der spaetere Stand");
        assert_eq!(liste[1].schluessel, "zk-2");
    }

    #[test]
    fn was_von_drueben_kam_steht_nicht_im_protokoll() {
        // Der Test, der das Pingpong verhindert.
        let s = speicher();

        s.notiz_schreiben(&Notiz::mit_id("zk-1"), Protokoll::Still)
            .unwrap();

        let (liste, _, _) = s.aenderungen_seit(0).unwrap();

        assert!(
            liste.is_empty(),
            "sonst schoebe der naechste Lauf es zurueck"
        );
        assert!(
            s.notiz("zk-1").unwrap().is_some(),
            "gespeichert wurde sie trotzdem"
        );
    }

    #[test]
    fn die_marke_grenzt_ab() {
        let s = speicher();

        s.notiz_schreiben(&Notiz::mit_id("zk-1"), Protokoll::Merken)
            .unwrap();
        let (_, marke, _) = s.aenderungen_seit(0).unwrap();
        s.notiz_schreiben(&Notiz::mit_id("zk-2"), Protokoll::Merken)
            .unwrap();

        let (liste, _, _) = s.aenderungen_seit(marke).unwrap();

        assert_eq!(liste.len(), 1);
        assert_eq!(liste[0].schluessel, "zk-2");
    }

    #[test]
    fn eine_volle_seite_meldet_mehr() {
        let s = speicher();

        for i in 0..SEITE {
            s.notiz_schreiben(&Notiz::mit_id(&format!("zk-{i}")), Protokoll::Merken)
                .unwrap();
        }
        s.notiz_schreiben(&Notiz::mit_id("zk-danach"), Protokoll::Merken)
            .unwrap();

        let (liste, marke, mehr) = s.aenderungen_seit(0).unwrap();

        assert_eq!(liste.len(), SEITE);
        assert!(mehr);

        let (rest, _, mehr_danach) = s.aenderungen_seit(marke).unwrap();

        assert_eq!(rest.len(), 1);
        assert!(!mehr_danach);
    }

    #[test]
    fn kuerzen_laesst_den_grabstein_stehen() {
        let s = speicher();

        s.notiz_schreiben(&Notiz::mit_id("zk-1"), Protokoll::Merken)
            .unwrap();
        s.notiz_schreiben(&Notiz::mit_id("zk-1"), Protokoll::Merken)
            .unwrap();
        s.loeschen(&Art::Notiz, "zk-1", Protokoll::Merken).unwrap();

        // Alle drei Zeilen kuenstlich altern lassen.
        s.mit_verbindung(|db| {
            db.execute(
                "UPDATE aenderungen SET at = '2020-01-01T00:00:00+00:00'",
                [],
            )
        })
        .unwrap();

        let fort = s.protokoll_kuerzen(90).unwrap();

        assert_eq!(fort, 2, "die beiden ueberholten");

        let (liste, _, _) = s.aenderungen_seit(0).unwrap();

        assert_eq!(liste.len(), 1);
        assert_eq!(
            liste[0].was,
            Was::Fort,
            "der Grabstein bleibt -- sonst spielt die Gegenseite die Notiz zurueck"
        );
    }

    #[test]
    fn kuerzen_laesst_junges_stehen() {
        let s = speicher();

        s.notiz_schreiben(&Notiz::mit_id("zk-1"), Protokoll::Merken)
            .unwrap();
        s.notiz_schreiben(&Notiz::mit_id("zk-1"), Protokoll::Merken)
            .unwrap();

        assert_eq!(s.protokoll_kuerzen(90).unwrap(), 0);
    }
}
