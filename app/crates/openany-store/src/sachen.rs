//! Die Sachen selbst -- Notizen, Kalender, Termine, Kontakte.
//!
//! **Ein Termin traegt seine neun Felder als [`Terminfelder`]** und nicht als
//! neun eigene Felder. Das ist der wichtigste Griff in dieser Datei: Der
//! Abdruck, ueber den drueben Konflikte entschieden werden, wird aus genau
//! dieser Struktur gerechnet. Haette der Speicher seine eigene Vorstellung
//! von einem Termin, koennten die beiden auseinanderlaufen -- und zwar
//! lautlos, in Konfliktkopien bei jedem Lauf. So ist das ausgeschlossen: Was
//! gespeichert ist, ist was gehasht wird.
//!
//! **Alle Schreibwege verlangen ein [`Protokoll`].** Als Pflichtangabe und
//! nicht als Voreinstellung -- siehe dort, warum Vergessen nicht die
//! bequemere Haelfte sein darf.

use crate::{Ergebnis, Protokoll, Speicher};
use chrono::Utc;
use openany_client::{Art, Terminfelder, Was};
use rusqlite::{params, OptionalExtension, Row};

/// Eine Notiz. Der Text steht mit in der Zeile -- anders als drueben, wo er
/// auf der Platte liegt und ueber `sync/content` geholt wird.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Notiz {
    /// Die Identitaet ueber Instanzen hinweg -- drueben `notes.zk_id`.
    pub zk_id: String,
    pub titel: String,
    pub inhalt: String,
    /// Der Mappen**pfad**, wie er ueber die Leitung geht. `None` = Wurzel.
    pub mappe: Option<String>,
    pub papierkorb_at: Option<String>,
    pub geaendert_at: String,
}

impl Notiz {
    pub fn mit_id(zk_id: &str) -> Self {
        Self {
            zk_id: zk_id.to_string(),
            ..Default::default()
        }
    }

    pub fn im_papierkorb(&self) -> bool {
        self.papierkorb_at.is_some()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Kalender {
    pub uuid: String,
    pub name: String,
    pub farbe: String,
    /// Die Adresse eines ICS-Feeds -- `None` heisst: ein eigener Kalender.
    ///
    /// **Ein Abo ist eine Adresse, kein Inhalt.** Sein Inhalt kommt von
    /// woanders und wird bei jedem Holen ersetzt; geschrieben wird in ihn
    /// nicht. Genau deshalb reist er auch nicht ueber den Abgleich -- drueben
    /// haelt `Calendar::shouldRecordChanges` ihn heraus, und dieses Programm
    /// holt ihn selbst.
    pub abo_url: Option<String>,
    /// Wann zuletzt geholt. `None` = noch nie.
    pub zuletzt_geholt: Option<String>,
    pub papierkorb_at: Option<String>,
    pub geaendert_at: String,
}

impl Kalender {
    pub fn ist_abo(&self) -> bool {
        self.abo_url
            .as_deref()
            .is_some_and(|u| !u.trim().is_empty())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Termin {
    pub uuid: String,
    /// Die **uuid** des Kalenders, nicht eine lokale Zeilennummer: Genau so
    /// reist er ueber die Leitung, und was hier anders hiesse, muesste an
    /// jeder Grenze uebersetzt werden.
    pub kalender_uuid: String,
    pub felder: Terminfelder,
    pub papierkorb_at: Option<String>,
    pub geaendert_at: String,
}

impl Termin {
    /// Der Abdruck ueber die neun Felder -- die Zahl, mit der drueben
    /// entschieden wird.
    pub fn abdruck(&self) -> String {
        self.felder.abdruck()
    }
}

/// Ein Weg, auf dem ein Kontakt erreichbar ist -- Nummer, E-Mail, Anschrift,
/// Kennung, oder ein durchgereichtes vCard-Feld (`vcard:…`).
///
/// **Die Namen ueber der Leitung** (`kind`, `label`, `value`) stehen auch in
/// der Datei: Die Liste liegt als JSON in `kontakte.wege`, genau so, wie sie im
/// Delta reist. Eine zweite Schreibweise dazwischen waere eine Uebersetzung
/// mehr, die sich irren kann.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Weg {
    #[serde(rename = "kind")]
    pub art: String,
    #[serde(rename = "label", default)]
    pub beschriftung: Option<String>,
    #[serde(rename = "value")]
    pub wert: String,
}

/// Ein Kontakt.
///
/// **Seit Stand 3 (15.09.2026) vollstaendig**: beliebig viele Wege in der
/// Reihenfolge des Menschen und der Abdruck seines Fotos. Vorher kannte das
/// Programm nur zwei Kennungen (Matrix, Meshtastic) -- der Stand vom
/// 06.09.2026, bevor die Webapp mehrere Nummern und Adressen bekam.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Kontakt {
    pub uuid: String,
    pub anzeigename: String,
    pub wege: Vec<Weg>,
    /// Abdruck des Fotos, wie der Server ihn nennt; `None` = keines. Die
    /// Bytes liegen getrennt (`kontaktfotos.rs`).
    pub foto: Option<String>,
    pub papierkorb_at: Option<String>,
    pub geaendert_at: String,
}

impl Kontakt {
    /// Der erste Weg einer Art -- etwa die Matrix-Kennung.
    pub fn kennung(&self, art: &str) -> Option<&str> {
        self.wege
            .iter()
            .find(|w| w.art == art && !w.wert.trim().is_empty())
            .map(|w| w.wert.as_str())
    }

    /// Der Abdruck ueber alles, was ueber die Leitung geht.
    ///
    /// **Ein Kontakt wird drueben nicht dreiseitig entschieden**, sondern mit
    /// dem letzten Schreiben -- ein Adressbucheintrag ist kurz und
    /// nachtippbar, eine doppelte Person dagegen laestig. Der Abdruck dient
    /// hier deshalb nur der Frage "hat sich ueberhaupt etwas geaendert",
    /// nicht der Konfliktentscheidung.
    pub fn abdruck(&self) -> String {
        openany_client::notizabdruck(&format!(
            "{}\u{1f}{}\u{1f}{}",
            self.anzeigename,
            serde_json::to_string(&self.wege).unwrap_or_default(),
            self.foto.as_deref().unwrap_or(""),
        ))
    }
}

/// Zu welcher Tabelle gehoert eine Art?
///
/// **Aus einer geschlossenen Aufzaehlung und nie aus einer Zeichenkette von
/// aussen.** Der Rueckgabewert wird in SQL eingesetzt; kaeme er aus einem
/// Delta-Eintrag, waere das eine Einladung, die dieses Programm nicht
/// aussprechen soll.
fn tabelle(art: &Art) -> Option<(&'static str, &'static str)> {
    match art {
        Art::Notiz => Some(("notizen", "zk_id")),
        Art::Kalender => Some(("kalender", "uuid")),
        Art::Termin => Some(("termine", "uuid")),
        Art::Kontakt => Some(("kontakte", "uuid")),
        Art::Datei => Some(("dateien", "uuid")),
        Art::Album => Some(("alben", "uuid")),
        Art::Medium => Some(("medien", "uuid")),
        // NOTIZANHAENGE FEHLEN HIER MIT ABSICHT, seit sie getragen werden
        // (16.09.2026): Ihr Schluessel ist zweiteilig (Mappe und Pfad), und
        // diese Tabelle kennt nur EINE Spalte. `loeschen` hat dafuer einen
        // eigenen Zweig; `papierkorb` bleibt bei `false`, denn eine Zuordnung
        // hat keinen -- sie ist da oder sie ist weg.
        //
        // `None` heisst hier weiterhin "uebergehen", nicht "Fehler": Ein
        // Abgleich, der an einer unbekannten Art abbricht, kaeme nie durch
        // die erste Seite eines gewachsenen Kontos.
        _ => None,
    }
}

impl Speicher {
    // --- Notizen ---------------------------------------------------------

    pub fn notiz_schreiben(&self, notiz: &Notiz, protokoll: Protokoll) -> Ergebnis<()> {
        self.db().execute(
            "INSERT INTO notizen (zk_id, titel, inhalt, mappe, papierkorb_at, geaendert_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT (zk_id) DO UPDATE SET
                titel = excluded.titel, inhalt = excluded.inhalt,
                mappe = excluded.mappe, papierkorb_at = excluded.papierkorb_at,
                geaendert_at = excluded.geaendert_at",
            params![
                notiz.zk_id,
                notiz.titel,
                notiz.inhalt,
                notiz.mappe,
                notiz.papierkorb_at,
                jetzt(&notiz.geaendert_at),
            ],
        )?;

        self.vielleicht_merken(protokoll, &Art::Notiz, &notiz.zk_id, Was::Da)
    }

    pub fn notiz(&self, zk_id: &str) -> Ergebnis<Option<Notiz>> {
        Ok(self
            .db()
            .query_row(
                "SELECT zk_id, titel, inhalt, mappe, papierkorb_at, geaendert_at
                 FROM notizen WHERE zk_id = ?1",
                params![zk_id],
                notiz_aus,
            )
            .optional()?)
    }

    /// Alle lebendigen Notizen, zuletzt geaenderte zuerst.
    pub fn notizen(&self) -> Ergebnis<Vec<Notiz>> {
        let db = self.db();
        let mut abfrage = db.prepare(
            "SELECT zk_id, titel, inhalt, mappe, papierkorb_at, geaendert_at
             FROM notizen WHERE papierkorb_at IS NULL ORDER BY geaendert_at DESC",
        )?;

        let liste = abfrage
            .query_map([], notiz_aus)?
            .collect::<Result<_, _>>()?;

        Ok(liste)
    }

    // --- Kalender --------------------------------------------------------

    pub fn kalender_schreiben(&self, k: &Kalender, protokoll: Protokoll) -> Ergebnis<()> {
        self.db().execute(
            "INSERT INTO kalender (uuid, name, farbe, abo_url, zuletzt_geholt, papierkorb_at, geaendert_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT (uuid) DO UPDATE SET
                name = excluded.name, farbe = excluded.farbe,
                abo_url = excluded.abo_url,
                zuletzt_geholt = excluded.zuletzt_geholt,
                papierkorb_at = excluded.papierkorb_at,
                geaendert_at = excluded.geaendert_at",
            params![
                k.uuid,
                k.name,
                k.farbe,
                k.abo_url,
                k.zuletzt_geholt,
                k.papierkorb_at,
                jetzt(&k.geaendert_at)
            ],
        )?;

        self.vielleicht_merken(protokoll, &Art::Kalender, &k.uuid, Was::Da)
    }

    pub fn kalender(&self, uuid: &str) -> Ergebnis<Option<Kalender>> {
        Ok(self
            .db()
            .query_row(
                "SELECT uuid, name, farbe, abo_url, zuletzt_geholt, papierkorb_at, geaendert_at
                 FROM kalender WHERE uuid = ?1",
                params![uuid],
                kalender_aus,
            )
            .optional()?)
    }

    pub fn kalender_alle(&self) -> Ergebnis<Vec<Kalender>> {
        let db = self.db();
        let mut abfrage = db.prepare(
            "SELECT uuid, name, farbe, abo_url, zuletzt_geholt, papierkorb_at, geaendert_at
             FROM kalender WHERE papierkorb_at IS NULL ORDER BY name",
        )?;

        let liste = abfrage
            .query_map([], kalender_aus)?
            .collect::<Result<_, _>>()?;

        Ok(liste)
    }

    // --- Termine ---------------------------------------------------------

    pub fn termin_schreiben(&self, t: &Termin, protokoll: Protokoll) -> Ergebnis<()> {
        self.db().execute(
            "INSERT INTO termine (uuid, kalender_uuid, titel, beschreibung, ort,
                                  beginn, ende, ganztags, rrule, rrule_bis, exdates,
                                  papierkorb_at, geaendert_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
             ON CONFLICT (uuid) DO UPDATE SET
                kalender_uuid = excluded.kalender_uuid, titel = excluded.titel,
                beschreibung = excluded.beschreibung, ort = excluded.ort,
                beginn = excluded.beginn, ende = excluded.ende,
                ganztags = excluded.ganztags, rrule = excluded.rrule,
                rrule_bis = excluded.rrule_bis, exdates = excluded.exdates,
                papierkorb_at = excluded.papierkorb_at,
                geaendert_at = excluded.geaendert_at",
            params![
                t.uuid,
                t.kalender_uuid,
                t.felder.titel,
                t.felder.beschreibung,
                t.felder.ort,
                t.felder.beginn,
                t.felder.ende,
                t.felder.ganztags,
                t.felder.rrule,
                t.felder.rrule_bis,
                t.felder.exdates,
                t.papierkorb_at,
                jetzt(&t.geaendert_at),
            ],
        )?;

        self.vielleicht_merken(protokoll, &Art::Termin, &t.uuid, Was::Da)
    }

    pub fn termin(&self, uuid: &str) -> Ergebnis<Option<Termin>> {
        Ok(self
            .db()
            .query_row(TERMIN_SPALTEN_WHERE, params![uuid], termin_aus)
            .optional()?)
    }

    /// Termine eines Kalenders, nach Beginn.
    pub fn termine_im_kalender(&self, kalender_uuid: &str) -> Ergebnis<Vec<Termin>> {
        let db = self.db();
        let mut abfrage = db.prepare(&format!(
            "{TERMIN_SPALTEN} WHERE kalender_uuid = ?1 AND papierkorb_at IS NULL
             ORDER BY beginn"
        ))?;

        let liste = abfrage
            .query_map(params![kalender_uuid], termin_aus)?
            .collect::<Result<_, _>>()?;

        Ok(liste)
    }

    /// Termine, die im Fenster `[von, bis)` beginnen -- die Abfrage der
    /// Tages- und Wochenansicht.
    ///
    /// Zeichenkettenvergleich, und das geht auf, weil die Zeiten als
    /// `Y-m-d\TH:i:s` abgelegt sind: In dieser Schreibweise ist die
    /// lexikalische Ordnung die zeitliche. Wer das Format aendert, bricht
    /// diese Abfrage still -- deshalb steht es hier.
    ///
    /// **Serientermine sind damit nicht abgedeckt.** Eine `rrule` faltet
    /// dieses Crate nicht aus; das gehoert in die Schicht darueber, die auch
    /// Zeitzonen und Ausnahmen kennt. Ein halb ausgefalteter Serientermin
    /// waere schlimmer als gar keiner.
    pub fn termine_zwischen(&self, von: &str, bis: &str) -> Ergebnis<Vec<Termin>> {
        let db = self.db();
        let mut abfrage = db.prepare(&format!(
            "{TERMIN_SPALTEN} WHERE papierkorb_at IS NULL AND beginn >= ?1 AND beginn < ?2
             ORDER BY beginn"
        ))?;

        let liste = abfrage
            .query_map(params![von, bis], termin_aus)?
            .collect::<Result<_, _>>()?;

        Ok(liste)
    }

    /// Alle Termine ausser denen im Papierkorb, nach Beginn -- fuer das
    /// Zeitfenster des Kalenders (siehe `kalenderbuch.rs`).
    pub fn termine_alle(&self) -> Ergebnis<Vec<Termin>> {
        let db = self.db();
        let mut abfrage = db.prepare(&format!(
            "{TERMIN_SPALTEN} WHERE papierkorb_at IS NULL ORDER BY beginn"
        ))?;

        let liste = abfrage
            .query_map([], termin_aus)?
            .collect::<Result<_, _>>()?;

        Ok(liste)
    }

    // --- Kontakte --------------------------------------------------------

    pub fn kontakt_schreiben(&self, k: &Kontakt, protokoll: Protokoll) -> Ergebnis<()> {
        // `matrix_id`/`meshtastic_id` bleiben als Spalten aus Stand 1 stehen,
        // werden aber nicht mehr geschrieben: Die Wege tragen die Kennungen.
        self.db().execute(
            "INSERT INTO kontakte (uuid, anzeigename, wege, foto_abdruck,
                                   papierkorb_at, geaendert_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT (uuid) DO UPDATE SET
                anzeigename = excluded.anzeigename, wege = excluded.wege,
                foto_abdruck = excluded.foto_abdruck,
                papierkorb_at = excluded.papierkorb_at,
                geaendert_at = excluded.geaendert_at",
            params![
                k.uuid,
                k.anzeigename,
                serde_json::to_string(&k.wege).unwrap_or_else(|_| "[]".into()),
                k.foto,
                k.papierkorb_at,
                jetzt(&k.geaendert_at),
            ],
        )?;

        self.vielleicht_merken(protokoll, &Art::Kontakt, &k.uuid, Was::Da)
    }

    pub fn kontakt(&self, uuid: &str) -> Ergebnis<Option<Kontakt>> {
        Ok(self
            .db()
            .query_row(
                "SELECT uuid, anzeigename, wege, foto_abdruck, papierkorb_at, geaendert_at
                 FROM kontakte WHERE uuid = ?1",
                params![uuid],
                kontakt_aus,
            )
            .optional()?)
    }

    pub fn kontakte(&self) -> Ergebnis<Vec<Kontakt>> {
        let db = self.db();
        let mut abfrage = db.prepare(
            "SELECT uuid, anzeigename, wege, foto_abdruck, papierkorb_at, geaendert_at
             FROM kontakte WHERE papierkorb_at IS NULL ORDER BY anzeigename",
        )?;

        let liste = abfrage
            .query_map([], kontakt_aus)?
            .collect::<Result<_, _>>()?;

        Ok(liste)
    }

    // --- Papierkorb und Loeschen -----------------------------------------

    /// In den Papierkorb -- zurueckholbar, auf beiden Seiten.
    ///
    /// @return ob es die Sache ueberhaupt gab
    pub fn papierkorb(&self, art: &Art, schluessel: &str, protokoll: Protokoll) -> Ergebnis<bool> {
        let Some((tabelle, spalte)) = tabelle(art) else {
            return Ok(false);
        };

        let getroffen = self.db().execute(
            &format!(
                "UPDATE {tabelle} SET papierkorb_at = ?1, geaendert_at = ?1
                 WHERE {spalte} = ?2 AND papierkorb_at IS NULL"
            ),
            params![Utc::now().to_rfc3339(), schluessel],
        )?;

        if getroffen > 0 {
            self.vielleicht_merken(protokoll, art, schluessel, Was::Papierkorb)?;
        }

        Ok(getroffen > 0)
    }

    /// Aus dem Papierkorb zurueck.
    pub fn wiederherstellen(
        &self,
        art: &Art,
        schluessel: &str,
        protokoll: Protokoll,
    ) -> Ergebnis<bool> {
        let Some((tabelle, spalte)) = tabelle(art) else {
            return Ok(false);
        };

        let getroffen = self.db().execute(
            &format!(
                "UPDATE {tabelle} SET papierkorb_at = NULL, geaendert_at = ?1 WHERE {spalte} = ?2"
            ),
            params![Utc::now().to_rfc3339(), schluessel],
        )?;

        if getroffen > 0 {
            self.vielleicht_merken(protokoll, art, schluessel, Was::Da)?;
        }

        Ok(getroffen > 0)
    }

    /// Endgueltig fort.
    ///
    /// **Der Protokolleintrag entsteht auch dann, wenn die Zeile schon weg
    /// war.** Er ist der Grabstein, und ohne ihn haelt die Gegenseite die
    /// Sache fuer nie dagewesen und spielt sie zurueck. Das ist der eine
    /// Fall, in dem "nichts getroffen" trotzdem etwas zu melden hat.
    pub fn loeschen(&self, art: &Art, schluessel: &str, protokoll: Protokoll) -> Ergebnis<bool> {
        let getroffen = if matches!(art, Art::Notizanhang) {
            /*
             * EIN ANHANG HAT EINEN ZWEITEILIGEN SCHLUESSEL -- Mappe und Pfad.
             * `tabelle()` liefert Tabelle und EINE Spalte; das passt fuer
             * alles andere und fuer ihn nicht.
             *
             * Ohne diesen Zweig fiele er in `Ok(false)` und liesse sich nie
             * loeschen: Der Grabstein von drueben kaeme bei jedem Lauf
             * wieder, und die Zuordnung bliebe stehen -- ein Bild im Text,
             * das drueben laengst geloest ist.
             */
            let Some((mappe, pfad)) = schluessel.split_once('|') else {
                return Ok(false);
            };

            self.db().execute(
                "DELETE FROM anhaenge WHERE mappe = ?1 AND pfad = ?2",
                params![mappe, pfad],
            )?
        } else {
            let Some((tabelle, spalte)) = tabelle(art) else {
                return Ok(false);
            };

            self.db().execute(
                &format!("DELETE FROM {tabelle} WHERE {spalte} = ?1"),
                params![schluessel],
            )?
        };

        // Angenommenes nur, wenn es wirklich etwas loeschte -- sonst liefe
        // ein Grabstein zwischen drei Geraeten ewig im Kreis.
        if getroffen > 0 || !matches!(protokoll, Protokoll::Von(_)) {
            self.vielleicht_merken(protokoll, art, schluessel, Was::Fort)?;
        }

        Ok(getroffen > 0)
    }

    fn vielleicht_merken(
        &self,
        protokoll: Protokoll,
        art: &Art,
        schluessel: &str,
        was: Was,
    ) -> Ergebnis<()> {
        match protokoll {
            Protokoll::Merken => self.merken(art, schluessel, was),
            Protokoll::Still => Ok(()),
            Protokoll::Von(herkunft) => self.merken_von(art, schluessel, was, Some(herkunft)),
        }
    }
}

const TERMIN_SPALTEN: &str = "SELECT uuid, kalender_uuid, titel, beschreibung, ort, beginn, ende,
            ganztags, rrule, rrule_bis, exdates, papierkorb_at, geaendert_at
     FROM termine";

const TERMIN_SPALTEN_WHERE: &str =
    "SELECT uuid, kalender_uuid, titel, beschreibung, ort, beginn, ende,
            ganztags, rrule, rrule_bis, exdates, papierkorb_at, geaendert_at
     FROM termine WHERE uuid = ?1";

/// Leer heisst "jetzt". So muss kein Aufrufer eine Zeit setzen, nur damit
/// etwas gespeichert wird -- und wer eine mitbringt (weil er einen fremden
/// Stand uebernimmt), behaelt sie.
fn jetzt(wert: &str) -> String {
    if wert.is_empty() {
        Utc::now().to_rfc3339()
    } else {
        wert.to_string()
    }
}

fn notiz_aus(z: &Row) -> rusqlite::Result<Notiz> {
    Ok(Notiz {
        zk_id: z.get(0)?,
        titel: z.get(1)?,
        inhalt: z.get(2)?,
        mappe: z.get(3)?,
        papierkorb_at: z.get(4)?,
        geaendert_at: z.get(5)?,
    })
}

fn kalender_aus(z: &Row) -> rusqlite::Result<Kalender> {
    Ok(Kalender {
        uuid: z.get(0)?,
        name: z.get(1)?,
        farbe: z.get(2)?,
        abo_url: z.get(3)?,
        zuletzt_geholt: z.get(4)?,
        papierkorb_at: z.get(5)?,
        geaendert_at: z.get(6)?,
    })
}

fn termin_aus(z: &Row) -> rusqlite::Result<Termin> {
    Ok(Termin {
        uuid: z.get(0)?,
        kalender_uuid: z.get(1)?,
        felder: Terminfelder {
            titel: z.get(2)?,
            beschreibung: z.get(3)?,
            ort: z.get(4)?,
            beginn: z.get(5)?,
            ende: z.get(6)?,
            ganztags: z.get(7)?,
            rrule: z.get(8)?,
            rrule_bis: z.get(9)?,
            exdates: z.get(10)?,
        },
        papierkorb_at: z.get(11)?,
        geaendert_at: z.get(12)?,
    })
}

fn kontakt_aus(z: &Row) -> rusqlite::Result<Kontakt> {
    let wege: String = z.get(2)?;

    Ok(Kontakt {
        uuid: z.get(0)?,
        anzeigename: z.get(1)?,
        // Eine unlesbare Liste ergibt keine Wege statt eines Absturzes: Der
        // Name bleibt lesbar, und der naechste Abgleich bringt die Wege zurueck.
        wege: serde_json::from_str(&wege).unwrap_or_default(),
        foto: z.get(3)?,
        papierkorb_at: z.get(4)?,
        geaendert_at: z.get(5)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn speicher() -> Speicher {
        Speicher::im_arbeitsspeicher().unwrap()
    }

    fn termin(uuid: &str, kalender: &str, titel: &str, beginn: &str) -> Termin {
        Termin {
            uuid: uuid.into(),
            kalender_uuid: kalender.into(),
            felder: Terminfelder {
                titel: titel.into(),
                beginn: beginn.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn eine_notiz_kommt_zurueck_wie_sie_hineinging() {
        let s = speicher();
        let notiz = Notiz {
            zk_id: "zk-1".into(),
            titel: "Einkauf".into(),
            inhalt: "Milch\nBrot".into(),
            mappe: Some("Haushalt".into()),
            ..Default::default()
        };

        s.notiz_schreiben(&notiz, Protokoll::Merken).unwrap();
        let zurueck = s.notiz("zk-1").unwrap().unwrap();

        assert_eq!(zurueck.titel, "Einkauf");
        assert_eq!(zurueck.inhalt, "Milch\nBrot");
        assert_eq!(zurueck.mappe.as_deref(), Some("Haushalt"));
        assert!(!zurueck.geaendert_at.is_empty(), "leer heisst jetzt");
    }

    #[test]
    fn zweimal_schreiben_ergibt_eine_zeile() {
        let s = speicher();

        s.notiz_schreiben(&Notiz::mit_id("zk-1"), Protokoll::Merken)
            .unwrap();
        s.notiz_schreiben(
            &Notiz {
                titel: "Neu".into(),
                ..Notiz::mit_id("zk-1")
            },
            Protokoll::Merken,
        )
        .unwrap();

        assert_eq!(s.notizen().unwrap().len(), 1);
        assert_eq!(s.notiz("zk-1").unwrap().unwrap().titel, "Neu");
    }

    #[test]
    fn der_papierkorb_ist_nicht_das_loeschen() {
        let s = speicher();
        s.notiz_schreiben(&Notiz::mit_id("zk-1"), Protokoll::Merken)
            .unwrap();

        s.papierkorb(&Art::Notiz, "zk-1", Protokoll::Merken)
            .unwrap();

        assert!(s.notizen().unwrap().is_empty(), "aus der Liste fort");
        let notiz = s.notiz("zk-1").unwrap().unwrap();
        assert!(notiz.im_papierkorb(), "aber noch da");

        s.wiederherstellen(&Art::Notiz, "zk-1", Protokoll::Merken)
            .unwrap();

        assert_eq!(s.notizen().unwrap().len(), 1, "und zurueckholbar");
    }

    #[test]
    fn loeschen_hinterlaesst_einen_grabstein() {
        let s = speicher();
        s.notiz_schreiben(&Notiz::mit_id("zk-1"), Protokoll::Merken)
            .unwrap();

        s.loeschen(&Art::Notiz, "zk-1", Protokoll::Merken).unwrap();

        assert!(s.notiz("zk-1").unwrap().is_none());

        let (liste, _, _) = s.aenderungen_seit(0).unwrap();

        assert_eq!(liste.last().unwrap().was, Was::Fort);
    }

    #[test]
    fn ein_grabstein_entsteht_auch_fuer_etwas_das_es_hier_nie_gab() {
        // Der Fall: Drueben geloescht, hier war es nie angekommen. Ohne
        // Grabstein meldete dieses Geraet die Sache spaeter als Neuzugang.
        let s = speicher();

        assert!(!s
            .loeschen(&Art::Notiz, "nie-dagewesen", Protokoll::Merken)
            .unwrap());

        let (liste, _, _) = s.aenderungen_seit(0).unwrap();

        assert_eq!(liste.len(), 1);
        assert_eq!(liste[0].was, Was::Fort);
    }

    /// EIN ANHANG HAT KEINEN PAPIERKORB, und das ist kein Mangel: Drueben hat
    /// `note_assets` auch keinen. Eine Zuordnung ist da oder sie ist weg --
    /// eine weggelegte waere ein Bild, das im Text steht und trotzdem nicht
    /// gelten soll. Geloescht werden kann er sehr wohl (seit dem 16.09.2026).
    #[test]
    fn eine_art_die_dieses_programm_nicht_traegt_wird_uebergangen() {
        let s = speicher();

        assert!(!s
            .papierkorb(&Art::Notizanhang, "a-1", Protokoll::Merken)
            .unwrap());
        assert!(!s
            .loeschen(&Art::Unbekannt("rezept".into()), "m-1", Protokoll::Merken)
            .unwrap());

        let (liste, _, _) = s.aenderungen_seit(0).unwrap();

        assert!(liste.is_empty(), "und ergibt kein Protokoll");
    }

    #[test]
    fn ein_termin_traegt_die_neun_felder_ueber_die_gehasht_wird() {
        let s = speicher();
        let mut t = termin("t-1", "k-1", "Zahnarzt", "2026-09-10T09:00:00");
        t.felder.ende = "2026-09-10T09:30:00".into();

        s.termin_schreiben(&t, Protokoll::Merken).unwrap();
        let zurueck = s.termin("t-1").unwrap().unwrap();

        assert_eq!(zurueck.felder, t.felder);
        assert_eq!(
            zurueck.abdruck(),
            "8a25a4ca136e584ecf499150247f7ae606621c87ad4d2d01f9d4636082c969a9",
            "derselbe Abdruck wie drueben in PHP -- ueber den Speicher hinweg"
        );
    }

    #[test]
    fn ganztags_ueberlebt_die_datei() {
        // SQLite kennt kein BOOLEAN. Ginge der Schalter als 0/1 hinein und
        // als Zeichenkette heraus, waere der Abdruck ein anderer -- und jeder
        // ganztaegige Termin ein ewiger Konflikt.
        let s = speicher();
        let mut t = termin("t-1", "k-1", "Urlaub", "2026-09-10T00:00:00");
        t.felder.ganztags = true;

        s.termin_schreiben(&t, Protokoll::Merken).unwrap();

        assert!(s.termin("t-1").unwrap().unwrap().felder.ganztags);
    }

    #[test]
    fn ein_termin_darf_vor_seinem_kalender_ankommen() {
        // Kein Fremdschluessel: Beim Ziehen kommt die Reihenfolge vom Server,
        // und eine abgewiesene Zeile waere ein verlorener Termin.
        let s = speicher();

        s.termin_schreiben(
            &termin("t-1", "noch-nicht-da", "Zahnarzt", "2026-09-10T09:00:00"),
            Protokoll::Still,
        )
        .unwrap();

        assert!(s.termin("t-1").unwrap().is_some());
        assert!(s.kalender("noch-nicht-da").unwrap().is_none());
    }

    #[test]
    fn termine_im_fenster_kommen_in_zeitlicher_ordnung() {
        let s = speicher();

        s.termin_schreiben(
            &termin("t-3", "k", "Drittens", "2026-09-10T18:00:00"),
            Protokoll::Still,
        )
        .unwrap();
        s.termin_schreiben(
            &termin("t-1", "k", "Erstens", "2026-09-10T08:00:00"),
            Protokoll::Still,
        )
        .unwrap();
        s.termin_schreiben(
            &termin("t-2", "k", "Zweitens", "2026-09-10T12:00:00"),
            Protokoll::Still,
        )
        .unwrap();
        s.termin_schreiben(
            &termin("t-x", "k", "Tags darauf", "2026-09-11T08:00:00"),
            Protokoll::Still,
        )
        .unwrap();

        let tag = s
            .termine_zwischen("2026-09-10T00:00:00", "2026-09-11T00:00:00")
            .unwrap();

        assert_eq!(
            tag.iter().map(|t| t.uuid.as_str()).collect::<Vec<_>>(),
            vec!["t-1", "t-2", "t-3"],
            "lexikalisch = zeitlich, solange das Format bleibt"
        );
    }

    #[test]
    fn ein_termin_im_papierkorb_faellt_aus_der_tagesansicht() {
        let s = speicher();
        s.termin_schreiben(
            &termin("t-1", "k", "Abgesagt", "2026-09-10T08:00:00"),
            Protokoll::Still,
        )
        .unwrap();

        s.papierkorb(&Art::Termin, "t-1", Protokoll::Merken)
            .unwrap();

        assert!(s
            .termine_zwischen("2026-09-10T00:00:00", "2026-09-11T00:00:00")
            .unwrap()
            .is_empty());
    }

    #[test]
    fn ein_kontakt_ohne_kennung_ist_erlaubt() {
        // Ein Kontakt ist eine Notiz darueber, wie jemand erreichbar ist --
        // keine Berechtigung. Wer nur einen Namen hat, steht trotzdem drin.
        let s = speicher();

        s.kontakt_schreiben(
            &Kontakt {
                uuid: "c-1".into(),
                anzeigename: "Hannah".into(),
                ..Default::default()
            },
            Protokoll::Merken,
        )
        .unwrap();

        let k = s.kontakt("c-1").unwrap().unwrap();

        assert_eq!(k.anzeigename, "Hannah");
        assert!(k.wege.is_empty());
    }

    #[test]
    fn der_kontaktabdruck_verwechselt_die_felder_nicht() {
        // Ohne Trennzeichen waeren ("ab", "c") und ("a", "bc") dasselbe.
        let a = Kontakt {
            anzeigename: "ab".into(),
            foto: Some("c".into()),
            ..Default::default()
        };
        let b = Kontakt {
            anzeigename: "a".into(),
            foto: Some("bc".into()),
            ..Default::default()
        };

        assert_ne!(a.abdruck(), b.abdruck());
    }

    #[test]
    fn wege_kommen_in_ihrer_reihenfolge_zurueck() {
        let s = speicher();
        let wege = vec![
            Weg {
                art: "phone".into(),
                beschriftung: Some("mobil".into()),
                wert: "0171".into(),
            },
            Weg {
                art: "matrix".into(),
                beschriftung: None,
                wert: "@h:m.org".into(),
            },
            Weg {
                art: "vcard:nickname".into(),
                beschriftung: None,
                wert: "Hanni".into(),
            },
            Weg {
                art: "phone".into(),
                beschriftung: Some("dienst".into()),
                wert: "030".into(),
            },
        ];

        s.kontakt_schreiben(
            &Kontakt {
                uuid: "c-1".into(),
                anzeigename: "Hannah".into(),
                wege: wege.clone(),
                ..Default::default()
            },
            Protokoll::Merken,
        )
        .unwrap();

        let k = s.kontakt("c-1").unwrap().unwrap();
        assert_eq!(k.wege, wege);
        assert_eq!(k.kennung("matrix"), Some("@h:m.org"));
        assert_eq!(k.kennung("meshtastic"), None);
    }
}
