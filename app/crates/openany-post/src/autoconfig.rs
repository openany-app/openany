//! Die Servernamen zu einer Adresse finden -- damit niemand „imap.gmx.net,
//! 993, SSL" abtippen muss.
//!
//! Dieselben Quellen wie Thunderbird, in derselben Reihenfolge:
//! 1. der Anbieter selbst (`autoconfig.<domain>/mail/config-v1.1.xml`),
//! 2. die Datenbank von Thunderbird (ISPDB, `autoconfig.thunderbird.net`).
//!
//! Beides ist dasselbe XML-Format. Findet sich nichts, trägt der Mensch die
//! Server von Hand ein -- `None` ist hier kein Fehler.

use crate::{Server, Sicherheit};

/// Was gefunden wurde: Server und wie der Benutzername aussieht.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Gefunden {
    pub imap: Server,
    pub smtp: Server,
    pub benutzer: String,
}

/// Sucht die Server zu `adresse`. `None`, wenn keine Quelle etwas weiß.
pub async fn finden(adresse: &str) -> Option<Gefunden> {
    let (_, domain) = adresse.trim().rsplit_once('@')?;
    let domain = domain.to_lowercase();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()?;

    let quellen = [
        format!("https://autoconfig.{domain}/mail/config-v1.1.xml?emailaddress={adresse}"),
        format!("https://autoconfig.thunderbird.net/v1.1/{domain}"),
    ];
    for url in quellen {
        let Ok(antwort) = client.get(&url).send().await else {
            continue;
        };
        if !antwort.status().is_success() {
            continue;
        }
        let Ok(text) = antwort.text().await else {
            continue;
        };
        if let Some(g) = lesen(&text, adresse) {
            return Some(g);
        }
    }
    None
}

/// Das XML von autoconfig lesen. Genommen wird der erste IMAP- und der erste
/// SMTP-Server mit Verschlüsselung -- Klartext-Server bietet openany nicht an.
pub(crate) fn lesen(xml: &str, adresse: &str) -> Option<Gefunden> {
    let dok = roxmltree::Document::parse(xml).ok()?;
    let lokal = adresse.split('@').next().unwrap_or(adresse);

    let server = |knoten: &str, art: &str| -> Option<(Server, String)> {
        dok.descendants()
            .filter(|n| n.has_tag_name(knoten) && n.attribute("type") == Some(art))
            .find_map(|n| {
                let kind = |name: &str| {
                    n.children()
                        .find(|c| c.has_tag_name(name))
                        .and_then(|c| c.text())
                        .map(str::trim)
                };
                let sicherheit = match kind("socketType")? {
                    "SSL" => Sicherheit::Ssl,
                    "STARTTLS" => Sicherheit::Starttls,
                    _ => return None,
                };
                let benutzer = kind("username")
                    .unwrap_or("%EMAILADDRESS%")
                    .replace("%EMAILADDRESS%", adresse)
                    .replace("%EMAILLOCALPART%", lokal);
                Some((
                    Server {
                        host: kind("hostname")?.to_string(),
                        port: kind("port")?.parse().ok()?,
                        sicherheit,
                    },
                    benutzer,
                ))
            })
    };

    let (imap, benutzer) = server("incomingServer", "imap")?;
    let (smtp, _) = server("outgoingServer", "smtp")?;
    Some(Gefunden {
        imap,
        smtp,
        benutzer,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// So liefert die ISPDB z. B. für gmx.net (gekürzt).
    const GMX: &str = r#"<?xml version="1.0"?>
<clientConfig version="1.1">
  <emailProvider id="gmx.net">
    <incomingServer type="pop3"><hostname>pop.gmx.net</hostname><port>995</port><socketType>SSL</socketType><username>%EMAILADDRESS%</username></incomingServer>
    <incomingServer type="imap"><hostname>imap.gmx.net</hostname><port>143</port><socketType>plain</socketType><username>%EMAILADDRESS%</username></incomingServer>
    <incomingServer type="imap"><hostname>imap.gmx.net</hostname><port>993</port><socketType>SSL</socketType><username>%EMAILADDRESS%</username></incomingServer>
    <outgoingServer type="smtp"><hostname>mail.gmx.net</hostname><port>587</port><socketType>STARTTLS</socketType><username>%EMAILLOCALPART%</username></outgoingServer>
  </emailProvider>
</clientConfig>"#;

    #[test]
    fn nimmt_den_ersten_verschluesselten_imap_und_smtp_server() {
        let g = lesen(GMX, "tiffy@beispiel.test").unwrap();
        assert_eq!(
            g.imap,
            Server {
                host: "imap.gmx.net".into(),
                port: 993,
                sicherheit: Sicherheit::Ssl
            }
        );
        assert_eq!(g.smtp.host, "mail.gmx.net");
        assert_eq!(g.smtp.port, 587);
        assert_eq!(g.smtp.sicherheit, Sicherheit::Starttls);
        assert_eq!(g.benutzer, "tiffy@beispiel.test");
    }

    #[test]
    fn ohne_verschluesselten_server_nichts() {
        let nur_klar = GMX.replace("SSL", "plain").replace("STARTTLS", "plain");
        assert!(lesen(&nur_klar, "tiffy@beispiel.test").is_none());
    }
}
