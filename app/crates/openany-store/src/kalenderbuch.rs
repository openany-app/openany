//! Der Kalender, wie die Arbeitsflaeche ihn braucht.
//!
//! Die Arbeitsflaeche (`packages/oberflaeche/kalender`) ist dieselbe wie in der
//! Webapp. Zwei Stellen muessen hier genau so rechnen wie drueben, sonst
//! entstehen beim Abgleich Konfliktkopien, ohne dass etwas fehlschlaegt:
//!
//! **Die Zeitform.** Der Termin-Dialog schickt `2026-09-15 10:00:00`, der
//! Server speichert und hasht `2026-09-15T10:00:00` (Cast des Models). Wuerde
//! die App den Text so ablegen, wie er kommt, haette derselbe Termin hier und
//! drueben verschiedene Abdruecke. [`zeit`] bringt ihn in die Serverform.
//!
//! **Das Zeitfenster.** Einmaltermine muessen das Fenster ueberlappen; ein
//! Serientermin darf vorher begonnen haben, solange seine Serie nicht vor dem
//! Fenster ausgelaufen ist -- die Arbeitsflaeche faltet ihn selbst aus.
//! Dieselbe Regel wie `EventController::index`.

use crate::{Ergebnis, Speicher, Termin};

/// Eine Zeitangabe in die Form `Y-m-d\TH:i:s`. Leeres bleibt leer.
pub fn zeit(roh: &str) -> String {
    let roh = roh.trim();
    if roh.is_empty() {
        return String::new();
    }

    let (datum, uhr) = match roh.split_once(['T', ' ']) {
        Some((d, u)) => (d, u.trim()),
        None => (roh, ""),
    };

    // Zeitzone oder Bruchteile abschneiden: Der Server speichert Wanduhrzeit.
    let uhr: String = uhr
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == ':')
        .collect();
    let teile: Vec<&str> = uhr.split(':').filter(|s| !s.is_empty()).collect();
    let feld = |i: usize| {
        teile
            .get(i)
            .map(|s| format!("{:0>2}", s))
            .unwrap_or_else(|| "00".into())
    };

    format!("{datum}T{}:{}:{}", feld(0), feld(1), feld(2))
}

fn ist_serie(t: &Termin) -> bool {
    !t.felder.rrule.is_empty() && t.felder.rrule != "NONE"
}

impl Speicher {
    /// Termine, die das Fenster `[von, bis]` (Tage, `Y-m-d`) beruehren.
    pub fn termine_im_fenster(&self, von: &str, bis: &str) -> Ergebnis<Vec<Termin>> {
        let von = format!("{von}T00:00:00");
        let bis = format!("{bis}T23:59:59");

        Ok(self
            .termine_alle()?
            .into_iter()
            .filter(|t| {
                let beginn = t.felder.beginn.as_str();
                if beginn.is_empty() || beginn > bis.as_str() {
                    return false;
                }
                if ist_serie(t) {
                    t.felder.rrule_bis.is_empty() || t.felder.rrule_bis.as_str() >= von.as_str()
                } else {
                    let ende = if t.felder.ende.is_empty() {
                        beginn
                    } else {
                        t.felder.ende.as_str()
                    };
                    ende >= von.as_str()
                }
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Protokoll;
    use openany_client::Terminfelder;

    #[test]
    fn zeit_in_serverform() {
        assert_eq!(zeit("2026-09-15 10:00:00"), "2026-09-15T10:00:00");
        assert_eq!(zeit("2026-09-15T10:00"), "2026-09-15T10:00:00");
        assert_eq!(zeit("2026-09-15"), "2026-09-15T00:00:00");
        assert_eq!(zeit("2026-09-15 23:59:59"), "2026-09-15T23:59:59");
        assert_eq!(zeit("2026-09-15T08:05:00.000000Z"), "2026-09-15T08:05:00");
        assert_eq!(zeit(""), "");
    }

    fn termin(s: &Speicher, uuid: &str, beginn: &str, ende: &str, rrule: &str, bis: &str) {
        s.termin_schreiben(
            &Termin {
                uuid: uuid.into(),
                kalender_uuid: "k".into(),
                felder: Terminfelder {
                    titel: uuid.into(),
                    beginn: beginn.into(),
                    ende: ende.into(),
                    rrule: rrule.into(),
                    rrule_bis: bis.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            Protokoll::Merken,
        )
        .unwrap();
    }

    #[test]
    fn fenster_wie_der_server() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        termin(
            &s,
            "davor",
            "2026-07-01T10:00:00",
            "2026-07-01T11:00:00",
            "NONE",
            "",
        );
        termin(
            &s,
            "ueberlappt",
            "2026-07-30T00:00:00",
            "2026-08-02T23:59:59",
            "NONE",
            "",
        );
        termin(
            &s,
            "letzter-tag",
            "2026-10-31T18:00:00",
            "2026-10-31T19:00:00",
            "",
            "",
        );
        termin(
            &s,
            "serie-offen",
            "2025-01-06T08:00:00",
            "2025-01-06T09:00:00",
            "WEEKLY",
            "",
        );
        termin(
            &s,
            "serie-aus",
            "2025-01-06T08:00:00",
            "2025-01-06T09:00:00",
            "WEEKLY",
            "2026-07-15T00:00:00",
        );
        termin(
            &s,
            "danach",
            "2026-11-01T00:00:00",
            "2026-11-01T01:00:00",
            "NONE",
            "",
        );

        let mut namen: Vec<String> = s
            .termine_im_fenster("2026-08-01", "2026-10-31")
            .unwrap()
            .into_iter()
            .map(|t| t.uuid)
            .collect();
        namen.sort();

        assert_eq!(namen, vec!["letzter-tag", "serie-offen", "ueberlappt"]);
    }
}
