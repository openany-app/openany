//! Der Wachdienst: haelt die Leitung zu ntfy und weckt bei einem Signal.
//!
//! Plan: `docs/plan-app-neuaufsatz.md`, Phase 6. Auf Android laeuft das in
//! einem Vordergrunddienst (`Wachdienst.kt`, Typ `remoteMessaging`), der
//! ueber JNI hier hereinruft und **in diesem Aufruf bleibt**, bis er beendet
//! wird. Die Leitung selbst ist [`openany_meldungen::halten`] -- hier steht
//! nur, was ein Signal ausloest:
//!
//! * **Nachricht** -> das Postfach fragen und die neueste ungelesene Zeile
//!   als Benachrichtigung zeigen (Name und Anfang). Der Inhalt kommt ueber
//!   die eigene, angemeldete Leitung -- ntfy hat ihn nie gesehen.
//! * **Kalender** -> abgleichen, keine Benachrichtigung. Das Geraet soll den
//!   neuen Stand haben, nicht davon erzaehlen.
//! * **Matrix** -> vorerst nur „Neue Matrix-Nachricht". Den Raum selbst im
//!   Hintergrund zu synchronisieren ist Schritt 5.
//! * **Unbekannt** -> abgleichen, still. Lieber einmal zu viel als nie.
//!
//! **Daneben die E-Mail-Postfächer** (docs/plan-email-pgp.md, Schritt 4):
//! je Postfach eine IMAP-IDLE-Verbindung zum Posteingang. Meldet der Server
//! etwas, wird abgeholt und die neueste neue Mail angezeigt. Das braucht
//! KEIN openany -- ohne Instanz wacht der Dienst nur über die Postfächer.
//!
//! **Mehrere Signale kurz hintereinander ergeben einen Durchgang**: Nach
//! jedem Signal wird geleert, was schon wartet.

// Der Einstieg ist JNI; auf dem Schreibtisch ruft (noch) niemand herein.
#![cfg_attr(not(target_os = "android"), allow(dead_code))]

use crate::Zustand;
use openany_client::OpenanyClient;
use openany_meldungen::{halten, Leitung, Signal, Vorgang};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot, watch};

/// Was der Wachdienst dem System zeigen will. Plattformfrei; auf Android
/// geht es ueber JNI an `Wachdienst.zeige`.
pub(crate) trait Anzeige {
    fn zeige(&self, titel: &str, text: &str);
    /// Ein offenes Fenster soll den neuen Stand zeigen.
    fn aufgefrischt(&self);
    /// Eine Zeile ins Protokoll des Systems (logcat). `eprintln!` landet
    /// auf Android im Nichts.
    fn protokoll(&self, text: &str);
}

/// Der Ausschalter und die Netzmeldung des laufenden Wachdiensts, damit
/// Kotlin beides von aussen erreicht.
struct Steuerung {
    stopp: Option<oneshot::Sender<()>>,
    netz: watch::Sender<bool>,
}

static STEUERUNG: Mutex<Option<Steuerung>> = Mutex::new(None);

/// Wohin die laufende Leitung geht -- damit ein spaeter verbundenes
/// Matrix-Konto sein Push-Ziel nachtragen kann ([`matrix_nachtragen`]).
static AKTUELL: Mutex<Option<(String, String)>> = Mutex::new(None);

fn matrix_ziel(basis: &str, thema: &str) -> (String, String) {
    (
        // `?up=1` ist UnifiedPushs Kennzeichen; ntfys Gateway erkennt daran
        // ein Thema, an das es Matrix-Meldungen weiterreichen darf.
        format!("{basis}/{thema}?up=1"),
        format!("{basis}/_matrix/push/v1/notify"),
    )
}

/// Das Push-Ziel beim Homeserver eintragen, falls ein Konto verbunden ist.
async fn matrix_eintragen(zustand: &Arc<Zustand>, basis: &str, thema: &str) -> Option<String> {
    let (pushkey, gateway) = matrix_ziel(basis, thema);
    let name = zustand.einstellungen.lock().await.name().to_string();
    match crate::nachrichtenbefehle::weckruf(zustand, &pushkey, &gateway, &name, true).await {
        None => None,
        Some(Ok(())) => Some("Matrix: push target registered".into()),
        Some(Err(e)) => Some(format!("Matrix: push target not registered ({e})")),
    }
}

/// Nach dem Verbinden eines Matrix-Kontos: Laeuft der Wachdienst schon,
/// soll auch dieses Konto ihn wecken -- sonst erst nach dem naechsten
/// Neustart des Diensts, und niemand wuesste, warum Matrix still bleibt.
pub(crate) async fn matrix_nachtragen(zustand: &Arc<Zustand>) {
    let ziel = AKTUELL.lock().unwrap_or_else(|p| p.into_inner()).clone();
    if let Some((basis, thema)) = ziel {
        let _ = matrix_eintragen(zustand, &basis, &thema).await;
    }
}

fn client(
    zustand: &Arc<Zustand>,
    e: &crate::einstellungen::Einstellungen,
) -> Result<OpenanyClient, String> {
    let schluessel = zustand
        .schluesselablage()
        .lesen()
        .map_err(|err| err.to_string())?
        .ok_or_else(|| "not paired".to_string())?;
    OpenanyClient::neu_mit_ca(&e.openany_basis, &schluessel, e.ca().as_deref())
        .map_err(|err| err.to_string())
}

/// Laeuft, bis [`anhalten`] gerufen wird. Gibt den Grund zurueck, warum er
/// endete -- ein Wachdienst, der sofort wieder aufhoert, soll sagen, warum.
pub(crate) fn laufen(ordner: PathBuf, zwischenspeicher: PathBuf, anzeige: &dyn Anzeige) -> String {
    let zustand = match Zustand::holen(ordner, zwischenspeicher) {
        Ok(z) => z,
        Err(e) => return format!("Storage: {e}"),
    };

    let (stopp_tx, stopp_rx) = oneshot::channel();
    let (netz_tx, netz_rx) = watch::channel(true);
    *STEUERUNG.lock().unwrap_or_else(|p| p.into_inner()) = Some(Steuerung {
        stopp: Some(stopp_tx),
        netz: netz_tx,
    });

    // `block_on` auf DIESEM Faden: Er gehoert Kotlin, und nur hier findet
    // JNI die Klassen der App. Beide Haelften -- Leitung und Verarbeitung --
    // laufen deshalb als `join` im selben Aufruf, nicht als `spawn`.
    let grund = tauri::async_runtime::block_on(async {
        tokio::select! {
            g = wachen(&zustand, netz_rx, anzeige) => g,
            _ = stopp_rx => "angehalten".to_string(),
        }
    });

    *STEUERUNG.lock().unwrap_or_else(|p| p.into_inner()) = None;
    *AKTUELL.lock().unwrap_or_else(|p| p.into_inner()) = None;
    grund
}

async fn wachen(
    zustand: &Arc<Zustand>,
    netz_rx: watch::Receiver<bool>,
    anzeige: &dyn Anzeige,
) -> String {
    // Nichts zu bewachen: keine Instanz und kein Postfach. Dann endet der
    // Dienst, statt mit Dauerhinweis leer herumzustehen.
    let instanz = zustand.einstellungen.lock().await.verbunden();
    if !instanz && crate::mailbefehle::adressen(zustand).is_empty() {
        return "no instance and no mailbox".into();
    }
    tokio::join!(
        openany_wachen(zustand, netz_rx.clone(), anzeige),
        post_wachen(zustand, netz_rx, anzeige),
    );
    "connection ended".to_string()
}

/// Die Leitung zu ntfy (openany und Matrix). Ohne Instanz oder Kopplung
/// ruht dieser Teil -- die Postfächer wachen trotzdem.
async fn openany_wachen(
    zustand: &Arc<Zustand>,
    netz_rx: watch::Receiver<bool>,
    anzeige: &dyn Anzeige,
) {
    let (basis, thema) = match anmelden(zustand, netz_rx.clone(), anzeige).await {
        Ok(bt) => bt,
        Err(grund) => {
            anzeige.protokoll(&format!("Watch service: without openany ({grund})"));
            return std::future::pending().await;
        }
    };
    *AKTUELL.lock().unwrap_or_else(|p| p.into_inner()) = Some((basis.clone(), thema.clone()));
    if let Some(bericht) = matrix_eintragen(zustand, &basis, &thema).await {
        anzeige.protokoll(&bericht);
    }

    let (tx, rx) = mpsc::unbounded_channel();
    let leitung = halten(Leitung::new(basis, thema), netz_rx, |v| match v {
        Vorgang::Verbunden => anzeige.protokoll("Watch service: connected"),
        Vorgang::Abriss { grund, pause } => {
            anzeige.protokoll(&format!("Watch service: {grund} -- retrying in {pause:?}"))
        }
        Vorgang::Ereignis(e) => {
            let _ = tx.send(e.signal);
        }
    });
    tokio::join!(leitung, verarbeiten(zustand, rx, anzeige));
}

/* ── E-Mail: IMAP IDLE je Postfach ─────────────────────────────────────── */

/// Wie lange eine IDLE-Verbindung höchstens still steht, bevor sie neu
/// aufgebaut wird. Unter den 30 Minuten, nach denen viele Server und Router
/// eine stille Verbindung stillschweigend fallen lassen (RFC 2177: „alle
/// 29 Minuten neu").
const IDLE_HOECHSTENS: std::time::Duration = std::time::Duration::from_secs(20 * 60);

/// Über alle Postfächer wachen. Die Liste wird regelmäßig neu gelesen, damit
/// ein neu verbundenes oder getrenntes Postfach ohne Neustart zählt.
async fn post_wachen(zustand: &Arc<Zustand>, netz: watch::Receiver<bool>, anzeige: &dyn Anzeige) {
    loop {
        let adressen = crate::mailbefehle::adressen(zustand);
        let runde = futures::future::join_all(
            adressen
                .iter()
                .map(|a| postfach_wachen(zustand, a, netz.clone(), anzeige)),
        );
        // Nach einer Stunde (oder sofort, wenn es keine gibt, nach zehn
        // Minuten) die Liste neu lesen.
        let neu_lesen = if adressen.is_empty() {
            10 * 60
        } else {
            60 * 60
        };
        tokio::select! {
            _ = runde => {}
            _ = tokio::time::sleep(std::time::Duration::from_secs(neu_lesen)) => {}
        }
    }
}

async fn postfach_wachen(
    zustand: &Arc<Zustand>,
    adresse: &str,
    mut netz: watch::Receiver<bool>,
    anzeige: &dyn Anzeige,
) {
    let mut pause = openany_meldungen::Wartezeit::new(
        std::time::Duration::from_secs(15),
        std::time::Duration::from_secs(10 * 60),
    );
    loop {
        // Erst abholen, was schon da ist -- auch das, was während eines
        // Abrisses kam.
        match crate::mailbefehle::abholen_still(zustand, adresse).await {
            Ok((neue, anders)) => {
                pause.zuruecksetzen();
                if !neue.is_empty() {
                    anzeige.protokoll(&format!("Watch service: {adresse}: {} new", neue.len()));
                }
                mails_zeigen(&neue, anzeige);
                if !neue.is_empty() || anders {
                    anzeige.aufgefrischt();
                }
            }
            Err(e) => {
                let p = pause.naechste();
                anzeige.protokoll(&format!(
                    "Watch service: {adresse}: {e} -- retrying in {p:?}"
                ));
                tokio::select! {
                    _ = tokio::time::sleep(p) => {}
                    _ = netz.changed() => {}
                }
                continue;
            }
        }
        let Some((konto, eingang)) = crate::mailbefehle::konto_und_eingang(zustand, adresse) else {
            return; // getrennt
        };
        anzeige.protokoll(&format!("Watch service: {adresse} waiting (IDLE)"));
        tokio::select! {
            r = openany_post::warten(&konto, &eingang, IDLE_HOECHSTENS) => {
                if let Ok(neu) = r {
                    anzeige.protokoll(&format!("Watch service: {adresse} IDLE returned, notice: {neu}"));
                }
                if let Err(e) = r {
                    let p = pause.naechste();
                    anzeige.protokoll(&format!("Watch service: {adresse} IDLE: {e} -- retrying in {p:?}"));
                    tokio::select! {
                        _ = tokio::time::sleep(p) => {}
                        _ = netz.changed() => {}
                    }
                }
            }
            // Netz gewechselt: Die alte Verbindung ist vermutlich tot.
            _ = netz.changed() => {}
        }
    }
}

/// Neue fremde, ungelesene Mails anzeigen -- die neueste, und wie viele
/// sonst noch kamen. VERSCHLÜSSELTE ZEIGEN IHREN BETREFF NICHT: Die
/// Benachrichtigung steht auf dem Sperrbildschirm, und wer verschlüsselt
/// schreibt, will das nicht dort lesen lassen.
fn mails_zeigen(neue: &[openany_store::Mail], anzeige: &dyn Anzeige) {
    let mut fremde: Vec<&openany_store::Mail> = neue
        .iter()
        .filter(|m| !m.von_mir && m.gelesen_at.is_none())
        .collect();
    fremde.sort_by(|a, b| b.zeit.cmp(&a.zeit));
    let Some(m) = fremde.first() else {
        return;
    };
    let von = if m.gegenueber_name.trim().is_empty() {
        m.gegenueber.clone()
    } else {
        m.gegenueber_name.clone()
    };
    let mut text = if m.pgp.is_some() {
        "🔒 Encrypted email".to_string()
    } else if m.betreff.trim().is_empty() {
        m.text.chars().take(120).collect()
    } else {
        m.betreff.chars().take(120).collect()
    };
    if fremde.len() > 1 {
        text.push_str(&format!(" (+{} more)", fremde.len() - 1));
    }
    anzeige.zeige(&von, &text);
}

/// Beim Server anmelden -- GEDULDIG. Nach dem Einschalten des Geraets ist
/// oft noch kein Netz da; gaebe der Dienst dann auf, bliebe er bis zum
/// naechsten Oeffnen der App tot, und niemand merkte es. Aufgegeben wird
/// nur, wenn es nichts zu wecken gibt: keine Instanz, keine Kopplung.
async fn anmelden(
    zustand: &Arc<Zustand>,
    mut netz: watch::Receiver<bool>,
    anzeige: &dyn Anzeige,
) -> Result<(String, String), String> {
    let mut warten = openany_meldungen::Wartezeit::default();
    loop {
        let e = zustand.einstellungen.lock().await.clone();
        if !e.verbunden() {
            return Err("no instance".into());
        }
        let c = client(zustand, &e)?;
        match c.weckruf_anmelden().await {
            Ok(bt) => return Ok(bt),
            Err(err) => {
                let pause = warten.naechste();
                anzeige.protokoll(&format!(
                    "Watch service: sign-in failed ({err}) -- retrying in {pause:?}"
                ));
                tokio::select! {
                    _ = tokio::time::sleep(pause) => {}
                    _ = netz.changed() => {}
                }
            }
        }
    }
}

async fn verarbeiten(
    zustand: &Arc<Zustand>,
    mut rx: mpsc::UnboundedReceiver<Signal>,
    anzeige: &dyn Anzeige,
) {
    while let Some(erstes) = rx.recv().await {
        // Was schon wartet, reitet mit: ein Durchgang fuer alle.
        let mut signale = vec![erstes];
        while let Ok(s) = rx.try_recv() {
            signale.push(s);
        }

        let nachricht = signale
            .iter()
            .any(|s| matches!(s, Signal::Nachricht { .. }));
        // Von mehreren Matrix-Signalen zaehlt das letzte: Es ist die neueste
        // Nachricht, und nur eine wird angezeigt.
        let matrix = signale.iter().rev().find_map(|s| match s {
            Signal::Matrix { ereignis, .. } => Some(ereignis.clone()),
            _ => None,
        });
        let abgleichen = signale
            .iter()
            .any(|s| matches!(s, Signal::Kalender { .. } | Signal::Unbekannt { .. }));

        if nachricht {
            match neueste_ungelesene(zustand).await {
                Some((von, text)) => anzeige.zeige(&von, &text),
                // Nichts Ungelesenes (schon gelesen, oder eine Projekt-
                // nachricht, die das Postfach nicht fuehrt): trotzdem sagen,
                // dass etwas kam -- sonst waere das Signal umsonst gewesen.
                // Leerer Text: Kotlin setzt „Neue Nachricht" in der Sprache
                // des Systems ein (res/values*/wachdienst.xml).
                None => anzeige.zeige("openany", ""),
            }
        }
        if let Some(ereignis) = matrix {
            // Kein Fallback-Hinweis: Kommt nichts Fremdes heraus, war es das
            // eigene Echo von einem anderen Geraet, eine Reaktion oder ein
            // Beitritt -- nichts, wofuer jemand geweckt werden will.
            if let Some((von, text)) =
                crate::nachrichtenbefehle::im_hintergrund_holen(zustand, ereignis.as_deref()).await
            {
                anzeige.zeige(&von, &text);
            }
        }
        if abgleichen {
            // Im Hintergrund nur die Liste; Dateien holt der Auffrischer im WLAN.
            let bericht = crate::hintergrund::auffrischen(zustand, true).await;
            anzeige.protokoll(&format!("Watch service: synced {bericht}"));
        }
        anzeige.aufgefrischt();
    }
}

/// Die neueste ungelesene Nachricht an mich: `(Name, Anfang)`.
async fn neueste_ungelesene(zustand: &Arc<Zustand>) -> Option<(String, String)> {
    let e = zustand.einstellungen.lock().await.clone();
    let c = client(zustand, &e).ok()?;
    let seite = c.nachrichten(1).await.ok()?;
    let zeilen = seite.get("items").and_then(|v| v.as_array())?;
    // Matrix-Zeilen nicht: Die meldet das Matrix-Signal selbst, und dieses
    // Geraet ist dort ein eigenes Matrix-Geraet (nachrichtenbefehle.rs).
    let z = zeilen.iter().find(|z| {
        let von_mir = z.get("von_mir").and_then(|v| v.as_bool()).unwrap_or(false);
        let gelesen = z.get("read_at").is_some_and(|r| !r.is_null());
        let matrix = z.get("transport").and_then(|v| v.as_str()) == Some("matrix");
        !von_mir && !gelesen && !matrix
    })?;
    let von = z
        .pointer("/sender/name")
        .and_then(|v| v.as_str())
        .or_else(|| z.get("peer").and_then(|v| v.as_str()))
        .unwrap_or("openany")
        .to_string();
    let text: String = z
        .get("body")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .chars()
        .take(120)
        .collect();
    Some((von, text))
}

/// Beendet einen laufenden Wachdienst; [`laufen`] kehrt dann zurueck.
pub(crate) fn anhalten() {
    if let Some(s) = STEUERUNG.lock().unwrap_or_else(|p| p.into_inner()).as_mut() {
        if let Some(stopp) = s.stopp.take() {
            let _ = stopp.send(());
        }
    }
}

/// Das Netz hat sich geaendert (oder ist weg). Jede Meldung mit `true`
/// beendet die laufende Leitung und waehlt sofort neu.
pub(crate) fn netz(da: bool) {
    if let Some(s) = STEUERUNG.lock().unwrap_or_else(|p| p.into_inner()).as_ref() {
        let _ = s.netz.send(da);
    }
}

/// Beim Ausschalten: Der Server soll das Thema vergessen.
pub(crate) fn abmelden(ordner: PathBuf, zwischenspeicher: PathBuf) -> String {
    let zustand = match Zustand::holen(ordner, zwischenspeicher) {
        Ok(z) => z,
        Err(e) => return e,
    };
    tauri::async_runtime::block_on(async {
        let e = zustand.einstellungen.lock().await.clone();
        let c = match client(&zustand, &e) {
            Ok(c) => c,
            Err(e) => return e,
        };
        // ERST beim Homeserver austragen, DANN beim Server: Das Thema
        // erfaehrt man nur von dort (Anmelden ist wiederholbar und liefert
        // dasselbe). Umgekehrt wuesste niemand mehr, welches Push-Ziel beim
        // Homeserver steht -- und er schickte weiter an ein Thema, das
        // keiner mehr liest.
        let mut bericht = String::new();
        if let Ok((basis, thema)) = c.weckruf_anmelden().await {
            let (pushkey, gateway) = matrix_ziel(&basis, &thema);
            if let Some(Err(err)) =
                crate::nachrichtenbefehle::weckruf(&zustand, &pushkey, &gateway, "", false).await
            {
                bericht = format!("Matrix: {err}; ");
            }
        }
        match c.weckruf_abmelden().await {
            Ok(()) => format!("{bericht}abgemeldet"),
            Err(e) => format!("{bericht}Sign-out: {e}"),
        }
    })
}

/// JNI: `Wachdienst.laufen/anhalten/netz/abmelden` in Kotlin, und der
/// Rueckweg `Wachdienst.zeige`. Wie beim Auffrischer darf kein Absturz
/// ueber die Grenze -- er naehme den ganzen Prozess mit.
#[cfg(target_os = "android")]
mod android {
    use jni::objects::{JClass, JString, JValue};
    use jni::sys::{jboolean, jstring, JNI_TRUE};
    use jni::JNIEnv;
    use std::cell::RefCell;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    fn text(env: &mut JNIEnv, s: &JString) -> String {
        env.get_string(s).map(Into::into).unwrap_or_default()
    }

    /// Der Rueckweg nach Kotlin -- auf dem Faden, der `laufen` rief, also
    /// mit einem gueltigen `JNIEnv` und den Klassen der App.
    ///
    /// JEDER AUFRUF IN EINEM EIGENEN RAHMEN (`with_local_frame`). `laufen`
    /// kehrt tagelang nicht zurueck, und lokale Referenzen werden erst beim
    /// Zurueckkehren frei. Ohne Rahmen saemmelte jede Benachrichtigung zwei
    /// Strings an, bis die Tabelle der JVM ueberliefe -- ein Absturz nach
    /// ein paar hundert Nachrichten, lange nach jedem Test.
    struct Jni<'a, 'b> {
        env: RefCell<&'a mut JNIEnv<'b>>,
    }

    impl Jni<'_, '_> {
        fn rufen(&self, klasse: &str, methode: &str, sig: &str, texte: &[&str]) {
            let mut env = self.env.borrow_mut();
            let r = env.with_local_frame(8, |env| -> jni::errors::Result<()> {
                let mut objekte = Vec::with_capacity(texte.len());
                for t in texte {
                    objekte.push(env.new_string(t)?);
                }
                let werte: Vec<JValue> = objekte.iter().map(|o| JValue::Object(o)).collect();
                env.call_static_method(klasse, methode, sig, &werte)?;
                Ok(())
            });
            if r.is_err() {
                // Eine offene Java-Ausnahme bliebe haengen und braeche den
                // naechsten JNI-Aufruf.
                let _ = env.exception_clear();
            }
        }
    }

    impl super::Anzeige for Jni<'_, '_> {
        fn zeige(&self, titel: &str, text: &str) {
            self.rufen(
                "de/openany/app/Wachdienst",
                "zeige",
                "(Ljava/lang/String;Ljava/lang/String;)V",
                &[titel, text],
            );
        }

        fn aufgefrischt(&self) {
            self.rufen("de/openany/app/MainActivity", "aufgefrischt", "()V", &[]);
        }

        fn protokoll(&self, text: &str) {
            self.rufen(
                "de/openany/app/Wachdienst",
                "protokoll",
                "(Ljava/lang/String;)V",
                &[text],
            );
        }
    }

    #[no_mangle]
    pub extern "system" fn Java_de_openany_app_Wachdienst_laufen(
        mut env: JNIEnv,
        _klasse: JClass,
        daten: JString,
        zwischen: JString,
    ) -> jstring {
        let daten = text(&mut env, &daten);
        let zwischen = text(&mut env, &zwischen);
        let ergebnis = {
            let anzeige = Jni {
                env: RefCell::new(&mut env),
            };
            catch_unwind(AssertUnwindSafe(|| {
                super::laufen(daten.into(), zwischen.into(), &anzeige)
            }))
            .unwrap_or_else(|_| "Crash in the watch service".to_string())
        };
        env.new_string(ergebnis)
            .map(|s| s.into_raw())
            .unwrap_or(std::ptr::null_mut())
    }

    #[no_mangle]
    pub extern "system" fn Java_de_openany_app_Wachdienst_anhalten(_env: JNIEnv, _klasse: JClass) {
        let _ = catch_unwind(super::anhalten);
    }

    #[no_mangle]
    pub extern "system" fn Java_de_openany_app_Wachdienst_netz(
        _env: JNIEnv,
        _klasse: JClass,
        da: jboolean,
    ) {
        let _ = catch_unwind(|| super::netz(da == JNI_TRUE));
    }

    #[no_mangle]
    pub extern "system" fn Java_de_openany_app_Wachdienst_abmelden(
        mut env: JNIEnv,
        _klasse: JClass,
        daten: JString,
        zwischen: JString,
    ) -> jstring {
        let daten = text(&mut env, &daten);
        let zwischen = text(&mut env, &zwischen);
        let ergebnis = catch_unwind(AssertUnwindSafe(|| {
            super::abmelden(daten.into(), zwischen.into())
        }))
        .unwrap_or_else(|_| "Crash while signing out".to_string());
        env.new_string(ergebnis)
            .map(|s| s.into_raw())
            .unwrap_or(std::ptr::null_mut())
    }
}
