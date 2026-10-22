//! Eine Mappe mit einem Ordner verbinden -- fuer Zettlr, Obsidian & Co.
//! (docs/plan-notizen-e2e.md, 6d). Nur auf dem Schreibtisch; unter Android
//! lehnen die Befehle ab.
//!
//! Die Regeln des Spiegels stehen im Speicher (`openany_store::notizordner`).
//! Hier: die Befehle fuers Menue der Mappe und der Dienst, der spiegelt,
//! solange das Programm laeuft -- geweckt vom Dateiwaechter, sobald sich im
//! Ordner etwas tut, und ohnehin alle paar Sekunden, damit Aenderungen aus
//! der App (oder aus dem Abgleich) hinausgehen.
//!
//! **Den Ordner waehlt der Mensch im Dialog des Systems**, in Rust -- wie bei
//! `ablagebefehle.rs`. Ein Befehl „spiegle nach Pfad X" waere eine Tuer fuer
//! jede fremde Zeile im Webview.

use crate::Zustand;
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct Ordnerlage {
    /// Mappen-Id wie in `notizbuch_mappen`.
    mappe: u32,
    /// Der Ordner auf diesem Computer.
    ordner: String,
    /// Ist er gerade nicht da (ausgehaengt, umbenannt)?
    fehlt: bool,
}

fn nicht_hier() -> String {
    "Only available in the desktop app.".into()
}

/// Was auf diesem Computer verbunden ist -- das Menue der Mappe fragt danach.
#[tauri::command]
pub async fn notizordner_liste(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Vec<Ordnerlage>, String> {
    if cfg!(any(target_os = "android", target_os = "ios")) {
        return Ok(Vec::new());
    }
    let speicher = zustand.speicher.lock().await;
    Ok(speicher
        .notizordner()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|v| Ordnerlage {
            mappe: openany_store::mappen_id(&v.mappe),
            fehlt: !v.ordner.is_dir(),
            ordner: v.ordner.to_string_lossy().to_string(),
        })
        .collect())
}

/// Ordner waehlen und verbinden. `None`: im Dialog abgebrochen.
#[tauri::command]
pub async fn notizordner_verbinden(
    app: tauri::AppHandle,
    zustand: tauri::State<'_, Arc<Zustand>>,
    mappe: u32,
) -> Result<Option<String>, String> {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = (app, zustand, mappe);
        Err(nicht_hier())
    }
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        use tauri_plugin_dialog::DialogExt;
        let pfad = {
            let speicher = zustand.speicher.lock().await;
            speicher
                .mappen_pfad(mappe)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "This folder no longer exists.".to_string())?
        };
        let gewaehlt = app
            .dialog()
            .file()
            .set_title(&pfad)
            .blocking_pick_folder()
            .and_then(|p| p.into_path().ok());
        let Some(ordner) = gewaehlt else {
            return Ok(None);
        };
        {
            let speicher = zustand.speicher.lock().await;
            if !speicher
                .notizordner_verbinden(&pfad, &ordner)
                .map_err(|e| e.to_string())?
            {
                return Err("A folder above or below is already connected.".into());
            }
        }
        schreibtisch::wecken();
        Ok(Some(ordner.to_string_lossy().to_string()))
    }
}

/// Trennen. Die Dateien bleiben liegen.
#[tauri::command]
pub async fn notizordner_trennen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    mappe: u32,
) -> Result<(), String> {
    if cfg!(any(target_os = "android", target_os = "ios")) {
        return Err(nicht_hier());
    }
    let speicher = zustand.speicher.lock().await;
    let Some(pfad) = speicher.mappen_pfad(mappe).map_err(|e| e.to_string())? else {
        return Ok(());
    };
    speicher
        .notizordner_trennen(&pfad)
        .map_err(|e| e.to_string())
}

/// Den Ordner im Dateimanager zeigen.
#[tauri::command]
pub async fn notizordner_zeigen(
    app: tauri::AppHandle,
    zustand: tauri::State<'_, Arc<Zustand>>,
    mappe: u32,
) -> Result<(), String> {
    if cfg!(any(target_os = "android", target_os = "ios")) {
        let _ = app;
        return Err(nicht_hier());
    }
    let ordner = {
        let speicher = zustand.speicher.lock().await;
        let Some(pfad) = speicher.mappen_pfad(mappe).map_err(|e| e.to_string())? else {
            return Ok(());
        };
        speicher
            .notizordner_von(&pfad)
            .map_err(|e| e.to_string())?
            .map(|v| v.ordner)
    };
    if let Some(o) = ordner {
        use tauri_plugin_opener::OpenerExt;
        app.opener()
            .open_path(o.to_string_lossy(), None::<&str>)
            .map_err(|e| e.to_string())?;
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
    use std::path::PathBuf;
    use std::sync::{Arc, OnceLock};
    use std::time::Duration;
    use tauri::Manager;
    use tokio::sync::Notify;

    /// Ohne Weckruf: so oft, damit Aenderungen aus der App hinausgehen.
    const TAKT: Duration = Duration::from_secs(4);
    /// Nach einem Weckruf kurz warten -- ein Editor speichert in mehreren Zuegen.
    const RUHE: Duration = Duration::from_millis(400);

    fn wecker() -> &'static Notify {
        static W: OnceLock<Notify> = OnceLock::new();
        W.get_or_init(Notify::new)
    }

    pub(crate) fn wecken() {
        wecker().notify_one();
    }

    /// Beim Start: der Dienst laeuft, solange das Programm laeuft. Ohne
    /// verbundene Mappe tut er bei jedem Takt nichts als eine Abfrage.
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
                let speicher = z.speicher.lock().await;
                let Ok(verbindungen) = speicher.notizordner() else {
                    continue;
                };

                // Den Waechter nachfuehren, wenn sich die Ordner geaendert haben.
                let ordner: Vec<PathBuf> = verbindungen.iter().map(|v| v.ordner.clone()).collect();
                if ordner != beobachtet {
                    waechter = neuer_waechter(&ordner);
                    beobachtet = ordner;
                }
                let _ = &waechter;

                let mut getan = false;
                let mut bilder = Vec::new();
                for v in &verbindungen {
                    match speicher.notizordner_spiegeln(v) {
                        Ok(b) => getan |= b.etwas_getan(),
                        // Ein fehlender Ordner ist kein Fehler fuers Protokoll
                        // bei jedem Takt -- das Menue zeigt ihn als fehlend.
                        Err(openany_store::SpeicherFehler::OrdnerFehlt(_)) => continue,
                        Err(e) => {
                            eprintln!("Note folder {}: {e}", v.ordner.display());
                            continue;
                        }
                    }
                    // Die Bilder daneben (7c-3), nach den Notizen.
                    match speicher.notizordner_bilder(v, &z.inhalte) {
                        Ok(l) => {
                            getan |= l.geschrieben > 0;
                            if !l.neu.is_empty() || !l.geaendert.is_empty() {
                                bilder.push((v.clone(), l));
                            }
                        }
                        Err(e) => eprintln!("Note folder images {}: {e}", v.ordner.display()),
                    }
                }
                drop(speicher);
                // Neue und geaenderte Bilder brauchen Vorschau und Galerie --
                // ohne den Speicher zu sperren, solange gerechnet wird.
                for (v, l) in bilder {
                    getan |= crate::anhangbefehle::aus_dem_notizordner(&z, &v, l).await;
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
            // Nur echte Aenderungen; Lesen (Zugriffe) weckt nicht.
            if e.is_ok_and(|e| !matches!(e.kind, notify::EventKind::Access(_))) {
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
