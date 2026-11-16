//! Die unterschriebene Mitgliederliste fuer Projekte auf openany.de
//! (docs/plan-projekte-eine-art.md, E2).
//!
//! **Dieselbe Kette wie vor Ort** (`openany_nahbereich::Mitgliederliste`):
//! Gruendung, Beitritt, Rolle, Austritt, jeder Eintrag von einem Geraet einer
//! Eigentuemerin unterschrieben. Der Server bewahrt sie auf und nimmt nur
//! Fortsetzungen an; geprueft wird hier.
//!
//! **Was sie schuetzt.** Den Projektschluessel packt dieses Geraet nur fuer
//! Konten, die in der Liste stehen, und es nimmt Kisten und Inhalte eines
//! Hubs nur von solchen an ([`konto_mitglied`]). Ein Server, der sich selbst
//! als Mitglied eintraegt, steht in keiner gueltigen Liste -- er bekommt
//! keine Kiste und kann keine unterschieben.
//!
//! **Was sie sich merkt.** Die zuletzt gepruefte Liste (`server_kette`). Eine
//! neue vom Server muss sie fortsetzen; eine gekuerzte oder umgeschriebene
//! gilt nicht. Das traegt ab dem ersten Blick auf die Liste, nicht davor.
//!
//! **Namen sind Kontonamen.** Bei Projekten auf openany.de steht im Eintrag
//! der Login-Name des Kontos -- daran findet das Geraet die Person
//! (`personenbindung::pruefen`) und das Konto auf dem Server.
//!
//! **Grenze:** Als Geraet einer Eigentuemerin gilt das Geraet, mit dem sie in
//! die Liste kam, und was dafuer vor Ort buergt (gepaarte eigene Geraete).
//! Ein weiteres eigenes Geraet, das nur am Konto haengt, darf (noch) nicht
//! unterschreiben.

use std::sync::Arc;

use openany_client::{Method, OpenanyClient, OpenanyError};
use openany_nahbereich::mitglieder::{Mitgliedereintrag, EIGENTUEMER, MITGLIED};
use openany_nahbereich::{Gastgeber, Mitglied, Mitgliederliste};
use serde_json::{json, Value};

use crate::{direktpost, personenbindung, Zustand};

fn text(e: impl ToString) -> String {
    e.to_string()
}

fn pfad(projekt: &str) -> String {
    format!("/projects/{projekt}/mitgliederliste")
}

/// Die Mitglieder einer Liste -- mit den Personen, die dieses Geraet kennt.
pub(crate) fn mitglieder(
    zustand: &Zustand,
    liste: &Mitgliederliste,
) -> Result<Vec<Mitglied>, String> {
    let g = zustand.gastgeber.clone();
    liste.mitglieder(&|id| g.person_von(id)).map_err(text)
}

/// Setzt `neu` die gemerkte Liste fort? (Ohne gemerkte: ja.)
///
/// Verglichen wird ohne Leerraum an den Enden: Der Server hat Listen frueher
/// beschnitten gespeichert (das Zertifikat ohne seinen Zeilenumbruch). Die
/// Unterschrift deckt das Zertifikat nicht, die Eintraege sind dieselben.
pub(crate) fn setzt_fort(gemerkt: Option<&Mitgliederliste>, neu: &Mitgliederliste) -> bool {
    gemerkt.is_none_or(|g| {
        g.projekt == neu.projekt
            && neu.eintraege.len() >= g.eintraege.len()
            && neu.eintraege[..g.eintraege.len()]
                .iter()
                .zip(&g.eintraege)
                .all(|(a, b)| gleich_eintrag(a, b))
    })
}

fn gleich_eintrag(a: &Mitgliedereintrag, b: &Mitgliedereintrag) -> bool {
    let felder = |e: &Mitgliedereintrag| {
        [
            e.art.trim().to_string(),
            e.personen_id.trim().to_string(),
            e.name.trim().to_string(),
            e.rolle.trim().to_string(),
            e.geraet.trim().to_string(),
            e.at.trim().to_string(),
            e.vorher.trim().to_string(),
            e.unterzeichner.trim().to_string(),
            e.zertifikat_pem.trim().to_string(),
            e.signatur.trim().to_string(),
        ]
    };
    felder(a) == felder(b)
}

/// Die Liste vom Server holen, pruefen und merken. `None`: Das Projekt hat
/// (noch) keine -- es entstand vor E2.
pub(crate) async fn holen(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    projekt: &str,
) -> Result<Option<Mitgliederliste>, String> {
    let antwort = match client.projekt_json(&pfad(projekt), &[]).await {
        Ok(a) => a,
        // Ein Server vor E2 kennt den Weg nicht: wie ein Projekt ohne Liste.
        Err(OpenanyError::Abgelehnt { status, .. }) if status.as_u16() == 404 => return Ok(None),
        Err(e) => return Err(text(e)),
    };
    let Some(roh) = antwort.get("liste").filter(|l| !l.is_null()) else {
        // Der Server nennt keine -- kennt dieses Geraet eine, gilt die: Eine
        // Liste verschwindet nicht dadurch, dass der Server sie weglaesst.
        let speicher = zustand.speicher.lock().await;
        return Ok(speicher
            .projekt_server_kette(projekt)
            .map_err(text)?
            .and_then(|t| serde_json::from_str::<Mitgliederliste>(&t).ok()));
    };
    let liste: Mitgliederliste = serde_json::from_value(roh.clone()).map_err(text)?;
    if liste.projekt != projekt {
        return Err("The member list belongs to another project.".into());
    }
    // Erst fragen, wofuer die Konten der Liste buergen -- unterschrieb eine
    // Eigentuemerin von einem weiteren Geraet, gilt die Liste sonst nicht.
    crate::buergen::auffrischen(zustand, client, &liste, false).await;
    if mitglieder(zustand, &liste).is_err() {
        crate::buergen::auffrischen(zustand, client, &liste, true).await;
    }
    mitglieder(zustand, &liste)?;

    let speicher = zustand.speicher.lock().await;
    let gemerkt = speicher
        .projekt_server_kette(projekt)
        .map_err(text)?
        .and_then(|t| serde_json::from_str::<Mitgliederliste>(&t).ok());
    if !setzt_fort(gemerkt.as_ref(), &liste) {
        // Vor Ort weiter als der Server (E4: Wolke-Projekte gleichen auch vor
        // Ort ab)? Dann gilt die hiesige; eine Eigentuemerin bringt sie hoch.
        if let Some(g) = gemerkt.filter(|g| setzt_fort(Some(&liste), g)) {
            drop(speicher);
            let _ = hochladen(zustand, client, &g).await;
            return Ok(Some(g));
        }
        return Err(
            "The member list from the server does not continue the one this device knows.".into(),
        );
    }
    speicher
        .projekt_server_kette_schreiben(projekt, &serde_json::to_string(&liste).map_err(text)?)
        .map_err(text)?;
    Ok(Some(liste))
}

/// Hochladen und merken. `Ok(false)`: Jemand war schneller (409).
pub(crate) async fn hochladen(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    liste: &Mitgliederliste,
) -> Result<bool, String> {
    match client
        .projekt_senden(
            Method::PUT,
            &pfad(&liste.projekt),
            Some(json!({ "liste": liste })),
        )
        .await
    {
        Ok(_) => {}
        Err(OpenanyError::Abgelehnt { status, .. }) if status.as_u16() == 409 => return Ok(false),
        Err(e) => return Err(text(e)),
    }
    zustand
        .speicher
        .lock()
        .await
        .projekt_server_kette_schreiben(
            &liste.projekt,
            &serde_json::to_string(liste).map_err(text)?,
        )
        .map_err(text)?;
    crate::projektchat::zugehoerigkeit_vergessen(&liste.projekt);
    Ok(true)
}

/// Ein neues Projekt auf openany.de: die Gruendung, mit dem Kontonamen.
pub(crate) async fn gruenden(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    projekt: &str,
) -> Result<(), String> {
    let ident = crate::geraet_identitaet(zustand)?;
    let person = zustand
        .gastgeber
        .person()
        .ok_or("This device has no person yet -- has \"Nearby devices\" started?")?;
    let name = direktpost::mein_name(zustand).ok_or("This device has no account name yet.")?;
    let mut liste = Mitgliederliste {
        projekt: projekt.to_string(),
        eintraege: Vec::new(),
    };
    liste
        .eintragen(
            &ident,
            "gruendung",
            &person.personen_id,
            &name,
            EIGENTUEMER,
            &ident.fingerabdruck,
        )
        .map_err(text)?;
    if !hochladen(zustand, client, &liste).await? {
        return Err("The project already has a member list.".into());
    }
    Ok(())
}

/// Einen Eintrag anhaengen: holen, bauen, pruefen, hochladen -- bei 409 noch
/// einmal mit der frischen Liste. `None`: Das Projekt hat keine Liste (vor
/// E2) -- dann aendert sich nur der Server, wie bisher.
async fn anhaengen(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    projekt: &str,
    bauen: &Bauen<'_>,
) -> Result<Option<()>, String> {
    let ident = crate::geraet_identitaet(zustand)?;
    for _versuch in 0..2 {
        let Some(mut liste) = holen(zustand, client, projekt).await? else {
            return Ok(None);
        };
        let jetzt = mitglieder(zustand, &liste)?;
        let Some(e) = bauen(&jetzt)? else {
            return Ok(Some(()));
        };
        liste
            .eintragen(&ident, e.art, &e.personen_id, &e.name, e.rolle, &e.geraet)
            .map_err(text)?;
        // Gilt das? (Nur Eigentuemerinnen, die letzte bleibt.)
        mitglieder(zustand, &liste)?;
        if hochladen(zustand, client, &liste).await? {
            return Ok(Some(()));
        }
    }
    Err("The member list changed meanwhile -- please try again.".into())
}

/// Was [`anhaengen`] aus den jetzigen Mitgliedern baut -- `None`: nichts zu tun.
type Bauen<'a> = dyn Fn(&[Mitglied]) -> Result<Option<Eintrag>, String> + Sync + 'a;

struct Eintrag {
    art: &'static str,
    personen_id: String,
    name: String,
    rolle: &'static str,
    geraet: String,
}

fn gleich(a: &str, b: &str) -> bool {
    a.trim().eq_ignore_ascii_case(b.trim())
}

/// Das Mitglied hinter einem Konto: ueber seine Person, sonst ueber den
/// Namen. Wer vor Ort beitrat, steht unter dem Namen seiner Person in der
/// Liste, nicht unter dem Kontonamen (E3).
fn finden<'a>(
    jetzt: &'a [Mitglied],
    person: Option<&personenbindung::FernePerson>,
    name: &str,
) -> Option<&'a Mitglied> {
    person
        .and_then(|p| jetzt.iter().find(|m| m.personen_id == p.personen_id))
        .or_else(|| jetzt.iter().find(|m| gleich(&m.name, name)))
}

/// Den Hub aufloesen (docs/plan-hub-aufloesen.md): der letzte Eintrag der
/// Liste, von einer Eigentuemerin unterschrieben. ERST danach loescht der
/// Aufrufer beim Server -- an diesem Eintrag, nicht an der Loeschzeile des
/// Servers, erkennen die Geraete der Mitglieder, dass der Hub fort soll.
/// Ohne Liste (ein Hub von vor E2) gibt es nichts zu schreiben.
pub(crate) async fn aufloesen(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    projekt: &str,
) -> Result<(), String> {
    // Schon aufgeloest (das Loeschen danach schlug fehl): nichts anhaengen.
    if holen(zustand, client, projekt)
        .await?
        .is_none_or(|l| l.aufgeloest())
    {
        return Ok(());
    }
    let ident = crate::geraet_identitaet(zustand)?;
    let person = zustand
        .gastgeber
        .person()
        .ok_or("This device has no person yet -- has \"Nearby devices\" started?")?;
    anhaengen(zustand, client, projekt, &|_| {
        Ok(Some(Eintrag {
            art: openany_nahbereich::mitglieder::AUFLOESUNG,
            personen_id: person.personen_id.clone(),
            name: String::new(),
            rolle: "",
            geraet: ident.fingerabdruck.clone(),
        }))
    })
    .await
    .map(|_| ())
}

/// Sagt die Liste `neu`, dass der Hub auf DIESEM Geraet fort soll -- und
/// setzt sie alles fort, was das Geraet kennt? Fuer den Grabstein des
/// Servers (`ServerGegenstelle::projekt_fort_belegt`).
pub(crate) fn fort_belegt(
    neu: &Mitgliederliste,
    gemerkt: &[Mitgliederliste],
    jetzt: &[Mitglied],
    ich: &openany_nahbereich::Person,
) -> bool {
    gemerkt.iter().all(|g| setzt_fort(Some(g), neu))
        && (neu.aufgeloest()
            || !jetzt
                .iter()
                .any(|m| m.personen_id == ich.personen_id || ich.frueher.contains(&m.personen_id)))
}

/// Ein Konto aus der Ferne aufnehmen: seine Person pruefen, den Beitritt
/// unterschreiben. Erst danach nimmt der Server das Konto auf.
pub(crate) async fn einladen(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    projekt: &str,
    name: &str,
) -> Result<(), String> {
    let person = personenbindung::pruefen(zustand, client, name)
        .await?
        .ok_or_else(|| format!("{name} has not set up the openany app with openany.de yet -- they can be invited once they have synced."))?;
    let anker = person
        .anker
        .first()
        .cloned()
        .ok_or("No device to anchor the person.")?;
    anhaengen(zustand, client, projekt, &|jetzt| {
        if jetzt.iter().any(|m| m.personen_id == person.personen_id) {
            return Ok(None);
        }
        if jetzt.len() >= openany_nahbereich::mitglieder::MAX_MITGLIEDER {
            return Err(format!(
                "This hub already has {} members -- that is the limit.",
                openany_nahbereich::mitglieder::MAX_MITGLIEDER
            ));
        }
        Ok(Some(Eintrag {
            art: "beitritt",
            personen_id: person.personen_id.clone(),
            name: name.to_string(),
            rolle: MITGLIED,
            geraet: anker.clone(),
        }))
    })
    .await
    .map(|_| ())
}

/// Ein Konto aus der Liste nehmen (Entfernen durch eine Eigentuemerin).
pub(crate) async fn entfernen(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    projekt: &str,
    name: &str,
) -> Result<(), String> {
    let person = personenbindung::pruefen(zustand, client, name)
        .await
        .ok()
        .flatten();
    anhaengen(zustand, client, projekt, &|jetzt| {
        Ok(finden(jetzt, person.as_ref(), name).map(|m| Eintrag {
            art: "austritt",
            personen_id: m.personen_id.clone(),
            name: m.name.clone(),
            rolle: "",
            geraet: m.geraet.clone(),
        }))
    })
    .await
    .map(|_| ())
}

/// Die Rolle eines Kontos setzen.
pub(crate) async fn rolle(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    projekt: &str,
    name: &str,
    rolle: &str,
) -> Result<(), String> {
    let rolle: &'static str = match rolle {
        EIGENTUEMER => EIGENTUEMER,
        MITGLIED => MITGLIED,
        _ => return Err("Unknown role.".into()),
    };
    let person = personenbindung::pruefen(zustand, client, name)
        .await
        .ok()
        .flatten();
    anhaengen(zustand, client, projekt, &|jetzt| {
        let m = finden(jetzt, person.as_ref(), name)
            .ok_or_else(|| format!("{name} is not in the signed member list."))?;
        if m.rolle == rolle {
            return Ok(None);
        }
        Ok(Some(Eintrag {
            art: "rolle",
            personen_id: m.personen_id.clone(),
            name: m.name.clone(),
            rolle,
            geraet: m.geraet.clone(),
        }))
    })
    .await
    .map(|_| ())
}

/// Nach dem Abgleich, bei Projekten, in denen dieses Geraet unterschreiben
/// darf: Wer auf dem Server nicht mehr Mitglied ist (selbst gegangen), wird
/// auch aus der Liste ausgetragen. Still -- ein Fehler haelt nichts auf.
pub(crate) async fn nachziehen(zustand: &Arc<Zustand>, client: &OpenanyClient, projekt: &str) {
    let Ok(kopf) = client
        .projekt_json(&format!("/projects/{projekt}"), &[])
        .await
    else {
        return;
    };
    if kopf.get("my_role").and_then(Value::as_str) != Some(EIGENTUEMER) {
        return;
    }
    let auf_dem_server: Vec<String> = kopf
        .get("members")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|m| m.get("name").and_then(Value::as_str).map(str::to_string))
        .collect();
    let liste = match holen(zustand, client, projekt).await {
        Ok(Some(l)) => l,
        // Noch keine Liste, und die Eigentuemerin ist allein: jetzt gruenden.
        // Faengt ein Anlegen auf, bei dem die Gruendung nicht durchkam.
        Ok(None) if auf_dem_server.len() == 1 => {
            if let Err(e) = gruenden(zustand, client, projekt).await {
                eprintln!("Mitgliederliste nicht gegruendet ({projekt}): {e}");
            }
            return;
        }
        _ => return,
    };
    let Ok(jetzt) = mitglieder(zustand, &liste) else {
        return;
    };
    // Wer nur vor Ort dabei ist (ohne Konto), steht nicht auf dem Server --
    // und bleibt. Ausgetragen werden Konten, die gegangen sind.
    let ich = zustand.gastgeber.person();
    for m in jetzt {
        if ich
            .as_ref()
            .is_some_and(|i| i.personen_id == m.personen_id || i.frueher.contains(&m.personen_id))
        {
            continue;
        }
        let Ok(Some(p)) = personenbindung::pruefen(zustand, client, &m.name).await else {
            continue;
        };
        if p.personen_id == m.personen_id && !auf_dem_server.iter().any(|n| gleich(n, &m.name)) {
            if let Err(e) = entfernen(zustand, client, projekt, &m.name).await {
                eprintln!("Mitgliederliste nicht nachgezogen ({projekt}): {e}");
            }
        }
    }
}

/// Steht das Konto `name` als Mitglied in der Liste?
///
/// Beides muss stimmen: Der Eintrag traegt den KONTONAMEN, und die Person,
/// die die gesiegelten Geraete dieses Kontos nennen, ist die des Eintrags.
/// Die Personen-Id allein traegt nichts -- die nennt jedes Konto in seinen
/// Bindungen selbst, auch die eines anderen. Der Name allein auch nicht: Wer
/// vor Ort beitrat, steht unter dem Namen seiner Person in der Liste, und der
/// kann zufaellig wie ein fremdes Konto heissen.
fn mitglied_des_kontos(
    jetzt: &[Mitglied],
    name: &str,
    person: &personenbindung::FernePerson,
) -> bool {
    jetzt
        .iter()
        .any(|m| gleich(&m.name, name) && m.personen_id == person.personen_id)
}

/// Gehoert das Konto `name` zu diesem Hub? Davon haengt ab, ob dieses Geraet
/// ihm den Hub-Schluessel packt UND ob es von ihm eine Kiste oder einen
/// Inhalt annimmt. Das eigene Konto immer (die eigenen Geraete); sonst nur
/// mit einer geprueften Liste, in der es steht ([`mitglied_des_kontos`]).
/// Ohne Liste niemand sonst: Welche Konten zum Hub gehoeren, sagte dann
/// allein der Server.
pub(crate) async fn konto_mitglied(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    liste: Option<&Mitgliederliste>,
    name: &str,
) -> bool {
    if direktpost::mein_name(zustand).is_some_and(|n| gleich(&n, name)) {
        return true;
    }
    let Some(liste) = liste else {
        return false;
    };
    let Ok(jetzt) = mitglieder(zustand, liste) else {
        return false;
    };
    // Erst der Name: Er kostet keine Anfrage.
    if !jetzt.iter().any(|m| gleich(&m.name, name)) {
        return false;
    }
    let Ok(Some(person)) = personenbindung::pruefen(zustand, client, name).await else {
        return false;
    };
    mitglied_des_kontos(&jetzt, name, &person)
}

#[cfg(test)]
mod tests {
    use super::*;
    use openany_nahbereich::{Identitaet, Person};

    fn mitglied(name: &str, personen_id: &str) -> Mitglied {
        Mitglied {
            personen_id: personen_id.into(),
            name: name.into(),
            rolle: MITGLIED.into(),
            geraet: format!("fp-{name}"),
        }
    }

    fn ferne(personen_id: &str) -> personenbindung::FernePerson {
        personenbindung::FernePerson {
            personen_id: personen_id.into(),
            anker: vec!["fp".into()],
        }
    }

    /// Ein Konto zaehlt nur unter seinem Namen UND mit seiner Person.
    #[test]
    fn mitglied_ist_nur_das_konto_das_die_liste_nennt() {
        let jetzt = vec![mitglied("Anna", "p-anna"), mitglied("ben", "p-ben")];
        assert!(mitglied_des_kontos(&jetzt, "anna", &ferne("p-anna")));
        assert!(mitglied_des_kontos(&jetzt, " Ben ", &ferne("p-ben")));
        // Ein fremdes Konto, dessen Geraete Annas Personen-Id nennen.
        assert!(!mitglied_des_kontos(&jetzt, "mallory", &ferne("p-anna")));
        // Ein Konto, das wie ein Mitglied heisst, aber eine andere Person ist.
        assert!(!mitglied_des_kontos(&jetzt, "anna", &ferne("p-fremd")));
        // Niemand in der Liste.
        assert!(!mitglied_des_kontos(&[], "anna", &ferne("p-anna")));
    }

    #[test]
    fn nur_fortsetzungen_gelten() {
        let tel = Identitaet::erzeugen().unwrap();
        let anna = Person::neu(&tel, "Telefon").unwrap();
        let mut a = Mitgliederliste {
            projekt: "p1".into(),
            eintraege: Vec::new(),
        };
        a.eintragen(
            &tel,
            "gruendung",
            &anna.personen_id,
            "anna",
            EIGENTUEMER,
            &tel.fingerabdruck,
        )
        .unwrap();
        let mut b = a.clone();
        b.eintragen(&tel, "beitritt", "ben-id", "ben", MITGLIED, "fp-ben")
            .unwrap();

        assert!(setzt_fort(None, &a));
        assert!(setzt_fort(Some(&a), &a));
        assert!(setzt_fort(Some(&a), &b));
        // Gekuerzt.
        assert!(!setzt_fort(Some(&b), &a));
        // Umgeschrieben.
        let mut c = a.clone();
        c.eintragen(&tel, "beitritt", "mallory-id", "mallory", MITGLIED, "fp-m")
            .unwrap();
        assert!(!setzt_fort(Some(&b), &c));
        // Anderes Projekt.
        let mut d = b.clone();
        d.projekt = "p2".into();
        assert!(!setzt_fort(Some(&b), &d));
    }

    /// Der Grabstein des Servers gilt nur, wenn die Liste ihn belegt
    /// (docs/plan-hub-aufloesen.md).
    #[test]
    fn der_grabstein_gilt_nur_mit_aufloesung_oder_eigenem_austritt() {
        let niemand = |_: &str, _: &str, _: &str| false;
        let jetzt = |l: &Mitgliederliste| l.mitglieder_mit(&|_| None, &niemand).unwrap();
        let tel = Identitaet::erzeugen().unwrap();
        let ben_tel = Identitaet::erzeugen().unwrap();
        let anna = Person::neu(&tel, "Telefon").unwrap();
        let ben = Person::neu(&ben_tel, "Telefon").unwrap();
        let mut a = Mitgliederliste {
            projekt: "p1".into(),
            eintraege: Vec::new(),
        };
        a.eintragen(
            &tel,
            "gruendung",
            &anna.personen_id,
            "anna",
            EIGENTUEMER,
            &tel.fingerabdruck,
        )
        .unwrap();
        let mut b = a.clone();
        b.eintragen(
            &tel,
            "beitritt",
            &ben.personen_id,
            "ben",
            MITGLIED,
            &ben_tel.fingerabdruck,
        )
        .unwrap();

        // Ben ist weiter Mitglied: Der Grabstein ist nicht belegt.
        assert!(!fort_belegt(&b, &[b.clone()], &jetzt(&b), &ben));
        assert!(!fort_belegt(&b, &[a.clone()], &jetzt(&b), &ben));

        // Entfernt: fuer Ben belegt, fuer Anna nicht.
        let mut ohne_ben = b.clone();
        ohne_ben
            .eintragen(&tel, "austritt", &ben.personen_id, "", "", "")
            .unwrap();
        assert!(fort_belegt(
            &ohne_ben,
            &[b.clone()],
            &jetzt(&ohne_ben),
            &ben
        ));
        assert!(!fort_belegt(
            &ohne_ben,
            &[b.clone()],
            &jetzt(&ohne_ben),
            &anna
        ));

        // Aufgeloest: fuer alle belegt.
        let mut fort = b.clone();
        fort.eintragen(
            &tel,
            openany_nahbereich::mitglieder::AUFLOESUNG,
            &anna.personen_id,
            "",
            "",
            &tel.fingerabdruck,
        )
        .unwrap();
        assert!(fort_belegt(&fort, &[b.clone()], &jetzt(&fort), &ben));
        assert!(fort_belegt(
            &fort,
            &[a.clone(), b.clone()],
            &jetzt(&fort),
            &anna
        ));

        // Eine Liste, die die gemerkte NICHT fortsetzt, belegt nichts -- auch
        // wenn Ben darin fehlt (die alte Liste von vor seinem Beitritt).
        assert!(!fort_belegt(&a, &[b.clone()], &jetzt(&a), &ben));
        let mut fremd = fort.clone();
        fremd.projekt = "p2".into();
        assert!(!fort_belegt(&fremd, &[b], &[], &ben));
    }

    #[test]
    fn beschnittenes_zertifikat_setzt_trotzdem_fort() {
        let tel = Identitaet::erzeugen().unwrap();
        let anna = Person::neu(&tel, "Telefon").unwrap();
        let mut a = Mitgliederliste {
            projekt: "p1".into(),
            eintraege: Vec::new(),
        };
        a.eintragen(
            &tel,
            "gruendung",
            &anna.personen_id,
            "anna",
            EIGENTUEMER,
            &tel.fingerabdruck,
        )
        .unwrap();
        // So kam sie vom Server: Zeilenumbruch am Ende fort.
        let mut vom_server = a.clone();
        vom_server.eintraege[0].zertifikat_pem = a.eintraege[0].zertifikat_pem.trim_end().into();
        assert_ne!(vom_server.eintraege[0], a.eintraege[0]);
        assert!(setzt_fort(Some(&a), &vom_server));
        assert!(setzt_fort(Some(&vom_server), &a));
    }
}
