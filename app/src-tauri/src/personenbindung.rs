//! Person am Konto (docs/plan-projekte-eine-art.md).
//!
//! **Wozu.** Wer jemanden aus der Ferne in ein Projekt einlaedt, muss die
//! PERSON hinter einem Konto kennen: ihre Personen-Id und ein Geraet, das
//! sicher zu ihr gehoert (der Anker in der Mitgliederliste). Vor Ort
//! bestaetigen das die 6 Ziffern. Aus der Ferne traegt es das
//! Schluesselverzeichnis der Konten.
//!
//! **Wie.** Jedes Geraet mit Konto unterschreibt mit seinem Schluessel aus
//! dem Verzeichnis: „mein TLS-Fingerabdruck ist im Nahbereich die Person P"
//! und legt das beim Server ab ([`melden`]). Wer einlaedt, holt Verzeichnis
//! und Bindungen und prueft jede Unterschrift selbst ([`pruefen`]). Der
//! Server prueft nichts; einem Verzeichnis, das er faelscht, hilft die
//! Sicherheitsnummer ab -- wie bei den Direktnachrichten.
//!
//! **Eine Person je Konto.** Geraete eines Kontos, die sich nie vor Ort
//! gepaart haben, haben verschiedene Personen-Ids. Beim Melden nimmt jedes die
//! kleinste, die ein freigegebenes eigenes Geraet gemeldet hat -- dieselbe
//! Regel wie beim Paaren (`Person::zusammenlegen`); die alte steht danach
//! unter `frueher`.

use std::collections::BTreeMap;
use std::sync::Arc;

use openany_client::OpenanyClient;
use openany_nahbereich::Gastgeber;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::direktpost::{self, Eintrag};
use crate::Zustand;

/// Eine Bindung, wie der Server sie liefert.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Bindung {
    pub geraet: i64,
    pub personen_id: String,
    pub fingerabdruck: String,
    pub signatur: String,
}

/// Die Person hinter einem Konto, aus der Ferne geprueft.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct FernePerson {
    pub personen_id: String,
    /// Fingerabdruecke der Geraete, fuer die das Verzeichnis buergt -- jeder
    /// taugt als Anker in der Mitgliederliste.
    pub anker: Vec<String>,
}

fn marke(zustand: &Zustand) -> std::path::PathBuf {
    zustand.ordner.join("personenbindung.json")
}

fn bindungen_aus(antwort: &Value) -> Vec<Bindung> {
    antwort
        .get("bindungen")
        .cloned()
        .and_then(|b| serde_json::from_value(b).ok())
        .unwrap_or_default()
}

/// Die Bindungen, deren Unterschrift zu einem aktiven Geraet im Verzeichnis
/// passt. `nur_freigegebene`: beim EIGENEN Konto zaehlt nur, wer den
/// Kontoschluessel hat -- ein Geraet, das noch auf Freigabe wartet, koennte
/// jedes sein.
fn gueltige<'a>(
    name: &str,
    geraete: &[Eintrag],
    bindungen: &'a [Bindung],
    nur_freigegebene: bool,
) -> Vec<&'a Bindung> {
    bindungen
        .iter()
        .filter(|b| {
            geraete.iter().any(|g| {
                g.geraet == b.geraet
                    && g.aktiv
                    && !g.abgelehnt
                    && (!nur_freigegebene || g.freigegeben)
                    && openany_direktpost::personenbindung_pruefen(
                        &g.ed25519,
                        name,
                        b.geraet,
                        &b.personen_id,
                        &b.fingerabdruck,
                        &b.signatur,
                    )
                    .is_ok()
            })
        })
        .collect()
}

/// Aus gueltigen Bindungen die Person: die kleinste Id (dieselbe Regel, nach
/// der sich die Geraete angleichen) und alle Geraete, die sie tragen.
fn person_aus(gueltige: &[&Bindung]) -> Option<FernePerson> {
    let mut nach_id: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for b in gueltige {
        nach_id
            .entry(b.personen_id.as_str())
            .or_default()
            .push(b.fingerabdruck.clone());
    }
    let (id, mut anker) = nach_id.into_iter().next()?;
    anker.sort();
    anker.dedup();
    Some(FernePerson {
        personen_id: id.to_string(),
        anker,
    })
}

/// Die Person hinter dem Konto `name`, geprueft gegen das Verzeichnis.
/// `None`, wenn keines ihrer Geraete eine gueltige Bindung hat (die App dort
/// noch nie mit openany.de abgeglichen).
pub(crate) async fn pruefen(
    zustand: &Zustand,
    client: &OpenanyClient,
    name: &str,
) -> Result<Option<FernePerson>, String> {
    let verzeichnis = direktpost::verzeichnis_vertraut(zustand, client, name)
        .await
        .map_err(|e| e.to_string())?;
    let antwort = client
        .personen_bindungen(name)
        .await
        .map_err(|e| e.to_string())?;
    let bindungen = bindungen_aus(&antwort);
    Ok(person_aus(&gueltige(
        name,
        &verzeichnis.geraete,
        &bindungen,
        false,
    )))
}

/// Was zuletzt gemeldet wurde -- gemeldet wird nur, was sich geaendert hat.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Gemeldet {
    geraet: i64,
    personen_id: String,
    fingerabdruck: String,
}

/// Nach dem Abgleich: die Personen-Id an die der eigenen Geraete angleichen
/// und die eigene Bindung melden. Still -- ein Fehler hier haelt keinen
/// Abgleich auf, beim naechsten Lauf geht es wieder.
pub(crate) async fn melden(zustand: &Arc<Zustand>, client: &OpenanyClient) {
    if let Err(e) = melden_oder_fehler(zustand, client).await {
        eprintln!("Personenbindung nicht gemeldet: {e}");
    }
}

async fn melden_oder_fehler(zustand: &Arc<Zustand>, client: &OpenanyClient) -> Result<(), String> {
    // Ohne Eintrag im Verzeichnis (Direktpost noch nicht bereit) und ohne
    // Person (Nahbereich nie gestartet) gibt es nichts zu binden.
    let (Some(geraet), Some(name)) = (
        direktpost::mein_geraet(zustand),
        direktpost::mein_name(zustand),
    ) else {
        return Ok(());
    };
    let Some(mut person) = zustand.gastgeber.person() else {
        return Ok(());
    };
    let ident = crate::geraet_identitaet(zustand)?;

    // Angleichen an die eigenen, freigegebenen Geraete.
    let verzeichnis = direktpost::verzeichnis_vertraut(zustand, client, &name)
        .await
        .map_err(|e| e.to_string())?;
    let antwort = client
        .personen_bindungen(&name)
        .await
        .map_err(|e| e.to_string())?;
    let bindungen = bindungen_aus(&antwort);
    let eigene = gueltige(&name, &verzeichnis.geraete, &bindungen, true);
    if let Some(kleinste) = eigene
        .iter()
        .filter(|b| b.geraet != geraet)
        .map(|b| b.personen_id.as_str())
        .min()
    {
        if kleinste < person.personen_id.as_str() {
            let alt = std::mem::replace(&mut person.personen_id, kleinste.to_string());
            if !person.frueher.contains(&alt) {
                person.frueher.push(alt);
            }
            person.frueher.retain(|f| f != kleinste);
            zustand.gastgeber.person_merken(person.clone());
        }
    }

    let jetzt = Gemeldet {
        geraet,
        personen_id: person.personen_id.clone(),
        fingerabdruck: ident.fingerabdruck.clone(),
    };
    let schon = std::fs::read_to_string(marke(zustand))
        .ok()
        .and_then(|t| serde_json::from_str::<Gemeldet>(&t).ok());
    // Auch melden, wenn der Server die eigene Bindung nicht (mehr) hat.
    let beim_server = bindungen.iter().any(|b| {
        b.geraet == geraet
            && b.personen_id == jetzt.personen_id
            && b.fingerabdruck == jetzt.fingerabdruck
    });
    if schon.as_ref() == Some(&jetzt) && beim_server {
        return Ok(());
    }

    let schluessel = direktpost::schluessel(zustand)?;
    let signatur = openany_direktpost::personenbindung_unterschreiben(
        &schluessel,
        &name,
        geraet,
        &jetzt.personen_id,
        &jetzt.fingerabdruck,
    );
    client
        .personen_bindung_setzen(&jetzt.personen_id, &jetzt.fingerabdruck, &signatur)
        .await
        .map_err(|e| e.to_string())?;
    let _ = std::fs::write(
        marke(zustand),
        serde_json::to_vec(&jetzt).unwrap_or_default(),
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use openany_direktpost::{personenbindung_unterschreiben, Schluessel};

    fn eintrag(geraet: i64, ed: &str, aktiv: bool, freigegeben: bool) -> Eintrag {
        serde_json::from_value(serde_json::json!({
            "geraet": geraet, "ed25519": ed, "aktiv": aktiv, "freigegeben": freigegeben,
        }))
        .unwrap()
    }

    fn bindung(s: &Schluessel, name: &str, geraet: i64, id: &str, fp: &str) -> Bindung {
        Bindung {
            geraet,
            personen_id: id.into(),
            fingerabdruck: fp.into(),
            signatur: personenbindung_unterschreiben(s, name, geraet, id, fp),
        }
    }

    #[test]
    fn nur_gueltige_unterschriften_aktiver_geraete_zaehlen() {
        let tablet = Schluessel::erzeugen().unwrap();
        let telefon = Schluessel::erzeugen().unwrap();
        let fremd = Schluessel::erzeugen().unwrap();
        let geraete = vec![
            eintrag(1, &tablet.oeffentlich().1, true, true),
            eintrag(2, &telefon.oeffentlich().1, false, true),
        ];
        let bindungen = vec![
            bindung(&tablet, "anna", 1, "b-person", "fp-tablet"),
            // Entferntes Geraet: zaehlt nicht mehr.
            bindung(&telefon, "anna", 2, "a-person", "fp-telefon"),
            // Fremder Schluessel unter Geraet 1: zaehlt nicht.
            bindung(&fremd, "anna", 1, "a-person", "fp-fremd"),
        ];
        let g = gueltige("anna", &geraete, &bindungen, false);
        assert_eq!(
            person_aus(&g),
            Some(FernePerson {
                personen_id: "b-person".into(),
                anker: vec!["fp-tablet".into()],
            })
        );
        // Fuer ein anderes Konto unterschrieben: nichts.
        assert!(gueltige("ben", &geraete, &bindungen, false).is_empty());
    }

    #[test]
    fn eigene_geraete_zaehlen_nur_freigegeben_und_die_kleinste_id_gilt() {
        let a = Schluessel::erzeugen().unwrap();
        let b = Schluessel::erzeugen().unwrap();
        let c = Schluessel::erzeugen().unwrap();
        let geraete = vec![
            eintrag(1, &a.oeffentlich().1, true, true),
            eintrag(2, &b.oeffentlich().1, true, true),
            // Wartet noch auf Freigabe.
            eintrag(3, &c.oeffentlich().1, true, false),
        ];
        let bindungen = vec![
            bindung(&a, "anna", 1, "p-2", "fp-a"),
            bindung(&b, "anna", 2, "p-1", "fp-b"),
            bindung(&c, "anna", 3, "p-0", "fp-c"),
        ];
        assert_eq!(gueltige("anna", &geraete, &bindungen, true).len(), 2);
        assert_eq!(
            person_aus(&gueltige("anna", &geraete, &bindungen, true))
                .unwrap()
                .personen_id,
            "p-1"
        );
    }
}
