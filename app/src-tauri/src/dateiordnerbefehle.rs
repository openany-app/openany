//! Einen Ordner auf diesem Computer abgleichen, wie Nextcloud
//! (docs/plan-viele-dateien.md, D4). Nur auf dem Schreibtisch; unter Android
//! lehnen die Befehle ab.
//!
//! Die Regeln stehen im Speicher (`openany_store::dateiordner`). Hier: die
//! Befehle und der Dienst, der abgleicht, solange das Programm läuft --
//! geweckt vom Dateiwächter und ohnehin alle paar Sekunden, damit
//! Änderungen aus der App und aus dem Abgleich mit openany.de hinausgehen.
//! Zu openany.de trägt es danach der Abgleich, wenn er für den Bereich an ist.
//!
//! **Den Ordner wählt der Mensch im Dialog des Systems**, in Rust -- wie bei
//! den Notiz-Ordnern (`notizordnerbefehle.rs`).

use crate::Zustand;
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct Paarlage {
    id: i64,
    zone: String,
    /// Der Ordner in openany; `None` = oberste Ebene der Zone.
    wurzel: Option<String>,
    /// Der Ordner auf diesem Computer.
    ordner: String,
    /// Ist er gerade nicht da (Stick ab, umbenannt)?
    fehlt: bool,
}

fn nicht_hier() -> String {
    "Only available in the desktop app.".into()
}

fn fehler(e: impl ToString) -> String {
    e.to_string()
}

/// Was auf diesem Computer verbunden ist.
#[tauri::command]
pub async fn dateiordner_liste(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Vec<Paarlage>, String> {
    if cfg!(any(target_os = "android", target_os = "ios")) {
        return Ok(Vec::new());
    }
    let speicher = zustand.speicher.lock().await;
    Ok(speicher
        .dateiordner()
        .map_err(fehler)?
        .into_iter()
        .map(|p| Paarlage {
            id: p.id,
            fehlt: !p.ordner.is_dir(),
            ordner: p.ordner.to_string_lossy().to_string(),
            zone: p.zone,
            wurzel: p.wurzel,
        })
        .collect())
}

/// Ordner wählen und mit `zone` ab `wurzel` verbinden. `None`: im Dialog
/// abgebrochen.
#[tauri::command]
pub async fn dateiordner_verbinden(
    app: tauri::AppHandle,
    zustand: tauri::State<'_, Arc<Zustand>>,
    zone: String,
    wurzel: Option<String>,
) -> Result<Option<String>, String> {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = (app, zustand, zone, wurzel);
        Err(nicht_hier())
    }
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        use tauri_plugin_dialog::DialogExt;
        if zone != "documents" && zone != "files" {
            return Err("This zone does not exist.".into());
        }
        {
            let speicher = zustand.speicher.lock().await;
            crate::dateibefehle::ordner_holen(&speicher, &zone, wurzel.as_deref())?;
        }
        let gewaehlt = app
            .dialog()
            .file()
            .blocking_pick_folder()
            .and_then(|p| p.into_path().ok());
        let Some(ordner) = gewaehlt else {
            return Ok(None);
        };
        // Der App-Ordner selbst (und was darüber liegt) nie: Der Spiegel
        // läse seine eigenen Inhalte ein.
        if zustand.ordner.starts_with(&ordner) || ordner.starts_with(&zustand.ordner) {
            return Err("This folder cannot be connected.".into());
        }
        let verbunden = zustand
            .speicher
            .lock()
            .await
            .dateiordner_verbinden(&zone, wurzel.as_deref(), &ordner)
            .map_err(fehler)?;
        if verbunden.is_none() {
            return Err("This area, or a folder above or below, is already connected.".into());
        }
        schreibtisch::wecken();
        Ok(Some(ordner.to_string_lossy().to_string()))
    }
}

/// Trennen. Die Dateien bleiben -- im Ordner und in openany.
#[tauri::command]
pub async fn dateiordner_trennen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: i64,
) -> Result<(), String> {
    if cfg!(any(target_os = "android", target_os = "ios")) {
        return Err(nicht_hier());
    }
    zustand
        .speicher
        .lock()
        .await
        .dateiordner_trennen(id)
        .map_err(fehler)?;
    schreibtisch::wecken();
    Ok(())
}

/// Den Ordner im Dateimanager zeigen.
#[tauri::command]
pub async fn dateiordner_zeigen(
    app: tauri::AppHandle,
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: i64,
) -> Result<(), String> {
    if cfg!(any(target_os = "android", target_os = "ios")) {
        let _ = app;
        return Err(nicht_hier());
    }
    let ordner = zustand
        .speicher
        .lock()
        .await
        .dateiordner()
        .map_err(fehler)?
        .into_iter()
        .find(|p| p.id == id)
        .map(|p| p.ordner);
    if let Some(o) = ordner {
        use tauri_plugin_opener::OpenerExt;
        app.opener()
            .open_path(o.to_string_lossy(), None::<&str>)
            .map_err(fehler)?;
    }
    Ok(())
}

#[cfg(any(target_os = "android", target_os = "ios"))]
pub(crate) mod schreibtisch {
    pub(crate) fn wecken() {}
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub(crate) mod schreibtisch {
    use crate::Zustand;
    use notify::{RecursiveMode, Watcher};
    use openany_store::{Ordnerpaar, SpeicherFehler};
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, OnceLock};
    use std::time::Duration;
    use tauri::Manager;
    use tokio::sync::Notify;

    /// Ohne Weckruf: so oft, damit Änderungen aus der App hinausgehen.
    const TAKT: Duration = Duration::from_secs(8);
    /// Nach einem Weckruf kurz warten -- ein Programm speichert in mehreren Zügen.
    const RUHE: Duration = Duration::from_secs(1);
    /// Was jünger ist, schreibt womöglich gerade jemand.
    const FRISCH: Duration = Duration::from_secs(2);

    fn wecker() -> &'static Notify {
        static W: OnceLock<Notify> = OnceLock::new();
        W.get_or_init(Notify::new)
    }

    pub(crate) fn wecken() {
        wecker().notify_one();
    }

    fn wegwerfen(p: &Path) -> std::io::Result<()> {
        trash::delete(p).map_err(std::io::Error::other)
    }

    fn mime(name: &str) -> String {
        mime_guess::from_path(name)
            .first()
            .map(|m| m.essence_str().to_string())
            .unwrap_or_default()
    }

    /// Ein Paar einmal abgleichen. `true`, wenn sich etwas getan hat.
    async fn abgleichen(z: &Arc<Zustand>, paar: &Ordnerpaar) -> bool {
        let stand = match z.speicher.lock().await.dateiordner_stand(paar.id) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Folder {}: {e}", paar.ordner.display());
                return false;
            }
        };
        // Lesen und Einlesen ohne Sperre: ein großes Video braucht.
        let (p, inhalte) = (paar.clone(), z.inhalte.clone());
        let scan = tauri::async_runtime::spawn_blocking(move || {
            openany_store::scannen(&p, &stand, &inhalte, FRISCH)
        })
        .await;
        let scan = match scan {
            Ok(Ok(s)) => s,
            // Ein fehlender Ordner ist kein Fehler fürs Protokoll bei jedem
            // Takt -- die Oberfläche zeigt ihn als fehlend.
            Ok(Err(SpeicherFehler::OrdnerFehlt(_))) => return false,
            Ok(Err(e)) => {
                eprintln!("Folder {}: {e}", paar.ordner.display());
                return false;
            }
            Err(e) => {
                eprintln!("Folder {}: {e}", paar.ordner.display());
                return false;
            }
        };
        let geraet = z.einstellungen.lock().await.name();
        let bericht = {
            let speicher = z.speicher.lock().await;
            speicher.dateiordner_abgleichen(paar, &scan, &z.inhalte, &wegwerfen, &mime, &geraet)
        };
        let bericht = match bericht {
            Ok(b) => b,
            Err(SpeicherFehler::OrdnerFehlt(_)) => return false,
            Err(e) => {
                eprintln!("Folder {}: {e}", paar.ordner.display());
                return false;
            }
        };
        for (uuid, eltern) in &bericht.neue_ordner {
            crate::dateibefehle::neuen_behaelter_waehlen(z, uuid, eltern.as_deref()).await;
        }
        // Zuerst alles herein (Entscheidung 12): Was hier noch fehlt, holen;
        // der nächste Durchlauf schreibt es dann in den Ordner.
        for (abdruck, groesse) in &bericht.fehlt {
            if let Err(e) = crate::dateibefehle::von_geraeten_holen(z, abdruck, *groesse).await {
                eprintln!("Folder {}: {e}", paar.ordner.display());
                break;
            }
        }
        bericht.etwas_getan() || !bericht.fehlt.is_empty()
    }

    /// Beim Start: der Dienst läuft, solange das Programm läuft. Ohne
    /// verbundenen Ordner tut er bei jedem Takt nichts als eine Abfrage.
    pub(crate) fn starten(app: &tauri::AppHandle, zustand: &Arc<Zustand>) {
        let (app, z) = (app.clone(), zustand.clone());
        tauri::async_runtime::spawn(async move {
            let mut waechter: Option<notify::RecommendedWatcher> = None;
            let mut beobachtet: Vec<PathBuf> = Vec::new();
            loop {
                tokio::select! {
                    _ = tokio::time::sleep(TAKT) => {}
                    _ = wecker().notified() => tokio::time::sleep(RUHE).await,
                }
                let Ok(paare) = z.speicher.lock().await.dateiordner() else {
                    continue;
                };
                let ordner: Vec<PathBuf> = paare.iter().map(|p| p.ordner.clone()).collect();
                if ordner != beobachtet {
                    waechter = neuer_waechter(&ordner);
                    beobachtet = ordner;
                }
                let _ = &waechter;

                let mut getan = false;
                for p in &paare {
                    getan |= abgleichen(&z, p).await;
                }
                if getan {
                    if let Some(f) = app.get_webview_window("main") {
                        let _ = f.eval("window.dispatchEvent(new Event('openany-aufgefrischt'))");
                    }
                }
            }
        });
    }

    fn neuer_waechter(ordner: &[PathBuf]) -> Option<notify::RecommendedWatcher> {
        if ordner.is_empty() {
            return None;
        }
        let mut w = notify::recommended_watcher(|e: notify::Result<notify::Event>| {
            // Nur echte Änderungen; Lesen weckt nicht. Die eigenen
            // Zwischendateien auch nicht, sonst weckte jeder Durchlauf den
            // nächsten.
            if e.is_ok_and(|e| {
                !matches!(e.kind, notify::EventKind::Access(_))
                    && !e
                        .paths
                        .iter()
                        .all(|p| p.to_string_lossy().ends_with(".openany-tmp"))
            }) {
                super::schreibtisch::wecken();
            }
        })
        .ok()?;
        for o in ordner {
            let _ = w.watch(o, RecursiveMode::Recursive);
        }
        Some(w)
    }
}
