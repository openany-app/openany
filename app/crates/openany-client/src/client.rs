//! Der duenne Teil: HTTP gegen openany.
//!
//! Vier Wege, mehr braucht der Abgleich nicht:
//!
//! | | |
//! |---|---|
//! | `GET /api/delta?since=` | was hat sich drueben geaendert |
//! | `GET /api/sync/content` | die Bytes zu einem Eintrag |
//! | `POST /api/sync/apply` | was sich hier geaendert hat |
//! | `POST /api/geraete/schluessel` | einmalig: aus einem Ticket wird der Schluessel |
//!
//! Der letzte faellt aus der Reihe: Er ist der einzige **ohne** Schluessel --
//! das Programm holt ihn ja gerade ab.

use crate::vertrag::{Delta, Eintrag, Ergebnis};
use reqwest::{Client as Http, StatusCode};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OpenanyError {
    #[error("Network error: {0}")]
    Transport(#[from] reqwest::Error),

    /// **Der einzige Fehler, auf den anders zu reagieren ist als mit "spaeter
    /// nochmal".** Ein widerrufener Geraeteschluessel wird durch Wiederholen
    /// nicht wieder gueltig -- das Geraet muss neu gekoppelt werden. Wer das
    /// nicht unterscheidet, baut ein Programm, das nach einem Widerruf still
    /// alle fuenf Minuten weiterklopft und dem Menschen nie sagt, warum
    /// nichts mehr ankommt.
    #[error("This device key is no longer valid.")]
    SchluesselUngueltig,

    /// 403: Der Schluessel lebt, darf aber das hier nicht. In der Praxis
    /// heisst das eine fehlende Ability -- ein Schluessel ohne `sync:write`
    /// etwa. Auch das hilft kein Wiederholen, aber es ist ein anderer Fehler:
    /// Der Schluessel muss nicht ersetzt, sondern richtig ausgestellt werden.
    #[error("This key is not allowed to do that: {0}")]
    NichtErlaubt(String),

    /// Status 429 -- der Server nennt die Wartezeit im Kopf `Retry-After`.
    /// Wer sie ignoriert, wird laenger gebremst als noetig.
    #[error("Too many requests – try again in {sekunden}s.")]
    Gebremst { sekunden: u64 },

    /// Status 413 -- der Speicher drueben reicht nicht.
    ///
    /// **Ein eigener Fehler, weil er ein eigenes Verhalten verlangt.** Platz
    /// wird durch Warten nicht mehr: Ein Abgleich, der es nachts alle zehn
    /// Minuten erneut versucht, fuellt bis zum Morgen nur das Protokoll. Der
    /// Mensch muss es erfahren, und zwar einmal und deutlich.
    #[error("No space left at openany.de: {0}")]
    KeinPlatz(String),

    #[error("openany refused ({status}): {meldung}")]
    Abgelehnt { status: StatusCode, meldung: String },

    #[error("The response could not be read: {0}")]
    Unlesbar(String),
}

/// Die Gegenstelle: eine openany-Instanz.
///
/// **Eine je Instanz und nicht eine im Programm.** Heute ist es genau
/// eine (openany.de); ein Programm, das "wahlweise gegen die eigene oder gegen gar
/// keine" verspricht, kann eines Tages mit zweien sprechen. Deshalb traegt
/// diese Struktur ihre Basis-Adresse mit sich, und alles, was der Abgleich
/// sich merkt, haengt an genau dieser Adresse.
/// `Clone`, weil ein Geraet mehrere Stroeme derselben Instanz abgleicht:
/// den persoenlichen und je einen fuer jedes Projekt. Alle sprechen mit
/// derselben Adresse und demselben Schluessel; `reqwest::Client` teilt seinen
/// Verbindungspool dabei mit, statt je Projekt einen neuen aufzubauen.
#[derive(Clone)]
pub struct OpenanyClient {
    http: Http,
    basis: String,
    schluessel: String,
}

impl OpenanyClient {
    /// `basis` ist die Wurzel der Instanz, etwa `https://openany.de`.
    pub fn neu(
        basis: impl Into<String>,
        schluessel: impl Into<String>,
    ) -> Result<Self, OpenanyError> {
        Self::neu_mit_ca(basis, schluessel, None)
    }

    /// Mit einer zusaetzlichen Wurzel-CA -- fuer die Entwicklung hinter
    /// Caddys eigener CA und fuer Haeuser, die ihren Verkehr ueber eine
    /// eigene fuehren.
    ///
    /// **Kein Abschalten der Pruefung.** Die Kette wird weiter geprueft, nur
    /// gegen eine Wurzel mehr. Ein `danger_accept_invalid_certs` waere die
    /// Gewohnheit, die irgendwann in einem ausgelieferten Programm landet --
    /// und dann traegt ein Geraeteschluessel, der auf alle Notizen reicht,
    /// ueber eine Leitung, der niemand mehr ansieht, wer mithoert.
    pub fn neu_mit_ca(
        basis: impl Into<String>,
        schluessel: impl Into<String>,
        ca_pem: Option<&[u8]>,
    ) -> Result<Self, OpenanyError> {
        Ok(Self {
            http: Self::bauen(ca_pem)?,
            basis: basis.into().trim_end_matches('/').to_string(),
            schluessel: schluessel.into(),
        })
    }

    fn bauen(ca_pem: Option<&[u8]>) -> Result<Http, OpenanyError> {
        // 60 Sekunden und nicht 30 wie bei anyid: Dort geht es um kurze
        // Kopplungsantworten, hier kann eine Delta-Seite 500 Eintraege
        // beschreiben und eine Inhaltsanfrage 16 MiB liefern.
        let mut bauer = Http::builder().timeout(Duration::from_secs(60));

        if let Some(pem) = ca_pem {
            bauer = bauer.add_root_certificate(reqwest::Certificate::from_pem(pem)?);
        }

        Ok(bauer.build()?)
    }

    fn url(&self, pfad: &str) -> String {
        format!("{}/api{}", self.basis, pfad)
    }

    pub fn basis(&self) -> &str {
        &self.basis
    }

    /// Was hat sich seit `seit` drueben geaendert?
    pub async fn delta(&self, seit: i64) -> Result<Delta, OpenanyError> {
        let antwort = self
            .http
            .get(self.url("/delta"))
            .bearer_auth(&self.schluessel)
            .header("Accept", "application/json")
            .query(&[("since", seit.to_string())])
            .send()
            .await?;

        let roh: serde_json::Value = pruefen(antwort).await?.json().await?;

        Delta::lesen(&roh).map_err(|e| OpenanyError::Unlesbar(e.to_string()))
    }

    /// Das Delta EINES PROJEKTS -- der Strom, der dem Projekt gehoert.
    ///
    /// Derselbe Vertrag wie beim persoenlichen (`type`, `key`, `action`,
    /// `cursor`, `more`), nur unter einer anderen Adresse. Deshalb steht hier
    /// dieselbe Antwort-Lesung und kein zweites Format.
    pub async fn projekt_delta(&self, uuid: &str, seit: i64) -> Result<Delta, OpenanyError> {
        let antwort = self
            .http
            .get(self.url(&format!("/projects/{uuid}/delta")))
            .bearer_auth(&self.schluessel)
            .header("Accept", "application/json")
            .query(&[("since", seit.to_string())])
            .send()
            .await?;

        let roh: serde_json::Value = pruefen(antwort).await?.json().await?;

        Delta::lesen(&roh).map_err(|e| OpenanyError::Unlesbar(e.to_string()))
    }

    /// Eintraege in den Strom eines Projekts anwenden.
    pub async fn projekt_anwenden(
        &self,
        uuid: &str,
        eintraege: &[Eintrag],
    ) -> Result<Vec<Ergebnis>, OpenanyError> {
        let fracht: Vec<_> = eintraege.iter().map(Eintrag::ueber_die_leitung).collect();

        let antwort = self
            .http
            .post(self.url(&format!("/projects/{uuid}/sync/apply")))
            .bearer_auth(&self.schluessel)
            .header("Accept", "application/json")
            .json(&serde_json::json!({ "entries": fracht }))
            .send()
            .await?;

        let koerper: serde_json::Value = pruefen(antwort).await?.json().await?;

        serde_json::from_value(koerper.get("results").cloned().unwrap_or_default())
            .map_err(|e| OpenanyError::Unlesbar(e.to_string()))
    }

    /// Die Bytes zu einem Eintrag.
    ///
    /// **Am Stueck und nicht in Stuecken.** Der Server kann Bereiche
    /// (`Range:`), aber die Wiederaufnahme sitzt eine Ebene hoeher: Die Marke
    /// steht noch auf der alten Seite, also kommt der Eintrag beim naechsten
    /// Lauf wieder. Die Bereiche hier auch noch auszunutzen waere Buchfuehrung
    /// fuer einen Gewinn, den erst sehr grosse Dateien spuerbar machen --
    /// dieselbe Abwaegung wie beim frueheren Laeufer des Sticks.
    pub async fn inhalt(&self, art: &str, key: &str) -> Result<Vec<u8>, OpenanyError> {
        let antwort = self
            .http
            .get(self.url("/sync/content"))
            .bearer_auth(&self.schluessel)
            .query(&[("type", art), ("key", key)])
            .send()
            .await?;

        // 404 heisst "gibt es hier nicht (mehr)" und ist kein Fehler des
        // Laufs: Zwischen dem Lesen der Seite und dem Holen der Bytes kann
        // drueben geloescht worden sein. Der Aufrufer uebergeht den Eintrag.
        if antwort.status() == StatusCode::NOT_FOUND {
            return Err(OpenanyError::Abgelehnt {
                status: StatusCode::NOT_FOUND,
                meldung: "This content does not exist there.".into(),
            });
        }

        Ok(pruefen(antwort).await?.bytes().await?.to_vec())
    }

    /// Ein Stueck der Bytes zu einem Eintrag.
    ///
    /// **Anders als [`inhalt`](Self::inhalt), und seit dem 16.09.2026.** Dort
    /// steht, warum dort am Stueck geholt wird: Die Wiederaufnahme sitzt eine
    /// Ebene hoeher, der Eintrag kommt beim naechsten Lauf wieder. Fuer eine
    /// App auf einem Telefon stimmt das nicht mehr. Ein Video von 800 MB ueber
    /// Mobilfunk hat zwischen zwei Laeufen keine Gnadenfrist -- es faengt
    /// jedes Mal von vorn an und wird nie fertig. Deshalb hier `Range:`, und
    /// die Wiederaufnahme sitzt da, wo die Bytes liegen.
    ///
    /// Der Server liefert hoechstens 16 MiB je Antwort (`SyncContentController`),
    /// auch wenn mehr verlangt wird. Der Aufrufer muss also ohnehin so lange
    /// fragen, bis er hat, was er wollte -- was er beim Holen in Stuecken
    /// sowieso tut.
    pub async fn inhalt_bereich(
        &self,
        art: &str,
        key: &str,
        von: u64,
        laenge: u64,
    ) -> Result<Vec<u8>, OpenanyError> {
        let bis = von + laenge.max(1) - 1;

        let antwort = self
            .http
            .get(self.url("/sync/content"))
            .bearer_auth(&self.schluessel)
            .query(&[("type", art), ("key", key)])
            .header("Range", format!("bytes={von}-{bis}"))
            .send()
            .await?;

        if antwort.status() == StatusCode::NOT_FOUND {
            return Err(OpenanyError::Abgelehnt {
                status: StatusCode::NOT_FOUND,
                meldung: "This content does not exist there.".into(),
            });
        }

        // 416 heisst: hinter dem Ende. Fuer den Aufrufer ist das keine
        // Stoerung, sondern das Ende der Datei -- er hat alles.
        if antwort.status() == StatusCode::RANGE_NOT_SATISFIABLE {
            return Ok(Vec::new());
        }

        Ok(pruefen(antwort).await?.bytes().await?.to_vec())
    }

    /// Den Text einer Notiz -- derselbe Weg, nur schon als Zeichenkette.
    pub async fn notiztext(&self, zk_id: &str) -> Result<String, OpenanyError> {
        let bytes = self.inhalt("note", zk_id).await?;

        String::from_utf8(bytes).map_err(|e| OpenanyError::Unlesbar(e.to_string()))
    }

    /// Eigene Aenderungen hinueberschicken -- gebuendelt.
    ///
    /// **Ein Stapel und nicht je Eintrag eine Runde.** Ein Abgleich bringt
    /// selten eine Aenderung mit, sondern zwanzig; je eine Netzwerkrunde
    /// hiesse auf einer Mobilfunkleitung zwanzig Mal Wartezeit fuer zwanzig
    /// Mal ein paar hundert Bytes. Der Server nimmt hoechstens 500 auf einmal.
    pub async fn anwenden(&self, eintraege: &[Eintrag]) -> Result<Vec<Ergebnis>, OpenanyError> {
        let fracht: Vec<_> = eintraege.iter().map(Eintrag::ueber_die_leitung).collect();

        let antwort = self
            .http
            .post(self.url("/sync/apply"))
            .bearer_auth(&self.schluessel)
            .header("Accept", "application/json")
            .json(&serde_json::json!({ "entries": fracht }))
            .send()
            .await?;

        let koerper: serde_json::Value = pruefen(antwort).await?.json().await?;

        serde_json::from_value(koerper.get("results").cloned().unwrap_or_default())
            .map_err(|e| OpenanyError::Unlesbar(e.to_string()))
    }

    // --- Bytes hinueberbringen ------------------------------------------

    /// Eine Uebertragung anmelden -- oder die angefangene fortsetzen.
    ///
    /// Der Server erkennt dieselbe Datei am Abdruck wieder und antwortet mit
    /// dem, was er schon hat. Wir muessen uns also nichts merken, auch nicht
    /// ueber einen Neustart des Programms hinweg.
    pub async fn upload_beginnen(
        &self,
        ziel: &Hochzuladen<'_>,
    ) -> Result<Uebertragung, OpenanyError> {
        let antwort = self
            .http
            .post(self.url("/sync/uploads"))
            .bearer_auth(&self.schluessel)
            .header("Accept", "application/json")
            .json(&ziel.json())
            .send()
            .await?;

        let koerper: serde_json::Value = pruefen(antwort).await?.json().await?;

        Ok(Uebertragung {
            token: koerper
                .get("token")
                .and_then(|w| w.as_str())
                .ok_or_else(|| OpenanyError::Unlesbar("No ID received.".into()))?
                .to_string(),
            angekommen: koerper
                .get("received")
                .and_then(|w| w.as_u64())
                .unwrap_or(0),
        })
    }

    /// Ein Stueck ab dem angegebenen Byte. Antwortet mit dem neuen Stand.
    ///
    /// **Der Versatz zaehlt, nicht die Stuecknummer**, und was zurueckkommt,
    /// gilt: Ein wiederholtes Stueck bestaetigt der Server, ohne es doppelt
    /// zu zaehlen. Wer stattdessen selbst weiterzaehlt, laeuft ihm davon und
    /// laesst eine Luecke, die erst ganz am Ende auffaellt.
    pub async fn stueck_schicken(
        &self,
        token: &str,
        versatz: u64,
        bytes: Vec<u8>,
    ) -> Result<u64, OpenanyError> {
        let abdruck = {
            use sha2::{Digest, Sha256};

            let mut h = Sha256::new();
            h.update(&bytes);
            format!("{:x}", h.finalize())
        };

        let antwort = self
            .http
            .patch(self.url(&format!("/sync/uploads/{token}")))
            .bearer_auth(&self.schluessel)
            .header("Accept", "application/json")
            .header("Content-Type", "application/octet-stream")
            .header("X-Chunk-Sha256", abdruck)
            .query(&[("offset", versatz.to_string())])
            .body(bytes)
            .send()
            .await?;

        // 409 ist keine Absage, sondern eine Einladung: "mach dort weiter".
        // Der Aufrufer bekommt die genannte Stelle und springt hin.
        if antwort.status() == StatusCode::CONFLICT {
            let koerper: serde_json::Value = antwort.json().await.unwrap_or_default();

            return koerper
                .get("received")
                .and_then(|w| w.as_u64())
                .ok_or_else(|| OpenanyError::Unlesbar("No state in the conflict.".into()));
        }

        let koerper: serde_json::Value = pruefen(antwort).await?.json().await?;

        koerper
            .get("received")
            .and_then(|w| w.as_u64())
            .ok_or_else(|| OpenanyError::Unlesbar("No state in the response.".into()))
    }

    /// Abschliessen: Der Server prueft den Abdruck und macht daraus eine
    /// Datei oder ein Bild.
    pub async fn upload_abschliessen(&self, token: &str) -> Result<Angekommen, OpenanyError> {
        let antwort = self
            .http
            .post(self.url(&format!("/sync/uploads/{token}/complete")))
            .bearer_auth(&self.schluessel)
            .header("Accept", "application/json")
            .send()
            .await?;

        let koerper: serde_json::Value = pruefen(antwort).await?.json().await?;

        Ok(Angekommen {
            art: koerper
                .get("type")
                .and_then(|w| w.as_str())
                .unwrap_or_default()
                .to_string(),
            schluessel: koerper
                .get("key")
                .and_then(|w| w.as_str())
                .unwrap_or_default()
                .to_string(),
            konflikt: koerper
                .get("conflict")
                .and_then(|w| w.as_bool())
                .unwrap_or(false),
            zone: koerper
                .get("zone")
                .and_then(|w| w.as_str())
                .map(str::to_string),
        })
    }

    /// Eine angefangene Uebertragung aufgeben -- Teilstueck weg, Platz zurueck.
    pub async fn upload_aufgeben(&self, token: &str) -> Result<(), OpenanyError> {
        let antwort = self
            .http
            .delete(self.url(&format!("/sync/uploads/{token}")))
            .bearer_auth(&self.schluessel)
            .send()
            .await?;

        pruefen(antwort).await.map(|_| ())
    }

    /* ── Freigaben in Projekten (Weg 1, 30.09.2026) ───────────────────── */

    /// Die Freigaben eines Projekts (unter seiner uuid): je Freigabe `type`,
    /// `uuid`, `own`, `name`, `permission`, `shared_by` -- roh, wie
    /// `ProjectShareController::index` sie liefert.
    pub async fn projekt_freigaben(&self, uuid: &str) -> Result<serde_json::Value, OpenanyError> {
        self.projekt_json(&format!("/projects/{uuid}/shares"), &[])
            .await
    }

    /// Eine lesende Anfrage unter `/projects/…` (Ordner, Album, Mappe einer
    /// Freigabe). NUR DORT: Der Pfad kommt aus Antworten des Servers und
    /// wird hier auf den Projektbereich begrenzt.
    pub async fn projekt_json(
        &self,
        pfad: &str,
        abfrage: &[(&str, String)],
    ) -> Result<serde_json::Value, OpenanyError> {
        let antwort = self
            .http
            .get(self.url(Self::projektpfad(pfad)?))
            .bearer_auth(&self.schluessel)
            .header("Accept", "application/json")
            .query(abfrage)
            .send()
            .await?;
        Ok(pruefen(antwort).await?.json().await?)
    }

    /// Die Bytes einer Datei oder eines Bilds aus einer Freigabe.
    pub async fn projekt_bytes(&self, pfad: &str) -> Result<Vec<u8>, OpenanyError> {
        let antwort = self
            .http
            .get(self.url(Self::projektpfad(pfad)?))
            .bearer_auth(&self.schluessel)
            .send()
            .await?;
        Ok(pruefen(antwort).await?.bytes().await?.to_vec())
    }

    /// `/api/projects/…` oder `/projects/…` -> `/projects/…`; alles andere
    /// wird abgelehnt, ebenso `..`.
    fn projektpfad(pfad: &str) -> Result<&str, OpenanyError> {
        let p = pfad.strip_prefix("/api").unwrap_or(pfad);
        if p.starts_with("/projects/") && !p.contains("..") {
            Ok(p)
        } else {
            Err(OpenanyError::Abgelehnt {
                status: StatusCode::BAD_REQUEST,
                meldung: format!("Not a project path: {pfad}"),
            })
        }
    }

    /* ── Das Postfach ──────────────────────────────────────────────────── */

    /// Eine Seite des internen Verlaufs (neueste zuerst).
    ///
    /// **Roh und ungelesen zurueck.** Was drueben in einer Zeile steht, ist
    /// die Entscheidung des Servers (`MessageResource`); eine zweite
    /// Vorstellung davon hier haette nur zwei Stellen zum Nachziehen. Der
    /// Befehl, der sie anzeigt, liest heraus, was die Ansicht braucht.
    pub async fn nachrichten(&self, seite: usize) -> Result<serde_json::Value, OpenanyError> {
        self.nachrichten_gefiltert(seite, &[]).await
    }

    /// Eine Seite, eingegrenzt: `filter` sind die Parameter, wie der Server
    /// sie nimmt (`wege[]`, `ungelesen`, `anhang`, `q`, `mit[]`).
    pub async fn nachrichten_gefiltert(
        &self,
        seite: usize,
        filter: &[(String, String)],
    ) -> Result<serde_json::Value, OpenanyError> {
        let antwort = self
            .http
            .get(self.url("/messages"))
            .bearer_auth(&self.schluessel)
            .header("Accept", "application/json")
            .query(&[("page", seite.to_string())])
            .query(filter)
            .send()
            .await?;

        Ok(pruefen(antwort).await?.json().await?)
    }

    /// „Nach Kontakt": je Gegenüber die jüngste Nachricht, dieselben Filter.
    pub async fn unterhaltungen(
        &self,
        filter: &[(String, String)],
    ) -> Result<serde_json::Value, OpenanyError> {
        let antwort = self
            .http
            .get(self.url("/messages/unterhaltungen"))
            .bearer_auth(&self.schluessel)
            .header("Accept", "application/json")
            .query(filter)
            .send()
            .await?;

        Ok(pruefen(antwort).await?.json().await?)
    }

    /// Intern an einen openany-Namen schreiben.
    ///
    /// **Kein `transport`.** Ohne Angabe nimmt der Server den internen Weg --
    /// und ueber Matrix sendet dieses Geraet ohnehin selbst, mit eigenen
    /// Schluesseln. Ein Geraeteschluessel wird drueben abgewiesen, wenn er es
    /// doch versucht.
    pub async fn nachricht_senden(
        &self,
        empfaenger: &str,
        text: &str,
    ) -> Result<serde_json::Value, OpenanyError> {
        let antwort = self
            .http
            .post(self.url("/messages"))
            .bearer_auth(&self.schluessel)
            .header("Accept", "application/json")
            .json(&serde_json::json!({ "username": empfaenger, "body": text }))
            .send()
            .await?;

        Ok(pruefen(antwort).await?.json().await?)
    }

    /// Eine interne Nachricht als gelesen vermerken.
    pub async fn nachricht_gelesen(&self, id: i64) -> Result<(), OpenanyError> {
        let antwort = self
            .http
            .post(self.url(&format!("/messages/{id}/read")))
            .bearer_auth(&self.schluessel)
            .header("Accept", "application/json")
            .send()
            .await?;

        pruefen(antwort).await.map(|_| ())
    }

    /* ── Das Standbild eines Videos (Weg C) ────────────────────────────── */

    /// Das Standbild eines Videos zum Server schicken. Der Server kann aus
    /// Videos keine Bilder ziehen, Firefox auf Android auch nicht -- das
    /// Programm kann es. Gibt den `thumb_hash` zurück, unter dem der Server
    /// das Standbild ablegt (er kodiert neu, der Abdruck ist also seiner).
    ///
    /// Scheitert mit 404, solange das Video drüben noch nicht angekommen ist.
    pub async fn media_standbild(
        &self,
        uuid: &str,
        jpeg: Vec<u8>,
        dauer: Option<f64>,
    ) -> Result<Option<String>, OpenanyError> {
        #[derive(serde::Deserialize)]
        struct Antwort {
            thumb_hash: Option<String>,
        }

        let mut anfrage = self
            .http
            .post(self.url(&format!("/sync/media/{uuid}/vorschau")))
            .bearer_auth(&self.schluessel)
            .header("Accept", "application/json")
            .header("Content-Type", "image/jpeg");
        if let Some(d) = dauer {
            anfrage = anfrage.query(&[("dauer", format!("{d:.1}"))]);
        }
        let antwort = anfrage.body(jpeg).send().await?;

        let a: Antwort = pruefen(antwort).await?.json().await?;
        Ok(a.thumb_hash)
    }

    /* ── Der Weckruf (Phase 6) ─────────────────────────────────────────── */

    /// Dieses Geraet zum Wecken anmelden: `(basis, thema)` fuer die Leitung
    /// zu ntfy. Wiederholbar -- derselbe Schluessel bekommt dasselbe Thema.
    pub async fn weckruf_anmelden(&self) -> Result<(String, String), OpenanyError> {
        #[derive(serde::Deserialize)]
        struct Antwort {
            basis: String,
            thema: String,
        }

        let antwort = self
            .http
            .post(self.url("/geraete/weckruf"))
            .bearer_auth(&self.schluessel)
            .header("Accept", "application/json")
            .send()
            .await?;

        let a: Antwort = pruefen(antwort).await?.json().await?;
        Ok((a.basis, a.thema))
    }

    /// Abmelden: Der Server vergisst das Thema dieses Geraets.
    pub async fn weckruf_abmelden(&self) -> Result<(), OpenanyError> {
        let antwort = self
            .http
            .delete(self.url("/geraete/weckruf"))
            .bearer_auth(&self.schluessel)
            .header("Accept", "application/json")
            .send()
            .await?;

        pruefen(antwort).await.map(|_| ())
    }

    /// Eine interne Nachricht wegraeumen -- fuer die eigene Seite.
    pub async fn nachricht_loeschen(&self, id: i64) -> Result<(), OpenanyError> {
        let antwort = self
            .http
            .delete(self.url(&format!("/messages/{id}")))
            .bearer_auth(&self.schluessel)
            .header("Accept", "application/json")
            .send()
            .await?;

        pruefen(antwort).await.map(|_| ())
    }
}

/// Wie gross ein Stueck zum Server ist: **16 MiB**, die Hoechstgroesse drueben.
///
/// Gegenueber einem Geraet in der Naehe sind es 4 MB (siehe
/// `openany_sync::inhalte_holen::STUECK`) -- dort ist die Leitung das
/// Nadeloehr, und ein kleines Stueck kostet bei einem Abbruch weniger.
/// Hier ist es die Drossel: `throttle:api-token` laesst einem Schluessel 60
/// Anfragen je Minute. Bei 4 MB waeren das 240 MB je Minute und damit eine
/// kuenstliche Bremse; bei 16 MiB sind es knapp ein Gigabyte, mehr als jede
/// echte Leitung traegt.
pub const STUECK_ZUM_SERVER: u64 = 16 * 1024 * 1024;

/// Was aus einer Uebertragung werden soll.
#[derive(Debug, Clone)]
pub struct Hochzuladen<'a> {
    /// `file_node` oder `media`.
    pub art: &'a str,
    /// Die uuid der Sache, die ersetzt wird -- oder `None` fuer einen Neuzugang.
    pub schluessel: Option<&'a str>,
    /// Wo ein Neuzugang landet: Ordner- bzw. Album-uuid.
    pub eltern: Option<&'a str>,
    /// `files` oder `documents`; nur fuer Dateien und nur ohne Elternordner
    /// von Belang -- der sagt es sonst selbst.
    pub zone: Option<&'a str>,
    pub name: &'a str,
    pub groesse: u64,
    /// sha256 des ganzen Inhalts, hexadezimal. Der Server prueft ihn am Ende.
    pub abdruck: &'a str,
    /// Wie diese Seite in einer Konfliktkopie heisst.
    pub herkunft: Option<&'a str>,
}

impl Hochzuladen<'_> {
    fn json(&self) -> serde_json::Value {
        let mut feld = serde_json::Map::new();
        feld.insert("target_type".into(), self.art.into());
        feld.insert("name".into(), self.name.into());
        feld.insert("total_bytes".into(), self.groesse.into());
        feld.insert("content_hash".into(), self.abdruck.into());

        for (name, wert) in [
            ("target_key", self.schluessel),
            ("parent_key", self.eltern),
            ("zone", self.zone),
            ("origin", self.herkunft),
        ] {
            if let Some(w) = wert {
                feld.insert(name.into(), w.into());
            }
        }

        serde_json::Value::Object(feld)
    }
}

/// Eine angemeldete Uebertragung.
#[derive(Debug, Clone)]
pub struct Uebertragung {
    pub token: String,
    /// Wie weit der Server schon ist -- bei einer Wiederaufnahme mehr als 0.
    pub angekommen: u64,
}

/// Was drueben aus der Uebertragung geworden ist.
#[derive(Debug, Clone)]
pub struct Angekommen {
    pub art: String,
    pub schluessel: String,
    /// Drueben hat inzwischen jemand geschrieben: Es entstand eine zweite
    /// Sache statt einer Ueberschreibung.
    pub konflikt: bool,
    /// In welchem Baum die Datei gelandet ist. Der Ordner und der Dateityp
    /// reden mit, der Wunsch allein entscheidet nicht.
    pub zone: Option<String>,
}

/// Wie viele Eintraege `POST /api/sync/apply` hoechstens auf einmal nimmt.
///
/// Der Wert steht drueben in `SyncApplyController::MAX_ENTRIES`. Hier als
/// Konstante und nicht als Zahl irgendwo im Laeufer: Wer den Stapel schnuert,
/// soll die Grenze beim Namen nennen koennen.
pub const MAX_EINTRAEGE: usize = 500;

/// Aus einem anyid-Ticket einen Geraeteschluessel machen -- **einmalig, ohne
/// Schluessel**.
///
/// Das Ende eines Ablaufs, der nicht hier beginnt: Vorher hat das Programm
/// bei anyid eine Kopplung begonnen, ein Mensch hat sie IM BROWSER bestaetigt,
/// und das Programm hat sich sein Geraetetoken mit dem PKCE-Beweis abgeholt.
/// Damit erbittet es Tickets; eines davon kommt hierher.
///
/// **Freie Funktion und keine Methode**, weil es zu diesem Zeitpunkt noch
/// keinen [`OpenanyClient`] gibt: Der braucht einen Schluessel, und genau den
/// gibt es erst danach.
///
/// Der zurueckgegebene Schluessel ist **genau einmal zu sehen** -- danach
/// steht drueben nur noch sein Hash. Wer ihn nicht sofort ablegt, muss neu
/// koppeln.
/// Wo das anyid DIESER Instanz liegt -- `GET /api/instanz`.
///
/// **Damit es niemand abtippen muss.** Zu einer openany-Instanz gehoert genau
/// ein anyid. Das ist keine Wahl, sondern eine Auskunft -- und eine Frage, auf
/// die es nur eine richtige Antwort gibt, gehoert nicht an einen Menschen
/// gestellt. Wer die beiden falsch paart, bekommt beim Koppeln einen Fehler,
/// der nicht sagt, warum.
///
/// **Freie Funktion und kein `Result`**, aus zwei Gruenden. Zu diesem
/// Zeitpunkt gibt es noch keinen [`OpenanyClient`] -- er braucht einen
/// Schluessel, und den holt man sich erst danach. Und das hier ist eine
/// Bequemlichkeit: Eine Instanz, die gerade nicht antwortet, eine Adresse mit
/// einem Tippfehler, eine aeltere Fassung ohne diesen Weg -- in allen drei
/// Faellen bleibt das Feld eben leer und ein Mensch fuellt es aus. Daran
/// etwas scheitern zu lassen hiesse, dass sich eine falsch eingetippte Adresse
/// nicht einmal mehr korrigieren laesst.
pub async fn anyid_der_instanz(basis: &str, ca_pem: Option<&[u8]>) -> Option<String> {
    let http = OpenanyClient::bauen(ca_pem).ok()?;
    let basis = basis.trim_end_matches('/');

    let antwort = http
        .get(format!("{basis}/api/instanz"))
        .header("Accept", "application/json")
        .send()
        .await
        .ok()?;

    if !antwort.status().is_success() {
        return None;
    }

    let koerper: serde_json::Value = antwort.json().await.ok()?;

    koerper
        .get("anyid")
        .and_then(|w| w.as_str())
        .map(|s| s.trim_end_matches('/').to_string())
        .filter(|s| !s.is_empty())
}

pub async fn geraeteschluessel_holen(
    basis: &str,
    ticket: &str,
    geraetename: &str,
    ca_pem: Option<&[u8]>,
) -> Result<Geraeteschluessel, OpenanyError> {
    let http = OpenanyClient::bauen(ca_pem)?;
    let basis = basis.trim_end_matches('/');

    let antwort = http
        .post(format!("{basis}/api/geraete/schluessel"))
        .header("Accept", "application/json")
        .json(&serde_json::json!({ "ticket": ticket, "geraet": geraetename }))
        .send()
        .await?;

    Ok(pruefen(antwort).await?.json().await?)
}

/// Was beim Koppeln herauskommt.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Geraeteschluessel {
    /// Der Sanctum-Token im Klartext. Gehoert in den Schluesselbund
    /// beziehungsweise den Android-Keystore, nicht neben die SQLite-Datei.
    pub schluessel: String,
    pub geraet: String,
    /// Wem das Konto gehoert -- fuer die Anzeige, damit der Mensch sieht,
    /// womit er sich gerade verbunden hat.
    pub name: String,
}

async fn pruefen(antwort: reqwest::Response) -> Result<reqwest::Response, OpenanyError> {
    let status = antwort.status();

    if status.is_success() {
        return Ok(antwort);
    }

    match status {
        StatusCode::UNAUTHORIZED => Err(OpenanyError::SchluesselUngueltig),
        StatusCode::FORBIDDEN => Err(OpenanyError::NichtErlaubt(meldung(antwort).await)),
        StatusCode::PAYLOAD_TOO_LARGE => Err(OpenanyError::KeinPlatz(meldung(antwort).await)),
        StatusCode::TOO_MANY_REQUESTS => Err(OpenanyError::Gebremst {
            sekunden: antwort
                .headers()
                .get("Retry-After")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse().ok())
                // Kein Kopf, keine Auskunft -- dann eine Minute, so lang wie
                // die Fenster drueben (`throttle:10,1`).
                .unwrap_or(60),
        }),
        _ => Err(OpenanyError::Abgelehnt {
            status,
            meldung: meldung(antwort).await,
        }),
    }
}

/// Laravel schreibt `message`, openanys eigene Antworten `fehler`. Beide
/// nachsehen, statt sich auf eine zu verlassen.
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
        .unwrap_or_else(|| "openany refused the request.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn die_basis_verliert_ihren_schraegstrich() {
        // Sonst entstuenden Adressen mit `//api/delta`, und die Marke haenge
        // in `sync_state` unter zwei verschiedenen Basis-Adressen -- zwei
        // Gegenstellen, wo eine gemeint ist.
        let c = OpenanyClient::neu("https://openany.de/", "geheim").unwrap();

        assert_eq!(c.basis(), "https://openany.de");
        assert_eq!(c.url("/delta"), "https://openany.de/api/delta");
    }

    #[test]
    fn die_stapelgrenze_ist_die_des_servers() {
        assert_eq!(MAX_EINTRAEGE, 500, "SyncApplyController::MAX_ENTRIES");
    }
}
