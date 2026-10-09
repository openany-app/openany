//! Aktualisierung auf dem Schreibtisch (docs/plan-desktop.md, Schritt 3).
//!
//! Kein Store: `tauri-plugin-updater` fragt im oeffentlichen Repo nach, im
//! festen Release `desktop-aktuell`, je Plattform eine Datei
//! (`{{target}}-{{arch}}.json`, siehe tauri.conf.json). So kann jede
//! Plattform ihre eigene Versionsnummer haben. Was heruntergeladen wird, ist
//! mit `~/anyx/geheim/updater.key` signiert; die App prueft gegen den
//! oeffentlichen Schluessel in der Konfiguration und installiert nichts
//! anderes.
//!
//! Unter Android und iOS gibt es das nicht -- dort aktualisiert der Store
//! oder die neue APK; die Befehle lehnen ab.

use serde::Serialize;

#[derive(Serialize)]
pub struct Neu {
    version: String,
    notizen: Option<String>,
}

#[derive(Serialize)]
pub struct Lage {
    version: String,
    neu: Option<Neu>,
}

#[tauri::command]
pub async fn aktualisierung_lage(app: tauri::AppHandle) -> Result<Lage, String> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        use tauri_plugin_updater::UpdaterExt;
        let version = app.package_info().version.to_string();
        let neu = app
            .updater()
            .map_err(|e| e.to_string())?
            .check()
            .await
            .map_err(|e| e.to_string())?
            .map(|u| Neu {
                version: u.version,
                notizen: u.body,
            });
        Ok(Lage { version, neu })
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = app;
        Err("Not available on this platform.".into())
    }
}

/// Herunterladen, Signatur pruefen, installieren, neu starten.
#[tauri::command]
pub async fn aktualisierung_installieren(app: tauri::AppHandle) -> Result<(), String> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        use tauri_plugin_updater::UpdaterExt;
        let Some(u) = app
            .updater()
            .map_err(|e| e.to_string())?
            .check()
            .await
            .map_err(|e| e.to_string())?
        else {
            return Err("No update available.".into());
        };
        u.download_and_install(|_, _| {}, || {})
            .await
            .map_err(|e| e.to_string())?;
        app.restart();
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = app;
        Err("Not available on this platform.".into())
    }
}

/// Beim Start, leise und spaet: Gibt es eine neue Fassung, sagt die Seite es
/// einmal an (`openany-aktualisierung`). Kein Netz, kein Release -- nichts.
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub(crate) fn beim_start_fragen(app: tauri::AppHandle) {
    use tauri::Manager;
    use tauri_plugin_updater::UpdaterExt;
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(15)).await;
        let Ok(updater) = app.updater() else { return };
        let Ok(Some(u)) = updater.check().await else {
            return;
        };
        if let Some(fenster) = app.get_webview_window("main") {
            let version = serde_json::to_string(&u.version).unwrap_or_default();
            let _ = fenster.eval(format!(
                "window.dispatchEvent(new CustomEvent('openany-aktualisierung', {{ detail: {version} }}))"
            ));
        }
    });
}
