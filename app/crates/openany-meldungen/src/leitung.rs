//! Die Leitung selbst: verbinden, lesen, bei Abriss sparsam neu.

use std::time::Duration;

use futures_util::StreamExt;
use tokio::sync::watch;

use crate::signal::{zeile_lesen, Ereignis, Gelesen};
use crate::warten::Wartezeit;

/// Wie lange nach einem Netzwechsel gewartet wird, bevor neu gewaehlt wird.
pub const NACH_NETZWECHSEL: Duration = Duration::from_secs(1);

/// Wohin die Leitung geht und wo sie nach einer Luecke weitermacht.
#[derive(Debug, Clone)]
pub struct Leitung {
    /// `https://ntfy.openany.de` -- ohne Schraegstrich am Ende.
    pub basis: String,
    /// Das geheime Thema dieses Geraets (`up` + Zufall). Laravel vergibt es.
    pub thema: String,
    /// Die letzte gesehene ntfy-Kennung. Beim Neuverbinden geht sie als
    /// `since=` mit; ohne sie kommt nur, was ab jetzt geschieht.
    pub letzte_id: Option<String>,
    /// Bleibt es laenger still, gilt die Leitung als tot. Knapp ueber dem
    /// Lebenszeichen des Servers (3 min): Eine Minute Luft fuer ein
    /// langsames Netz, und nicht mehr -- jede Minute darueber ist eine
    /// Minute, in der ein totes Netz fuer lebendig gehalten wird.
    pub lese_frist: Duration,
}

impl Leitung {
    pub fn new(basis: impl Into<String>, thema: impl Into<String>) -> Self {
        Self {
            basis: basis.into().trim_end_matches('/').to_owned(),
            thema: thema.into(),
            letzte_id: None,
            lese_frist: Duration::from_secs(4 * 60),
        }
    }

    /// `…/<thema>/json`, mit `since=` wenn es eine letzte Kennung gibt.
    pub fn adresse(&self) -> Result<url::Url, Leitungsfehler> {
        // Dieselbe Zeichenregel wie ntfy. Ein Thema mit `/` oder `?` wuerde
        // hier sonst still eine ganz andere Adresse ergeben.
        let gueltig = !self.thema.is_empty()
            && self.thema.len() <= 64
            && self
                .thema
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_');
        if !gueltig {
            return Err(Leitungsfehler::Adresse(format!(
                "Invalid topic: {:?}",
                self.thema
            )));
        }
        let mut u = url::Url::parse(&format!("{}/{}/json", self.basis, self.thema))
            .map_err(|e| Leitungsfehler::Adresse(e.to_string()))?;
        if let Some(id) = &self.letzte_id {
            u.query_pairs_mut().append_pair("since", id);
        }
        Ok(u)
    }
}

/// Warum eine Leitung endete. Jeder Abriss hat einen Grund, und der Grund
/// geht an die Schale -- eine Leitung, die ohne Auskunft immer wieder
/// abreisst, ist am Geraet nicht zu untersuchen.
#[derive(Debug, thiserror::Error)]
pub enum Leitungsfehler {
    #[error("Address: {0}")]
    Adresse(String),
    #[error("Connection: {0}")]
    Verbindung(reqwest::Error),
    #[error("ntfy responds {0}")]
    Status(u16),
    #[error("silent for {0:?}")]
    Stille(Duration),
    #[error("closed by the server")]
    Beendet,
    #[error("network changed")]
    Netzwechsel,
    #[error("unreadable line: {0}")]
    Zeile(#[from] serde_json::Error),
}

/// OHNE ADRESSE. reqwest haengt die URL an seine Fehler, und in der URL
/// steht das geheime Thema. Der Grund eines Abrisses geht ins Protokoll
/// des Geraets (logcat) -- das Thema hat dort nichts zu suchen.
impl From<reqwest::Error> for Leitungsfehler {
    fn from(e: reqwest::Error) -> Self {
        Leitungsfehler::Verbindung(e.without_url())
    }
}

/// Was die Schale erfaehrt.
#[derive(Debug)]
pub enum Vorgang {
    /// Die Leitung steht (ntfy hat `open` geschickt).
    Verbunden,
    /// Ein Signal. `letzte_id` ist danach schon fortgeschrieben.
    Ereignis(Ereignis),
    /// Die Leitung ist abgerissen; der naechste Versuch kommt nach `pause`
    /// (oder frueher, wenn das Netz wechselt).
    Abriss {
        grund: Leitungsfehler,
        pause: Duration,
    },
}

/// Haelt die Leitung, bis die umgebende Aufgabe abgebrochen wird.
///
/// `netz` meldet, ob ein Netz da ist. Ohne Netz wird nicht gewaehlt; jeder
/// Wechsel beendet eine laufende Leitung sofort, statt vier Minuten auf die
/// Lese-Frist zu warten -- nach einem Wechsel von WLAN zu Mobilfunk ist die
/// alte Verbindung tot, auch wenn es noch niemand gemerkt hat. Wird der
/// Sender fallen gelassen, gilt das Netz als immer da (Schreibtisch).
pub async fn halten<F>(mut leitung: Leitung, mut netz: watch::Receiver<bool>, mut bei: F)
where
    F: FnMut(Vorgang),
{
    let client = match reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(20))
        // KEIN Gesamt-Zeitlimit: Die Antwort endet nie. Und kein
        // TCP-Keepalive vom Geraet aus -- jedes davon weckte den Funk; das
        // Lebenszeichen kommt vom Server.
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            // Ohne Client gibt es nichts zu halten. Einmal sagen, warum.
            bei(Vorgang::Abriss {
                grund: e.into(),
                pause: Duration::MAX,
            });
            return;
        }
    };
    let mut warten = Wartezeit::default();
    let mut netz_gemeldet = true;

    loop {
        // Kein Netz: auf die naechste Meldung warten, nicht waehlen.
        while netz_gemeldet && !*netz.borrow_and_update() {
            if netz.changed().await.is_err() {
                netz_gemeldet = false;
            }
        }

        let mut stand = false;
        let grund = einmal(
            &client,
            &mut leitung,
            &mut netz,
            &mut netz_gemeldet,
            &mut stand,
            &mut bei,
        )
        .await;
        if stand {
            warten.zuruecksetzen();
        }
        // Ein Netzwechsel ist kein Fehler des Servers: gleich neu, ohne die
        // Wartezeit hochzuzaehlen. Aber nicht in derselben Millisekunde --
        // Android meldet das neue Netz, bevor es Verbindungen annimmt, und
        // die ersten zwei Versuche liefen am Tablet ins Leere (26.09.2026).
        let pause = if matches!(grund, Leitungsfehler::Netzwechsel) {
            NACH_NETZWECHSEL
        } else {
            warten.naechste()
        };
        bei(Vorgang::Abriss { grund, pause });

        if netz_gemeldet {
            tokio::select! {
                _ = tokio::time::sleep(pause) => {}
                r = netz.changed() => if r.is_err() { netz_gemeldet = false },
            }
        } else {
            tokio::time::sleep(pause).await;
        }
    }
}

/// Eine Leitung, von Anfang bis Abriss. Kehrt immer mit einem Grund zurueck.
async fn einmal<F>(
    client: &reqwest::Client,
    leitung: &mut Leitung,
    netz: &mut watch::Receiver<bool>,
    netz_gemeldet: &mut bool,
    stand: &mut bool,
    bei: &mut F,
) -> Leitungsfehler
where
    F: FnMut(Vorgang),
{
    let adresse = match leitung.adresse() {
        Ok(a) => a,
        Err(e) => return e,
    };
    let antwort = match client.get(adresse).send().await {
        Ok(a) => a,
        Err(e) => return e.into(),
    };
    if !antwort.status().is_success() {
        return Leitungsfehler::Status(antwort.status().as_u16());
    }

    let mut strom = antwort.bytes_stream();
    let mut puffer: Vec<u8> = Vec::new();
    loop {
        let stueck = if *netz_gemeldet {
            tokio::select! {
                s = tokio::time::timeout(leitung.lese_frist, strom.next()) => s,
                r = netz.changed() => {
                    if r.is_err() { *netz_gemeldet = false; continue; }
                    return Leitungsfehler::Netzwechsel;
                }
            }
        } else {
            tokio::time::timeout(leitung.lese_frist, strom.next()).await
        };
        let bytes = match stueck {
            Err(_) => return Leitungsfehler::Stille(leitung.lese_frist),
            Ok(None) => return Leitungsfehler::Beendet,
            Ok(Some(Err(e))) => return e.into(),
            Ok(Some(Ok(b))) => b,
        };
        puffer.extend_from_slice(&bytes);

        // Nur ganze Zeilen; ein Rest wartet auf das naechste Stueck.
        while let Some(ende) = puffer.iter().position(|&b| b == b'\n') {
            let zeile: Vec<u8> = puffer.drain(..=ende).collect();
            let text = String::from_utf8_lossy(&zeile);
            match zeile_lesen(&text) {
                Ok(Gelesen::Offen) => {
                    *stand = true;
                    bei(Vorgang::Verbunden);
                }
                Ok(Gelesen::Ereignis(e)) => {
                    leitung.letzte_id = Some(e.id.clone());
                    bei(Vorgang::Ereignis(e));
                }
                Ok(Gelesen::Lebenszeichen | Gelesen::Uebergangen) => {}
                Err(e) => return e.into(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adresse_mit_und_ohne_since() {
        let mut l = Leitung::new("https://ntfy.openany.de/", "upAbc_1-2");
        assert_eq!(
            l.adresse().unwrap().as_str(),
            "https://ntfy.openany.de/upAbc_1-2/json"
        );
        l.letzte_id = Some("ebO6norqkeAV".into());
        assert_eq!(
            l.adresse().unwrap().as_str(),
            "https://ntfy.openany.de/upAbc_1-2/json?since=ebO6norqkeAV"
        );
    }

    #[test]
    fn thema_das_die_adresse_verbiegen_wuerde() {
        for schlecht in ["", "up/../x", "up?x=1", "up x", &"u".repeat(65)] {
            let l = Leitung::new("https://ntfy.openany.de", schlecht);
            assert!(
                matches!(l.adresse(), Err(Leitungsfehler::Adresse(_))),
                "{schlecht:?}"
            );
        }
    }
}
