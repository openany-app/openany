//! Die anrufende Seite: ein anderes Geraet ansprechen, dessen Fingerabdruck
//! man erwartet. Stimmt er nicht, scheitert schon der TLS-Handschlag -- es
//! geht kein einziges Byte der Anfrage hinaus.

use crate::dienst::{
    abschliessen, Anfrage, Antwort, Gastgeber, GeheimnisAnfrage, GemeinsamePaarungen, Stand,
};
use crate::tls::anruf_konfiguration;
use crate::Identitaet;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub(crate) fn wurzel(adresse: &str, port: u16) -> String {
    if adresse.contains(':') {
        // IPv6: in Klammern, ohne Zonenangabe -- die versteht die Adresszeile nicht.
        let ohne_zone = adresse.split('%').next().unwrap_or(adresse);
        format!("https://[{ohne_zone}]:{port}")
    } else {
        format!("https://{adresse}:{port}")
    }
}

fn client(ident: &Identitaet, erwartet: &str) -> Result<reqwest::Client, String> {
    let tls = anruf_konfiguration(&ident.zertifikat_pem, &ident.schluessel_pem, erwartet)
        .map_err(|e| e.to_string())?;
    reqwest::Client::builder()
        .use_preconfigured_tls(tls)
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| e.to_string())
}

async fn senden<A: Serialize, T: DeserializeOwned>(
    ident: &Identitaet,
    adresse: &str,
    port: u16,
    erwartet: &str,
    methode: reqwest::Method,
    pfad: &str,
    koerper: Option<&A>,
) -> Result<T, String> {
    let mut anfrage =
        client(ident, erwartet)?.request(methode, format!("{}{pfad}", wurzel(adresse, port)));
    if let Some(k) = koerper {
        anfrage = anfrage.json(k);
    }
    let antwort = anfrage
        .send()
        .await
        .map_err(|e| format!("The other device is not reachable: {e}"))?;
    if !antwort.status().is_success() {
        let meldung = antwort
            .json::<serde_json::Value>()
            .await
            .ok()
            .and_then(|v| v["message"].as_str().map(str::to_string))
            .unwrap_or_else(|| "The other device refused.".into());
        return Err(meldung);
    }
    antwort.json::<T>().await.map_err(|e| e.to_string())
}

/// Die Person mit einem gepaarten eigenen Geraet austauschen (person.rs).
///
/// ZWEIMAL HINUEBER: Das erste Mal nimmt das andere Geraet auf und buergt fuer
/// dieses; dann nimmt dieses die Antwort auf und buergt seinerseits -- und
/// diese Buergschaft muss noch hinueber. Danach haben beide dieselbe Liste.
pub async fn person_tauschen(
    ident: &Identitaet,
    gastgeber: &dyn Gastgeber,
    adresse: &str,
    port: u16,
    fp_anderer: &str,
) -> Result<crate::Person, String> {
    let mut eigene = gastgeber.person().ok_or("No person yet.")?;
    for _ in 0..2 {
        let antwort: crate::Person = senden(
            ident,
            adresse,
            port,
            fp_anderer,
            reqwest::Method::POST,
            "/openany/v1/person",
            Some(&eigene),
        )
        .await?;
        crate::person::aufnehmen(ident, &mut eigene, &antwort, fp_anderer)
            .map_err(|e| e.to_string())?;
        gastgeber.person_merken(eigene.clone());
    }
    Ok(eigene)
}

/* ── Einladung vor Ort (einladen.rs) ─────────────────────────────────── */

/// A laedt das Geraet `fp_anderer` in ein Projekt ein. Gibt den Code zurueck.
#[allow(clippy::too_many_arguments)]
pub async fn einladen(
    ident: &Identitaet,
    einladungen: &crate::einladen::GemeinsameEinladungen,
    adresse: &str,
    port: u16,
    fp_anderer: &str,
    anfrage: &crate::einladen::Anfrage,
) -> Result<String, String> {
    let antwort: crate::einladen::Antwort = senden(
        ident,
        adresse,
        port,
        fp_anderer,
        reqwest::Method::POST,
        "/openany/v1/einladung",
        Some(anfrage),
    )
    .await?;
    let e = einladungen
        .lock()
        .map_err(|_| "Invitation not possible.".to_string())?
        .ausgang(
            fp_anderer,
            &antwort.geraet,
            &ident.fingerabdruck,
            anfrage,
            &antwort.zufall,
        );
    Ok(e.code)
}

/// A: Hat B schon bestaetigt? Dann kommt B's Person mit und wird gemerkt.
pub async fn einladung_nachsehen(
    ident: &Identitaet,
    einladungen: &crate::einladen::GemeinsameEinladungen,
    adresse: &str,
    port: u16,
    fp_anderer: &str,
) -> Result<bool, String> {
    let stand: crate::einladen::Stand = senden::<(), _>(
        ident,
        adresse,
        port,
        fp_anderer,
        reqwest::Method::GET,
        "/openany/v1/einladung/stand",
        None,
    )
    .await?;
    match (stand.bestaetigt, stand.person) {
        (true, Some(p)) => {
            // Stellt die Person sich mit DIESEM Geraet vor? Sonst gilt sie nicht.
            if !p.gueltige(&[fp_anderer]).contains_key(fp_anderer) {
                return Err("The other device does not introduce itself.".into());
            }
            einladungen
                .lock()
                .map_err(|_| "Invitation not possible.".to_string())?
                .dort_bestaetigt(fp_anderer, p);
            Ok(true)
        }
        _ => Ok(false),
    }
}

/// A: Die Liste mit B's Beitritt hinueber.
pub async fn aufnahme_senden(
    ident: &Identitaet,
    adresse: &str,
    port: u16,
    fp_anderer: &str,
    aufnahme: &crate::einladen::Aufnahme,
) -> Result<(), String> {
    let _: serde_json::Value = senden(
        ident,
        adresse,
        port,
        fp_anderer,
        reqwest::Method::POST,
        "/openany/v1/einladung/aufnahme",
        Some(aufnahme),
    )
    .await?;
    Ok(())
}

/// A: nach beidseitigem „Passt" die eigene Person hinueber (Kontakt).
pub async fn kontakt_aufnahme_senden(
    ident: &Identitaet,
    adresse: &str,
    port: u16,
    fp_anderer: &str,
    ich: &crate::Person,
) -> Result<(), String> {
    let _: serde_json::Value = senden(
        ident,
        adresse,
        port,
        fp_anderer,
        reqwest::Method::POST,
        "/openany/v1/kontakt/aufnahme",
        Some(ich),
    )
    .await?;
    Ok(())
}

/// A: abbrechen, drueben auch.
pub async fn einladung_abbrechen(
    ident: &Identitaet,
    einladungen: &crate::einladen::GemeinsameEinladungen,
    adresse: &str,
    port: u16,
    fp_anderer: &str,
) {
    if let Ok(mut e) = einladungen.lock() {
        e.entfernen(fp_anderer);
    }
    let _ = senden::<(), serde_json::Value>(
        ident,
        adresse,
        port,
        fp_anderer,
        reqwest::Method::POST,
        "/openany/v1/einladung/abbrechen",
        None,
    )
    .await;
}

/// Den Stand eines lokalen Projekts bei einem Mitgliedsgeraet holen
/// (projektnah.rs). Uebernommen wird er vom Aufrufer, nach Pruefung.
pub async fn projekt_stand(
    ident: &Identitaet,
    adresse: &str,
    port: u16,
    fp_anderer: &str,
    projekt: &str,
) -> Result<crate::projektnah::Stand, String> {
    senden(
        ident,
        adresse,
        port,
        fp_anderer,
        reqwest::Method::POST,
        "/openany/v1/projekt/stand",
        Some(&crate::projektnah::Frage {
            projekt: projekt.to_string(),
        }),
    )
    .await
}

/// Die Bytes einer freigegebenen Datei bei ihrem Geraet holen und ablegen
/// (stueckweise, wie im Abgleich). Stimmt der Abdruck am Ende nicht, gilt
/// sie nicht.
#[allow(clippy::too_many_arguments)]
pub async fn projekt_inhalt_holen(
    ident: &Identitaet,
    adresse: &str,
    port: u16,
    fp_anderer: &str,
    projekt: &str,
    abdruck: &str,
    groesse: u64,
    inhalte: &openany_store::Inhalte,
) -> Result<(), String> {
    if inhalte.hat(abdruck) {
        return Ok(());
    }
    let c = client(ident, fp_anderer)?;
    let mut ladung = inhalte.ladung().map_err(|e| e.to_string())?;
    let mut von = 0u64;
    let mut versuche = 0;
    loop {
        let antwort = c
            .get(format!(
                "{}/openany/v1/projekt/inhalt",
                wurzel(adresse, port)
            ))
            .query(&[
                ("projekt", projekt.to_string()),
                ("hash", abdruck.to_string()),
                ("von", von.to_string()),
                ("laenge", openany_sync::STUECK.to_string()),
            ])
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| format!("The other device is not reachable: {e}"))?;
        if !antwort.status().is_success() {
            // Drueben wird das Original gerade besorgt: gleich noch einmal,
            // hoechstens fuenf Minuten lang.
            if antwort.status() == reqwest::StatusCode::SERVICE_UNAVAILABLE && versuche < 100 {
                versuche += 1;
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                continue;
            }
            let meldung = antwort
                .json::<serde_json::Value>()
                .await
                .ok()
                .and_then(|v| v["message"].as_str().map(str::to_string))
                .unwrap_or_else(|| "The other device refused.".into());
            inhalte.verwerfen(ladung);
            return Err(meldung);
        }
        let stueck = antwort.bytes().await.map_err(|e| e.to_string())?;
        if stueck.is_empty() {
            break;
        }
        ladung.schreiben(&stueck).map_err(|e| e.to_string())?;
        von += stueck.len() as u64;
        if von >= groesse || (stueck.len() as u64) < openany_sync::STUECK {
            break;
        }
    }
    let (da, _) = inhalte.ablegen(ladung).map_err(|e| e.to_string())?;
    if da != abdruck {
        return Err("The content does not match its fingerprint.".into());
    }
    Ok(())
}

/// Eine fortgesetzte Mitgliederliste hinueberschicken (Austritt vor Ort).
pub async fn liste_senden(
    ident: &Identitaet,
    adresse: &str,
    port: u16,
    fp_anderer: &str,
    liste: &crate::Mitgliederliste,
) -> Result<(), String> {
    let _: serde_json::Value = senden(
        ident,
        adresse,
        port,
        fp_anderer,
        reqwest::Method::POST,
        "/openany/v1/projekt/liste",
        Some(liste),
    )
    .await?;
    Ok(())
}

/// Chat-Nachrichten sofort an ein Mitgliedsgeraet in der Naehe.
pub async fn chat_senden(
    ident: &Identitaet,
    adresse: &str,
    port: u16,
    fp_anderer: &str,
    projekt: &str,
    nachrichten: &[crate::chat::Nachricht],
) -> Result<(), String> {
    let _: serde_json::Value = senden(
        ident,
        adresse,
        port,
        fp_anderer,
        reqwest::Method::POST,
        "/openany/v1/projekt/chat",
        Some(&crate::dienst::ChatPost {
            projekt: projekt.to_string(),
            nachrichten: nachrichten.to_vec(),
        }),
    )
    .await?;
    Ok(())
}

/// Wer ist dieses Geraet? (direkt.rs)
pub async fn wer(
    ident: &Identitaet,
    adresse: &str,
    port: u16,
    fp_anderer: &str,
) -> Result<crate::direkt::Wer, String> {
    senden::<(), _>(
        ident,
        adresse,
        port,
        fp_anderer,
        reqwest::Method::GET,
        "/openany/v1/wer",
        None,
    )
    .await
}

/// Wie eine Direktnachricht ausging.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Zustellung {
    Zugestellt,
    /// Das andere Geraet hat sie gesehen und abgelehnt -- mit einer Kennung
    /// aus `direkt::grund`. Noch einmal versuchen hilft nicht.
    Abgelehnt(String),
    /// Nicht angekommen (Netz, Geraet fort): spaeter noch einmal.
    NichtErreichbar(String),
}

/// Eine Direktnachricht abgeben (direkt.rs) -- und sagen, ob sie abgelehnt
/// oder nur nicht angekommen ist. Das trennt den Postausgang: Eine
/// abgelehnte Nachricht alle drei Sekunden neu zu schicken, waere sinnlos
/// und fuer den Empfaenger laestig.
pub async fn nachricht_zustellen(
    ident: &Identitaet,
    adresse: &str,
    port: u16,
    fp_anderer: &str,
    n: &crate::direkt::DirektNachricht,
) -> Zustellung {
    let anfrage = match client(ident, fp_anderer) {
        Ok(c) => c.post(format!("{}/openany/v1/nachricht", wurzel(adresse, port))),
        Err(e) => return Zustellung::NichtErreichbar(e),
    };
    let antwort = match anfrage.json(n).send().await {
        Ok(a) => a,
        Err(e) => {
            return Zustellung::NichtErreichbar(format!("The other device is not reachable: {e}"))
        }
    };
    let status = antwort.status();
    if status.is_success() {
        return Zustellung::Zugestellt;
    }
    let v = antwort
        .json::<serde_json::Value>()
        .await
        .unwrap_or_default();
    if status == reqwest::StatusCode::FORBIDDEN {
        // Aeltere Gegenstellen schicken keinen Grund: dann gilt er als
        // „nicht angenommen".
        let kennung = v["grund"]
            .as_str()
            .unwrap_or(crate::direkt::grund::NICHT_ANGENOMMEN);
        return Zustellung::Abgelehnt(kennung.to_string());
    }
    Zustellung::NichtErreichbar(
        v["message"]
            .as_str()
            .unwrap_or("The other device refused.")
            .to_string(),
    )
}

/// Eine Direktnachricht abgeben (direkt.rs); jede Nichtzustellung als Fehler.
pub async fn nachricht_senden(
    ident: &Identitaet,
    adresse: &str,
    port: u16,
    fp_anderer: &str,
    n: &crate::direkt::DirektNachricht,
) -> Result<(), String> {
    match nachricht_zustellen(ident, adresse, port, fp_anderer, n).await {
        Zustellung::Zugestellt => Ok(()),
        Zustellung::Abgelehnt(k) => Err(crate::direkt::ablehnung_text(&k).to_string()),
        Zustellung::NichtErreichbar(e) => Err(e),
    }
}

/// Eine freigegebene Notiz beim fuehrenden Geraet sperren -- gibt den
/// frischen Inhalt zurueck (sperren.rs).
pub async fn notiz_sperren(
    ident: &Identitaet,
    adresse: &str,
    port: u16,
    fp_anderer: &str,
    projekt: &str,
    notiz: &str,
) -> Result<crate::dienst::NotizInhalt, String> {
    senden(
        ident,
        adresse,
        port,
        fp_anderer,
        reqwest::Method::POST,
        "/openany/v1/projekt/notiz/sperren",
        Some(&crate::dienst::NotizPost {
            projekt: projekt.into(),
            notiz: notiz.into(),
            titel: None,
            inhalt: None,
        }),
    )
    .await
}

/// ... speichern (verlaengert die Sperre) ...
#[allow(clippy::too_many_arguments)]
pub async fn notiz_speichern(
    ident: &Identitaet,
    adresse: &str,
    port: u16,
    fp_anderer: &str,
    projekt: &str,
    notiz: &str,
    titel: &str,
    inhalt: &str,
) -> Result<(), String> {
    let _: serde_json::Value = senden(
        ident,
        adresse,
        port,
        fp_anderer,
        reqwest::Method::POST,
        "/openany/v1/projekt/notiz/speichern",
        Some(&crate::dienst::NotizPost {
            projekt: projekt.into(),
            notiz: notiz.into(),
            titel: Some(titel.into()),
            inhalt: Some(inhalt.into()),
        }),
    )
    .await?;
    Ok(())
}

/// ... und loslassen.
pub async fn notiz_entsperren(
    ident: &Identitaet,
    adresse: &str,
    port: u16,
    fp_anderer: &str,
    projekt: &str,
    notiz: &str,
) -> Result<(), String> {
    let _: serde_json::Value = senden(
        ident,
        adresse,
        port,
        fp_anderer,
        reqwest::Method::POST,
        "/openany/v1/projekt/notiz/entsperren",
        Some(&crate::dienst::NotizPost {
            projekt: projekt.into(),
            notiz: notiz.into(),
            titel: None,
            inhalt: None,
        }),
    )
    .await?;
    Ok(())
}

/// Eine Paarung anfragen. Gibt den Vergleichscode zurueck.
pub async fn anfragen(
    ident: &Identitaet,
    paarungen: &GemeinsamePaarungen,
    adresse: &str,
    port: u16,
    fp_anderer: &str,
    mein_name: &str,
) -> Result<String, String> {
    let zufall = uuid::Uuid::new_v4().simple().to_string();
    let antwort: Antwort = senden(
        ident,
        adresse,
        port,
        fp_anderer,
        reqwest::Method::POST,
        "/openany/v1/paaren",
        Some(&Anfrage {
            name: mein_name.to_string(),
            zufall: zufall.clone(),
        }),
    )
    .await?;

    let code = paarungen
        .lock()
        .map_err(|_| "Pairing not possible.".to_string())?
        .ausgang(
            fp_anderer,
            &antwort.name,
            &ident.fingerabdruck,
            &zufall,
            &antwort.zufall,
        );
    Ok(code)
}

/// Hier bestaetigt (anfragende Seite): hinueber melden, dann nachsehen, ob
/// drueben auch schon bestaetigt ist. `true` = fertig gepaart.
pub async fn bestaetigen(
    ident: &Identitaet,
    paarungen: &GemeinsamePaarungen,
    gastgeber: &dyn Gastgeber,
    adresse: &str,
    port: u16,
    fp_anderer: &str,
) -> Result<bool, String> {
    paarungen
        .lock()
        .map_err(|_| "Pairing not possible.".to_string())?
        .hier_bestaetigen(fp_anderer);
    let _: serde_json::Value = senden::<(), _>(
        ident,
        adresse,
        port,
        fp_anderer,
        reqwest::Method::POST,
        "/openany/v1/paaren/bestaetigt",
        None,
    )
    .await?;
    nachsehen(ident, paarungen, gastgeber, adresse, port, fp_anderer).await
}

/// Hat drueben schon jemand bestaetigt? `true` = fertig gepaart.
pub async fn nachsehen(
    ident: &Identitaet,
    paarungen: &GemeinsamePaarungen,
    gastgeber: &dyn Gastgeber,
    adresse: &str,
    port: u16,
    fp_anderer: &str,
) -> Result<bool, String> {
    let stand: Stand = senden::<(), _>(
        ident,
        adresse,
        port,
        fp_anderer,
        reqwest::Method::GET,
        "/openany/v1/paaren/stand",
        None,
    )
    .await?;
    if !stand.bekannt {
        if let Ok(mut p) = paarungen.lock() {
            p.abbrechen(fp_anderer);
        }
        return Err("The other device cancelled the pairing.".into());
    }
    if stand.bestaetigt {
        if let Ok(mut p) = paarungen.lock() {
            p.dort_bestaetigt(fp_anderer);
        }
    }
    Ok(abschliessen(paarungen, gastgeber, fp_anderer).is_some())
}

pub async fn abbrechen(
    ident: &Identitaet,
    paarungen: &GemeinsamePaarungen,
    adresse: &str,
    port: u16,
    fp_anderer: &str,
) {
    if let Ok(mut p) = paarungen.lock() {
        p.abbrechen(fp_anderer);
    }
    let _ = senden::<(), serde_json::Value>(
        ident,
        adresse,
        port,
        fp_anderer,
        reqwest::Method::POST,
        "/openany/v1/paaren/abbrechen",
        None,
    )
    .await;
}

/// Der QR-Weg: mit dem Geheimnis aus dem gescannten Code paaren.
pub async fn mit_geheimnis(
    ident: &Identitaet,
    gastgeber: &dyn Gastgeber,
    adresse: &str,
    port: u16,
    fp_anderer: &str,
    geheimnis: &str,
) -> Result<String, String> {
    let antwort: Antwort = senden(
        ident,
        adresse,
        port,
        fp_anderer,
        reqwest::Method::POST,
        "/openany/v1/paaren/geheimnis",
        Some(&GeheimnisAnfrage {
            name: gastgeber.mein_name(),
            geheimnis: geheimnis.to_string(),
        }),
    )
    .await?;
    gastgeber.gepaart(fp_anderer, &antwort.name);
    Ok(antwort.name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dienst::Dienst;
    use crate::paaren::Paarungen;
    use std::sync::{Arc, Mutex};

    struct Merker {
        name: String,
        geheimnis: Option<String>,
        gepaart: Mutex<Vec<(String, String)>>,
        person: Mutex<Option<crate::Person>>,
        einladungen: crate::einladen::GemeinsameEinladungen,
        speicher: crate::GemeinsamerSpeicher,
        fremde: Mutex<Vec<crate::Person>>,
        inhalte: Arc<openany_store::Inhalte>,
        /// Was ein „Server" auf Wunsch nachliefert (Abdruck -> Bytes).
        nachschub: Mutex<std::collections::HashMap<String, Vec<u8>>>,
        sperren: crate::sperren::GemeinsameSperren,
    }

    struct Nachschub(Arc<openany_store::Inhalte>, Vec<u8>);

    #[async_trait::async_trait]
    impl crate::dienst::Besorger for Nachschub {
        async fn besorgen(&self, _abdruck: String, _groesse: u64) -> Result<(), String> {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            let mut l = self.0.ladung().map_err(|e| e.to_string())?;
            l.schreiben(&self.1).map_err(|e| e.to_string())?;
            self.0.ablegen(l).map_err(|e| e.to_string())?;
            Ok(())
        }
    }

    impl Gastgeber for Merker {
        fn mein_name(&self) -> String {
            self.name.clone()
        }
        // Wie die App: nur an die eigene Person (direktbefehle.rs prueft mehr).
        fn direkt_annehmen(
            &self,
            n: &crate::direkt::DirektNachricht,
            _anrufer: &str,
        ) -> Result<(), String> {
            match self.person() {
                Some(p) if p.personen_id == n.an_person => Ok(()),
                _ => Err(crate::direkt::grund::NICHT_ANGENOMMEN.into()),
            }
        }
        fn qr_geheimnis(&self) -> Option<String> {
            self.geheimnis.clone()
        }
        fn gepaart(&self, fp: &str, name: &str) {
            self.gepaart.lock().unwrap().push((fp.into(), name.into()));
        }
        fn ist_gepaart(&self, fp: &str) -> bool {
            self.gepaart.lock().unwrap().iter().any(|(f, _)| f == fp)
        }
        fn person(&self) -> Option<crate::Person> {
            self.person.lock().unwrap().clone()
        }
        fn person_merken(&self, p: crate::Person) {
            *self.person.lock().unwrap() = Some(p);
        }
        fn einladungen(&self) -> Option<crate::einladen::GemeinsameEinladungen> {
            Some(self.einladungen.clone())
        }
        fn speicher(&self) -> Option<crate::GemeinsamerSpeicher> {
            Some(self.speicher.clone())
        }
        fn inhalte(&self) -> Option<Arc<openany_store::Inhalte>> {
            Some(self.inhalte.clone())
        }
        fn sperren(&self) -> Option<crate::sperren::GemeinsameSperren> {
            Some(self.sperren.clone())
        }
        fn besorger(&self) -> Option<Arc<dyn crate::dienst::Besorger>> {
            let n = self.nachschub.lock().unwrap();
            let bytes = n.values().next()?.clone();
            Some(Arc::new(Nachschub(self.inhalte.clone(), bytes)))
        }
        fn person_fremd_merken(&self, p: crate::Person) {
            self.fremde.lock().unwrap().push(p);
        }
        fn person_von(&self, id: &str) -> Option<crate::Person> {
            self.person().filter(|p| p.personen_id == id).or_else(|| {
                self.fremde
                    .lock()
                    .unwrap()
                    .iter()
                    .find(|p| p.personen_id == id)
                    .cloned()
            })
        }
    }

    fn geraet(
        name: &str,
        geheimnis: Option<&str>,
    ) -> (Identitaet, GemeinsamePaarungen, Arc<Merker>) {
        let ordner = tempfile::tempdir().unwrap();
        let ident = Identitaet::laden_oder_erzeugen(&ordner.path().join("i.json")).unwrap();
        (
            ident.clone(),
            Arc::new(Mutex::new(Paarungen::default())),
            Arc::new(Merker {
                name: name.into(),
                geheimnis: geheimnis.map(Into::into),
                gepaart: Mutex::new(vec![]),
                person: Mutex::new(Some(crate::Person::neu(&ident, name).unwrap())),
                einladungen: Default::default(),
                speicher: Arc::new(tokio::sync::Mutex::new(
                    openany_store::Speicher::im_arbeitsspeicher().unwrap(),
                )),
                fremde: Mutex::new(vec![]),
                inhalte: Arc::new(
                    openany_store::Inhalte::oeffnen(
                        std::env::temp_dir().join(format!("oa-inhalte-{}", uuid::Uuid::new_v4())),
                    )
                    .unwrap(),
                ),
                nachschub: Mutex::new(Default::default()),
                sperren: Default::default(),
            }),
        )
    }

    #[tokio::test]
    async fn zwei_geraete_paaren_sich_ueber_echtes_tls() {
        let (id_a, p_a, g_a) = geraet("Tablet", None);
        let (id_b, p_b, g_b) = geraet("Telefon", None);
        let dienst_b = Dienst::starten(&id_b, 0, p_b.clone(), g_b.clone())
            .await
            .unwrap();

        let code_a = anfragen(
            &id_a,
            &p_a,
            "127.0.0.1",
            dienst_b.port,
            &id_b.fingerabdruck,
            "Tablet",
        )
        .await
        .unwrap();
        let code_b = p_b
            .lock()
            .unwrap()
            .get(&id_a.fingerabdruck)
            .unwrap()
            .code
            .clone();
        assert_eq!(code_a, code_b, "beide zeigen denselben Code");

        // A bestaetigt zuerst: noch nicht fertig, B hat nicht bestaetigt.
        assert!(!bestaetigen(
            &id_a,
            &p_a,
            g_a.as_ref(),
            "127.0.0.1",
            dienst_b.port,
            &id_b.fingerabdruck
        )
        .await
        .unwrap());
        assert!(g_b.gepaart.lock().unwrap().is_empty());

        // B bestaetigt am Geraet.
        p_b.lock().unwrap().hier_bestaetigen(&id_a.fingerabdruck);
        abschliessen(&p_b, g_b.as_ref(), &id_a.fingerabdruck);
        assert_eq!(
            g_b.gepaart.lock().unwrap()[0],
            (id_a.fingerabdruck.clone(), "Tablet".into())
        );

        // A sieht nach und ist fertig.
        assert!(nachsehen(
            &id_a,
            &p_a,
            g_a.as_ref(),
            "127.0.0.1",
            dienst_b.port,
            &id_b.fingerabdruck
        )
        .await
        .unwrap());
        assert_eq!(
            g_a.gepaart.lock().unwrap()[0],
            (id_b.fingerabdruck.clone(), "Telefon".into())
        );
    }

    #[tokio::test]
    async fn ein_falscher_fingerabdruck_scheitert_im_handschlag() {
        let (id_a, p_a, _) = geraet("Tablet", None);
        let (id_b, p_b, g_b) = geraet("Telefon", None);
        let (id_m, _, _) = geraet("Fremd", None);
        let dienst_b = Dienst::starten(&id_b, 0, p_b.clone(), g_b).await.unwrap();

        let fehler = anfragen(
            &id_a,
            &p_a,
            "127.0.0.1",
            dienst_b.port,
            &id_m.fingerabdruck,
            "Tablet",
        )
        .await;
        assert!(fehler.is_err());
        assert!(
            p_b.lock().unwrap().alle().is_empty(),
            "drueben kam nichts an"
        );
    }

    #[tokio::test]
    async fn der_qr_weg_braucht_das_richtige_geheimnis() {
        let (id_a, _, g_a) = geraet("Tablet", None);
        let (id_b, p_b, g_b) = geraet("Telefon", Some("geheim-123"));
        let dienst_b = Dienst::starten(&id_b, 0, p_b, g_b.clone()).await.unwrap();

        assert!(mit_geheimnis(
            &id_a,
            g_a.as_ref(),
            "127.0.0.1",
            dienst_b.port,
            &id_b.fingerabdruck,
            "falsch"
        )
        .await
        .is_err());
        assert!(g_b.gepaart.lock().unwrap().is_empty());

        let name = mit_geheimnis(
            &id_a,
            g_a.as_ref(),
            "127.0.0.1",
            dienst_b.port,
            &id_b.fingerabdruck,
            "geheim-123",
        )
        .await
        .unwrap();
        assert_eq!(name, "Telefon");
        assert_eq!(g_b.gepaart.lock().unwrap().len(), 1);
        assert_eq!(g_a.gepaart.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn gepaarte_tauschen_ihre_person_und_buergen_fuereinander() {
        let (id_a, _, g_a) = geraet("Tablet", None);
        let (id_b, p_b, g_b) = geraet("Telefon", None);
        let dienst_b = Dienst::starten(&id_b, 0, p_b, g_b.clone()).await.unwrap();

        // Ungepaart: abgewiesen.
        assert!(person_tauschen(
            &id_a,
            g_a.as_ref(),
            "127.0.0.1",
            dienst_b.port,
            &id_b.fingerabdruck
        )
        .await
        .is_err());

        g_b.gepaart(&id_a.fingerabdruck, "Tablet");
        g_a.gepaart(&id_b.fingerabdruck, "Telefon");
        let a = person_tauschen(
            &id_a,
            g_a.as_ref(),
            "127.0.0.1",
            dienst_b.port,
            &id_b.fingerabdruck,
        )
        .await
        .unwrap();
        let b = g_b.person().unwrap();

        assert_eq!(a.personen_id, b.personen_id);
        for p in [&a, &b] {
            assert_eq!(p.gueltige(&[&id_a.fingerabdruck]).len(), 2);
            assert_eq!(p.gueltige(&[&id_b.fingerabdruck]).len(), 2);
        }
    }

    #[tokio::test]
    async fn ein_kontakt_vor_ort_ueber_echtes_tls() {
        use crate::einladen::{Anfrage, KONTAKT};
        let (id_a, _, g_a) = geraet("Annas Telefon", None);
        let (id_b, p_b, g_b) = geraet("Bens Telefon", None);
        let dienst_b = Dienst::starten(&id_b, 0, p_b, g_b.clone()).await.unwrap();
        let port = dienst_b.port;
        let fp_b = id_b.fingerabdruck.clone();
        let anna = g_a.person().unwrap();

        let kontakt = Anfrage {
            projekt: KONTAKT.into(),
            projekt_name: String::new(),
            von: "Anna".into(),
            geraet: "Annas Telefon".into(),
            zufall: "za".into(),
        };
        einladen(&id_a, &g_a.einladungen, "127.0.0.1", port, &fp_b, &kontakt)
            .await
            .unwrap();

        // B hat noch nicht bestaetigt: Die Person wird abgewiesen.
        assert!(
            kontakt_aufnahme_senden(&id_a, "127.0.0.1", port, &fp_b, &anna)
                .await
                .is_err()
        );

        g_b.einladungen
            .lock()
            .unwrap()
            .hier_bestaetigen(&id_a.fingerabdruck);
        assert!(
            einladung_nachsehen(&id_a, &g_a.einladungen, "127.0.0.1", port, &fp_b)
                .await
                .unwrap()
        );
        // Eine Person, die sich nicht mit Annas Geraet vorstellt: abgewiesen.
        let (_, _, g_c) = geraet("Carlas Telefon", None);
        assert!(
            kontakt_aufnahme_senden(&id_a, "127.0.0.1", port, &fp_b, &g_c.person().unwrap())
                .await
                .is_err()
        );
        kontakt_aufnahme_senden(&id_a, "127.0.0.1", port, &fp_b, &anna)
            .await
            .unwrap();
        // Erledigt: drueben ist nichts mehr offen.
        assert!(g_b
            .einladungen
            .lock()
            .unwrap()
            .get(&id_a.fingerabdruck)
            .is_none());
    }

    #[tokio::test]
    async fn eine_einladung_vor_ort_ueber_echtes_tls() {
        use crate::einladen::{Anfrage, Aufnahme};
        use crate::mitglieder::MITGLIED;
        let (id_a, _, g_a) = geraet("Annas Telefon", None);
        let (id_b, p_b, g_b) = geraet("Bens Telefon", None);
        let dienst_b = Dienst::starten(&id_b, 0, p_b, g_b.clone()).await.unwrap();
        let port = dienst_b.port;
        let fp_b = id_b.fingerabdruck.clone();

        let mut anna = g_a.person().unwrap();
        anna.name = "Anna".into();
        let mut liste = crate::Mitgliederliste::gruenden(&id_a, &anna, "p1").unwrap();

        let anfrage = Anfrage {
            projekt: "p1".into(),
            projekt_name: "Garten".into(),
            von: "Anna".into(),
            geraet: "Annas Telefon".into(),
            zufall: "za".into(),
        };
        let code_a = einladen(&id_a, &g_a.einladungen, "127.0.0.1", port, &fp_b, &anfrage)
            .await
            .unwrap();
        let bei_b = g_b
            .einladungen
            .lock()
            .unwrap()
            .get(&id_a.fingerabdruck)
            .unwrap()
            .clone();
        assert_eq!(code_a, bei_b.code);
        assert_eq!(
            (bei_b.von.as_str(), bei_b.projekt_name.as_str()),
            ("Anna", "Garten")
        );

        // B hat noch nicht bestaetigt: Es kommt keine Person, und eine
        // Aufnahme wird abgewiesen.
        assert!(
            !einladung_nachsehen(&id_a, &g_a.einladungen, "127.0.0.1", port, &fp_b)
                .await
                .unwrap()
        );
        let ben = g_b.person().unwrap();
        let mut vorschnell = liste.clone();
        vorschnell
            .eintragen(&id_a, "beitritt", &ben.personen_id, "", MITGLIED, &fp_b)
            .unwrap();
        let aufnahme = Aufnahme {
            projekt_name: "Garten".into(),
            mitgliederliste: vorschnell,
            eigentuemer: anna.clone(),
        };
        assert!(aufnahme_senden(&id_a, "127.0.0.1", port, &fp_b, &aufnahme)
            .await
            .is_err());

        // B bestaetigt; A sieht nach, bekommt B's Person, unterschreibt.
        g_b.einladungen
            .lock()
            .unwrap()
            .hier_bestaetigen(&id_a.fingerabdruck);
        assert!(
            einladung_nachsehen(&id_a, &g_a.einladungen, "127.0.0.1", port, &fp_b)
                .await
                .unwrap()
        );
        let b_person = g_a
            .einladungen
            .lock()
            .unwrap()
            .get(&fp_b)
            .unwrap()
            .person
            .clone()
            .unwrap();
        liste
            .eintragen(
                &id_a,
                "beitritt",
                &b_person.personen_id,
                &b_person.name,
                MITGLIED,
                &fp_b,
            )
            .unwrap();
        let aufnahme = Aufnahme {
            projekt_name: "Garten".into(),
            mitgliederliste: liste,
            eigentuemer: anna,
        };
        aufnahme_senden(&id_a, "127.0.0.1", port, &fp_b, &aufnahme)
            .await
            .unwrap();

        let bei_ben = g_b.speicher.lock().await.projekt("p1").unwrap().unwrap();
        assert_eq!(bei_ben.name, "Garten");
        assert_eq!(bei_ben.rolle, MITGLIED);
        assert!(bei_ben.mitgliederliste.is_some());
        assert_eq!(g_b.fremde.lock().unwrap().len(), 1);
        assert!(g_b
            .einladungen
            .lock()
            .unwrap()
            .get(&id_a.fingerabdruck)
            .is_none());
    }

    #[tokio::test]
    async fn ein_mitglied_bekommt_nur_freigegebene_notizen_und_nur_als_mitglied() {
        use crate::freigaben::{Freigabe, LESEN, NOTIZ_MAPPE};
        use crate::mitglieder::MITGLIED;
        use crate::projektnah::{stand_uebernehmen, GETEILTE_NOTIZ};
        use openany_store::{Notiz, Projekt, Protokoll};

        let (id_a, p_a, g_a) = geraet("Annas Telefon", None);
        let (id_b, _, g_b) = geraet("Bens Telefon", None);
        let (id_c, _, g_c) = geraet("Fremd", None);
        let dienst_a = Dienst::starten(&id_a, 0, p_a, g_a.clone()).await.unwrap();
        let port = dienst_a.port;
        let anna = g_a.person().unwrap();
        let ben = g_b.person().unwrap();

        // Annas Projekt mit Ben als Mitglied, auf beiden Geraeten.
        let mut liste = crate::Mitgliederliste::gruenden(&id_a, &anna, "p1").unwrap();
        liste
            .eintragen(
                &id_a,
                "beitritt",
                &ben.personen_id,
                "Ben",
                MITGLIED,
                &id_b.fingerabdruck,
            )
            .unwrap();
        let json = serde_json::to_string(&liste).unwrap();
        let projekt = Projekt {
            uuid: "p1".into(),
            name: "Garten".into(),
            rolle: "owner".into(),
            ..Default::default()
        };
        for g in [&g_a, &g_b] {
            g.speicher
                .lock()
                .await
                .projekt_lokal_schreiben(&projekt, &json)
                .unwrap();
        }
        g_b.person_fremd_merken(anna.clone());

        // Anna: zwei Notizen, nur die Mappe "Garten" ist freigegeben.
        {
            let s = g_a.speicher.lock().await;
            for (id, mappe) in [("n1", "Garten/Beete"), ("n2", "Privat")] {
                s.notiz_schreiben(
                    &Notiz {
                        zk_id: id.into(),
                        titel: format!("Notiz {id}"),
                        inhalt: "Inhalt".into(),
                        mappe: Some(mappe.into()),
                        ..Default::default()
                    },
                    Protokoll::Merken,
                )
                .unwrap();
            }
            let f = Freigabe::unterschreiben(
                &id_a,
                "p1",
                &anna.personen_id,
                NOTIZ_MAPPE,
                "Garten",
                "Garten",
                LESEN,
            )
            .unwrap();
            s.projekt_freigaben_schreiben("p1", &serde_json::to_string(&vec![f]).unwrap())
                .unwrap();
        }

        // Ein Fremder bekommt nichts.
        assert!(
            projekt_stand(&id_c, "127.0.0.1", port, &id_a.fingerabdruck, "p1")
                .await
                .is_err()
        );
        drop(g_c);

        // Ben holt und uebernimmt: nur n1.
        let stand = projekt_stand(&id_b, "127.0.0.1", port, &id_a.fingerabdruck, "p1")
            .await
            .unwrap();
        let bericht = {
            let s = g_b.speicher.lock().await;
            stand_uebernehmen(&s, g_b.as_ref(), "p1", &id_a.fingerabdruck, &stand).unwrap()
        };
        assert_eq!((bericht.freigaben, bericht.notizen), (1, 1));
        let bei_ben = g_b
            .speicher
            .lock()
            .await
            .projektsachen("p1", GETEILTE_NOTIZ)
            .unwrap();
        assert_eq!(bei_ben.len(), 1);
        assert_eq!(bei_ben[0].text("titel"), "Notiz n1");

        // Anna nimmt die Freigabe zurueck: Bei Ben verschwindet die Notiz.
        std::thread::sleep(std::time::Duration::from_millis(5));
        {
            let s = g_a.speicher.lock().await;
            let mut alle: Vec<Freigabe> =
                serde_json::from_str(&s.projekt_freigaben("p1").unwrap().unwrap()).unwrap();
            alle.push(
                Freigabe::unterschreiben(
                    &id_a,
                    "p1",
                    &anna.personen_id,
                    NOTIZ_MAPPE,
                    "Garten",
                    "Garten",
                    "",
                )
                .unwrap(),
            );
            s.projekt_freigaben_schreiben("p1", &serde_json::to_string(&alle).unwrap())
                .unwrap();
        }
        let stand = projekt_stand(&id_b, "127.0.0.1", port, &id_a.fingerabdruck, "p1")
            .await
            .unwrap();
        {
            let s = g_b.speicher.lock().await;
            stand_uebernehmen(&s, g_b.as_ref(), "p1", &id_a.fingerabdruck, &stand).unwrap();
        }
        assert!(g_b
            .speicher
            .lock()
            .await
            .projektsachen("p1", GETEILTE_NOTIZ)
            .unwrap()
            .is_empty());
    }

    #[tokio::test]
    async fn eine_freigegebene_datei_reist_mit_ihren_bytes_nur_zu_mitgliedern() {
        use crate::freigaben::{Freigabe, LESEN, ORDNER};
        use crate::mitglieder::MITGLIED;
        use crate::projektnah::{stand_uebernehmen, GETEILTE_DATEI};
        use openany_store::{Datei, Projekt, Protokoll};

        let (id_a, p_a, g_a) = geraet("Annas Telefon", None);
        let (id_b, _, g_b) = geraet("Bens Telefon", None);
        let (id_c, _, _) = geraet("Fremd", None);
        let dienst_a = Dienst::starten(&id_a, 0, p_a, g_a.clone()).await.unwrap();
        let port = dienst_a.port;
        let anna = g_a.person().unwrap();
        let ben = g_b.person().unwrap();

        let mut liste = crate::Mitgliederliste::gruenden(&id_a, &anna, "p1").unwrap();
        liste
            .eintragen(
                &id_a,
                "beitritt",
                &ben.personen_id,
                "Ben",
                MITGLIED,
                &id_b.fingerabdruck,
            )
            .unwrap();
        let json = serde_json::to_string(&liste).unwrap();
        let projekt = Projekt {
            uuid: "p1".into(),
            name: "Garten".into(),
            rolle: "owner".into(),
            ..Default::default()
        };
        for g in [&g_a, &g_b] {
            g.speicher
                .lock()
                .await
                .projekt_lokal_schreiben(&projekt, &json)
                .unwrap();
        }
        g_b.person_fremd_merken(anna.clone());

        // Anna: Ordner "Pläne" mit plan.pdf, daneben privat.pdf.
        let ablegen = |text: &[u8]| {
            let mut l = g_a.inhalte.ladung().unwrap();
            l.schreiben(text).unwrap();
            g_a.inhalte.ablegen(l).unwrap()
        };
        let (plan, plan_groesse) = ablegen(b"%PDF-1.4 Plan");
        let (privat, _) = ablegen(b"%PDF-1.4 Privat");
        {
            let s = g_a.speicher.lock().await;
            let datei = |uuid: &str,
                         ordner: bool,
                         eltern: Option<&str>,
                         name: &str,
                         abdruck: Option<&str>,
                         groesse: u64| Datei {
                uuid: uuid.into(),
                zone: "documents".into(),
                ist_ordner: ordner,
                eltern: eltern.map(Into::into),
                name: name.into(),
                groesse,
                mime: if ordner {
                    String::new()
                } else {
                    "application/pdf".into()
                },
                abdruck: abdruck.map(Into::into),
                ..Default::default()
            };
            s.datei_schreiben(
                &datei("o1", true, None, "Pläne", None, 0),
                Protokoll::Merken,
            )
            .unwrap();
            s.datei_schreiben(
                &datei(
                    "d1",
                    false,
                    Some("o1"),
                    "plan.pdf",
                    Some(&plan),
                    plan_groesse,
                ),
                Protokoll::Merken,
            )
            .unwrap();
            s.datei_schreiben(
                &datei("d2", false, None, "privat.pdf", Some(&privat), 15),
                Protokoll::Merken,
            )
            .unwrap();
            let f = Freigabe::unterschreiben(
                &id_a,
                "p1",
                &anna.personen_id,
                ORDNER,
                "o1",
                "Pläne",
                LESEN,
            )
            .unwrap();
            s.projekt_freigaben_schreiben("p1", &serde_json::to_string(&vec![f]).unwrap())
                .unwrap();
        }

        // Ben gleicht ab: plan.pdf steht da, privat.pdf nicht.
        let stand = projekt_stand(&id_b, "127.0.0.1", port, &id_a.fingerabdruck, "p1")
            .await
            .unwrap();
        let bericht = {
            let s = g_b.speicher.lock().await;
            stand_uebernehmen(&s, g_b.as_ref(), "p1", &id_a.fingerabdruck, &stand).unwrap()
        };
        assert_eq!(bericht.dateien, 1);
        let bei_ben = g_b
            .speicher
            .lock()
            .await
            .projektsachen("p1", GETEILTE_DATEI)
            .unwrap();
        assert_eq!(bei_ben[0].text("name"), "plan.pdf");
        assert_eq!(bei_ben[0].text("zone"), "documents");

        // Die Bytes: Ben bekommt plan.pdf ...
        projekt_inhalt_holen(
            &id_b,
            "127.0.0.1",
            port,
            &id_a.fingerabdruck,
            "p1",
            &plan,
            plan_groesse,
            &g_b.inhalte,
        )
        .await
        .unwrap();
        assert!(g_b.inhalte.hat(&plan));
        // ... aber nicht privat.pdf, und ein Fremder gar nichts.
        assert!(projekt_inhalt_holen(
            &id_b,
            "127.0.0.1",
            port,
            &id_a.fingerabdruck,
            "p1",
            &privat,
            15,
            &g_b.inhalte
        )
        .await
        .is_err());
        let fremd = openany_store::Inhalte::oeffnen(
            std::env::temp_dir().join(format!("oa-fremd-{}", uuid::Uuid::new_v4())),
        )
        .unwrap();
        assert!(projekt_inhalt_holen(
            &id_c,
            "127.0.0.1",
            port,
            &id_a.fingerabdruck,
            "p1",
            &plan,
            plan_groesse,
            &fremd
        )
        .await
        .is_err());

        // Ein Original, das bei Anna nur „bei Bedarf" liegt: Annas Geraet
        // besorgt es erst (hier nach einer halben Sekunde), Ben wartet.
        let spaeter = b"%PDF-1.4 erst vom Server".to_vec();
        let abdruck_spaeter = {
            use sha2::{Digest, Sha256};
            Sha256::digest(&spaeter)
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        };
        g_a.speicher
            .lock()
            .await
            .datei_schreiben(
                &Datei {
                    uuid: "d3".into(),
                    zone: "documents".into(),
                    eltern: Some("o1".into()),
                    name: "spaeter.pdf".into(),
                    groesse: spaeter.len() as u64,
                    mime: "application/pdf".into(),
                    abdruck: Some(abdruck_spaeter.clone()),
                    ..Default::default()
                },
                Protokoll::Merken,
            )
            .unwrap();
        assert!(!g_a.inhalte.hat(&abdruck_spaeter));
        g_a.nachschub
            .lock()
            .unwrap()
            .insert(abdruck_spaeter.clone(), spaeter.clone());
        projekt_inhalt_holen(
            &id_b,
            "127.0.0.1",
            port,
            &id_a.fingerabdruck,
            "p1",
            &abdruck_spaeter,
            spaeter.len() as u64,
            &g_b.inhalte,
        )
        .await
        .unwrap();
        assert!(g_b.inhalte.hat(&abdruck_spaeter));
    }

    #[tokio::test]
    async fn entfernte_erfahren_es_und_wer_geht_sagt_es_vorher() {
        use crate::mitglieder::MITGLIED;
        use crate::projektnah::stand_uebernehmen;
        use openany_store::Projekt;

        let (id_a, p_a, g_a) = geraet("Annas Telefon", None);
        let (id_b, _, g_b) = geraet("Bens Telefon", None);
        let dienst_a = Dienst::starten(&id_a, 0, p_a, g_a.clone()).await.unwrap();
        let port = dienst_a.port;
        let anna = g_a.person().unwrap();
        let ben = g_b.person().unwrap();
        let mut liste = crate::Mitgliederliste::gruenden(&id_a, &anna, "p1").unwrap();
        liste
            .eintragen(
                &id_a,
                "beitritt",
                &ben.personen_id,
                "Ben",
                MITGLIED,
                &id_b.fingerabdruck,
            )
            .unwrap();
        let projekt = Projekt {
            uuid: "p1".into(),
            name: "Garten".into(),
            rolle: "owner".into(),
            ..Default::default()
        };
        let schreiben = |l: &crate::Mitgliederliste| serde_json::to_string(l).unwrap();
        for g in [&g_a, &g_b] {
            g.speicher
                .lock()
                .await
                .projekt_lokal_schreiben(&projekt, &schreiben(&liste))
                .unwrap();
        }
        g_b.person_fremd_merken(anna.clone());

        // 1. Ben tritt aus: Er schickt die fortgesetzte Liste zu Anna.
        let mut austritt = liste.clone();
        austritt
            .eintragen(
                &id_b,
                "austritt",
                &ben.personen_id,
                "Ben",
                "",
                &id_b.fingerabdruck,
            )
            .unwrap();
        liste_senden(&id_b, "127.0.0.1", port, &id_a.fingerabdruck, &austritt)
            .await
            .unwrap();
        let bei_anna = crate::projektnah::lesen(&*g_a.speicher.lock().await, "p1")
            .unwrap()
            .1;
        assert_eq!(bei_anna.mitglieder(&|_| None).unwrap().len(), 1);
        // Eine Liste, die nicht fortsetzt, nimmt Anna nicht an.
        assert!(
            liste_senden(&id_b, "127.0.0.1", port, &id_a.fingerabdruck, &liste)
                .await
                .is_err()
        );

        // 2. Anna laedt Ben wieder ein und entfernt ihn dann.
        let mut neu = bei_anna.clone();
        neu.eintragen(
            &id_a,
            "beitritt",
            &ben.personen_id,
            "Ben",
            MITGLIED,
            &id_b.fingerabdruck,
        )
        .unwrap();
        g_b.speicher
            .lock()
            .await
            .projekt_lokal_schreiben(&projekt, &schreiben(&neu))
            .unwrap();
        neu.eintragen(
            &id_a,
            "austritt",
            &ben.personen_id,
            "Ben",
            "",
            &id_b.fingerabdruck,
        )
        .unwrap();
        g_a.speicher
            .lock()
            .await
            .projekt_lokal_schreiben(&projekt, &schreiben(&neu))
            .unwrap();

        // Ben holt noch die Liste -- und erfaehrt, dass er draussen ist.
        let stand = projekt_stand(&id_b, "127.0.0.1", port, &id_a.fingerabdruck, "p1")
            .await
            .unwrap();
        assert!(stand.freigaben.is_empty() && stand.notizen.is_empty());
        let bericht = {
            let s = g_b.speicher.lock().await;
            stand_uebernehmen(&s, g_b.as_ref(), "p1", &id_a.fingerabdruck, &stand).unwrap()
        };
        assert!(bericht.nicht_mehr_mitglied);
    }

    #[tokio::test]
    async fn der_chat_kommt_sofort_an_und_reist_mit_dem_stand() {
        use crate::chat::Nachricht;
        use crate::mitglieder::MITGLIED;
        use crate::projektnah::{chat_lesen, stand_uebernehmen};
        use openany_store::Projekt;

        let (id_a, p_a, g_a) = geraet("Annas Telefon", None);
        let (id_b, _, g_b) = geraet("Bens Telefon", None);
        let (id_c, _, _) = geraet("Fremd", None);
        let dienst_a = Dienst::starten(&id_a, 0, p_a, g_a.clone()).await.unwrap();
        let port = dienst_a.port;
        let anna = g_a.person().unwrap();
        let ben = g_b.person().unwrap();
        let mut liste = crate::Mitgliederliste::gruenden(&id_a, &anna, "p1").unwrap();
        liste
            .eintragen(
                &id_a,
                "beitritt",
                &ben.personen_id,
                "Ben",
                MITGLIED,
                &id_b.fingerabdruck,
            )
            .unwrap();
        let projekt = Projekt {
            uuid: "p1".into(),
            name: "Garten".into(),
            rolle: "owner".into(),
            ..Default::default()
        };
        let json = serde_json::to_string(&liste).unwrap();
        for g in [&g_a, &g_b] {
            g.speicher
                .lock()
                .await
                .projekt_lokal_schreiben(&projekt, &json)
                .unwrap();
        }
        g_b.person_fremd_merken(anna.clone());

        // Ben schreibt (bei sich abgelegt, wie es die App tut) -- sofort bei Anna.
        let hallo =
            Nachricht::schreiben(&id_b, "p1", &ben.personen_id, "Ben", "Hallo Anna!").unwrap();
        crate::projektnah::chat_aufnehmen(
            &*g_b.speicher.lock().await,
            g_b.as_ref(),
            "p1",
            std::slice::from_ref(&hallo),
        )
        .unwrap();
        chat_senden(
            &id_b,
            "127.0.0.1",
            port,
            &id_a.fingerabdruck,
            "p1",
            std::slice::from_ref(&hallo),
        )
        .await
        .unwrap();
        let bei_anna = chat_lesen(&*g_a.speicher.lock().await, "p1").unwrap();
        assert_eq!(bei_anna.len(), 1);
        assert_eq!(bei_anna[0].text, "Hallo Anna!");

        // Ein Fremder: abgewiesen.
        let spam = Nachricht::schreiben(&id_c, "p1", "x", "X", "Spam").unwrap();
        assert!(
            chat_senden(&id_c, "127.0.0.1", port, &id_a.fingerabdruck, "p1", &[spam])
                .await
                .is_err()
        );

        // Annas Antwort kommt bei Ben mit dem Stand an -- doppelt zaehlt nichts.
        let antwort =
            Nachricht::schreiben(&id_a, "p1", &anna.personen_id, "Anna", "Hallo Ben!").unwrap();
        crate::projektnah::chat_aufnehmen(
            &*g_a.speicher.lock().await,
            g_a.as_ref(),
            "p1",
            &[antwort],
        )
        .unwrap();
        let stand = projekt_stand(&id_b, "127.0.0.1", port, &id_a.fingerabdruck, "p1")
            .await
            .unwrap();
        let bericht = {
            let s = g_b.speicher.lock().await;
            stand_uebernehmen(&s, g_b.as_ref(), "p1", &id_a.fingerabdruck, &stand).unwrap()
        };
        assert_eq!(
            bericht.chat, 1,
            "nur Annas Antwort ist neu -- Bens eigene hatte Ben nicht"
        );
        let bei_ben = chat_lesen(&*g_b.speicher.lock().await, "p1").unwrap();
        assert_eq!(
            bei_ben.iter().map(|n| n.text.as_str()).collect::<Vec<_>>(),
            ["Hallo Anna!", "Hallo Ben!"]
        );
    }

    #[tokio::test]
    async fn die_planung_reist_mit_dem_stand_und_geloeschtes_bleibt_fort() {
        use crate::mitglieder::MITGLIED;
        use crate::projektnah::stand_uebernehmen;
        use openany_store::{Projekt, Projektsache, Protokoll};

        let (id_a, p_a, g_a) = geraet("Annas Telefon", None);
        let (id_b, _, g_b) = geraet("Bens Telefon", None);
        let dienst_a = Dienst::starten(&id_a, 0, p_a, g_a.clone()).await.unwrap();
        let port = dienst_a.port;
        let anna = g_a.person().unwrap();
        let ben = g_b.person().unwrap();
        let mut liste = crate::Mitgliederliste::gruenden(&id_a, &anna, "p1").unwrap();
        liste
            .eintragen(
                &id_a,
                "beitritt",
                &ben.personen_id,
                "Ben",
                MITGLIED,
                &id_b.fingerabdruck,
            )
            .unwrap();
        let projekt = Projekt {
            uuid: "p1".into(),
            name: "Garten".into(),
            rolle: "owner".into(),
            ..Default::default()
        };
        let json = serde_json::to_string(&liste).unwrap();
        for g in [&g_a, &g_b] {
            g.speicher
                .lock()
                .await
                .projekt_lokal_schreiben(&projekt, &json)
                .unwrap();
        }
        g_b.person_fremd_merken(anna.clone());

        let sache = |art: &str, uuid: &str, eltern: Option<&str>, name: &str| Projektsache {
            projekt: "p1".into(),
            art: art.into(),
            uuid: uuid.into(),
            eltern: eltern.map(Into::into),
            felder: serde_json::json!({ "name": name }),
            geaendert_at: String::new(),
        };
        {
            let s = g_a.speicher.lock().await;
            s.projektsache_schreiben(&sache("board", "b1", None, "Beete"), Protokoll::Merken)
                .unwrap();
            s.projektsache_schreiben(
                &sache("column", "c1", Some("b1"), "Offen"),
                Protokoll::Merken,
            )
            .unwrap();
            s.projektsache_schreiben(
                &sache("column", "c2", Some("b1"), "Fertig"),
                Protokoll::Merken,
            )
            .unwrap();
        }

        let holen = || async {
            let stand = projekt_stand(&id_b, "127.0.0.1", port, &id_a.fingerabdruck, "p1")
                .await
                .unwrap();
            let s = g_b.speicher.lock().await;
            stand_uebernehmen(&s, g_b.as_ref(), "p1", &id_a.fingerabdruck, &stand).unwrap()
        };

        assert_eq!(holen().await.planung, 3);
        assert_eq!(holen().await.planung, 0, "schon da -- nichts neu");
        let s_b = || async { g_b.speicher.lock().await };
        assert_eq!(
            s_b()
                .await
                .projektsache("p1", "c1")
                .unwrap()
                .unwrap()
                .eltern
                .as_deref(),
            Some("b1")
        );

        // Anna loescht eine Spalte: Bei Ben ist sie danach fort, und sein
        // naechster Stand braechte sie Anna nicht zurueck.
        g_a.speicher
            .lock()
            .await
            .projektsache_entfernen("p1", "c2", Protokoll::Merken)
            .unwrap();
        assert_eq!(holen().await.planung, 1);
        assert!(s_b().await.projektsache("p1", "c2").unwrap().is_none());
        let bens = crate::projektnah::planung_lesen(&*s_b().await, "p1").unwrap();
        assert_eq!(
            crate::projektnah::planung_aufnehmen(&*g_a.speicher.lock().await, "p1", &bens, &[])
                .unwrap(),
            0
        );
        assert!(g_a
            .speicher
            .lock()
            .await
            .projektsache("p1", "c2")
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn eine_direktnachricht_kommt_nur_echt_und_nur_an_die_richtige_person() {
        use crate::direkt::{DirektNachricht, Wer, KONTO};

        let (id_a, _, g_a) = geraet("Annas Telefon", None);
        let (id_b, p_b, g_b) = geraet("Bens Telefon", None);
        let (id_c, _, _) = geraet("Fremd", None);
        let dienst_b = Dienst::starten(&id_b, 0, p_b, g_b.clone()).await.unwrap();
        let port = dienst_b.port;

        // Wer ist das Geraet? Jeder darf fragen.
        let ben = wer(&id_a, "127.0.0.1", port, &id_b.fingerabdruck)
            .await
            .unwrap();
        assert_eq!(ben.personen_id, g_b.person().unwrap().personen_id);

        let anna = Wer {
            personen_id: g_a.person().unwrap().personen_id,
            name: "Anna".into(),
        };
        let hallo =
            DirektNachricht::schreiben(&id_a, &anna, &ben.personen_id, "Hallo Ben").unwrap();
        nachricht_senden(&id_a, "127.0.0.1", port, &id_b.fingerabdruck, &hallo)
            .await
            .unwrap();
        let bei_ben = g_b
            .speicher
            .lock()
            .await
            .nachricht(KONTO, &hallo.id)
            .unwrap()
            .unwrap();
        assert_eq!(bei_ben.text, "Hallo Ben");
        assert_eq!(bei_ben.raum, anna.personen_id);
        assert_eq!(bei_ben.gegenueber.as_deref(), Some("Anna"));
        assert!(!bei_ben.von_mir);

        // Ein anderes Geraet reicht Annas Nachricht ein: nicht echt.
        assert!(
            nachricht_senden(&id_c, "127.0.0.1", port, &id_b.fingerabdruck, &hallo)
                .await
                .is_err()
        );
        // An jemand anderen: abgelehnt -- mit Grund, nicht „nicht erreichbar".
        let falsch = DirektNachricht::schreiben(&id_a, &anna, "p-jemand", "Hallo?").unwrap();
        assert_eq!(
            nachricht_zustellen(&id_a, "127.0.0.1", port, &id_b.fingerabdruck, &falsch).await,
            Zustellung::Abgelehnt(crate::direkt::grund::NICHT_ANGENOMMEN.into())
        );
        assert!(g_b
            .speicher
            .lock()
            .await
            .nachricht(KONTO, &falsch.id)
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn bearbeiten_nur_mit_sperre_beim_fuehrenden_geraet() {
        use crate::freigaben::{Freigabe, BEARBEITEN, LESEN, NOTIZ_MAPPE};
        use crate::mitglieder::MITGLIED;
        use openany_store::{Notiz, Projekt, Protokoll};

        let (id_a, p_a, g_a) = geraet("Annas Telefon", None);
        let (id_b, _, g_b) = geraet("Bens Telefon", None);
        let dienst_a = Dienst::starten(&id_a, 0, p_a, g_a.clone()).await.unwrap();
        let port = dienst_a.port;
        let fp_a = id_a.fingerabdruck.clone();
        let anna = g_a.person().unwrap();
        let ben = g_b.person().unwrap();
        let mut liste = crate::Mitgliederliste::gruenden(&id_a, &anna, "p1").unwrap();
        liste
            .eintragen(
                &id_a,
                "beitritt",
                &ben.personen_id,
                "Ben",
                MITGLIED,
                &id_b.fingerabdruck,
            )
            .unwrap();
        let projekt = Projekt {
            uuid: "p1".into(),
            name: "Garten".into(),
            rolle: "owner".into(),
            ..Default::default()
        };
        {
            let s = g_a.speicher.lock().await;
            s.projekt_lokal_schreiben(&projekt, &serde_json::to_string(&liste).unwrap())
                .unwrap();
            for (id, mappe) in [("n1", "Garten"), ("n2", "Lesen")] {
                s.notiz_schreiben(
                    &Notiz {
                        zk_id: id.into(),
                        titel: "Beete".into(),
                        inhalt: "Tomaten".into(),
                        mappe: Some(mappe.into()),
                        ..Default::default()
                    },
                    Protokoll::Merken,
                )
                .unwrap();
            }
            let f = vec![
                Freigabe::unterschreiben(
                    &id_a,
                    "p1",
                    &anna.personen_id,
                    NOTIZ_MAPPE,
                    "Garten",
                    "Garten",
                    BEARBEITEN,
                )
                .unwrap(),
                Freigabe::unterschreiben(
                    &id_a,
                    "p1",
                    &anna.personen_id,
                    NOTIZ_MAPPE,
                    "Lesen",
                    "Lesen",
                    LESEN,
                )
                .unwrap(),
            ];
            s.projekt_freigaben_schreiben("p1", &serde_json::to_string(&f).unwrap())
                .unwrap();
        }

        // Nur lesen: keine Sperre.
        assert!(notiz_sperren(&id_b, "127.0.0.1", port, &fp_a, "p1", "n2")
            .await
            .is_err());

        // Anna schreibt selbst gerade: Ben bekommt sie nicht.
        let anna_halter = crate::sperren::Halter {
            personen_id: anna.personen_id.clone(),
            name: "Anna".into(),
            geraet: String::new(),
        };
        g_a.sperren
            .lock()
            .unwrap()
            .nehmen("n1", anna_halter.clone())
            .unwrap();
        let e = notiz_sperren(&id_b, "127.0.0.1", port, &fp_a, "p1", "n1")
            .await
            .unwrap_err();
        assert!(e.contains("Anna"), "{e}");
        g_a.sperren.lock().unwrap().loslassen("n1", &anna_halter);

        // Speichern ohne Sperre: nein.
        assert!(
            notiz_speichern(&id_b, "127.0.0.1", port, &fp_a, "p1", "n1", "Beete", "x")
                .await
                .is_err()
        );

        // Ben sperrt, bekommt den Inhalt, speichert -- in Annas Notizbuch.
        let inhalt = notiz_sperren(&id_b, "127.0.0.1", port, &fp_a, "p1", "n1")
            .await
            .unwrap();
        assert_eq!(inhalt.inhalt, "Tomaten");
        notiz_speichern(
            &id_b,
            "127.0.0.1",
            port,
            &fp_a,
            "p1",
            "n1",
            "Beete",
            "Tomaten und Bohnen",
        )
        .await
        .unwrap();
        assert_eq!(
            g_a.speicher
                .lock()
                .await
                .notiz("n1")
                .unwrap()
                .unwrap()
                .inhalt,
            "Tomaten und Bohnen"
        );
        // Solange Ben haelt, wird Anna abgewiesen.
        assert!(g_a
            .sperren
            .lock()
            .unwrap()
            .nehmen("n1", anna_halter.clone())
            .is_err());

        // Ben schliesst: frei.
        notiz_entsperren(&id_b, "127.0.0.1", port, &fp_a, "p1", "n1")
            .await
            .unwrap();
        assert!(g_a
            .sperren
            .lock()
            .unwrap()
            .nehmen("n1", anna_halter)
            .is_ok());
    }
}
