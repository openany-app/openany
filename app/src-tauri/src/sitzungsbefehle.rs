//! Gemeinsam schreiben (docs/plan-gemeinsam-schreiben.md, Schritt G2).
//!
//! Die Oberflaeche spricht mit genau einer Sitzung -- entweder **hier**
//! (dieses Geraet ist der Knoten, `geraet` leer) oder **beim fuehrenden
//! Geraet** im Netz (`geraet` = dessen Fingerabdruck). Die Befehle verbergen
//! den Unterschied; die Yjs-Seite (`app/src/sitzung.js`) kennt nur sie.
//!
//! **Wer ist der Knoten?**
//! - Eine fremde Notiz aus einem Projekt: das Geraet der Person, die sie
//!   freigegeben hat (wie beim Sperren, `projektbefehle::notiz_herkunft`).
//!   Ist es nicht in der Naehe, gibt es keine Sitzung -- nur lesen.
//! - Eine eigene Notiz: Laeuft auf einem eigenen gepaarten Geraet in der
//!   Naehe schon eine Sitzung, dort; sonst hier.
//! - Ein Whiteboard (`whiteboard:<uuid>`, W3b): das Geraet des Besitzers --
//!   laeuft auf einem seiner Geraete in der Naehe schon eine Sitzung, dort.
//!   Gespeichert wird beim Server, vom Knoten.
//! - Eine Notiz aus einer geteilten Mappe eines SERVER-Projekts
//!   (docs/plan-whiteboard.md, W3a): das Geraet in der Naehe, das sie fuehrt
//!   (Frage `hat`, ohne Ausweis); beigetreten wird mit Ausweis
//!   (serverpruefer.rs). Ist die Mappe die eigene: hier.

use std::sync::Arc;

use base64::{engine::general_purpose::STANDARD as B64, Engine};
use openany_nahbereich::sitzungen::http::{
    leitung_beitritt, leitung_neues, BeitrittLeitung, NeuesLeitung, SitzungPost,
};
use openany_nahbereich::Gastgeber;
use openany_store::Protokoll;
use serde::Serialize;

use crate::Zustand;

#[derive(Serialize)]
pub struct SitzungStart {
    /// Leer: dieses Geraet ist der Knoten. Sonst der Fingerabdruck des
    /// fuehrenden Geraets -- alle weiteren Befehle bekommen ihn zurueck.
    geraet: String,
    projekt: String,
    /// Die zk_id der Notiz beim Knoten.
    notiz: String,
    /// Server-Projekt: die geteilte Mappe. Sonst leer.
    freigabe: String,
    #[serde(flatten)]
    beitritt: BeitrittLeitung,
}

fn fehler(e: impl ToString) -> String {
    e.to_string()
}

async fn ich(zustand: &Zustand) -> Result<openany_nahbereich::Person, String> {
    zustand
        .gastgeber
        .person()
        .ok_or_else(|| "This device has no person yet.".to_string())
}

/// Hier beitreten -- dieses Geraet ist der Knoten.
async fn hier_beitreten(zustand: &Zustand, zk_id: &str) -> Result<SitzungStart, String> {
    let notiz = zustand
        .speicher
        .lock()
        .await
        .notiz(zk_id)
        .map_err(fehler)?
        .filter(|n| !n.im_papierkorb())
        .ok_or("This note no longer exists.")?;
    let g = &zustand.gastgeber;
    // Haelt ein Mitglied mit alter App die Sperre, schreibt hier niemand mit.
    if let Some(h) = g.sperren.lock().ok().and_then(|mut s| s.halter(zk_id)) {
        if !h.geraet.is_empty() {
            return Err(format!(
                "{} is editing this note with an older app.",
                h.name
            ));
        }
    }
    hier_beitreten_mit(zustand, zk_id, notiz.titel, notiz.inhalt).await
}

/// Hier beitreten mit gegebenem Ausgangsstand (auch fuer Whiteboards).
async fn hier_beitreten_mit(
    zustand: &Zustand,
    schluessel: &str,
    titel: String,
    inhalt: String,
) -> Result<SitzungStart, String> {
    let ich = ich(zustand).await?;
    let g = &zustand.gastgeber;
    let name = if ich.name.is_empty() {
        g.mein_name()
    } else {
        ich.name.clone()
    };
    let b = g
        .sitzungen
        .lock()
        .map_err(fehler)?
        .beitreten(schluessel, &ich.personen_id, &name, "");
    Ok(SitzungStart {
        freigabe: String::new(),
        geraet: String::new(),
        projekt: String::new(),
        notiz: schluessel.to_string(),
        beitritt: leitung_beitritt(b, titel, inhalt),
    })
}

/// Ein Whiteboard gemeinsam (W3b).
async fn whiteboard_beitreten(
    zustand: &Arc<Zustand>,
    projekt: String,
    schluessel: String,
    wb: &str,
) -> Result<SitzungStart, String> {
    let eigen = crate::projektwhiteboard::eigenes(zustand, &projekt, wb).await;
    let mut post = SitzungPost {
        projekt: projekt.clone(),
        notiz: schluessel.clone(),
        ..Default::default()
    };
    // Wer in der Naehe fuehrt es -- und wo laeuft schon eine Sitzung?
    let mut kandidaten = Vec::new();
    for d in crate::direktbefehle::in_der_naehe(zustand).await {
        let Ok(v) = anrufen::<serde_json::Value>(zustand, &d.fingerabdruck, "hat", &post).await
        else {
            continue;
        };
        if v.get("hat").and_then(|h| h.as_bool()) == Some(true) {
            let laeuft = v.get("laeuft").and_then(|l| l.as_bool()) == Some(true);
            kandidaten.push((laeuft, d.fingerabdruck));
        }
    }
    kandidaten.sort_by_key(|(laeuft, _)| !laeuft);
    // Das eigene: nur zu einem anderen eigenen Geraet, wenn dort schon
    // gezeichnet wird; sonst ist dieses der Knoten.
    if eigen.is_some() {
        kandidaten.retain(|(laeuft, _)| *laeuft);
    }
    let mut letzter = String::from(
        "Read only: the device of the person who created this whiteboard is not nearby right now.",
    );
    for (_, fp) in kandidaten {
        post.ausweis = Some(crate::serverpruefer::ausweis(zustand, &fp)?);
        match anrufen::<BeitrittLeitung>(zustand, &fp, "beitreten", &post).await {
            Ok(b) => {
                return Ok(SitzungStart {
                    freigabe: String::new(),
                    geraet: fp,
                    projekt,
                    notiz: schluessel,
                    beitritt: b,
                })
            }
            Err(e) => letzter = e,
        }
    }
    match eigen {
        Some((titel, inhalt)) => {
            let mut s = hier_beitreten_mit(zustand, &schluessel, titel, inhalt).await?;
            s.projekt = projekt;
            Ok(s)
        }
        None => Err(letzter),
    }
}

async fn anrufen<T: serde::de::DeserializeOwned>(
    zustand: &Zustand,
    geraet: &str,
    was: &str,
    post: &SitzungPost,
) -> Result<T, String> {
    let adresse = crate::adresse_von_geraet(zustand, geraet).await?;
    let ident = crate::geraet_identitaet(zustand)?;
    openany_nahbereich::anruf::sitzung(
        &ident,
        &adresse,
        openany_nahbereich::DIENST_PORT,
        geraet,
        was,
        post,
    )
    .await
}

/// Einer Sitzung beitreten. `projekt` gesetzt: `id` ist die Kennung der
/// Notiz im Projekt (`von:zk_id`). Sonst ist `id` die zk_id einer eigenen
/// Notiz.
#[tauri::command]
pub async fn sitzung_beitreten(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: Option<String>,
    id: String,
    freigabe: Option<String>,
) -> Result<SitzungStart, String> {
    let ich = ich(&zustand).await?;

    if let (Some(projekt), Some(wb)) = (
        projekt.clone().filter(|p| !p.is_empty()),
        openany_nahbereich::sitzungen::ist_whiteboard(&id),
    ) {
        return whiteboard_beitreten(&zustand, projekt, id.clone(), wb).await;
    }

    if let (Some(projekt), Some(freigabe)) = (
        projekt.clone().filter(|p| !p.is_empty()),
        freigabe.filter(|f| !f.is_empty()),
    ) {
        return server_beitreten(&zustand, projekt, freigabe, id).await;
    }

    if let Some(projekt) = projekt.filter(|p| !p.is_empty()) {
        let (von, anker, zk_id) =
            crate::projektbefehle::notiz_herkunft(&zustand, &projekt, &id).await?;
        // Die eigene Notiz in einem Projekt: Knoten bin ich.
        if von == ich.personen_id || ich.frueher.contains(&von) {
            return hier_beitreten(&zustand, &zk_id).await;
        }
        let geraete = crate::projektbefehle::geraete_von(&zustand, &von, &anker).await;
        if geraete.is_empty() {
            return Err(
                "Read only: the device that shared this note is not nearby right now.".into(),
            );
        }
        let post = SitzungPost {
            projekt: projekt.clone(),
            notiz: zk_id.clone(),
            ..Default::default()
        };
        let mut letzter = String::new();
        for d in geraete {
            match anrufen::<BeitrittLeitung>(&zustand, &d.fingerabdruck, "beitreten", &post).await {
                Ok(b) => {
                    return Ok(SitzungStart {
                        freigabe: String::new(),
                        geraet: d.fingerabdruck,
                        projekt,
                        notiz: zk_id,
                        beitritt: b,
                    })
                }
                Err(e) => letzter = e,
            }
        }
        return Err(letzter);
    }

    // Eine eigene Notiz: erst bei den eigenen Geraeten in der Naehe
    // nachsehen, ob dort schon geschrieben wird.
    let eigene: Vec<_> = crate::direktbefehle::in_der_naehe(&zustand)
        .await
        .into_iter()
        .filter(|d| zustand.gastgeber.ist_gepaart(&d.fingerabdruck))
        .collect();
    let post = SitzungPost {
        notiz: id.clone(),
        ..Default::default()
    };
    for d in eigene {
        let laeuft = anrufen::<serde_json::Value>(&zustand, &d.fingerabdruck, "laeuft", &post)
            .await
            .ok()
            .and_then(|v| v.get("laeuft").and_then(|l| l.as_bool()))
            .unwrap_or(false);
        if laeuft {
            if let Ok(b) =
                anrufen::<BeitrittLeitung>(&zustand, &d.fingerabdruck, "beitreten", &post).await
            {
                return Ok(SitzungStart {
                    freigabe: String::new(),
                    geraet: d.fingerabdruck,
                    projekt: String::new(),
                    notiz: id,
                    beitritt: b,
                });
            }
        }
    }
    hier_beitreten(&zustand, &id).await
}

/// Eine Notiz aus einer geteilten Mappe eines Server-Projekts (W3a).
async fn server_beitreten(
    zustand: &Arc<Zustand>,
    projekt: String,
    freigabe: String,
    zk_id: String,
) -> Result<SitzungStart, String> {
    use openany_nahbereich::sitzungen::ServerPruefer;
    // Die eigene Mappe: Knoten bin ich.
    if crate::serverpruefer::ServerPruefen
        .hat(&projekt, &freigabe, &zk_id)
        .await
    {
        let mut s = hier_beitreten(zustand, &zk_id).await?;
        s.projekt = projekt;
        s.freigabe = freigabe;
        return Ok(s);
    }
    let mut post = SitzungPost {
        projekt: projekt.clone(),
        freigabe: freigabe.clone(),
        notiz: zk_id.clone(),
        ..Default::default()
    };
    let mut letzter =
        String::from("Read only: the device that shared this note is not nearby right now.");
    for d in crate::direktbefehle::in_der_naehe(zustand).await {
        let hat = anrufen::<serde_json::Value>(zustand, &d.fingerabdruck, "hat", &post)
            .await
            .ok()
            .and_then(|v| v.get("hat").and_then(|h| h.as_bool()))
            .unwrap_or(false);
        if !hat {
            continue;
        }
        // Erst jetzt zeigen, wer ruft -- nur dem Geraet, das die Notiz fuehrt.
        post.ausweis = Some(crate::serverpruefer::ausweis(zustand, &d.fingerabdruck)?);
        match anrufen::<BeitrittLeitung>(zustand, &d.fingerabdruck, "beitreten", &post).await {
            Ok(b) => {
                return Ok(SitzungStart {
                    freigabe,
                    geraet: d.fingerabdruck,
                    projekt,
                    notiz: zk_id,
                    beitritt: b,
                })
            }
            Err(e) => letzter = e,
        }
    }
    Err(letzter)
}

fn bytes(b64: &str) -> Result<Vec<u8>, String> {
    B64.decode(b64).map_err(fehler)
}

/// Aenderungen und/oder die eigene Awareness einliefern.
#[tauri::command]
pub async fn sitzung_senden(
    zustand: tauri::State<'_, Arc<Zustand>>,
    geraet: String,
    projekt: String,
    notiz: String,
    kennung: String,
    aenderungen: Vec<String>,
    awareness: Option<String>,
) -> Result<u64, String> {
    if geraet.is_empty() {
        let a = aenderungen
            .iter()
            .map(|x| bytes(x))
            .collect::<Result<Vec<_>, _>>()?;
        let w = awareness.as_deref().map(bytes).transpose()?;
        return zustand
            .gastgeber
            .sitzungen
            .lock()
            .map_err(fehler)?
            .senden(&notiz, &kennung, "", a, w)
            .map_err(fehler);
    }
    let v: serde_json::Value = anrufen(
        &zustand,
        &geraet,
        "senden",
        &SitzungPost {
            projekt,
            notiz,
            kennung,
            aenderungen,
            awareness,
            ..Default::default()
        },
    )
    .await?;
    Ok(v.get("nr").and_then(|n| n.as_u64()).unwrap_or(0))
}

/// Auf Neues warten (hoechstens 25 s).
#[tauri::command]
pub async fn sitzung_warten(
    zustand: tauri::State<'_, Arc<Zustand>>,
    geraet: String,
    projekt: String,
    notiz: String,
    kennung: String,
    ab: u64,
) -> Result<NeuesLeitung, String> {
    if geraet.is_empty() {
        let sitzungen = zustand.gastgeber.sitzungen.clone();
        return openany_nahbereich::sitzungen::warten(&sitzungen, &notiz, &kennung, "", ab)
            .await
            .map(leitung_neues)
            .map_err(fehler);
    }
    anrufen(
        &zustand,
        &geraet,
        "warten",
        &SitzungPost {
            projekt,
            notiz,
            kennung,
            ab,
            ..Default::default()
        },
    )
    .await
}

/// Einen zusammengefassten Stand einliefern (alle Aenderungen bis `bis`).
#[tauri::command]
pub async fn sitzung_zusammenfassen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    geraet: String,
    projekt: String,
    notiz: String,
    kennung: String,
    stand: String,
    bis: u64,
) -> Result<(), String> {
    if geraet.is_empty() {
        let s = bytes(&stand)?;
        return zustand
            .gastgeber
            .sitzungen
            .lock()
            .map_err(fehler)?
            .zusammenfassen(&notiz, &kennung, "", s, bis)
            .map_err(fehler);
    }
    let _: serde_json::Value = anrufen(
        &zustand,
        &geraet,
        "zusammenfassen",
        &SitzungPost {
            projekt,
            notiz,
            kennung,
            stand: Some(stand),
            bis,
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

/// Speichern -- nur, wer der Schreiber der Sitzung ist. Hier: die eigene
/// Notiz schreiben. Beim fuehrenden Geraet: dort speichern und die Kopie im
/// Projekt hier nachziehen (`id` = Kennung im Projekt).
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn sitzung_speichern(
    zustand: tauri::State<'_, Arc<Zustand>>,
    geraet: String,
    projekt: String,
    notiz: String,
    kennung: String,
    titel: Option<String>,
    inhalt: String,
    id: Option<String>,
    freigabe: Option<String>,
) -> Result<(), String> {
    if geraet.is_empty() {
        if !zustand
            .gastgeber
            .sitzungen
            .lock()
            .map_err(fehler)?
            .ist_schreiber(&notiz, &kennung, "")
        {
            return Err("Someone else saves this session.".into());
        }
        if let Some(wb) = openany_nahbereich::sitzungen::ist_whiteboard(&notiz) {
            return crate::projektwhiteboard::fuer_sitzung_speichern(
                &zustand, &projekt, wb, titel, &inhalt,
            )
            .await;
        }
        let speicher = zustand.speicher.lock().await;
        let mut n = speicher
            .notiz(&notiz)
            .map_err(fehler)?
            .filter(|n| !n.im_papierkorb())
            .ok_or("This note no longer exists.")?;
        if let Some(t) = titel {
            n.titel = t;
        }
        n.inhalt = inhalt;
        n.geaendert_at = String::new();
        return speicher
            .notiz_schreiben(&n, Protokoll::Merken)
            .map_err(fehler);
    }
    let _: serde_json::Value = anrufen(
        &zustand,
        &geraet,
        "speichern",
        &SitzungPost {
            projekt: projekt.clone(),
            notiz,
            kennung,
            titel: titel.clone(),
            inhalt: Some(inhalt.clone()),
            freigabe: freigabe.unwrap_or_default(),
            ..Default::default()
        },
    )
    .await?;
    if let (false, Some(id)) = (projekt.is_empty(), id) {
        let speicher = zustand.speicher.lock().await;
        if let Some(mut s) = speicher.projektsache(&projekt, &id).map_err(fehler)? {
            if let Some(t) = titel {
                s.felder["titel"] = serde_json::json!(t);
            }
            s.felder["inhalt"] = serde_json::json!(inhalt);
            s.geaendert_at = String::new();
            speicher
                .projektsache_schreiben(&s, Protokoll::Still)
                .map_err(fehler)?;
        }
    }
    Ok(())
}

/// Die Sitzung verlassen.
#[tauri::command]
pub async fn sitzung_verlassen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    geraet: String,
    projekt: String,
    notiz: String,
    kennung: String,
) -> Result<(), String> {
    if geraet.is_empty() {
        zustand
            .gastgeber
            .sitzungen
            .lock()
            .map_err(fehler)?
            .verlassen(&notiz, &kennung, "");
        return Ok(());
    }
    let _: serde_json::Value = anrufen(
        &zustand,
        &geraet,
        "verlassen",
        &SitzungPost {
            projekt,
            notiz,
            kennung,
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}
