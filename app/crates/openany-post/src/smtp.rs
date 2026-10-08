//! Senden über SMTP.
//!
//! **Die Mail kommt roh zurück**, damit die App sie über IMAP in „Gesendet"
//! ablegen kann ([`crate::imap::ablegen`]). SMTP tut das nicht von selbst;
//! ohne diesen Schritt fehlte die eigene Hälfte der Unterhaltung im
//! Mailprogramm und nach einem Neuaufsetzen auch in openany.

use crate::mime::AnhangDaten;
use crate::{Ergebnis, Konto, PostFehler, Sicherheit};
use lettre::message::header::ContentType;
use lettre::message::{Attachment, Mailbox, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

/// Was gesendet werden soll.
#[derive(Debug, Clone, Default)]
pub struct Entwurf {
    pub an: Vec<String>,
    pub betreff: String,
    pub text: String,
    pub anhaenge: Vec<AnhangDaten>,
    /// Die Message-ID der Mail, auf die geantwortet wird (ohne `<>`).
    pub antwort_auf: Option<String>,
    /// Die Kette davor (ohne `<>`), für `References`.
    pub references: Vec<String>,
    /// Die Kopfzeile `Autocrypt:`, schon gefaltet
    /// ([`crate::schluessel::autocrypt_kopf`]) -- damit das Gegenüber den
    /// eigenen Schlüssel findet.
    pub autocrypt: Option<String>,
    /// Verschlüsseln (PGP/MIME, [`crate::pgpmime`]) -- `None`: im Klartext.
    pub verschluesselung: Option<Verschluesselung>,
}

/// Womit verschlüsselt wird.
#[derive(Debug, Clone, Default)]
pub struct Verschluesselung {
    /// Die öffentlichen Schlüssel der Empfänger, armored.
    pub empfaenger: Vec<String>,
    /// Der eigene geheime Schlüssel (entsperrt) -- zum Signieren, und damit
    /// die Mail auch an sich selbst verschlüsselt wird.
    pub eigener_geheim: String,
}

#[derive(Debug, Clone)]
pub struct Gesendet {
    /// Ohne spitze Klammern.
    pub message_id: String,
    pub roh: Vec<u8>,
}

fn bau(e: impl std::fmt::Display) -> PostFehler {
    PostFehler::Bau(e.to_string())
}

/// Die Mail bauen, ohne sie zu senden (getrennt, damit sie sich prüfen lässt).
pub(crate) fn bauen(konto: &Konto, entwurf: &Entwurf) -> Ergebnis<(Message, String)> {
    let von = if konto.anzeigename.trim().is_empty() {
        konto.adresse.parse::<Mailbox>().map_err(bau)?
    } else {
        Mailbox::new(
            Some(konto.anzeigename.trim().to_string()),
            konto.adresse.parse().map_err(bau)?,
        )
    };
    let domain = konto
        .adresse
        .rsplit_once('@')
        .map(|(_, d)| d)
        .unwrap_or("openany.local");
    let message_id = format!("{}@{}", uuid_ohne_bindestriche(), domain);

    // Verschlüsselt steht der echte Betreff nur innen; außen „...", wie bei
    // Thunderbird.
    let betreff_aussen = if entwurf.verschluesselung.is_some() {
        "..."
    } else {
        entwurf.betreff.trim()
    };
    let von_text = von.to_string();
    let mut b = Message::builder()
        .from(von)
        .subject(betreff_aussen)
        .message_id(Some(format!("<{message_id}>")))
        .user_agent("openany".into());
    if entwurf.an.is_empty() {
        return Err(PostFehler::Bau("No recipient.".into()));
    }
    for an in &entwurf.an {
        b = b.to(an
            .trim()
            .parse()
            .map_err(|_| PostFehler::Bau(format!("Not a valid address: {an}")))?);
    }
    if let Some(vorher) = &entwurf.antwort_auf {
        b = b.in_reply_to(format!("<{vorher}>"));
        let kette: Vec<String> = entwurf
            .references
            .iter()
            .chain(std::iter::once(vorher))
            .map(|r| format!("<{r}>"))
            .collect();
        b = b.references(kette.join(" "));
    }

    if let Some(kopf) = &entwurf.autocrypt {
        // Nur ASCII ohne Zeilenumbruch außer der Faltung „\r\n ": sonst
        // ließe sich über den Wert eine fremde Kopfzeile einschleusen.
        let sauber = kopf.is_ascii() && kopf.split("\r\n ").all(|z| !z.contains(['\r', '\n']));
        if !sauber {
            return Err(PostFehler::Bau("Invalid Autocrypt header.".into()));
        }
        b = b.raw_header(
            lettre::message::header::HeaderValue::dangerous_new_pre_encoded(
                lettre::message::header::HeaderName::new_from_ascii_str("Autocrypt"),
                kopf.clone(),
                kopf.clone(),
            ),
        );
    }

    if let Some(v) = &entwurf.verschluesselung {
        let innen = crate::pgpmime::inneres(
            &von_text,
            &entwurf.an,
            entwurf.betreff.trim(),
            &entwurf.text,
            &entwurf.anhaenge,
        );
        let zu = crate::pgpmime::verschluesseln(innen, &v.empfaenger, &v.eigener_geheim)?;
        let teile = MultiPart::encrypted("application/pgp-encrypted".into())
            .singlepart(
                SinglePart::builder()
                    .header(ContentType::parse("application/pgp-encrypted").map_err(bau)?)
                    .body("Version: 1\r\n".to_string()),
            )
            .singlepart(
                SinglePart::builder()
                    .header(
                        ContentType::parse("application/octet-stream; name=\"encrypted.asc\"")
                            .map_err(bau)?,
                    )
                    .header(
                        lettre::message::header::ContentDisposition::inline_with_name(
                            "encrypted.asc",
                        ),
                    )
                    .body(zu),
            );
        return Ok((b.multipart(teile).map_err(bau)?, message_id));
    }

    let text = SinglePart::plain(entwurf.text.clone());
    let mail = if entwurf.anhaenge.is_empty() {
        b.singlepart(text).map_err(bau)?
    } else {
        let mut teile = MultiPart::mixed().singlepart(text);
        for a in &entwurf.anhaenge {
            let art = ContentType::parse(&a.mime)
                .unwrap_or(ContentType::parse("application/octet-stream").map_err(bau)?);
            teile = teile.singlepart(Attachment::new(a.name.clone()).body(a.daten.clone(), art));
        }
        b.multipart(teile).map_err(bau)?
    };
    Ok((mail, message_id))
}

fn uuid_ohne_bindestriche() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    // Zeit + Zufall aus dem Speicherort eines frischen Werts genügt nicht;
    // deshalb 128 Bit aus ring.
    let mut zufall = [0u8; 16];
    ring_zufall(&mut zufall);
    let zeit = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!(
        "{zeit:x}.{}",
        zufall
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    )
}

fn ring_zufall(ziel: &mut [u8]) {
    use rustls::crypto::ring::default_provider;
    let _ = default_provider().secure_random.fill(ziel);
}

/// Senden. Gibt Message-ID und rohe Mail zurück.
pub async fn senden(konto: &Konto, entwurf: &Entwurf) -> Ergebnis<Gesendet> {
    let (mail, message_id) = bauen(konto, entwurf)?;
    let roh = mail.formatted();

    let s = &konto.smtp;
    let bauer = match s.sicherheit {
        Sicherheit::Ssl => AsyncSmtpTransport::<Tokio1Executor>::relay(&s.host),
        Sicherheit::Starttls => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&s.host),
        #[cfg(feature = "testserver")]
        Sicherheit::Klartext => Ok(AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(
            &s.host,
        )),
    }
    .map_err(|e| PostFehler::Verbindung(s.host.clone(), e.to_string()))?;
    let transport = bauer
        .port(s.port)
        .credentials(Credentials::new(
            konto.benutzer.clone(),
            konto.passwort.clone(),
        ))
        .timeout(Some(std::time::Duration::from_secs(60)))
        .build();

    transport.send(mail).await.map_err(|e| {
        if e.to_string().contains("535") || e.to_string().to_lowercase().contains("auth") {
            PostFehler::Anmeldung
        } else {
            PostFehler::Server(e.to_string())
        }
    })?;

    Ok(Gesendet { message_id, roh })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Server, Sicherheit};

    fn konto() -> Konto {
        Konto {
            adresse: "tiffy@beispiel.test".into(),
            anzeigename: "Tiffy".into(),
            imap: Server {
                host: "imap.beispiel.test".into(),
                port: 993,
                sicherheit: Sicherheit::Ssl,
            },
            smtp: Server {
                host: "smtp.beispiel.test".into(),
                port: 465,
                sicherheit: Sicherheit::Ssl,
            },
            benutzer: "tiffy@beispiel.test".into(),
            passwort: "geheim".into(),
        }
    }

    #[test]
    fn eine_antwort_mit_anhang_laesst_sich_wieder_lesen() {
        let entwurf = Entwurf {
            an: vec!["ferdinand@beispiel.test".into()],
            betreff: "Re: Grüße".into(),
            text: "Danke dir!".into(),
            anhaenge: vec![AnhangDaten {
                name: "plan.pdf".into(),
                mime: "application/pdf".into(),
                daten: b"%PDF-1.4\n".to_vec(),
            }],
            antwort_auf: Some("abc123@beispiel.test".into()),
            references: vec!["erst@beispiel.test".into()],
            autocrypt: None,
            verschluesselung: None,
        };
        let (mail, id) = bauen(&konto(), &entwurf).unwrap();
        let gelesen = crate::mime::lesen(&mail.formatted()).unwrap();

        assert_eq!(gelesen.message_id, id);
        assert_eq!(gelesen.betreff, "Re: Grüße");
        assert_eq!(gelesen.in_reply_to.as_deref(), Some("abc123@beispiel.test"));
        assert_eq!(
            gelesen.references,
            vec!["erst@beispiel.test", "abc123@beispiel.test"]
        );
        assert_eq!(gelesen.von.unwrap().adresse, "tiffy@beispiel.test");
        assert_eq!(gelesen.text, "Danke dir!");
        assert_eq!(gelesen.anhaenge[0].daten, b"%PDF-1.4\n");
    }

    #[test]
    fn ohne_empfaenger_wird_nichts_gebaut() {
        assert!(bauen(&konto(), &Entwurf::default()).is_err());
    }
}

#[cfg(test)]
mod autocrypt_tests {
    use super::*;
    use crate::{Server, Sicherheit};

    #[test]
    fn die_eigene_kopfzeile_kommt_beim_gegenueber_an() {
        let konto = Konto {
            adresse: "tiffy@beispiel.test".into(),
            anzeigename: String::new(),
            imap: Server {
                host: "i".into(),
                port: 993,
                sicherheit: Sicherheit::Ssl,
            },
            smtp: Server {
                host: "s".into(),
                port: 465,
                sicherheit: Sicherheit::Ssl,
            },
            benutzer: "tiffy@beispiel.test".into(),
            passwort: "x".into(),
        };
        let k = crate::schluessel::erzeugen("tiffy@beispiel.test", "").unwrap();
        let entwurf = Entwurf {
            an: vec!["ferdinand@beispiel.test".into()],
            betreff: "Hallo".into(),
            text: "Text".into(),
            autocrypt: Some(
                crate::schluessel::autocrypt_kopf("tiffy@beispiel.test", &k.oeffentlich).unwrap(),
            ),
            ..Default::default()
        };
        let (mail, _) = bauen(&konto, &entwurf).unwrap();
        let gelesen = crate::mime::lesen(&mail.formatted()).unwrap();
        let fremd = crate::schluessel::autocrypt_lesen(
            gelesen.autocrypt.as_deref().unwrap(),
            "tiffy@beispiel.test",
        )
        .unwrap();
        assert_eq!(fremd.fingerabdruck, k.fingerabdruck);
    }
}

#[cfg(test)]
mod pgp_tests {
    use super::*;
    use crate::{Server, Sicherheit};

    fn konto(adresse: &str) -> Konto {
        Konto {
            adresse: adresse.into(),
            anzeigename: "Tiffy".into(),
            imap: Server {
                host: "i".into(),
                port: 993,
                sicherheit: Sicherheit::Ssl,
            },
            smtp: Server {
                host: "s".into(),
                port: 465,
                sicherheit: Sicherheit::Ssl,
            },
            benutzer: adresse.into(),
            passwort: "x".into(),
        }
    }

    #[test]
    fn verschluesselte_mail_aussen_nichts_innen_alles() {
        let tiffy = crate::schluessel::erzeugen("tiffy@beispiel.test", "Tiffy").unwrap();
        let ferdinand = crate::schluessel::erzeugen("ferdinand@beispiel.test", "").unwrap();
        let entwurf = Entwurf {
            an: vec!["ferdinand@beispiel.test".into()],
            betreff: "Der Plan für Freitag".into(),
            text: "Geheim!".into(),
            anhaenge: vec![AnhangDaten {
                name: "plan.pdf".into(),
                mime: "application/pdf".into(),
                daten: b"%PDF".to_vec(),
            }],
            verschluesselung: Some(Verschluesselung {
                empfaenger: vec![ferdinand.oeffentlich.clone()],
                eigener_geheim: tiffy.geheim.clone(),
            }),
            ..Default::default()
        };
        let (mail, id) = bauen(&konto("tiffy@beispiel.test"), &entwurf).unwrap();
        let roh = mail.formatted();
        let roh_text = String::from_utf8_lossy(&roh);
        assert!(!roh_text.contains("Freitag") && !roh_text.contains("Geheim!"));

        let aussen = crate::mime::lesen(&roh).unwrap();
        assert_eq!(aussen.message_id, id);
        assert_eq!(aussen.betreff, "...");
        assert!(aussen.text.is_empty() && aussen.anhaenge.is_empty());
        let pgp = aussen.pgp.unwrap();
        assert!(!pgp.inline);

        let (innen, sig) = crate::pgpmime::entschluesseln(
            &pgp.nachricht,
            &ferdinand.geheim,
            Some(&tiffy.oeffentlich),
        )
        .unwrap();
        assert_eq!(sig, crate::pgpmime::Signatur::Gueltig);
        let innen = crate::mime::lesen(&innen).unwrap();
        assert_eq!(innen.betreff, "Der Plan für Freitag");
        assert_eq!(innen.text, "Geheim!");
        assert_eq!(innen.anhaenge[0].name, "plan.pdf");
    }

    #[test]
    fn inline_verschluesselt_wird_erkannt() {
        let ferdinand = crate::schluessel::erzeugen("ferdinand@beispiel.test", "").unwrap();
        let tiffy = crate::schluessel::erzeugen("tiffy@beispiel.test", "").unwrap();
        let zu = crate::pgpmime::verschluesseln(
            b"Nur Text".to_vec(),
            std::slice::from_ref(&ferdinand.oeffentlich),
            &tiffy.geheim,
        )
        .unwrap();
        let roh = format!("From: tiffy@beispiel.test\r\nTo: ferdinand@beispiel.test\r\nSubject: x\r\n\r\n{zu}\r\n");
        let m = crate::mime::lesen(roh.as_bytes()).unwrap();
        let pgp = m.pgp.unwrap();
        assert!(pgp.inline);
        let (klar, _) =
            crate::pgpmime::entschluesseln(&pgp.nachricht, &ferdinand.geheim, None).unwrap();
        assert_eq!(klar, b"Nur Text");
    }
}
