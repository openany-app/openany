//! Was auf dem Server liegt.
//!
//! Notizen, Kalender, Kontakte, Nachrichten und die Namen von Alben, Akten und
//! Ordnern gehen immer hinauf. Von Fotos und Dateien nur, was einen Grund hat:
//!
//! * es ist in einem Hub geteilt,
//! * es hängt an einer Notiz,
//! * oder es soll dorthin: das Foto oder die Datei selbst oder der Behälter,
//!   in dem es liegt ([`auf_server_setzen`]).
//!
//! Alles andere reist nur zwischen den eigenen Geräten.
//!
//! **Vom Server nehmen ist kein Löschen.** Der Server meldet es nicht als
//! Änderung, die anderen Geräte behalten ihre Kopie. Und es geht nur, wenn die
//! Sache hier oder auf einem erreichbaren Gerät nachweislich liegt: Die letzte
//! Kopie bleibt, wo sie ist.

use crate::Zustand;
use openany_client::{Art, OpenanyClient};
use openany_store::{Protokoll, Speicher, SpeicherFehler};
use serde::Serialize;
use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

fn fehler(e: impl std::fmt::Display) -> String {
    e.to_string()
}

/// Einmal gelesen, was auf den Server soll. Für viele Fragen hintereinander.
pub(crate) struct Gewuenscht {
    /// Sachen und Behälter mit eigenem Schalter.
    eigene: BTreeSet<String>,
    /// Alle Ordner und Alben in und unter einem Behälter mit Schalter.
    bereiche: BTreeSet<String>,
    /// Geteilt oder an einer Notiz. Bleibt immer oben.
    pflicht: BTreeSet<String>,
}

impl Gewuenscht {
    pub(crate) fn lesen(
        speicher: &Speicher,
        freigegeben: &HashMap<String, String>,
    ) -> Result<Self, SpeicherFehler> {
        let eigene = speicher.alle_auf_server()?;
        let mut bereiche = BTreeSet::new();
        for id in &eigene {
            bereiche.extend(speicher.ordner_mit_inhalt(id)?);
            bereiche.extend(speicher.album_mit_unteralben(id)?);
        }
        let mut pflicht: BTreeSet<String> = freigegeben.keys().cloned().collect();
        pflicht.extend(speicher.anhaenge_alle()?.into_iter().map(|a| a.ziel_uuid));
        Ok(Self {
            eigene,
            bereiche,
            pflicht,
        })
    }

    pub(crate) fn eigen(&self, uuid: &str) -> bool {
        self.eigene.contains(uuid)
    }

    pub(crate) fn geerbt(&self, behaelter: Option<&str>) -> bool {
        behaelter.is_some_and(|b| self.bereiche.contains(b))
    }

    pub(crate) fn soll(&self, uuid: &str, behaelter: Option<&str>) -> bool {
        self.pflicht.contains(uuid) || self.eigen(uuid) || self.geerbt(behaelter)
    }
}

/// Eine Sache, die auf dem Server liegt.
struct Oben {
    uuid: String,
    art: &'static str,
    abdruck: String,
    groesse: u64,
}

/// Was oben liegt und keinen Grund mehr dafür hat. `nur_in`: nur Sachen, die
/// genannt sind oder in einem dieser Bereiche liegen.
fn ueberzaehlig(
    speicher: &Speicher,
    basis: &str,
    freigegeben: &HashMap<String, String>,
    nur_in: Option<&BTreeSet<String>>,
) -> Result<Vec<Oben>, SpeicherFehler> {
    let gewuenscht = Gewuenscht::lesen(speicher, freigegeben)?;
    let dort = speicher.sachen_dort(basis)?;
    let trifft = |uuid: &str, behaelter: Option<&str>| {
        dort.contains_key(uuid)
            && !gewuenscht.soll(uuid, behaelter)
            && nur_in.is_none_or(|b| b.contains(uuid) || behaelter.is_some_and(|x| b.contains(x)))
    };
    let mut aus = Vec::new();
    for d in speicher.lebendige_dateien()? {
        if d.ist_ordner || !trifft(&d.uuid, d.eltern.as_deref()) {
            continue;
        }
        if let Some(abdruck) = d.abdruck {
            aus.push(Oben {
                uuid: d.uuid,
                art: "file_node",
                abdruck,
                groesse: d.groesse,
            });
        }
    }
    for b in speicher.lebendige_bilder()? {
        if !trifft(&b.uuid, b.album.as_deref()) {
            continue;
        }
        if let Some(abdruck) = b.abdruck {
            aus.push(Oben {
                uuid: b.uuid,
                art: "media",
                abdruck,
                groesse: b.groesse,
            });
        }
    }
    Ok(aus)
}

/// Client und eigene Freigaben, wie sie auch der Abgleich nimmt.
async fn verbindung(
    zustand: &Arc<Zustand>,
) -> Result<(OpenanyClient, HashMap<String, String>), String> {
    let e = zustand.einstellungen.lock().await.clone();
    let schluessel = zustand
        .schluesselablage()
        .lesen()
        .map_err(fehler)?
        .ok_or("This device is not paired.")?;
    let client = OpenanyClient::neu_mit_ca(&e.openany_basis, &schluessel, e.ca().as_deref())
        .map_err(fehler)?;
    let freigaben = crate::mappenteilen::eigene(zustand, &client).await?;
    let speicher = zustand.speicher.lock().await;
    let karte = crate::mappenteilen::dateifreigaben(&speicher, &freigaben);
    drop(speicher);
    Ok((client, karte))
}

#[derive(Serialize, Default)]
pub struct Genommen {
    entfernt: usize,
    bytes: u64,
    /// Liegen sonst nirgends erreichbar und bleiben deshalb oben.
    einzig_dort: usize,
}

async fn nehmen(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    liste: Vec<Oben>,
) -> Result<Genommen, String> {
    let fehlen_hier: Vec<String> = liste
        .iter()
        .filter(|o| !zustand.inhalte.hat(&o.abdruck))
        .map(|o| o.abdruck.clone())
        .collect();
    let mut woanders = BTreeSet::new();
    if !fehlen_hier.is_empty() {
        for g in crate::dateibefehle::gegenstellen(zustand).await {
            for teil in fehlen_hier.chunks(500) {
                if let Ok(da) = openany_sync::Gegenstelle::inhalte_da(&g, teil).await {
                    woanders.extend(da);
                }
            }
        }
    }
    let mut aus = Genommen::default();
    let basis = OpenanyClient::basis(client).to_string();
    for o in liste {
        if !zustand.inhalte.hat(&o.abdruck) && !woanders.contains(&o.abdruck) {
            aus.einzig_dort += 1;
            continue;
        }
        client
            .vom_server_nehmen(o.art, &o.uuid)
            .await
            .map_err(fehler)?;
        zustand
            .speicher
            .lock()
            .await
            .sache_dort_vergessen(&basis, &o.uuid)
            .map_err(fehler)?;
        aus.entfernt += 1;
        aus.bytes += o.groesse;
    }
    Ok(aus)
}

/// Den Schalter „auf den Server" für ein Foto, eine Datei oder einen Behälter
/// setzen. Beim Ausschalten geht von oben, was keinen anderen Grund mehr hat.
#[tauri::command]
pub async fn auf_server_setzen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: String,
    an: bool,
) -> Result<Genommen, String> {
    let zustand = zustand.inner().clone();
    let bereich: BTreeSet<String> = {
        let speicher = zustand.speicher.lock().await;
        let art = if speicher.datei(&id).map_err(fehler)?.is_some() {
            Art::Datei
        } else if speicher.album(&id).map_err(fehler)?.is_some() {
            Art::Album
        } else if speicher.bild(&id).map_err(fehler)?.is_some() {
            Art::Medium
        } else {
            return Err("Not found.".into());
        };
        speicher
            .auf_server_setzen(&art, &id, an, Protokoll::Merken)
            .map_err(fehler)?;
        let mut b = BTreeSet::from([id.clone()]);
        b.extend(speicher.ordner_mit_inhalt(&id).map_err(fehler)?);
        b.extend(speicher.album_mit_unteralben(&id).map_err(fehler)?);
        b
    };

    let verbunden = zustand.einstellungen.lock().await.verbunden();
    let mut aus = Genommen::default();
    if !an && verbunden {
        if let Ok((client, freigegeben)) = verbindung(&zustand).await {
            let liste = {
                let speicher = zustand.speicher.lock().await;
                ueberzaehlig(
                    &speicher,
                    OpenanyClient::basis(&client),
                    &freigegeben,
                    Some(&bereich),
                )
                .map_err(fehler)?
            };
            aus = nehmen(&zustand, &client, liste).await?;
        }
    }
    if verbunden {
        tauri::async_runtime::spawn(async move {
            let _ = crate::server_abgleich(&zustand, true).await;
        });
    }
    Ok(aus)
}

#[derive(Serialize)]
pub struct Serverbelegung {
    nachrichten: u64,
    notizen: u64,
    kalender: u64,
    kontakte: u64,
    galerie: u64,
    dateien: u64,
    dokumente: u64,
    kontingent_belegt: u64,
    kontingent: Option<u64>,
}

/// Wofür der Platz auf dem Server belegt ist. Dokumente rechnet das Gerät aus
/// den eigenen Einträgen heraus, weil der Server die Zone nicht lesen kann.
#[tauri::command]
pub async fn server_belegung(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Serverbelegung, String> {
    let e = zustand.einstellungen.lock().await.clone();
    let schluessel = zustand
        .schluesselablage()
        .lesen()
        .map_err(fehler)?
        .ok_or("This device is not paired.")?;
    let client = OpenanyClient::neu_mit_ca(&e.openany_basis, &schluessel, e.ca().as_deref())
        .map_err(fehler)?;
    let b = client.belegung().await.map_err(fehler)?;
    let dokumente = {
        let speicher = zustand.speicher.lock().await;
        dokumente_dort(&speicher, OpenanyClient::basis(&client)).map_err(fehler)?
    };
    let dokumente = dokumente.min(b.dateien);
    Ok(Serverbelegung {
        nachrichten: b.nachrichten,
        notizen: b.notizen,
        kalender: b.kalender,
        kontakte: b.kontakte,
        galerie: b.galerie,
        dateien: b.dateien - dokumente,
        dokumente,
        kontingent_belegt: b.kontingent_belegt,
        kontingent: b.kontingent,
    })
}

fn dokumente_dort(speicher: &Speicher, basis: &str) -> Result<u64, SpeicherFehler> {
    let dort = speicher.sachen_dort(basis)?;
    Ok(speicher
        .lebendige_dateien()?
        .into_iter()
        .filter(|d| !d.ist_ordner && d.zone == "documents" && dort.contains_key(&d.uuid))
        .map(|d| d.groesse)
        .sum())
}

#[derive(Serialize)]
pub struct Reste {
    anzahl: usize,
    bytes: u64,
}

/// Was auf dem Server liegt und keinen Grund mehr dafür hat.
#[tauri::command]
pub async fn server_reste(zustand: tauri::State<'_, Arc<Zustand>>) -> Result<Reste, String> {
    let zustand = zustand.inner().clone();
    let (client, freigegeben) = verbindung(&zustand).await?;
    let speicher = zustand.speicher.lock().await;
    let liste = ueberzaehlig(&speicher, OpenanyClient::basis(&client), &freigegeben, None)
        .map_err(fehler)?;
    Ok(Reste {
        anzahl: liste.len(),
        bytes: liste.iter().map(|o| o.groesse).sum(),
    })
}

/// All das vom Server nehmen.
#[tauri::command]
pub async fn server_reste_nehmen(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Genommen, String> {
    let zustand = zustand.inner().clone();
    let (client, freigegeben) = verbindung(&zustand).await?;
    let liste = {
        let speicher = zustand.speicher.lock().await;
        ueberzaehlig(&speicher, OpenanyClient::basis(&client), &freigegeben, None)
            .map_err(fehler)?
    };
    nehmen(&zustand, &client, liste).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use openany_store::{Album, Bild};

    fn speicher() -> Speicher {
        Speicher::im_arbeitsspeicher().unwrap()
    }

    #[test]
    fn nur_was_einen_grund_hat_soll_hinauf() {
        let s = speicher();
        for (uuid, eltern) in [
            ("urlaub", None),
            ("strand", Some("urlaub")),
            ("garten", None),
        ] {
            s.album_schreiben(
                &Album {
                    uuid: uuid.into(),
                    name: uuid.into(),
                    eltern: eltern.map(Into::into),
                    ..Default::default()
                },
                Protokoll::Merken,
            )
            .unwrap();
        }
        s.bild_schreiben(
            &Bild {
                uuid: "foto".into(),
                album: Some("garten".into()),
                name: "foto.jpg".into(),
                ..Default::default()
            },
            Protokoll::Merken,
        )
        .unwrap();

        let keiner = Gewuenscht::lesen(&s, &HashMap::new()).unwrap();
        assert!(!keiner.soll("foto", Some("garten")));

        s.auf_server_setzen(&Art::Album, "urlaub", true, Protokoll::Merken)
            .unwrap();
        s.auf_server_setzen(&Art::Medium, "foto", true, Protokoll::Merken)
            .unwrap();
        let g = Gewuenscht::lesen(&s, &HashMap::new()).unwrap();
        assert!(g.soll("x", Some("strand")), "gilt auch im Unteralbum");
        assert!(g.soll("foto", Some("garten")), "einzelnes Foto");
        assert!(!g.soll("y", Some("garten")), "das Album daneben nicht");

        let geteilt = HashMap::from([("z".to_string(), "hub".to_string())]);
        let g = Gewuenscht::lesen(&s, &geteilt).unwrap();
        assert!(g.soll("z", Some("garten")), "im Hub geteilt");
    }

    #[test]
    fn ueberzaehlig_ist_nur_was_oben_liegt_und_nicht_mehr_gewuenscht_wird() {
        let s = speicher();
        let basis = "https://openany.test";
        for uuid in ["oben", "gewuenscht", "nur-hier"] {
            s.bild_schreiben(
                &Bild {
                    uuid: uuid.into(),
                    album: Some("a".into()),
                    name: uuid.into(),
                    abdruck: Some(format!("abdruck-{uuid}")),
                    ..Default::default()
                },
                Protokoll::Merken,
            )
            .unwrap();
        }
        s.sache_dort_merken(basis, "oben", "abdruck-oben").unwrap();
        s.sache_dort_merken(basis, "gewuenscht", "abdruck-gewuenscht")
            .unwrap();
        s.auf_server_setzen(&Art::Medium, "gewuenscht", true, Protokoll::Merken)
            .unwrap();

        let liste = ueberzaehlig(&s, basis, &HashMap::new(), None).unwrap();
        assert_eq!(
            liste.iter().map(|o| o.uuid.as_str()).collect::<Vec<_>>(),
            ["oben"]
        );
    }
}
