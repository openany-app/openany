//! Der duenne Teil: HTTP gegen anyid.

use crate::kopplung::{Kopplungsstart, Verifier};
use crate::{Abholung, Geraet, Kopplungscodes, Ticket};
use reqwest::{Client as Http, StatusCode};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AnyidError {
    #[error("Netzwerkfehler: {0}")]
    Transport(#[from] reqwest::Error),

    /// Das Geraet ist nicht (mehr) gekoppelt. Der einzige Fehler, auf den das
    /// Programm anders reagieren muss als mit "spaeter nochmal": Ein
    /// widerrufenes Geraet wird durch Wiederholen nicht wieder gueltig - es
    /// muss neu gekoppelt werden.
    #[error("Dieses Gerät ist nicht mehr gekoppelt.")]
    GeraetWiderrufen,

    /// Das Konto ist stillgelegt. Auch das hilft kein Wiederholen.
    #[error("Dieses Konto ist stillgelegt.")]
    KontoGesperrt,

    #[error("anyid hat abgelehnt ({status}): {meldung}")]
    Abgelehnt { status: StatusCode, meldung: String },
}

/// Die Gegenstelle des Programms bei anyid.
pub struct AnyidClient {
    http: Http,
    basis: String,
}

impl AnyidClient {
    /// `basis` ist die Wurzel von anyid, etwa `https://id.anytail.de`.
    pub fn neu(basis: impl Into<String>) -> Result<Self, AnyidError> {
        Self::neu_mit_ca(basis, None)
    }

    /// Mit einer zusaetzlichen Wurzel-CA.
    ///
    /// Gebraucht in der Entwicklung -- dort steht ein Caddy mit eigener CA
    /// davor -- und spaeter auch in Haeusern, die ihren Verkehr ueber eine
    /// eigene CA fuehren. **Kein Abschalten der Pruefung**: Die Kette wird
    /// weiterhin geprueft, nur gegen eine Wurzel mehr. Ein
    /// `danger_accept_invalid_certs` waere die Gewohnheit, die irgendwann in
    /// einem ausgelieferten Programm landet.
    pub fn neu_mit_ca(basis: impl Into<String>, ca_pem: Option<&[u8]>) -> Result<Self, AnyidError> {
        let mut bauer = Http::builder().timeout(Duration::from_secs(30));

        if let Some(pem) = ca_pem {
            bauer = bauer.add_root_certificate(reqwest::Certificate::from_pem(pem)?);
        }

        Ok(Self {
            http: bauer.build()?,
            basis: basis.into().trim_end_matches('/').to_string(),
        })
    }

    fn url(&self, pfad: &str) -> String {
        format!("{}/geraet{}", self.basis, pfad)
    }

    /// Schritt 1: Eine Kopplung beginnen.
    ///
    /// Der Verifier entsteht hier und verlaesst das Programm nicht. Hinaus
    /// geht nur sein Hash.
    pub async fn kopplung_beginnen(
        &self,
        anwendung: &str,
        geraetename: &str,
    ) -> Result<Kopplungsstart, AnyidError> {
        let verifier = Verifier::neu();

        let antwort = self
            .http
            .post(self.url("/kopplung"))
            .header("Accept", "application/json")
            .json(&serde_json::json!({
                "anwendung": anwendung,
                "geraet": geraetename,
                "challenge": verifier.challenge(),
            }))
            .send()
            .await?;

        let codes: Kopplungscodes = pruefen(antwort).await?.json().await?;

        Ok(Kopplungsstart {
            code: codes.code,
            geraetecode: codes.geraetecode,
            verifier,
            intervall: codes.intervall,
            ablauf_in: codes.ablauf_in,
        })
    }

    /// Schritt 3: Abholen.
    ///
    /// `Ungueltig` fasst vier Faelle zusammen - abgelaufen, erfunden, falscher
    /// Verifier, schon abgeholt. anyid unterscheidet sie absichtlich nicht;
    /// wer raet, soll aus der Antwort nicht lernen, wie nah er war.
    pub async fn kopplung_abholen(&self, start: &Kopplungsstart) -> Result<Abholung, AnyidError> {
        let antwort = self
            .http
            .post(self.url("/kopplung/abholen"))
            .header("Accept", "application/json")
            .json(&serde_json::json!({
                "geraetecode": start.geraetecode,
                "verifier": start.verifier.als_str(),
            }))
            .send()
            .await?;

        match antwort.status() {
            StatusCode::ACCEPTED => Ok(Abholung::Wartet {
                intervall: antwort
                    .json::<serde_json::Value>()
                    .await
                    .ok()
                    .and_then(|b| b.get("intervall").and_then(|i| i.as_u64()))
                    .unwrap_or(start.intervall),
            }),
            StatusCode::OK => {
                let körper: serde_json::Value = antwort.json().await?;

                let token = körper
                    .get("token")
                    .and_then(|t| t.as_str())
                    .unwrap_or_default()
                    .to_string();

                let geraet: Geraet = serde_json::from_value(
                    körper
                        .get("geraet")
                        .cloned()
                        .unwrap_or(serde_json::Value::Null),
                )
                .map_err(|_| AnyidError::Abgelehnt {
                    status: StatusCode::OK,
                    meldung: "Die Antwort von anyid war unvollständig.".into(),
                })?;

                Ok(Abholung::Fertig { token, geraet })
            }
            StatusCode::UNPROCESSABLE_ENTITY => Ok(Abholung::Ungueltig),
            status => Err(AnyidError::Abgelehnt {
                status,
                meldung: meldung(antwort).await,
            }),
        }
    }

    /// Wozu das Token gut ist: ein Ticket, das die Anwendung wie immer
    /// einloest.
    pub async fn ticket(&self, token: &str) -> Result<Ticket, AnyidError> {
        let antwort = self
            .http
            .post(self.url("/ticket"))
            .header("Accept", "application/json")
            // Eigenes Schema statt `Bearer`: Ein Geraetetoken ist kein
            // Zugriffstoken fuer eine API, sondern ein Ausweis fuer genau
            // einen Schalter.
            .header("Authorization", format!("Geraet {token}"))
            .send()
            .await?;

        match antwort.status() {
            StatusCode::UNAUTHORIZED => Err(AnyidError::GeraetWiderrufen),
            StatusCode::FORBIDDEN => Err(AnyidError::KontoGesperrt),
            _ => Ok(pruefen(antwort).await?.json().await?),
        }
    }
}

async fn pruefen(antwort: reqwest::Response) -> Result<reqwest::Response, AnyidError> {
    if antwort.status().is_success() {
        return Ok(antwort);
    }

    let status = antwort.status();

    Err(AnyidError::Abgelehnt {
        status,
        meldung: meldung(antwort).await,
    })
}

async fn meldung(antwort: reqwest::Response) -> String {
    antwort
        .json::<serde_json::Value>()
        .await
        .ok()
        .and_then(|b| {
            b.get("fehler")
                .or_else(|| b.get("message"))
                .and_then(|m| m.as_str())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "anyid hat die Anfrage abgelehnt.".to_string())
}
