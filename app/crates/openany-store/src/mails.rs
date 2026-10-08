//! Mails -- der E-Mail-Verlauf auf DIESEM Gerät (Schema-Stand 14,
//! docs/plan-email-pgp.md, Schritt 2; seit Stand 15 aus mehreren
//! Postfächern).
//!
//! Wie die Matrix-Nachrichten: vom Mailserver, nicht über den Abgleich, und
//! auch nicht hinein. Das Postfach ist der Abgleich.
//!
//! **Eine Mail ist einmal da, auch wenn sie zweimal kommt.** Eine selbst
//! gesendete legt die App gleich ab; später kommt sie aus „Gesendet" noch
//! einmal (mit Ordner und UID). Dann wird die Zeile ergänzt, nicht verdoppelt
//! -- erkannt an der Message-ID.

use crate::verlaufsfilter::{Spalten, Verlaufsfilter};
use crate::{Ergebnis, Speicher};
use rusqlite::{params, OptionalExtension, Row};

/// Wie viele Mails eine Seite des Verlaufs trägt.
pub const MAILS_SEITE: usize = 50;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Mail {
    /// Eigene Kennung (uuid) -- Message-IDs sind nicht verlässlich eindeutig.
    pub id: String,
    /// Das Postfach (seine Adresse, klein), aus dem sie kam oder über das
    /// sie ging.
    pub postfach: String,
    /// Wo sie auf dem Server liegt; leer, solange eine gesendete noch nicht
    /// aus „Gesendet" zurückkam.
    pub ordner: String,
    pub uidvalidity: i64,
    pub uid: i64,
    pub message_id: String,
    pub in_reply_to: Option<String>,
    /// Die Kette davor, als JSON-Liste.
    pub references: String,
    pub von_mir: bool,
    /// Die Adresse des Gegenübers (klein geschrieben): bei einer eigenen der
    /// erste Empfänger, sonst der Absender.
    pub gegenueber: String,
    /// Der Name, wie ihn die Mail selbst nennt (darf leer sein).
    pub gegenueber_name: String,
    pub betreff: String,
    pub text: String,
    /// ISO-8601.
    pub zeit: String,
    pub gelesen_at: Option<String>,
    /// Anhänge als JSON-Liste `[{name, mime, groesse, abdruck}]`.
    pub anhaenge: String,
    /// `verschluesselt`, `unlesbar` (Schlüssel fehlt; die Nachricht steht
    /// dann in `text`) oder `None`.
    pub pgp: Option<String>,
    /// `gueltig`, `ungueltig`, `unbekannt` oder `None`.
    pub signatur: Option<String>,
}

const SPALTEN: &str =
    "id, ordner, uidvalidity, uid, message_id, in_reply_to, referenzen, von_mir, \
gegenueber, gegenueber_name, betreff, text, zeit, gelesen_at, anhaenge, postfach, pgp, signatur";

// Eine unlesbare Mail trägt in `text` die verschlüsselte Nachricht -- darin
// zu suchen träfe Zufall. Betreff und Absender bleiben durchsuchbar.
const SPALTEN_FILTER: Spalten = Spalten {
    gegenueber: "gegenueber",
    anhang: "anhaenge != '[]'",
    suchfelder: &[
        "gegenueber_name",
        "betreff",
        "CASE WHEN pgp = 'unlesbar' THEN '' ELSE text END",
    ],
    konto: None,
};

fn aus(z: &Row<'_>) -> rusqlite::Result<Mail> {
    Ok(Mail {
        id: z.get(0)?,
        ordner: z.get(1)?,
        uidvalidity: z.get(2)?,
        uid: z.get(3)?,
        message_id: z.get(4)?,
        in_reply_to: z.get(5)?,
        references: z.get(6)?,
        von_mir: z.get::<_, i64>(7)? != 0,
        gegenueber: z.get(8)?,
        gegenueber_name: z.get(9)?,
        betreff: z.get(10)?,
        text: z.get(11)?,
        zeit: z.get(12)?,
        gelesen_at: z.get(13)?,
        anhaenge: z.get(14)?,
        postfach: z.get(15)?,
        pgp: z.get(16)?,
        signatur: z.get(17)?,
    })
}

fn jetzt() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Wie weit ein Ordner auf dem Server gelesen ist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailStand {
    pub postfach: String,
    pub ordner: String,
    pub uidvalidity: i64,
    pub letzte_uid: i64,
}

impl Speicher {
    /// Eine Mail ablegen. `true`, wenn sie neu war.
    ///
    /// Gibt es im selben Postfach schon eine Zeile mit derselben Message-ID
    /// und derselben Richtung, wird sie nur um Ordner und UID ergänzt (die eigene, die aus
    /// „Gesendet" zurückkommt). Schon Bekanntes an derselben Stelle auf dem
    /// Server ändert nichts -- auch nicht, wenn es hier gelöscht wurde.
    pub fn mail_ablegen(&self, m: &Mail) -> Ergebnis<bool> {
        let db = self.db();
        if !m.ordner.is_empty() {
            let da: bool = db.query_row(
                "SELECT EXISTS (SELECT 1 FROM mails
                 WHERE postfach = ?4 AND ordner = ?1 AND uidvalidity = ?2 AND uid = ?3)",
                params![m.ordner, m.uidvalidity, m.uid, m.postfach],
                |z| z.get::<_, i64>(0),
            )? != 0;
            if da {
                return Ok(false);
            }
        }
        if !m.message_id.is_empty() {
            let ergaenzt = db.execute(
                "UPDATE mails SET ordner = ?2, uidvalidity = ?3, uid = ?4
                 WHERE message_id = ?1 AND von_mir = ?5 AND postfach = ?6 AND ordner = ''",
                params![
                    m.message_id,
                    m.ordner,
                    m.uidvalidity,
                    m.uid,
                    m.von_mir as i64,
                    m.postfach
                ],
            )?;
            if ergaenzt > 0 {
                return Ok(false);
            }
            let schon: bool = db.query_row(
                "SELECT EXISTS (SELECT 1 FROM mails
                 WHERE message_id = ?1 AND von_mir = ?2 AND postfach = ?3)",
                params![m.message_id, m.von_mir as i64, m.postfach],
                |z| z.get::<_, i64>(0),
            )? != 0;
            if schon {
                return Ok(false);
            }
        }
        db.execute(
            &format!(
                "INSERT INTO mails ({SPALTEN}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)"
            ),
            params![
                m.id,
                m.ordner,
                m.uidvalidity,
                m.uid,
                m.message_id,
                m.in_reply_to,
                m.references,
                m.von_mir as i64,
                m.gegenueber,
                m.gegenueber_name,
                m.betreff,
                m.text,
                m.zeit,
                m.gelesen_at,
                m.anhaenge,
                m.postfach,
                m.pgp,
                m.signatur
            ],
        )?;
        Ok(true)
    }

    /// Eine Seite des Verlaufs, neueste zuerst, ohne Gelöschte.
    pub fn mails(&self, seite: usize) -> Ergebnis<(Vec<Mail>, bool)> {
        self.mails_gefiltert(seite, &Verlaufsfilter::default())
    }

    /// Dasselbe, eingegrenzt (Filter, Suche, eine Unterhaltung).
    pub fn mails_gefiltert(
        &self,
        seite: usize,
        filter: &Verlaufsfilter,
    ) -> Ergebnis<(Vec<Mail>, bool)> {
        let (bedingung, mut werte) = filter.sql(&SPALTEN_FILTER);
        werte.push(((MAILS_SEITE + 1) as i64).into());
        werte.push(((seite.saturating_sub(1) * MAILS_SEITE) as i64).into());
        let db = self.db();
        let mut abfrage = db.prepare(&format!(
            "SELECT {SPALTEN} FROM mails WHERE geloescht_at IS NULL{bedingung}
             ORDER BY zeit DESC, id DESC LIMIT ? OFFSET ?"
        ))?;
        let mut liste: Vec<Mail> = abfrage
            .query_map(rusqlite::params_from_iter(werte), aus)?
            .collect::<Result<_, _>>()?;
        let mehr = liste.len() > MAILS_SEITE;
        liste.truncate(MAILS_SEITE);
        Ok((liste, mehr))
    }

    /// Je Gegenüber die jüngste Mail und wie viele offen sind.
    pub fn mails_unterhaltungen(&self, filter: &Verlaufsfilter) -> Ergebnis<Vec<(Mail, usize)>> {
        let (bedingung, werte) = filter.sql(&SPALTEN_FILTER);
        let db = self.db();
        let mut abfrage = db.prepare(&format!(
            "SELECT {SPALTEN}, MAX(zeit), SUM(von_mir = 0 AND gelesen_at IS NULL)
             FROM mails WHERE geloescht_at IS NULL AND gegenueber != ''{bedingung}
             GROUP BY gegenueber"
        ))?;
        let liste = abfrage
            .query_map(rusqlite::params_from_iter(werte), |z| {
                Ok((aus(z)?, z.get::<_, i64>(19)? as usize))
            })?
            .collect::<Result<_, _>>()?;
        Ok(liste)
    }

    pub fn mail(&self, id: &str) -> Ergebnis<Option<Mail>> {
        Ok(self
            .db()
            .query_row(
                &format!("SELECT {SPALTEN} FROM mails WHERE id = ?1"),
                params![id],
                aus,
            )
            .optional()?)
    }

    pub fn mail_gelesen(&self, id: &str) -> Ergebnis<bool> {
        Ok(self.db().execute(
            "UPDATE mails SET gelesen_at = ?2 WHERE id = ?1 AND gelesen_at IS NULL",
            params![id, jetzt()],
        )? > 0)
    }

    /// Nur hier ausblenden; ob sie auch auf dem Server gelöscht wird,
    /// entscheidet der Aufrufer.
    pub fn mail_loeschen(&self, id: &str) -> Ergebnis<bool> {
        Ok(self.db().execute(
            "UPDATE mails SET geloescht_at = ?2 WHERE id = ?1 AND geloescht_at IS NULL",
            params![id, jetzt()],
        )? > 0)
    }

    pub fn ungelesene_mails(&self) -> Ergebnis<usize> {
        Ok(self.db().query_row(
            "SELECT COUNT(*) FROM mails WHERE von_mir = 0 AND gelesen_at IS NULL AND geloescht_at IS NULL",
            [],
            |z| z.get::<_, i64>(0),
        )? as usize)
    }

    pub fn mail_staende(&self, postfach: &str) -> Ergebnis<Vec<MailStand>> {
        let db = self.db();
        let mut a = db.prepare(
            "SELECT postfach, ordner, uidvalidity, letzte_uid FROM mail_staende WHERE postfach = ?1",
        )?;
        let liste = a
            .query_map(params![postfach], |z| {
                Ok(MailStand {
                    postfach: z.get(0)?,
                    ordner: z.get(1)?,
                    uidvalidity: z.get(2)?,
                    letzte_uid: z.get(3)?,
                })
            })?
            .collect::<Result<_, _>>()?;
        Ok(liste)
    }

    pub fn mail_stand_setzen(&self, s: &MailStand) -> Ergebnis<()> {
        self.db().execute(
            "INSERT INTO mail_staende (postfach, ordner, uidvalidity, letzte_uid) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT (postfach, ordner) DO UPDATE
             SET uidvalidity = excluded.uidvalidity, letzte_uid = excluded.letzte_uid",
            params![s.postfach, s.ordner, s.uidvalidity, s.letzte_uid],
        )?;
        Ok(())
    }

    /// Beim Trennen eines Postfachs: sein Verlauf und seine Stände gehören
    /// zu ihm.
    pub fn mails_vergessen(&self, postfach: &str) -> Ergebnis<()> {
        let db = self.db();
        db.execute("DELETE FROM mails WHERE postfach = ?1", params![postfach])?;
        db.execute(
            "DELETE FROM mail_staende WHERE postfach = ?1",
            params![postfach],
        )?;
        Ok(())
    }

    /// Die Mails eines Postfachs, die noch auf ihren Schlüssel warten.
    pub fn unlesbare_mails(&self, postfach: &str) -> Ergebnis<Vec<Mail>> {
        let db = self.db();
        let mut a = db.prepare(&format!(
            "SELECT {SPALTEN} FROM mails WHERE postfach = ?1 AND pgp = 'unlesbar'"
        ))?;
        let liste = a
            .query_map(params![postfach], aus)?
            .collect::<Result<_, _>>()?;
        Ok(liste)
    }

    /// Eine unlesbare Mail ist entschlüsselt: Inhalt und Kennzeichen setzen.
    pub fn mail_entschluesselt(
        &self,
        id: &str,
        betreff: &str,
        text: &str,
        anhaenge: &str,
        signatur: Option<&str>,
    ) -> Ergebnis<()> {
        self.db().execute(
            "UPDATE mails SET betreff = ?2, text = ?3, anhaenge = ?4, pgp = 'verschluesselt',
                signatur = ?5 WHERE id = ?1",
            params![id, betreff, text, anhaenge, signatur],
        )?;
        Ok(())
    }

    /// Einmal nach Stand 15: was noch keinem Postfach gehört, dem bisher
    /// einzigen zuordnen.
    pub fn mails_zuordnen(&self, postfach: &str) -> Ergebnis<()> {
        let db = self.db();
        db.execute(
            "UPDATE mails SET postfach = ?1 WHERE postfach = ''",
            params![postfach],
        )?;
        db.execute(
            "UPDATE mail_staende SET postfach = ?1 WHERE postfach = ''",
            params![postfach],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(id: &str, msgid: &str, ordner: &str, uid: i64, von_mir: bool) -> Mail {
        Mail {
            id: id.into(),
            postfach: "tiffy@beispiel.test".into(),
            ordner: ordner.into(),
            uidvalidity: 7,
            uid,
            message_id: msgid.into(),
            references: "[]".into(),
            von_mir,
            gegenueber: "ferdinand@beispiel.test".into(),
            betreff: "Hallo".into(),
            text: "Text".into(),
            zeit: format!("2026-09-29T10:00:{uid:02}Z"),
            anhaenge: "[]".into(),
            ..Default::default()
        }
    }

    #[test]
    fn filter_und_suche_ohne_unlesbares() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        let mut a = m("a", "a@t", "INBOX", 1, false);
        a.betreff = "Rechnung September".into();
        a.anhaenge = r#"[{"name":"r.pdf"}]"#.into();
        s.mail_ablegen(&a).unwrap();
        let mut b = m("b", "b@t", "INBOX", 2, false);
        b.pgp = Some("unlesbar".into());
        b.text = "-----BEGIN PGP MESSAGE----- rechnung".into();
        b.gelesen_at = Some("2026-09-29T11:00:00Z".into());
        s.mail_ablegen(&b).unwrap();
        let mut c = m("c", "c@t", "INBOX", 3, false);
        c.gegenueber = "greta@beispiel.test".into();
        c.gegenueber_name = "Greta G.".into();
        s.mail_ablegen(&c).unwrap();

        let ids = |f: &Verlaufsfilter| -> Vec<String> {
            s.mails_gefiltert(1, f)
                .unwrap()
                .0
                .into_iter()
                .map(|m| m.id)
                .collect()
        };
        let f = Verlaufsfilter {
            suche: "rechnung".into(),
            ..Default::default()
        };
        assert_eq!(ids(&f), ["a"]);
        let f = Verlaufsfilter {
            suche: "greta g".into(),
            ..Default::default()
        };
        assert_eq!(ids(&f), ["c"]);
        let f = Verlaufsfilter {
            anhang: true,
            ..Default::default()
        };
        assert_eq!(ids(&f), ["a"]);
        let f = Verlaufsfilter {
            ungelesen: true,
            ..Default::default()
        };
        assert_eq!(ids(&f), ["c", "a"]);

        let mut u = s.mails_unterhaltungen(&Verlaufsfilter::default()).unwrap();
        u.sort_by(|x, y| x.0.id.cmp(&y.0.id));
        assert_eq!(
            u.iter()
                .map(|(m, n)| (m.id.as_str(), *n))
                .collect::<Vec<_>>(),
            [("b", 1), ("c", 1)]
        );
    }

    #[test]
    fn dieselbe_stelle_auf_dem_server_nur_einmal() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        assert!(s.mail_ablegen(&m("a", "x@t", "INBOX", 1, false)).unwrap());
        assert!(!s.mail_ablegen(&m("b", "x@t", "INBOX", 1, false)).unwrap());
        assert_eq!(s.mails(1).unwrap().0.len(), 1);
    }

    #[test]
    fn die_gesendete_wird_aus_gesendet_nur_ergaenzt() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        s.mail_ablegen(&m("eigen", "neu@t", "", 0, true)).unwrap();
        assert!(!s
            .mail_ablegen(&m("anders", "neu@t", "Sent", 5, true))
            .unwrap());
        let zeile = s.mail("eigen").unwrap().unwrap();
        assert_eq!((zeile.ordner.as_str(), zeile.uid), ("Sent", 5));
        assert_eq!(s.mails(1).unwrap().0.len(), 1);
    }

    #[test]
    fn geloeschte_fehlen_und_kommen_nicht_zurueck() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        s.mail_ablegen(&m("a", "x@t", "INBOX", 1, false)).unwrap();
        assert_eq!(s.ungelesene_mails().unwrap(), 1);
        assert!(s.mail_loeschen("a").unwrap());
        assert!(!s.mail_ablegen(&m("a2", "x@t", "INBOX", 1, false)).unwrap());
        assert!(s.mails(1).unwrap().0.is_empty());
        assert_eq!(s.ungelesene_mails().unwrap(), 0);
    }

    #[test]
    fn staende_werden_ersetzt() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        s.mail_stand_setzen(&MailStand {
            postfach: "a@t".into(),
            ordner: "INBOX".into(),
            uidvalidity: 1,
            letzte_uid: 4,
        })
        .unwrap();
        s.mail_stand_setzen(&MailStand {
            postfach: "a@t".into(),
            ordner: "INBOX".into(),
            uidvalidity: 1,
            letzte_uid: 9,
        })
        .unwrap();
        assert_eq!(
            s.mail_staende("a@t").unwrap(),
            vec![MailStand {
                postfach: "a@t".into(),
                ordner: "INBOX".into(),
                uidvalidity: 1,
                letzte_uid: 9
            }]
        );
        assert!(s.mail_staende("b@t").unwrap().is_empty());
        s.mails_vergessen("a@t").unwrap();
        assert!(s.mail_staende("a@t").unwrap().is_empty());
    }

    #[test]
    fn postfaecher_sind_getrennt() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        let mut b = m("b", "x@t", "INBOX", 1, false);
        b.postfach = "zwei@t".into();
        assert!(s.mail_ablegen(&m("a", "x@t", "INBOX", 1, false)).unwrap());
        assert!(
            s.mail_ablegen(&b).unwrap(),
            "dieselbe Stelle in einem anderen Postfach"
        );
        s.mails_vergessen("zwei@t").unwrap();
        assert_eq!(s.mails(1).unwrap().0.len(), 1);
    }

    #[test]
    fn unlesbare_werden_spaeter_lesbar() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        let mut zu = m("a", "x@t", "INBOX", 1, false);
        zu.pgp = Some("unlesbar".into());
        zu.text = "-----BEGIN PGP MESSAGE-----".into();
        s.mail_ablegen(&zu).unwrap();
        assert_eq!(s.unlesbare_mails("tiffy@beispiel.test").unwrap().len(), 1);
        s.mail_entschluesselt("a", "Betreff", "Klartext", "[]", Some("gueltig"))
            .unwrap();
        let z = s.mail("a").unwrap().unwrap();
        assert_eq!(
            (z.text.as_str(), z.pgp.as_deref(), z.signatur.as_deref()),
            ("Klartext", Some("verschluesselt"), Some("gueltig"))
        );
        assert!(s.unlesbare_mails("tiffy@beispiel.test").unwrap().is_empty());
    }

    #[test]
    fn alte_zeilen_werden_zugeordnet() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        let mut alt = m("a", "x@t", "INBOX", 1, false);
        alt.postfach = String::new();
        s.mail_ablegen(&alt).unwrap();
        s.mails_zuordnen("tiffy@beispiel.test").unwrap();
        assert_eq!(
            s.mail("a").unwrap().unwrap().postfach,
            "tiffy@beispiel.test"
        );
    }
}
