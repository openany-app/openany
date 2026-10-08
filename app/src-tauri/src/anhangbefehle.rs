//! Notiz-Anhaenge ANLEGEN und OEFFNEN -- das Gegenstueck zu `notizbuch_anhaenge`.
//!
//! Bis zum 17.09.2026 konnte die App Anhaenge nur lesen. `uploadNoteAsset`
//! warf "Anhaenge kann dieses Programm noch nicht speichern", und ein Bild,
//! das jemand in der App in eine Notiz setzen wollte, hatte keinen Weg.
//!
//! **Dieselbe Aufteilung wie drueben** (`NoteAssetService::upload`): Ein
//! Bild geht in die Galerie, in ein Album "Notizen" beziehungsweise
//! "Notizen: <Mappe>"; ein Dokument (PDF, Text, Office) in die Akte
//! "Notizen" der Dokumente; alles andere in einen Ordner "Notizen" der
//! Dateien. Die Bytes gehoeren dem Ziel; der Anhang ist nur die Zuordnung
//! "dieser Pfad im Text meint jene Sache".
//!
//! **Warum die Behaelter dieselben Namen tragen wie drueben.** Nach einem
//! Abgleich liegt das Album "Notizen" des Servers auch hier -- und wird
//! wiederverwendet, statt ein zweites daneben zu stellen. Nur wenn beide
//! Seiten eines anlegen, bevor sie sich je gesehen haben, gibt es zwei.
//!
//! **Die Bytes kommen in Stuecken** ueber dieselben Befehle wie beim
//! Speichern einer Datei (`datei_hochladen_stueck`); nur Anfang und Ende
//! sind eigene, weil am Ende drei Zeilen entstehen statt einer.

use crate::dateibefehle::{self, hochladen_nehmen, Hochladen};
use crate::galeriebefehle::{exif_json, vorschau_jpeg};
use crate::Zustand;
use openany_store::{Album, Anhang, Bild, Datei, Protokoll, Speicher};
use serde::Serialize;
use std::sync::Arc;

/// Die "Zone" eines angemeldeten Anhang-Speicherns. Kein Ort, nur ein
/// Kennzeichen: `hochladen_nehmen` gibt eine Ladung nur an den heraus, der
/// sie angemeldet hat.
const ZONE: &str = "anhang";

/// Wie der Sammel-Behaelter heisst -- drueben und hier.
const BEHAELTER: &str = "Notizen";

fn fehler(e: impl ToString) -> String {
    e.to_string()
}

/// Was die Notiz-Oberflaeche nach dem Speichern bekommt -- dieselben Felder
/// wie drueben (`{id, path, target_type}`), dazu `name` fuer den Einfuege-
/// Dialog und `pfad`, weil `assetUrl` in der App synchron aus dem lokalen
/// Pfad baut.
#[derive(Serialize)]
pub struct NeuerAnhang {
    id: String,
    path: String,
    target_type: String,
    target_uuid: String,
    name: String,
    pfad: Option<String>,
}

/// Eine zum Oeffnen bereitgelegte Datei (nur auf dem Telefon; auf dem
/// Schreibtisch oeffnet die Schale selbst).
#[derive(Serialize)]
pub struct Geoeffnet {
    pfad: String,
    mime: String,
}

/// Ein Anhang-Speichern anmelden. Gibt das Kennzeichen zurueck, unter dem
/// die Stuecke kommen (`datei_hochladen_stueck`).
#[tauri::command]
pub async fn notizbuch_anhang_beginnen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    mappe: Option<u32>,
    name: String,
    mime: String,
) -> Result<String, String> {
    let name = dateibefehle::name_saeubern(&name)?;
    // Die Mappe jetzt aufloesen, nicht erst am Ende: Wer in eine eben
    // geloeschte Mappe speichert, soll das hoeren, bevor er Bytes schickt.
    let mappenpfad = {
        let speicher = zustand.speicher.lock().await;
        crate::pfad_zu(&speicher, mappe)?
    };
    let ladung = zustand.inhalte.ladung().map_err(fehler)?;
    let kennzeichen = uuid::Uuid::new_v4().simple().to_string();
    zustand.hochladungen.lock().map_err(fehler)?.insert(
        kennzeichen.clone(),
        Hochladen {
            ladung,
            zone: ZONE.into(),
            ordner: mappenpfad,
            name,
            mime,
        },
    );
    Ok(kennzeichen)
}

/// Fertig: ablegen, die Sache anlegen (Bild oder Datei, samt Behaelter) und
/// die Zuordnung schreiben -- alle mit `Protokoll::Merken`, damit sie beim
/// naechsten Abgleich hinausgehen. Der Anhang als LETZTES: So steht er im
/// Protokoll hinter seinem Ziel und geht auch in dieser Reihenfolge hinueber.
#[tauri::command]
pub async fn notizbuch_anhang_fertig(
    zustand: tauri::State<'_, Arc<Zustand>>,
    kennzeichen: String,
) -> Result<NeuerAnhang, String> {
    let h = hochladen_nehmen(&zustand, &kennzeichen, ZONE)?;
    let mappe = h.ordner.clone().unwrap_or_default();
    let (abdruck, groesse) = zustand.inhalte.ablegen(h.ladung).map_err(fehler)?;
    let ist_bild = h.mime.starts_with("image/");

    // Vorschau und EXIF wie in der Galerie -- nicht auf dem Laufzeitkern.
    let (vorschau, exif) = if ist_bild {
        let pfad = zustand
            .inhalte
            .pfad(&abdruck)
            .ok_or("fingerprint without path")?;
        let (jpeg, exif) = tauri::async_runtime::spawn_blocking(move || {
            let bytes = std::fs::read(pfad).unwrap_or_default();
            (vorschau_jpeg(&bytes), exif_json(&bytes))
        })
        .await
        .map_err(fehler)?;
        let vorschau = match jpeg {
            Some(jpeg) => {
                let mut l = zustand.inhalte.ladung().map_err(fehler)?;
                l.schreiben(&jpeg).map_err(fehler)?;
                Some(zustand.inhalte.ablegen(l).map_err(fehler)?.0)
            }
            None => None,
        };
        (vorschau, exif)
    } else {
        (None, String::new())
    };

    let speicher = zustand.speicher.lock().await;

    let (ziel_art, ziel_uuid) = if ist_bild {
        let album = behaelter_album(&speicher, &mappe)?;
        let b = Bild {
            uuid: uuid::Uuid::new_v4().to_string(),
            album: Some(album),
            name: h.name.clone(),
            mime: h.mime.clone(),
            groesse,
            abdruck: Some(abdruck.clone()),
            vorschau: vorschau.clone(),
            exif,
            ..Default::default()
        };
        speicher
            .bild_schreiben(&b, Protokoll::Merken)
            .map_err(fehler)?;
        ("media", b.uuid)
    } else {
        let zone = if ist_dokument(&h.mime, &h.name) {
            "documents"
        } else {
            "files"
        };
        let ordner = behaelter_ordner(&speicher, zone)?;
        let name = dateibefehle::freier_name(&speicher, zone, Some(&ordner), &h.name)?;
        let d = Datei {
            uuid: uuid::Uuid::new_v4().to_string(),
            zone: zone.into(),
            ist_ordner: false,
            eltern: Some(ordner),
            name,
            groesse,
            mime: if h.mime.is_empty() {
                "application/octet-stream".into()
            } else {
                h.mime.clone()
            },
            abdruck: Some(abdruck.clone()),
            ..Default::default()
        };
        speicher
            .datei_schreiben(&d, Protokoll::Merken)
            .map_err(fehler)?;
        ("file", d.uuid)
    };

    let pfad = freier_pfad(&speicher, &mappe, &h.name)?;
    let a = Anhang {
        mappe,
        pfad: pfad.clone(),
        ziel_art: ziel_art.into(),
        ziel_uuid: ziel_uuid.clone(),
        groesse,
        abdruck: Some(abdruck.clone()),
        vorschau: vorschau.clone(),
    };
    speicher
        .anhang_schreiben(&a, Protokoll::Merken)
        .map_err(fehler)?;

    // Die Vorschau zuerst, wie in `notizbuch_anhaenge`: Im Flusstext ist sie
    // die richtige Groesse.
    let lokal = vorschau.as_deref().unwrap_or(&abdruck);

    Ok(NeuerAnhang {
        id: a.schluessel(),
        path: pfad,
        target_type: ziel_art.into(),
        target_uuid: ziel_uuid,
        name: h.name,
        pfad: zustand
            .inhalte
            .pfad(lokal)
            .filter(|p| p.is_file())
            .map(|p| p.to_string_lossy().to_string()),
    })
}

/// Einen Anhang mit einem anderen Programm oeffnen -- das Original, nicht
/// die Vorschau. Fehlt es hier, wird es erst geholt (Nachbarn, dann Server).
///
/// Derselbe Weg wie `datei_oeffnen`: Die Datei wird unter
/// `cache/oeffnen/<zufall>/<Name>` bereitgelegt; auf Android reicht sie der
/// `FileProvider` in `MainActivity` hinaus, nur aus diesem Ordner.
#[tauri::command]
pub async fn notizbuch_anhang_oeffnen(
    app: tauri::AppHandle,
    zustand: tauri::State<'_, Arc<Zustand>>,
    mappe: Option<u32>,
    path: String,
) -> Result<Option<Geoeffnet>, String> {
    let (a, mime) = {
        let speicher = zustand.speicher.lock().await;
        let mappenpfad = crate::pfad_zu(&speicher, mappe)?.unwrap_or_default();
        let a = speicher
            .anhang(&mappenpfad, &path)
            .map_err(fehler)?
            .ok_or("This attachment no longer exists.")?;
        let mime = if a.ziel_art == "media" {
            speicher.bild(&a.ziel_uuid).map_err(fehler)?.map(|b| b.mime)
        } else {
            speicher
                .datei(&a.ziel_uuid)
                .map_err(fehler)?
                .map(|d| d.mime)
        };
        (a, mime.unwrap_or_default())
    };

    // Kein Abdruck heisst: Die Gegenseite hat nie gesagt, wo die Bytes
    // liegen. Das ist nicht "noch nicht geholt", das ist nicht holbar.
    let abdruck = a
        .abdruck
        .ok_or("No content is known for this attachment.")?;
    dateibefehle::von_geraeten_holen(&zustand, &abdruck, a.groesse).await?;
    let quelle = zustand
        .inhalte
        .pfad(&abdruck)
        .filter(|p| p.is_file())
        .ok_or("The content did not arrive.")?;

    let dateiname = a.pfad.rsplit('/').next().unwrap_or(&a.pfad).to_string();
    let ordner = zustand
        .zwischenspeicher
        .join("oeffnen")
        .join(uuid::Uuid::new_v4().simple().to_string());
    std::fs::create_dir_all(&ordner).map_err(fehler)?;
    let ziel = ordner.join(dateiname);
    if std::fs::hard_link(&quelle, &ziel).is_err() {
        std::fs::copy(&quelle, &ziel).map_err(fehler)?;
    }

    #[cfg(desktop)]
    {
        use tauri_plugin_opener::OpenerExt;
        let _ = mime;
        app.opener()
            .open_path(ziel.to_string_lossy(), None::<&str>)
            .map_err(fehler)?;
        Ok(None)
    }
    #[cfg(mobile)]
    {
        let _ = app;
        Ok(Some(Geoeffnet {
            pfad: ziel.to_string_lossy().to_string(),
            mime,
        }))
    }
}

/* ── Behaelter ────────────────────────────────────────────────────────── */

/// Das Album fuer Bilder aus Notizen dieser Mappe -- vorhanden oder neu.
fn behaelter_album(speicher: &Speicher, mappe: &str) -> Result<String, String> {
    let name = match mappe.rsplit('/').next().filter(|m| !m.is_empty()) {
        Some(m) => format!("{BEHAELTER}: {m}"),
        None => BEHAELTER.to_string(),
    };
    if let Some(a) = speicher
        .alben_unter(None)
        .map_err(fehler)?
        .into_iter()
        .find(|a| a.name == name)
    {
        return Ok(a.uuid);
    }
    let a = Album {
        uuid: uuid::Uuid::new_v4().to_string(),
        name,
        beschreibung: String::new(),
        eltern: None,
        papierkorb_at: None,
        geaendert_at: String::new(),
    };
    speicher
        .album_schreiben(&a, Protokoll::Merken)
        .map_err(fehler)?;
    Ok(a.uuid)
}

/// Der Ordner "Notizen" oben in der Zone -- vorhanden oder neu.
fn behaelter_ordner(speicher: &Speicher, zone: &str) -> Result<String, String> {
    if let Some(d) = speicher
        .dateien_im_ordner(zone, None)
        .map_err(fehler)?
        .into_iter()
        .find(|d| d.ist_ordner && d.name == BEHAELTER)
    {
        return Ok(d.uuid);
    }
    let d = Datei {
        uuid: uuid::Uuid::new_v4().to_string(),
        zone: zone.into(),
        ist_ordner: true,
        eltern: None,
        name: BEHAELTER.into(),
        ..Default::default()
    };
    speicher
        .datei_schreiben(&d, Protokoll::Merken)
        .map_err(fehler)?;
    Ok(d.uuid)
}

/// Dokument oder sonstige Datei -- dieselbe Regel wie drueben
/// (`NoteAssetService::isDocument`): nach dem Typ, sonst nach der Endung,
/// weil die Typ-Erkennung nicht ueberall verlaesslich ist.
fn ist_dokument(mime: &str, name: &str) -> bool {
    if mime.starts_with("text/") || mime == "application/pdf" {
        return true;
    }
    let endung = name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    matches!(
        endung.as_str(),
        "pdf"
            | "doc"
            | "docx"
            | "xls"
            | "xlsx"
            | "ppt"
            | "pptx"
            | "odt"
            | "ods"
            | "odp"
            | "rtf"
            | "csv"
            | "md"
            | "txt"
    )
}

/// Ein Pfad, den es in dieser Mappe noch nicht gibt: "bild.png", sonst
/// "bild (2).png". Wie drueben (`uniquePath`).
fn freier_pfad(speicher: &Speicher, mappe: &str, name: &str) -> Result<String, String> {
    let name = name.split_whitespace().collect::<Vec<_>>().join(" ");
    let name = if name.is_empty() {
        "datei".to_string()
    } else {
        name
    };
    if speicher.anhang(mappe, &name).map_err(fehler)?.is_none() {
        return Ok(name);
    }
    let (stamm, endung) = match name.rfind('.') {
        Some(i) if i > 0 => (&name[..i], &name[i..]),
        _ => (name.as_str(), ""),
    };
    for n in 2..1000 {
        let versuch = format!("{stamm} ({n}){endung}");
        if speicher.anhang(mappe, &versuch).map_err(fehler)?.is_none() {
            return Ok(versuch);
        }
    }
    Err("Too many attachments with the same name in this folder.".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dokumente_werden_wie_drueben_erkannt() {
        assert!(ist_dokument("application/pdf", "x.bin"));
        assert!(ist_dokument("text/plain", "x"));
        assert!(ist_dokument("application/octet-stream", "Brief.DOCX"));
        assert!(!ist_dokument("application/zip", "archiv.zip"));
        assert!(!ist_dokument("video/mp4", "film.mp4"));
    }

    #[test]
    fn ein_freier_pfad_zaehlt_hoch() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        let a = Anhang {
            mappe: "Reise".into(),
            pfad: "strand.jpg".into(),
            ziel_art: "media".into(),
            ziel_uuid: "b-1".into(),
            ..Default::default()
        };
        s.anhang_schreiben(&a, Protokoll::Still).unwrap();

        assert_eq!(
            freier_pfad(&s, "Reise", "strand.jpg").unwrap(),
            "strand (2).jpg"
        );
        assert_eq!(freier_pfad(&s, "", "strand.jpg").unwrap(), "strand.jpg");
        assert_eq!(freier_pfad(&s, "Reise", "  ").unwrap(), "datei");
    }

    /// Der Behaelter wird wiederverwendet -- ein zweiter Aufruf legt kein
    /// zweites Album "Notizen" daneben.
    #[test]
    fn der_behaelter_entsteht_einmal() {
        let s = Speicher::im_arbeitsspeicher().unwrap();

        let erstes = behaelter_album(&s, "").unwrap();
        let zweites = behaelter_album(&s, "").unwrap();
        assert_eq!(erstes, zweites);

        let mappe = behaelter_album(&s, "Reise/Sommer").unwrap();
        assert_ne!(mappe, erstes);
        assert_eq!(s.album(&mappe).unwrap().unwrap().name, "Notizen: Sommer");

        let ordner = behaelter_ordner(&s, "documents").unwrap();
        assert_eq!(behaelter_ordner(&s, "documents").unwrap(), ordner);
        assert_ne!(behaelter_ordner(&s, "files").unwrap(), ordner);
    }
}
