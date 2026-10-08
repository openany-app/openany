//! E-Mail -- der dritte Weg im Verlauf (docs/plan-email-pgp.md, Schritt 2).
//!
//! **Das Gerät spricht selbst mit dem Mailserver**, wie bei Matrix: IMAP zum
//! Lesen, SMTP zum Senden (`openany-post`). openany.de sieht keine Mail.
//! Zugangsdaten und Passwort liegen im Tresor (Android-Keystore).
//!
//! **Abgeholt wird, solange das Programm läuft**, alle drei Minuten und beim
//! Öffnen der Nachrichten. Die dauerhafte Verbindung (IMAP IDLE) im
//! Wachdienst ist Schritt 4.
//!
//! **Mehrere Postfächer, ein Verlauf.** Jede Mail weiß, zu welchem Postfach
//! sie gehört; eine Antwort geht von dort. Das erste in der Liste ist das
//! Standard-Postfach für neue Mails. Fällt eines aus (Passwort geändert),
//! holen die anderen weiter ab.

use crate::Zustand;
use anyid_client::Tokenspeicher;
use openany_post as post;
use openany_store::Mail;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

fn fehler(e: impl ToString) -> String {
    e.to_string()
}

/// Wie viele der neuesten Mails ein Ordner beim ersten Abholen liefert.
const ERSTMALS: usize = 50;
/// openanys Grenze für die Anhänge einer Mail, zusammen: „kleine Dateien".
pub const ANHAENGE_HOECHSTENS: u64 = 15 * 1024 * 1024;

/// Was im Tresor liegt, je Postfach: das Konto (samt Passwort) und die
/// gefundenen Ordner.
#[derive(Serialize, Deserialize, Clone)]
struct Postfach {
    konto: post::Konto,
    ordner: post::Ordner,
    /// Das eigene OpenPGP-Schlüsselpaar (Schritt 3a) -- entsperrt, denn der
    /// Tresor selbst ist verschlossen.
    #[serde(default)]
    pgp: Option<post::schluessel::EigenerSchluessel>,
}

impl Postfach {
    fn adresse(&self) -> &str {
        &self.konto.adresse
    }
}

/// Bis Stand 15 lag hier EIN Postfach als Objekt, jetzt eine Liste.
#[derive(Deserialize)]
#[serde(untagged)]
enum Abgelegt {
    Liste(Vec<Postfach>),
    Eines(Box<Postfach>),
}

fn ablage(zustand: &Zustand) -> Box<dyn Tokenspeicher + Send + Sync> {
    crate::tresor::ablage(zustand.ordner.join("ausweise").join("postfach"))
}

/// Alle Postfächer, das Standard-Postfach zuerst.
fn postfaecher(zustand: &Zustand) -> Vec<Postfach> {
    let Some(roh) = ablage(zustand).lesen().ok().flatten() else {
        return Vec::new();
    };
    match serde_json::from_str(&roh) {
        Ok(Abgelegt::Liste(l)) => l,
        Ok(Abgelegt::Eines(p)) => vec![*p],
        Err(_) => Vec::new(),
    }
}

fn postfach(zustand: &Zustand, adresse: &str) -> Option<Postfach> {
    postfaecher(zustand)
        .into_iter()
        .find(|p| p.adresse() == adresse)
}

fn speichern(zustand: &Zustand, liste: &[Postfach]) -> Result<(), String> {
    if liste.is_empty() {
        return ablage(zustand).vergessen().map_err(fehler);
    }
    ablage(zustand)
        .schreiben(&serde_json::to_string(liste).map_err(fehler)?)
        .map_err(fehler)
}

/// Der laufende Abholer und was zuletzt schiefging, je Postfach.
#[derive(Default)]
pub struct PostLage {
    aufgabe: Option<tauri::async_runtime::JoinHandle<()>>,
    fehler: std::collections::HashMap<String, String>,
    /// Wie viele Mails beim letzten Abholen im Spam-Ordner lagen.
    spam: std::collections::HashMap<String, u32>,
}

/* ── Was die Oberfläche sieht ─────────────────────────────────────────── */

#[derive(Serialize)]
pub struct PostfachLage {
    adresse: String,
    anzeigename: String,
    fehler: Option<String>,
    /// Mails im Spam-Ordner -- `None`, wenn es keinen gibt oder noch nicht
    /// abgeholt wurde.
    spam: Option<u32>,
    /// Der Fingerabdruck des eigenen OpenPGP-Schlüssels, wenn es einen gibt.
    pgp: Option<String>,
}

#[derive(Serialize)]
pub struct MailLage {
    /// Das Standard-Postfach zuerst.
    postfaecher: Vec<PostfachLage>,
    ungelesen: usize,
}

#[tauri::command]
pub async fn mail_lage(zustand: tauri::State<'_, Arc<Zustand>>) -> Result<MailLage, String> {
    let (fehler, spam) = {
        let lage = zustand.post.lock().await;
        (lage.fehler.clone(), lage.spam.clone())
    };
    Ok(MailLage {
        postfaecher: postfaecher(&zustand)
            .into_iter()
            .map(|p| PostfachLage {
                fehler: fehler.get(p.adresse()).cloned(),
                spam: spam.get(p.adresse()).copied(),
                pgp: p.pgp.as_ref().map(|k| k.fingerabdruck.clone()),
                adresse: p.konto.adresse,
                anzeigename: p.konto.anzeigename,
            })
            .collect(),
        ungelesen: zustand
            .speicher
            .lock()
            .await
            .ungelesene_mails()
            .unwrap_or(0),
    })
}

/// Die Server zu einer Adresse, aus der Autokonfiguration -- `None`, wenn
/// nichts zu finden ist (dann trägt der Mensch sie ein).
#[tauri::command]
pub async fn mail_server_finden(adresse: String) -> Result<Option<post::Gefunden>, String> {
    Ok(post::serverdaten_finden(&adresse).await)
}

/// Ein Postfach hinzufügen: anmelden, Ordner finden, im Tresor ablegen,
/// abholen. Das erste wird Standard.
#[tauri::command]
pub async fn mail_verbinden(
    app: tauri::AppHandle,
    zustand: tauri::State<'_, Arc<Zustand>>,
    konto: post::Konto,
) -> Result<MailLage, String> {
    let mut konto = konto;
    konto.adresse = konto.adresse.trim().to_lowercase();
    if konto.benutzer.trim().is_empty() {
        konto.benutzer = konto.adresse.clone();
    }
    if postfach(&zustand, &konto.adresse).is_some() {
        return Err("This mailbox is already connected.".into());
    }
    let ordner = post::pruefen(&konto).await.map_err(fehler)?;
    let adresse = konto.adresse.clone();
    let mut liste = postfaecher(&zustand);
    liste.push(Postfach {
        konto,
        ordner,
        pgp: None,
    });
    speichern(&zustand, &liste)?;
    // Läuft der Abholer schon (ein weiteres Postfach), das neue gleich
    // einmal abholen -- sonst käme es erst in drei Minuten dran.
    if !starten(app.clone(), zustand.inner().clone()).await {
        let z = zustand.inner().clone();
        tauri::async_runtime::spawn(async move {
            let _ = abholen_eines(&app, &z, &adresse).await;
        });
    }
    mail_lage(zustand).await
}

/// Ein Postfach trennen: Zugang weg, sein Verlauf weg. Auf dem Server bleibt
/// alles, wie es ist.
#[tauri::command]
pub async fn mail_trennen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    adresse: String,
) -> Result<(), String> {
    let mut liste = postfaecher(&zustand);
    liste.retain(|p| p.adresse() != adresse);
    speichern(&zustand, &liste)?;
    {
        let mut lage = zustand.post.lock().await;
        lage.fehler.remove(&adresse);
        lage.spam.remove(&adresse);
        if liste.is_empty() {
            if let Some(a) = lage.aufgabe.take() {
                a.abort();
            }
        }
    }
    zustand
        .speicher
        .lock()
        .await
        .mails_vergessen(&adresse)
        .map_err(fehler)?;
    Ok(())
}

/// Zum Standard-Postfach für neue Mails machen (nach vorn).
#[tauri::command]
pub async fn mail_standard(
    zustand: tauri::State<'_, Arc<Zustand>>,
    adresse: String,
) -> Result<MailLage, String> {
    let mut liste = postfaecher(&zustand);
    let i = liste
        .iter()
        .position(|p| p.adresse() == adresse)
        .ok_or("This mailbox is not connected.")?;
    let p = liste.remove(i);
    liste.insert(0, p);
    speichern(&zustand, &liste)?;
    mail_lage(zustand).await
}

/// Jetzt abholen -- ein Postfach oder alle (beim Öffnen der Nachrichten).
/// Gibt zurück, wie viele neu waren. Bei allen zählt ein Ausfall nicht als
/// Fehler des Ganzen; er steht in der Lage beim Postfach.
#[tauri::command]
pub async fn mail_abholen(
    app: tauri::AppHandle,
    zustand: tauri::State<'_, Arc<Zustand>>,
    adresse: Option<String>,
) -> Result<usize, String> {
    match adresse {
        Some(a) => abholen_eines(&app, &zustand, &a).await,
        None => Ok(abholen_alle(&app, &zustand).await),
    }
}

/* ── Abholen ───────────────────────────────────────────────────────────── */

/// Beim Start und nach dem Verbinden: alle drei Minuten abholen, solange
/// das Programm läuft. `true`, wenn der Abholer jetzt erst anläuft (dann
/// holt er gleich alle ab).
pub async fn starten(app: tauri::AppHandle, zustand: Arc<Zustand>) -> bool {
    let liste = postfaecher(&zustand);
    let Some(erstes) = liste.first() else {
        return false;
    };
    // Einmal nach Stand 15: Was noch keinem Postfach gehört, stammt aus der
    // Zeit mit genau einem -- und das steht jetzt vorn. Die alte Form im
    // Tresor gleich als Liste zurückschreiben.
    let _ = zustand
        .speicher
        .lock()
        .await
        .mails_zuordnen(erstes.adresse());
    let _ = speichern(&zustand, &liste);

    let mut lage = zustand.post.lock().await;
    if lage
        .aufgabe
        .as_ref()
        .is_some_and(|a| !a.inner().is_finished())
    {
        return false;
    }
    let z = zustand.clone();
    lage.aufgabe = Some(tauri::async_runtime::spawn(async move {
        loop {
            abholen_alle(&app, &z).await;
            tokio::time::sleep(std::time::Duration::from_secs(180)).await;
        }
    }));
    true
}

async fn abholen_alle(app: &tauri::AppHandle, zustand: &Arc<Zustand>) -> usize {
    let mut neu = 0;
    for p in postfaecher(zustand) {
        neu += abholen_eines(app, zustand, p.adresse()).await.unwrap_or(0);
    }
    neu
}

async fn abholen_eines(
    app: &tauri::AppHandle,
    zustand: &Arc<Zustand>,
    adresse: &str,
) -> Result<usize, String> {
    match abholen_still(zustand, adresse).await {
        Ok((neue, anders)) => {
            if !neue.is_empty() || anders {
                crate::nachrichtenbefehle::melden(app);
            }
            Ok(neue.len())
        }
        Err(e) => {
            crate::nachrichtenbefehle::melden(app);
            Err(e)
        }
    }
}

/// Abholen ohne Fenster -- auch für den Wachdienst, der keines hat.
/// Zurück kommen die neu abgelegten Mails und ob sich sonst etwas an der
/// Lage geändert hat (Fehler weg, Spam-Zahl anders).
pub(crate) async fn abholen_still(
    zustand: &Arc<Zustand>,
    adresse: &str,
) -> Result<(Vec<Mail>, bool), String> {
    let p = postfach(zustand, adresse).ok_or("This mailbox is not connected.")?;
    let staende: Vec<post::Stand> = zustand
        .speicher
        .lock()
        .await
        .mail_staende(adresse)
        .map_err(fehler)?
        .into_iter()
        .map(|s| post::Stand {
            ordner: s.ordner,
            uidvalidity: s.uidvalidity as u32,
            letzte_uid: s.letzte_uid as u32,
        })
        .collect();

    let abgeholt = match post::abholen(&p.konto, &staende, ERSTMALS).await {
        Ok(a) => a,
        Err(e) => {
            zustand
                .post
                .lock()
                .await
                .fehler
                .insert(adresse.to_string(), e.to_string());
            return Err(e.to_string());
        }
    };

    let mut neu = Vec::new();
    for roh in &abgeholt.mails {
        let Some((zeile, autocrypt)) = aus_roh(zustand, &p, roh).await else {
            continue;
        };
        let speicher = zustand.speicher.lock().await;
        if speicher.mail_ablegen(&zeile).unwrap_or(false) {
            neu.push(zeile.clone());
        }
        // Der Schlüssel des Absenders aus seiner Autocrypt-Kopfzeile.
        if let Some(k) = autocrypt {
            let _ = speicher.pgp_ablegen(
                &zeile.gegenueber,
                &k.fingerabdruck,
                &k.oeffentlich,
                "autocrypt",
            );
        }
    }
    {
        let speicher = zustand.speicher.lock().await;
        for s in &abgeholt.staende {
            let _ = speicher.mail_stand_setzen(&openany_store::MailStand {
                postfach: adresse.to_string(),
                ordner: s.ordner.clone(),
                uidvalidity: s.uidvalidity as i64,
                letzte_uid: s.letzte_uid as i64,
            });
        }
    }
    // Die Ordner merken, falls der Server sie umbenannt hat.
    if abgeholt.ordner != p.ordner {
        let mut liste = postfaecher(zustand);
        if let Some(q) = liste.iter_mut().find(|q| q.adresse() == adresse) {
            q.ordner = abgeholt.ordner.clone();
            let _ = speichern(zustand, &liste);
        }
    }
    let (hatte_fehler, spam_anders) = {
        let mut lage = zustand.post.lock().await;
        let vorher = match abgeholt.spam {
            Some(n) => lage.spam.insert(adresse.to_string(), n),
            None => lage.spam.remove(adresse),
        };
        (
            lage.fehler.remove(adresse).is_some(),
            vorher != abgeholt.spam,
        )
    };
    Ok((neu, hatte_fehler || spam_anders))
}

/// Die Adressen der verbundenen Postfächer (für den Wachdienst).
pub(crate) fn adressen(zustand: &Zustand) -> Vec<String> {
    postfaecher(zustand)
        .into_iter()
        .map(|p| p.konto.adresse)
        .collect()
}

/// Das Konto eines Postfachs und der Name seines Posteingangs.
pub(crate) fn konto_und_eingang(zustand: &Zustand, adresse: &str) -> Option<(post::Konto, String)> {
    postfach(zustand, adresse).map(|p| (p.konto, p.ordner.eingang))
}

/// Anhänge in die Inhaltsablage; zurück kommt die Liste für die Zeile.
fn anhaenge_ablegen(zustand: &Zustand, anhaenge: &[post::AnhangDaten]) -> String {
    let liste: Vec<serde_json::Value> = anhaenge
        .iter()
        .filter_map(|a| {
            let mut ladung = zustand.inhalte.ladung().ok()?;
            ladung.schreiben(&a.daten).ok()?;
            let (abdruck, groesse) = zustand.inhalte.ablegen(ladung).ok()?;
            Some(serde_json::json!({ "name": a.name, "mime": a.mime, "groesse": groesse, "abdruck": abdruck }))
        })
        .collect();
    serde_json::to_string(&liste).unwrap_or_else(|_| "[]".into())
}

/// Aus einer abgeholten Mail eine Zeile -- und bei einer fremden der
/// Schlüssel aus ihrer Autocrypt-Kopfzeile, wenn sie eine trägt.
async fn aus_roh(
    zustand: &Zustand,
    p: &Postfach,
    roh: &post::Roh,
) -> Option<(Mail, Option<post::schluessel::FremderSchluessel>)> {
    let m = post::mail_lesen(&roh.daten)?;
    let eigene = p.konto.adresse.to_lowercase();
    let im_gesendet = p.ordner.gesendet.as_deref() == Some(roh.ordner.as_str());
    let von_mir = im_gesendet || m.von.as_ref().is_some_and(|v| v.adresse == eigene);
    let gegenueber = if von_mir {
        m.an.first().cloned()
    } else {
        m.von.clone()
    };
    let zeit = m
        .zeit
        .and_then(|s| chrono::DateTime::from_timestamp(s, 0))
        .unwrap_or_else(chrono::Utc::now)
        .to_rfc3339();
    let autocrypt = match (&m.autocrypt, &m.von) {
        (Some(kopf), Some(von)) if !von_mir => {
            post::schluessel::autocrypt_lesen(kopf, &von.adresse)
        }
        _ => None,
    };

    // VERSCHLÜSSELT: gleich hier entschlüsseln. Geprüft wird gegen den
    // Schlüssel des Absenders -- den aus dieser Mail, sonst den bekannten;
    // bei einer eigenen gegen den eigenen.
    let (betreff, text, anhaenge, pgp, signatur) = match &m.pgp {
        None => (
            m.betreff.clone(),
            m.text.clone(),
            m.anhaenge.clone(),
            None,
            None,
        ),
        Some(zu) => {
            let absender = if von_mir {
                p.pgp.as_ref().map(|k| k.oeffentlich.clone())
            } else if let Some(k) = &autocrypt {
                Some(k.oeffentlich.clone())
            } else {
                let von = m
                    .von
                    .as_ref()
                    .map(|v| v.adresse.clone())
                    .unwrap_or_default();
                zustand
                    .speicher
                    .lock()
                    .await
                    .pgp_schluessel(&von)
                    .ok()
                    .flatten()
                    .map(|k| k.oeffentlich)
            };
            match p.pgp.as_ref().and_then(|k| {
                post::pgpmime::entschluesseln(&zu.nachricht, &k.geheim, absender.as_deref()).ok()
            }) {
                Some((klar, sig)) => {
                    let (b, t, a) = inhalt_aus(&klar, &m.betreff);
                    (b, t, a, Some("verschluesselt"), sig.als_text())
                }
                // Ohne passenden Schlüssel: die Nachricht aufheben, damit
                // sie sich lesen lässt, sobald er da ist.
                None => (
                    m.betreff.clone(),
                    String::from_utf8_lossy(&zu.nachricht).to_string(),
                    Vec::new(),
                    Some("unlesbar"),
                    None,
                ),
            }
        }
    };

    let zeile = Mail {
        id: uuid::Uuid::new_v4().to_string(),
        postfach: p.konto.adresse.clone(),
        ordner: roh.ordner.clone(),
        uidvalidity: roh.uidvalidity as i64,
        uid: roh.uid as i64,
        message_id: m.message_id.clone(),
        in_reply_to: m.in_reply_to.clone(),
        references: serde_json::to_string(&m.references).unwrap_or_else(|_| "[]".into()),
        von_mir,
        gegenueber: gegenueber
            .as_ref()
            .map(|g| g.adresse.clone())
            .unwrap_or_default(),
        gegenueber_name: gegenueber.and_then(|g| g.name).unwrap_or_default(),
        betreff,
        text,
        zeit,
        // Im Mailprogramm schon gelesen, oder die eigene: gelesen.
        gelesen_at: (roh.gelesen || von_mir).then(|| chrono::Utc::now().to_rfc3339()),
        anhaenge: anhaenge_ablegen(zustand, &anhaenge),
        pgp: pgp.map(str::to_string),
        signatur: signatur.map(str::to_string),
    };
    Some((zeile, autocrypt))
}

/// Was in einer entschlüsselten Nachricht steht: bei PGP/MIME ein MIME-Teil
/// (mit dem echten Betreff, wenn er geschützt mitkam), inline nur Text.
fn inhalt_aus(klar: &[u8], betreff_aussen: &str) -> (String, String, Vec<post::AnhangDaten>) {
    let kopf = klar
        .iter()
        .take(64)
        .map(|b| b.to_ascii_lowercase())
        .collect::<Vec<_>>();
    let ist_mime = kopf.starts_with(b"content-") || kopf.starts_with(b"mime-version");
    match ist_mime.then(|| post::mail_lesen(klar)).flatten() {
        Some(innen) => (
            if innen.betreff.trim().is_empty() {
                betreff_aussen.to_string()
            } else {
                innen.betreff
            },
            innen.text,
            innen.anhaenge,
        ),
        None => (
            betreff_aussen.to_string(),
            String::from_utf8_lossy(klar).trim_end().to_string(),
            Vec::new(),
        ),
    }
}

/// Nach einem neuen eigenen Schlüssel: die Mails, die bisher unlesbar waren,
/// noch einmal versuchen.
async fn unlesbare_nachholen(zustand: &Zustand, adresse: &str) {
    let Some(k) = postfach(zustand, adresse).and_then(|p| p.pgp) else {
        return;
    };
    let liste = zustand
        .speicher
        .lock()
        .await
        .unlesbare_mails(adresse)
        .unwrap_or_default();
    for m in liste {
        let absender = if m.von_mir {
            Some(k.oeffentlich.clone())
        } else {
            zustand
                .speicher
                .lock()
                .await
                .pgp_schluessel(&m.gegenueber)
                .ok()
                .flatten()
                .map(|k| k.oeffentlich)
        };
        let Ok((klar, sig)) =
            post::pgpmime::entschluesseln(m.text.as_bytes(), &k.geheim, absender.as_deref())
        else {
            continue;
        };
        let (betreff, text, anhaenge) = inhalt_aus(&klar, &m.betreff);
        let anhaenge = anhaenge_ablegen(zustand, &anhaenge);
        let _ = zustand.speicher.lock().await.mail_entschluesselt(
            &m.id,
            &betreff,
            &text,
            &anhaenge,
            sig.als_text(),
        );
    }
}

/* ── Senden ────────────────────────────────────────────────────────────── */

#[derive(Deserialize)]
pub struct AnhangHin {
    name: String,
    mime: String,
    /// Base64 -- auf Android kommen Bytes sonst nicht als Bytes an.
    daten: String,
}

/// Eine Mail senden, optional als Antwort auf `antwort_auf` (eine Zeile im
/// Verlauf) und mit Anhängen. Danach in „Gesendet" ablegen und in den
/// Verlauf.
///
/// Von welchem Postfach: bei einer Antwort von dem, an das die Mail kam;
/// sonst `von`, sonst dem Standard-Postfach.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn mail_senden(
    app: tauri::AppHandle,
    zustand: tauri::State<'_, Arc<Zustand>>,
    an: String,
    betreff: String,
    text: String,
    anhaenge: Option<Vec<AnhangHin>>,
    antwort_auf: Option<String>,
    von: Option<String>,
    verschluesseln: Option<bool>,
) -> Result<(), String> {
    use base64::Engine;
    let an: Vec<String> = an
        .split([',', ';'])
        .map(|a| a.trim().to_lowercase())
        .filter(|a| !a.is_empty())
        .collect();
    if an.is_empty() || an.iter().any(|a| !a.contains('@')) {
        return Err("Please enter a valid email address.".into());
    }

    let mut daten = Vec::new();
    for a in anhaenge.unwrap_or_default() {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(a.daten.as_bytes())
            .map_err(|_| "An attachment arrived unreadable.".to_string())?;
        daten.push(post::AnhangDaten {
            name: a.name,
            mime: a.mime,
            daten: bytes,
        });
    }
    if daten.iter().map(|a| a.daten.len() as u64).sum::<u64>() > ANHAENGE_HOECHSTENS {
        return Err("The attachments are too large: at most 15 MB in total.".into());
    }

    let vorher = match antwort_auf.as_deref() {
        Some(id) => zustand.speicher.lock().await.mail(id).map_err(fehler)?,
        None => None,
    };
    let liste = postfaecher(&zustand);
    let gewuenscht = vorher.as_ref().map(|v| v.postfach.clone()).or(von);
    let p = gewuenscht
        .and_then(|a| liste.iter().find(|p| p.adresse() == a))
        .or(liste.first())
        .cloned()
        .ok_or("No mailbox connected.")?;
    let entwurf = post::Entwurf {
        an: an.clone(),
        betreff: betreff.trim().to_string(),
        text: text.clone(),
        anhaenge: daten.clone(),
        antwort_auf: vorher
            .as_ref()
            .map(|v| v.message_id.clone())
            .filter(|m| !m.is_empty()),
        references: vorher
            .as_ref()
            .and_then(|v| serde_json::from_str(&v.references).ok())
            .unwrap_or_default(),
        // Mit eigenem Schlüssel trägt jede Mail ihn als Autocrypt-Kopfzeile:
        // So findet das Gegenüber ihn, ohne danach zu fragen.
        autocrypt: p
            .pgp
            .as_ref()
            .and_then(|k| post::schluessel::autocrypt_kopf(p.adresse(), &k.oeffentlich).ok()),
        verschluesselung: None,
    };
    // VERSCHLÜSSELT NUR AUF WUNSCH (Tiffy, 29.09.2026) -- und dann ganz oder
    // gar nicht: Fehlt ein Schlüssel, geht die Mail nicht still im Klartext.
    let mut entwurf = entwurf;
    if verschluesseln.unwrap_or(false) {
        let eigen = p
            .pgp
            .as_ref()
            .ok_or("This mailbox has no key of its own yet (Settings).")?;
        let mut empfaenger = Vec::new();
        for a in &an {
            let (_, k) = empfaenger_schluessel(&zustand, a)
                .await
                .ok_or_else(|| format!("No key is known for {a}."))?;
            empfaenger.push(k);
        }
        entwurf.verschluesselung = Some(post::Verschluesselung {
            empfaenger,
            eigener_geheim: eigen.geheim.clone(),
        });
    }
    let gesendet = post::senden(&p.konto, &entwurf).await.map_err(fehler)?;

    // In „Gesendet" ablegen. Scheitert das, ist die Mail trotzdem raus --
    // nur das Mailprogramm sähe sie nicht. Kein Grund, das Senden als
    // gescheitert zu melden.
    if let Some(ordner) = p.ordner.gesendet.as_deref() {
        let _ = post::ablegen(&p.konto, ordner, &gesendet.roh).await;
    }

    let jetzt = chrono::Utc::now().to_rfc3339();
    let zeile = Mail {
        id: uuid::Uuid::new_v4().to_string(),
        postfach: p.konto.adresse.clone(),
        message_id: gesendet.message_id,
        in_reply_to: entwurf.antwort_auf.clone(),
        references: serde_json::to_string(&entwurf.references).unwrap_or_else(|_| "[]".into()),
        von_mir: true,
        gegenueber: an[0].clone(),
        betreff: entwurf.betreff,
        text,
        zeit: jetzt.clone(),
        gelesen_at: Some(jetzt),
        anhaenge: anhaenge_ablegen(&zustand, &daten),
        pgp: entwurf
            .verschluesselung
            .is_some()
            .then(|| "verschluesselt".to_string()),
        ..Default::default()
    };
    zustand
        .speicher
        .lock()
        .await
        .mail_ablegen(&zeile)
        .map_err(fehler)?;
    crate::nachrichtenbefehle::melden(&app);
    Ok(())
}

/* ── Gelesen, löschen, Anhang ─────────────────────────────────────────── */

/// Hier gelesen -- und im Hintergrund auch auf dem Server (`\Seen`), damit
/// das Mailprogramm dasselbe zeigt.
pub async fn gelesen(zustand: &Arc<Zustand>, id: &str) -> Result<(), String> {
    let zeile = zustand.speicher.lock().await.mail(id).map_err(fehler)?;
    zustand
        .speicher
        .lock()
        .await
        .mail_gelesen(id)
        .map_err(fehler)?;
    if let Some((z, p)) = zeile.and_then(|z| postfach(zustand, &z.postfach).map(|p| (z, p))) {
        if !z.ordner.is_empty() && !z.von_mir {
            tauri::async_runtime::spawn(async move {
                let _ = post::als_gelesen(&p.konto, &z.ordner, z.uid as u32).await;
            });
        }
    }
    Ok(())
}

/// Ausblenden -- und mit `auch_server` erst auf dem Server löschen (in dessen
/// Papierkorb-Ordner). Scheitert das, bleibt sie auch hier stehen.
pub async fn loeschen(zustand: &Arc<Zustand>, id: &str, auch_server: bool) -> Result<(), String> {
    let zeile = zustand
        .speicher
        .lock()
        .await
        .mail(id)
        .map_err(fehler)?
        .ok_or("This mail does not exist here.")?;
    if auch_server {
        let p = postfach(zustand, &zeile.postfach)
            .ok_or("The mailbox of this mail is no longer connected.")?;
        let papierkorb = p.ordner.papierkorb.as_deref();
        let mut getroffen = 0;
        if !zeile.ordner.is_empty() {
            post::loeschen(&p.konto, &zeile.ordner, zeile.uid as u32, papierkorb)
                .await
                .map_err(fehler)?;
            getroffen += 1;
        }
        // Eine eigene kann noch woanders liegen: in „Gesendet", solange sie
        // von dort nicht zurückkam, und bei einer Mail an sich selbst
        // zusätzlich im Posteingang.
        if zeile.von_mir {
            let mut orte: Vec<&str> = p.ordner.gesendet.as_deref().into_iter().collect();
            if zeile.gegenueber == p.konto.adresse.to_lowercase() {
                orte.push(&p.ordner.eingang);
            }
            for ort in orte.into_iter().filter(|o| *o != zeile.ordner) {
                getroffen += post::loeschen_nach_id(&p.konto, ort, &zeile.message_id, papierkorb)
                    .await
                    .map_err(fehler)?;
            }
        }
        if getroffen == 0 {
            return Err("This mail could not be found on the mail server. \"Only hide here\" removes it from the timeline.".into());
        }
    }
    zustand
        .speicher
        .lock()
        .await
        .mail_loeschen(id)
        .map_err(fehler)?;
    Ok(())
}

/// Die Bytes eines Mail-Anhangs, als Base64.
pub async fn anhang(zustand: &Arc<Zustand>, id: &str, index: usize) -> Result<String, String> {
    use base64::Engine;
    let zeile = zustand
        .speicher
        .lock()
        .await
        .mail(id)
        .map_err(fehler)?
        .ok_or("This mail does not exist here.")?;
    let liste: Vec<serde_json::Value> = serde_json::from_str(&zeile.anhaenge).unwrap_or_default();
    let abdruck = liste
        .get(index)
        .and_then(|a| a.get("abdruck"))
        .and_then(|a| a.as_str())
        .ok_or("This attachment does not exist.")?;
    let bytes = zustand
        .inhalte
        .lesen(abdruck, 0, (ANHAENGE_HOECHSTENS * 4) as usize)
        .map_err(fehler)?;
    Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
}

/* ── Spamverdacht ──────────────────────────────────────────────────────── */
//
// NICHT IM VERLAUF (Tiffy, 29.09.2026). Was der Anbieter als Spam einsortiert,
// steht in einer eigenen Liste, die bei jedem Öffnen frisch vom Server kommt
// -- nichts davon landet im Speicher. „Kein Spam" verschiebt die Mail in den
// Posteingang; von dort holt sie das nächste Abholen wie jede andere.

/// Wie viele der neuesten Spam-Mails die Liste zeigt.
const SPAM_HOECHSTENS: usize = 50;

#[derive(Serialize)]
pub struct SpamZeile {
    uid: u32,
    von: String,
    von_name: String,
    betreff: String,
    /// ISO-8601.
    zeit: String,
    /// Der Anfang des Textes.
    text: String,
    /// Nur die Namen: Anhänge aus dem Spam-Ordner öffnet die App nicht.
    anhaenge: Vec<String>,
}

fn spam_ort(zustand: &Zustand, adresse: &str) -> Result<(Postfach, String), String> {
    let p = postfach(zustand, adresse).ok_or("This mailbox is not connected.")?;
    let ordner = p
        .ordner
        .spam
        .clone()
        .ok_or("This mailbox has no spam folder.")?;
    Ok((p, ordner))
}

#[tauri::command]
pub async fn mail_spam(
    zustand: tauri::State<'_, Arc<Zustand>>,
    adresse: String,
) -> Result<Vec<SpamZeile>, String> {
    let (p, ordner) = spam_ort(&zustand, &adresse)?;
    let roh = post::neueste(&p.konto, &ordner, SPAM_HOECHSTENS)
        .await
        .map_err(fehler)?;
    let liste: Vec<SpamZeile> = roh
        .into_iter()
        .filter_map(|r| {
            let m = post::mail_lesen(&r.daten)?;
            Some(SpamZeile {
                uid: r.uid,
                von: m
                    .von
                    .as_ref()
                    .map(|v| v.adresse.clone())
                    .unwrap_or_default(),
                von_name: m.von.and_then(|v| v.name).unwrap_or_default(),
                betreff: m.betreff,
                zeit: m
                    .zeit
                    .and_then(|s| chrono::DateTime::from_timestamp(s, 0))
                    .unwrap_or_else(chrono::Utc::now)
                    .to_rfc3339(),
                text: m.text.chars().take(400).collect(),
                anhaenge: m.anhaenge.into_iter().map(|a| a.name).collect(),
            })
        })
        .collect();
    // Die Zahl gleich angleichen -- die Liste ist frischer als das letzte
    // Abholen.
    if liste.len() < SPAM_HOECHSTENS {
        zustand
            .post
            .lock()
            .await
            .spam
            .insert(adresse, liste.len() as u32);
    }
    Ok(liste)
}

/// „Kein Spam": in den Posteingang verschieben und gleich abholen, damit sie
/// im Verlauf steht.
#[tauri::command]
pub async fn mail_kein_spam(
    app: tauri::AppHandle,
    zustand: tauri::State<'_, Arc<Zustand>>,
    adresse: String,
    uid: u32,
) -> Result<(), String> {
    let (p, ordner) = spam_ort(&zustand, &adresse)?;
    post::verschieben(&p.konto, &ordner, uid, &p.ordner.eingang)
        .await
        .map_err(fehler)?;
    let _ = abholen_eines(&app, &zustand, &adresse).await;
    Ok(())
}

/// Aus dem Spam-Ordner löschen (in den Papierkorb-Ordner, sonst endgültig).
#[tauri::command]
pub async fn mail_spam_loeschen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    adresse: String,
    uid: u32,
) -> Result<(), String> {
    let (p, ordner) = spam_ort(&zustand, &adresse)?;
    post::loeschen(&p.konto, &ordner, uid, p.ordner.papierkorb.as_deref())
        .await
        .map_err(fehler)?;
    if let Some(n) = zustand.post.lock().await.spam.get_mut(&adresse) {
        *n = n.saturating_sub(1);
    }
    Ok(())
}

/* ── OpenPGP-Schlüssel (Schritt 3a) ────────────────────────────────────── */

/// Den eigenen Schlüssel eines Postfachs setzen (oder mit `None` entfernen).
fn eigenen_setzen(
    zustand: &Zustand,
    adresse: &str,
    k: Option<post::schluessel::EigenerSchluessel>,
) -> Result<(), String> {
    let mut liste = postfaecher(zustand);
    let p = liste
        .iter_mut()
        .find(|p| p.adresse() == adresse)
        .ok_or("This mailbox is not connected.")?;
    p.pgp = k;
    speichern(zustand, &liste)
}

/// Ein neues Schlüsselpaar für ein Postfach erzeugen.
#[tauri::command]
pub async fn mail_pgp_erzeugen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    adresse: String,
) -> Result<MailLage, String> {
    let p = postfach(&zustand, &adresse).ok_or("This mailbox is not connected.")?;
    if p.pgp.is_some() {
        return Err("This mailbox already has a key.".into());
    }
    let name = p.konto.anzeigename.clone();
    let k =
        tauri::async_runtime::spawn_blocking(move || post::schluessel::erzeugen(&adresse, &name))
            .await
            .map_err(fehler)?
            .map_err(fehler)?;
    eigenen_setzen(&zustand, p.adresse(), Some(k))?;
    unlesbare_nachholen(&zustand, p.adresse()).await;
    mail_lage(zustand).await
}

/// Einen vorhandenen geheimen Schlüssel einlesen (etwa aus Thunderbird),
/// als Text (armored). Er muss die Adresse des Postfachs tragen.
#[tauri::command]
pub async fn mail_pgp_einlesen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    adresse: String,
    daten: String,
    passphrase: String,
) -> Result<MailLage, String> {
    let p = postfach(&zustand, &adresse).ok_or("This mailbox is not connected.")?;
    let k = post::schluessel::einlesen(daten.as_bytes(), &passphrase).map_err(fehler)?;
    let adressen = post::schluessel::oeffentlich_lesen(k.oeffentlich.as_bytes())
        .map_err(fehler)?
        .adressen;
    if !adressen.iter().any(|a| a == p.adresse()) {
        return Err(format!(
            "This key belongs to {}, not to {}.",
            if adressen.is_empty() {
                "no address".to_string()
            } else {
                adressen.join(", ")
            },
            p.adresse()
        ));
    }
    eigenen_setzen(&zustand, p.adresse(), Some(k))?;
    unlesbare_nachholen(&zustand, p.adresse()).await;
    mail_lage(zustand).await
}

/// Den eigenen Schlüssel entfernen. Ohne Ausfuhr ist er danach fort, und
/// an ihn verschlüsselte Mails lassen sich hier nicht mehr lesen.
#[tauri::command]
pub async fn mail_pgp_entfernen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    adresse: String,
) -> Result<MailLage, String> {
    eigenen_setzen(&zustand, &adresse, None)?;
    mail_lage(zustand).await
}

/// Ein bekannter Schlüssel eines Gegenübers, für die Oberfläche.
#[derive(Serialize)]
pub struct PgpZeile {
    adresse: String,
    fingerabdruck: String,
    quelle: String,
    aktualisiert_at: String,
    /// Der Fingerabdruck davor, wenn er sich geändert hat.
    vorher: Option<String>,
}

fn pgp_zeile(k: openany_store::PgpSchluessel) -> PgpZeile {
    PgpZeile {
        adresse: k.adresse,
        fingerabdruck: k.fingerabdruck,
        quelle: k.quelle,
        aktualisiert_at: k.aktualisiert_at,
        vorher: k.vorher,
    }
}

#[tauri::command]
pub async fn mail_pgp_liste(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Vec<PgpZeile>, String> {
    Ok(zustand
        .speicher
        .lock()
        .await
        .pgp_liste()
        .map_err(fehler)?
        .into_iter()
        .map(pgp_zeile)
        .collect())
}

/// Den Schlüssel zu einer Adresse suchen: erst hier, dann beim Mailanbieter
/// des Gegenübers (WKD), und nur mit `schluesselserver` auch bei
/// keys.openpgp.org (Tiffy, 29.09.2026). `None`, wenn nirgends einer ist.
#[tauri::command]
pub async fn mail_pgp_suchen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    adresse: String,
    schluesselserver: Option<bool>,
) -> Result<Option<PgpZeile>, String> {
    let adresse = adresse.trim().to_lowercase();
    if !adresse.contains('@') {
        return Err("Please enter an email address.".into());
    }
    if let Some(k) = zustand
        .speicher
        .lock()
        .await
        .pgp_schluessel(&adresse)
        .map_err(fehler)?
    {
        if !schluesselserver.unwrap_or(false) {
            return Ok(Some(pgp_zeile(k)));
        }
    }
    let (gefunden, quelle) = match post::schluessel::wkd_suchen(&adresse).await {
        Some(k) => (Some(k), "wkd"),
        None if schluesselserver.unwrap_or(false) => (
            post::schluessel::schluesselserver_suchen(&adresse).await,
            "keys.openpgp.org",
        ),
        None => (None, ""),
    };
    let speicher = zustand.speicher.lock().await;
    if let Some(k) = gefunden {
        speicher
            .pgp_ablegen(&adresse, &k.fingerabdruck, &k.oeffentlich, quelle)
            .map_err(fehler)?;
    }
    Ok(speicher
        .pgp_schluessel(&adresse)
        .map_err(fehler)?
        .map(pgp_zeile))
}

/// Einen öffentlichen Schlüssel von Hand einlesen (Text, armored). Er wird
/// unter jeder Adresse abgelegt, die er trägt.
#[tauri::command]
pub async fn mail_pgp_hand(
    zustand: tauri::State<'_, Arc<Zustand>>,
    daten: String,
) -> Result<Vec<PgpZeile>, String> {
    let k = post::schluessel::oeffentlich_lesen(daten.as_bytes()).map_err(fehler)?;
    if k.adressen.is_empty() {
        return Err("This key carries no email address.".into());
    }
    let speicher = zustand.speicher.lock().await;
    let mut zeilen = Vec::new();
    for a in &k.adressen {
        speicher
            .pgp_ablegen(a, &k.fingerabdruck, &k.oeffentlich, "hand")
            .map_err(fehler)?;
        if let Some(z) = speicher.pgp_schluessel(a).map_err(fehler)? {
            zeilen.push(pgp_zeile(z));
        }
    }
    Ok(zeilen)
}

#[tauri::command]
pub async fn mail_pgp_vergessen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    adresse: String,
) -> Result<(), String> {
    zustand
        .speicher
        .lock()
        .await
        .pgp_loeschen(&adresse)
        .map_err(fehler)?;
    Ok(())
}

/// Der Hinweis auf einen geänderten Schlüssel ist gesehen.
#[tauri::command]
pub async fn mail_pgp_wechsel_gesehen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    adresse: String,
) -> Result<(), String> {
    zustand
        .speicher
        .lock()
        .await
        .pgp_wechsel_gesehen(&adresse)
        .map_err(fehler)?;
    Ok(())
}

/// Was das Schreibfeld über eine verschlüsselte Mail wissen muss: Hat das
/// Postfach einen eigenen Schlüssel, und ist der des Empfängers bekannt?
/// Ist er es nicht, wird beim Mailanbieter gesucht (WKD) -- das ist die
/// erlaubte automatische Suche; keys.openpgp.org nicht.
#[derive(Serialize)]
pub struct PgpStatus {
    eigener: bool,
    /// Der Fingerabdruck des Empfängers, wenn bekannt.
    empfaenger: Option<String>,
}

#[tauri::command]
pub async fn mail_pgp_status(
    zustand: tauri::State<'_, Arc<Zustand>>,
    von: Option<String>,
    an: String,
) -> Result<PgpStatus, String> {
    let liste = postfaecher(&zustand);
    let p = von
        .and_then(|v| liste.iter().find(|p| p.adresse() == v))
        .or(liste.first());
    let eigener = p.is_some_and(|p| p.pgp.is_some());
    let an = an.trim().to_lowercase();
    if !an.contains('@') {
        return Ok(PgpStatus {
            eigener,
            empfaenger: None,
        });
    }
    let empfaenger = match empfaenger_schluessel(&zustand, &an).await {
        Some((f, _)) => Some(f),
        None => match post::schluessel::wkd_suchen(&an).await {
            Some(k) => {
                zustand
                    .speicher
                    .lock()
                    .await
                    .pgp_ablegen(&an, &k.fingerabdruck, &k.oeffentlich, "wkd")
                    .map_err(fehler)?;
                Some(k.fingerabdruck)
            }
            None => None,
        },
    };
    Ok(PgpStatus {
        eigener,
        empfaenger,
    })
}

/// Der öffentliche Schlüssel eines Empfängers: `(Fingerabdruck, armored)`.
/// Aus der Ablage -- oder, ist die Adresse ein eigenes Postfach mit
/// Schlüssel, dessen eigener (an sich selbst, zwischen eigenen Postfächern).
async fn empfaenger_schluessel(zustand: &Zustand, adresse: &str) -> Option<(String, String)> {
    let adresse = adresse.trim().to_lowercase();
    if let Some(k) = postfach(zustand, &adresse).and_then(|p| p.pgp) {
        return Some((k.fingerabdruck, k.oeffentlich));
    }
    zustand
        .speicher
        .lock()
        .await
        .pgp_schluessel(&adresse)
        .ok()
        .flatten()
        .map(|k| (k.fingerabdruck, k.oeffentlich))
}

/// Den eigenen geheimen Schlüssel als Datei ausführen, gesperrt mit einer
/// Passphrase -- die Sicherungskopie. Zurück kommt der Text (armored); wohin
/// er gelegt wird, entscheidet die Oberfläche (auf Android: „Download",
/// nicht „Dateien", denn die gehen in den Abgleich).
#[tauri::command]
pub async fn mail_pgp_ausfuhr(
    zustand: tauri::State<'_, Arc<Zustand>>,
    adresse: String,
    passphrase: String,
) -> Result<String, String> {
    let k = postfach(&zustand, &adresse)
        .and_then(|p| p.pgp)
        .ok_or("This mailbox has no key of its own.")?;
    tauri::async_runtime::spawn_blocking(move || post::schluessel::ausfuhr(&k.geheim, &passphrase))
        .await
        .map_err(fehler)?
        .map_err(fehler)
}

/// Der eigene öffentliche Schlüssel, zum Weitergeben.
#[tauri::command]
pub async fn mail_pgp_oeffentlich(
    zustand: tauri::State<'_, Arc<Zustand>>,
    adresse: String,
) -> Result<String, String> {
    postfach(&zustand, &adresse)
        .and_then(|p| p.pgp)
        .map(|k| k.oeffentlich)
        .ok_or_else(|| "This mailbox has no key of its own.".to_string())
}
