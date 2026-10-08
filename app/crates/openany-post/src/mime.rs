//! Eine rohe Mail in das, was der Verlauf zeigt.
//!
//! **Text statt HTML.** Hat eine Mail einen Textteil, gilt der; sonst wird
//! das HTML zu Text (mail-parser kann das). HTML wird nie als HTML gezeigt:
//! In einer fremden Mail ist es die bequemste Stelle für Nachverfolgung und
//! Schlimmeres.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Adresse {
    pub name: Option<String>,
    pub adresse: String,
}

#[derive(Debug, Clone)]
pub struct AnhangDaten {
    pub name: String,
    pub mime: String,
    pub daten: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Mail {
    /// `Message-ID` ohne spitze Klammern; fehlt sie, bleibt sie leer.
    pub message_id: String,
    pub in_reply_to: Option<String>,
    pub references: Vec<String>,
    pub von: Option<Adresse>,
    pub an: Vec<Adresse>,
    pub betreff: String,
    pub text: String,
    /// Sekunden seit 1970 (Date-Kopf); `None`, wenn er fehlt oder unlesbar ist.
    pub zeit: Option<i64>,
    pub anhaenge: Vec<AnhangDaten>,
    /// Die Kopfzeile `Autocrypt:`, roh -- ob sie gilt, prüft
    /// [`crate::schluessel::autocrypt_lesen`] gegen den Absender.
    pub autocrypt: Option<String>,
    /// Ist die Mail verschlüsselt: die OpenPGP-Nachricht (armored). Text und
    /// Anhänge sind dann leer; was drin steht, sagt erst
    /// [`crate::pgpmime::entschluesseln`].
    pub pgp: Option<Pgp>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pgp {
    pub nachricht: Vec<u8>,
    /// `true`: „inline" -- der Text selbst war die Nachricht, entschlüsselt
    /// kommt Text heraus. `false`: PGP/MIME, entschlüsselt kommt ein
    /// MIME-Teil heraus.
    pub inline: bool,
}

/// Eine rohe Mail lesen. `None`, wenn es keine ist.
pub fn lesen(roh: &[u8]) -> Option<Mail> {
    use mail_parser::{Address, HeaderValue, MessageParser, MimeHeaders};

    let m = MessageParser::default().parse(roh)?;

    let adressen = |a: Option<&Address>| -> Vec<Adresse> {
        a.map(|a| {
            a.iter()
                .filter_map(|x| {
                    Some(Adresse {
                        name: x
                            .name()
                            .map(str::to_string)
                            .filter(|n| !n.trim().is_empty()),
                        adresse: x.address()?.trim().to_lowercase(),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
    };
    let ids = |h: &HeaderValue| -> Vec<String> {
        match h {
            HeaderValue::Text(t) => vec![t.to_string()],
            HeaderValue::TextList(l) => l.iter().map(|t| t.to_string()).collect(),
            _ => Vec::new(),
        }
    };

    let text = m
        .body_text(0)
        .map(|t| t.into_owned())
        .unwrap_or_default()
        .trim_end()
        .to_string();

    let anhaenge = m
        .attachments()
        .filter_map(|teil| {
            let mime = teil
                .content_type()
                .map(|c| match c.subtype() {
                    Some(sub) => format!("{}/{}", c.ctype(), sub),
                    None => c.ctype().to_string(),
                })
                .unwrap_or_else(|| "application/octet-stream".into())
                .to_lowercase();
            // Eingebettete Mails (message/rfc822) und leere Teile nicht als
            // Anhang zeigen -- sie sind kein Dokument, das jemand öffnen will.
            let daten = teil.contents().to_vec();
            if daten.is_empty() {
                return None;
            }
            Some(AnhangDaten {
                name: teil.attachment_name().unwrap_or("Anhang").to_string(),
                mime,
                daten,
            })
        })
        .collect();

    // VERSCHLÜSSELT? PGP/MIME (RFC 3156): multipart/encrypted mit einem Teil,
    // der die Nachricht trägt. Inline: Der Text beginnt mit ihr.
    const ANFANG: &str = "-----BEGIN PGP MESSAGE-----";
    let pgp_mime = m
        .content_type()
        .is_some_and(|c| {
            c.ctype().eq_ignore_ascii_case("multipart")
                && c.subtype()
                    .is_some_and(|s| s.eq_ignore_ascii_case("encrypted"))
        })
        .then(|| {
            m.attachments()
                .map(|t| t.contents())
                .find(|d| d.windows(ANFANG.len()).any(|w| w == ANFANG.as_bytes()))
                .map(|d| d.to_vec())
        })
        .flatten();
    let pgp = match pgp_mime {
        Some(nachricht) => Some(Pgp {
            nachricht,
            inline: false,
        }),
        None => text.trim_start().starts_with(ANFANG).then(|| Pgp {
            nachricht: text.trim().as_bytes().to_vec(),
            inline: true,
        }),
    };
    let (text, anhaenge) = if pgp.is_some() {
        (String::new(), Vec::new())
    } else {
        (text, anhaenge)
    };

    Some(Mail {
        message_id: m.message_id().unwrap_or_default().to_string(),
        in_reply_to: ids(m.in_reply_to()).into_iter().next(),
        references: ids(m.references()),
        von: adressen(m.from()).into_iter().next(),
        an: adressen(m.to()),
        betreff: m.subject().unwrap_or_default().to_string(),
        text,
        zeit: m.date().map(|d| d.to_timestamp()),
        anhaenge,
        autocrypt: m.header_raw("Autocrypt").map(|k| k.trim().to_string()),
        pgp,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const EINFACH: &str = "From: Ferdinand <Ferdinand@Beispiel.test>\r\n\
To: tiffy@beispiel.test\r\n\
Subject: =?UTF-8?Q?Gr=C3=BC=C3=9Fe?=\r\n\
Message-ID: <abc123@beispiel.test>\r\n\
In-Reply-To: <vorher@beispiel.test>\r\n\
Date: Mon, 28 Sep 2026 10:00:00 +0200\r\n\
Content-Type: text/plain; charset=utf-8\r\n\
\r\n\
Hallo Tiffy,\r\nwie geht's?\r\n";

    #[test]
    fn kopf_und_text() {
        let m = lesen(EINFACH.as_bytes()).unwrap();
        assert_eq!(m.message_id, "abc123@beispiel.test");
        assert_eq!(m.in_reply_to.as_deref(), Some("vorher@beispiel.test"));
        assert_eq!(m.betreff, "Grüße");
        let von = m.von.unwrap();
        assert_eq!(von.adresse, "ferdinand@beispiel.test");
        assert_eq!(von.name.as_deref(), Some("Ferdinand"));
        assert_eq!(m.an[0].adresse, "tiffy@beispiel.test");
        assert_eq!(m.text, "Hallo Tiffy,\r\nwie geht's?");
        assert_eq!(m.zeit, Some(1_790_582_400));
        assert!(m.anhaenge.is_empty());
    }

    #[test]
    fn html_wird_text_und_anhang_wird_erkannt() {
        let roh = "From: a@b.test\r\nTo: c@d.test\r\nSubject: Plan\r\n\
MIME-Version: 1.0\r\nContent-Type: multipart/mixed; boundary=\"XX\"\r\n\r\n\
--XX\r\nContent-Type: text/html; charset=utf-8\r\n\r\n<p>Siehe <b>Anhang</b></p>\r\n\
--XX\r\nContent-Type: application/pdf; name=\"plan.pdf\"\r\nContent-Disposition: attachment; filename=\"plan.pdf\"\r\n\
Content-Transfer-Encoding: base64\r\n\r\nJVBERi0xLjQK\r\n--XX--\r\n";
        let m = lesen(roh.as_bytes()).unwrap();
        assert!(m.text.contains("Siehe"));
        assert!(!m.text.contains("<p>"));
        assert_eq!(m.anhaenge.len(), 1);
        assert_eq!(m.anhaenge[0].name, "plan.pdf");
        assert_eq!(m.anhaenge[0].mime, "application/pdf");
        assert_eq!(m.anhaenge[0].daten, b"%PDF-1.4\n");
    }
}
