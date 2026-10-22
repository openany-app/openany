//! Ein Geraet schaltet ein Geraet frei (Z4, docs/plan-zero-knowledge.md).
//!
//! **Das neue Geraet** nennt einen Anmeldenamen (oder ist schon per Passwort
//! angemeldet und braucht nur den Kontoschluessel), wuerfelt 6 Ziffern und
//! zeigt sie an. Die Oberflaeche fragt dann im Takt nach
//! ([`freischaltung_schritt`]): Antwort des alten Geraets lesen, eigene
//! Schluessel schicken, bei anyid abholen.
//!
//! **Das alte Geraet** sieht die Anfrage, der Mensch tippt die Ziffern ein,
//! und [`anfrage_freischalten`] erledigt den PAKE in einem Aufruf. Danach
//! merkt es sich die Schluessel, die das neue Geraet bewiesen hat; sobald ein
//! Geraet mit genau diesen Schluesseln im Verzeichnis steht, legt
//! [`crate::direktpost::bereit`] ihm die Kiste mit dem Kontoschluessel --
//! ohne zweites Tippen.
//!
//! Die Nachrichten selbst entstehen in `openany_direktpost::freischalten`;
//! anyid reicht sie nur durch.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use openany_anmeldung::{Kopplung, Kopplungsstand};
use openany_direktpost::freischalten::{
    ziffern_wuerfeln, AltesGeraet, Freischaltfehler, Geraeteschluessel, NeuesGeraet,
};
use serde::{Deserialize, Serialize};

use crate::{direktpost, Zustand};

/// Wie lange gemerkte Schluessel auf ihr Geraet warten.
const ERWARTET_SEKUNDEN: u64 = 60 * 60;
/// Nach so vielen Fehlschlaegen in einer Stunde nimmt dieses Geraet keine
/// Ziffern mehr an -- ein Server, der raet, bekommt nicht beliebig viele
/// Versuche.
const FEHLSCHLAEGE_JE_STUNDE: usize = 3;
/// Wie lange das alte Geraet auf die letzte Nachricht wartet.
const WARTEN_RUNDEN: u32 = 20;
const WARTEN_TAKT: Duration = Duration::from_millis(1500);

fn jetzt() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// --- Was dieses Geraet sich merkt -----------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct Merkliste {
    /// Schluessel neuer Geraete, denen die Kiste noch fehlt (altes Geraet).
    #[serde(default)]
    erwartet: Vec<Erwartet>,
    /// Zeitpunkte gescheiterter Freischaltungen (altes Geraet).
    #[serde(default)]
    fehlschlaege: Vec<u64>,
    /// Bis wann DIESES Geraet eine Kiste erwartet (neues Geraet).
    #[serde(default)]
    kiste_bis: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Erwartet {
    pub(crate) age: String,
    pub(crate) ed25519: String,
    bis: u64,
}

fn pfad(zustand: &Zustand) -> PathBuf {
    zustand.ordner.join("freischalten.json")
}

fn lesen(zustand: &Zustand) -> Merkliste {
    let mut m: Merkliste = std::fs::read(pfad(zustand))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    let t = jetzt();
    m.erwartet.retain(|e| e.bis > t);
    m.fehlschlaege.retain(|f| f + 3600 > t);
    if m.kiste_bis.is_some_and(|b| b <= t) {
        m.kiste_bis = None;
    }
    m
}

fn schreiben(zustand: &Zustand, m: &Merkliste) {
    if let Ok(b) = serde_json::to_vec(m) {
        let _ = std::fs::write(pfad(zustand), b);
    }
}

/// Die Schluessel, denen dieses (alte) Geraet noch eine Kiste legen soll.
pub(crate) fn erwartete(zustand: &Zustand) -> Vec<Erwartet> {
    lesen(zustand).erwartet
}

/// Eine Kiste ist gelegt -- diese Schluessel nicht mehr erwarten.
pub(crate) fn erledigt(zustand: &Zustand, age: &str, ed25519: &str) {
    let mut m = lesen(zustand);
    m.erwartet
        .retain(|e| !(e.age == age && e.ed25519 == ed25519));
    schreiben(zustand, &m);
}

/// Erwartet DIESES Geraet gerade eine Kiste von einem anderen? Dann zeigt die
/// Oberflaeche „kommt gleich" statt „Ziffern anzeigen".
pub(crate) fn kiste_erwartet(zustand: &Zustand) -> bool {
    lesen(zustand).kiste_bis.is_some()
}

fn kiste_erwarten(zustand: &Zustand) {
    let mut m = lesen(zustand);
    m.kiste_bis = Some(jetzt() + 15 * 60);
    schreiben(zustand, &m);
}

// --- Das neue Geraet --------------------------------------------------------------

/// Eine laufende Freischaltung auf DIESEM Geraet.
pub(crate) struct NeuLauf {
    kopplung: Kopplung,
    ziffern: String,
    /// Bis die Antwort des alten Geraets da ist.
    pake: Option<NeuesGeraet>,
    /// Nur der Kontoschluessel -- dieses Geraet ist schon angemeldet.
    nur_schluessel: bool,
}

#[derive(Serialize)]
pub struct FreischaltAnzeige {
    /// Anzeigen, nicht verschicken -- sie gehen nie an den Server.
    ziffern: String,
    ablauf_in: u64,
    intervall: u64,
    nur_schluessel: bool,
}

impl FreischaltAnzeige {
    fn aus(lauf: &NeuLauf) -> Self {
        Self {
            ziffern: lauf.ziffern.clone(),
            ablauf_in: lauf.kopplung.ablauf_in,
            intervall: lauf.kopplung.intervall,
            nur_schluessel: lauf.nur_schluessel,
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "stand", rename_all = "snake_case")]
pub enum Freischaltstand {
    /// Noch keine Antwort, oder angemeldet wird erst noch.
    Wartet { intervall: u64 },
    /// Nur-Schluessel: Die Schluessel sind beim alten Geraet; die Kiste
    /// kommt mit dem naechsten Blick auf die Nachrichtenschluessel.
    Gesendet,
    /// Angemeldet -- `name` ist, wem das Konto gehoert.
    Gekoppelt { name: String },
    /// Die Ziffern passten nicht (oder jemand dazwischen hat geraten).
    StimmtNicht,
    /// Abgelaufen oder vom alten Geraet abgelehnt.
    Ungueltig,
}

/// Anfangen. Mit `name`: dieses Geraet ist noch nicht verbunden und meldet
/// sich ueber ein anderes an. Ohne: es ist angemeldet und braucht nur den
/// Kontoschluessel.
#[tauri::command]
pub async fn freischaltung_beginnen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    name: Option<String>,
) -> Result<FreischaltAnzeige, String> {
    let name = name.map(|n| n.trim().to_string()).filter(|n| !n.is_empty());

    let kopplung = match &name {
        Some(name) => {
            crate::openany_vorbereiten(&zustand).await?;
            zustand
                .anmeldung()
                .await?
                .koppeln_ueber_geraet(name)
                .await
                .map_err(|e| e.to_string())?
        }
        None => zustand
            .anmeldung()
            .await?
            .schluessel_erbitten()
            .await
            .map_err(|e| e.to_string())?,
    };
    let anfrage = kopplung
        .anfrage
        .clone()
        .ok_or("anyid did not open a request. Update anyid and try again.")?;

    let (age, ed25519) = direktpost::schluessel(&zustand)?.oeffentlich();
    let ziffern = ziffern_wuerfeln();
    let (pake, erste) =
        NeuesGeraet::beginnen(&anfrage, &ziffern, Geraeteschluessel { age, ed25519 })
            .map_err(|e| e.to_string())?;

    let anmeldung = zustand.anmeldung().await?;
    anmeldung
        .briefkasten(&kopplung, Some(&erste))
        .await
        .map_err(|e| e.to_string())?
        .ok_or("The request expired. Try again.")?;

    let lauf = NeuLauf {
        kopplung,
        ziffern,
        pake: Some(pake),
        nur_schluessel: name.is_none(),
    };
    let anzeige = FreischaltAnzeige::aus(&lauf);
    *zustand.freischaltung.lock().await = Some(lauf);
    Ok(anzeige)
}

/// Laeuft hier eine Freischaltung? Fuer den Neuaufbau der Oberflaeche.
#[tauri::command]
pub async fn freischaltung_laeuft(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Option<FreischaltAnzeige>, String> {
    Ok(zustand
        .freischaltung
        .lock()
        .await
        .as_ref()
        .map(FreischaltAnzeige::aus))
}

#[tauri::command]
pub async fn freischaltung_abbrechen(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<(), String> {
    *zustand.freischaltung.lock().await = None;
    Ok(())
}

/// Einmal nachfragen -- im Takt, den anyid vorgibt.
#[tauri::command]
pub async fn freischaltung_schritt(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Freischaltstand, String> {
    let mut halter = zustand.freischaltung.lock().await;
    let Some(lauf) = halter.as_mut() else {
        return Ok(Freischaltstand::Ungueltig);
    };
    let anmeldung = zustand.anmeldung().await?;

    if lauf.pake.is_some() {
        let Some(post) = anmeldung
            .briefkasten(&lauf.kopplung, None)
            .await
            .map_err(|e| e.to_string())?
        else {
            *halter = None;
            return Ok(Freischaltstand::Ungueltig);
        };

        let Some(antwort) = post.first() else {
            return Ok(Freischaltstand::Wartet {
                intervall: lauf.kopplung.intervall,
            });
        };

        let pake = lauf.pake.take().expect("eben geprueft");
        match pake.antworten(antwort) {
            Ok(letzte) => {
                anmeldung
                    .briefkasten(&lauf.kopplung, Some(&letzte))
                    .await
                    .map_err(|e| e.to_string())?;
                kiste_erwarten(&zustand);
            }
            Err(_) => {
                *halter = None;
                return Ok(Freischaltstand::StimmtNicht);
            }
        }

        if lauf.nur_schluessel {
            *halter = None;
            return Ok(Freischaltstand::Gesendet);
        }
    }

    if lauf.nur_schluessel {
        return Ok(Freischaltstand::Wartet {
            intervall: lauf.kopplung.intervall,
        });
    }

    match anmeldung
        .koppeln_abholen(&lauf.kopplung)
        .await
        .map_err(|e| e.to_string())?
    {
        Kopplungsstand::Wartet { intervall } => Ok(Freischaltstand::Wartet { intervall }),
        Kopplungsstand::Gekoppelt { name } => {
            *halter = None;
            drop(halter);
            // Gleich melden, damit das alte Geraet die Schluessel im
            // Verzeichnis findet und die Kiste legen kann.
            if let Some(client) = crate::server_client(&zustand).await {
                let _ = direktpost::bereit(&zustand, &client).await;
            }
            Ok(Freischaltstand::Gekoppelt { name })
        }
        Kopplungsstand::Ungueltig => {
            *halter = None;
            Ok(Freischaltstand::Ungueltig)
        }
    }
}

// --- Das alte Geraet --------------------------------------------------------------

/// Eine Anfrage, wie die Oberflaeche sie zeigt.
#[derive(Serialize)]
pub struct AnfrageAnzeige {
    anfrage: String,
    geraet: String,
    /// `anmelden` oder `schluessel`.
    art: String,
    seit: String,
}

/// Die offenen Anfragen -- nur, wenn dieses Geraet den Kontoschluessel hat;
/// sonst koennte es nichts weitergeben. Fehler machen die Liste leer: Der
/// Hinweis soll nicht wegen eines Netzaussetzers rot werden.
#[tauri::command]
pub async fn freischaltanfragen(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Vec<AnfrageAnzeige>, String> {
    if direktpost::konto(&zustand).is_none() {
        return Ok(Vec::new());
    }
    let Ok(anmeldung) = zustand.anmeldung().await else {
        return Ok(Vec::new());
    };
    if !anmeldung.einsatzbereit().unwrap_or(false) {
        return Ok(Vec::new());
    }
    Ok(anmeldung
        .anfragen()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|a| AnfrageAnzeige {
            anfrage: a.anfrage,
            geraet: a.geraet,
            art: a.art,
            seit: a.seit,
        })
        .collect())
}

#[tauri::command]
pub async fn anfrage_ablehnen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    anfrage: String,
) -> Result<(), String> {
    zustand
        .anmeldung()
        .await?
        .anfrage_verwerfen(&anfrage)
        .await
        .map_err(|e| e.to_string())
}

/// Mit den getippten Ziffern freischalten. Kehrt erst zurueck, wenn der PAKE
/// durch ist (oder gescheitert).
#[tauri::command]
pub async fn anfrage_freischalten(
    zustand: tauri::State<'_, Arc<Zustand>>,
    anfrage: String,
    ziffern: String,
) -> Result<(), String> {
    if lesen(&zustand).fehlschlaege.len() >= FEHLSCHLAEGE_JE_STUNDE {
        return Err("Too many failed attempts. Try again in an hour.".into());
    }
    if direktpost::konto(&zustand).is_none() {
        return Err("This device does not have the account key yet.".into());
    }

    let anmeldung = zustand.anmeldung().await?;
    let art = anmeldung
        .anfragen()
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|a| a.anfrage == anfrage)
        .map(|a| a.art)
        .ok_or("This request no longer exists.")?;

    let post = anmeldung
        .anfrage_briefkasten(&anfrage, None)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("This request no longer exists.")?;
    let Some(erste) = post.first() else {
        return Err("The new device is not ready yet. Try again in a moment.".into());
    };

    let (pake, antwort) = match AltesGeraet::antworten(&anfrage, &ziffern, erste) {
        Ok(x) => x,
        Err(Freischaltfehler::Ziffern) => return Err("Enter the 6 digits.".into()),
        Err(e) => {
            let _ = anmeldung.anfrage_verwerfen(&anfrage).await;
            return Err(e.to_string());
        }
    };
    anmeldung
        .anfrage_briefkasten(&anfrage, Some(&antwort))
        .await
        .map_err(|e| e.to_string())?
        .ok_or("This request no longer exists.")?;

    // Auf die letzte Nachricht warten. Passen die Ziffern nicht, kommt sie
    // nie -- das neue Geraet schickt seine Schluessel nur nach dem Beweis.
    let mut letzte = None;
    for _ in 0..WARTEN_RUNDEN {
        tokio::time::sleep(WARTEN_TAKT).await;
        match anmeldung.anfrage_briefkasten(&anfrage, None).await {
            Ok(Some(post)) if post.len() >= 2 => {
                letzte = Some(post[1].clone());
                break;
            }
            Ok(Some(_)) => {}
            Ok(None) => break,
            Err(_) => {}
        }
    }

    let schluessel = letzte
        .ok_or(Freischaltfehler::StimmtNicht)
        .and_then(|n| pake.abschliessen(&n));

    let schluessel = match schluessel {
        Ok(s) => s,
        Err(_) => {
            let mut m = lesen(&zustand);
            m.fehlschlaege.push(jetzt());
            schreiben(&zustand, &m);
            let _ = anmeldung.anfrage_verwerfen(&anfrage).await;
            return Err("The digits do not match. Start again on the new device.".into());
        }
    };

    // Erst merken, dann anmelden: Meldet sich das neue Geraet schneller, als
    // hier geschrieben wird, faende es sonst keine Erwartung vor.
    let mut m = lesen(&zustand);
    m.erwartet.push(Erwartet {
        age: schluessel.age,
        ed25519: schluessel.ed25519,
        bis: jetzt() + ERWARTET_SEKUNDEN,
    });
    schreiben(&zustand, &m);

    if art == "anmelden" {
        if !anmeldung
            .anfrage_bestaetigen(&anfrage)
            .await
            .map_err(|e| e.to_string())?
        {
            return Err("This request no longer exists.".into());
        }
    } else {
        let _ = anmeldung.anfrage_verwerfen(&anfrage).await;
    }

    // Schon im Verzeichnis (Nur-Schluessel)? Dann gleich die Kiste.
    if let Some(client) = crate::server_client(&zustand).await {
        let _ = direktpost::bereit(&zustand, &client).await;
    }

    Ok(())
}
