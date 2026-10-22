//! Kalender in Projekten teilen -- nur lesen (docs/plan-kalender-e2e.md, 4c).
//!
//! **Der Besitzer:** Sein Geraet legt einen eigenen Kalender verschluesselt
//! ins Projekt ([`teilen`](projekt_kalender_teilen)) und zieht nach jedem
//! Abgleich nach ([`nachziehen`]): Es fragt den Server, welche Termine mit
//! welchem (verdeckten) Abdruck dort liegen, und schickt nur, was sich
//! geaendert hat oder fort ist. Ist der Kalender hier geloescht, endet das
//! Teilen.
//!
//! **Die Mitglieder:** lesen mit dem Projektschluessel. Was sie gelesen
//! haben, liegt in einer Ablage (`projektkalender.json`) -- so zeigt der
//! eigene Kalender die geteilten Termine auch ohne Netz ([`auffrischen`],
//! [`fremde`]).
//!
//! Verschluesselt wird wie der Projektname ([`openany_direktpost::projektfeld_verschluesseln`]):
//! der Kopf als Feld `kalender:<freigabe>`, jeder Termin als
//! `termin:<freigabe>:<uuid>`. So laesst sich kein Termin in eine andere
//! Freigabe oder ein anderes Projekt schieben.

use std::collections::HashMap;
use std::sync::Arc;

use openany_client::{Method, OpenanyClient, Terminfelder};
use openany_direktpost::{Kontoschluessel, Verschluesselt};
use openany_store::Termin;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::direktpost;
use crate::projektchat::{self, Verzeichnisse};
use crate::Zustand;

/// So viele Termine je Anfrage (der Server nimmt bis 500).
const STUECK: usize = 400;

fn text(e: impl ToString) -> String {
    e.to_string()
}

#[derive(Serialize, Deserialize)]
struct Kopf {
    v: u8,
    /// Der Kalender beim Besitzer -- nur sein Geraet braucht ihn.
    kalender: String,
    name: String,
    color: String,
}

/// Ein geteilter Kalender, wie ein Mitglied ihn gelesen hat.
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Geteilt {
    pub uuid: String,
    pub von_name: String,
    pub eigen: bool,
    pub name: String,
    pub color: String,
    pub stand: String,
    /// uuid → die neun Felder.
    pub termine: Vec<(String, Map<String, Value>)>,
}

/// Die Ablage: Projekt-uuid → geteilte Kalender.
#[derive(Serialize, Deserialize, Default)]
struct Ablage {
    projekte: HashMap<String, Vec<Geteilt>>,
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

fn kopf_feld(freigabe: &str) -> String {
    format!("kalender:{freigabe}")
}

fn termin_feld(freigabe: &str, uuid: &str) -> String {
    format!("termin:{freigabe}:{uuid}")
}

fn chiffrat(v: &Value) -> Option<(Verschluesselt, u32, i64)> {
    Some((
        Verschluesselt {
            inhalt: v.get("inhalt")?.as_str()?.to_string(),
            signatur: v.get("signatur")?.as_str()?.to_string(),
        },
        v.get("version")?.as_u64()? as u32,
        v.get("geraet")?.as_i64()?,
    ))
}

/// Ein Feld oeffnen -- unterschrieben von einem Geraet von `von`.
async fn feld_oeffnen(
    p: &Kontoschluessel,
    projekt: &str,
    feld: &str,
    von: &str,
    verz: &mut Verzeichnisse<'_>,
    v: &Value,
) -> Option<String> {
    let (c, version, geraet) = chiffrat(v)?;
    let ed = verz.ed25519(von, geraet).await?;
    openany_direktpost::projektfeld_oeffnen(p, projekt, feld, version, &c, &ed).ok()
}

/// Die Teile, die das Geraet des Besitzers verschickt.
struct Schreiber {
    ich: openany_direktpost::Schluessel,
    p: Kontoschluessel,
    konto: Kontoschluessel,
}

impl Schreiber {
    fn kopf(
        &self,
        projekt: &str,
        freigabe: &str,
        k: &openany_store::Kalender,
    ) -> Result<Value, String> {
        let roh = serde_json::to_string(&Kopf {
            v: 1,
            kalender: k.uuid.clone(),
            name: k.name.clone(),
            color: k.farbe.clone(),
        })
        .map_err(text)?;
        let (v, version) = openany_direktpost::projektfeld_verschluesseln(
            &self.ich,
            &self.p,
            projekt,
            &kopf_feld(freigabe),
            &roh,
        )
        .map_err(text)?;
        Ok(json!({ "inhalt": v.inhalt, "signatur": v.signatur, "version": version }))
    }

    fn termin(&self, projekt: &str, freigabe: &str, t: &Termin) -> Result<Value, String> {
        let roh = serde_json::to_string(&t.felder.als_eintrag()).map_err(text)?;
        let (v, version) = openany_direktpost::projektfeld_verschluesseln(
            &self.ich,
            &self.p,
            projekt,
            &termin_feld(freigabe, &t.uuid),
            &roh,
        )
        .map_err(text)?;
        Ok(json!({
            "uuid": t.uuid,
            "inhalt": v.inhalt,
            "signatur": v.signatur,
            "version": version,
            "abdruck": self.abdruck(t),
        }))
    }

    fn abdruck(&self, t: &Termin) -> String {
        self.konto.abdruck_verdecken(&t.uuid, &t.felder.abdruck())
    }

    /// Kopf, geaenderte und fortgefallene Termine schicken -- in Stuecken.
    async fn schicken(
        &self,
        client: &OpenanyClient,
        projekt: &str,
        freigabe: &str,
        kopf: Value,
        termine: Vec<Value>,
        fort: Vec<String>,
    ) -> Result<(), String> {
        let pfad = format!("/projects/{projekt}/kalender/{freigabe}");
        let mut stuecke: Vec<Vec<Value>> = termine.chunks(STUECK).map(<[Value]>::to_vec).collect();
        if stuecke.is_empty() {
            stuecke.push(Vec::new());
        }
        for (i, stueck) in stuecke.into_iter().enumerate() {
            let mut koerper = kopf.clone();
            koerper["termine"] = Value::Array(stueck);
            if i == 0 && !fort.is_empty() {
                koerper["fort"] = json!(fort);
            }
            client
                .projekt_senden(Method::PUT, &pfad, Some(koerper))
                .await
                .map_err(text)?;
        }
        Ok(())
    }
}

async fn schreiber(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    projekt: &str,
) -> Result<Schreiber, String> {
    let p = projektchat::schluessel(zustand, client, projekt)
        .await?
        .ok_or("This device has no key for this project yet.")?;
    Ok(Schreiber {
        ich: direktpost::schluessel(zustand)?,
        p,
        konto: direktpost::konto(zustand).ok_or("This device is not approved yet.")?,
    })
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

/// Einen eigenen Kalender im Projekt teilen. Antwort: die Freigabe-uuid.
#[tauri::command]
pub async fn projekt_kalender_teilen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    kalender: String,
) -> Result<String, String> {
    let client = projektchat::verbunden(&zustand).await?;
    let s = schreiber(&zustand, &client, &projekt).await?;
    let freigabe = uuid::Uuid::new_v4().to_string();
    let (kopf, termine) = {
        let speicher = zustand.speicher.lock().await;
        let k = lebender_kalender(&speicher, &kalender).ok_or("This calendar no longer exists.")?;
        let termine = speicher.termine_im_kalender(&kalender).map_err(text)?;
        let kopf = s.kopf(&projekt, &freigabe, &k)?;
        let termine = termine
            .iter()
            .map(|t| s.termin(&projekt, &freigabe, t))
            .collect::<Result<Vec<_>, _>>()?;
        (kopf, termine)
    };
    s.schicken(&client, &projekt, &freigabe, kopf, termine, Vec::new())
        .await?;
    let _ = auffrischen_projekt(&zustand, &client, &projekt).await;
    Ok(freigabe)
}

/// Teilen beenden -- nur der Besitzer (das prueft der Server).
#[tauri::command]
pub async fn projekt_kalender_beenden(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    freigabe: String,
) -> Result<(), String> {
    let client = projektchat::verbunden(&zustand).await?;
    client
        .projekt_senden(
            Method::DELETE,
            &format!("/projects/{projekt}/kalender/{freigabe}"),
            None,
        )
        .await
        .map_err(text)?;
    let _ = auffrischen_projekt(&zustand, &client, &projekt).await;
    Ok(())
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
    let liste = ablage_lesen(&zustand)
        .projekte
        .remove(&projekt)
        .unwrap_or_default();
    Ok(json!({
        "kalender": liste.into_iter().map(|g| json!({
            "uuid": g.uuid,
            "von_name": g.von_name,
            "eigen": g.eigen,
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

/// Fremde geteilte Kalender aus der Ablage -- (Projektname, Kalender).
pub(crate) async fn fremde(zustand: &Zustand) -> Vec<(String, Geteilt)> {
    let ablage = ablage_lesen(zustand);
    let speicher = zustand.speicher.lock().await;
    let mut aus = Vec::new();
    for (projekt, liste) in ablage.projekte {
        let name = speicher
            .projekt(&projekt)
            .ok()
            .flatten()
            .map(|p| p.name)
            .unwrap_or_default();
        for g in liste.into_iter().filter(|g| !g.eigen) {
            aus.push((name.clone(), g));
        }
    }
    aus
}

/// Ein Projekt frisch lesen und in die Ablage legen. Nur was sich geaendert
/// hat (`stand`), wird samt Terminen neu geholt.
async fn auffrischen_projekt(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    projekt: &str,
) -> Result<(), String> {
    let liste = client
        .projekt_json(&format!("/projects/{projekt}/kalender"), &[])
        .await
        .map_err(text)?;
    let kopfe = liste
        .get("kalender")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut ablage = ablage_lesen(zustand);
    let alt: HashMap<String, Geteilt> = ablage
        .projekte
        .remove(projekt)
        .unwrap_or_default()
        .into_iter()
        .map(|g| (g.uuid.clone(), g))
        .collect();

    let mut neu = Vec::new();
    if !kopfe.is_empty() {
        let p = projektchat::schluessel(zustand, client, projekt).await?;
        let ich = direktpost::mein_name(zustand).unwrap_or_default();
        let mut verz = Verzeichnisse::neu(client);
        for k in kopfe {
            let (Some(freigabe), Some(von), Some(stand)) = (
                k.get("uuid").and_then(Value::as_str),
                k.get("von_name").and_then(Value::as_str),
                k.get("stand").and_then(Value::as_str),
            ) else {
                continue;
            };
            if let Some(g) = alt.get(freigabe).filter(|g| g.stand == stand) {
                neu.push(g.clone());
                continue;
            }
            let Some(p) = &p else { continue };
            let Some(kopf) = feld_oeffnen(p, projekt, &kopf_feld(freigabe), von, &mut verz, &k)
                .await
                .and_then(|s| serde_json::from_str::<Kopf>(&s).ok())
            else {
                continue;
            };
            let voll = client
                .projekt_json(&format!("/projects/{projekt}/kalender/{freigabe}"), &[])
                .await
                .map_err(text)?;
            let mut termine = Vec::new();
            for t in voll
                .get("termine")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let Some(uuid) = t.get("uuid").and_then(Value::as_str) else {
                    continue;
                };
                if let Some(f) =
                    feld_oeffnen(p, projekt, &termin_feld(freigabe, uuid), von, &mut verz, t)
                        .await
                        .and_then(|s| serde_json::from_str::<Map<String, Value>>(&s).ok())
                {
                    termine.push((uuid.to_string(), f));
                }
            }
            neu.push(Geteilt {
                uuid: freigabe.to_string(),
                von_name: von.to_string(),
                eigen: !ich.is_empty() && von == ich,
                name: kopf.name,
                color: kopf.color,
                stand: stand.to_string(),
                termine,
            });
        }
    }
    if !neu.is_empty() {
        ablage.projekte.insert(projekt.to_string(), neu);
    }
    ablage_schreiben(zustand, &ablage);
    Ok(())
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
        ablage_schreiben(zustand, &ablage);
    }
    for projekt in uuids {
        let _ = auffrischen_projekt(zustand, client, &projekt).await;
    }
}

/// Nach dem Abgleich: was ich teile, auf den Stand bringen.
pub(crate) async fn nachziehen(zustand: &Arc<Zustand>, client: &OpenanyClient) {
    let Ok(liste) = client.projekt_json("/kalender/geteilt", &[]).await else {
        return;
    };
    let Some(ich) = direktpost::mein_name(zustand) else {
        return;
    };
    for k in liste
        .get("kalender")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
    {
        let (Some(projekt), Some(freigabe)) = (
            k.get("projekt").and_then(Value::as_str),
            k.get("uuid").and_then(Value::as_str),
        ) else {
            continue;
        };
        let _ = eine_nachziehen(zustand, client, projekt, freigabe, &ich, &k).await;
    }
}

async fn eine_nachziehen(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    projekt: &str,
    freigabe: &str,
    ich: &str,
    k: &Value,
) -> Result<(), String> {
    let s = schreiber(zustand, client, projekt).await?;
    let mut verz = Verzeichnisse::neu(client);
    let kopf: Kopf = feld_oeffnen(&s.p, projekt, &kopf_feld(freigabe), ich, &mut verz, k)
        .await
        .and_then(|t| serde_json::from_str(&t).ok())
        .ok_or("The shared calendar does not open here.")?;

    let hier = {
        let speicher = zustand.speicher.lock().await;
        lebender_kalender(&speicher, &kopf.kalender)
            .map(|kal| (kal, speicher.termine_im_kalender(&kopf.kalender)))
    };
    // Hier geloescht: Teilen endet.
    let Some((kal, termine)) = hier else {
        client
            .projekt_senden(
                Method::DELETE,
                &format!("/projects/{projekt}/kalender/{freigabe}"),
                None,
            )
            .await
            .map_err(text)?;
        return Ok(());
    };
    let termine = termine.map_err(text)?;

    let aktuell = s.p.aktuell_fassung();
    let veraltet = k.get("version").and_then(Value::as_u64).unwrap_or(0) < u64::from(aktuell);
    let kopf_neu = veraltet || kopf.name != kal.name || kopf.color != kal.farbe;

    let dort = client
        .projekt_json(&format!("/projects/{projekt}/kalender/{freigabe}"), &[])
        .await
        .map_err(text)?;
    let mut drueben: HashMap<String, (String, u64)> = HashMap::new();
    for t in dort
        .get("termine")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(uuid) = t.get("uuid").and_then(Value::as_str) {
            drueben.insert(
                uuid.to_string(),
                (
                    t.get("abdruck")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    t.get("version").and_then(Value::as_u64).unwrap_or(0),
                ),
            );
        }
    }

    let mut geaendert = Vec::new();
    for t in &termine {
        let gleich = drueben
            .get(&t.uuid)
            .is_some_and(|(a, v)| *a == s.abdruck(t) && *v >= u64::from(aktuell));
        if !gleich {
            geaendert.push(s.termin(projekt, freigabe, t)?);
        }
    }
    let fort: Vec<String> = drueben
        .keys()
        .filter(|u| !termine.iter().any(|t| &t.uuid == *u))
        .cloned()
        .collect();

    if !kopf_neu && geaendert.is_empty() && fort.is_empty() {
        return Ok(());
    }
    let kopf = s.kopf(projekt, freigabe, &kal)?;
    s.schicken(client, projekt, freigabe, kopf, geaendert, fort)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn felder_binden_freigabe_und_termin() {
        assert_eq!(kopf_feld("f-1"), "kalender:f-1");
        assert_eq!(termin_feld("f-1", "t-1"), "termin:f-1:t-1");
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
}
