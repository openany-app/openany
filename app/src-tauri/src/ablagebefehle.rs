//! Dateien aus der App hinaus legen und von außen wählen -- auf dem
//! Schreibtisch über die Dialoge des Systems (docs/plan-desktop.md, 1b).
//!
//! Unter Android tut das `MainActivity.kt` (`openanyAblage`); diese Befehle
//! gibt es dort zwar, sie lehnen aber ab.
//!
//! **Den Pfad wählt der Mensch, nicht die Seite.** Der Dialog öffnet sich
//! HIER, in Rust. Ein Befehl „schreib diese Bytes nach X" wäre eine Tür, durch
//! die eine fremde Zeile im Webview jede Datei des Nutzers überschreiben
//! könnte.

use std::path::PathBuf;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn speichern_unter(app: &tauri::AppHandle, name: &str, endung: Option<&str>) -> Option<PathBuf> {
    use tauri_plugin_dialog::DialogExt;
    let mut d = app.dialog().file().set_file_name(name);
    if let Some(e) = endung {
        d = d.add_filter(e, &[e]);
    }
    d.blocking_save_file().and_then(|p| p.into_path().ok())
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn oeffnen(app: &tauri::AppHandle, endung: &str) -> Option<PathBuf> {
    use tauri_plugin_dialog::DialogExt;
    app.dialog()
        .file()
        .add_filter(endung, &[endung])
        .blocking_pick_file()
        .and_then(|p| p.into_path().ok())
}

/// Bytes (Base64) unter einem Namen ablegen, den Ort wählt der Mensch.
/// `None`: abgebrochen.
#[tauri::command]
pub async fn aufs_geraet(
    app: tauri::AppHandle,
    name: String,
    daten: String,
) -> Result<Option<String>, String> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(daten.as_bytes())
            .map_err(|e| e.to_string())?;
        tauri::async_runtime::spawn_blocking(move || {
            let Some(ziel) = speichern_unter(&app, &name, None) else {
                return Ok(None);
            };
            std::fs::write(&ziel, bytes).map_err(|e| e.to_string())?;
            Ok(Some(ziel.to_string_lossy().to_string()))
        })
        .await
        .map_err(|e| e.to_string())?
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = (app, name, daten);
        Err("Not available on this platform.".into())
    }
}

/// Die fertige Sicherung aus dem Arbeitsordner an einen Ort nach Wahl.
/// `None`: abgebrochen (die Datei bleibt dann liegen, bis zum nächsten
/// Anlegen oder Verwerfen).
pub async fn sicherung_ablegen_mit_dialog(
    app: tauri::AppHandle,
    quelle: PathBuf,
) -> Result<Option<String>, String> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        tauri::async_runtime::spawn_blocking(move || {
            let name = quelle
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let Some(ziel) = speichern_unter(&app, &name, Some("age")) else {
                return Ok(None);
            };
            if std::fs::rename(&quelle, &ziel).is_err() {
                std::fs::copy(&quelle, &ziel).map_err(|e| e.to_string())?;
                let _ = std::fs::remove_file(&quelle);
            }
            Ok(Some(ziel.to_string_lossy().to_string()))
        })
        .await
        .map_err(|e| e.to_string())?
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = (app, quelle);
        Err("Not available on this platform.".into())
    }
}

/// Eine Sicherungsdatei wählen. `None`: abgebrochen.
#[tauri::command]
pub async fn sicherung_waehlen(app: tauri::AppHandle) -> Result<Option<String>, String> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        tauri::async_runtime::spawn_blocking(move || {
            Ok(oeffnen(&app, "age").map(|p| p.to_string_lossy().to_string()))
        })
        .await
        .map_err(|e| e.to_string())?
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = app;
        Err("Not available on this platform.".into())
    }
}
