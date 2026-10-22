//! Kalender und Termine Ende-zu-Ende (docs/plan-kalender-e2e.md, Schritt 4b).
//!
//! Dasselbe Muster wie [`crate::adressbuch`]: **Uebersetzt wird an der
//! Grenze zum Server, sonst nirgends.** Speicher, Laeufer und der Abgleich
//! vor Ort kennen Kalender und Termine im Klartext; erst die
//! [`ServerGegenstelle`](crate::servergegenstelle::ServerGegenstelle)
//! verschliesst beim Schieben und oeffnet beim Ziehen. Verschluesselt wird an
//! den Konto-Schluessel -- jedes freigegebene eigene Geraet liest.
//!
//! **Der Abdruck.** Der Server entscheidet bei Terminen weiter dreiseitig,
//! sieht aber nur einen verdeckten Abdruck
//! ([`Kontoschluessel::abdruck_verdecken`](openany_direktpost::Kontoschluessel::abdruck_verdecken)).
//! Hinaus geht der verdeckte, herein kommt -- nach dem Oeffnen -- wieder der
//! Klartext-Abdruck, mit dem der Laeufer rechnet. Solange das Konto nicht
//! umgestellt ist, liegt der Termin drueben vielleicht noch im Klartext;
//! dann reist die Basis zusaetzlich offen mit (`base_klar`), denn nur mit ihr
//! erkennt der Server das Umschluesseln als Aenderung.

use openany_client::{Art, Eintrag, Terminfelder, Was};
use openany_direktpost::Verschluesselt;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::adressbuch::Adressbuch;

#[derive(Debug, Serialize, Deserialize)]
struct KalenderInhalt {
    v: u8,
    uuid: String,
    name: String,
    color: String,
    /// `None`: ein eigener Kalender, kein Abo.
    #[serde(default)]
    sync_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TerminInhalt {
    v: u8,
    uuid: String,
    /// Auch drinnen: Der Server kann den Termin nicht in einen anderen
    /// Kalender schieben.
    calendar: String,
    /// Die neun Felder unter ihren Leitungsnamen ([`Terminfelder::als_eintrag`]).
    felder: Map<String, Value>,
}

/// Ein Kalender oder Termin, den der Server verschluesselt liefert?
pub(crate) fn ist_verschluesselt(e: &Eintrag) -> bool {
    matches!(e.art, Art::Kalender | Art::Termin)
        && e.was == Was::Da
        && e.felder.contains_key("inhalt")
}

fn chiffrat(e: &Eintrag) -> Option<(Verschluesselt, u32, i64)> {
    Some((
        Verschluesselt {
            inhalt: e.text("inhalt")?.to_string(),
            signatur: e.text("signatur")?.to_string(),
        },
        e.felder.get("version").and_then(Value::as_u64)? as u32,
        e.felder.get("geraet").and_then(Value::as_i64)?,
    ))
}

fn hinaus(buch: &Adressbuch, e: &Eintrag, klartext: &[u8]) -> Result<Eintrag, String> {
    let art = e.art.ueber_die_leitung();
    let (v, version) =
        openany_direktpost::sache_verschluesseln(&buch.ich, &buch.konto, art, &e.key, klartext)
            .map_err(|e| e.to_string())?;
    let mut aus = Eintrag::neu(e.art.clone(), e.key.clone(), Was::Da)
        .mit("inhalt", v.inhalt)
        .mit("signatur", v.signatur)
        .mit("geraet", buch.geraet)
        .mit("version", version);
    for feld in ["updated_at", "origin"] {
        if let Some(w) = e.felder.get(feld) {
            aus = aus.mit(feld, w.clone());
        }
    }
    Ok(aus)
}

fn herein(buch: &Adressbuch, e: &Eintrag) -> Option<Vec<u8>> {
    let (v, version, von) = chiffrat(e)?;
    let ed25519 = buch.geraete.get(&von)?;
    openany_direktpost::sache_oeffnen(
        &buch.konto,
        e.art.ueber_die_leitung(),
        &e.key,
        version,
        &v,
        ed25519,
    )
    .ok()
}

impl Adressbuch {
    /// Einen Kalender zum Hinausschicken verschliessen.
    pub fn kalender_verschliessen(&self, e: &Eintrag) -> Result<Eintrag, String> {
        let klartext = serde_json::to_vec(&KalenderInhalt {
            v: 1,
            uuid: e.key.clone(),
            name: e.text("name").unwrap_or("Kalender").to_string(),
            color: e.text("color").unwrap_or_default().to_string(),
            sync_url: e
                .text("sync_url")
                .filter(|s| !s.trim().is_empty())
                .map(str::to_string),
        })
        .map_err(|e| e.to_string())?;
        hinaus(self, e, &klartext)
    }

    /// Einen gezogenen Kalender oeffnen -- `None`, wenn er nicht aufgeht.
    pub fn kalender_oeffnen(&self, e: &Eintrag) -> Option<Eintrag> {
        let inhalt: KalenderInhalt = serde_json::from_slice(&herein(self, e)?).ok()?;
        if inhalt.uuid != e.key {
            return None;
        }
        let mut aus = Eintrag::neu(Art::Kalender, e.key.clone(), Was::Da)
            .mit("name", inhalt.name)
            .mit("color", inhalt.color)
            // Immer dabei: `null` heisst „kein Abo", ein fehlendes Feld hiesse
            // „unbekannt" und liesse ein altes Abo stehen.
            .mit("sync_url", inhalt.sync_url.map_or(Value::Null, Value::from));
        if let Some(t) = e.felder.get("updated_at") {
            aus = aus.mit("updated_at", t.clone());
        }
        Some(aus)
    }

    /// Einen Termin verschliessen. `klar`: die Basis zusaetzlich offen
    /// mitgeben (Konto noch nicht umgestellt).
    pub fn termin_verschliessen(&self, e: &Eintrag, klar: bool) -> Result<Eintrag, String> {
        let kalender = e
            .text("calendar")
            .ok_or("event without calendar")?
            .to_string();
        let felder = Terminfelder::aus_eintrag(&e.felder);
        let abdruck = felder.abdruck();
        let klartext = serde_json::to_vec(&TerminInhalt {
            v: 1,
            uuid: e.key.clone(),
            calendar: kalender.clone(),
            felder: felder.als_eintrag(),
        })
        .map_err(|e| e.to_string())?;

        let basis = e.text("base_hash").filter(|s| !s.is_empty());
        let mut aus = hinaus(self, e, &klartext)?
            .mit("calendar", kalender)
            .mit("hash", self.konto.abdruck_verdecken(&e.key, &abdruck))
            .mit(
                "base_hash",
                basis.map_or(Value::Null, |b| {
                    self.konto.abdruck_verdecken(&e.key, b).into()
                }),
            );
        if klar {
            aus = aus.mit("base_klar", basis.map_or(Value::Null, Value::from));
        }
        Ok(aus)
    }

    /// Einen gezogenen Termin oeffnen -- mit dem Klartext-Abdruck als `hash`.
    pub fn termin_oeffnen(&self, e: &Eintrag) -> Option<Eintrag> {
        let inhalt: TerminInhalt = serde_json::from_slice(&herein(self, e)?).ok()?;
        if inhalt.uuid != e.key || Some(inhalt.calendar.as_str()) != e.text("calendar") {
            return None;
        }
        let felder = Terminfelder::aus_eintrag(&inhalt.felder);
        let mut aus = Eintrag::neu(Art::Termin, e.key.clone(), Was::Da);
        aus.felder = felder.als_eintrag();
        aus = aus
            .mit("calendar", inhalt.calendar)
            .mit("hash", felder.abdruck());
        if let Some(t) = e.felder.get("updated_at") {
            aus = aus.mit("updated_at", t.clone());
        }
        Some(aus)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use openany_direktpost::{Kontoschluessel, Schluessel};
    use std::collections::HashMap;

    fn buch() -> Adressbuch {
        let ich = Schluessel::erzeugen().unwrap();
        let ed = ich.oeffentlich().1;
        Adressbuch {
            ich,
            konto: Kontoschluessel::neu(),
            geraet: 7,
            geraete: HashMap::from([(7, ed)]),
        }
    }

    fn termin() -> Eintrag {
        let mut e = Eintrag::neu(Art::Termin, "t-1", Was::Da);
        e.felder = Terminfelder {
            titel: "Beispieltermin".into(),
            ort: "Beispielort".into(),
            beginn: "2026-11-02T10:00:00".into(),
            ende: "2026-11-02T11:00:00".into(),
            rrule: "NONE".into(),
            ..Default::default()
        }
        .als_eintrag();
        e.mit("calendar", "k-1")
            .mit("base_hash", "alt")
            .mit("updated_at", "2026-10-19T10:00:00+00:00")
    }

    #[test]
    fn termin_hin_und_zurueck() {
        let b = buch();
        let zu = b.termin_verschliessen(&termin(), true).unwrap();
        assert!(ist_verschluesselt(&zu));
        assert!(zu.felder.get("title").is_none());
        assert!(!zu.text("inhalt").unwrap().contains("Beispiel"));
        let abdruck = Terminfelder::aus_eintrag(&termin().felder).abdruck();
        assert_ne!(zu.text("hash"), Some(abdruck.as_str()));
        assert_eq!(zu.text("base_klar"), Some("alt"));
        assert_ne!(zu.text("base_hash"), Some("alt"));

        let auf = b.termin_oeffnen(&zu).unwrap();
        assert_eq!(auf.text("title"), Some("Beispieltermin"));
        assert_eq!(auf.text("calendar"), Some("k-1"));
        assert_eq!(auf.text("hash"), Some(abdruck.as_str()));

        // Umgestellt: Die Basis bleibt verdeckt.
        let zu = b.termin_verschliessen(&termin(), false).unwrap();
        assert!(zu.felder.get("base_klar").is_none());
    }

    #[test]
    fn verschoben_oder_fremd_geht_nicht_auf() {
        let b = buch();
        let mut zu = b.termin_verschliessen(&termin(), false).unwrap();
        zu.felder.insert("calendar".into(), "k-2".into());
        assert!(b.termin_oeffnen(&zu).is_none());

        let mut zu = b.termin_verschliessen(&termin(), false).unwrap();
        zu.art = Art::Kalender;
        assert!(b.kalender_oeffnen(&zu).is_none());

        let zu = b.termin_verschliessen(&termin(), false).unwrap();
        assert!(buch().termin_oeffnen(&zu).is_none());
    }

    #[test]
    fn kalender_hin_und_zurueck() {
        let b = buch();
        let e = Eintrag::neu(Art::Kalender, "k-1", Was::Da)
            .mit("name", "Beispiel")
            .mit("color", "#6366f1")
            .mit("sync_url", "https://beispiel.test/feed.ics");
        let zu = b.kalender_verschliessen(&e).unwrap();
        assert!(zu.felder.get("name").is_none());
        assert!(zu.felder.get("sync_url").is_none());
        let auf = b.kalender_oeffnen(&zu).unwrap();
        assert_eq!(auf.text("name"), Some("Beispiel"));
        assert_eq!(auf.text("sync_url"), Some("https://beispiel.test/feed.ics"));

        let ohne = Eintrag::neu(Art::Kalender, "k-2", Was::Da).mit("name", "Eigen");
        let auf = b
            .kalender_oeffnen(&b.kalender_verschliessen(&ohne).unwrap())
            .unwrap();
        assert_eq!(auf.felder.get("sync_url"), Some(&Value::Null));
    }
}
