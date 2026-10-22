//! Die Quellenliste einer Notiz-Mappe -- in der App (15.10.2026).
//!
//! Wie drueben (`QuellenListe.php`) ist sie KEIN eigener Speicher, sondern
//! eine CSL-JSON-Datei unter Dateien, die an der Mappe als Bibliothek haengt.
//! Die Datei reist als Datei, die Verknuepfung mit den Mappen-Angaben
//! (`note_folder`, mappenangaben.rs). So bleibt es eine Wahrheit -- fuer
//! Webapp, App, Zotero und Zettlr.
//!
//! **Vererbt.** Hat eine Mappe keine eigene Liste, gilt die der naechsten
//! Mappe darueber. Speichern aendert dann DEREN Datei, wie drueben.
//!
//! **Das letzte Schreiben gewinnt** (Tiffy, 15.10.2026) -- die Datei als
//! Ganzes, wie jede andere Datei im Abgleich.

use crate::dateibefehle::{freier_name, von_geraeten_holen};
use crate::Zustand;
use openany_store::{Datei, Mappenangaben, Protokoll, Speicher};
use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;

/// Groesste Liste, die hier geschrieben wird -- wie drueben.
const HOECHSTENS_BYTES: usize = 2 * 1024 * 1024;

fn fehler(e: impl ToString) -> String {
    e.to_string()
}

#[derive(Serialize)]
pub struct DateiKurz {
    id: String,
    name: String,
}

/// Dieselbe Form wie `GET note-folders/{id}/quellen` drueben.
#[derive(Serialize)]
pub struct Quellen {
    eintraege: Vec<Value>,
    datei: Option<DateiKurz>,
    geerbt: bool,
    bearbeitbar: bool,
}

#[derive(Serialize)]
pub struct Gespeichert {
    datei: DateiKurz,
}

fn mappenpfad(speicher: &Speicher, id: u32) -> Result<String, String> {
    speicher
        .mappen_pfad(id)
        .map_err(fehler)?
        .ok_or_else(|| "This folder no longer exists.".to_string())
}

/// Die Quellendatei der Mappe oder der naechsten darueber: (Datei, geerbt).
fn bibliothek(speicher: &Speicher, pfad: &str) -> Result<Option<(Datei, bool)>, String> {
    let mut aktuell = Some(pfad.to_string());
    while let Some(p) = aktuell {
        if let Some(uuid) = speicher
            .mappenangaben(&p)
            .map_err(fehler)?
            .and_then(|m| m.bibliothek)
        {
            if let Some(d) = speicher.datei(&uuid).map_err(fehler)? {
                if d.papierkorb_at.is_none() && !d.ist_ordner {
                    return Ok(Some((d, p != pfad)));
                }
            }
        }
        aktuell = p.rsplit_once('/').map(|(e, _)| e.to_string());
    }
    Ok(None)
}

#[tauri::command]
pub async fn notizbuch_quellen_lesen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: u32,
) -> Result<Quellen, String> {
    let gefunden = {
        let speicher = zustand.speicher.lock().await;
        let pfad = mappenpfad(&speicher, id)?;
        bibliothek(&speicher, &pfad)?
    };
    let Some((datei, geerbt)) = gefunden else {
        return Ok(Quellen {
            eintraege: vec![],
            datei: None,
            geerbt: false,
            bearbeitbar: true,
        });
    };

    let abdruck = datei.abdruck.clone().unwrap_or_default();
    // „Bei Bedarf": Fehlt der Inhalt hier, holen wie beim Oeffnen einer Datei.
    if !abdruck.is_empty() && !zustand.inhalte.hat(&abdruck) {
        von_geraeten_holen(&zustand, &abdruck, datei.groesse).await?;
    }
    let bytes = if abdruck.is_empty() {
        Vec::new()
    } else {
        zustand
            .inhalte
            .lesen(&abdruck, 0, HOECHSTENS_BYTES * 4)
            .map_err(fehler)?
    };
    let eintraege = match serde_json::from_slice::<Value>(&bytes) {
        Ok(Value::Array(liste)) => liste.into_iter().filter(Value::is_object).collect(),
        _ => vec![],
    };

    Ok(Quellen {
        eintraege,
        datei: Some(DateiKurz {
            id: datei.uuid,
            name: datei.name,
        }),
        geerbt,
        bearbeitbar: true,
    })
}

/// Wie drueben: was sich mit `[@…]` zitieren laesst.
fn schluessel_gueltig(id: &str) -> bool {
    !id.is_empty()
        && id.chars().count() <= 200
        && !id
            .chars()
            .any(|z| z.is_whitespace() || matches!(z, '[' | ']' | ';' | ',' | '@'))
}

#[tauri::command]
pub async fn notizbuch_quellen_speichern(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: u32,
    eintraege: Vec<Value>,
) -> Result<Gespeichert, String> {
    let mut gesehen = std::collections::HashSet::new();
    for e in &eintraege {
        let schluessel = e.get("id").and_then(Value::as_str).unwrap_or_default();
        if !schluessel_gueltig(schluessel) || !gesehen.insert(schluessel.to_string()) {
            return Err(format!("Invalid or duplicate short name: {schluessel}"));
        }
        if e.get("type")
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
        {
            return Err(format!("Missing type: {schluessel}"));
        }
    }
    let mut json = serde_json::to_vec_pretty(&eintraege).map_err(fehler)?;
    json.push(b'\n');
    if json.len() > HOECHSTENS_BYTES {
        return Err("The source list is too large.".into());
    }

    // Erst die Bytes ablegen, dann eintragen: Scheitert das Ablegen, bleibt
    // die alte Liste, wie sie war.
    let mut ladung = zustand.inhalte.ladung().map_err(fehler)?;
    ladung.schreiben(&json).map_err(fehler)?;
    let (abdruck, groesse) = zustand.inhalte.ablegen(ladung).map_err(fehler)?;

    let speicher = zustand.speicher.lock().await;
    let pfad = mappenpfad(&speicher, id)?;

    if let Some((mut d, _)) = bibliothek(&speicher, &pfad)? {
        let alt = d.abdruck.replace(abdruck);
        d.groesse = groesse;
        d.geaendert_at = String::new();
        speicher
            .datei_schreiben(&d, Protokoll::Merken)
            .map_err(fehler)?;
        if let Some(alt) = alt.filter(|a| Some(a) != d.abdruck.as_ref()) {
            if !speicher.abdruck_benutzt(&alt).map_err(fehler)? {
                let _ = zustand.inhalte.entfernen(&alt);
            }
        }
        return Ok(Gespeichert {
            datei: DateiKurz {
                id: d.uuid,
                name: d.name,
            },
        });
    }

    // Noch keine: anlegen wie drueben -- „Quellen – <Mappe>.json" im Ordner
    // „Notizen" unter Dateien -- und an der Mappe verknuepfen.
    let ordner = notizen_ordner(&speicher)?;
    let mappenname = pfad
        .rsplit('/')
        .next()
        .unwrap_or(&pfad)
        .replace(['/', '\\'], "");
    let name = freier_name(
        &speicher,
        "files",
        Some(&ordner),
        &format!("Quellen – {mappenname}.json"),
    )?;
    let datei = Datei {
        uuid: uuid::Uuid::new_v4().to_string(),
        zone: "files".into(),
        ist_ordner: false,
        eltern: Some(ordner),
        name,
        groesse,
        mime: "application/json".into(),
        abdruck: Some(abdruck),
        ..Default::default()
    };
    speicher
        .datei_schreiben(&datei, Protokoll::Merken)
        .map_err(fehler)?;

    let mut angaben = speicher
        .mappenangaben(&pfad)
        .map_err(fehler)?
        .unwrap_or(Mappenangaben {
            pfad: pfad.clone(),
            ..Default::default()
        });
    angaben.bibliothek = Some(datei.uuid.clone());
    angaben.geaendert_at = String::new();
    speicher
        .mappenangaben_schreiben(&angaben, Protokoll::Merken)
        .map_err(fehler)?;

    Ok(Gespeichert {
        datei: DateiKurz {
            id: datei.uuid,
            name: datei.name,
        },
    })
}

/// Der Ordner „Notizen" oben unter Dateien -- angelegt, wenn es ihn nicht gibt.
fn notizen_ordner(speicher: &Speicher) -> Result<String, String> {
    if let Some(o) = speicher
        .dateien_im_ordner("files", None)
        .map_err(fehler)?
        .into_iter()
        .find(|d| d.ist_ordner && d.papierkorb_at.is_none() && d.name == "Notizen")
    {
        return Ok(o.uuid);
    }
    let ordner = Datei {
        uuid: uuid::Uuid::new_v4().to_string(),
        zone: "files".into(),
        ist_ordner: true,
        eltern: None,
        name: "Notizen".into(),
        ..Default::default()
    };
    speicher
        .datei_schreiben(&ordner, Protokoll::Merken)
        .map_err(fehler)?;
    Ok(ordner.uuid)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn datei(s: &Speicher, uuid: &str) {
        s.datei_schreiben(
            &Datei {
                uuid: uuid.into(),
                zone: "files".into(),
                name: format!("{uuid}.json"),
                mime: "application/json".into(),
                abdruck: Some("a".repeat(64)),
                ..Default::default()
            },
            Protokoll::Still,
        )
        .unwrap();
    }

    fn verknuepfen(s: &Speicher, pfad: &str, uuid: &str) {
        s.mappenangaben_schreiben(
            &Mappenangaben {
                pfad: pfad.into(),
                bibliothek: Some(uuid.into()),
                ..Default::default()
            },
            Protokoll::Still,
        )
        .unwrap();
    }

    #[test]
    fn eine_untermappe_erbt_die_liste_darueber() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        datei(&s, "oben");
        verknuepfen(&s, "Uni", "oben");

        let (d, geerbt) = bibliothek(&s, "Uni/Arbeit/Kapitel").unwrap().unwrap();
        assert_eq!(d.uuid, "oben");
        assert!(geerbt);

        let (_, geerbt) = bibliothek(&s, "Uni").unwrap().unwrap();
        assert!(!geerbt);
        assert!(bibliothek(&s, "Anderes").unwrap().is_none());
    }

    #[test]
    fn die_eigene_liste_geht_vor_und_eine_weggelegte_zaehlt_nicht() {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        datei(&s, "oben");
        datei(&s, "eigen");
        verknuepfen(&s, "Uni", "oben");
        verknuepfen(&s, "Uni/Arbeit", "eigen");
        assert_eq!(
            bibliothek(&s, "Uni/Arbeit").unwrap().unwrap().0.uuid,
            "eigen"
        );

        s.datei_papierkorb("eigen", Protokoll::Still).unwrap();
        assert_eq!(
            bibliothek(&s, "Uni/Arbeit").unwrap().unwrap().0.uuid,
            "oben"
        );
    }

    #[test]
    fn kurznamen_wie_drueben() {
        assert!(schluessel_gueltig("mueller2020"));
        assert!(!schluessel_gueltig("mit leer"));
        assert!(!schluessel_gueltig("a;b"));
        assert!(!schluessel_gueltig("[x]"));
        assert!(!schluessel_gueltig(""));
    }
}
