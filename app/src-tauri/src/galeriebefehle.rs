/*
 * Die Galerie (Phase 3, Stufe C) -- dieselben Antworten wie drüben unter
 * /api/albums und /api/gallery, damit `src/quellen/galerie.js` nur umverpackt.
 *
 * BILDER KOMMEN ALS PFAD, NICHT ALS BYTES. Die geteilte Arbeitsfläche braucht
 * für jedes Bild sofort eine Adresse (`<img :src>`). Die Oberfläche macht aus
 * dem Pfad mit `convertFileSrc` eine Adresse des Asset-Protokolls; Tauri
 * liefert die Datei dann direkt aus der Ablage, ohne Umweg durch Rust-Aufrufe
 * und ohne Kopie im Speicher der Webansicht. Freigegeben ist nur `inhalte/`
 * (tauri.conf.json).
 *
 * VORSCHAU UND EXIF ENTSTEHEN BEIM SPEICHERN, hier auf dem Gerät: ein JPEG mit
 * langer Seite 480 px, gedreht, wie das Foto es verlangt, und Aufnahmedatum
 * und GPS in derselben Form wie drüben (`custom_properties.exif`).
 */

use crate::Zustand;
use openany_store::{Album, Bild, Protokoll, Speicher};
use serde::Serialize;
use serde_json::{json, Value};
use std::io::Cursor;
use std::sync::Arc;

fn fehler(e: impl ToString) -> String {
    e.to_string()
}

/* ── Vorschau und EXIF ─────────────────────────────────────────────────── */

/// Ein Vorschaubild: JPEG, lange Seite höchstens 480 px, in der Drehung, die
/// das Foto angibt. `None`, wenn das Bild nicht zu lesen ist.
pub fn vorschau_jpeg(bytes: &[u8]) -> Option<Vec<u8>> {
    use image::{DynamicImage, ImageDecoder, ImageReader};

    let mut decoder = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()?
        .into_decoder()
        .ok()?;
    let drehung = decoder.orientation().ok();
    let mut bild = DynamicImage::from_decoder(decoder).ok()?;
    if let Some(d) = drehung {
        bild.apply_orientation(d);
    }
    let klein = bild.thumbnail(480, 480).to_rgb8();
    let mut aus = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut aus, 80)
        .encode_image(&klein)
        .ok()?;
    Some(aus)
}

/// Aufnahmedatum und GPS wie drüben: `{"date": "YYYY:MM:DD HH:MM:SS",
/// "gps": {"lat": …, "lng": …}}`. Leer, wenn nichts zu lesen ist.
pub fn exif_json(bytes: &[u8]) -> String {
    use exif::{In, Reader, Tag, Value as EV};

    let Ok(exif) = Reader::new().read_from_container(&mut Cursor::new(bytes)) else {
        return String::new();
    };
    let text = |tag| {
        exif.get_field(tag, In::PRIMARY)
            .and_then(|f| match &f.value {
                EV::Ascii(teile) => teile
                    .first()
                    .map(|b| String::from_utf8_lossy(b).trim().to_string()),
                _ => None,
            })
    };
    let grad = |tag, ref_tag| -> Option<f64> {
        let f = exif.get_field(tag, In::PRIMARY)?;
        let EV::Rational(r) = &f.value else {
            return None;
        };
        if r.len() < 3 {
            return None;
        }
        let mut wert = r[0].to_f64() + r[1].to_f64() / 60.0 + r[2].to_f64() / 3600.0;
        if matches!(text(ref_tag).as_deref(), Some("S") | Some("W")) {
            wert = -wert;
        }
        Some(wert)
    };

    let mut aus = serde_json::Map::new();
    if let Some(datum) = text(Tag::DateTimeOriginal).or_else(|| text(Tag::DateTime)) {
        aus.insert("date".into(), datum.into());
    }
    if let (Some(lat), Some(lng)) = (
        grad(Tag::GPSLatitude, Tag::GPSLatitudeRef),
        grad(Tag::GPSLongitude, Tag::GPSLongitudeRef),
    ) {
        aus.insert("gps".into(), json!({ "lat": lat, "lng": lng }));
    }
    if aus.is_empty() {
        String::new()
    } else {
        Value::Object(aus).to_string()
    }
}

/* ── Anzeigen ──────────────────────────────────────────────────────────── */

#[derive(Serialize)]
pub struct BildAnzeige {
    id: String,
    name: String,
    file_name: String,
    mime_type: String,
    size: u64,
    custom_properties: Value,
    /// Liegt das Original hier?
    vorhanden: bool,
    /// Pfade in der Ablage -- für `convertFileSrc`. `None`, wenn nicht hier.
    pfad: Option<String>,
    vorschau_pfad: Option<String>,
    created_at: String,
}

#[derive(Serialize)]
pub struct AlbumAnzeige {
    id: String,
    name: String,
    description: String,
    parent_id: Option<String>,
    media_count: usize,
    titelbild_pfad: Option<String>,
    /// Auf diesem Gerät markiert zum Behalten.
    behalten: bool,
}

fn pfad_wenn_da(zustand: &Zustand, abdruck: Option<&str>) -> Option<String> {
    let a = abdruck?;
    zustand
        .inhalte
        .pfad(a)
        .filter(|p| p.is_file())
        .map(|p| p.to_string_lossy().to_string())
}

fn bild_anzeige(zustand: &Zustand, b: Bild) -> BildAnzeige {
    let exif = serde_json::from_str::<Value>(&b.exif).unwrap_or(Value::Null);
    let pfad = pfad_wenn_da(zustand, b.abdruck.as_deref());
    let ist_video = ist_video_mime(&b.mime);
    BildAnzeige {
        id: b.uuid,
        file_name: b.name.clone(),
        name: b.name,
        mime_type: b.mime,
        size: b.groesse,
        custom_properties: if exif.is_null() {
            json!({})
        } else {
            json!({ "exif": exif })
        },
        vorhanden: pfad.is_some(),
        // Ohne Vorschau fällt ein BILD aufs Original zurück. Ein Video nicht:
        // Ein <img> mit einem Video darin lädt es für nichts, und die Kachel
        // soll ihr Film-Symbol zeigen (MediaGrid.vue).
        vorschau_pfad: pfad_wenn_da(zustand, b.vorschau.as_deref()).or_else(|| {
            if ist_video {
                None
            } else {
                pfad.clone()
            }
        }),
        pfad,
        created_at: b.geaendert_at,
    }
}

fn album_anzeige(zustand: &Zustand, speicher: &Speicher, a: Album) -> Result<AlbumAnzeige, String> {
    let behalten = zustand
        .einstellungen
        .try_lock()
        .map(|e| e.behalten.contains(&a.uuid))
        .unwrap_or(false);
    let (anzahl, titel) = speicher.album_kennzahlen(&a.uuid).map_err(fehler)?;
    let titelbild_pfad = titel.and_then(|b| {
        pfad_wenn_da(zustand, b.vorschau.as_deref())
            .or_else(|| pfad_wenn_da(zustand, b.abdruck.as_deref()))
    });
    Ok(AlbumAnzeige {
        id: a.uuid,
        name: a.name,
        description: a.beschreibung,
        parent_id: a.eltern,
        media_count: anzahl,
        titelbild_pfad,
        behalten,
    })
}

fn lebendiges_album(speicher: &Speicher, id: &str) -> Result<Album, String> {
    match speicher.album(id).map_err(fehler)? {
        Some(a) if a.papierkorb_at.is_none() => Ok(a),
        _ => Err("This album no longer exists.".into()),
    }
}

/* ── Befehle ───────────────────────────────────────────────────────────── */

#[derive(Serialize)]
pub struct Liste<T> {
    items: Vec<T>,
    next_page: Option<usize>,
}

#[tauri::command]
pub async fn galerie_alben(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Liste<AlbumAnzeige>, String> {
    let speicher = zustand.speicher.lock().await;
    let items = speicher
        .alben_unter(None)
        .map_err(fehler)?
        .into_iter()
        .map(|a| album_anzeige(&zustand, &speicher, a))
        .collect::<Result<_, _>>()?;
    Ok(Liste {
        items,
        next_page: None,
    })
}

#[derive(Serialize)]
pub struct Krume {
    id: String,
    name: String,
}

#[derive(Serialize)]
pub struct AlbumDetail {
    #[serde(flatten)]
    album: AlbumAnzeige,
    media: Vec<BildAnzeige>,
    children: Vec<AlbumAnzeige>,
    breadcrumb: Vec<Krume>,
    depth: usize,
}

#[tauri::command]
pub async fn galerie_album(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: String,
) -> Result<AlbumDetail, String> {
    let speicher = zustand.speicher.lock().await;
    let a = lebendiges_album(&speicher, &id)?;
    let weg = speicher.albumweg(&id).map_err(fehler)?;
    let media = speicher
        .bilder_im_album(Some(&id))
        .map_err(fehler)?
        .into_iter()
        .map(|b| bild_anzeige(&zustand, b))
        .collect();
    let children = speicher
        .alben_unter(Some(&id))
        .map_err(fehler)?
        .into_iter()
        .map(|c| album_anzeige(&zustand, &speicher, c))
        .collect::<Result<_, _>>()?;
    Ok(AlbumDetail {
        depth: weg.len() + 1,
        breadcrumb: weg
            .into_iter()
            .map(|w| Krume {
                id: w.uuid,
                name: w.name,
            })
            .collect(),
        album: album_anzeige(&zustand, &speicher, a)?,
        media,
        children,
    })
}

#[tauri::command]
pub async fn galerie_album_anlegen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    name: String,
    beschreibung: String,
    eltern: Option<String>,
) -> Result<AlbumAnzeige, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("Please enter a name.".into());
    }
    let speicher = zustand.speicher.lock().await;
    if let Some(e) = eltern.as_deref() {
        lebendiges_album(&speicher, e)?;
    }
    let a = Album {
        uuid: uuid::Uuid::new_v4().to_string(),
        name,
        beschreibung: beschreibung.trim().to_string(),
        eltern,
        ..Default::default()
    };
    speicher
        .album_schreiben(&a, Protokoll::Merken)
        .map_err(fehler)?;
    album_anzeige(&zustand, &speicher, a)
}

#[tauri::command]
pub async fn galerie_album_verschieben(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: String,
    eltern: Option<String>,
) -> Result<(), String> {
    let speicher = zustand.speicher.lock().await;
    let mut a = lebendiges_album(&speicher, &id)?;
    if let Some(ziel) = eltern.as_deref() {
        lebendiges_album(&speicher, ziel)?;
        if speicher.album_liegt_in(ziel, &id).map_err(fehler)? {
            return Err("An album cannot be inside itself.".into());
        }
    }
    a.eltern = eltern;
    a.geaendert_at = String::new();
    speicher
        .album_schreiben(&a, Protokoll::Merken)
        .map_err(fehler)
}

#[derive(Serialize)]
pub struct Baumalbum {
    id: String,
    name: String,
    parent_id: Option<String>,
}

#[derive(Serialize)]
pub struct Albenbaum {
    albums: Vec<Baumalbum>,
}

#[tauri::command]
pub async fn galerie_albenbaum(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Albenbaum, String> {
    let speicher = zustand.speicher.lock().await;
    Ok(Albenbaum {
        albums: speicher
            .alle_alben()
            .map_err(fehler)?
            .into_iter()
            .map(|a| Baumalbum {
                id: a.uuid,
                name: a.name,
                parent_id: a.eltern,
            })
            .collect(),
    })
}

#[tauri::command]
pub async fn galerie_album_papierkorb(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: String,
) -> Result<(), String> {
    let speicher = zustand.speicher.lock().await;
    speicher
        .album_papierkorb(&id, Protokoll::Merken)
        .map_err(fehler)?;
    Ok(())
}

#[tauri::command]
pub async fn galerie_bilder(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Liste<BildAnzeige>, String> {
    let speicher = zustand.speicher.lock().await;
    let items = speicher
        .bilder_im_album(None)
        .map_err(fehler)?
        .into_iter()
        .map(|b| bild_anzeige(&zustand, b))
        .collect();
    Ok(Liste {
        items,
        next_page: None,
    })
}

#[tauri::command]
pub async fn galerie_bild_papierkorb(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: String,
) -> Result<(), String> {
    let speicher = zustand.speicher.lock().await;
    speicher
        .bild_papierkorb(&id, Protokoll::Merken)
        .map_err(fehler)?;
    Ok(())
}

/// Ein Bild speichern: die Stücke kamen über `datei_hochladen_stueck`
/// (Zone `galerie`, Ordner = Album). Hier: ablegen, Vorschau und EXIF
/// erzeugen, eintragen.
#[tauri::command]
pub async fn galerie_bild_fertig(
    zustand: tauri::State<'_, Arc<Zustand>>,
    kennzeichen: String,
) -> Result<BildAnzeige, String> {
    let h = crate::dateibefehle::hochladen_nehmen(&zustand, &kennzeichen, "galerie")?;
    let (abdruck, groesse) = zustand.inhalte.ablegen(h.ladung).map_err(fehler)?;

    // Lesen, rechnen, ablegen -- nicht auf dem Laufzeitkern, ein 12-MP-Foto
    // zu dekodieren dauert auf einem Tablet eine Sekunde.
    //
    // EIN VIDEO WIRD HIER NICHT GELESEN. `fs::read` holt die ganze Datei in
    // den Speicher -- bei einem Foto ein paar MB, bei einem Video ein
    // Gigabyte, und das Programm fiele auf dem Tablet um. Das `image`-Crate
    // könnte ohnehin nichts damit anfangen. Das Standbild zieht das
    // Betriebssystem (`standbilder_nachziehen`, weiter unten in dieser Funktion).
    let ist_video = ist_video_mime(&h.mime);
    let pfad = zustand
        .inhalte
        .pfad(&abdruck)
        .ok_or("fingerprint without path")?;
    let (vorschau, exif) = if ist_video {
        (None, String::new())
    } else {
        tauri::async_runtime::spawn_blocking(move || {
            let bytes = std::fs::read(pfad).unwrap_or_default();
            (vorschau_jpeg(&bytes), exif_json(&bytes))
        })
        .await
        .map_err(fehler)?
    };
    let vorschau = match vorschau {
        Some(jpeg) => {
            let mut l = zustand.inhalte.ladung().map_err(fehler)?;
            l.schreiben(&jpeg).map_err(fehler)?;
            Some(zustand.inhalte.ablegen(l).map_err(fehler)?.0)
        }
        None => None,
    };

    let speicher = zustand.speicher.lock().await;
    if let Some(a) = h.ordner.as_deref() {
        lebendiges_album(&speicher, a)?;
    }
    let b = Bild {
        uuid: uuid::Uuid::new_v4().to_string(),
        album: h.ordner,
        name: h.name,
        mime: h.mime,
        groesse,
        abdruck: Some(abdruck),
        vorschau,
        exif,
        ..Default::default()
    };
    speicher
        .bild_schreiben(&b, Protokoll::Merken)
        .map_err(fehler)?;
    drop(speicher);

    // Ein Video bekommt sein Standbild gleich -- vom Betriebssystem.
    if ist_video {
        let id = b.uuid.clone();
        standbilder_nachziehen(&zustand).await;
        let speicher = zustand.speicher.lock().await;
        if let Ok(Some(neu)) = speicher.bild(&id) {
            return Ok(bild_anzeige(&zustand, neu));
        }
    }
    Ok(bild_anzeige(&zustand, b))
}

fn ist_video_mime(mime: &str) -> bool {
    mime.starts_with("video/") || mime == "application/mp4"
}

/// Ablegen, eintragen und zum Server schicken.
///
/// LOKAL `Still`, NICHT `Merken`: Zum Server geht das Standbild über seinen
/// eigenen Weg (`media_standbild`), und dessen neu kodierte Fassung kommt beim
/// nächsten Abgleich als `thumb_hash` zurück. Ein Protokolleintrag schöbe nur
/// den Eintrag noch einmal durch die Leitung.
async fn standbild_ablegen(
    zustand: &Zustand,
    id: &str,
    jpeg: Vec<u8>,
    dauer: Option<f64>,
) -> Result<(Bild, bool), String> {
    let mut l = zustand.inhalte.ladung().map_err(fehler)?;
    l.schreiben(&jpeg).map_err(fehler)?;
    let abdruck = zustand.inhalte.ablegen(l).map_err(fehler)?.0;

    let b = {
        let speicher = zustand.speicher.lock().await;
        let mut b = speicher
            .bild(id)
            .map_err(fehler)?
            .ok_or("This video no longer exists.")?;
        if !ist_video_mime(&b.mime) {
            return Err("Only a video gets a still frame.".into());
        }
        b.vorschau = Some(abdruck);
        speicher
            .bild_schreiben(&b, Protokoll::Still)
            .map_err(fehler)?;
        b
    };

    let am_server = standbild_hinauf(zustand, id, jpeg, dauer).await;
    Ok((b, am_server))
}

/// Fehlende Standbilder ziehen: für jedes Video, dessen Inhalt auf diesem
/// Gerät liegt und das noch keine Vorschau hat -- mit dem Werkzeug des
/// Betriebssystems (`standbild.rs`, auf Android `MediaMetadataRetriever`).
///
/// Läuft nach dem Hochladen, nach dem Holen eines Videos und nach jedem
/// Abgleich, auch im Hintergrund: Holt der Abgleich ein Video, das in Firefox
/// auf Android ohne Standbild hochgeladen wurde, entsteht es hier und geht
/// zum Server -- ohne dass jemand die Galerie öffnet. Gibt zurück, wie viele
/// entstanden sind.
///
/// Scheitert das Schicken zum Server (offline, noch nicht angekommen), bleibt
/// das Standbild hier. Bringt der nächste Abgleich den Eintrag drüben noch
/// ohne Standbild zurück, fehlt es hier wieder und wird neu gezogen und
/// geschickt -- so kommt es am Ende an.
pub(crate) async fn standbilder_nachziehen(zustand: &Zustand) -> usize {
    let offen: Vec<(String, std::path::PathBuf)> = {
        let speicher = zustand.speicher.lock().await;
        let Ok(bilder) = speicher.lebendige_bilder() else {
            return 0;
        };
        bilder
            .into_iter()
            .filter(|b| ist_video_mime(&b.mime) && b.vorschau.is_none())
            .filter_map(|b| {
                let pfad = zustand.inhalte.pfad(b.abdruck.as_deref()?)?;
                pfad.is_file().then_some((b.uuid, pfad))
            })
            .collect()
    };

    let mut entstanden = 0;
    for (id, pfad) in offen {
        let Ok(Some(s)) =
            tauri::async_runtime::spawn_blocking(move || crate::standbild::ziehen(&pfad)).await
        else {
            continue;
        };
        let Some(jpeg) = s.jpeg else {
            continue;
        };
        if standbild_ablegen(zustand, &id, jpeg, s.dauer).await.is_ok() {
            entstanden += 1;
        }
    }
    entstanden
}

async fn standbild_hinauf(zustand: &Zustand, id: &str, jpeg: Vec<u8>, dauer: Option<f64>) -> bool {
    let e = zustand.einstellungen.lock().await.clone();
    if !e.verbunden() {
        return false;
    }
    let Ok(Some(schluessel)) = zustand.schluesselablage().lesen() else {
        return false;
    };
    let Ok(client) =
        openany_client::OpenanyClient::neu_mit_ca(&e.openany_basis, &schluessel, e.ca().as_deref())
    else {
        return false;
    };
    client.media_standbild(id, jpeg, dauer).await.is_ok()
}

/// Das Original von einem gepaarten Gerät holen. Gibt den Pfad zurück.
#[tauri::command]
pub async fn galerie_bild_holen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: String,
) -> Result<String, String> {
    let b = {
        let speicher = zustand.speicher.lock().await;
        speicher
            .bild(&id)
            .map_err(fehler)?
            .ok_or("This picture no longer exists.")?
    };
    let abdruck = b.abdruck.ok_or("This picture has no content.")?;
    crate::dateibefehle::von_geraeten_holen(&zustand, &abdruck, b.groesse).await?;
    if ist_video_mime(&b.mime) {
        standbilder_nachziehen(&zustand).await;
    }
    pfad_wenn_da(&zustand, Some(&abdruck)).ok_or_else(|| "The picture did not arrive.".into())
}

/// Ein Video der Galerie mit dem Videoplayer des Systems abspielen.
///
/// WARUM NICHT IN DER WEBANSICHT (27.09.2026): Inhalte aus der Ablage kommen
/// dort über abgefangene Anfragen (`asset:`), und damit spielten Videos am
/// Tablet nicht zuverlässig -- der Player holte die ersten Stücke und hing
/// dann am Ende der Datei. Der Player des Systems liest die Datei direkt.
/// Gibt wie `datei_oeffnen` auf Android den Pfad zurück (MainActivity.kt gibt
/// ihn über den FileProvider weiter), am Schreibtisch öffnet die Schale selbst.
#[tauri::command]
pub async fn galerie_oeffnen(
    app: tauri::AppHandle,
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: String,
) -> Result<Option<String>, String> {
    let b = {
        let speicher = zustand.speicher.lock().await;
        speicher
            .bild(&id)
            .map_err(fehler)?
            .ok_or("This video no longer exists.")?
    };
    let abdruck = b.abdruck.ok_or("This video has no content.")?;
    crate::dateibefehle::zum_oeffnen(app, &zustand, &abdruck, &b.name)
}

/// Vom Papierkorb aus: zurück.
pub async fn wiederherstellen(zustand: &Zustand, art: &str, id: &str) -> Result<(), String> {
    let speicher = zustand.speicher.lock().await;
    if art == "album" {
        speicher.album_wiederherstellen(id, Protokoll::Merken)
    } else {
        speicher.bild_wiederherstellen(id, Protokoll::Merken)
    }
    .map_err(fehler)?;
    Ok(())
}

/// Vom Papierkorb aus: endgültig, und Inhalte, die niemand mehr nennt, gleich mit.
pub async fn endgueltig(zustand: &Zustand, art: &str, id: &str) -> Result<(), String> {
    let speicher = zustand.speicher.lock().await;
    let frei = if art == "album" {
        speicher.album_loeschen(id, Protokoll::Merken)
    } else {
        speicher.bild_loeschen(id, Protokoll::Merken)
    }
    .map_err(fehler)?;
    for abdruck in frei {
        if !speicher.abdruck_benutzt(&abdruck).map_err(fehler)? {
            let _ = zustand.inhalte.entfernen(&abdruck);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vorschau_eines_gedrehten_pngs() {
        let mut png = Vec::new();
        image::RgbImage::from_pixel(1200, 600, image::Rgb([200, 30, 30]))
            .write_to(&mut Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();
        let jpeg = vorschau_jpeg(&png).expect("lesbar");
        let klein = image::load_from_memory(&jpeg).unwrap();
        assert_eq!((klein.width(), klein.height()), (480, 240));
    }

    #[test]
    fn kein_bild_keine_vorschau_und_kein_exif() {
        assert!(vorschau_jpeg(b"kein Bild").is_none());
        assert_eq!(exif_json(b"kein Bild"), "");
    }
}
