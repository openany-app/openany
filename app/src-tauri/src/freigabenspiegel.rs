//! Freigaben auf beiden Wegen (docs/plan-projekte-eine-art.md, E6).
//!
//! **Zwei Wege, ein Stand.** Vor Ort ist eine Freigabe ein unterschriebener
//! Eintrag in der Liste des Projekts (`openany_nahbereich::freigaben`); das
//! Geraet der Besitzerin liefert Notizen, Dateien und Bilder an Mitglieder in
//! der Naehe. Ueber openany.de ist sie eine Freigabe mit eigenem Schluessel
//! (`mappenteilen.rs`, `dateiteilen.rs`); Mitglieder holen die Inhalte vom
//! Server. Beide bleiben, wie sie gebaut sind.
//!
//! **Spiegeln.** Die Vor-Ort-Freigabe ist die, die immer gilt. Solange ein
//! Projekt auf openany.de liegt, gibt es zu jeder EIGENEN Freigabe auch die
//! ueber den Server -- und umgekehrt. So
//! - bekommen Mitglieder ohne Konto die Inhalte vor Ort,
//! - bleibt beim Ausschalten jede Freigabe vor Ort bestehen,
//! - entsteht beim Einschalten jede Freigabe auf dem Server neu.
//!
//! Jedes Geraet spiegelt nur, was seine Person freigibt; fremde Freigaben
//! spiegelt deren Besitzerin. Kalender (E7) spiegelt `projektkalender.rs`.

use std::sync::Arc;

use openany_nahbereich::freigaben::{
    geltende, ALBUM, BEARBEITEN, KALENDER, LESEN, NOTIZ_MAPPE, ORDNER,
};
use openany_nahbereich::{projektnah, Gastgeber};
use serde_json::Value;

use crate::Zustand;

/// Eine eigene Freigabe, wie beide Wege sie vergleichen koennen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Eigene {
    /// Die uuid beim Server (leer, wenn es nur die vor Ort gibt).
    pub uuid: String,
    /// Wie vor Ort: `note_folder`, `file_folder`, `album`.
    pub art: String,
    /// Mappenpfad bzw. uuid des Ordners oder Albums.
    pub schluessel: String,
    pub name: String,
    pub bearbeiten: bool,
}

impl Eigene {
    fn gleich(&self, andere: &Eigene) -> bool {
        self.art == andere.art && self.schluessel == andere.schluessel
    }
}

/// Die Art vor Ort zu der beim Server.
fn art_vor_ort(server: &str) -> Option<&'static str> {
    match server {
        "mappe" => Some(NOTIZ_MAPPE),
        "ordner" => Some(ORDNER),
        "album" => Some(ALBUM),
        _ => None,
    }
}

/// Die eigenen Freigaben beim Server.
pub(crate) async fn eigene_server(
    zustand: &Arc<Zustand>,
    projekt: &str,
) -> Result<Vec<Eigene>, String> {
    let liste = crate::mappenteilen::mappen_liste(zustand, projekt.to_string()).await?;
    Ok(liste
        .get("mappen")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|m| m.get("eigen").and_then(Value::as_bool) == Some(true))
        .filter_map(|m| {
            let art = art_vor_ort(m.get("art").and_then(Value::as_str).unwrap_or("mappe"))?;
            let schluessel = if art == NOTIZ_MAPPE {
                m.get("pfad").and_then(Value::as_str)?.to_string()
            } else {
                m.get("wurzel").and_then(Value::as_str)?.to_string()
            };
            Some(Eigene {
                uuid: m
                    .get("uuid")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                art: art.to_string(),
                schluessel,
                name: m
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                bearbeiten: m.get("stufe").and_then(Value::as_str) == Some("edit"),
            })
        })
        .collect())
}

/// Die eigenen Freigaben vor Ort (geltend, nicht zurueckgenommen).
pub(crate) async fn eigene_vor_ort(
    zustand: &Arc<Zustand>,
    projekt: &str,
) -> Result<Vec<Eigene>, String> {
    let g = zustand.gastgeber.clone();
    let ich = g.person().ok_or("This device has no person yet.")?;
    let speicher = zustand.speicher.lock().await;
    let (_, liste, freigaben) = projektnah::lesen(&speicher, projekt)?;
    let m = liste
        .mitglieder(&|id| g.person_von(id))
        .map_err(|e| e.to_string())?;
    Ok(geltende(&freigaben, projekt, &m, &|id| g.person_von(id))
        .into_iter()
        .filter(|f| f.personen_id == ich.personen_id || ich.frueher.contains(&f.personen_id))
        .filter(|f| f.stufe == LESEN || f.stufe == BEARBEITEN)
        .filter(|f| f.art != KALENDER)
        .map(|f| Eigene {
            uuid: String::new(),
            art: f.art.clone(),
            schluessel: f.schluessel.clone(),
            name: f.name.clone(),
            bearbeiten: f.stufe == BEARBEITEN,
        })
        .collect())
}

/// Beide Wege angleichen -- nur fuer Projekte auf openany.de, nur die
/// eigenen Freigaben. Was nur beim Server steht, kommt vor Ort dazu; was nur
/// vor Ort steht, auf den Server. Antwort: wie viele angelegt wurden.
pub(crate) async fn spiegeln(zustand: &Arc<Zustand>, projekt: &str) -> Result<usize, String> {
    let server = eigene_server(zustand, projekt).await?;
    let vor_ort = eigene_vor_ort(zustand, projekt).await?;
    let mut angelegt = 0;

    for s in server
        .iter()
        .filter(|s| !vor_ort.iter().any(|v| v.gleich(s)))
    {
        crate::projektbefehle::lokal_freigeben(
            zustand,
            projekt.to_string(),
            s.art.clone(),
            s.schluessel.clone(),
            s.name.clone(),
            true,
            Some(s.bearbeiten),
        )
        .await?;
        angelegt += 1;
    }

    for v in vor_ort
        .iter()
        .filter(|v| !server.iter().any(|s| s.gleich(v)))
    {
        let ergebnis = if v.art == NOTIZ_MAPPE {
            let mappe = zustand
                .speicher
                .lock()
                .await
                .mappen()
                .map_err(|e| e.to_string())?
                .into_iter()
                .find(|m| m.pfad == v.schluessel)
                .map(|m| m.id);
            match mappe {
                Some(id) => crate::mappenteilen::mappe_teilen(
                    zustand,
                    projekt.to_string(),
                    id,
                    if v.bearbeiten { "edit" } else { "view" }.to_string(),
                )
                .await
                .map(|_| ()),
                // Die Mappe gibt es hier nicht (mehr): nichts zu teilen.
                None => Ok(()),
            }
        } else {
            let art = if v.art == ORDNER { "ordner" } else { "album" };
            crate::dateiteilen::sache_teilen(
                zustand,
                projekt.to_string(),
                art.to_string(),
                v.schluessel.clone(),
            )
            .await
            .map(|_| ())
        };
        match ergebnis {
            Ok(()) => angelegt += 1,
            Err(e) => eprintln!(
                "Freigabe nicht auf openany.de gespiegelt ({projekt}, {}): {e}",
                v.name
            ),
        }
    }
    match crate::projektkalender::spiegeln(zustand, projekt).await {
        Ok(n) => angelegt += n,
        Err(e) => eprintln!("Kalender nicht gespiegelt ({projekt}): {e}"),
    }
    Ok(angelegt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn die_arten_beider_wege_passen_zueinander() {
        assert_eq!(art_vor_ort("mappe"), Some(NOTIZ_MAPPE));
        assert_eq!(art_vor_ort("ordner"), Some(ORDNER));
        assert_eq!(art_vor_ort("album"), Some(ALBUM));
        assert_eq!(art_vor_ort("kalender"), None);
    }

    #[test]
    fn gleich_ist_dieselbe_sache_nicht_dieselbe_stufe() {
        let a = Eigene {
            uuid: "u1".into(),
            art: NOTIZ_MAPPE.into(),
            schluessel: "Uni/Hausarbeit".into(),
            name: "Hausarbeit".into(),
            bearbeiten: true,
        };
        let b = Eigene {
            uuid: String::new(),
            bearbeiten: false,
            ..a.clone()
        };
        assert!(a.gleich(&b));
        let c = Eigene {
            schluessel: "Uni".into(),
            ..a.clone()
        };
        assert!(!a.gleich(&c));
    }
}
