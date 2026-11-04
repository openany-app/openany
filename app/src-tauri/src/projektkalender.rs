//! Kalender in Projekten teilen (docs/plan-kalender-bearbeiten.md).
//!
//! **Mit Wolke** ist ein geteilter Kalender eine Freigabe der Art
//! `kalender` (wie eine Mappe, [`crate::mappenteilen`]): eigener Schluessel
//! in Kisten, Kalender und Termine damit verschluesselt. Beim Besitzer
//! erledigt das der eigene Abgleich ([`crate::servergegenstelle`]); hier
//! steht das Teilen und Beenden, und wie Mitglieder lesen
//! ([`auffrischen`]): Kiste ihres Projekts oeffnen, dann Kalender und
//! Termine. Was sie gelesen haben, liegt in einer Ablage
//! (`projektkalender.json`) -- so zeigt der eigene Kalender die geteilten
//! Termine auch ohne Netz ([`fremde`]).
//!
//! **Abo-Kalender** teilen sich vorerst nur vor Ort: Ihre Termine reisen
//! nicht ueber den Abgleich (`openany_sync::abo`).
//!
//! **Vor Ort (E7, docs/plan-projekte-eine-art.md):** Ein geteilter Kalender
//! ist zugleich eine Freigabe der Art `calendar` in der Liste des Projekts;
//! das Geraet des Besitzers liefert ihn samt Terminen im Stand an Mitglieder
//! in der Naehe (`openany_nahbereich::projektnah::GETEILTER_KALENDER`). Mit
//! Wolke gibt es beide Wege ([`spiegeln`]), ohne nur den vor Ort. Steht
//! derselbe Kalender auf beiden, zeigt sich der vom Server; vor Ort heisst
//! er `vor-ort:<kalender>`. Vor Ort wird nur gelesen.

use std::collections::HashMap;
use std::sync::Arc;

use openany_client::{Art, Eintrag, Method, OpenanyClient, OpenanyError, Terminfelder, Was};
use openany_store::Termin;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use openany_nahbereich::freigaben::KALENDER;
use openany_nahbereich::projektnah::{self, GETEILTER_KALENDER};
use openany_nahbereich::Gastgeber;

use crate::projektchat::{self, Verzeichnisse};
use crate::Zustand;
use crate::{direktpost, kalenderschloss, mappenteilen};

/// So beginnt die Kennung eines Kalenders, der nur vor Ort geteilt ist.
const VOR_ORT: &str = "vor-ort:";

fn text(e: impl ToString) -> String {
    e.to_string()
}

/// Ein geteilter Kalender, wie ein Mitglied ihn gelesen hat.
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Geteilt {
    pub uuid: String,
    /// Der Kalender beim Besitzer (aeltere Ablagen kennen ihn nicht).
    #[serde(default)]
    pub kalender: String,
    pub von_name: String,
    pub eigen: bool,
    pub name: String,
    pub color: String,
    /// `view` oder `edit` (aeltere Ablagen und vor Ort: `view`).
    #[serde(default = "nur_ansehen")]
    pub stufe: String,
    /// uuid → die neun Felder.
    pub termine: Vec<(String, Map<String, Value>)>,
}

fn nur_ansehen() -> String {
    "view".into()
}

/// Die Ablage: Projekt-uuid → geteilte Kalender.
#[derive(Serialize, Deserialize, Default)]
struct Ablage {
    projekte: HashMap<String, Vec<Geteilt>>,
    /// Aenderungen an fremden geteilten Kalendern, die noch nicht beim
    /// Server sind (ohne Netz geschrieben).
    #[serde(default)]
    ausstehend: Vec<Ausstehend>,
}

/// Eine Aenderung eines Mitglieds an einem geteilten Kalender.
#[derive(Serialize, Deserialize, Clone)]
struct Ausstehend {
    projekt: String,
    freigabe: String,
    uuid: String,
    /// Die neun Felder -- `None`: loeschen.
    felder: Option<Map<String, Value>>,
    /// Klartext-Abdruck der Fassung, auf der die Aenderung beruht.
    basis: Option<String>,
}

impl Ausstehend {
    /// In die gelesene Fassung uebernehmen, damit sie gleich zu sehen ist.
    fn anwenden(&self, g: &mut Geteilt) {
        g.termine.retain(|(u, _)| u != &self.uuid);
        if let Some(f) = &self.felder {
            g.termine.push((self.uuid.clone(), f.clone()));
        }
    }
}

fn ablage_pfad(zustand: &Zustand) -> std::path::PathBuf {
    zustand.ordner.join("projektkalender.json")
}

fn ablage_lesen(zustand: &Zustand) -> Ablage {
    std::fs::read(ablage_pfad(zustand))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

fn ablage_schreiben(zustand: &Zustand, a: &Ablage) {
    if let Ok(b) = serde_json::to_vec(a) {
        let _ = std::fs::write(ablage_pfad(zustand), b);
    }
}

fn lebender_kalender(
    speicher: &openany_store::Speicher,
    uuid: &str,
) -> Option<openany_store::Kalender> {
    speicher
        .kalender(uuid)
        .ok()
        .flatten()
        .filter(|k| k.papierkorb_at.is_none())
}

async fn auf_server(zustand: &Zustand, projekt: &str) -> Result<bool, String> {
    Ok(zustand
        .speicher
        .lock()
        .await
        .projekt(projekt)
        .map_err(text)?
        .is_some_and(|p| p.auf_server))
}

/// Die Freigabe vor Ort setzen oder zuruecknehmen.
async fn vor_ort_freigeben(
    zustand: &Arc<Zustand>,
    projekt: &str,
    kalender: &str,
    freigeben: bool,
) -> Result<(), String> {
    let name = {
        let speicher = zustand.speicher.lock().await;
        lebender_kalender(&speicher, kalender)
            .map(|k| k.name)
            .unwrap_or_default()
    };
    crate::projektbefehle::lokal_freigeben(
        zustand,
        projekt.to_string(),
        KALENDER.to_string(),
        kalender.to_string(),
        name,
        freigeben,
        None,
    )
    .await
}

/// Einen eigenen Kalender im Projekt teilen -- vor Ort immer, mit Wolke auch
/// ueber openany.de. Antwort: die Kennung, wie [`projekt_kalender`] sie zeigt.
#[tauri::command]
pub async fn projekt_kalender_teilen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    kalender: String,
    stufe: Option<String>,
) -> Result<String, String> {
    let stufe = stufe.unwrap_or_else(nur_ansehen);
    if !matches!(stufe.as_str(), "view" | "edit") {
        return Err("Unknown level.".into());
    }
    let mit_server = auf_server(&zustand, &projekt).await?;
    // Ein Abo geht (noch) nicht ueber openany.de -- dann gar nicht teilen,
    // statt nur halb.
    if mit_server {
        let speicher = zustand.speicher.lock().await;
        if lebender_kalender(&speicher, &kalender).is_some_and(|k| k.abo_url.is_some()) {
            return Err("Subscribed calendars can only be shared nearby for now.".into());
        }
    }
    vor_ort_freigeben(&zustand, &projekt, &kalender, true).await?;
    if !mit_server {
        return Ok(format!("{VOR_ORT}{kalender}"));
    }
    server_teilen(&zustand, &projekt, &kalender, &stufe).await
}

/// Mit Wolke teilen: eine Freigabe der Art `kalender` (oder die schon
/// vorhandene), dazu die Kiste fuer das Projekt. Kalender und Termine gehen
/// beim naechsten Abgleich mit dem Schluessel der Freigabe hinaus -- der
/// Abgleich wird gleich angestossen.
async fn server_teilen(
    zustand: &Arc<Zustand>,
    projekt: &str,
    kalender: &str,
    stufe: &str,
) -> Result<String, String> {
    let client = projektchat::verbunden(zustand).await?;
    let (name, termine) = {
        let speicher = zustand.speicher.lock().await;
        let k = lebender_kalender(&speicher, kalender).ok_or("This calendar no longer exists.")?;
        if k.abo_url.is_some() {
            return Err("Subscribed calendars can only be shared nearby for now.".into());
        }
        (
            k.name,
            speicher.termine_im_kalender(kalender).map_err(text)?,
        )
    };
    let kopf = mappenteilen::Kopf {
        v: 1,
        pfad: String::new(),
        name,
        wurzel: Some(kalender.to_string()),
        zone: None,
    };
    let f = mappenteilen::teilen_in(zustand, &client, projekt, "kalender", kopf, stufe).await?;
    {
        let speicher = zustand.speicher.lock().await;
        speicher
            .merken(&Art::Kalender, kalender, Was::Da)
            .map_err(text)?;
        for t in &termine {
            speicher
                .merken(&Art::Termin, &t.uuid, Was::Da)
                .map_err(text)?;
        }
    }
    let _ = crate::server_abgleich(zustand, false).await;
    let _ = auffrischen_projekt(zustand, &client, projekt).await;
    Ok(f.uuid)
}

/// Die eigenen Freigaben dieses Kalenders beim Server (aus der Ablage).
fn eigene_beim_server(zustand: &Zustand, projekt: &str, kalender: &str) -> Vec<String> {
    ablage_lesen(zustand)
        .projekte
        .remove(projekt)
        .unwrap_or_default()
        .into_iter()
        .filter(|g| g.eigen && g.kalender == kalender)
        .map(|g| g.uuid)
        .collect()
}

/// Teilen beenden -- auf beiden Wegen. `freigabe` ist die Kennung, wie
/// [`projekt_kalender`] sie zeigt. Laesst sich nicht klaeren, welcher
/// Kalender gemeint ist, bricht es ab: Sonst lebte er beim Spiegeln wieder auf.
#[tauri::command]
pub async fn projekt_kalender_beenden(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    freigabe: String,
) -> Result<(), String> {
    let mit_server = auf_server(&zustand, &projekt).await?;
    let client = if mit_server {
        let c = projektchat::verbunden(&zustand).await?;
        auffrischen_projekt(&zustand, &c, &projekt).await?;
        Some(c)
    } else {
        None
    };
    let kalender = match freigabe.strip_prefix(VOR_ORT) {
        Some(k) => k.to_string(),
        None => ablage_lesen(&zustand)
            .projekte
            .remove(&projekt)
            .unwrap_or_default()
            .into_iter()
            .find(|g| g.uuid == freigabe)
            .map(|g| g.kalender)
            .filter(|k| !k.is_empty())
            .ok_or("This shared calendar could not be found.")?,
    };
    if let Some(client) = &client {
        for f in eigene_beim_server(&zustand, &projekt, &kalender) {
            client
                .projekt_senden(
                    Method::DELETE,
                    &format!("/projects/{projekt}/mappen/{f}"),
                    None,
                )
                .await
                .map_err(text)?;
        }
        let _ = auffrischen_projekt(&zustand, client, &projekt).await;
    }
    vor_ort_freigeben(&zustand, &projekt, &kalender, false).await
}

/// Die Kalender vor Ort: fremde aus dem Stand der Mitglieder, eigene aus den
/// eigenen Freigaben.
async fn vor_ort(zustand: &Zustand, projekt: &str) -> Vec<Geteilt> {
    let g = zustand.gastgeber.clone();
    let speicher = zustand.speicher.lock().await;
    let mut aus: Vec<Geteilt> = speicher
        .projektsachen(projekt, GETEILTER_KALENDER)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|s| {
            let k: projektnah::GeteilterKalender = serde_json::from_value(s.felder.clone()).ok()?;
            Some(Geteilt {
                uuid: format!("{VOR_ORT}{}", k.kalender),
                kalender: k.kalender,
                von_name: s.text("von_name").to_string(),
                eigen: false,
                name: k.name,
                color: k.farbe,
                stufe: nur_ansehen(),
                termine: k.termine,
            })
        })
        .collect();
    let eigene = projektnah::lesen(&speicher, projekt).and_then(|(_, liste, freigaben)| {
        let m = liste
            .mitglieder(&|id| g.person_von(id))
            .map_err(|e| e.to_string())?;
        projektnah::meine_kalender(&speicher, g.as_ref(), projekt, &m, &freigaben)
    });
    let ich = g.person().map(|p| p.name).unwrap_or_default();
    for k in eigene.unwrap_or_default() {
        aus.push(Geteilt {
            uuid: format!("{VOR_ORT}{}", k.kalender),
            kalender: k.kalender,
            von_name: ich.clone(),
            eigen: true,
            name: k.name,
            color: k.farbe,
            stufe: nur_ansehen(),
            termine: k.termine,
        });
    }
    aus
}

/// Beide Wege zusammen: Was der Server hat, gilt; vor Ort kommt dazu, was
/// dort fehlt.
fn zusammen(mut server: Vec<Geteilt>, vor_ort: Vec<Geteilt>) -> Vec<Geteilt> {
    for v in vor_ort {
        if !server.iter().any(|s| s.kalender == v.kalender) {
            server.push(v);
        }
    }
    server
}

/// Alle geteilten Kalender eines Projekts, wie sie hier liegen. Ohne Wolke
/// zaehlt nur, was vor Ort gilt -- auch wenn die Ablage noch Altes kennt.
async fn alle(zustand: &Zustand, projekt: &str) -> Vec<Geteilt> {
    let server = if auf_server(zustand, projekt).await.unwrap_or(false) {
        ablage_lesen(zustand)
            .projekte
            .remove(projekt)
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    zusammen(server, vor_ort(zustand, projekt).await)
}

/// Mit Wolke: zu jedem eigenen geteilten Kalender beide Wege (wie E6).
/// Antwort: wie viele angelegt wurden.
pub(crate) async fn spiegeln(zustand: &Arc<Zustand>, projekt: &str) -> Result<usize, String> {
    let client = projektchat::verbunden(zustand).await?;
    auffrischen_projekt(zustand, &client, projekt).await?;
    let server: Vec<String> = ablage_lesen(zustand)
        .projekte
        .remove(projekt)
        .unwrap_or_default()
        .into_iter()
        .filter(|g| g.eigen && !g.kalender.is_empty())
        .map(|g| g.kalender)
        .collect();
    let hier: Vec<String> = vor_ort(zustand, projekt)
        .await
        .into_iter()
        .filter(|g| g.eigen)
        .map(|g| g.kalender)
        .collect();
    let mut angelegt = 0;
    for k in server.iter().filter(|k| !hier.contains(k)) {
        // Fehlt der Kalender hier, endet das Teilen beim naechsten Nachziehen.
        if vor_ort_freigeben(zustand, projekt, k, true).await.is_ok() {
            angelegt += 1;
        }
    }
    for k in hier.iter().filter(|k| !server.contains(k)) {
        match server_teilen(zustand, projekt, k, "view").await {
            Ok(_) => angelegt += 1,
            Err(e) => eprintln!("Kalender nicht auf openany.de gespiegelt ({projekt}, {k}): {e}"),
        }
    }
    Ok(angelegt)
}

/// Die geteilten Kalender eines Projekts -- frisch vom Server, sonst aus der
/// Ablage. Fuer den Abschnitt in „Planung".
#[tauri::command]
pub async fn projekt_kalender(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
) -> Result<Value, String> {
    if let Ok(client) = projektchat::verbunden(&zustand).await {
        let _ = auffrischen_projekt(&zustand, &client, &projekt).await;
    }
    let liste = alle(&zustand, &projekt).await;
    Ok(json!({
        "kalender": liste.into_iter().map(|g| json!({
            "uuid": g.uuid,
            "kalender": g.kalender,
            "von_name": g.von_name,
            "eigen": g.eigen,
            "stufe": g.stufe,
            "name": g.name,
            "color": g.color,
            "termine": g.termine.iter().map(|(uuid, f)| anzeige(&g.uuid, &g.color, uuid, f)).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    }))
}

/// Ein Termin in der Form, die die Kalender-Arbeitsflaeche kennt
/// (`TerminAnzeige` in lib.rs).
pub(crate) fn anzeige(freigabe: &str, farbe: &str, uuid: &str, f: &Map<String, Value>) -> Value {
    let t = Terminfelder::aus_eintrag(f);
    json!({
        "id": format!("p:{freigabe}:{uuid}"),
        "calendar_id": format!("p:{freigabe}"),
        "title": t.titel,
        "description": t.beschreibung,
        "location": t.ort,
        "start_date": t.beginn,
        "end_date": t.ende,
        "is_all_day": t.ganztags,
        "rrule": if t.rrule.is_empty() { "NONE".to_string() } else { t.rrule },
        "rrule_until": Some(t.rrule_bis).filter(|s| !s.is_empty()),
        "exdates": t.exdates,
        "color": farbe,
    })
}

/// Fremde geteilte Kalender -- aus der Ablage und vor Ort, (Projektname,
/// Kalender).
pub(crate) async fn fremde(zustand: &Zustand) -> Vec<(String, Geteilt)> {
    let projekte = zustand.speicher.lock().await.projekte().unwrap_or_default();
    let mut aus = Vec::new();
    for p in projekte {
        for g in alle(zustand, &p.uuid)
            .await
            .into_iter()
            .filter(|g| !g.eigen)
        {
            aus.push((p.name.clone(), g));
        }
    }
    aus
}

/// Ein Projekt frisch lesen und in die Ablage legen: seine Kalender-
/// Freigaben, ueber die Kiste des Projekts geoeffnet.
async fn auffrischen_projekt(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    projekt: &str,
) -> Result<(), String> {
    let liste = client
        .projekt_json(&format!("/projects/{projekt}/mappen"), &[])
        .await
        .map_err(text)?;
    let freigaben: Vec<Value> = liste
        .get("mappen")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|m| m.get("art").and_then(Value::as_str) == Some("kalender"))
        .cloned()
        .collect();

    let mut neu = Vec::new();
    if !freigaben.is_empty() {
        let p = projektchat::schluessel(zustand, client, projekt)
            .await?
            .ok_or("This device has no key for this project yet.")?;
        let mut verz = Verzeichnisse::neu(client);
        for m in freigaben {
            if let Some(g) = eine_lesen(client, projekt, &p, &m, &mut verz).await {
                neu.push(g);
            }
        }
    }
    let mut ablage = ablage_lesen(zustand);
    // Was noch aussteht, bleibt zu sehen.
    for a in ablage.ausstehend.iter().filter(|a| a.projekt == projekt) {
        if let Some(g) = neu.iter_mut().find(|g| g.uuid == a.freigabe) {
            a.anwenden(g);
        }
    }
    ablage.projekte.remove(projekt);
    if !neu.is_empty() {
        ablage.projekte.insert(projekt.to_string(), neu);
    }
    ablage_schreiben(zustand, &ablage);
    Ok(())
}

/// Eine Kalender-Freigabe lesen: Schluessel aus der Kiste des Projekts,
/// dann Kalender und Termine. Was nicht aufgeht, fehlt.
async fn eine_lesen(
    client: &OpenanyClient,
    projekt: &str,
    p: &openany_direktpost::Kontoschluessel,
    m: &Value,
    verz: &mut Verzeichnisse<'_>,
) -> Option<Geteilt> {
    let freigabe = m.get("uuid")?.as_str()?;
    let besitzer = m.get("besitzer_name")?.as_str()?;
    let kisten = m.get("kisten")?.as_array()?.clone();
    let s = mappenteilen::bester(
        &kisten,
        p,
        &mappenteilen::projekt_kiste_ort(freigabe, projekt),
        besitzer,
        verz,
    )
    .await?;
    let voll = client
        .projekt_json(
            &format!("/projects/{projekt}/mappen/{freigabe}/kalender"),
            &[],
        )
        .await
        .ok()?;
    let k = voll.get("kalender")?;
    let kal_uuid = k.get("uuid")?.as_str()?.to_string();
    let kal_eintrag = eintrag(Art::Kalender, &kal_uuid, k);
    let ed = verz.ed25519(besitzer, k.get("geraet")?.as_i64()?).await?;
    let kal = kalenderschloss::kalender_oeffnen_mit(&s, &ed, &kal_eintrag)?;

    let mut termine = Vec::new();
    for t in voll
        .get("termine")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let (Some(uuid), Some(von), Some(geraet)) = (
            t.get("uuid").and_then(Value::as_str),
            t.get("von_name").and_then(Value::as_str),
            t.get("geraet").and_then(Value::as_i64),
        ) else {
            continue;
        };
        let Some(ed) = verz.ed25519(von, geraet).await else {
            continue;
        };
        let e = eintrag(Art::Termin, uuid, t).mit("calendar", kal_uuid.clone());
        if let Some(auf) = kalenderschloss::termin_oeffnen_mit(&s, &ed, &e) {
            termine.push((
                uuid.to_string(),
                Terminfelder::aus_eintrag(&auf.felder).als_eintrag(),
            ));
        }
    }
    Some(Geteilt {
        uuid: freigabe.to_string(),
        kalender: kal_uuid,
        von_name: besitzer.to_string(),
        eigen: m.get("eigen").and_then(Value::as_bool) == Some(true),
        name: kal.text("name").unwrap_or_default().to_string(),
        color: kal.text("color").unwrap_or_default().to_string(),
        stufe: m
            .get("stufe")
            .and_then(Value::as_str)
            .unwrap_or("view")
            .to_string(),
        termine,
    })
}

/// Ein Chiffrat vom Server als Eintrag, wie [`kalenderschloss`] ihn oeffnet.
fn eintrag(art: Art, uuid: &str, v: &Value) -> Eintrag {
    let mut e = Eintrag::neu(art, uuid, Was::Da);
    for feld in ["inhalt", "signatur", "geraet", "version", "updated_at"] {
        if let Some(w) = v.get(feld) {
            e = e.mit(feld, w.clone());
        }
    }
    e
}

/// Nach dem Abgleich, fuer alle Projekte auf openany.de: fremde geteilte
/// Kalender lesen. Projekte, die es nicht mehr gibt, fallen aus der Ablage.
pub(crate) async fn auffrischen(zustand: &Arc<Zustand>, client: &OpenanyClient) {
    let Ok(liste) = client.projekt_json("/projects", &[]).await else {
        return;
    };
    let uuids: Vec<String> = liste
        .get("items")
        .or_else(|| liste.get("data"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|p| p.get("uuid").and_then(Value::as_str).map(str::to_string))
        .collect();
    {
        let mut ablage = ablage_lesen(zustand);
        ablage.projekte.retain(|p, _| uuids.contains(p));
        ablage.ausstehend.retain(|a| uuids.contains(&a.projekt));
        ablage_schreiben(zustand, &ablage);
    }
    ausstehende_schicken(zustand, client).await;
    for projekt in uuids {
        let _ = auffrischen_projekt(zustand, client, &projekt).await;
    }
}

/* ── Mitglieder schreiben (Stufe „bearbeiten") ─────────────────────────── */

/// Die Freigabe eines Kalenders, wie die Arbeitsflaeche ihn nennt (`p:<freigabe>`).
pub(crate) fn freigabe_von_kalender(id: &str) -> Option<&str> {
    id.strip_prefix("p:").filter(|f| !f.contains(':'))
}

/// Freigabe und uuid eines geteilten Termins (`p:<freigabe>:<uuid>`).
pub(crate) fn termin_kennung(id: &str) -> Option<(&str, &str)> {
    id.strip_prefix("p:")?.rsplit_once(':')
}

/// Ein geteilter Kalender aus der Ablage -- samt Projekt.
fn geteilt_finden(zustand: &Zustand, freigabe: &str) -> Option<(String, Geteilt)> {
    ablage_lesen(zustand)
        .projekte
        .into_iter()
        .find_map(|(p, liste)| {
            liste
                .into_iter()
                .find(|g| g.uuid == freigabe)
                .map(|g| (p, g))
        })
}

/// Warum eine Aenderung nicht ankam: nur fehlendes Netz (dann wartet sie)
/// oder ein anderer Fehler.
enum Senden {
    Netz,
    Fehler(String),
}

/// Eine Aenderung zum Server schicken. Antwort: die Konfliktkopie, falls
/// jemand schneller war -- die eigene Fassung liegt dann daneben.
async fn senden(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    a: &Ausstehend,
) -> Result<Option<Ausstehend>, Senden> {
    let (_, s, kopf) = mappenteilen::ueber_projekt(zustand, client, &a.projekt, &a.freigabe)
        .await
        .map_err(Senden::Fehler)?;
    let kalender = kopf
        .wurzel
        .ok_or_else(|| Senden::Fehler("This shared calendar does not open here.".into()))?;
    let pfad = |uuid: &str| {
        format!(
            "/projects/{}/mappen/{}/termine/{uuid}",
            a.projekt, a.freigabe
        )
    };
    let netz = |e: OpenanyError| match e {
        OpenanyError::Transport(_) => Senden::Netz,
        e => Senden::Fehler(e.to_string()),
    };
    let Some(felder) = &a.felder else {
        client
            .projekt_senden(Method::DELETE, &pfad(&a.uuid), None)
            .await
            .map_err(netz)?;
        return Ok(None);
    };
    let ich = direktpost::schluessel(zustand).map_err(Senden::Fehler)?;
    let koerper = |uuid: &str, felder: &Map<String, Value>, basis: Option<&str>| {
        let mut e = Eintrag::neu(Art::Termin, uuid, Was::Da).mit("calendar", kalender.clone());
        for (k, v) in felder {
            e = e.mit(k, v.clone());
        }
        if let Some(b) = basis {
            e = e.mit("base_hash", b);
        }
        kalenderschloss::termin_zu(&ich, &a.freigabe, &s, &e)
            .map(|zu| {
                json!({
                    "inhalt": zu.felder["inhalt"], "signatur": zu.felder["signatur"],
                    "version": zu.felder["version"], "hash": zu.felder["hash"],
                    "base_hash": zu.felder["base_hash"],
                })
            })
            .map_err(Senden::Fehler)
    };
    let k = koerper(&a.uuid, felder, a.basis.as_deref())?;
    match client
        .projekt_senden(Method::PUT, &pfad(&a.uuid), Some(k))
        .await
    {
        Ok(_) => Ok(None),
        // 409: jemand war schneller -- die eigene Fassung als Kopie daneben.
        Err(OpenanyError::Abgelehnt { status, .. }) if status.as_u16() == 409 => {
            let mut f = Terminfelder::aus_eintrag(felder);
            f.titel = format!("{} (Konflikt {})", f.titel, openany_store::konfliktzeit());
            let kopie = Ausstehend {
                uuid: uuid::Uuid::new_v4().to_string(),
                felder: Some(f.als_eintrag()),
                basis: None,
                ..a.clone()
            };
            let k = koerper(&kopie.uuid, &f.als_eintrag(), None)?;
            client
                .projekt_senden(Method::PUT, &pfad(&kopie.uuid), Some(k))
                .await
                .map_err(netz)?;
            Ok(Some(kopie))
        }
        Err(e) => Err(netz(e)),
    }
}

/// Eine Aenderung uebernehmen: schicken, sonst (ohne Netz) merken -- und
/// gleich in der Ablage zeigen.
async fn aendern(zustand: &Arc<Zustand>, a: Ausstehend) -> Result<(), String> {
    let client = projektchat::verbunden(zustand).await.ok();
    let gesendet = match &client {
        Some(c) => senden(zustand, c, &a).await,
        None => Err(Senden::Netz),
    };
    // Konflikt: Am Termin gilt die Fassung des Servers, die eigene liegt als
    // Kopie daneben -- beides kommt mit dem frischen Lesen.
    if let (Ok(Some(_)), Some(c)) = (&gesendet, &client) {
        return auffrischen_projekt(zustand, c, &a.projekt).await;
    }
    let mut ablage = ablage_lesen(zustand);
    let mut zeigen = vec![a.clone()];
    match gesendet {
        Ok(kopie) => zeigen.extend(kopie),
        Err(Senden::Netz) => {
            ablage
                .ausstehend
                .retain(|x| !(x.freigabe == a.freigabe && x.uuid == a.uuid));
            ablage.ausstehend.push(a.clone());
        }
        Err(Senden::Fehler(e)) => return Err(e),
    }
    if let Some(g) = ablage
        .projekte
        .get_mut(&a.projekt)
        .and_then(|l| l.iter_mut().find(|g| g.uuid == a.freigabe))
    {
        for z in &zeigen {
            z.anwenden(g);
        }
    }
    ablage_schreiben(zustand, &ablage);
    Ok(())
}

/// Nur mit „bearbeiten" -- das prueft auch der Server.
fn zum_schreiben(zustand: &Zustand, freigabe: &str) -> Result<(String, Geteilt), String> {
    let (projekt, g) =
        geteilt_finden(zustand, freigabe).ok_or("This shared calendar could not be found.")?;
    if g.stufe != "edit" {
        return Err("This calendar is shared for viewing only.".into());
    }
    Ok((projekt, g))
}

/// Einen Termin in einem fremden geteilten Kalender speichern -- neu
/// (`uuid` leer) oder geaendert. Was die Arbeitsflaeche nicht kennt
/// (Serienende, Ausnahmen), bleibt stehen. Antwort: der Termin, wie die
/// Arbeitsflaeche ihn kennt, und die Farbe des Kalenders.
pub(crate) async fn termin_speichern(
    zustand: &Arc<Zustand>,
    freigabe: &str,
    uuid: Option<String>,
    setzen: impl FnOnce(&mut Terminfelder),
) -> Result<(Termin, String), String> {
    let (projekt, g) = zum_schreiben(zustand, freigabe)?;
    let uuid = uuid
        .filter(|u| !u.is_empty())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let alt = g
        .termine
        .iter()
        .find(|(u, _)| u == &uuid)
        .map(|(_, f)| Terminfelder::aus_eintrag(f));
    let basis = alt.as_ref().map(Terminfelder::abdruck);
    let mut felder = alt.unwrap_or_default();
    setzen(&mut felder);
    aendern(
        zustand,
        Ausstehend {
            projekt,
            freigabe: freigabe.to_string(),
            uuid: uuid.clone(),
            felder: Some(felder.als_eintrag()),
            basis,
        },
    )
    .await?;
    Ok((
        Termin {
            uuid: format!("p:{freigabe}:{uuid}"),
            kalender_uuid: format!("p:{freigabe}"),
            felder,
            papierkorb_at: None,
            geaendert_at: String::new(),
        },
        g.color,
    ))
}

/// Einen Termin aus einem fremden geteilten Kalender loeschen -- endgueltig,
/// geteilte Termine haben keinen Papierkorb.
pub(crate) async fn termin_loeschen(
    zustand: &Arc<Zustand>,
    freigabe: &str,
    uuid: &str,
) -> Result<(), String> {
    let (projekt, _) = zum_schreiben(zustand, freigabe)?;
    aendern(
        zustand,
        Ausstehend {
            projekt,
            freigabe: freigabe.to_string(),
            uuid: uuid.to_string(),
            felder: None,
            basis: None,
        },
    )
    .await
}

/// Was ohne Netz liegen blieb, schicken. Was der Server ablehnt, faellt
/// fort -- es kommt beim Lesen in seiner Fassung zurueck.
async fn ausstehende_schicken(zustand: &Arc<Zustand>, client: &OpenanyClient) {
    let offen = ablage_lesen(zustand).ausstehend;
    if offen.is_empty() {
        return;
    }
    let mut bleibt = Vec::new();
    for a in offen {
        match senden(zustand, client, &a).await {
            Ok(_) => {}
            Err(Senden::Netz) => bleibt.push(a),
            Err(Senden::Fehler(e)) => eprintln!("Termin nicht geschickt ({}): {e}", a.uuid),
        }
    }
    let mut ablage = ablage_lesen(zustand);
    ablage.ausstehend = bleibt;
    ablage_schreiben(zustand, &ablage);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn geteilt(uuid: &str, kalender: &str) -> Geteilt {
        Geteilt {
            uuid: uuid.into(),
            kalender: kalender.into(),
            von_name: "Beispiel".into(),
            eigen: false,
            name: "Kalender".into(),
            color: "#123456".into(),
            stufe: nur_ansehen(),
            termine: Vec::new(),
        }
    }

    #[test]
    fn derselbe_kalender_zeigt_sich_einmal_der_vom_server() {
        let liste = zusammen(
            vec![geteilt("f-1", "k-1")],
            vec![geteilt("vor-ort:k-1", "k-1"), geteilt("vor-ort:k-2", "k-2")],
        );
        let uuids: Vec<&str> = liste.iter().map(|g| g.uuid.as_str()).collect();
        assert_eq!(uuids, ["f-1", "vor-ort:k-2"]);
    }

    #[test]
    fn anzeige_hat_die_form_der_arbeitsflaeche() {
        let f = Terminfelder {
            titel: "Beispiel".into(),
            beginn: "2026-11-02T10:00:00".into(),
            ende: "2026-11-02T11:00:00".into(),
            ..Default::default()
        }
        .als_eintrag();
        let a = anzeige("f-1", "#123456", "t-1", &f);
        assert_eq!(a["calendar_id"], "p:f-1");
        assert_eq!(a["id"], "p:f-1:t-1");
        assert_eq!(a["rrule"], "NONE");
        assert_eq!(a["color"], "#123456");
        assert!(a["rrule_until"].is_null());
    }

    #[test]
    fn kennungen_geteilter_kalender_und_termine() {
        assert_eq!(freigabe_von_kalender("p:f-1"), Some("f-1"));
        assert_eq!(freigabe_von_kalender("p:vor-ort:k-1"), None);
        assert_eq!(freigabe_von_kalender("k-1"), None);
        assert_eq!(termin_kennung("p:f-1:t-1"), Some(("f-1", "t-1")));
        assert_eq!(
            termin_kennung("p:vor-ort:k-1:t-1"),
            Some(("vor-ort:k-1", "t-1"))
        );
        assert_eq!(termin_kennung("t-1"), None);
    }

    #[test]
    fn ausstehendes_zeigt_sich_gleich() {
        let mut g = geteilt("f-1", "k-1");
        g.termine.push(("t-1".into(), Map::new()));
        let a = Ausstehend {
            projekt: "p-1".into(),
            freigabe: "f-1".into(),
            uuid: "t-2".into(),
            felder: Some(Map::new()),
            basis: None,
        };
        a.anwenden(&mut g);
        assert_eq!(g.termine.len(), 2);
        Ausstehend {
            uuid: "t-1".into(),
            felder: None,
            ..a
        }
        .anwenden(&mut g);
        let uuids: Vec<&str> = g.termine.iter().map(|(u, _)| u.as_str()).collect();
        assert_eq!(uuids, ["t-2"]);
    }
}
