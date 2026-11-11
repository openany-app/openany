//! openany als Teilen-Ziel unter Android (docs/plan-viele-dateien.md,
//! Skizze C, Frage 8).
//!
//! Geteilte Dateien legt `MainActivity.kt` sofort als Kopie in den
//! Zwischenspeicher (`teilen/<stapel>/<Name>`) -- die Freigabe der teilenden
//! App gilt nur kurz. Hier wird nachgesehen, was bereitliegt, und es geht
//! in die Schlange aus D2, verteilt nach Art:
//!
//! * Bilder, Videos und Ton in die Mediathek -- ins gewählte Album,
//! * Textdateien und Dokumente (PDF, Office, Text) nach Dokumente,
//! * alles andere nach Dateien.
//!
//! Die Kopien im Zwischenspeicher räumt der Lauf weg, sobald sie übernommen
//! (oder abgewiesen) sind ([`aufraeumen`]).

use crate::Zustand;
use openany_store::{Eingereiht, NeueUebernahme};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn fehler(e: impl ToString) -> String {
    e.to_string()
}

fn ordner(zustand: &Zustand) -> PathBuf {
    zustand.zwischenspeicher.join("teilen")
}

/// Wohin ein Stapel kommt, sobald er in der Schlange ist -- damit er nicht
/// noch einmal als „bereit“ gilt.
fn laufend(zustand: &Zustand) -> PathBuf {
    zustand.zwischenspeicher.join("teilen-laeuft")
}

/// Wohin eine geteilte Datei gehört (Frage 8).
fn zone_fuer(mime: &str) -> &'static str {
    let m = mime.to_ascii_lowercase();
    if m.starts_with("image/") || m.starts_with("video/") || m.starts_with("audio/") {
        "galerie"
    } else if m.starts_with("text/")
        || m == "application/pdf"
        || m == "application/rtf"
        || m.contains("msword")
        || m.contains("ms-excel")
        || m.contains("ms-powerpoint")
        || m.contains("officedocument")
        || m.contains("opendocument")
    {
        "documents"
    } else {
        "files"
    }
}

fn mime_von(pfad: &Path) -> String {
    mime_guess::from_path(pfad)
        .first()
        .map(|m| m.essence_str().to_string())
        .unwrap_or_default()
}

/// Was in `wurzel` liegt: je Datei Pfad und Zone.
fn bereit(wurzel: &Path) -> Vec<(PathBuf, &'static str)> {
    let mut alle = Vec::new();
    let Ok(stapel) = std::fs::read_dir(wurzel) else {
        return alle;
    };
    for s in stapel.flatten() {
        alle.extend(bereit_in(&s.path()));
    }
    alle.sort();
    alle
}

/// Die Dateien eines Stapels.
fn bereit_in(stapel: &Path) -> Vec<(PathBuf, &'static str)> {
    let mut alle: Vec<_> = std::fs::read_dir(stapel)
        .into_iter()
        .flatten()
        .flatten()
        .map(|d| d.path())
        .filter(|p| p.is_file())
        .map(|p| {
            let zone = zone_fuer(&mime_von(&p));
            (p, zone)
        })
        .collect();
    alle.sort();
    alle
}

#[derive(Serialize, Default)]
pub struct Geteilt {
    /// Bilder, Videos, Ton -- für die Mediathek.
    medien: usize,
    /// Für Dokumente.
    dokumente: usize,
    /// Für Dateien.
    sonst: usize,
}

/// Liegt etwas Geteiltes bereit? Die Oberfläche fragt beim Start und nach
/// dem Ereignis `openany-teilen`.
#[tauri::command]
pub async fn geteilt_stand(zustand: tauri::State<'_, Arc<Zustand>>) -> Result<Geteilt, String> {
    let mut g = Geteilt::default();
    for (_, zone) in bereit(&ordner(&zustand)) {
        match zone {
            "galerie" => g.medien += 1,
            "documents" => g.dokumente += 1,
            _ => g.sonst += 1,
        }
    }
    Ok(g)
}

/// Übernehmen: Medien ins Album `album` (oder in ein neues mit dem Namen
/// `neues_album`; ohne beides auf die oberste Ebene der Mediathek), den Rest
/// nach Art auf die oberste Ebene von Dokumente bzw. Dateien.
#[tauri::command]
pub async fn geteilt_uebernehmen(
    app: tauri::AppHandle,
    zustand: tauri::State<'_, Arc<Zustand>>,
    album: Option<String>,
    neues_album: Option<String>,
) -> Result<Eingereiht, String> {
    // Erst die Stapel beiseite: Was in der Schlange ist, liegt nicht mehr
    // bereit -- auch wenn die App gleich wieder fragt.
    let laeuft = laufend(&zustand);
    std::fs::create_dir_all(&laeuft).map_err(fehler)?;
    let mut dateien = Vec::new();
    for s in std::fs::read_dir(ordner(&zustand))
        .into_iter()
        .flatten()
        .flatten()
    {
        let ziel = laeuft.join(s.file_name());
        if std::fs::rename(s.path(), &ziel).is_ok() {
            dateien.extend(bereit_in(&ziel));
        }
    }
    let album = match neues_album
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty())
    {
        Some(name) if dateien.iter().any(|(_, z)| *z == "galerie") => {
            let (uuid, neu) = {
                let speicher = zustand.speicher.lock().await;
                crate::ablegen::behaelter(&speicher, "galerie", None, &name)?
            };
            if neu {
                crate::dateibefehle::neuen_behaelter_waehlen(&zustand, &uuid, None).await;
            }
            Some(uuid)
        }
        _ => album,
    };
    if let Some(a) = album.as_deref() {
        crate::uebernahme::ziel_pruefen(&zustand, "galerie", Some(a)).await?;
    }
    let neue: Vec<NeueUebernahme> = dateien
        .iter()
        .filter_map(|(p, zone)| {
            let ziel = (*zone == "galerie").then(|| album.clone()).flatten();
            let mut n = crate::uebernahme::als_neue(p, zone, ziel.as_deref())?;
            // Jede geteilte Kopie ist eine eigene Quelle; doppelt wird sie
            // erst über den Inhalt erkannt (gleicher Abdruck im selben Ziel).
            n.quelle_schluessel = format!("teilen|{}", p.to_string_lossy());
            Some(n)
        })
        .collect();
    crate::uebernahme::platz_pruefen(&zustand, &neue)?;
    crate::uebernahme::einreihen_neue(&app, &zustand, &neue).await
}

/// Verwerfen, was bereitliegt.
#[tauri::command]
pub async fn geteilt_verwerfen(zustand: tauri::State<'_, Arc<Zustand>>) -> Result<(), String> {
    let o = ordner(&zustand);
    if o.exists() {
        std::fs::remove_dir_all(&o).map_err(fehler)?;
    }
    Ok(())
}

/// Nach einer Übernahme: die geteilte Kopie weg (und ein leerer Stapel).
pub(crate) fn aufraeumen(zustand: &Zustand, quelle: &str) {
    let p = Path::new(quelle);
    if !p.starts_with(laufend(zustand)) {
        return;
    }
    let _ = std::fs::remove_file(p);
    if let Some(stapel) = p.parent() {
        let _ = std::fs::remove_dir(stapel);
    }
}

#[cfg(test)]
mod tests {
    use super::zone_fuer;

    #[test]
    fn jede_art_an_ihren_platz() {
        assert_eq!(zone_fuer("image/heic"), "galerie");
        assert_eq!(zone_fuer("video/mp4"), "galerie");
        assert_eq!(zone_fuer("audio/mpeg"), "galerie");
        assert_eq!(zone_fuer("application/pdf"), "documents");
        assert_eq!(zone_fuer("text/plain"), "documents");
        assert_eq!(
            zone_fuer("application/vnd.openxmlformats-officedocument.wordprocessingml.document"),
            "documents"
        );
        assert_eq!(
            zone_fuer("application/vnd.oasis.opendocument.spreadsheet"),
            "documents"
        );
        assert_eq!(zone_fuer("application/zip"), "files");
        assert_eq!(zone_fuer(""), "files");
    }
}
