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

use anyid_client::{Abholung, AnyidClient, AnyidError, Kopplungsstart, Ticket};
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
