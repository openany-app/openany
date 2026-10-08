//! Ankuendigen und finden.
//!
//! **Zwei Wege, dasselbe Ergebnis.** Ein Geraet erfaehrt von einem anderen
//! entweder, weil es dessen Ankuendigung hoert und nachfragt (Discovery), oder
//! weil das andere sich bei ihm meldet (Register am eigenen Server). Beide
//! landen in derselben Liste, geschluesselt nach Fingerabdruck -- eine Adresse
//! ist Laufzeitzustand und wechselt, der Fingerabdruck nicht.
//!
//! **Uploads werden abgelehnt.** Der LocalSend-Server nimmt in diesem Schritt
//! keine Dateien an; eine Anfrage bekommt ohne Antwort einen Fehler zurueck.
//! Dateien sind APK 0.1 Schritt 5.

use crate::Identitaet;
use chrono::{DateTime, Utc};
use localsend::discovery::{DeviceIdentity, DiscoveryConfig, DiscoveryEvent, DiscoveryHandle};
use localsend::http::server::v2::ServerEventV2;
use localsend::http::server::web::WebConfig;
use localsend::http::server::{start_with_port, ServerConfigV2, ServerHandle, TlsConfig};
use localsend::http::state::ClientInfo;
use localsend::model::discovery::{DeviceType, ProtocolType, PROTOCOL_VERSION_V2};
use localsend::multicast::{
    MulticastDevice, DEFAULT_MULTICAST_GROUP, DEFAULT_MULTICAST_GROUP_V6, DEFAULT_PORT,
};
use localsend::util::interface::{local_interface_addresses, InterfaceFilter};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};

/// Woran ein anderes Geraet erkennt, dass hier openany laeuft und nicht eine
/// gewoehnliche LocalSend-App.
pub const KENNUNG_MODELL: &str = "openany";

#[derive(Debug, thiserror::Error)]
pub enum NahbereichFehler {
    #[error("Nearby devices could not be started: {0}")]
    Start(String),
}

/// Ein gefundenes Geraet.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct GeraetInDerNaehe {
    pub fingerabdruck: String,
    pub name: String,
    /// `openany` bei einem Programm wie diesem, sonst das, was die
    /// LocalSend-App meldet (etwa "Samsung").
    pub modell: Option<String>,
    pub openany: bool,
    pub adresse: String,
    pub port: u16,
    pub zuletzt: DateTime<Utc>,
}

type Liste = Arc<Mutex<HashMap<String, GeraetInDerNaehe>>>;

/// Ein laufender Nahbereich. Solange es ihn gibt, ist dieses Geraet im Netz
/// sichtbar; faellt er weg, halten Server und Suche an.
pub struct Nahbereich {
    liste: Liste,
    discovery: Arc<DiscoveryHandle>,
    _server: ServerHandle,
    _server_halt: oneshot::Sender<()>,
    _discovery_halt: oneshot::Sender<()>,
}

impl Nahbereich {
    pub async fn starten(ident: &Identitaet, name: &str) -> Result<Self, NahbereichFehler> {
        let (server_tx, mut server_rx) = mpsc::channel::<ServerEventV2>(64);
        let (server_halt, server_halt_rx) = oneshot::channel::<()>();

        let server = start_with_port(
            DEFAULT_PORT,
            Some(TlsConfig {
                cert: ident.zertifikat_pem.clone(),
                private_key: ident.schluessel_pem.clone(),
            }),
            ClientInfo {
                alias: name.to_string(),
                version: PROTOCOL_VERSION_V2.into(),
                device_model: Some(KENNUNG_MODELL.into()),
                device_type: Some(DeviceType::Mobile),
                token: ident.fingerabdruck.clone(),
            },
            None,
            Some(ServerConfigV2 {
                pin: None,
                verify_checksums: true,
                event_tx: server_tx,
            }),
            WebConfig::default(),
            server_halt_rx,
        )
        .await
        .map_err(|e| NahbereichFehler::Start(e.to_string()))?;

        let (disc_tx, mut disc_rx) = mpsc::channel::<DiscoveryEvent>(64);
        let (discovery_halt, discovery_halt_rx) = oneshot::channel::<()>();
        let discovery = Arc::new(
            localsend::discovery::start(
                DiscoveryConfig {
                    group: DEFAULT_MULTICAST_GROUP,
                    group_v6: Some(DEFAULT_MULTICAST_GROUP_V6),
                    port: DEFAULT_PORT,
                    interface_filter: InterfaceFilter::default(),
                    device: MulticastDevice {
                        alias: name.to_string(),
                        version: PROTOCOL_VERSION_V2.into(),
                        device_model: Some(KENNUNG_MODELL.into()),
                        device_type: Some(DeviceType::Mobile),
                        fingerprint: ident.fingerabdruck.clone(),
                        port: server.port(),
                        protocol: ProtocolType::Https,
                        download: false,
                    },
                    identity: DeviceIdentity {
                        cert_pem: ident.zertifikat_pem.clone(),
                        private_key_pem: ident.schluessel_pem.clone(),
                    },
                    timeout: Duration::from_millis(800),
                    event_tx: Some(disc_tx),
                },
                discovery_halt_rx,
            )
            .await,
        );

        let liste: Liste = Arc::new(Mutex::new(HashMap::new()));
        let eigener = ident.fingerabdruck.clone();

        // Gefunden ueber die Suche.
        tokio::spawn({
            let liste = liste.clone();
            let eigener = eigener.clone();
            async move {
                while let Some(ereignis) = disc_rx.recv().await {
                    let (DiscoveryEvent::Discovered { device }
                    | DiscoveryEvent::Updated { device }) = ereignis
                    else {
                        continue;
                    };
                    let Some(kanal) = device.channel.http() else {
                        continue;
                    };
                    eintragen(
                        &liste,
                        &eigener,
                        GeraetInDerNaehe {
                            openany: device.device_model.as_deref() == Some(KENNUNG_MODELL),
                            fingerabdruck: device.fingerprint,
                            name: device.alias,
                            modell: device.device_model,
                            adresse: kanal.host.clone(),
                            port: kanal.port,
                            zuletzt: Utc::now(),
                        },
                    );
                }
            }
        });

        // Gemeldet am eigenen Server. Alles andere (Uploads) wird fallen
        // gelassen -- ohne Antwort lehnt der Server ab.
        tokio::spawn({
            let liste = liste.clone();
            async move {
                while let Some(ereignis) = server_rx.recv().await {
                    if let ServerEventV2::Register { ip, info } = ereignis {
                        eintragen(
                            &liste,
                            &eigener,
                            GeraetInDerNaehe {
                                openany: info.device_model.as_deref() == Some(KENNUNG_MODELL),
                                fingerabdruck: info.fingerprint,
                                name: info.alias,
                                modell: info.device_model,
                                adresse: ip.ip.to_string(),
                                port: info.port,
                                zuletzt: Utc::now(),
                            },
                        );
                    }
                }
            }
        });

        let nah = Self {
            liste,
            discovery,
            _server: server,
            _server_halt: server_halt,
            _discovery_halt: discovery_halt,
        };
        nah.suchen();

        // ALLE 30 SEKUNDEN NEU. Die Liste verfaellt nach zwei Minuten; ohne
        // Nachsuchen stand ein gepaartes Geraet am 15.09.2026 als "nicht in
        // der Naehe" da, obwohl es im selben Hotspot offen war -- gesucht
        // wurde nur beim Start und auf Knopfdruck in den Einstellungen.
        //
        // `Weak`: Faellt der Nahbereich weg (neuer Name, Neustart), endet
        // auch diese Schleife, statt die alte Suche am Leben zu halten.
        let schwach = Arc::downgrade(&nah.discovery);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(NACHSUCHEN).await;
                let Some(discovery) = schwach.upgrade() else {
                    return;
                };
                suchen_mit(discovery);
            }
        });

        Ok(nah)
    }

    /// Ankuendigen und, falls Multicast nichts bringt, das Subnetz absuchen.
    ///
    /// Nur private Adressen -- das haelt oeffentliche Adressen heraus, aber
    /// NICHT jede Mobilfunk-Schnittstelle: Die des Telefons lag am 15.09.2026
    /// bei 10.230.x und ist damit ebenfalls privat. Ihr Scan kostet ein paar
    /// Sekunden fuer null Antworten; stoerend, aber nicht falsch.
    pub fn suchen(&self) {
        suchen_mit(self.discovery.clone());
    }

    /// Die letzte bekannte Adresse eines Geraets -- auch wenn es aus der
    /// Liste schon herausgefallen ist.
    ///
    /// Fuer den Abgleich mit einem GEPAARTEN Geraet reicht das: Ob unter der
    /// Adresse wirklich dieses Geraet antwortet, prueft der TLS-Handschlag
    /// gegen den Fingerabdruck. Eine veraltete Adresse ergibt einen Fehler,
    /// nie ein falsches Gegenueber.
    pub fn letzte_adresse(&self, fingerabdruck: &str) -> Option<String> {
        self.liste
            .lock()
            .ok()?
            .get(fingerabdruck)
            .map(|g| g.adresse.clone())
    }

    /// Wer ist da -- zuletzt Gesehene zuerst. Wer seit zwei Minuten nicht mehr
    /// geantwortet hat, faellt heraus.
    pub fn geraete(&self) -> Vec<GeraetInDerNaehe> {
        let grenze = Utc::now() - chrono::Duration::minutes(2);
        let mut liste: Vec<_> = self
            .liste
            .lock()
            .map(|l| {
                l.values()
                    .filter(|g| g.zuletzt >= grenze)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        liste.sort_by(|a, b| b.openany.cmp(&a.openany).then(b.zuletzt.cmp(&a.zuletzt)));
        liste
    }
}

/// Wie oft von selbst nachgesucht wird.
const NACHSUCHEN: Duration = Duration::from_secs(30);

fn suchen_mit(discovery: Arc<DiscoveryHandle>) {
    tokio::spawn(async move {
        let adressen = local_interface_addresses(&InterfaceFilter::default())
            .unwrap_or_default()
            .into_iter()
            .filter(|ip| ip.is_private())
            .collect();
        let _ = discovery
            .discover_staged(
                vec![],
                adressen,
                DEFAULT_PORT,
                ProtocolType::Https,
                Duration::from_secs(2),
            )
            .await;
    });
}

fn eintragen(liste: &Liste, eigener: &str, mut geraet: GeraetInDerNaehe) {
    if geraet.fingerabdruck == eigener {
        return;
    }
    if let Ok(mut l) = liste.lock() {
        // EINE IPv4-ADRESSE BLEIBT STEHEN. Ein Geraet meldet sich oft ueber
        // mehrere Wege, und ein link-lokales `fe80::…%11` gilt nur auf dieser
        // einen Schnittstelle -- zum Anzeigen und spaeter zum Verbinden ist die
        // IPv4-Adresse der bessere Weg.
        if let Some(alt) = l.get(&geraet.fingerabdruck) {
            if geraet.adresse.contains(':') && !alt.adresse.contains(':') {
                geraet.adresse = alt.adresse.clone();
                geraet.port = alt.port;
            }
        }
        l.insert(geraet.fingerabdruck.clone(), geraet);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn geraet(adresse: &str, name: &str) -> GeraetInDerNaehe {
        GeraetInDerNaehe {
            fingerabdruck: "F".into(),
            name: name.into(),
            modell: Some(KENNUNG_MODELL.into()),
            openany: true,
            adresse: adresse.into(),
            port: 53317,
            zuletzt: Utc::now(),
        }
    }

    #[test]
    fn ipv4_bleibt_stehen_der_name_wird_neu() {
        let liste: Liste = Arc::new(Mutex::new(HashMap::new()));
        eintragen(&liste, "ICH", geraet("10.218.109.44", "Dieses Gerät"));
        eintragen(
            &liste,
            "ICH",
            geraet("fe80::4c95:fbff:febc:4afc%11", "Telefon"),
        );

        let l = liste.lock().unwrap();
        assert_eq!(l["F"].adresse, "10.218.109.44");
        assert_eq!(l["F"].name, "Telefon");
    }

    #[test]
    fn das_eigene_geraet_steht_nicht_in_der_liste() {
        let liste: Liste = Arc::new(Mutex::new(HashMap::new()));
        let mut ich = geraet("10.0.0.1", "ich");
        ich.fingerabdruck = "ICH".into();
        eintragen(&liste, "ICH", ich);
        assert!(liste.lock().unwrap().is_empty());
    }
}
