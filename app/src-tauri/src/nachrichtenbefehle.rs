//! Nachrichten (Phase 4): Matrix, Ende-zu-Ende **auf diesem Geraet**.
//!
//! **Der Unterschied zur Webapp, und er wird nicht weggeraeumt.** Drueben
//! haengt ein Sidecar am Konto, und der Server kann mitlesen -- das sagt die
//! Einwilligung dort ausdruecklich. Hier ist das Telefon selbst ein
//! Matrix-Geraet mit eigenen Schluesseln. openany.de sieht keine Zeile, und
//! der Verlauf reist auch nicht ueber den Abgleich (Schema-Stand 11).
//!
//! **Dieselbe Kiste wie der Sidecar** (`matrix/core`): anmelden, wieder
//! aufnehmen, senden, abgleichen, abmelden. Was dort ein Axum-Handler ruft,
//! ruft hier ein Tauri-Befehl.
//!
//! **Der Abgleich laeuft, solange das Fenster offen ist.** Im Hintergrund
//! (`hintergrund.rs`) gibt es ihn nicht: Ein dauerhafter Sync braeuchte einen
//! Vordergrunddienst mit Benachrichtigung, und das ist eine eigene
//! Entscheidung.
//!
//! **Mehrere Konten, ein Verlauf** (Tiffy, 29.09.2026). Jedes Konto hat
//! seine Sitzung, seinen SDK-Speicher und seinen Abgleich; jede Nachricht
//! weiss, zu welchem sie gehoert (Schema-Stand 16). Eine Antwort geht von dem
//! Konto, bei dem die Nachricht ankam; das erste ist das Standard-Konto.
//!
//! **Offen: die Geraete-Verifizierung** (Emoji-Vergleich). Ohne sie steht
//! dieses Geraet bei den Gegenuebern als "nicht verifiziert". Nachrichten
//! gehen trotzdem verschluesselt hin und her; die Oberflaeche sagt es.

use crate::Zustand;
use anyid_client::Tokenspeicher;
use openany_matrix_core as matrix;
use openany_store::Nachricht;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;

fn fehler(e: impl ToString) -> String {
    e.to_string()
}

/// Was aufbewahrt wird, um die Sitzung wieder aufzunehmen. **Kein Passwort.**
#[derive(Serialize, Deserialize, Clone)]
struct Sitzung {
    homeserver: String,
    mxid: String,
    device_id: String,
    access_token: String,
    /// Schluessel fuer den verschluesselten Speicher des SDK -- je Geraet
    /// zufaellig, einmal erzeugt.
    passphrase: String,
    /// Der Ordner des SDK-Speichers unter dem Programmordner. Das erste
    /// Konto (aus der Zeit mit nur einem) liegt in `matrix`.
    #[serde(default = "erster_ordner")]
    ordner: String,
}

fn erster_ordner() -> String {
    "matrix".into()
}

/// Bis Stand 16 lag EINE Sitzung im Tresor, jetzt eine Liste.
#[derive(Deserialize)]
#[serde(untagged)]
enum Abgelegt {
    Liste(Vec<Sitzung>),
    Eine(Box<Sitzung>),
}

/// Ein laufender Abgleich mit dem Homeserver.
pub struct Lauf {
    client: matrix::Client,
    aufgabe: tauri::async_runtime::JoinHandle<()>,
}

impl Lauf {
    fn laeuft(&self) -> bool {
        !self.aufgabe.inner().is_finished()
    }
}

/// Die Sitzung traegt auch die Passphrase fuer den Speicher des SDK -- sie
/// gehoert deshalb in den Tresor wie die beiden anderen Ausweise.
fn sitzungsablage(zustand: &Zustand) -> Box<dyn Tokenspeicher + Send + Sync> {
    crate::tresor::ablage(zustand.ordner.join("ausweise").join("matrix-sitzung"))
}

fn speicherordner(zustand: &Zustand, s: &Sitzung) -> PathBuf {
    zustand.ordner.join(&s.ordner)
}

/// Alle Sitzungen, das Standard-Konto zuerst.
fn sitzungen(zustand: &Zustand) -> Vec<Sitzung> {
    let Some(roh) = sitzungsablage(zustand).lesen().ok().flatten() else {
        return Vec::new();
    };
    match serde_json::from_str(&roh) {
        Ok(Abgelegt::Liste(l)) => l,
        Ok(Abgelegt::Eine(s)) => vec![*s],
        Err(_) => Vec::new(),
    }
}

fn sitzung(zustand: &Zustand, mxid: &str) -> Option<Sitzung> {
    sitzungen(zustand).into_iter().find(|s| s.mxid == mxid)
}

fn sitzungen_speichern(zustand: &Zustand, liste: &[Sitzung]) -> Result<(), String> {
    if liste.is_empty() {
        return sitzungsablage(zustand).vergessen().map_err(fehler);
    }
    sitzungsablage(zustand)
        .schreiben(&serde_json::to_string(liste).map_err(fehler)?)
        .map_err(fehler)
}

fn wiederaufnahme<'a>(s: &'a Sitzung) -> matrix::Wiederaufnahme<'a> {
    matrix::Wiederaufnahme {
        homeserver: &s.homeserver,
        mxid: &s.mxid,
        device_id: &s.device_id,
        access_token: &s.access_token,
    }
}

/// Eine Kennung aus der Ansicht: `<konto> <event_id>` (beide ohne
/// Leerzeichen). Ohne Konto -- eine Zeile von vor Stand 16, die noch offen
/// war -- das Standard-Konto.
fn teilen(zustand: &Zustand, id: &str) -> (String, String) {
    match id.split_once(' ') {
        Some((konto, ereignis)) => (konto.to_string(), ereignis.to_string()),
        None => (
            sitzungen(zustand)
                .first()
                .map(|s| s.mxid.clone())
                .unwrap_or_default(),
            id.to_string(),
        ),
    }
}

/* ── Was die Oberflaeche sieht ─────────────────────────────────────────── */

#[derive(Serialize)]
pub struct KontoLage {
    mxid: String,
    homeserver: String,
    /// Steht der Abgleich gerade?
    laeuft: bool,
    /// Warum er zuletzt abbrach -- `None`, wenn nichts war.
    fehler: Option<String>,
}

#[derive(Serialize)]
pub struct NachrichtenLage {
    /// Die verbundenen Konten, das Standard-Konto zuerst.
    konten: Vec<KontoLage>,
    /// Das Standard-Konto -- `None` ohne Konto. Für wen nur wissen will, ob
    /// Matrix geht.
    mxid: Option<String>,
    ungelesen: usize,
    /// Mit einer openany-Instanz gekoppelt? Ohne Kopplung gibt es den Weg
    /// „openany" nicht (local-first: nur zeigen, was hier geht).
    server: bool,
}

/// In derselben Form wie drueben (`MessageResource`), damit die Ansicht
/// dieselben Felder liest: `id`, `body`, `created_at`, `read_at`, `peer`.
#[derive(Serialize)]
pub struct NachrichtAnzeige {
    id: String,
    body: String,
    /// `matrix` oder `openany` -- die Ansicht zeigt es an der Zeile, denn die
    /// beiden Wege haben verschiedene Zusagen: Was ueber Matrix geht, sieht
    /// openany.de nicht; was intern geht, liegt auf dem Server.
    transport: &'static str,
    created_at: String,
    read_at: Option<String>,
    von_mir: bool,
    peer: Option<String>,
    /// Ein Anhang, ohne seine Bytes -- die holt `nachricht_anhang`.
    anhang: Option<AnhangAnzeige>,
    /// E-Mail: alle Anhänge (eine Mail kann mehrere tragen).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    anhaenge: Vec<AnhangAnzeige>,
    /// E-Mail: der Betreff.
    #[serde(skip_serializing_if = "Option::is_none")]
    betreff: Option<String>,
    /// E-Mail: an wen eine Antwort geht (die Adresse, auch wenn `peer` einen
    /// Namen aus dem Adressbuch zeigt).
    #[serde(skip_serializing_if = "Option::is_none")]
    antwort_an: Option<String>,
    /// Matrix und E-Mail: über welches eigene Konto (Kennung, Adresse). Eine
    /// Antwort geht von dort.
    #[serde(skip_serializing_if = "Option::is_none")]
    konto: Option<String>,
    /// E-Mail: `verschluesselt` oder `unlesbar` (Schlüssel fehlt).
    #[serde(skip_serializing_if = "Option::is_none")]
    pgp: Option<String>,
    /// E-Mail: `gueltig`, `ungueltig` oder `unbekannt`.
    #[serde(skip_serializing_if = "Option::is_none")]
    signatur: Option<String>,
    /// Vor Ort: Die Person ist keine bekannte (kein gemeinsames Projekt).
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    unbekannt: bool,
    /// Vor Ort: eine Anfrage -- von jemandem, der weder bekannt noch
    /// angenommen ist. Die Ansicht legt sie in den Anfragen-Ordner.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    anfrage: bool,
    /// Vor Ort und openany: Die Nachricht wartet im Postausgang.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    wartet: bool,
    /// openany: Der Server hat eine wartende Nachricht abgelehnt (Satz).
    /// Vor Ort: Das andere Geraet hat sie abgelehnt -- als Kennung aus
    /// `direkt::grund`, die Ansicht sagt es in ihrer Sprache.
    #[serde(skip_serializing_if = "Option::is_none")]
    fehler: Option<String>,
}

/// Was die Ansicht von einem Anhang wissen muss (docs/plan-email-pgp.md).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AnhangAnzeige {
    name: String,
    mime: String,
    groesse: u64,
}

/// Was im Speicher am Anhang steht (Spalte `anhang`, JSON).
///
/// `abdruck`: ein selbst gesendeter Anhang, dessen Bytes in der
/// Inhaltsablage liegen. `quelle`: wo er beim Homeserver liegt, bei
/// verschlüsselten Räumen samt Schlüssel -- kommt mit dem Echo.
#[derive(Serialize, Deserialize, Default)]
struct AnhangGespeichert {
    name: String,
    mime: String,
    groesse: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    abdruck: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    quelle: Option<String>,
}

fn anhang_lesen(roh: Option<&str>) -> Option<AnhangGespeichert> {
    serde_json::from_str(roh?).ok()
}

/// openanys eigene Obergrenze fuer einen Matrix-Anhang: „kleine Dateien".
/// Der Homeserver darf weniger erlauben (`m.upload.size`), nie mehr.
pub const ANHANG_HOECHSTENS: u64 = 10 * 1024 * 1024;

#[derive(Serialize)]
pub struct NachrichtenSeite {
    items: Vec<NachrichtAnzeige>,
    next_page: Option<usize>,
    /// Warum der interne Teil fehlt -- `None`, wenn er da ist.
    ///
    /// NICHT ALS FEHLER DES GANZEN BEFEHLS: Der Matrix-Verlauf liegt auf dem
    /// Geraet und steht auch ohne Netz. Waere der Server der Grund, die Liste
    /// gar nicht zu zeigen, saehe ein Mensch im Zug nichts mehr.
    serverfehler: Option<String>,
}

fn anzeige(n: Nachricht) -> NachrichtAnzeige {
    let anhang = anhang_lesen(n.anhang.as_deref()).map(|a| AnhangAnzeige {
        name: a.name,
        mime: a.mime,
        groesse: a.groesse,
    });
    NachrichtAnzeige {
        anhang,
        anhaenge: Vec::new(),
        betreff: None,
        antwort_an: None,
        id: format!("{} {}", n.konto, n.event_id),
        konto: Some(n.konto),
        pgp: None,
        signatur: None,
        body: n.text,
        transport: "matrix",
        created_at: n.zeit,
        read_at: n.gelesen_at,
        von_mir: n.von_mir,
        peer: n.gegenueber,
        unbekannt: false,
        anfrage: false,
        wartet: false,
        fehler: None,
    }
}

/// Eine Nachricht aus dem openany-Postausgang (postausgang.rs).
fn wartend_anzeige(w: crate::postausgang::Wartend) -> NachrichtAnzeige {
    NachrichtAnzeige {
        id: format!("{}{}", crate::postausgang::VORSILBE, w.id),
        body: w.text,
        transport: "openany",
        created_at: w.at,
        read_at: None,
        von_mir: true,
        peer: Some(w.ziel),
        anhang: None,
        anhaenge: Vec::new(),
        betreff: None,
        antwort_an: None,
        konto: None,
        pgp: None,
        signatur: None,
        unbekannt: false,
        anfrage: false,
        wartet: w.fehler.is_none(),
        fehler: w.fehler,
    }
}

/// Passt eine wartende Nachricht auf den Filter? Sie ist eigene, ohne
/// Anhang -- nur Suche und Unterhaltung zaehlen.
fn wartend_passt(w: &crate::postausgang::Wartend, f: &FilterArg) -> bool {
    if f.ungelesen || f.anhang {
        return false;
    }
    if !f.mit.is_empty() && !f.mit.iter().any(|m| m == &format!("openany:{}", w.ziel)) {
        return false;
    }
    let q = f.q.trim().to_lowercase();
    q.is_empty() || w.text.to_lowercase().contains(&q) || w.ziel.to_lowercase().contains(&q)
}

/// Eine Direktnachricht vor Ort. Geantwortet wird an die Person (`raum`).
fn nah_anzeige(n: Nachricht, g: &crate::NahGastgeber) -> NachrichtAnzeige {
    let unbekannt = !crate::direktbefehle::ist_bekannt(g, &n.raum);
    let anfrage = !n.von_mir && crate::direktbefehle::ist_anfrage(g, &n.raum);
    let wartet = n.von_mir && g.direkt.wartet(&n.event_id);
    let fehler = if n.von_mir {
        g.direkt.abgelehnt_weil(&n.event_id)
    } else {
        None
    };
    let antwort_an = Some(format!("person:{}", n.raum));
    NachrichtAnzeige {
        transport: "nah",
        konto: None,
        antwort_an,
        unbekannt,
        anfrage,
        wartet,
        fehler,
        ..anzeige(n)
    }
}

async fn lage_von(zustand: &Zustand) -> NachrichtenLage {
    // NICHT AUF DIE SPERRE WARTEN (29.09.2026). Hält sie gerade ein
    // Matrix-Gang (Wachdienst, Anhang holen), hing hier die ganze Ansicht:
    // Ladeanzeige, „+ Nachricht" aus. Belegt heißt: es wird gearbeitet.
    let laufend: Option<Vec<String>> = zustand.matrix.try_lock().ok().map(|l| {
        l.iter()
            .filter(|(_, l)| l.laeuft())
            .map(|(k, _)| k.clone())
            .collect()
    });
    // EINE Sperre für beide Zählungen. Zwei `lock()` in einem Ausdruck
    // verklemmten sich (29.09.2026): Die erste lebt bis zum Ende der
    // Anweisung, die zweite wartet auf sie -- für immer.
    let ungelesen = {
        let speicher = zustand.speicher.lock().await;
        // Ungelesene Anfragen zählen nicht mit: Sie stehen nicht im Verlauf,
        // sondern im Anfragen-Ordner, der seine eigene Zahl trägt.
        let anfragen = speicher
            .nachrichten_gefiltert(
                1,
                &openany_store::Verlaufsfilter {
                    ungelesen: true,
                    nur_konto: Some(openany_nahbereich::direkt::KONTO.into()),
                    ..Default::default()
                },
            )
            .map(|(l, _)| {
                l.iter()
                    .filter(|n| crate::direktbefehle::ist_anfrage(&zustand.gastgeber, &n.raum))
                    .count()
            })
            .unwrap_or(0);
        (speicher.ungelesene_nachrichten().unwrap_or(0) + speicher.ungelesene_mails().unwrap_or(0))
            .saturating_sub(anfragen)
    };
    let fehler = zustand.matrix_fehler.lock().await.clone();

    let konten: Vec<KontoLage> = sitzungen(zustand)
        .into_iter()
        .map(|s| KontoLage {
            laeuft: laufend.as_ref().is_none_or(|l| l.contains(&s.mxid)),
            fehler: fehler.get(&s.mxid).cloned(),
            mxid: s.mxid,
            homeserver: s.homeserver,
        })
        .collect();
    NachrichtenLage {
        mxid: konten.first().map(|k| k.mxid.clone()),
        konten,
        ungelesen,
        server: zustand.einstellungen.lock().await.verbunden(),
    }
}

#[tauri::command]
pub async fn nachrichten_lage(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<NachrichtenLage, String> {
    Ok(lage_von(&zustand).await)
}

/* ── Anmelden und abmelden ─────────────────────────────────────────────── */

/// Ein Konto hinzufuegen -- dieses Geraet wird dort ein neues Matrix-Geraet.
///
/// **Jedes Konto bekommt einen frischen Speicherordner.** Ein alter gehoerte
/// zu einem anderen Geraet (oder Konto); seine Schluessel passen nicht zu
/// dem, das gleich entsteht, und das SDK weigerte sich, ihn zu oeffnen.
#[tauri::command]
pub async fn nachrichten_anmelden(
    app: tauri::AppHandle,
    zustand: tauri::State<'_, Arc<Zustand>>,
    homeserver: String,
    benutzer: String,
    passwort: String,
) -> Result<NachrichtenLage, String> {
    let homeserver = homeserver.trim().trim_end_matches('/').to_string();
    let benutzer = benutzer.trim().to_string();
    if homeserver.is_empty() || benutzer.is_empty() || passwort.is_empty() {
        return Err("Please enter homeserver, user name and password.".into());
    }
    let mut liste = sitzungen(&zustand);
    if benutzer.starts_with('@') && liste.iter().any(|s| s.mxid.eq_ignore_ascii_case(&benutzer)) {
        return Err("This account is already connected.".into());
    }

    // Das erste Konto in `matrix` (wie bisher), jedes weitere daneben.
    let ordnername = if liste.is_empty() {
        erster_ordner()
    } else {
        format!(
            "matrix-{}",
            &uuid::Uuid::new_v4().simple().to_string()[..12]
        )
    };
    let ordner = zustand.ordner.join(&ordnername);
    let _ = std::fs::remove_dir_all(&ordner);
    std::fs::create_dir_all(&ordner).map_err(fehler)?;
    let passphrase = format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    );

    let angemeldet = match matrix::anmelden(
        matrix::Anmeldung {
            homeserver: &homeserver,
            benutzer: &benutzer,
            passwort: &passwort,
        },
        &ordner,
        &passphrase,
    )
    .await
    {
        Ok(a) => a,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&ordner);
            return Err(format!("{e:#}"));
        }
    };

    let sitzung = Sitzung {
        homeserver,
        mxid: angemeldet.mxid.to_string(),
        device_id: angemeldet.device_id.to_string(),
        access_token: angemeldet.access_token,
        passphrase,
        ordner: ordnername,
    };
    // Mit Kurzname angemeldet und doch schon da: das eben entstandene Geraet
    // wieder abmelden, sonst stuende es beim Homeserver herum.
    if liste.iter().any(|s| s.mxid == sitzung.mxid) {
        if let Ok(c) =
            matrix::wiederaufnehmen(wiederaufnahme(&sitzung), &ordner, &sitzung.passphrase).await
        {
            let _ = matrix::abmelden(&c).await;
        }
        let _ = std::fs::remove_dir_all(&ordner);
        return Err("This account is already connected.".into());
    }
    liste.push(sitzung.clone());
    sitzungen_speichern(&zustand, &liste)?;

    starten_eines(app, zustand.inner().clone(), sitzung).await;
    // Laeuft der Wachdienst schon, soll auch dieses neue Konto ihn wecken.
    crate::wachdienst::matrix_nachtragen(zustand.inner()).await;
    Ok(lage_von(&zustand).await)
}

/// Zum Standard-Konto fuer neue Nachrichten machen (nach vorn).
#[tauri::command]
pub async fn nachrichten_standard(
    zustand: tauri::State<'_, Arc<Zustand>>,
    mxid: String,
) -> Result<NachrichtenLage, String> {
    let mut liste = sitzungen(&zustand);
    let i = liste
        .iter()
        .position(|s| s.mxid == mxid)
        .ok_or("This account is not connected.")?;
    let s = liste.remove(i);
    liste.insert(0, s);
    sitzungen_speichern(&zustand, &liste)?;
    Ok(lage_von(&zustand).await)
}

#[derive(Serialize)]
pub struct Getrennt {
    /// Hat der Homeserver das Geraet abgemeldet? Wenn nicht, steht es dort
    /// noch in der Liste -- das soll die Oberflaeche sagen, nicht verschweigen.
    geraet_abgemeldet: bool,
}

/// Ein Konto trennen: Geraet abmelden, Sitzung und Schluessel weg, sein
/// Verlauf weg.
#[tauri::command]
pub async fn nachrichten_abmelden(
    zustand: tauri::State<'_, Arc<Zustand>>,
    mxid: String,
) -> Result<Getrennt, String> {
    let s = sitzung(&zustand, &mxid).ok_or("This account is not connected.")?;
    let lauf = zustand.matrix.lock().await.remove(&mxid);
    let geraet_abgemeldet = match lauf {
        Some(l) => {
            l.aufgabe.abort();
            matrix::abmelden(&l.client).await.is_ok()
        }
        // Laeuft keiner (kein Netz beim Start), fuer das Abmelden kurz einen
        // aufnehmen -- sonst bliebe das Geraet beim Homeserver stehen.
        None => match matrix::wiederaufnehmen(
            wiederaufnahme(&s),
            &speicherordner(&zustand, &s),
            &s.passphrase,
        )
        .await
        {
            Ok(c) => matrix::abmelden(&c).await.is_ok(),
            Err(_) => false,
        },
    };

    let mut liste = sitzungen(&zustand);
    liste.retain(|x| x.mxid != mxid);
    sitzungen_speichern(&zustand, &liste)?;
    let _ = std::fs::remove_dir_all(speicherordner(&zustand, &s));
    zustand.matrix_fehler.lock().await.remove(&mxid);
    zustand
        .speicher
        .lock()
        .await
        .nachrichten_vergessen(&mxid)
        .map_err(fehler)?;

    Ok(Getrennt { geraet_abgemeldet })
}

/* ── Der Abgleich ──────────────────────────────────────────────────────── */

/// Die Abgleiche starten, fuer jedes Konto, dessen nicht schon laeuft. Beim
/// Start des Programms.
pub async fn starten(app: tauri::AppHandle, zustand: Arc<Zustand>) {
    let liste = sitzungen(&zustand);
    let Some(erste) = liste.first() else {
        return;
    };
    // Einmal nach Stand 16: Was noch keinem Konto gehoert, stammt aus der
    // Zeit mit genau einem -- und das steht jetzt vorn. Die alte Form im
    // Tresor gleich als Liste zurueckschreiben.
    let _ = zustand
        .speicher
        .lock()
        .await
        .nachrichten_zuordnen(&erste.mxid);
    let _ = sitzungen_speichern(&zustand, &liste);
    for s in liste {
        starten_eines(app.clone(), zustand.clone(), s).await;
    }
}

async fn starten_eines(app: tauri::AppHandle, zustand: Arc<Zustand>, sitzung: Sitzung) {
    let mut laeufe = zustand.matrix.lock().await;
    if laeufe.get(&sitzung.mxid).is_some_and(Lauf::laeuft) {
        return;
    }

    let client = match matrix::wiederaufnehmen(
        wiederaufnahme(&sitzung),
        &speicherordner(&zustand, &sitzung),
        &sitzung.passphrase,
    )
    .await
    {
        Ok(c) => c,
        Err(e) => {
            zustand
                .matrix_fehler
                .lock()
                .await
                .insert(sitzung.mxid.clone(), format!("{e:#}"));
            return;
        }
    };

    let aufgabe = tauri::async_runtime::spawn(schleife(
        app,
        zustand.clone(),
        client.clone(),
        sitzung.mxid.clone(),
    ));
    laeufe.insert(sitzung.mxid, Lauf { client, aufgabe });
}

/// Gleicht ab, bis es bricht -- und versucht es dann wieder, mit wachsender
/// Pause (15 s bis 2 min). Dieselbe Regel wie im Sidecar: Ein Abgleich, der
/// einmal abreisst und nie wiederkommt, sieht aus wie einer, dem niemand
/// schreibt.
///
/// **Die Pause faellt nach einer geglueckten Runde auf 15 s zurueck.** Sie
/// wuchs sonst mit jedem Abriss weiter, auch ueber Stunden hinweg mit Netz
/// dazwischen -- denn zurueckgesetzt wurde sie nur bei `Ok`, und das kommt
/// nie. Nach einem Tag im Zug wartete die App zwei Minuten, bevor sie eine
/// Nachricht holte, die laengst da war.
async fn schleife(
    app: tauri::AppHandle,
    zustand: Arc<Zustand>,
    client: matrix::Client,
    mxid: String,
) {
    let anfangs = std::time::Duration::from_secs(15);
    let hoechstens = std::time::Duration::from_secs(120);
    let mut warten = anfangs;

    loop {
        let app_h = app.clone();
        let zustand_h = zustand.clone();
        let mxid_h = mxid.clone();

        let app_r = app.clone();
        let zustand_r = zustand.clone();
        let mxid_r = mxid.clone();
        // Wurde ueberhaupt eine Runde fertig? Nur dann war die Leitung da.
        let lief = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let lief_r = lief.clone();

        let ergebnis = matrix::abgleichen(
            &client,
            move |eingang| {
                let app = app_h.clone();
                let zustand = zustand_h.clone();
                let mxid = mxid_h.clone();
                async move {
                    // Weder Text noch Anhang -- dafuer hat openany keine Zeile.
                    if !eingang.hat_inhalt() {
                        return;
                    }
                    let neu = zustand
                        .speicher
                        .lock()
                        .await
                        .nachricht_ablegen(&aus_eingang(eingang, &mxid))
                        .unwrap_or(false);
                    if neu {
                        melden(&app);
                    }
                }
            },
            // JEDE GEGLUECKTE RUNDE LOESCHT DIE STOERUNG. Frueher stand sie,
            // bis `abgleichen` mit `Ok` zurueckkam -- und das tut ein
            // laufender Abgleich nie. Eine Minute ohne Netz hinterliess
            // deshalb eine Warnung, die auch nach dem Wiederverbinden blieb.
            move || {
                let app = app_r.clone();
                let zustand = zustand_r.clone();
                let lief = lief_r.clone();
                let mxid = mxid_r.clone();
                async move {
                    lief.store(true, std::sync::atomic::Ordering::Relaxed);
                    let mut fehler = zustand.matrix_fehler.lock().await;
                    if fehler.remove(&mxid).is_some() {
                        drop(fehler);
                        melden(&app);
                    }
                }
            },
        )
        .await;

        if lief.load(std::sync::atomic::Ordering::Relaxed) {
            warten = anfangs;
        }

        match ergebnis {
            Ok(()) => warten = anfangs,
            Err(e) => {
                zustand
                    .matrix_fehler
                    .lock()
                    .await
                    .insert(mxid.clone(), kurzgefasst(&format!("{e:#}")));
                melden(&app);
                tokio::time::sleep(warten).await;
                warten = (warten * 2).min(hoechstens);
                continue;
            }
        }
        zustand.matrix_fehler.lock().await.remove(&mxid);
    }
}

/// Was von einem abgerissenen Abgleich in der Oberflaeche stehen soll.
///
/// **Warum nicht `{e:#}`.** Bricht der Abgleich, weil das Geraet gerade kein
/// Netz hat, haengt das SDK die ganze Sync-Adresse an -- samt `since`-Marke,
/// hundert Zeichen ohne Leerraum. Das half niemandem und sprengte die Zeile.
/// Ein Netzproblem ist ausserdem kein Fehler, den jemand beheben soll: Der
/// naechste Versuch laeuft von selbst.
fn kurzgefasst(voll: &str) -> String {
    let klein = voll.to_lowercase();
    let netz = [
        "dns error",
        "error sending request",
        "connect",
        "timed out",
        "os error",
    ];

    if netz.iter().any(|w| klein.contains(w)) {
        "No connection to the homeserver -- openany keeps trying.".into()
    } else {
        voll.to_string()
    }
}

/// Eine Zeile aus einem Ereignis. **Das eigene Echo wird hier erkannt**, an
/// der Kennung -- der Sidecar ueberlaesst das Laravel, die App hat niemanden,
/// dem sie es ueberlassen koennte.
fn aus_eingang(e: matrix::Eingang, mxid: &str) -> Nachricht {
    let konto = mxid.to_string();
    let von_mir = e.absender.eq_ignore_ascii_case(mxid);
    let gegenueber = if von_mir {
        e.gegenueber
    } else {
        Some(e.absender.clone())
    };
    let zeit = chrono::DateTime::from_timestamp_millis(e.zeit_ms as i64)
        .unwrap_or_else(chrono::Utc::now)
        .to_rfc3339();
    let anhang = e.anhang.and_then(|a| {
        serde_json::to_string(&AnhangGespeichert {
            name: a.name,
            mime: a.mime,
            groesse: a.groesse,
            abdruck: None,
            quelle: Some(a.quelle),
        })
        .ok()
    });

    Nachricht {
        konto,
        event_id: e.event_id,
        raum: e.raum,
        gegenueber,
        absender: e.absender,
        von_mir,
        text: e.text,
        zeit,
        // Die eigenen sind gelesen -- auch die von einem anderen Geraet.
        gelesen_at: von_mir.then(|| chrono::Utc::now().to_rfc3339()),
        anhang,
    }
}

/// Dem offenen Fenster sagen, dass etwas kam. Ohne Berechtigung: Ein
/// Ereignis ueber `eval` braucht keine, ein `listen` in der Oberflaeche
/// schon -- und eine fehlende Berechtigung schweigt (16.09.2026).
pub(crate) fn melden(app: &tauri::AppHandle) {
    if let Some(fenster) = app.get_webview_window("main") {
        let _ = fenster.eval("window.dispatchEvent(new Event('openany-nachrichten'))");
    }
}

/* ── Fuer den Wachdienst (Phase 6) ─────────────────────────────────────── */

/// Die App-Kennung beim Homeserver. Zusammen mit dem `pushkey` bestimmt sie
/// das Push-Ziel -- ein zweites Eintragen ersetzt das erste.
const PUSH_APP_ID: &str = "de.openany.app";

/// Ein Client fuer einen kurzen Gang -- **der laufende, wenn es einen gibt.**
///
/// Zwei Clients auf demselben verschluesselten Speicher gleichzeitig waeren
/// zwei Schreiber auf einer SQLite-Datei samt Krypto-Zustand: im besten
/// Fall eine Sperre, im schlechtesten ein Schluessel, der doppelt verbraucht
/// wird. Laeuft der Abgleich der App, nimmt der Wachdienst dessen Client.
/// Laeuft keiner, haelt er die Sperre `zustand.matrix`, solange er arbeitet
/// -- ein `starten` der App wartet dann, statt daneben einen zu bauen.
async fn mit_client<T, F, Fut>(zustand: &Arc<Zustand>, mxid: &str, f: F) -> Option<T>
where
    F: FnOnce(matrix::Client, String, bool) -> Fut,
    Fut: std::future::Future<Output = Option<T>>,
{
    // EIN GANG DARF DIE SPERRE NICHT FÜR IMMER HALTEN (29.09.2026). Riss
    // mittendrin das Netz ab (WLAN-Wechsel), wartete ein Abgleich oder ein
    // Download ohne Ende -- und mit ihm alles, was die Sperre braucht. Nach
    // 90 Sekunden wird aufgegeben; der nächste Anlauf kommt von selbst.
    const HOECHSTENS: std::time::Duration = std::time::Duration::from_secs(90);

    let sitzung = sitzung(zustand, mxid)?;
    let laeufe = zustand.matrix.lock().await;
    if let Some(l) = laeufe.get(mxid).filter(|l| l.laeuft()) {
        let client = l.client.clone();
        drop(laeufe);
        return tokio::time::timeout(HOECHSTENS, f(client, sitzung.mxid, true))
            .await
            .ok()
            .flatten();
    }
    let client = matrix::wiederaufnehmen(
        wiederaufnahme(&sitzung),
        &speicherordner(zustand, &sitzung),
        &sitzung.passphrase,
    )
    .await
    .ok()?;
    let ergebnis = tokio::time::timeout(HOECHSTENS, f(client, sitzung.mxid.clone(), false))
        .await
        .ok()
        .flatten();
    drop(laeufe);
    ergebnis
}

/// ntfy als Push-Ziel beim Homeserver eintragen (oder austragen) -- fuer
/// jedes Konto. `None`, wenn keines verbunden ist; ein Fehler nennt die
/// Konten, bei denen es nicht ging.
pub(crate) async fn weckruf(
    zustand: &Arc<Zustand>,
    pushkey: &str,
    gateway: &str,
    geraetename: &str,
    eintragen: bool,
) -> Option<Result<(), String>> {
    let liste = sitzungen(zustand);
    if liste.is_empty() {
        return None;
    }
    let mut fehler = Vec::new();
    for s in liste {
        let r = mit_client(zustand, &s.mxid, |client, _, _| async move {
            let r = if eintragen {
                matrix::weckruf_eintragen(&client, PUSH_APP_ID, pushkey, gateway, geraetename).await
            } else {
                matrix::weckruf_austragen(&client, PUSH_APP_ID, pushkey).await
            };
            Some(r.map_err(|e| format!("{e:#}")))
        })
        .await;
        match r {
            Some(Ok(())) => {}
            Some(Err(e)) => fehler.push(format!("{}: {e}", s.mxid)),
            None => fehler.push(format!("{}: not reachable", s.mxid)),
        }
    }
    Some(if fehler.is_empty() {
        Ok(())
    } else {
        Err(fehler.join("; "))
    })
}

/// Nach einem Matrix-Signal: die Nachricht holen, entschluesselt ablegen
/// und `(Absender, Anfang)` fuer die Benachrichtigung zurueckgeben.
///
/// Laeuft der Abgleich der App, holt DER sie -- hier wird nur kurz
/// gewartet, bis sie im Speicher steht. Sonst ein einzelner Durchgang.
/// `None`, wenn nichts Fremdes kam (das eigene Echo von einem anderen Geraet
/// weckt auch, soll aber nichts melden).
pub(crate) async fn im_hintergrund_holen(
    zustand: &Arc<Zustand>,
    ereignis: Option<&str>,
) -> Option<(String, String)> {
    // Welches Konto gemeint ist, sagt das Signal nicht (ein Push-Ziel fuer
    // alle). Also der Reihe nach, bis eines etwas Fremdes gefunden hat.
    for s in sitzungen(zustand) {
        if let Some(treffer) = im_hintergrund_holen_bei(zustand, &s.mxid, ereignis).await {
            return Some(treffer);
        }
    }
    None
}

async fn im_hintergrund_holen_bei(
    zustand: &Arc<Zustand>,
    konto: &str,
    ereignis: Option<&str>,
) -> Option<(String, String)> {
    let z = zustand.clone();
    mit_client(zustand, konto, |client, mxid, laeuft| async move {
        if laeuft {
            for _ in 0..10 {
                if let Some(treffer) = suchen(&z, ereignis).await {
                    return Some(treffer);
                }
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
            return None;
        }
        let ablage = z.clone();
        let r = matrix::einmal_abgleichen(&client, move |eingang| {
            let z = ablage.clone();
            let mxid = mxid.clone();
            async move {
                if !eingang.hat_inhalt() {
                    return;
                }
                let _ = z
                    .speicher
                    .lock()
                    .await
                    .nachricht_ablegen(&aus_eingang(eingang, &mxid));
            }
        })
        .await;
        if r.is_err() {
            return None;
        }
        suchen(&z, ereignis).await
    })
    .await
}

/// Die gemeldete Nachricht im Speicher -- oder, ohne Kennung, die neueste
/// ungelesene von jemand anderem. Schreiben sich zwei eigene Konten, steht
/// dasselbe Ereignis zweimal da; gemeldet wird die empfangene Zeile.
async fn suchen(zustand: &Zustand, ereignis: Option<&str>) -> Option<(String, String)> {
    let (liste, _) = zustand.speicher.lock().await.nachrichten(1).ok()?;
    let n = liste.into_iter().find(|n| {
        !n.von_mir
            && match ereignis {
                Some(id) => n.event_id == id,
                None => n.gelesen_at.is_none(),
            }
    })?;
    let mut anfang: String = n.text.chars().take(120).collect();
    if anfang.is_empty() {
        if let Some(a) = anhang_lesen(n.anhang.as_deref()) {
            anfang = format!("📎 {}", a.name);
        }
    }
    Some((n.absender, anfang))
}

/* ── Verlauf ───────────────────────────────────────────────────────────── */

/// Der Verlauf -- **beide Wege in einer Liste**, nach Zeit sortiert.
///
/// **Warum zusammen und nicht in zwei Reitern.** Es ist ein Postfach. Wer
/// nachsieht, ob jemand geschrieben hat, will nicht erst waehlen, WOMIT
/// jemand geschrieben haben koennte. Die Zeile sagt den Weg, und die beiden
/// unterscheiden sich in dem, was sie zusagen: Matrix liegt nur hier,
/// openany liegt auf dem Server.
///
/// **Der Serverteil darf fehlen.** Der Matrix-Verlauf steht auch ohne Netz;
/// daran die ganze Liste scheitern zu lassen hiesse, im Zug gar nichts mehr
/// zu zeigen. Was fehlt, sagt `serverfehler`.
/// FILTER UND SUCHE (Tiffy, 01.10.2026), wie die Ansicht sie schickt. `mit`
/// nennt eine Unterhaltung als `weg:wert` (`matrix:@x:y`, `email:a@b`,
/// `openany:name`).
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct FilterArg {
    wege: Vec<String>,
    ungelesen: bool,
    anhang: bool,
    q: String,
    mit: Vec<String>,
}

/// Der Filter, zerlegt in die drei Quellen. `None` heißt: diese Quelle
/// gar nicht erst fragen (Weg abgewählt, oder die Unterhaltung kennt ihn
/// nicht).
struct Teile {
    matrix: Option<openany_store::Verlaufsfilter>,
    email: Option<openany_store::Verlaufsfilter>,
    /// Direktnachrichten vor Ort (direktbefehle.rs).
    nah: Option<openany_store::Verlaufsfilter>,
    openany: Option<Vec<(String, String)>>,
}

impl FilterArg {
    fn teile(&self, speicher: &openany_store::Speicher) -> Teile {
        let mit_von = |weg: &str| -> Vec<String> {
            self.mit
                .iter()
                .filter_map(|m| m.split_once(':'))
                .filter(|(w, _)| *w == weg)
                .map(|(_, wert)| wert.to_string())
                .collect()
        };
        // Wer im Adressbuch auf die Suche passt: Dessen Nachrichten treffen
        // auch, wenn im Text nichts davon steht.
        let suche = self.q.trim().to_lowercase();
        let passende: Vec<openany_store::Kontakt> = if suche.is_empty() {
            Vec::new()
        } else {
            speicher
                .kontakte()
                .unwrap_or_default()
                .into_iter()
                .filter(|k| {
                    k.papierkorb_at.is_none() && k.anzeigename.to_lowercase().contains(&suche)
                })
                .collect()
        };
        let kennungen = |art: &str| -> Vec<String> {
            passende
                .iter()
                .flat_map(|k| k.wege.iter())
                .filter(|w| w.art == art && !w.wert.trim().is_empty())
                .map(|w| {
                    if art == "email" {
                        w.wert.trim().to_lowercase()
                    } else {
                        w.wert.trim().to_string()
                    }
                })
                .collect()
        };
        let teil = |weg: &str| -> Option<openany_store::Verlaufsfilter> {
            if !self.wege.is_empty() && !self.wege.iter().any(|w| w == weg) {
                return None;
            }
            let mit = mit_von(weg);
            if !self.mit.is_empty() && mit.is_empty() {
                return None;
            }
            Some(openany_store::Verlaufsfilter {
                ungelesen: self.ungelesen,
                anhang: self.anhang,
                suche: self.q.clone(),
                suche_gegenueber: kennungen(weg),
                mit,
                nur_konto: None,
                ohne_konto: None,
            })
        };
        // Matrix und „Vor Ort" teilen sich die Tabelle; das Konto trennt sie.
        let matrix = teil("matrix").map(|mut f| {
            f.ohne_konto = Some(openany_nahbereich::direkt::KONTO.into());
            f
        });
        let nah = teil("nah").map(|mut f| {
            f.nur_konto = Some(openany_nahbereich::direkt::KONTO.into());
            f
        });
        let email = teil("email");
        // Der Server liefert hier nur den internen Weg: Matrix steht schon
        // entschlüsselt im eigenen Speicher (siehe `server_zeile`).
        let openany = teil("openany").map(|f| {
            let mut p = vec![("wege[]".to_string(), "openany".to_string())];
            if f.ungelesen {
                p.push(("ungelesen".into(), "1".into()));
            }
            if f.anhang {
                p.push(("anhang".into(), "1".into()));
            }
            if !f.suche.trim().is_empty() {
                p.push(("q".into(), f.suche.trim().to_string()));
            }
            for m in f.mit {
                p.push(("mit[]".into(), format!("openany:{m}")));
            }
            p
        });
        Teile {
            matrix,
            email,
            nah,
            openany,
        }
    }
}

#[tauri::command]
pub async fn nachrichten_liste(
    app: tauri::AppHandle,
    zustand: tauri::State<'_, Arc<Zustand>>,
    seite: Option<usize>,
    filter: Option<FilterArg>,
) -> Result<NachrichtenSeite, String> {
    let seite = seite.unwrap_or(1).max(1);
    let filter = filter.unwrap_or_default();

    let mut items: Vec<NachrichtAnzeige> = Vec::new();
    let mut mehr_da = false;

    // Matrix und E-Mail aus dem eigenen Speicher, unter EINER Sperre. Die
    // Gegenstelle einer Mail mit dem Namen aus dem Adressbuch, sonst mit
    // ihrer Adresse (Tiffy, 29.09.2026).
    let teile = {
        let speicher = zustand.speicher.lock().await;
        let teile = filter.teile(&speicher);
        if let Some(f) = &teile.matrix {
            let (liste, mehr) = speicher.nachrichten_gefiltert(seite, f).map_err(fehler)?;
            items.extend(liste.into_iter().map(anzeige));
            mehr_da = mehr_da || mehr;
        }
        if let Some(f) = &teile.nah {
            let (liste, mehr) = speicher.nachrichten_gefiltert(seite, f).map_err(fehler)?;
            items.extend(
                liste
                    .into_iter()
                    .map(|n| nah_anzeige(n, &zustand.gastgeber))
                    // Anfragen liegen im eigenen Ordner, nicht im Verlauf.
                    .filter(|a| !a.anfrage),
            );
            mehr_da = mehr_da || mehr;
        }
        if let Some(f) = &teile.email {
            let (mails, mehr) = speicher.mails_gefiltert(seite, f).map_err(fehler)?;
            let namen = email_namen(&speicher);
            items.extend(mails.into_iter().map(|m| mail_anzeige(m, &namen)));
            mehr_da = mehr_da || mehr;
        }
        teile
    };
    let mut serverfehler = None;

    if let Some(p) = &teile.openany {
        // Was wartet, im Hintergrund hinaus -- ohne Netz soll die Liste nicht
        // auf eine Zeitueberschreitung warten. Ging etwas hinaus, laedt die
        // Ansicht neu (`openany-nachrichten`).
        {
            let zustand = zustand.inner().clone();
            tauri::async_runtime::spawn(async move {
                if crate::postausgang::leeren(&zustand).await > 0 {
                    melden(&app);
                }
            });
        }
        if seite == 1 {
            items.extend(
                crate::postausgang::alle(&zustand)
                    .into_iter()
                    .filter(|w| wartend_passt(w, &filter))
                    .map(wartend_anzeige),
            );
        }
        if let Some(client) = crate::server_client(&zustand).await {
            match client.nachrichten_gefiltert(seite, p).await {
                Ok(roh) => {
                    let (server_items, server_mehr) = server_seite(&roh);
                    items.extend(server_items);
                    mehr_da = mehr_da || server_mehr;
                }
                Err(e) => serverfehler = Some(format!("{e}")),
            }
        }
    }

    // Absteigend nach Zeit. Die Zeitangaben sind beide ISO-8601 in UTC, und
    // dafuer ist die Zeichenfolge dieselbe Ordnung wie das Datum -- deshalb
    // hier kein Parsen, das bei einer krummen Angabe stolpern koennte.
    items.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(NachrichtenSeite {
        items,
        next_page: mehr_da.then_some(seite + 1),
        serverfehler,
    })
}

/// Eine Zeile in „Nach Kontakt".
#[derive(Serialize)]
pub struct UnterhaltungAnzeige {
    name: String,
    /// Alle Wege dieses Gegenübers, als `weg:wert` -- so gehen sie als
    /// `mit` in den Filter zurück.
    mit: Vec<String>,
    ungelesen: usize,
    zuletzt: NachrichtAnzeige,
}

#[derive(Serialize)]
pub struct Unterhaltungen {
    items: Vec<UnterhaltungAnzeige>,
    #[serde(skip_serializing_if = "Option::is_none")]
    serverfehler: Option<String>,
}

/// NACH KONTAKT (Tiffy, 01.10.2026): je Gegenüber eine Zeile, über alle drei
/// Wege. Steht jemand im Adressbuch, wird aus Kennung, Adresse und Konto
/// EINE Zeile -- ein Tipp darauf zeigt dann den ganzen Verlauf mit ihm.
#[tauri::command]
pub async fn nachrichten_unterhaltungen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    filter: Option<FilterArg>,
) -> Result<Unterhaltungen, String> {
    let filter = filter.unwrap_or_default();
    // (weg:wert) -> Kontakt, damit Zeilen eines Menschen zusammenfallen.
    let mut roh: Vec<(String, String, usize, NachrichtAnzeige)> = Vec::new(); // (weg, wert, offen, letzte)
    let (teile, kontakte) = {
        let speicher = zustand.speicher.lock().await;
        let teile = filter.teile(&speicher);
        if let Some(f) = &teile.matrix {
            for (n, offen) in speicher.nachrichten_unterhaltungen(f).map_err(fehler)? {
                let wert = n.gegenueber.clone().unwrap_or_default();
                roh.push(("matrix".into(), wert, offen, anzeige(n)));
            }
        }
        if let Some(f) = &teile.nah {
            for (n, offen) in speicher.nachrichten_unterhaltungen(f).map_err(fehler)? {
                // Anfragen liegen im eigenen Ordner (direktbefehle.rs).
                if crate::direktbefehle::ist_anfrage(&zustand.gastgeber, &n.raum) {
                    continue;
                }
                let wert = n.gegenueber.clone().unwrap_or_default();
                roh.push((
                    "nah".into(),
                    wert,
                    offen,
                    nah_anzeige(n, &zustand.gastgeber),
                ));
            }
        }
        if let Some(f) = &teile.email {
            let namen = email_namen(&speicher);
            for (m, offen) in speicher.mails_unterhaltungen(f).map_err(fehler)? {
                let wert = m.gegenueber.clone();
                roh.push(("email".into(), wert, offen, mail_anzeige(m, &namen)));
            }
        }
        let kontakte: Vec<openany_store::Kontakt> = speicher
            .kontakte()
            .unwrap_or_default()
            .into_iter()
            .filter(|k| k.papierkorb_at.is_none())
            .collect();
        (teile, kontakte)
    };

    let mut serverfehler = None;
    if let Some(p) = &teile.openany {
        if let Some(client) = crate::server_client(&zustand).await {
            match client.unterhaltungen(p).await {
                Ok(antwort) => {
                    for z in antwort
                        .get("items")
                        .and_then(|w| w.as_array())
                        .into_iter()
                        .flatten()
                    {
                        let Some(letzte) = z.get("zuletzt").and_then(server_zeile) else {
                            continue;
                        };
                        let offen =
                            z.get("ungelesen").and_then(|w| w.as_u64()).unwrap_or(0) as usize;
                        let name = letzte.peer.clone().unwrap_or_default();
                        roh.push(("openany".into(), name, offen, letzte));
                    }
                }
                Err(e) => serverfehler = Some(format!("{e}")),
            }
        }
    }

    let kontakt_von = |weg: &str, wert: &str| -> Option<&openany_store::Kontakt> {
        let wert = wert.trim().to_lowercase();
        kontakte.iter().find(|k| {
            k.wege
                .iter()
                .any(|w| w.art == weg && w.wert.trim().to_lowercase() == wert)
        })
    };

    let mut zeilen: Vec<(String, UnterhaltungAnzeige)> = Vec::new();
    for (weg, wert, offen, letzte) in roh {
        let kontakt = kontakt_von(&weg, &wert);
        let schluessel = kontakt
            .map(|k| format!("k:{}", k.uuid))
            .unwrap_or_else(|| format!("{weg}:{wert}"));
        let name = kontakt
            .map(|k| k.anzeigename.clone())
            .filter(|n| !n.trim().is_empty())
            .or_else(|| letzte.peer.clone())
            .unwrap_or_else(|| wert.clone());
        let mit = format!("{weg}:{wert}");
        match zeilen.iter_mut().find(|(s, _)| *s == schluessel) {
            Some((_, z)) => {
                z.mit.push(mit);
                z.ungelesen += offen;
                if letzte.created_at > z.zuletzt.created_at {
                    z.zuletzt = letzte;
                }
            }
            None => zeilen.push((
                schluessel,
                UnterhaltungAnzeige {
                    name,
                    mit: vec![mit],
                    ungelesen: offen,
                    zuletzt: letzte,
                },
            )),
        }
    }
    let mut items: Vec<UnterhaltungAnzeige> = zeilen.into_iter().map(|(_, z)| z).collect();
    items.sort_by(|a, b| b.zuletzt.created_at.cmp(&a.zuletzt.created_at));
    Ok(Unterhaltungen {
        items,
        serverfehler,
    })
}

/// Adresse (klein) -> Name, aus den E-Mail-Wegen des Adressbuchs.
fn email_namen(speicher: &openany_store::Speicher) -> std::collections::HashMap<String, String> {
    let mut namen = std::collections::HashMap::new();
    for k in speicher.kontakte().unwrap_or_default() {
        if k.papierkorb_at.is_some() || k.anzeigename.trim().is_empty() {
            continue;
        }
        for w in k.wege.iter().filter(|w| w.art == "email") {
            namen
                .entry(w.wert.trim().to_lowercase())
                .or_insert_with(|| k.anzeigename.clone());
        }
    }
    namen
}

fn mail_anzeige(
    m: openany_store::Mail,
    namen: &std::collections::HashMap<String, String>,
) -> NachrichtAnzeige {
    let anhaenge: Vec<AnhangAnzeige> = serde_json::from_str::<Vec<AnhangGespeichert>>(&m.anhaenge)
        .unwrap_or_default()
        .into_iter()
        .map(|a| AnhangAnzeige {
            name: a.name,
            mime: a.mime,
            groesse: a.groesse,
        })
        .collect();
    let peer = namen
        .get(&m.gegenueber)
        .cloned()
        .unwrap_or_else(|| m.gegenueber.clone());
    NachrichtAnzeige {
        id: m.id,
        // Unlesbar: In `text` steht die verschlüsselte Nachricht -- die
        // gehört nicht in den Verlauf; die Ansicht sagt, was fehlt.
        body: if m.pgp.as_deref() == Some("unlesbar") {
            String::new()
        } else {
            m.text
        },
        transport: "email",
        created_at: m.zeit,
        read_at: m.gelesen_at,
        von_mir: m.von_mir,
        peer: Some(peer),
        anhang: None,
        anhaenge,
        betreff: Some(m.betreff),
        antwort_an: Some(m.gegenueber),
        konto: Some(m.postfach),
        pgp: m.pgp,
        signatur: m.signatur,
        unbekannt: false,
        anfrage: false,
        wartet: false,
        fehler: None,
    }
}

/// Was der Server schickt, in die Form der Ansicht.
///
/// Der Server nennt das Gegenueber beim internen Weg nicht als Kennung,
/// sondern als Konto: `sender`/`recipient` mit `name`. Welcher von beiden das
/// Gegenueber ist, haengt daran, wer geschrieben hat.
fn server_seite(roh: &serde_json::Value) -> (Vec<NachrichtAnzeige>, bool) {
    let mehr = roh.get("next_page").map(|w| !w.is_null()).unwrap_or(false);

    let items = roh
        .get("items")
        .and_then(|w| w.as_array())
        .map(|zeilen| zeilen.iter().filter_map(server_zeile).collect())
        .unwrap_or_default();

    (items, mehr)
}

fn server_zeile(z: &serde_json::Value) -> Option<NachrichtAnzeige> {
    let text = |feld: &str| z.get(feld).and_then(|w| w.as_str()).map(str::to_string);

    // Matrix-Zeilen kommen vom Server auch mit -- die stehen hier aber schon
    // aus dem eigenen Speicher, und zwar entschluesselt. Zweimal dieselbe
    // Nachricht waere das Gegenteil von hilfreich.
    if text("transport").as_deref() == Some("matrix") {
        return None;
    }

    let von_mir = z.get("von_mir").and_then(|w| w.as_bool()).unwrap_or(false);
    let name = |feld: &str| {
        z.get(feld)
            .and_then(|w| w.get("name"))
            .and_then(|w| w.as_str())
            .map(str::to_string)
    };

    Some(NachrichtAnzeige {
        id: z
            .get("id")
            .map(|w| w.to_string())?
            .trim_matches('"')
            .to_string(),
        body: text("body").unwrap_or_default(),
        transport: "openany",
        created_at: text("created_at").unwrap_or_default(),
        read_at: text("read_at"),
        von_mir,
        peer: if von_mir {
            name("recipient")
        } else {
            name("sender")
        },
        anhang: None,
        anhaenge: Vec::new(),
        betreff: None,
        antwort_an: None,
        konto: None,
        pgp: None,
        signatur: None,
        unbekannt: false,
        anfrage: false,
        wartet: false,
        fehler: None,
    })
}

/// Eine Nachricht -- ueber Matrix oder intern ueber openany.
///
/// **Zwei Wege, ein Befehl, und der Weg steht dabei.** Am Ziel zu erraten,
/// was gemeint ist, waere falsch: Ein openany-Konto darf „@tiffy:matrix.org"
/// heissen. Dieselbe Entscheidung wie drueben im `MessageController`.
///
/// Ueber Matrix sendet dieses Geraet SELBST, mit eigenen Schluesseln; intern
/// legt der Server die Zeile an. Deshalb landet nur die Matrix-Zeile hier im
/// Speicher -- die interne holt die Liste beim naechsten Laden vom Server.
#[tauri::command]
pub async fn nachricht_senden(
    app: tauri::AppHandle,
    zustand: tauri::State<'_, Arc<Zustand>>,
    ziel: String,
    text: String,
    weg: Option<String>,
    von: Option<String>,
) -> Result<NachrichtAnzeige, String> {
    let ziel = ziel.trim().to_string();
    let text = text.trim().to_string();
    if text.is_empty() {
        return Err("Please enter a message.".into());
    }

    // Vor Ort, ohne Server (direktbefehle.rs).
    if weg.as_deref() == Some("nah") {
        let (n, name) = crate::direktbefehle::senden(&zustand, &ziel, &text).await?;
        let speicher = zustand.speicher.lock().await;
        let gespeichert = speicher
            .nachricht(openany_nahbereich::direkt::KONTO, &n.id)
            .map_err(fehler)?
            .ok_or("The message has not been stored.")?;
        drop(speicher);
        let mut a = nah_anzeige(gespeichert, &zustand.gastgeber);
        a.peer = Some(name);
        return Ok(a);
    }

    if weg.as_deref() == Some("openany") {
        if ziel.is_empty() {
            return Err("Please enter an openany name.".into());
        }
        let client = crate::server_client(&zustand)
            .await
            .ok_or("Not connected to openany.de.")?;

        // Ohne Netz: in den Postausgang statt eines Fehlers (postausgang.rs).
        let roh = match client.nachricht_senden(&ziel, &text).await {
            Ok(roh) => roh,
            Err(e) if crate::postausgang::spaeter_nochmal(&e) => {
                let w = crate::postausgang::einreihen(&zustand, &ziel, &text);
                return Ok(wartend_anzeige(w));
            }
            Err(e) => return Err(format!("{e}")),
        };

        melden(&app);

        return server_zeile(&roh)
            .ok_or_else(|| "Unexpected response from the server.".to_string());
    }

    if !(ziel.starts_with('@') && ziel.contains(':')) {
        return Err("A Matrix ID looks like this: @name:server.org".into());
    }

    let (client, mxid) = laufender_client(&zustand, von.as_deref()).await?;

    let event_id = matrix::senden(&client, &ziel, &text)
        .await
        .map_err(|e| format!("{e:#}"))?;

    let jetzt = chrono::Utc::now().to_rfc3339();
    let n = Nachricht {
        konto: mxid.clone(),
        event_id: event_id.to_string(),
        raum: String::new(),
        gegenueber: Some(ziel),
        absender: mxid,
        von_mir: true,
        text,
        zeit: jetzt.clone(),
        gelesen_at: Some(jetzt),
        anhang: None,
    };
    zustand
        .speicher
        .lock()
        .await
        .nachricht_ablegen(&n)
        .map_err(fehler)?;
    melden(&app);
    Ok(anzeige(n))
}

/// Der laufende Matrix-Client eines Kontos (`von`, sonst das
/// Standard-Konto) und dessen Kennung -- oder der Grund, warum es keinen
/// gibt.
async fn laufender_client(
    zustand: &Zustand,
    von: Option<&str>,
) -> Result<(matrix::Client, String), String> {
    let mxid = match von.filter(|v| !v.is_empty()) {
        Some(v) => v.to_string(),
        None => sitzungen(zustand)
            .first()
            .map(|s| s.mxid.clone())
            .ok_or("No Matrix account connected.")?,
    };
    let laeufe = zustand.matrix.lock().await;
    let l = laeufe
        .get(&mxid)
        .ok_or_else(|| format!("{mxid}: not connected, or the connection is down right now."))?;
    Ok((l.client.clone(), mxid))
}

/// Wie groß ein Anhang über Matrix sein darf, in Bytes: openanys „kleine
/// Dateien", oder weniger, wenn der Homeserver es sagt. Die Ansicht prüft
/// damit, bevor sie die Bytes überhaupt einsammelt.
#[tauri::command]
pub async fn nachrichten_anhang_grenze(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<u64, String> {
    let Ok((client, _)) = laufender_client(&zustand, None).await else {
        return Ok(ANHANG_HOECHSTENS);
    };
    Ok(matrix::hochladegrenze(&client)
        .await
        .map_or(ANHANG_HOECHSTENS, |g| g.min(ANHANG_HOECHSTENS)))
}

/// Eine Datei ueber Matrix senden, optional mit Text (docs/plan-email-pgp.md,
/// Schritt 1). Nur Matrix: Der interne Weg kennt noch keine Anhaenge.
///
/// **In EINEM Stueck, als Base64.** Die Grenze liegt bei 10 MB; das passt
/// durch die Bruecke der Webansicht (Base64 wie beim Hochladen von Dateien,
/// siehe `quellen/dateien.js`). Groesseres gehoert nicht in eine Nachricht.
///
/// **Die Bytes bleiben auf diesem Geraet**, in der Inhaltsablage: So zeigt
/// die eigene Zeile den Anhang sofort, auch ohne Netz, und das Echo traegt
/// nur noch nach, wo er beim Homeserver liegt.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn nachricht_anhang_senden(
    app: tauri::AppHandle,
    zustand: tauri::State<'_, Arc<Zustand>>,
    ziel: String,
    name: String,
    mime: String,
    daten: String,
    text: Option<String>,
    von: Option<String>,
) -> Result<NachrichtAnzeige, String> {
    use base64::Engine;
    let ziel = ziel.trim().to_string();
    if !(ziel.starts_with('@') && ziel.contains(':')) {
        return Err("A Matrix ID looks like this: @name:server.org".into());
    }
    let name = match name.trim() {
        "" => "Anhang".to_string(),
        n => n.to_string(),
    };
    let mime = match mime.trim() {
        "" => "application/octet-stream".to_string(),
        m => m.to_string(),
    };
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(daten.as_bytes())
        .map_err(|_| "The attachment arrived unreadable.".to_string())?;

    let (client, mxid) = laufender_client(&zustand, von.as_deref()).await?;
    let grenze = matrix::hochladegrenze(&client)
        .await
        .map_or(ANHANG_HOECHSTENS, |g| g.min(ANHANG_HOECHSTENS));
    if bytes.len() as u64 > grenze {
        return Err(format!(
            "The attachment is too large: at most {} MB.",
            grenze / (1024 * 1024)
        ));
    }

    // Erst ablegen, dann senden: Scheitert das Senden, liegt nur ein
    // Inhalt herum, den niemand nennt -- das naechste Aufraeumen nimmt ihn.
    let mut ladung = zustand.inhalte.ladung().map_err(fehler)?;
    ladung.schreiben(&bytes).map_err(fehler)?;
    let (abdruck, groesse) = zustand.inhalte.ablegen(ladung).map_err(fehler)?;

    let text = text.map(|t| t.trim().to_string()).unwrap_or_default();
    let event_id = matrix::anhang_senden(
        &client,
        &ziel,
        &name,
        &mime,
        bytes,
        (!text.is_empty()).then_some(text.as_str()),
    )
    .await
    .map_err(|e| format!("{e:#}"))?;

    let jetzt = chrono::Utc::now().to_rfc3339();
    let n = Nachricht {
        konto: mxid.clone(),
        event_id: event_id.to_string(),
        raum: String::new(),
        gegenueber: Some(ziel),
        absender: mxid,
        von_mir: true,
        text,
        zeit: jetzt.clone(),
        gelesen_at: Some(jetzt),
        anhang: serde_json::to_string(&AnhangGespeichert {
            name,
            mime,
            groesse,
            abdruck: Some(abdruck),
            quelle: None,
        })
        .ok(),
    };
    zustand
        .speicher
        .lock()
        .await
        .nachricht_ablegen(&n)
        .map_err(fehler)?;
    melden(&app);
    Ok(anzeige(n))
}

/// Die Bytes eines Anhangs -- aus der eigenen Ablage, wenn er von hier kam,
/// sonst vom Homeserver (entschluesselt; das SDK haelt ihn danach in seinem
/// verschluesselten Zwischenspeicher).
///
/// **ALS BASE64, NICHT ALS `ipc::Response`.** Auf Android kommt eine
/// Response in der Webansicht als JSON-Zahlenliste an (29.09.2026 am
/// Tablet): Aus einem 3,8-MB-Bild wurden rund 13 MB Text, und wer die Liste
/// ungewandelt in einen Blob steckte, bekam „37,80,68,…" statt eines Bildes.
#[tauri::command]
pub async fn nachricht_anhang(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: String,
    weg: Option<String>,
    index: Option<usize>,
) -> Result<String, String> {
    use base64::Engine;
    if weg.as_deref() == Some("email") {
        return crate::mailbefehle::anhang(zustand.inner(), &id, index.unwrap_or(0)).await;
    }
    let bytes = anhang_bytes(&zustand, &id).await?;
    Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
}

async fn anhang_bytes(zustand: &Arc<Zustand>, id: &str) -> Result<Vec<u8>, String> {
    let (konto, ereignis) = teilen(zustand, id);
    let n = zustand
        .speicher
        .lock()
        .await
        .nachricht(&konto, &ereignis)
        .map_err(fehler)?
        .ok_or("This message does not exist here.")?;
    let a = anhang_lesen(n.anhang.as_deref()).ok_or("This message has no attachment.")?;

    if let Some(abdruck) = a.abdruck.as_deref().filter(|d| zustand.inhalte.hat(d)) {
        return zustand
            .inhalte
            .lesen(abdruck, 0, (ANHANG_HOECHSTENS * 2) as usize)
            .map_err(fehler);
    }

    let quelle = a
        .quelle
        .ok_or("The attachment is still on its way -- try again in a moment.")?;
    mit_client(zustand, &konto, |client, _, _| async move {
        Some(
            matrix::anhang_holen(&client, &quelle)
                .await
                .map_err(|e| format!("{e:#}")),
        )
    })
    .await
    .ok_or("The Matrix account of this message is not connected.")?
}

/// Eine interne Id ist eine Zahl -- die von Matrix faengt mit `$` an.
///
/// Der Weg kommt trotzdem aus der Ansicht mit und wird nicht hier geraten:
/// Die Zeile weiss ihn, und an der Form einer Id zu haengen hiesse, sich auf
/// etwas zu verlassen, das drueben jemand aendern darf.
fn interne_id(id: &str) -> Result<i64, String> {
    id.parse()
        .map_err(|_| format!("Not an internal message: {id}"))
}

#[tauri::command]
pub async fn nachricht_gelesen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: String,
    weg: Option<String>,
) -> Result<(), String> {
    if weg.as_deref() == Some("email") {
        return crate::mailbefehle::gelesen(zustand.inner(), &id).await;
    }
    if weg.as_deref() == Some("openany") {
        let client = crate::server_client(&zustand)
            .await
            .ok_or("Not connected to openany.de.")?;

        return client
            .nachricht_gelesen(interne_id(&id)?)
            .await
            .map_err(|e| format!("{e}"));
    }

    let (konto, ereignis) = teilen(&zustand, &id);
    zustand
        .speicher
        .lock()
        .await
        .nachricht_gelesen(&konto, &ereignis)
        .map_err(fehler)?;
    Ok(())
}

/// Bei Matrix nur auf diesem Geraet -- im Raum bleibt sie stehen. Intern
/// raeumt der Server sie fuer die eigene Seite weg, wie in der Webapp.
#[tauri::command]
pub async fn nachricht_loeschen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: String,
    weg: Option<String>,
    auch_server: Option<bool>,
) -> Result<(), String> {
    // E-Mail: ausblenden, und auf Wunsch auch auf dem Mailserver löschen
    // (Tiffy, 29.09.2026).
    if weg.as_deref() == Some("email") {
        return crate::mailbefehle::loeschen(zustand.inner(), &id, auch_server.unwrap_or(false))
            .await;
    }
    if weg.as_deref() == Some("openany") {
        // Eine wartende: zuruecknehmen, bevor sie hinausgeht.
        if let Some(w) = id.strip_prefix(crate::postausgang::VORSILBE) {
            crate::postausgang::entfernen(&zustand, w);
            return Ok(());
        }
        let client = crate::server_client(&zustand)
            .await
            .ok_or("Not connected to openany.de.")?;

        return client
            .nachricht_loeschen(interne_id(&id)?)
            .await
            .map_err(|e| format!("{e}"));
    }

    let (konto, ereignis) = teilen(&zustand, &id);
    zustand
        .speicher
        .lock()
        .await
        .nachricht_loeschen(&konto, &ereignis)
        .map_err(fehler)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eingang(absender: &str, gegenueber: Option<&str>) -> matrix::Eingang {
        matrix::Eingang {
            event_id: "$1".into(),
            absender: absender.into(),
            text: "Hallo".into(),
            raum: "!r:m.org".into(),
            gegenueber: gegenueber.map(Into::into),
            zeit_ms: 1_789_000_000_000,
            anhang: None,
        }
    }

    #[test]
    fn eine_fremde_nachricht_hat_den_absender_als_gegenueber() {
        let n = aus_eingang(
            eingang("@ferdinand:m.org", Some("@ferdinand:m.org")),
            "@tiffy:m.org",
        );
        assert!(!n.von_mir);
        assert_eq!(n.gegenueber.as_deref(), Some("@ferdinand:m.org"));
        assert!(n.gelesen_at.is_none());
    }

    /// Von einem ANDEREN eigenen Geraet geschrieben: Absender ist man selbst,
    /// das Gegenueber kommt aus dem Raum.
    #[test]
    fn das_eigene_echo_nennt_den_empfaenger() {
        let n = aus_eingang(
            eingang("@Tiffy:m.org", Some("@ferdinand:m.org")),
            "@tiffy:m.org",
        );
        assert!(n.von_mir);
        assert_eq!(n.gegenueber.as_deref(), Some("@ferdinand:m.org"));
        assert!(n.gelesen_at.is_some());
    }

    #[test]
    fn die_zeit_kommt_vom_homeserver() {
        let n = aus_eingang(eingang("@f:m.org", None), "@tiffy:m.org");
        assert_eq!(n.zeit, "2026-09-10T00:26:40+00:00");
    }

    /// Genau der Fall vom Tablet (24.09.2026): Das Geraet hatte kurz kein
    /// Netz, und das SDK haengte die ganze Sync-Adresse an die Meldung.
    #[test]
    fn ein_netzausfall_wird_zu_einem_satz() {
        let voll = "Abgleich mit dem Homeserver abgebrochen: error sending request \
            for url (https://matrix.org/_matrix/client/v3/sync?since=m7403448551%7E1): \
            dns error: failed to lookup address information";

        assert_eq!(
            kurzgefasst(voll),
            "No connection to the homeserver -- openany keeps trying."
        );
    }

    /// Was KEIN Netzproblem ist, bleibt wortwoertlich stehen -- sonst
    /// verschwiege die Oberflaeche gerade das, was jemand beheben koennte.
    #[test]
    fn ein_abgelehntes_token_bleibt_im_wortlaut() {
        let voll = "Sitzung konnte nicht wieder aufgenommen werden: M_UNKNOWN_TOKEN";

        assert_eq!(kurzgefasst(voll), voll);
    }
}
