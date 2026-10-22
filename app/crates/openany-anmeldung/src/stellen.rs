//! Die beiden Schalter, an denen dieses Programm ansteht -- als Traits.
//!
//! **Warum Traits und nicht die Clients selbst.** Die Kopplung ist eine
//! Zustandsmaschine mit vier Schritten ueber zwei Server, und genau die will
//! man pruefen koennen, ohne beide zu haben: Was passiert, wenn der Mensch nie
//! bestaetigt? Wenn er zu spaet bestaetigt? Wenn anyid das Geraet zwischen
//! Schritt drei und vier widerruft? Mit echten Servern sind das Faelle, die
//! man nur mit Warten und Gluecksspiel herstellt.
//!
//! Dieselbe Form wie [`openany_sync::Gegenstelle`] -- ein Trait je Gegenueber,
//! so eng wie moeglich geschnitten.

use anyid_client::{Abholung, Anfrage, AnyidClient, AnyidError, Kopplungsstart, Ticket};
use async_trait::async_trait;
use openany_client::{geraeteschluessel_holen, Geraeteschluessel, OpenanyError};

/// **anyid** -- sagt, *wer* jemand ist.
#[async_trait]
pub trait Ausweisstelle: Send + Sync {
    async fn kopplung_beginnen(
        &self,
        anwendung: &str,
        geraetename: &str,
    ) -> Result<Kopplungsstart, AnyidError>;

    async fn kopplung_abholen(&self, start: &Kopplungsstart) -> Result<Abholung, AnyidError>;

    async fn ticket(&self, geraetetoken: &str) -> Result<Ticket, AnyidError>;

    // ─── Z4: ein Geraet schaltet ein Geraet frei ──────────────────────────

    /// Eine Kopplung als Anfrage an andere Geraete: mit `name` an die des
    /// Kontos, mit `geraetetoken` an die eigenen (nur der Kontoschluessel).
    async fn kopplung_beginnen_fuer(
        &self,
        anwendung: &str,
        geraetename: &str,
        name: Option<&str>,
        geraetetoken: Option<&str>,
    ) -> Result<Kopplungsstart, AnyidError>;

    /// Briefkasten, Seite des neuen Geraets. `None`: die Kopplung ist fort.
    async fn briefkasten_neu(
        &self,
        start: &Kopplungsstart,
        inhalt: Option<&str>,
    ) -> Result<Option<Vec<String>>, AnyidError>;

    async fn anfragen(&self, geraetetoken: &str) -> Result<Vec<Anfrage>, AnyidError>;

    /// Briefkasten, Seite des alten Geraets. `None`: die Anfrage ist fort.
    async fn briefkasten_alt(
        &self,
        geraetetoken: &str,
        anfrage: &str,
        inhalt: Option<&str>,
    ) -> Result<Option<Vec<String>>, AnyidError>;

    async fn anfrage_bestaetigen(
        &self,
        geraetetoken: &str,
        anfrage: &str,
    ) -> Result<bool, AnyidError>;

    async fn anfrage_verwerfen(&self, geraetetoken: &str, anfrage: &str) -> Result<(), AnyidError>;
}

/// **openany** -- entscheidet, *was* dieses Geraet darf.
#[async_trait]
pub trait Schluesselstelle: Send + Sync {
    async fn schluessel_loesen(
        &self,
        ticket: &str,
        geraetename: &str,
    ) -> Result<Geraeteschluessel, OpenanyError>;
}

#[async_trait]
impl Ausweisstelle for AnyidClient {
    async fn kopplung_beginnen(
        &self,
        anwendung: &str,
        geraetename: &str,
    ) -> Result<Kopplungsstart, AnyidError> {
        AnyidClient::kopplung_beginnen(self, anwendung, geraetename).await
    }

    async fn kopplung_abholen(&self, start: &Kopplungsstart) -> Result<Abholung, AnyidError> {
        AnyidClient::kopplung_abholen(self, start).await
    }

    async fn ticket(&self, geraetetoken: &str) -> Result<Ticket, AnyidError> {
        AnyidClient::ticket(self, geraetetoken).await
    }

    async fn kopplung_beginnen_fuer(
        &self,
        anwendung: &str,
        geraetename: &str,
        name: Option<&str>,
        geraetetoken: Option<&str>,
    ) -> Result<Kopplungsstart, AnyidError> {
        AnyidClient::kopplung_beginnen_fuer(self, anwendung, geraetename, name, geraetetoken).await
    }

    async fn briefkasten_neu(
        &self,
        start: &Kopplungsstart,
        inhalt: Option<&str>,
    ) -> Result<Option<Vec<String>>, AnyidError> {
        AnyidClient::briefkasten_neu(self, start, inhalt).await
    }

    async fn anfragen(&self, geraetetoken: &str) -> Result<Vec<Anfrage>, AnyidError> {
        AnyidClient::anfragen(self, geraetetoken).await
    }

    async fn briefkasten_alt(
        &self,
        geraetetoken: &str,
        anfrage: &str,
        inhalt: Option<&str>,
    ) -> Result<Option<Vec<String>>, AnyidError> {
        AnyidClient::briefkasten_alt(self, geraetetoken, anfrage, inhalt).await
    }

    async fn anfrage_bestaetigen(
        &self,
        geraetetoken: &str,
        anfrage: &str,
    ) -> Result<bool, AnyidError> {
        AnyidClient::anfrage_bestaetigen(self, geraetetoken, anfrage).await
    }

    async fn anfrage_verwerfen(&self, geraetetoken: &str, anfrage: &str) -> Result<(), AnyidError> {
        AnyidClient::anfrage_verwerfen(self, geraetetoken, anfrage).await
    }
}

/// Der openany-Schalter.
///
/// Eine eigene kleine Struktur, weil das Einloesen drueben eine **freie
/// Funktion** ist und keine Methode: Zu diesem Zeitpunkt gibt es noch keinen
/// `OpenanyClient`, denn der braucht einen Schluessel -- und genau den gibt es
/// erst danach.
pub struct OpenanySchalter {
    basis: String,
    ca_pem: Option<Vec<u8>>,
}

impl OpenanySchalter {
    pub fn neu(basis: impl Into<String>) -> Self {
        Self {
            basis: basis.into(),
            ca_pem: None,
        }
    }

    /// Mit einer zusaetzlichen Wurzel-CA -- fuer die Entwicklung hinter Caddys
    /// eigener CA. **Kein Abschalten der Pruefung**, eine Wurzel mehr.
    pub fn mit_ca(mut self, pem: Vec<u8>) -> Self {
        self.ca_pem = Some(pem);
        self
    }
}

#[async_trait]
impl Schluesselstelle for OpenanySchalter {
    async fn schluessel_loesen(
        &self,
        ticket: &str,
        geraetename: &str,
    ) -> Result<Geraeteschluessel, OpenanyError> {
        geraeteschluessel_holen(&self.basis, ticket, geraetename, self.ca_pem.as_deref()).await
    }
}
