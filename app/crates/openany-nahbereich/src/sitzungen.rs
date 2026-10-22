//! Gemeinsam schreiben im lokalen Netz (docs/plan-gemeinsam-schreiben.md,
//! Tiffy 22.10.2026).
//!
//! **Das Geraet, das die Notiz fuehrt, ist der Knoten.** Es haelt je Notiz
//! eine Sitzung: den zusammengefassten Stand, die Yjs-Aenderungen danach, die
//! Awareness (Cursor, Name, Farbe) und die Teilnehmer. Es deutet nichts davon
//! -- es sind Bytes, die es in der Reihenfolge ihres Eintreffens weiterreicht.
//! Yjs fuehrt sie auf jedem Geraet zum selben Text zusammen, gleich in welcher
//! Reihenfolge sie ankommen.
//!
//! **Long-Polling.** Wer wartet, bekommt alles mit einer Nummer ueber der
//! zuletzt gesehenen -- sofort, wenn es schon da ist, sonst sobald es kommt.
//!
//! **Ausgangstext genau einmal.** Ist die Sitzung leer, bekommt genau EIN
//! Teilnehmer den Auftrag, sie aus dem gespeicherten Markdown anzulegen.
//! Kaemen zwei gleichzeitig und legten beide an, stuende der Text doppelt da.
//!
//! **Gespeichert wird von einem.** Der `schreiber` ist das fuehrende Geraet
//! selbst, wenn es die Notiz offen hat, sonst wer am laengsten dabei ist.
//! Da alle denselben Stand haben, ist egal, wer -- nur nicht alle zugleich.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::Notify;

/// Wer so lange nichts geschickt und nicht gewartet hat, ist fort.
pub const TEILNEHMER_GILT: Duration = Duration::from_secs(60);
/// Laenger wartet eine Anfrage nicht; dann kommt sie leer zurueck.
pub const WARTEN_HOECHSTENS: Duration = Duration::from_secs(25);
/// Ab so vielen Aenderungen bittet der Knoten um einen zusammengefassten Stand.
pub const ZUSAMMENFASSEN_AB: usize = 400;

/// Die Farben der Cursor, der Reihe nach vergeben.
pub const FARBEN: [&str; 8] = [
    "#0f766e", "#b45309", "#7c3aed", "#be123c", "#1d4ed8", "#4d7c0f", "#a21caf", "#0e7490",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Teilnehmer {
    /// Fuer diese Sitzung erzeugt -- ein Geraet kann mit zwei Fenstern zwei
    /// Teilnehmer sein.
    pub kennung: String,
    pub personen_id: String,
    pub name: String,
    /// Fingerabdruck des Geraets; leer, wenn es auf dem fuehrenden Geraet
    /// selbst geschieht.
    pub geraet: String,
    pub farbe: String,
}

/// Was ein Beitretender bekommt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Beitritt {
    pub kennung: String,
    pub farbe: String,
    /// Der zusammengefasste Stand, falls es einen gibt.
    pub stand: Option<Vec<u8>>,
    /// Die Aenderungen danach, in Reihenfolge.
    pub aenderungen: Vec<Vec<u8>>,
    pub awareness: Vec<(String, Vec<u8>)>,
    pub teilnehmer: Vec<Teilnehmer>,
    /// Bis hierher ist alles enthalten.
    pub nr: u64,
    /// Dieser Teilnehmer legt die leere Sitzung aus dem Markdown an.
    pub anlegen: bool,
    pub schreiber: String,
}

/// Was seit einer Nummer neu ist.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Neues {
    /// Gesetzt, wenn der Wartende hinter einer Zusammenfassung lag: Dann
    /// kommt der ganze Stand, und `aenderungen` folgen darauf.
    pub stand: Option<Vec<u8>>,
    pub aenderungen: Vec<Vec<u8>>,
    pub awareness: Vec<(String, Vec<u8>)>,
    /// Wer seit `ab` gegangen ist -- deren Cursor verschwinden.
    pub weg: Vec<String>,
    pub teilnehmer: Vec<Teilnehmer>,
    pub nr: u64,
    pub schreiber: String,
    /// Der Knoten haette gern einen zusammengefassten Stand.
    pub zusammenfassen: bool,
    pub anlegen: bool,
}

#[derive(Debug)]
struct Dabei {
    t: Teilnehmer,
    seit: u64,
    zuletzt: Instant,
}

#[derive(Debug, Default)]
struct Sitzung {
    nr: u64,
    stand: Option<(u64, Vec<u8>)>,
    aenderungen: VecDeque<(u64, Vec<u8>)>,
    awareness: HashMap<String, (u64, Vec<u8>)>,
    weg: VecDeque<(u64, String)>,
    teilnehmer: HashMap<String, Dabei>,
    /// Wer die leere Sitzung gerade anlegt.
    anleger: Option<String>,
    beitritte: u64,
    wecker: Arc<Notify>,
}

impl Sitzung {
    fn leer(&self) -> bool {
        self.stand.is_none() && self.aenderungen.is_empty()
    }

    fn aufraeumen(&mut self) {
        let fort: Vec<String> = self
            .teilnehmer
            .iter()
            .filter(|(_, d)| d.zuletzt.elapsed() >= TEILNEHMER_GILT)
            .map(|(k, _)| k.clone())
            .collect();
        for k in fort {
            self.gehen(&k);
        }
    }

    fn gehen(&mut self, kennung: &str) {
        if self.teilnehmer.remove(kennung).is_none() {
            return;
        }
        self.awareness.remove(kennung);
        self.nr += 1;
        self.weg.push_back((self.nr, kennung.to_string()));
        while self.weg.len() > 100 {
            self.weg.pop_front();
        }
        if self.anleger.as_deref() == Some(kennung) {
            self.anleger = None;
        }
        self.wecker.notify_waiters();
    }

    fn schreiber(&self) -> String {
        let mut alle: Vec<&Dabei> = self.teilnehmer.values().collect();
        // Das fuehrende Geraet selbst zuerst, dann wer am laengsten dabei ist.
        alle.sort_by_key(|d| (!d.t.geraet.is_empty(), d.seit));
        alle.first()
            .map(|d| d.t.kennung.clone())
            .unwrap_or_default()
    }

    fn teilnehmerliste(&self) -> Vec<Teilnehmer> {
        let mut alle: Vec<&Dabei> = self.teilnehmer.values().collect();
        alle.sort_by_key(|d| d.seit);
        alle.into_iter().map(|d| d.t.clone()).collect()
    }

    /// Soll `kennung` die leere Sitzung anlegen? Ja, wenn sie leer ist und
    /// niemand anderes gerade anlegt.
    fn anlegen_fuer(&mut self, kennung: &str) -> bool {
        if !self.leer() {
            self.anleger = None;
            return false;
        }
        match &self.anleger {
            Some(a) if a != kennung && self.teilnehmer.contains_key(a) => false,
            _ => {
                self.anleger = Some(kennung.to_string());
                true
            }
        }
    }
}

/// Alle Sitzungen dieses Geraets, nach zk_id der Notiz.
#[derive(Debug, Default)]
pub struct Sitzungen {
    offen: HashMap<String, Sitzung>,
}

pub type GemeinsameSitzungen = Arc<Mutex<Sitzungen>>;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SitzungsFehler {
    #[error("not part of this session")]
    NichtDabei,
}

impl Sitzungen {
    /// Gibt es zu dieser Notiz gerade jemanden, der mitschreibt?
    pub fn laeuft(&mut self, notiz: &str) -> bool {
        match self.offen.get_mut(notiz) {
            Some(s) => {
                s.aufraeumen();
                !s.teilnehmer.is_empty()
            }
            None => false,
        }
    }

    pub fn beitreten(
        &mut self,
        notiz: &str,
        personen_id: &str,
        name: &str,
        geraet: &str,
    ) -> Beitritt {
        let s = self.offen.entry(notiz.to_string()).or_default();
        s.aufraeumen();
        s.beitritte += 1;
        let kennung = uuid::Uuid::new_v4().simple().to_string();
        let farbe = FARBEN[((s.beitritte - 1) as usize) % FARBEN.len()].to_string();
        let t = Teilnehmer {
            kennung: kennung.clone(),
            personen_id: personen_id.to_string(),
            name: name.to_string(),
            geraet: geraet.to_string(),
            farbe: farbe.clone(),
        };
        s.nr += 1;
        let seit = s.nr;
        s.teilnehmer.insert(
            kennung.clone(),
            Dabei {
                t,
                seit,
                zuletzt: Instant::now(),
            },
        );
        let anlegen = s.anlegen_fuer(&kennung);
        s.wecker.notify_waiters();
        Beitritt {
            kennung,
            farbe,
            stand: s.stand.as_ref().map(|(_, b)| b.clone()),
            aenderungen: s.aenderungen.iter().map(|(_, b)| b.clone()).collect(),
            awareness: s
                .awareness
                .iter()
                .map(|(k, (_, b))| (k.clone(), b.clone()))
                .collect(),
            teilnehmer: s.teilnehmerliste(),
            nr: s.nr,
            anlegen,
            schreiber: s.schreiber(),
        }
    }

    /// Der Teilnehmer, wenn `kennung` dabei ist und zu `geraet` gehoert.
    pub fn teilnehmer(&mut self, notiz: &str, kennung: &str, geraet: &str) -> Option<Teilnehmer> {
        let s = self.offen.get_mut(notiz)?;
        s.aufraeumen();
        s.teilnehmer
            .get(kennung)
            .filter(|d| d.t.geraet == geraet)
            .map(|d| d.t.clone())
    }

    fn dabei<'a>(
        &'a mut self,
        notiz: &str,
        kennung: &str,
        geraet: &str,
    ) -> Result<&'a mut Sitzung, SitzungsFehler> {
        let s = self
            .offen
            .get_mut(notiz)
            .ok_or(SitzungsFehler::NichtDabei)?;
        s.aufraeumen();
        match s.teilnehmer.get_mut(kennung) {
            Some(d) if d.t.geraet == geraet => {
                d.zuletzt = Instant::now();
                Ok(s)
            }
            _ => Err(SitzungsFehler::NichtDabei),
        }
    }

    /// Aenderungen und/oder die eigene Awareness einliefern.
    pub fn senden(
        &mut self,
        notiz: &str,
        kennung: &str,
        geraet: &str,
        aenderungen: Vec<Vec<u8>>,
        awareness: Option<Vec<u8>>,
    ) -> Result<u64, SitzungsFehler> {
        let s = self.dabei(notiz, kennung, geraet)?;
        for a in aenderungen {
            s.nr += 1;
            s.aenderungen.push_back((s.nr, a));
        }
        if let Some(a) = awareness {
            s.nr += 1;
            s.awareness.insert(kennung.to_string(), (s.nr, a));
        }
        if !s.leer() && s.anleger.as_deref() == Some(kennung) {
            s.anleger = None;
        }
        s.wecker.notify_waiters();
        Ok(s.nr)
    }

    /// Einen zusammengefassten Stand einliefern, der alle Aenderungen bis
    /// `bis` enthaelt. Spaetere bleiben stehen.
    pub fn zusammenfassen(
        &mut self,
        notiz: &str,
        kennung: &str,
        geraet: &str,
        stand: Vec<u8>,
        bis: u64,
    ) -> Result<(), SitzungsFehler> {
        let s = self.dabei(notiz, kennung, geraet)?;
        if bis > s.nr || s.stand.as_ref().is_some_and(|(n, _)| *n >= bis) {
            return Ok(());
        }
        while s.aenderungen.front().is_some_and(|(n, _)| *n <= bis) {
            s.aenderungen.pop_front();
        }
        s.stand = Some((bis, stand));
        Ok(())
    }

    /// Was seit `ab` neu ist -- ohne zu warten.
    pub fn seit(
        &mut self,
        notiz: &str,
        kennung: &str,
        geraet: &str,
        ab: u64,
    ) -> Result<Neues, SitzungsFehler> {
        let s = self.dabei(notiz, kennung, geraet)?;
        let hinter_stand = s.stand.as_ref().is_some_and(|(n, _)| *n > ab);
        let anlegen = s.anlegen_fuer(kennung);
        let s = self
            .offen
            .get_mut(notiz)
            .ok_or(SitzungsFehler::NichtDabei)?;
        Ok(Neues {
            stand: if hinter_stand {
                s.stand.as_ref().map(|(_, b)| b.clone())
            } else {
                None
            },
            aenderungen: s
                .aenderungen
                .iter()
                .filter(|(n, _)| hinter_stand || *n > ab)
                .map(|(_, b)| b.clone())
                .collect(),
            awareness: s
                .awareness
                .iter()
                .filter(|(k, (n, _))| *n > ab && k.as_str() != kennung)
                .map(|(k, (_, b))| (k.clone(), b.clone()))
                .collect(),
            weg: s
                .weg
                .iter()
                .filter(|(n, _)| *n > ab)
                .map(|(_, k)| k.clone())
                .collect(),
            teilnehmer: s.teilnehmerliste(),
            nr: s.nr,
            schreiber: s.schreiber(),
            zusammenfassen: s.aenderungen.len() >= ZUSAMMENFASSEN_AB,
            anlegen,
        })
    }

    /// Ist seit `ab` etwas geschehen, das den Wartenden angeht?
    fn etwas_seit(&self, notiz: &str, ab: u64) -> bool {
        self.offen.get(notiz).is_some_and(|s| s.nr > ab)
    }

    fn wecker(&self, notiz: &str) -> Option<Arc<Notify>> {
        self.offen.get(notiz).map(|s| s.wecker.clone())
    }

    /// Ist `kennung` der, der speichern soll?
    pub fn ist_schreiber(&mut self, notiz: &str, kennung: &str, geraet: &str) -> bool {
        self.dabei(notiz, kennung, geraet)
            .map(|s| s.schreiber() == kennung)
            .unwrap_or(false)
    }

    pub fn verlassen(&mut self, notiz: &str, kennung: &str, geraet: &str) {
        let leer = match self.dabei(notiz, kennung, geraet) {
            Ok(s) => {
                s.gehen(kennung);
                s.teilnehmer.is_empty()
            }
            Err(_) => false,
        };
        // Niemand mehr da: Der Stand ist gespeichert (als Markdown beim
        // fuehrenden Geraet); die naechste Sitzung beginnt von dort.
        if leer {
            self.offen.remove(notiz);
        }
    }
}

/// Warten, bis seit `ab` etwas geschehen ist -- hoechstens
/// [`WARTEN_HOECHSTENS`]. Dann `seit` lesen.
pub async fn warten(
    sitzungen: &GemeinsameSitzungen,
    notiz: &str,
    kennung: &str,
    geraet: &str,
    ab: u64,
) -> Result<Neues, SitzungsFehler> {
    warten_hoechstens(sitzungen, notiz, kennung, geraet, ab, WARTEN_HOECHSTENS).await
}

pub async fn warten_hoechstens(
    sitzungen: &GemeinsameSitzungen,
    notiz: &str,
    kennung: &str,
    geraet: &str,
    ab: u64,
    dauer: Duration,
) -> Result<Neues, SitzungsFehler> {
    let ende = tokio::time::Instant::now() + dauer;
    loop {
        let wecker = {
            let mut s = sitzungen.lock().map_err(|_| SitzungsFehler::NichtDabei)?;
            // Erst pruefen, ob man dabei ist -- sonst wartete ein Fremder.
            s.teilnehmer(notiz, kennung, geraet)
                .ok_or(SitzungsFehler::NichtDabei)?;
            if s.etwas_seit(notiz, ab) {
                return s.seit(notiz, kennung, geraet, ab);
            }
            s.wecker(notiz).ok_or(SitzungsFehler::NichtDabei)?
        };
        // Den Wecker VOR dem erneuten Pruefen scharf machen: Sonst ginge ein
        // Wecken zwischen Pruefen und Warten verloren.
        let geweckt = wecker.notified();
        tokio::pin!(geweckt);
        geweckt.as_mut().enable();
        {
            let mut s = sitzungen.lock().map_err(|_| SitzungsFehler::NichtDabei)?;
            if s.etwas_seit(notiz, ab) {
                return s.seit(notiz, kennung, geraet, ab);
            }
        }
        if tokio::time::timeout_at(ende, geweckt).await.is_err() {
            let mut s = sitzungen.lock().map_err(|_| SitzungsFehler::NichtDabei)?;
            return s.seit(notiz, kennung, geraet, ab);
        }
    }
}

/// Ein Ausweis vor Ort (docs/plan-whiteboard.md, W3a): Name und Geraet im
/// Schluesselverzeichnis des Servers, die Zeit und eine Unterschrift ueber
/// beides samt den Fingerabdruecken dieser Leitung
/// (`openany_direktpost::ausweis_unterschreiben`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Ausweis {
    pub name: String,
    pub geraet: i64,
    pub zeit: u64,
    pub signatur: String,
}

/// Was der Pruefer fuer ein Server-Projekt beantwortet bekommt.
#[derive(Debug, Clone)]
pub struct ServerAnfrage {
    pub projekt: String,
    /// Die geteilte Mappe, in der die Notiz liegt.
    pub freigabe: String,
    pub notiz: String,
    /// TLS-Fingerabdruck des Anrufers.
    pub anrufer: String,
    pub ausweis: Ausweis,
}

/// Mitglieder von SERVER-Projekten erkennen: Die Mitgliederliste und das
/// Schluesselverzeichnis kennt nur die Anwendung (mit Server).
#[async_trait::async_trait]
pub trait ServerPruefer: Send + Sync {
    /// Liegt die Notiz hier, in einer eigenen Mappe, die in `projekt` zum
    /// Bearbeiten geteilt ist? Verraet nichts: Wer Projekt, Freigabe und
    /// Notiz nennen kann, kennt sie schon.
    async fn hat(&self, projekt: &str, freigabe: &str, notiz: &str) -> bool;
    /// Ausweis und Mitgliedschaft pruefen -- und `hat`. Antwort: (Personen-Id,
    /// Name) fuer die Anwesenden.
    async fn pruefen(&self, anfrage: ServerAnfrage) -> Result<(String, String), String>;
    /// Ein EIGENES Whiteboard (docs/plan-whiteboard.md, W3b): (Titel,
    /// Zeichnung als JSON). `None`: nicht hier oder nicht eigenes.
    async fn whiteboard(&self, _projekt: &str, _whiteboard: &str) -> Option<(String, String)> {
        None
    }
    /// Ein eigenes Whiteboard speichern -- fuer den Schreiber der Sitzung.
    async fn whiteboard_speichern(
        &self,
        _projekt: &str,
        _whiteboard: &str,
        _titel: Option<String>,
        _inhalt: String,
    ) -> Result<(), String> {
        Err("Not available on this device.".into())
    }
}

/// Eine Whiteboard-Sitzung heisst `whiteboard:<uuid>` (wo sonst die zk_id
/// einer Notiz steht). Gibt die uuid zurueck.
pub fn ist_whiteboard(schluessel: &str) -> Option<&str> {
    schluessel.strip_prefix("whiteboard:")
}

/// Die Wege im Nahbereich-Dienst (dienst.rs) -- alle unter
/// `/openany/v1/projekt/notiz/sitzung/`.
pub mod http {
    use super::*;
    use crate::dienst::{fehler, json, lesen, Gastgeber};
    use base64::{engine::general_purpose::STANDARD as B64, Engine};
    use bytes::Bytes;
    use http_body_util::Full;
    use hyper::body::Incoming;
    use hyper::{Request, Response, StatusCode};

    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct SitzungPost {
        /// Leer: eine eigene Notiz zwischen eigenen gepaarten Geraeten.
        #[serde(default)]
        pub projekt: String,
        pub notiz: String,
        #[serde(default)]
        pub kennung: String,
        #[serde(default)]
        pub aenderungen: Vec<String>,
        #[serde(default)]
        pub awareness: Option<String>,
        #[serde(default)]
        pub stand: Option<String>,
        #[serde(default)]
        pub bis: u64,
        #[serde(default)]
        pub ab: u64,
        #[serde(default)]
        pub titel: Option<String>,
        #[serde(default)]
        pub inhalt: Option<String>,
        /// Server-Projekt (W3a): die geteilte Mappe der Notiz. Leer: lokales
        /// Projekt oder eigene Notiz.
        #[serde(default)]
        pub freigabe: String,
        /// Server-Projekt: wer ruft (nur beim Beitreten).
        #[serde(default)]
        pub ausweis: Option<super::Ausweis>,
    }

    /// `Beitritt` fuer die Leitung: Bytes als Base64, dazu die Notiz.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct BeitrittLeitung {
        pub kennung: String,
        pub farbe: String,
        pub stand: Option<String>,
        pub aenderungen: Vec<String>,
        pub awareness: Vec<(String, String)>,
        pub teilnehmer: Vec<Teilnehmer>,
        pub nr: u64,
        pub anlegen: bool,
        pub schreiber: String,
        pub titel: String,
        /// Der gespeicherte Text -- Ausgangspunkt fuer `anlegen`.
        pub inhalt: String,
    }

    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct NeuesLeitung {
        pub stand: Option<String>,
        pub aenderungen: Vec<String>,
        pub awareness: Vec<(String, String)>,
        pub weg: Vec<String>,
        pub teilnehmer: Vec<Teilnehmer>,
        pub nr: u64,
        pub schreiber: String,
        pub zusammenfassen: bool,
        pub anlegen: bool,
    }

    pub fn leitung_beitritt(b: Beitritt, titel: String, inhalt: String) -> BeitrittLeitung {
        BeitrittLeitung {
            kennung: b.kennung,
            farbe: b.farbe,
            stand: b.stand.map(|x| B64.encode(x)),
            aenderungen: b.aenderungen.iter().map(|x| B64.encode(x)).collect(),
            awareness: b
                .awareness
                .into_iter()
                .map(|(k, x)| (k, B64.encode(x)))
                .collect(),
            teilnehmer: b.teilnehmer,
            nr: b.nr,
            anlegen: b.anlegen,
            schreiber: b.schreiber,
            titel,
            inhalt,
        }
    }

    pub fn leitung_neues(n: Neues) -> NeuesLeitung {
        NeuesLeitung {
            stand: n.stand.map(|x| B64.encode(x)),
            aenderungen: n.aenderungen.iter().map(|x| B64.encode(x)).collect(),
            awareness: n
                .awareness
                .into_iter()
                .map(|(k, x)| (k, B64.encode(x)))
                .collect(),
            weg: n.weg,
            teilnehmer: n.teilnehmer,
            nr: n.nr,
            schreiber: n.schreiber,
            zusammenfassen: n.zusammenfassen,
            anlegen: n.anlegen,
        }
    }

    pub fn bytes_von(b64: &str) -> Result<Vec<u8>, String> {
        B64.decode(b64).map_err(|e| e.to_string())
    }

    /// Darf `anrufer` an dieser Notiz mitschreiben? Projekt-Freigabe mit
    /// der Stufe „bearbeiten“ (wie beim Sperren) -- oder, ohne Projekt, ein
    /// eigenes gepaartes Geraet. Gibt (Personen-Id, Name, Notiz) zurueck.
    pub fn erlaubt(
        speicher: &openany_store::Speicher,
        g: &dyn Gastgeber,
        projekt: &str,
        anrufer: &str,
        zk_id: &str,
    ) -> Result<(String, String, openany_store::Notiz), String> {
        if !projekt.is_empty() {
            let (wer, notiz) =
                crate::projektnah::bearbeitbare_notiz(speicher, g, projekt, anrufer, zk_id)?;
            return Ok((wer.personen_id, wer.name, notiz));
        }
        if !g.ist_gepaart(anrufer) {
            return Err("Not one of your own devices.".into());
        }
        let ich = g.person().ok_or("This device has no person yet.")?;
        let notiz = speicher
            .notiz(zk_id)
            .map_err(|e| e.to_string())?
            .filter(|n| !n.im_papierkorb())
            .ok_or("This note no longer exists here.")?;
        let name = if ich.name.is_empty() {
            g.mein_name()
        } else {
            ich.name
        };
        Ok((ich.personen_id, name, notiz))
    }

    /// Server-Projekt (W3a/W3b): eine Notiz aus einer geteilten Mappe oder
    /// ein Whiteboard?
    fn ist_server(post: &SitzungPost) -> bool {
        !post.freigabe.is_empty() || ist_whiteboard(&post.notiz).is_some()
    }

    /// Was in einem Server-Projekt gemeinsam bearbeitet wird: eine Notiz aus
    /// einer eigenen Mappe, die dort zum Bearbeiten geteilt ist, oder ein
    /// eigenes Whiteboard. Mit Ausweis: auch wer ruft. Antwort: (Personen-Id,
    /// Name, Titel, Inhalt).
    async fn server_inhalt(
        g: &dyn Gastgeber,
        post: &SitzungPost,
        anrufer: &str,
        mit_ausweis: bool,
    ) -> Result<(String, String, String, String), (StatusCode, String)> {
        let verboten = |e: String| (StatusCode::FORBIDDEN, e);
        let pruefer = g
            .server_pruefer()
            .ok_or_else(|| verboten("Not available on this device.".into()))?;
        let (personen_id, name) = if mit_ausweis {
            let ausweis = post
                .ausweis
                .clone()
                .ok_or_else(|| verboten("No identification given.".into()))?;
            pruefer
                .pruefen(ServerAnfrage {
                    projekt: post.projekt.clone(),
                    freigabe: post.freigabe.clone(),
                    notiz: post.notiz.clone(),
                    anrufer: anrufer.to_string(),
                    ausweis,
                })
                .await
                .map_err(verboten)?
        } else {
            (String::new(), String::new())
        };
        if let Some(wb) = ist_whiteboard(&post.notiz) {
            let (titel, inhalt) = pruefer
                .whiteboard(&post.projekt, wb)
                .await
                .ok_or_else(|| verboten("This whiteboard is not here.".into()))?;
            return Ok((personen_id, name, titel, inhalt));
        }
        if !mit_ausweis
            && !pruefer
                .hat(&post.projekt, &post.freigabe, &post.notiz)
                .await
        {
            return Err(verboten("This note is not shared for editing.".into()));
        }
        let speicher = g.speicher().ok_or((
            StatusCode::SERVICE_UNAVAILABLE,
            "Nothing is being edited here.".to_string(),
        ))?;
        let speicher = speicher.lock().await;
        let notiz = speicher
            .notiz(&post.notiz)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            .filter(|n| !n.im_papierkorb())
            .ok_or_else(|| verboten("This note no longer exists here.".into()))?;
        Ok((personen_id, name, notiz.titel, notiz.inhalt))
    }

    pub async fn beantworten(
        pfad: &str,
        anfrage: Request<Incoming>,
        anrufer: &str,
        g: &dyn Gastgeber,
    ) -> Response<Full<Bytes>> {
        let Some(sitzungen) = g.sitzungen() else {
            return fehler(
                StatusCode::SERVICE_UNAVAILABLE,
                "Nothing is being edited here.",
            );
        };
        let Some(post) = lesen::<SitzungPost>(anfrage).await else {
            return fehler(StatusCode::BAD_REQUEST, "unlesbar");
        };
        let was = pfad.trim_start_matches("/openany/v1/projekt/notiz/sitzung/");

        // Server-Projekt (W3a): Gibt es die Notiz hier? Ohne Ausweis -- so
        // findet ein Mitglied das Geraet, das sie fuehrt, bevor es sich zeigt.
        if was == "hat" {
            let hat = ist_server(&post) && server_inhalt(g, &post, anrufer, false).await.is_ok();
            let laeuft = hat
                && sitzungen
                    .lock()
                    .map(|mut s| s.laeuft(&post.notiz))
                    .unwrap_or(false);
            return json(
                StatusCode::OK,
                &serde_json::json!({ "hat": hat, "laeuft": laeuft }),
            );
        }

        if was == "beitreten" && ist_server(&post) {
            let (personen_id, name, titel, inhalt) =
                match server_inhalt(g, &post, anrufer, true).await {
                    Ok(v) => v,
                    Err((status, e)) => return fehler(status, &e),
                };
            let Ok(mut s) = sitzungen.lock() else {
                return fehler(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Sessions cannot be read.",
                );
            };
            let b = s.beitreten(&post.notiz, &personen_id, &name, anrufer);
            return json(StatusCode::OK, &leitung_beitritt(b, titel, inhalt));
        }

        if was == "beitreten" {
            let Some(speicher) = g.speicher() else {
                return fehler(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Nothing is being edited here.",
                );
            };
            let speicher = speicher.lock().await;
            let (personen_id, name, notiz) =
                match erlaubt(&speicher, g, &post.projekt, anrufer, &post.notiz) {
                    Ok(v) => v,
                    Err(e) => return fehler(StatusCode::FORBIDDEN, &e),
                };
            drop(speicher);
            // Haelt eine alte App die Sperre, gibt es keine Sitzung.
            // Nur ein ANDERES Geraet zaehlt -- die Sperre, die das fuehrende
            // Geraet beim eigenen Speichern kurz selbst haelt, nicht.
            if let Some(h) = g
                .sperren()
                .and_then(|s| s.lock().ok().and_then(|mut s| s.halter(&notiz.zk_id)))
                .filter(|h| !h.geraet.is_empty())
            {
                return fehler(
                    StatusCode::CONFLICT,
                    &format!(
                        "{} is editing this note with an older app.",
                        if h.name.is_empty() { "Jemand" } else { &h.name }
                    ),
                );
            }
            let Ok(mut s) = sitzungen.lock() else {
                return fehler(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Sessions cannot be read.",
                );
            };
            let b = s.beitreten(&notiz.zk_id, &personen_id, &name, anrufer);
            return json(
                StatusCode::OK,
                &leitung_beitritt(b, notiz.titel, notiz.inhalt),
            );
        }

        // Laeuft hier schon eine Sitzung? Fuer eigene Geraete, die erst
        // nachsehen, bevor sie selbst zum Knoten werden.
        if was == "laeuft" {
            let Some(speicher) = g.speicher() else {
                return fehler(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Nothing is being edited here.",
                );
            };
            let speicher = speicher.lock().await;
            let (_, _, notiz) = match erlaubt(&speicher, g, &post.projekt, anrufer, &post.notiz) {
                Ok(v) => v,
                Err(e) => return fehler(StatusCode::FORBIDDEN, &e),
            };
            drop(speicher);
            let laeuft = sitzungen
                .lock()
                .map(|mut s| s.laeuft(&notiz.zk_id))
                .unwrap_or(false);
            return json(StatusCode::OK, &serde_json::json!({ "laeuft": laeuft }));
        }

        if was == "warten" {
            return match warten(&sitzungen, &post.notiz, &post.kennung, anrufer, post.ab).await {
                Ok(n) => json(StatusCode::OK, &leitung_neues(n)),
                Err(e) => fehler(StatusCode::GONE, &e.to_string()),
            };
        }

        if was == "speichern" {
            let schreiber = sitzungen
                .lock()
                .map(|mut s| s.ist_schreiber(&post.notiz, &post.kennung, anrufer))
                .unwrap_or(false);
            if !schreiber {
                return fehler(StatusCode::CONFLICT, "Someone else saves this session.");
            }
            // Ein Whiteboard speichert dieses Geraet beim Server (W3b) --
            // es gehoert ihm, gezeichnet hat die Sitzung.
            if let Some(wb) = ist_whiteboard(&post.notiz) {
                let Some(pruefer) = g.server_pruefer() else {
                    return fehler(StatusCode::FORBIDDEN, "Not available on this device.");
                };
                return match pruefer
                    .whiteboard_speichern(
                        &post.projekt,
                        wb,
                        post.titel.clone(),
                        post.inhalt.clone().unwrap_or_default(),
                    )
                    .await
                {
                    Ok(()) => json(StatusCode::OK, &serde_json::json!({})),
                    Err(e) => fehler(StatusCode::BAD_GATEWAY, &e),
                };
            }
            let Some(speicher) = g.speicher() else {
                return fehler(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Nothing is being edited here.",
                );
            };
            // Server-Projekt: Wer Schreiber dieser Sitzung ist, wurde beim
            // Beitreten geprueft; hier nur, ob die Mappe noch geteilt ist.
            let mut notiz = if post.freigabe.is_empty() {
                let speicher = speicher.lock().await;
                match erlaubt(&speicher, g, &post.projekt, anrufer, &post.notiz) {
                    Ok((_, _, n)) => n,
                    Err(e) => return fehler(StatusCode::FORBIDDEN, &e),
                }
            } else {
                if let Err((status, e)) = server_inhalt(g, &post, anrufer, false).await {
                    return fehler(status, &e);
                }
                match speicher.lock().await.notiz(&post.notiz) {
                    Ok(Some(n)) => n,
                    _ => return fehler(StatusCode::FORBIDDEN, "This note no longer exists here."),
                }
            };
            let speicher = speicher.lock().await;
            if let Some(t) = post.titel {
                notiz.titel = t;
            }
            if let Some(i) = post.inhalt {
                notiz.inhalt = i;
            }
            notiz.geaendert_at = String::new();
            return match speicher.notiz_schreiben(&notiz, openany_store::Protokoll::Merken) {
                Ok(()) => {
                    g.angenommen();
                    json(StatusCode::OK, &serde_json::json!({}))
                }
                Err(e) => fehler(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };
        }

        let Ok(mut s) = sitzungen.lock() else {
            return fehler(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Sessions cannot be read.",
            );
        };
        let ergebnis = match was {
            "senden" => {
                let mut aenderungen = Vec::new();
                for a in &post.aenderungen {
                    match bytes_von(a) {
                        Ok(b) => aenderungen.push(b),
                        Err(e) => return fehler(StatusCode::BAD_REQUEST, &e),
                    }
                }
                let awareness = match post.awareness.as_deref().map(bytes_von).transpose() {
                    Ok(a) => a,
                    Err(e) => return fehler(StatusCode::BAD_REQUEST, &e),
                };
                s.senden(&post.notiz, &post.kennung, anrufer, aenderungen, awareness)
                    .map(|nr| serde_json::json!({ "nr": nr }))
            }
            "zusammenfassen" => {
                let Some(stand) = post
                    .stand
                    .as_deref()
                    .map(bytes_von)
                    .transpose()
                    .ok()
                    .flatten()
                else {
                    return fehler(StatusCode::BAD_REQUEST, "unlesbar");
                };
                s.zusammenfassen(&post.notiz, &post.kennung, anrufer, stand, post.bis)
                    .map(|()| serde_json::json!({}))
            }
            "verlassen" => {
                s.verlassen(&post.notiz, &post.kennung, anrufer);
                Ok(serde_json::json!({}))
            }
            _ => return fehler(StatusCode::NOT_FOUND, "Unbekannt."),
        };
        match ergebnis {
            Ok(v) => json(StatusCode::OK, &v),
            Err(e) => fehler(StatusCode::GONE, &e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const N: &str = "notiz-1";

    fn gemeinsam() -> GemeinsameSitzungen {
        Arc::new(Mutex::new(Sitzungen::default()))
    }

    #[test]
    fn nur_der_erste_legt_an() {
        let mut s = Sitzungen::default();
        let a = s.beitreten(N, "p-a", "Anna", "fp-a");
        let b = s.beitreten(N, "p-b", "Ben", "fp-b");
        assert!(a.anlegen);
        assert!(
            !b.anlegen,
            "zwei Anleger hiessen: der Text steht doppelt da"
        );

        s.senden(N, &a.kennung, "fp-a", vec![vec![1, 2, 3]], None)
            .unwrap();
        let c = s.beitreten(N, "p-c", "Cleo", "fp-c");
        assert!(!c.anlegen);
        assert_eq!(c.aenderungen, vec![vec![1, 2, 3]]);
    }

    #[test]
    fn geht_der_anleger_ohne_anzulegen_uebernimmt_der_naechste() {
        let mut s = Sitzungen::default();
        let a = s.beitreten(N, "p-a", "Anna", "fp-a");
        let b = s.beitreten(N, "p-b", "Ben", "fp-b");
        s.verlassen(N, &a.kennung, "fp-a");

        let neu = s.seit(N, &b.kennung, "fp-b", b.nr).unwrap();
        assert!(neu.anlegen);
        assert_eq!(neu.weg, vec![a.kennung]);
    }

    #[test]
    fn aenderungen_kommen_bei_den_anderen_an_die_eigene_awareness_nicht() {
        let mut s = Sitzungen::default();
        let a = s.beitreten(N, "p-a", "Anna", "fp-a");
        let b = s.beitreten(N, "p-b", "Ben", "fp-b");
        s.senden(N, &a.kennung, "fp-a", vec![vec![7]], Some(vec![9]))
            .unwrap();

        let fuer_b = s.seit(N, &b.kennung, "fp-b", b.nr).unwrap();
        assert_eq!(fuer_b.aenderungen, vec![vec![7]]);
        assert_eq!(fuer_b.awareness, vec![(a.kennung.clone(), vec![9])]);

        let fuer_a = s.seit(N, &a.kennung, "fp-a", b.nr).unwrap();
        assert!(fuer_a.awareness.is_empty());
    }

    #[test]
    fn fremde_geraete_kommen_nicht_hinein() {
        let mut s = Sitzungen::default();
        let a = s.beitreten(N, "p-a", "Anna", "fp-a");
        assert_eq!(
            s.senden(N, &a.kennung, "fp-fremd", vec![vec![1]], None),
            Err(SitzungsFehler::NichtDabei)
        );
        assert!(s.seit(N, "ausgedacht", "fp-a", 0).is_err());
    }

    #[test]
    fn das_fuehrende_geraet_speichert_sonst_der_dienstaelteste() {
        let mut s = Sitzungen::default();
        let a = s.beitreten(N, "p-a", "Anna", "fp-a");
        let b = s.beitreten(N, "p-b", "Ben", "fp-b");
        assert!(s.ist_schreiber(N, &a.kennung, "fp-a"));
        assert!(!s.ist_schreiber(N, &b.kennung, "fp-b"));

        let hier = s.beitreten(N, "p-ich", "Ich", "");
        assert!(s.ist_schreiber(N, &hier.kennung, ""));
        assert!(!s.ist_schreiber(N, &a.kennung, "fp-a"));
    }

    #[test]
    fn zusammenfassen_ersetzt_die_alten_aenderungen() {
        let mut s = Sitzungen::default();
        let a = s.beitreten(N, "p-a", "Anna", "fp-a");
        let b = s.beitreten(N, "p-b", "Ben", "fp-b");
        let bis = s
            .senden(N, &a.kennung, "fp-a", vec![vec![1], vec![2]], None)
            .unwrap();
        s.senden(N, &a.kennung, "fp-a", vec![vec![3]], None)
            .unwrap();
        s.zusammenfassen(N, &a.kennung, "fp-a", vec![12], bis)
            .unwrap();

        let c = s.beitreten(N, "p-c", "Cleo", "fp-c");
        assert_eq!(c.stand, Some(vec![12]));
        assert_eq!(c.aenderungen, vec![vec![3]]);

        // Wer vor der Zusammenfassung stand, bekommt den Stand und den Rest.
        let fuer_b = s.seit(N, &b.kennung, "fp-b", b.nr).unwrap();
        assert_eq!(fuer_b.stand, Some(vec![12]));
        assert_eq!(fuer_b.aenderungen, vec![vec![3]]);
    }

    #[test]
    fn wer_als_letzter_geht_beendet_die_sitzung() {
        let mut s = Sitzungen::default();
        let a = s.beitreten(N, "p-a", "Anna", "fp-a");
        s.senden(N, &a.kennung, "fp-a", vec![vec![1]], None)
            .unwrap();
        assert!(s.laeuft(N));
        s.verlassen(N, &a.kennung, "fp-a");
        assert!(!s.laeuft(N));
        assert!(s.beitreten(N, "p-a", "Anna", "fp-a").anlegen);
    }

    #[tokio::test]
    async fn wer_wartet_wird_geweckt() {
        let g = gemeinsam();
        let (a, b) = {
            let mut s = g.lock().unwrap();
            (
                s.beitreten(N, "p-a", "Anna", "fp-a"),
                s.beitreten(N, "p-b", "Ben", "fp-b"),
            )
        };
        let g2 = g.clone();
        let bk = b.kennung.clone();
        let wartend = tokio::spawn(async move {
            warten_hoechstens(&g2, N, &bk, "fp-b", b.nr, Duration::from_secs(5)).await
        });
        tokio::time::sleep(Duration::from_millis(50)).await;
        g.lock()
            .unwrap()
            .senden(N, &a.kennung, "fp-a", vec![vec![42]], None)
            .unwrap();

        let neu = tokio::time::timeout(Duration::from_secs(2), wartend)
            .await
            .expect("geweckt statt ausgelaufen")
            .unwrap()
            .unwrap();
        assert_eq!(neu.aenderungen, vec![vec![42]]);
    }

    #[tokio::test]
    async fn ohne_neues_kommt_das_warten_leer_zurueck() {
        let g = gemeinsam();
        let b = g.lock().unwrap().beitreten(N, "p-b", "Ben", "fp-b");
        let neu = warten_hoechstens(&g, N, &b.kennung, "fp-b", b.nr, Duration::from_millis(50))
            .await
            .unwrap();
        assert!(neu.aenderungen.is_empty());
        assert_eq!(neu.nr, b.nr);
    }

    #[tokio::test]
    async fn ein_fremder_wartet_nicht() {
        let g = gemeinsam();
        g.lock().unwrap().beitreten(N, "p-a", "Anna", "fp-a");
        assert_eq!(
            warten_hoechstens(&g, N, "ausgedacht", "fp-x", 0, Duration::from_millis(50)).await,
            Err(SitzungsFehler::NichtDabei)
        );
    }
}

#[cfg(test)]
mod leitung_tests {
    use super::http::{BeitrittLeitung, NeuesLeitung, SitzungPost};
    use super::*;
    use crate::abgleich::GemeinsamerSpeicher;
    use crate::dienst::{Dienst, Gastgeber};
    use crate::Identitaet;
    use base64::{engine::general_purpose::STANDARD as B64, Engine};
    use openany_store::{Notiz, Protokoll, Speicher};

    struct Geraet {
        name: String,
        ident: Identitaet,
        speicher: GemeinsamerSpeicher,
        gepaart: Mutex<Vec<String>>,
        sitzungen: GemeinsameSitzungen,
        sperren: crate::sperren::GemeinsameSperren,
        pruefer: Option<Arc<dyn ServerPruefer>>,
        _ordner: tempfile::TempDir,
    }

    /// Ein Server-Projekt „P“ mit der Mappe „F“, in der ZK liegt; Mitglied
    /// ist nur „anna“, und nur vom Geraet `anna_fp`.
    struct Pruefer {
        anna_fp: String,
        /// Was das Whiteboard „WB“ zuletzt gespeichert bekam.
        gespeichert: Mutex<Option<String>>,
    }

    #[async_trait::async_trait]
    impl ServerPruefer for Pruefer {
        async fn hat(&self, projekt: &str, freigabe: &str, notiz: &str) -> bool {
            projekt == "P" && freigabe == "F" && notiz == ZK
        }
        async fn pruefen(&self, a: ServerAnfrage) -> Result<(String, String), String> {
            let wb = ist_whiteboard(&a.notiz);
            if wb.is_some() && self.whiteboard(&a.projekt, wb.unwrap()).await.is_none() {
                return Err("not here".into());
            }
            if wb.is_none() && !self.hat(&a.projekt, &a.freigabe, &a.notiz).await {
                return Err("not shared".into());
            }
            if a.ausweis.name != "anna" || a.anrufer != self.anna_fp {
                return Err("Not a member of this project.".into());
            }
            Ok(("konto:anna".into(), "anna".into()))
        }
        async fn whiteboard(&self, projekt: &str, wb: &str) -> Option<(String, String)> {
            (projekt == "P" && wb == "WB").then(|| ("Skizze".into(), r#"{"elements":[]}"#.into()))
        }
        async fn whiteboard_speichern(
            &self,
            _projekt: &str,
            wb: &str,
            _titel: Option<String>,
            inhalt: String,
        ) -> Result<(), String> {
            if wb != "WB" {
                return Err("not here".into());
            }
            *self.gespeichert.lock().unwrap() = Some(inhalt);
            Ok(())
        }
    }

    impl Gastgeber for Geraet {
        fn mein_name(&self) -> String {
            self.name.clone()
        }
        fn qr_geheimnis(&self) -> Option<String> {
            None
        }
        fn gepaart(&self, fp: &str, _name: &str) {
            self.gepaart.lock().unwrap().push(fp.into());
        }
        fn ist_gepaart(&self, fp: &str) -> bool {
            self.gepaart.lock().unwrap().iter().any(|f| f == fp)
        }
        fn speicher(&self) -> Option<GemeinsamerSpeicher> {
            Some(self.speicher.clone())
        }
        fn person(&self) -> Option<crate::Person> {
            Some(crate::Person {
                personen_id: "p-ich".into(),
                name: self.name.clone(),
                frueher: vec![],
                eintraege: vec![],
            })
        }
        fn sitzungen(&self) -> Option<GemeinsameSitzungen> {
            Some(self.sitzungen.clone())
        }
        fn sperren(&self) -> Option<crate::sperren::GemeinsameSperren> {
            Some(self.sperren.clone())
        }
        fn server_pruefer(&self) -> Option<Arc<dyn ServerPruefer>> {
            self.pruefer.clone()
        }
    }

    fn geraet(name: &str) -> Arc<Geraet> {
        let ordner = tempfile::tempdir().unwrap();
        Arc::new(Geraet {
            name: name.into(),
            ident: Identitaet::laden_oder_erzeugen(&ordner.path().join("i.json")).unwrap(),
            speicher: Arc::new(tokio::sync::Mutex::new(
                Speicher::im_arbeitsspeicher().unwrap(),
            )),
            gepaart: Mutex::new(vec![]),
            sitzungen: Default::default(),
            sperren: Default::default(),
            pruefer: None,
            _ordner: ordner,
        })
    }

    const ZK: &str = "20261022120000";

    async fn rufen<T: serde::de::DeserializeOwned>(
        von: &Geraet,
        zu: &Geraet,
        port: u16,
        was: &str,
        post: SitzungPost,
    ) -> Result<T, String> {
        crate::anruf::sitzung(
            &von.ident,
            "127.0.0.1",
            port,
            &zu.ident.fingerabdruck,
            was,
            &post,
        )
        .await
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn zwei_eigene_geraete_schreiben_gemeinsam_ueber_echtes_tls() {
        let tablet = geraet("Tablet");
        let laptop = geraet("Laptop");
        let fremd = geraet("Fremd");
        tablet.gepaart(&laptop.ident.fingerabdruck, "Laptop");
        laptop.gepaart(&tablet.ident.fingerabdruck, "Tablet");
        tablet
            .speicher
            .lock()
            .await
            .notiz_schreiben(
                &Notiz {
                    zk_id: ZK.into(),
                    titel: "Einkauf".into(),
                    inhalt: "Milch".into(),
                    ..Default::default()
                },
                Protokoll::Merken,
            )
            .unwrap();
        let dienst = Dienst::starten(&tablet.ident, 0, Default::default(), tablet.clone())
            .await
            .unwrap();
        let port = dienst.port;

        // Das Tablet selbst schreibt mit (lokal, ohne Netz).
        let hier = tablet
            .sitzungen
            .lock()
            .unwrap()
            .beitreten(ZK, "p-ich", "Tablet", "");
        assert!(hier.anlegen);
        tablet
            .sitzungen
            .lock()
            .unwrap()
            .senden(ZK, &hier.kennung, "", vec![vec![1, 2]], None)
            .unwrap();

        // Der Laptop tritt bei und bekommt Stand und Ausgangstext.
        let b: BeitrittLeitung = rufen(
            &laptop,
            &tablet,
            port,
            "beitreten",
            SitzungPost {
                notiz: ZK.into(),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(!b.anlegen);
        assert_eq!(b.aenderungen, vec![B64.encode([1, 2])]);
        assert_eq!(b.inhalt, "Milch");
        assert_eq!(b.schreiber, hier.kennung, "das fuehrende Geraet speichert");

        // Der Laptop wartet; das Tablet schreibt; der Laptop wird geweckt.
        let laptop2 = laptop.clone();
        let tablet2 = tablet.clone();
        let (kennung, ab) = (b.kennung.clone(), b.nr);
        let wartend = tokio::spawn(async move {
            rufen::<NeuesLeitung>(
                &laptop2,
                &tablet2,
                port,
                "warten",
                SitzungPost {
                    notiz: ZK.into(),
                    kennung,
                    ab,
                    ..Default::default()
                },
            )
            .await
        });
        tokio::time::sleep(Duration::from_millis(200)).await;
        tablet
            .sitzungen
            .lock()
            .unwrap()
            .senden(ZK, &hier.kennung, "", vec![vec![3]], None)
            .unwrap();
        let neu = tokio::time::timeout(Duration::from_secs(5), wartend)
            .await
            .expect("geweckt")
            .unwrap()
            .unwrap();
        assert_eq!(neu.aenderungen, vec![B64.encode([3])]);

        // Der Laptop sendet; das Tablet sieht es.
        let _: serde_json::Value = rufen(
            &laptop,
            &tablet,
            port,
            "senden",
            SitzungPost {
                notiz: ZK.into(),
                kennung: b.kennung.clone(),
                aenderungen: vec![B64.encode([4])],
                awareness: Some(B64.encode([9])),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        let fuer_tablet = tablet
            .sitzungen
            .lock()
            .unwrap()
            .seit(ZK, &hier.kennung, "", neu.nr)
            .unwrap();
        assert_eq!(fuer_tablet.aenderungen, vec![vec![4]]);
        assert_eq!(fuer_tablet.awareness, vec![(b.kennung.clone(), vec![9])]);

        // Speichern darf nur der Schreiber -- hier das Tablet, nicht der Laptop.
        let abgelehnt: Result<serde_json::Value, _> = rufen(
            &laptop,
            &tablet,
            port,
            "speichern",
            SitzungPost {
                notiz: ZK.into(),
                kennung: b.kennung.clone(),
                inhalt: Some("Milch\nBrot".into()),
                ..Default::default()
            },
        )
        .await;
        assert!(abgelehnt.is_err());

        // Geht das Tablet, speichert der Laptop.
        tablet
            .sitzungen
            .lock()
            .unwrap()
            .verlassen(ZK, &hier.kennung, "");
        let _: serde_json::Value = rufen(
            &laptop,
            &tablet,
            port,
            "speichern",
            SitzungPost {
                notiz: ZK.into(),
                kennung: b.kennung.clone(),
                inhalt: Some("Milch\nBrot".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(
            tablet
                .speicher
                .lock()
                .await
                .notiz(ZK)
                .unwrap()
                .unwrap()
                .inhalt,
            "Milch\nBrot"
        );

        // Haelt eine alte App die Sperre, gibt es keine Sitzung (und
        // umgekehrt -- das prueft der Weg `…/notiz/sperren` in dienst.rs).
        let alt = crate::sperren::Halter {
            personen_id: "p-alt".into(),
            name: "Beispiel Alt".into(),
            geraet: "fp-alt".into(),
        };
        tablet
            .sperren
            .lock()
            .unwrap()
            .nehmen(ZK, alt.clone())
            .unwrap();
        let gesperrt: Result<BeitrittLeitung, _> = rufen(
            &laptop,
            &tablet,
            port,
            "beitreten",
            SitzungPost {
                notiz: ZK.into(),
                ..Default::default()
            },
        )
        .await;
        assert!(gesperrt.unwrap_err().contains("older app"));
        tablet.sperren.lock().unwrap().loslassen(ZK, &alt);

        // Ein fremdes Geraet kommt nicht hinein.
        let draussen: Result<BeitrittLeitung, _> = rufen(
            &fremd,
            &tablet,
            port,
            "beitreten",
            SitzungPost {
                notiz: ZK.into(),
                ..Default::default()
            },
        )
        .await;
        assert!(draussen.is_err());
        // Und kann nicht unter fremder Kennung senden.
        let unter_fremder: Result<serde_json::Value, _> = rufen(
            &fremd,
            &tablet,
            port,
            "senden",
            SitzungPost {
                notiz: ZK.into(),
                kennung: b.kennung.clone(),
                aenderungen: vec![B64.encode([6])],
                ..Default::default()
            },
        )
        .await;
        assert!(unter_fremder.is_err());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn mitglieder_eines_server_projekts_treten_mit_ausweis_bei() {
        let anna = geraet("Anna");
        let ben = geraet("Ben");
        let pruefer = Arc::new(Pruefer {
            anna_fp: anna.ident.fingerabdruck.clone(),
            gespeichert: Mutex::new(None),
        });
        let ordner = tempfile::tempdir().unwrap();
        let besitzer = Arc::new(Geraet {
            name: "Besitzer".into(),
            ident: Identitaet::laden_oder_erzeugen(&ordner.path().join("i.json")).unwrap(),
            speicher: Arc::new(tokio::sync::Mutex::new(
                Speicher::im_arbeitsspeicher().unwrap(),
            )),
            gepaart: Mutex::new(vec![]),
            sitzungen: Default::default(),
            sperren: Default::default(),
            pruefer: Some(pruefer.clone()),
            _ordner: ordner,
        });
        besitzer
            .speicher
            .lock()
            .await
            .notiz_schreiben(
                &Notiz {
                    zk_id: ZK.into(),
                    titel: "Protokoll".into(),
                    inhalt: "Punkt 1".into(),
                    ..Default::default()
                },
                Protokoll::Merken,
            )
            .unwrap();
        let dienst = Dienst::starten(&besitzer.ident, 0, Default::default(), besitzer.clone())
            .await
            .unwrap();
        let port = dienst.port;
        let post = |ausweis: Option<Ausweis>| SitzungPost {
            projekt: "P".into(),
            freigabe: "F".into(),
            notiz: ZK.into(),
            ausweis,
            ..Default::default()
        };
        let ausweis = |name: &str| {
            Some(Ausweis {
                name: name.into(),
                geraet: 1,
                zeit: 1,
                signatur: "x".into(),
            })
        };

        // Nachfragen geht ohne Ausweis -- und verraet bei falscher Mappe nichts.
        let hat: serde_json::Value = rufen(&ben, &besitzer, port, "hat", post(None))
            .await
            .unwrap();
        assert_eq!(hat["hat"], serde_json::json!(true));
        let mut falsch = post(None);
        falsch.freigabe = "G".into();
        let hat: serde_json::Value = rufen(&ben, &besitzer, port, "hat", falsch).await.unwrap();
        assert_eq!(hat["hat"], serde_json::json!(false));

        // Ohne Ausweis, mit fremdem Namen, mit Annas Namen vom falschen
        // Geraet: kein Beitritt.
        assert!(
            rufen::<BeitrittLeitung>(&anna, &besitzer, port, "beitreten", post(None))
                .await
                .is_err()
        );
        assert!(
            rufen::<BeitrittLeitung>(&ben, &besitzer, port, "beitreten", post(ausweis("ben")))
                .await
                .is_err()
        );
        assert!(rufen::<BeitrittLeitung>(
            &ben,
            &besitzer,
            port,
            "beitreten",
            post(ausweis("anna"))
        )
        .await
        .is_err());

        // Anna: dabei, mit Ausgangstext; als Einzige auch Schreiberin.
        let b: BeitrittLeitung = rufen(&anna, &besitzer, port, "beitreten", post(ausweis("anna")))
            .await
            .unwrap();
        assert_eq!(b.inhalt, "Punkt 1");
        assert_eq!(b.schreiber, b.kennung);
        let mut speichern = post(None);
        speichern.kennung = b.kennung.clone();
        speichern.inhalt = Some("Punkt 1\nPunkt 2".into());
        let _: serde_json::Value = rufen(&anna, &besitzer, port, "speichern", speichern)
            .await
            .unwrap();
        let n = besitzer.speicher.lock().await.notiz(ZK).unwrap().unwrap();
        assert_eq!(n.inhalt, "Punkt 1\nPunkt 2");

        // Ein Whiteboard (W3b): dieselbe Pruefung, Inhalt vom Pruefer, und
        // gespeichert wird ueber ihn -- nicht in den Notizen.
        let wb = |ausweis: Option<Ausweis>| SitzungPost {
            projekt: "P".into(),
            notiz: "whiteboard:WB".into(),
            ausweis,
            ..Default::default()
        };
        let hat: serde_json::Value = rufen(&ben, &besitzer, port, "hat", wb(None)).await.unwrap();
        assert_eq!(hat["hat"], serde_json::json!(true));
        let mut anderes = wb(None);
        anderes.notiz = "whiteboard:XX".into();
        let hat: serde_json::Value = rufen(&ben, &besitzer, port, "hat", anderes).await.unwrap();
        assert_eq!(hat["hat"], serde_json::json!(false));
        assert!(
            rufen::<BeitrittLeitung>(&ben, &besitzer, port, "beitreten", wb(ausweis("ben")))
                .await
                .is_err()
        );
        let b: BeitrittLeitung = rufen(&anna, &besitzer, port, "beitreten", wb(ausweis("anna")))
            .await
            .unwrap();
        assert_eq!(b.titel, "Skizze");
        assert_eq!(b.inhalt, r#"{"elements":[]}"#);
        let mut speichern = wb(None);
        speichern.kennung = b.kennung.clone();
        speichern.inhalt = Some(r#"{"elements":[1]}"#.into());
        let _: serde_json::Value = rufen(&anna, &besitzer, port, "speichern", speichern)
            .await
            .unwrap();
        assert_eq!(
            pruefer.gespeichert.lock().unwrap().as_deref(),
            Some(r#"{"elements":[1]}"#)
        );
    }
}
