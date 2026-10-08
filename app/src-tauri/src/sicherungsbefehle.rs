//! Die Sicherungsdatei (08.10.2026) -- der Weg von der Kachel zur Crate
//! `openany-sicherung` und zurueck.
//!
//! ## Was mitkommt und was nicht
//!
//! **Mit:** die Datenbank (Notizen, Kalender, Kontakte, Dateien, Galerie,
//! Projekte, Nachrichten, Mails, PGP-Schluessel der Kontakte), die Inhalte,
//! die hier liegen, die Listen der Nachrichten vor Ort und das E-Mail-
//! Postfach samt Passwort und eigenem PGP-Schluessel (Tiffy: „samt
//! Passwort").
//!
//! **Nicht mit:** was dieses Geraet ausweist (anyid, openany-Schluessel,
//! Matrix-Sitzung, die Identitaet fuer Geraete in der Naehe), die gepaarten
//! Geraete und die Einstellungen dieses Geraets -- sie „reisen nicht mit"
//! (einstellungen.rs).
//!
//! ## Einspielen heisst ersetzen
//!
//! Fassung 1 fuehrt nicht zusammen. Geoeffnet wird in einen Ordner neben
//! den Daten; erst nach der Rueckfrage wird er als bereit markiert, und beim
//! naechsten Start -- bevor die Datenbank offen ist -- tritt er an die Stelle
//! des Bisherigen ([`einspielen_falls_bereit`]). Danach ist das Geraet von
//! openany.de getrennt: Die Daten koennen von einem anderen Konto stammen.

use crate::einstellungen::Einstellungen;
use crate::{tresor, Zustand};
use anyid_client::Tokenspeicher;
use openany_sicherung::Kopf;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Wo die neue Fassung wartet, bis sie eingespielt wird.
const NEU: &str = ".sicherung-neu";
/// Steht diese Datei im Ordner, ist er bestaetigt.
const BEREIT: &str = "BEREIT";
/// Das Postfach, im Archiv unverschluesselt -- die Datei selbst ist es.
const POSTFACH: &str = "geheim/postfach.json";

/// Was aus `nah/` mitkommt. NICHT `identitaet.json` (das Geraet) und nicht
/// `gepaart.json` (die Paare dieses Geraets).
const NAH: &[&str] = &[
    "absender.json",
    "blockiert.json",
    "postausgang.json",
    "angenommen.json",
    "anfragen.json",
    "abgelehnt.json",
    "einstellungen.json",
];

fn fehler(e: impl ToString) -> String {
    e.to_string()
}

fn ablage(ordner: &Path, name: &str) -> Box<dyn Tokenspeicher + Send + Sync> {
    tresor::ablage(ordner.join("ausweise").join(name))
}

/// Der Arbeitsordner im Zwischenspeicher. Auf Android darf `MainActivity`
/// nur aus ihm weitergeben (`Ablage.sicherungAblegen`).
fn arbeitsordner(zustand: &Zustand) -> PathBuf {
    zustand.zwischenspeicher.join("sicherung")
}

#[tauri::command]
pub async fn sicherung_vorschlag() -> String {
    openany_sicherung::passphrase_vorschlagen()
}

#[derive(Serialize)]
pub struct Angelegt {
    pfad: String,
    name: String,
    groesse: u64,
}

/// Die Datei im Zwischenspeicher anlegen. Wohin sie dann geht, entscheidet
/// die Oberflaeche: auf Android die Auswahl des Systems (auch ein Stick), auf
/// dem Schreibtisch „Downloads".
#[tauri::command]
pub async fn sicherung_anlegen(
    app: tauri::AppHandle,
    zustand: tauri::State<'_, Arc<Zustand>>,
    passphrase: String,
) -> Result<Angelegt, String> {
    // Die Nummer dieser Plattform (tauri.android.conf.json usw.), nicht die
    // aus Cargo.toml -- jede Plattform zaehlt fuer sich.
    let programm = format!("openany-app {}", app.package_info().version);
    let arbeit = arbeitsordner(&zustand);
    let _ = std::fs::remove_dir_all(&arbeit);
    std::fs::create_dir_all(&arbeit).map_err(fehler)?;

    let abbild = arbeit.join("openany.sqlite");
    zustand
        .speicher
        .lock()
        .await
        .abbild_schreiben(&abbild)
        .map_err(fehler)?;

    let postfach = ablage(&zustand.ordner, "postfach")
        .lesen()
        .map_err(fehler)?;
    let geraet = zustand.einstellungen.lock().await.name();
    let ordner = zustand.ordner.clone();
    let name = format!(
        "openany-backup-{}.age",
        chrono::Local::now().format("%Y-%m-%d")
    );
    let ziel = arbeit.join(&name);

    let ergebnis = tauri::async_runtime::spawn_blocking({
        let ziel = ziel.clone();
        let abbild = abbild.clone();
        move || -> Result<u64, String> {
            let mut dateien = vec![("openany.sqlite".to_string(), abbild)];
            dateien.extend(inhalte(&ordner.join("inhalte")));
            for n in NAH {
                let p = ordner.join("nah").join(n);
                if p.is_file() {
                    dateien.push((format!("nah/{n}"), p));
                }
            }
            let p = ordner.join("postausgang-openany.json");
            if p.is_file() {
                dateien.push(("postausgang-openany.json".into(), p));
            }
            let beilagen: Vec<(String, Vec<u8>)> = postfach
                .map(|t| vec![(POSTFACH.to_string(), t.into_bytes())])
                .unwrap_or_default();

            let datei = std::io::BufWriter::new(std::fs::File::create(&ziel).map_err(fehler)?);
            let kopf = Kopf::jetzt(&programm, &geraet);
            let mut datei =
                openany_sicherung::schreiben(datei, &passphrase, &kopf, &beilagen, &dateien)
                    .map_err(fehler)?;
            std::io::Write::flush(&mut datei).map_err(fehler)?;
            Ok(std::fs::metadata(&ziel).map_err(fehler)?.len())
        }
    })
    .await
    .map_err(fehler)?;

    let _ = std::fs::remove_file(&abbild);
    let groesse = match ergebnis {
        Ok(g) => g,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&arbeit);
            return Err(e);
        }
    };

    Ok(Angelegt {
        pfad: ziel.to_string_lossy().to_string(),
        name,
        groesse,
    })
}

/// Alles unter `inhalte/`, ohne halbe Ladungen (`.laden`).
fn inhalte(wurzel: &Path) -> Vec<(String, PathBuf)> {
    let mut aus = Vec::new();
    let mut offen = vec![wurzel.to_path_buf()];
    while let Some(o) = offen.pop() {
        let Ok(liste) = std::fs::read_dir(&o) else {
            continue;
        };
        for e in liste.flatten() {
            let p = e.path();
            if e.file_name().to_string_lossy().starts_with('.') {
                continue;
            }
            let Ok(art) = e.file_type() else { continue };
            if art.is_dir() {
                offen.push(p);
            } else if art.is_file() {
                if let Ok(rest) = p.strip_prefix(wurzel) {
                    let teile: Vec<String> = rest
                        .components()
                        .map(|c| c.as_os_str().to_string_lossy().to_string())
                        .collect();
                    aus.push((format!("inhalte/{}", teile.join("/")), p));
                }
            }
        }
    }
    aus
}

/// Auf dem Schreibtisch: die fertige Datei nach „Downloads". Zurueck kommt,
/// wo sie liegt.
#[tauri::command]
pub async fn sicherung_in_downloads(
    app: tauri::AppHandle,
    zustand: tauri::State<'_, Arc<Zustand>>,
    pfad: String,
) -> Result<String, String> {
    use tauri::Manager;
    let quelle = im_arbeitsordner(&zustand, &pfad)?;
    let ordner = app.path().download_dir().map_err(fehler)?;
    let name = quelle.file_name().ok_or("No file.")?;
    let ziel = ordner.join(name);
    if std::fs::rename(&quelle, &ziel).is_err() {
        std::fs::copy(&quelle, &ziel).map_err(fehler)?;
        let _ = std::fs::remove_file(&quelle);
    }
    Ok(ziel.to_string_lossy().to_string())
}

/// Auf dem Schreibtisch: die Sicherungen in „Downloads", neueste zuerst.
#[tauri::command]
pub async fn sicherung_in_downloads_finden(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    use tauri::Manager;
    let ordner = app.path().download_dir().map_err(fehler)?;
    let mut liste: Vec<(std::time::SystemTime, String)> = std::fs::read_dir(&ordner)
        .map_err(fehler)?
        .flatten()
        .filter(|e| {
            let n = e.file_name().to_string_lossy().to_string();
            n.starts_with("openany-backup-") && n.ends_with(".age")
        })
        .map(|e| {
            let zeit = e
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::UNIX_EPOCH);
            (zeit, e.path().to_string_lossy().to_string())
        })
        .collect();
    liste.sort_by_key(|e| std::cmp::Reverse(e.0));
    Ok(liste.into_iter().map(|(_, p)| p).collect())
}

fn im_arbeitsordner(zustand: &Zustand, pfad: &str) -> Result<PathBuf, String> {
    let erlaubt = arbeitsordner(zustand).canonicalize().map_err(fehler)?;
    let p = PathBuf::from(pfad).canonicalize().map_err(fehler)?;
    if !p.starts_with(&erlaubt) || !p.is_file() {
        return Err("Not a backup of this app.".into());
    }
    Ok(p)
}

/// Eine Sicherung oeffnen und neben die Daten legen -- noch nicht
/// einspielen. Zurueck kommt der Kopf, damit die Rueckfrage sagen kann,
/// von wann und von welchem Geraet sie ist.
#[tauri::command]
pub async fn sicherung_oeffnen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    pfad: String,
    passphrase: String,
) -> Result<Kopf, String> {
    let neu = zustand.ordner.join(NEU);
    let _ = std::fs::remove_dir_all(&neu);
    std::fs::create_dir_all(&neu).map_err(fehler)?;

    let ergebnis = tauri::async_runtime::spawn_blocking({
        let neu = neu.clone();
        let pfad = PathBuf::from(&pfad);
        move || -> Result<Kopf, String> {
            let datei = std::io::BufReader::new(std::fs::File::open(&pfad).map_err(fehler)?);
            let kopf = openany_sicherung::lesen(datei, &passphrase, &neu).map_err(fehler)?;
            let db = neu.join("openany.sqlite");
            if !db.is_file() {
                return Err(openany_sicherung::SicherungFehler::KeineSicherung.to_string());
            }
            // Oeffnen prueft den Stand (eine neuere Datei wird abgewiesen)
            // und zieht eine aeltere nach -- jetzt, nicht erst beim Start.
            let s = openany_store::Speicher::oeffnen(&db).map_err(fehler)?;
            s.fremde_marken_vergessen().map_err(fehler)?;
            Ok(kopf)
        }
    })
    .await
    .map_err(fehler)?;

    // Was Android in den Zwischenspeicher kopiert hat, braucht es nach dem
    // Oeffnen nicht mehr. Bei einem Tippfehler in der Passphrase bleibt es,
    // damit man nicht noch einmal waehlen muss (fort mit `verwerfen`).
    match &ergebnis {
        Ok(_) => {
            if let Ok(p) = im_arbeitsordner(&zustand, &pfad) {
                let _ = std::fs::remove_file(p);
            }
        }
        Err(_) => {
            let _ = std::fs::remove_dir_all(&neu);
        }
    }
    ergebnis
}

/// Die geoeffnete Sicherung doch nicht einspielen.
#[tauri::command]
pub async fn sicherung_verwerfen(zustand: tauri::State<'_, Arc<Zustand>>) -> Result<(), String> {
    let _ = std::fs::remove_dir_all(zustand.ordner.join(NEU));
    let _ = std::fs::remove_dir_all(arbeitsordner(&zustand));
    Ok(())
}

/// Bestaetigt: beim naechsten Start ersetzt sie das Bisherige. Auf dem
/// Schreibtisch startet das Programm gleich neu; auf Android tut das die
/// Oberflaeche (`openanyAblage.neuStarten`).
#[tauri::command]
pub async fn sicherung_einspielen(
    app: tauri::AppHandle,
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<(), String> {
    let neu = zustand.ordner.join(NEU);
    if !neu.join("openany.sqlite").is_file() {
        return Err("No backup opened.".into());
    }
    std::fs::write(neu.join(BEREIT), b"").map_err(fehler)?;

    #[cfg(not(target_os = "android"))]
    app.restart();
    #[cfg(target_os = "android")]
    {
        let _ = app;
        Ok(())
    }
}

/// Beim Start, bevor die Datenbank offen ist. Ein Ordner ohne `BEREIT` ist
/// eine abgebrochene Rueckfrage und kommt fort.
pub fn einspielen_falls_bereit(ordner: &Path) -> std::io::Result<()> {
    let neu = ordner.join(NEU);
    if !neu.exists() {
        return Ok(());
    }
    if !neu.join(BEREIT).is_file() || !neu.join("openany.sqlite").is_file() {
        return std::fs::remove_dir_all(&neu);
    }

    // Das Postfach zuerst in den Tresor -- und die Klartextdatei sofort fort.
    let postfach = ablage(ordner, "postfach");
    let klartext = neu.join(POSTFACH);
    match std::fs::read_to_string(&klartext) {
        Ok(t) => {
            postfach.schreiben(&t)?;
            let _ = std::fs::remove_file(&klartext);
        }
        Err(_) => postfach.vergessen()?,
    }

    // Was dieses Geraet ausweist, gilt fuer die bisherigen Daten: fort.
    for name in ["anyid-ausweis", "openany-schluessel", "matrix-sitzung"] {
        let _ = ablage(ordner, name).vergessen();
    }
    if let Ok(liste) = std::fs::read_dir(ordner) {
        for e in liste.flatten() {
            if e.file_name().to_string_lossy().starts_with("matrix-") && e.path().is_dir() {
                let _ = std::fs::remove_dir_all(e.path());
            }
        }
    }

    // Das Bisherige fort ...
    for n in ["", "-wal", "-shm", "-journal"] {
        let _ = std::fs::remove_file(ordner.join(format!("openany.sqlite{n}")));
    }
    let _ = std::fs::remove_dir_all(ordner.join("inhalte"));
    for n in NAH {
        let _ = std::fs::remove_file(ordner.join("nah").join(n));
    }
    let _ = std::fs::remove_file(ordner.join("postausgang-openany.json"));

    // ... und das Neue an seine Stelle.
    for n in ["", "-wal", "-shm"] {
        let q = neu.join(format!("openany.sqlite{n}"));
        if q.is_file() {
            std::fs::rename(&q, ordner.join(format!("openany.sqlite{n}")))?;
        }
    }
    if neu.join("inhalte").is_dir() {
        std::fs::rename(neu.join("inhalte"), ordner.join("inhalte"))?;
    }
    std::fs::create_dir_all(ordner.join("nah"))?;
    for n in NAH {
        let q = neu.join("nah").join(n);
        if q.is_file() {
            std::fs::rename(&q, ordner.join("nah").join(n))?;
        }
    }
    let q = neu.join("postausgang-openany.json");
    if q.is_file() {
        std::fs::rename(&q, ordner.join("postausgang-openany.json"))?;
    }

    // Getrennt von openany.de; Name und Speicherregel bleiben die dieses
    // Geraets.
    let pfad = ordner.join("einstellungen.json");
    Einstellungen::lesen(&pfad)
        .ohne_openany()
        .schreiben(&pfad)?;

    std::fs::remove_dir_all(&neu)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schreib(p: &Path, inhalt: &str) {
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, inhalt).unwrap();
    }

    #[test]
    fn ohne_bestaetigung_bleibt_alles_wie_es_ist() {
        let o = tempfile::tempdir().unwrap();
        let o = o.path();
        schreib(&o.join("openany.sqlite"), "alt");
        schreib(&o.join(NEU).join("openany.sqlite"), "neu");

        einspielen_falls_bereit(o).unwrap();

        assert_eq!(
            std::fs::read_to_string(o.join("openany.sqlite")).unwrap(),
            "alt"
        );
        assert!(!o.join(NEU).exists());
    }

    #[test]
    fn bestaetigt_ersetzt_die_daten_und_behaelt_das_geraet() {
        let o = tempfile::tempdir().unwrap();
        let o = o.path();
        schreib(&o.join("openany.sqlite"), "alt");
        schreib(&o.join("openany.sqlite-wal"), "alt-wal");
        schreib(&o.join("inhalte/ab/alt"), "alt");
        schreib(&o.join("nah/angenommen.json"), "alt");
        schreib(&o.join("nah/identitaet.json"), "geraet");
        schreib(&o.join("nah/gepaart.json"), "paare");
        schreib(&o.join("ausweise/anyid-ausweis"), "ausweis");
        schreib(&o.join("matrix-abc/x"), "sitzung");
        schreib(
            &o.join("einstellungen.json"),
            r#"{"openany_basis":"https://openany.de","geraetename":"Tablet"}"#,
        );

        let n = o.join(NEU);
        schreib(&n.join("openany.sqlite"), "neu");
        schreib(&n.join("inhalte/cd/neu"), "neu");
        schreib(&n.join("nah/angenommen.json"), "neu");
        schreib(&n.join(POSTFACH), "[]");
        schreib(&n.join(BEREIT), "");

        einspielen_falls_bereit(o).unwrap();

        assert_eq!(
            std::fs::read_to_string(o.join("openany.sqlite")).unwrap(),
            "neu"
        );
        assert!(!o.join("openany.sqlite-wal").exists());
        assert!(!o.join("inhalte/ab/alt").exists());
        assert!(o.join("inhalte/cd/neu").exists());
        assert_eq!(
            std::fs::read_to_string(o.join("nah/angenommen.json")).unwrap(),
            "neu"
        );
        assert_eq!(
            std::fs::read_to_string(o.join("nah/identitaet.json")).unwrap(),
            "geraet"
        );
        assert_eq!(
            std::fs::read_to_string(o.join("nah/gepaart.json")).unwrap(),
            "paare"
        );
        assert!(!o.join("ausweise/anyid-ausweis").exists());
        assert!(!o.join("matrix-abc").exists());
        assert_eq!(
            std::fs::read_to_string(o.join("ausweise/postfach")).unwrap(),
            "[]"
        );

        let e = Einstellungen::lesen(&o.join("einstellungen.json"));
        assert_eq!(e.openany_basis, "");
        assert_eq!(e.geraetename, "Tablet");
        assert!(!n.exists(), "auch der Klartext des Postfachs ist fort");
    }

    #[test]
    fn inhalte_ohne_halbe_ladungen() {
        let o = tempfile::tempdir().unwrap();
        let w = o.path().join("inhalte");
        schreib(&w.join("ab/cdef"), "x");
        schreib(&w.join(".laden/halb"), "x");

        let liste: Vec<String> = inhalte(&w).into_iter().map(|(n, _)| n).collect();
        assert_eq!(liste, vec!["inhalte/ab/cdef".to_string()]);
    }
}
