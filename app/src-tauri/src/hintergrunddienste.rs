//! Auffrischen und Wachdienst auf dem Schreibtisch (docs/plan-desktop.md, 1c).
//!
//! Unter Android plant das System beides (`Auffrischer.kt`, `Wachdienst.kt`)
//! und ruft ueber JNI herein. Auf dem Schreibtisch gibt es keinen solchen
//! Planer: Beides laeuft, **solange das Programm laeuft** -- der Wachdienst in
//! einem eigenen Faden ([`crate::wachdienst::laufen`] blockiert), das
//! Auffrischen als Takt etwa stuendlich. Mitteilungen gehen ueber
//! `tauri-plugin-notification` an das System.
//!
//! Damit das auch ohne offenes Fenster gilt, kann das Programm beim Anmelden
//! starten und im Menue-Symbol weiterlaufen (`weiterlaufen`).
//!
//! Die Schalter liegen in `einstellungen.json` (`auffrischen`, `sofort`,
//! `weiterlaufen`);
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
    weiterlaufen: bool,
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
        weiterlaufen: e.weiterlaufen,
    })
}

#[tauri::command]
pub async fn hintergrund_setzen(
    app: tauri::AppHandle,
    zustand: tauri::State<'_, Arc<Zustand>>,
    auffrischen: Option<bool>,
    sofort: Option<bool>,
    weiterlaufen: Option<bool>,
) -> Result<Lage, String> {
    if cfg!(any(target_os = "android", target_os = "ios")) {
        let _ = (app, auffrischen, sofort, weiterlaufen);
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
        if let Some(w) = weiterlaufen {
            e.weiterlaufen = w;
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
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    if let Some(w) = weiterlaufen {
        schreibtisch::weiterlaufen_setzen(&app, w);
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    let _ = app;
    Ok(Lage {
        auffrischen: neu.auffrischen,
        sofort: neu.sofort,
        weiterlaufen: neu.weiterlaufen,
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
    /// Der Fenster-Ereignis-Haken liest das ohne Warten auf eine Sperre.
    static WEITERLAUFEN: AtomicBool = AtomicBool::new(false);

    const SYMBOL: &str = "openany";
    /// Daran erkennt der Start, dass ihn die Anmeldung ausgeloest hat.
    pub(crate) const STILLER_START: &str = "--im-hintergrund";

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

        let (sofort, weiterlaufen) = tauri::async_runtime::block_on(async {
            let e = zustand.einstellungen.lock().await;
            (e.sofort, e.weiterlaufen)
        });
        if sofort {
            wachdienst_starten(app, zustand);
        }
        weiterlaufen_setzen(app, weiterlaufen);
        if weiterlaufen && std::env::args().any(|a| a == STILLER_START) {
            if let Some(fenster) = app.get_webview_window("main") {
                let _ = fenster.hide();
            }
        }
    }

    /// Symbol und Anmeldestart gehen zusammen an und aus: Ohne Symbol käme man
    /// an ein verstecktes Fenster nicht mehr heran.
    pub(crate) fn weiterlaufen_setzen(app: &tauri::AppHandle, an: bool) {
        use tauri_plugin_autostart::ManagerExt;
        WEITERLAUFEN.store(an, Ordering::SeqCst);
        let autostart = app.autolaunch();
        let ergebnis = if an {
            autostart.enable()
        } else {
            autostart.disable()
        };
        if let Err(e) = ergebnis {
            eprintln!("Autostart: {e}");
        }
        if an {
            if app.tray_by_id(SYMBOL).is_none() {
                if let Err(e) = symbol_anlegen(app) {
                    eprintln!("Tray icon: {e}");
                }
            }
        } else {
            let _ = app.remove_tray_by_id(SYMBOL);
        }
    }

    fn symbol_anlegen(app: &tauri::AppHandle) -> tauri::Result<()> {
        use tauri::menu::{Menu, MenuItem};
        use tauri::tray::TrayIconBuilder;
        let (oeffnen, beenden) = symbol_texte();
        let oeffnen = MenuItem::with_id(app, "oeffnen", oeffnen, true, None::<&str>)?;
        let beenden = MenuItem::with_id(app, "beenden", beenden, true, None::<&str>)?;
        let menue = Menu::with_items(app, &[&oeffnen, &beenden])?;
        let mut bau = TrayIconBuilder::with_id(SYMBOL)
            .tooltip("openany")
            .menu(&menue)
            .on_menu_event(|app, ereignis| match ereignis.id().as_ref() {
                "oeffnen" => fenster_zeigen(app),
                "beenden" => app.exit(0),
                _ => {}
            });
        if let Some(bild) = app.default_window_icon() {
            bau = bau.icon(bild.clone());
        }
        bau.build(app)?;
        Ok(())
    }

    /// Das Menue entsteht vor der Oberflaeche und kennt deren Sprache nicht,
    /// deshalb gilt hier die des Systems.
    fn symbol_texte() -> (&'static str, &'static str) {
        let sprache = sys_locale::get_locale().unwrap_or_default();
        match sprache.get(..2).unwrap_or("") {
            "de" => ("Öffnen", "Beenden"),
            "es" => ("Abrir", "Salir"),
            "fr" => ("Ouvrir", "Quitter"),
            "pl" => ("Otwórz", "Zakończ"),
            "pt" => ("Abrir", "Sair"),
            "uk" => ("Відкрити", "Вийти"),
            _ => ("Open", "Quit"),
        }
    }

    pub(crate) fn fenster_zeigen(app: &tauri::AppHandle) {
        if let Some(fenster) = app.get_webview_window("main") {
            let _ = fenster.show();
            let _ = fenster.unminimize();
            let _ = fenster.set_focus();
        }
    }

    /// Fenster schliessen heisst, wenn gewuenscht, nur verstecken.
    pub(crate) fn beim_schliessen(fenster: &tauri::Window, api: &tauri::CloseRequestApi) {
        if WEITERLAUFEN.load(Ordering::SeqCst) {
            api.prevent_close();
            let _ = fenster.hide();
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
