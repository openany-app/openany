//! Wie weit ist der Abgleich -- und was war zuletzt beidseitig bekannt?
//!
//! Zwei Tabellen, die nur der Laeufer anfasst, und beide tragen die
//! **Gegenstelle** im Schluessel. Warum das von Anfang an so ist und nicht
//! erst, wenn es zwei gibt, steht im Kopf von [`crate`].

use crate::{Ergebnis, Speicher};
use chrono::Utc;
use openany_client::Art;
use rusqlite::{params, OptionalExtension};

/// Der Stand gegenueber einer Gegenstelle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Marke {
    /// Die Basis-Adresse, etwa `https://openany.de`. Sie ist der Name der
    /// Gegenstelle -- deshalb muss sie stabil geschrieben sein (ohne
    /// Schraegstrich am Ende), sonst entstehen zwei Gegenstellen, wo eine
    /// gemeint ist.
    pub gegenstelle: String,
    /// Wie weit dort gelesen wurde.
    pub fremde: i64,
    /// Wie weit von hier geschickt wurde.
    ///
    /// **Zwei Marken und nicht eine**: Die beiden Richtungen gehen
    /// unterschiedlich weit. Mit einer gemeinsamen risse ein abgebrochener
    /// Lauf beide mit -- ein Fehler beim Schicken machte dann auch das schon
    /// Gelesene ungueltig.
    pub eigene: i64,
    pub letzter_lauf: Option<String>,
    pub letzter_fehler: Option<String>,
}

impl Speicher {
    /// Den Stand holen -- und anlegen, falls es ihn noch nicht gibt.
    pub fn marke(&self, gegenstelle: &str) -> Ergebnis<Marke> {
        self.db().execute(
            "INSERT OR IGNORE INTO marken (gegenstelle) VALUES (?1)",
            params![gegenstelle],
        )?;

        Ok(self.db().query_row(
            "SELECT gegenstelle, fremde_marke, eigene_marke, letzter_lauf, letzter_fehler
             FROM marken WHERE gegenstelle = ?1",
            params![gegenstelle],
            |z| {
                Ok(Marke {
                    gegenstelle: z.get(0)?,
                    fremde: z.get(1)?,
                    eigene: z.get(2)?,
                    letzter_lauf: z.get(3)?,
                    letzter_fehler: z.get(4)?,
                })
            },
        )?)
    }

    /// Alle bekannten Gegenstellen -- heute eine, morgen vielleicht zwei.
    pub fn gegenstellen(&self) -> Ergebnis<Vec<String>> {
        let db = self.db();
        let mut abfrage = db.prepare("SELECT gegenstelle FROM marken ORDER BY gegenstelle")?;

        let liste = abfrage
            .query_map([], |z| z.get(0))?
            .collect::<Result<_, _>>()?;

        Ok(liste)
    }

    /// **Seitenweise vorruecken, nicht am Ende.**
    ///
    /// Ein Abbruch auf Seite sieben kostet Seite sieben, nicht die sechs
    /// davor. Auf einer Mobilfunkleitung ist das der Unterschied zwischen
    /// einem Erstabgleich, der irgendwann fertig wird, und einem, der bei
    /// jedem Funkloch von vorn beginnt.
    pub fn fremde_marke_setzen(&self, gegenstelle: &str, wert: i64) -> Ergebnis<()> {
        self.marke(gegenstelle)?;
        self.db().execute(
            "UPDATE marken SET fremde_marke = ?2 WHERE gegenstelle = ?1",
            params![gegenstelle, wert],
        )?;

        Ok(())
    }

    /// Nach dem Einspielen einer Sicherung (08.10.2026): Jede Gegenstelle
    /// liefert beim naechsten Lauf ihren ganzen Bestand. Die Sicherung ist
    /// aelter als das, was drueben inzwischen geschah; ein Bestand fuehrt das
    /// zusammen, ohne etwas zu loeschen (siehe `bestand_neu_holen`). Die
    /// eigene Marke bleibt -- was von hier schon hinaus ist, ist hinaus.
    pub fn fremde_marken_vergessen(&self) -> Ergebnis<()> {
        self.db()
            .execute("UPDATE marken SET fremde_marke = 0", [])?;
        Ok(())
    }

    pub fn eigene_marke_setzen(&self, gegenstelle: &str, wert: i64) -> Ergebnis<()> {
        self.marke(gegenstelle)?;
        self.db().execute(
            "UPDATE marken SET eigene_marke = ?2 WHERE gegenstelle = ?1",
            params![gegenstelle, wert],
        )?;

        Ok(())
    }

    /// Einen Lauf abschliessen. `fehler` ist `None`, wenn er durchlief.
    ///
    /// **Der Fehler bleibt stehen, die Marken auch.** Was bis dahin gelaufen
    /// ist, ist gelaufen, und der naechste Versuch setzt dort an statt von
    /// vorn zu beginnen.
    pub fn lauf_beendet(&self, gegenstelle: &str, fehler: Option<&str>) -> Ergebnis<()> {
        self.marke(gegenstelle)?;
        self.db().execute(
            "UPDATE marken SET letzter_lauf = ?2, letzter_fehler = ?3 WHERE gegenstelle = ?1",
            params![gegenstelle, Utc::now().to_rfc3339(), fehler],
        )?;

        Ok(())
    }

    // --- Urspruenge ------------------------------------------------------

    /// Der zuletzt beidseitig bekannte Abdruck -- oder `None`.
    ///
    /// **`None` ist kein Nichts, sondern eine Aussage**: "wir hatten nie
    /// einen gemeinsamen Stand". Der Vergleich ist dann nur zweiseitig, und
    /// zwei abweichende Staende sehen genauso aus, ob nun eine Seite
    /// geaendert hat oder beide. Deshalb gilt in diesem Fall jeder
    /// Unterschied als Konflikt -- unbequem und absichtlich so: Ein Abgleich,
    /// der im Zweifel ueberschreibt, verliert Text.
    pub fn ursprung(
        &self,
        gegenstelle: &str,
        art: &Art,
        schluessel: &str,
    ) -> Ergebnis<Option<String>> {
        Ok(self
            .db()
            .query_row(
                "SELECT abdruck FROM ursprung
                 WHERE gegenstelle = ?1 AND art = ?2 AND schluessel = ?3",
                params![gegenstelle, art.ueber_die_leitung(), schluessel],
                |z| z.get(0),
            )
            .optional()?)
    }

    pub fn ursprung_merken(
        &self,
        gegenstelle: &str,
        art: &Art,
        schluessel: &str,
        abdruck: &str,
    ) -> Ergebnis<()> {
        self.db().execute(
            "INSERT INTO ursprung (gegenstelle, art, schluessel, abdruck, at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT (gegenstelle, art, schluessel)
             DO UPDATE SET abdruck = excluded.abdruck, at = excluded.at",
            params![
                gegenstelle,
                art.ueber_die_leitung(),
                schluessel,
                abdruck,
                chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
            ],
        )?;

        Ok(())
    }

    /// Der zuletzt gemerkte Ursprung dieser Sache -- mit welcher Gegenstelle
    /// auch immer.
    ///
    /// **Nur zum Mitschicken, nie zum Entscheiden.** Wer schiebt, sagt damit:
    /// "Von diesem Stand stammt meiner ab" -- und das stimmt, denn es ist der
    /// Stand, den dieses Geraet zuletzt mit irgendwem geteilt hat. Ein drittes
    /// Geraet, das denselben Stand ueber einen Umweg bekam, erkennt ihn so
    /// wieder, statt beim ersten direkten Treffen einen Konflikt zu sehen.
    ///
    /// Umgekehrt waere es falsch: Stimmte der HIESIGE Stand mit irgendeinem
    /// fremden Ursprung ueberein, hiesse das nicht, dass die ankommende
    /// Fassung von ihm abstammt.
    pub fn juengster_ursprung(&self, art: &Art, schluessel: &str) -> Ergebnis<Option<String>> {
        Ok(self
            .db()
            .query_row(
                "SELECT abdruck FROM ursprung WHERE art = ?1 AND schluessel = ?2
                 ORDER BY at DESC LIMIT 1",
                params![art.ueber_die_leitung(), schluessel],
                |z| z.get(0),
            )
            .optional()?)
    }

    /// Beim Loeschen: Ein Ursprung zu etwas, das es nicht mehr gibt,
    /// behauptete beim naechsten Auftauchen eine Uebereinstimmung, die es nie
    /// gab.
    pub fn ursprung_vergessen(
        &self,
        gegenstelle: &str,
        art: &Art,
        schluessel: &str,
    ) -> Ergebnis<()> {
        self.db().execute(
            "DELETE FROM ursprung WHERE gegenstelle = ?1 AND art = ?2 AND schluessel = ?3",
            params![gegenstelle, art.ueber_die_leitung(), schluessel],
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn speicher() -> Speicher {
        Speicher::im_arbeitsspeicher().unwrap()
    }

    #[test]
    fn eine_neue_gegenstelle_faengt_bei_null_an() {
        let s = speicher();
        let m = s.marke("https://openany.de").unwrap();

        assert_eq!(m.fremde, 0);
        assert_eq!(m.eigene, 0);
        assert_eq!(m.letzter_lauf, None);
    }

    #[test]
    fn die_beiden_marken_ruecken_getrennt_vor() {
        let s = speicher();

        s.fremde_marke_setzen("https://openany.de", 42).unwrap();

        let m = s.marke("https://openany.de").unwrap();

        assert_eq!(m.fremde, 42);
        assert_eq!(
            m.eigene, 0,
            "ein Fehler beim Schicken darf das Lesen nicht mitreissen"
        );
    }

    #[test]
    fn nach_einer_sicherung_kommt_der_bestand_aber_nichts_doppelt_hinaus() {
        let s = speicher();
        s.fremde_marke_setzen("https://openany.de", 42).unwrap();
        s.eigene_marke_setzen("https://openany.de", 9).unwrap();
        s.fremde_marke_setzen("nah:geraet", 5).unwrap();

        s.fremde_marken_vergessen().unwrap();

        assert_eq!(s.marke("https://openany.de").unwrap().fremde, 0);
        assert_eq!(s.marke("https://openany.de").unwrap().eigene, 9);
        assert_eq!(s.marke("nah:geraet").unwrap().fremde, 0);
    }

    #[test]
    fn zwei_gegenstellen_kommen_sich_nicht_in_die_quere() {
        // Der Grund, warum die Gegenstelle von Anfang an im Schluessel steht.
        let s = speicher();

        s.fremde_marke_setzen("https://openany.de", 100).unwrap();
        s.fremde_marke_setzen("https://zweite.example", 7).unwrap();

        assert_eq!(s.marke("https://openany.de").unwrap().fremde, 100);
        assert_eq!(s.marke("https://zweite.example").unwrap().fremde, 7);
        assert_eq!(s.gegenstellen().unwrap().len(), 2);
    }

    #[test]
    fn ein_ursprung_gilt_je_gegenstelle() {
        let s = speicher();

        s.ursprung_merken("https://openany.de", &Art::Notiz, "zk-1", "aaa")
            .unwrap();

        assert_eq!(
            s.ursprung("https://openany.de", &Art::Notiz, "zk-1")
                .unwrap(),
            Some("aaa".into())
        );
        assert_eq!(
            s.ursprung("https://zweite.example", &Art::Notiz, "zk-1")
                .unwrap(),
            None,
            "gegenueber der zweiten gab es nie einen gemeinsamen Stand"
        );
    }

    #[test]
    fn dieselbe_kennung_in_zwei_arten_ist_nicht_dieselbe_sache() {
        let s = speicher();

        s.ursprung_merken("g", &Art::Notiz, "gleiche-id", "notiz")
            .unwrap();
        s.ursprung_merken("g", &Art::Termin, "gleiche-id", "termin")
            .unwrap();

        assert_eq!(
            s.ursprung("g", &Art::Notiz, "gleiche-id").unwrap(),
            Some("notiz".into())
        );
        assert_eq!(
            s.ursprung("g", &Art::Termin, "gleiche-id").unwrap(),
            Some("termin".into())
        );
    }

    #[test]
    fn merken_ueberschreibt_und_haeuft_nicht_an() {
        let s = speicher();

        s.ursprung_merken("g", &Art::Notiz, "zk-1", "alt").unwrap();
        s.ursprung_merken("g", &Art::Notiz, "zk-1", "neu").unwrap();

        assert_eq!(
            s.ursprung("g", &Art::Notiz, "zk-1").unwrap(),
            Some("neu".into())
        );
    }

    #[test]
    fn vergessen_loescht_nur_den_einen() {
        let s = speicher();

        s.ursprung_merken("g", &Art::Notiz, "zk-1", "a").unwrap();
        s.ursprung_merken("g", &Art::Notiz, "zk-2", "b").unwrap();

        s.ursprung_vergessen("g", &Art::Notiz, "zk-1").unwrap();

        assert_eq!(s.ursprung("g", &Art::Notiz, "zk-1").unwrap(), None);
        assert_eq!(
            s.ursprung("g", &Art::Notiz, "zk-2").unwrap(),
            Some("b".into())
        );
    }

    #[test]
    fn ein_fehler_bleibt_stehen_bis_ein_lauf_durchlaeuft() {
        let s = speicher();

        s.lauf_beendet("g", Some("Netz weg")).unwrap();
        assert_eq!(
            s.marke("g").unwrap().letzter_fehler.as_deref(),
            Some("Netz weg")
        );

        s.lauf_beendet("g", None).unwrap();
        assert_eq!(s.marke("g").unwrap().letzter_fehler, None);
    }
}
