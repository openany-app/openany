//! Projekte im Programm -- die Verdrahtung.
//!
//! Ein Projekt ist eine weitere Gegenstelle (`<basis>#projekt:<uuid>`), und
//! der Laeufer dafuer steht in `openany-sync`. Hier steht nur, wie ein Klick
//! dorthin kommt und was die Oberflaeche zu sehen bekommt.
//!
//! **Welche Projekte es gibt, sagt der persoenliche Strom.** Der Laeufer
//! traegt sie als Art `project` ein; endet eine Mitgliedschaft, raeumt er sie
//! weg. Ein Projekt vom Server legt man in der Webapp an.
//!
//! **Lokale Projekte (01.10.2026, docs/konzept-lokale-mitgliedschaften.md)**
//! entstehen dagegen HIER, ohne Konto: mit einer unterschriebenen
//! Mitgliederliste (`openany_nahbereich::Mitgliederliste`) und ohne Weg zum
//! Server. Eingeladen wird spaeter vor Ort.

use crate::Zustand;
use openany_client::{Delta, Eintrag, Ergebnis, OpenanyClient, OpenanyError};
use openany_store::Projekt;
use openany_sync::{projekt_lauf, Projektbericht, Projektgegenstelle};
use serde::Serialize;
use std::sync::Arc;

fn fehler(e: impl ToString) -> String {
    e.to_string()
}

/// Der Strom eines Projekts auf openany.de.
pub struct ProjektGegenstelle {
    client: OpenanyClient,
    projekt: String,
    basis: String,
}

impl ProjektGegenstelle {
    pub fn neu(client: OpenanyClient, projekt: &str) -> Self {
        let basis = Projekt::gegenstelle(client.basis(), projekt);

        Self {
            client,
            projekt: projekt.to_string(),
            basis,
        }
    }
}

#[async_trait::async_trait]
impl Projektgegenstelle for ProjektGegenstelle {
    fn basis(&self) -> &str {
        &self.basis
    }

    fn projekt(&self) -> &str {
        &self.projekt
    }

    async fn delta(&self, seit: i64) -> Result<Delta, OpenanyError> {
        self.client.projekt_delta(&self.projekt, seit).await
    }

    async fn anwenden(&self, eintraege: &[Eintrag]) -> Result<Vec<Ergebnis>, OpenanyError> {
        self.client.projekt_anwenden(&self.projekt, eintraege).await
    }
}

/* ── Was die Oberflaeche sieht ─────────────────────────────────────────── */

#[derive(Serialize)]
pub struct ProjektAnzeige {
    id: String,
    name: String,
    /// `owner` oder `member` -- die Rolle kommt aus dem persoenlichen Strom.
    rolle: String,
    /// Auf diesem Geraet entstanden, ohne Server (lokales Projekt).
    lokal: bool,
    /// Wie viele Abstimmungen in diesem Projekt liegen.
    abstimmungen: usize,
}

#[derive(Serialize)]
pub struct Projektlauf {
    projekt: String,
    name: String,
    gezogen: usize,
    geschoben: usize,
    uebersprungen: usize,
    fehler: Vec<String>,
}

fn lauf_anzeige(projekt: &Projekt, bericht: Projektbericht) -> Projektlauf {
    Projektlauf {
        projekt: projekt.uuid.clone(),
        name: projekt.name.clone(),
        gezogen: bericht.gezogen,
        geschoben: bericht.geschoben,
        uebersprungen: bericht.uebersprungen,
        fehler: bericht.fehler,
    }
}

/* ── Befehle ───────────────────────────────────────────────────────────── */

/// Die Projekte, von denen dieses Geraet weiss.
#[tauri::command]
pub async fn projekte_liste(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Vec<ProjektAnzeige>, String> {
    let speicher = zustand.speicher.lock().await;
    let mut liste = Vec::new();

    for p in speicher.projekte().map_err(fehler)? {
        let zaehlen = |art: &str| -> Result<usize, String> {
            Ok(speicher.projektsachen(&p.uuid, art).map_err(fehler)?.len())
        };
        // Planung ist seit dem 16.10.2026 nur noch Abstimmungen (Kanban,
        // Roadmaps, Orte und Schulplanung sind zurueckgebaut).
        // Auf openany.de seit 19.10.2026 Ende-zu-Ende: gezaehlt wird, was
        // zuletzt geoeffnet wurde (projektabstimmung.rs) -- nicht, was vom
        // alten Projektstrom noch im Speicher liegt.
        let lokal = p.mitgliederliste.is_some();
        let abstimmungen = if lokal {
            zaehlen("poll")?
        } else {
            // Dazu die Whiteboards (docs/plan-whiteboard.md) -- es gibt sie
            // nur in Server-Projekten.
            crate::projektabstimmung::anzahl(&zustand, &p.uuid)
                + crate::projektwhiteboard::anzahl(&zustand, &p.uuid)
        };

        liste.push(ProjektAnzeige {
            lokal,
            id: p.uuid,
            name: p.name,
            rolle: p.rolle,
            abstimmungen,
        });
    }

    Ok(liste)
}

// Abstimmen ueber den Projektstrom (`projekt_abstimmen`) gab es bis zum
// 19.10.2026. Seitdem sind Abstimmungen Ende-zu-Ende verschluesselt und
// laufen ueber projektabstimmung.rs (docs/plan-abstimmungen-e2e.md).

/// Mit allen bekannten Projekten abgleichen.
///
/// **Ein Lauf je Projekt** -- elf Projekte sind elf Delta-Abfragen. Das
/// traegt die Drossel (`throttle:abgleich`), aber es ist der Grund, warum der
/// Auffrischer im Hintergrund Projekte seltener fragen sollte als den
/// persoenlichen Strom.
#[tauri::command]
pub async fn projekte_abgleichen(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Vec<Projektlauf>, String> {
    let Some(client) = crate::server_client(&zustand).await else {
        return Err("Not connected to openany.de.".into());
    };

    let projekte = {
        let speicher = zustand.speicher.lock().await;
        speicher.projekte().map_err(fehler)?
    };

    let mut laeufe = Vec::new();

    // Lokale Projekte kennt der Server nicht -- sie gleichen sich vor Ort ab.
    for p in projekte.into_iter().filter(|p| p.mitgliederliste.is_none()) {
        let gegenstelle = ProjektGegenstelle::neu(client.clone(), &p.uuid);
        let bericht = {
            let speicher = zustand.speicher.lock().await;
            projekt_lauf(&speicher, &gegenstelle).await
        };

        laeufe.push(lauf_anzeige(&p, bericht));
    }

    // Verschluesselte Namen nachziehen und eigene umstellen
    // (docs/plan-projektchat-e2e.md, 3c) -- wie nach dem persoenlichen Lauf.
    crate::projektchat::namen_umstellen(zustand.inner(), &client).await;

    // „Behalten“ fuer die alten, lesbaren Freigaben: Die gibt es seit 7d
    // nicht mehr (docs/plan-dateien-e2e.md) -- was dort stand, haelt nichts
    // mehr fest.
    {
        let mut e = zustand.einstellungen.lock().await;
        if !e.projekt_behalten.is_empty() {
            e.projekt_behalten.clear();
            e.schreiben(&zustand.einstellungspfad()).map_err(fehler)?;
        }
    }

    Ok(laeufe)
}

/* ── Projektsachen allgemein (01.10.2026) ──────────────────────────────── */
//
/* ── Mitglieder eines Server-Projekts (17.10.2026) ─────────────────────── */

/// Die Mitgliederliste eines Server-Projekts -- dieselbe Ansicht wie in der
/// Webapp (`ProjectMembers.vue`). Hier fragt die App ausnahmsweise den
/// Server statt ihres Speichers: Wer Mitglied ist, ist Verwaltungswissen des
/// Servers, auch mit Ende-zu-Ende-Verschluesselung (docs/plan-ende-zu-ende.md,
/// Schritt 1). Erlaubt sind genau diese Aktionen; was davon wer darf, prueft
/// der Server.
#[tauri::command]
pub async fn projekt_mitglieder_server(
    zustand: tauri::State<'_, Arc<Zustand>>,
    aktion: String,
    projekt: Option<String>,
    nutzer: Option<i64>,
    daten: Option<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    use openany_client::Method;
    let client = crate::server_client(&zustand)
        .await
        .ok_or("Not connected to openany.de.")?;
    let p = || {
        projekt
            .clone()
            .filter(|p| !p.is_empty() && !p.contains('/'))
            .ok_or_else(|| "No project given.".to_string())
    };
    let n = || nutzer.ok_or_else(|| "No member given.".to_string());
    let antwort = match aktion.as_str() {
        "projekt" => {
            client
                .projekt_json(&format!("/projects/{}", p()?), &[])
                .await
        }
        "suche" => {
            let q = daten
                .as_ref()
                .and_then(|d| d.get("q"))
                .and_then(|q| q.as_str())
                .unwrap_or_default()
                .to_string();
            client
                .projekt_json("/projects/user-search", &[("q", q)])
                .await
        }
        "hinzufuegen" => {
            client
                .projekt_senden(
                    Method::POST,
                    &format!("/projects/{}/members", p()?),
                    Some(serde_json::json!({ "user_id": n()? })),
                )
                .await
        }
        "konto" => {
            client
                .projekt_senden(
                    Method::POST,
                    &format!("/projects/{}/members/account", p()?),
                    daten,
                )
                .await
        }
        "entfernen" => {
            client
                .projekt_senden(
                    Method::DELETE,
                    &format!("/projects/{}/members/{}", p()?, n()?),
                    None,
                )
                .await
        }
        "rolle" => {
            client
                .projekt_senden(
                    Method::PUT,
                    &format!("/projects/{}/members/{}/role", p()?, n()?),
                    daten,
                )
                .await
        }
        "aendern" => {
            client
                .projekt_senden(Method::PUT, &format!("/projects/{}", p()?), daten)
                .await
        }
        _ => return Err(format!("Unknown member action: {aktion}")),
    };
    // Der Satz des Servers („Der letzte Owner kann nicht gehen") so, wie er
    // dort steht -- die Ansicht zeigt ihn wörtlich.
    antwort.map_err(|e| match e {
        openany_client::OpenanyError::Abgelehnt { meldung, .. }
        | openany_client::OpenanyError::NichtErlaubt(meldung) => meldung,
        anders => anders.to_string(),
    })
}

/* ── Lokale Projekte (01.10.2026) ──────────────────────────────────────── */

/// Ein Projekt nur auf diesem Geraet anlegen -- ohne Konto, ohne Server.
/// Die eigene Person ist Eigentuemerin.
#[tauri::command]
pub async fn projekt_lokal_anlegen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    name: String,
) -> Result<String, String> {
    use openany_nahbereich::Gastgeber;
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("A project needs a name.".into());
    }
    let ident = crate::geraet_identitaet(&zustand)?;
    let person = zustand
        .gastgeber
        .person()
        .ok_or("This device has no person yet -- has \"Nearby devices\" started?")?;
    let uuid = uuid::Uuid::new_v4().to_string();
    let liste =
        openany_nahbereich::Mitgliederliste::gruenden(&ident, &person, &uuid).map_err(fehler)?;
    let projekt = Projekt {
        uuid: uuid.clone(),
        name,
        rolle: openany_nahbereich::mitglieder::EIGENTUEMER.into(),
        geaendert_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        mitgliederliste: None,
    };
    zustand
        .speicher
        .lock()
        .await
        .projekt_lokal_schreiben(&projekt, &serde_json::to_string(&liste).map_err(fehler)?)
        .map_err(fehler)?;
    Ok(uuid)
}

#[derive(Serialize)]
pub struct MitgliedAnzeige {
    id: String,
    name: String,
    rolle: String,
    /// Das bin ich (meine Person).
    ich: bool,
    /// Darf ich es entfernen? (Ich bin Eigentuemer, und es bin nicht ich.)
    entfernbar: bool,
}

#[derive(Serialize)]
pub struct MitgliederAnzeige {
    mitglieder: Vec<MitgliedAnzeige>,
    /// Warum die Liste nicht gilt -- dann zeigt die Oberflaeche nichts an.
    fehler: Option<String>,
    /// Die Eigentuemerin hat nur dieses eine Geraet und keine Instanz: Geht
    /// es verloren, ist das Projekt nicht mehr zu verwalten (§11).
    einziges_geraet: bool,
    /// Darf ich austreten? (Ich bin Mitglied, und nicht die letzte
    /// Eigentuemerin.)
    kann_austreten: bool,
    /// Stehe ich (noch) in der Liste? Wer entfernt wurde, sieht es hier --
    /// auch ohne Abgleich, sobald die Liste einmal angekommen ist.
    bin_mitglied: bool,
}

/// Die Mitglieder eines LOKALEN Projekts, wie die unterschriebene Liste sie
/// ergibt. `None` bei einem Projekt vom Server.
#[tauri::command]
pub async fn projekt_mitglieder(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
) -> Result<Option<MitgliederAnzeige>, String> {
    use openany_nahbereich::Gastgeber;
    let Some(json) = zustand
        .speicher
        .lock()
        .await
        .projekt(&projekt)
        .map_err(fehler)?
        .and_then(|p| p.mitgliederliste)
    else {
        return Ok(None);
    };
    let liste: openany_nahbereich::Mitgliederliste = serde_json::from_str(&json).map_err(fehler)?;
    let ich = zustand.gastgeber.person();
    let ident = crate::geraet_identitaet(&zustand).ok();
    // Bekannte Personen: die eigene und die anderer Mitglieder; fuer
    // Unbekannte gilt ihr Anker (das Geraet, das beim Beitritt dabei war).
    let g = zustand.gastgeber.clone();
    let kennt = move |id: &str| g.bekannte_person(id);
    let einziges_geraet = match (&ich, &ident) {
        (Some(p), Some(i)) => {
            p.geraete(&[&i.fingerabdruck]).len() < 2
                && crate::server_client(&zustand).await.is_none()
        }
        _ => true,
    };
    Ok(Some(match liste.mitglieder(&kennt) {
        Ok(m) => {
            let bin = |id: &str| {
                ich.as_ref()
                    .is_some_and(|p| p.personen_id == id || p.frueher.iter().any(|f| f == id))
            };
            let eigentuemer = |x: &openany_nahbereich::Mitglied| {
                x.rolle == openany_nahbereich::mitglieder::EIGENTUEMER
            };
            let bin_eigentuemer = m.iter().any(|x| bin(&x.personen_id) && eigentuemer(x));
            let zahl_eigentuemer = m.iter().filter(|x| eigentuemer(x)).count();
            let kann_austreten = m
                .iter()
                .any(|x| bin(&x.personen_id) && !(eigentuemer(x) && zahl_eigentuemer == 1));
            MitgliederAnzeige {
                mitglieder: m
                    .iter()
                    .map(|x| MitgliedAnzeige {
                        id: x.personen_id.clone(),
                        name: x.name.clone(),
                        rolle: x.rolle.clone(),
                        ich: bin(&x.personen_id),
                        entfernbar: bin_eigentuemer && !bin(&x.personen_id),
                    })
                    .collect(),
                fehler: None,
                einziges_geraet,
                kann_austreten,
                bin_mitglied: m.iter().any(|x| bin(&x.personen_id)),
            }
        }
        Err(e) => MitgliederAnzeige {
            mitglieder: Vec::new(),
            fehler: Some(e.to_string()),
            einziges_geraet,
            kann_austreten: false,
            bin_mitglied: false,
        },
    }))
}

/// Ein Mitglied entfernen (nur die Eigentuemerin). Die anderen erfahren es
/// beim naechsten Abgleich; das entfernte Geraet auch -- es bekommt noch die
/// Liste, aber nichts mehr sonst, und raeumt das Projekt weg. Was es schon
/// hatte, bleibt auf seinen Geraeten.
#[tauri::command]
pub async fn projekt_mitglied_entfernen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    personen_id: String,
) -> Result<(), String> {
    use openany_nahbereich::Gastgeber;
    let g = zustand.gastgeber.clone();
    let ident = crate::geraet_identitaet(&zustand)?;
    let speicher = zustand.speicher.lock().await;
    let (p, mut liste, _) = openany_nahbereich::projektnah::lesen(&speicher, &projekt)?;
    let m = liste.mitglieder(&|id| g.person_von(id)).map_err(fehler)?;
    let wer = m
        .iter()
        .find(|x| x.personen_id == personen_id)
        .ok_or("Not a member.")?
        .clone();
    liste
        .eintragen(
            &ident,
            "austritt",
            &wer.personen_id,
            &wer.name,
            "",
            &wer.geraet,
        )
        .map_err(fehler)?;
    // Gilt das? (Nur die Eigentuemerin entfernt; die letzte bleibt.)
    let neu = liste.mitglieder(&|id| g.person_von(id)).map_err(fehler)?;
    speicher
        .projekt_lokal_schreiben(&p, &serde_json::to_string(&liste).map_err(fehler)?)
        .map_err(fehler)?;
    for art in [
        openany_nahbereich::projektnah::GETEILTE_NOTIZ,
        openany_nahbereich::projektnah::GETEILTE_DATEI,
        openany_nahbereich::projektnah::GETEILTES_BILD,
    ] {
        for s in speicher.projektsachen(&projekt, art).map_err(fehler)? {
            if !neu.iter().any(|x| x.personen_id == s.text("von")) {
                speicher
                    .projektsache_entfernen(&projekt, &s.uuid, openany_store::Protokoll::Still)
                    .map_err(fehler)?;
            }
        }
    }
    Ok(())
}

/// Selbst austreten -- NUR VOR ORT: Der unterschriebene Austritt geht erst an
/// ein Mitgliedsgeraet in der Naehe, dann raeumt dieses Geraet das Projekt
/// weg. Sonst erfuehre niemand davon.
#[tauri::command]
pub async fn projekt_austreten(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
) -> Result<String, String> {
    use openany_nahbereich::Gastgeber;
    let g = zustand.gastgeber.clone();
    let ich = g.person().ok_or("This device has no person yet.")?;
    let ident = crate::geraet_identitaet(&zustand)?;
    let (mut liste, m) = {
        let speicher = zustand.speicher.lock().await;
        let (_, liste, _) = openany_nahbereich::projektnah::lesen(&speicher, &projekt)?;
        let m = liste.mitglieder(&|id| g.person_von(id)).map_err(fehler)?;
        (liste, m)
    };
    let selbst = m
        .iter()
        .find(|x| x.personen_id == ich.personen_id || ich.frueher.contains(&x.personen_id))
        .ok_or("You are not a member of this project.")?
        .clone();
    liste
        .eintragen(
            &ident,
            "austritt",
            &selbst.personen_id,
            &selbst.name,
            "",
            &selbst.geraet,
        )
        .map_err(fehler)?;
    liste.mitglieder(&|id| g.person_von(id)).map_err(fehler)?;

    // Wer von den anderen ist in der Naehe?
    let geraete: Vec<openany_nahbereich::GeraetInDerNaehe> = zustand
        .nah
        .lock()
        .await
        .as_ref()
        .map(|n| n.geraete())
        .unwrap_or_default()
        .into_iter()
        .filter(|d| d.openany && d.fingerabdruck != ident.fingerabdruck)
        .filter(|d| {
            m.iter().any(|x| {
                x.personen_id != selbst.personen_id
                    && openany_nahbereich::freigaben::geraet_von(x, &d.fingerabdruck, &|id| {
                        g.person_von(id)
                    })
            })
        })
        .collect();
    let mut bei = None;
    let mut letzter = "No other member is nearby -- leaving only works in person.".to_string();
    for d in geraete {
        match openany_nahbereich::anruf::liste_senden(
            &ident,
            &d.adresse,
            openany_nahbereich::DIENST_PORT,
            &d.fingerabdruck,
            &liste,
        )
        .await
        {
            Ok(()) => {
                bei = Some(d.name.clone());
                break;
            }
            Err(e) => letzter = format!("{}: {e}", d.name),
        }
    }
    let bei = bei.ok_or(letzter)?;
    zustand
        .speicher
        .lock()
        .await
        .projekt_vergessen("lokal", &projekt)
        .map_err(fehler)?;
    Ok(bei)
}

/* ── Fremde Notizen bearbeiten (01.10.2026, strenge Sperre) ────────────── */
//
// Die Sperre vergibt das Geraet der Person, die freigegeben hat; gespeichert
// wird dort (openany-nahbereich/src/sperren.rs). Ist es nicht in der Naehe,
// bleibt die Notiz nur lesbar.

/// Wer hat diese fremde Notiz freigegeben, und wo ist sie dort?
pub(crate) async fn notiz_herkunft(
    zustand: &Zustand,
    projekt: &str,
    id: &str,
) -> Result<(String, String, String), String> {
    use openany_nahbereich::Gastgeber;
    let g = zustand.gastgeber.clone();
    let speicher = zustand.speicher.lock().await;
    let s = speicher
        .projektsache(projekt, id)
        .map_err(fehler)?
        .ok_or("This note does not exist here.")?;
    let von = s.text("von").to_string();
    let zk_id = id
        .strip_prefix(&format!("{von}:"))
        .ok_or("This note does not exist here.")?
        .to_string();
    let (_, liste, _) = openany_nahbereich::projektnah::lesen(&speicher, projekt)?;
    let anker = liste
        .mitglieder(&|x| g.person_von(x))
        .map_err(fehler)?
        .into_iter()
        .find(|m| m.personen_id == von)
        .map(|m| m.geraet)
        .ok_or("Whoever shared this is no longer a member.")?;
    Ok((von, anker, zk_id))
}

#[derive(Serialize)]
pub struct GesperrteNotiz {
    titel: String,
    inhalt: String,
    /// Das fuehrende Geraet -- dort wird gespeichert und losgelassen.
    geraet: String,
}

/// Eine fremde Notiz zum Bearbeiten sperren -- beim Geraet dessen, der sie
/// freigegeben hat. Gibt den frischen Inhalt zurueck.
#[tauri::command]
pub async fn projekt_notiz_sperren(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    id: String,
) -> Result<GesperrteNotiz, String> {
    let (von, anker, zk_id) = notiz_herkunft(&zustand, &projekt, &id).await?;
    let geraete = geraete_von(&zustand, &von, &anker).await;
    if geraete.is_empty() {
        return Err("Read only: the device that shared this note is not nearby right now.".into());
    }
    let ident = crate::geraet_identitaet(&zustand)?;
    let mut letzter = String::new();
    for d in geraete {
        match openany_nahbereich::anruf::notiz_sperren(
            &ident,
            &d.adresse,
            openany_nahbereich::DIENST_PORT,
            &d.fingerabdruck,
            &projekt,
            &zk_id,
        )
        .await
        {
            Ok(n) => {
                return Ok(GesperrteNotiz {
                    titel: n.titel,
                    inhalt: n.inhalt,
                    geraet: d.fingerabdruck,
                })
            }
            Err(e) => letzter = e,
        }
    }
    Err(letzter)
}

/// Speichern -- auf dem fuehrenden Geraet; die Kopie hier zieht gleich mit.
#[tauri::command]
pub async fn projekt_notiz_speichern(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    id: String,
    geraet: String,
    titel: String,
    inhalt: String,
) -> Result<(), String> {
    let (_, _, zk_id) = notiz_herkunft(&zustand, &projekt, &id).await?;
    let adresse = crate::adresse_von_geraet(&zustand, &geraet).await?;
    let ident = crate::geraet_identitaet(&zustand)?;
    openany_nahbereich::anruf::notiz_speichern(
        &ident,
        &adresse,
        openany_nahbereich::DIENST_PORT,
        &geraet,
        &projekt,
        &zk_id,
        &titel,
        &inhalt,
    )
    .await?;
    let speicher = zustand.speicher.lock().await;
    if let Some(mut s) = speicher.projektsache(&projekt, &id).map_err(fehler)? {
        s.felder["titel"] = serde_json::json!(titel);
        s.felder["inhalt"] = serde_json::json!(inhalt);
        s.geaendert_at = String::new();
        speicher
            .projektsache_schreiben(&s, openany_store::Protokoll::Still)
            .map_err(fehler)?;
    }
    Ok(())
}

/// Fertig: die Sperre loslassen.
#[tauri::command]
pub async fn projekt_notiz_entsperren(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    id: String,
    geraet: String,
) -> Result<(), String> {
    let (_, _, zk_id) = notiz_herkunft(&zustand, &projekt, &id).await?;
    let adresse = crate::adresse_von_geraet(&zustand, &geraet).await?;
    let ident = crate::geraet_identitaet(&zustand)?;
    openany_nahbereich::anruf::notiz_entsperren(
        &ident,
        &adresse,
        openany_nahbereich::DIENST_PORT,
        &geraet,
        &projekt,
        &zk_id,
    )
    .await
}

/* ── Chat vor Ort (01.10.2026) ─────────────────────────────────────────── */

#[derive(Serialize)]
pub struct ChatZeile {
    id: String,
    name: String,
    text: String,
    at: String,
    /// Von mir (meiner Person).
    ich: bool,
}

fn chat_zeile(
    n: openany_nahbereich::chat::Nachricht,
    ich: &openany_nahbereich::Person,
) -> ChatZeile {
    ChatZeile {
        ich: n.personen_id == ich.personen_id || ich.frueher.contains(&n.personen_id),
        id: n.id,
        name: n.name,
        text: n.text,
        at: n.at,
    }
}

/// Die Nachrichten eines lokalen Projekts, aelteste zuerst.
#[tauri::command]
pub async fn projekt_chat(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
) -> Result<Vec<ChatZeile>, String> {
    use openany_nahbereich::Gastgeber;
    let ich = zustand
        .gastgeber
        .person()
        .ok_or("This device has no person yet.")?;
    let alle =
        openany_nahbereich::projektnah::chat_lesen(&*zustand.speicher.lock().await, &projekt)?;
    Ok(alle.into_iter().map(|n| chat_zeile(n, &ich)).collect())
}

/*
 * UNGELESEN IM CHAT (Tiffy, 02.10.2026: der Zaehler am Reiter blieb leer).
 * Bis wohin gelesen ist, merkt sich dieses Geraet als Projektsache
 * `chat_gelesen` -- still, sie reist weder zum Server noch zu Mitgliedern
 * (projektnah::PLANUNG nennt sie nicht): Gelesen ist, was HIER gelesen ist.
 */
const CHAT_GELESEN: &str = "chat_gelesen";

fn chat_gelesen_bis(speicher: &openany_store::Speicher, projekt: &str) -> Option<String> {
    speicher
        .projektsache(projekt, CHAT_GELESEN)
        .ok()
        .flatten()
        .map(|s| s.text("bis").to_string())
        .filter(|b| !b.is_empty())
}

fn spaeter_als(at: &str, bis: Option<&str>) -> bool {
    let Some(bis) = bis else { return true };
    match (
        chrono::DateTime::parse_from_rfc3339(at),
        chrono::DateTime::parse_from_rfc3339(bis),
    ) {
        (Ok(a), Ok(b)) => a > b,
        _ => at > bis,
    }
}

/// Wie viele Chat-Nachrichten anderer hier noch niemand gelesen hat.
#[tauri::command]
pub async fn projekt_chat_ungelesen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
) -> Result<usize, String> {
    use openany_nahbereich::Gastgeber;
    let Some(ich) = zustand.gastgeber.person() else {
        return Ok(0);
    };
    let speicher = zustand.speicher.lock().await;
    let bis = chat_gelesen_bis(&speicher, &projekt);
    let alle = openany_nahbereich::projektnah::chat_lesen(&speicher, &projekt)?;
    Ok(alle
        .into_iter()
        .map(|n| chat_zeile(n, &ich))
        .filter(|z| !z.ich && spaeter_als(&z.at, bis.as_deref()))
        .count())
}

/// Der Chat ist gelesen -- bis zur juengsten Nachricht.
#[tauri::command]
pub async fn projekt_chat_gelesen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
) -> Result<(), String> {
    let speicher = zustand.speicher.lock().await;
    let alle = openany_nahbereich::projektnah::chat_lesen(&speicher, &projekt)?;
    let Some(juengste) = alle.iter().map(|n| n.at.clone()).max() else {
        return Ok(());
    };
    if !spaeter_als(&juengste, chat_gelesen_bis(&speicher, &projekt).as_deref()) {
        return Ok(());
    }
    speicher
        .projektsache_schreiben(
            &openany_store::Projektsache {
                projekt: projekt.clone(),
                art: CHAT_GELESEN.into(),
                uuid: CHAT_GELESEN.into(),
                eltern: None,
                felder: serde_json::json!({ "bis": juengste }),
                geaendert_at: String::new(),
            },
            openany_store::Protokoll::Still,
        )
        .map_err(fehler)
}

/// Eine Nachricht schreiben: hier ablegen und SOFORT an jedes Mitgliedsgeraet
/// in der Naehe schicken (im Hintergrund -- ein Geraet, das nicht antwortet,
/// haelt das Schreiben nicht auf). Wer nicht da ist, bekommt sie beim
/// naechsten Abgleich.
#[tauri::command]
pub async fn projekt_chat_senden(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    text: String,
) -> Result<ChatZeile, String> {
    use openany_nahbereich::Gastgeber;
    let text = text.trim().to_string();
    if text.is_empty() {
        return Err("Empty message.".into());
    }
    let g = zustand.gastgeber.clone();
    let ich = g.person().ok_or("This device has no person yet.")?;
    let ident = crate::geraet_identitaet(&zustand)?;
    let (nachricht, m) = {
        let speicher = zustand.speicher.lock().await;
        let (_, liste, _) = openany_nahbereich::projektnah::lesen(&speicher, &projekt)?;
        let m = liste.mitglieder(&|id| g.person_von(id)).map_err(fehler)?;
        let selbst = m
            .iter()
            .find(|x| x.personen_id == ich.personen_id || ich.frueher.contains(&x.personen_id))
            .ok_or("You are not a member of this project.")?;
        let n = openany_nahbereich::chat::Nachricht::schreiben(
            &ident,
            &projekt,
            &selbst.personen_id,
            &selbst.name,
            &text,
        )
        .map_err(fehler)?;
        openany_nahbereich::projektnah::chat_aufnehmen(
            &speicher,
            g.as_ref(),
            &projekt,
            std::slice::from_ref(&n),
        )?;
        (n, m)
    };

    let geraete: Vec<openany_nahbereich::GeraetInDerNaehe> = zustand
        .nah
        .lock()
        .await
        .as_ref()
        .map(|n| n.geraete())
        .unwrap_or_default()
        .into_iter()
        .filter(|d| d.openany && d.fingerabdruck != ident.fingerabdruck)
        .filter(|d| {
            openany_nahbereich::freigaben::mitglied_von(&m, &d.fingerabdruck, &|id| {
                g.person_von(id)
            })
            .is_some()
        })
        .collect();
    let raus = nachricht.clone();
    let projekt_raus = projekt.clone();
    tokio::spawn(async move {
        for d in geraete {
            let _ = openany_nahbereich::anruf::chat_senden(
                &ident,
                &d.adresse,
                openany_nahbereich::DIENST_PORT,
                &d.fingerabdruck,
                &projekt_raus,
                std::slice::from_ref(&raus),
            )
            .await;
        }
    });
    Ok(chat_zeile(nachricht, &ich))
}

/// Ein lokales Projekt loeschen -- auf diesem Geraet. Nur fuer Projekte, die
/// es nur hier gibt; eines vom Server verlaesst man in der Webapp.
#[tauri::command]
pub async fn projekt_lokal_loeschen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
) -> Result<(), String> {
    let speicher = zustand.speicher.lock().await;
    let ist_lokal = speicher
        .projekt(&projekt)
        .map_err(fehler)?
        .is_some_and(|p| p.mitgliederliste.is_some());
    if !ist_lokal {
        return Err("Only a local project can be deleted here.".into());
    }
    speicher
        .projekt_vergessen("lokal", &projekt)
        .map_err(fehler)?;
    Ok(())
}

/* ── Freigaben in lokale Projekte (01.10.2026) ─────────────────────────── */
//
// Notiz-Mappen, Ordner (Dateien, Dokumente) und Alben, erst nur lesen; was
// andere freigeben, liegt im Projekt (Tiffy, 01.10.2026). Die Regeln stehen
// in openany-nahbereich (freigaben.rs, projektnah.rs). Bytes kommen beim
// Oeffnen vom Geraet dessen, der freigegeben hat, und bleiben dann hier.

#[derive(Serialize)]
pub struct GeteiltesAnzeige {
    id: String,
    name: String,
    /// Der Weg unterhalb der Freigabe (Unterordner, Unteralben, Mappe).
    pfad: String,
    mime: String,
    groesse: u64,
    /// Liegen die Bytes auf diesem Geraet?
    da: bool,
    /// Nur bei Bildern: Liegt das Vorschaubild hier?
    vorschau_da: bool,
}

#[derive(Serialize)]
pub struct LokaleFreigabeAnzeige {
    /// `note_folder`, `file_folder`, `album`.
    art: String,
    /// `read` oder `edit` (nur Notiz-Mappen).
    stufe: String,
    schluessel: String,
    name: String,
    /// Bei Ordnern: `files` oder `documents` (soweit bekannt).
    zone: String,
    von: String,
    eigen: bool,
    eintraege: Vec<GeteiltesAnzeige>,
}

#[derive(Serialize)]
pub struct Wahl {
    art: String,
    schluessel: String,
    name: String,
    /// Wo es liegt, zum Unterscheiden gleicher Namen.
    ort: String,
}

#[derive(Serialize)]
pub struct LokaleFreigaben {
    freigaben: Vec<LokaleFreigabeAnzeige>,
    /// Was sich von dieser Person noch freigeben laesst.
    waehlbar: Vec<Wahl>,
}

fn zonenname(zone: &str) -> &'static str {
    if zone == "documents" {
        "Dokumente"
    } else {
        "Dateien"
    }
}

#[tauri::command]
pub async fn projekt_lokale_freigaben(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
) -> Result<LokaleFreigaben, String> {
    use openany_nahbereich::freigaben::{geltende, ALBUM, NOTIZ_MAPPE, ORDNER};
    use openany_nahbereich::projektnah::{GETEILTES_BILD, GETEILTE_DATEI, GETEILTE_NOTIZ};
    use openany_nahbereich::Gastgeber;
    let g = zustand.gastgeber.clone();
    let ich = g.person().ok_or("This device has no person yet.")?;
    let speicher = zustand.speicher.lock().await;
    let (_, liste, freigaben) = openany_nahbereich::projektnah::lesen(&speicher, &projekt)?;
    let m = liste.mitglieder(&|id| g.person_von(id)).map_err(fehler)?;
    let (n, d, b) = openany_nahbereich::projektnah::meine_geteilten(
        &speicher,
        g.as_ref(),
        &projekt,
        &m,
        &freigaben,
    )?;
    let hat = |a: &Option<String>| a.as_deref().is_some_and(|a| zustand.inhalte.hat(a));

    let mut aus = Vec::new();
    for f in geltende(&freigaben, &projekt, &m, &|id| g.person_von(id)) {
        let eigen = f.personen_id == ich.personen_id || ich.frueher.contains(&f.personen_id);
        let von = m
            .iter()
            .find(|x| x.personen_id == f.personen_id)
            .map(|x| x.name.clone())
            .unwrap_or_default();
        let mut eintraege: Vec<GeteiltesAnzeige> = if eigen {
            match f.art.as_str() {
                NOTIZ_MAPPE => n
                    .iter()
                    .filter(|x| {
                        openany_nahbereich::freigaben::in_mappe(x.mappe.as_deref(), &f.schluessel)
                    })
                    .map(|x| GeteiltesAnzeige {
                        id: x.zk_id.clone(),
                        name: x.titel.clone(),
                        pfad: String::new(),
                        mime: "text/markdown".into(),
                        groesse: 0,
                        da: true,
                        vorschau_da: false,
                    })
                    .collect(),
                art => (if art == ALBUM { &b } else { &d })
                    .iter()
                    .filter(|x| x.freigabe == f.schluessel)
                    .map(|x| GeteiltesAnzeige {
                        id: x.uuid.clone(),
                        name: x.name.clone(),
                        pfad: x.pfad.clone(),
                        mime: x.mime.clone(),
                        groesse: x.groesse,
                        da: hat(&x.abdruck),
                        vorschau_da: hat(&x.vorschau),
                    })
                    .collect(),
            }
        } else {
            let art = match f.art.as_str() {
                NOTIZ_MAPPE => GETEILTE_NOTIZ,
                ALBUM => GETEILTES_BILD,
                _ => GETEILTE_DATEI,
            };
            speicher
                .projektsachen(&projekt, art)
                .map_err(fehler)?
                .into_iter()
                .filter(|s| s.text("von") == f.personen_id)
                .filter(|s| {
                    if art == GETEILTE_NOTIZ {
                        openany_nahbereich::freigaben::in_mappe(
                            s.felder.get("mappe").and_then(|v| v.as_str()),
                            &f.schluessel,
                        )
                    } else {
                        s.text("freigabe") == f.schluessel
                    }
                })
                .map(|s| {
                    let feld =
                        |k: &str| s.felder.get(k).and_then(|v| v.as_str()).map(str::to_string);
                    GeteiltesAnzeige {
                        id: s.uuid.clone(),
                        name: if art == GETEILTE_NOTIZ {
                            s.text("titel")
                        } else {
                            s.text("name")
                        }
                        .to_string(),
                        pfad: s.text("pfad").to_string(),
                        mime: s.text("mime").to_string(),
                        groesse: s.zahl("groesse") as u64,
                        da: art == GETEILTE_NOTIZ || hat(&feld("abdruck")),
                        vorschau_da: hat(&feld("vorschau")),
                    }
                })
                .collect()
        };
        eintraege.sort_by_key(|e| (e.pfad.to_lowercase(), e.name.to_lowercase()));
        let zone = if f.art == ORDNER {
            if eigen {
                speicher
                    .datei(&f.schluessel)
                    .ok()
                    .flatten()
                    .map(|o| o.zone)
                    .unwrap_or_default()
            } else {
                speicher
                    .projektsachen(&projekt, GETEILTE_DATEI)
                    .ok()
                    .and_then(|l| l.into_iter().find(|s| s.text("freigabe") == f.schluessel))
                    .map(|s| s.text("zone").to_string())
                    .unwrap_or_default()
            }
        } else {
            String::new()
        };
        aus.push(LokaleFreigabeAnzeige {
            stufe: f.stufe.clone(),
            art: f.art.clone(),
            schluessel: f.schluessel.clone(),
            name: f.name.clone(),
            zone,
            von,
            eigen,
            eintraege,
        });
    }

    // Was sich noch freigeben laesst: eigene Mappen, Ordner, Alben.
    let schon: Vec<(String, String)> = aus
        .iter()
        .filter(|f| f.eigen)
        .map(|f| (f.art.clone(), f.schluessel.clone()))
        .collect();
    let frei = |art: &str, k: &str| !schon.iter().any(|(a, s)| a == art && s == k);
    let mut waehlbar: Vec<Wahl> = speicher
        .mappen()
        .map_err(fehler)?
        .into_iter()
        .filter(|x| frei(NOTIZ_MAPPE, &x.pfad))
        .map(|x| Wahl {
            art: NOTIZ_MAPPE.into(),
            ort: format!("Notizen/{}", x.pfad),
            schluessel: x.pfad,
            name: x.name,
        })
        .collect();
    for zone in ["files", "documents"] {
        for o in speicher.ordner_der_zone(zone).map_err(fehler)? {
            if o.papierkorb_at.is_some() || !frei(ORDNER, &o.uuid) {
                continue;
            }
            let weg: Vec<String> = speicher
                .ordnerweg(Some(&o.uuid))
                .map_err(fehler)?
                .into_iter()
                .map(|x| x.name)
                .collect();
            waehlbar.push(Wahl {
                art: ORDNER.into(),
                schluessel: o.uuid,
                name: o.name,
                ort: format!("{}/{}", zonenname(zone), weg.join("/")),
            });
        }
    }
    for a in speicher.alle_alben().map_err(fehler)? {
        if a.papierkorb_at.is_some() || !frei(ALBUM, &a.uuid) {
            continue;
        }
        let mut weg: Vec<String> = speicher
            .albumweg(&a.uuid)
            .map_err(fehler)?
            .into_iter()
            .map(|x| x.name)
            .collect();
        weg.push(a.name.clone());
        waehlbar.push(Wahl {
            art: ALBUM.into(),
            schluessel: a.uuid,
            name: a.name,
            ort: format!("Galerie/{}", weg.join("/")),
        });
    }
    Ok(LokaleFreigaben {
        freigaben: aus,
        waehlbar,
    })
}

/// Etwas Eigenes in ein lokales Projekt freigeben -- oder die Freigabe
/// zuruecknehmen (`freigeben` = false).
#[tauri::command]
pub async fn projekt_lokal_freigeben(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    art: String,
    schluessel: String,
    name: String,
    freigeben: bool,
    bearbeiten: Option<bool>,
) -> Result<(), String> {
    use openany_nahbereich::freigaben::{Freigabe, ALBUM, BEARBEITEN, LESEN, NOTIZ_MAPPE, ORDNER};
    use openany_nahbereich::Gastgeber;
    if ![NOTIZ_MAPPE, ORDNER, ALBUM].contains(&art.as_str()) {
        return Err("Unknown kind.".into());
    }
    let g = zustand.gastgeber.clone();
    let ich = g.person().ok_or("This device has no person yet.")?;
    let ident = crate::geraet_identitaet(&zustand)?;
    let speicher = zustand.speicher.lock().await;
    let (_, liste, mut freigaben) = openany_nahbereich::projektnah::lesen(&speicher, &projekt)?;
    let m = liste.mitglieder(&|id| g.person_von(id)).map_err(fehler)?;
    let meine_id = m
        .iter()
        .find(|x| x.personen_id == ich.personen_id || ich.frueher.contains(&x.personen_id))
        .map(|x| x.personen_id.clone())
        .ok_or("You are not a member of this project.")?;
    // Freigeben laesst sich nur, was hier liegt -- es ist ja das Eigene.
    let da = match art.as_str() {
        NOTIZ_MAPPE => true,
        ORDNER => speicher
            .datei(&schluessel)
            .map_err(fehler)?
            .is_some_and(|d| d.ist_ordner),
        _ => speicher.album(&schluessel).map_err(fehler)?.is_some(),
    };
    if freigeben && !da {
        return Err("This does not exist on this device.".into());
    }
    let f = Freigabe::unterschreiben(
        &ident,
        &projekt,
        &meine_id,
        &art,
        &schluessel,
        &name,
        match (freigeben, art == NOTIZ_MAPPE && bearbeiten.unwrap_or(false)) {
            (false, _) => "",
            (true, true) => BEARBEITEN,
            (true, false) => LESEN,
        },
    )
    .map_err(fehler)?;
    if !f.gilt(&projekt, &m, &|id| g.person_von(id)) {
        return Err("This share would not be valid.".into());
    }
    freigaben.push(f);
    speicher
        .projekt_freigaben_schreiben(
            &projekt,
            &serde_json::to_string(&freigaben).map_err(fehler)?,
        )
        .map_err(fehler)
}

#[derive(Serialize)]
pub struct GeteilteNotizInhalt {
    titel: String,
    inhalt: String,
    von: String,
}

/// Eine Notiz aus einer Freigabe lesen: eine eigene aus dem Notizbuch, eine
/// fremde aus dem Projekt.
#[tauri::command]
pub async fn projekt_geteilte_notiz(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    id: String,
) -> Result<GeteilteNotizInhalt, String> {
    let speicher = zustand.speicher.lock().await;
    if let Some(s) = speicher.projektsache(&projekt, &id).map_err(fehler)? {
        return Ok(GeteilteNotizInhalt {
            titel: s.text("titel").to_string(),
            inhalt: s.text("inhalt").to_string(),
            von: s.text("von_name").to_string(),
        });
    }
    let n = speicher
        .notiz(&id)
        .map_err(fehler)?
        .ok_or("This note does not exist here.")?;
    Ok(GeteilteNotizInhalt {
        titel: n.titel,
        inhalt: n.inhalt,
        von: String::new(),
    })
}

/// Die Geraete in der Naehe, die zu dieser Person gehoeren (Anker und Kette).
pub(crate) async fn geraete_von(
    zustand: &Zustand,
    personen_id: &str,
    anker: &str,
) -> Vec<openany_nahbereich::GeraetInDerNaehe> {
    use openany_nahbereich::Gastgeber;
    let gueltig: Vec<String> = zustand
        .gastgeber
        .person_von(personen_id)
        .map(|p| p.gueltige(&[anker]).into_keys().collect())
        .unwrap_or_default();
    zustand
        .nah
        .lock()
        .await
        .as_ref()
        .map(|n| n.geraete())
        .unwrap_or_default()
        .into_iter()
        .filter(|d| d.fingerabdruck == anker || gueltig.contains(&d.fingerabdruck))
        .collect()
}

/// Die Bytes einer freigegebenen Datei oder eines Bilds (oder seines
/// Vorschaubilds), als Base64. Eigene kommen aus der eigenen Ablage; fremde
/// aus dem Projekt -- und liegen sie noch nicht hier, vom Geraet dessen, der
/// sie freigegeben hat, wenn es in der Naehe ist.
#[tauri::command]
pub async fn projekt_geteilter_inhalt(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    id: String,
    vorschau: bool,
) -> Result<String, String> {
    use base64::Engine;
    use openany_nahbereich::Gastgeber;
    let (abdruck, groesse, von) = {
        let speicher = zustand.speicher.lock().await;
        if let Some(s) = speicher.projektsache(&projekt, &id).map_err(fehler)? {
            let feld = if vorschau { "vorschau" } else { "abdruck" };
            let a = s
                .felder
                .get(feld)
                .and_then(|v| v.as_str())
                .ok_or("There is no content for this.")?
                .to_string();
            let g = if vorschau {
                u64::MAX
            } else {
                s.zahl("groesse") as u64
            };
            (a, g, Some(s.text("von").to_string()))
        } else if let Some(d) = speicher.datei(&id).map_err(fehler)? {
            (
                d.abdruck.ok_or("There is no content for this.")?,
                d.groesse,
                None,
            )
        } else if let Some(b) = speicher.bild(&id).map_err(fehler)? {
            let a = if vorschau { b.vorschau } else { b.abdruck };
            (a.ok_or("There is no content for this.")?, b.groesse, None)
        } else {
            return Err("This does not exist here.".into());
        }
    };

    if !zustand.inhalte.hat(&abdruck) {
        let Some(von) = von else {
            // Eigenes, das hier nur „bei Bedarf" liegt: holen wie beim
            // eigenen Oeffnen (Server oder gepaartes Geraet).
            crate::dateibefehle::von_geraeten_holen(zustand.inner(), &abdruck, groesse).await?;
            let bytes = zustand
                .inhalte
                .lesen(&abdruck, 0, usize::MAX)
                .map_err(fehler)?;
            return Ok(base64::engine::general_purpose::STANDARD.encode(bytes));
        };
        // Wer hat freigegeben, und welches seiner Geraete ist in der Naehe?
        let g = zustand.gastgeber.clone();
        let anker = {
            let speicher = zustand.speicher.lock().await;
            let (_, liste, _) = openany_nahbereich::projektnah::lesen(&speicher, &projekt)?;
            liste
                .mitglieder(&|id| g.person_von(id))
                .map_err(fehler)?
                .into_iter()
                .find(|m| m.personen_id == von)
                .map(|m| m.geraet)
                .ok_or("Whoever shared this is no longer a member.")?
        };
        let geraete = geraete_von(&zustand, &von, &anker).await;
        if geraete.is_empty() {
            return Err(
                "Not on this device yet -- the device that shared it is not nearby right now."
                    .into(),
            );
        }
        let ident = crate::geraet_identitaet(&zustand)?;
        let mut letzter = String::new();
        for d in geraete {
            match openany_nahbereich::anruf::projekt_inhalt_holen(
                &ident,
                &d.adresse,
                openany_nahbereich::DIENST_PORT,
                &d.fingerabdruck,
                &projekt,
                &abdruck,
                groesse,
                &zustand.inhalte,
            )
            .await
            {
                Ok(()) => break,
                Err(e) => letzter = e,
            }
        }
        if !zustand.inhalte.hat(&abdruck) {
            return Err(letzter);
        }
    }
    let bytes = zustand
        .inhalte
        .lesen(&abdruck, 0, usize::MAX)
        .map_err(fehler)?;
    Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
}

#[derive(Serialize)]
pub struct NahProjektlauf {
    projekt: String,
    name: String,
    mit: String,
    neue_mitglieder: usize,
    notizen: usize,
    dateien: usize,
    bilder: usize,
    /// Dieses Geraet wurde aus dem Projekt entfernt -- es ist jetzt fort.
    entfernt: bool,
    /// Neue Chat-Nachrichten.
    chat: usize,
    /// Sachen der Planung, die kamen, sich aenderten oder gingen.
    planung: usize,
    fehler: Option<String>,
}

/// Lokale Projekte mit den Geraeten in der Naehe abgleichen, die zu einem
/// Mitglied gehoeren (projektnah.rs). Vorschaubilder kommen gleich mit.
#[tauri::command]
pub async fn projekte_nah_abgleichen(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Vec<NahProjektlauf>, String> {
    use openany_nahbereich::Gastgeber;
    let g = zustand.gastgeber.clone();
    let ident = crate::geraet_identitaet(&zustand)?;
    let geraete: Vec<openany_nahbereich::GeraetInDerNaehe> = zustand
        .nah
        .lock()
        .await
        .as_ref()
        .map(|n| n.geraete())
        .unwrap_or_default()
        .into_iter()
        .filter(|d| d.openany && d.fingerabdruck != ident.fingerabdruck)
        .collect();
    let lokale: Vec<Projekt> = zustand
        .speicher
        .lock()
        .await
        .projekte()
        .map_err(fehler)?
        .into_iter()
        .filter(|p| p.mitgliederliste.is_some())
        .collect();

    let mut laeufe = Vec::new();
    for p in lokale {
        for d in &geraete {
            // Gehoert das Geraet zu einem Mitglied? Sonst gar nicht fragen.
            let mitglied = {
                let speicher = zustand.speicher.lock().await;
                openany_nahbereich::projektnah::lesen(&speicher, &p.uuid)
                    .ok()
                    .and_then(|(_, liste, _)| liste.mitglieder(&|id| g.person_von(id)).ok())
                    .is_some_and(|m| {
                        openany_nahbereich::freigaben::mitglied_von(&m, &d.fingerabdruck, &|id| {
                            g.person_von(id)
                        })
                        .is_some()
                    })
            };
            if !mitglied {
                continue;
            }
            let ergebnis = match openany_nahbereich::anruf::projekt_stand(
                &ident,
                &d.adresse,
                openany_nahbereich::DIENST_PORT,
                &d.fingerabdruck,
                &p.uuid,
            )
            .await
            {
                Ok(stand) => {
                    let bericht = {
                        let speicher = zustand.speicher.lock().await;
                        openany_nahbereich::projektnah::stand_uebernehmen(
                            &speicher,
                            g.as_ref(),
                            &p.uuid,
                            &d.fingerabdruck,
                            &stand,
                        )
                    };
                    // Vorschaubilder gleich mit: klein, und ohne sie ist ein
                    // Album nur eine Liste von Namen.
                    if bericht.is_ok() {
                        for b in &stand.bilder {
                            if let Some(v) = &b.vorschau {
                                let _ = openany_nahbereich::anruf::projekt_inhalt_holen(
                                    &ident,
                                    &d.adresse,
                                    openany_nahbereich::DIENST_PORT,
                                    &d.fingerabdruck,
                                    &p.uuid,
                                    v,
                                    u64::MAX,
                                    &zustand.inhalte,
                                )
                                .await;
                            }
                        }
                    }
                    bericht
                }
                Err(e) => Err(e),
            };
            laeufe.push(match ergebnis {
                Ok(b) => NahProjektlauf {
                    projekt: p.uuid.clone(),
                    name: p.name.clone(),
                    mit: d.name.clone(),
                    neue_mitglieder: b.neue_mitglieder,
                    notizen: b.notizen,
                    dateien: b.dateien,
                    bilder: b.bilder,
                    entfernt: b.nicht_mehr_mitglied,
                    chat: b.chat,
                    planung: b.planung,
                    fehler: None,
                },
                Err(e) => NahProjektlauf {
                    projekt: p.uuid.clone(),
                    name: p.name.clone(),
                    mit: d.name.clone(),
                    neue_mitglieder: 0,
                    notizen: 0,
                    dateien: 0,
                    bilder: 0,
                    entfernt: false,
                    chat: 0,
                    planung: 0,
                    fehler: Some(e),
                },
            });
            // Entfernt: Das Projekt ist hier fort (was schon da war, inklusive).
            if laeufe.last().is_some_and(|l| l.entfernt) {
                zustand
                    .speicher
                    .lock()
                    .await
                    .projekt_vergessen("lokal", &p.uuid)
                    .map_err(fehler)?;
                break;
            }
        }
    }
    Ok(laeufe)
}
