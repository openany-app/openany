//! Der Abgleich zwischen zwei gepaarten Geraeten -- die Leitung.
//!
//! Die Regeln stehen in `openany-sync` ([`Auskunft`] und `Laeufer`). Hier
//! steht nur, wie sie ueber das lokale Netz reisen: dieselbe JSON-Form wie
//! gegen openany, unter `/openany/v1/abgleich/…`, auf dem Dienst mit
//! gegenseitiger Zertifikatspruefung.
//!
//! **Nur gepaarte Geraete.** Der Fingerabdruck des Anrufers kommt aus dem
//! TLS-Handschlag; wer nicht in der Liste steht, bekommt 403, bevor der
//! Speicher auch nur aufgeschlossen wird.
//!
//! **Beide Geraete duerfen gleichzeitig tippen.** Der Laeufer haelt den
//! eigenen Speicher waehrend des ganzen Laufs. Tippen beide zugleich, wartet
//! jede Auskunft auf einen Speicher, den der eigene Laeufer haelt, der
//! seinerseits auf die andere Auskunft wartet. Deshalb wartet die Auskunft
//! nur kurz und sagt dann "beschaeftigt" -- ein Lauf, der mit einer klaren
//! Meldung scheitert, ist besser als zwei, die haengen.

use crate::dienst::Gastgeber;
use crate::tls::anruf_konfiguration;
use crate::Identitaet;
use async_trait::async_trait;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::{Method, Request, Response, StatusCode};
use openany_client::{Delta, Eintrag, Ergebnis, OpenanyError};
use openany_store::Speicher;
use openany_sync::{Auskunft, Gegenstelle};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;

/// Wie lange die Auskunft auf den eigenen Speicher wartet.
const WARTEN_AUF_SPEICHER: Duration = Duration::from_secs(3);

/// Wie ein anderes Geraet hier heisst, wenn es um Marken und Urspruenge geht.
///
/// **Nach dem Fingerabdruck, nicht nach der Adresse.** Adressen wechseln mit
/// jedem WLAN; eine neue duerfte keinen Erstabgleich voller Konfliktkopien
/// ausloesen.
pub fn basis_fuer(fingerabdruck: &str) -> String {
    format!("nah:{fingerabdruck}")
}

pub(crate) fn ist_abgleich(pfad: &str) -> bool {
    pfad.starts_with("/openany/v1/abgleich/")
}

fn antwort(status: StatusCode, typ: &str, koerper: Vec<u8>) -> Response<Full<Bytes>> {
    Response::builder()
        .status(status)
        .header("content-type", typ)
        .body(Full::new(Bytes::from(koerper)))
        .unwrap_or_default()
}

fn json_antwort(status: StatusCode, wert: &Value) -> Response<Full<Bytes>> {
    antwort(
        status,
        "application/json",
        serde_json::to_vec(wert).unwrap_or_default(),
    )
}

fn meldung(status: StatusCode, text: &str) -> Response<Full<Bytes>> {
    json_antwort(status, &json!({ "message": text }))
}

/// Ein Wert aus der Adresszeile. Die Schluessel hier (zk_id, uuid) brauchen
/// keine Umschreibung; was doch `%` traegt, wird nicht gefunden statt falsch.
pub(crate) fn abfrage<'a>(anfrage: &'a Request<Incoming>, name: &str) -> Option<&'a str> {
    anfrage.uri().query()?.split('&').find_map(|teil| {
        let (k, v) = teil.split_once('=')?;
        (k == name).then_some(v)
    })
}

#[derive(Deserialize)]
struct Marken {
    eigene: i64,
    fremde: i64,
}

/// Eine Abgleich-Anfrage beantworten.
pub(crate) async fn beantworten(
    anfrage: Request<Incoming>,
    anrufer: &str,
    gastgeber: &dyn Gastgeber,
) -> Response<Full<Bytes>> {
    if !gastgeber.ist_gepaart(anrufer) {
        return meldung(
            StatusCode::FORBIDDEN,
            "This device is not (or no longer) paired here.",
        );
    }

    // INHALTE OHNE DEN SPEICHER. Die Ablage braucht keine Sperre; ein Video,
    // das stueckweise hinuebergeht, soll weder den Laeufer dieses Geraets
    // noch seine Oberflaeche anhalten.
    match (anfrage.method(), anfrage.uri().path()) {
        (&Method::GET, "/openany/v1/abgleich/inhalt") => {
            let Some(inhalte) = gastgeber.inhalte() else {
                return meldung(
                    StatusCode::NOT_FOUND,
                    "This device does not deliver content.",
                );
            };
            let hash = abfrage(&anfrage, "hash").unwrap_or_default().to_string();
            let von = abfrage(&anfrage, "von")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
            let laenge = abfrage(&anfrage, "laenge")
                .and_then(|v| v.parse().ok())
                .unwrap_or(openany_sync::STUECK)
                .min(openany_sync::STUECK);
            if !inhalte.hat(&hash) {
                return meldung(StatusCode::NOT_FOUND, "This content is not here.");
            }
            return match inhalte.lesen(&hash, von, laenge as usize) {
                Ok(b) => antwort(StatusCode::OK, "application/octet-stream", b),
                Err(e) => meldung(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };
        }
        (&Method::POST, "/openany/v1/abgleich/inhalte_da") => {
            let inhalte = gastgeber.inhalte();
            let Ok(koerper) = anfrage.into_body().collect().await else {
                return meldung(StatusCode::BAD_REQUEST, "unlesbar");
            };
            let Ok(roh) = serde_json::from_slice::<Value>(&koerper.to_bytes()) else {
                return meldung(StatusCode::BAD_REQUEST, "unlesbar");
            };
            let da: Vec<String> = match inhalte {
                Some(inhalte) => roh["hashes"]
                    .as_array()
                    .map(|l| {
                        l.iter()
                            .filter_map(Value::as_str)
                            .filter(|h| inhalte.hat(h))
                            .map(str::to_string)
                            .collect()
                    })
                    .unwrap_or_default(),
                None => Vec::new(),
            };
            return json_antwort(StatusCode::OK, &json!({ "da": da }));
        }
        _ => {}
    }

    let Some(speicher) = gastgeber.speicher() else {
        return meldung(StatusCode::NOT_FOUND, "This device does not sync.");
    };

    let methode = anfrage.method().clone();
    let pfad = anfrage.uri().path().to_string();
    let seit = abfrage(&anfrage, "seit").and_then(|s| s.parse::<i64>().ok());
    let key = abfrage(&anfrage, "key").map(str::to_string);

    // Den Koerper lesen, BEVOR der Speicher gesperrt wird -- ein langsames
    // Netz soll nicht die Oberflaeche dieses Geraets anhalten.
    let koerper = match anfrage.into_body().collect().await {
        Ok(k) => k.to_bytes(),
        Err(_) => return meldung(StatusCode::BAD_REQUEST, "unlesbar"),
    };

    let Ok(speicher) = tokio::time::timeout(WARTEN_AUF_SPEICHER, speicher.lock()).await else {
        return meldung(
            StatusCode::SERVICE_UNAVAILABLE,
            "The other device is syncing itself right now. Try again in a moment.",
        );
    };

    let auskunft = Auskunft::neu(&speicher, basis_fuer(anrufer), gastgeber.mein_name());

    let ergebnis = match (methode, pfad.as_str()) {
        (Method::GET, "/openany/v1/abgleich/delta") => auskunft.delta(seit.unwrap_or(0)).map(|d| {
            json_antwort(
                StatusCode::OK,
                &json!({
                    "cursor": d.cursor,
                    "more": d.more,
                    "entries": d.eintraege.iter().map(Eintrag::ueber_die_leitung).collect::<Vec<_>>(),
                }),
            )
        }),
        (Method::GET, "/openany/v1/abgleich/notiz") => auskunft
            .notiztext(key.as_deref().unwrap_or_default())
            .map(|text| match text {
                Some(t) => json_antwort(StatusCode::OK, &json!({ "content": t })),
                None => meldung(StatusCode::NOT_FOUND, "This note does not exist here."),
            }),
        (Method::GET, "/openany/v1/abgleich/kontaktfoto") => auskunft
            .kontaktfoto(key.as_deref().unwrap_or_default())
            .map(|bytes| match bytes {
                Some(b) => antwort(StatusCode::OK, "application/octet-stream", b),
                None => meldung(StatusCode::NOT_FOUND, "Not a photo."),
            }),
        (Method::POST, "/openany/v1/abgleich/anwenden") => {
            let Ok(roh) = serde_json::from_slice::<Value>(&koerper) else {
                return meldung(StatusCode::BAD_REQUEST, "unlesbar");
            };
            let eintraege: Vec<Eintrag> = roh["entries"]
                .as_array()
                .map(|l| l.iter().filter_map(Eintrag::aus_der_leitung).collect())
                .unwrap_or_default();
            auskunft.anwenden(&eintraege).map(|r| {
                let geaendert = r.iter().any(|e| {
                    !matches!(e.action.as_str(), "unchanged" | "gone" | "ignored" | "skipped")
                });
                if geaendert {
                    gastgeber.angenommen();
                }
                json_antwort(StatusCode::OK, &json!({ "results": r }))
            })
        }
        (Method::POST, "/openany/v1/abgleich/marken") => {
            let Ok(m) = serde_json::from_slice::<Marken>(&koerper) else {
                return meldung(StatusCode::BAD_REQUEST, "unlesbar");
            };
            auskunft
                .marken_gemeldet(m.eigene, m.fremde)
                .map(|()| json_antwort(StatusCode::OK, &json!({})))
        }
        _ => return meldung(StatusCode::NOT_FOUND, "Unbekannt."),
    };

    ergebnis.unwrap_or_else(|e| meldung(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))
}

/// Ein gepaartes Geraet als [`Gegenstelle`] fuer den Laeufer.
pub struct NahGegenstelle {
    basis: String,
    name: String,
    wurzel: String,
    http: reqwest::Client,
}

impl NahGegenstelle {
    /// * `adresse` -- wo das Geraet gerade ist (aus der Suche).
    /// * `fingerabdruck` -- wer es sein MUSS; sonst scheitert der Handschlag.
    /// * `name` -- wie es sich beim Paaren genannt hat.
    pub fn neu(
        ident: &Identitaet,
        adresse: &str,
        port: u16,
        fingerabdruck: &str,
        name: &str,
    ) -> Result<Self, String> {
        let tls = anruf_konfiguration(&ident.zertifikat_pem, &ident.schluessel_pem, fingerabdruck)
            .map_err(|e| e.to_string())?;
        let http = reqwest::Client::builder()
            .use_preconfigured_tls(tls)
            // Eine Seite mit 500 Eintraegen oder ein Stapel Notiztexte ueber
            // ein wackliges WLAN -- laenger als eine Paarungsfrage.
            .timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(8))
            .build()
            .map_err(|e| e.to_string())?;

        Ok(Self {
            basis: basis_fuer(fingerabdruck),
            name: name.to_string(),
            wurzel: crate::anruf::wurzel(adresse, port),
            http,
        })
    }

    fn adresse(&self, pfad: &str) -> String {
        format!("{}/openany/v1/abgleich/{pfad}", self.wurzel)
    }

    async fn pruefen(antwort: reqwest::Response) -> Result<reqwest::Response, OpenanyError> {
        let status = antwort.status();
        if status.is_success() {
            return Ok(antwort);
        }
        let meldung = antwort
            .json::<Value>()
            .await
            .ok()
            .and_then(|v| v["message"].as_str().map(str::to_string))
            .unwrap_or_else(|| "The other device refused.".into());
        Err(OpenanyError::Abgelehnt { status, meldung })
    }

    async fn json(&self, anfrage: reqwest::RequestBuilder) -> Result<Value, OpenanyError> {
        Self::pruefen(anfrage.send().await?)
            .await?
            .json::<Value>()
            .await
            .map_err(|e| OpenanyError::Unlesbar(e.to_string()))
    }
}

#[async_trait]
impl Gegenstelle for NahGegenstelle {
    fn basis(&self) -> &str {
        &self.basis
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn fotos_mitschicken(&self) -> bool {
        true
    }

    fn traegt_dateien(&self) -> bool {
        true
    }

    async fn inhalt(&self, abdruck: &str, von: u64, laenge: u64) -> Result<Vec<u8>, OpenanyError> {
        let antwort = self
            .http
            .get(self.adresse("inhalt"))
            .query(&[
                ("hash", abdruck.to_string()),
                ("von", von.to_string()),
                ("laenge", laenge.to_string()),
            ])
            .send()
            .await?;
        Ok(Self::pruefen(antwort).await?.bytes().await?.to_vec())
    }

    async fn inhalte_da(&self, abdruecke: &[String]) -> Result<Vec<String>, OpenanyError> {
        let roh = self
            .json(
                self.http
                    .post(self.adresse("inhalte_da"))
                    .json(&json!({ "hashes": abdruecke })),
            )
            .await?;
        Ok(roh["da"]
            .as_array()
            .map(|l| {
                l.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default())
    }

    async fn delta(&self, seit: i64) -> Result<Delta, OpenanyError> {
        let roh = self
            .json(
                self.http
                    .get(self.adresse("delta"))
                    .query(&[("seit", seit)]),
            )
            .await?;
        Delta::lesen(&roh).map_err(|e| OpenanyError::Unlesbar(e.to_string()))
    }

    async fn notiztext(&self, zk_id: &str) -> Result<String, OpenanyError> {
        let roh = self
            .json(
                self.http
                    .get(self.adresse("notiz"))
                    .query(&[("key", zk_id)]),
            )
            .await?;
        roh["content"]
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| OpenanyError::Unlesbar("Note without text.".into()))
    }

    async fn anwenden(&self, eintraege: &[Eintrag]) -> Result<Vec<Ergebnis>, OpenanyError> {
        let roh = self
            .json(self.http.post(self.adresse("anwenden")).json(&json!({
                "entries": eintraege.iter().map(Eintrag::ueber_die_leitung).collect::<Vec<_>>(),
            })))
            .await?;
        serde_json::from_value(roh["results"].clone())
            .map_err(|e| OpenanyError::Unlesbar(e.to_string()))
    }

    async fn kontaktfoto(&self, uuid: &str) -> Result<Vec<u8>, OpenanyError> {
        let antwort = self
            .http
            .get(self.adresse("kontaktfoto"))
            .query(&[("key", uuid)])
            .send()
            .await?;
        Ok(Self::pruefen(antwort).await?.bytes().await?.to_vec())
    }

    async fn marken_melden(&self, eigene: i64, fremde: i64) -> Result<(), OpenanyError> {
        self.json(
            self.http
                .post(self.adresse("marken"))
                .json(&json!({ "eigene": eigene, "fremde": fremde })),
        )
        .await
        .map(|_| ())
    }
}

/// Fuer den Gastgeber: der Speicher so, wie die Anwendung ihn haelt.
pub type GemeinsamerSpeicher = Arc<tokio::sync::Mutex<Speicher>>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dienst::Dienst;
    use openany_store::{Notiz, Protokoll};
    use openany_sync::Laeufer;
    use std::sync::Mutex;

    struct Geraet {
        name: String,
        ident: Identitaet,
        speicher: GemeinsamerSpeicher,
        inhalte: Arc<openany_store::Inhalte>,
        gepaart: Mutex<Vec<String>>,
        _ordner: tempfile::TempDir,
    }

    impl Gastgeber for Geraet {
        fn mein_name(&self) -> String {
            self.name.clone()
        }
        fn qr_geheimnis(&self) -> Option<String> {
            None
        }
        fn gepaart(&self, fp: &str, _name: &str) {
            self.gepaart.lock().unwrap().push(fp.into());
        }
        fn ist_gepaart(&self, fp: &str) -> bool {
            self.gepaart.lock().unwrap().iter().any(|f| f == fp)
        }
        fn speicher(&self) -> Option<GemeinsamerSpeicher> {
            Some(self.speicher.clone())
        }
        fn inhalte(&self) -> Option<Arc<openany_store::Inhalte>> {
            Some(self.inhalte.clone())
        }
    }

    fn geraet(name: &str) -> Arc<Geraet> {
        let ordner = tempfile::tempdir().unwrap();
        Arc::new(Geraet {
            name: name.into(),
            ident: Identitaet::laden_oder_erzeugen(&ordner.path().join("i.json")).unwrap(),
            speicher: Arc::new(tokio::sync::Mutex::new(
                Speicher::im_arbeitsspeicher().unwrap(),
            )),
            inhalte: Arc::new(
                openany_store::Inhalte::oeffnen(ordner.path().join("inhalte")).unwrap(),
            ),
            gepaart: Mutex::new(vec![]),
            _ordner: ordner,
        })
    }

    async fn dienst(g: &Arc<Geraet>) -> Dienst {
        Dienst::starten(&g.ident, 0, Default::default(), g.clone())
            .await
            .unwrap()
    }

    async fn abgleichen(ich: &Geraet, dort: &Geraet, port: u16) -> openany_sync::Bericht {
        let gegenstelle = NahGegenstelle::neu(
            &ich.ident,
            "127.0.0.1",
            port,
            &dort.ident.fingerabdruck,
            &dort.name,
        )
        .unwrap();
        let speicher = ich.speicher.lock().await;
        Laeufer::neu(&speicher, &gegenstelle, ich.name.clone())
            .lauf()
            .await
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn zwei_gepaarte_geraete_gleichen_ueber_echtes_tls_ab() {
        let tablet = geraet("Tablet");
        let telefon = geraet("Telefon");
        tablet.gepaart(&telefon.ident.fingerabdruck, "Telefon");
        telefon.gepaart(&tablet.ident.fingerabdruck, "Tablet");
        let d_tablet = dienst(&tablet).await;
        let d_telefon = dienst(&telefon).await;

        telefon
            .speicher
            .lock()
            .await
            .notiz_schreiben(
                &Notiz {
                    zk_id: "20260915120000".into(),
                    titel: "Einkauf".into(),
                    inhalt: "Milch\nBrot".into(),
                    ..Default::default()
                },
                Protokoll::Merken,
            )
            .unwrap();
        tablet
            .speicher
            .lock()
            .await
            .kontaktfoto_setzen("c-1", b"bild")
            .ok();

        let bericht = abgleichen(&tablet, &telefon, d_telefon.port).await;
        assert!(bericht.durchgelaufen(), "{:?}", bericht.fehler);
        assert_eq!(bericht.gezogen, 1);
        assert_eq!(
            tablet
                .speicher
                .lock()
                .await
                .notiz("20260915120000")
                .unwrap()
                .unwrap()
                .inhalt,
            "Milch\nBrot"
        );

        // Der zweite Lauf -- von beiden Seiten -- meldet nichts mehr.
        let zweiter = abgleichen(&tablet, &telefon, d_telefon.port).await;
        assert_eq!((zweiter.gezogen, zweiter.geschoben), (0, 0));
        let andersrum = abgleichen(&telefon, &tablet, d_tablet.port).await;
        assert!(andersrum.durchgelaufen(), "{:?}", andersrum.fehler);
        assert_eq!((andersrum.gezogen, andersrum.geschoben), (0, 0));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn ein_nicht_gepaartes_geraet_bekommt_nichts() {
        let tablet = geraet("Tablet");
        let fremd = geraet("Fremd");
        let d_tablet = dienst(&tablet).await;

        let bericht = abgleichen(&fremd, &tablet, d_tablet.port).await;
        assert!(!bericht.durchgelaufen());
        assert!(
            bericht.fehler[0].contains("not (or no longer) paired"),
            "{:?}",
            bericht.fehler
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn wer_selbst_gerade_abgleicht_sagt_beschaeftigt_statt_zu_haengen() {
        let tablet = geraet("Tablet");
        let telefon = geraet("Telefon");
        tablet.gepaart(&telefon.ident.fingerabdruck, "Telefon");
        telefon.gepaart(&tablet.ident.fingerabdruck, "Tablet");
        let d_telefon = dienst(&telefon).await;

        // Das Telefon haelt seinen Speicher, als liefe dort gerade ein Lauf.
        let _sperre = telefon.speicher.lock().await;

        let bericht = abgleichen(&tablet, &telefon, d_telefon.port).await;
        assert!(!bericht.durchgelaufen());
        assert!(
            bericht.fehler[0].contains("is syncing itself"),
            "{:?}",
            bericht.fehler
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn eine_datei_reist_mit_ihrem_inhalt_ueber_echtes_tls() {
        let tablet = geraet("Tablet");
        let telefon = geraet("Telefon");
        tablet.gepaart(&telefon.ident.fingerabdruck, "Telefon");
        telefon.gepaart(&tablet.ident.fingerabdruck, "Tablet");
        let d_telefon = dienst(&telefon).await;

        let bytes: Vec<u8> = (0..5_000_000u32).map(|i| (i % 253) as u8).collect();
        let mut ladung = telefon.inhalte.ladung().unwrap();
        ladung.schreiben(&bytes).unwrap();
        let (abdruck, groesse) = telefon.inhalte.ablegen(ladung).unwrap();
        telefon
            .speicher
            .lock()
            .await
            .datei_schreiben(
                &openany_store::Datei {
                    uuid: "d-1".into(),
                    zone: "files".into(),
                    name: "gross.bin".into(),
                    groesse,
                    mime: "application/octet-stream".into(),
                    abdruck: Some(abdruck.clone()),
                    ..Default::default()
                },
                Protokoll::Merken,
            )
            .unwrap();

        let bericht = abgleichen(&tablet, &telefon, d_telefon.port).await;
        assert!(bericht.durchgelaufen(), "{:?}", bericht.fehler);
        assert!(tablet.speicher.lock().await.datei("d-1").unwrap().is_some());

        let gegenstelle = NahGegenstelle::neu(
            &tablet.ident,
            "127.0.0.1",
            d_telefon.port,
            &telefon.ident.fingerabdruck,
            "Telefon",
        )
        .unwrap();
        let geholt = openany_sync::fehlende_inhalte_holen(
            &tablet.inhalte,
            &gegenstelle,
            &[(abdruck.clone(), groesse)],
            || u64::MAX / 4,
            u64::MAX / 4,
        )
        .await;
        assert_eq!(geholt.geholt, 1, "{geholt:?}");
        assert!(tablet.inhalte.hat(&abdruck));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn ein_fremdes_geraet_bekommt_keinen_inhalt() {
        let telefon = geraet("Telefon");
        let fremd = geraet("Fremd");
        let d_telefon = dienst(&telefon).await;
        let mut ladung = telefon.inhalte.ladung().unwrap();
        ladung.schreiben(b"privat").unwrap();
        let (abdruck, _) = telefon.inhalte.ablegen(ladung).unwrap();

        let gegenstelle = NahGegenstelle::neu(
            &fremd.ident,
            "127.0.0.1",
            d_telefon.port,
            &telefon.ident.fingerabdruck,
            "Telefon",
        )
        .unwrap();
        let fehler = gegenstelle.inhalt(&abdruck, 0, 100).await.unwrap_err();
        assert!(
            fehler.to_string().contains("not (or no longer) paired"),
            "{fehler}"
        );
    }
}
