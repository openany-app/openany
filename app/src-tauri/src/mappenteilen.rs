//! Geteilte Notiz-Mappen Ende-zu-Ende (docs/plan-notizen-e2e.md, 6c-2).
//!
//! **Der Mappenschluessel.** Eine geteilte Mappe hat einen eigenen
//! Schluessel (mit Fassungen, derselbe Typ wie Konto- und Projektschluessel).
//! Er liegt beim Server in Kisten: eine an den eigenen Kontoschluessel, eine
//! je Projekt an dessen Projektschluessel. Gespeichert wird er hier nicht --
//! die Kiste ist in Millisekunden geoeffnet, wie beim Projektschluessel.
//!
//! **Die Notizen der Mappe** (und ihrer Unterordner) sind damit
//! verschluesselt, als Projektfeld `notiz:<zk_id>` unter `mappe:<freigabe>`.
//! Der eigene Abgleich macht das an der Grenze zum Server
//! ([`crate::servergegenstelle`]); Mitglieder lesen und schreiben ueber die
//! Befehle hier. Ihr Abdruck ist mit dem Mappenschluessel verdeckt -- so
//! rechnen Besitzer und Mitglieder denselben.
//!
//! **Der Kopf** (Pfad und Name der Mappe beim Besitzer) ist ebenso
//! verschluesselt; Mitglieder brauchen den Pfad, um Unterordner zu zeigen.
//!
//! **Alben und Ordner** (docs/plan-dateien-e2e.md, 7c) gehen denselben Weg:
//! eine Freigabe der Art `album` oder `ordner`, im Kopf die uuid des Albums
//! bzw. Ordners (`wurzel`). Was darin liegt, reist mit dem
//! Freigabeschluessel ([`dateifreigaben`], [`crate::dateischloss`]).

use std::sync::Arc;

use openany_client::{Method, OpenanyClient};
use openany_direktpost::{Kontoschluessel, Schluessel, Verschluesselt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::direktpost;
use crate::projektchat::{self, Verzeichnisse};
use crate::Zustand;

fn text(e: impl ToString) -> String {
    e.to_string()
}

fn ort(freigabe: &str) -> String {
    format!("mappe:{freigabe}")
}

fn konto_kiste_ort(freigabe: &str) -> String {
    format!("mappe:{freigabe}:konto")
}

fn projekt_kiste_ort(freigabe: &str, projekt: &str) -> String {
    format!("mappe:{freigabe}:projekt:{projekt}")
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Kopf {
    pub v: u8,
    /// Der Pfad der Mappe beim Besitzer (bei Alben und Ordnern leer).
    pub pfad: String,
    pub name: String,
    /// Bei Alben und Ordnern: deren uuid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wurzel: Option<String>,
}

/// Was in einer Notiz der Mappe steht -- dieselben Felder wie bei einer
/// eigenen Notiz (notizschloss.rs).
#[derive(Serialize, Deserialize)]
pub(crate) struct NotizInhalt {
    pub v: u8,
    pub zk_id: String,
    pub title: String,
    #[serde(default)]
    pub folder: Option<String>,
    pub content: String,
}

/// Eine eigene Freigabe, geoeffnet.
#[derive(Clone)]
pub(crate) struct Freigabe {
    pub uuid: String,
    /// `mappe`, `album` oder `ordner`.
    pub art: String,
    pub pfad: String,
    /// Bei Alben und Ordnern: deren uuid.
    pub wurzel: Option<String>,
    pub schluessel: Kontoschluessel,
}

impl Freigabe {
    /// Gehoert eine Notiz mit dieser Mappe hierher (die Mappe selbst oder darunter)?
    pub fn umfasst(&self, mappe: Option<&str>) -> bool {
        self.art == "mappe"
            && mappe.is_some_and(|m| m == self.pfad || m.starts_with(&format!("{}/", self.pfad)))
    }
}

/// Welche Sache (Album, Ordner, Datei, Bild -- und Anhang-Zuordnung, unter
/// ihrem Schluessel `mappe|pfad`) in welcher eigenen Freigabe liegt.
/// **Die erste gewinnt:** eine Sache, ein Schluessel. Gerechnet VOR dem
/// Abgleich, der Speicher ist waehrenddessen gesperrt.
pub(crate) fn dateifreigaben(
    speicher: &openany_store::Speicher,
    freigaben: &[Freigabe],
) -> std::collections::HashMap<String, String> {
    let mut karte = std::collections::HashMap::new();
    let mut eintragen = |k: String, f: &Freigabe| {
        karte.entry(k).or_insert_with(|| f.uuid.clone());
    };
    for f in freigaben {
        match (f.art.as_str(), &f.wurzel) {
            ("album", Some(w)) => {
                let alben = speicher.album_mit_unteralben(w).unwrap_or_default();
                for b in speicher.lebendige_bilder().unwrap_or_default() {
                    if b.album.as_ref().is_some_and(|a| alben.contains(a)) {
                        eintragen(b.uuid, f);
                    }
                }
                for a in alben {
                    eintragen(a, f);
                }
            }
            ("ordner", Some(w)) => {
                for d in speicher.ordner_mit_inhalt(w).unwrap_or_default() {
                    eintragen(d, f);
                }
            }
            ("mappe", _) => {
                for a in speicher.anhaenge_alle().unwrap_or_default() {
                    if f.umfasst(Some(&a.mappe)) {
                        eintragen(a.ziel_uuid.clone(), f);
                        eintragen(a.schluessel(), f);
                    }
                }
            }
            _ => {}
        }
    }
    karte
}

/// Die Freigabe fuer eine Notiz-Mappe -- die naechstgelegene.
pub(crate) fn zustaendig<'a>(
    freigaben: &'a [Freigabe],
    mappe: Option<&str>,
) -> Option<&'a Freigabe> {
    freigaben
        .iter()
        .filter(|f| f.umfasst(mappe))
        .max_by_key(|f| f.pfad.len())
}

/// Eine Notiz verschluesseln (Besitzer und Mitglieder gleich).
pub(crate) fn notiz_zu(
    ich: &Schluessel,
    f_uuid: &str,
    schluessel: &Kontoschluessel,
    inhalt: &NotizInhalt,
) -> Result<(Verschluesselt, u32), String> {
    let roh = serde_json::to_string(inhalt).map_err(text)?;
    openany_direktpost::projektfeld_verschluesseln(
        ich,
        schluessel,
        &ort(f_uuid),
        &format!("notiz:{}", inhalt.zk_id),
        &roh,
    )
    .map_err(text)
}

/// Eine Notiz oeffnen -- `None`, wenn sie nicht aufgeht.
pub(crate) fn notiz_auf(
    f_uuid: &str,
    schluessel: &Kontoschluessel,
    zk_id: &str,
    version: u32,
    v: &Verschluesselt,
    ed25519: &str,
) -> Option<NotizInhalt> {
    let roh = openany_direktpost::projektfeld_oeffnen(
        schluessel,
        &ort(f_uuid),
        &format!("notiz:{zk_id}"),
        version,
        v,
        ed25519,
    )
    .ok()?;
    let n: NotizInhalt = serde_json::from_str(&roh).ok()?;
    (n.zk_id == zk_id).then_some(n)
}

fn kopf_zu(
    ich: &Schluessel,
    f_uuid: &str,
    schluessel: &Kontoschluessel,
    kopf: &Kopf,
) -> Result<Value, String> {
    let roh = serde_json::to_string(kopf).map_err(text)?;
    let (v, version) =
        openany_direktpost::projektfeld_verschluesseln(ich, schluessel, &ort(f_uuid), "kopf", &roh)
            .map_err(text)?;
    Ok(json!({ "inhalt": v.inhalt, "signatur": v.signatur, "version": version }))
}

fn chiffrat(v: &Value) -> Option<Verschluesselt> {
    Some(Verschluesselt {
        inhalt: v.get("inhalt")?.as_str()?.to_string(),
        signatur: v.get("signatur")?.as_str()?.to_string(),
    })
}

async fn kopf_auf(
    f: &Value,
    schluessel: &Kontoschluessel,
    verz: &mut Verzeichnisse<'_>,
) -> Option<Kopf> {
    let uuid = f.get("uuid")?.as_str()?;
    let besitzer = f.get("besitzer_name")?.as_str()?;
    let geraet = f.get("geraet")?.as_i64()?;
    let version = f.get("version")?.as_u64()? as u32;
    let ed = verz.ed25519(besitzer, geraet).await?;
    let roh = openany_direktpost::projektfeld_oeffnen(
        schluessel,
        &ort(uuid),
        "kopf",
        version,
        &chiffrat(f)?,
        &ed,
    )
    .ok()?;
    serde_json::from_str(&roh).ok()
}

/// Eine Kiste oeffnen (Konto- oder Projekt-Kiste), unterschrieben von einem
/// Geraet des Besitzers.
pub(crate) async fn kiste_auf(
    k: &Value,
    oeffner: &Kontoschluessel,
    wo: &str,
    besitzer: &str,
    verz: &mut Verzeichnisse<'_>,
) -> Option<Kontoschluessel> {
    let ed = verz
        .ed25519(besitzer, k.get("von_geraet")?.as_i64()?)
        .await?;
    openany_direktpost::projektkiste_oeffnen(
        oeffner,
        wo,
        0,
        k.get("version")?.as_u64()? as u32,
        k.get("chiffrat")?.as_str()?,
        k.get("signatur")?.as_str()?,
        &ed,
    )
    .ok()
}

/// Aus mehreren Kisten den Schluessel mit der hoechsten Fassung.
async fn bester(
    kisten: &[Value],
    oeffner: &Kontoschluessel,
    wo: &str,
    besitzer: &str,
    verz: &mut Verzeichnisse<'_>,
) -> Option<Kontoschluessel> {
    let mut beste: Option<Kontoschluessel> = None;
    for k in kisten {
        if let Some(s) = kiste_auf(k, oeffner, wo, besitzer, verz).await {
            if beste
                .as_ref()
                .is_none_or(|b| s.aktuell_fassung() > b.aktuell_fassung())
            {
                beste = Some(s);
            }
        }
    }
    beste
}

/// Die eigenen Freigaben, geoeffnet -- und nebenbei gepflegt: faelligen
/// Wechsel erledigen, fehlende Projekt-Kisten nachreichen.
///
/// **Ein Fehler beim Holen ist ein Fehler** (kein leeres Ergebnis): Ohne die
/// Freigaben gingen Notizen geteilter Mappen mit dem Kontoschluessel hinaus,
/// und die Mitglieder verloeren sie.
pub(crate) async fn eigene(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
) -> Result<Vec<Freigabe>, String> {
    let (Some(konto), Ok(ich), Some(name)) = (
        direktpost::konto(zustand),
        direktpost::schluessel(zustand),
        direktpost::mein_name(zustand),
    ) else {
        return Ok(Vec::new());
    };
    let antwort = client
        .projekt_json("/mappen-freigaben", &[])
        .await
        .map_err(text)?;
    let mut verz = Verzeichnisse::neu(client);
    let mut aus = Vec::new();
    for f in antwort
        .get("freigaben")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
    {
        let Some(uuid) = f.get("uuid").and_then(Value::as_str).map(str::to_string) else {
            continue;
        };
        let kisten = f
            .get("kisten")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let Some(mut schluessel) =
            bester(&kisten, &konto, &konto_kiste_ort(&uuid), &name, &mut verz).await
        else {
            continue;
        };
        let Some(kopf) = kopf_auf(&f, &schluessel, &mut verz).await else {
            continue;
        };
        let art = f
            .get("art")
            .and_then(Value::as_str)
            .unwrap_or("mappe")
            .to_string();
        let _ = pflegen(
            zustand,
            client,
            &ich,
            &konto,
            &uuid,
            &mut schluessel,
            &kopf,
            &f,
        )
        .await;
        aus.push(Freigabe {
            uuid,
            art,
            pfad: kopf.pfad,
            wurzel: kopf.wurzel,
            schluessel,
        });
    }
    Ok(aus)
}

/// Wechsel faellig: neue Fassung, Kisten fuer mich und alle Projekte, Kopf
/// neu. Fehlt einem Projekt die Kiste der gueltigen Fassung: nachreichen.
#[allow(clippy::too_many_arguments)]
async fn pflegen(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    ich: &Schluessel,
    konto: &Kontoschluessel,
    uuid: &str,
    schluessel: &mut Kontoschluessel,
    kopf: &Kopf,
    f: &Value,
) -> Result<(), String> {
    let wechsel = f.get("wechsel_faellig").and_then(Value::as_bool) == Some(true);
    let projekte: Vec<(String, bool)> = f
        .get("projekte")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|p| {
            Some((
                p.get("projekt")?.as_str()?.to_string(),
                p.get("kiste_fehlt")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            ))
        })
        .collect();
    if !wechsel && !projekte.iter().any(|(_, fehlt)| *fehlt) {
        return Ok(());
    }
    if wechsel {
        schluessel.wechseln();
    }
    let mut kisten = Vec::new();
    if wechsel {
        let (c, s) = openany_direktpost::projektkiste_packen(
            ich,
            schluessel,
            &konto_kiste_ort(uuid),
            0,
            &konto.oeffentlich(),
        )
        .map_err(text)?;
        kisten.push(json!({ "version": schluessel.aktuell_fassung(), "ziel_art": "konto", "chiffrat": c, "signatur": s }));
    }
    for (projekt, fehlt) in &projekte {
        if !wechsel && !fehlt {
            continue;
        }
        let Some(p) = projektchat::schluessel(zustand, client, projekt).await? else {
            continue;
        };
        let (c, s) = openany_direktpost::projektkiste_packen(
            ich,
            schluessel,
            &projekt_kiste_ort(uuid, projekt),
            0,
            &p.oeffentlich(),
        )
        .map_err(text)?;
        kisten.push(json!({ "version": schluessel.aktuell_fassung(), "ziel_art": "projekt", "projekt": projekt, "chiffrat": c, "signatur": s }));
    }
    let mut koerper = json!({ "kisten": kisten });
    if wechsel {
        let k = kopf_zu(ich, uuid, schluessel, kopf)?;
        for feld in ["inhalt", "signatur", "version"] {
            koerper[feld] = k[feld].clone();
        }
    }
    client
        .projekt_senden(
            Method::PUT,
            &format!("/mappen-freigaben/{uuid}"),
            Some(koerper),
        )
        .await
        .map_err(text)?;
    Ok(())
}

/* ── Befehle ───────────────────────────────────────────────────────────── */

/// Die geteilten Mappen eines Projekts -- Name, Stufe, von wem.
#[tauri::command]
pub async fn projekt_mappen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
) -> Result<Value, String> {
    let client = projektchat::verbunden(&zustand).await?;
    let liste = client
        .projekt_json(&format!("/projects/{projekt}/mappen"), &[])
        .await
        .map_err(text)?;
    let p = projektchat::schluessel(&zustand, &client, &projekt).await?;
    let mut verz = Verzeichnisse::neu(&client);
    let mut aus = Vec::new();
    for m in liste
        .get("mappen")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
    {
        let (Some(uuid), Some(besitzer)) = (
            m.get("uuid").and_then(Value::as_str),
            m.get("besitzer_name").and_then(Value::as_str),
        ) else {
            continue;
        };
        let kisten = m
            .get("kisten")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let kopf = match &p {
            Some(p) => match bester(
                &kisten,
                p,
                &projekt_kiste_ort(uuid, &projekt),
                besitzer,
                &mut verz,
            )
            .await
            {
                Some(s) => kopf_auf(&m, &s, &mut verz).await,
                None => None,
            },
            None => None,
        };
        aus.push(json!({
            "uuid": uuid,
            "name": kopf.as_ref().map(|k| k.name.clone()),
            "pfad": kopf.as_ref().map(|k| k.pfad.clone()),
            "stufe": m.get("stufe"),
            "eigen": m.get("eigen"),
            "von_name": besitzer,
            "art": m.get("art").cloned().unwrap_or_else(|| "mappe".into()),
            "anzahl": m.get("anzahl").or_else(|| m.get("notizen_anzahl")),
        }));
    }
    Ok(json!({ "mappen": aus }))
}

/// Eine eigene Sache im Projekt teilen: die vorhandene Freigabe dafuer
/// (gleiche Art, gleicher Pfad bzw. gleiche Wurzel) oder eine neue mit
/// frischem Schluessel, dazu die Kiste fuer das Projekt.
pub(crate) async fn teilen_in(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    projekt: &str,
    art: &str,
    kopf: Kopf,
    stufe: &str,
) -> Result<Freigabe, String> {
    let konto = direktpost::konto(zustand).ok_or("This device is not approved yet.")?;
    let ich = direktpost::schluessel(zustand)?;
    let eigene = eigene(zustand, client).await?;
    let gleich = |f: &&Freigabe| {
        f.art == art
            && if art == "mappe" {
                f.pfad == kopf.pfad
            } else {
                f.wurzel == kopf.wurzel
            }
    };
    let f = match eigene.iter().find(gleich) {
        Some(f) => f.clone(),
        None => {
            let uuid = uuid::Uuid::new_v4().to_string();
            let schluessel = Kontoschluessel::neu();
            let k = kopf_zu(&ich, &uuid, &schluessel, &kopf)?;
            let (c, s) = openany_direktpost::projektkiste_packen(
                &ich,
                &schluessel,
                &konto_kiste_ort(&uuid),
                0,
                &konto.oeffentlich(),
            )
            .map_err(text)?;
            client
                .projekt_senden(
                    Method::POST,
                    "/mappen-freigaben",
                    Some(json!({ "uuid": uuid, "art": art, "inhalt": k["inhalt"], "signatur": k["signatur"], "version": k["version"], "kiste": { "chiffrat": c, "signatur": s } })),
                )
                .await
                .map_err(text)?;
            Freigabe {
                uuid,
                art: art.to_string(),
                pfad: kopf.pfad,
                wurzel: kopf.wurzel,
                schluessel,
            }
        }
    };

    let p = projektchat::schluessel(zustand, client, projekt)
        .await?
        .ok_or("This device has no key for this project yet.")?;
    let (c, s) = openany_direktpost::projektkiste_packen(
        &ich,
        &f.schluessel,
        &projekt_kiste_ort(&f.uuid, projekt),
        0,
        &p.oeffentlich(),
    )
    .map_err(text)?;
    client
        .projekt_senden(
            Method::POST,
            &format!("/projects/{projekt}/mappen"),
            Some(json!({ "freigabe": f.uuid, "stufe": stufe, "kiste": { "version": f.schluessel.aktuell_fassung(), "chiffrat": c, "signatur": s } })),
        )
        .await
        .map_err(text)?;
    Ok(f)
}

/// Eine eigene Mappe im Projekt teilen. Gibt es fuer sie noch keine
/// Freigabe, entsteht sie hier (neuer Mappenschluessel); ihre Notizen und
/// die Bilder darin gehen beim naechsten Abgleich mit dem Mappenschluessel
/// hinaus.
#[tauri::command]
pub async fn projekt_mappe_teilen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    mappe: u32,
    stufe: String,
) -> Result<String, String> {
    if !matches!(stufe.as_str(), "view" | "edit") {
        return Err("Unknown level.".into());
    }
    let client = projektchat::verbunden(&zustand).await?;
    let (pfad, name) = {
        let speicher = zustand.speicher.lock().await;
        let m = speicher
            .mappen()
            .map_err(text)?
            .into_iter()
            .find(|m| m.id == mappe)
            .ok_or("This folder no longer exists.")?;
        (m.pfad, m.name)
    };
    let kopf = Kopf {
        v: 1,
        pfad,
        name,
        wurzel: None,
    };
    let f = teilen_in(&zustand, &client, &projekt, "mappe", kopf, &stufe).await?;

    // Die Notizen der Mappe noch einmal hinaus -- jetzt mit dem
    // Mappenschluessel. Ebenso ihre Bilder (7c): Zuordnung und Bild.
    {
        use openany_client::{Art, Was};
        let speicher = zustand.speicher.lock().await;
        for n in speicher.notizen().map_err(text)? {
            if n.papierkorb_at.is_none() && f.umfasst(n.mappe.as_deref()) {
                speicher
                    .merken(&Art::Notiz, &n.zk_id, Was::Da)
                    .map_err(text)?;
            }
        }
        for a in speicher.anhaenge_alle().map_err(text)? {
            if f.umfasst(Some(&a.mappe)) {
                speicher
                    .merken(&Art::Notizanhang, &a.schluessel(), Was::Da)
                    .map_err(text)?;
                let ziel = if a.ziel_art == "media" {
                    Art::Medium
                } else {
                    Art::Datei
                };
                speicher
                    .merken(&ziel, &a.ziel_uuid, Was::Da)
                    .map_err(text)?;
            }
        }
    }
    let _ = crate::server_abgleich(&zustand, false).await;
    Ok(f.uuid)
}

/// Teilen im Projekt beenden -- nur der Besitzer (das prueft der Server).
#[tauri::command]
pub async fn projekt_mappe_beenden(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    freigabe: String,
) -> Result<(), String> {
    let client = projektchat::verbunden(&zustand).await?;
    client
        .projekt_senden(
            Method::DELETE,
            &format!("/projects/{projekt}/mappen/{freigabe}"),
            None,
        )
        .await
        .map_err(text)?;
    Ok(())
}

/// Der Mappenschluessel ueber die Projekt-Kiste -- fuer Mitglieder (und den
/// Besitzer, wenn er die Mappe im Projekt oeffnet).
pub(crate) async fn ueber_projekt(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    projekt: &str,
    freigabe: &str,
) -> Result<(Value, Kontoschluessel, Kopf), String> {
    let liste = client
        .projekt_json(&format!("/projects/{projekt}/mappen"), &[])
        .await
        .map_err(text)?;
    let m = liste
        .get("mappen")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|m| m.get("uuid").and_then(Value::as_str) == Some(freigabe))
        .cloned()
        .ok_or("This folder is not shared here (anymore).")?;
    let p = projektchat::schluessel(zustand, client, projekt)
        .await?
        .ok_or("This device has no key for this project yet.")?;
    let besitzer = m
        .get("besitzer_name")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let kisten = m
        .get("kisten")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut verz = Verzeichnisse::neu(client);
    let s = bester(
        &kisten,
        &p,
        &projekt_kiste_ort(freigabe, projekt),
        &besitzer,
        &mut verz,
    )
    .await
    .ok_or("The folder key does not open here.")?;
    let kopf = kopf_auf(&m, &s, &mut verz)
        .await
        .ok_or("The folder does not open here.")?;
    Ok((m, s, kopf))
}

/// Die Notizen einer geteilten Mappe, geoeffnet. `pfad` ist der Pfad der
/// Mappe beim Besitzer -- Unterordner stehen darunter.
#[tauri::command]
pub async fn projekt_mappe_notizen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    freigabe: String,
) -> Result<Value, String> {
    let client = projektchat::verbunden(&zustand).await?;
    let (m, s, kopf) = ueber_projekt(&zustand, &client, &projekt, &freigabe).await?;
    let roh = client
        .projekt_json(
            &format!("/projects/{projekt}/mappen/{freigabe}/notizen"),
            &[],
        )
        .await
        .map_err(text)?;
    let mut verz = Verzeichnisse::neu(&client);
    let mut notizen = Vec::new();
    for n in roh
        .get("notizen")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let (Some(zk), Some(von), Some(geraet), Some(version), Some(v)) = (
            n.get("zk_id").and_then(Value::as_str),
            n.get("von_name").and_then(Value::as_str),
            n.get("geraet").and_then(Value::as_i64),
            n.get("version").and_then(Value::as_u64),
            chiffrat(n),
        ) else {
            continue;
        };
        let Some(ed) = verz.ed25519(von, geraet).await else {
            continue;
        };
        let Some(inhalt) = notiz_auf(&freigabe, &s, zk, version as u32, &v, &ed) else {
            continue;
        };
        notizen.push(json!({
            "zk_id": zk,
            "title": inhalt.title,
            "folder": inhalt.folder,
            "content": inhalt.content,
            "hash": openany_client::notizabdruck(&inhalt.content),
            "von_name": von,
            "updated_at": n.get("updated_at"),
        }));
    }
    Ok(json!({
        "pfad": kopf.pfad,
        "name": kopf.name,
        "stufe": m.get("stufe"),
        "notizen": notizen,
    }))
}

/// Eine Notiz der geteilten Mappe speichern (Stufe „bearbeiten") -- neu
/// (`zk_id` leer) oder geaendert, mit der Basis, auf der sie beruht. Bei
/// einem Konflikt entsteht eine Kopie daneben; die Antwort sagt es.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn projekt_mappe_notiz_speichern(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    freigabe: String,
    zk_id: Option<String>,
    titel: String,
    inhalt: String,
    mappe: Option<String>,
    basis: Option<String>,
) -> Result<Value, String> {
    let client = projektchat::verbunden(&zustand).await?;
    let (_, s, kopf) = ueber_projekt(&zustand, &client, &projekt, &freigabe).await?;
    let ich = direktpost::schluessel(&zustand)?;
    // Nur innerhalb der Mappe.
    let mappe = mappe.filter(|m| m == &kopf.pfad || m.starts_with(&format!("{}/", kopf.pfad)));
    let mappe = Some(mappe.unwrap_or(kopf.pfad.clone()));

    let schreiben = |zk: String, titel: String, basis: Option<String>| {
        let s = s.clone();
        let ich = &ich;
        let inhalt = inhalt.clone();
        let mappe = mappe.clone();
        let client = &client;
        let projekt = &projekt;
        let freigabe = &freigabe;
        async move {
            let abdruck = openany_client::notizabdruck(&inhalt);
            let (v, version) = notiz_zu(
                ich,
                freigabe,
                &s,
                &NotizInhalt {
                    v: 1,
                    zk_id: zk.clone(),
                    title: titel,
                    folder: mappe,
                    content: inhalt,
                },
            )?;
            client
                .projekt_senden(
                    Method::PUT,
                    &format!("/projects/{projekt}/mappen/{freigabe}/notizen/{zk}"),
                    Some(json!({
                        "inhalt": v.inhalt, "signatur": v.signatur, "version": version,
                        "hash": s.abdruck_verdecken(&zk, &abdruck),
                        "base_hash": basis.map(|b| s.abdruck_verdecken(&zk, &b)),
                    })),
                )
                .await
                .map(|_| zk)
                .map_err(|e| e.to_string())
        }
    };

    let neue_zk = || chrono::Local::now().format("%Y%m%d%H%M%S").to_string();
    match zk_id.filter(|z| !z.is_empty()) {
        Some(zk) => match schreiben(zk.clone(), titel.clone(), basis).await {
            Ok(zk) => Ok(json!({ "zk_id": zk, "konflikt": false })),
            // 409: jemand war schneller -- die eigene Fassung als Kopie daneben.
            Err(e) if e.contains("409") => {
                let kopie = schreiben(neue_zk(), format!("{titel} (Konflikt)"), None).await?;
                Ok(json!({ "zk_id": kopie, "konflikt": true }))
            }
            Err(e) => Err(e),
        },
        None => {
            // Eine neue Notiz; trifft die Kennung zufaellig eine vorhandene,
            // eine Sekunde weiter.
            let mut zk = neue_zk();
            for _ in 0..5 {
                match schreiben(zk.clone(), titel.clone(), None).await {
                    Ok(zk) => return Ok(json!({ "zk_id": zk, "konflikt": false })),
                    Err(e) if e.contains("409") || e.contains("422") => {
                        zk = (zk.parse::<u64>().unwrap_or(0) + 1).to_string();
                    }
                    Err(e) => return Err(e),
                }
            }
            Err("No free note id.".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn die_naechstgelegene_freigabe_ist_zustaendig() {
        let f = |uuid: &str, pfad: &str| Freigabe {
            uuid: uuid.into(),
            art: "mappe".into(),
            pfad: pfad.into(),
            wurzel: None,
            schluessel: Kontoschluessel::neu(),
        };
        // Ein geteiltes Album ist fuer keine Notiz zustaendig.
        let album = Freigabe {
            art: "album".into(),
            pfad: String::new(),
            wurzel: Some("a-1".into()),
            ..f("c", "")
        };
        let liste = vec![album, f("a", "Uni"), f("b", "Uni/Hausarbeit")];
        assert_eq!(
            zustaendig(&liste, Some("Uni/Hausarbeit/Teil"))
                .unwrap()
                .uuid,
            "b"
        );
        assert_eq!(zustaendig(&liste, Some("Uni")).unwrap().uuid, "a");
        assert!(zustaendig(&liste, Some("Unisport")).is_none());
        assert!(zustaendig(&liste, None).is_none());
    }

    #[test]
    fn notiz_hin_und_zurueck_nur_in_ihrer_mappe() {
        let ich = Schluessel::erzeugen().unwrap();
        let ed = ich.oeffentlich().1;
        let s = Kontoschluessel::neu();
        let n = NotizInhalt {
            v: 1,
            zk_id: "20261019120000".into(),
            title: "Beispiel".into(),
            folder: Some("Uni".into()),
            content: "Text".into(),
        };
        let (v, version) = notiz_zu(&ich, "f-1", &s, &n).unwrap();
        let auf = notiz_auf("f-1", &s, "20261019120000", version, &v, &ed).unwrap();
        assert_eq!(auf.content, "Text");
        assert!(notiz_auf("f-2", &s, "20261019120000", version, &v, &ed).is_none());
        assert!(notiz_auf("f-1", &s, "20261019120001", version, &v, &ed).is_none());
    }
}
