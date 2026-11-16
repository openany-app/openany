//! Der Buerge fuer die weiteren Geraete eines Mitglieds (docs/plan-buergen.md).
//!
//! Die Mitgliederliste eines Hubs nennt je Person ein Geraet. Fuer die
//! weiteren buergt hier das gesiegelte Verzeichnis ihres Kontos: Jedes Geraet
//! hat unterschrieben „ich bin im Nahbereich Person P“ (`personenbindung`),
//! und gezaehlt wird nur, was ein gesiegeltes Geraet des Kontos so
//! unterschrieben hat (`direktpost::verzeichnis_vertraut`).
//!
//! **Das Konto kommt aus der Mitgliederliste**, nie vom Server. Gemerkt wird
//! je Konto, welche Person seine Geraete nennen und welche Geraete das sind;
//! gebuergt wird nur, wenn die Liste dem Mitglied genau dieses Konto UND genau
//! diese Personen-Id zuschreibt. Die Personen-Id allein traegt nichts: Die
//! nennt jedes Konto in seinen Bindungen selbst, auch die eines anderen.
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

/// Was das gesiegelte Verzeichnis eines Kontos sagt.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct Konto {
    /// Der Kontoname, wie er geschrieben wird.
    name: String,
    /// Die Person, die seine Geraete nennen; leer, wenn keines eine gueltige
    /// Bindung hat.
    personen_id: String,
    /// Die Fingerabdruecke dieser Geraete.
    geraete: Vec<String>,
    /// Wann zuletzt gefragt (Unix-Sekunden).
    gefragt: u64,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct Stand {
    /// Konto (klein geschrieben) → sein Verzeichnis.
    verzeichnisse: HashMap<String, Konto>,
}

impl Stand {
    /// Buergt das Verzeichnis von `konto` fuer `fp` als Geraet der Person
    /// `personen_id`? Beides muss stimmen: das Konto und die Person.
    fn buergt(&self, konto: &str, personen_id: &str, fp: &str) -> bool {
        !personen_id.is_empty()
            && self.verzeichnisse.get(&schluessel(konto)).is_some_and(|k| {
                k.personen_id == personen_id && k.geraete.iter().any(|x| x.eq_ignore_ascii_case(fp))
            })
    }

    /// Das Verzeichnis eines Kontos neu eintragen -- was es vorher buergte,
    /// gilt nicht mehr; ein zurueckgezogenes Geraet faellt so heraus.
    fn merken(
        &mut self,
        konto: String,
        person: Option<crate::personenbindung::FernePerson>,
        jetzt: u64,
    ) {
        let (personen_id, geraete) = person.map(|p| (p.personen_id, p.anker)).unwrap_or_default();
        self.verzeichnisse.insert(
            schluessel(&konto),
            Konto {
                name: konto,
                personen_id,
                geraete,
                gefragt: jetzt,
            },
        );
    }
}

fn schluessel(konto: &str) -> String {
    konto.trim().to_lowercase()
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

fn buergt(konto: &str, personen_id: &str, fp: &str) -> bool {
    STAND
        .read()
        .ok()
        .and_then(|s| Some(s.as_ref()?.buergt(konto, personen_id, fp)))
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

/// Das Konto zu einer Person -- NUR FUER DIE ANZEIGE, wenn ein Eintrag
/// keinen Namen traegt. Nennen mehrere Konten dieselbe Person, bleibt die
/// Antwort aus: Dann behauptet mindestens eines etwas Falsches.
pub(crate) fn konto_von(personen_id: &str) -> Option<String> {
    let stand = STAND.read().ok()?;
    let mut treffer = stand
        .as_ref()?
        .verzeichnisse
        .values()
        .filter(|k| !personen_id.is_empty() && k.personen_id == personen_id);
    let erstes = treffer.next()?;
    treffer.next().is_none().then(|| erstes.name.clone())
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
    // Nur die Konten der Liste. Was der Server als Mitglied nennt, zaehlt
    // hier nicht: Gebuergt wird fuer ein Mitglied allein aus dem Konto, das
    // die unterschriebene Liste ihm zuschreibt.
    let mut alle = konten(liste);
    alle.sort_by_key(|k| schluessel(k));
    alle.dedup_by_key(|k| schluessel(k));
    let faellig: Vec<String> = alle
        .into_iter()
        .filter(|k| {
            immer
                || alt
                    .verzeichnisse
                    .get(&schluessel(k))
                    .is_none_or(|v| jetzt().saturating_sub(v.gefragt) > FRISCH)
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
        s.merken(konto, person, jetzt());
    }
    schreiben(s);
}

/// Vor dem Aufnehmen: Gehoeren Eintraege zu Personen, fuer deren Geraet hier
/// noch niemand buergt, die Konten dieser Liste neu fragen.
pub(crate) async fn nachfragen(zustand: &Zustand, client: &OpenanyClient, liste: &Mitgliederliste) {
    auffrischen(zustand, client, liste, true).await;
}

#[cfg(test)]
mod tests {
    use super::Stand;
    use crate::personenbindung::FernePerson;

    fn person(id: &str, geraete: &[&str]) -> Option<FernePerson> {
        Some(FernePerson {
            personen_id: id.into(),
            anker: geraete.iter().map(|g| (*g).to_string()).collect(),
        })
    }

    #[test]
    fn das_konto_buergt_fuer_seine_geraete_und_seine_person() {
        let mut s = Stand::default();
        s.merken("Anna".into(), person("p-anna", &["FP-TEL", "fp-mac"]), 1);
        assert!(s.buergt("anna", "p-anna", "fp-tel"));
        assert!(s.buergt(" Anna ", "p-anna", "fp-mac"));
        assert!(!s.buergt("anna", "p-anna", "fp-fremd"));
        assert!(!s.buergt("anna", "p-andere", "fp-mac"));
    }

    /// Ein Konto, dessen Geraete die Personen-Id eines ANDEREN Mitglieds
    /// nennen, bekommt nichts -- und nimmt dem anderen nichts weg.
    #[test]
    fn eine_fremde_personen_id_buergt_nicht() {
        let mut s = Stand::default();
        s.merken("anna".into(), person("p-anna", &["fp-anna-mac"]), 1);
        s.merken("mallory".into(), person("p-anna", &["fp-mallory"]), 2);
        // Die Liste fuehrt Anna unter dem Konto anna: Mallorys Geraet zaehlt nicht.
        assert!(!s.buergt("anna", "p-anna", "fp-mallory"));
        // Annas eigenes weiteres Geraet zaehlt weiter.
        assert!(s.buergt("anna", "p-anna", "fp-anna-mac"));
        // Und fuer Mallory selbst (in der Liste mit ihrer Id) buergt es auch nicht.
        assert!(!s.buergt("mallory", "p-mallory", "fp-mallory"));
    }

    #[test]
    fn ein_zurueckgezogenes_geraet_faellt_heraus() {
        let mut s = Stand::default();
        s.merken("anna".into(), person("p-anna", &["fp-alt", "fp-neu"]), 1);
        s.merken("anna".into(), person("p-anna", &["fp-neu"]), 2);
        assert!(!s.buergt("anna", "p-anna", "fp-alt"));
        s.merken("anna".into(), None, 3);
        assert!(!s.buergt("anna", "p-anna", "fp-neu"));
        assert!(!s.buergt("anna", "", "fp-neu"));
    }
}
