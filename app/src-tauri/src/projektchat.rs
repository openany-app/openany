//! Projektschluessel + Projektchat Ende-zu-Ende (docs/plan-projektchat-e2e.md, 3b).
//!
//! **Der Schluessel pflegt sich selbst**: Wer den Chat
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
//! die Chat-Ansicht kommt aus dem Paket (`ProjectChat.vue`). `body` traegt
//! den entschluesselten Text, `e2e` sagt `verschluesselt` oder `unlesbar`.

use std::collections::HashMap;
use std::sync::Arc;

use openany_client::{Method, OpenanyClient};
use openany_direktpost::{Kontoschluessel, Verschluesselt};
use serde_json::{json, Value};
use std::sync::Mutex;
use std::time::{Duration, Instant};

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

/// Verzeichnisse je Kontoname, einmal je Aufruf geholt -- auf die Geraete
/// beschnitten, denen dieses Geraet traut
/// ([`direktpost::verzeichnis_vertraut`]): Ein Geraet ohne Siegel des
/// Kontos unterschreibt nichts.
///
/// Fuer einen HUB ([`Verzeichnisse::fuer_hub`]) kommt dazu: Einen
/// Pruefschluessel gibt es nur fuer Konten, die in der unterschriebenen
/// Mitgliederliste stehen. Wer eine Kiste oder einen Inhalt unterschrieben
/// haben soll, nennt der Server -- ohne diese Pruefung genuegte ihm ein
/// eigenes Konto mit gesiegeltem Geraet, um einen Hub-Schluessel
/// unterzuschieben.
pub(crate) struct Verzeichnisse<'a> {
    zustand: Option<&'a Zustand>,
    client: &'a OpenanyClient,
    da: HashMap<String, Verzeichnis>,
    hub: Option<Hub<'a>>,
}

struct Hub<'a> {
    zustand: &'a Arc<Zustand>,
    projekt: String,
}

/// Wer zu einem Hub gehoert -- eine Minute lang gemerkt, wie der Schluessel
/// ([`schluessel`]): Liste und Bindungen zu holen kostet je Konto Anfragen.
#[derive(Default)]
struct Zugehoerigkeit {
    /// Die gepruefte Liste; `None`: keine, oder sie liess sich nicht pruefen.
    liste: Option<openany_nahbereich::Mitgliederliste>,
    /// Kontoname (klein) → gehoert dazu.
    konten: HashMap<String, bool>,
}

type Zugehoerig = Mutex<Option<HashMap<String, (Instant, Zugehoerigkeit)>>>;
static ZUGEHOERIG: Zugehoerig = Mutex::new(None);

/// Gehoert das Konto `name` zum Hub `projekt`
/// ([`crate::serverkette::konto_mitglied`])?
pub(crate) async fn gehoert_zum_hub(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    projekt: &str,
    name: &str,
) -> bool {
    let konto = name.trim().to_lowercase();
    let frisch = |t: &Instant| t.elapsed() < Duration::from_secs(60);
    let gemerkt = ZUGEHOERIG.lock().ok().and_then(|g| {
        let (t, z) = g.as_ref()?.get(projekt)?;
        frisch(t).then(|| (z.liste.clone(), z.konten.get(&konto).copied()))
    });
    let liste = match gemerkt {
        Some((_, Some(ja))) => return ja,
        Some((liste, None)) => liste,
        None => {
            let liste = match crate::serverkette::holen(zustand, client, projekt).await {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("Mitgliederliste nicht uebernommen ({projekt}): {e}");
                    None
                }
            };
            if let Ok(mut g) = ZUGEHOERIG.lock() {
                g.get_or_insert_with(HashMap::new).insert(
                    projekt.to_string(),
                    (
                        Instant::now(),
                        Zugehoerigkeit {
                            liste: liste.clone(),
                            konten: HashMap::new(),
                        },
                    ),
                );
            }
            liste
        }
    };
    let ja = crate::serverkette::konto_mitglied(zustand, client, liste.as_ref(), name).await;
    if let Ok(mut g) = ZUGEHOERIG.lock() {
        if let Some((_, z)) = g.as_mut().and_then(|g| g.get_mut(projekt)) {
            z.konten.insert(konto, ja);
        }
    }
    ja
}

/// Die gemerkte Zugehoerigkeit vergessen -- wenn sich die Mitgliederliste
/// hier eben geaendert hat.
pub(crate) fn zugehoerigkeit_vergessen(projekt: &str) {
    if let Ok(mut g) = ZUGEHOERIG.lock() {
        if let Some(k) = g.as_mut() {
            k.remove(projekt);
        }
    }
}

impl<'a> Verzeichnisse<'a> {
    pub(crate) fn neu(zustand: &'a Zustand, client: &'a OpenanyClient) -> Self {
        Self::mit(Some(zustand), client)
    }

    /// Fuer alles, was in einem Hub unterschrieben ist: Kisten, Chat, Board,
    /// geteilte Mappen, Kalender, Dateien.
    pub(crate) fn fuer_hub(
        zustand: &'a Arc<Zustand>,
        client: &'a OpenanyClient,
        projekt: &str,
    ) -> Self {
        Self {
            hub: Some(Hub {
                zustand,
                projekt: projekt.to_string(),
            }),
            ..Self::mit(Some(zustand.as_ref()), client)
        }
    }

    /// Ohne Zustand (eine Gegenstelle, die ohne gebaut wurde) ungeprueft wie
    /// vor dem Siegel.
    pub(crate) fn mit(zustand: Option<&'a Zustand>, client: &'a OpenanyClient) -> Self {
        Self {
            zustand,
            client,
            da: HashMap::new(),
            hub: None,
        }
    }

    pub(crate) async fn ed25519(&mut self, name: &str, geraet: i64) -> Option<String> {
        if let Some(hub) = &self.hub {
            if !gehoert_zum_hub(hub.zustand, self.client, &hub.projekt, name).await {
                return None;
            }
        }
        if !self.da.contains_key(name) {
            let v = match self.zustand {
                Some(z) => direktpost::verzeichnis_vertraut(z, self.client, name).await,
                None => direktpost::verzeichnis(self.client, name).await,
            }
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

type Gemerkt = Mutex<Option<HashMap<String, (Instant, String)>>>;
static GEMERKT: Gemerkt = Mutex::new(None);

/// Den gemerkten Projektschluessel vergessen -- beim Ein- und Ausschalten:
/// Das Projekt bekommt dort einen NEUEN Schluessel; mit dem alten
/// verschluesselt, waere alles danach unlesbar.
pub(crate) fn schluessel_vergessen(projekt: &str) {
    if let Ok(mut g) = GEMERKT.lock() {
        if let Some(k) = g.as_mut() {
            k.remove(projekt);
        }
    }
}

/// Den Projektschluessel -- eine Minute lang aus dem Zwischenspeicher.
///
/// Chat, Planung, Freigaben und Mitgliederliste fragen ihn je Projekt, und
/// ihn zu holen kostet mehrere Anfragen; der Abgleich aller Projekte stiesse
/// sonst an die Drossel des Servers. Wechselt der Schluessel in
/// dieser Minute, merkt es der naechste Lauf.
pub(crate) async fn schluessel(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    projekt: &str,
) -> Result<Option<Kontoschluessel>, String> {
    if let Some(text) = GEMERKT.lock().ok().and_then(|g| {
        g.as_ref()?
            .get(projekt)
            .filter(|(t, _)| t.elapsed() < Duration::from_secs(60))
            .map(|(_, k)| k.clone())
    }) {
        if let Ok(k) = Kontoschluessel::aus_text(&text) {
            return Ok(Some(k));
        }
    }
    let k = schluessel_frisch(zustand, client, projekt).await?;
    if let (Some(k), Ok(mut g)) = (&k, GEMERKT.lock()) {
        g.get_or_insert_with(HashMap::new)
            .insert(projekt.to_string(), (Instant::now(), k.als_text()));
    }
    Ok(k)
}

/// Den Projektschluessel holen und pflegen. `None`: Dieses Geraet hat (noch)
/// keinen -- es wartet auf seine Freigabe oder auf eine Kiste.
async fn schluessel_frisch(
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
        // Nur Kisten von Konten des Hubs: `von_name` nennt der Server.
        let mut verz = Verzeichnisse::fuer_hub(zustand, client, projekt);

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

        // Kisten nur fuer Konten, die in der unterschriebenen Mitgliederliste
        // stehen (docs/plan-projekte-eine-art.md, E2). Ohne gepruefte Liste
        // nur fuer das eigene Konto.
        let fehlende = {
            let mut erlaubt = Vec::new();
            for f in fehlende {
                let name = f.get("name").and_then(Value::as_str).unwrap_or_default();
                if gehoert_zum_hub(zustand, client, projekt, name).await {
                    erlaubt.push(f);
                }
            }
            erlaubt
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
pub(crate) async fn oeffnen(
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
    match (p, verz.ed25519(&von, geraet).await) {
        (Some(p), Some(ed)) => {
            match openany_direktpost::projekt_oeffnen(p, projekt, &von, version, &v, &ed) {
                Ok(i) => gelesen = Some(i.text),
                Err(e) => {
                    eprintln!("Hub {projekt}: von {von} (Geraet {geraet}) nicht zu oeffnen: {e}")
                }
            }
        }
        (None, _) => eprintln!("Hub {projekt}: kein Hub-Schluessel hier"),
        (_, None) => {
            eprintln!("Hub {projekt}: Geraet {geraet} von {von} nicht im vertrauten Verzeichnis")
        }
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
    let ed = Verzeichnisse::fuer_hub(zustand, client, projekt)
        .ed25519(von, geraet)
        .await?;
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
            auf_server: true,
        });
    p.name = name.trim().to_string();
    // Wer hier einen Namen beim Server setzt, hat das Projekt dort.
    p.auf_server = true;
    speicher.projekt_schreiben(&p).map_err(text)
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
