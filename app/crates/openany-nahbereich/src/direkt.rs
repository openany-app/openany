//! Direktnachrichten vor Ort, ohne Server (Tiffy, 02.10.2026).
//!
//! **An jedes openany-Geraet in der Naehe** -- nicht nur an bekannte
//! Personen. Fremde im selben WLAN mit offener App hielt Tiffy fuer
//! unwahrscheinlich; Anfragen-Ordner, Einstellung und Grenzen kommen vor der
//! oeffentlichen Freigabe. Bis dahin gibt es zweierlei: Wer keine bekannte
//! Person ist, heisst in der Ansicht „unbekannt", und man kann blockieren.
//!
//! **Unterschrieben und nur direkt.** Die Nachricht traegt Zertifikat und
//! Signatur des sendenden Geraets; angenommen wird sie nur von genau diesem
//! Geraet (TLS-Fingerabdruck = Unterzeichner). Weitergereicht wird sie nicht:
//! Ein drittes Geraet koennte sie lesen. Das ginge erst mit
//! Ende-zu-Ende-Verschluesselung.
//!
//! **Wer sich als bekannte Person ausgibt, muss eines ihrer Geraete sein.**
//! Sonst koennte sich jeder „Mama" nennen und unter Mamas Kennung schreiben.
//! Eine unbekannte Kennung dagegen laesst sich nicht pruefen -- sie bleibt
//! „unbekannt".

use crate::person::{der, pruefen, unterschreiben, PersonFehler};
use crate::tls::fingerabdruck;
use crate::Identitaet;
use serde::{Deserialize, Serialize};

/// Wie lang eine Nachricht hoechstens sein darf.
pub const LAENGE: usize = 20_000;

/// ANFRAGEN (Tiffy, 06.10.2026): Wer keine bekannte Person ist und noch nicht
/// angenommen wurde, schreibt nur, wenn der Empfaenger Anfragen erlaubt
/// (Vorgabe: aus) -- und dann hoechstens [`ANFRAGEN_HOECHSTENS`] Nachrichten
/// zu je [`ANFRAGE_LAENGE`] Zeichen, bis angenommen ist. Keine Anhaenge (die
/// es vor Ort ohnehin noch nicht gibt).
pub const ANFRAGE_LAENGE: usize = 500;
pub const ANFRAGEN_HOECHSTENS: u32 = 3;

/// Warum eine Direktnachricht nicht angenommen wurde -- als Kennung, damit
/// die Oberflaeche des Absenders es in ihrer Sprache sagt.
///
/// BLOCKIERT UND „ANFRAGEN AUS" SIND DASSELBE: `nicht-angenommen`. Wer
/// blockiert wurde, soll es nicht daran erkennen, dass die Antwort anders
/// lautet als bei jemandem, der gar keine Anfragen annimmt.
pub mod grund {
    pub const NICHT_ANGENOMMEN: &str = "nicht-angenommen";
    pub const ANFRAGEN_VOLL: &str = "anfragen-voll";
    pub const ZU_LANG: &str = "zu-lang";
}

/// Der Satz zu einem Grund, fuer Protokoll und aeltere Gegenstellen.
pub fn ablehnung_text(kennung: &str) -> &'static str {
    match kennung {
        grund::ANFRAGEN_VOLL => "Request limit reached -- wait until the request is accepted.",
        grund::ZU_LANG => "Too long for a request.",
        _ => "Not accepted.",
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DirektNachricht {
    pub id: String,
    pub von_person: String,
    /// Wie die sendende Person sich nennt -- ungeprueft.
    pub von_name: String,
    pub an_person: String,
    pub text: String,
    pub at: String,
    pub unterzeichner: String,
    pub zertifikat_pem: String,
    pub signatur: String,
}

/// Wer ein Geraet ist -- das fragt, wer ihm schreiben will.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Wer {
    pub personen_id: String,
    pub name: String,
}

fn inhalt(n: &DirektNachricht) -> Vec<u8> {
    let mut b = b"openany-direkt-v1".to_vec();
    for teil in [
        &n.id,
        &n.von_person,
        &n.von_name,
        &n.an_person,
        &n.text,
        &n.at,
        &n.unterzeichner,
    ] {
        b.push(0x1f);
        b.extend_from_slice(teil.as_bytes());
    }
    b
}

impl DirektNachricht {
    pub fn schreiben(
        ident: &Identitaet,
        von: &Wer,
        an_person: &str,
        text: &str,
    ) -> Result<Self, PersonFehler> {
        let mut n = Self {
            id: uuid::Uuid::new_v4().to_string(),
            von_person: von.personen_id.clone(),
            von_name: von.name.clone(),
            an_person: an_person.to_string(),
            text: text.to_string(),
            at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            unterzeichner: ident.fingerabdruck.clone(),
            zertifikat_pem: ident.zertifikat_pem.clone(),
            signatur: String::new(),
        };
        n.signatur = unterschreiben(ident, &inhalt(&n))?;
        Ok(n)
    }

    /// Echt, unveraendert, vom Geraet `anrufer` und nicht leer.
    pub fn echt_von(&self, anrufer: &str) -> bool {
        !self.text.trim().is_empty()
            && self.text.len() <= LAENGE
            && !self.id.is_empty()
            && self.unterzeichner == anrufer
            && der(&self.zertifikat_pem).is_some_and(|d| fingerabdruck(&d) == self.unterzeichner)
            && pruefen(&self.zertifikat_pem, &inhalt(self), &self.signatur)
    }
}

/// Das Konto, unter dem Direktnachrichten im Verlauf liegen.
pub const KONTO: &str = "nah";

/// Im Verlauf ablegen (Tabelle `nachrichten`, Konto [`KONTO`]). Die
/// Unterhaltung haengt an der Person auf der anderen Seite (`raum`), ihr Name
/// steht als Gegenueber.
pub fn ablegen(
    speicher: &openany_store::Speicher,
    n: &DirektNachricht,
    von_mir: bool,
) -> Result<(), String> {
    ablegen_mit(speicher, n, von_mir, &n.von_name)
}

/// Dasselbe mit dem Namen der anderen Seite (beim Senden: der Empfaenger).
pub fn ablegen_mit(
    speicher: &openany_store::Speicher,
    n: &DirektNachricht,
    von_mir: bool,
    gegenueber: &str,
) -> Result<(), String> {
    let andere = if von_mir { &n.an_person } else { &n.von_person };
    speicher
        .nachricht_ablegen(&openany_store::Nachricht {
            konto: KONTO.into(),
            event_id: n.id.clone(),
            raum: andere.clone(),
            gegenueber: Some(if gegenueber.trim().is_empty() {
                "unnamed".into()
            } else {
                gegenueber.to_string()
            }),
            absender: n.von_person.clone(),
            von_mir,
            text: n.text.clone(),
            zeit: n.at.clone(),
            gelesen_at: None,
            anhang: None,
        })
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nur_unveraendert_und_vom_unterzeichnenden_geraet() {
        let anna = Identitaet::erzeugen().unwrap();
        let ben = Identitaet::erzeugen().unwrap();
        let wer = Wer {
            personen_id: "p-anna".into(),
            name: "Anna".into(),
        };
        let n = DirektNachricht::schreiben(&anna, &wer, "p-ben", "Hallo Ben").unwrap();

        assert!(n.echt_von(&anna.fingerabdruck));
        // Ein anderes Geraet reicht sie weiter: nein.
        assert!(!n.echt_von(&ben.fingerabdruck));

        let mut falsch = n.clone();
        falsch.text = "Gib mir dein Passwort".into();
        assert!(!falsch.echt_von(&anna.fingerabdruck));

        let mut anders = n.clone();
        anders.von_name = "Mama".into();
        assert!(!anders.echt_von(&anna.fingerabdruck));

        let leer = DirektNachricht::schreiben(&anna, &wer, "p-ben", "  ").unwrap();
        assert!(!leer.echt_von(&anna.fingerabdruck));
    }
}
