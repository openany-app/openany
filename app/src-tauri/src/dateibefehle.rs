/*
 * Dateien und Dokumente (Phase 3, Stufe A) -- dieselben Antworten wie drüben
 * unter /api/files, damit `src/quellen/dateien.js` nur umverpackt.
 *
 * `id` ist hier die uuid. Die Größe kommt wie drüben schon formatiert
 * (`FileNode::formattedSize`), weil die geteilte Arbeitsfläche sie so anzeigt.
 *
 * SPEICHERN IN STÜCKEN, ÜBER VIELE AUFRUFE. Eine 300-MB-Aufnahme als ein
 * Aufruf läge zweimal im Speicher des Telefons (Webansicht und Rust), und ein
 * Abbruch kostete alles. Jedes Stück wird sofort auf die Platte geschrieben
 * und mitgerechnet.
 */

use crate::Zustand;
use openany_store::{Datei, Ladung, Protokoll, Speicher};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

/// Eine Ladung, die gerade ankommt, und wohin sie gehört.
pub struct Hochladen {
    pub ladung: Ladung,
    pub zone: String,
    pub ordner: Option<String>,
    pub name: String,
    pub mime: String,
}

/// Ein angemeldetes Speichern herausnehmen -- nur, wenn es zu dieser Zone gehört.
pub fn hochladen_nehmen(
    zustand: &Zustand,
    kennzeichen: &str,
    zone: &str,
) -> Result<Hochladen, String> {
    let mut alle = zustand.hochladungen.lock().map_err(fehler)?;
    match alle.get(kennzeichen) {
        Some(h) if h.zone == zone => Ok(alle.remove(kennzeichen).expect("just seen")),
        Some(_) => Err("This save belongs somewhere else.".into()),
        None => Err("This save is not (or no longer) registered.".into()),
    }
}

pub type Hochladungen = std::sync::Mutex<HashMap<String, Hochladen>>;

fn zone_pruefen(zone: &str) -> Result<&str, String> {
    match zone {
        "files" | "documents" | "galerie" => Ok(zone),
        _ => Err("This zone does not exist.".into()),
    }
}

fn fehler(e: impl ToString) -> String {
    e.to_string()
}

/// Wie drüben: `1.5 MB`, bei Ordnern und leeren Dateien nichts.
fn groesse_text(d: &Datei) -> String {
    if d.ist_ordner || d.groesse == 0 {
        return String::new();
    }
    let mut wert = d.groesse as f64;
    let mut i = 0;
    let einheiten = ["B", "KB", "MB", "GB"];
    while wert >= 1024.0 && i < einheiten.len() - 1 {
        wert /= 1024.0;
        i += 1;
    }
    let zahl = format!("{wert:.2}");
    let zahl = zahl.trim_end_matches('0').trim_end_matches('.');
    format!("{zahl} {}", einheiten[i])
}

/// Namen säubern wie in der Webapp: Zeichen, die in Dateinamen nichts
/// verloren haben, fallen weg.
pub(crate) fn name_saeubern(roh: &str) -> Result<String, String> {
    let name: String = roh
        .trim()
        .chars()
        .filter(|c| {
            !matches!(
                c,
                '/' | '\\' | '?' | '%' | '*' | ':' | '|' | '"' | '<' | '>'
            )
        })
        .collect();
    if name.is_empty() || name == "." || name == ".." {
        return Err("Please enter a valid name.".into());
    }
    Ok(name)
}

const NAME_VERGEBEN: &str = "This name is already taken in this folder.";

#[derive(Serialize)]
pub struct DateiAnzeige {
    id: String,
    name: String,
    #[serde(rename = "type")]
    typ: &'static str,
    size: String,
    bytes: u64,
    mime: String,
    #[serde(rename = "updatedAt")]
    updated_at: String,
    /// Liegt der Inhalt auf diesem Gerät? (Stufe B: sonst „liegt auf …")
    vorhanden: bool,
    /// Ordner: auf diesem Gerät markiert zum Behalten.
    behalten: bool,
}

#[derive(Serialize)]
pub struct Krume {
    id: String,
    name: String,
}

#[derive(Serialize)]
pub struct DateiSeite {
    files: Vec<DateiAnzeige>,
    has_more: bool,
    next_page: Option<usize>,
    breadcrumbs: Vec<Krume>,
}

#[derive(Serialize)]
pub struct Baumordner {
    id: String,
    name: String,
    parent_id: Option<String>,
}

#[derive(Serialize)]
pub struct Baum {
    folders: Vec<Baumordner>,
}

fn anzeige(zustand: &Zustand, d: Datei, markiert: &[String]) -> DateiAnzeige {
    DateiAnzeige {
        behalten: markiert.contains(&d.uuid),
        vorhanden: d.ist_ordner || d.abdruck.as_deref().is_some_and(|a| zustand.inhalte.hat(a)),
        id: d.uuid.clone(),
        typ: d.anzeigetyp(),
        size: groesse_text(&d),
        bytes: d.groesse,
        name: d.name,
        mime: d.mime,
        updated_at: d.geaendert_at,
    }
}

fn ordner_holen(speicher: &Speicher, zone: &str, id: Option<&str>) -> Result<(), String> {
    let Some(id) = id else { return Ok(()) };
    match speicher.datei(id).map_err(fehler)? {
        Some(d) if d.ist_ordner && d.zone == zone && d.papierkorb_at.is_none() => Ok(()),
        _ => Err("This folder no longer exists.".into()),
    }
}

fn lebendig(speicher: &Speicher, id: &str) -> Result<Datei, String> {
    match speicher.datei(id).map_err(fehler)? {
        Some(d) if d.papierkorb_at.is_none() => Ok(d),
        _ => Err("This file no longer exists.".into()),
    }
}

#[tauri::command]
pub async fn dateien_liste(
    zustand: tauri::State<'_, Arc<Zustand>>,
    zone: String,
    ordner: Option<String>,
) -> Result<DateiSeite, String> {
    let zone = zone_pruefen(&zone)?;
    let speicher = zustand.speicher.lock().await;
    let liste = speicher
        .dateien_im_ordner(zone, ordner.as_deref())
        .map_err(fehler)?;
    let weg = speicher.ordnerweg(ordner.as_deref()).map_err(fehler)?;
    drop(speicher);
    let markiert = zustand.einstellungen.lock().await.behalten.clone();

    Ok(DateiSeite {
        files: liste
            .into_iter()
            .map(|d| anzeige(&zustand, d, &markiert))
            .collect(),
        has_more: false,
        next_page: None,
        breadcrumbs: weg
            .into_iter()
            .map(|d| Krume {
                id: d.uuid,
                name: d.name,
            })
            .collect(),
    })
}

/// Ein Treffer der Suche: wie in der Liste, dazu wo er liegt.
#[derive(Serialize)]
pub struct Treffer {
    #[serde(flatten)]
    datei: DateiAnzeige,
    parent_id: Option<String>,
    /// Die Ordner von oben bis zu ihm.
    pfad: Vec<String>,
    /// Im Inhalt gefunden: ein Stueck um die Fundstelle.
    ausschnitt: Option<String>,
}

#[derive(Serialize)]
pub struct Suchergebnis {
    results: Vec<Treffer>,
}

/// Passt der Name auf die Suche? Ohne Ruecksicht auf Gross und klein --
/// auch bei Umlauten.
fn name_passt(name: &str, q: &str) -> bool {
    name.to_lowercase().contains(&q.to_lowercase())
}

/// Ein Stueck Text um die erste Fundstelle, auf eine Zeile gebracht -- wie
/// drueben (FileNodeController::ausschnitt).
fn ausschnitt(text: &str, q: &str) -> Option<String> {
    let zeichen: Vec<char> = text.chars().collect();
    let klein: Vec<char> = text.to_lowercase().chars().collect();
    let such: Vec<char> = q.to_lowercase().chars().collect();
    // Faellt die Kleinschreibung laenger aus als das Original (selten),
    // stimmen die Stellen nicht mehr -- dann lieber ohne Ausschnitt.
    if klein.len() != zeichen.len() || such.is_empty() {
        return None;
    }
    let pos = klein
        .windows(such.len())
        .position(|w| w == such.as_slice())?;
    let von = pos.saturating_sub(60);
    let bis = (pos + such.len() + 60).min(zeichen.len());
    let stueck: String = zeichen[von..bis].iter().collect();
    let stueck = stueck.split_whitespace().collect::<Vec<_>>().join(" ");
    Some(format!(
        "{}{}{}",
        if von > 0 { "…" } else { "" },
        stueck,
        if bis < zeichen.len() { "…" } else { "" }
    ))
}

/// Suche ueber alle Ordner einer Zone -- nach Namen (Stufe 1) und im Inhalt
/// (Stufe 2, 02.10.2026: soweit der Inhaltsleser ihn hier gelesen hat) --
/// im eigenen Speicher, also auch ohne Netz.
/// Hoechstens 50 Treffer, wie drueben. Was unter einem Ordner im Papierkorb
/// liegt, faellt heraus.
#[tauri::command]
pub async fn dateien_suchen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    zone: String,
    q: String,
) -> Result<Suchergebnis, String> {
    let zone = zone_pruefen(&zone)?;
    let q = q.trim().to_string();
    if q.chars().count() < 2 {
        return Ok(Suchergebnis {
            results: Vec::new(),
        });
    }
    let speicher = zustand.speicher.lock().await;
    let texte = speicher.dateitexte_der_zone(zone).map_err(fehler)?;
    let mut gefunden = Vec::new();
    for d in speicher.dateien_der_zone(zone).map_err(fehler)? {
        let stelle = if name_passt(&d.name, &q) {
            None
        } else {
            match texte.get(&d.uuid).and_then(|t| ausschnitt(t, &q)) {
                Some(a) => Some(a),
                None => continue,
            }
        };
        let weg = speicher.ordnerweg(d.eltern.as_deref()).map_err(fehler)?;
        if weg.iter().any(|o| o.papierkorb_at.is_some()) {
            continue;
        }
        gefunden.push((d, weg, stelle));
        if gefunden.len() == 50 {
            break;
        }
    }
    drop(speicher);
    let markiert = zustand.einstellungen.lock().await.behalten.clone();
    Ok(Suchergebnis {
        results: gefunden
            .into_iter()
            .map(|(d, weg, stelle)| Treffer {
                parent_id: d.eltern.clone(),
                pfad: weg.into_iter().map(|o| o.name).collect(),
                ausschnitt: stelle,
                datei: anzeige(&zustand, d, &markiert),
            })
            .collect(),
    })
}

/// Eine Datei, deren Text der Inhaltsleser holen soll.
#[derive(Serialize)]
pub struct Ungelesen {
    id: String,
    name: String,
    mime: String,
    bytes: u64,
    abdruck: String,
}

#[derive(Serialize)]
pub struct TextOffen {
    offen: usize,
    files: Vec<Ungelesen>,
}

/// Was der Inhaltsleser noch lesen kann -- nur, was HIER liegt: Eine Datei
/// eigens dafuer herunterzuladen, hiesse den ganzen Speicher aufs Geraet zu
/// holen. Hoechstens 20 auf einmal.
#[tauri::command]
pub async fn dateien_text_offen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    zone: String,
    version: Option<i64>,
) -> Result<TextOffen, String> {
    let zone = zone_pruefen(&zone)?;
    let liste = zustand
        .speicher
        .lock()
        .await
        .dateien_ohne_text(zone, version.unwrap_or(1))
        .map_err(fehler)?;
    let hier: Vec<Ungelesen> = liste
        .into_iter()
        .filter_map(|d| {
            let abdruck = d.abdruck.clone()?;
            zustand.inhalte.hat(&abdruck).then_some(Ungelesen {
                id: d.uuid,
                name: d.name,
                mime: d.mime,
                bytes: d.groesse,
                abdruck,
            })
        })
        .collect();
    Ok(TextOffen {
        offen: hier.len(),
        files: hier.into_iter().take(20).collect(),
    })
}

/// Den gelesenen Text ablegen (stand: ok | keinText | unlesbar | zuLang |
/// format | zuGross).
#[tauri::command]
pub async fn datei_text_setzen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: String,
    abdruck: String,
    stand: String,
    text: Option<String>,
    version: Option<i64>,
) -> Result<(), String> {
    if !["ok", "keinText", "unlesbar", "zuLang", "format", "zuGross"].contains(&stand.as_str()) {
        return Err(format!("Unknown state: {stand}"));
    }
    let text: Option<String> = text
        .filter(|_| stand == "ok")
        .map(|t| t.chars().take(200_000).collect());
    let gesetzt = zustand
        .speicher
        .lock()
        .await
        .dateitext_setzen(&id, &abdruck, &stand, text.as_deref(), version.unwrap_or(1))
        .map_err(fehler)?;
    if gesetzt {
        Ok(())
    } else {
        Err("The file has changed in the meantime.".into())
    }
}

#[tauri::command]
pub async fn dateien_baum(
    zustand: tauri::State<'_, Arc<Zustand>>,
    zone: String,
) -> Result<Baum, String> {
    let zone = zone_pruefen(&zone)?;
    let speicher = zustand.speicher.lock().await;
    Ok(Baum {
        folders: speicher
            .ordner_der_zone(zone)
            .map_err(fehler)?
            .into_iter()
            .map(|d| Baumordner {
                id: d.uuid,
                name: d.name,
                parent_id: d.eltern,
            })
            .collect(),
    })
}

#[tauri::command]
pub async fn datei_ordner_anlegen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    zone: String,
    ordner: Option<String>,
    name: String,
) -> Result<String, String> {
    let zone = zone_pruefen(&zone)?;
    let name = name_saeubern(&name)?;
    let speicher = zustand.speicher.lock().await;
    ordner_holen(&speicher, zone, ordner.as_deref())?;
    if speicher
        .dateiname_vergeben(zone, ordner.as_deref(), &name, None)
        .map_err(fehler)?
    {
        return Err(NAME_VERGEBEN.into());
    }
    let uuid = uuid::Uuid::new_v4().to_string();
    speicher
        .datei_schreiben(
            &Datei {
                uuid: uuid.clone(),
                zone: zone.into(),
                ist_ordner: true,
                eltern: ordner,
                name,
                ..Default::default()
            },
            Protokoll::Merken,
        )
        .map_err(fehler)?;
    Ok(uuid)
}

#[tauri::command]
pub async fn datei_umbenennen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: String,
    name: String,
) -> Result<(), String> {
    let name = name_saeubern(&name)?;
    let speicher = zustand.speicher.lock().await;
    let mut d = lebendig(&speicher, &id)?;
    if d.name == name {
        return Ok(());
    }
    if speicher
        .dateiname_vergeben(&d.zone, d.eltern.as_deref(), &name, Some(&d.uuid))
        .map_err(fehler)?
    {
        return Err(NAME_VERGEBEN.into());
    }
    d.name = name;
    d.geaendert_at = String::new();
    speicher
        .datei_schreiben(&d, Protokoll::Merken)
        .map_err(fehler)
}

#[tauri::command]
pub async fn datei_verschieben(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: String,
    ordner: Option<String>,
) -> Result<(), String> {
    let speicher = zustand.speicher.lock().await;
    let mut d = lebendig(&speicher, &id)?;
    ordner_holen(&speicher, &d.zone, ordner.as_deref())?;
    if let Some(ziel) = ordner.as_deref() {
        if d.ist_ordner && speicher.liegt_in(ziel, &d.uuid).map_err(fehler)? {
            return Err("A folder cannot be inside itself.".into());
        }
    }
    if d.eltern == ordner {
        return Ok(());
    }
    if speicher
        .dateiname_vergeben(&d.zone, ordner.as_deref(), &d.name, Some(&d.uuid))
        .map_err(fehler)?
    {
        return Err(NAME_VERGEBEN.into());
    }
    d.eltern = ordner;
    d.geaendert_at = String::new();
    speicher
        .datei_schreiben(&d, Protokoll::Merken)
        .map_err(fehler)
}

#[tauri::command]
pub async fn datei_papierkorb(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: String,
) -> Result<(), String> {
    let speicher = zustand.speicher.lock().await;
    speicher
        .datei_papierkorb(&id, Protokoll::Merken)
        .map_err(fehler)?;
    Ok(())
}

/// Zurück aus dem Papierkorb (vom Papierkorb aus gerufen).
pub async fn wiederherstellen(zustand: &Zustand, id: &str) -> Result<(), String> {
    let speicher = zustand.speicher.lock().await;
    speicher
        .datei_wiederherstellen(id, Protokoll::Merken)
        .map_err(fehler)?;
    Ok(())
}

/// Endgültig -- und die Inhalte, die danach niemand mehr nennt, gleich mit.
pub async fn endgueltig(zustand: &Zustand, id: &str) -> Result<(), String> {
    let speicher = zustand.speicher.lock().await;
    for abdruck in speicher
        .datei_loeschen(id, Protokoll::Merken)
        .map_err(fehler)?
    {
        if !speicher.abdruck_benutzt(&abdruck).map_err(fehler)? {
            let _ = zustand.inhalte.entfernen(&abdruck);
        }
    }
    Ok(())
}

/// Ein Hochladen anmelden. Gibt das Kennzeichen zurück, unter dem die Stücke
/// kommen.
#[tauri::command]
pub async fn datei_hochladen_beginnen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    zone: String,
    ordner: Option<String>,
    name: String,
    mime: String,
) -> Result<String, String> {
    let zone = zone_pruefen(&zone)?.to_string();
    let name = name_saeubern(&name)?;
    if zone != "galerie" {
        let speicher = zustand.speicher.lock().await;
        ordner_holen(&speicher, &zone, ordner.as_deref())?;
    }
    let ladung = zustand.inhalte.ladung().map_err(fehler)?;
    let kennzeichen = uuid::Uuid::new_v4().simple().to_string();
    zustand.hochladungen.lock().map_err(fehler)?.insert(
        kennzeichen.clone(),
        Hochladen {
            ladung,
            zone,
            ordner,
            name,
            mime,
        },
    );
    Ok(kennzeichen)
}

/// Ein Stück, Base64 (siehe `src/quellen/dateien.js`, warum nicht roh).
/// Gibt zurück, wie viel bisher angekommen ist.
#[tauri::command]
pub async fn datei_hochladen_stueck(
    zustand: tauri::State<'_, Arc<Zustand>>,
    kennzeichen: String,
    stueck: String,
) -> Result<u64, String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(stueck.as_bytes())
        .map_err(|_| "A chunk arrived unreadable.".to_string())?;
    let mut alle = zustand.hochladungen.lock().map_err(fehler)?;
    let h = alle
        .get_mut(&kennzeichen)
        .ok_or("This save is not (or no longer) registered.")?;
    h.ladung.schreiben(&bytes).map_err(fehler)?;
    Ok(h.ladung.groesse())
}

/// Fertig: ablegen und eintragen. Ist der Name im Ordner vergeben, bekommt
/// die neue Datei „Name (2).pdf" -- ein Hochladen soll nichts überschreiben.
#[tauri::command]
pub async fn datei_hochladen_fertig(
    zustand: tauri::State<'_, Arc<Zustand>>,
    kennzeichen: String,
) -> Result<String, String> {
    let h = {
        let alle = zustand.hochladungen.lock().map_err(fehler)?;
        let zone = alle
            .get(&kennzeichen)
            .map(|h| h.zone.clone())
            .ok_or("This save is not (or no longer) registered.")?;
        drop(alle);
        if zone == "galerie" {
            return Err("Pictures are saved via the gallery.".into());
        }
        hochladen_nehmen(&zustand, &kennzeichen, &zone)?
    };
    let (abdruck, groesse) = zustand.inhalte.ablegen(h.ladung).map_err(fehler)?;

    let speicher = zustand.speicher.lock().await;
    let name = freier_name(&speicher, &h.zone, h.ordner.as_deref(), &h.name)?;
    let uuid = uuid::Uuid::new_v4().to_string();
    speicher
        .datei_schreiben(
            &Datei {
                uuid: uuid.clone(),
                zone: h.zone,
                ist_ordner: false,
                eltern: h.ordner,
                name,
                groesse,
                mime: if h.mime.is_empty() {
                    "application/octet-stream".into()
                } else {
                    h.mime
                },
                abdruck: Some(abdruck),
                ..Default::default()
            },
            Protokoll::Merken,
        )
        .map_err(fehler)?;
    Ok(uuid)
}

/// Den INHALT einer Datei ersetzen -- gleiche Kennung, gleicher Platz,
/// gleicher Name. Die Stuecke kommen wie beim Hochladen
/// (`datei_hochladen_beginnen` + `_stueck`), nur das Ende ist ein anderes.
///
/// Gebraucht von der Texterkennung (`@oberflaeche/texterkennung`): Sie
/// schreibt eine unsichtbare Textebene in ein eingescanntes PDF. Eine NEUE
/// Datei daneben waere falsch -- Verweise, Freigaben und der Platz in der
/// Akte haengen an der alten. Der Abgleich sieht einen neuen Abdruck und
/// traegt den Inhalt hinaus wie jede andere Aenderung.
///
/// Der alte Inhalt wird weggeraeumt, wenn ihn sonst niemand mehr nennt.
///
/// MIT `vorfassung` (seit 27.09.2026, der PDF-Betrachter beim Speichern
/// ausgefuellter Formulare) bleibt er stattdessen als eigene Datei
/// „Name (vor Bearbeitung).pdf" im Papierkorb. `vorfassung` ist der Zusatz
/// in der Sprache der Oberflaeche („vor Bearbeitung", „before editing" …) -- der Rueckweg ohne
/// Versionsgeschichte, wie auf dem Server (FileNodeController::replaceContent).
#[tauri::command]
pub async fn datei_ersetzen_fertig(
    zustand: tauri::State<'_, Arc<Zustand>>,
    kennzeichen: String,
    id: String,
    vorfassung: Option<String>,
) -> Result<(), String> {
    let d = {
        let speicher = zustand.speicher.lock().await;
        lebendig(&speicher, &id)?
    };
    if d.ist_ordner {
        return Err("A folder has no content.".into());
    }
    let h = hochladen_nehmen(&zustand, &kennzeichen, &d.zone)?;
    let (abdruck, groesse) = zustand.inhalte.ablegen(h.ladung).map_err(fehler)?;

    let speicher = zustand.speicher.lock().await;
    // Noch einmal lesen: Waehrend die Stuecke kamen, kann die Datei
    // umbenannt oder verschoben worden sein -- das soll nicht verloren gehen.
    let mut d = lebendig(&speicher, &id)?;
    let vorher = d.clone();
    let alt = d.abdruck.replace(abdruck);
    d.groesse = groesse;
    if !h.mime.is_empty() {
        d.mime = h.mime;
    }
    d.geaendert_at = String::new();
    speicher
        .datei_schreiben(&d, Protokoll::Merken)
        .map_err(fehler)?;
    let zusatz = vorfassung
        .map(|z| z.trim().to_string())
        .filter(|z| !z.is_empty());
    if let (Some(zusatz), true) = (zusatz, alt.is_some()) {
        // Eine zweite Zeile mit dem alten Abdruck, gleich in den Papierkorb.
        // Der Inhalt bleibt in der Ablage, weil diese Zeile ihn nennt.
        let name = freier_name(
            &speicher,
            &vorher.zone,
            vorher.eltern.as_deref(),
            &vorfassungs_name(&vorher.name, &zusatz),
        )?;
        let kopie = Datei {
            uuid: uuid::Uuid::new_v4().to_string(),
            name,
            geaendert_at: String::new(),
            ..vorher
        };
        speicher
            .datei_schreiben(&kopie, Protokoll::Merken)
            .map_err(fehler)?;
        speicher
            .datei_papierkorb(&kopie.uuid, Protokoll::Merken)
            .map_err(fehler)?;
    } else if let Some(alt) = alt {
        if !speicher.abdruck_benutzt(&alt).map_err(fehler)? {
            let _ = zustand.inhalte.entfernen(&alt);
        }
    }
    Ok(())
}

/// („Vertrag.pdf", „vor Bearbeitung") → „Vertrag (vor Bearbeitung).pdf".
fn vorfassungs_name(name: &str, zusatz: &str) -> String {
    match name.rfind('.') {
        Some(punkt) if punkt > 0 => format!("{} ({zusatz}){}", &name[..punkt], &name[punkt..]),
        _ => format!("{name} ({zusatz})"),
    }
}

#[tauri::command]
pub async fn datei_hochladen_abbrechen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    kennzeichen: String,
) -> Result<(), String> {
    if let Some(h) = zustand
        .hochladungen
        .lock()
        .map_err(fehler)?
        .remove(&kennzeichen)
    {
        zustand.inhalte.verwerfen(h.ladung);
    }
    Ok(())
}

pub(crate) fn freier_name(
    speicher: &Speicher,
    zone: &str,
    ordner: Option<&str>,
    name: &str,
) -> Result<String, String> {
    if !speicher
        .dateiname_vergeben(zone, ordner, name, None)
        .map_err(fehler)?
    {
        return Ok(name.to_string());
    }
    let (stamm, endung) = match name.rfind('.') {
        Some(i) if i > 0 => (&name[..i], &name[i..]),
        _ => (name, ""),
    };
    for n in 2..1000 {
        let versuch = format!("{stamm} ({n}){endung}");
        if !speicher
            .dateiname_vergeben(zone, ordner, &versuch, None)
            .map_err(fehler)?
        {
            return Ok(versuch);
        }
    }
    Err(NAME_VERGEBEN.into())
}

/// Der Inhalt einer Datei als rohe Bytes -- für Vorschau und Öffnen.
///
/// Mit Obergrenze: Was größer ist, gehört nicht durch die Webansicht, sondern
/// an ein anderes Programm (Öffnen, A6).
#[tauri::command]
pub async fn datei_inhalt(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: String,
) -> Result<tauri::ipc::Response, String> {
    const HOECHSTENS: u64 = 64 * 1024 * 1024;
    let d = {
        let speicher = zustand.speicher.lock().await;
        lebendig(&speicher, &id)?
    };
    let abdruck = d.abdruck.ok_or("A folder has no content.")?;
    if d.groesse > HOECHSTENS {
        return Err("This file is too large for the preview.".into());
    }
    if !zustand.inhalte.hat(&abdruck) {
        return Err("The content is not on this device.".into());
    }
    let bytes = zustand
        .inhalte
        .lesen(&abdruck, 0, HOECHSTENS as usize)
        .map_err(fehler)?;
    Ok(tauri::ipc::Response::new(bytes))
}

/// Eine Datei mit einem anderen Programm öffnen.
///
/// Die Ablage kennt nur Abdrücke, kein anderes Programm weiß mit
/// `ab/abcdef…` etwas anzufangen. Deshalb entsteht im Zwischenspeicher
/// `oeffnen/<zufall>/<Name>` -- als harte Verknüpfung (kostet keinen Platz,
/// dieselbe Partition), sonst als Kopie. Beim nächsten Start wird der Ordner
/// geleert.
///
/// Am Schreibtisch öffnet die Schale selbst und gibt `None` zurück. Auf
/// Android gibt sie den Pfad zurück; die Oberfläche reicht ihn an
/// `window.openanyOeffnen` (MainActivity.kt), das ihn über den FileProvider
/// an ein anderes Programm gibt.
#[tauri::command]
pub async fn datei_oeffnen(
    app: tauri::AppHandle,
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: String,
) -> Result<Option<String>, String> {
    let d = {
        let speicher = zustand.speicher.lock().await;
        lebendig(&speicher, &id)?
    };
    let abdruck = d.abdruck.ok_or("A folder cannot be opened.")?;
    zum_oeffnen(app, &zustand, &abdruck, &d.name)
}

/// Einen Inhalt der Ablage einem anderen Programm geben -- für Dateien und
/// (seit 27.09.2026) für Videos der Galerie, die der Videoplayer des Systems
/// abspielt. Siehe `datei_oeffnen` für den Weg über `oeffnen/<zufall>/<Name>`.
pub(crate) fn zum_oeffnen(
    app: tauri::AppHandle,
    zustand: &Zustand,
    abdruck: &str,
    name: &str,
) -> Result<Option<String>, String> {
    let quelle = zustand
        .inhalte
        .pfad(abdruck)
        .filter(|p| p.is_file())
        .ok_or("The content is not on this device.")?;

    let ordner = zustand
        .zwischenspeicher
        .join("oeffnen")
        .join(uuid::Uuid::new_v4().simple().to_string());
    std::fs::create_dir_all(&ordner).map_err(fehler)?;
    let ziel = ordner.join(name);
    if std::fs::hard_link(&quelle, &ziel).is_err() {
        std::fs::copy(&quelle, &ziel).map_err(fehler)?;
    }

    #[cfg(desktop)]
    {
        use tauri_plugin_opener::OpenerExt;
        app.opener()
            .open_path(ziel.to_string_lossy(), None::<&str>)
            .map_err(fehler)?;
        Ok(None)
    }
    #[cfg(mobile)]
    {
        let _ = app;
        Ok(Some(ziel.to_string_lossy().to_string()))
    }
}

/* ── Inhalte zwischen Geräten (Stufe B) ─────────────────────────────── */

fn frei(zustand: &Zustand) -> u64 {
    fs4::available_space(zustand.inhalte_wurzel()).unwrap_or(0)
}

fn gesamt(zustand: &Zustand) -> u64 {
    fs4::total_space(zustand.inhalte_wurzel()).unwrap_or(0)
}

/// Welche Originale geholt werden.
pub enum Originale {
    Keine,
    Alle,
    /// Nur in diesen Ordnern und Alben (samt allem darunter).
    Ausgewaehlt(Vec<String>),
}

impl Originale {
    pub async fn nach_regel(zustand: &Zustand) -> Self {
        let e = zustand.einstellungen.lock().await;
        // Eigene Freigaben in Projekten bleiben IMMER -- auch „bei Bedarf".
        let freigaben = e.freigaben_behalten();
        match e.regel() {
            "alles" => Originale::Alle,
            "ausgewaehlt" => Originale::Ausgewaehlt(e.alles_behalten()),
            _ if !freigaben.is_empty() => Originale::Ausgewaehlt(freigaben),
            _ => Originale::Keine,
        }
    }
}

/// Die Abdrücke aller Originale in markierten Ordnern und Alben -- ob sie hier
/// liegen oder nicht.
async fn fehlend_ausgewaehlt(
    zustand: &Zustand,
    markiert: &[String],
) -> Result<std::collections::BTreeSet<String>, String> {
    let bereiche = behaltene_bereiche(zustand, markiert).await?;
    let speicher = zustand.speicher.lock().await;
    let mut aus = std::collections::BTreeSet::new();
    for d in speicher.lebendige_dateien().map_err(fehler)? {
        if d.eltern.as_deref().is_some_and(|e| bereiche.contains(e)) {
            aus.extend(d.abdruck);
        }
    }
    for b in speicher.lebendige_bilder().map_err(fehler)? {
        if b.album.as_deref().is_some_and(|a| bereiche.contains(a)) {
            aus.extend(b.abdruck);
        }
    }
    Ok(aus)
}

/// Die markierten Ordner und Alben, aufgefaltet zu allem, was darin liegt.
async fn behaltene_bereiche(
    zustand: &Zustand,
    markiert: &[String],
) -> Result<std::collections::BTreeSet<String>, String> {
    let speicher = zustand.speicher.lock().await;
    let mut alle = std::collections::BTreeSet::new();
    for m in markiert {
        alle.extend(speicher.ordner_mit_inhalt(m).map_err(fehler)?);
        alle.extend(speicher.album_mit_unteralben(m).map_err(fehler)?);
    }
    Ok(alle)
}

/// Was hier als Inhalt fehlt: (Abdruck, Größe), je Inhalt einmal.
///
/// Vorschaubilder immer -- sie sind klein, und ohne sie wäre die Galerie auf
/// einem Gerät „bei Bedarf" eine Wand aus leeren Kacheln. Originale (Dateien
/// und Bilder) nur, wenn `originale`.
async fn fehlend(zustand: &Zustand, originale: Originale) -> Result<Vec<(String, u64)>, String> {
    const VORSCHAU_GROESSE: u64 = 256 * 1024;
    let behalten = match &originale {
        Originale::Ausgewaehlt(liste) => behaltene_bereiche(zustand, liste).await?,
        _ => Default::default(),
    };
    let nimmt = |bereich: Option<&str>| match &originale {
        Originale::Keine => false,
        Originale::Alle => true,
        Originale::Ausgewaehlt(_) => bereich.is_some_and(|b| behalten.contains(b)),
    };
    let speicher = zustand.speicher.lock().await;
    let mut gesehen = std::collections::BTreeSet::new();
    let mut liste: Vec<(String, u64)> = Vec::new();
    let bilder = speicher.lebendige_bilder().map_err(fehler)?;
    for b in &bilder {
        if let Some(v) = &b.vorschau {
            liste.push((v.clone(), VORSCHAU_GROESSE));
        }
    }

    /*
     * ANHAENGE: DIE VORSCHAU IMMER, das Original nie von hier aus.
     *
     * Ein Bild, das an einer Notiz haengt, kommt drueben NICHT im
     * `media`-Strom (Regel 1: jede Datei genau einmal) -- dieses Programm hat
     * also keine Bildzeile dafuer und faende es in der Schleife oben nicht.
     * Ohne diesen Zusatz kamen 26 Zuordnungen an und kein einziges Bild.
     *
     * Das Original bleibt aussen vor: Im Flusstext steht die Vorschau, und
     * wer den Anhang wirklich oeffnet, holt ihn ueber `von_geraeten_holen`.
     * Sonst zoege eine Notiz mit zwanzig Fotos zwanzig Originale nach.
     */
    for a in speicher.anhaenge_alle().map_err(fehler)? {
        if let Some(v) = a.vorschau {
            liste.push((v, VORSCHAU_GROESSE));
        }
    }
    for d in speicher.lebendige_dateien().map_err(fehler)? {
        if let Some(a) = d.abdruck.filter(|_| nimmt(d.eltern.as_deref())) {
            liste.push((a, d.groesse));
        }
    }
    for b in bilder {
        if let Some(a) = b.abdruck.filter(|_| nimmt(b.album.as_deref())) {
            liste.push((a, b.groesse));
        }
    }
    Ok(liste
        .into_iter()
        .filter(|(a, _)| !zustand.inhalte.hat(a) && gesehen.insert(a.clone()))
        .collect())
}

/// Nach einem Abgleich: Vorschaubilder immer, Originale bei „alles behalten".
pub async fn fehlende_holen<G: openany_sync::Gegenstelle + ?Sized>(
    zustand: &Zustand,
    gegenstelle: &G,
) -> openany_sync::InhalteBericht {
    let originale = Originale::nach_regel(zustand).await;
    let liste = match fehlend(zustand, originale).await {
        Ok(l) => l,
        Err(e) => {
            return openany_sync::InhalteBericht {
                fehler: vec![e],
                ..Default::default()
            }
        }
    };
    let gesamt = gesamt(zustand);
    openany_sync::fehlende_inhalte_holen(
        &zustand.inhalte,
        gegenstelle,
        &liste,
        || frei(zustand),
        gesamt,
    )
    .await
}

/// Was hier liegt und beim Server fehlt -- hinaufbringen.
///
/// **Die Gegenrichtung zu [`fehlende_holen`], und sie sieht anders aus.** Ein
/// Gerät in der Nähe holt sich selbst, was es braucht; beide Seiten laufen
/// gleich. Der Server holt nichts -- er wartet. Was dort fehlt, muss diese
/// Seite hinbringen, und zwar in Stücken über `POST /api/sync/uploads`.
///
/// **Vorschaubilder bleiben hier.** Der Server rechnet sie selbst
/// (`GenerateMediaThumbnail`). Sie hinaufzuschicken wäre doppelte Fracht für
/// ein Bild, das drüben ohnehin neu entsteht.
///
/// **Ein Vermerk erst nach dem Gelingen.** Bricht die Übertragung ab, gilt
/// der Inhalt weiter als fehlend und kommt beim nächsten Lauf wieder. Der
/// Server erkennt die angefangene Übertragung am Abdruck wieder und sagt beim
/// Anmelden, wie weit er ist -- es geht dort weiter, nicht von vorn.
pub async fn fehlende_hinauf(
    zustand: &Zustand,
    gegenstelle: &crate::servergegenstelle::ServerGegenstelle,
) -> openany_sync::InhalteBericht {
    use openany_client::{Hochzuladen, OpenanyError, STUECK_ZUM_SERVER};

    let mut bericht = openany_sync::InhalteBericht::default();
    let liste = match hier_und_nicht_dort(zustand, gegenstelle).await {
        Ok(l) => l,
        Err(e) => {
            bericht.fehler.push(e);
            return bericht;
        }
    };

    for s in liste {
        let ziel = Hochzuladen {
            art: &s.art,
            // Die Sache gibt es drüben schon -- der Läufer hat sie eben
            // hinübergebracht, und ohne einen durchgelaufenen Lauf kommen wir
            // hier gar nicht an.
            schluessel: Some(&s.uuid),
            // Ordner und Zone trotzdem dazu. Sollte die Sache drüben wider
            // Erwarten fehlen, legt der Server sie aus dieser Übertragung neu
            // an -- ohne die beiden Angaben landete sie in der Wurzel der
            // Akten, und zwar still.
            eltern: s.eltern.as_deref(),
            zone: s.zone.as_deref(),
            name: &s.name,
            groesse: s.groesse,
            abdruck: &s.abdruck,
            herkunft: None,
        };

        match hinaufbringen(zustand, gegenstelle, &ziel, STUECK_ZUM_SERVER).await {
            Ok(()) => {
                bericht.geholt += 1;
                let speicher = zustand.speicher.lock().await;
                if let Err(e) = speicher.inhalt_dort_merken(gegenstelle.basis_adresse(), &s.abdruck)
                {
                    bericht.fehler.push(e.to_string());
                }
            }
            // Kein Platz drüben: Weitermachen hieße, denselben Fehler für
            // jede weitere Datei zu erzeugen. Einmal sagen und aufhören.
            Err(e @ OpenanyError::KeinPlatz(_)) => {
                bericht.fehler.push(e.to_string());
                break;
            }
            Err(e) => bericht.fehler.push(e.to_string()),
        }
    }

    bericht
}

/// Eine Datei in Stücken hinüberbringen.
async fn hinaufbringen(
    zustand: &Zustand,
    gegenstelle: &crate::servergegenstelle::ServerGegenstelle,
    ziel: &openany_client::Hochzuladen<'_>,
    stueck: u64,
) -> Result<(), openany_client::OpenanyError> {
    use openany_client::OpenanyError;

    let client = gegenstelle.client();
    let uebertragung = client.upload_beginnen(ziel).await?;

    // Nicht bei 0 anfangen, sondern da, wo der Server steht: Bei einer
    // Wiederaufnahme ist das mitten in der Datei.
    let mut versatz = uebertragung.angekommen;

    while versatz < ziel.groesse {
        let bytes = zustand
            .inhalte
            .lesen(ziel.abdruck, versatz, stueck as usize)
            .map_err(|e| OpenanyError::Unlesbar(e.to_string()))?;

        if bytes.is_empty() {
            // Die Ablage hat weniger, als die Zeile behauptet. Die
            // angefangene Übertragung wegräumen, sonst hält sie drüben Platz
            // besetzt, bis sie von selbst abläuft.
            let _ = client.upload_aufgeben(&uebertragung.token).await;

            return Err(OpenanyError::Unlesbar(
                "The content is shorter than announced.".into(),
            ));
        }

        // Was der Server sagt, nicht was wir gerechnet haben: Ein wiederholtes
        // Stück bestätigt er, ohne es doppelt zu zählen.
        versatz = client
            .stueck_schicken(&uebertragung.token, versatz, bytes)
            .await?;
    }

    client.upload_abschliessen(&uebertragung.token).await?;

    Ok(())
}

/// Ein Inhalt, der hinaufgebracht werden will -- alles, was das Anmelden
/// einer Übertragung braucht.
struct Hinauf {
    abdruck: String,
    /// `file_node` oder `media`.
    art: String,
    uuid: String,
    name: String,
    groesse: u64,
    /// Ordner- bzw. Album-uuid.
    eltern: Option<String>,
    /// Nur bei Dateien.
    zone: Option<String>,
}

/// Inhalte, die hier vollständig liegen und drüben noch fehlen.
async fn hier_und_nicht_dort(
    zustand: &Zustand,
    gegenstelle: &crate::servergegenstelle::ServerGegenstelle,
) -> Result<Vec<Hinauf>, String> {
    let speicher = zustand.speicher.lock().await;
    let mut liste = Vec::new();

    for d in speicher.lebendige_dateien().map_err(fehler)? {
        if let Some(a) = d.abdruck.filter(|a| zustand.inhalte.hat(a)) {
            liste.push(Hinauf {
                abdruck: a,
                art: "file_node".into(),
                uuid: d.uuid,
                name: d.name,
                groesse: d.groesse,
                eltern: d.eltern,
                zone: Some(d.zone),
            });
        }
    }

    for b in speicher.lebendige_bilder().map_err(fehler)? {
        if let Some(a) = b.abdruck.filter(|a| zustand.inhalte.hat(a)) {
            liste.push(Hinauf {
                abdruck: a,
                art: "media".into(),
                uuid: b.uuid,
                name: b.name,
                groesse: b.groesse,
                eltern: b.album,
                zone: None,
            });
        }
    }

    // Die Gegenstelle traegt die Buchfuehrung schon mit sich -- derselbe
    // Stand, den auch der Wegweiser benutzt. Sie hier ein zweites Mal zu
    // lesen waere eine zweite Wahrheit ueber dieselbe Frage.
    Ok(liste
        .into_iter()
        .filter(|h| !gegenstelle.liegt_dort(&h.abdruck))
        .collect())
}

/// Die gepaarten Geräte als Gegenstellen -- die gerade in der Nähe sind zuerst.
async fn gegenstellen(zustand: &Zustand) -> Vec<openany_nahbereich::NahGegenstelle> {
    let Ok(ident) = crate::ident_laden(zustand).await else {
        return Vec::new();
    };
    let mut liste = Vec::new();
    for (fp, name) in zustand.gastgeber.gepaarte() {
        // Auch kurz nach dem Start, wenn noch niemand gefunden wurde: dann
        // wird gesucht und kurz gewartet (`adresse_zum_abgleichen`).
        if let Ok(adresse) = crate::adresse_zum_abgleichen(zustand, &fp).await {
            if let Ok(g) = openany_nahbereich::NahGegenstelle::neu(
                &ident,
                &adresse,
                openany_nahbereich::DIENST_PORT,
                &fp,
                &name,
            ) {
                liste.push(g);
            }
        }
    }
    liste
}

/// Alle, die einen Inhalt haben könnten: Geräte in der Nähe UND der Server.
///
/// **DIE REIHENFOLGE IST DIE AUSSAGE.** Ein Gerät im selben WLAN antwortet
/// schneller und kostet kein Mobilfunkvolumen; der Server ist der Rückfall,
/// der immer da ist. Deshalb Nachbarn zuerst.
///
/// Bis zum 16.09.2026 gab es den zweiten Teil nicht -- und zwar nicht als
/// Entscheidung, sondern als Typ: `Vec<NahGegenstelle>` konnte den Server gar
/// nicht enthalten. Wer eine Datei öffnete, deren Inhalt nur dort liegt, las
/// „keines ist gerade erreichbar", während das Gerät gekoppelt und online war.
///
/// OFFEN UND BEWUSST NICHT GEBAUT: nachfragen, bevor über Mobilfunk etwas
/// Großes geholt wird. Im WLAN wäre die Frage nur im Weg; unterwegs wäre sie
/// nötig. Solange es die Unterscheidung nicht gibt, holt dieses Programm
/// wortlos.
async fn alle_gegenstellen(zustand: &Arc<Zustand>) -> Vec<Box<dyn openany_sync::Gegenstelle>> {
    let mut liste: Vec<Box<dyn openany_sync::Gegenstelle>> = gegenstellen(zustand)
        .await
        .into_iter()
        .map(|g| Box::new(g) as Box<dyn openany_sync::Gegenstelle>)
        .collect();

    if let Some(server) = crate::server_gegenstelle(zustand).await {
        liste.push(Box::new(server));
    }

    liste
}

/// Den Inhalt einer Datei von einem gepaarten Gerät holen -- beim Öffnen,
/// wenn er hier fehlt („bei Bedarf").
#[tauri::command]
pub async fn datei_holen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: String,
) -> Result<(), String> {
    let d = {
        let speicher = zustand.speicher.lock().await;
        lebendig(&speicher, &id)?
    };
    let abdruck = d.abdruck.ok_or("A folder has no content.")?;
    von_geraeten_holen(&zustand, &abdruck, d.groesse).await
}

/// Einen Inhalt von irgendeinem erreichbaren gepaarten Gerät holen, das ihn hat.
pub async fn von_geraeten_holen(
    zustand: &Arc<Zustand>,
    abdruck: &str,
    groesse: u64,
) -> Result<(), String> {
    if zustand.inhalte.hat(abdruck) {
        return Ok(());
    }
    if frei(zustand) < groesse.saturating_add(openany_sync::reserve(gesamt(zustand))) {
        return Err(
            "Not enough storage on this device. You can free up space under Settings → Storage."
                .into(),
        );
    }

    let gegenueber = alle_gegenstellen(zustand).await;
    if gegenueber.is_empty() {
        return Err("The content is on another device, and none is reachable right now.".into());
    }
    let gesucht = [abdruck.to_string()];
    let mut letzter = String::new();
    for g in &gegenueber {
        match g.inhalte_da(&gesucht).await {
            Ok(da) if da.iter().any(|a| a == abdruck) => {}
            Ok(_) => continue,
            Err(e) => {
                letzter = format!("{}: {e}", g.name());
                continue;
            }
        }
        match openany_sync::inhalt_holen(&zustand.inhalte, &**g, abdruck, groesse).await {
            Ok(()) => return Ok(()),
            Err(e) => letzter = format!("{}: {e}", g.name()),
        }
    }
    Err(if letzter.is_empty() {
        "No reachable device has this content.".into()
    } else {
        letzter
    })
}

#[derive(Serialize)]
pub struct SpeicherLage {
    /// Was die Inhalte hier belegen.
    belegt: u64,
    frei: u64,
    gesamt: u64,
    reserve: u64,
    /// `alles`, `ausgewaehlt` oder `bei_bedarf`.
    regel: &'static str,
    /// Inhalte, die hier (noch) fehlen.
    fehlend: usize,
    fehlend_bytes: u64,
    /// Die markierten Ordner und Alben.
    behalten: Vec<Behalten>,
}

#[derive(Serialize)]
pub struct Behalten {
    id: String,
    name: String,
    /// `ordner` oder `album`.
    art: &'static str,
}

#[tauri::command]
pub async fn speicher_lage(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<SpeicherLage, String> {
    let liste = fehlend(&zustand, Originale::Alle).await?;
    let gesamt = gesamt(&zustand);
    let (regel, markiert) = {
        let e = zustand.einstellungen.lock().await;
        (e.regel(), e.behalten.clone())
    };
    let behalten = {
        let speicher = zustand.speicher.lock().await;
        markiert
            .into_iter()
            .filter_map(|id| {
                if let Ok(Some(d)) = speicher.datei(&id) {
                    return Some(Behalten {
                        id,
                        name: d.name,
                        art: "ordner",
                    });
                }
                if let Ok(Some(a)) = speicher.album(&id) {
                    return Some(Behalten {
                        id,
                        name: a.name,
                        art: "album",
                    });
                }
                None
            })
            .collect()
    };
    Ok(SpeicherLage {
        belegt: zustand.inhalte.belegt(),
        frei: frei(&zustand),
        gesamt,
        reserve: openany_sync::reserve(gesamt),
        regel,
        behalten,
        fehlend: liste.len(),
        fehlend_bytes: liste.iter().map(|(_, g)| g).sum(),
    })
}

#[tauri::command]
pub async fn speicher_regel_setzen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    regel: String,
) -> Result<(), String> {
    let mut e = zustand.einstellungen.lock().await;
    e.inhalte_regel = match regel.as_str() {
        "alles" | "ausgewaehlt" => regel,
        _ => "bei_bedarf".into(),
    };
    e.schreiben(&zustand.einstellungspfad()).map_err(fehler)?;
    drop(e);
    nachholen_im_hintergrund(zustand.inner().clone());
    Ok(())
}

/// Einen Ordner oder ein Album auf diesem Gerät behalten (oder nicht mehr).
/// Markiert heißt zugleich: Die Regel wird „ausgewählt", wenn sie „bei
/// Bedarf" war -- sonst hätte die Markierung keine Wirkung.
#[tauri::command]
pub async fn behalten_setzen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: String,
    an: bool,
) -> Result<(), String> {
    let mut e = zustand.einstellungen.lock().await;
    e.behalten.retain(|b| *b != id);
    if an {
        e.behalten.push(id);
        if e.regel() == "bei_bedarf" {
            e.inhalte_regel = "ausgewaehlt".into();
        }
    }
    e.schreiben(&zustand.einstellungspfad()).map_err(fehler)?;
    drop(e);
    if an {
        nachholen_im_hintergrund(zustand.inner().clone());
    }
    Ok(())
}

/// Nach einer neuen Markierung oder Regel: von jedem erreichbaren Gerät holen,
/// was jetzt fehlt -- im Hintergrund, die Oberfläche wartet nicht darauf.
pub(crate) fn nachholen_im_hintergrund(zustand: Arc<Zustand>) {
    tauri::async_runtime::spawn(async move {
        for g in alle_gegenstellen(&zustand).await {
            let _ = fehlende_holen(&zustand, &*g).await;
        }
    });
}

#[derive(Serialize)]
pub struct Freigegeben {
    entfernt: usize,
    bytes: u64,
    /// Liegen nur hier -- bleiben.
    einzig_hier: usize,
}

/// Platz freigeben: Inhalte löschen, die ein erreichbares gepaartes Gerät
/// nachweislich hat. NIE die letzte Kopie -- was kein anderes Gerät bestätigt,
/// bleibt.
#[tauri::command]
pub async fn platz_freigeben(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Freigegeben, String> {
    let hier: Vec<(String, u64)> = {
        let speicher = zustand.speicher.lock().await;
        let mut gesehen = std::collections::BTreeSet::new();
        speicher
            .lebendige_dateien()
            .map_err(fehler)?
            .into_iter()
            .filter_map(|d| Some((d.abdruck?, d.groesse)))
            .chain(
                speicher
                    .lebendige_bilder()
                    .map_err(fehler)?
                    .into_iter()
                    .filter_map(|b| Some((b.abdruck?, b.groesse))),
            )
            .filter(|(a, _)| zustand.inhalte.hat(a) && gesehen.insert(a.clone()))
            .collect()
    };
    // Was ausdrücklich behalten werden soll, bleibt -- auch wenn es woanders
    // liegt. Dazu die eigenen Freigaben in Projekten.
    let markiert = zustand.einstellungen.lock().await.alles_behalten();
    let geschuetzt: std::collections::BTreeSet<String> =
        fehlend_ausgewaehlt(&zustand, &markiert).await?;
    let hier: Vec<(String, u64)> = hier
        .into_iter()
        .filter(|(a, _)| !geschuetzt.contains(a))
        .collect();
    let abdruecke: Vec<String> = hier.iter().map(|(a, _)| a.clone()).collect();

    let mut woanders = std::collections::BTreeSet::new();
    for g in alle_gegenstellen(&zustand).await {
        for teil in abdruecke.chunks(500) {
            if let Ok(da) = g.inhalte_da(teil).await {
                woanders.extend(da);
            }
        }
    }
    if woanders.is_empty() && !hier.is_empty() {
        return Err(
            "No paired device is reachable -- nothing is deleted without confirmation.".into(),
        );
    }

    let mut frei = Freigegeben {
        entfernt: 0,
        bytes: 0,
        einzig_hier: 0,
    };
    for (abdruck, groesse) in hier {
        if woanders.contains(&abdruck) {
            zustand.inhalte.entfernen(&abdruck).map_err(fehler)?;
            frei.entfernt += 1;
            frei.bytes += groesse;
        } else {
            frei.einzig_hier += 1;
        }
    }
    Ok(frei)
}

#[cfg(test)]
mod tests {
    #[test]
    fn vorfassungs_name_setzt_den_zusatz_vor_die_endung() {
        use super::vorfassungs_name;
        assert_eq!(
            vorfassungs_name("Antrag.pdf", "vor Bearbeitung"),
            "Antrag (vor Bearbeitung).pdf"
        );
        assert_eq!(
            vorfassungs_name("Antrag", "vor Bearbeitung"),
            "Antrag (vor Bearbeitung)"
        );
        assert_eq!(vorfassungs_name(".pdf", "x"), ".pdf (x)");
    }

    use super::*;

    #[test]
    fn groesse_wie_drueben() {
        let d = |g| Datei {
            groesse: g,
            ..Default::default()
        };
        assert_eq!(groesse_text(&d(0)), "");
        assert_eq!(groesse_text(&d(512)), "512 B");
        assert_eq!(groesse_text(&d(1536)), "1.5 KB");
        assert_eq!(groesse_text(&d(33_594_508)), "32.04 MB");
    }

    #[test]
    fn namen_ohne_verbotene_zeichen() {
        assert_eq!(name_saeubern(" a/b:c ").unwrap(), "abc");
        assert!(name_saeubern("  ").is_err());
        assert!(name_saeubern("..").is_err());
    }

    #[test]
    fn die_suche_ignoriert_gross_und_klein_auch_bei_umlauten() {
        assert!(name_passt("Ärztliche Bescheinigung.pdf", "ärzt"));
        assert!(name_passt("05 HGB 02-2026.pdf", "hgb"));
        assert!(!name_passt("Rechnung.pdf", "hgb"));
    }

    #[test]
    fn der_ausschnitt_steht_um_die_fundstelle() {
        let text = format!(
            "{}Die Kündigungsfrist beträgt\ndrei Monate.",
            "x ".repeat(50)
        );
        let a = ausschnitt(&text, "kündigungsfrist").unwrap();
        assert!(a.starts_with('…'));
        assert!(a.contains("Die Kündigungsfrist beträgt drei Monate."));
        assert_eq!(ausschnitt("kurz", "kurz").as_deref(), Some("kurz"));
        assert!(ausschnitt("nichts", "anderes").is_none());
    }
}
