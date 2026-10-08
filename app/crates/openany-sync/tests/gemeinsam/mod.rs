//! Zwei, drei Geraete ohne Leitung -- was die Abgleich-Tests teilen.

#![allow(dead_code)]

use async_trait::async_trait;
use openany_client::{Delta, Eintrag, Ergebnis, OpenanyError};
use openany_store::{Inhalte, Notiz, Protokoll, Speicher};
use openany_sync::{Auskunft, Bericht, Gegenstelle, Laeufer};

pub struct Geraet {
    pub name: &'static str,
    pub speicher: Speicher,
    pub inhalte: Inhalte,
    _ordner: tempfile::TempDir,
}

impl Geraet {
    pub fn neu(name: &'static str) -> Self {
        let ordner = tempfile::tempdir().unwrap();
        Self {
            name,
            speicher: Speicher::im_arbeitsspeicher().unwrap(),
            inhalte: Inhalte::oeffnen(ordner.path().join("inhalte")).unwrap(),
            _ordner: ordner,
        }
    }

    pub fn basis(&self) -> String {
        format!("nah:{}", self.name)
    }
}

/// Das andere Geraet, von diesem aus gesehen -- die Auskunft direkt, ohne
/// Leitung, aber mit derselben Umrechnung in und aus JSON, damit ein Feld,
/// das die Leitung nicht uebersteht, auch hier auffaellt.
struct Drueben<'a> {
    ich: &'a Geraet,
    dort: &'a Geraet,
    basis: String,
}

impl<'a> Drueben<'a> {
    fn auskunft(&self) -> Auskunft<'a> {
        Auskunft::neu(&self.dort.speicher, self.ich.basis(), self.dort.name)
            .mit_inhalten(&self.dort.inhalte)
    }
}

fn ueber_die_leitung(eintraege: &[Eintrag]) -> Vec<Eintrag> {
    let roh = serde_json::json!({
        "cursor": 0,
        "entries": eintraege.iter().map(Eintrag::ueber_die_leitung).collect::<Vec<_>>(),
    });
    Delta::lesen(&roh).unwrap().eintraege
}

fn fehler(e: impl ToString) -> OpenanyError {
    OpenanyError::Unlesbar(e.to_string())
}

#[async_trait]
impl Gegenstelle for Drueben<'_> {
    fn basis(&self) -> &str {
        &self.basis
    }

    fn name(&self) -> String {
        self.dort.name.to_string()
    }

    fn fotos_mitschicken(&self) -> bool {
        true
    }

    fn traegt_dateien(&self) -> bool {
        true
    }

    async fn inhalt(&self, abdruck: &str, von: u64, laenge: u64) -> Result<Vec<u8>, OpenanyError> {
        self.auskunft()
            .inhalt(abdruck, von, laenge)
            .ok_or_else(|| fehler("nicht da"))
    }

    async fn inhalte_da(&self, abdruecke: &[String]) -> Result<Vec<String>, OpenanyError> {
        Ok(self.auskunft().inhalte_da(abdruecke))
    }

    async fn delta(&self, seit: i64) -> Result<Delta, OpenanyError> {
        let mut d = self.auskunft().delta(seit).map_err(fehler)?;
        d.eintraege = ueber_die_leitung(&d.eintraege);
        Ok(d)
    }

    async fn notiztext(&self, zk_id: &str) -> Result<String, OpenanyError> {
        self.auskunft()
            .notiztext(zk_id)
            .map_err(fehler)?
            .ok_or_else(|| fehler("fort"))
    }

    async fn anwenden(&self, eintraege: &[Eintrag]) -> Result<Vec<Ergebnis>, OpenanyError> {
        self.auskunft()
            .anwenden(&ueber_die_leitung(eintraege))
            .map_err(fehler)
    }

    async fn kontaktfoto(&self, uuid: &str) -> Result<Vec<u8>, OpenanyError> {
        self.auskunft()
            .kontaktfoto(uuid)
            .map_err(fehler)?
            .ok_or_else(|| fehler("kein Foto"))
    }

    async fn marken_melden(&self, eigene: i64, fremde: i64) -> Result<(), OpenanyError> {
        self.auskunft()
            .marken_gemeldet(eigene, fremde)
            .map_err(fehler)
    }
}

/// Die Gegenstelle `dort`, von `ich` aus -- fuer Aufrufe ausserhalb des Laeufers.
pub fn gegenueber<'a>(ich: &'a Geraet, dort: &'a Geraet) -> impl Gegenstelle + 'a {
    Drueben {
        ich,
        dort,
        basis: dort.basis(),
    }
}

/// `ich` tippt auf "Jetzt abgleichen" mit `dort`.
pub async fn abgleichen(ich: &Geraet, dort: &Geraet) -> Bericht {
    let drueben = Drueben {
        ich,
        dort,
        basis: dort.basis(),
    };
    let bericht = Laeufer::neu(&ich.speicher, &drueben, ich.name).lauf().await;
    assert!(bericht.durchgelaufen(), "{:?}", bericht.fehler);
    bericht
}

pub fn nichts(b: &Bericht) -> bool {
    b.gezogen == 0 && b.geschoben == 0 && b.konflikte == 0
}

pub fn notiz(g: &Geraet, zk_id: &str, inhalt: &str) {
    g.speicher
        .notiz_schreiben(
            &Notiz {
                zk_id: zk_id.into(),
                titel: "Einkauf".into(),
                inhalt: inhalt.into(),
                ..Default::default()
            },
            Protokoll::Merken,
        )
        .unwrap();
}

pub fn inhalt(g: &Geraet, zk_id: &str) -> String {
    g.speicher.notiz(zk_id).unwrap().unwrap().inhalt
}
