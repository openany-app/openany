//! Viele Dateien auf einmal übernehmen -- der Lauf (docs/plan-viele-dateien.md, D2).
//!
//! Die Schlange steht im Speicher (`openany_store::uebernahmen`); hier wird sie
//! abgearbeitet: Datei lesen, in den Inhaltsspeicher legen, Doppelte
//! überspringen, eintragen -- über denselben Weg wie ein einzelnes Hochladen
//! (`galeriebefehle::bild_eintragen`, `dateibefehle::datei_eintragen`). Zum
//! Server trägt es danach der Abgleich.
//!
//! **Ein Lauf zur Zeit**, im Hintergrund, unabhängig vom Fenster. Er hört auf,
//! wenn die Schlange leer oder pausiert ist; neues Einreihen und das Fortsetzen
//! stoßen ihn wieder an, ebenso der Start der App.
//!
//! **Pfade kommen nie aus der WebView.** Was übernommen wird, wählt der Mensch
//! im Dialog des Systems (hier, in Rust) oder legt es ab (D1) -- ein Befehl
//! „übernimm Pfad X“ wäre eine Tür für jede fremde Zeile im Webview.

use crate::Zustand;
use openany_store::{Eingereiht, NeueUebernahme, NichtUebernommen, Uebernahme, Uebernahmestand};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::Manager;

/// Was auf dem Gerät mindestens frei bleiben soll -- die App braucht selbst
/// Platz für Datenbank und Vorschauen.
const RESERVE: u64 = 200 * 1024 * 1024;

/// Größer nimmt Dateien keine Datei an -- dieselbe Grenze wie beim Hochladen
/// in der Oberfläche (`DateiArbeitsflaeche.vue`).
const HOECHSTENS_DATEI: u64 = 1024 * 1024 * 1024;

/// Ob ein Lauf geht und ob er pausiert ist. Im Zustand der App.
#[derive(Default)]
pub struct Steuerung {
    laeuft: AtomicBool,
    pausiert: AtomicBool,
}

/// Der Stand für die Zeile in Mediathek, Dateien und Dokumente (Skizze D).
#[derive(serde::Serialize, Clone, Copy)]
pub struct Anzeige {
    #[serde(flatten)]
    stand: Uebernahmestand,
    laeuft: bool,
    pausiert: bool,
}

fn fehler(e: impl ToString) -> String {
    e.to_string()
}

async fn anzeige(zustand: &Zustand) -> Result<Anzeige, String> {
    let stand = zustand
        .speicher
        .lock()
        .await
        .uebernahmestand()
        .map_err(fehler)?;
    Ok(Anzeige {
        stand,
        laeuft: zustand.uebernahme.laeuft.load(Ordering::SeqCst),
        pausiert: zustand.uebernahme.pausiert.load(Ordering::SeqCst),
    })
}

/// Dem offenen Fenster sagen, dass sich der Stand geändert hat; es holt ihn
/// dann mit `uebernahme_stand` (über `eval`, wie `nachrichtenbefehle::melden`
/// -- ein `listen` bräuchte eine Berechtigung).
fn melden(app: &tauri::AppHandle) {
    if let Some(fenster) = app.get_webview_window("main") {
        let _ = fenster.eval("window.dispatchEvent(new Event('openany-uebernahme'))");
    }
}

/// Den Lauf anstoßen, falls keiner geht und nicht pausiert ist.
pub(crate) fn anstossen(app: &tauri::AppHandle, zustand: &Arc<Zustand>) {
    if zustand.uebernahme.pausiert.load(Ordering::SeqCst)
        || zustand.uebernahme.laeuft.swap(true, Ordering::SeqCst)
    {
        return;
    }
    let (app, z) = (app.clone(), zustand.clone());
    #[cfg(target_os = "android")]
    android::starten(&app, &z);
    tauri::async_runtime::spawn(async move {
        abarbeiten(&z, &|| melden(&app)).await;
        z.uebernahme.laeuft.store(false, Ordering::SeqCst);
        melden(&app);
    });
}

/// Die Schlange abarbeiten, bis sie leer oder pausiert ist. `nach_jeder`
/// meldet den Stand.
async fn abarbeiten(z: &Arc<Zustand>, nach_jeder: &(dyn Fn() + Sync)) -> usize {
    let mut getan = 0;
    loop {
        if z.uebernahme.pausiert.load(Ordering::SeqCst) {
            break;
        }
        let naechste = z.speicher.lock().await.uebernahme_naechste();
        let Ok(Some(u)) = naechste else {
            break;
        };
        #[cfg(target_os = "android")]
        if let Ok(a) = anzeige(z).await {
            android::stand_merken(&a);
        }
        let ergebnis = eine(z, &u).await;
        {
            let speicher = z.speicher.lock().await;
            let _ = match ergebnis {
                Ok(Ausgang::Neu(uuid)) => speicher.uebernahme_fertig(&u, &uuid),
                Ok(Ausgang::SchonDa(uuid)) => speicher.uebernahme_uebersprungen(&u, &uuid),
                Err(grund) => speicher.uebernahme_fehler(u.id, &grund),
            };
        }
        // Eine geteilte Kopie (Android) wird nicht mehr gebraucht.
        crate::teilen::aufraeumen(z, &u.quelle);
        getan += 1;
        nach_jeder();
    }
    getan
}

/// Ohne Fenster (Android, WorkManager -- `fotowaechter.rs`): die Schlange
/// gleich hier abarbeiten. Läuft schon ein Lauf (das Fenster ist offen),
/// übernimmt der. Gibt zurück, wie viele Dateien drankamen.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub(crate) async fn ohne_fenster_abarbeiten(z: &Arc<Zustand>) -> usize {
    if z.uebernahme.pausiert.load(Ordering::SeqCst)
        || z.uebernahme.laeuft.swap(true, Ordering::SeqCst)
    {
        return 0;
    }
    let getan = abarbeiten(z, &|| {}).await;
    z.uebernahme.laeuft.store(false, Ordering::SeqCst);
    getan
}

/// Unter Android hält ein Vordergrunddienst das Programm am Leben, solange
/// die Schlange läuft (`Uebernahmedienst.kt`). Er liest den Stand hier und
/// gibt „Pausieren" hierher.
#[cfg(target_os = "android")]
mod android {
    use super::{anstossen, Anzeige, Ordering};
    use crate::Zustand;
    use jni::objects::JClass;
    use jni::sys::{jboolean, jstring};
    use jni::JNIEnv;
    use std::sync::{Arc, Mutex, OnceLock};

    /// Für die Rückrufe aus Kotlin: Programm und Zustand.
    static LAUF: OnceLock<(tauri::AppHandle, Arc<Zustand>)> = OnceLock::new();
    /// `erledigt|gesamt`
    static STAND: Mutex<(u64, u64)> = Mutex::new((0, 0));

    pub(super) fn starten(app: &tauri::AppHandle, z: &Arc<Zustand>) {
        let _ = LAUF.set((app.clone(), z.clone()));
        crate::medien::dienst_starten();
    }

    pub(super) fn stand_merken(a: &Anzeige) {
        let erledigt = a.stand.fertig + a.stand.uebersprungen + a.stand.fehler;
        *STAND.lock().unwrap_or_else(|e| e.into_inner()) = (erledigt, erledigt + a.stand.wartet);
    }

    #[no_mangle]
    pub extern "system" fn Java_de_openany_app_Uebernahmedienst_stand(
        env: JNIEnv,
        _k: JClass,
    ) -> jstring {
        let (erledigt, gesamt) = *STAND.lock().unwrap_or_else(|e| e.into_inner());
        let laeuft = LAUF
            .get()
            .is_some_and(|(_, z)| z.uebernahme.laeuft.load(Ordering::SeqCst));
        env.new_string(format!("{erledigt}|{gesamt}|{}", laeuft as u8))
            .map(|s| s.into_raw())
            .unwrap_or(std::ptr::null_mut())
    }

    #[no_mangle]
    pub extern "system" fn Java_de_openany_app_Uebernahmedienst_pausieren(
        _env: JNIEnv,
        _k: JClass,
        an: jboolean,
    ) {
        if let Some((app, z)) = LAUF.get() {
            z.uebernahme.pausiert.store(an != 0, Ordering::SeqCst);
            if an == 0 {
                anstossen(app, z);
            }
            super::melden(app);
        }
    }
}

enum Ausgang {
    Neu(String),
    SchonDa(String),
}

/// Eine Datei übernehmen. `Err` ist der Grund für die Liste „nicht übernommen“:
/// ein Kürzel, das die Oberfläche übersetzt (`quelle_fort`, `keine_datei`,
/// `kein_medium`, `zu_gross`, `kein_platz`), sonst die Meldung, wie sie kam.
async fn eine(zustand: &Zustand, u: &Uebernahme) -> Result<Ausgang, String> {
    let (datei, meta) = quelle_oeffnen(&u.quelle)?;
    if !meta.is_file() {
        return Err("keine_datei".into());
    }
    if u.zone == "galerie" && !fuer_die_mediathek(&u.mime) {
        return Err("kein_medium".into());
    }
    if u.zone != "galerie" && meta.len() > HOECHSTENS_DATEI {
        return Err("zu_gross".into());
    }
    if frei(&zustand.ordner) < meta.len() + RESERVE {
        return Err("kein_platz".into());
    }

    // Lesen und ablegen, nicht auf dem Laufzeitkern: Ein Video kann Gigabytes
    // groß sein. In Stücken, nie die ganze Datei im Speicher.
    let mut ladung = zustand.inhalte.ladung().map_err(fehler)?;
    let ladung = tauri::async_runtime::spawn_blocking(move || -> Result<_, String> {
        let mut datei = datei;
        let mut puffer = vec![0u8; 1024 * 1024];
        loop {
            let n = datei.read(&mut puffer).map_err(fehler)?;
            if n == 0 {
                break;
            }
            ladung.schreiben(&puffer[..n]).map_err(fehler)?;
        }
        Ok(ladung)
    })
    .await
    .map_err(fehler)??;
    let (abdruck, groesse) = zustand.inhalte.ablegen(ladung).map_err(fehler)?;

    // Derselbe Inhalt liegt im Ziel schon: nichts anlegen. Die Bytes teilen
    // sich beide ohnehin -- der Inhaltsspeicher legt nach Abdruck ab.
    if let Some(vorhanden) = zustand
        .speicher
        .lock()
        .await
        .uebernahme_schon_da(&u.zone, u.ziel.as_deref(), &abdruck)
        .map_err(fehler)?
    {
        return Ok(Ausgang::SchonDa(vorhanden));
    }

    // Ohne Kopie: Nennt schon eine andere Zeile denselben Inhalt, bleibt er
    // hier -- sonst nur bis Vorschau und Standbild gerechnet sind.
    let ohne_kopie = u.ohne_kopie
        && !zustand
            .speicher
            .lock()
            .await
            .abdruck_benutzt(&abdruck)
            .map_err(fehler)?;
    let behalten = abdruck.clone();

    let uuid = if u.zone == "galerie" {
        crate::galeriebefehle::bild_eintragen(
            zustand,
            abdruck,
            groesse,
            u.name.clone(),
            u.mime.clone(),
            u.ziel.clone(),
        )
        .await?
        .uuid
    } else {
        crate::dateibefehle::datei_eintragen(
            zustand,
            abdruck,
            groesse,
            u.zone.clone(),
            u.ziel.clone(),
            &u.name,
            u.mime.clone(),
        )
        .await?
    };
    if ohne_kopie {
        zustand
            .speicher
            .lock()
            .await
            .aussen_merken(&openany_store::Aussen {
                abdruck: behalten.clone(),
                quelle: u.quelle.clone(),
                groesse,
                nur_wlan: u.nur_wlan,
            })
            .map_err(fehler)?;
        let _ = zustand.inhalte.entfernen(&behalten);
    }
    Ok(Ausgang::Neu(uuid))
}

/// Die Quelle zum Lesen: ein Pfad auf diesem Computer oder (Android) ein
/// Foto der Medienbibliothek (`content://media/…`, `medien.rs`).
fn quelle_oeffnen(quelle: &str) -> Result<(std::fs::File, std::fs::Metadata), String> {
    let datei = if quelle.starts_with("content://") {
        crate::medien::oeffnen(quelle)
    } else {
        std::fs::File::open(PathBuf::from(quelle)).ok()
    };
    let datei = datei.ok_or("quelle_fort")?;
    let meta = datei.metadata().map_err(|_| "quelle_fort")?;
    Ok((datei, meta))
}

fn fuer_die_mediathek(mime: &str) -> bool {
    mime.starts_with("image/") || mime.starts_with("video/") || mime.starts_with("audio/")
}

/// Freier Platz auf dem Laufwerk des App-Ordners; im Zweifel „genug“, damit
/// eine Abfrage, die scheitert, nicht jede Übernahme verhindert.
fn frei(ordner: &Path) -> u64 {
    fs4::available_space(ordner).unwrap_or(u64::MAX)
}

/// Was auf dem Gerät für Übernahmen frei ist (ohne die Reserve).
pub(crate) fn frei_fuer_uebernahme(zustand: &Zustand) -> u64 {
    frei(&zustand.ordner).saturating_sub(RESERVE)
}

/// Eine Datei auf diesem Computer als Eintrag für die Schlange. `None` für
/// alles, was keine lesbare Datei ist.
pub(crate) fn als_neue(pfad: &Path, zone: &str, ziel: Option<&str>) -> Option<NeueUebernahme> {
    let meta = std::fs::metadata(pfad).ok()?;
    if !meta.is_file() {
        return None;
    }
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_secs());
    let quelle = pfad.to_string_lossy().to_string();
    let name = crate::dateibefehle::name_saeubern(&pfad.file_name()?.to_string_lossy()).ok()?;
    Some(NeueUebernahme {
        quelle_schluessel: format!("{quelle}|{}|{mtime}", meta.len()),
        quelle,
        zone: zone.to_string(),
        ziel: ziel.map(str::to_string),
        mime: mime_guess::from_path(pfad)
            .first()
            .map(|m| m.essence_str().to_string())
            .unwrap_or_default(),
        name,
        groesse: meta.len(),
        ..Default::default()
    })
}

/// Dateien in die Schlange stellen und den Lauf anstoßen -- nach einer
/// Prüfung, ob der Platz auf dem Gerät reicht.
pub(crate) async fn einreihen(
    app: &tauri::AppHandle,
    zustand: &Arc<Zustand>,
    pfade: &[PathBuf],
    zone: &str,
    ziel: Option<&str>,
) -> Result<Eingereiht, String> {
    let neue: Vec<NeueUebernahme> = pfade
        .iter()
        .filter_map(|p| als_neue(p, zone, ziel))
        .collect();
    platz_pruefen(zustand, &neue)?;
    einreihen_neue(app, zustand, &neue).await
}

/// Reicht der Platz auf dem Gerät für alles zusammen? Sonst
/// `kein_platz:<gebraucht>:<frei>` in Bytes -- die Oberfläche macht den Satz
/// daraus.
pub(crate) fn platz_pruefen(zustand: &Zustand, neue: &[NeueUebernahme]) -> Result<(), String> {
    let bedarf: u64 = neue.iter().map(|n| n.groesse).sum();
    let frei = frei(&zustand.ordner).saturating_sub(RESERVE);
    if bedarf > frei {
        return Err(format!("kein_platz:{bedarf}:{frei}"));
    }
    Ok(())
}

/// Fertige Einträge einreihen und den Lauf anstoßen.
pub(crate) async fn einreihen_neue(
    app: &tauri::AppHandle,
    zustand: &Arc<Zustand>,
    neue: &[NeueUebernahme],
) -> Result<Eingereiht, String> {
    let e = zustand
        .speicher
        .lock()
        .await
        .uebernahmen_einreihen(neue)
        .map_err(fehler)?;
    anstossen(app, zustand);
    melden(app);
    Ok(e)
}

/// Prüfen, ob das Ziel (Ordner, Akte, Album) da ist.
pub(crate) async fn ziel_pruefen(
    zustand: &Zustand,
    zone: &str,
    ziel: Option<&str>,
) -> Result<(), String> {
    let speicher = zustand.speicher.lock().await;
    if zone == "galerie" {
        if let Some(a) = ziel {
            crate::galeriebefehle::lebendiges_album(&speicher, a)?;
        }
        Ok(())
    } else {
        crate::dateibefehle::zone_pruefen(zone)?;
        crate::dateibefehle::ordner_holen(&speicher, zone, ziel)
    }
}

/// Dateien wählen (Dialog des Systems) und übernehmen -- nur auf dem
/// Schreibtisch; unter Android bleibt das Auswahlfenster der Oberfläche.
#[tauri::command]
pub async fn uebernahme_waehlen(
    app: tauri::AppHandle,
    zustand: tauri::State<'_, Arc<Zustand>>,
    zone: String,
    ziel: Option<String>,
) -> Result<Option<Eingereiht>, String> {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = (&app, &zustand, &zone, &ziel);
        Err("Only available in the desktop app.".into())
    }
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        use tauri_plugin_dialog::DialogExt;
        ziel_pruefen(&zustand, &zone, ziel.as_deref()).await?;
        let gewaehlt = app.dialog().file().blocking_pick_files();
        let Some(gewaehlt) = gewaehlt else {
            return Ok(None);
        };
        let pfade: Vec<PathBuf> = gewaehlt
            .into_iter()
            .filter_map(|p| p.into_path().ok())
            .collect();
        einreihen(&app, &zustand, &pfade, &zone, ziel.as_deref())
            .await
            .map(Some)
    }
}

#[tauri::command]
pub async fn uebernahme_stand(zustand: tauri::State<'_, Arc<Zustand>>) -> Result<Anzeige, String> {
    anzeige(&zustand).await
}

/// Pausieren oder fortsetzen.
#[tauri::command]
pub async fn uebernahme_pausieren(
    app: tauri::AppHandle,
    zustand: tauri::State<'_, Arc<Zustand>>,
    pausiert: bool,
) -> Result<Anzeige, String> {
    zustand
        .uebernahme
        .pausiert
        .store(pausiert, Ordering::SeqCst);
    if !pausiert {
        anstossen(&app, &zustand);
    }
    anzeige(&zustand).await
}

/// Was nicht übernommen wurde, mit Grund.
#[tauri::command]
pub async fn uebernahme_nicht_uebernommen(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Vec<NichtUebernommen>, String> {
    zustand
        .speicher
        .lock()
        .await
        .uebernahmen_fehlgeschlagen()
        .map_err(fehler)
}

/// Die Zeile „fertig“ schließen: Erledigtes aus der Anzeige nehmen.
#[tauri::command]
pub async fn uebernahme_abschliessen(
    app: tauri::AppHandle,
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<(), String> {
    zustand
        .speicher
        .lock()
        .await
        .uebernahmen_abschliessen()
        .map_err(fehler)?;
    melden(&app);
    Ok(())
}

/// Alles Wartende verwerfen (Abbrechen).
#[tauri::command]
pub async fn uebernahme_abbrechen(
    app: tauri::AppHandle,
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<(), String> {
    zustand
        .speicher
        .lock()
        .await
        .uebernahmen_verwerfen()
        .map_err(fehler)?;
    zustand.uebernahme.pausiert.store(false, Ordering::SeqCst);
    melden(&app);
    Ok(())
}

/// „Liste der Übernommenen vergessen“ (Entscheidung 9): Danach darf alles
/// noch einmal übernommen werden.
#[tauri::command]
pub async fn uebernahme_vergessen(zustand: tauri::State<'_, Arc<Zustand>>) -> Result<u64, String> {
    zustand
        .speicher
        .lock()
        .await
        .uebernommen_vergessen()
        .map_err(fehler)
}

/// Beim Start: Wartet noch etwas vom letzten Mal, weitermachen.
pub(crate) fn beim_start(app: &tauri::AppHandle, zustand: &Arc<Zustand>) {
    anstossen(app, zustand);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn die_mediathek_nimmt_bild_video_und_ton() {
        assert!(fuer_die_mediathek("image/jpeg"));
        assert!(fuer_die_mediathek("video/mp4"));
        assert!(fuer_die_mediathek("audio/mpeg"));
        assert!(!fuer_die_mediathek("application/pdf"));
        assert!(!fuer_die_mediathek(""));
    }

    #[test]
    fn eine_datei_wird_mit_groesse_und_aenderungszeit_wiedererkannt() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("Urlaub 1.jpg");
        std::fs::write(&p, b"0123456789").unwrap();

        let n = als_neue(&p, "galerie", Some("album-1")).unwrap();
        assert_eq!(n.name, "Urlaub 1.jpg");
        assert_eq!(n.mime, "image/jpeg");
        assert_eq!(n.groesse, 10);
        assert!(n
            .quelle_schluessel
            .starts_with(&format!("{}|10|", p.to_string_lossy())));

        // Ein Ordner ist keine Datei.
        assert!(als_neue(d.path(), "galerie", None).is_none());
    }
}
