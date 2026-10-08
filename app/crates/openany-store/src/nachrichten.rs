//! Nachrichten -- der Verlauf auf DIESEM Geraet.
//!
//! Sie kommen vom Homeserver (Matrix), nicht ueber den Abgleich, und gehen
//! auch nicht hinein: kein Protokoll, keine Marke. Siehe Schema-Stand 11;
//! seit Stand 16 aus mehreren Konten, jede Zeile mit ihrem.

use crate::verlaufsfilter::{Spalten, Verlaufsfilter};
use crate::{Ergebnis, Speicher};
use rusqlite::{params, Row};

/// Wie viele Nachrichten eine Seite des Verlaufs traegt.
pub const NACHRICHTEN_SEITE: usize = 50;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Nachricht {
    /// Das eigene Konto (Matrix-Kennung), über das sie kam oder ging.
    pub konto: String,
    pub event_id: String,
    pub raum: String,
    /// Mit wem gesprochen wird -- bei einer eigenen Nachricht der Empfaenger,
    /// bei einer fremden der Absender. `None`, wenn der Raum keines nennt.
    pub gegenueber: Option<String>,
    pub absender: String,
    pub von_mir: bool,
    pub text: String,
    /// ISO-8601, vom Homeserver.
    pub zeit: String,
    pub gelesen_at: Option<String>,
    /// Der Anhang als JSON (`{name, mime, groesse, quelle}`), `None` bei
    /// reinem Text. Was darin steht, weiß die App, nicht der Speicher.
    pub anhang: Option<String>,
}

const SPALTEN: &str =
    "event_id, raum, gegenueber, absender, von_mir, text, zeit, gelesen_at, anhang, konto";

const SPALTEN_FILTER: Spalten = Spalten {
    gegenueber: "gegenueber",
    anhang: "anhang IS NOT NULL",
    suchfelder: &["text"],
    konto: Some("konto"),
};

fn aus(z: &Row<'_>) -> rusqlite::Result<Nachricht> {
    Ok(Nachricht {
        event_id: z.get(0)?,
        raum: z.get(1)?,
        gegenueber: z.get(2)?,
        absender: z.get(3)?,
        von_mir: z.get::<_, i64>(4)? != 0,
        text: z.get(5)?,
        zeit: z.get(6)?,
        gelesen_at: z.get(7)?,
        anhang: z.get(8)?,
        konto: z.get(9)?,
    })
}

fn jetzt() -> String {
    chrono::Utc::now().to_rfc3339()
}

impl Speicher {
    /// Eine Nachricht ablegen. `true`, wenn sie neu war oder ihr Raum
    /// nachgetragen wurde.
    ///
    /// Eine schon bekannte Ereignis-Id aendert nichts -- auch nicht, wenn sie
    /// hier geloescht wurde: Das Echo einer geloeschten Nachricht soll sie
    /// nicht wieder hervorholen.
    pub fn nachricht_ablegen(&self, n: &Nachricht) -> Ergebnis<bool> {
        let neu = self.db().execute(
            // Der Raum darf nachgetragen werden: Beim Senden kennt die App
            // ihn noch nicht, das Echo nennt ihn. Ebenso die Quelle eines
            // selbst gesendeten Anhangs -- erst das Echo sagt, wo er beim
            // Homeserver liegt. Name, Größe und der lokale Abdruck bleiben,
            // wie sie beim Senden standen. Sonst bleibt alles, wie es zuerst
            // ankam.
            &format!(
                "INSERT INTO nachrichten ({SPALTEN}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                 ON CONFLICT (konto, event_id) DO UPDATE SET
                    raum = CASE WHEN nachrichten.raum = '' THEN excluded.raum ELSE nachrichten.raum END,
                    anhang = CASE
                        WHEN nachrichten.anhang IS NULL THEN excluded.anhang
                        WHEN json_extract(nachrichten.anhang, '$.quelle') IS NULL
                            THEN json_set(nachrichten.anhang, '$.quelle',
                                          json_extract(excluded.anhang, '$.quelle'))
                        ELSE nachrichten.anhang END
                 WHERE (nachrichten.raum = '' AND excluded.raum <> '')
                    OR (excluded.anhang IS NOT NULL AND (
                        nachrichten.anhang IS NULL
                        OR (json_extract(nachrichten.anhang, '$.quelle') IS NULL
                            AND json_extract(excluded.anhang, '$.quelle') IS NOT NULL)))"
            ),
            params![
                n.event_id,
                n.raum,
                n.gegenueber,
                n.absender,
                n.von_mir as i64,
                n.text,
                n.zeit,
                n.gelesen_at,
                n.anhang,
                n.konto
            ],
        )?;
        Ok(neu > 0)
    }

    /// Eine Seite des Verlaufs, neueste zuerst, ohne Geloeschte.
    pub fn nachrichten(&self, seite: usize) -> Ergebnis<(Vec<Nachricht>, bool)> {
        self.nachrichten_gefiltert(seite, &Verlaufsfilter::default())
    }

    /// Dasselbe, eingegrenzt (Filter, Suche, eine Unterhaltung).
    pub fn nachrichten_gefiltert(
        &self,
        seite: usize,
        filter: &Verlaufsfilter,
    ) -> Ergebnis<(Vec<Nachricht>, bool)> {
        let (bedingung, mut werte) = filter.sql(&SPALTEN_FILTER);
        werte.push(((NACHRICHTEN_SEITE + 1) as i64).into());
        werte.push(((seite.saturating_sub(1) * NACHRICHTEN_SEITE) as i64).into());
        let db = self.db();
        let mut abfrage = db.prepare(&format!(
            "SELECT {SPALTEN} FROM nachrichten WHERE geloescht_at IS NULL{bedingung}
             ORDER BY zeit DESC, event_id DESC LIMIT ? OFFSET ?"
        ))?;
        let mut liste: Vec<Nachricht> = abfrage
            .query_map(rusqlite::params_from_iter(werte), aus)?
            .collect::<Result<_, _>>()?;
        let mehr = liste.len() > NACHRICHTEN_SEITE;
        liste.truncate(NACHRICHTEN_SEITE);
        Ok((liste, mehr))
    }

    /// Je Gegenüber die jüngste Nachricht und wie viele offen sind -- für
    /// „Nach Kontakt". Dieselben Filter wie der Verlauf.
    pub fn nachrichten_unterhaltungen(
        &self,
        filter: &Verlaufsfilter,
    ) -> Ergebnis<Vec<(Nachricht, usize)>> {
        let (bedingung, werte) = filter.sql(&SPALTEN_FILTER);
        let db = self.db();
        // SQLite nimmt die übrigen Spalten aus der Zeile, die MAX() trifft --
        // so kommt die jüngste Nachricht je Gegenüber in einem Gang.
        let mut abfrage = db.prepare(&format!(
            "SELECT {SPALTEN}, MAX(zeit), SUM(von_mir = 0 AND gelesen_at IS NULL)
             FROM nachrichten WHERE geloescht_at IS NULL AND gegenueber IS NOT NULL{bedingung}
             GROUP BY gegenueber"
        ))?;
        let liste = abfrage
            .query_map(rusqlite::params_from_iter(werte), |z| {
                Ok((aus(z)?, z.get::<_, i64>(11)? as usize))
            })?
            .collect::<Result<_, _>>()?;
        Ok(liste)
    }

    /// Eine einzelne Nachricht, auch eine geloeschte -- fuer ihren Anhang.
    pub fn nachricht(&self, konto: &str, event_id: &str) -> Ergebnis<Option<Nachricht>> {
        use rusqlite::OptionalExtension;
        Ok(self
            .db()
            .query_row(
                &format!("SELECT {SPALTEN} FROM nachrichten WHERE konto = ?1 AND event_id = ?2"),
                params![konto, event_id],
                aus,
            )
            .optional()?)
    }

    pub fn nachricht_gelesen(&self, konto: &str, event_id: &str) -> Ergebnis<bool> {
        Ok(self.db().execute(
            "UPDATE nachrichten SET gelesen_at = ?3
             WHERE konto = ?1 AND event_id = ?2 AND gelesen_at IS NULL",
            params![konto, event_id, jetzt()],
        )? > 0)
    }

    /// Nur auf diesem Geraet -- drueben im Raum bleibt sie stehen.
    pub fn nachricht_loeschen(&self, konto: &str, event_id: &str) -> Ergebnis<bool> {
        Ok(self.db().execute(
            "UPDATE nachrichten SET geloescht_at = ?3
             WHERE konto = ?1 AND event_id = ?2 AND geloescht_at IS NULL",
            params![konto, event_id, jetzt()],
        )? > 0)
    }

    pub fn ungelesene_nachrichten(&self) -> Ergebnis<usize> {
        Ok(self.db().query_row(
            "SELECT COUNT(*) FROM nachrichten
             WHERE von_mir = 0 AND gelesen_at IS NULL AND geloescht_at IS NULL",
            [],
            |z| z.get::<_, i64>(0),
        )? as usize)
    }

    /// Beim Trennen eines Kontos: Sein Verlauf gehoert zu ihm.
    pub fn nachrichten_vergessen(&self, konto: &str) -> Ergebnis<()> {
        self.db()
            .execute("DELETE FROM nachrichten WHERE konto = ?1", params![konto])?;
        Ok(())
    }

    /// Einmal nach Stand 16: was noch keinem Konto gehört, dem bisher
    /// einzigen zuordnen.
    pub fn nachrichten_zuordnen(&self, konto: &str) -> Ergebnis<()> {
        self.db().execute(
            "UPDATE nachrichten SET konto = ?1 WHERE konto = ''",
            params![konto],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(id: &str, zeit: &str, von_mir: bool) -> Nachricht {
        Nachricht {
            konto: "@tiffy:m.org".into(),
            event_id: id.into(),
            raum: "!r:m.org".into(),
            gegenueber: Some("@ferdinand:m.org".into()),
            absender: if von_mir {
                "@tiffy:m.org"
            } else {
                "@ferdinand:m.org"
            }
            .into(),
            von_mir,
            text: format!("Text {id}"),
            zeit: zeit.into(),
            gelesen_at: None,
            anhang: None,
        }
    }

    #[test]
    fn filter_suche_und_unterhaltungen() {
        use crate::Verlaufsfilter;
        let s = Speicher::im_arbeitsspeicher().unwrap();
        s.nachricht_ablegen(&n("$1", "2026-09-28T10:00:00Z", false))
            .unwrap();
        let mut zwei = n("$2", "2026-09-28T11:00:00Z", true);
        zwei.text = "Treffen am Kanal".into();
        s.nachricht_ablegen(&zwei).unwrap();
        let mut drei = n("$3", "2026-09-28T12:00:00Z", false);
        drei.gegenueber = Some("@greta:m.org".into());
        drei.anhang = Some(r#"{"name":"a.pdf"}"#.into());
        s.nachricht_ablegen(&drei).unwrap();

        let ids = |f: &Verlaufsfilter| -> Vec<String> {
            s.nachrichten_gefiltert(1, f)
                .unwrap()
                .0
                .into_iter()
                .map(|n| n.event_id)
                .collect()
        };
        assert_eq!(ids(&Verlaufsfilter::default()), ["$3", "$2", "$1"]);
        let f = Verlaufsfilter {
            ungelesen: true,
            ..Default::default()
        };
        assert_eq!(ids(&f), ["$3", "$1"]);
        let f = Verlaufsfilter {
            anhang: true,
            ..Default::default()
        };
        assert_eq!(ids(&f), ["$3"]);
        let f = Verlaufsfilter {
            suche: "KANAL".into(),
            ..Default::default()
        };
        assert_eq!(ids(&f), ["$2"]);
        let f = Verlaufsfilter {
            suche: "greta".into(),
            ..Default::default()
        };
        assert_eq!(ids(&f), ["$3"]);
        // Ein Name aus dem Adressbuch trifft die Kennung dahinter.
        let f = Verlaufsfilter {
            suche: "Ferdi F.".into(),
            suche_gegenueber: vec!["@ferdinand:m.org".into()],
            ..Default::default()
        };
        assert_eq!(ids(&f), ["$2", "$1"]);
        let f = Verlaufsfilter {
            mit: vec!["@ferdinand:m.org".into()],
            ungelesen: true,
            ..Default::default()
        };
        assert_eq!(ids(&f), ["$1"]);

        let mut u = s
            .nachrichten_unterhaltungen(&Verlaufsfilter::default())
            .unwrap();
        u.sort_by(|a, b| a.0.event_id.cmp(&b.0.event_id));
        assert_eq!(u.len(), 2);
        assert_eq!((u[0].0.event_id.as_str(), u[0].1), ("$2", 1));
        assert_eq!((u[1].0.event_id.as_str(), u[1].1), ("$3", 1));
    }

    #[test]
    fn das_echo_traegt_die_quelle_des_anhangs_nach() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        let mut gesendet = n("$1", "2026-09-28T10:00:00Z", true);
        gesendet.raum = String::new();
        gesendet.anhang = Some(r#"{"name":"a.pdf","abdruck":"abc"}"#.into());
        s.nachricht_ablegen(&gesendet).unwrap();

        let mut echo = n("$1", "2026-09-28T10:00:00Z", true);
        echo.anhang = Some(r#"{"name":"a.pdf","quelle":"mxc-quelle"}"#.into());
        assert!(s.nachricht_ablegen(&echo).unwrap());
        let zeile = &s.nachrichten(1).unwrap().0[0];
        assert_eq!(zeile.raum, "!r:m.org");
        let anhang: serde_json::Value =
            serde_json::from_str(zeile.anhang.as_deref().unwrap()).unwrap();
        assert_eq!(anhang["abdruck"], "abc");
        assert_eq!(anhang["quelle"], "mxc-quelle");

        // Ein zweites Echo ändert nichts mehr.
        let mut anders = echo.clone();
        anders.anhang = Some(r#"{"name":"b.pdf","quelle":"andere"}"#.into());
        assert!(!s.nachricht_ablegen(&anders).unwrap());
        assert!(s.nachrichten(1).unwrap().0[0]
            .anhang
            .as_deref()
            .unwrap()
            .contains("mxc-quelle"));
    }

    #[test]
    fn das_echo_wird_keine_zweite_zeile() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        assert!(s
            .nachricht_ablegen(&n("$1", "2026-09-17T10:00:00Z", true))
            .unwrap());
        assert!(!s
            .nachricht_ablegen(&n("$1", "2026-09-17T10:00:01Z", true))
            .unwrap());
        assert_eq!(s.nachrichten(1).unwrap().0.len(), 1);
    }

    #[test]
    fn das_echo_traegt_den_raum_nach() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        let mut gesendet = n("$1", "2026-09-17T10:00:00Z", true);
        gesendet.raum = String::new();
        s.nachricht_ablegen(&gesendet).unwrap();

        s.nachricht_ablegen(&n("$1", "2026-09-17T10:00:00Z", true))
            .unwrap();
        assert_eq!(s.nachrichten(1).unwrap().0[0].raum, "!r:m.org");
    }

    #[test]
    fn neueste_zuerst_und_geloeschte_fehlen() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        s.nachricht_ablegen(&n("$1", "2026-09-17T10:00:00Z", false))
            .unwrap();
        s.nachricht_ablegen(&n("$2", "2026-09-17T11:00:00Z", true))
            .unwrap();
        s.nachricht_ablegen(&n("$3", "2026-09-17T12:00:00Z", false))
            .unwrap();
        assert!(s.nachricht_loeschen("@tiffy:m.org", "$3").unwrap());

        let (liste, mehr) = s.nachrichten(1).unwrap();
        assert_eq!(
            liste
                .iter()
                .map(|m| m.event_id.as_str())
                .collect::<Vec<_>>(),
            ["$2", "$1"]
        );
        assert!(!mehr);

        // Das Echo der geloeschten holt sie nicht zurueck.
        s.nachricht_ablegen(&n("$3", "2026-09-17T12:00:00Z", false))
            .unwrap();
        assert_eq!(s.nachrichten(1).unwrap().0.len(), 2);
    }

    #[test]
    fn nur_fremde_ungelesene_zaehlen() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        s.nachricht_ablegen(&n("$1", "2026-09-17T10:00:00Z", false))
            .unwrap();
        s.nachricht_ablegen(&n("$2", "2026-09-17T11:00:00Z", true))
            .unwrap();
        assert_eq!(s.ungelesene_nachrichten().unwrap(), 1);
        assert!(s.nachricht_gelesen("@tiffy:m.org", "$1").unwrap());
        assert!(!s.nachricht_gelesen("@tiffy:m.org", "$1").unwrap());
        assert_eq!(s.ungelesene_nachrichten().unwrap(), 0);
    }

    #[test]
    fn seiten_sagen_ob_es_mehr_gibt() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        for i in 0..(NACHRICHTEN_SEITE + 3) {
            s.nachricht_ablegen(&n(
                &format!("${i:03}"),
                &format!("2026-09-17T10:{:02}:{:02}Z", i / 60, i % 60),
                false,
            ))
            .unwrap();
        }
        let (erste, mehr) = s.nachrichten(1).unwrap();
        assert_eq!(erste.len(), NACHRICHTEN_SEITE);
        assert!(mehr);
        let (zweite, mehr) = s.nachrichten(2).unwrap();
        assert_eq!(zweite.len(), 3);
        assert!(!mehr);
    }

    #[test]
    fn ein_gesendeter_anhang_gilt_als_benutzt() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        let mut gesendet = n("$1", "2026-09-28T10:00:00Z", true);
        gesendet.anhang = Some(r#"{"name":"a.pdf","abdruck":"abc123"}"#.into());
        s.nachricht_ablegen(&gesendet).unwrap();
        assert!(s.abdruck_benutzt("abc123").unwrap());
        assert!(s.benutzte_abdruecke().unwrap().contains("abc123"));

        s.nachrichten_vergessen("@tiffy:m.org").unwrap();
        assert!(!s.abdruck_benutzt("abc123").unwrap());
    }

    #[test]
    fn dasselbe_ereignis_bei_zwei_konten() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        let gesendet = n("$1", "2026-09-29T10:00:00Z", true);
        let mut empfangen = n("$1", "2026-09-29T10:00:00Z", false);
        empfangen.konto = "@ferdinand:m.org".into();
        assert!(s.nachricht_ablegen(&gesendet).unwrap());
        assert!(s.nachricht_ablegen(&empfangen).unwrap());
        assert_eq!(s.nachrichten(1).unwrap().0.len(), 2);
        assert!(s.nachricht_gelesen("@ferdinand:m.org", "$1").unwrap());
        s.nachrichten_vergessen("@ferdinand:m.org").unwrap();
        assert_eq!(s.nachrichten(1).unwrap().0.len(), 1);
    }

    #[test]
    fn alte_zeilen_werden_zugeordnet() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        let mut alt = n("$1", "2026-09-29T10:00:00Z", false);
        alt.konto = String::new();
        s.nachricht_ablegen(&alt).unwrap();
        s.nachrichten_zuordnen("@tiffy:m.org").unwrap();
        assert!(s.nachricht("@tiffy:m.org", "$1").unwrap().is_some());
    }
}
