//! Der Buerge fuer die weiteren Geraete eines Mitglieds (docs/plan-buergen.md).
//!
//! Die Mitgliederliste eines Hubs nennt je Person ein Geraet. Fuer die
//! weiteren buergt hier das gesiegelte Verzeichnis ihres Kontos: Jedes Geraet
//! hat unterschrieben „ich bin im Nahbereich Person P“ (`personenbindung`),
//! und gezaehlt wird nur, was ein gesiegeltes Geraet des Kontos so
//! unterschrieben hat (`direktpost::verzeichnis_vertraut`).
//!
//! **Das Konto kommt aus der Mitgliederliste**, nie vom Server als Absender
//! einer Nachricht. Gemerkt wird je Personen-Id der Bindung -- eine Liste, die
//! einer Person ein fremdes Konto zuschreibt, bekommt so nichts.
//!
//! Eingehaengt in `openany_nahbereich::freigaben` -- damit gilt es fuer
//! Mitgliederliste, Chat, Board und Freigaben zugleich. Aufgefrischt beim
//! Hub-Abgleich; ohne Netz gilt der zuletzt gemerkte Stand.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use openany_client::OpenanyClient;
use openany_nahbereich::Mitgliederliste;
use serde::{Deserialize, Serialize};

use crate::Zustand;

/// Wie lange ein Konto als frisch gilt -- danach fragt der naechste Abgleich
/// das Verzeichnis neu.
const FRISCH: u64 = 10 * 60;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct Stand {
    /// Personen-Id → Fingerabdruecke, fuer die ihr Konto buergt.
    personen: HashMap<String, Vec<String>>,
    /// Konto (klein geschrieben) → Personen-Id, wann zuletzt gefragt
    /// (Unix-Sekunden) und der Kontoname, wie er geschrieben wird.
    konten: HashMap<String, (String, u64, String)>,
}

static STAND: RwLock<Option<Stand>> = RwLock::new(None);
static PFAD: RwLock<Option<PathBuf>> = RwLock::new(None);

fn jetzt() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Beim Oeffnen des Zustands: den gemerkten Stand laden und den Buergen
/// einhaengen.
pub(crate) fn laden(ordner: &Path) {
    let pfad = ordner.join("buergen.json");
    let stand: Stand = std::fs::read(&pfad)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    *STAND.write().unwrap_or_else(|e| e.into_inner()) = Some(stand);
    *PFAD.write().unwrap_or_else(|e| e.into_inner()) = Some(pfad);
    openany_nahbereich::freigaben::buergen_einhaengen(Box::new(buergt));
}

/// Buergt das Verzeichnis fuer `fp` als Geraet der Person `personen_id`?
fn buergt(personen_id: &str, fp: &str) -> bool {
    STAND
        .read()
        .ok()
        .and_then(|s| {
            s.as_ref()?
                .personen
                .get(personen_id)
                .map(|a| a.iter().any(|x| x.eq_ignore_ascii_case(fp)))
        })
        .unwrap_or(false)
}

fn schreiben(stand: &Stand) {
    if let Some(p) = PFAD.read().ok().and_then(|p| p.clone()) {
        let _ = std::fs::write(p, serde_json::to_vec(stand).unwrap_or_default());
    }
}

/// Die Konten einer Liste: die Namen aus Gruendung und Beitritt. Bei Hubs auf
/// openany.de sind das Kontonamen.
fn konten(liste: &Mitgliederliste) -> Vec<String> {
    let mut k: Vec<String> = liste
        .eintraege
        .iter()
        .filter(|e| (e.art == "gruendung" || e.art == "beitritt") && !e.name.trim().is_empty())
        .map(|e| e.name.trim().to_string())
        .collect();
    k.sort();
    k.dedup();
    k
}

/// Das Konto zu einer Person, wie der Buerge es kennt -- fuer die Anzeige,
/// wenn ein Eintrag keinen Namen traegt.
pub(crate) fn konto_von(personen_id: &str) -> Option<String> {
    let stand = STAND.read().ok()?;
    stand
        .as_ref()?
        .konten
        .values()
        .find(|(pid, _, _)| pid == personen_id)
        .map(|(_, _, konto)| konto.clone())
}

/// Die Mitglieder des Hubs, wie der Server sie nennt (Kontonamen).
async fn server_konten(client: &OpenanyClient, projekt: &str) -> Vec<String> {
    client
        .projekt_json(&format!("/projects/{projekt}"), &[])
        .await
        .ok()
        .and_then(|k| k.get("members").and_then(|m| m.as_array()).cloned())
        .unwrap_or_default()
        .iter()
        .filter_map(|m| m.get("name").and_then(|n| n.as_str()).map(str::to_string))
        .filter(|n| !n.trim().is_empty())
        .collect()
}

/// Fuer die Konten einer Liste nachfragen, was ihr Verzeichnis buergt --
/// nur fuer die, die nicht frisch sind (`immer`: auch fuer die). Ein Fehler
/// (kein Netz) laesst den gemerkten Stand stehen.
pub(crate) async fn auffrischen(
    zustand: &Zustand,
    client: &OpenanyClient,
    liste: &Mitgliederliste,
    immer: bool,
) {
    let alt = STAND
        .read()
        .ok()
        .and_then(|s| s.clone())
        .unwrap_or_default();
    // Die Konten der Liste -- und die, die der Server als Mitglieder nennt:
    // Ein Gruendungseintrag traegt nicht immer den Kontonamen. Gemerkt wird
    // nur unter der Person, die das gesiegelte Verzeichnis selbst nennt; ein
    // eingeschobenes Konto bekommt so nichts.
    let mut alle = konten(liste);
    alle.extend(server_konten(client, &liste.projekt).await);
    alle.sort_by_key(|k| k.to_lowercase());
    alle.dedup_by_key(|k| k.to_lowercase());
    let faellig: Vec<String> = alle
        .into_iter()
        .filter(|k| {
            immer
                || alt
                    .konten
                    .get(&k.to_lowercase())
                    .is_none_or(|(_, t, _)| jetzt().saturating_sub(*t) > FRISCH)
        })
        .collect();
    if faellig.is_empty() {
        return;
    }
    let mut neu: Vec<(String, Option<crate::personenbindung::FernePerson>)> = Vec::new();
    for konto in faellig {
        match crate::personenbindung::pruefen(zustand, client, &konto).await {
            Ok(p) => neu.push((konto, p)),
            Err(e) => eprintln!("Buerge: Verzeichnis von {konto} nicht lesbar: {e}"),
        }
    }
    let mut stand = STAND.write().unwrap_or_else(|e| e.into_inner());
    let s = stand.get_or_insert_with(Stand::default);
    for (konto, person) in neu {
        let schluessel = konto.to_lowercase();
        // Was das Konto vorher buergte, gilt nicht mehr -- ein
        // zurueckgezogenes Geraet faellt so heraus.
        if let Some((pid, _, _)) = s.konten.get(&schluessel) {
            s.personen.remove(pid);
        }
        match person {
            Some(p) => {
                s.konten
                    .insert(schluessel, (p.personen_id.clone(), jetzt(), konto));
                s.personen.insert(p.personen_id, p.anker);
            }
            None => {
                s.konten.insert(schluessel, (String::new(), jetzt(), konto));
            }
        }
    }
    schreiben(s);
}

/// Vor dem Aufnehmen: Gehoeren Eintraege zu Personen, fuer deren Geraet hier
/// noch niemand buergt, die Konten dieser Liste neu fragen.
pub(crate) async fn nachfragen(zustand: &Zustand, client: &OpenanyClient, liste: &Mitgliederliste) {
    auffrischen(zustand, client, liste, true).await;
}
