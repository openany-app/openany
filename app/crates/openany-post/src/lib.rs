//! E-Mail über ein vorhandenes Postfach -- der dritte Weg im Verlauf
//! (docs/plan-email-pgp.md, Schritt 2).
//!
//! **Kein Mailprogramm.** openany zeigt Mails als Unterhaltung: nur
//! Posteingang und „Gesendet", Text statt HTML, Anhänge als Kacheln. Das
//! Postfach gehört weiter dem Mailprogramm (Thunderbird o. ä.); openany
//! liest mit `BODY.PEEK` und setzt `\Seen` erst, wenn jemand eine Mail als
//! gelesen markiert.
//!
//! **Diese Kiste kennt weder Tauri noch den Speicher.** Sie bekommt ein
//! [`Konto`] und spricht mit dem Mailserver. Was davon wohin gehört, weiß die
//! App -- wie bei `openany-matrix-core`.
//!
//! **TLS mit den mitgelieferten Wurzeln** (`webpki-roots`), wie der
//! HTTP-Client der App: auf Android und am Schreibtisch dieselbe Liste.

mod autoconfig;
mod imap;
mod mime;
pub mod pgpmime;
pub mod schluessel;
mod smtp;

pub use autoconfig::finden as serverdaten_finden;
pub use autoconfig::Gefunden;
pub use imap::{
    abholen, ablegen, als_gelesen, loeschen, loeschen_nach_id, neueste, ordner_finden, pruefen,
    verschieben, warten, Abgeholt, Ordner, Roh, Stand,
};
pub use mime::{lesen as mail_lesen, Adresse, AnhangDaten, Mail};
pub use smtp::{senden, Entwurf, Gesendet, Verschluesselung};

use serde::{Deserialize, Serialize};

/// Wie mit dem Server gesprochen wird.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Sicherheit {
    /// Verschlüsselt von Anfang an (IMAP 993, SMTP 465).
    Ssl,
    /// Erst unverschlüsselt, dann `STARTTLS` (IMAP 143, SMTP 587).
    Starttls,
    /// NUR für Tests gegen einen Test-Mailserver -- gibt es in der App nicht.
    #[cfg(feature = "testserver")]
    Klartext,
}

/// Ein Server: Name, Anschluss, Art der Verschlüsselung.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Server {
    pub host: String,
    pub port: u16,
    pub sicherheit: Sicherheit,
}

/// Was nötig ist, um mit einem Postfach zu sprechen.
///
/// **Das Passwort steht hier drin** -- die App bewahrt das ganze Konto
/// deshalb im Tresor auf (Android-Keystore), wie die Matrix-Sitzung.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Konto {
    /// Die eigene Adresse, auch Absender beim Senden.
    pub adresse: String,
    /// Der Name, der beim Gegenüber als Absender steht (darf leer sein).
    #[serde(default)]
    pub anzeigename: String,
    pub imap: Server,
    pub smtp: Server,
    /// Meist die Adresse selbst; manche Anbieter wollen nur den Teil vor `@`.
    pub benutzer: String,
    pub passwort: String,
}

/// Was schiefgehen kann -- so, dass die Oberfläche es sagen kann.
#[derive(Debug, thiserror::Error)]
pub enum PostFehler {
    #[error("No connection to {0}: {1}")]
    Verbindung(String, String),
    #[error("Sign-in refused. Are user name and password correct? Some providers require a separate app password.")]
    Anmeldung,
    #[error("The mail server refused: {0}")]
    Server(String),
    #[error("The mail could not be built: {0}")]
    Bau(String),
    #[error("{0}")]
    Schluessel(String),
}

pub type Ergebnis<T> = Result<T, PostFehler>;

/// TLS-Einstellungen mit den mitgelieferten Wurzelzertifikaten.
fn tls() -> std::sync::Arc<rustls::ClientConfig> {
    let mut wurzeln = rustls::RootCertStore::empty();
    wurzeln.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config = rustls::ClientConfig::builder_with_provider(std::sync::Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .expect("ring knows the standard protocols")
    .with_root_certificates(wurzeln)
    .with_no_client_auth();
    std::sync::Arc::new(config)
}
