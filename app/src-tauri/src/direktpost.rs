//! Direktnachrichten Ende-zu-Ende -- Konto-Modell (docs/plan-direktnachrichten-e2e.md,
//! Tiffy 16.10.2026).
//!
//! Die Kryptographie steht in `openany-direktpost`; hier die Verdrahtung:
//!
//! - **Geraeteschluessel** im Tresor (`ausweise/nachrichten-schluessel`),
//!   **Konto-Schluessel** samt frueheren Fassungen ebenda
//!   (`ausweise/nachrichten-konto`). Beide gehen in die Sicherungsdatei.
//! - [`bereit`]: Geraeteschluessel melden; gibt es noch keinen
//!   Konto-Schluessel, ihn anlegen (erstes Geraet); fehlt er hier, die Kiste
//!   abholen -- oder, solange keine da ist, auf Freigabe warten.
//! - Senden verschluesselt an den Konto-Schluessel des Empfaengers und den
//!   eigenen; hat der Empfaenger noch keinen, geht nichts hinaus (seit
//!   Schritt 3 nimmt der Server keinen Klartext mehr an).
//! - Lesen prueft gegen das unterschreibende Geraet und entschluesselt mit
//!   irgendeiner Fassung des eigenen Konto-Schluessels.
//! - Freigeben, Ablehnen, Zurueckziehen, Wechseln: die Befehle unten fuer die
//!   Kachel in den Einstellungen und den Hinweis in den Nachrichten.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::Arc;

use openany_client::{OpenanyClient, OpenanyError};
use openany_direktpost::{Geraet, Kontoschluessel, Schluessel, Verschluesselt};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{tresor, Zustand};

/// Die Namen im Tresor -- und der Dateien in der Sicherung (`geheim/…`).
pub(crate) const TRESOR: &str = "nachrichten-schluessel";
pub(crate) const TRESOR_KONTO: &str = "nachrichten-konto";

/// Was beim Server gemeldet ist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Gemeldet {
    /// Der eigene Eintrag im Verzeichnis (unterschreibt).
    pub geraet: i64,
    /// Der eigene Konto-Name (Absender im Inhalt).
    pub name: String,
    age: String,
    /// Abdruck des Geraetezugangs, unter dem gemeldet wurde.
    zugang: u64,
}

fn marke(zustand: &Zustand) -> PathBuf {
    zustand.ordner.join("direktpost.json")
}

fn ablage(zustand: &Zustand, name: &str) -> Box<dyn anyid_client::Tokenspeicher + Send + Sync> {
    tresor::ablage(zustand.ordner.join("ausweise").join(name))
}

fn unlesbar(e: impl ToString) -> OpenanyError {
    OpenanyError::Unlesbar(e.to_string())
}

/// Die Geraeteschluessel -- erzeugt, wenn es noch keine gibt.
pub(crate) fn schluessel(zustand: &Zustand) -> Result<Schluessel, String> {
    let ablage = ablage(zustand, TRESOR);
    if let Some(text) = ablage.lesen().map_err(|e| e.to_string())? {
        return Schluessel::aus_text(&text).map_err(|e| e.to_string());
    }
    let neu = Schluessel::erzeugen().map_err(|e| e.to_string())?;
    ablage
        .schreiben(&neu.als_text())
        .map_err(|e| e.to_string())?;
    Ok(neu)
}

/// Der Konto-Schluessel, falls dieses Geraet ihn hat.
pub(crate) fn konto(zustand: &Zustand) -> Option<Kontoschluessel> {
    let text = ablage(zustand, TRESOR_KONTO).lesen().ok()??;
    Kontoschluessel::aus_text(&text).ok()
}

fn konto_merken(zustand: &Zustand, k: &Kontoschluessel) -> Result<(), String> {
    ablage(zustand, TRESOR_KONTO)
        .schreiben(&k.als_text())
        .map_err(|e| e.to_string())
}

fn abdruck(text: &str) -> u64 {
    let mut h = DefaultHasher::new();
    text.hash(&mut h);
    h.finish()
}

/// Die Geraeteschluessel beim Server melden -- nur, wenn noetig.
async fn melden(zustand: &Arc<Zustand>, client: &OpenanyClient) -> Result<Gemeldet, OpenanyError> {
    let s = schluessel(zustand).map_err(unlesbar)?;
    let (age, ed25519) = s.oeffentlich();
    let zugang = abdruck(
        &zustand
            .schluesselablage()
            .lesen()
            .ok()
            .flatten()
            .unwrap_or_default(),
    );

    if let Ok(text) = std::fs::read_to_string(marke(zustand)) {
        if let Ok(g) = serde_json::from_str::<Gemeldet>(&text) {
            if g.age == age && g.zugang == zugang {
                return Ok(g);
            }
        }
    }

    let antwort = client.nachrichten_schluessel_setzen(&age, &ed25519).await?;
    let g = Gemeldet {
        geraet: antwort
            .get("geraet")
            .and_then(Value::as_i64)
            .ok_or_else(|| unlesbar("no device id"))?,
        name: antwort
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        age,
        zugang,
    };
    let _ = std::fs::write(marke(zustand), serde_json::to_vec(&g).unwrap_or_default());
    Ok(g)
}

/// Ein Eintrag im Verzeichnis eines Kontos (beim eigenen mit mehr Feldern).
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Eintrag {
    pub(crate) geraet: i64,
    pub(crate) ed25519: String,
    #[serde(default)]
    aktiv: bool,
    #[serde(default)]
    age: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    freigegeben: bool,
    #[serde(default)]
    abgelehnt: bool,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Verzeichnis {
    /// (Fassung, `age1…`) des gueltigen Konto-Schluessels.
    pub(crate) konto: Option<(u32, String)>,
    pub(crate) geraete: Vec<Eintrag>,
}

pub(crate) async fn verzeichnis(
    client: &OpenanyClient,
    name: &str,
) -> Result<Verzeichnis, OpenanyError> {
    let a = client.nachrichten_schluessel(name).await?;
    let konto = a.get("konto").filter(|k| !k.is_null()).and_then(|k| {
        Some((
            k.get("version")?.as_u64()? as u32,
            k.get("age")?.as_str()?.to_string(),
        ))
    });
    let geraete = a
        .get("geraete")
        .cloned()
        .and_then(|w| serde_json::from_value(w).ok())
        .unwrap_or_default();
    Ok(Verzeichnis { konto, geraete })
}

/// Wie es um die Schluessel dieses Geraets steht.
#[derive(Debug, Clone, Serialize)]
pub struct Lage {
    /// `bereit` (Konto-Schluessel hier), `wartet` (auf Freigabe).
    stand: &'static str,
    /// Wurde dieses Geraet eben von einem anderen freigeschaltet? Dann kommt
    /// die Kiste von selbst (freischalten.rs).
    kiste_erwartet: bool,
    fassung: Option<u32>,
    geraete: Vec<GeraetAnzeige>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GeraetAnzeige {
    geraet: i64,
    name: String,
    dieses: bool,
    aktiv: bool,
    freigegeben: bool,
    /// Wartet auf Freigabe (aktiv, nicht freigegeben, nicht abgelehnt).
    wartet: bool,
}

/// Bereit machen: melden, Konto-Schluessel anlegen oder abholen.
pub(crate) async fn bereit(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
) -> Result<(Gemeldet, Lage), OpenanyError> {
    let ich = melden(zustand, client).await?;
    let mut v = verzeichnis(client, &ich.name).await?;
    let s = schluessel(zustand).map_err(unlesbar)?;
    let mut hier = konto(zustand);
    let hier_fassung = hier.as_ref().map_or(0, |k| k.aktuell_fassung());

    match &v.konto {
        // Noch kein Konto-Schluessel -- dies ist das erste Geraet. (Liegt
        // hier schon einer, etwa aus einer Sicherung, wird DER angelegt.)
        None => {
            let k = hier.take().unwrap_or_else(Kontoschluessel::neu);
            let (c, sig) = openany_direktpost::kiste_packen(&s, &k, ich.geraet, &s.oeffentlich().0)
                .map_err(unlesbar)?;
            client
                .nachrichten_konto_setzen(
                    k.aktuell_fassung(),
                    &k.oeffentlich(),
                    &[(ich.geraet, c, sig)],
                )
                .await?;
            konto_merken(zustand, &k).map_err(unlesbar)?;
            hier = Some(k);
            v = verzeichnis(client, &ich.name).await?;
        }
        // Hier fehlt die gueltige Fassung: die Kiste abholen.
        Some((fassung, _)) if hier_fassung < *fassung => {
            if let Some(k) = kiste_abholen(client, &s, &ich, &v).await? {
                konto_merken(zustand, &k).map_err(unlesbar)?;
                hier = Some(k);
            }
        }
        _ => {}
    }

    // Z4: Geraete, die ein Mensch hier eben per Ziffern freigeschaltet hat,
    // bekommen ihre Kiste, sobald sie mit genau den bewiesenen Schluesseln im
    // Verzeichnis stehen -- und nur dann.
    if let Some(k) = hier.as_ref() {
        if freigeschaltete_bedienen(zustand, client, &s, &ich, &v, k).await? {
            v = verzeichnis(client, &ich.name).await?;
        }
    }

    let lage = lage_aus(zustand, &ich, &v, hier.as_ref());
    Ok((ich, lage))
}

/// Die Kisten fuer frisch freigeschaltete Geraete legen. `true`, wenn eine
/// gelegt wurde.
async fn freigeschaltete_bedienen(
    zustand: &Zustand,
    client: &OpenanyClient,
    s: &Schluessel,
    ich: &Gemeldet,
    v: &Verzeichnis,
    k: &Kontoschluessel,
) -> Result<bool, OpenanyError> {
    let mut gelegt = false;
    for e in crate::freischalten::erwartete(zustand) {
        let Some(g) = v.geraete.iter().find(|g| {
            g.aktiv
                && g.geraet != ich.geraet
                && g.ed25519 == e.ed25519
                && g.age.as_deref() == Some(e.age.as_str())
        }) else {
            continue;
        };
        if !g.freigegeben {
            let (c, sig) =
                openany_direktpost::kiste_packen(s, k, g.geraet, &e.age).map_err(unlesbar)?;
            client
                .nachrichten_kiste_legen(g.geraet, k.aktuell_fassung(), &c, &sig)
                .await?;
            gelegt = true;
        }
        crate::freischalten::erledigt(zustand, &e.age, &e.ed25519);
    }
    Ok(gelegt)
}

/// Fuer das Adressbuch (adressbuch.rs): Schluessel, Konto-Schluessel und die
/// eigenen Geraete -- `None`, solange dieses Geraet auf Freigabe wartet.
pub(crate) async fn adressbuch(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
) -> Option<crate::adressbuch::Adressbuch> {
    let (ich, lage) = bereit(zustand, client).await.ok()?;
    if lage.stand != "bereit" {
        return None;
    }
    let konto = konto(zustand)?;
    let v = verzeichnis(client, &ich.name).await.ok()?;
    Some(crate::adressbuch::Adressbuch {
        ich: schluessel(zustand).ok()?,
        konto,
        geraet: ich.geraet,
        geraete: v
            .geraete
            .into_iter()
            .map(|g| (g.geraet, g.ed25519))
            .collect(),
    })
}

/// Alle lebenden Kontakte zum Hinausschicken vormerken -- beim Umstellen und
/// nach einem Wechsel: Dann gehen sie an die neue Fassung, und ein
/// zurueckgezogenes Geraet liest den neuen Stand nicht mehr.
pub(crate) async fn kontakte_vormerken(zustand: &Zustand) -> Result<usize, String> {
    let speicher = zustand.speicher.lock().await;
    let mut n = 0;
    for k in speicher.kontakte().map_err(|e| e.to_string())? {
        if k.papierkorb_at.is_none() {
            speicher
                .merken(
                    &openany_client::Art::Kontakt,
                    &k.uuid,
                    openany_client::Was::Da,
                )
                .map_err(|e| e.to_string())?;
            n += 1;
        }
    }
    Ok(n)
}

/// Alle eigenen Kalender und Termine noch einmal hinausschicken -- zum
/// Umstellen (docs/plan-kalender-e2e.md). Termine aus Abos nicht: Sie reisen
/// nie, jedes Geraet holt den Feed selbst.
pub(crate) async fn kalender_vormerken(zustand: &Zustand) -> Result<usize, String> {
    use openany_client::{Art, Was};
    let speicher = zustand.speicher.lock().await;
    let mut n = 0;
    let kalender = speicher.kalender_alle().map_err(|e| e.to_string())?;
    for k in &kalender {
        speicher
            .merken(&Art::Kalender, &k.uuid, Was::Da)
            .map_err(|e| e.to_string())?;
        n += 1;
    }
    for t in speicher.termine_alle().map_err(|e| e.to_string())? {
        let abo = kalender
            .iter()
            .any(|k| k.uuid == t.kalender_uuid && k.ist_abo());
        if !abo {
            speicher
                .merken(&Art::Termin, &t.uuid, Was::Da)
                .map_err(|e| e.to_string())?;
            n += 1;
        }
    }
    Ok(n)
}

pub(crate) async fn notizen_vormerken(zustand: &Zustand) -> Result<usize, String> {
    use openany_client::{Art, Was};
    let speicher = zustand.speicher.lock().await;
    let mut n = 0;
    for notiz in speicher.notizen().map_err(|e| e.to_string())? {
        if notiz.papierkorb_at.is_none() {
            speicher
                .merken(&Art::Notiz, &notiz.zk_id, Was::Da)
                .map_err(|e| e.to_string())?;
            n += 1;
        }
    }
    for m in speicher.mappenangaben_alle().map_err(|e| e.to_string())? {
        speicher
            .merken(&Art::Mappe, &m.pfad, Was::Da)
            .map_err(|e| e.to_string())?;
        n += 1;
    }
    for a in speicher.anhaenge_alle().map_err(|e| e.to_string())? {
        speicher
            .merken(
                &Art::Notizanhang,
                &format!("{}|{}", a.mappe, a.pfad),
                Was::Da,
            )
            .map_err(|e| e.to_string())?;
        n += 1;
    }
    Ok(n)
}

async fn kiste_abholen(
    client: &OpenanyClient,
    s: &Schluessel,
    ich: &Gemeldet,
    v: &Verzeichnis,
) -> Result<Option<Kontoschluessel>, OpenanyError> {
    let a = client.nachrichten_kisten().await?;
    for k in a
        .get("kisten")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let version = k.get("version").and_then(Value::as_u64).unwrap_or(0) as u32;
        let von = k.get("von_geraet").and_then(Value::as_i64);
        let Some(von) = v.geraete.iter().find(|g| Some(g.geraet) == von) else {
            continue;
        };
        let (Some(c), Some(sig)) = (
            k.get("chiffrat").and_then(Value::as_str),
            k.get("signatur").and_then(Value::as_str),
        ) else {
            continue;
        };
        if let Ok(konto) =
            openany_direktpost::kiste_oeffnen(s, ich.geraet, version, c, sig, &von.ed25519)
        {
            return Ok(Some(konto));
        }
    }
    Ok(None)
}

fn lage_aus(
    zustand: &Zustand,
    ich: &Gemeldet,
    v: &Verzeichnis,
    hier: Option<&Kontoschluessel>,
) -> Lage {
    let fassung = v.konto.as_ref().map(|(f, _)| *f);
    let bereit = matches!((hier, fassung), (Some(k), Some(f)) if k.aktuell_fassung() >= f);
    Lage {
        stand: if bereit { "bereit" } else { "wartet" },
        kiste_erwartet: !bereit && crate::freischalten::kiste_erwartet(zustand),
        fassung,
        geraete: v
            .geraete
            .iter()
            .map(|g| GeraetAnzeige {
                geraet: g.geraet,
                name: g.name.clone().unwrap_or_default(),
                dieses: g.geraet == ich.geraet,
                aktiv: g.aktiv,
                freigegeben: g.freigegeben,
                wartet: g.aktiv && !g.freigegeben && !g.abgelehnt && g.geraet != ich.geraet,
            })
            .collect(),
    }
}

/// Ein Anhang, wie er aus der Ansicht kommt -- noch unverschluesselt.
pub(crate) struct Datei {
    pub name: String,
    pub mime: String,
    pub bytes: Vec<u8>,
}

/// Die Anhang-Angaben der zuletzt entschluesselten Nachrichten, nach
/// Server-Id. Der Schluessel einer Datei steht nur im verschluesselten
/// Inhalt; wer einen Anhang oeffnet, hat die Zeile gerade gesehen -- so
/// braucht es keinen zweiten Weg zum Inhalt.
static ANHAENGE: std::sync::Mutex<Option<HashMap<i64, openany_direktpost::AnhangInfo>>> =
    std::sync::Mutex::new(None);

fn anhang_merken(id: i64, info: openany_direktpost::AnhangInfo) {
    if let Ok(mut m) = ANHAENGE.lock() {
        m.get_or_insert_with(HashMap::new).insert(id, info);
    }
}

/// Die Bytes eines Anhangs: vom Server holen, pruefen, entschluesseln.
pub(crate) async fn anhang_laden(client: &OpenanyClient, id: i64) -> Result<Vec<u8>, String> {
    let info = ANHAENGE
        .lock()
        .ok()
        .and_then(|m| m.as_ref().and_then(|m| m.get(&id).cloned()))
        .ok_or("Reload the messages, then open the attachment again.")?;
    let chiffrat = client
        .nachricht_anhang(id)
        .await
        .map_err(|e| e.to_string())?;
    openany_direktpost::anhang_oeffnen(&chiffrat, &info).map_err(|e| e.to_string())
}

/// Eine Direktnachricht hinaus -- verschluesselt an den Konto-Schluessel des
/// Empfaengers und den eigenen. Hat der Empfaenger noch keinen, ein Fehler:
/// Er muss die App erst einmal oeffnen. Ein Anhang wird mit eigenem
/// Schluessel verschluesselt; der steht im Inhalt.
pub(crate) async fn senden(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    an: &str,
    text: &str,
    datei: Option<Datei>,
) -> Result<Value, OpenanyError> {
    let (ich, _) = bereit(zustand, client).await?;
    let empfaenger = verzeichnis(client, an).await?;
    let Some((_, an_age)) = empfaenger.konto else {
        return Err(OpenanyError::OhneSchluessel(an.to_string()));
    };
    let mut ziele = vec![an_age];
    // Der eigene Konto-Schluessel -- der oeffentliche Teil steht im
    // Verzeichnis, auch solange dieses Geraet noch auf Freigabe wartet.
    if let Some((_, eigen)) = verzeichnis(client, &ich.name).await?.konto {
        ziele.push(eigen);
    }

    let s = schluessel(zustand).map_err(unlesbar)?;
    let (chiffrat, info) = match datei {
        Some(d) => {
            let (c, i) = openany_direktpost::anhang_verschluesseln(&d.name, &d.mime, &d.bytes)
                .map_err(unlesbar)?;
            (Some(c), Some(i))
        }
        None => (None, None),
    };
    let v = openany_direktpost::verschluesseln(&s, &ich.name, an, text, info.clone(), &ziele)
        .map_err(unlesbar)?;
    let mut antwort = client
        .nachricht_senden_verschluesselt(an, &v.inhalt, &v.signatur, ich.geraet, chiffrat)
        .await?;
    // Die eigene Zeile zeigt gleich, was hinausging.
    if let Some(i) = info {
        if let Some(id) = antwort.get("id").and_then(Value::as_i64) {
            anhang_merken(id, i.clone());
        }
        anhang_eintragen(&mut antwort, &i);
    }
    markieren(&mut antwort, Some(text.to_string()));
    Ok(antwort)
}

/// Die verschluesselten Zeilen einer Server-Antwort entschluesseln -- an Ort
/// und Stelle: `body` bekommt den Text, `e2e` sagt `verschluesselt` oder
/// `unlesbar`.
pub(crate) async fn entschluesseln(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    zeilen: Vec<&mut Value>,
) {
    if zeilen
        .iter()
        .all(|z| z.get("inhalt").is_none_or(Value::is_null))
    {
        return;
    }
    let k = konto(zustand);
    let mut verzeichnisse: HashMap<String, Verzeichnis> = HashMap::new();
    let ich = mein_name(zustand);
    let pruefungen = pruefungen(zustand).await;

    for z in zeilen {
        let Some(inhalt) = z.get("inhalt").and_then(Value::as_str).map(str::to_string) else {
            continue;
        };
        let name = |feld: &str| {
            z.get(feld)
                .and_then(|w| w.get("name"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string()
        };
        let (von, an) = (name("sender"), name("recipient"));
        let signatur = z
            .get("signatur")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let geraet_id = z.get("geraet").and_then(Value::as_i64);

        // Das Gegenueber: bei eigenen Nachrichten der Empfaenger.
        let gegenueber = if ich.as_deref() == Some(von.as_str()) {
            an.clone()
        } else {
            von.clone()
        };
        for n in [&von, &gegenueber] {
            if !verzeichnisse.contains_key(n) {
                let v = verzeichnis(client, n).await.unwrap_or_default();
                verzeichnisse.insert(n.clone(), v);
            }
        }
        let absender = verzeichnisse
            .get(&von)
            .and_then(|v| v.geraete.iter().find(|g| Some(g.geraet) == geraet_id))
            .map(|g| Geraet {
                geraet: g.geraet,
                ed25519: g.ed25519.clone(),
                aktiv: g.aktiv,
            });

        let inhalt = match (&k, absender) {
            (Some(k), Some(g)) => {
                openany_direktpost::oeffnen(k, &von, &an, &Verschluesselt { inhalt, signatur }, &g)
                    .ok()
            }
            _ => None,
        };
        if let Some(info) = inhalt.as_ref().and_then(|i| i.anhang.clone()) {
            if let Some(id) = z.get("id").and_then(Value::as_i64) {
                anhang_merken(id, info.clone());
            }
            anhang_eintragen(z, &info);
        }
        let gelesen = inhalt.is_some();
        markieren(z, inhalt.map(|i| i.text));

        // Sicherheitsnummer (2c): geprueft, oder seither geaendert?
        if gelesen {
            let age = verzeichnisse
                .get(&gegenueber)
                .and_then(|v| v.konto.as_ref())
                .map(|(_, a)| a.as_str());
            if let (Some(stand), Some(o)) =
                (stand_aus(&pruefungen, &gegenueber, age), z.as_object_mut())
            {
                o.insert("e2e_geprueft".into(), Value::String(stand.into()));
            }
        }
    }
}

/// Name, Typ und Groesse fuer die Ansicht -- ohne Schluessel.
fn anhang_eintragen(z: &mut Value, info: &openany_direktpost::AnhangInfo) {
    if let Some(o) = z.as_object_mut() {
        o.insert(
            "e2e_anhang".into(),
            serde_json::json!({ "name": info.name, "mime": info.mime, "groesse": info.groesse }),
        );
    }
}

fn markieren(z: &mut Value, text: Option<String>) {
    if let Some(o) = z.as_object_mut() {
        match text {
            Some(t) => {
                o.insert("body".into(), Value::String(t));
                o.insert("e2e".into(), Value::String("verschluesselt".into()));
            }
            None => {
                o.insert("body".into(), Value::String(String::new()));
                o.insert("e2e".into(), Value::String("unlesbar".into()));
            }
        }
    }
}

// --- Befehle fuer die Oberflaeche ------------------------------------------

async fn verbunden(zustand: &Arc<Zustand>) -> Result<OpenanyClient, String> {
    crate::server_client(zustand)
        .await
        .ok_or_else(|| "Not connected to openany.de.".to_string())
}

/// Wie es steht -- fuer die Kachel und den Hinweis in den Nachrichten.
#[tauri::command]
pub async fn nachrichtenschluessel_lage(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Lage, String> {
    let client = verbunden(&zustand).await?;
    Ok(bereit(&zustand, &client)
        .await
        .map_err(|e| e.to_string())?
        .1)
}

/*
 * `nachrichtenschluessel_freigeben` (Codes vergleichen, dann Kiste legen) ist
 * seit Z4 fort: Die Ziffern kamen aus den Schluesseln des neuen Geraets, und
 * ein luegender Server konnte sich in Sekunden passende erzeugen. Freigegeben
 * wird jetzt per SPAKE2 (freischalten.rs).
 */

#[tauri::command]
pub async fn nachrichtenschluessel_ablehnen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    geraet: i64,
) -> Result<Lage, String> {
    let client = verbunden(&zustand).await?;
    client
        .nachrichten_geraet(geraet, "ablehnen")
        .await
        .map_err(|e| e.to_string())?;
    Ok(bereit(&zustand, &client)
        .await
        .map_err(|e| e.to_string())?
        .1)
}

/// Eine Supportanfrage (docs/plan-support.md): als verschluesselte
/// Direktnachricht an jeden Verwalter. Die Antwort kommt in den Nachrichten.
/// Ob der Text personenbezogene Daten enthaelt, prueft die Oberflaeche vor
/// dem Senden (packages/oberflaeche/support/personenbezug.js); der Server
/// koennte es gar nicht.
#[tauri::command]
pub async fn support_senden(
    zustand: tauri::State<'_, Arc<Zustand>>,
    text: String,
) -> Result<usize, String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("Empty request.".into());
    }
    let client = verbunden(&zustand).await?;
    let verwalter = client
        .support_verwalter()
        .await
        .map_err(|e| e.to_string())?;
    if verwalter.is_empty() {
        return Err("grund:kein_verwalter".into());
    }

    let nachricht = format!("Supportanfrage\n\n{text}");
    let mut zugestellt = 0;
    let mut letzter_fehler = None;
    for name in &verwalter {
        match senden(&zustand, &client, name, &nachricht, None).await {
            Ok(_) => zugestellt += 1,
            Err(e) => letzter_fehler = Some(e.to_string()),
        }
    }
    if zugestellt == 0 {
        return Err(letzter_fehler.unwrap_or_else(|| "grund:kein_verwalter".into()));
    }
    Ok(zugestellt)
}

/// Das eigene Konto loeschen (docs/plan-konto-loeschen.md) -- sofort und
/// endgueltig. Danach ist dieses Geraet getrennt; was hier liegt, bleibt
/// hier. Lehnt der Server ab, kommt sein Grund als `grund:<code>` zurueck,
/// damit die Oberflaeche ihn in der eigenen Sprache sagen kann.
#[tauri::command]
pub async fn konto_loeschen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    name: String,
) -> Result<(), String> {
    let client = verbunden(&zustand).await?;
    if let Some(grund) = client
        .konto_loeschen(name.trim())
        .await
        .map_err(|e| e.to_string())?
    {
        return Err(format!("grund:{grund}"));
    }
    // Den Kontoschluessel des geloeschten Kontos vergessen -- sonst legte
    // `bereit` ihn bei der naechsten Anmeldung als Schluessel eines neuen
    // Kontos an. Die Geraeteschluessel bleiben; sie gehoeren dem Geraet.
    let _ = ablage(&zustand, TRESOR_KONTO).vergessen();
    let _ = std::fs::remove_file(marke(&zustand));
    crate::verbindung_vergessen(&zustand).await
}

/// Ein Geraet zurueckziehen (verloren, verkauft) -- und gleich wechseln, damit
/// es neue Nachrichten nicht mehr liest.
#[tauri::command]
pub async fn nachrichtenschluessel_zurueckziehen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    geraet: i64,
) -> Result<Lage, String> {
    let client = verbunden(&zustand).await?;
    client
        .nachrichten_geraet(geraet, "zurueckziehen")
        .await
        .map_err(|e| e.to_string())?;
    wechseln(&zustand, &client).await
}

/// Den Konto-Schluessel wechseln: neue Fassung, Kisten fuer alle aktiven,
/// freigegebenen Geraete. Alte Fassungen bleiben -- der Verlauf bleibt lesbar.
#[tauri::command]
pub async fn nachrichtenschluessel_wechseln(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Lage, String> {
    let client = verbunden(&zustand).await?;
    wechseln(&zustand, &client).await
}

async fn wechseln(zustand: &Arc<Zustand>, client: &OpenanyClient) -> Result<Lage, String> {
    let (ich, _) = bereit(zustand, client).await.map_err(|e| e.to_string())?;
    let mut k = konto(zustand).ok_or("This device does not have the message key yet.")?;
    let v = verzeichnis(client, &ich.name)
        .await
        .map_err(|e| e.to_string())?;
    let s = schluessel(zustand)?;
    k.wechseln();

    let mut kisten = Vec::new();
    for g in v
        .geraete
        .iter()
        .filter(|g| g.aktiv && (g.freigegeben || g.geraet == ich.geraet))
    {
        let Some(age) = &g.age else { continue };
        let (c, sig) =
            openany_direktpost::kiste_packen(&s, &k, g.geraet, age).map_err(|e| e.to_string())?;
        kisten.push((g.geraet, c, sig));
    }
    client
        .nachrichten_konto_setzen(k.aktuell_fassung(), &k.oeffentlich(), &kisten)
        .await
        .map_err(|e| e.to_string())?;
    konto_merken(zustand, &k)?;
    kontakte_vormerken(zustand).await?;
    Ok(bereit(zustand, client).await.map_err(|e| e.to_string())?.1)
}

// --- Sicherheitsnummer (docs/plan-adressbuch-e2e.md, 2c) --------------------

/// Der eigene Konto-Name, wie er beim Melden hinterlegt wurde.
pub(crate) fn mein_name(zustand: &Zustand) -> Option<String> {
    let text = std::fs::read_to_string(marke(zustand)).ok()?;
    serde_json::from_str::<Gemeldet>(&text).ok().map(|g| g.name)
}

/// Der eigene Eintrag im Verzeichnis (fuer den Ausweis vor Ort, W3a).
pub(crate) fn mein_geraet(zustand: &Zustand) -> Option<i64> {
    let text = std::fs::read_to_string(marke(zustand)).ok()?;
    serde_json::from_str::<Gemeldet>(&text)
        .ok()
        .map(|g| g.geraet)
}

/// Der Unterschriftsschluessel eines AKTIVEN, nicht abgelehnten Geraets von
/// `name` -- fuer den Ausweis vor Ort (serverpruefer.rs).
pub(crate) async fn aktives_geraet(
    client: &OpenanyClient,
    name: &str,
    geraet: i64,
) -> Option<String> {
    verzeichnis(client, name)
        .await
        .ok()?
        .geraete
        .into_iter()
        .find(|g| g.geraet == geraet && g.aktiv && !g.abgelehnt)
        .map(|g| g.ed25519)
}

/// Alle Pruefungen aus dem Adressbuch: Name (klein) -> (Schluessel, Kontakt).
async fn pruefungen(zustand: &Zustand) -> HashMap<String, (String, String)> {
    let speicher = zustand.speicher.lock().await;
    let mut aus = HashMap::new();
    for k in speicher.kontakte().unwrap_or_default() {
        for g in &k.geprueft {
            aus.entry(g.name.trim().to_lowercase())
                .or_insert((g.age.clone(), k.uuid.clone()));
        }
    }
    aus
}

/// `geprueft`, `geaendert` -- oder `None`, wenn nie verglichen (bzw. drueben
/// kein Schluessel bekannt ist).
fn stand_aus(
    pruefungen: &HashMap<String, (String, String)>,
    name: &str,
    age: Option<&str>,
) -> Option<&'static str> {
    let (geprueft, _) = pruefungen.get(&name.trim().to_lowercase())?;
    let age = age?;
    Some(if geprueft == age {
        "geprueft"
    } else {
        "geaendert"
    })
}

/// Der erste Kontakt mit diesem openany-Namen.
fn kontakt_mit_namen(
    speicher: &openany_store::Speicher,
    name: &str,
) -> Option<openany_store::Kontakt> {
    let name = name.trim().to_lowercase();
    speicher
        .kontakte()
        .unwrap_or_default()
        .into_iter()
        .find(|k| {
            k.wege
                .iter()
                .any(|w| w.art == "openany" && w.wert.trim().to_lowercase() == name)
                || k.geprueft
                    .iter()
                    .any(|g| g.name.trim().to_lowercase() == name)
        })
}

#[derive(Debug, Clone, Serialize)]
pub struct Sicherheit {
    name: String,
    ich: String,
    /// Zwoelf Fuenfergruppen -- `None`, wenn einer der beiden noch keinen
    /// Konto-Schluessel hat.
    nummer: Option<String>,
    /// `geprueft`, `geaendert`, `offen` oder `ohne_schluessel`.
    stand: &'static str,
    /// Der Kontakt, an dem die Pruefung steht bzw. stehen wird.
    kontakt: Option<String>,
}

async fn sicherheit(
    zustand: &Arc<Zustand>,
    client: &OpenanyClient,
    name: &str,
) -> Result<(Sicherheit, Option<String>), String> {
    let (ich, _) = bereit(zustand, client).await.map_err(|e| e.to_string())?;
    let mein = konto(zustand).map(|k| k.oeffentlich());
    let sein = verzeichnis(client, name)
        .await
        .map_err(|e| e.to_string())?
        .konto
        .map(|(_, a)| a);
    let nummer = match (&mein, &sein) {
        (Some(m), Some(s)) => Some(openany_direktpost::sicherheitsnummer(
            (&ich.name, m),
            (name, s),
        )),
        _ => None,
    };
    let kontakt = {
        let speicher = zustand.speicher.lock().await;
        kontakt_mit_namen(&speicher, name).map(|k| k.uuid)
    };
    let stand = match nummer {
        None => "ohne_schluessel",
        Some(_) => stand_aus(&pruefungen(zustand).await, name, sein.as_deref()).unwrap_or("offen"),
    };
    Ok((
        Sicherheit {
            name: name.trim().to_string(),
            ich: ich.name,
            nummer,
            stand,
            kontakt,
        },
        sein,
    ))
}

/// Die Sicherheitsnummer mit einem Konto -- zum Vergleichen.
#[tauri::command]
pub async fn sicherheitsnummer(
    zustand: tauri::State<'_, Arc<Zustand>>,
    name: String,
) -> Result<Sicherheit, String> {
    let client = verbunden(&zustand).await?;
    Ok(sicherheit(&zustand, &client, &name).await?.0)
}

/// „Stimmt überein": den jetzigen Schluessel als geprueft im Kontakt
/// festhalten. Steht der Name in keinem Kontakt, wird einer angelegt.
#[tauri::command]
pub async fn sicherheitsnummer_bestaetigen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    name: String,
) -> Result<Sicherheit, String> {
    let client = verbunden(&zustand).await?;
    let (_, sein) = sicherheit(&zustand, &client, &name).await?;
    let age = sein.ok_or("This account has no message key yet.")?;
    let name = name.trim().to_string();
    {
        let speicher = zustand.speicher.lock().await;
        let mut k = kontakt_mit_namen(&speicher, &name).unwrap_or_else(|| openany_store::Kontakt {
            uuid: uuid::Uuid::new_v4().to_string(),
            anzeigename: name.clone(),
            wege: vec![openany_store::Weg {
                art: "openany".into(),
                beschriftung: None,
                wert: name.clone(),
            }],
            ..Default::default()
        });
        k.geprueft
            .retain(|g| g.name.trim().to_lowercase() != name.to_lowercase());
        k.geprueft.push(openany_store::Geprueft {
            name: name.clone(),
            age,
        });
        k.geaendert_at = String::new();
        speicher
            .kontakt_schreiben(&k, openany_store::Protokoll::Merken)
            .map_err(|e| e.to_string())?;
    }
    Ok(sicherheit(&zustand, &client, &name).await?.0)
}

/// Die Pruefung zuruecknehmen.
#[tauri::command]
pub async fn sicherheitsnummer_aufheben(
    zustand: tauri::State<'_, Arc<Zustand>>,
    name: String,
) -> Result<Sicherheit, String> {
    let client = verbunden(&zustand).await?;
    let klein = name.trim().to_lowercase();
    {
        let speicher = zustand.speicher.lock().await;
        for mut k in speicher.kontakte().map_err(|e| e.to_string())? {
            let vorher = k.geprueft.len();
            k.geprueft.retain(|g| g.name.trim().to_lowercase() != klein);
            if k.geprueft.len() != vorher {
                k.geaendert_at = String::new();
                speicher
                    .kontakt_schreiben(&k, openany_store::Protokoll::Merken)
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(sicherheit(&zustand, &client, &name).await?.0)
}

/// Wie es um mehrere Namen steht (Abschnitt „Sicherheit" im Kontakt):
/// Name -> `geprueft`/`geaendert`/`offen`/`ohne_schluessel`.
#[tauri::command]
pub async fn sicherheitsnummer_stand(
    zustand: tauri::State<'_, Arc<Zustand>>,
    namen: Vec<String>,
) -> Result<HashMap<String, &'static str>, String> {
    let client = verbunden(&zustand).await?;
    let pruefungen = pruefungen(&zustand).await;
    let mut aus = HashMap::new();
    for name in namen.into_iter().take(20) {
        let sein = verzeichnis(&client, &name)
            .await
            .ok()
            .and_then(|v| v.konto)
            .map(|(_, a)| a);
        let stand = match &sein {
            None => "ohne_schluessel",
            Some(a) => stand_aus(&pruefungen, &name, Some(a)).unwrap_or("offen"),
        };
        aus.insert(name, stand);
    }
    Ok(aus)
}
