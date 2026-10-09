//! Auffrischen und Wachdienst auf dem Schreibtisch (docs/plan-desktop.md, 1c).
//!
//! Unter Android plant das System beides (`Auffrischer.kt`, `Wachdienst.kt`)
//! und ruft ueber JNI herein. Auf dem Schreibtisch gibt es keinen solchen
//! Planer: Beides laeuft, **solange das Programm laeuft** -- der Wachdienst in
//! einem eigenen Faden ([`crate::wachdienst::laufen`] blockiert), das
//! Auffrischen als Takt etwa stuendlich. Mitteilungen gehen ueber
//! `tauri-plugin-notification` an das System.
//!
//! Die Schalter liegen in `einstellungen.json` (`auffrischen`, `sofort`);
//! die Oberflaeche fragt [`hintergrund_lage`] und setzt mit
//! [`hintergrund_setzen`]. Unter Android lehnen beide ab -- dort gelten die
//! Bruecken der Schale.

use crate::Zustand;
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct Lage {
    auffrischen: bool,
    sofort: bool,
}

#[tauri::command]
pub async fn hintergrund_lage(zustand: tauri::State<'_, Arc<Zustand>>) -> Result<Lage, String> {
    if cfg!(any(target_os = "android", target_os = "ios")) {
        return Err("Not available on this platform.".into());
    }
    let e = zustand.einstellungen.lock().await;
    Ok(Lage {
        auffrischen: e.auffrischen,
        sofort: e.sofort,
    })
}

#[tauri::command]
pub async fn hintergrund_setzen(
    app: tauri::AppHandle,
    zustand: tauri::State<'_, Arc<Zustand>>,
    auffrischen: Option<bool>,
    sofort: Option<bool>,
) -> Result<Lage, String> {
    if cfg!(any(target_os = "android", target_os = "ios")) {
        let _ = (app, auffrischen, sofort);
        return Err("Not available on this platform.".into());
    }
    let neu = {
        let mut e = zustand.einstellungen.lock().await;
        if let Some(a) = auffrischen {
            e.auffrischen = a;
        }
        if let Some(s) = sofort {
            e.sofort = s;
        }
        e.schreiben(&zustand.einstellungspfad())
            .map_err(|e| e.to_string())?;
        e.clone()
    };
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    if let Some(s) = sofort {
        if s {
            schreibtisch::wachdienst_starten(&app, &zustand);
        } else {
            crate::wachdienst::anhalten();
        }
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    let _ = app;
    Ok(Lage {
        auffrischen: neu.auffrischen,
        sofort: neu.sofort,
    })
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub(crate) mod schreibtisch {
    use crate::wachdienst::Anzeige;
    use crate::Zustand;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use tauri::Manager;

    /// Etwa stuendlich, wie der Auffrischer unter Android.
    const TAKT: Duration = Duration::from_secs(60 * 60);

    static WACHDIENST_LAEUFT: AtomicBool = AtomicBool::new(false);

    /// Beim Start: den Takt anwerfen und, falls gewuenscht, den Wachdienst.
    pub(crate) fn starten(app: &tauri::AppHandle, zustand: &Arc<Zustand>) {
        let (app2, z) = (app.clone(), zustand.clone());
        tauri::async_runtime::spawn(async move {
            loop {
                tokio::time::sleep(TAKT).await;
                if !z.einstellungen.lock().await.auffrischen {
                    continue;
                }
                let bericht = crate::hintergrund::auffrischen(&z, false).await;
                eprintln!("Refresher: {bericht}");
                aufgefrischt(&app2);
            }
        });

        let sofort =
            tauri::async_runtime::block_on(async { zustand.einstellungen.lock().await.sofort });
        if sofort {
            wachdienst_starten(app, zustand);
        }
    }

    /// Hoechstens einmal: Ein zweiter Wachdienst haette zwei Leitungen zu
    /// ntfy und zeigte jede Nachricht doppelt.
    pub(crate) fn wachdienst_starten(app: &tauri::AppHandle, zustand: &Zustand) {
        if WACHDIENST_LAEUFT.swap(true, Ordering::SeqCst) {
            return;
        }
        let app = app.clone();
        let ordner = zustand.ordner.clone();
        let zwischen = zustand.zwischenspeicher.clone();
        std::thread::spawn(move || {
            let grund = crate::wachdienst::laufen(ordner, zwischen, &SchreibtischAnzeige { app });
            eprintln!("Watchdog ended: {grund}");
            WACHDIENST_LAEUFT.store(false, Ordering::SeqCst);
        });
    }

    fn aufgefrischt(app: &tauri::AppHandle) {
        if let Some(fenster) = app.get_webview_window("main") {
            let _ = fenster.eval("window.dispatchEvent(new Event('openany-aufgefrischt'))");
        }
    }

    struct SchreibtischAnzeige {
        app: tauri::AppHandle,
    }

    impl Anzeige for SchreibtischAnzeige {
        fn zeige(&self, titel: &str, text: &str) {
            use tauri_plugin_notification::NotificationExt;
            let _ = self
                .app
                .notification()
                .builder()
                .title(titel)
                .body(text)
                .show();
        }

        fn aufgefrischt(&self) {
            aufgefrischt(&self.app);
        }

        fn protokoll(&self, text: &str) {
            eprintln!("Watchdog: {text}");
        }
    }
}
