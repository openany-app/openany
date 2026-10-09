//! Die Schale.
//!
//! **Hier steht nur die Verdrahtung.** Abgleich, Speicher, Konfliktentscheidung
//! und Kopplung liegen in `crates/` -- ohne Webview und ohne Emulator pruefbar,
//! mit 99 Tests. Was in dieser Datei steht, ist der Weg von einem Klick zu
//! einem Aufruf und zurueck.
//!
//! ## Die Kernregel dieses Programms
//!
//! **Es funktioniert ohne Anmeldung.** Notizen, Kalender und Kontakte liegen
//! in der lokalen SQLite; wer nie eine Instanz eintraegt, hat trotzdem ein
//! vollstaendiges Programm. Der Abgleich ist ein Zusatz, kein Fundament.
//!
//! Das ist der Unterschied zu `anytail-app`, dessen Schale ohne Kopplung
//! nichts zeigt -- dort sagt der Server, welche Apps es gibt. Die beiden
//! Saetze passen nicht in ein Programm; deshalb sind es zwei Programme, und
//! deshalb faengt hier keine Ansicht mit einer Anmeldung an.
//!
//! ## Wo die Dateien liegen
//!
//! Alles im App-Datenordner, den Tauri je Plattform kennt:
//!
//! ```text
//!   openany.sqlite          Notizen, Kalender, Termine, Kontakte, Marken
//!   einstellungen.json      Adressen der Instanz
//!   ausweise/anyid-ausweis      unter Android verschlossen (Keystore)
//!   ausweise/openany-schluessel unter Android verschlossen (Keystore)
//!   ausweise/matrix-sitzung     unter Android verschlossen (Keystore)
//! ```
//!
//! Die Ausweise liegen hinter `anyid_client::Tokenspeicher`. Unter Android
//! verschliesst sie seit dem 22.09.2026 ein Schluessel aus dem Keystore
//! (`tresor.rs`, `Tresor.kt`); auf dem Schreibtisch sind sie noch Dateien --
//! der Schluesselbund dort ist eine eigene Stufe.

// Die Futures von matrix-sdk sind so tief verschachtelt, dass die Pruefung
// auf `Send` beim Spawnen die Vorgabe (128) sprengt. Dieselbe Zeile steht im
// Sidecar (`matrix/sidecar/src/main.rs`).
#![recursion_limit = "256"]

mod ablagebefehle;
mod aktualisierung;
mod anhangbefehle;
mod dateibefehle;
mod direktbefehle;
mod einstellungen;
mod galeriebefehle;
// Auf dem Schreibtisch ruft den Auffrischer niemand -- der Weg ist Android.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
mod hintergrund;
mod hintergrunddienste;
mod mailbefehle;
mod nachrichtenbefehle;
mod postausgang;
mod projektbefehle;
mod servergegenstelle;
mod sicherungsbefehle;
mod standbild;
mod tresor;
mod wachdienst;

use anyid_client::{AnyidClient, Tokenspeicher};
use einstellungen::Einstellungen;
use openany_anmeldung::{Anmeldung, Kopplung, Kopplungsstand, OpenanySchalter};
use openany_client::OpenanyClient;
use openany_store::{
    kalenderbuch, mappen_id, Filter, Kalender, Kontakt, Notiz, Protokoll, Speicher, Termin,
};
use openany_sync::Laeufer;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

pub struct Zustand {
    ordner: PathBuf,
    /// Die SQLite-Verbindung ist `Send`, aber nicht `Sync` -- deshalb ein
    /// Mutex, und zwar der von tokio: Der Abgleich haelt ihn ueber ein
    /// `await` hinweg, und ein `std::sync::Mutex` waere dabei genau die Falle,
    /// die den ganzen Laufzeitkern blockiert.
    ///
    /// Geteilt (`Arc`), weil auch der Dienst fuer gepaarte Geraete daran muss
    /// -- und zwar derselbe Mutex, sonst schriebe er am Laeufer vorbei.
    speicher: openany_nahbereich::GemeinsamerSpeicher,
    einstellungen: Mutex<Einstellungen>,
    /// Eine laufende Kopplung -- zwischen "Code anzeigen" und "abgeholt".
    kopplung: Mutex<Option<Kopplung>>,
    /// Geraete in der Naehe (APK 0.1). `None`, solange er startet oder wenn
    /// er nicht starten konnte -- dann steht der Grund in `nah_fehler`.
    nah: Mutex<Option<openany_nahbereich::Nahbereich>>,
    nah_fehler: Mutex<Option<String>>,
    /// Der eigene Dienst fuer Paaren und Abgleich (Port 53318) und was er
    /// ueber gepaarte Geraete weiss.
    nah_dienst: Mutex<Option<openany_nahbereich::Dienst>>,
    /// Die Identitaet dieses Geraets, einmal aus dem Tresor gelesen
    /// (`geraet_identitaet`).
    identitaet: std::sync::OnceLock<openany_nahbereich::Identitaet>,
    paarungen: openany_nahbereich::GemeinsamePaarungen,
    gastgeber: Arc<NahGastgeber>,
    /// Die Bytes der Dateien, nach Abdruck (`inhalte/` im App-Ordner).
    /// Geteilt, weil der Dienst sie an gepaarte Geraete ausliefert.
    inhalte: Arc<openany_store::Inhalte>,
    /// Hochladen, die gerade Stück für Stück ankommen.
    hochladungen: dateibefehle::Hochladungen,
    /// Der Zwischenspeicher des Systems -- dort entstehen Dateien zum Öffnen.
    zwischenspeicher: PathBuf,
    /// Nachrichten: die laufenden Abgleiche mit den Homeservern, je
    /// Matrix-Konto (Kennung), solange das Fenster offen ist.
    matrix: Mutex<std::collections::HashMap<String, nachrichtenbefehle::Lauf>>,
    /// Warum ein Matrix-Abgleich zuletzt abbrach, je Konto.
    matrix_fehler: Mutex<std::collections::HashMap<String, String>>,
    /// E-Mail: der laufende Abholer und sein letzter Fehler.
    post: Mutex<mailbefehle::PostLage>,
}

/// Der EINE Zustand dieses Prozesses.
///
/// Auf Android laeuft der Auffrischer (WorkManager) im selben Prozess wie
/// das Fenster -- wenn es eines gibt. Beide muessen an DENSELBEN Speicher,
/// hinter demselben Mutex: Zwei SQLite-Verbindungen in einem Prozess
/// schrieben aneinander vorbei, und der Laeufer hielte seinen Stand fuer den
/// einzigen. Wer zuerst kommt, oeffnet; der andere bekommt denselben.
static GETEILT: std::sync::OnceLock<Arc<Zustand>> = std::sync::OnceLock::new();

impl Zustand {
    /// Den Zustand oeffnen -- oder den schon offenen bekommen.
    ///
    /// Alles, was beim Start einmal aufzuraeumen ist, passiert hier: Der
    /// Papierkorb, halbe Ladungen, zum Oeffnen Verknuepftes. Es passiert
    /// also auch dann, wenn zuerst der Auffrischer den Prozess weckt.
    pub(crate) fn holen(
        ordner: PathBuf,
        zwischenspeicher: PathBuf,
    ) -> Result<Arc<Zustand>, String> {
        if let Some(z) = GETEILT.get() {
            return Ok(z.clone());
        }
        let z = Self::oeffnen(ordner, zwischenspeicher)?;
        // Kam ein anderer dazwischen, gilt seiner: `set` schlaegt fehl, und
        // wir geben den zurueck, der drin ist.
        let _ = GETEILT.set(z.clone());
        Ok(GETEILT.get().cloned().unwrap_or(z))
    }

    fn oeffnen(ordner: PathBuf, zwischenspeicher: PathBuf) -> Result<Arc<Zustand>, String> {
        std::fs::create_dir_all(&ordner).map_err(|e| e.to_string())?;

        // Eine bestaetigte Sicherung tritt an die Stelle des Bisherigen --
        // jetzt, bevor irgendetwas die Datenbank oeffnet. Scheitert es,
        // startet das Programm mit dem, was da ist.
        let _ = sicherungsbefehle::einspielen_falls_bereit(&ordner);

        // EIN Krypto-Anbieter fuer den ganzen Prozess. Im Baum sind ring und
        // aws-lc-rs beide aktiv, und `ClientConfig::builder()` ohne Anbieter
        // bricht dann ab -- matrix-sdk ruft genau das auf Android. Scheitert
        // es, hat ihn schon jemand gesetzt; das ist ebenso gut.
        let _ = rustls::crypto::ring::default_provider().install_default();

        // **Der Speicher wird beim Start geoeffnet, nicht beim ersten
        // Abgleich.** Ohne Netz und ohne Kopplung ist er das ganze
        // Programm; ein Fenster, das erst nach einer Antwort etwas zeigen
        // kann, ist in der U-Bahn kein Programm.
        let speicher =
            Speicher::oeffnen(ordner.join("openany.sqlite")).map_err(|e| e.to_string())?;
        // Die 30 Tage, die der Papierkorb ansagt. Scheitert das, startet
        // das Programm trotzdem: Aufraeumen ist Beiwerk, die Notizen nicht.
        let _ = speicher.papierkorb_aufraeumen();
        let einstellungen = Einstellungen::lesen(&ordner.join("einstellungen.json"));
        let gepaart_pfad = ordner.join("nah").join("gepaart.json");
        let einstellungen_name = einstellungen.name();

        let inhalte = Arc::new(
            openany_store::Inhalte::oeffnen(ordner.join("inhalte")).map_err(|e| e.to_string())?,
        );
        // Zum Öffnen Verknüpftes vom letzten Mal: fort.
        let _ = std::fs::remove_dir_all(zwischenspeicher.join("oeffnen"));
        // Was keine Zeile mehr nennt, und halbe Ladungen vom letzten Mal.
        if let Ok(benutzt) = speicher.benutzte_abdruecke() {
            let _ = inhalte.aufraeumen(&benutzt);
        }
        let speicher = Arc::new(Mutex::new(speicher));

        Ok(Arc::new(Zustand {
            ordner,
            speicher: speicher.clone(),
            einstellungen: Mutex::new(einstellungen),
            kopplung: Mutex::new(None),
            nah: Mutex::new(None),
            nah_fehler: Mutex::new(None),
            nah_dienst: Mutex::new(None),
            identitaet: Default::default(),
            paarungen: Default::default(),
            gastgeber: Arc::new(NahGastgeber::laden(
                &gepaart_pfad,
                einstellungen_name,
                speicher,
                inhalte.clone(),
            )),
            inhalte,
            hochladungen: Default::default(),
            zwischenspeicher,
            matrix: Mutex::new(Default::default()),
            matrix_fehler: Mutex::new(Default::default()),
            post: Mutex::new(Default::default()),
        }))
    }

    fn einstellungspfad(&self) -> PathBuf {
        self.ordner.join("einstellungen.json")
    }

    /// Unter Android im Tresor (Keystore), sonst als Datei -- siehe `tresor`.
    fn ausweis(&self) -> Box<dyn Tokenspeicher + Send + Sync> {
        tresor::ablage(self.ordner.join("ausweise").join("anyid-ausweis"))
    }

    fn inhalte_wurzel(&self) -> PathBuf {
        self.ordner.join("inhalte")
    }

    fn schluesselablage(&self) -> Box<dyn Tokenspeicher + Send + Sync> {
        tresor::ablage(self.ordner.join("ausweise").join("openany-schluessel"))
    }

    /// Eine Anmeldung aus dem aktuellen Stand der Einstellungen.
    ///
    /// **Bei jedem Aufruf neu und nicht einmal im Zustand.** Die Adressen
    /// koennen sich zwischen zwei Klicks aendern -- ein Client, der beim Start
    /// gebaut wurde, spraeche danach mit der alten Instanz weiter, ohne dass
    /// irgendetwas fehlschluege.
    async fn anmeldung(&self) -> Result<Anmeldung<AnyidClient, OpenanySchalter>, String> {
        let e = self.einstellungen.lock().await.clone();

        if e.anyid_basis.trim().is_empty() || !e.verbunden() {
            return Err("The addresses of anyid and openany are missing.".into());
        }

        let ca = e.ca();
        let anyid = AnyidClient::neu_mit_ca(e.anyid_basis.trim(), ca.as_deref())
            .map_err(|err| err.to_string())?;

        let mut openany = OpenanySchalter::neu(e.openany_basis.trim());

        if let Some(pem) = ca {
            openany = openany.mit_ca(pem);
        }

        Ok(Anmeldung::neu(
            anyid,
            openany,
            e.name(),
            format!(
                "{}/einstellungen/geraete",
                e.anyid_basis.trim().trim_end_matches('/')
            ),
            self.ausweis(),
            self.schluesselablage(),
        ))
    }
}

/* ── Was die Oberflaeche zu sehen bekommt ─────────────────────────────── */

#[derive(Serialize)]
pub struct Lage {
    /// Mit openany.de verbunden (die Adresse steht)? Sagt nichts ueber die
    /// Ausweise -- das ist `gekoppelt`.
    verbunden: bool,
    /// Liegen beide Ausweise? **Beide** -- mit dem einen allein laesst sich
    /// nichts abgleichen, mit dem anderen nichts erneuern.
    gekoppelt: bool,
    geraetename: String,
    ordner: String,
    /// Liegen Ausweise und Geraeteschluessel verschlossen (Android-Keystore,
    /// tresor.rs)? Auf dem Schreibtisch noch nicht.
    tresor: bool,
    /// Was der letzte Lauf gemeldet hat -- `None`, wenn er durchlief.
    ///
    /// **Damit bekommt das pulsierende Logo eine Bedeutung.** In der Webapp
    /// ist es Zierde; hier heisst es "die Verbindung steht". Ein Programm,
    /// das offline arbeiten kann, muss den Unterschied zeigen koennen --
    /// sonst sieht ein drei Stunden alter Stand aus wie ein frischer.
    letzter_fehler: Option<String>,
    letzter_lauf: Option<String>,
}

#[derive(Serialize)]
pub struct Kopplungsanzeige {
    /// Anzeigen, nicht verschicken -- abgeholt wird nie mit ihm.
    code: String,
    bestaetigen_url: String,
    intervall: u64,
    ablauf_in: u64,
}

impl Kopplungsanzeige {
    /// Was die Oberflaeche von einer Kopplung sehen darf.
    ///
    /// Der Verifier bleibt drin: Er ist das Geheimnis, mit dem abgeholt wird,
    /// und er hat in keiner Ansicht etwas zu suchen.
    fn aus(kopplung: &Kopplung) -> Self {
        Self {
            code: kopplung.code.clone(),
            bestaetigen_url: kopplung.browserziel.clone(),
            intervall: kopplung.intervall,
            ablauf_in: kopplung.ablauf_in,
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "stand", rename_all = "snake_case")]
pub enum Abholanzeige {
    Wartet {
        intervall: u64,
    },
    Gekoppelt {
        name: String,
    },
    /// Abgelaufen, erfunden, falscher Verifier oder schon abgeholt. Die vier
    /// sind nicht zu unterscheiden, und das ist Absicht.
    Ungueltig,
}

#[derive(Serialize)]
pub struct Abgleichanzeige {
    gezogen: usize,
    geschoben: usize,
    uebersprungen: usize,
    /// Je Grund, wie oft -- damit "uebersprungen 7" eine Auskunft ist und
    /// keine Zahl, die jemand deuten muss (siehe `openany_sync::Bericht`).
    uebersprungen_weil: std::collections::BTreeMap<String, usize>,
    konflikte: usize,
    fehler: Vec<String>,
    belegt: Option<u64>,
    grenze: Option<u64>,
    /// Dateiinhalte, die dieses Geraet nach seiner Regel geholt hat.
    inhalte: Option<openany_sync::InhalteBericht>,
}

/* ── Einstellungen und Lage ───────────────────────────────────────────── */

#[tauri::command]
async fn lage(zustand: tauri::State<'_, Arc<Zustand>>) -> Result<Lage, String> {
    let e = zustand.einstellungen.lock().await.clone();

    // Die Ausweise werden GELESEN und nicht gemerkt: Wer von Hand abmeldet
    // oder die Datei loescht, soll das beim naechsten Blick sehen.
    let gekoppelt = match zustand.anmeldung().await {
        Ok(a) => a.einsatzbereit().unwrap_or(false),
        Err(_) => false,
    };

    // Der Stand gegenueber DIESER Instanz. Ohne eingetragene Adresse gibt es
    // keine Gegenstelle und damit auch nichts zu melden -- das ist kein
    // Fehler, sondern der gewaehlte Zustand "nur auf diesem Geraet".
    let marke = if e.verbunden() {
        zustand.speicher.lock().await.marke(&e.openany_basis).ok()
    } else {
        None
    };

    Ok(Lage {
        verbunden: e.verbunden(),
        gekoppelt,
        geraetename: e.geraetename.clone(),
        ordner: zustand.ordner.display().to_string(),
        tresor: cfg!(target_os = "android"),
        letzter_fehler: marke.as_ref().and_then(|m| m.letzter_fehler.clone()),
        letzter_lauf: marke.and_then(|m| m.letzter_lauf),
    })
}

/// Den Namen dieses Geraets speichern (Kachel „Geraete in der Naehe").
/// Die Adressen setzt seit dem 08.10.2026 nur noch `openany_verbinden`.
#[tauri::command]
async fn einstellungen_speichern(
    zustand: tauri::State<'_, Arc<Zustand>>,
    geraetename: String,
) -> Result<(), String> {
    // EINMAL SPERREN (15.-16.09.2026): zwei `lock().await` im selben Ausdruck
    // verklemmten sich -- das Speichern hing, ohne Fehler.
    let neu = zustand.einstellungen.lock().await.mit_namen(&geraetename);

    neu.schreiben(&zustand.einstellungspfad())
        .map_err(|e| e.to_string())?;

    let name_neu = {
        let mut e = zustand.einstellungen.lock().await;
        let anders = e.name() != neu.name();
        *e = neu;
        anders
    };

    // Ein neuer Name gilt SOFORT in der Naehe: Er steht in jeder Ankuendigung,
    // und die wird beim Start des Nahbereichs festgelegt. Am 15.09.2026 hiess
    // das Telefon nach dem Umbenennen weiter "Dieses Geraet", weil Android die
    // App beim Schliessen nicht beendet hatte.
    if name_neu {
        let name = zustand.einstellungen.lock().await.name();
        zustand.gastgeber.name_setzen(name.clone());
        if let Ok(ident) = geraet_identitaet(&zustand) {
            zustand.gastgeber.person_anlegen(&ident, &name);
        }
        nah_neu_starten(zustand.inner().clone()).await;
    }

    Ok(())
}

/* ── Kopplung ─────────────────────────────────────────────────────────── */

/// „Mit openany.de verbinden" (08.10.2026): die feste Adresse setzen, bei
/// der Instanz erfragen, wo ihr anyid liegt, und die Kopplung beginnen. Gibt
/// den Code fuer den Browser zurueck.
///
/// Zu einer Instanz gehoert genau ein anyid; das ist keine Wahl, sondern eine
/// Auskunft. Ist openany.de nicht erreichbar, bleibt alles, wie es war.
#[tauri::command]
async fn openany_verbinden(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Kopplungsanzeige, String> {
    let ca = zustand.einstellungen.lock().await.ca();
    let anyid = openany_client::anyid_der_instanz(einstellungen::OPENANY, ca.as_deref())
        .await
        .ok_or("openany.de is not reachable right now. Check the connection and try again.")?;
    let neu = zustand.einstellungen.lock().await.mit_openany(&anyid);
    neu.schreiben(&zustand.einstellungspfad())
        .map_err(|e| e.to_string())?;
    *zustand.einstellungen.lock().await = neu;
    kopplung_beginnen(zustand).await
}

async fn kopplung_beginnen(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Kopplungsanzeige, String> {
    let anmeldung = zustand.anmeldung().await?;
    let kopplung = anmeldung
        .koppeln_beginnen()
        .await
        .map_err(|e| e.to_string())?;

    let anzeige = Kopplungsanzeige::aus(&kopplung);

    *zustand.kopplung.lock().await = Some(kopplung);

    Ok(anzeige)
}

/// Laeuft hier noch eine Kopplung? -- gefragt beim Aufbau der Oberflaeche.
///
/// **WARUM ES DAS BRAUCHT.** Die Kopplung geht ueber zwei Programme: Hier
/// steht der Code, bestaetigt wird im Browser. Wer dorthin wechselt, laesst
/// dieses Fenster los -- und Android baut die Webansicht beim Zurueckkommen
/// neu auf. Die Oberflaeche hatte den Code dann vergessen, obwohl er im
/// Zustand des Programms unveraendert weiterlag, und mit ihm den Takt, der
/// nachfragt, ob schon bestaetigt wurde.
///
/// AM 16.09.2026 IST GENAU DAS PASSIERT: Der Code war nach dem Wechsel weg,
/// die Kopplung wurde ein zweites Mal begonnen, und die Bestaetigung im
/// Browser kam bei niemandem an -- es fragte ja keiner mehr nach. Von aussen
/// sah es aus, als habe der Browser nichts bewirkt.
///
/// Der Zustand selbst war nie das Problem; es fehlte nur die Frage danach.
#[tauri::command]
async fn kopplung_laeuft(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Option<Kopplungsanzeige>, String> {
    Ok(zustand
        .kopplung
        .lock()
        .await
        .as_ref()
        .map(Kopplungsanzeige::aus))
}

#[tauri::command]
async fn kopplung_abholen(zustand: tauri::State<'_, Arc<Zustand>>) -> Result<Abholanzeige, String> {
    let laufend = zustand.kopplung.lock().await.clone();

    let Some(kopplung) = laufend else {
        return Ok(Abholanzeige::Ungueltig);
    };

    let anmeldung = zustand.anmeldung().await?;

    match anmeldung
        .koppeln_abholen(&kopplung)
        .await
        .map_err(|e| e.to_string())?
    {
        Kopplungsstand::Wartet { intervall } => Ok(Abholanzeige::Wartet { intervall }),
        Kopplungsstand::Gekoppelt { name } => {
            *zustand.kopplung.lock().await = None;

            Ok(Abholanzeige::Gekoppelt { name })
        }
        Kopplungsstand::Ungueltig => {
            *zustand.kopplung.lock().await = None;

            Ok(Abholanzeige::Ungueltig)
        }
    }
}

/// Beide Ausweise vergessen.
///
/// **Das ist kein Widerruf.** Drueben leben beide weiter, bis sie dort
/// widerrufen werden. Die Oberflaeche sagt das dazu; ein Programm, das
/// "abgemeldet" mit "widerrufen" verwechselt, laesst nach einem
/// Geraeteverlust einen Zugang offen, von dem der Mensch glaubt, er sei zu.
#[tauri::command]
async fn abmelden(zustand: tauri::State<'_, Arc<Zustand>>) -> Result<(), String> {
    zustand.ausweis().vergessen().map_err(|e| e.to_string())?;
    zustand
        .schluesselablage()
        .vergessen()
        .map_err(|e| e.to_string())?;
    *zustand.kopplung.lock().await = None;
    // „Verbindung trennen" (08.10.2026): auch die Adresse fort -- danach ist
    // die App wieder nur auf diesem Geraet. Die Daten hier bleiben.
    let neu = zustand.einstellungen.lock().await.ohne_openany();
    neu.schreiben(&zustand.einstellungspfad())
        .map_err(|e| e.to_string())?;
    *zustand.einstellungen.lock().await = neu;

    Ok(())
}

/* ── Abgleich ─────────────────────────────────────────────────────────── */

/// Beim naechsten Lauf wieder bei null anfangen -- also den BESTAND holen.
///
/// **Wozu.** Die Marke sagt „so weit kenne ich die Gegenstelle". Steht sie auf
/// 0, fragt der Laeufer nicht nach Aenderungen, sondern nach dem Bestand
/// (serverseitig `ChangeLog::bestand`). Das ist der Weg fuer ein Geraet, das
/// nachweislich etwas verpasst hat.
///
/// **Warum es das geben muss.** Bis zum 16.09.2026 lieferte der Server auch
/// bei Marke 0 nur sein Aenderungsprotokoll, und das reichte nicht bis zum
/// Anfang. Geraete, die davor gekoppelt wurden, tragen eine Marke groesser als
/// 0 und wuerden den Bestand nie erfragen -- die Luecke bliebe, obwohl sie
/// drueben geschlossen ist. Ohne diesen Knopf haette der einzige Ausweg
/// „loeschen und neu einrichten" geheissen.
///
/// **Es loescht nichts.** Was hier liegt, bleibt liegen; es kommt nur alles
/// noch einmal herueber und wird dabei zusammengefuehrt. Ein zweiter Abgleich
/// derselben Sache ist kein Streitfall, sondern ein Upsert auf denselben
/// Stand.
#[tauri::command]
async fn bestand_neu_holen(zustand: tauri::State<'_, Arc<Zustand>>) -> Result<(), String> {
    let basis = zustand.einstellungen.lock().await.openany_basis.clone();

    if basis.trim().is_empty() {
        return Err("No instance entered.".into());
    }

    let speicher = zustand.speicher.lock().await;

    speicher
        .fremde_marke_setzen(basis.trim(), 0)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn abgleichen(zustand: tauri::State<'_, Arc<Zustand>>) -> Result<Abgleichanzeige, String> {
    server_abgleich(&zustand, true).await
}

/// Der Abgleich mit openany.de -- fuer den Knopf UND fuer den Auffrischer im
/// Hintergrund (`hintergrund.rs`). Eine Fassung, damit beide dasselbe tun.
///
/// `inhalte_auch`: nach dem Lauf die Bytes holen und hinaufbringen. Im
/// Hintergrund auf einer Mobilfunkleitung lieber nicht -- die Liste ist
/// klein, ein Original nicht.
pub(crate) async fn server_abgleich(
    zustand: &Arc<Zustand>,
    inhalte_auch: bool,
) -> Result<Abgleichanzeige, String> {
    let e = zustand.einstellungen.lock().await.clone();

    if !e.verbunden() {
        return Err("No instance entered.".into());
    }

    let Some(schluessel) = zustand
        .schluesselablage()
        .lesen()
        .map_err(|err| err.to_string())?
    else {
        return Err("This device is not paired.".into());
    };

    let client = OpenanyClient::neu_mit_ca(&e.openany_basis, &schluessel, e.ca().as_deref())
        .map_err(|err| err.to_string())?;

    let gegenstelle = servergegenstelle::ServerGegenstelle::neu(client);

    let mut bericht = {
        let speicher = zustand.speicher.lock().await;
        Laeufer::neu(&speicher, &gegenstelle, e.name()).lauf().await
    };

    // Erst NACH dem Lauf und ohne gesperrten Speicher: Inhalte koennen
    // Minuten dauern, und die Oberflaeche soll bedienbar bleiben.
    //
    // Beide Richtungen, anders als bei einem Geraet in der Naehe: Der Server
    // holt sich nichts, er wartet. Was hier liegt und dort fehlt, muss diese
    // Seite hinbringen.
    let inhalte = if bericht.durchgelaufen() && inhalte_auch {
        let gegenstelle = wegweiser_bauen(zustand, gegenstelle).await?;
        let mut zusammen = dateibefehle::fehlende_holen(zustand, &gegenstelle).await;
        let hinauf = dateibefehle::fehlende_hinauf(zustand, &gegenstelle).await;

        zusammen.geholt += hinauf.geholt;
        zusammen.fehler.extend(hinauf.fehler);

        // NACHREICHEN. Ein hier angelegter Anhang zeigt auf ein Bild, dessen
        // Bytes eben erst hinuebergingen -- der Server legt das Bild erst
        // damit an und hat den Anhang deshalb im Lauf davor uebergangen. Er
        // ist wieder vorgemerkt (`Bericht::offen`); ein zweiter Lauf bringt
        // ihn jetzt, wo das Ziel drueben steht. Nur dann: Ohne Bytes und
        // ohne Offenes waere es ein Lauf, der nichts zu tun hat.
        if bericht.offen > 0 && hinauf.geholt > 0 {
            let speicher = zustand.speicher.lock().await;
            let zweiter = Laeufer::neu(&speicher, &gegenstelle, e.name()).lauf().await;
            bericht.dazu(zweiter);
        }

        Some(zusammen)
    } else {
        None
    };

    // Videos, deren Inhalt jetzt hier liegt, aber noch ohne Standbild: vom
    // Betriebssystem ziehen und zum Server schicken (Weg C, galeriebefehle.rs).
    // Auch im Hintergrund -- der Auffrischer ruft genau diese Funktion.
    galeriebefehle::standbilder_nachziehen(zustand).await;

    Ok(Abgleichanzeige {
        gezogen: bericht.gezogen,
        geschoben: bericht.geschoben,
        uebersprungen: bericht.uebersprungen,
        uebersprungen_weil: bericht.gruende,
        konflikte: bericht.konflikte,
        fehler: bericht.fehler,
        belegt: bericht.speicher.map(|s| s.belegt),
        grenze: bericht.speicher.and_then(|s| s.grenze),
        inhalte,
    })
}

/// Die Server-Gegenstelle, fertig mit Wegweiser -- oder `None`, wenn dieses
/// Geraet keine Instanz hat oder nicht gekoppelt ist.
///
/// **WOZU AUSSERHALB DES ABGLEICHS.** `von_geraeten_holen` fragte bis zum
/// 16.09.2026 ausschliesslich Geraete in der Naehe -- der Typ der Liste
/// (`Vec<NahGegenstelle>`) schloss den Server aus. Wer bei "bei Bedarf" eine
/// Datei oeffnete, deren Inhalt nur beim Server liegt, bekam:
///
/// ```text
/// Der Inhalt liegt auf einem anderen Geraet, und keines ist gerade erreichbar.
/// ```
///
/// Das Geraet war gekoppelt und online, und die Datei lag bereit. Gefragt
/// wurde nur nie.
///
/// Es ging auch erst seit heute: Ohne Abdruck im Feed haette der Wegweiser
/// fuer Dateien und Bilder nichts zu uebersetzen gehabt.
/// Ein Client fuer die eingetragene Instanz -- oder `None`, wenn dieses
/// Geraet keine hat oder nicht gekoppelt ist.
///
/// Fuer die Projektstroeme: Sie sprechen mit derselben Instanz und demselben
/// Schluessel wie der persoenliche Lauf, nur unter anderen Adressen.
pub(crate) async fn server_client(zustand: &Arc<Zustand>) -> Option<OpenanyClient> {
    let e = zustand.einstellungen.lock().await.clone();

    if !e.verbunden() {
        return None;
    }

    let schluessel = zustand.schluesselablage().lesen().ok()??;

    OpenanyClient::neu_mit_ca(&e.openany_basis, &schluessel, e.ca().as_deref()).ok()
}

pub(crate) async fn server_gegenstelle(
    zustand: &Arc<Zustand>,
) -> Option<servergegenstelle::ServerGegenstelle> {
    let e = zustand.einstellungen.lock().await.clone();

    if !e.verbunden() {
        return None;
    }

    let schluessel = zustand.schluesselablage().lesen().ok()??;
    let client =
        OpenanyClient::neu_mit_ca(&e.openany_basis, &schluessel, e.ca().as_deref()).ok()?;

    wegweiser_bauen(zustand, servergegenstelle::ServerGegenstelle::neu(client))
        .await
        .ok()
}

/// Die beiden Tabellen bauen, mit denen die Gegenstelle uebersetzen kann.
///
/// Erst jetzt, nach dem Lauf: Was drueben neu ist, steht hier auch erst seit
/// eben, und ein Wegweiser von vorher kennte es nicht.
async fn wegweiser_bauen(
    zustand: &Arc<Zustand>,
    gegenstelle: servergegenstelle::ServerGegenstelle,
) -> Result<servergegenstelle::ServerGegenstelle, String> {
    let basis = gegenstelle.basis_adresse().to_string();
    let speicher = zustand.speicher.lock().await;

    let mut wegweiser = std::collections::BTreeMap::new();
    let mut alle = Vec::new();

    for d in speicher.lebendige_dateien().map_err(|e| e.to_string())? {
        if let Some(a) = d.abdruck {
            wegweiser.insert(a.clone(), ("file_node".to_string(), d.uuid));
            alle.push(a);
        }
    }

    for b in speicher.lebendige_bilder().map_err(|e| e.to_string())? {
        /*
         * ZWEI STROEME, ZWEI ADRESSEN. Ein Bild hat ein Original und eine
         * Vorschau; ein Abdruck zeigt auf genau einen von beiden.
         *
         * Bis zum 16.09.2026 stand hier nur das Original. Die Vorschauen
         * wurden zwar zum Holen angemeldet (`dateibefehle::fehlend` nimmt sie
         * IMMER auf, auch bei "bei Bedarf"), fanden aber keine Adresse und
         * fielen still durch. In der Galerie stand "Leer" ueber einem Album,
         * das "5 Bilder" sagte.
         *
         * Die Vorschau zuerst, und das ist nicht Geschmack: Traegt ein Bild
         * denselben Abdruck fuer beides -- bei einem Bild, das kleiner ist
         * als seine Vorschau, faellt die Verkleinerung auf das Original
         * zurueck -- dann soll das ORIGINAL gewinnen. Es ist die
         * vollstaendige Antwort; die Vorschau waere nur zufaellig dieselbe.
         */
        if let Some(v) = b.vorschau {
            wegweiser.insert(v.clone(), ("media_vorschau".to_string(), b.uuid.clone()));
            alle.push(v);
        }

        if let Some(a) = b.abdruck {
            wegweiser.insert(a.clone(), ("media".to_string(), b.uuid));
            alle.push(a);
        }
    }

    /*
     * ANHAENGE BRAUCHEN EINE EIGENE ZEILE IM WEGWEISER.
     *
     * Ein Bild an einer Notiz kommt drueben nicht im `media`-Strom (Regel 1),
     * also steht es nicht in `lebendige_bilder` und faende hier keine
     * Adresse. Sein Ziel ist aber ein gewoehnliches Medium -- geholt wird es
     * also als `media_vorschau` beziehungsweise `media`.
     */
    for a in speicher.anhaenge_alle().map_err(|e| e.to_string())? {
        let art = if a.ziel_art == "media" {
            "media"
        } else {
            "file_node"
        };

        if let Some(v) = a.vorschau {
            wegweiser.insert(v.clone(), (format!("{art}_vorschau"), a.ziel_uuid.clone()));
            alle.push(v);
        }

        if let Some(d) = a.abdruck {
            wegweiser.insert(d.clone(), (art.to_string(), a.ziel_uuid));
            alle.push(d);
        }
    }

    let dort = speicher
        .inhalte_dort(&basis, &alle)
        .map_err(|e| e.to_string())?;

    Ok(gegenstelle.mit_wegweiser(wegweiser, dort))
}

/* ── Notizen ──────────────────────────────────────────────────────────── */

/*
 * Die Notizen-Arbeitsflaeche ist dieselbe wie in der Webapp
 * (`packages/oberflaeche/notizen`). Diese Befehle liefern ihr, was drueben
 * die Endpunkte unter /api/notes liefern -- in denselben Feldnamen (`id`,
 * `title`, `note_folder_id` …), damit `src/quellen/notizen.js` nur umverpackt
 * und nichts umbenennt.
 *
 * `id` ist hier die `zk_id` (Text), Mappen-Ids sind Abdruecke der Pfade
 * (openany_store::mappen_id).
 */

#[derive(Serialize)]
pub struct NotizKurz {
    id: String,
    title: String,
    note_folder_id: Option<u32>,
    updated_at: String,
    access: &'static str,
}

#[derive(Serialize)]
pub struct NotizVoll {
    id: String,
    title: String,
    content: String,
    note_folder_id: Option<u32>,
    updated_at: String,
    access: &'static str,
}

#[derive(Serialize)]
pub struct Notizseite {
    items: Vec<NotizKurz>,
    next_page: Option<usize>,
}

#[derive(Serialize)]
pub struct MappeAnzeige {
    id: u32,
    name: String,
    parent_id: Option<u32>,
}

#[derive(Serialize)]
pub struct Titel {
    id: String,
    title: String,
}

#[derive(Serialize)]
pub struct TagAnzeige {
    tag: String,
    count: usize,
}

fn mappe_von(n: &Notiz) -> Option<u32> {
    n.mappe.as_deref().filter(|m| !m.is_empty()).map(mappen_id)
}

fn voll(n: Notiz) -> NotizVoll {
    NotizVoll {
        note_folder_id: mappe_von(&n),
        id: n.zk_id,
        title: n.titel,
        content: n.inhalt,
        updated_at: n.geaendert_at,
        access: "owner",
    }
}

/// Id -> Pfad. Eine unbekannte Id ist ein Fehler und nicht die Wurzel: Wer
/// eine Notiz in eine eben geloeschte Mappe schiebt, soll das hoeren, statt sie
/// still ganz oben wiederzufinden.
fn pfad_zu(speicher: &Speicher, id: Option<u32>) -> Result<Option<String>, String> {
    match id {
        None => Ok(None),
        Some(id) => speicher
            .mappen_pfad(id)
            .map_err(|e| e.to_string())?
            .map(Some)
            .ok_or_else(|| "This folder no longer exists.".to_string()),
    }
}

#[tauri::command]
async fn notizbuch_liste(
    zustand: tauri::State<'_, Arc<Zustand>>,
    mappe: Option<u32>,
    seite: Option<usize>,
    tag: Option<String>,
    suche: Option<String>,
) -> Result<Notizseite, String> {
    let speicher = zustand.speicher.lock().await;
    let seite = seite.unwrap_or(1).max(1);
    let filter = Filter {
        mappe: pfad_zu(&speicher, mappe)?,
        ueberall: false,
        tag: tag.filter(|t| !t.is_empty()),
        suche,
    };
    let ergebnis = speicher
        .notizliste(&filter, seite)
        .map_err(|e| e.to_string())?;

    Ok(Notizseite {
        items: ergebnis
            .notizen
            .into_iter()
            .map(|n| NotizKurz {
                note_folder_id: mappe_von(&n),
                id: n.zk_id,
                title: n.titel,
                updated_at: n.geaendert_at,
                access: "owner",
            })
            .collect(),
        next_page: ergebnis.weitere.then_some(seite + 1),
    })
}

#[tauri::command]
async fn notizbuch_notiz(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: String,
) -> Result<NotizVoll, String> {
    let speicher = zustand.speicher.lock().await;

    speicher
        .notiz(&id)
        .map_err(|e| e.to_string())?
        .filter(|n| !n.im_papierkorb())
        .map(voll)
        .ok_or_else(|| "This note no longer exists.".to_string())
}

#[tauri::command]
async fn notizbuch_anlegen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    titel: String,
    inhalt: String,
    mappe: Option<u32>,
) -> Result<NotizVoll, String> {
    let speicher = zustand.speicher.lock().await;
    let zk_id = freie_zk_id(&speicher)?;
    let notiz = Notiz {
        zk_id: zk_id.clone(),
        titel,
        inhalt,
        mappe: pfad_zu(&speicher, mappe)?,
        papierkorb_at: None,
        geaendert_at: String::new(),
    };

    // MERKEN: eine Aenderung dieses Geraets, beim naechsten Lauf hinueber.
    speicher
        .notiz_schreiben(&notiz, Protokoll::Merken)
        .map_err(|e| e.to_string())?;

    speicher
        .notiz(&zk_id)
        .map_err(|e| e.to_string())?
        .map(voll)
        .ok_or_else(|| "The note could not be created.".to_string())
}

/// Aendern, was mitkommt. `mappe_setzen` unterscheidet "nach ganz oben"
/// (`mappe: null`) von "Mappe nicht anfassen" -- beides waere sonst `null`.
#[tauri::command]
async fn notizbuch_aendern(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: String,
    titel: Option<String>,
    inhalt: Option<String>,
    mappe_setzen: Option<bool>,
    mappe: Option<u32>,
) -> Result<NotizVoll, String> {
    // BEARBEITET JEMAND VOR ORT diese Notiz (sie liegt in einer Mappe, die
    // dieses Geraet zum Bearbeiten in ein lokales Projekt freigibt)? Dann
    // wartet das eigene Speichern -- sonst gaebe es zwei Fassungen. Und
    // umgekehrt haelt, wer hier schreibt, die Sperre selbst (sperren.rs).
    {
        use openany_nahbereich::Gastgeber;
        let g = &zustand.gastgeber;
        if let Some(ich) = g.person() {
            let selbst = openany_nahbereich::sperren::Halter {
                personen_id: ich.personen_id.clone(),
                name: ich.name.clone(),
                geraet: String::new(),
            };
            if let Ok(mut s) = g.sperren.lock() {
                if let Err(h) = s.nehmen(&id, selbst) {
                    return Err(format!(
                        "{} is editing this note nearby right now -- saving works again once {} is done.",
                        h.name, h.name
                    ));
                }
            }
        }
    }
    let speicher = zustand.speicher.lock().await;
    let mut notiz = speicher
        .notiz(&id)
        .map_err(|e| e.to_string())?
        .filter(|n| !n.im_papierkorb())
        .ok_or_else(|| "This note no longer exists.".to_string())?;

    if let Some(t) = titel {
        notiz.titel = t;
    }
    if let Some(i) = inhalt {
        notiz.inhalt = i;
    }
    if mappe_setzen.unwrap_or(false) {
        notiz.mappe = pfad_zu(&speicher, mappe)?;
    }
    notiz.geaendert_at = String::new();

    speicher
        .notiz_schreiben(&notiz, Protokoll::Merken)
        .map_err(|e| e.to_string())?;

    speicher
        .notiz(&id)
        .map_err(|e| e.to_string())?
        .map(voll)
        .ok_or_else(|| "The note could not be saved.".to_string())
}

/// Ein Anhang, wie ihn die geteilte Notiz-Oberflaeche braucht.
#[derive(Serialize)]
pub struct AnhangAnzeige {
    /// Mappenpfad|Pfad -- wie auf der Leitung. Die Oberflaeche reicht ihn
    /// nur durch (drueben ist es eine Nummer).
    id: String,
    /// Der relative Pfad, wie er im Notiztext steht -- der Schluessel, unter
    /// dem `assetResolver` nachschlaegt.
    path: String,
    target_type: String,
    target_uuid: String,
    /// Wo die Bytes auf DIESEM Geraet liegen -- `None`, wenn sie fehlen.
    ///
    /// **Er reist mit der Liste und wird nicht einzeln erfragt**, weil
    /// `assetResolver` synchron ist: Er bekommt einen Pfad und muss sofort
    /// eine Adresse liefern. Ein Nachschlagen je Bild waere ein Aufruf je
    /// Zeichenvorgang.
    pfad: Option<String>,
}

/// Die Anhaenge einer Mappe -- Zuordnung von Pfad im Text zu Bild oder Datei.
///
/// **Ein Bild wird als VORSCHAU eingebettet**, wenn eine da ist. Dieselbe
/// Wahl trifft die Webapp (`?thumb=1`): Im Flusstext ist das Vorschaubild die
/// richtige Groesse, und ein Geraet "bei Bedarf" hat das Original oft gar
/// nicht -- die Vorschau dagegen immer.
#[tauri::command]
async fn notizbuch_anhaenge(
    zustand: tauri::State<'_, Arc<Zustand>>,
    mappe: Option<u32>,
) -> Result<Vec<AnhangAnzeige>, String> {
    let speicher = zustand.speicher.lock().await;

    // Die Oberflaeche kennt die Mappe als Id, der Anhang haengt am PFAD --
    // so kommt er ueber die Leitung, und so heisst die Mappe im Speicher.
    let pfad = match mappe {
        None => String::new(),
        Some(id) => speicher
            .mappen()
            .map_err(|e| e.to_string())?
            .into_iter()
            .find(|m| m.id == id)
            .map(|m| m.pfad)
            .unwrap_or_default(),
    };

    let mut liste = Vec::new();

    for a in speicher
        .anhaenge_der_mappe(&pfad)
        .map_err(|e| e.to_string())?
    {
        /*
         * DIE ABDRUECKE STEHEN AM ANHANG, nicht am Ziel.
         *
         * Ein Bild an einer Notiz hat hier keine Bildzeile: Drueben haelt
         * "Regel 1" es aus dem `media`-Strom heraus, es reist als Anhang.
         * Wer `speicher.bild(ziel_uuid)` fragte, bekaeme `None` -- und die
         * Notiz zeigte weiter nichts.
         *
         * Vorschau zuerst: Im Flusstext ist sie die richtige Groesse, und auf
         * einem Geraet "bei Bedarf" ist sie oft die einzige, die da ist.
         */
        let abdruck = a.vorschau.clone().or_else(|| a.abdruck.clone());

        liste.push(AnhangAnzeige {
            id: a.schluessel(),
            pfad: abdruck.and_then(|a| {
                zustand
                    .inhalte
                    .pfad(&a)
                    .filter(|p| p.is_file())
                    .map(|p| p.to_string_lossy().to_string())
            }),
            path: a.pfad,
            target_type: a.ziel_art,
            target_uuid: a.ziel_uuid,
        });
    }

    Ok(liste)
}

#[tauri::command]
async fn notizbuch_mappen(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Vec<MappeAnzeige>, String> {
    let speicher = zustand.speicher.lock().await;

    Ok(speicher
        .mappen()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|m| MappeAnzeige {
            id: m.id,
            name: m.name,
            parent_id: m.eltern_id,
        })
        .collect())
}

#[tauri::command]
async fn notizbuch_mappe_anlegen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    name: String,
    eltern: Option<u32>,
) -> Result<u32, String> {
    let speicher = zustand.speicher.lock().await;
    let eltern = pfad_zu(&speicher, eltern)?;
    let pfad = speicher
        .mappe_anlegen(eltern.as_deref(), &name)
        .map_err(|e| e.to_string())?;

    Ok(mappen_id(&pfad))
}

#[tauri::command]
async fn notizbuch_mappe_umbenennen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: u32,
    name: String,
) -> Result<u32, String> {
    let speicher = zustand.speicher.lock().await;
    let pfad = pfad_zu(&speicher, Some(id))?.unwrap_or_default();
    let neu = speicher
        .mappe_umbenennen(&pfad, &name)
        .map_err(|e| e.to_string())?;

    Ok(mappen_id(&neu))
}

/// `modus`: `delete_notes` oder `move_to_parent` -- dieselben Woerter wie drueben.
#[tauri::command]
async fn notizbuch_mappe_loeschen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: u32,
    modus: String,
) -> Result<(), String> {
    let speicher = zustand.speicher.lock().await;
    let pfad = pfad_zu(&speicher, Some(id))?.unwrap_or_default();

    speicher
        .mappe_loeschen(&pfad, modus == "delete_notes")
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn notizbuch_titel(zustand: tauri::State<'_, Arc<Zustand>>) -> Result<Vec<Titel>, String> {
    let speicher = zustand.speicher.lock().await;

    Ok(speicher
        .notiztitel()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|(id, title)| Titel { id, title })
        .collect())
}

#[tauri::command]
async fn notizbuch_aufloesen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    ziel: String,
) -> Result<Option<Titel>, String> {
    let speicher = zustand.speicher.lock().await;

    Ok(speicher
        .verweisziel(&ziel)
        .map_err(|e| e.to_string())?
        .map(|n| Titel {
            id: n.zk_id,
            title: n.titel,
        }))
}

#[tauri::command]
async fn notizbuch_tags(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Vec<TagAnzeige>, String> {
    let speicher = zustand.speicher.lock().await;

    Ok(speicher
        .notiztags()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|(tag, count)| TagAnzeige { tag, count })
        .collect())
}

#[tauri::command]
async fn notizbuch_rueckverweise(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: String,
) -> Result<Vec<Titel>, String> {
    let speicher = zustand.speicher.lock().await;

    Ok(speicher
        .rueckverweise(&id)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|(id, title)| Titel { id, title })
        .collect())
}

/// Der Notiz-Graph in der Form, die drueben `/api/notes/graph` liefert --
/// `NotesGraph.vue` (im Paket) zeichnet beides.
#[tauri::command]
async fn notizbuch_graph(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<serde_json::Value, String> {
    let g = zustand
        .speicher
        .lock()
        .await
        .notizgraph()
        .map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "nodes": g.knoten.iter().map(|k| serde_json::json!({
            "id": k.id,
            "title": k.titel,
            "folder": k.mappe,
            "group": k.gruppe,
            "groupName": k.gruppenname,
        })).collect::<Vec<_>>(),
        "edges": g.kanten.iter().map(|(von, nach)| serde_json::json!({ "from": von, "to": nach })).collect::<Vec<_>>(),
        "tags": g.tags.iter().map(|(tag, zahl)| serde_json::json!({ "tag": tag, "count": zahl })).collect::<Vec<_>>(),
        "tagEdges": g.tag_kanten.iter().map(|(notiz, tag)| serde_json::json!({ "note": notiz, "tag": tag })).collect::<Vec<_>>(),
    }))
}

#[tauri::command]
async fn notiz_papierkorb(
    zustand: tauri::State<'_, Arc<Zustand>>,
    zk_id: String,
) -> Result<(), String> {
    let speicher = zustand.speicher.lock().await;

    speicher
        .papierkorb(&openany_client::Art::Notiz, &zk_id, Protokoll::Merken)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Eine freie `zk_id` im Zettlr-Format -- dieselbe Regel wie drueben in
/// `Note::generateZkId()`: bei Kollision je eine Sekunde weiter.
fn freie_zk_id(speicher: &Speicher) -> Result<String, String> {
    let mut moment = chrono::Local::now();

    for _ in 0..86_400 {
        let vorschlag = moment.format("%Y%m%d%H%M%S").to_string();

        if speicher
            .notiz(&vorschlag)
            .map_err(|e| e.to_string())?
            .is_none()
        {
            return Ok(vorschlag);
        }

        moment += chrono::Duration::seconds(1);
    }

    Err("No free ID found.".into())
}

/* ── Kalender ─────────────────────────────────────────────────────────── */

/*
 * Der Kalender -- dieselbe Arbeitsflaeche wie in der Webapp
 * (`packages/oberflaeche/kalender`), mit den Feldnamen von /api/calendars und
 * /api/events. Ids sind hier die uuids. Zeiten gehen durch
 * `kalenderbuch::zeit`, damit der Abdruck dem des Servers gleicht.
 */

#[derive(Serialize)]
pub struct KalenderAnzeige {
    id: String,
    name: String,
    color: String,
    /// Kein Abo-Abgleich und kein Abo-Link ohne Server.
    sync_url: Option<String>,
    ics_feed_token: Option<String>,
}

#[derive(Serialize)]
pub struct TerminAnzeige {
    id: String,
    calendar_id: String,
    title: String,
    description: String,
    location: String,
    start_date: String,
    end_date: String,
    is_all_day: bool,
    rrule: String,
    rrule_until: Option<String>,
    exdates: String,
    color: String,
}

/// `sync_url` traegt hier doppelt: Die Oberflaeche zeigt die Adresse damit an
/// UND leitet daraus ab, dass in diesen Kalender nicht geschrieben wird
/// (`beschreibbare` in KalenderArbeitsflaeche.vue). Beides ist dasselbe
/// Merkmal, also soll es auch dasselbe Feld sein.
///
/// `ics_feed_token` bleibt leer: Einen Feed ANBIETEN kann nur ein Server, der
/// unter einer Adresse steht. Ein Telefon steht unter keiner.
fn kalender_anzeige(k: Kalender) -> KalenderAnzeige {
    KalenderAnzeige {
        id: k.uuid,
        name: k.name,
        color: k.farbe,
        sync_url: k.abo_url,
        ics_feed_token: None,
    }
}

fn termin_anzeige(t: Termin, farbe: &str) -> TerminAnzeige {
    let f = t.felder;
    TerminAnzeige {
        id: t.uuid,
        calendar_id: t.kalender_uuid,
        title: f.titel,
        description: f.beschreibung,
        location: f.ort,
        start_date: f.beginn,
        end_date: f.ende,
        is_all_day: f.ganztags,
        rrule: if f.rrule.is_empty() {
            "NONE".into()
        } else {
            f.rrule
        },
        rrule_until: Some(f.rrule_bis).filter(|s| !s.is_empty()),
        exdates: f.exdates,
        color: farbe.to_string(),
    }
}

fn kalender_holen(speicher: &Speicher, id: &str) -> Result<Kalender, String> {
    speicher
        .kalender(id)
        .map_err(|e| e.to_string())?
        .filter(|k| k.papierkorb_at.is_none())
        .ok_or_else(|| "This calendar no longer exists.".to_string())
}

#[tauri::command]
async fn kalenderbuch_kalender(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Vec<KalenderAnzeige>, String> {
    let speicher = zustand.speicher.lock().await;

    Ok(speicher
        .kalender_alle()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(kalender_anzeige)
        .collect())
}

/// Einen Kalender anlegen.
///
/// `abo_url` ist die Adresse eines ICS-Feeds; leer oder fehlend heisst: ein
/// eigener Kalender. Geprueft wird sie in `openany_sync::adresse_pruefen`.
#[tauri::command]
async fn kalenderbuch_kalender_anlegen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    name: String,
    farbe: Option<String>,
    abo_url: Option<String>,
) -> Result<KalenderAnzeige, String> {
    let abo = abo_url
        .map(|u| u.trim().to_string())
        .filter(|u| !u.is_empty());

    if let Some(u) = &abo {
        openany_sync::adresse_pruefen(u).map_err(|e| e.to_string())?;
    }

    let speicher = zustand.speicher.lock().await;
    let kalender = Kalender {
        uuid: uuid::Uuid::new_v4().to_string(),
        name,
        // Dieselbe Vorgabe wie drueben (CalendarController::store).
        farbe: farbe
            .filter(|f| !f.is_empty())
            .unwrap_or_else(|| "#4f46e5".into()),
        abo_url: abo,
        zuletzt_geholt: None,
        papierkorb_at: None,
        geaendert_at: String::new(),
    };

    speicher
        .kalender_schreiben(&kalender, Protokoll::Merken)
        .map_err(|e| e.to_string())?;

    Ok(kalender_anzeige(kalender_holen(&speicher, &kalender.uuid)?))
}

/// Einen abonnierten Kalender jetzt auffrischen.
///
/// Von Hand angestossen. Regelmaessig passiert dasselbe im Hintergrund --
/// das ist der naechste Schritt und steht noch nicht.
#[tauri::command]
async fn kalenderbuch_abo_auffrischen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: String,
) -> Result<Abo, String> {
    // Die Quelle wird VOR dem Schloss gebaut und das Holen laeuft ausserhalb:
    // Ein Netzabruf kann dreissig Sekunden dauern, und so lange darf nichts
    // anderes in der App auf den Speicher warten muessen.
    let quelle = openany_sync::UeberDasNetz::neu().map_err(|e| e.to_string())?;

    let speicher = zustand.speicher.lock().await;
    let kalender = kalender_holen(&speicher, &id)?;

    if !kalender.ist_abo() {
        return Err("This calendar is not a subscription.".into());
    }

    let bericht = openany_sync::auffrischen(&speicher, &kalender, &quelle)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Abo {
        neu: bericht.neu,
        geaendert: bericht.geaendert,
        entfernt: bericht.entfernt,
        unveraendert: bericht.unveraendert,
        uebersprungen: bericht.uebersprungen,
    })
}

/// Was ein Auffrischen ergeben hat -- fuer die Oberflaeche.
#[derive(Serialize)]
pub struct Abo {
    neu: usize,
    geaendert: usize,
    entfernt: usize,
    unveraendert: usize,
    uebersprungen: Vec<String>,
}

/// Einen Kalender aendern.
///
/// Bei `abo_url` heisst ein leerer Text: Abo aufgeben, der Kalender bleibt
/// mit seinen Terminen stehen. Ein fehlendes Feld heisst: unveraendert.
#[tauri::command]
async fn kalenderbuch_kalender_aendern(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: String,
    name: Option<String>,
    farbe: Option<String>,
    abo_url: Option<String>,
) -> Result<KalenderAnzeige, String> {
    if let Some(u) = abo_url.as_deref().map(str::trim).filter(|u| !u.is_empty()) {
        openany_sync::adresse_pruefen(u).map_err(|e| e.to_string())?;
    }

    let speicher = zustand.speicher.lock().await;
    let mut kalender = kalender_holen(&speicher, &id)?;

    if let Some(n) = name {
        kalender.name = n;
    }
    if let Some(f) = farbe {
        kalender.farbe = f;
    }
    if let Some(u) = abo_url {
        let u = u.trim().to_string();
        // Eine neue Adresse heisst: noch nie geholt. Sonst haelte der
        // Auffrischer die alte Stunde fuer die neue.
        if kalender.abo_url.as_deref() != Some(u.as_str()) {
            kalender.zuletzt_geholt = None;
        }
        kalender.abo_url = Some(u).filter(|u| !u.is_empty());
    }
    kalender.geaendert_at = String::new();

    speicher
        .kalender_schreiben(&kalender, Protokoll::Merken)
        .map_err(|e| e.to_string())?;

    Ok(kalender_anzeige(kalender_holen(&speicher, &id)?))
}

/// In den Papierkorb -- wie drueben (SoftDeletes). Die Termine bleiben liegen
/// und tauchen mit dem Kalender wieder auf, wenn er zurueckgeholt wird.
#[tauri::command]
async fn kalenderbuch_kalender_loeschen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: String,
) -> Result<(), String> {
    let speicher = zustand.speicher.lock().await;

    speicher
        .papierkorb(&openany_client::Art::Kalender, &id, Protokoll::Merken)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn kalenderbuch_termine(
    zustand: tauri::State<'_, Arc<Zustand>>,
    von: String,
    bis: String,
) -> Result<Vec<TerminAnzeige>, String> {
    let speicher = zustand.speicher.lock().await;

    // Nur Termine lebender Kalender -- wie drueben, wo die Abfrage ueber die
    // Kalender des Menschen laeuft. Die Farbe reist mit, wie dort.
    let farben: std::collections::HashMap<String, String> = speicher
        .kalender_alle()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|k| (k.uuid, k.farbe))
        .collect();

    Ok(speicher
        .termine_im_fenster(&von, &bis)
        .map_err(|e| e.to_string())?
        .into_iter()
        .filter_map(|t| {
            let farbe = farben.get(&t.kalender_uuid)?.clone();
            Some(termin_anzeige(t, &farbe))
        })
        .collect())
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
async fn kalenderbuch_termin_speichern(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: Option<String>,
    kalender: String,
    titel: String,
    beschreibung: Option<String>,
    ort: Option<String>,
    beginn: String,
    ende: String,
    ganztags: bool,
    rrule: Option<String>,
) -> Result<TerminAnzeige, String> {
    let speicher = zustand.speicher.lock().await;
    let farbe = kalender_holen(&speicher, &kalender)?.farbe;
    let uuid = id
        .filter(|u| !u.is_empty())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // Was die Arbeitsflaeche nicht kennt (Serienende, Ausnahmen), bleibt
    // stehen: Ein Termin, der beim Umbenennen seine Ausnahmen verliert, ist
    // ein Datenverlust ohne Fehlermeldung.
    let mut felder = speicher
        .termin(&uuid)
        .map_err(|e| e.to_string())?
        .map(|t| t.felder)
        .unwrap_or_default();

    felder.titel = titel;
    felder.beschreibung = beschreibung.unwrap_or_default();
    felder.ort = ort.unwrap_or_default();
    felder.beginn = kalenderbuch::zeit(&beginn);
    felder.ende = kalenderbuch::zeit(&ende);
    felder.ganztags = ganztags;
    felder.rrule = rrule.unwrap_or_else(|| "NONE".into());

    let termin = Termin {
        uuid: uuid.clone(),
        kalender_uuid: kalender,
        felder,
        papierkorb_at: None,
        geaendert_at: String::new(),
    };

    speicher
        .termin_schreiben(&termin, Protokoll::Merken)
        .map_err(|e| e.to_string())?;

    Ok(termin_anzeige(termin, &farbe))
}

#[tauri::command]
async fn termin_papierkorb(
    zustand: tauri::State<'_, Arc<Zustand>>,
    uuid: String,
) -> Result<(), String> {
    let speicher = zustand.speicher.lock().await;

    speicher
        .papierkorb(&openany_client::Art::Termin, &uuid, Protokoll::Merken)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/* ── Kontakte ─────────────────────────────────────────────────────────── */

/*
 * Das Adressbuch -- dieselbe Arbeitsflaeche wie in der Webapp
 * (`packages/oberflaeche/adressbuch`), mit den Feldnamen von /api/contacts.
 * Ids sind die uuids. Das Foto kommt als fertige Adresse (`data:`) mit: Es
 * gibt keinen Server, der es unter einer URL ausliefert.
 */

#[derive(Serialize)]
pub struct FotoAnzeige {
    hash: String,
    url: String,
}

#[derive(Serialize)]
pub struct KontaktAnzeige {
    id: String,
    display_name: String,
    channels: Vec<openany_store::Weg>,
    photo: Option<FotoAnzeige>,
}

/// Welche Bildart die Bytes tragen -- fuer die `data:`-Adresse.
fn bildart(bytes: &[u8]) -> &'static str {
    match bytes {
        [0x89, b'P', b'N', b'G', ..] => "image/png",
        [b'R', b'I', b'F', b'F', _, _, _, _, b'W', b'E', b'B', b'P', ..] => "image/webp",
        [b'G', b'I', b'F', ..] => "image/gif",
        _ => "image/jpeg",
    }
}

fn kontakt_anzeige(speicher: &Speicher, k: Kontakt) -> Result<KontaktAnzeige, String> {
    use base64::Engine;

    let photo = match &k.foto {
        Some(hash) => speicher
            .kontaktfoto(&k.uuid)
            .map_err(|e| e.to_string())?
            .map(|bytes| FotoAnzeige {
                hash: hash.clone(),
                url: format!(
                    "data:{};base64,{}",
                    bildart(&bytes),
                    base64::engine::general_purpose::STANDARD.encode(&bytes)
                ),
            }),
        None => None,
    };

    Ok(KontaktAnzeige {
        id: k.uuid,
        display_name: k.anzeigename,
        channels: k.wege,
        photo,
    })
}

fn kontakt_holen(speicher: &Speicher, id: &str) -> Result<Kontakt, String> {
    speicher
        .kontakt(id)
        .map_err(|e| e.to_string())?
        .filter(|k| k.papierkorb_at.is_none())
        .ok_or_else(|| "This contact no longer exists.".to_string())
}

#[tauri::command]
async fn adressbuch_kontakte(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Vec<KontaktAnzeige>, String> {
    let speicher = zustand.speicher.lock().await;
    let liste = speicher.kontakte().map_err(|e| e.to_string())?;

    liste
        .into_iter()
        .map(|k| kontakt_anzeige(&speicher, k))
        .collect()
}

/// Anlegen oder aendern. Wie drueben (`Contact::wegeSetzen`): leere Zeilen
/// fallen weg, eine leere Beschriftung ist keine.
#[tauri::command]
async fn adressbuch_speichern(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: Option<String>,
    name: String,
    wege: Vec<openany_store::Weg>,
) -> Result<KontaktAnzeige, String> {
    let speicher = zustand.speicher.lock().await;
    let uuid = id
        .filter(|u| !u.is_empty())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let foto = speicher
        .kontakt(&uuid)
        .map_err(|e| e.to_string())?
        .and_then(|k| k.foto);

    let wege = wege
        .into_iter()
        .filter_map(|mut w| {
            w.wert = w.wert.trim().to_string();
            w.beschriftung = w
                .beschriftung
                .map(|b| b.trim().to_string())
                .filter(|b| !b.is_empty());
            (!w.wert.is_empty()).then_some(w)
        })
        .collect();

    speicher
        .kontakt_schreiben(
            &Kontakt {
                uuid: uuid.clone(),
                anzeigename: name.trim().to_string(),
                wege,
                foto,
                papierkorb_at: None,
                geaendert_at: String::new(),
            },
            Protokoll::Merken,
        )
        .map_err(|e| e.to_string())?;

    kontakt_anzeige(&speicher, kontakt_holen(&speicher, &uuid)?)
}

#[tauri::command]
async fn adressbuch_loeschen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: String,
) -> Result<(), String> {
    let speicher = zustand.speicher.lock().await;

    speicher
        .papierkorb(&openany_client::Art::Kontakt, &id, Protokoll::Merken)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Das Foto als Base64 -- die Oberflaeche hat es vorher verkleinert.
#[tauri::command]
async fn adressbuch_foto_setzen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: String,
    bild: String,
) -> Result<(), String> {
    use base64::Engine;

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(bild.trim())
        .map_err(|_| "The picture cannot be read.".to_string())?;
    let speicher = zustand.speicher.lock().await;
    kontakt_holen(&speicher, &id)?;

    speicher
        .kontaktfoto_setzen(&id, &bytes)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn adressbuch_foto_entfernen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    id: String,
) -> Result<(), String> {
    let speicher = zustand.speicher.lock().await;

    speicher
        .kontaktfoto_entfernen(&id)
        .map_err(|e| e.to_string())
}

/*
 * Der Papierkorb -- dieselbe Ansicht wie in der Webapp
 * (`packages/oberflaeche/papierkorb`), mit den Feldnamen von /api/trash.
 */

#[derive(Serialize)]
pub struct WeggelegtAnzeige {
    #[serde(rename = "type")]
    art: &'static str,
    id: String,
    name: String,
    context: Option<String>,
    deleted_at: String,
}

#[derive(Serialize)]
pub struct PapierkorbSeite {
    items: Vec<WeggelegtAnzeige>,
    next_page: Option<usize>,
}

/// Nur die Arten, die dieses Programm traegt -- eine andere Zeichenkette von
/// der Oberflaeche wird abgewiesen und nicht in eine Tabelle uebersetzt.
fn papierkorb_art(art: &str) -> Result<openany_client::Art, String> {
    match openany_client::Art::aus(art) {
        a @ (openany_client::Art::Notiz
        | openany_client::Art::Kalender
        | openany_client::Art::Termin
        | openany_client::Art::Kontakt) => Ok(a),
        _ => Err("The trash here does not know this kind.".into()),
    }
}

#[tauri::command]
async fn papierkorb_liste(
    zustand: tauri::State<'_, Arc<Zustand>>,
    seite: Option<usize>,
) -> Result<PapierkorbSeite, String> {
    let speicher = zustand.speicher.lock().await;
    let seite = seite.unwrap_or(1).max(1);
    let (liste, weitere) = speicher
        .papierkorb_seite(seite)
        .map_err(|e| e.to_string())?;

    Ok(PapierkorbSeite {
        items: liste
            .into_iter()
            .map(|w| WeggelegtAnzeige {
                art: w.art,
                id: w.schluessel,
                name: w.name,
                context: w.kontext,
                deleted_at: w.seit,
            })
            .collect(),
        next_page: weitere.then_some(seite + 1),
    })
}

#[tauri::command]
async fn papierkorb_zurueck(
    zustand: tauri::State<'_, Arc<Zustand>>,
    art: String,
    id: String,
) -> Result<(), String> {
    if matches!(art.as_str(), "file" | "document") {
        return dateibefehle::wiederherstellen(&zustand, &id).await;
    }
    if matches!(art.as_str(), "album" | "album_image" | "gallery_image") {
        return galeriebefehle::wiederherstellen(&zustand, &art, &id).await;
    }
    let art = papierkorb_art(&art)?;
    let speicher = zustand.speicher.lock().await;

    speicher
        .wiederherstellen(&art, &id, Protokoll::Merken)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Endgueltig -- samt Grabstein im Protokoll, damit die Gegenseite es nicht
/// zurueckspielt. Ein Kontaktfoto geht mit.
#[tauri::command]
async fn papierkorb_endgueltig(
    zustand: tauri::State<'_, Arc<Zustand>>,
    art: String,
    id: String,
) -> Result<(), String> {
    if matches!(art.as_str(), "file" | "document") {
        return dateibefehle::endgueltig(&zustand, &id).await;
    }
    if matches!(art.as_str(), "album" | "album_image" | "gallery_image") {
        return galeriebefehle::endgueltig(&zustand, &art, &id).await;
    }
    let art = papierkorb_art(&art)?;
    let speicher = zustand.speicher.lock().await;

    if art == openany_client::Art::Kontakt {
        speicher
            .kontaktfoto_uebernehmen(&id, None, None, Protokoll::Still)
            .map_err(|e| e.to_string())?;
    }

    speicher
        .loeschen(&art, &id, Protokoll::Merken)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/*
 * Die Startseite -- dieselben Kacheln wie in der Webapp
 * (`packages/oberflaeche/start/HomeDashboard.vue`), mit der Antwortform von
 * /api/dashboard, aber nur fuer das, was dieses Programm traegt: Termine der
 * laufenden Woche, zuletzt bearbeitete Notizen, Zahl der Kontakte.
 */

#[derive(Serialize)]
pub struct StartKalender {
    id: String,
    color: String,
}

#[derive(Serialize)]
pub struct StartNotiz {
    id: String,
    title: String,
    updated_at: String,
}

#[derive(Serialize)]
pub struct StartKontakte {
    total: usize,
}

#[derive(Serialize)]
pub struct StartUebersicht {
    events: Vec<TerminAnzeige>,
    calendars: Vec<StartKalender>,
    notes: Vec<StartNotiz>,
    contacts: StartKontakte,
    files: Vec<StartDatei>,
    storage: StartSpeicher,
}

#[derive(Serialize)]
pub struct StartDatei {
    id: String,
    name: String,
    zone: String,
    size: u64,
}

/// Belegt heißt hier: was die Inhaltsablage auf DIESEM Gerät braucht. Ein
/// Kontingent gibt es ohne Server nicht (`quota` null = unbegrenzt).
#[derive(Serialize)]
pub struct StartSpeicher {
    used: u64,
    quota: Option<u64>,
}

#[tauri::command]
async fn start_uebersicht(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<StartUebersicht, String> {
    use chrono::Datelike;

    let speicher = zustand.speicher.lock().await;

    // Die laufende Woche, Montag bis Sonntag -- wie die Kachel sie zeigt.
    let heute = chrono::Local::now().date_naive();
    let montag = heute - chrono::Duration::days(i64::from(heute.weekday().num_days_from_monday()));
    let sonntag = montag + chrono::Duration::days(6);

    let kalender = speicher.kalender_alle().map_err(|e| e.to_string())?;
    let farben: std::collections::HashMap<String, String> = kalender
        .iter()
        .map(|k| (k.uuid.clone(), k.farbe.clone()))
        .collect();

    let events = speicher
        .termine_im_fenster(&montag.to_string(), &sonntag.to_string())
        .map_err(|e| e.to_string())?
        .into_iter()
        .filter_map(|t| {
            let farbe = farben.get(&t.kalender_uuid)?.clone();
            Some(termin_anzeige(t, &farbe))
        })
        .collect();

    // `notizen()` liefert schon nach Aenderung absteigend.
    let notes = speicher
        .notizen()
        .map_err(|e| e.to_string())?
        .into_iter()
        .take(5)
        .map(|n| StartNotiz {
            id: n.zk_id,
            title: n.titel,
            updated_at: n.geaendert_at,
        })
        .collect();

    let total = speicher.kontakte().map_err(|e| e.to_string())?.len();

    let files = speicher
        .zuletzt_geaenderte_dateien(5)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|d| StartDatei {
            id: d.uuid,
            name: d.name,
            zone: d.zone,
            size: d.groesse,
        })
        .collect();

    Ok(StartUebersicht {
        events,
        calendars: kalender
            .into_iter()
            .map(|k| StartKalender {
                id: k.uuid,
                color: k.farbe,
            })
            .collect(),
        notes,
        contacts: StartKontakte { total },
        files,
        storage: StartSpeicher {
            used: zustand.inhalte.belegt(),
            quota: None,
        },
    })
}

/*
 * Geraete in der Naehe -- APK 0.1: finden (Schritt 3), paaren (Schritt 4),
 * abgleichen (Schritt 5).
 */

/// Den laufenden Nahbereich beenden und mit dem aktuellen Namen neu starten.
///
/// Der alte gibt seinen Port erst frei, wenn sein Server wirklich steht --
/// deshalb ein paar Versuche statt eines "Adresse belegt".
async fn nah_neu_starten(zustand: Arc<Zustand>) {
    zustand.nah.lock().await.take();
    *zustand.nah_fehler.lock().await = None;

    for _ in 0..10 {
        nah_starten(zustand.clone()).await;
        if zustand.nah.lock().await.is_some() {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    }
}

async fn nah_starten(zustand: Arc<Zustand>) {
    let name = zustand.einstellungen.lock().await.name();

    // Die Person hinter diesem Geraet: beim ersten Start angelegt, danach
    // mit dem Geraetenamen nachgefuehrt (person.rs).
    if let Ok(ident) = geraet_identitaet(&zustand) {
        zustand.gastgeber.person_anlegen(&ident, &name);
    }

    let ergebnis = match geraet_identitaet(&zustand) {
        Ok(ident) => openany_nahbereich::Nahbereich::starten(&ident, &name)
            .await
            .map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    };

    match ergebnis {
        Ok(nah) => *zustand.nah.lock().await = Some(nah),
        Err(fehler) => *zustand.nah_fehler.lock().await = Some(fehler),
    }

    // Der Dienst haengt nicht am Namen; einmal gestartet, laeuft er weiter.
    if zustand.nah_dienst.lock().await.is_none() {
        if let Ok(ident) = geraet_identitaet(&zustand) {
            match openany_nahbereich::Dienst::starten(
                &ident,
                openany_nahbereich::DIENST_PORT,
                zustand.paarungen.clone(),
                zustand.gastgeber.clone(),
            )
            .await
            {
                Ok(dienst) => *zustand.nah_dienst.lock().await = Some(dienst),
                Err(fehler) => *zustand.nah_fehler.lock().await = Some(fehler),
            }
        }
    }
}

/*
 * Wer gepaart ist -- eine kleine Datei im App-Ordner, nicht die SQLite.
 *
 * Paarungen reisen nicht mit dem Abgleich (jedes Geraet kennt seine eigenen),
 * und der Dienst fragt sie mitten im TLS-Gespraech ab, ohne auf die Sperre der
 * Datenbank warten zu duerfen.
 */
pub struct NahGastgeber {
    pfad: PathBuf,
    name: std::sync::Mutex<String>,
    gepaart: std::sync::Mutex<std::collections::BTreeMap<String, String>>,
    qr: std::sync::Mutex<Option<String>>,
    speicher: openany_nahbereich::GemeinsamerSpeicher,
    inhalte: Arc<openany_store::Inhalte>,
    /// Wie oft ein anderes Geraet hier etwas geaendert hat. Die Oberflaeche
    /// vergleicht nur, ob die Zahl gewachsen ist.
    angenommen: std::sync::atomic::AtomicU64,
    /// Die Person hinter diesem Geraet (`nah/person.json`). Darin steht
    /// nichts Geheimes -- Zertifikate und Unterschriften sind zum Zeigen da.
    person: std::sync::Mutex<Option<openany_nahbereich::Person>>,
    /// Offene Einladungen in lokale Projekte, in beide Richtungen.
    einladungen: openany_nahbereich::einladen::GemeinsameEinladungen,
    /// Zaehler: Sind Chat-Nachrichten angekommen? Die Oberflaeche vergleicht
    /// nur, ob er gewachsen ist (eigener Zaehler: ein Chat soll die offene
    /// Ansicht nicht neu aufbauen, nur nachlesen).
    pub(crate) chat: std::sync::atomic::AtomicU64,
    /// Wer gerade eine Notiz bearbeitet, die dieses Geraet zum Bearbeiten
    /// freigibt (sperren.rs) -- ein Mitglied vor Ort, oder diese Person
    /// selbst.
    pub(crate) sperren: openany_nahbereich::sperren::GemeinsameSperren,
    /// Die Personen ANDERER Menschen, die mit diesem Geraet ein Projekt
    /// teilen (`nah/personen.json`, nach Personen-Id).
    fremde: std::sync::Mutex<std::collections::BTreeMap<String, openany_nahbereich::Person>>,
    /// Direktnachrichten vor Ort: Absender, Blockierte, Postausgang
    /// (direktbefehle.rs).
    pub(crate) direkt: direktbefehle::DirektStand,
}

impl NahGastgeber {
    fn laden(
        pfad: &std::path::Path,
        name: String,
        speicher: openany_nahbereich::GemeinsamerSpeicher,
        inhalte: Arc<openany_store::Inhalte>,
    ) -> Self {
        let gepaart = std::fs::read_to_string(pfad)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default();
        let person = std::fs::read_to_string(pfad.with_file_name("person.json"))
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok());
        let fremde = std::fs::read_to_string(pfad.with_file_name("personen.json"))
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default();
        Self {
            person: std::sync::Mutex::new(person),
            einladungen: Default::default(),
            chat: Default::default(),
            sperren: Default::default(),
            fremde: std::sync::Mutex::new(fremde),
            direkt: direktbefehle::DirektStand::laden(
                pfad.parent().unwrap_or(std::path::Path::new(".")),
            ),
            pfad: pfad.to_path_buf(),
            name: std::sync::Mutex::new(name),
            gepaart: std::sync::Mutex::new(gepaart),
            qr: std::sync::Mutex::new(None),
            speicher,
            inhalte,
            angenommen: Default::default(),
        }
    }

    fn name_setzen(&self, name: String) {
        if let Ok(mut n) = self.name.lock() {
            *n = name;
        }
    }

    /// Beim ersten Start die Person anlegen; danach den Geraetenamen in ihr
    /// nachfuehren (ein neuer Eintrag nur, wenn er sich geaendert hat).
    fn person_anlegen(&self, ident: &openany_nahbereich::Identitaet, geraetename: &str) {
        use openany_nahbereich::Gastgeber;
        let neu = match self.person() {
            Some(mut p) => {
                let vorher = p.eintraege.len();
                if p.umbenennen(ident, geraetename).is_err() || p.eintraege.len() == vorher {
                    return;
                }
                p
            }
            None => match openany_nahbereich::Person::neu(ident, geraetename) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Keine Person angelegt: {e}");
                    return;
                }
            },
        };
        self.person_merken(neu);
    }

    /// Eine bekannte Person: die eigene oder eine fremde, auch unter einer
    /// frueheren Id.
    /// Die Personen anderer Menschen, die dieses Geraet kennt: Id -> Name.
    pub(crate) fn fremde_personen(&self) -> std::collections::BTreeMap<String, String> {
        self.fremde
            .lock()
            .map(|f| {
                f.iter()
                    .map(|(id, p)| (id.clone(), direktbefehle::anzeigename(p)))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub(crate) fn bekannte_person(&self, id: &str) -> Option<openany_nahbereich::Person> {
        use openany_nahbereich::Gastgeber;
        let passt = |p: &openany_nahbereich::Person| {
            p.personen_id == id || p.frueher.iter().any(|f| f == id)
        };
        if let Some(p) = self.person().filter(passt) {
            return Some(p);
        }
        self.fremde
            .lock()
            .ok()?
            .values()
            .find(|p| passt(p))
            .cloned()
    }

    fn person_name(&self, name: &str) {
        use openany_nahbereich::Gastgeber;
        if let Some(mut p) = self.person() {
            p.name = name.to_string();
            self.person_merken(p);
        }
    }

    fn schreiben(&self, liste: &std::collections::BTreeMap<String, String>) {
        if let Some(ordner) = self.pfad.parent() {
            let _ = std::fs::create_dir_all(ordner);
        }
        let _ = std::fs::write(
            &self.pfad,
            serde_json::to_string_pretty(liste).unwrap_or_default(),
        );
    }

    fn gepaarte(&self) -> Vec<(String, String)> {
        self.gepaart
            .lock()
            .map(|l| l.iter().map(|(f, n)| (f.clone(), n.clone())).collect())
            .unwrap_or_default()
    }

    fn vergessen(&self, fp: &str) {
        if let Ok(mut l) = self.gepaart.lock() {
            l.remove(fp);
            self.schreiben(&l);
        }
    }
}

impl openany_nahbereich::Gastgeber for NahGastgeber {
    fn mein_name(&self) -> String {
        self.name.lock().map(|n| n.clone()).unwrap_or_default()
    }
    fn qr_geheimnis(&self) -> Option<String> {
        self.qr.lock().ok().and_then(|q| q.clone())
    }
    fn gepaart(&self, fingerabdruck: &str, name: &str) {
        if let Ok(mut l) = self.gepaart.lock() {
            l.insert(fingerabdruck.to_string(), name.to_string());
            self.schreiben(&l);
        }
    }
    fn ist_gepaart(&self, fingerabdruck: &str) -> bool {
        self.gepaart
            .lock()
            .map(|l| l.contains_key(fingerabdruck))
            .unwrap_or(false)
    }
    fn speicher(&self) -> Option<openany_nahbereich::GemeinsamerSpeicher> {
        Some(self.speicher.clone())
    }
    fn inhalte(&self) -> Option<Arc<openany_store::Inhalte>> {
        Some(self.inhalte.clone())
    }
    fn person(&self) -> Option<openany_nahbereich::Person> {
        self.person.lock().ok().and_then(|p| p.clone())
    }
    fn person_merken(&self, person: openany_nahbereich::Person) {
        let _ = std::fs::write(
            self.pfad.with_file_name("person.json"),
            serde_json::to_string_pretty(&person).unwrap_or_default(),
        );
        if let Ok(mut p) = self.person.lock() {
            *p = Some(person);
        }
    }
    fn einladungen(&self) -> Option<openany_nahbereich::einladen::GemeinsameEinladungen> {
        Some(self.einladungen.clone())
    }
    fn person_fremd_merken(&self, person: openany_nahbereich::Person) {
        if let Ok(mut f) = self.fremde.lock() {
            match f.get_mut(&person.personen_id) {
                Some(da) => da.zusammenlegen(&person),
                None => {
                    f.insert(person.personen_id.clone(), person);
                }
            }
            let _ = std::fs::write(
                self.pfad.with_file_name("personen.json"),
                serde_json::to_string_pretty(&*f).unwrap_or_default(),
            );
        }
    }
    fn projekt_aufgenommen(&self) {
        self.angenommen();
    }
    fn sperren(&self) -> Option<openany_nahbereich::sperren::GemeinsameSperren> {
        Some(self.sperren.clone())
    }
    fn chat_angekommen(&self) {
        self.chat.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    fn direkt_annehmen(
        &self,
        n: &openany_nahbereich::direkt::DirektNachricht,
        anrufer: &str,
    ) -> Result<(), String> {
        direktbefehle::annehmen(self, n, anrufer)
    }
    fn kontakt_bestaetigt(&self, person: openany_nahbereich::Person, geraet: &str) {
        direktbefehle::kontakt_merken(self, &person, geraet);
    }
    fn direkt_angekommen(&self) {
        self.direkt
            .zaehler
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    fn person_von(&self, id: &str) -> Option<openany_nahbereich::Person> {
        self.bekannte_person(id)
    }
    fn besorger(&self) -> Option<Arc<dyn openany_nahbereich::dienst::Besorger>> {
        Some(Arc::new(Besorgen))
    }
    fn angenommen(&self) {
        self.angenommen
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}

#[derive(Serialize)]
pub struct GeraetAnzeige {
    #[serde(flatten)]
    geraet: openany_nahbereich::GeraetInDerNaehe,
    gepaart: bool,
}

#[derive(Serialize)]
pub struct GepaartAnzeige {
    fingerabdruck: String,
    name: String,
    /// Gerade in der Naehe?
    da: bool,
    /// Wann zuletzt abgeglichen, und ob es scheiterte. `None`, solange noch
    /// nie -- oder solange gerade ein Lauf den Speicher haelt.
    letzter_lauf: Option<String>,
    letzter_fehler: Option<String>,
}

#[derive(Serialize)]
pub struct NahLage {
    laeuft: bool,
    fehler: Option<String>,
    name: String,
    fingerabdruck: Option<String>,
    geraete: Vec<GeraetAnzeige>,
    gepaarte: Vec<GepaartAnzeige>,
    offen: Vec<openany_nahbereich::paaren::OffenePaarung>,
    /// Zaehler: Hat ein anderes Geraet hier etwas geaendert?
    angenommen: u64,
    /// Die Person hinter diesem Geraet und ihre Geraete.
    person: Option<PersonAnzeige>,
    /// Offene Einladungen in lokale Projekte, in beide Richtungen.
    einladungen: Vec<openany_nahbereich::einladen::OffeneEinladung>,
    /// Zaehler: neue Chat-Nachrichten.
    chat: u64,
    /// Zaehler: Direktnachrichten angekommen oder aus dem Postausgang hinaus.
    direkt: u64,
}

pub(crate) async fn ident_laden(
    zustand: &Zustand,
) -> Result<openany_nahbereich::Identitaet, String> {
    geraet_identitaet(zustand)
}

/*
 * DIE IDENTITAET DIESES GERAETS IM TRESOR (01.10.2026). Ihr Schluessel macht
 * aus dem Geraet ein gepaartes Geraet und unterschreibt fuer die Person
 * (person.rs) -- er gehoert also genauso verschlossen wie die drei Ausweise
 * (tresor.rs). Eine alte Klartext-Datei liest die Tresorablage einmal und
 * verschliesst sie gleich: Paarungen bleiben erhalten.
 *
 * Laesst sich der Tresor nicht mehr oeffnen (Schluessel fort), entsteht eine
 * neue Identitaet -- fuer andere Geraete ein neues Geraet, das sich neu paart.
 * Ein FEHLER beim Lesen dagegen erzeugt nichts: Er ueberschriebe sonst eine
 * Identitaet, die beim naechsten Versuch wieder lesbar waere.
 */
pub(crate) fn geraet_identitaet(
    zustand: &Zustand,
) -> Result<openany_nahbereich::Identitaet, String> {
    if let Some(i) = zustand.identitaet.get() {
        return Ok(i.clone());
    }
    let ablage = tresor::ablage(zustand.ordner.join("nah").join("identitaet.json"));
    let ident = match ablage.lesen().map_err(|e| e.to_string())? {
        Some(json) => serde_json::from_str(&json).map_err(|e| e.to_string())?,
        None => {
            let neu = openany_nahbereich::Identitaet::erzeugen().map_err(|e| e.to_string())?;
            ablage
                .schreiben(&serde_json::to_string(&neu).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
            neu
        }
    };
    Ok(zustand.identitaet.get_or_init(|| ident).clone())
}

/// Wer dieses Geraet ist -- als Person (person.rs).
#[derive(Serialize)]
pub struct PersonAnzeige {
    /// Die ersten acht Zeichen, zum Wiedererkennen.
    personen_id: String,
    name: String,
    geraete: Vec<openany_nahbereich::Geraet>,
}

/// Mit einem gepaarten eigenen Geraet die Person austauschen. Fehler sind
/// hier kein Grund, etwas anderes scheitern zu lassen: Beim naechsten
/// Abgleich wird es wieder versucht.
async fn person_tauschen(zustand: &Zustand, adresse: &str, fp: &str) {
    let Ok(ident) = geraet_identitaet(zustand) else {
        return;
    };
    if let Err(e) = openany_nahbereich::anruf::person_tauschen(
        &ident,
        zustand.gastgeber.as_ref(),
        adresse,
        openany_nahbereich::DIENST_PORT,
        fp,
    )
    .await
    {
        eprintln!("Person not exchanged with {fp}: {e}");
    }
}

/*
 * ORIGINALE FUER MITGLIEDER BESORGEN (01.10.2026). Fragt ein Mitglied eines
 * lokalen Projekts nach einem Original, das hier nur „bei Bedarf" liegt,
 * holt dieses Geraet es auf demselben Weg wie beim eigenen Oeffnen -- vom
 * Server oder einem gepaarten Geraet (`dateibefehle::von_geraeten_holen`).
 * Je Abdruck nur einmal gleichzeitig: Das Mitglied fragt alle drei Sekunden
 * nach, bis es da ist.
 */
struct Besorgen;

static BESORGT_GERADE: std::sync::Mutex<std::collections::BTreeSet<String>> =
    std::sync::Mutex::new(std::collections::BTreeSet::new());

#[async_trait::async_trait]
impl openany_nahbereich::dienst::Besorger for Besorgen {
    async fn besorgen(&self, abdruck: String, groesse: u64) -> Result<(), String> {
        // Vorschaubilder (Groesse unbekannt) reisen ohnehin immer mit.
        if groesse == u64::MAX {
            return Err("Unknown size.".into());
        }
        let zustand = GETEILT.get().cloned().ok_or("Not started yet.")?;
        if !BESORGT_GERADE
            .lock()
            .map_err(|_| "gesperrt".to_string())?
            .insert(abdruck.clone())
        {
            return Ok(());
        }
        let ergebnis = dateibefehle::von_geraeten_holen(&zustand, &abdruck, groesse).await;
        if let Ok(mut l) = BESORGT_GERADE.lock() {
            l.remove(&abdruck);
        }
        ergebnis
    }
}

/* ── Einladung vor Ort (lokale Mitgliedschaften, Schritt 3) ─────────────── */

/// Die Mitgliederliste eines LOKALEN Projekts aus dem Speicher.
async fn lokale_liste(
    zustand: &Zustand,
    projekt: &str,
) -> Result<(openany_store::Projekt, openany_nahbereich::Mitgliederliste), String> {
    let p = zustand
        .speicher
        .lock()
        .await
        .projekt(projekt)
        .map_err(|e| e.to_string())?
        .ok_or("This project does not exist here.")?;
    let json = p
        .mitgliederliste
        .clone()
        .ok_or("Only local projects take invitations nearby.")?;
    let liste = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    Ok((p, liste))
}

/// Ein Geraet in der Naehe in ein lokales Projekt einladen. Gibt den Code
/// zurueck; das Geraet drueben zeigt dieselben Ziffern.
#[tauri::command]
async fn nah_einladen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    projekt: String,
    fingerabdruck: String,
) -> Result<String, String> {
    use openany_nahbereich::Gastgeber;
    let g = &zustand.gastgeber;
    let ich = g.person().ok_or("This device has no person yet.")?;
    let (p, liste) = lokale_liste(&zustand, &projekt).await?;
    let mitglieder = liste
        .mitglieder(&|id| g.bekannte_person(id))
        .map_err(|e| e.to_string())?;
    let bin_eigentuemer = mitglieder.iter().any(|m| {
        m.rolle == openany_nahbereich::mitglieder::EIGENTUEMER
            && (m.personen_id == ich.personen_id || ich.frueher.contains(&m.personen_id))
    });
    if !bin_eigentuemer {
        return Err("Only the owner can invite.".into());
    }
    let ident = ident_laden(&zustand).await?;
    let adresse = adresse_von(&zustand, &fingerabdruck).await?;
    let anfrage = openany_nahbereich::einladen::Anfrage {
        projekt: p.uuid,
        projekt_name: p.name,
        von: if ich.name.trim().is_empty() {
            g.mein_name()
        } else {
            ich.name
        },
        geraet: g.mein_name(),
        zufall: uuid::Uuid::new_v4().simple().to_string(),
    };
    openany_nahbereich::anruf::einladen(
        &ident,
        &g.einladungen,
        &adresse,
        openany_nahbereich::DIENST_PORT,
        &fingerabdruck,
        &anfrage,
    )
    .await
}

/// "Der Code passt" bei einer Einladung -- auf welcher Seite auch immer.
/// `true`: fertig (nur auf der einladenden Seite moeglich).
#[tauri::command]
async fn nah_einladung_bestaetigen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    fingerabdruck: String,
) -> Result<bool, String> {
    let rolle = zustand
        .gastgeber
        .einladungen
        .lock()
        .map_err(|_| "Invitation not possible.".to_string())?
        .hier_bestaetigen(&fingerabdruck)
        .map(|e| e.rolle)
        .ok_or("This invitation has expired.")?;
    match rolle {
        openany_nahbereich::paaren::Rolle::Gefragt => Ok(false),
        openany_nahbereich::paaren::Rolle::Anfragend => {
            einladung_abschliessen(&zustand, &fingerabdruck).await
        }
    }
}

#[tauri::command]
async fn nah_einladung_abbrechen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    fingerabdruck: String,
) -> Result<(), String> {
    let g = &zustand.gastgeber;
    if let (Ok(ident), Ok(adresse)) = (
        ident_laden(&zustand).await,
        adresse_von(&zustand, &fingerabdruck).await,
    ) {
        openany_nahbereich::anruf::einladung_abbrechen(
            &ident,
            &g.einladungen,
            &adresse,
            openany_nahbereich::DIENST_PORT,
            &fingerabdruck,
        )
        .await;
    }
    if let Ok(mut e) = g.einladungen.lock() {
        e.entfernen(&fingerabdruck);
    }
    Ok(())
}

/// Die einladende Seite: Hat drueben jemand bestaetigt? Dann den Beitritt
/// unterschreiben, hier speichern und hinueber schicken. `true`: fertig.
async fn einladung_abschliessen(zustand: &Zustand, fp: &str) -> Result<bool, String> {
    use openany_nahbereich::Gastgeber;
    let g = &zustand.gastgeber;
    let Some(offen) = g
        .einladungen
        .lock()
        .ok()
        .and_then(|mut e| e.get(fp).cloned())
    else {
        return Ok(false);
    };
    if !offen.hier_bestaetigt {
        return Ok(false);
    }
    // Ein Kontakt statt eines Projekts (direktbefehle.rs).
    if offen.projekt == openany_nahbereich::einladen::KONTAKT {
        return direktbefehle::kontakt_abschliessen(zustand, fp, &offen).await;
    }
    let ident = ident_laden(zustand).await?;
    let adresse = adresse_von(zustand, fp).await?;
    if !offen.dort_bestaetigt
        && !openany_nahbereich::anruf::einladung_nachsehen(
            &ident,
            &g.einladungen,
            &adresse,
            openany_nahbereich::DIENST_PORT,
            fp,
        )
        .await?
    {
        return Ok(false);
    }
    let Some(person_b) = g
        .einladungen
        .lock()
        .ok()
        .and_then(|mut e| e.get(fp)?.person.clone())
    else {
        return Ok(false);
    };
    let ich = g.person().ok_or("This device has no person yet.")?;
    let (p, mut liste) = lokale_liste(zustand, &offen.projekt).await?;
    let name_b = if person_b.name.trim().is_empty() {
        offen.name.clone()
    } else {
        person_b.name.clone()
    };
    liste
        .eintragen(
            &ident,
            "beitritt",
            &person_b.personen_id,
            &name_b,
            openany_nahbereich::mitglieder::MITGLIED,
            fp,
        )
        .map_err(|e| e.to_string())?;
    // Gilt die Liste mit dem neuen Eintrag? Sonst nichts speichern, nichts senden.
    g.person_fremd_merken(person_b);
    liste
        .mitglieder(&|id| g.bekannte_person(id))
        .map_err(|e| e.to_string())?;
    openany_nahbereich::anruf::aufnahme_senden(
        &ident,
        &adresse,
        openany_nahbereich::DIENST_PORT,
        fp,
        &openany_nahbereich::einladen::Aufnahme {
            projekt_name: p.name.clone(),
            mitgliederliste: liste.clone(),
            eigentuemer: ich,
        },
    )
    .await?;
    zustand
        .speicher
        .lock()
        .await
        .projekt_lokal_schreiben(
            &p,
            &serde_json::to_string(&liste).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
    if let Ok(mut e) = g.einladungen.lock() {
        e.entfernen(fp);
    }
    Ok(true)
}

/// Wie diese Person heisst -- fuer spaetere Einladungen (gilt auf allen
/// eigenen Geraeten, sobald sie sich abgleichen).
#[tauri::command]
async fn nah_person_name(
    zustand: tauri::State<'_, Arc<Zustand>>,
    name: String,
) -> Result<(), String> {
    zustand.gastgeber.person_name(name.trim());
    Ok(())
}

/// Fuer die Projektbefehle: wo ist dieses Geraet gerade (auch die letzte
/// bekannte Adresse)?
pub(crate) async fn adresse_von_geraet(zustand: &Zustand, fp: &str) -> Result<String, String> {
    match adresse_von(zustand, fp).await {
        Ok(a) => Ok(a),
        Err(_) => adresse_zum_abgleichen(zustand, fp).await,
    }
}

/// Wo ist dieses Geraet gerade? Aus der Liste der Gefundenen.
pub(crate) async fn adresse_von(zustand: &Zustand, fp: &str) -> Result<String, String> {
    zustand
        .nah
        .lock()
        .await
        .as_ref()
        .and_then(|n| n.geraete().into_iter().find(|g| g.fingerabdruck == fp))
        .map(|g| g.adresse)
        .ok_or_else(|| "This device is not nearby right now.".to_string())
}

/// Fuer den Abgleich: auch die letzte bekannte Adresse, wenn das Geraet aus
/// der Liste gefallen ist. Ist gar keine bekannt, wird gesucht und kurz
/// gewartet.
async fn adresse_zum_abgleichen(zustand: &Zustand, fp: &str) -> Result<String, String> {
    if let Some(a) = zustand
        .nah
        .lock()
        .await
        .as_ref()
        .and_then(|n| n.letzte_adresse(fp))
    {
        return Ok(a);
    }
    if let Some(n) = zustand.nah.lock().await.as_ref() {
        n.suchen();
    }
    tokio::time::sleep(std::time::Duration::from_secs(4)).await;
    zustand
        .nah
        .lock()
        .await
        .as_ref()
        .and_then(|n| n.letzte_adresse(fp))
        .ok_or_else(|| "Not found. Is openany open there and the device unlocked?".to_string())
}

#[tauri::command]
async fn nah_lage(zustand: tauri::State<'_, Arc<Zustand>>) -> Result<NahLage, String> {
    let name = zustand.einstellungen.lock().await.name();
    let nah = zustand.nah.lock().await;
    let ident = geraet_identitaet(&zustand).ok();
    let fingerabdruck = ident.as_ref().map(|i| i.kurz());

    let gefunden = nah.as_ref().map(|n| n.geraete()).unwrap_or_default();
    drop(nah);
    // Wartende Direktnachrichten, deren Geraet jetzt da ist (direktbefehle.rs).
    direktbefehle::postausgang_leeren(
        zustand.inner().clone(),
        gefunden.iter().filter(|d| d.openany).cloned().collect(),
    );
    let g = &zustand.gastgeber;

    // Wer angefragt und hier schon bestaetigt hat, schaut nach, ob drueben
    // auch -- bei jedem Blick der Oberflaeche, also alle paar Sekunden.
    let offen = zustand
        .paarungen
        .lock()
        .map(|mut p| p.alle())
        .unwrap_or_default();
    if let Ok(ident) = ident_laden(&zustand).await {
        for o in offen.iter().filter(|o| {
            o.rolle == openany_nahbereich::paaren::Rolle::Anfragend && o.hier_bestaetigt
        }) {
            if let Some(ziel) = gefunden.iter().find(|x| x.fingerabdruck == o.fingerabdruck) {
                let fertig = openany_nahbereich::anruf::nachsehen(
                    &ident,
                    &zustand.paarungen,
                    g.as_ref(),
                    &ziel.adresse,
                    openany_nahbereich::DIENST_PORT,
                    &o.fingerabdruck,
                )
                .await;
                if fertig == Ok(true) {
                    person_tauschen(&zustand, &ziel.adresse, &o.fingerabdruck).await;
                }
            }
        }
    }

    // Einladungen, die hier bestaetigt sind und auf das andere Geraet warten:
    // nachsehen und, wenn es soweit ist, abschliessen (einladen.rs).
    let wartende: Vec<String> = g
        .einladungen
        .lock()
        .map(|mut e| {
            e.alle()
                .into_iter()
                .filter(|o| {
                    o.rolle == openany_nahbereich::paaren::Rolle::Anfragend && o.hier_bestaetigt
                })
                .map(|o| o.fingerabdruck)
                .collect()
        })
        .unwrap_or_default();
    for fp in wartende {
        if let Err(e) = einladung_abschliessen(&zustand, &fp).await {
            eprintln!("Einladung an {fp}: {e}");
        }
    }

    use openany_nahbereich::Gastgeber;
    // `try_lock` und nicht warten: Diese Frage kommt alle drei Sekunden, und
    // ein laufender Abgleich haelt den Speicher womoeglich eine Minute.
    let marken: std::collections::BTreeMap<String, openany_store::Marke> =
        match zustand.speicher.try_lock() {
            Ok(speicher) => g
                .gepaarte()
                .into_iter()
                .filter_map(|(fp, _)| {
                    speicher
                        .marke(&openany_nahbereich::basis_fuer(&fp))
                        .ok()
                        .map(|m| (fp, m))
                })
                .collect(),
            Err(_) => Default::default(),
        };
    Ok(NahLage {
        laeuft: zustand.nah.lock().await.is_some(),
        fehler: zustand.nah_fehler.lock().await.clone(),
        name,
        fingerabdruck,
        gepaarte: g
            .gepaarte()
            .into_iter()
            .map(|(fp, n)| GepaartAnzeige {
                da: gefunden.iter().any(|x| x.fingerabdruck == fp),
                letzter_lauf: marken.get(&fp).and_then(|m| m.letzter_lauf.clone()),
                letzter_fehler: marken.get(&fp).and_then(|m| m.letzter_fehler.clone()),
                fingerabdruck: fp,
                name: n,
            })
            .collect(),
        geraete: gefunden
            .into_iter()
            .map(|geraet| GeraetAnzeige {
                gepaart: g.ist_gepaart(&geraet.fingerabdruck),
                geraet,
            })
            .collect(),
        offen: zustand
            .paarungen
            .lock()
            .map(|mut p| p.alle())
            .unwrap_or_default(),
        angenommen: g.angenommen.load(std::sync::atomic::Ordering::Relaxed),
        einladungen: g
            .einladungen
            .lock()
            .map(|mut e| e.alle())
            .unwrap_or_default(),
        chat: g.chat.load(std::sync::atomic::Ordering::Relaxed),
        direkt: g.direkt.zaehler.load(std::sync::atomic::Ordering::Relaxed),
        person: g.person().map(|p| PersonAnzeige {
            personen_id: p.personen_id.chars().take(8).collect(),
            name: p.name.clone(),
            geraete: ident
                .as_ref()
                .map(|i| p.geraete(&[&i.fingerabdruck]))
                .unwrap_or_default(),
        }),
    })
}

/// Eine Paarung mit einem gefundenen Geraet anfragen; gibt den Code zurueck.
#[tauri::command]
async fn nah_paaren(
    zustand: tauri::State<'_, Arc<Zustand>>,
    fingerabdruck: String,
) -> Result<String, String> {
    use openany_nahbereich::Gastgeber;
    let ident = ident_laden(&zustand).await?;
    let adresse = adresse_von(&zustand, &fingerabdruck).await?;
    openany_nahbereich::anruf::anfragen(
        &ident,
        &zustand.paarungen,
        &adresse,
        openany_nahbereich::DIENST_PORT,
        &fingerabdruck,
        &zustand.gastgeber.mein_name(),
    )
    .await
}

/// "Der Code passt" -- auf welcher Seite auch immer.
#[tauri::command]
async fn nah_bestaetigen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    fingerabdruck: String,
) -> Result<bool, String> {
    use openany_nahbereich::paaren::Rolle;
    let rolle = zustand
        .paarungen
        .lock()
        .map_err(|_| "Pairing not possible.".to_string())?
        .get(&fingerabdruck)
        .map(|o| o.rolle)
        .ok_or_else(|| "This pairing has expired.".to_string())?;

    match rolle {
        Rolle::Anfragend => {
            let ident = ident_laden(&zustand).await?;
            let adresse = adresse_von(&zustand, &fingerabdruck).await?;
            let fertig = openany_nahbereich::anruf::bestaetigen(
                &ident,
                &zustand.paarungen,
                zustand.gastgeber.as_ref(),
                &adresse,
                openany_nahbereich::DIENST_PORT,
                &fingerabdruck,
            )
            .await?;
            if fertig {
                person_tauschen(&zustand, &adresse, &fingerabdruck).await;
            }
            Ok(fertig)
        }
        Rolle::Gefragt => {
            if let Ok(mut p) = zustand.paarungen.lock() {
                p.hier_bestaetigen(&fingerabdruck);
            }
            Ok(openany_nahbereich::dienst::abschliessen(
                &zustand.paarungen,
                zustand.gastgeber.as_ref(),
                &fingerabdruck,
            )
            .is_some())
        }
    }
}

#[tauri::command]
async fn nah_abbrechen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    fingerabdruck: String,
) -> Result<(), String> {
    let rolle = zustand
        .paarungen
        .lock()
        .ok()
        .and_then(|mut p| p.get(&fingerabdruck).map(|o| o.rolle));
    if rolle == Some(openany_nahbereich::paaren::Rolle::Anfragend) {
        if let (Ok(ident), Ok(adresse)) = (
            ident_laden(&zustand).await,
            adresse_von(&zustand, &fingerabdruck).await,
        ) {
            openany_nahbereich::anruf::abbrechen(
                &ident,
                &zustand.paarungen,
                &adresse,
                openany_nahbereich::DIENST_PORT,
                &fingerabdruck,
            )
            .await;
            return Ok(());
        }
    }
    if let Ok(mut p) = zustand.paarungen.lock() {
        p.abbrechen(&fingerabdruck);
    }
    Ok(())
}

/// Mit einem gepaarten Geraet abgleichen -- APK 0.1, Schritt 5.
///
/// Derselbe Laeufer wie gegen openany; die Gegenstelle ist das andere Geraet.
/// Es muss gerade in der Naehe sein und die App offen haben.
#[tauri::command]
async fn nah_abgleichen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    fingerabdruck: String,
) -> Result<Abgleichanzeige, String> {
    use openany_nahbereich::Gastgeber;
    let g = &zustand.gastgeber;
    let Some((_, name)) = g
        .gepaarte()
        .into_iter()
        .find(|(fp, _)| *fp == fingerabdruck)
    else {
        return Err("This device is not paired.".into());
    };

    let ident = ident_laden(&zustand).await?;
    let adresse = adresse_zum_abgleichen(&zustand, &fingerabdruck).await?;
    // Die Person gleich mit: neue Geraete, ein neuer Name.
    person_tauschen(&zustand, &adresse, &fingerabdruck).await;
    let gegenstelle = openany_nahbereich::NahGegenstelle::neu(
        &ident,
        &adresse,
        openany_nahbereich::DIENST_PORT,
        &fingerabdruck,
        &name,
    )?;

    let bericht = {
        let speicher = zustand.speicher.lock().await;
        Laeufer::neu(&speicher, &gegenstelle, g.mein_name())
            .lauf()
            .await
    };

    // Erst NACH dem Lauf, ohne gesperrten Speicher: Inhalte koennen dauern.
    // Vorschaubilder immer, Originale nach der Regel dieses Geraets.
    let inhalte = if bericht.durchgelaufen() {
        Some(dateibefehle::fehlende_holen(&zustand, &gegenstelle).await)
    } else {
        None
    };

    Ok(Abgleichanzeige {
        gezogen: bericht.gezogen,
        geschoben: bericht.geschoben,
        uebersprungen: bericht.uebersprungen,
        uebersprungen_weil: bericht.gruende,
        konflikte: bericht.konflikte,
        fehler: bericht.fehler,
        belegt: None,
        grenze: None,
        inhalte,
    })
}

/// Eine Paarung vergessen -- nur hier. Das andere Geraet merkt es erst, wenn
/// es beim naechsten Abgleich abgewiesen wird.
#[tauri::command]
async fn nah_vergessen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    fingerabdruck: String,
) -> Result<(), String> {
    zustand.gastgeber.vergessen(&fingerabdruck);
    Ok(())
}

#[tauri::command]
async fn nah_suchen(zustand: tauri::State<'_, Arc<Zustand>>) -> Result<(), String> {
    if let Some(nah) = zustand.nah.lock().await.as_ref() {
        nah.suchen();
    }
    Ok(())
}

/* ── Start ────────────────────────────────────────────────────────────── */

/// Der Einstiegspunkt.
///
/// **Die Anmerkung ist auf Android keine Zierde, sondern der Einstieg
/// selbst.** Dort startet kein Binaerprogramm, sondern das System laedt
/// `libopenany_app_lib.so` und ruft eine Funktion darin. Ohne
/// `mobile_entry_point` entsteht sie nicht -- der Bau laeuft durch, und erst
/// das Zusammensetzen des APK bricht ab mit "does not include required
/// runtime symbols". Auf dem Schreibtisch faellt das nie auf, weil dort
/// `main()` ruft.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn starten() {
    let bau = tauri::Builder::default().plugin(tauri_plugin_opener::init());
    // Die Dialoge des Systems -- nur auf dem Schreibtisch (ablagebefehle.rs).
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let bau = bau
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build());
    bau.setup(|app| {
        // Derselbe Ordner, den auf Android auch der Auffrischer nennt
        // (`Auffrischer.kt`: dataDir und cacheDir): `app_data_dir` ist
        // dort `getDataDir` ohne Zusatz. Weicht das je ab, oeffnet der
        // Hintergrund eine ZWEITE Datenbank -- und niemand merkt es.
        let ordner = app.path().app_data_dir()?;
        let zwischenspeicher = app.path().app_cache_dir()?;
        let zustand = Zustand::holen(ordner, zwischenspeicher)?;
        app.manage(zustand.clone());

        // Geraete in der Naehe: im Hintergrund, damit das Fenster nicht auf
        // Netz und Zertifikat wartet. Scheitert es (Port belegt, kein
        // Netz), laeuft das Programm ohne -- und sagt, warum.
        tauri::async_runtime::spawn(nah_starten(zustand.clone()));

        // Auffrischen und Wachdienst auf dem Schreibtisch, solange das
        // Programm laeuft (hintergrunddienste.rs). Unter Android plant das
        // System beides.
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        hintergrunddienste::schreibtisch::starten(app.handle(), &zustand);
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        aktualisierung::beim_start_fragen(app.handle().clone());

        // Nachrichten: Ist ein Matrix-Konto verbunden, gleich abgleichen
        // -- im Hintergrund, ein Homeserver darf das Fenster nicht aufhalten.
        tauri::async_runtime::spawn(nachrichtenbefehle::starten(
            app.handle().clone(),
            zustand.clone(),
        ));
        // E-Mail: ist ein Postfach verbunden, im Hintergrund abholen.
        tauri::async_runtime::spawn(mailbefehle::starten(app.handle().clone(), zustand));

        Ok(())
    })
    .invoke_handler(tauri::generate_handler![
        lage,
        einstellungen_speichern,
        openany_verbinden,
        sicherungsbefehle::sicherung_vorschlag,
        sicherungsbefehle::sicherung_anlegen,
        sicherungsbefehle::sicherung_ablegen,
        ablagebefehle::sicherung_waehlen,
        ablagebefehle::aufs_geraet,
        aktualisierung::aktualisierung_lage,
        aktualisierung::aktualisierung_installieren,
        hintergrunddienste::hintergrund_lage,
        hintergrunddienste::hintergrund_setzen,
        sicherungsbefehle::sicherung_oeffnen,
        sicherungsbefehle::sicherung_verwerfen,
        sicherungsbefehle::sicherung_einspielen,
        kopplung_laeuft,
        kopplung_abholen,
        abmelden,
        abgleichen,
        bestand_neu_holen,
        notizbuch_liste,
        notizbuch_notiz,
        notizbuch_anlegen,
        notizbuch_aendern,
        notizbuch_mappen,
        notizbuch_anhaenge,
        notizbuch_mappe_anlegen,
        notizbuch_mappe_umbenennen,
        notizbuch_mappe_loeschen,
        notizbuch_titel,
        notizbuch_aufloesen,
        notizbuch_tags,
        notizbuch_rueckverweise,
        notizbuch_graph,
        notiz_papierkorb,
        kalenderbuch_kalender,
        kalenderbuch_kalender_anlegen,
        kalenderbuch_kalender_aendern,
        kalenderbuch_kalender_loeschen,
        kalenderbuch_abo_auffrischen,
        kalenderbuch_termine,
        kalenderbuch_termin_speichern,
        termin_papierkorb,
        adressbuch_kontakte,
        adressbuch_speichern,
        adressbuch_loeschen,
        adressbuch_foto_setzen,
        adressbuch_foto_entfernen,
        papierkorb_liste,
        papierkorb_zurueck,
        papierkorb_endgueltig,
        start_uebersicht,
        nah_lage,
        nah_suchen,
        nah_paaren,
        nah_bestaetigen,
        nah_abbrechen,
        nah_vergessen,
        nah_person_name,
        nah_einladen,
        nah_einladung_bestaetigen,
        nah_einladung_abbrechen,
        nah_abgleichen,
        dateibefehle::dateien_liste,
        dateibefehle::dateien_baum,
        dateibefehle::dateien_suchen,
        dateibefehle::dateien_text_offen,
        dateibefehle::datei_text_setzen,
        dateibefehle::datei_ordner_anlegen,
        dateibefehle::datei_umbenennen,
        dateibefehle::datei_verschieben,
        dateibefehle::datei_papierkorb,
        dateibefehle::datei_hochladen_beginnen,
        dateibefehle::datei_hochladen_stueck,
        dateibefehle::datei_hochladen_fertig,
        dateibefehle::datei_ersetzen_fertig,
        galeriebefehle::galerie_oeffnen,
        galeriebefehle::galerie_standbild_setzen,
        dateibefehle::datei_hochladen_abbrechen,
        dateibefehle::datei_inhalt,
        dateibefehle::datei_oeffnen,
        dateibefehle::datei_holen,
        dateibefehle::speicher_lage,
        dateibefehle::speicher_regel_setzen,
        dateibefehle::platz_freigeben,
        dateibefehle::behalten_setzen,
        galeriebefehle::galerie_alben,
        galeriebefehle::galerie_album,
        galeriebefehle::galerie_album_anlegen,
        galeriebefehle::galerie_album_verschieben,
        galeriebefehle::galerie_albenbaum,
        galeriebefehle::galerie_album_papierkorb,
        galeriebefehle::galerie_bilder,
        galeriebefehle::galerie_bild_papierkorb,
        galeriebefehle::galerie_bild_fertig,
        galeriebefehle::galerie_bild_holen,
        anhangbefehle::notizbuch_anhang_beginnen,
        anhangbefehle::notizbuch_anhang_fertig,
        anhangbefehle::notizbuch_anhang_oeffnen,
        nachrichtenbefehle::nachrichten_lage,
        nachrichtenbefehle::nachrichten_anmelden,
        nachrichtenbefehle::nachrichten_abmelden,
        nachrichtenbefehle::nachrichten_standard,
        nachrichtenbefehle::nachrichten_liste,
        nachrichtenbefehle::nachrichten_unterhaltungen,
        nachrichtenbefehle::nachricht_senden,
        nachrichtenbefehle::nachricht_anhang_senden,
        mailbefehle::mail_lage,
        mailbefehle::mail_server_finden,
        mailbefehle::mail_verbinden,
        mailbefehle::mail_trennen,
        mailbefehle::mail_abholen,
        mailbefehle::mail_standard,
        mailbefehle::mail_spam,
        mailbefehle::mail_kein_spam,
        mailbefehle::mail_spam_loeschen,
        mailbefehle::mail_pgp_erzeugen,
        mailbefehle::mail_pgp_einlesen,
        mailbefehle::mail_pgp_entfernen,
        mailbefehle::mail_pgp_liste,
        mailbefehle::mail_pgp_suchen,
        mailbefehle::mail_pgp_hand,
        mailbefehle::mail_pgp_vergessen,
        mailbefehle::mail_pgp_wechsel_gesehen,
        mailbefehle::mail_pgp_status,
        mailbefehle::mail_pgp_ausfuhr,
        mailbefehle::mail_pgp_oeffentlich,
        mailbefehle::mail_senden,
        nachrichtenbefehle::nachricht_anhang,
        nachrichtenbefehle::nachrichten_anhang_grenze,
        nachrichtenbefehle::nachricht_gelesen,
        nachrichtenbefehle::nachricht_loeschen,
        projektbefehle::projekte_liste,
        projektbefehle::projekt_sachen,
        projektbefehle::projekt_sache,
        projektbefehle::projekt_sache_anlegen,
        projektbefehle::projekt_sache_aendern,
        projektbefehle::projekt_sache_entfernen,
        projektbefehle::projekt_lokal_anlegen,
        projektbefehle::projekt_mitglieder,
        projektbefehle::projekt_lokal_loeschen,
        projektbefehle::projekt_mitglied_entfernen,
        projektbefehle::projekt_austreten,
        projektbefehle::projekt_chat,
        projektbefehle::projekt_chat_senden,
        projektbefehle::projekt_chat_ungelesen,
        projektbefehle::projekt_chat_gelesen,
        direktbefehle::nah_nachricht_ziele,
        direktbefehle::nah_blockieren,
        direktbefehle::nah_anfragen,
        direktbefehle::nah_anfragen_erlauben,
        direktbefehle::nah_kontakt_bestaetigen,
        projektbefehle::projekt_notiz_sperren,
        projektbefehle::projekt_notiz_speichern,
        projektbefehle::projekt_notiz_entsperren,
        projektbefehle::projekt_lokale_freigaben,
        projektbefehle::projekt_lokal_freigeben,
        projektbefehle::projekt_geteilte_notiz,
        projektbefehle::projekt_geteilter_inhalt,
        projektbefehle::projekte_nah_abgleichen,
        projektbefehle::projekt_abstimmen,
        projektbefehle::projekte_abgleichen,
        projektbefehle::projekt_freigaben,
        projektbefehle::projekt_freigabe_inhalt,
        projektbefehle::projekt_freigabe_datei,
    ])
    .run(tauri::generate_context!())
    .expect("The window could not be opened.");
}
