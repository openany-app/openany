//! Die Form, in der openany und dieses Programm miteinander reden.
//!
//! Ein Delta-Eintrag hat drei Felder, die IMMER da sind -- `type`, `key`,
//! `action` -- und je nach Art beliebig viele weitere. Deshalb steht hier
//! kein Typ je Art, sondern einer mit einer Tasche: die drei benannt, der
//! Rest roh.
//!
//! **Warum nicht je Art eine Struktur.** Weil dieser Client nicht der einzige
//! Abnehmer ist. Die Delta-API bedient jede Fassung der App, die unterwegs
//! ist, und ein Server, der ein Feld ergaenzt, darf ein Programm auf einem Telefon
//! nicht ausser Gefecht setzen, das seit einem halben Jahr nicht aktualisiert
//! wurde. Mit einer festen Struktur je Art waere `serde` bei jedem neuen Feld
//! grosszuegig (`deny_unknown_fields` waere es nicht) -- aber bei einer neuen
//! ART bliebe nur der Abbruch. Mit der Tasche bleibt der Eintrag lesbar, auch
//! wenn niemand ihn versteht, und wird uebergangen statt zu reissen.
//!
//! Die getippten Sichten darauf ([`crate::Terminfelder`] etwa) sind Lesarten,
//! nicht der Vertrag selbst.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Was mit einer Sache geschehen ist.
///
/// **Papierkorb und Loeschen sind nicht dasselbe**, und das ist die
/// teuerste Verwechslung an dieser Stelle: Ein Griff in den Papierkorb auf
/// der einen Seite wuerde auf der anderen zur endgueltigen Vernichtung -- und
/// der Papierkorb ist genau die Zusage, dass das nicht passiert.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Was {
    /// `upsert` -- da und lebendig.
    Da,
    /// `trash` -- im Papierkorb, drueben wie hier zurueckholbar.
    Papierkorb,
    /// `delete` -- endgueltig fort. Pflanzt sich fort, sonst wuechse der
    /// lokale Speicher ewig weiter.
    Fort,
}

impl Was {
    pub fn ueber_die_leitung(&self) -> &'static str {
        match self {
            Self::Da => "upsert",
            Self::Papierkorb => "trash",
            Self::Fort => "delete",
        }
    }

    /// Oeffentlich wie das Gegenstueck bei [`Art`]: Der lokale Speicher legt
    /// diese Woerter in einer Spalte ab und liest sie zurueck.
    pub fn aus(wert: &str) -> Option<Self> {
        match wert {
            "upsert" => Some(Self::Da),
            "trash" => Some(Self::Papierkorb),
            "delete" => Some(Self::Fort),
            _ => None,
        }
    }
}

/// Die Arten, die ueber den Abgleich gehen.
///
/// Stufe 4 traegt `note`, `calendar`, `event` und `contact`; die uebrigen
/// stehen mit hier, weil der Server sie schickt und ein Client, der sie nicht
/// benennen kann, sie auch nicht sauber uebergehen kann.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Art {
    Notiz,
    Kalender,
    Termin,
    Kontakt,
    Album,
    Datei,
    Notizanhang,
    Medium,
    /// Eine Art, die dieser Stand nicht kennt -- ein neuerer Server. Der
    /// Eintrag wird uebergangen, nicht verworfen: Die Marke darf trotzdem
    /// vorruecken, sonst haengt der Abgleich fuer immer an dieser Seite.
    Unbekannt(String),
}

impl Art {
    pub fn ueber_die_leitung(&self) -> &str {
        match self {
            Self::Notiz => "note",
            Self::Kalender => "calendar",
            Self::Termin => "event",
            Self::Kontakt => "contact",
            Self::Album => "album",
            Self::Datei => "file_node",
            Self::Notizanhang => "note_asset",
            Self::Medium => "media",
            Self::Unbekannt(s) => s,
        }
    }

    pub fn aus(wert: &str) -> Self {
        match wert {
            "note" => Self::Notiz,
            "calendar" => Self::Kalender,
            "event" => Self::Termin,
            "contact" => Self::Kontakt,
            "album" => Self::Album,
            "file_node" => Self::Datei,
            "note_asset" => Self::Notizanhang,
            "media" => Self::Medium,
            anderes => Self::Unbekannt(anderes.to_string()),
        }
    }

    /// Bringt diese Art Bytes mit, die einzeln zu holen sind?
    ///
    /// Eine Notiz bringt ihren Text nicht im Eintrag mit -- er kommt ueber
    /// `GET /api/sync/content`. Bilder und Dateien ebenso. Ein Termin dagegen
    /// reist vollstaendig: neun kurze Felder wiegen weniger als die zweite
    /// Anfrage, die sie holen wuerde.
    pub fn holt_inhalt(&self) -> bool {
        matches!(self, Self::Notiz | Self::Medium | Self::Datei)
    }
}

/// Ein Eintrag im Delta -- die drei festen Felder und der Rest als Tasche.
#[derive(Debug, Clone)]
pub struct Eintrag {
    pub art: Art,
    pub key: String,
    pub was: Was,
    /// Alles ausser `type`, `key` und `action`. Beim Hinausschicken wird es
    /// wieder flach danebengelegt, so wie der Server es ausgibt.
    pub felder: Map<String, Value>,
}

impl Eintrag {
    pub fn neu(art: Art, key: impl Into<String>, was: Was) -> Self {
        Self {
            art,
            key: key.into(),
            was,
            felder: Map::new(),
        }
    }

    pub fn mit(mut self, name: &str, wert: impl Into<Value>) -> Self {
        self.felder.insert(name.into(), wert.into());
        self
    }

    pub fn text(&self, name: &str) -> Option<&str> {
        self.felder.get(name).and_then(Value::as_str)
    }

    /// Ob das Feld ueberhaupt dasteht -- auch als `null`.
    ///
    /// ABWESEND IST NICHT LEER, und an mehreren Stellen im Vertrag haengt
    /// genau dieser Unterschied: Ein fehlendes Feld heisst "unbekannt, lass
    /// stehen", ein `null` heisst "ausdruecklich nichts". `text()` gibt fuer
    /// beides `None` und kann sie deshalb nicht auseinanderhalten.
    ///
    /// Wer das verwechselt, laesst eine aeltere Gegenstelle bei jedem Lauf
    /// loeschen, was sie nur nicht kennt.
    pub fn hat(&self, name: &str) -> bool {
        self.felder.contains_key(name)
    }

    /// Einen Eintrag aus der flachen Form lesen -- `None`, wenn `type`, `key`
    /// oder eine bekannte `action` fehlen.
    pub fn aus_der_leitung(wert: &Value) -> Option<Self> {
        Delta::eintrag_lesen(wert)
    }

    /// Die flache Form fuer `POST /api/sync/apply`.
    ///
    /// Flach und nicht verschachtelt, weil der Server genau diese Form
    /// ausgibt: "Wer beide gelesen hat, hat den ganzen Vertrag gelesen." Ein
    /// zweites Format fuer dieselben Dinge waere die Stelle, an der Hin- und
    /// Rueckweg auseinanderliefen.
    pub fn ueber_die_leitung(&self) -> Value {
        let mut m = self.felder.clone();

        // Zuletzt eingefuegt und damit die drei festen Felder als Sieger:
        // Ein Feld `type` in der Tasche -- wie auch immer es dorthin kaeme --
        // darf die Art nicht ueberschreiben.
        m.insert("type".into(), self.art.ueber_die_leitung().into());
        m.insert("key".into(), self.key.clone().into());
        m.insert("action".into(), self.was.ueber_die_leitung().into());

        Value::Object(m)
    }
}

/// Wie voll die Box ist -- faehrt seit Stufe 4 in jeder Delta-Antwort mit.
///
/// Damit kann das Programm warnen, BEVOR es etwas versucht, statt einen
/// Fehlschlag zu erklaeren. Nach einer Woche offline haengen sonst
/// womoeglich hundert Dateien an derselben Absage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Speicherstand {
    #[serde(rename = "used")]
    pub belegt: u64,
    /// `None` heisst **unbegrenzt** -- so steht es in `users.storage_quota`.
    /// Eine 0 hiesse "kein Platz", das Gegenteil. Wer die beiden verwechselt,
    /// baut ein Programm, das entweder nie oder immer warnt.
    #[serde(rename = "quota")]
    pub grenze: Option<u64>,
}

impl Speicherstand {
    /// Passen `bytes` noch? Ohne Grenze immer.
    pub fn passt(&self, bytes: u64) -> bool {
        match self.grenze {
            None => true,
            Some(grenze) => self.belegt.saturating_add(bytes) <= grenze,
        }
    }

    /// Anteil von 0.0 bis 1.0; ohne Grenze `None`.
    pub fn anteil(&self) -> Option<f64> {
        match self.grenze {
            None | Some(0) => None,
            Some(grenze) => Some(self.belegt as f64 / grenze as f64),
        }
    }
}

/// Eine Seite des Delta-Endpunkts.
#[derive(Debug, Clone)]
pub struct Delta {
    pub cursor: i64,
    pub more: bool,
    pub eintraege: Vec<Eintrag>,
    /// Fehlt bei einer aelteren Gegenstelle -- der Speicherstand kam erst mit
    /// Stufe 4 dazu. Deshalb `Option` und keine 0: "weiss ich nicht" ist
    /// etwas anderes als "nichts belegt".
    pub speicher: Option<Speicherstand>,
}

impl Delta {
    /// Eine Antwort lesen.
    ///
    /// **Was hier nicht passiert: abbrechen, weil ein Eintrag fremd aussieht.**
    /// Ein Eintrag ohne `type`, `key` oder mit einer unbekannten `action`
    /// wird uebergangen. Der Rest der Seite und vor allem die Marke bleiben
    /// gueltig -- sonst haengt der Abgleich fuer immer an genau dieser Seite,
    /// und zwar auf jedem Geraet gleichzeitig.
    pub fn lesen(roh: &Value) -> Result<Self, Vertragsfehler> {
        let obj = roh.as_object().ok_or(Vertragsfehler::KeinObjekt)?;

        let cursor = obj
            .get("cursor")
            .and_then(Value::as_i64)
            .ok_or(Vertragsfehler::OhneMarke)?;

        let mut eintraege = Vec::new();

        if let Some(liste) = obj.get("entries").and_then(Value::as_array) {
            for wert in liste {
                if let Some(e) = Self::eintrag_lesen(wert) {
                    eintraege.push(e);
                }
            }
        }

        Ok(Self {
            cursor,
            more: obj.get("more").and_then(Value::as_bool).unwrap_or(false),
            eintraege,
            speicher: obj
                .get("storage")
                .and_then(|s| serde_json::from_value(s.clone()).ok()),
        })
    }

    fn eintrag_lesen(wert: &Value) -> Option<Eintrag> {
        let obj = wert.as_object()?;
        let art = Art::aus(obj.get("type")?.as_str()?);
        let key = obj.get("key")?.as_str()?.to_string();
        let was = Was::aus(obj.get("action")?.as_str()?)?;

        let mut felder = obj.clone();
        felder.remove("type");
        felder.remove("key");
        felder.remove("action");

        Some(Eintrag {
            art,
            key,
            was,
            felder,
        })
    }
}

/// Was ein Ergebnis von `POST /api/sync/apply` je Eintrag sagt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ergebnis {
    #[serde(rename = "type")]
    pub art: String,
    pub key: String,
    /// `created`, `updated`, `unchanged`, `conflict`, `skipped`, `ignored`,
    /// `trashed`, `deleted`. Absichtlich als Zeichenkette und nicht als
    /// Aufzaehlung: Der Server darf hier etwas Neues sagen, ohne dass ein
    /// altes Programm den ganzen Stapel verwirft.
    pub action: String,
    #[serde(default)]
    pub conflict: bool,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Vertragsfehler {
    #[error("The response was not an object.")]
    KeinObjekt,
    /// Ohne Marke gibt es kein Weiterruecken -- das ist der eine Fall, in dem
    /// die Seite wirklich unbrauchbar ist.
    #[error("The response carried no cursor (`cursor`).")]
    OhneMarke,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn liest_eine_seite_mit_speicherstand() {
        let delta = Delta::lesen(&json!({
            "cursor": 4711,
            "more": true,
            "entries": [
                {"type": "calendar", "key": "k-1", "action": "upsert",
                 "name": "Privat", "color": "#005F60"},
                {"type": "note", "key": "zk-1", "action": "trash",
                 "deleted_at": "2026-09-06T10:00:00+02:00"}
            ],
            "storage": {"used": 1024500, "quota": 10485760}
        }))
        .unwrap();

        assert_eq!(delta.cursor, 4711);
        assert!(delta.more);
        assert_eq!(delta.eintraege.len(), 2);
        assert_eq!(delta.eintraege[0].art, Art::Kalender);
        assert_eq!(delta.eintraege[0].text("name"), Some("Privat"));
        assert_eq!(delta.eintraege[1].was, Was::Papierkorb);
        assert_eq!(delta.speicher.unwrap().belegt, 1024500);
    }

    #[test]
    fn eine_aeltere_gegenstelle_ohne_speicherstand_ist_kein_fehler() {
        let delta = Delta::lesen(&json!({"cursor": 1, "more": false, "entries": []})).unwrap();

        assert!(delta.speicher.is_none(), "nicht 0 -- weiss ich nicht");
    }

    #[test]
    fn ohne_marke_ist_die_seite_unbrauchbar() {
        let fehler = Delta::lesen(&json!({"entries": []})).unwrap_err();

        assert_eq!(fehler, Vertragsfehler::OhneMarke);
    }

    #[test]
    fn eine_unbekannte_art_reisst_die_seite_nicht_mit() {
        // Der Server ist neuer als dieses Programm. Der fremde Eintrag muss
        // lesbar bleiben -- uebergangen wird er eine Ebene hoeher, aber die
        // Marke darf trotzdem vorruecken.
        let delta = Delta::lesen(&json!({
            "cursor": 9, "more": false,
            "entries": [
                {"type": "rezept", "key": "r-1", "action": "upsert"},
                {"type": "note", "key": "zk-1", "action": "upsert"}
            ]
        }))
        .unwrap();

        assert_eq!(delta.eintraege.len(), 2);
        assert_eq!(delta.eintraege[0].art, Art::Unbekannt("rezept".into()));
        assert_eq!(delta.eintraege[1].art, Art::Notiz);
    }

    #[test]
    fn ein_kaputter_eintrag_wird_uebergangen_und_der_rest_bleibt() {
        let delta = Delta::lesen(&json!({
            "cursor": 9, "more": false,
            "entries": [
                {"key": "ohne-art", "action": "upsert"},
                {"type": "note", "action": "upsert"},
                {"type": "note", "key": "zk-1", "action": "verschieben"},
                {"type": "note", "key": "zk-2", "action": "upsert"}
            ]
        }))
        .unwrap();

        assert_eq!(delta.eintraege.len(), 1);
        assert_eq!(delta.eintraege[0].key, "zk-2");
        assert_eq!(delta.cursor, 9, "die Marke rueckt trotzdem vor");
    }

    #[test]
    fn ein_eintrag_geht_flach_hinaus() {
        let eintrag = Eintrag::neu(Art::Kontakt, "c-1", Was::Da)
            .mit("display_name", "Hannah")
            .mit("matrix_id", "@hannah:matrix.org");

        assert_eq!(
            eintrag.ueber_die_leitung(),
            json!({
                "type": "contact", "key": "c-1", "action": "upsert",
                "display_name": "Hannah", "matrix_id": "@hannah:matrix.org"
            })
        );
    }

    #[test]
    fn die_drei_festen_felder_lassen_sich_nicht_aus_der_tasche_ueberschreiben() {
        let eintrag = Eintrag::neu(Art::Notiz, "zk-1", Was::Da).mit("type", "contact");

        assert_eq!(eintrag.ueber_die_leitung()["type"], json!("note"));
    }

    #[test]
    fn unbegrenzt_ist_nicht_dasselbe_wie_voll() {
        let ohne = Speicherstand {
            belegt: 999_999_999,
            grenze: None,
        };
        let leer = Speicherstand {
            belegt: 0,
            grenze: Some(0),
        };

        assert!(ohne.passt(1_000_000), "None heisst unbegrenzt");
        assert_eq!(ohne.anteil(), None);
        assert!(!leer.passt(1), "0 Byte Kontingent heisst: nichts geht");
    }

    #[test]
    fn passt_rechnet_die_neue_datei_mit() {
        let stand = Speicherstand {
            belegt: 900,
            grenze: Some(1000),
        };

        assert!(stand.passt(100));
        assert!(
            !stand.passt(101),
            "nicht nur 'ist noch Platz', sondern 'passt das'"
        );
    }

    #[test]
    fn nur_diese_drei_arten_holen_bytes_nach() {
        assert!(Art::Notiz.holt_inhalt());
        assert!(Art::Medium.holt_inhalt());
        assert!(Art::Datei.holt_inhalt());
        // Ein Termin reist vollstaendig -- eine zweite Anfrage je Termin
        // waere teurer als die neun Felder, die sie holen soll.
        assert!(!Art::Termin.holt_inhalt());
        assert!(!Art::Kalender.holt_inhalt());
        assert!(!Art::Kontakt.holt_inhalt());
    }
}
