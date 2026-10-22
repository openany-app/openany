//! Projektschluessel + Projektchat Ende-zu-Ende (docs/plan-projektchat-e2e.md, 3b).
//!
//! **Der Schluessel pflegt sich selbst** (Tiffy, 17.10.2026): Wer den Chat
//! oeffnet oder schreibt, holt seine Kiste -- und erledigt nebenbei, was der
//! Server als offen meldet:
//!
//! - noch keine Fassung: Fassung 1 anlegen, Kisten fuer alle Mitglieder mit
//!   Kontoschluessel;
//! - Wechsel faellig (jemand ist gegangen): neue Fassung, Kisten fuer alle
//!   Verbliebenen;
//! - einem Mitglied fehlt die Kiste der gueltigen Fassung: nachreichen.
//!
//! Gespeichert wird der Projektschluessel hier nicht: Die Kiste liegt beim
//! Server und ist in Millisekunden geoeffnet. So gibt es keinen zweiten Ort,
//! an dem ein Schluessel veralten kann.
//!
//! Die Antworten haben die Form des Servers (`ProjectMessageResource`) --
//! die Chat-Ansicht kommt aus der Webapp (`ProjectChat.vue`). `body` traegt
//! den entschluesselten Text, `e2e` sagt `verschluesselt` oder `unlesbar`.

use std::collections::HashMap;
use std::sync::Arc;

use openany_client::{Method, OpenanyClient};
use openany_direktpost::{Kontoschluessel, Verschluesselt};
use serde_json::{json, Value};

use crate::direktpost::{self, Verzeichnis};
use crate::Zustand;

fn text(e: impl ToString) -> String {
    e.to_string()
}

fn gueltige_uuid(projekt: &str) -> Result<&str, String> {
    if projekt.is_empty() || projekt.contains('/') || projekt.contains("..") {
        return Err("No project given.".into());
    }
    Ok(projekt)
}

/// Verzeichnisse je Kontoname, einmal je Aufruf geholt.
pub(crate) struct Verzeichnisse<'a> {
    client: &'a OpenanyClient,
    da: HashMap<String, Verzeichnis>,
}

impl<'a> Verzeichnisse<'a> {
    pub(crate) fn neu(client: &'a OpenanyClient) -> Self {
        Self {
            client,
            da: HashMap::new(),
        }
    }

    pub(crate) async fn ed25519(&mut self, name: &str, geraet: i64) -> Option<String> {
        if !self.da.contains_key(name) {
            let v = direktpost::verzeichnis(self.client, name)
                .await
                .unwrap_or_default();
            self.da.insert(name.to_string(), v);
        }
        self.da
            .get(name)?
            .geraete
            .iter()
            .find(|g| g.geraet == geraet)
            .map(|g| g.ed25519.clone())
    }
}

/// Kisten fuer `fehlende` (vom Server, mit Kontoschluessel) packen.
fn kisten_packen(
    ich: &openany_direktpost::Schluessel,
    p: &Kontoschluessel,
    projekt: &str,
    fehlende: &[Value],
) -> Result<Vec<Value>, String> {
    let mut aus = Vec::new();
    for f in fehlende {
        let (Some(nutzer), Some(age)) = (
            f.get("user_id").and_then(Value::as_i64),
            f.pointer("/konto/age").and_then(Value::as_str),
        ) else {
            // Ohne Kontoschluessel (App nie eingerichtet): spaeter.
            continue;
        };
        let (chiffrat, signatur) =
            openany_direktpost::projektkiste_packen(ich, p, projekt, nutzer, age).map_err(text)?;
        aus.push(json!({ "user_id": nutzer, "chiffrat": chiffrat, "signatur": signatur }));
    }
    Ok(aus)
}

/// Den Projektschluessel holen und pflegen. `None`: Dieses Geraet hat (noch)
/// keinen -- es wartet auf seine Freigabe oder auf eine Kiste.
pub(crate) async fn schluessel(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    projekt: &str,
) -> Result<Option<Kontoschluessel>, String> {
    // Kontoschluessel bereit? Ohne ihn oeffnet sich keine Kiste.
    direktpost::bereit(zustand, client).await.map_err(text)?;
    let Some(konto) = direktpost::konto(zustand) else {
        return Ok(None);
    };
    let ich = direktpost::schluessel(zustand)?;
    let pfad = format!("/projects/{projekt}/schluessel");

    for _versuch in 0..2 {
        let stand = client.projekt_json(&pfad, &[]).await.map_err(text)?;
        let mein_id = stand.get("ich").and_then(Value::as_i64).unwrap_or(0);
        let mut verz = Verzeichnisse::neu(client);

        // Die eigenen Kisten -- die mit der hoechsten Fassung, die aufgeht.
        let mut p: Option<Kontoschluessel> = None;
        for k in stand
            .get("kisten")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let version = k.get("version").and_then(Value::as_u64).unwrap_or(0) as u32;
            let (Some(c), Some(sig), Some(von), Some(name)) = (
                k.get("chiffrat").and_then(Value::as_str),
                k.get("signatur").and_then(Value::as_str),
                k.get("von_geraet").and_then(Value::as_i64),
                k.get("von_name").and_then(Value::as_str),
            ) else {
                continue;
            };
            let Some(ed) = verz.ed25519(name, von).await else {
                continue;
            };
            if let Ok(offen) = openany_direktpost::projektkiste_oeffnen(
                &konto, projekt, mein_id, version, c, sig, &ed,
            ) {
                if p.as_ref()
                    .is_none_or(|x| x.aktuell_fassung() < offen.aktuell_fassung())
                {
                    p = Some(offen);
                }
            }
        }

        let fehlende: Vec<Value> = stand
            .get("fehlende")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let gueltig = stand
            .pointer("/fassung/version")
            .and_then(Value::as_u64)
            .map(|v| v as u32);
        let wechsel = stand
            .get("wechsel_faellig")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let neu = match (gueltig, &p) {
            // Noch keine Fassung: anlegen.
            (None, _) => Some(Kontoschluessel::neu()),
            // Wechsel faellig, und hier liegt die gueltige: neue Fassung.
            (Some(g), Some(k)) if wechsel && k.aktuell_fassung() >= g => {
                let mut k = Kontoschluessel::aus_text(&k.als_text()).map_err(text)?;
                k.wechseln();
                Some(k)
            }
            _ => None,
        };

        if let Some(k) = neu {
            let kisten = kisten_packen(&ich, &k, projekt, &fehlende)?;
            let antwort = client
                .projekt_senden(
                    Method::POST,
                    &pfad,
                    Some(json!({ "version": k.aktuell_fassung(), "age": k.oeffentlich(), "kisten": kisten })),
                )
                .await;
            match antwort {
                Ok(_) => return Ok(Some(k)),
                // Jemand war schneller (409) -- noch einmal von vorn.
                Err(_) => continue,
            }
        }

        // Fehlende Kisten der gueltigen Fassung nachreichen.
        if let (Some(g), Some(k)) = (gueltig, &p) {
            if k.aktuell_fassung() >= g {
                let kisten = kisten_packen(&ich, k, projekt, &fehlende)?;
                if !kisten.is_empty() {
                    let _ = client
                        .projekt_senden(
                            Method::POST,
                            &format!("{pfad}/kisten"),
                            Some(json!({ "version": g, "kisten": kisten })),
                        )
                        .await;
                }
            }
        }
        return Ok(p);
    }
    Ok(None)
}

/// Eine Nachricht vom Server oeffnen -- an Ort und Stelle.
async fn oeffnen(
    p: Option<&Kontoschluessel>,
    projekt: &str,
    verz: &mut Verzeichnisse<'_>,
    m: &mut Value,
) {
    let von = m
        .pointer("/user/name")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let version = m.get("version").and_then(Value::as_u64).unwrap_or(0) as u32;
    let geraet = m.get("geraet").and_then(Value::as_i64).unwrap_or(0);
    let v = Verschluesselt {
        inhalt: m
            .get("inhalt")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        signatur: m
            .get("signatur")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
    };
    let mut gelesen = None;
    if let (Some(p), Some(ed)) = (p, verz.ed25519(&von, geraet).await) {
        gelesen = openany_direktpost::projekt_oeffnen(p, projekt, &von, version, &v, &ed)
            .ok()
            .map(|i| i.text);
    }
    if let Some(o) = m.as_object_mut() {
        o.remove("inhalt");
        o.remove("signatur");
        match gelesen {
            Some(t) => {
                o.insert("body".into(), Value::String(t));
                o.insert("e2e".into(), Value::String("verschluesselt".into()));
            }
            None => {
                o.insert("body".into(), Value::String(String::new()));
                o.insert("e2e".into(), Value::String("unlesbar".into()));
            }
        }
    }
}

pub(crate) async fn verbunden(zustand: &Arc<Zustand>) -> Result<OpenanyClient, String> {
    crate::server_client(zustand)
        .await
        .ok_or_else(|| "Not connected to openany.de.".to_string())
}

/// Der Verlauf (wie `GET /projects/{p}/messages`), entschluesselt.
#[tauri::command]
pub async fn projekt_server_chat(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    vor: Option<i64>,
    nach: Option<i64>,
) -> Result<Value, String> {
    let projekt = gueltige_uuid(&projekt)?;
    let client = verbunden(&zustand).await?;
    let p = schluessel(&zustand, &client, projekt).await?;
    let mut abfrage = Vec::new();
    if let Some(v) = vor {
        abfrage.push(("before_id", v.to_string()));
    }
    if let Some(n) = nach {
        abfrage.push(("after_id", n.to_string()));
    }
    let mut antwort = client
        .projekt_json(&format!("/projects/{projekt}/messages"), &abfrage)
        .await
        .map_err(text)?;
    let mut verz = Verzeichnisse::neu(&client);
    for feld in ["messages", "pinned"] {
        if let Some(liste) = antwort.get_mut(feld).and_then(Value::as_array_mut) {
            for m in liste {
                oeffnen(p.as_ref(), projekt, &mut verz, m).await;
            }
        }
    }
    if let Some(o) = antwort.as_object_mut() {
        // Ohne Schluessel: Die Ansicht sagt, warum sie nichts lesen kann.
        o.insert("ohne_schluessel".into(), Value::Bool(p.is_none()));
    }
    Ok(antwort)
}

/// Schreiben: verschluesselt an die gueltige Fassung.
#[tauri::command]
pub async fn projekt_server_chat_senden(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    inhalt: String,
    benachrichtigen: bool,
) -> Result<Value, String> {
    let projekt = gueltige_uuid(&projekt)?;
    let client = verbunden(&zustand).await?;
    let p = schluessel(&zustand, &client, projekt)
        .await?
        .ok_or("This device has no key for this project yet.")?;
    let ich = direktpost::schluessel(&zustand)?;
    let von =
        direktpost::mein_name(&zustand).ok_or("This device is not set up for messages yet.")?;
    let (v, version) = openany_direktpost::projekt_verschluesseln(&ich, &p, projekt, &von, &inhalt)
        .map_err(text)?;
    let mut antwort = client
        .projekt_senden(
            Method::POST,
            &format!("/projects/{projekt}/messages"),
            Some(json!({ "inhalt": v.inhalt, "signatur": v.signatur, "version": version, "notify_members": benachrichtigen })),
        )
        .await
        .map_err(text)?;
    let mut verz = Verzeichnisse::neu(&client);
    oeffnen(Some(&p), projekt, &mut verz, &mut antwort).await;
    Ok(antwort)
}

/// Anheften/Loesen -- Antwort entschluesselt.
#[tauri::command]
pub async fn projekt_server_chat_anheften(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    nachricht: i64,
    an: bool,
) -> Result<Value, String> {
    let projekt = gueltige_uuid(&projekt)?;
    let client = verbunden(&zustand).await?;
    let mut antwort = client
        .projekt_senden(
            Method::POST,
            &format!("/projects/{projekt}/messages/{nachricht}/pin"),
            Some(json!({ "pinned": an })),
        )
        .await
        .map_err(text)?;
    let p = schluessel(&zustand, &client, projekt).await?;
    let mut verz = Verzeichnisse::neu(&client);
    oeffnen(p.as_ref(), projekt, &mut verz, &mut antwort).await;
    Ok(antwort)
}

#[tauri::command]
pub async fn projekt_server_chat_loeschen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    nachricht: i64,
) -> Result<(), String> {
    let projekt = gueltige_uuid(&projekt)?;
    let client = verbunden(&zustand).await?;
    client
        .projekt_senden(
            Method::DELETE,
            &format!("/projects/{projekt}/messages/{nachricht}"),
            None,
        )
        .await
        .map_err(text)?;
    Ok(())
}

/// Die verlinkbaren Ziele (`GET /projects/{p}/link-targets`) -- fuer die
/// Vorschlagsliste und die Aufloesung der [[Verweise]] in der App.
#[tauri::command]
pub async fn projekt_server_verweisziele(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
) -> Result<Value, String> {
    let projekt = gueltige_uuid(&projekt)?;
    let client = verbunden(&zustand).await?;
    client
        .projekt_json(&format!("/projects/{projekt}/link-targets"), &[])
        .await
        .map_err(text)
}

// --- Projektname (3c) ----------------------------------------------------------

/// Den Namen aus einer Server-Antwort (ProjectResource, DeltaFeed) oeffnen --
/// `None`, wenn er nicht verschluesselt ist oder hier nicht aufgeht.
pub(crate) async fn name_oeffnen(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    projekt: &str,
    e: &serde_json::Map<String, Value>,
) -> Option<String> {
    let v = Verschluesselt {
        inhalt: e.get("name_inhalt")?.as_str()?.to_string(),
        signatur: e.get("name_signatur")?.as_str()?.to_string(),
    };
    let version = e.get("name_version")?.as_u64()? as u32;
    let geraet = e.get("name_geraet")?.as_i64()?;
    let von = e.get("name_von")?.as_str()?;
    let p = schluessel(zustand, client, projekt).await.ok()??;
    let ed = Verzeichnisse::neu(client).ed25519(von, geraet).await?;
    openany_direktpost::projektfeld_oeffnen(&p, projekt, "name", version, &v, &ed).ok()
}

/// Den Namen verschluesselt setzen (nur der Eigentuemer; das prueft der Server).
pub(crate) async fn name_setzen(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    projekt: &str,
    name: &str,
) -> Result<Value, String> {
    let p = schluessel(zustand, client, projekt)
        .await?
        .ok_or("This device has no key for this project yet.")?;
    let ich = direktpost::schluessel(zustand)?;
    let (v, version) =
        openany_direktpost::projektfeld_verschluesseln(&ich, &p, projekt, "name", name.trim())
            .map_err(text)?;
    client
        .projekt_senden(
            Method::PUT,
            &format!("/projects/{projekt}"),
            Some(json!({ "name_inhalt": v.inhalt, "name_signatur": v.signatur, "name_version": version })),
        )
        .await
        .map_err(text)
}

/// Den Namen hier im Speicher nachziehen -- sonst stuende bis zum naechsten
/// Abgleich der alte da.
async fn name_hier(
    zustand: &Arc<Zustand>,
    projekt: &str,
    name: &str,
    rolle: &str,
) -> Result<(), String> {
    let speicher = zustand.speicher.lock().await;
    let mut p = speicher
        .projekt(projekt)
        .map_err(text)?
        .unwrap_or(openany_store::Projekt {
            uuid: projekt.to_string(),
            name: String::new(),
            rolle: rolle.to_string(),
            geaendert_at: String::new(),
            mitgliederliste: None,
        });
    p.name = name.trim().to_string();
    speicher.projekt_schreiben(&p).map_err(text)
}

/// Ein Projekt auf openany.de anlegen: ohne Namen, dann den Schluessel, dann
/// den Namen verschluesselt. Antwort: die uuid.
#[tauri::command]
pub async fn projekt_server_anlegen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    name: String,
) -> Result<String, String> {
    if name.trim().is_empty() {
        return Err("No name given.".into());
    }
    let client = verbunden(&zustand).await?;
    let neu = client
        .projekt_senden(Method::POST, "/projects", Some(json!({})))
        .await
        .map_err(text)?;
    let uuid = neu
        .get("uuid")
        .and_then(Value::as_str)
        .ok_or("The server did not name the project.")?
        .to_string();
    name_setzen(&zustand, &client, &uuid, &name).await?;
    name_hier(&zustand, &uuid, &name, "owner").await?;
    Ok(uuid)
}

#[tauri::command]
pub async fn projekt_server_umbenennen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    name: String,
) -> Result<(), String> {
    let projekt = gueltige_uuid(&projekt)?;
    if name.trim().is_empty() {
        return Err("No name given.".into());
    }
    let client = verbunden(&zustand).await?;
    name_setzen(&zustand, &client, projekt, &name).await?;
    name_hier(&zustand, projekt, &name, "owner").await
}

/// Nach dem Abgleich: die Namen eigener Projekte, die noch im Klartext beim
/// Server liegen, verschluesseln (wie das Umstellen des Adressbuchs).
pub(crate) async fn namen_umstellen(zustand: &Arc<Zustand>, client: &OpenanyClient) {
    let Ok(liste) = client.projekt_json("/projects", &[]).await else {
        return;
    };
    let eintraege = liste
        .get("data")
        .or_else(|| liste.get("items"))
        .and_then(Value::as_array)
        .cloned()
        .or_else(|| liste.as_array().cloned())
        .unwrap_or_default();
    for p in eintraege {
        // Hier fehlt ein verschluesselter Name (leer oder „…"): vom Server
        // holen und oeffnen -- heilt auch, was ein aelterer Lauf geleert hat.
        if let (true, Some(uuid)) = (
            p.get("name_verschluesselt").and_then(Value::as_bool) == Some(true),
            p.get("uuid").and_then(Value::as_str),
        ) {
            let hier = {
                let speicher = zustand.speicher.lock().await;
                speicher.projekt(uuid).ok().flatten().map(|x| x.name)
            };
            if hier
                .as_deref()
                .is_none_or(|n| n.trim().is_empty() || n == "…")
            {
                if let Ok(voll) = client.projekt_json(&format!("/projects/{uuid}"), &[]).await {
                    if let Some(o) = voll.as_object() {
                        if let Some(name) = name_oeffnen(zustand, client, uuid, o).await {
                            let rolle =
                                p.get("my_role").and_then(Value::as_str).unwrap_or("member");
                            let _ = name_hier(zustand, uuid, &name, rolle).await;
                        }
                    }
                }
            }
        }
        let eigen = p.get("my_role").and_then(Value::as_str) == Some("owner");
        let klar = p
            .get("name")
            .and_then(Value::as_str)
            .filter(|n| !n.is_empty());
        let verschluesselt = p
            .get("name_verschluesselt")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        if let (true, Some(name), false, Some(uuid)) = (
            eigen,
            klar,
            verschluesselt,
            p.get("uuid").and_then(Value::as_str),
        ) {
            let _ = name_setzen(zustand, client, uuid, name).await;
        }
    }
}
