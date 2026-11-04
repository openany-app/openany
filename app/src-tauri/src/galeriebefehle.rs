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
    auf_server: bool,
    auf_server_geerbt: bool,
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
    /// Gibt es ein Standbild/Vorschau ueberhaupt -- auch wenn es nur „bei
    /// Bedarf" hier liegt und `vorschau_pfad` deshalb fehlt?
    hat_vorschau: bool,
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
    /// Hat es ein selbst gewähltes Titelbild?
    titelbild_eigen: bool,
    /// Liegt darin nur Ton? Dann zeigt die Kachel ein Musiksymbol.
    nur_ton: bool,
    /// Auf diesem Gerät markiert zum Behalten.
    behalten: bool,
    auf_server: bool,
    auf_server_geerbt: bool,
}

fn pfad_wenn_da(zustand: &Zustand, abdruck: Option<&str>) -> Option<String> {
    let a = abdruck?;
    zustand
        .inhalte
        .pfad(a)
        .filter(|p| p.is_file())
        .map(|p| p.to_string_lossy().to_string())
}

fn bild_anzeige(zustand: &Zustand, b: Bild, server: &crate::aufserver::Gewuenscht) -> BildAnzeige {
    let auf_server = server.eigen(&b.uuid);
    let auf_server_geerbt = server.geerbt(b.album.as_deref());
    let exif = serde_json::from_str::<Value>(&b.exif).unwrap_or(Value::Null);
    let pfad = pfad_wenn_da(zustand, b.abdruck.as_deref());
    let ohne_bild = ist_video_mime(&b.mime) || ist_ton_mime(&b.mime);
    let dauer = exif.get("dauer").cloned();
    BildAnzeige {
        auf_server,
        auf_server_geerbt,
        id: b.uuid,
        file_name: b.name.clone(),
        name: b.name,
        mime_type: b.mime,
        size: b.groesse,
        custom_properties: match (exif.is_null(), dauer) {
            (true, _) => json!({}),
            (false, Some(dauer)) => json!({ "exif": exif, "dauer": dauer }),
            (false, None) => json!({ "exif": exif }),
        },
        vorhanden: pfad.is_some(),
        hat_vorschau: b.vorschau.is_some(),
        // Ohne Vorschau fällt ein BILD aufs Original zurück. Video und Ton nicht:
        // Ein <img> damit lädt die Datei für nichts, und die Kachel soll ihr
        // Symbol zeigen (MediaGrid.vue).
        vorschau_pfad: pfad_wenn_da(zustand, b.vorschau.as_deref()).or_else(|| {
            if ohne_bild {
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
    let (behalten, sichtbar) = zustand
        .einstellungen
        .try_lock()
        .map(|e| {
            let sichtbar = crate::dateibefehle::sichtbare_bereiche(&e, speicher)
                .is_none_or(|b| b.contains(&a.uuid));
            (e.behalten.contains(&a.uuid), sichtbar)
        })
        .unwrap_or((false, true));
    let (anzahl, titel) = if sichtbar {
        speicher.album_kennzahlen(&a.uuid).map_err(fehler)?
    } else {
        (0, None)
    };
    let titelbild_eigen = titel.as_ref().is_some_and(Bild::ist_titelbild);
    let nur_ton = anzahl > 0 && titel.is_none();
    let titelbild_pfad = titel.and_then(|b| {
        pfad_wenn_da(zustand, b.vorschau.as_deref())
            .or_else(|| pfad_wenn_da(zustand, b.abdruck.as_deref()))
    });
    let server =
        crate::aufserver::Gewuenscht::lesen(speicher, &Default::default()).map_err(fehler)?;
    let auf_server = server.eigen(&a.uuid);
    let auf_server_geerbt = server.geerbt(a.eltern.as_deref());
    Ok(AlbumAnzeige {
        id: a.uuid,
        name: a.name,
        description: a.beschreibung,
        parent_id: a.eltern,
        media_count: anzahl,
        titelbild_pfad,
        titelbild_eigen,
        nur_ton,
        behalten,
        auf_server,
        auf_server_geerbt,
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
    /// Dieses Album ist auf diesem Gerät nicht gewählt: nur der Name.
    nur_name: bool,
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
    let nur_name = {
        let e = zustand.einstellungen.lock().await;
        crate::dateibefehle::sichtbare_bereiche(&e, &speicher).is_some_and(|b| !b.contains(&id))
    };
    let server =
        crate::aufserver::Gewuenscht::lesen(&speicher, &Default::default()).map_err(fehler)?;
    let media = speicher
        .bilder_im_album(Some(&id))
        .map_err(fehler)?
        .into_iter()
        .filter(|b| !nur_name && !b.ist_titelbild())
        .map(|b| bild_anzeige(&zustand, b, &server))
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
        nur_name,
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
    drop(speicher);
    crate::dateibefehle::neuen_behaelter_waehlen(&zustand, &a.uuid, a.eltern.as_deref()).await;
    let speicher = zustand.speicher.lock().await;
    album_anzeige(&zustand, &speicher, a)
}

/// Das Album für neue Kamerafotos. Es entsteht beim ersten Foto und bleibt auf
/// dem Gerät gewählt, das fotografiert.
#[tauri::command]
pub async fn galerie_kamera_album(
    zustand: tauri::State<'_, Arc<Zustand>>,
    name: String,
) -> Result<String, String> {
    let speicher = zustand.speicher.lock().await;
    let da = speicher
        .alben_unter(None)
        .map_err(fehler)?
        .into_iter()
        .find(|a| a.name == name);
    let uuid = match da {
        Some(a) => a.uuid,
        None => {
            let a = Album {
                uuid: uuid::Uuid::new_v4().to_string(),
                name,
                ..Default::default()
            };
            speicher
                .album_schreiben(&a, Protokoll::Merken)
                .map_err(fehler)?;
            a.uuid
        }
    };
    let mut e = zustand.einstellungen.lock().await;
    if e.regel() != "alles" && !e.behalten.contains(&uuid) {
        e.behalten.push(uuid.clone());
        e.schreiben(&zustand.einstellungspfad()).map_err(fehler)?;
    }
    Ok(uuid)
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
    let server =
        crate::aufserver::Gewuenscht::lesen(&speicher, &Default::default()).map_err(fehler)?;
    let items = speicher
        .bilder_im_album(None)
        .map_err(fehler)?
        .into_iter()
        .map(|b| bild_anzeige(&zustand, b, &server))
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

/// Ein Bild in ein anderes Album legen; `album = None` heißt: ohne Album.
#[tauri::command]
pub async fn galerie_bild_verschieben(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: String,
    album: Option<String>,
) -> Result<(), String> {
    let speicher = zustand.speicher.lock().await;
    if let Some(ziel) = album.as_deref() {
        lebendiges_album(&speicher, ziel)?;
    }
    let mut b = speicher
        .bild(&id)
        .map_err(fehler)?
        .ok_or("This picture no longer exists.")?;
    // Das Datum bleibt: Ohne Aufnahmedaten ordnet die Galerie das Foto danach ein.
    b.album = album;
    speicher
        .bild_schreiben(&b, Protokoll::Merken)
        .map_err(fehler)
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
    // EIN VIDEO ODER TON WIRD HIER NICHT GELESEN. `fs::read` holt die ganze Datei in
    // den Speicher -- bei einem Foto ein paar MB, bei einem Video ein
    // Gigabyte, und das Programm fiele auf dem Tablet um. Das `image`-Crate
    // könnte ohnehin nichts damit anfangen. Das Standbild zieht das
    // Betriebssystem (`standbilder_nachziehen`, weiter unten in dieser Funktion).
    let ist_video = ist_video_mime(&h.mime);
    let pfad = zustand
        .inhalte
        .pfad(&abdruck)
        .ok_or("fingerprint without path")?;
    let (vorschau, exif) = if ist_video || ist_ton_mime(&h.mime) {
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
    let server =
        crate::aufserver::Gewuenscht::lesen(&speicher, &Default::default()).map_err(fehler)?;
    drop(speicher);

    // Ein Video bekommt sein Standbild gleich -- vom Betriebssystem.
    if ist_video {
        let id = b.uuid.clone();
        standbilder_nachziehen(&zustand).await;
        let speicher = zustand.speicher.lock().await;
        if let Ok(Some(neu)) = speicher.bild(&id) {
            return Ok(bild_anzeige(&zustand, neu, &server));
        }
    }
    Ok(bild_anzeige(&zustand, b, &server))
}

fn ist_video_mime(mime: &str) -> bool {
    mime.starts_with("video/") || mime == "application/mp4"
}

fn ist_ton_mime(mime: &str) -> bool {
    mime.starts_with("audio/")
}

/// Die Dauer gehört zum Ton wie das Aufnahmedatum zum Foto: Sie steht im
/// `exif`-Feld und geht so in den Abgleich.
fn dauer_eintragen(exif: &str, dauer: f64) -> String {
    let mut aus = serde_json::from_str::<Value>(exif)
        .ok()
        .and_then(|w| w.as_object().cloned())
        .unwrap_or_default();
    aus.insert("dauer".into(), json!((dauer * 10.0).round() / 10.0));
    Value::Object(aus).to_string()
}

/// Die Dauer einer Tondatei, die die Oberfläche beim Hinzufügen gelesen hat.
#[tauri::command]
pub async fn galerie_ton_dauer(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: String,
    dauer: f64,
) -> Result<BildAnzeige, String> {
    let speicher = zustand.speicher.lock().await;
    let mut b = speicher
        .bild(&id)
        .map_err(fehler)?
        .ok_or("This recording no longer exists.")?;
    if !ist_ton_mime(&b.mime) {
        return Err("Only audio has a duration here.".into());
    }
    b.exif = dauer_eintragen(&b.exif, dauer);
    speicher
        .bild_schreiben(&b, Protokoll::Merken)
        .map_err(fehler)?;
    let server =
        crate::aufserver::Gewuenscht::lesen(&speicher, &Default::default()).map_err(fehler)?;
    drop(speicher);
    Ok(bild_anzeige(&zustand, b, &server))
}

/// Die eigenen Titelbilder eines Albums löschen, samt Inhalt, wenn ihn nichts
/// sonst nennt. `ausser`: das Bild, das es bleiben soll.
fn titelbilder_loeschen(
    zustand: &Zustand,
    speicher: &Speicher,
    album: &str,
    ausser: Option<&str>,
) -> Result<(), String> {
    for b in speicher.bilder_im_album(Some(album)).map_err(fehler)? {
        if !b.ist_titelbild() || ausser == Some(b.uuid.as_str()) {
            continue;
        }
        for abdruck in speicher
            .bild_loeschen(&b.uuid, Protokoll::Merken)
            .map_err(fehler)?
        {
            if !speicher.abdruck_benutzt(&abdruck).map_err(fehler)? {
                let _ = zustand.inhalte.entfernen(&abdruck);
            }
        }
    }
    Ok(())
}

/// Ein frisch ins Album gelegtes Bild wird sein Titelbild; ein früheres geht.
#[tauri::command]
pub async fn galerie_titelbild_setzen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    album: String,
    id: String,
) -> Result<(), String> {
    let speicher = zustand.speicher.lock().await;
    let mut b = speicher
        .bild(&id)
        .map_err(fehler)?
        .ok_or("This picture no longer exists.")?;
    if b.album.as_deref() != Some(album.as_str()) || !b.mime.starts_with("image/") {
        return Err("Only a picture in this album can be its cover.".into());
    }
    let mut exif = serde_json::from_str::<Value>(&b.exif)
        .ok()
        .and_then(|w| w.as_object().cloned())
        .unwrap_or_default();
    exif.insert("titelbild".into(), json!(true));
    b.exif = Value::Object(exif).to_string();
    speicher
        .bild_schreiben(&b, Protokoll::Merken)
        .map_err(fehler)?;
    titelbilder_loeschen(&zustand, &speicher, &album, Some(&id))
}

#[tauri::command]
pub async fn galerie_titelbild_entfernen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    album: String,
) -> Result<(), String> {
    let speicher = zustand.speicher.lock().await;
    titelbilder_loeschen(&zustand, &speicher, &album, None)
}

/// Die Bytes einer Tondatei für den Player der Oberfläche.
///
/// WARUM NICHT ÜBER `asset:`: Auf Android spielen Medien aus der Ablage dort
/// nicht zuverlässig (siehe `galerie_oeffnen`). Als Blob schon -- darum die
/// Grenze; was größer ist, spielt der Player des Systems.
#[tauri::command]
pub async fn galerie_inhalt(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: String,
) -> Result<tauri::ipc::Response, String> {
    const HOECHSTENS: u64 = 64 * 1024 * 1024;
    let b = {
        let speicher = zustand.speicher.lock().await;
        speicher
            .bild(&id)
            .map_err(fehler)?
            .ok_or("This recording no longer exists.")?
    };
    let abdruck = b.abdruck.ok_or("This recording has no content.")?;
    if b.groesse > HOECHSTENS {
        return Err("This file is too large for the player.".into());
    }
    let bytes = zustand
        .inhalte
        .lesen(&abdruck, 0, HOECHSTENS as usize)
        .map_err(fehler)?;
    Ok(tauri::ipc::Response::new(bytes))
}

/// Eine fertige Aufnahme des Android-Recorders (`Tonaufnahme.kt`) in die
/// Mediathek. Der Pfad steht hier fest und kommt nicht von der Seite: So kann
/// sie keine andere Datei des Programms hineinziehen.
#[tauri::command]
pub async fn galerie_aufnahme_ablegen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    album: Option<String>,
    name: String,
    dauer: Option<f64>,
) -> Result<BildAnzeige, String> {
    let datei = zustand.zwischenspeicher.join("aufnahme").join("fertig.m4a");
    if !datei.is_file() {
        return Err("There is no finished recording.".into());
    }
    if let Some(a) = album.as_deref() {
        lebendiges_album(&*zustand.speicher.lock().await, a)?;
    }
    // Eine lange Aufnahme zu prüfsummen dauert: nicht auf dem Laufzeitkern und
    // nicht unter der Sperre der Datenbank.
    let inhalte = zustand.inhalte.clone();
    let (abdruck, groesse) =
        tauri::async_runtime::spawn_blocking(move || inhalte.uebernehmen(&datei))
            .await
            .map_err(fehler)?
            .map_err(fehler)?;
    let speicher = zustand.speicher.lock().await;
    let b = Bild {
        uuid: uuid::Uuid::new_v4().to_string(),
        album,
        name,
        mime: "audio/mp4".into(),
        groesse,
        abdruck: Some(abdruck),
        exif: dauer.map(|d| dauer_eintragen("", d)).unwrap_or_default(),
        ..Default::default()
    };
    speicher
        .bild_schreiben(&b, Protokoll::Merken)
        .map_err(fehler)?;
    let server =
        crate::aufserver::Gewuenscht::lesen(&speicher, &Default::default()).map_err(fehler)?;
    drop(speicher);
    Ok(bild_anzeige(&zustand, b, &server))
}

/// Ablegen, eintragen und zum Server schicken.
///
/// LOKAL `Still`, NICHT `Merken`: Zum Server geht das Standbild über seinen
/// eigenen Weg (`media_standbild`), und dessen neu kodierte Fassung kommt beim
/// nächsten Abgleich als `thumb_hash` zurück. Ein Protokolleintrag schöbe nur
/// den Eintrag noch einmal durch die Leitung.
/// Ein Standbild, das die Oberflaeche aus dem Video gezogen hat -- auf dem
/// Schreibtisch, wo es keinen `MediaMetadataRetriever` gibt
/// (docs/plan-desktop.md, 1d). Wie aus standbild.rs: hier ablegen und, wenn
/// verbunden, zum Server schicken.
#[tauri::command]
pub async fn galerie_standbild_setzen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: String,
    jpeg: String,
    dauer: Option<f64>,
) -> Result<(), String> {
    use base64::Engine;
    let jpeg = base64::engine::general_purpose::STANDARD
        .decode(jpeg.as_bytes())
        .map_err(fehler)?;
    if jpeg.is_empty() {
        return Err("No still image.".into());
    }
    standbild_ablegen(&zustand, &id, jpeg, dauer).await?;
    Ok(())
}

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
        // Auch Fotos: Der Server rechnet keine Vorschau, die App reicht ihre
        // nach (siehe `standbilder_nachziehen`).
        if !ist_video_mime(&b.mime) && !b.mime.starts_with("image/") {
            return Err("Only a picture or a video gets a preview.".into());
        }
        b.vorschau = Some(abdruck);
        speicher
            .bild_schreiben(&b, Protokoll::Still)
            .map_err(fehler)?;
        b
    };

    let am_server = standbild_hinauf(zustand, &b, jpeg, dauer).await;
    Ok((b, am_server))
}

/// Fehlende Vorschauen machen: für jedes Video und jedes Foto, dessen Inhalt
/// auf diesem Gerät liegt und das noch keine Vorschau hat. Beim Video mit dem
/// Werkzeug des Betriebssystems (`standbild.rs`, auf Android
/// `MediaMetadataRetriever`), beim Foto mit `vorschau_jpeg`.
///
/// FOTOS SEIT OKTOBER 2026. Der Server rechnet keine Vorschau mehr. Lädt die
/// App ein Foto hoch, bringt der Abgleich es drüben OHNE Vorschau zurück, die
/// lokale fällt damit weg -- und hier entsteht sie neu und geht hinauf. Ein
/// Foto, das nie zum Server reist, hat seine Vorschau seit dem Speichern und
/// kommt hier gar nicht vorbei.
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
    let offen: Vec<(String, std::path::PathBuf, bool)> = {
        let speicher = zustand.speicher.lock().await;
        let Ok(bilder) = speicher.lebendige_bilder() else {
            return 0;
        };
        bilder
            .into_iter()
            .filter(|b| {
                (ist_video_mime(&b.mime) || b.mime.starts_with("image/")) && b.vorschau.is_none()
            })
            .filter_map(|b| {
                let pfad = zustand.inhalte.pfad(b.abdruck.as_deref()?)?;
                let video = ist_video_mime(&b.mime);
                pfad.is_file().then_some((b.uuid, pfad, video))
            })
            .collect()
    };

    let mut entstanden = 0;
    for (id, pfad, video) in offen {
        let gezogen = tauri::async_runtime::spawn_blocking(move || {
            if video {
                crate::standbild::ziehen(&pfad).map(|s| (s.jpeg, s.dauer))
            } else {
                let bytes = std::fs::read(&pfad).ok()?;
                Some((vorschau_jpeg(&bytes), None))
            }
        })
        .await;
        let Ok(Some((Some(jpeg), dauer))) = gezogen else {
            continue;
        };
        if standbild_ablegen(zustand, &id, jpeg, dauer).await.is_ok() {
            entstanden += 1;
        }
    }
    entstanden
}

async fn standbild_hinauf(zustand: &Zustand, b: &Bild, jpeg: Vec<u8>, dauer: Option<f64>) -> bool {
    let id = b.uuid.as_str();
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
    // VERSCHLUESSELT (docs/plan-dateien-e2e.md, 7b): Liegt das Bild drueben
    // verschluesselt, geht auch die Vorschau mit seinem Schluessel zu -- und
    // das Bild wird vorgemerkt, damit sein Kopf die Vorschau nennt.
    let schluessel = {
        let speicher = zustand.speicher.lock().await;
        b.abdruck
            .as_deref()
            .and_then(|a| speicher.inhaltsschluessel(a).ok().flatten())
    };
    let liegt_dort = {
        let speicher = zustand.speicher.lock().await;
        speicher
            .sachen_dort(openany_client::OpenanyClient::basis(&client))
            .is_ok_and(|d| d.contains_key(id))
    };
    if !liegt_dort {
        return false;
    }
    let Some(k) = schluessel else {
        return client.media_standbild(id, jpeg, dauer).await.is_ok();
    };
    let Ok(c) = openany_direktpost::dateischloss::bytes_verschluesseln(&k.schluessel, &jpeg) else {
        return false;
    };
    let chiffrat: String = {
        use sha2::{Digest, Sha256};
        Sha256::digest(&c)
            .iter()
            .map(|x| format!("{x:02x}"))
            .collect()
    };
    if client.media_standbild(id, c, None).await.is_err() {
        return false;
    }
    if let Some(v) = &b.vorschau {
        let speicher = zustand.speicher.lock().await;
        let _ = speicher.inhaltsschluessel_merken(&openany_store::Inhaltsschluessel {
            abdruck: v.clone(),
            schluessel: k.schluessel,
            chiffrat,
            groesse: 0,
        });
        let _ = speicher.merken(&openany_client::Art::Medium, id, openany_client::Was::Da);
    }
    true
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

/// Ein Bild aufs Gerät legen; siehe `dateibefehle::original_aufs_geraet`.
#[tauri::command]
pub async fn galerie_bild_aufs_geraet(
    app: tauri::AppHandle,
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: String,
) -> Result<Option<String>, String> {
    let b = {
        let speicher = zustand.speicher.lock().await;
        speicher
            .bild(&id)
            .map_err(fehler)?
            .ok_or("This picture no longer exists.")?
    };
    let abdruck = b.abdruck.ok_or("This picture has no content.")?;
    crate::dateibefehle::original_aufs_geraet(app, &zustand, &abdruck, b.groesse, b.name).await
}

/// Ein Video der Galerie mit dem Videoplayer des Systems abspielen.
///
/// WARUM NICHT IN DER WEBANSICHT: Inhalte aus der Ablage kommen dort über
/// abgefangene Anfragen (`asset:`), und damit spielen Videos nicht
/// zuverlässig -- der Player holt die ersten Stücke und hängt dann am Ende
/// der Datei. Der Player des Systems liest die Datei direkt.
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
    fn dauer_steht_neben_den_fotodaten() {
        let exif: Value =
            serde_json::from_str(&dauer_eintragen(r#"{"date":"2026:08:15 14:03:00"}"#, 75.26))
                .expect("JSON");
        assert_eq!(exif["dauer"], json!(75.3));
        assert_eq!(exif["date"], json!("2026:08:15 14:03:00"));
        let leer: Value = serde_json::from_str(&dauer_eintragen("", 3.0)).expect("JSON");
        assert_eq!(leer["dauer"], json!(3.0));
    }

    #[test]
    fn kein_bild_keine_vorschau_und_kein_exif() {
        assert!(vorschau_jpeg(b"kein Bild").is_none());
        assert_eq!(exif_json(b"kein Bild"), "");
    }
}
