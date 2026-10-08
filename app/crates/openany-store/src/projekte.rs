//! Projekte und was in ihnen liegt.
//!
//! **Eine Tabelle fuer alle Arten.** Drueben haben Boards, Spalten und Karten
//! eigene Tabellen -- dort haengen Fremdschluessel, Zaehler und die Abfragen
//! der Webapp daran. Hier zeichnet und aendert das Programm nur, und zehn
//! Schema-Staende fuer zehn Planungstypen waeren zehnmal derselbe Handgriff.
//! Was eine Art ausmacht, sagt der Server (`Abgleichsarten`); die Felder
//! reisen als JSON und werden hier nicht gedeutet.
//!
//! **Ein Projekt ist eine Gegenstelle.** Der Name lautet
//! `<basis>#projekt:<uuid>`, und damit haengen Marken und Urspruenge daran,
//! ohne dass irgendetwas Neues erfunden werden muesste -- sie tragen die
//! Gegenstelle seit dem ersten Tag im Schluessel.
//!
//! **Das Protokoll trennt beides.** Eine Aenderung an einer Karte gehoert in
//! den Strom ihres Projekts, eine an einer Notiz in den persoenlichen. Ohne
//! diese Trennung schoebe jeder Lauf dem anderen seine Sachen unter.

use crate::{Ergebnis, Protokoll, Speicher};
use chrono::Utc;
use openany_client::Was;
use rusqlite::{params, OptionalExtension, Row};

/// Ein Projekt, wie es der persoenliche Strom meldet.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Projekt {
    pub uuid: String,
    pub name: String,
    /// `owner` oder `member` -- was das Geraet darf, entscheidet drueben.
    pub rolle: String,
    pub geaendert_at: String,
    /// Nur bei einem LOKALEN Projekt: die unterschriebene Mitgliederliste
    /// (JSON, `openany_nahbereich::Mitgliederliste`). `None`: vom Server.
    pub mitgliederliste: Option<String>,
}

impl Projekt {
    /// Der Name der Gegenstelle fuer diesen Strom.
    ///
    /// **Die Basis gehoert dazu.** Dasselbe Projekt auf zwei Instanzen waere
    /// sonst dieselbe Gegenstelle -- und ihre Marken widersprechen sich.
    pub fn gegenstelle(basis: &str, uuid: &str) -> String {
        format!("{}#projekt:{}", basis.trim_end_matches('/'), uuid)
    }
}

/// Eine Sache in einem Projekt -- Board, Spalte, Karte, spaeter mehr.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Projektsache {
    pub projekt: String,
    /// Wie sie auf der Leitung heisst (`board`, `column`, `card`).
    pub art: String,
    pub uuid: String,
    /// Die uuid des Elternteils; `None` bei einer Art, die am Projekt haengt.
    pub eltern: Option<String>,
    /// Die Felder der Art, wie sie ueber die Leitung kamen -- als JSON.
    pub felder: serde_json::Value,
    pub geaendert_at: String,
}

impl Projektsache {
    /// Ein Feld als Text -- was fehlt, ist leer.
    pub fn text(&self, feld: &str) -> &str {
        self.felder.get(feld).and_then(|w| w.as_str()).unwrap_or("")
    }

    /// Ein Feld als Zahl -- was fehlt, ist 0.
    pub fn zahl(&self, feld: &str) -> i64 {
        self.felder.get(feld).and_then(|w| w.as_i64()).unwrap_or(0)
    }
}

const SACH_SPALTEN: &str = "projekt, art, uuid, eltern, felder, geaendert_at";

fn sache_aus(z: &Row<'_>) -> rusqlite::Result<Projektsache> {
    Ok(Projektsache {
        projekt: z.get(0)?,
        art: z.get(1)?,
        uuid: z.get(2)?,
        eltern: z.get(3)?,
        felder: serde_json::from_str(&z.get::<_, String>(4)?).unwrap_or(serde_json::json!({})),
        geaendert_at: z.get(5)?,
    })
}

fn projekt_aus(z: &Row<'_>) -> rusqlite::Result<Projekt> {
    Ok(Projekt {
        uuid: z.get(0)?,
        name: z.get(1)?,
        rolle: z.get(2)?,
        geaendert_at: z.get(3)?,
        mitgliederliste: z.get(4)?,
    })
}

impl Speicher {
    /* ── Die Projekte selbst ─────────────────────────────────────────── */

    pub fn projekt_schreiben(&self, p: &Projekt) -> Ergebnis<()> {
        self.db().execute(
            "INSERT INTO projekte (uuid, name, rolle, geaendert_at) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT (uuid) DO UPDATE SET
                name = excluded.name,
                rolle = excluded.rolle,
                geaendert_at = excluded.geaendert_at",
            params![p.uuid, p.name, p.rolle, p.geaendert_at],
        )?;

        Ok(())
    }

    /// Ein LOKALES Projekt anlegen oder seine Mitgliederliste ersetzen. Der
    /// Server-Abgleich (`projekt_schreiben`) faesst die Liste nie an.
    pub fn projekt_lokal_schreiben(&self, p: &Projekt, mitgliederliste: &str) -> Ergebnis<()> {
        self.db().execute(
            "INSERT INTO projekte (uuid, name, rolle, geaendert_at, mitgliederliste)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT (uuid) DO UPDATE SET
                name = excluded.name,
                rolle = excluded.rolle,
                geaendert_at = excluded.geaendert_at,
                mitgliederliste = excluded.mitgliederliste",
            params![p.uuid, p.name, p.rolle, p.geaendert_at, mitgliederliste],
        )?;

        Ok(())
    }

    /// Die unterschriebenen Freigaben eines LOKALEN Projekts (JSON), oder
    /// `None`.
    pub fn projekt_freigaben(&self, uuid: &str) -> Ergebnis<Option<String>> {
        Ok(self
            .db()
            .query_row(
                "SELECT freigaben FROM projekte WHERE uuid = ?1",
                params![uuid],
                |z| z.get(0),
            )
            .optional()?
            .flatten())
    }

    pub fn projekt_freigaben_schreiben(&self, uuid: &str, json: &str) -> Ergebnis<()> {
        self.db().execute(
            "UPDATE projekte SET freigaben = ?2 WHERE uuid = ?1",
            params![uuid, json],
        )?;
        Ok(())
    }

    pub fn projekt(&self, uuid: &str) -> Ergebnis<Option<Projekt>> {
        Ok(self
            .db()
            .query_row(
                "SELECT uuid, name, rolle, geaendert_at, mitgliederliste FROM projekte WHERE uuid = ?1",
                params![uuid],
                projekt_aus,
            )
            .optional()?)
    }

    pub fn projekte(&self) -> Ergebnis<Vec<Projekt>> {
        let db = self.db();
        let mut abfrage = db.prepare(
            "SELECT uuid, name, rolle, geaendert_at, mitgliederliste FROM projekte ORDER BY name",
        )?;
        let liste = abfrage
            .query_map([], projekt_aus)?
            .collect::<Result<_, _>>()?;

        Ok(liste)
    }

    /// Ein Projekt und ALLES darin vergessen.
    ///
    /// **Der Fall ist das Ende einer Mitgliedschaft.** Was hier bliebe, waere
    /// eine Kopie, die niemand mehr sehen darf und die niemand mehr
    /// aktualisiert. Mit den Sachen gehen Marke und Urspruenge: Kommt die
    /// Mitgliedschaft je zurueck, faengt der Strom sauber von vorn an.
    pub fn projekt_vergessen(&self, basis: &str, uuid: &str) -> Ergebnis<bool> {
        let gegenstelle = Projekt::gegenstelle(basis, uuid);
        let db = self.db();

        db.execute(
            "DELETE FROM projektsachen WHERE projekt = ?1",
            params![uuid],
        )?;
        db.execute("DELETE FROM aenderungen WHERE projekt = ?1", params![uuid])?;
        db.execute(
            "DELETE FROM marken WHERE gegenstelle = ?1",
            params![gegenstelle],
        )?;
        db.execute(
            "DELETE FROM ursprung WHERE gegenstelle = ?1",
            params![gegenstelle],
        )?;

        Ok(db.execute("DELETE FROM projekte WHERE uuid = ?1", params![uuid])? > 0)
    }

    /* ── Was in ihnen liegt ──────────────────────────────────────────── */

    /// Eine Sache schreiben. `Protokoll::Merken` traegt sie in den Strom DES
    /// PROJEKTS ein, nicht in den persoenlichen.
    pub fn projektsache_schreiben(&self, s: &Projektsache, protokoll: Protokoll) -> Ergebnis<()> {
        let felder = s.felder.to_string();
        let geaendert = if s.geaendert_at.is_empty() {
            Utc::now().to_rfc3339()
        } else {
            s.geaendert_at.clone()
        };

        self.db().execute(
            &format!(
                "INSERT INTO projektsachen ({SACH_SPALTEN}) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT (projekt, uuid) DO UPDATE SET
                    art = excluded.art,
                    eltern = excluded.eltern,
                    felder = excluded.felder,
                    geaendert_at = excluded.geaendert_at"
            ),
            params![s.projekt, s.art, s.uuid, s.eltern, felder, geaendert],
        )?;

        self.projekt_merken(&s.projekt, &s.art, &s.uuid, Was::Da, protokoll)
    }

    pub fn projektsache(&self, projekt: &str, uuid: &str) -> Ergebnis<Option<Projektsache>> {
        Ok(self
            .db()
            .query_row(
                &format!(
                    "SELECT {SACH_SPALTEN} FROM projektsachen WHERE projekt = ?1 AND uuid = ?2"
                ),
                params![projekt, uuid],
                sache_aus,
            )
            .optional()?)
    }

    /// Alle Sachen einer Art in einem Projekt -- was die Ansicht zeichnet.
    pub fn projektsachen(&self, projekt: &str, art: &str) -> Ergebnis<Vec<Projektsache>> {
        let db = self.db();
        let mut abfrage = db.prepare(&format!(
            "SELECT {SACH_SPALTEN} FROM projektsachen
             WHERE projekt = ?1 AND art = ?2 ORDER BY uuid"
        ))?;
        let liste = abfrage
            .query_map(params![projekt, art], sache_aus)?
            .collect::<Result<_, _>>()?;

        Ok(liste)
    }

    /// Die Kinder einer Sache -- Spalten eines Boards, Karten einer Spalte.
    pub fn projektsachen_unter(&self, projekt: &str, eltern: &str) -> Ergebnis<Vec<Projektsache>> {
        let db = self.db();
        let mut abfrage = db.prepare(&format!(
            "SELECT {SACH_SPALTEN} FROM projektsachen
             WHERE projekt = ?1 AND eltern = ?2 ORDER BY uuid"
        ))?;
        let liste = abfrage
            .query_map(params![projekt, eltern], sache_aus)?
            .collect::<Result<_, _>>()?;

        Ok(liste)
    }

    /// Eine Sache entfernen -- MIT ALLEM DARUNTER.
    ///
    /// Dieselbe Regel wie drueben: Der Behaelter nimmt seinen Inhalt mit. Ein
    /// weggeraeumtes Board laesst keine Spalten stehen, deren Board es nicht
    /// mehr gibt; und der Grabstein dafuer ist einer, nicht dreihundert.
    pub fn projektsache_entfernen(
        &self,
        projekt: &str,
        uuid: &str,
        protokoll: Protokoll,
    ) -> Ergebnis<bool> {
        let Some(sache) = self.projektsache(projekt, uuid)? else {
            return Ok(false);
        };

        for kind in self.projektsachen_unter(projekt, uuid)? {
            // Still: Der Grabstein des Behaelters sagt alles. Ein Eintrag je
            // Kind waere dieselbe Auskunft dreihundertmal.
            self.projektsache_entfernen(projekt, &kind.uuid, Protokoll::Still)?;
        }

        self.db().execute(
            "DELETE FROM projektsachen WHERE projekt = ?1 AND uuid = ?2",
            params![projekt, uuid],
        )?;

        self.projekt_merken(projekt, &sache.art, uuid, Was::Fort, protokoll)?;

        Ok(true)
    }

    /* ── Das Protokoll des Projekts ──────────────────────────────────── */

    fn projekt_merken(
        &self,
        projekt: &str,
        art: &str,
        schluessel: &str,
        was: Was,
        protokoll: Protokoll,
    ) -> Ergebnis<()> {
        let herkunft = match protokoll {
            Protokoll::Still => return Ok(()),
            Protokoll::Merken => None,
            Protokoll::Von(h) => Some(h),
        };

        self.db().execute(
            "INSERT INTO aenderungen (art, schluessel, was, at, herkunft, projekt)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                art,
                schluessel,
                was.ueber_die_leitung(),
                Utc::now().to_rfc3339(),
                herkunft,
                projekt,
            ],
        )?;

        Ok(())
    }

    /// Was sich in diesem Projekt seit der Marke geaendert hat.
    ///
    /// Wie [`Speicher::aenderungen_seit`], nur fuer einen Projektstrom -- und
    /// der persoenliche Lauf sieht diese Zeilen nicht (er fragt
    /// `projekt IS NULL`).
    ///
    /// @return (Aenderungen, Marke danach, gibt es mehr)
    pub fn projekt_aenderungen_seit(
        &self,
        projekt: &str,
        marke: i64,
    ) -> Ergebnis<(Vec<Projektaenderung>, i64, bool)> {
        let db = self.db();
        let mut abfrage = db.prepare(
            "SELECT id, art, schluessel, was, herkunft FROM aenderungen
             WHERE projekt = ?1 AND id > ?2 ORDER BY id LIMIT ?3",
        )?;

        let rohe: Vec<Projektaenderung> = abfrage
            .query_map(
                params![projekt, marke, crate::protokoll::SEITE as i64],
                |z| {
                    Ok(Projektaenderung {
                        id: z.get(0)?,
                        art: z.get(1)?,
                        schluessel: z.get(2)?,
                        was: Was::aus(&z.get::<_, String>(3)?).unwrap_or(Was::Da),
                        herkunft: z.get(4)?,
                    })
                },
            )?
            .collect::<Result<_, _>>()?;

        let gelesen = rohe.len();
        let marke_danach = rohe.last().map(|a| a.id).unwrap_or(marke);

        // Je Sache nur der letzte Stand -- wer eine Karte zwanzigmal
        // verschoben hat, schickt sie einmal. Die Marke wandert trotzdem
        // ueber alle gelesenen Zeilen.
        let mut gesehen = std::collections::BTreeSet::new();
        let mut letzte: Vec<Projektaenderung> = Vec::new();

        for a in rohe.into_iter().rev() {
            if gesehen.insert(a.schluessel.clone()) {
                letzte.push(a);
            }
        }

        letzte.reverse();

        Ok((letzte, marke_danach, gelesen == crate::protokoll::SEITE))
    }

    /// Die Grabsteine eines Projekts: was hier geloescht wurde und nicht
    /// wieder da ist -- `(art, uuid, wann)`, je Sache der juengste.
    ///
    /// Fuer den Abgleich vor Ort (lokale Projekte): Ohne Grabstein brachte
    /// das naechste Mitglied eine geloeschte Karte einfach wieder mit.
    pub fn projekt_grabsteine(&self, projekt: &str) -> Ergebnis<Vec<(String, String, String)>> {
        let db = self.db();
        let mut abfrage = db.prepare(
            "SELECT a.art, a.schluessel, MAX(a.at) FROM aenderungen a
             WHERE a.projekt = ?1 AND a.was = 'delete'
               AND NOT EXISTS (SELECT 1 FROM projektsachen s
                               WHERE s.projekt = a.projekt AND s.uuid = a.schluessel)
             GROUP BY a.art, a.schluessel",
        )?;
        let liste = abfrage
            .query_map(params![projekt], |z| Ok((z.get(0)?, z.get(1)?, z.get(2)?)))?
            .collect::<Result<_, _>>()?;

        Ok(liste)
    }

    /// Welche Sachen dieses Projekts tragen eine hiesige Aenderung, die noch
    /// nicht hinaus ist?
    ///
    /// Alles nach der eigenen Marke, das NICHT von `gegenstelle` kam. Der
    /// Projektlauf zieht zuerst und schiebt danach; ohne diese Auskunft
    /// ueberschriebe das Ziehen eine eben verschobene Karte mit dem alten
    /// Stand von drueben -- und das Schieben hielte sie danach fuer etwas,
    /// das von drueben kam. Am 22.09.2026 auf dem Tablet so gesehen.
    pub fn projekt_offene(
        &self,
        projekt: &str,
        marke: i64,
        gegenstelle: &str,
    ) -> Ergebnis<std::collections::BTreeSet<String>> {
        let db = self.db();
        let mut abfrage = db.prepare(
            "SELECT DISTINCT schluessel FROM aenderungen
             WHERE projekt = ?1 AND id > ?2 AND (herkunft IS NULL OR herkunft != ?3)",
        )?;

        let offene = abfrage
            .query_map(params![projekt, marke, gegenstelle], |z| z.get(0))?
            .collect::<Result<_, _>>()?;

        Ok(offene)
    }
}

/// Eine Aenderung im Strom eines Projekts.
///
/// Die Art steht als TEXT und nicht als [`Art`]: Welche Arten es gibt, sagt
/// der Server in seiner Karte, und eine Aufzaehlung hier muesste bei jedem
/// neuen Planungstyp nachgezogen werden -- genau das, was die Karte abschafft.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Projektaenderung {
    pub id: i64,
    pub art: String,
    pub schluessel: String,
    pub was: Was,
    pub herkunft: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn speicher() -> Speicher {
        Speicher::im_arbeitsspeicher().unwrap()
    }

    fn sache(projekt: &str, art: &str, uuid: &str, eltern: Option<&str>) -> Projektsache {
        Projektsache {
            projekt: projekt.into(),
            art: art.into(),
            uuid: uuid.into(),
            eltern: eltern.map(Into::into),
            felder: serde_json::json!({ "name": "Einkauf", "position": 2 }),
            geaendert_at: String::new(),
        }
    }

    #[test]
    fn eine_sache_kommt_mit_ihren_feldern_zurueck() {
        let s = speicher();
        s.projektsache_schreiben(&sache("p1", "board", "b1", None), Protokoll::Merken)
            .unwrap();

        let zurueck = s.projektsache("p1", "b1").unwrap().unwrap();

        assert_eq!(zurueck.art, "board");
        assert_eq!(zurueck.text("name"), "Einkauf");
        assert_eq!(zurueck.zahl("position"), 2);
        assert!(!zurueck.geaendert_at.is_empty(), "ein Zeitpunkt entsteht");
    }

    /// DER PERSOENLICHE LAUF SIEHT KEINE PROJEKTZEILEN.
    ///
    /// Ohne diese Trennung schoebe er Karten an openany.de -- unter einer
    /// Art, die der persoenliche Endpunkt gar nicht kennt.
    #[test]
    fn projektzeilen_bleiben_aus_dem_persoenlichen_strom() {
        let s = speicher();
        s.projektsache_schreiben(&sache("p1", "card", "k1", Some("s1")), Protokoll::Merken)
            .unwrap();

        let (persoenlich, _, _) = s.aenderungen_seit(0).unwrap();
        let (projekt, _, _) = s.projekt_aenderungen_seit("p1", 0).unwrap();

        assert!(persoenlich.is_empty(), "nichts fuer openany.de");
        assert_eq!(projekt.len(), 1);
        assert_eq!(projekt[0].art, "card");
    }

    /// Und andersherum: Eine Notiz gehoert nicht in den Strom eines Projekts.
    #[test]
    fn persoenliches_bleibt_aus_dem_projektstrom() {
        let s = speicher();
        s.notiz_schreiben(
            &crate::Notiz {
                zk_id: "n1".into(),
                titel: "Zettel".into(),
                ..Default::default()
            },
            Protokoll::Merken,
        )
        .unwrap();

        let (projekt, _, _) = s.projekt_aenderungen_seit("p1", 0).unwrap();

        assert!(projekt.is_empty());
    }

    /// Zwei Projekte sehen einander nicht -- auch nicht im Protokoll.
    #[test]
    fn projekte_bleiben_unter_sich() {
        let s = speicher();
        s.projektsache_schreiben(&sache("p1", "board", "b1", None), Protokoll::Merken)
            .unwrap();
        s.projektsache_schreiben(&sache("p2", "board", "b2", None), Protokoll::Merken)
            .unwrap();

        assert_eq!(s.projektsachen("p1", "board").unwrap().len(), 1);
        assert_eq!(s.projekt_aenderungen_seit("p2", 0).unwrap().0.len(), 1);
    }

    /// DER BEHAELTER NIMMT SEINEN INHALT MIT -- mit EINEM Grabstein.
    #[test]
    fn ein_board_raeumt_seine_spalten_und_karten_weg() {
        let s = speicher();
        s.projektsache_schreiben(&sache("p1", "board", "b1", None), Protokoll::Still)
            .unwrap();
        s.projektsache_schreiben(&sache("p1", "column", "s1", Some("b1")), Protokoll::Still)
            .unwrap();
        s.projektsache_schreiben(&sache("p1", "card", "k1", Some("s1")), Protokoll::Still)
            .unwrap();

        assert!(s
            .projektsache_entfernen("p1", "b1", Protokoll::Merken)
            .unwrap());

        assert!(s.projektsache("p1", "s1").unwrap().is_none());
        assert!(s.projektsache("p1", "k1").unwrap().is_none());

        let (eintraege, _, _) = s.projekt_aenderungen_seit("p1", 0).unwrap();
        assert_eq!(eintraege.len(), 1, "ein Grabstein, nicht drei");
        assert_eq!(eintraege[0].was, Was::Fort);
    }

    #[test]
    fn kinder_finden_ihren_behaelter() {
        let s = speicher();
        s.projektsache_schreiben(&sache("p1", "column", "s1", Some("b1")), Protokoll::Still)
            .unwrap();
        s.projektsache_schreiben(&sache("p1", "column", "s2", Some("b1")), Protokoll::Still)
            .unwrap();
        s.projektsache_schreiben(&sache("p1", "column", "s3", Some("b2")), Protokoll::Still)
            .unwrap();

        assert_eq!(s.projektsachen_unter("p1", "b1").unwrap().len(), 2);
    }

    /// Je Sache nur der letzte Stand -- zwanzigmal verschoben ist einmal.
    #[test]
    fn je_sache_nur_der_letzte_stand() {
        let s = speicher();
        for i in 1..=5 {
            let mut k = sache("p1", "card", "k1", Some("s1"));
            k.felder = serde_json::json!({ "position": i });
            s.projektsache_schreiben(&k, Protokoll::Merken).unwrap();
        }

        let (eintraege, marke, mehr) = s.projekt_aenderungen_seit("p1", 0).unwrap();

        assert_eq!(eintraege.len(), 1);
        assert!(marke >= 5, "die Marke wandert ueber alle Zeilen");
        assert!(!mehr);
    }

    /// Das Ende einer Mitgliedschaft nimmt alles mit -- samt Marke.
    #[test]
    fn ein_vergessenes_projekt_laesst_nichts_stehen() {
        let s = speicher();
        s.projekt_schreiben(&Projekt {
            uuid: "p1".into(),
            name: "Haushalt".into(),
            rolle: "owner".into(),
            geaendert_at: String::new(),
            mitgliederliste: None,
        })
        .unwrap();
        s.projektsache_schreiben(&sache("p1", "board", "b1", None), Protokoll::Merken)
            .unwrap();
        let gegenstelle = Projekt::gegenstelle("https://openany.de", "p1");
        s.fremde_marke_setzen(&gegenstelle, 42).unwrap();

        assert!(s.projekt_vergessen("https://openany.de", "p1").unwrap());

        assert!(s.projekt("p1").unwrap().is_none());
        assert!(s.projektsachen("p1", "board").unwrap().is_empty());
        assert_eq!(s.projekt_aenderungen_seit("p1", 0).unwrap().0.len(), 0);
        assert_eq!(s.marke(&gegenstelle).unwrap().fremde, 0);
    }

    /// Der Name der Gegenstelle traegt die Basis -- sonst waere dasselbe
    /// Projekt auf zwei Instanzen dieselbe Gegenstelle.
    #[test]
    fn die_gegenstelle_heisst_nach_basis_und_projekt() {
        assert_eq!(
            Projekt::gegenstelle("https://openany.de/", "p1"),
            "https://openany.de#projekt:p1"
        );
        assert_ne!(
            Projekt::gegenstelle("https://openany.de", "p1"),
            Projekt::gegenstelle("https://anders.example", "p1")
        );
    }

    #[test]
    fn ein_lokales_projekt_behaelt_seine_liste_auch_wenn_der_server_schreibt() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        let p = Projekt {
            uuid: "lokal1".into(),
            name: "Garten".into(),
            rolle: "owner".into(),
            geaendert_at: "2026-10-01T10:00:00Z".into(),
            mitgliederliste: None,
        };
        s.projekt_lokal_schreiben(&p, "{\"projekt\":\"lokal1\"}")
            .unwrap();
        // Ein Server, der zufaellig dieselbe uuid meldete, aendert die Liste nicht.
        s.projekt_schreiben(&Projekt {
            name: "Garten 2".into(),
            ..p.clone()
        })
        .unwrap();
        let wieder = s.projekt("lokal1").unwrap().unwrap();
        assert_eq!(
            wieder.mitgliederliste.as_deref(),
            Some("{\"projekt\":\"lokal1\"}")
        );
        assert_eq!(
            s.projekte().unwrap()[0].mitgliederliste,
            wieder.mitgliederliste
        );
    }

    #[test]
    fn ein_grabstein_bleibt_bis_die_sache_wiederkommt() {
        let s = speicher();
        s.projektsache_schreiben(&sache("p1", "board", "b1", None), Protokoll::Merken)
            .unwrap();
        s.projektsache_schreiben(&sache("p1", "column", "c1", Some("b1")), Protokoll::Merken)
            .unwrap();
        assert!(s.projekt_grabsteine("p1").unwrap().is_empty());

        s.projektsache_entfernen("p1", "b1", Protokoll::Merken)
            .unwrap();
        let steine = s.projekt_grabsteine("p1").unwrap();
        // Einer fuer den Behaelter -- die Spalte geht still mit.
        assert_eq!(steine.len(), 1);
        assert_eq!(
            (steine[0].0.as_str(), steine[0].1.as_str()),
            ("board", "b1")
        );
        assert!(s.projekt_grabsteine("p2").unwrap().is_empty());

        s.projektsache_schreiben(&sache("p1", "board", "b1", None), Protokoll::Merken)
            .unwrap();
        assert!(s.projekt_grabsteine("p1").unwrap().is_empty());
    }
}
