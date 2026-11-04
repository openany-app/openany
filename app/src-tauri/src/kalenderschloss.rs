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
//! Klartext-Abdruck, mit dem der Laeufer rechnet.
//!
//! **Geteilte Kalender** (docs/plan-kalender-bearbeiten.md) sind eine
//! Freigabe der Art `kalender`: Kalender und Termine gehen dann mit deren
//! Schluessel hinaus (`freigabe` im Eintrag), und der Abdruck ist damit
//! verdeckt -- so rechnen Besitzer und Mitglieder denselben. Was ein
//! Mitglied schrieb, hat dessen Geraet unterschrieben.

use openany_client::{Art, Eintrag, Terminfelder, Was};
use openany_direktpost::{Kontoschluessel, Verschluesselt};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::adressbuch::Adressbuch;
use crate::mappenteilen::Freigabe;

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

fn hinaus(
    buch: &Adressbuch,
    f: Option<&Freigabe>,
    e: &Eintrag,
    klartext: &[u8],
) -> Result<Eintrag, String> {
    let art = e.art.ueber_die_leitung();
    let schluessel = f.map_or(&buch.konto, |f| &f.schluessel);
    let (v, version) =
        openany_direktpost::sache_verschluesseln(&buch.ich, schluessel, art, &e.key, klartext)
            .map_err(|e| e.to_string())?;
    let mut aus = Eintrag::neu(e.art.clone(), e.key.clone(), Was::Da)
        .mit("inhalt", v.inhalt)
        .mit("signatur", v.signatur)
        .mit("geraet", buch.geraet)
        .mit("version", version);
    if let Some(f) = f {
        aus = aus.mit("freigabe", f.uuid.clone());
    }
    for feld in ["updated_at", "origin"] {
        if let Some(w) = e.felder.get(feld) {
            aus = aus.mit(feld, w.clone());
        }
    }
    Ok(aus)
}

fn herein(schluessel: &Kontoschluessel, ed25519: &str, e: &Eintrag) -> Option<Vec<u8>> {
    let (v, version, _) = chiffrat(e)?;
    openany_direktpost::sache_oeffnen(
        schluessel,
        e.art.ueber_die_leitung(),
        &e.key,
        version,
        &v,
        ed25519,
    )
    .ok()
}

/// Einen Kalender oeffnen -- mit dem Konto- oder Freigabeschluessel, gegen
/// den Geraeteschluessel dessen, der ihn schrieb.
pub(crate) fn kalender_oeffnen_mit(
    schluessel: &Kontoschluessel,
    ed25519: &str,
    e: &Eintrag,
) -> Option<Eintrag> {
    let inhalt: KalenderInhalt = serde_json::from_slice(&herein(schluessel, ed25519, e)?).ok()?;
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

/// Einen Termin oeffnen -- mit dem Klartext-Abdruck als `hash`.
pub(crate) fn termin_oeffnen_mit(
    schluessel: &Kontoschluessel,
    ed25519: &str,
    e: &Eintrag,
) -> Option<Eintrag> {
    let inhalt: TerminInhalt = serde_json::from_slice(&herein(schluessel, ed25519, e)?).ok()?;
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

/// Einen Termin mit dem Schluessel einer Freigabe verschliessen -- fuer
/// Mitglieder, die in einen geteilten Kalender schreiben. Welches Geraet,
/// weiss der Server aus dem Zugang; `base_hash` (Klartext) wird verdeckt.
pub(crate) fn termin_zu(
    ich: &openany_direktpost::Schluessel,
    f_uuid: &str,
    schluessel: &Kontoschluessel,
    e: &Eintrag,
) -> Result<Eintrag, String> {
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
    let (v, version) =
        openany_direktpost::sache_verschluesseln(ich, schluessel, "event", &e.key, &klartext)
            .map_err(|e| e.to_string())?;
    let basis = e.text("base_hash").filter(|s| !s.is_empty());
    Ok(Eintrag::neu(Art::Termin, e.key.clone(), Was::Da)
        .mit("inhalt", v.inhalt)
        .mit("signatur", v.signatur)
        .mit("version", version)
        .mit("calendar", kalender)
        .mit("freigabe", f_uuid)
        .mit("hash", schluessel.abdruck_verdecken(&e.key, &abdruck))
        .mit(
            "base_hash",
            basis.map_or(Value::Null, |b| {
                schluessel.abdruck_verdecken(&e.key, b).into()
            }),
        ))
}

impl Adressbuch {
    /// Einen Kalender zum Hinausschicken verschliessen -- in einer eigenen
    /// Kalender-Freigabe (`f`) mit deren Schluessel.
    pub fn kalender_verschliessen(
        &self,
        e: &Eintrag,
        f: Option<&Freigabe>,
    ) -> Result<Eintrag, String> {
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
        hinaus(self, f, e, &klartext)
    }

    /// Einen gezogenen eigenen Kalender oeffnen -- `None`, wenn er nicht aufgeht.
    pub fn kalender_oeffnen(&self, e: &Eintrag) -> Option<Eintrag> {
        let von = e.felder.get("geraet").and_then(Value::as_i64)?;
        kalender_oeffnen_mit(&self.konto, self.geraete.get(&von)?, e)
    }

    /// Einen Termin verschliessen -- in einer eigenen Kalender-Freigabe
    /// (`f`) mit deren Schluessel. `base_alt`: dieselbe Basis in den anderen
    /// Verdeckungen, fuer den Wechsel in eine Freigabe oder heraus.
    pub fn termin_verschliessen(
        &self,
        e: &Eintrag,
        f: Option<&Freigabe>,
        base_alt: Value,
    ) -> Result<Eintrag, String> {
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
        let schluessel = f.map_or(&self.konto, |f| &f.schluessel);
        hinaus(self, f, e, &klartext).map(|aus| {
            aus.mit("calendar", kalender)
                .mit("hash", schluessel.abdruck_verdecken(&e.key, &abdruck))
                .mit(
                    "base_hash",
                    basis.map_or(Value::Null, |b| {
                        schluessel.abdruck_verdecken(&e.key, b).into()
                    }),
                )
                .mit("base_alt", base_alt)
        })
    }

    /// Einen gezogenen eigenen Termin oeffnen -- mit dem Klartext-Abdruck als `hash`.
    pub fn termin_oeffnen(&self, e: &Eintrag) -> Option<Eintrag> {
        let von = e.felder.get("geraet").and_then(Value::as_i64)?;
        termin_oeffnen_mit(&self.konto, self.geraete.get(&von)?, e)
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
        let zu = b
            .termin_verschliessen(&termin(), None, Value::Array(vec![]))
            .unwrap();
        assert!(ist_verschluesselt(&zu));
        assert!(zu.felder.get("title").is_none());
        assert!(!zu.text("inhalt").unwrap().contains("Beispiel"));
        let abdruck = Terminfelder::aus_eintrag(&termin().felder).abdruck();
        assert_ne!(zu.text("hash"), Some(abdruck.as_str()));
        assert!(zu.felder.get("base_klar").is_none());
        assert_ne!(zu.text("base_hash"), Some("alt"));

        let auf = b.termin_oeffnen(&zu).unwrap();
        assert_eq!(auf.text("title"), Some("Beispieltermin"));
        assert_eq!(auf.text("calendar"), Some("k-1"));
        assert_eq!(auf.text("hash"), Some(abdruck.as_str()));
    }

    #[test]
    fn verschoben_oder_fremd_geht_nicht_auf() {
        let b = buch();
        let mut zu = b
            .termin_verschliessen(&termin(), None, Value::Array(vec![]))
            .unwrap();
        zu.felder.insert("calendar".into(), "k-2".into());
        assert!(b.termin_oeffnen(&zu).is_none());

        let mut zu = b
            .termin_verschliessen(&termin(), None, Value::Array(vec![]))
            .unwrap();
        zu.art = Art::Kalender;
        assert!(b.kalender_oeffnen(&zu).is_none());

        let zu = b
            .termin_verschliessen(&termin(), None, Value::Array(vec![]))
            .unwrap();
        assert!(buch().termin_oeffnen(&zu).is_none());
    }

    #[test]
    fn kalender_hin_und_zurueck() {
        let b = buch();
        let e = Eintrag::neu(Art::Kalender, "k-1", Was::Da)
            .mit("name", "Beispiel")
            .mit("color", "#6366f1")
            .mit("sync_url", "https://beispiel.test/feed.ics");
        let zu = b.kalender_verschliessen(&e, None).unwrap();
        assert!(zu.felder.get("name").is_none());
        assert!(zu.felder.get("sync_url").is_none());
        let auf = b.kalender_oeffnen(&zu).unwrap();
        assert_eq!(auf.text("name"), Some("Beispiel"));
        assert_eq!(auf.text("sync_url"), Some("https://beispiel.test/feed.ics"));

        let ohne = Eintrag::neu(Art::Kalender, "k-2", Was::Da).mit("name", "Eigen");
        let auf = b
            .kalender_oeffnen(&b.kalender_verschliessen(&ohne, None).unwrap())
            .unwrap();
        assert_eq!(auf.felder.get("sync_url"), Some(&Value::Null));
    }

    #[test]
    fn in_einer_freigabe_mit_deren_schluessel() {
        let b = buch();
        let f = Freigabe {
            uuid: "f-1".into(),
            art: "kalender".into(),
            pfad: String::new(),
            wurzel: Some("k-1".into()),
            schluessel: Kontoschluessel::neu(),
        };
        let zu = b
            .termin_verschliessen(&termin(), Some(&f), Value::Array(vec![]))
            .unwrap();
        assert_eq!(zu.text("freigabe"), Some("f-1"));
        // Mit dem Kontoschluessel geht er nicht auf, mit dem der Freigabe schon.
        assert!(b.termin_oeffnen(&zu).is_none());
        let ed = b.geraete[&7].clone();
        let auf = termin_oeffnen_mit(&f.schluessel, &ed, &zu).unwrap();
        assert_eq!(auf.text("title"), Some("Beispieltermin"));
        // Der Abdruck ist mit dem Schluessel der Freigabe verdeckt.
        let abdruck = Terminfelder::aus_eintrag(&termin().felder).abdruck();
        assert_eq!(
            zu.text("hash"),
            Some(f.schluessel.abdruck_verdecken("t-1", &abdruck).as_str())
        );
    }
}
