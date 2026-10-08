//! VEVENTs aus einer ICS-Datei lesen -- und zwar nur das.
//!
//! Diese Kiste legt nichts an, kennt keinen Kalender und fasst kein Netz an.
//! Text hinein, Termine heraus. Das Holen steht in [`crate::abo`], das
//! Einsortieren ebenfalls.
//!
//! # Die Vorlage
//!
//! Drueben im Server steht dasselbe als `App\Support\IcsParser` und
//! `App\Support\IcsTime`. Diese Datei ist absichtlich die zweite Uebersetzung
//! derselben Norm und keine Abwandlung: Wer ein Abo im Browser und dasselbe
//! Abo im Geraet hat, soll dieselben Ziffern sehen. Wo hier bewusst etwas
//! anderes passiert, steht es dabei.
//!
//! # Die Zeit-Vereinbarung
//!
//! `beginn` und `ende` tragen **Wanduhrzeit**: die Ziffern, die der Mensch
//! sehen soll, in SEINER Zone. Kein UTC, kein Versatz -- genau wie drueben in
//! `events.start_date`. Ein Termin, der als `20260815T120000Z` ankommt, steht
//! hier als `2026-08-15T14:00:00`, wenn das Geraet in Berlin steht.
//!
//! Was ganztaegig ist, wird nicht umgerechnet. Ein Geburtstag am 14. Mai ist
//! ueberall der 14. Mai; wer ihn durch eine Zone dreht, verschiebt ihn je
//! nach Richtung um einen Tag. Das ist der Fehler, den man bei Geburtstagen
//! sieht.
//!
//! # Was hier anders ist als drueben
//!
//! ICS maskiert Text: `\n` fuer den Zeilenumbruch, `\,` und `\;` fuer die
//! Zeichen, die sonst trennen ([RFC 5545, 3.3.11]). Der Server nimmt den Wert
//! bisher roh und zeigt deshalb ein wortwoertliches `\n` mitten im Text an.
//! Hier wird entmaskiert. Das ist eine bewusste Abweichung -- die Seite, die
//! es richtig macht, soll nicht auf die andere warten.
//!
//! [RFC 5545, 3.3.11]: https://www.rfc-editor.org/rfc/rfc5545#section-3.3.11

use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, TimeZone};
use chrono_tz::Tz;
use openany_client::Terminfelder;

/// Was beim Lesen einer Datei herauskam.
pub struct Gelesen {
    pub termine: Vec<AusDemFeed>,
    /// Angaben, die nicht gelesen werden konnten und deshalb FEHLEN --
    /// Serienenden und Ausnahmetermine.
    ///
    /// Sie stehen hier, statt still verschluckt zu werden: Ein Kalender, der
    /// aussieht wie eingelesen und dabei falsch ist, ist schlimmer als einer,
    /// der sich beschwert. Ein verschlucktes `UNTIL` macht aus einer Serie
    /// mit Ende eine ohne; ein verschlucktes `EXDATE` laesst einen einzeln
    /// abgesagten Termin wieder auftauchen.
    pub uebersprungen: Vec<String>,
}

/// Ein Termin, wie er aus dem Feed kam.
pub struct AusDemFeed {
    /// Das `UID` aus der Datei. Die fremde Quelle vergibt es, und sie haelt
    /// es ueber Aenderungen hinweg fest -- daran erkennt der naechste Abruf
    /// denselben Termin wieder, statt ihn ein zweites Mal anzulegen.
    ///
    /// Leer, wenn die Datei keines mitschickt (es kommt vor, auch wenn die
    /// Norm es verlangt). Dann traegt [`crate::abo`] die Wiedererkennung.
    pub uid: String,
    pub felder: Terminfelder,
}

/// Die VEVENTs einer ICS-Datei.
///
/// Bricht nicht ab. Eine Datei mit einem unlesbaren Termin soll die anderen
/// neunundneunzig nicht kosten -- was fehlt, steht in `uebersprungen`.
pub fn lesen(inhalt: &str) -> Gelesen {
    let mut termine = Vec::new();
    let mut uebersprungen = Vec::new();
    let mut offen: Option<Roh> = None;

    for zeile in zeilen(inhalt) {
        if zeile.starts_with("BEGIN:VEVENT") {
            offen = Some(Roh::default());
            continue;
        }

        if zeile.starts_with("END:VEVENT") {
            if let Some(roh) = offen.take() {
                if let Some(termin) = roh.fertig(&mut uebersprungen) {
                    termine.push(termin);
                }
            }
            continue;
        }

        // Alles ausserhalb eines VEVENT geht uns nichts an. Insbesondere der
        // VTIMEZONE-Block: Seine DTSTART-Zeilen saehen sonst aus wie die
        // eines Termins.
        if let Some(roh) = offen.as_mut() {
            roh.merke(&zeile, &mut uebersprungen);
        }
    }

    Gelesen {
        termine,
        uebersprungen,
    }
}

/// Die Zeilen der Datei, Fortsetzungen bereits angeklebt.
///
/// ICS bricht jede Zeile nach 75 Oktetts um und setzt sie mit einem
/// fuehrenden Leerzeichen oder Tabulator fort (RFC 5545, 3.1). Wer Zeile fuer
/// Zeile liest, verliert alles hinter dem Umbruch: Aus
/// `SUMMARY:Herbstferien Nordrhein-Westf` + ` alen` wird ein abgeschnittener
/// Name -- und aus einem umbrochenen `RRULE` eine Serie ohne Ende. Drueben
/// hat genau das jahrelang gefehlt.
///
/// Das Trennzeichen selbst gehoert NICHT zum Wert; es steht nur da, um den
/// Umbruch zu markieren.
fn zeilen(inhalt: &str) -> Vec<String> {
    let mut zeilen: Vec<String> = Vec::new();

    for zeile in inhalt.replace('\r', "").split('\n') {
        match zeile.strip_prefix([' ', '\t']) {
            Some(rest) if !zeilen.is_empty() => {
                zeilen.last_mut().expect("not empty").push_str(rest);
            }
            _ => zeilen.push(zeile.to_string()),
        }
    }

    zeilen
}

/// Ein VEVENT, waehrend es noch gelesen wird.
#[derive(Default)]
struct Roh {
    uid: String,
    summary: Option<String>,
    description: Option<String>,
    location: Option<String>,
    /// Diese drei stehen mit ihrer GANZEN Zeile drin: Der `TZID`-Parameter
    /// gehoert zum Wert und wird erst beim Umrechnen ausgewertet.
    dtstart: Option<String>,
    dtend: Option<String>,
    rrule: Option<String>,
    exdates: Vec<String>,
}

impl Roh {
    fn merke(&mut self, zeile: &str, uebersprungen: &mut Vec<String>) {
        // Der Name einer Zeile endet am `:` oder am `;` -- sonst faenge
        // `DTSTART` auch `DTSTAMP` nicht, aber `SUMMARY` auch `SUMMARYX`.
        let name = zeile
            .split_once([':', ';'])
            .map(|(name, _)| name)
            .unwrap_or(zeile);

        // Text: entmaskiert, siehe Kistenkommentar.
        match name {
            "SUMMARY" => self.summary = Some(entmaskieren(&wert_aus(zeile))),
            "DESCRIPTION" => self.description = Some(entmaskieren(&wert_aus(zeile))),
            "LOCATION" => self.location = Some(entmaskieren(&wert_aus(zeile))),
            "UID" => self.uid = wert_aus(zeile),
            // Diese drei mit ihrer GANZEN Zeile: Der TZID-Parameter gehoert
            // zum Wert und wird erst beim Umrechnen ausgewertet.
            "DTSTART" => self.dtstart = Some(zeile.to_string()),
            "DTEND" => self.dtend = Some(zeile.to_string()),
            "RRULE" => self.rrule = Some(zeile.to_string()),
            "EXDATE" => self.exdate(zeile, uebersprungen),
            _ => {}
        }
    }

    fn exdate(&mut self, zeile: &str, uebersprungen: &mut Vec<String>) {
        let tzid = tzid_aus(zeile);

        for ex in wert_aus(zeile).split(',') {
            match als_wanduhr(ex.trim(), tzid.as_deref()) {
                // Auf Wanduhrzeit gebracht -- dadurch passen Ausnahmen und
                // DTSTART hinterher zusammen.
                Some(w) => self.exdates.push(w.format("%Y%m%dT%H%M%S").to_string()),
                None => uebersprungen.push(format!("EXDATE: {}", ex.trim())),
            }
        }
    }

    /// Der fertige Termin -- oder nichts, wenn das Noetigste fehlt.
    ///
    /// Ohne `DTSTART` gibt es keinen Termin; ohne `SUMMARY` gibt es keinen,
    /// den ein Mensch in seiner Liste wiedererkennt. Drueben gilt dieselbe
    /// Bedingung.
    fn fertig(self, uebersprungen: &mut Vec<String>) -> Option<AusDemFeed> {
        let dtstart = self.dtstart.as_deref()?;
        let titel = self.summary.clone()?;

        let startwert = wert_aus(dtstart);
        let ganztags = !startwert.contains('T');

        let beginn = als_wanduhr(&startwert, tzid_aus(dtstart).as_deref())?;

        let ende = match self.dtend.as_deref() {
            Some(zeile) => {
                let mut e = als_wanduhr(&wert_aus(zeile), tzid_aus(zeile).as_deref())?;
                if ganztags {
                    // Bei ganztaegigen Terminen ist DTEND in ICS EXKLUSIV:
                    // Ein eintaegiger Termin am 14. traegt DTEND 15. Wer das
                    // uebernimmt, macht aus jedem Geburtstag zwei Tage.
                    e -= chrono::Duration::days(1);
                }
                e
            }
            // Ohne DTEND dauert der Termin bis zu seinem Beginn -- dieselbe
            // Annahme wie drueben.
            None => beginn,
        };

        let (rrule, rrule_bis) = self.wiederholung(dtstart, uebersprungen);

        Some(AusDemFeed {
            uid: self.uid,
            felder: Terminfelder {
                titel,
                beschreibung: self.description.unwrap_or_default(),
                ort: self.location.unwrap_or_default(),
                beginn: beginn.format("%Y-%m-%dT%H:%M:%S").to_string(),
                ende: ende.format("%Y-%m-%dT%H:%M:%S").to_string(),
                ganztags,
                rrule,
                rrule_bis,
                exdates: self.exdates.join(","),
            },
        })
    }

    /// Die Wiederholung als das Paar, das der Speicher fuehrt.
    ///
    /// openany kennt vier Frequenzen und ein Enddatum, nicht die ganze
    /// RRULE-Grammatik. Was darueber hinausgeht (`BYDAY`, `INTERVAL`,
    /// `COUNT`), faellt weg -- es faellt drueben genauso weg, und ein
    /// woechentlicher Termin bleibt dabei ein woechentlicher Termin.
    fn wiederholung(&self, dtstart: &str, uebersprungen: &mut Vec<String>) -> (String, String) {
        let Some(zeile) = self.rrule.as_deref() else {
            return ("NONE".into(), String::new());
        };

        let regel = wert_aus(zeile);

        let rrule = ["DAILY", "WEEKLY", "MONTHLY", "YEARLY"]
            .into_iter()
            .find(|f| regel.contains(&format!("FREQ={f}")))
            .unwrap_or("NONE")
            .to_string();

        let bis = regel
            .split(';')
            .find_map(|teil| teil.strip_prefix("UNTIL="))
            .map(|wert| {
                // UNTIL kommt laut Norm in UTC; ohne `Z` gilt es in der Zone
                // von DTSTART. Beides landet als Wanduhrzeit in der Spalte.
                match als_wanduhr(wert, tzid_aus(dtstart).as_deref()) {
                    Some(w) => w.format("%Y-%m-%dT%H:%M:%S").to_string(),
                    None => {
                        uebersprungen.push(format!("UNTIL: {wert}"));
                        String::new()
                    }
                }
            })
            .unwrap_or_default();

        (rrule, bis)
    }
}

/// Ein ICS-Wert als Wanduhrzeit in der Zone des Geraets.
///
/// `None` heisst: unlesbar. Der Aufrufer entscheidet dann, ob das den Termin
/// kostet (bei `DTSTART`) oder nur eine Angabe (bei `UNTIL`, `EXDATE`).
fn als_wanduhr(wert: &str, tzid: Option<&str>) -> Option<NaiveDateTime> {
    let wert = wert.trim();

    // Ganztaegig: ein Datum, kein Zeitpunkt. Nicht umrechnen.
    if !wert.contains('T') {
        return NaiveDate::parse_from_str(wert, "%Y%m%d")
            .ok()
            .map(|d| d.into());
    }

    let utc = wert.ends_with('Z');
    let roh = wert.trim_end_matches('Z');
    let naiv = NaiveDateTime::parse_from_str(roh, "%Y%m%dT%H%M%S").ok()?;

    // Ohne Z und ohne TZID sagt die Datei nichts ueber ihre Zone. Dann ist
    // die Zone des Geraets die einzig vertretbare Annahme -- und weil Quelle
    // und Ziel dann dieselbe sind, wird gar nicht gerechnet.
    let quelle: Option<Tz> = if utc {
        Some(Tz::UTC)
    } else {
        match tzid {
            None => return Some(naiv),
            // Ein TZID, den die Zonendatenbank nicht kennt (es gibt Programme,
            // die eigene Namen erfinden). Lieber die Ziffern stehen lassen,
            // als den Termin wegzuwerfen: Um ein paar Stunden daneben ist
            // besser als gar nicht da.
            Some(name) => name.parse::<Tz>().ok().or(None),
        }
    };

    let Some(quelle) = quelle else {
        return Some(naiv);
    };

    // Zu einer Ortszeit kann es zwei Zeitpunkte geben (die Stunde im Oktober,
    // die zweimal stattfindet) oder keinen (die Stunde im Maerz, die
    // ausfaellt). `earliest` nimmt dann den frueheren beziehungsweise gibt
    // auf -- und aufgeben heisst hier: die Ziffern unveraendert lassen.
    let zeitpunkt: DateTime<Tz> = quelle.from_local_datetime(&naiv).earliest()?;

    Some(zeitpunkt.with_timezone(&Local).naive_local())
}

/// Der `TZID`-Parameter einer Eigenschaftszeile, falls einer dransteht.
fn tzid_aus(zeile: &str) -> Option<String> {
    let rest = zeile.split_once(";TZID=")?.1;

    Some(
        rest.split([':', ';'])
            .next()
            .unwrap_or_default()
            .to_string(),
    )
}

/// Der Wert einer Eigenschaftszeile -- alles hinter dem ersten Doppelpunkt.
fn wert_aus(zeile: &str) -> String {
    zeile
        .split_once(':')
        .map(|(_, wert)| wert.trim().to_string())
        .unwrap_or_default()
}

/// Maskierter ICS-Text zurueck in lesbaren (RFC 5545, 3.3.11).
///
/// Die Reihenfolge ist keine Geschmacksfrage: `\\n` ist ein maskierter
/// Rueckstrich gefolgt von einem `n` und KEIN Zeilenumbruch. Wer zuerst
/// `\n` ersetzt, macht aus dem einen faelschlich den anderen. Deshalb Zeichen
/// fuer Zeichen von links.
fn entmaskieren(text: &str) -> String {
    let mut heraus = String::with_capacity(text.len());
    let mut zeichen = text.chars();

    while let Some(z) = zeichen.next() {
        if z != '\\' {
            heraus.push(z);
            continue;
        }

        match zeichen.next() {
            Some('n' | 'N') => heraus.push('\n'),
            Some(anderes) => heraus.push(anderes),
            // Ein Rueckstrich am Zeilenende. Kaputt, aber kein Grund zur Panik.
            None => heraus.push('\\'),
        }
    }

    heraus
}

#[cfg(test)]
mod lesen_test {
    use super::*;

    /// Die Zone, in der diese Tests rechnen. Steht das Geraet woanders,
    /// verschieben sich die erwarteten Ziffern -- deshalb pruefen die Tests
    /// mit Z-Zeiten gegen die Zone und nicht gegen feste Ziffern.
    fn ortszeit(utc: &str) -> String {
        let naiv = NaiveDateTime::parse_from_str(utc, "%Y%m%dT%H%M%S").unwrap();
        Tz::UTC
            .from_local_datetime(&naiv)
            .unwrap()
            .with_timezone(&Local)
            .naive_local()
            .format("%Y-%m-%dT%H:%M:%S")
            .to_string()
    }

    fn eine_datei(vevent: &str) -> Gelesen {
        lesen(&format!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\n{vevent}\r\nEND:VCALENDAR\r\n"
        ))
    }

    #[test]
    fn ein_gewoehnlicher_termin() {
        let g = eine_datei(
            "BEGIN:VEVENT\r\nUID:abc-123\r\nSUMMARY:Zahnarzt\r\nLOCATION:Hauptstr. 1\r\n\
             DTSTART:20260920T070000Z\r\nDTEND:20260920T073000Z\r\nEND:VEVENT",
        );

        assert_eq!(g.termine.len(), 1);
        let t = &g.termine[0];
        assert_eq!(t.uid, "abc-123");
        assert_eq!(t.felder.titel, "Zahnarzt");
        assert_eq!(t.felder.ort, "Hauptstr. 1");
        assert_eq!(t.felder.beginn, ortszeit("20260920T070000"));
        assert_eq!(t.felder.ende, ortszeit("20260920T073000"));
        assert!(!t.felder.ganztags);
        assert_eq!(t.felder.rrule, "NONE");
        assert!(g.uebersprungen.is_empty());
    }

    /// Ein umbrochener Titel. Wer Zeile fuer Zeile liest, bekommt hier
    /// "Herbstferien Nordrhein-Westf" -- der Fehler, den der Server jahrelang
    /// hatte.
    #[test]
    fn fortsetzungszeilen_werden_angeklebt() {
        let g = eine_datei(
            "BEGIN:VEVENT\r\nUID:x\r\nSUMMARY:Herbstferien Nordrhein-Westf\r\n alen\r\n\
             DTSTART;VALUE=DATE:20261012\r\nEND:VEVENT",
        );

        assert_eq!(
            g.termine[0].felder.titel,
            "Herbstferien Nordrhein-Westfalen"
        );
    }

    /// Ganztaegig: DTEND ist in ICS exklusiv. Ein eintaegiger Termin am 12.
    /// traegt DTEND 13. -- wer das uebernimmt, macht zwei Tage daraus.
    #[test]
    fn ganztaegig_endet_am_selben_tag() {
        let g = eine_datei(
            "BEGIN:VEVENT\r\nUID:x\r\nSUMMARY:Geburtstag\r\n\
             DTSTART;VALUE=DATE:20260514\r\nDTEND;VALUE=DATE:20260515\r\nEND:VEVENT",
        );

        let t = &g.termine[0];
        assert!(t.felder.ganztags);
        assert_eq!(t.felder.beginn, "2026-05-14T00:00:00");
        assert_eq!(t.felder.ende, "2026-05-14T00:00:00");
    }

    /// Ein Datum wird NICHT durch eine Zone gedreht. Sonst wandert der
    /// Geburtstag je nach Richtung auf den 13. oder den 15.
    #[test]
    fn ein_datum_bleibt_das_datum() {
        let g = eine_datei(
            "BEGIN:VEVENT\r\nUID:x\r\nSUMMARY:Feiertag\r\n\
             DTSTART;VALUE=DATE:20260101\r\nEND:VEVENT",
        );

        assert_eq!(g.termine[0].felder.beginn, "2026-01-01T00:00:00");
    }

    /// TZID nennt die Zone der Quelle. 14 Uhr in New York ist nicht 14 Uhr
    /// hier -- und 20 Uhr UTC ist es.
    #[test]
    fn tzid_wird_umgerechnet() {
        let g = eine_datei(
            "BEGIN:VEVENT\r\nUID:x\r\nSUMMARY:Anruf\r\n\
             DTSTART;TZID=America/New_York:20260815T140000\r\n\
             DTEND;TZID=America/New_York:20260815T150000\r\nEND:VEVENT",
        );

        assert_eq!(g.termine[0].felder.beginn, ortszeit("20260815T180000"));
    }

    /// Ohne Z und ohne TZID sagt die Datei nichts. Dann bleiben die Ziffern
    /// stehen -- die Zone des Geraets ist die einzige vertretbare Annahme,
    /// und dann ist nichts zu rechnen.
    #[test]
    fn ohne_zonenangabe_bleiben_die_ziffern() {
        let g = eine_datei(
            "BEGIN:VEVENT\r\nUID:x\r\nSUMMARY:Schwimmen\r\n\
             DTSTART:20260815T140000\r\nEND:VEVENT",
        );

        assert_eq!(g.termine[0].felder.beginn, "2026-08-15T14:00:00");
    }

    /// Ein erfundener Zonenname kostet nicht den Termin. Um zwei Stunden
    /// daneben ist besser als gar nicht da.
    #[test]
    fn unbekannte_zone_kostet_nicht_den_termin() {
        let g = eine_datei(
            "BEGIN:VEVENT\r\nUID:x\r\nSUMMARY:Irgendwas\r\n\
             DTSTART;TZID=Customized Time Zone:20260815T140000\r\nEND:VEVENT",
        );

        assert_eq!(g.termine.len(), 1);
        assert_eq!(g.termine[0].felder.beginn, "2026-08-15T14:00:00");
    }

    #[test]
    fn serie_mit_ende_und_ausnahmen() {
        let g = eine_datei(
            "BEGIN:VEVENT\r\nUID:x\r\nSUMMARY:Turnen\r\n\
             DTSTART:20260901T160000Z\r\n\
             RRULE:FREQ=WEEKLY;BYDAY=TU;UNTIL=20261215T160000Z\r\n\
             EXDATE:20261006T160000Z,20261013T160000Z\r\nEND:VEVENT",
        );

        let t = &g.termine[0];
        assert_eq!(t.felder.rrule, "WEEKLY");
        assert_eq!(t.felder.rrule_bis, ortszeit("20261215T160000"));
        assert_eq!(t.felder.exdates.split(',').count(), 2);
        assert!(g.uebersprungen.is_empty());
    }

    /// Ein unlesbares UNTIL macht aus einer Serie mit Ende eine ohne. Der
    /// Import laeuft weiter -- sagt es aber.
    #[test]
    fn ein_unlesbares_until_wird_gemeldet() {
        let g = eine_datei(
            "BEGIN:VEVENT\r\nUID:x\r\nSUMMARY:Turnen\r\n\
             DTSTART:20260901T160000Z\r\nRRULE:FREQ=WEEKLY;UNTIL=morgen\r\nEND:VEVENT",
        );

        assert_eq!(g.termine[0].felder.rrule, "WEEKLY");
        assert_eq!(g.termine[0].felder.rrule_bis, "");
        assert_eq!(g.uebersprungen, vec!["UNTIL: morgen"]);
    }

    /// ICS maskiert Text. Hier steht danach ein echter Umbruch -- und ein
    /// Komma bleibt ein Komma.
    #[test]
    fn maskierter_text_wird_lesbar() {
        let g = eine_datei(
            "BEGIN:VEVENT\r\nUID:x\r\nSUMMARY:Essen\\, Trinken\r\n\
             DESCRIPTION:Zeile eins\\nZeile zwei\\; und ein Rest\r\n\
             DTSTART:20260815T140000\r\nEND:VEVENT",
        );

        assert_eq!(g.termine[0].felder.titel, "Essen, Trinken");
        assert_eq!(
            g.termine[0].felder.beschreibung,
            "Zeile eins\nZeile zwei; und ein Rest"
        );
    }

    /// `\\n` ist ein maskierter Rueckstrich mit einem n dahinter und KEIN
    /// Umbruch. Wer von rechts ersetzt, macht aus dem einen den anderen.
    #[test]
    fn ein_maskierter_rueckstrich_bleibt_einer() {
        let g = eine_datei(
            // In der Datei steht: C:\\neu -- zwei Rueckstriche, dann "neu".
            "BEGIN:VEVENT\r\nUID:x\r\nSUMMARY:C:\\\\neu\r\n\
             DTSTART:20260815T140000\r\nEND:VEVENT",
        );

        // Heraus kommt EIN Rueckstrich und ein n. Kein Umbruch.
        assert_eq!(g.termine[0].felder.titel, "C:\\neu");
        assert!(!g.termine[0].felder.titel.contains('\n'));
    }

    /// Der VTIMEZONE-Block traegt eigene DTSTART-Zeilen. Wer ausserhalb von
    /// VEVENT mitliest, baut daraus einen Geistertermin.
    #[test]
    fn vtimezone_ist_kein_termin() {
        let g = lesen(
            "BEGIN:VCALENDAR\r\nBEGIN:VTIMEZONE\r\nTZID:Europe/Berlin\r\n\
             BEGIN:DAYLIGHT\r\nDTSTART:19700329T020000\r\nTZNAME:CEST\r\n\
             END:DAYLIGHT\r\nEND:VTIMEZONE\r\nEND:VCALENDAR\r\n",
        );

        assert!(g.termine.is_empty());
    }

    /// Ohne DTSTART gibt es keinen Termin, ohne SUMMARY keinen, den jemand
    /// wiedererkennt. Die anderen in derselben Datei kostet das nichts.
    #[test]
    fn unvollstaendige_kosten_nicht_die_ganze_datei() {
        let g = lesen(
            "BEGIN:VCALENDAR\r\n\
             BEGIN:VEVENT\r\nUID:a\r\nSUMMARY:Ohne Anfang\r\nEND:VEVENT\r\n\
             BEGIN:VEVENT\r\nUID:b\r\nDTSTART:20260815T140000\r\nEND:VEVENT\r\n\
             BEGIN:VEVENT\r\nUID:c\r\nSUMMARY:Gut\r\nDTSTART:20260815T140000\r\nEND:VEVENT\r\n\
             END:VCALENDAR\r\n",
        );

        assert_eq!(g.termine.len(), 1);
        assert_eq!(g.termine[0].uid, "c");
    }

    /// Dateien mit reinen \n-Umbruechen kommen vor, auch wenn die Norm CRLF
    /// verlangt.
    #[test]
    fn auch_ohne_wagenruecklauf() {
        let g = lesen(
            "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:x\nSUMMARY:Da\n\
             DTSTART:20260815T140000\nEND:VEVENT\nEND:VCALENDAR\n",
        );

        assert_eq!(g.termine.len(), 1);
    }

    #[test]
    fn leere_datei_ist_kein_fehler() {
        assert!(lesen("").termine.is_empty());
        assert!(lesen("Das ist gar keine ICS-Datei.").termine.is_empty());
    }
}
