//! openany.de als Gegenstelle -- **mit** Dateien und Bildern.
//!
//! Der [`OpenanyClient`] ist selbst schon eine
//! [`Gegenstelle`](openany_sync::Gegenstelle), aber eine ohne Inhalte:
//! `traegt_dateien()` ist dort `false`, und das war bis zum 16.09.2026
//! richtig. Was fehlte, ist nicht HTTP, sondern eine Uebersetzung.
//!
//! **Zwei Weltbilder.** Zwei Geraete benennen Inhalte nach ihrem Abdruck: Sie
//! legen sie unter `sha256` ab, derselbe Inhalt liegt genau einmal da, und
//! sie koennen einander fragen „welche dieser Abdruecke hast du?". Der Server
//! kennt diese Sicht ueberhaupt nicht -- bei ihm haengen Bytes an einer Datei
//! oder einem Bild, und `GET /api/sync/content` fragt nach `type` und `key`.
//! Einen Abdruck fuehrt er nirgends.
//!
//! Also uebersetzt diese Seite, und zwar mit dem, was nur sie weiss:
//!
//! * **Abdruck → Sache** kommt aus dem eigenen Speicher (`sache_zu_abdruck`).
//!   Die Datei, die diesen Inhalt traegt, hat drueben dieselbe uuid.
//! * **„hast du das?"** beantwortet die eigene Buchfuehrung
//!   (`inhalt_dort`): Was von dort kam oder erfolgreich dorthin ging.
//!
//! **Beides wird EINMAL vor der Uebertragung gebaut und dann nicht mehr
//! angefasst.** Waehrend Inhalte fliessen, darf der Speicher nicht gesperrt
//! sein -- ein Video kann Minuten dauern, und die Oberflaeche soll bedienbar
//! bleiben. Deshalb traegt diese Struktur zwei fertige Tabellen mit sich und
//! keine Verbindung zum Speicher.

//!
//! **Das Adressbuch** (seit Oktober 2026, docs/plan-adressbuch-e2e.md)
//! verschluesselt diese Seite auch: Kontakte gehen als Chiffrat hinaus und
//! kommen als Klartext beim Laeufer an ([`crate::adressbuch`]). Ein Geraet
//! ohne Konto-Schluessel (wartet auf Freigabe) uebergeht verschluesselte
//! Kontakte und zaehlt sie ([`ServerGegenstelle::verpasst`]) -- nach der
//! Freigabe holt es sie nach.
//!
//! **Kalender und Termine** ebenso (docs/plan-kalender-e2e.md,
//! [`crate::kalenderschloss`]).

use async_trait::async_trait;
use openany_client::{Art, Delta, Eintrag, Ergebnis, OpenanyClient, OpenanyError, Was};
use openany_direktpost::AnhangInfo;
use openany_sync::Gegenstelle;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use crate::adressbuch::{self, Adressbuch};
use crate::dateischloss;
use crate::kalenderschloss;
use crate::mappenteilen::{self, Freigabe};
use crate::notizschloss;
use openany_store::Inhaltsschluessel;

pub struct ServerGegenstelle {
    client: OpenanyClient,
    /// Abdruck → (`file_node`|`media`, uuid).
    wegweiser: BTreeMap<String, (String, String)>,
    /// Zum Ver- und Entschluesseln der Kontakte -- `None` ohne Konto-Schluessel.
    adressbuch: Option<Adressbuch>,
    /// uuid → wie das Foto aufgeht, aus dem Ziehen.
    fotos: Mutex<HashMap<String, AnhangInfo>>,
    /// Verschluesselte Kontakte, die dieses Geraet nicht lesen konnte.
    verpasst: AtomicUsize,
    /// Fuer verschluesselte Projektnamen (docs/plan-projektchat-e2e.md, 3c):
    /// Sie gehen mit dem Projektschluessel auf, den projektchat.rs holt.
    zustand: Option<std::sync::Arc<crate::Zustand>>,
    /// Kalender noch nicht umgestellt: Termine nennen ihre Basis auch offen
    /// (`base_klar`), weil sie drueben vielleicht noch im Klartext liegen.
    kalender_klar: bool,
    /// Notizen noch nicht umgestellt (wie `kalender_klar`).
    notizen_klar: bool,
    /// Texte verschluesselter Notizen aus dem Ziehen -- der Laeufer fragt sie
    /// einzeln ab (`notiztext`).
    notiztexte: Mutex<HashMap<String, String>>,
    /// Eigene geteilte Mappen (6c): Notizen darin reisen mit deren Schluessel.
    freigaben: Vec<Freigabe>,
    /// Schluessel verschluesselter Inhalte (docs/plan-dateien-e2e.md, 7b),
    /// nach Klartext-Abdruck -- VOR dem Lauf geladen, weil der Laeufer den
    /// Speicher waehrenddessen gesperrt haelt.
    inhaltsschluessel: HashMap<String, Inhaltsschluessel>,
    /// Was beim Ziehen aus Koepfen kam -- nach dem Lauf in den Speicher.
    gezogene_schluessel: Mutex<Vec<Inhaltsschluessel>>,
    /// Sache (uuid, bei Anhang-Zuordnungen `mappe|pfad`) → eigene Freigabe,
    /// in der sie liegt (7c, [`mappenteilen::dateifreigaben`]).
    dateifreigaben: HashMap<String, String>,
    /// Aus gezogenen Koepfen: an dieser uuid haengt drueben dieser Inhalt
    /// (Klartext-Abdruck) -- nach dem Lauf in `sache_dort`.
    gezogene_dort: Mutex<Vec<(String, String)>>,
}

impl ServerGegenstelle {
    /// Fuer den Lauf selbst: Der Laeufer holt keine Dateiinhalte, er gleicht
    /// ab, WAS es gibt. Die beiden Tabellen bleiben leer.
    pub fn neu(client: OpenanyClient) -> Self {
        Self {
            client,
            wegweiser: BTreeMap::new(),
            adressbuch: None,
            fotos: Mutex::new(HashMap::new()),
            verpasst: AtomicUsize::new(0),
            zustand: None,
            kalender_klar: false,
            notizen_klar: false,
            notiztexte: Mutex::new(HashMap::new()),
            freigaben: Vec::new(),
            inhaltsschluessel: HashMap::new(),
            gezogene_schluessel: Mutex::new(Vec::new()),
            dateifreigaben: HashMap::new(),
            gezogene_dort: Mutex::new(Vec::new()),
        }
    }

    /// Was beim Ziehen ueber Inhalte drueben bekannt wurde: (uuid, Abdruck).
    pub fn gezogene_dort(&self) -> Vec<(String, String)> {
        std::mem::take(&mut *self.gezogene_dort.lock().expect("dort"))
    }

    /// Welche Sache in welcher eigenen Freigabe liegt (7c).
    pub fn mit_dateifreigaben(mut self, karte: HashMap<String, String>) -> Self {
        self.dateifreigaben = karte;
        self
    }

    /// Die eigene Freigabe, in der diese Sache liegt -- `None`: nur die eigene.
    pub fn freigabe_fuer(&self, key: &str) -> Option<&Freigabe> {
        let uuid = self.dateifreigaben.get(key)?;
        self.freigaben.iter().find(|f| &f.uuid == uuid)
    }

    /// Eine eigene Freigabe nach ihrer uuid, wie der Server sie nennt.
    fn eigene_freigabe(&self, e: &Eintrag) -> Result<Option<&Freigabe>, ()> {
        match e.text("freigabe") {
            None => Ok(None),
            Some(u) => self
                .freigaben
                .iter()
                .find(|f| f.uuid == u)
                .map(Some)
                .ok_or(()),
        }
    }

    /// Mit den bekannten Schluesseln verschluesselter Inhalte (7b).
    pub fn mit_inhaltsschluesseln(mut self, alle: Vec<Inhaltsschluessel>) -> Self {
        self.inhaltsschluessel = alle.into_iter().map(|k| (k.abdruck.clone(), k)).collect();
        self
    }

    /// Was beim Ziehen an Schluesseln ankam -- zum Merken nach dem Lauf.
    pub fn gezogene_schluessel(&self) -> Vec<Inhaltsschluessel> {
        std::mem::take(&mut *self.gezogene_schluessel.lock().expect("schluessel"))
    }

    /// Mit den eigenen geteilten Mappen (mappenteilen.rs).
    pub fn mit_freigaben(mut self, freigaben: Vec<Freigabe>) -> Self {
        self.freigaben = freigaben;
        self
    }

    /// Eine Notiz einer geteilten Mappe verschliessen.
    fn mappennotiz_zu(
        &self,
        buch: &Adressbuch,
        f: &Freigabe,
        e: &Eintrag,
    ) -> Result<Eintrag, String> {
        let inhalt = mappenteilen::NotizInhalt {
            v: 1,
            zk_id: e.key.clone(),
            title: e.text("title").unwrap_or_default().to_string(),
            folder: e.text("folder").map(str::to_string),
            content: e.text("content").unwrap_or_default().to_string(),
        };
        let (v, version) = mappenteilen::notiz_zu(&buch.ich, &f.uuid, &f.schluessel, &inhalt)?;
        let hash = e.text("hash").unwrap_or_default();
        let basis = e.text("base_hash").filter(|s| !s.is_empty());
        let mut aus = Eintrag::neu(Art::Notiz, e.key.clone(), Was::Da)
            .mit("inhalt", v.inhalt)
            .mit("signatur", v.signatur)
            .mit("geraet", buch.geraet)
            .mit("version", version)
            .mit("freigabe", f.uuid.clone())
            .mit("hash", f.schluessel.abdruck_verdecken(&e.key, hash))
            .mit(
                "base_hash",
                basis.map_or(Value::Null, |b| {
                    f.schluessel.abdruck_verdecken(&e.key, b).into()
                }),
            )
            .mit(
                "base_alt",
                self.andere_basen(buch, &e.key, basis, Some(&f.uuid)),
            );
        for feld in ["size", "updated_at", "origin"] {
            if let Some(w) = e.felder.get(feld) {
                aus = aus.mit(feld, w.clone());
            }
        }
        Ok(aus)
    }

    /// Die Basis in den anderen Verdeckungen -- fuer eine Notiz, die in eine
    /// geteilte Mappe oder aus ihr heraus gewandert ist.
    fn andere_basen(
        &self,
        buch: &Adressbuch,
        zk: &str,
        basis: Option<&str>,
        ausser: Option<&str>,
    ) -> Value {
        let Some(b) = basis else {
            return Value::Array(vec![]);
        };
        let mut alle: Vec<Value> = self
            .freigaben
            .iter()
            .filter(|f| Some(f.uuid.as_str()) != ausser)
            .map(|f| f.schluessel.abdruck_verdecken(zk, b).into())
            .collect();
        if ausser.is_some() {
            alle.push(buch.konto.abdruck_verdecken(zk, b).into());
        }
        Value::Array(alle)
    }

    /// Sind die Notizen beim Server noch nicht umgestellt?
    pub fn mit_notizen_klar(mut self, klar: bool) -> Self {
        self.notizen_klar = klar;
        self
    }

    /// Verdeckte Schluessel → echte, aus dem eigenen Bestand (Mappenangaben,
    /// Anhang-Zuordnungen). Fuer Grabsteine vom Server: Sie nennen nur den
    /// verdeckten Schluessel.
    async fn verdeckte_karte(&self, buch: &Adressbuch) -> HashMap<String, String> {
        let mut karte = HashMap::new();
        let Some(z) = &self.zustand else {
            return karte;
        };
        let speicher = z.speicher.lock().await;
        for m in speicher.mappenangaben_alle().unwrap_or_default() {
            karte.insert(buch.schluessel_verdecken(&Art::Mappe, &m.pfad), m.pfad);
        }
        for a in speicher.anhaenge_alle().unwrap_or_default() {
            let key = format!("{}|{}", a.mappe, a.pfad);
            karte.insert(buch.schluessel_verdecken(&Art::Notizanhang, &key), key);
        }
        karte
    }

    /// Ist das Konto beim Kalender noch nicht umgestellt?
    pub fn mit_kalender_klar(mut self, klar: bool) -> Self {
        self.kalender_klar = klar;
        self
    }

    /// Mit Zugang zum Zustand: Projektnamen werden entschluesselt.
    pub fn mit_zustand(mut self, zustand: std::sync::Arc<crate::Zustand>) -> Self {
        self.zustand = Some(zustand);
        self
    }

    /// Mit dem Konto-Schluessel: Kontakte reisen verschluesselt.
    pub fn mit_adressbuch(mut self, adressbuch: Option<Adressbuch>) -> Self {
        self.adressbuch = adressbuch;
        self
    }

    /// Wie viele verschluesselte Kontakte ungelesen blieben, weil hier der
    /// Konto-Schluessel fehlt.
    pub fn verpasst(&self) -> usize {
        self.verpasst.load(Ordering::Relaxed)
    }

    fn foto_merken(&self, uuid: &str, info: Option<AnhangInfo>) {
        let mut fotos = self.fotos.lock().expect("fotos");
        match info {
            Some(i) => fotos.insert(uuid.to_string(), i),
            None => fotos.remove(uuid),
        };
    }

    fn foto(&self, uuid: &str) -> Option<AnhangInfo> {
        self.fotos.lock().expect("fotos").get(uuid).cloned()
    }

    /// Fuer die Inhalte danach -- mit dem Wissen, das der Lauf erst geschaffen
    /// hat.
    pub fn mit_wegweiser(mut self, wegweiser: BTreeMap<String, (String, String)>) -> Self {
        self.wegweiser = wegweiser;
        self
    }

    /// Zum Ver- und Entschluesseln -- `None` ohne Konto-Schluessel.
    pub fn adressbuch(&self) -> Option<&Adressbuch> {
        self.adressbuch.as_ref()
    }

    pub fn client(&self) -> &OpenanyClient {
        &self.client
    }

    /// Die Adresse, an der Marken, Urspruenge und die Buchfuehrung haengen.
    ///
    /// Eigener Name neben `Gegenstelle::basis`, damit ein Aufrufer sie nennen
    /// kann, ohne das Trait hereinzuholen -- und ohne dass zwei gleichnamige
    /// Methoden die Wahl zur Ratesache machen.
    pub fn basis_adresse(&self) -> &str {
        OpenanyClient::basis(&self.client)
    }

    /// Wie die Sache heisst, die diesen Inhalt traegt.
    pub fn sache(&self, abdruck: &str) -> Option<&(String, String)> {
        self.wegweiser.get(abdruck)
    }
}

#[async_trait]
impl Gegenstelle for ServerGegenstelle {
    fn basis(&self) -> &str {
        OpenanyClient::basis(&self.client)
    }

    async fn delta(&self, seit: i64) -> Result<Delta, OpenanyError> {
        let mut delta = self.client.delta(seit).await?;
        let mut aus = Vec::with_capacity(delta.eintraege.len());
        let mut karte: Option<HashMap<String, String>> = None;
        for mut e in delta.eintraege {
            // Ein Projekt mit verschluesseltem Namen: hier oeffnen. Geht er
            // nicht auf (noch keine Kiste), steht „…" da statt eines leeren Namens.
            if matches!(&e.art, Art::Unbekannt(a) if a == "project")
                && e.felder.get("name_inhalt").is_some_and(|v| !v.is_null())
            {
                let name = match &self.zustand {
                    Some(z) => {
                        crate::projektchat::name_oeffnen(z, &self.client, &e.key.clone(), &e.felder)
                            .await
                    }
                    None => None,
                };
                e.felder
                    .insert("name".into(), name.unwrap_or_else(|| "…".into()).into());
            }
            // Grabstein einer verdeckten Angabe: zurueck zum echten Schluessel.
            if notizschloss::verdeckte_art(&e.art) && e.was != Was::Da && e.key.len() == 64 {
                if let Some(buch) = &self.adressbuch {
                    karte.get_or_insert(self.verdeckte_karte(buch).await);
                }
                match karte.as_ref().and_then(|k| k.get(&e.key)) {
                    Some(echt) => e.key = echt.clone(),
                    // Hier nie gekannt: nichts zu loeschen.
                    None => continue,
                }
                aus.push(e);
                continue;
            }
            // Dateien, Bilder, Alben, Ordner (docs/plan-dateien-e2e.md, 7b).
            if dateischloss::ist_verschluesselt(&e) {
                let Some(buch) = &self.adressbuch else {
                    self.verpasst.fetch_add(1, Ordering::Relaxed);
                    continue;
                };
                // In einer eigenen Freigabe (7c): mit deren Schluessel. Ist sie
                // hier nicht bekannt, geht der Kopf nicht auf -- nichts ueberschreiben.
                let Ok(f) = self.eigene_freigabe(&e) else {
                    self.verpasst.fetch_add(1, Ordering::Relaxed);
                    continue;
                };
                if let Some(g) = buch.datei_kopf_oeffnen(&e, f) {
                    // Haengen drueben Bytes an dieser Sache, dann die, die der
                    // Kopf nennt.
                    if let (Some(k), true) = (&g.inhalt, e.text("inhalt_hash").is_some()) {
                        self.gezogene_dort
                            .lock()
                            .expect("dort")
                            .push((e.key.clone(), k.abdruck.clone()));
                    }
                    let mut gezogen = self.gezogene_schluessel.lock().expect("schluessel");
                    gezogen.extend(g.inhalt);
                    gezogen.extend(g.vorschau);
                    aus.push(g.eintrag);
                }
                continue;
            }
            if notizschloss::ist_verschluesselt(&e) {
                let Some(buch) = &self.adressbuch else {
                    self.verpasst.fetch_add(1, Ordering::Relaxed);
                    continue;
                };
                let freigabe = e.text("freigabe").map(str::to_string);
                if let (Art::Notiz, Some(fu)) = (&e.art, freigabe) {
                    if let Some(f) = self.freigaben.iter().find(|f| f.uuid == fu) {
                        let von = e.text("von_name").map(str::to_string).or_else(|| {
                            self.zustand
                                .as_ref()
                                .and_then(|z| crate::direktpost::mein_name(z))
                        });
                        let geraet = e.felder.get("geraet").and_then(Value::as_i64);
                        let version = e.felder.get("version").and_then(Value::as_u64);
                        let v = openany_direktpost::Verschluesselt {
                            inhalt: e.text("inhalt").unwrap_or_default().to_string(),
                            signatur: e.text("signatur").unwrap_or_default().to_string(),
                        };
                        let ed = match (&von, geraet) {
                            (Some(name), Some(g)) => {
                                crate::projektchat::Verzeichnisse::neu(&self.client)
                                    .ed25519(name, g)
                                    .await
                            }
                            _ => None,
                        };
                        if let (Some(ed), Some(version)) = (ed, version) {
                            if let Some(n) = mappenteilen::notiz_auf(
                                &f.uuid,
                                &f.schluessel,
                                &e.key,
                                version as u32,
                                &v,
                                &ed,
                            ) {
                                let mut klar = Eintrag::neu(Art::Notiz, e.key.clone(), Was::Da)
                                    .mit("title", n.title)
                                    .mit("folder", n.folder.map_or(Value::Null, Value::from))
                                    .mit("hash", openany_client::notizabdruck(&n.content))
                                    .mit("size", n.content.len() as u64);
                                if let Some(t) = e.felder.get("updated_at") {
                                    klar = klar.mit("updated_at", t.clone());
                                }
                                self.notiztexte
                                    .lock()
                                    .expect("notiztexte")
                                    .insert(e.key.clone(), n.content);
                                aus.push(klar);
                            }
                        }
                    }
                } else if e.art == Art::Notiz {
                    if let Some((klar, text)) = buch.notiz_oeffnen(&e) {
                        self.notiztexte
                            .lock()
                            .expect("notiztexte")
                            .insert(klar.key.clone(), text);
                        aus.push(klar);
                    }
                } else if let Ok(f) = self.eigene_freigabe(&e) {
                    aus.extend(buch.angabe_oeffnen(&e, f));
                }
                continue;
            }
            let kalender = kalenderschloss::ist_verschluesselt(&e);
            if !kalender && !adressbuch::ist_verschluesselt(&e) {
                aus.push(e);
                continue;
            }
            let Some(buch) = &self.adressbuch else {
                self.verpasst.fetch_add(1, Ordering::Relaxed);
                continue;
            };
            // Was nicht aufgeht (fremd unterschrieben, unbekanntes Geraet),
            // kommt hier nicht an -- und ueberschreibt nichts.
            if kalender {
                let klar = match e.art {
                    Art::Kalender => buch.kalender_oeffnen(&e),
                    _ => buch.termin_oeffnen(&e),
                };
                aus.extend(klar);
            } else if let Some((klar, foto)) = buch.oeffnen(&e) {
                self.foto_merken(&klar.key, foto);
                aus.push(klar);
            }
        }
        delta.eintraege = aus;
        Ok(delta)
    }

    async fn notiztext(&self, zk_id: &str) -> Result<String, OpenanyError> {
        if let Some(t) = self.notiztexte.lock().expect("notiztexte").get(zk_id) {
            return Ok(t.clone());
        }
        self.client.notiztext(zk_id).await
    }

    async fn anwenden(&self, eintraege: &[Eintrag]) -> Result<Vec<Ergebnis>, OpenanyError> {
        let Some(buch) = &self.adressbuch else {
            // Ohne Konto-Schluessel keine Dateien, Bilder, Alben: Der Server
            // nimmt sie nur verschluesselt (seit 7d), und unterwegs waeren
            // ihre Namen lesbar.
            let ohne: Vec<Eintrag> = eintraege
                .iter()
                .filter(|e| !(dateischloss::ist_dateiart(&e.art) && e.was == Was::Da))
                .cloned()
                .collect();
            return self.client.anwenden(&ohne).await;
        };
        let mut aus = Vec::with_capacity(eintraege.len());
        for e in eintraege {
            if e.art == Art::Kontakt && e.was == Was::Da {
                aus.push(
                    buch.verschliessen(e, self.foto(&e.key).as_ref())
                        .map_err(OpenanyError::Unlesbar)?,
                );
            } else if e.art == Art::Notiz && e.was == Was::Da {
                match mappenteilen::zustaendig(&self.freigaben, e.text("folder")) {
                    Some(f) => aus.push(
                        self.mappennotiz_zu(buch, f, e)
                            .map_err(OpenanyError::Unlesbar)?,
                    ),
                    None => {
                        let basis = e.text("base_hash").filter(|s| !s.is_empty());
                        let zu = buch
                            .notiz_verschliessen(e, self.notizen_klar)
                            .map_err(OpenanyError::Unlesbar)?
                            .mit("base_alt", self.andere_basen(buch, &e.key, basis, None));
                        aus.push(zu);
                    }
                }
            } else if notizschloss::verdeckte_art(&e.art) {
                let f = (e.art == Art::Notizanhang)
                    .then(|| self.freigabe_fuer(&e.key))
                    .flatten();
                aus.push(
                    buch.angabe_verschliessen(e, f)
                        .map_err(OpenanyError::Unlesbar)?,
                );
            } else if dateischloss::ist_dateiart(&e.art) && e.was == Was::Da {
                // Alben und Ordner immer als Kopf. Dateien und Bilder nur,
                // wenn ihr Inhalt schon verschluesselt druben liegt (Schluessel
                // bekannt) -- sonst reist der Kopf mit dem Hochladen. Ein
                // offener Eintrag bis dahin ginge nur als lesbarer Name auf
                // die Leitung (seit 7d nimmt der Server ihn ohnehin nicht).
                let f = self.freigabe_fuer(&e.key);
                if dateischloss::traegt_inhalt(e) {
                    if let Some(k) = e.text("hash").and_then(|a| self.inhaltsschluessel.get(a)) {
                        let vorschau = e
                            .text("thumb_hash")
                            .and_then(|t| self.inhaltsschluessel.get(t))
                            .map(|v| v.chiffrat.as_str());
                        aus.push(
                            buch.datei_kopf_verschliessen(e, Some(k), vorschau, f)
                                .map_err(OpenanyError::Unlesbar)?,
                        );
                    }
                } else {
                    aus.push(
                        buch.datei_kopf_verschliessen(e, None, None, f)
                            .map_err(OpenanyError::Unlesbar)?,
                    );
                }
            } else if e.art == Art::Kalender && e.was == Was::Da {
                aus.push(
                    buch.kalender_verschliessen(e)
                        .map_err(OpenanyError::Unlesbar)?,
                );
            } else if e.art == Art::Termin && e.was == Was::Da {
                aus.push(
                    buch.termin_verschliessen(e, self.kalender_klar)
                        .map_err(OpenanyError::Unlesbar)?,
                );
            } else {
                aus.push(e.clone());
            }
        }
        self.client.anwenden(&aus).await
    }

    async fn kontaktfoto(&self, uuid: &str) -> Result<Vec<u8>, OpenanyError> {
        let bytes = self.client.inhalt("contact_photo", uuid).await?;
        match self.foto(uuid) {
            Some(info) => openany_direktpost::anhang_oeffnen(&bytes, &info)
                .map_err(|e| OpenanyError::Unlesbar(e.to_string())),
            None => Ok(bytes),
        }
    }

    /// **Seit dem 16.09.2026 ja.** Davor stand hier `false` mit der
    /// Begruendung, die Bytes muessten ueber `UploadSession` kommen -- das
    /// war die Beschreibung der Luecke, die diese Datei schliesst.
    fn traegt_dateien(&self) -> bool {
        true
    }

    async fn inhalt(&self, abdruck: &str, von: u64, laenge: u64) -> Result<Vec<u8>, OpenanyError> {
        let Some((art, key)) = self.sache(abdruck) else {
            // Kein Wegweiser heisst: Diese Seite weiss nicht, welche Datei
            // den Inhalt traegt -- also kann sie ihn drueben nicht benennen.
            return Err(OpenanyError::Unlesbar(format!(
                "No file is known here for {abdruck}."
            )));
        };

        self.client.inhalt_bereich(art, key, von, laenge).await
    }

    /// **Was ich benennen kann, frage ich auch.**
    ///
    /// Der Server kann diese Frage nicht beantworten -- er fuehrt keine
    /// Abdruecke. Die eigene Buchfuehrung taugt
    /// hier auch nicht: Sie haelt fest, was HINAUF ging, und beim ersten Lauf
    /// ist sie leer. Wer sie hier befragte, holte nie etwas herunter.
    ///
    /// Die Antwort ist trotzdem fast immer richtig, und zwar aus einem
    /// Grund, der nichts mit dem Server zu tun hat: **Ein Inhalt, der hier
    /// fehlt, ist hier nie entstanden.** Was diese Seite selbst anlegt, legt
    /// sie mitsamt Bytes ab. Fehlt etwas, kam die Datei von drueben -- und
    /// dort liegen ihre Bytes.
    ///
    /// Bleibt der seltene Fall, dass auch drueben keine Bytes liegen (eine
    /// Datei, die ein drittes Geraet angelegt und noch nirgends hochgeladen
    /// hat). Dann antwortet der Server mit 404, und der Versuch steht als
    /// Fehler im Bericht statt als „nicht da". Das ist eine Zeile zu viel im
    /// Protokoll, kein falscher Inhalt.
    async fn inhalte_da(&self, abdruecke: &[String]) -> Result<Vec<String>, OpenanyError> {
        Ok(abdruecke
            .iter()
            .filter(|a| self.wegweiser.contains_key(*a))
            .cloned()
            .collect())
    }

    /// Im Klartext wie beim blanken Client: Der Server rechnet ein
    /// Kontaktfoto neu, und dasselbe Bild kaeme sonst bei jedem Lauf als
    /// Aenderung zurueck.
    ///
    /// **Verschluesselt ja:** Jeder neue Stand eines Kontakts traegt einen
    /// neuen Foto-Schluessel, also muss das Bild mit. Der Abdruck im Chiffrat
    /// ist der des Klartexts -- es kommt nichts als Aenderung zurueck.
    fn fotos_mitschicken(&self) -> bool {
        self.adressbuch.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ein Client, der nie etwas schickt -- hier wird nur uebersetzt.
    fn gegenstelle() -> ServerGegenstelle {
        ServerGegenstelle::neu(OpenanyClient::neu("https://beispiel.invalid", "x").unwrap())
    }

    fn mit(paare: &[(&str, &str, &str)]) -> ServerGegenstelle {
        gegenstelle().mit_wegweiser(
            paare
                .iter()
                .map(|(a, art, uuid)| (a.to_string(), (art.to_string(), uuid.to_string())))
                .collect(),
        )
    }

    #[test]
    fn ein_abdruck_wird_zur_sache() {
        let g = mit(&[("aa", "file_node", "u-1"), ("bb", "media", "u-2")]);

        assert_eq!(
            g.sache("aa"),
            Some(&("file_node".to_string(), "u-1".to_string()))
        );
        assert_eq!(
            g.sache("bb"),
            Some(&("media".to_string(), "u-2".to_string()))
        );
        assert_eq!(g.sache("cc"), None);
    }

    /// Ohne Wegweiser ist der Inhalt drueben nicht zu benennen -- und das
    /// muss ein Fehler sein, keine leere Antwort: Eine leere Antwort naehme
    /// [`openany_sync::inhalt_holen`] als „Datei zu Ende".
    #[tokio::test]
    async fn ohne_wegweiser_kein_inhalt() {
        let fehler = gegenstelle().inhalt("aa", 0, 10).await.unwrap_err();

        assert!(matches!(fehler, OpenanyError::Unlesbar(_)));
    }

    /// Gefragt wird nach allem, was sich benennen laesst -- nicht nach dem,
    /// was die Buchfuehrung kennt. Sonst holte der erste Lauf nie etwas.
    #[tokio::test]
    async fn gefragt_wird_was_benennbar_ist() {
        let g = mit(&[("aa", "file_node", "u-1")]);

        let da = g
            .inhalte_da(&["aa".into(), "unbekannt".into()])
            .await
            .unwrap();

        assert_eq!(da, vec!["aa".to_string()]);
    }

    #[test]
    fn dieser_server_traegt_dateien() {
        assert!(gegenstelle().traegt_dateien());
        // Und Kontaktfotos schickt er weiterhin nicht bei jedem Lauf mit:
        // Er rechnet sie neu, sie kaemen sonst als Aenderung zurueck.
        assert!(!gegenstelle().fotos_mitschicken());
    }
}
