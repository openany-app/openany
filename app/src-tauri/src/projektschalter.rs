//! Eine Art Projekt (docs/plan-projekte-eine-art.md).
//!
//! **Jedes Projekt entsteht auf dem Geraet**, mit unterschriebener
//! Mitgliederliste. Ob es AUCH ueber openany.de abgleicht, ist ein Schalter
//! (`projekte.auf_server`), den nur die Gruenderin bedient -- die Person aus
//! dem ersten Eintrag der Liste. Vorgabe beim Anlegen: an, wenn das Geraet
//! gekoppelt ist.
//!
//! **Einschalten:** Der Server legt das Projekt mit derselben uuid an (oder
//! belebt die Huelle von frueher), dann kommen die Liste, der Schluessel mit
//! dem verschluesselten Namen und die Mitglieder, hinter deren Person ein
//! Konto steht. Wer ohne Konto vor Ort beitrat, bleibt in der Liste und
//! gleicht weiter vor Ort ab.
//!
//! **Ausschalten:** Der Server loescht das Projekt samt Inhalt. Hier und bei
//! den anderen Mitgliedern laeuft es vor Ort weiter (die Geraete lesen
//! „abgeschaltet" im persoenlichen Strom).

use std::sync::Arc;

use openany_client::{Method, OpenanyClient, OpenanyError};
use openany_nahbereich::mitglieder::EIGENTUEMER;
use openany_nahbereich::{Gastgeber, Mitgliederliste};
use serde::Serialize;
use serde_json::{json, Value};

use crate::{personenbindung, projektchat, serverkette, Zustand};

fn text(e: impl ToString) -> String {
    e.to_string()
}

/// Die Liste eines Projekts hier -- und ob diese Person es gegruendet hat.
async fn liste_und_gruenderin(
    zustand: &Arc<Zustand>,
    projekt: &str,
) -> Result<(openany_store::Projekt, Mitgliederliste, bool), String> {
    let p = zustand
        .speicher
        .lock()
        .await
        .projekt(projekt)
        .map_err(text)?
        .ok_or("This project does not exist here.")?;
    let liste: Mitgliederliste = serde_json::from_str(
        p.mitgliederliste
            .as_deref()
            .ok_or("This project has no member list yet.")?,
    )
    .map_err(text)?;
    let ich = zustand.gastgeber.person();
    let gruenderin = match (liste.eintraege.first(), ich) {
        (Some(e), Some(i)) => e.personen_id == i.personen_id || i.frueher.contains(&e.personen_id),
        _ => false,
    };
    Ok((p, liste, gruenderin))
}

/// Ein Projekt anlegen -- auf dem Geraet, und mit Kopplung gleich auf
/// openany.de. `fehler`: Das Einschalten kam nicht durch; das Projekt gibt
/// es trotzdem, vor Ort.
#[derive(Debug, Serialize)]
pub struct Angelegt {
    pub id: String,
    pub fehler: Option<String>,
}

#[tauri::command]
pub async fn projekt_anlegen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    name: String,
) -> Result<Angelegt, String> {
    let id = crate::projektbefehle::projekt_lokal_anlegen(&zustand, name).await?;
    let fehler = match crate::server_client(&zustand).await {
        Some(client) => einschalten(&zustand, &client, &id).await.err(),
        None => None,
    };
    Ok(Angelegt { id, fehler })
}

/// Ein Projekt, das diese Person gegruendet hat, fuer die Liste unter Abgleich.
#[derive(Debug, Serialize)]
pub struct Gruendung {
    pub id: String,
    pub name: String,
    pub auf_server: bool,
}

#[tauri::command]
pub async fn projekte_gruendungen(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Vec<Gruendung>, String> {
    let projekte = zustand.speicher.lock().await.projekte().map_err(text)?;
    let mut aus = Vec::new();
    for p in projekte {
        if let Ok((_, _, true)) = liste_und_gruenderin(zustand.inner(), &p.uuid).await {
            aus.push(Gruendung {
                id: p.uuid,
                name: p.name,
                auf_server: p.auf_server,
            });
        }
    }
    Ok(aus)
}

/// Wie das Einschalten ausging -- fuer die Meldung.
#[derive(Debug, Default, Serialize)]
pub struct Eingeschaltet {
    /// Mitglieder, die der Server aufgenommen hat.
    pub aufgenommen: usize,
    /// Mitglieder ohne Konto: Sie gleichen weiter vor Ort ab.
    pub nur_vor_ort: usize,
}

#[tauri::command]
pub async fn projekt_einschalten(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
) -> Result<Eingeschaltet, String> {
    let client = projektchat::verbunden(&zustand).await?;
    einschalten(&zustand, &client, &projekt).await
}

async fn einschalten(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    projekt: &str,
) -> Result<Eingeschaltet, String> {
    let (p, liste, gruenderin) = liste_und_gruenderin(zustand, projekt).await?;
    if !gruenderin {
        return Err("Only the person who founded the project can switch it on.".into());
    }
    projektchat::schluessel_vergessen(projekt);

    // 1. Das Projekt mit SEINER uuid (oder die Huelle von frueher). Gibt es
    // es dort schon und kommt dieses Geraet hinein, brach ein frueheres
    // Einschalten ab: dann weiter, statt neu.
    match client
        .projekt_senden(Method::POST, "/projects", Some(json!({ "uuid": projekt })))
        .await
    {
        Ok(_) => {}
        Err(OpenanyError::Abgelehnt { status, .. })
            if status.as_u16() == 409
                && client
                    .projekt_json(&format!("/projects/{projekt}"), &[])
                    .await
                    .is_ok() => {}
        Err(OpenanyError::Abgelehnt { meldung, .. } | OpenanyError::NichtErlaubt(meldung)) => {
            return Err(meldung)
        }
        Err(anders) => return Err(anders.to_string()),
    }
    // Ein frueherer Strom dieses Projekts gilt nicht mehr: von vorn.
    {
        let speicher = zustand.speicher.lock().await;
        let _ = speicher.fremde_marke_setzen(
            &openany_store::Projekt::gegenstelle(OpenanyClient::basis(client), projekt),
            0,
        );
        speicher
            .projekt_auf_server_setzen(projekt, true)
            .map_err(text)?;
        // Chat und Planung liegen dort noch nicht (wieder): alles neu hoch.
        speicher
            .projekt_server_merker_vergessen(projekt)
            .map_err(text)?;
    }

    // 2. Die Liste, dann Schluessel und Name (die Kisten prueft schon die
    // Liste). Liegt sie schon dort (zweiter Versuch), ist das kein Fehler.
    if !serverkette::hochladen(zustand, client, &liste).await?
        && serverkette::holen(zustand, client, projekt)
            .await?
            .is_none()
    {
        return Err("The project already has another member list on openany.de.".into());
    }
    projektchat::name_setzen(zustand, client, projekt, &p.name).await?;

    // 3. Die Mitglieder mit Konto -- jedes ueber seine Bindung geprueft.
    let mitglieder = serverkette::mitglieder(zustand, &liste)?;
    let ich = zustand.gastgeber.person();
    let andere: Vec<_> = mitglieder
        .into_iter()
        .filter(|m| {
            !ich.as_ref().is_some_and(|i| {
                i.personen_id == m.personen_id || i.frueher.contains(&m.personen_id)
            })
        })
        .collect();
    let mut stand = Eingeschaltet::default();
    // Die eigenen Freigaben vor Ort entstehen auf dem Server neu (E6).
    if let Err(e) = crate::freigabenspiegel::spiegeln(zustand, projekt).await {
        eprintln!("Freigaben nicht gespiegelt ({projekt}): {e}");
    }
    if andere.is_empty() {
        return Ok(stand);
    }
    let konten = client
        .projekt_senden(
            Method::POST,
            "/personen/konten",
            Some(json!({ "personen": andere.iter().map(|m| &m.personen_id).collect::<Vec<_>>() })),
        )
        .await
        .map_err(text)?;
    let konten = konten
        .get("konten")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    for m in &andere {
        let konto = konten
            .iter()
            .find(|k| k.get("personen_id").and_then(Value::as_str) == Some(m.personen_id.as_str()));
        let (Some(nutzer), Some(name)) = (
            konto.and_then(|k| k.get("user_id")).and_then(Value::as_i64),
            konto.and_then(|k| k.get("name")).and_then(Value::as_str),
        ) else {
            stand.nur_vor_ort += 1;
            continue;
        };
        // Sagt der Server die Wahrheit? Die Bindung muss diese Person tragen.
        let echt = personenbindung::pruefen(client, name)
            .await
            .ok()
            .flatten()
            .is_some_and(|f| f.personen_id == m.personen_id);
        if !echt {
            stand.nur_vor_ort += 1;
            continue;
        }
        let pfad = format!("/projects/{projekt}/members");
        if client
            .projekt_senden(Method::POST, &pfad, Some(json!({ "user_id": nutzer })))
            .await
            .is_ok()
        {
            if m.rolle == EIGENTUEMER {
                let _ = client
                    .projekt_senden(
                        Method::PUT,
                        &format!("{pfad}/{nutzer}/role"),
                        Some(json!({ "role": EIGENTUEMER })),
                    )
                    .await;
            }
            stand.aufgenommen += 1;
        } else {
            stand.nur_vor_ort += 1;
        }
    }
    Ok(stand)
}

#[tauri::command]
pub async fn projekt_ausschalten(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
) -> Result<(), String> {
    let (_, _, gruenderin) = liste_und_gruenderin(&zustand, &projekt).await?;
    if !gruenderin {
        return Err("Only the person who founded the project can switch it off.".into());
    }
    projektchat::schluessel_vergessen(&projekt);
    let client = projektchat::verbunden(&zustand).await?;
    client
        .projekt_senden(
            Method::POST,
            &format!("/projects/{projekt}/abschalten"),
            None,
        )
        .await
        .map_err(|e| match e {
            OpenanyError::Abgelehnt { meldung, .. } | OpenanyError::NichtErlaubt(meldung) => {
                meldung
            }
            anders => anders.to_string(),
        })?;
    zustand
        .speicher
        .lock()
        .await
        .projekt_auf_server_setzen(&projekt, false)
        .map_err(text)
}
