//! Was ueber die Leitung kommt, und was davon ein Signal ist.

use serde::Deserialize;

/// Ein Signal, das die Schale wecken soll. Traegt keinen Inhalt, nur die
/// Stelle, an der es Neues gibt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Signal {
    /// Eine interne openany-Nachricht in diesem Projekt.
    Nachricht { projekt: Option<i64> },
    /// Ein Termin wurde angelegt, verschoben oder man wurde eingeladen.
    Kalender { projekt: Option<i64> },
    /// Ein Matrix-Homeserver meldet ein Ereignis (ueber ntfys Gateway,
    /// Format `event_id_only`). Raum und Ereignis stehen dabei, der Inhalt
    /// nie -- E2E bleibt E2E.
    Matrix {
        raum: Option<String>,
        ereignis: Option<String>,
    },
    /// Etwas, das diese Fassung nicht kennt. **Trotzdem weiterreichen:**
    /// Ein Signal aus einer neueren Serverfassung zu verschlucken hiesse,
    /// dass die App lautlos nichts tut. Die Schale gleicht dann einfach ab.
    Unbekannt { art: String },
}

/// Ein Signal samt ntfy-Kennung. Die Kennung ist der Punkt, ab dem nach
/// einem Funkloch nachgeholt wird -- die Schale darf sie sich merken.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ereignis {
    pub id: String,
    pub signal: Signal,
}

/// Eine Zeile aus ntfys JSON-Strom.
#[derive(Deserialize)]
struct Zeile {
    #[serde(default)]
    id: String,
    event: String,
    #[serde(default)]
    message: Option<String>,
}

/// Was eine Zeile fuer die Leitung bedeutet.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Gelesen {
    /// `open` -- die Leitung steht.
    Offen,
    /// `keepalive` -- nichts Neues, aber sie lebt.
    Lebenszeichen,
    Ereignis(Ereignis),
    /// Leer oder eine Art, die ntfy spaeter einmal dazunimmt
    /// (`poll_request`, ...). Kein Fehler.
    Uebergangen,
}

pub(crate) fn zeile_lesen(zeile: &str) -> Result<Gelesen, serde_json::Error> {
    let zeile = zeile.trim();
    if zeile.is_empty() {
        return Ok(Gelesen::Uebergangen);
    }
    let z: Zeile = serde_json::from_str(zeile)?;
    Ok(match z.event.as_str() {
        "open" => Gelesen::Offen,
        "keepalive" => Gelesen::Lebenszeichen,
        "message" => Gelesen::Ereignis(Ereignis {
            id: z.id,
            signal: signal_deuten(z.message.as_deref().unwrap_or("")),
        }),
        _ => Gelesen::Uebergangen,
    })
}

/// Der Koerper einer ntfy-Nachricht: entweder openanys eigenes Signal
/// (`{"art":"nachricht","projekt":12}`) oder die Push-Anfrage eines
/// Matrix-Homeservers, die ntfys Gateway unveraendert durchreicht
/// (`{"notification":{"room_id":...}}`).
fn signal_deuten(koerper: &str) -> Signal {
    let Ok(wert) = serde_json::from_str::<serde_json::Value>(koerper) else {
        return Signal::Unbekannt {
            art: "kein-json".into(),
        };
    };
    if let Some(n) = wert.get("notification") {
        let text = |feld: &str| n.get(feld).and_then(|r| r.as_str()).map(str::to_owned);
        return Signal::Matrix {
            raum: text("room_id"),
            ereignis: text("event_id"),
        };
    }
    let projekt = wert.get("projekt").and_then(|p| p.as_i64());
    match wert.get("art").and_then(|a| a.as_str()) {
        Some("nachricht") => Signal::Nachricht { projekt },
        Some("kalender") => Signal::Kalender { projekt },
        Some(andere) => Signal::Unbekannt { art: andere.into() },
        None => Signal::Unbekannt {
            art: "ohne-art".into(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ereignis(zeile: &str) -> Ereignis {
        match zeile_lesen(zeile).unwrap() {
            Gelesen::Ereignis(e) => e,
            anders => panic!("kein Ereignis: {anders:?}"),
        }
    }

    #[test]
    fn openany_signale() {
        let e = ereignis(
            r#"{"id":"a1","time":1,"event":"message","topic":"upX","message":"{\"art\":\"nachricht\",\"projekt\":12}"}"#,
        );
        assert_eq!(e.id, "a1");
        assert_eq!(e.signal, Signal::Nachricht { projekt: Some(12) });

        let e = ereignis(r#"{"id":"a2","event":"message","message":"{\"art\":\"kalender\"}"}"#);
        assert_eq!(e.signal, Signal::Kalender { projekt: None });
    }

    #[test]
    fn matrix_ueber_das_gateway() {
        let koerper = serde_json::json!({
            "notification": {
                "event_id": "$e", "room_id": "!raum:matrix.org",
                "counts": {"unread": 1}, "devices": [{"pushkey": "https://ntfy.openany.de/upX?up=1"}]
            }
        })
        .to_string();
        let zeile =
            serde_json::json!({"id": "m1", "event": "message", "message": koerper}).to_string();
        assert_eq!(
            ereignis(&zeile).signal,
            Signal::Matrix {
                raum: Some("!raum:matrix.org".into()),
                ereignis: Some("$e".into()),
            }
        );
    }

    #[test]
    fn unbekanntes_wird_weitergereicht_nicht_verschluckt() {
        let e = ereignis(r#"{"id":"u","event":"message","message":"{\"art\":\"umfrage\"}"}"#);
        assert_eq!(
            e.signal,
            Signal::Unbekannt {
                art: "umfrage".into()
            }
        );
        let e = ereignis(r#"{"id":"u","event":"message","message":"hallo"}"#);
        assert_eq!(
            e.signal,
            Signal::Unbekannt {
                art: "kein-json".into()
            }
        );
    }

    #[test]
    fn verwaltungszeilen() {
        assert_eq!(
            zeile_lesen(r#"{"id":"o","event":"open","topic":"upX"}"#).unwrap(),
            Gelesen::Offen
        );
        assert_eq!(
            zeile_lesen(r#"{"id":"k","event":"keepalive"}"#).unwrap(),
            Gelesen::Lebenszeichen
        );
        assert_eq!(
            zeile_lesen(r#"{"id":"p","event":"poll_request"}"#).unwrap(),
            Gelesen::Uebergangen
        );
        assert_eq!(zeile_lesen("  ").unwrap(), Gelesen::Uebergangen);
        assert!(zeile_lesen("{kaputt").is_err());
    }
}
