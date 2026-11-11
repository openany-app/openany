//! Ziehen und Ablegen am Schreibtisch (docs/plan-viele-dateien.md, D1).
//!
//! Das System meldet abgelegte Dateien und Ordner mit echten Pfaden ans
//! Fenster (`WindowEvent::DragDrop`). **Die Pfade bleiben hier.** Das Fenster
//! erfährt nur, *wo* gezogen und abgelegt wird (Ereignis `openany-ziehen` mit
//! `x`, `y` in CSS-Pixeln), sucht die Ablagefläche an dieser Stelle und nennt
//! das Ziel (`uebernahme_ablegen`). Ein Befehl, der Pfade aus dem Webview
//! annähme, wäre eine Tür für jede fremde Zeile darin.
//!
//! Ein abgelegter Ordner kommt samt Unterordnern: in Dateien als Ordnerbaum,
//! in der Mediathek als Album mit dem Ordnernamen (Entscheidung 3), Unterordner
//! als Unteralben -- so tief, wie Alben gehen dürfen; was tiefer liegt, kommt
//! ins tiefste. Gibt es am Ziel schon einen Ordner oder ein Album mit dem
//! Namen, kommt alles dort hinein: Zweimal ablegen legt nichts doppelt an.

use crate::Zustand;
use openany_store::{Album, Datei, Eingereiht, NeueUebernahme, Protokoll, Speicher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::Manager;

/// So tief dürfen Alben liegen (wie `MAX_DEPTH` in `AlbumAnsicht.vue`).
const ALBENTIEFE: usize = 7;

/// Was zuletzt abgelegt wurde -- bis das Fenster sagt, wohin.
pub type Abgelegt = std::sync::Mutex<Option<Vec<PathBuf>>>;

fn fehler(e: impl ToString) -> String {
    e.to_string()
}

/// Ein Ereignis des Fensters beim Ziehen. Unter Android zieht niemand Dateien
/// ins Fenster.
#[cfg_attr(any(target_os = "android", target_os = "ios"), allow(dead_code))]
pub(crate) fn beim_ziehen(fenster: &tauri::Window, ereignis: &tauri::DragDropEvent) {
    let massstab = fenster.scale_factor().unwrap_or(1.0);
    let stelle = |p: &tauri::PhysicalPosition<f64>| (p.x / massstab, p.y / massstab);
    let (art, ort) = match ereignis {
        tauri::DragDropEvent::Enter { position, .. } | tauri::DragDropEvent::Over { position } => {
            ("ueber", Some(stelle(position)))
        }
        tauri::DragDropEvent::Drop { paths, position } => {
            if let Some(z) = fenster.app_handle().try_state::<Arc<Zustand>>() {
                *z.abgelegt.lock().unwrap_or_else(|e| e.into_inner()) = Some(paths.clone());
            }
            ("ablegen", Some(stelle(position)))
        }
        tauri::DragDropEvent::Leave => ("weg", None),
        _ => return,
    };
    let (x, y) = ort.unwrap_or_default();
    if let Some(w) = fenster.app_handle().get_webview_window("main") {
        let _ = w.eval(format!(
            "window.dispatchEvent(new CustomEvent('openany-ziehen', {{ detail: {{ art: '{art}', x: {x}, y: {y} }} }}))"
        ));
    }
}

/// Eine Datei samt den Ordnern über ihr, vom abgelegten Ordner an.
#[derive(Debug, PartialEq)]
struct Fund {
    datei: PathBuf,
    kette: Vec<String>,
}

/// Dateien unter den abgelegten Pfaden. Versteckte (`.DS_Store`, `.git`)
/// und Verknüpfungen bleiben draußen -- eine Verknüpfung auf einen
/// Elternordner liefe sonst im Kreis.
fn sammeln(pfade: &[PathBuf]) -> Vec<Fund> {
    fn gang(pfad: &Path, kette: &mut Vec<String>, funde: &mut Vec<Fund>) {
        let Some(name) = pfad.file_name().map(|n| n.to_string_lossy().to_string()) else {
            return;
        };
        if name.starts_with('.') {
            return;
        }
        let Ok(meta) = std::fs::symlink_metadata(pfad) else {
            return;
        };
        if meta.is_file() {
            funde.push(Fund {
                datei: pfad.to_path_buf(),
                kette: kette.clone(),
            });
        } else if meta.is_dir() {
            let Ok(eintraege) = std::fs::read_dir(pfad) else {
                return;
            };
            let mut kinder: Vec<PathBuf> = eintraege.flatten().map(|e| e.path()).collect();
            kinder.sort();
            kette.push(name);
            for k in kinder {
                gang(&k, kette, funde);
            }
            kette.pop();
        }
    }
    let mut funde = Vec::new();
    for p in pfade {
        gang(p, &mut Vec::new(), &mut funde);
    }
    funde
}

/// Den Ordner (Dateien) bzw. das Album (Mediathek) mit diesem Namen unter
/// `eltern` -- den vorhandenen oder einen neuen. Gibt `(uuid, neu)`.
pub(crate) fn behaelter(
    speicher: &Speicher,
    zone: &str,
    eltern: Option<&str>,
    name: &str,
) -> Result<(String, bool), String> {
    if zone == "galerie" {
        let name = name.trim();
        if let Some(a) = speicher
            .alben_unter(eltern)
            .map_err(fehler)?
            .into_iter()
            .find(|a| a.name.to_lowercase() == name.to_lowercase())
        {
            return Ok((a.uuid, false));
        }
        let a = Album {
            uuid: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            eltern: eltern.map(str::to_string),
            ..Default::default()
        };
        speicher
            .album_schreiben(&a, Protokoll::Merken)
            .map_err(fehler)?;
        Ok((a.uuid, true))
    } else {
        let name = crate::dateibefehle::name_saeubern(name)?;
        if let Some(d) = speicher
            .dateien_im_ordner(zone, eltern)
            .map_err(fehler)?
            .into_iter()
            .find(|d| d.ist_ordner && d.name.to_lowercase() == name.to_lowercase())
        {
            return Ok((d.uuid, false));
        }
        if speicher
            .dateiname_vergeben(zone, eltern, &name, None)
            .map_err(fehler)?
        {
            // Eine Datei heißt so wie der Ordner: daneben, mit freiem Namen.
            let frei = crate::dateibefehle::freier_name(speicher, zone, eltern, &name)?;
            return neuer_ordner(speicher, zone, eltern, frei).map(|u| (u, true));
        }
        neuer_ordner(speicher, zone, eltern, name).map(|u| (u, true))
    }
}

fn neuer_ordner(
    speicher: &Speicher,
    zone: &str,
    eltern: Option<&str>,
    name: String,
) -> Result<String, String> {
    let uuid = uuid::Uuid::new_v4().to_string();
    speicher
        .datei_schreiben(
            &Datei {
                uuid: uuid.clone(),
                zone: zone.into(),
                ist_ordner: true,
                eltern: eltern.map(str::to_string),
                name,
                ..Default::default()
            },
            Protokoll::Merken,
        )
        .map_err(fehler)?;
    Ok(uuid)
}

/// Das zuletzt Abgelegte nach `zone` (`galerie` oder `files`) in `ziel`
/// übernehmen. `None`, wenn nichts (mehr) abgelegt ist.
#[tauri::command]
pub async fn uebernahme_ablegen(
    app: tauri::AppHandle,
    zustand: tauri::State<'_, Arc<Zustand>>,
    zone: String,
    ziel: Option<String>,
) -> Result<Option<Eingereiht>, String> {
    if zone != "galerie" && zone != "files" {
        return Err("This zone does not exist.".into());
    }
    let pfade = zustand
        .abgelegt
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .take();
    let Some(pfade) = pfade else {
        return Ok(None);
    };
    crate::uebernahme::ziel_pruefen(&zustand, &zone, ziel.as_deref()).await?;

    let funde = tauri::async_runtime::spawn_blocking(move || sammeln(&pfade))
        .await
        .map_err(fehler)?;
    let mut neue: Vec<(NeueUebernahme, Vec<String>)> = funde
        .into_iter()
        .filter_map(|f| crate::uebernahme::als_neue(&f.datei, &zone, None).map(|n| (n, f.kette)))
        .collect();
    // Erst prüfen, dann Ordner anlegen: Reicht der Platz nicht, bleiben
    // keine leeren Ordner zurück.
    let nur: Vec<NeueUebernahme> = neue.iter().map(|(n, _)| n.clone()).collect();
    crate::uebernahme::platz_pruefen(&zustand, &nur)?;

    let mut neu_angelegt: Vec<(String, Option<String>)> = Vec::new();
    {
        let speicher = zustand.speicher.lock().await;
        let tiefe_am_ziel = match (&*zone, ziel.as_deref()) {
            ("galerie", Some(a)) => speicher.albumweg(a).map_err(fehler)?.len() + 1,
            _ => 0,
        };
        let hoechstens = if zone == "galerie" {
            ALBENTIEFE.saturating_sub(tiefe_am_ziel)
        } else {
            usize::MAX
        };
        let mut schon: HashMap<Vec<String>, Option<String>> = HashMap::new();
        schon.insert(Vec::new(), ziel.clone());
        for (n, kette) in &mut neue {
            kette.truncate(hoechstens);
            for i in 1..=kette.len() {
                let teil = kette[..i].to_vec();
                if schon.contains_key(&teil) {
                    continue;
                }
                let eltern = schon[&kette[..i - 1]].clone();
                let (uuid, neu) = behaelter(&speicher, &zone, eltern.as_deref(), &kette[i - 1])?;
                if neu {
                    neu_angelegt.push((uuid.clone(), eltern));
                }
                schon.insert(teil, Some(uuid));
            }
            n.ziel = schon[kette.as_slice()].clone();
        }
    }
    // Neue Ordner und Alben sind auf diesem Gerät gewählt, wie von Hand
    // angelegte.
    for (uuid, eltern) in &neu_angelegt {
        crate::dateibefehle::neuen_behaelter_waehlen(&zustand, uuid, eltern.as_deref()).await;
    }
    let neue: Vec<NeueUebernahme> = neue.into_iter().map(|(n, _)| n).collect();
    crate::uebernahme::einreihen_neue(&app, &zustand, &neue)
        .await
        .map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ein_ordner_kommt_samt_unterordnern_ohne_versteckte() {
        let d = tempfile::tempdir().unwrap();
        let urlaub = d.path().join("Urlaub");
        std::fs::create_dir_all(urlaub.join("Tag 1")).unwrap();
        std::fs::create_dir_all(urlaub.join(".git")).unwrap();
        std::fs::write(urlaub.join("a.jpg"), b"a").unwrap();
        std::fs::write(urlaub.join(".DS_Store"), b"x").unwrap();
        std::fs::write(urlaub.join(".git/config"), b"x").unwrap();
        std::fs::write(urlaub.join("Tag 1/b.jpg"), b"b").unwrap();
        let einzeln = d.path().join("c.pdf");
        std::fs::write(&einzeln, b"c").unwrap();

        let funde = sammeln(&[urlaub.clone(), einzeln.clone()]);
        assert_eq!(
            funde,
            vec![
                Fund {
                    datei: urlaub.join("Tag 1/b.jpg"),
                    kette: vec!["Urlaub".into(), "Tag 1".into()]
                },
                Fund {
                    datei: urlaub.join("a.jpg"),
                    kette: vec!["Urlaub".into()]
                },
                Fund {
                    datei: einzeln,
                    kette: vec![]
                },
            ]
        );
    }

    #[test]
    fn zweimal_ablegen_nimmt_denselben_ordner_und_dasselbe_album() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        let (a, neu) = behaelter(&s, "galerie", None, "Urlaub").unwrap();
        assert!(neu);
        assert_eq!(
            behaelter(&s, "galerie", None, "urlaub").unwrap(),
            (a, false)
        );

        let (o, neu) = behaelter(&s, "files", None, "Belege").unwrap();
        assert!(neu);
        assert_eq!(
            behaelter(&s, "files", None, "Belege").unwrap(),
            (o.clone(), false)
        );
        let (u, _) = behaelter(&s, "files", Some(&o), "2026").unwrap();
        assert_ne!(u, o);
    }
}
