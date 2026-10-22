//! Geteilte Alben und Ordner Ende-zu-Ende (docs/plan-dateien-e2e.md, 7c-2).
//!
//! **Teilen** (Besitzer): eine Freigabe der Art `album` oder `ordner` --
//! derselbe Weg wie bei Mappen ([`mappenteilen::teilen_in`]), im Kopf die
//! uuid des Albums bzw. Ordners. Danach gehen alle Koepfe darin noch einmal
//! hinaus, jetzt mit dem Freigabeschluessel; die Bytes bleiben.
//!
//! **Ansehen** (Mitglieder, und der Besitzer im Projekt): Die Koepfe kommen
//! ueber das Projekt (`…/sachen`) und gehen mit dem Freigabeschluessel aus
//! der Kiste des Projekts auf. Bytes und Vorschauen holt die App auf
//! Verlangen, prueft und entschluesselt sie und legt sie in die eigene
//! Ablage (nach Klartext-Abdruck) -- von dort zeigt die Oberflaeche sie wie
//! jede andere Datei. Die Schluessel der Inhalte bleiben hier im Speicher,
//! nicht in der Oberflaeche.
//!
//! **Bilder in geteilten Mappen** gehen denselben Weg: Zuordnung und Bild
//! sind mit dem Mappenschluessel verschlossen ([`projekt_mappe_anhaenge`]).

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

use openany_client::{Art, Eintrag, OpenanyClient, Was};
use openany_direktpost::{dateischloss as schloss, Verschluesselt};
use openany_store::Inhaltsschluessel;
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::dateischloss;
use crate::mappenteilen::{self, Kopf};
use crate::notizschloss;
use crate::projektchat::{self, Verzeichnisse};
use crate::Zustand;

fn text(e: impl ToString) -> String {
    e.to_string()
}

fn hex(d: impl AsRef<[u8]>) -> String {
    d.as_ref().iter().map(|b| format!("{b:02x}")).collect()
}

/// So viel je Anfrage beim Holen.
const STUECK: u64 = 8 * 1024 * 1024;

/// Was zu einer geoeffneten Sache gehoert, um ihre Bytes zu holen.
#[derive(Clone)]
struct Offen {
    /// `media` oder `file_node`.
    art: &'static str,
    inhalt: Option<Inhaltsschluessel>,
    vorschau: Option<Inhaltsschluessel>,
}

/// `freigabe|uuid` → Schluessel. Nur im Speicher, nur fuer diese Sitzung.
fn offen() -> &'static Mutex<HashMap<String, Offen>> {
    static OFFEN: OnceLock<Mutex<HashMap<String, Offen>>> = OnceLock::new();
    OFFEN.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Raeumt eine Zwischendatei weg, wie auch immer es ausgeht.
struct Weg<'a>(&'a Path);
impl Drop for Weg<'_> {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(self.0);
    }
}

/* ── Teilen ────────────────────────────────────────────────────────────── */

/// Ein eigenes Album oder einen eigenen Ordner (auch eine Akte) im Projekt
/// teilen -- nur ansehen (Tiffy, 20.10.2026).
#[tauri::command]
pub async fn projekt_sache_teilen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    art: String,
    ziel: String,
) -> Result<String, String> {
    let client = projektchat::verbunden(&zustand).await?;
    let name = {
        let speicher = zustand.speicher.lock().await;
        match art.as_str() {
            "album" => speicher.album(&ziel).map_err(text)?.map(|a| a.name),
            "ordner" => speicher
                .datei(&ziel)
                .map_err(text)?
                .filter(|d| d.ist_ordner)
                .map(|d| d.name),
            _ => return Err("Unknown kind.".into()),
        }
        .ok_or("This album or folder no longer exists.")?
    };
    let kopf = Kopf {
        v: 1,
        pfad: String::new(),
        name,
        wurzel: Some(ziel.clone()),
    };
    let f = mappenteilen::teilen_in(&zustand, &client, &projekt, &art, kopf, "view").await?;

    // Alles darin noch einmal hinaus -- jetzt mit dem Freigabeschluessel.
    {
        let speicher = zustand.speicher.lock().await;
        let karte = mappenteilen::dateifreigaben(&speicher, std::slice::from_ref(&f));
        for uuid in karte.keys() {
            let art = if art == "ordner" {
                Some(Art::Datei)
            } else if speicher.album(uuid).ok().flatten().is_some() {
                Some(Art::Album)
            } else if speicher.bild(uuid).ok().flatten().is_some() {
                Some(Art::Medium)
            } else {
                None
            };
            if let Some(a) = art {
                speicher.merken(&a, uuid, Was::Da).map_err(text)?;
            }
        }
    }
    let _ = crate::server_abgleich(&zustand, false).await;
    Ok(f.uuid)
}

/* ── Ansehen ───────────────────────────────────────────────────────────── */

fn chiffrat(v: &Value) -> Option<Verschluesselt> {
    Some(Verschluesselt {
        inhalt: v.get("inhalt")?.as_str()?.to_string(),
        signatur: v.get("signatur")?.as_str()?.to_string(),
    })
}

/// Ein Kopf aus `…/sachen` als Eintrag, wie ihn [`dateischloss::kopf_auf`] erwartet.
fn als_eintrag(art: Art, v: &Value) -> Option<Eintrag> {
    let mut e = Eintrag::neu(art, v.get("uuid")?.as_str()?.to_string(), Was::Da);
    for feld in [
        "inhalt",
        "signatur",
        "geraet",
        "version",
        "updated_at",
        "thumb_hash",
    ] {
        if let Some(w) = v.get(feld).filter(|w| !w.is_null()) {
            e = e.mit(feld, w.clone());
        }
    }
    Some(e)
}

/// Was in einer Freigabe liegt, geoeffnet: Klartext-Eintraege und Anhang-
/// Zuordnungen. Die Inhaltsschluessel landen in [`offen`].
struct Geoeffnet {
    kopf: Kopf,
    stufe: Value,
    alben: Vec<Eintrag>,
    dateien: Vec<Eintrag>,
    bilder: Vec<Eintrag>,
    anhaenge: Vec<Eintrag>,
}

async fn oeffnen(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    projekt: &str,
    freigabe: &str,
) -> Result<Geoeffnet, String> {
    let (m, s, kopf) = mappenteilen::ueber_projekt(zustand, client, projekt, freigabe).await?;
    let besitzer = m
        .get("besitzer_name")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let roh = client
        .projekt_json(
            &format!("/projects/{projekt}/mappen/{freigabe}/sachen"),
            &[],
        )
        .await
        .map_err(text)?;
    let mut verz = Verzeichnisse::neu(client);
    let liste = |feld: &str| -> Vec<Value> {
        roh.get(feld)
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    };

    let mut aus = Geoeffnet {
        kopf,
        stufe: m.get("stufe").cloned().unwrap_or(Value::Null),
        alben: Vec::new(),
        dateien: Vec::new(),
        bilder: Vec::new(),
        anhaenge: Vec::new(),
    };
    let mut schluessel = Vec::new();
    for (feld, art) in [
        ("alben", Art::Album),
        ("dateien", Art::Datei),
        ("bilder", Art::Medium),
    ] {
        for v in liste(feld) {
            let Some(e) = als_eintrag(art.clone(), &v) else {
                continue;
            };
            let Some(geraet) = e.felder.get("geraet").and_then(Value::as_i64) else {
                continue;
            };
            let Some(ed) = verz.ed25519(&besitzer, geraet).await else {
                continue;
            };
            let Some(g) = dateischloss::kopf_auf(&s, &ed, &e) else {
                continue;
            };
            if g.inhalt.is_some() {
                schluessel.push((
                    g.eintrag.key.clone(),
                    Offen {
                        art: if art == Art::Medium {
                            "media"
                        } else {
                            "file_node"
                        },
                        inhalt: g.inhalt,
                        vorschau: g.vorschau,
                    },
                ));
            }
            match art {
                Art::Album => aus.alben.push(g.eintrag),
                Art::Datei => aus.dateien.push(g.eintrag),
                _ => aus.bilder.push(g.eintrag),
            }
        }
    }
    for v in liste("anhaenge") {
        let (Some(verdeckt), Some(geraet), Some(version), Some(c)) = (
            v.get("schluessel").and_then(Value::as_str),
            v.get("geraet").and_then(Value::as_i64),
            v.get("version").and_then(Value::as_u64),
            chiffrat(&v),
        ) else {
            continue;
        };
        let Some(ed) = verz.ed25519(&besitzer, geraet).await else {
            continue;
        };
        aus.anhaenge.extend(notizschloss::angabe_auf(
            &s,
            &ed,
            &Art::Notizanhang,
            verdeckt,
            version as u32,
            &c,
        ));
    }
    let mut o = offen().lock().expect("offen");
    for (uuid, k) in schluessel {
        o.insert(format!("{freigabe}|{uuid}"), k);
    }
    Ok(aus)
}

/// Liegt der Inhalt mit diesem Abdruck schon hier? Dann sein Pfad.
fn liegt_hier(zustand: &Zustand, abdruck: &str) -> Option<String> {
    zustand
        .inhalte
        .pfad(abdruck)
        .filter(|p| p.is_file())
        .map(|p| p.to_string_lossy().to_string())
}

fn wert(e: &Eintrag, feld: &str) -> Value {
    e.felder.get(feld).cloned().unwrap_or(Value::Null)
}

/// Was in einer geteilten Freigabe liegt -- Alben, Ordner, Dateien, Bilder,
/// im Klartext, mit dem Pfad, wo Original bzw. Vorschau schon hier liegen.
#[tauri::command]
pub async fn projekt_freigabe_sachen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    freigabe: String,
) -> Result<Value, String> {
    let client = projektchat::verbunden(&zustand).await?;
    let g = oeffnen(&zustand, &client, &projekt, &freigabe).await?;
    let pfad_von = |e: &Eintrag, feld: &str| e.text(feld).and_then(|a| liegt_hier(&zustand, a));
    Ok(json!({
        "name": g.kopf.name,
        "wurzel": g.kopf.wurzel,
        "stufe": g.stufe,
        "alben": g.alben.iter().map(|a| json!({
            "uuid": a.key, "name": wert(a, "name"), "eltern": wert(a, "parent"),
        })).collect::<Vec<_>>(),
        "dateien": g.dateien.iter().map(|d| json!({
            "uuid": d.key, "name": wert(d, "name"), "eltern": wert(d, "parent"),
            "ordner": d.felder.get("is_folder").and_then(Value::as_bool).unwrap_or(false),
            "zone": wert(d, "zone"), "groesse": wert(d, "size"), "mime": wert(d, "mime_type"),
            "pfad": pfad_von(d, "hash"),
        })).collect::<Vec<_>>(),
        "bilder": g.bilder.iter().map(|b| json!({
            "uuid": b.key, "name": wert(b, "name"), "album": wert(b, "album"),
            "groesse": wert(b, "size"), "mime": wert(b, "mime_type"),
            "pfad": pfad_von(b, "hash"),
            "vorschau_pfad": pfad_von(b, "thumb_hash"),
            "hat_vorschau": b.text("thumb_hash").is_some(),
        })).collect::<Vec<_>>(),
    }))
}

/// Verschluesselte Bytes einer Sache holen, pruefen, entschluesseln und in
/// die Ablage legen. Gibt den Pfad dort zurueck.
async fn holen(
    zustand: &Zustand,
    client: &OpenanyClient,
    projekt: &str,
    freigabe: &str,
    uuid: &str,
    o: &Offen,
    vorschau: bool,
) -> Result<String, String> {
    let k = if vorschau { &o.vorschau } else { &o.inhalt }
        .as_ref()
        .ok_or("There is nothing to load for this item.")?;
    if let Some(p) = liegt_hier(zustand, &k.abdruck) {
        return Ok(p);
    }
    let art = if vorschau { "media_vorschau" } else { o.art };
    let chiffrat = zustand.inhalte.zwischenpfad().map_err(text)?;
    let _weg = Weg(&chiffrat);
    {
        use std::io::Write;
        let mut f = std::fs::File::create(&chiffrat).map_err(text)?;
        let mut von = 0u64;
        loop {
            let stueck = client
                .projekt_bereich(
                    &format!("/projects/{projekt}/mappen/{freigabe}/inhalt"),
                    &[("type", art.to_string()), ("key", uuid.to_string())],
                    von,
                    STUECK,
                )
                .await
                .map_err(text)?;
            if stueck.is_empty() {
                break;
            }
            f.write_all(&stueck).map_err(text)?;
            von += stueck.len() as u64;
            if (stueck.len() as u64) < STUECK || (k.groesse > 0 && von >= k.groesse) {
                break;
            }
        }
        f.sync_all().map_err(text)?;
    }
    let klar = zustand.inhalte.zwischenpfad().map_err(text)?;
    let _weg2 = Weg(&klar);
    let abdruck = if vorschau {
        let c = std::fs::read(&chiffrat).map_err(text)?;
        if hex(Sha256::digest(&c)) != k.chiffrat {
            return Err("The preview does not match its header.".into());
        }
        let bytes = schloss::bytes_oeffnen(&k.schluessel, &c).map_err(text)?;
        std::fs::write(&klar, &bytes).map_err(text)?;
        hex(Sha256::digest(&bytes))
    } else {
        schloss::datei_oeffnen(&chiffrat, &klar, &k.schluessel, Some(&k.chiffrat)).map_err(text)?
    };
    if abdruck != k.abdruck {
        return Err("The decrypted content does not match its header.".into());
    }
    zustand.inhalte.uebernehmen(&klar).map_err(text)?;
    liegt_hier(zustand, &k.abdruck).ok_or_else(|| "The content went missing.".into())
}

/// Original oder Vorschau einer Sache der Freigabe auf dieses Geraet holen
/// -- der Pfad, unter dem sie dann liegt.
#[tauri::command]
pub async fn projekt_freigabe_holen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    freigabe: String,
    uuid: String,
    vorschau: bool,
) -> Result<String, String> {
    let client = projektchat::verbunden(&zustand).await?;
    let gemerkt = offen()
        .lock()
        .expect("offen")
        .get(&format!("{freigabe}|{uuid}"))
        .cloned();
    let o = match gemerkt {
        Some(o) => o,
        None => {
            oeffnen(&zustand, &client, &projekt, &freigabe).await?;
            offen()
                .lock()
                .expect("offen")
                .get(&format!("{freigabe}|{uuid}"))
                .cloned()
                .ok_or("This item is not shared here (anymore).")?
        }
    };
    holen(&zustand, &client, &projekt, &freigabe, &uuid, &o, vorschau).await
}

/// Das Original einer Sache der Freigabe als Bytes (Base64) -- fuer den
/// PDF-Betrachter und „Übernehmen".
#[tauri::command]
pub async fn projekt_freigabe_bytes(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    freigabe: String,
    uuid: String,
) -> Result<String, String> {
    use base64::Engine;
    let pfad = projekt_freigabe_holen(zustand, projekt, freigabe, uuid, false).await?;
    let bytes = std::fs::read(pfad).map_err(text)?;
    Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
}

/// Anhang in einer geteilten Mappe -- dieselbe Form wie `notizbuch_anhaenge`.
#[derive(Serialize)]
pub struct MappenAnhang {
    id: String,
    path: String,
    target_type: String,
    target_uuid: String,
    pfad: Option<String>,
}

/// Die Bilder einer geteilten Mappe (`mappe`: Pfad der Mappe beim
/// Besitzer). Die Vorschauen holt die App gleich mit -- der Editor fragt
/// beim Zeichnen und kann nicht warten.
#[tauri::command]
pub async fn projekt_mappe_anhaenge(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    freigabe: String,
    mappe: String,
) -> Result<Vec<MappenAnhang>, String> {
    let client = projektchat::verbunden(&zustand).await?;
    let g = oeffnen(&zustand, &client, &projekt, &freigabe).await?;
    let mut aus = Vec::new();
    for a in g.anhaenge {
        if a.text("folder") != Some(mappe.as_str()) {
            continue;
        }
        let ziel = a.text("target_uuid").unwrap_or_default().to_string();
        let gemerkt = offen()
            .lock()
            .expect("offen")
            .get(&format!("{freigabe}|{ziel}"))
            .cloned();
        let pfad = match gemerkt {
            // Vorschau zuerst (im Flusstext die richtige Groesse), sonst das Original.
            Some(o) => holen(
                &zustand,
                &client,
                &projekt,
                &freigabe,
                &ziel,
                &o,
                o.vorschau.is_some(),
            )
            .await
            .ok(),
            None => None,
        };
        aus.push(MappenAnhang {
            id: a.key.clone(),
            path: a.text("path").unwrap_or_default().to_string(),
            target_type: a.text("target_type").unwrap_or_default().to_string(),
            target_uuid: ziel,
            pfad,
        });
    }
    Ok(aus)
}
