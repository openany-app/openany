//! Der eigene Dienst jedes Geraets: HTTPS mit gegenseitiger
//! Zertifikatspruefung, fuer Paaren und Abgleich.
//!
//! **Ein eigener Port neben LocalSend** ([`DIENST_PORT`]). LocalSends Server
//! kennt nur seine eigenen Wege und laesst keine weiteren zu. Gefunden wird
//! weiter ueber LocalSend; gesprochen wird hier.
//!
//! **Wer anruft, steht im Zertifikat.** Jede Anfrage traegt den Fingerabdruck
//! des Anrufers aus dem TLS-Handschlag, nicht aus dem Inhalt -- er laesst sich
//! also nicht behaupten.

use crate::paaren::{OffenePaarung, Paarungen};
use crate::tls::{dienst_konfiguration, fingerabdruck};
use crate::Identitaet;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio_rustls::TlsAcceptor;

/// Der Port des Dienstes -- einer neben LocalSends 53317.
pub const DIENST_PORT: u16 = 53318;

/// Besorgt ein Original, das hier nur „bei Bedarf" liegt -- vom Server oder
/// einem eigenen gepaarten Geraet. Damit kann ein Geraet auch weitergeben,
/// was es selbst noch nicht heruntergeladen hat.
#[async_trait::async_trait]
pub trait Besorger: Send + Sync {
    async fn besorgen(&self, abdruck: String, groesse: u64) -> Result<(), String>;
}

/// Was der Dienst von der Anwendung wissen und ihr sagen muss.
pub trait Gastgeber: Send + Sync + 'static {
    fn mein_name(&self) -> String;
    /// Das Geheimnis im gerade gezeigten QR-Code, falls einer gezeigt wird.
    fn qr_geheimnis(&self) -> Option<String>;
    /// Eine Paarung ist fertig: das andere Geraet dauerhaft merken.
    fn gepaart(&self, fingerabdruck: &str, name: &str);
    /// Ist dieses Geraet schon gepaart?
    fn ist_gepaart(&self, fingerabdruck: &str) -> bool;
    /// Der Speicher fuer den Abgleich. `None`: Dieses Geraet paart nur.
    fn speicher(&self) -> Option<crate::abgleich::GemeinsamerSpeicher> {
        None
    }
    /// Die Inhaltsablage -- ohne sie liefert dieses Geraet keine Dateiinhalte.
    fn inhalte(&self) -> Option<std::sync::Arc<openany_store::Inhalte>> {
        None
    }
    /// Ein anderes Geraet hat hier etwas geaendert -- die Oberflaeche soll
    /// neu lesen, auch wenn hier niemand getippt hat.
    fn angenommen(&self) {}
    /// Die Person hinter diesem Geraet (person.rs). `None`: noch keine --
    /// dann tauscht dieses Geraet keine aus.
    fn person(&self) -> Option<crate::Person> {
        None
    }
    fn person_merken(&self, _person: crate::Person) {}
    /// Offene Einladungen in lokale Projekte (einladen.rs). `None`: Dieses
    /// Geraet nimmt keine an.
    fn einladungen(&self) -> Option<crate::einladen::GemeinsameEinladungen> {
        None
    }
    /// Die Person eines ANDEREN Menschen merken (Mitglied eines Projekts) --
    /// fuer seine weiteren Geraete.
    fn person_fremd_merken(&self, _person: crate::Person) {}
    /// Ein lokales Projekt wurde hier aufgenommen -- die Oberflaeche liest neu.
    fn projekt_aufgenommen(&self) {}
    /// Ein Kontakt ist vor Ort per 6 Ziffern bestaetigt (einladen.rs,
    /// `KONTAKT`): die Person, gebunden an das Geraet `geraet`.
    fn kontakt_bestaetigt(&self, _person: crate::Person, _geraet: &str) {}
    /// Neue Chat-Nachrichten sind angekommen -- die Oberflaeche zeigt sie.
    fn chat_angekommen(&self) {}
    /// Darf diese Direktnachricht hier ankommen (direkt.rs)? Sie ist schon
    /// geprueft: echt und vom Geraet `anrufer`. Der Gastgeber entscheidet den
    /// Rest -- an diese Person? blockiert? gibt sich jemand als Bekannter
    /// aus? Anfragen erlaubt, Grenze erreicht? -- und merkt sich, ueber
    /// welches Geraet die Person erreichbar ist. Der Fehler ist eine Kennung
    /// aus `direkt::grund`.
    fn direkt_annehmen(
        &self,
        _n: &crate::direkt::DirektNachricht,
        _anrufer: &str,
    ) -> Result<(), String> {
        Err(crate::direkt::grund::NICHT_ANGENOMMEN.into())
    }
    /// Eine Direktnachricht ist abgelegt (fuer den Zaehler der Oberflaeche).
    fn direkt_angekommen(&self) {}
    /// Die Sperren fuer Notizen, die dieses Geraet zum Bearbeiten freigibt.
    /// `None`: Hier wird nichts gesperrt -- und also nichts bearbeitet.
    fn sperren(&self) -> Option<crate::sperren::GemeinsameSperren> {
        None
    }
    /// Wer fehlende Originale besorgt. `None`: Was hier fehlt, fehlt.
    fn besorger(&self) -> Option<Arc<dyn Besorger>> {
        None
    }
    /// Eine bekannte Person -- die eigene oder die eines Mitglieds, auch
    /// unter einer frueheren Id.
    fn person_von(&self, id: &str) -> Option<crate::Person> {
        self.person()
            .filter(|p| p.personen_id == id || p.frueher.iter().any(|f| f == id))
    }
}

#[derive(Serialize, Deserialize)]
pub struct Anfrage {
    pub name: String,
    pub zufall: String,
}

#[derive(Serialize, Deserialize)]
pub struct Antwort {
    pub name: String,
    pub zufall: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Stand {
    /// Hat der Mensch am GEFRAGTEN Geraet schon bestaetigt?
    pub bestaetigt: bool,
    pub bekannt: bool,
    /// Drueben schon fertig und gemerkt. Dann gibt es dort keine offene
    /// Paarung mehr -- und das ist kein Abbruch, sondern das Gegenteil.
    #[serde(default)]
    pub gepaart: bool,
}

#[derive(Serialize, Deserialize)]
pub struct GeheimnisAnfrage {
    pub name: String,
    pub geheimnis: String,
}

/// Eine freigegebene Notiz bearbeiten (sperren.rs).
#[derive(Serialize, Deserialize)]
pub struct NotizPost {
    pub projekt: String,
    /// Die zk_id der Notiz (auf dem fuehrenden Geraet).
    pub notiz: String,
    #[serde(default)]
    pub titel: Option<String>,
    #[serde(default)]
    pub inhalt: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotizInhalt {
    pub titel: String,
    pub inhalt: String,
    pub geaendert_at: String,
}

#[derive(Serialize, Deserialize)]
pub struct ChatPost {
    pub projekt: String,
    pub nachrichten: Vec<crate::chat::Nachricht>,
}

pub type GemeinsamePaarungen = Arc<Mutex<Paarungen>>;

/// Eine offene Paarung ist fertig? Dann merken und aus der Liste nehmen.
pub fn abschliessen(
    paarungen: &GemeinsamePaarungen,
    gastgeber: &dyn Gastgeber,
    fp: &str,
) -> Option<OffenePaarung> {
    let fertig = paarungen.lock().ok()?.entfernen_wenn_fertig(fp)?;
    gastgeber.gepaart(&fertig.fingerabdruck, &fertig.name);
    Some(fertig)
}

pub struct Dienst {
    pub port: u16,
    _halt: oneshot::Sender<()>,
}

impl Dienst {
    pub async fn starten(
        ident: &Identitaet,
        port: u16,
        paarungen: GemeinsamePaarungen,
        gastgeber: Arc<dyn Gastgeber>,
    ) -> Result<Self, String> {
        let konfiguration = dienst_konfiguration(&ident.zertifikat_pem, &ident.schluessel_pem)
            .map_err(|e| e.to_string())?;
        let annehmer = TlsAcceptor::from(Arc::new(konfiguration));
        let lauscher = TcpListener::bind(("0.0.0.0", port))
            .await
            .map_err(|e| e.to_string())?;
        let port = lauscher.local_addr().map_err(|e| e.to_string())?.port();
        let ich = ident.fingerabdruck.clone();
        let ident = Arc::new(ident.clone());
        let (halt, mut halt_rx) = oneshot::channel::<()>();

        tokio::spawn(async move {
            loop {
                let (tcp, _) = tokio::select! {
                    angenommen = lauscher.accept() => match angenommen { Ok(v) => v, Err(_) => continue },
                    _ = &mut halt_rx => return,
                };
                let annehmer = annehmer.clone();
                let paarungen = paarungen.clone();
                let gastgeber = gastgeber.clone();
                let ich = ich.clone();
                let ident = ident.clone();
                tokio::spawn(async move {
                    let Ok(tls) = annehmer.accept(tcp).await else {
                        return;
                    };
                    let Some(anrufer) = tls
                        .get_ref()
                        .1
                        .peer_certificates()
                        .and_then(|k| k.first())
                        .map(fingerabdruck)
                    else {
                        return;
                    };
                    let dienst = hyper::service::service_fn(move |anfrage| {
                        let paarungen = paarungen.clone();
                        let gastgeber = gastgeber.clone();
                        let anrufer = anrufer.clone();
                        let ich = ich.clone();
                        let ident = ident.clone();
                        async move {
                            Ok::<_, std::convert::Infallible>(
                                beantworten(
                                    anfrage,
                                    &anrufer,
                                    &ich,
                                    &ident,
                                    &paarungen,
                                    gastgeber.as_ref(),
                                )
                                .await,
                            )
                        }
                    });
                    let _ = hyper::server::conn::http1::Builder::new()
                        .serve_connection(TokioIo::new(tls), dienst)
                        .await;
                });
            }
        });

        Ok(Self { port, _halt: halt })
    }
}

fn json<T: Serialize>(status: StatusCode, wert: &T) -> Response<Full<Bytes>> {
    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(
            serde_json::to_vec(wert).unwrap_or_default(),
        )))
        .unwrap_or_default()
}

fn fehler(status: StatusCode, meldung: &str) -> Response<Full<Bytes>> {
    json(status, &serde_json::json!({ "message": meldung }))
}

async fn lesen<T: for<'a> Deserialize<'a>>(anfrage: Request<Incoming>) -> Option<T> {
    let bytes = anfrage.into_body().collect().await.ok()?.to_bytes();
    serde_json::from_slice(&bytes).ok()
}

async fn beantworten(
    anfrage: Request<Incoming>,
    anrufer: &str,
    ich: &str,
    ident: &Identitaet,
    paarungen: &GemeinsamePaarungen,
    gastgeber: &dyn Gastgeber,
) -> Response<Full<Bytes>> {
    if crate::abgleich::ist_abgleich(anfrage.uri().path()) {
        return crate::abgleich::beantworten(anfrage, anrufer, gastgeber).await;
    }

    match (
        anfrage.method().clone(),
        anfrage.uri().path().to_string().as_str(),
    ) {
        (Method::POST, "/openany/v1/paaren") => {
            let Some(a) = lesen::<Anfrage>(anfrage).await else {
                return fehler(StatusCode::BAD_REQUEST, "unlesbar");
            };
            let zufall = uuid::Uuid::new_v4().simple().to_string();
            if let Ok(mut p) = paarungen.lock() {
                p.eingang(anrufer, &a.name, ich, &a.zufall, &zufall);
            }
            json(
                StatusCode::OK,
                &Antwort {
                    name: gastgeber.mein_name(),
                    zufall,
                },
            )
        }
        (Method::POST, "/openany/v1/paaren/bestaetigt") => {
            let bekannt = paarungen
                .lock()
                .map(|mut p| p.dort_bestaetigt(anrufer).is_some())
                .unwrap_or(false);
            if !bekannt {
                return fehler(StatusCode::NOT_FOUND, "No open pairing.");
            }
            abschliessen(paarungen, gastgeber, anrufer);
            json(StatusCode::OK, &serde_json::json!({}))
        }
        (Method::GET, "/openany/v1/paaren/stand") => {
            let stand = if gastgeber.ist_gepaart(anrufer) {
                Stand {
                    bestaetigt: true,
                    bekannt: true,
                    gepaart: true,
                }
            } else {
                paarungen
                    .lock()
                    .ok()
                    .and_then(|mut p| {
                        p.get(anrufer).map(|o| Stand {
                            bestaetigt: o.hier_bestaetigt,
                            bekannt: true,
                            gepaart: false,
                        })
                    })
                    .unwrap_or_default()
            };
            json(StatusCode::OK, &stand)
        }
        (Method::POST, "/openany/v1/paaren/abbrechen") => {
            if let Ok(mut p) = paarungen.lock() {
                p.abbrechen(anrufer);
            }
            json(StatusCode::OK, &serde_json::json!({}))
        }
        (Method::POST, "/openany/v1/paaren/geheimnis") => {
            let Some(a) = lesen::<GeheimnisAnfrage>(anfrage).await else {
                return fehler(StatusCode::BAD_REQUEST, "unlesbar");
            };
            match gastgeber.qr_geheimnis() {
                Some(g) if !g.is_empty() && g == a.geheimnis => {
                    if let Ok(mut p) = paarungen.lock() {
                        p.durch_geheimnis(anrufer, &a.name);
                    }
                    abschliessen(paarungen, gastgeber, anrufer);
                    json(
                        StatusCode::OK,
                        &Antwort {
                            name: gastgeber.mein_name(),
                            zufall: String::new(),
                        },
                    )
                }
                _ => fehler(
                    StatusCode::FORBIDDEN,
                    "This code is not (or no longer) valid.",
                ),
            }
        }
        // Die Person austauschen (person.rs) -- nur mit eigenen, gepaarten
        // Geraeten. Andere Menschen bekommen sie spaeter mit der Einladung.
        (Method::POST, "/openany/v1/person") => {
            if !gastgeber.ist_gepaart(anrufer) {
                return fehler(StatusCode::FORBIDDEN, "Not paired.");
            }
            let Some(fremde) = lesen::<crate::Person>(anfrage).await else {
                return fehler(StatusCode::BAD_REQUEST, "unlesbar");
            };
            let Some(mut eigene) = gastgeber.person() else {
                return fehler(StatusCode::SERVICE_UNAVAILABLE, "No person yet.");
            };
            match crate::person::aufnehmen(ident, &mut eigene, &fremde, anrufer) {
                Ok(()) => {
                    gastgeber.person_merken(eigene.clone());
                    json(StatusCode::OK, &eigene)
                }
                Err(e) => fehler(StatusCode::BAD_REQUEST, &e.to_string()),
            }
        }
        (Method::POST, "/openany/v1/einladung") => {
            let (Some(einladungen), Some(_)) = (gastgeber.einladungen(), gastgeber.person()) else {
                return fehler(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "This device does not accept invitations.",
                );
            };
            let Some(a) = lesen::<crate::einladen::Anfrage>(anfrage).await else {
                return fehler(StatusCode::BAD_REQUEST, "unlesbar");
            };
            let zufall = uuid::Uuid::new_v4().simple().to_string();
            if let Ok(mut e) = einladungen.lock() {
                e.eingang(anrufer, &a.geraet, ich, &a, &zufall);
            }
            json(
                StatusCode::OK,
                &crate::einladen::Antwort {
                    geraet: gastgeber.mein_name(),
                    zufall,
                },
            )
        }
        (Method::GET, "/openany/v1/einladung/stand") => {
            let bestaetigt = gastgeber
                .einladungen()
                .and_then(|e| e.lock().ok()?.get(anrufer).map(|o| o.hier_bestaetigt))
                .unwrap_or(false);
            json(
                StatusCode::OK,
                &crate::einladen::Stand {
                    bestaetigt,
                    person: if bestaetigt { gastgeber.person() } else { None },
                },
            )
        }
        // Der Stand eines lokalen Projekts, fuer ein Geraet eines Mitglieds
        // (projektnah.rs).
        (Method::POST, "/openany/v1/projekt/stand") => {
            let Some(speicher) = gastgeber.speicher() else {
                return fehler(StatusCode::SERVICE_UNAVAILABLE, "No storage.");
            };
            let Some(frage) = lesen::<crate::projektnah::Frage>(anfrage).await else {
                return fehler(StatusCode::BAD_REQUEST, "unlesbar");
            };
            let speicher = speicher.lock().await;
            match crate::projektnah::stand_bauen(&speicher, gastgeber, &frage.projekt, anrufer) {
                Ok(stand) => json(StatusCode::OK, &stand),
                Err(e) => fehler(StatusCode::FORBIDDEN, &e),
            }
        }
        // Die Bytes einer freigegebenen Datei (oder ihres Vorschaubilds) --
        // nur fuer ein Mitglied, und nur aus einer EIGENEN geltenden
        // Freigabe dieses Geraets (projektnah.rs, `inhalt_erlaubt`).
        (Method::GET, "/openany/v1/projekt/inhalt") => {
            let q = |n| {
                crate::abgleich::abfrage(&anfrage, n)
                    .unwrap_or_default()
                    .to_string()
            };
            let (projekt, hash) = (q("projekt"), q("hash"));
            let von: u64 = q("von").parse().unwrap_or(0);
            let laenge = q("laenge")
                .parse()
                .unwrap_or(openany_sync::STUECK)
                .min(openany_sync::STUECK);
            let (Some(speicher), Some(inhalte)) = (gastgeber.speicher(), gastgeber.inhalte())
            else {
                return fehler(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "This device does not deliver content.",
                );
            };
            let erlaubt = {
                let speicher = speicher.lock().await;
                crate::projektnah::inhalt_erlaubt(&speicher, gastgeber, &projekt, anrufer, &hash)
            };
            let groesse = match erlaubt {
                Ok(Some(g)) => g,
                Ok(None) => return fehler(StatusCode::FORBIDDEN, "Not shared."),
                Err(e) => return fehler(StatusCode::FORBIDDEN, &e),
            };
            if !inhalte.hat(&hash) {
                // Liegt hier nur „bei Bedarf": im Hintergrund besorgen und
                // den Anrufer gleich noch einmal fragen lassen -- ein grosses
                // Original vom Server dauerte laenger, als eine Anfrage
                // warten darf.
                let Some(besorger) = gastgeber.besorger() else {
                    return fehler(
                        StatusCode::NOT_FOUND,
                        "This content is not on the sharing device.",
                    );
                };
                let h = hash.clone();
                tokio::spawn(async move {
                    let _ = besorger.besorgen(h, groesse).await;
                });
                return Response::builder()
                    .status(StatusCode::SERVICE_UNAVAILABLE)
                    .header("content-type", "application/json")
                    .header("retry-after", "3")
                    .body(Full::new(Bytes::from(
                        serde_json::to_vec(&serde_json::json!({
                            "message": "The original is being fetched onto the sharing device right now.",
                        }))
                        .unwrap_or_default(),
                    )))
                    .unwrap_or_default();
            }
            match inhalte.lesen(&hash, von, laenge as usize) {
                Ok(b) => Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/octet-stream")
                    .body(Full::new(Bytes::from(b)))
                    .unwrap_or_default(),
                Err(e) => fehler(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        // Eine fortgesetzte Mitgliederliste annehmen -- etwa ein Austritt,
        // den ein Mitglied vor Ort herueberschickt, bevor es geht.
        (Method::POST, "/openany/v1/projekt/liste") => {
            let Some(speicher) = gastgeber.speicher() else {
                return fehler(StatusCode::SERVICE_UNAVAILABLE, "No storage.");
            };
            let Some(liste) = lesen::<crate::Mitgliederliste>(anfrage).await else {
                return fehler(StatusCode::BAD_REQUEST, "unlesbar");
            };
            let ergebnis = {
                let speicher = speicher.lock().await;
                crate::projektnah::liste_annehmen(
                    &speicher,
                    gastgeber,
                    &liste.projekt.clone(),
                    anrufer,
                    &liste,
                )
            };
            match ergebnis {
                Ok(()) => {
                    gastgeber.projekt_aufgenommen();
                    json(StatusCode::OK, &serde_json::json!({}))
                }
                Err(e) => fehler(StatusCode::FORBIDDEN, &e),
            }
        }
        // Chat-Nachrichten, die ein Mitglied vor Ort gerade geschrieben hat.
        // Wer bin ich? Fuer jedes Geraet -- wer schreiben will, muss wissen,
        // an wen (direkt.rs).
        (Method::GET, "/openany/v1/wer") => match gastgeber.person() {
            Some(p) => json(
                StatusCode::OK,
                &crate::direkt::Wer {
                    personen_id: p.personen_id,
                    name: p.name,
                },
            ),
            None => fehler(StatusCode::SERVICE_UNAVAILABLE, "No person yet."),
        },
        // Eine Direktnachricht -- von jedem Geraet, aber nur echt und nur von
        // dem, das sie unterschrieben hat.
        (Method::POST, "/openany/v1/nachricht") => {
            let Some(n) = lesen::<crate::direkt::DirektNachricht>(anfrage).await else {
                return fehler(StatusCode::BAD_REQUEST, "unlesbar");
            };
            if !n.echt_von(anrufer) {
                return fehler(StatusCode::FORBIDDEN, "Not genuine.");
            }
            // Der Grund geht als Kennung mit (`direkt::grund`), damit der
            // Absender „nicht zugestellt" in seiner Sprache sagen kann.
            if let Err(kennung) = gastgeber.direkt_annehmen(&n, anrufer) {
                return json(
                    StatusCode::FORBIDDEN,
                    &serde_json::json!({
                        "message": crate::direkt::ablehnung_text(&kennung),
                        "grund": kennung,
                    }),
                );
            }
            let Some(speicher) = gastgeber.speicher() else {
                return fehler(StatusCode::SERVICE_UNAVAILABLE, "No storage.");
            };
            let abgelegt = crate::direkt::ablegen(&*speicher.lock().await, &n, false);
            match abgelegt {
                Ok(()) => {
                    gastgeber.direkt_angekommen();
                    json(StatusCode::OK, &serde_json::json!({}))
                }
                Err(e) => fehler(StatusCode::INTERNAL_SERVER_ERROR, &e),
            }
        }
        (Method::POST, "/openany/v1/projekt/chat") => {
            let Some(speicher) = gastgeber.speicher() else {
                return fehler(StatusCode::SERVICE_UNAVAILABLE, "No storage.");
            };
            let Some(post) = lesen::<ChatPost>(anfrage).await else {
                return fehler(StatusCode::BAD_REQUEST, "unlesbar");
            };
            let ergebnis = {
                let speicher = speicher.lock().await;
                match crate::projektnah::ist_mitgliedsgeraet(
                    &speicher,
                    gastgeber,
                    &post.projekt,
                    anrufer,
                ) {
                    Ok(true) => crate::projektnah::chat_aufnehmen(
                        &speicher,
                        gastgeber,
                        &post.projekt,
                        &post.nachrichten,
                    ),
                    Ok(false) => Err("Not a member of this project.".to_string()),
                    Err(e) => Err(e),
                }
            };
            match ergebnis {
                Ok(neu) => {
                    if neu > 0 {
                        gastgeber.chat_angekommen();
                    }
                    json(StatusCode::OK, &serde_json::json!({ "neu": neu }))
                }
                Err(e) => fehler(StatusCode::FORBIDDEN, &e),
            }
        }
        (Method::POST, "/openany/v1/projekt/notiz/sperren")
        | (Method::POST, "/openany/v1/projekt/notiz/speichern")
        | (Method::POST, "/openany/v1/projekt/notiz/entsperren") => {
            let pfad = anfrage.uri().path().to_string();
            notiz_bearbeiten(&pfad, anfrage, anrufer, gastgeber).await
        }
        (Method::POST, "/openany/v1/einladung/aufnahme") => {
            einladung_aufnehmen(anfrage, anrufer, ich, gastgeber).await
        }
        // B: A schickt nach beidseitigem „Passt" die eigene Person (Kontakt).
        (Method::POST, "/openany/v1/kontakt/aufnahme") => {
            kontakt_aufnehmen(anfrage, anrufer, gastgeber).await
        }
        (Method::POST, "/openany/v1/einladung/abbrechen") => {
            if let Some(e) = gastgeber.einladungen() {
                if let Ok(mut e) = e.lock() {
                    e.entfernen(anrufer);
                }
            }
            json(StatusCode::OK, &serde_json::json!({}))
        }
        _ => fehler(StatusCode::NOT_FOUND, "Unbekannt."),
    }
}

/// Eine freigegebene Notiz sperren, speichern oder loslassen -- auf dem
/// Geraet, das sie fuehrt (sperren.rs).
async fn notiz_bearbeiten(
    pfad: &str,
    anfrage: Request<Incoming>,
    anrufer: &str,
    gastgeber: &dyn Gastgeber,
) -> Response<Full<Bytes>> {
    let (Some(speicher), Some(sperren)) = (gastgeber.speicher(), gastgeber.sperren()) else {
        return fehler(
            StatusCode::SERVICE_UNAVAILABLE,
            "Nothing is being edited here.",
        );
    };
    let Some(post) = lesen::<NotizPost>(anfrage).await else {
        return fehler(StatusCode::BAD_REQUEST, "unlesbar");
    };
    let speicher = speicher.lock().await;
    let (wer, mut notiz) = match crate::projektnah::bearbeitbare_notiz(
        &speicher,
        gastgeber,
        &post.projekt,
        anrufer,
        &post.notiz,
    ) {
        Ok(v) => v,
        Err(e) => return fehler(StatusCode::FORBIDDEN, &e),
    };
    let halter = crate::sperren::Halter {
        personen_id: wer.personen_id.clone(),
        name: wer.name.clone(),
        geraet: anrufer.to_string(),
    };
    let Ok(mut s) = sperren.lock() else {
        return fehler(StatusCode::INTERNAL_SERVER_ERROR, "Locks cannot be read.");
    };
    match pfad {
        "/openany/v1/projekt/notiz/sperren" => match s.nehmen(&notiz.zk_id, halter) {
            Ok(()) => json(
                StatusCode::OK,
                &NotizInhalt {
                    titel: notiz.titel,
                    inhalt: notiz.inhalt,
                    geaendert_at: notiz.geaendert_at,
                },
            ),
            Err(h) => fehler(
                StatusCode::CONFLICT,
                &format!(
                    "{} is editing this note right now.",
                    if h.name.is_empty() { "Jemand" } else { &h.name }
                ),
            ),
        },
        "/openany/v1/projekt/notiz/speichern" => {
            if !s.haelt(&notiz.zk_id, &halter) {
                return fehler(
                    StatusCode::CONFLICT,
                    "The lock has expired -- please open it for editing again.",
                );
            }
            let _ = s.nehmen(&notiz.zk_id, halter);
            drop(s);
            if let Some(t) = post.titel {
                notiz.titel = t;
            }
            if let Some(i) = post.inhalt {
                notiz.inhalt = i;
            }
            notiz.geaendert_at = String::new();
            // Gemerkt: Die Notiz gehoert der Person dieses Geraets und reist
            // zu ihren anderen Geraeten und ihrem Server wie jede eigene.
            match speicher.notiz_schreiben(&notiz, openany_store::Protokoll::Merken) {
                Ok(()) => {
                    gastgeber.angenommen();
                    json(StatusCode::OK, &serde_json::json!({}))
                }
                Err(e) => fehler(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        _ => {
            s.loslassen(&notiz.zk_id, &halter);
            json(StatusCode::OK, &serde_json::json!({}))
        }
    }
}

/// B nimmt einen Kontakt auf: nur zu einer hier bestaetigten
/// Kontakt-Einladung von genau diesem Geraet, und nur, wenn die Person sich
/// mit diesem Geraet vorstellt.
async fn kontakt_aufnehmen(
    anfrage: Request<Incoming>,
    anrufer: &str,
    gastgeber: &dyn Gastgeber,
) -> Response<Full<Bytes>> {
    let Some(einladungen) = gastgeber.einladungen() else {
        return fehler(
            StatusCode::SERVICE_UNAVAILABLE,
            "This device does not accept invitations.",
        );
    };
    let Some(person) = lesen::<crate::Person>(anfrage).await else {
        return fehler(StatusCode::BAD_REQUEST, "unlesbar");
    };
    let Some(offen) = einladungen
        .lock()
        .ok()
        .and_then(|mut e| e.get(anrufer).cloned())
    else {
        return fehler(StatusCode::NOT_FOUND, "No open invitation.");
    };
    if offen.projekt != crate::einladen::KONTAKT
        || offen.rolle != crate::paaren::Rolle::Gefragt
        || !offen.hier_bestaetigt
    {
        return fehler(StatusCode::FORBIDDEN, "not confirmed here");
    }
    if !person.gueltige(&[anrufer]).contains_key(anrufer) {
        return fehler(
            StatusCode::FORBIDDEN,
            "The other device does not introduce itself.",
        );
    }
    gastgeber.kontakt_bestaetigt(person, anrufer);
    if let Ok(mut e) = einladungen.lock() {
        e.entfernen(anrufer);
    }
    json(StatusCode::OK, &serde_json::json!({}))
}

/// B nimmt ein Projekt auf, in das A eingeladen hat -- erst nach eigener
/// Pruefung (einladen.rs, `aufnahme_pruefen`).
async fn einladung_aufnehmen(
    anfrage: Request<Incoming>,
    anrufer: &str,
    ich: &str,
    gastgeber: &dyn Gastgeber,
) -> Response<Full<Bytes>> {
    let (Some(einladungen), Some(person), Some(speicher)) = (
        gastgeber.einladungen(),
        gastgeber.person(),
        gastgeber.speicher(),
    ) else {
        return fehler(
            StatusCode::SERVICE_UNAVAILABLE,
            "This device does not accept invitations.",
        );
    };
    let Some(aufnahme) = lesen::<crate::einladen::Aufnahme>(anfrage).await else {
        return fehler(StatusCode::BAD_REQUEST, "unlesbar");
    };
    let Some(offen) = einladungen
        .lock()
        .ok()
        .and_then(|mut e| e.get(anrufer).cloned())
    else {
        return fehler(StatusCode::NOT_FOUND, "No open invitation.");
    };
    if let Err(e) = crate::einladen::aufnahme_pruefen(&aufnahme, &offen, &person, ich) {
        return fehler(StatusCode::FORBIDDEN, &e.to_string());
    }
    let Ok(liste) = serde_json::to_string(&aufnahme.mitgliederliste) else {
        return fehler(StatusCode::BAD_REQUEST, "unlesbar");
    };
    let projekt = openany_store::Projekt {
        uuid: aufnahme.mitgliederliste.projekt.clone(),
        name: aufnahme.projekt_name.clone(),
        rolle: crate::mitglieder::MITGLIED.into(),
        geaendert_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        mitgliederliste: None,
    };
    if let Err(e) = speicher
        .lock()
        .await
        .projekt_lokal_schreiben(&projekt, &liste)
    {
        return fehler(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
    }
    gastgeber.person_fremd_merken(aufnahme.eigentuemer);
    if let Ok(mut e) = einladungen.lock() {
        e.entfernen(anrufer);
    }
    gastgeber.projekt_aufgenommen();
    json(StatusCode::OK, &serde_json::json!({}))
}
