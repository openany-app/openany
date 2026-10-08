//! Der Laeufer: der Teil, in dem die Reihenfolge steckt.
//!
//! Zwei Durchgaenge je Lauf:
//!
//! 1. **Ziehen** -- `GET /api/delta` ab der eigenen Marke, anwenden.
//! 2. **Schieben** -- das eigene Protokoll ab der eigenen Marke, in dieselbe
//!    Form gebracht und hinuebergeschickt.
//!
//! **Erst ziehen, dann schieben.** Andersherum entstuenden Konfliktkopien
//! drueben statt hier -- und der Mensch, der sie aufloesen soll, sitzt vor
//! diesem Geraet, nicht vor dem Server.
//!
//! **Ein Lauf, der abbricht, ist kein verlorener Lauf.** Die Marken ruecken
//! seitenweise nach, nicht am Ende. Ein Abbruch nach der Haelfte kostet die
//! halbe Arbeit, nicht die ganze -- auf einer Leitung, die in jedem Tunnel
//! abreisst, ist das der Unterschied zwischen einem Erstabgleich, der
//! irgendwann fertig wird, und einem, der nie fertig wird.

use crate::entscheidung::{entscheiden, ursprung_waehlen, Entscheidung};
use crate::gegenstelle::Gegenstelle;
use crate::hinaus::{
    album_aus_eintrag, album_gleich, anhang_aus_eintrag, bild_aus_eintrag, bild_gleich,
    datei_aus_eintrag, datei_gleich, freie_zk_id, hinausbringen, ist_speicherart, konflikttitel,
    wege_von,
};
use openany_client::{
    abdruck_von_eintrag, notizabdruck, Art, Eintrag, OpenanyError, Speicherstand, Terminfelder,
    Was, MAX_EINTRAEGE,
};
use openany_store::{Kalender, Kontakt, Notiz, Protokoll, Speicher, SpeicherFehler, Termin};
use std::collections::BTreeMap;

/// Was ein Lauf getan hat.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Bericht {
    pub gezogen: usize,
    pub geschoben: usize,
    /// Eintraege, die dieses Programm nicht traegt -- Alben, Dateien, Bilder
    /// -- und solche aus einer neueren Fassung.
    pub uebersprungen: usize,
    /// WARUM uebersprungen wurde, je Grund gezaehlt.
    ///
    /// Am 16.09.2026 meldete der Bericht "uebersprungen 72", spaeter "33",
    /// dann "7" -- und niemand konnte die Zahl deuten. Drei der Ursachen
    /// waren Fehler (ein fehlender Zweig fuer Anhaenge, ein leerer
    /// Wegweiser, eine Drossel unter einer strengeren), und jede kostete
    /// Stunden, weil die Zahl keinen Namen hatte. Seitdem gilt: **Wo etwas
    /// uebersprungen wird, gehoert der Grund in den Bericht.**
    pub gruende: BTreeMap<String, usize>,
    /// Wie oft eine fremde Fassung als eigene Sache danebengelegt wurde.
    /// **Zaehlt getrennt**, weil sie das Einzige sind, was einen Menschen
    /// etwas angeht: Alles andere loest der Lauf selbst auf.
    pub konflikte: usize,
    pub fehler: Vec<String>,
    /// Wie voll die Box ist -- aus der letzten Delta-Seite.
    pub speicher: Option<Speicherstand>,
    /// Anhaenge, die drueben noch nicht angenommen wurden, weil ihr Ziel dort
    /// fehlt -- die Bytes kommen erst NACH dem Lauf hinueber. Sie sind wieder
    /// vorgemerkt; ein weiterer Lauf bringt sie, sobald das Ziel da ist.
    pub offen: usize,
}

impl Bericht {
    pub fn durchgelaufen(&self) -> bool {
        self.fehler.is_empty()
    }

    /// Einen Eintrag uebergehen -- und sagen, warum.
    ///
    /// Die einzige Stelle, die `uebersprungen` erhoeht: Ein `+= 1` ohne
    /// Grund waere wieder eine Zahl ohne Namen.
    pub fn ueberspringen(&mut self, grund: impl Into<String>) {
        self.uebersprungen += 1;
        *self.gruende.entry(grund.into()).or_insert(0) += 1;
    }

    /// Einen zweiten Lauf dazuzaehlen -- fuer das Nachreichen (siehe `offen`).
    pub fn dazu(&mut self, anderer: Bericht) {
        self.gezogen += anderer.gezogen;
        self.geschoben += anderer.geschoben;
        self.uebersprungen += anderer.uebersprungen;
        self.konflikte += anderer.konflikte;
        self.fehler.extend(anderer.fehler);
        for (grund, n) in anderer.gruende {
            *self.gruende.entry(grund).or_insert(0) += n;
        }
        if anderer.speicher.is_some() {
            self.speicher = anderer.speicher;
        }
        // Was der zweite Lauf noch offen liess, ist offen; was er brachte,
        // ist es nicht mehr.
        self.offen = anderer.offen;
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Lauffehler {
    #[error(transparent)]
    Gegenstelle(#[from] OpenanyError),
    #[error(transparent)]
    Speicher(#[from] SpeicherFehler),
    /// Die Gegenstelle meldet "es gibt mehr", ruckt die Marke aber nicht vor.
    ///
    /// **Ohne diese Pruefung liefe der Laeufer ewig** -- dieselbe Seite,
    /// dieselben Eintraege, bis der Akku leer ist. Auf einem Server faellt so
    /// etwas als Last auf; auf einem Telefon faellt es als leerer Akku auf,
    /// und niemand weiss, warum.
    #[error("The counterpart is not making progress (cursor stays at {marke}).")]
    TrittAufDerStelle { marke: i64 },
}

/// Der Laeufer -- fuer genau eine Gegenstelle.
pub struct Laeufer<'a, G: Gegenstelle> {
    speicher: &'a Speicher,
    gegenstelle: &'a G,
    /// Wie dieses Geraet heisst, wenn es drueben eine Konfliktkopie
    /// verursacht. Dasselbe Wort, das in der Geraeteliste steht.
    herkunft: String,
}

impl<'a, G: Gegenstelle> Laeufer<'a, G> {
    pub fn neu(speicher: &'a Speicher, gegenstelle: &'a G, herkunft: impl Into<String>) -> Self {
        Self {
            speicher,
            gegenstelle,
            herkunft: herkunft.into(),
        }
    }

    fn basis(&self) -> &str {
        self.gegenstelle.basis()
    }

    /// Ein vollstaendiger Lauf.
    ///
    /// **Ein Fehler beim Ziehen haelt das Schieben an.** Wer auf einem
    /// veralteten Stand schiebt, erzeugt Konfliktkopien drueben -- dort, wo
    /// niemand sitzt, der sie aufloest. Lieber gar nicht schieben und beim
    /// naechsten Lauf beides.
    pub async fn lauf(&self) -> Bericht {
        let mut bericht = Bericht::default();

        if let Err(e) = self.ziehen(&mut bericht).await {
            bericht.fehler.push(e.to_string());
        } else if let Err(e) = self.schieben(&mut bericht).await {
            bericht.fehler.push(e.to_string());
        } else if let Ok(marke) = self.speicher.marke(self.basis()) {
            // Nur nach einem ganzen Lauf, und ein Fehlschlag ist hier keiner:
            // Die Daten sind schon auf beiden Seiten. Er kostet beim naechsten
            // Lauf drueben nur eine Runde, die nichts mehr aendert.
            let _ = self
                .gegenstelle
                .marken_melden(marke.eigene, marke.fremde)
                .await;
        }

        // Der Stand wird auch nach einem Fehler festgehalten: Was bis hierher
        // gelaufen ist, ist gelaufen.
        let _ = self
            .speicher
            .lauf_beendet(self.basis(), bericht.fehler.first().map(String::as_str));

        bericht
    }

    // --- Ziehen ----------------------------------------------------------

    async fn ziehen(&self, bericht: &mut Bericht) -> Result<(), Lauffehler> {
        let mut marke = self.speicher.marke(self.basis())?.fremde;

        loop {
            let seite = self.gegenstelle.delta(marke).await?;

            if seite.speicher.is_some() {
                bericht.speicher = seite.speicher;
            }

            for eintrag in &seite.eintraege {
                self.einziehen(eintrag, bericht).await?;
            }

            // Seitenweise und nicht am Ende: Ein Abbruch auf Seite sieben
            // kostet Seite sieben, nicht die sechs davor.
            self.speicher
                .fremde_marke_setzen(self.basis(), seite.cursor)?;

            if !seite.more {
                return Ok(());
            }

            if seite.cursor <= marke {
                return Err(Lauffehler::TrittAufDerStelle { marke });
            }

            marke = seite.cursor;
        }
    }

    async fn einziehen(&self, eintrag: &Eintrag, bericht: &mut Bericht) -> Result<(), Lauffehler> {
        if ist_speicherart(&eintrag.art) && !self.gegenstelle.traegt_dateien() {
            bericht.ueberspringen("files and pictures: not agreed with this counterpart");
            return Ok(());
        }
        match eintrag.was {
            // **Der Ursprung wird vergessen.** Ein Ursprung zu etwas, das es
            // nicht mehr gibt, behauptete beim naechsten Auftauchen eine
            // Uebereinstimmung, die es nie gab.
            //
            // Und **`Protokoll::Von(self.basis())`**: Was von drueben kam, ist keine
            // hiesige Aenderung. Der Grabstein steht drueben; ihn hier noch
            // einmal zu setzen hiesse, ihn beim naechsten Lauf
            // zurueckzuschieben.
            // ENDET EINE MITGLIEDSCHAFT, faellt das ganze Projekt weg --
            // samt Sachen, Protokoll, Marke und Urspruengen. Was bliebe,
            // waere eine Kopie, die niemand mehr sehen darf.
            Was::Papierkorb | Was::Fort if eintrag.art.ueber_die_leitung() == "project" => {
                if self
                    .speicher
                    .projekt_vergessen(self.basis(), &eintrag.key)?
                {
                    bericht.gezogen += 1;
                } else {
                    bericht.ueberspringen("project this device did not know about");
                }

                Ok(())
            }
            Was::Papierkorb | Was::Fort => {
                let getan = if eintrag.was == Was::Papierkorb {
                    self.speicher.papierkorb(
                        &eintrag.art,
                        &eintrag.key,
                        Protokoll::Von(self.basis()),
                    )?
                } else {
                    self.speicher.loeschen(
                        &eintrag.art,
                        &eintrag.key,
                        Protokoll::Von(self.basis()),
                    )?
                };

                self.speicher
                    .ursprung_vergessen(self.basis(), &eintrag.art, &eintrag.key)?;

                if getan {
                    bericht.gezogen += 1;
                } else if eintrag.was == Was::Papierkorb {
                    // Lag hier schon im Papierkorb -- oder nie: etwa eine
                    // Sache, die drueben angelegt und weggelegt wurde, bevor
                    // dieses Geraet je davon hoerte.
                    bericht
                        .ueberspringen("trash note for something that is not (or no longer) here");
                } else {
                    bericht.ueberspringen("deletion of something that was already gone here");
                }

                Ok(())
            }
            Was::Da => match &eintrag.art {
                Art::Notiz => self.notiz_ziehen(eintrag, bericht).await,
                Art::Termin => self.termin_ziehen(eintrag, bericht),
                Art::Kalender => self.kalender_ziehen(eintrag, bericht),
                Art::Kontakt => self.kontakt_ziehen(eintrag, bericht).await,
                Art::Datei => self.datei_ziehen(eintrag, bericht),
                Art::Album => self.album_ziehen(eintrag, bericht),
                Art::Medium => self.bild_ziehen(eintrag, bericht),
                Art::Notizanhang => self.anhang_ziehen(eintrag, bericht),
                // WELCHE PROJEKTE ES GIBT, sagt nur dieser Strom -- der
                // Inhalt reist dann in einem eigenen
                // (`docs/plan-projekte-abgleich.md`). Hier steht deshalb
                // Name und Rolle, mehr nicht.
                Art::Unbekannt(art) if art == "project" => self.projekt_ziehen(eintrag, bericht),
                Art::Unbekannt(art) => {
                    bericht.ueberspringen(format!("unknown kind \"{art}\" (newer counterpart)"));
                    Ok(())
                }
            },
        }
    }

    /// Ein Projekt, von dem dieses Geraet Mitglied ist.
    ///
    /// **Nur die Auskunft, DASS es das Projekt gibt** -- Boards, Spalten und
    /// Karten kommen im Strom des Projekts. Die Rolle steht hier, weil sie
    /// eine Eigenschaft der Mitgliedschaft ist und nicht des Projekts.
    fn projekt_ziehen(&self, eintrag: &Eintrag, bericht: &mut Bericht) -> Result<(), Lauffehler> {
        let name = eintrag.text("name").unwrap_or("Projekt").to_string();
        let rolle = eintrag.text("role").unwrap_or("member").to_string();

        let neu = openany_store::Projekt {
            uuid: eintrag.key.clone(),
            name,
            rolle,
            geaendert_at: eintrag.text("updated_at").unwrap_or_default().to_string(),
            mitgliederliste: None,
        };

        if self.speicher.projekt(&eintrag.key)?.as_ref() == Some(&neu) {
            return Ok(());
        }

        self.speicher.projekt_schreiben(&neu)?;
        bericht.gezogen += 1;

        Ok(())
    }

    /// Eine Notiz -- dreiseitig entschieden ueber den Hash ihres Textes.
    async fn notiz_ziehen(
        &self,
        eintrag: &Eintrag,
        bericht: &mut Bericht,
    ) -> Result<(), Lauffehler> {
        // Der Text reist nicht mit; er kommt einzeln. Faellt das aus -- eine
        // drueben inzwischen geloeschte Notiz etwa --, wird uebergangen statt
        // eine leere Notiz anzulegen.
        let Ok(text) = self.gegenstelle.notiztext(&eintrag.key).await else {
            bericht.ueberspringen("note: the text could not be fetched from the other side");

            return Ok(());
        };

        let drueben = notizabdruck(&text);
        let hiesige = self.speicher.notiz(&eintrag.key)?;
        let hier = hiesige.as_ref().map(|n| notizabdruck(&n.inhalt));
        let ursprung = self.ursprung(eintrag, hier.as_deref(), &drueben)?;

        let entscheidung = entscheiden(ursprung.as_deref(), hier.as_deref(), &drueben);

        match entscheidung {
            Entscheidung::Unveraendert | Entscheidung::HierBleibt => {}
            Entscheidung::Anlegen | Entscheidung::Uebernehmen => {
                bericht.gezogen += 1;
                self.speicher.notiz_schreiben(
                    &Notiz {
                        zk_id: eintrag.key.clone(),
                        titel: eintrag.text("title").unwrap_or_default().to_string(),
                        inhalt: text,
                        mappe: eintrag.text("folder").map(str::to_string),
                        papierkorb_at: None,
                        geaendert_at: eintrag.text("updated_at").unwrap_or_default().to_string(),
                    },
                    Protokoll::Von(self.basis()),
                )?;
            }
            // **Der hiesige Stand bleibt unangetastet** -- die fremde Fassung
            // kommt als eigene Notiz daneben, mit EIGENER zk_id.
            // Zusammenfuehren hiesse raten, und geratener Text ist verlorener
            // Text.
            Entscheidung::Konflikt => {
                self.speicher.notiz_schreiben(
                    &Notiz {
                        zk_id: freie_zk_id(self.speicher)?,
                        titel: konflikttitel(
                            eintrag.text("title").unwrap_or_default(),
                            &self.gegenstelle.name(),
                        ),
                        inhalt: text,
                        mappe: eintrag.text("folder").map(str::to_string),
                        papierkorb_at: None,
                        geaendert_at: String::new(),
                    },
                    // MERKEN, anders als sonst beim Ziehen: Diese Notiz kennt
                    // drueben niemand. Ohne Protokolleintrag bliebe sie fuer
                    // immer nur auf diesem Geraet.
                    Protokoll::Merken,
                )?;

                bericht.gezogen += 1;
                bericht.konflikte += 1;
            }
        }

        // **Der Ursprung wird IMMER auf den fremden Stand gesetzt**, auch
        // nach einem Konflikt: Damit ist der Streit als beigelegt vermerkt,
        // und der naechste Lauf schiebt die hiesige Fassung als die neuere
        // hinueber, statt denselben Konflikt noch einmal zu finden.
        self.speicher
            .ursprung_merken(self.basis(), &Art::Notiz, &eintrag.key, &drueben)?;

        Ok(())
    }

    /// Ein Termin -- dreiseitig wie eine Notiz, nur ueber den Abdruck der
    /// neun Felder statt ueber den Text.
    fn termin_ziehen(&self, eintrag: &Eintrag, bericht: &mut Bericht) -> Result<(), Lauffehler> {
        let Some(kalender_uuid) = eintrag.text("calendar").map(str::to_string) else {
            // Ohne Kalender-uuid ist er nicht zuzuordnen -- das waere ein
            // kaputter Eintrag, kein fehlender Kalender.
            bericht.ueberspringen("event without calendar");

            return Ok(());
        };

        let drueben = abdruck_von_eintrag(&eintrag.felder);
        let hiesiger = self.speicher.termin(&eintrag.key)?;
        let hier = hiesiger.as_ref().map(Termin::abdruck);
        let ursprung = self.ursprung(eintrag, hier.as_deref(), &drueben)?;

        let felder = Terminfelder::aus_eintrag(&eintrag.felder);

        match entscheiden(ursprung.as_deref(), hier.as_deref(), &drueben) {
            Entscheidung::Unveraendert | Entscheidung::HierBleibt => {}
            Entscheidung::Anlegen | Entscheidung::Uebernehmen => {
                bericht.gezogen += 1;
                // **HIER WEICHT DIESES PROGRAMM VOM SERVER AB, und zwar
                // bewusst.** openanys `SyncApply` uebergeht einen Termin,
                // dessen Kalender es dort nicht gibt -- zu Recht, denn ihn in
                // irgendeinen zu legen hiesse raten. Hier wird er
                // aufgenommen, MIT seiner echten Kalender-uuid: Das ist kein
                // Raten, sondern ein Verweis auf etwas, das noch nicht da
                // ist. Er taucht auf, sobald der Kalender kommt -- statt
                // verloren zu sein, weil die Marke inzwischen vorbei ist.
                self.speicher.termin_schreiben(
                    &Termin {
                        uuid: eintrag.key.clone(),
                        kalender_uuid,
                        felder,
                        papierkorb_at: None,
                        geaendert_at: eintrag.text("updated_at").unwrap_or_default().to_string(),
                    },
                    Protokoll::Von(self.basis()),
                )?;
            }
            Entscheidung::Konflikt => {
                let mut kopie = felder;
                kopie.titel = konflikttitel(&kopie.titel, &self.gegenstelle.name());

                self.speicher.termin_schreiben(
                    &Termin {
                        // EIGENE uuid. Die des Originals waere der teure
                        // Fehler: zwei Zeilen mit derselben Identitaet, und
                        // der naechste Abgleich faende nicht mehr heraus,
                        // welche gemeint ist.
                        uuid: uuid::Uuid::new_v4().to_string(),
                        kalender_uuid,
                        felder: kopie,
                        papierkorb_at: None,
                        geaendert_at: String::new(),
                    },
                    Protokoll::Merken,
                )?;

                bericht.gezogen += 1;
                bericht.konflikte += 1;
            }
        }

        self.speicher
            .ursprung_merken(self.basis(), &Art::Termin, &eintrag.key, &drueben)?;

        Ok(())
    }

    /// Eine Datei oder ein Ordner -- nur, WAS es gibt. Der Inhalt kommt
    /// getrennt (`inhalte_holen`), je nach Regel dieses Geraets.
    fn datei_ziehen(&self, eintrag: &Eintrag, bericht: &mut Bericht) -> Result<(), Lauffehler> {
        let Some(neu) = datei_aus_eintrag(eintrag) else {
            bericht.ueberspringen("file without required fields");
            return Ok(());
        };
        if let Some(da) = self.speicher.datei(&eintrag.key)? {
            if datei_gleich(&da, &neu) {
                return Ok(());
            }
        }
        self.speicher
            .datei_schreiben(&neu, Protokoll::Von(self.basis()))?;
        bericht.gezogen += 1;
        Ok(())
    }

    fn album_ziehen(&self, eintrag: &Eintrag, bericht: &mut Bericht) -> Result<(), Lauffehler> {
        let Some(neu) = album_aus_eintrag(eintrag) else {
            bericht.ueberspringen("album without required fields");
            return Ok(());
        };
        if let Some(da) = self.speicher.album(&eintrag.key)? {
            if album_gleich(&da, &neu) {
                return Ok(());
            }
        }
        self.speicher
            .album_schreiben(&neu, Protokoll::Von(self.basis()))?;
        bericht.gezogen += 1;
        Ok(())
    }

    /// Ein Notiz-Anhang -- eine ZUORDNUNG, keine Datei.
    ///
    /// **Es reisen keine Bytes.** Der Eintrag nennt Mappe, Pfad und das Ziel
    /// (`target_type`, `target_uuid`); die Bytes gehoeren dem Ziel und kommen
    /// ueber dessen eigenen Eintrag. Genau deshalb laesst der `DeltaFeed` ein
    /// Bild, das an einer Notiz haengt, aus dem `media`-Strom weg: Regel 1 auf
    /// der Leitung ist "jede Datei genau einmal".
    ///
    /// BIS ZUM 16.09.2026 GAB ES DIESEN ZWEIG NICHT. Anhaenge fielen in den
    /// Sammelzweig fuer unbekannte Arten und wurden als "uebersprungen"
    /// gezaehlt -- bei Tiffys Konto 26 Stueck. In der App standen Bilder in
    /// Notizen als roher Markdown-Text, und der Bericht nannte eine Zahl, die
    /// niemand deuten konnte.
    fn anhang_ziehen(&self, eintrag: &Eintrag, bericht: &mut Bericht) -> Result<(), Lauffehler> {
        let Some(neu) = anhang_aus_eintrag(eintrag) else {
            bericht.ueberspringen("note attachment without target");
            return Ok(());
        };

        // Derselbe Stand noch einmal ist keine Aenderung -- sonst meldete
        // jeder Lauf alle Anhaenge als gezogen.
        if self.speicher.anhang(&neu.mappe, &neu.pfad)?.as_ref() == Some(&neu) {
            return Ok(());
        }

        self.speicher
            .anhang_schreiben(&neu, Protokoll::Von(self.basis()))?;

        bericht.gezogen += 1;

        Ok(())
    }

    fn bild_ziehen(&self, eintrag: &Eintrag, bericht: &mut Bericht) -> Result<(), Lauffehler> {
        let Some(neu) = bild_aus_eintrag(eintrag) else {
            bericht.ueberspringen("picture without required fields");
            return Ok(());
        };
        if let Some(da) = self.speicher.bild(&eintrag.key)? {
            if bild_gleich(&da, &neu) {
                return Ok(());
            }
        }
        self.speicher
            .bild_schreiben(&neu, Protokoll::Von(self.basis()))?;
        bericht.gezogen += 1;
        Ok(())
    }

    /// Ein Kalender ist ein Behaelter -- Name und Farbe, sonst nichts.
    ///
    /// **Kein Drei-Seiten-Vergleich**, sondern das letzte Schreiben gewinnt --
    /// dieselbe Wahl wie drueben. Ein Behaelter hat keinen Inhalt, den man
    /// verlieren koennte; was schlimmstenfalls verlorengeht, ist ein Name.
    fn kalender_ziehen(&self, eintrag: &Eintrag, bericht: &mut Bericht) -> Result<(), Lauffehler> {
        let da = self.speicher.kalender(&eintrag.key)?;

        let neu = Kalender {
            uuid: eintrag.key.clone(),
            name: eintrag.text("name").unwrap_or("Kalender").to_string(),
            farbe: eintrag.text("color").unwrap_or_default().to_string(),
            // DIE ABO-ADRESSE REIST MIT, der Inhalt des Abos nicht.
            //
            // Hier stand einmal das Gegenteil: Das Abo galt als Entscheidung
            // DIESES Geraets. Das hiess aber, dass man denselben Feed auf
            // jedem Geraet von Hand eintragen musste -- und dafuer gab es
            // keinen Grund. Die Termine sind es, die nicht reisen duerfen
            // (jedes Geraet holt sie selbst); die Adresse ist eine Angabe wie
            // Name und Farbe.
            //
            // ABWESEND HEISST "UNBEKANNT", `null` HEISST "KEIN ABO" --
            // dieselbe Regel wie bei den Kontaktwegen. Eine aeltere
            // Gegenstelle, die das Feld nicht kennt, soll kein Abo loeschen:
            // Sie wuerde es bei jedem Lauf wieder wegnehmen, und der Kalender
            // bliebe stehen und hoerte nur auf, sich zu fuellen.
            abo_url: match eintrag.hat("sync_url") {
                true => eintrag.text("sync_url").map(str::to_string),
                false => da.as_ref().and_then(|k| k.abo_url.clone()),
            },
            // Eine NEUE Adresse heisst: noch nie geholt. Sonst hielte der
            // Auffrischer die Stunde der alten fuer die der neuen und liesse
            // den frischen Feed eine Weile ungeholt.
            zuletzt_geholt: da.as_ref().and_then(|k| k.zuletzt_geholt.clone()),
            papierkorb_at: None,
            geaendert_at: eintrag.text("updated_at").unwrap_or_default().to_string(),
        };

        // Derselbe Stand noch einmal ist keine Aenderung -- sonst meldete
        // jeder Lauf alle Kalender als gezogen.
        let mut neu = neu;
        if da.as_ref().and_then(|k| k.abo_url.as_deref()) != neu.abo_url.as_deref() {
            neu.zuletzt_geholt = None;
        }

        if let Some(da) = &da {
            if da.name == neu.name
                && da.farbe == neu.farbe
                && da.abo_url == neu.abo_url
                && da.papierkorb_at.is_none()
            {
                return Ok(());
            }
        }

        self.speicher
            .kalender_schreiben(&neu, Protokoll::Von(self.basis()))?;

        bericht.gezogen += 1;

        Ok(())
    }

    /// Ein Kontakt -- ebenfalls das letzte Schreiben.
    ///
    /// Eine Konfliktkopie waere hier die schlechtere Haelfte: Zwei Zeilen
    /// derselben Person mit je einer halben Kennung sind schlimmer als eine
    /// mit der juengeren. Was schlimmstenfalls verlorengeht, ist eine
    /// Kennung, die sich in zwei Zeilen wieder eintippen laesst.
    ///
    /// **Abwesend heisst "unbekannt", leer heisst "keine"** -- dieselbe Regel
    /// wie drueben in `SyncApply::wege()`. Eine Gegenstelle ohne `channels`
    /// (ein alter Server) nennt nur `matrix_id`/`meshtastic_id`;
    /// dann werden NUR diese beiden Arten ersetzt und alle anderen Wege
    /// bleiben stehen. Ohne diese Unterscheidung loeschte der erste Abgleich
    /// mit einer alten Gegenstelle jede Telefonnummer.
    async fn kontakt_ziehen(
        &self,
        eintrag: &Eintrag,
        bericht: &mut Bericht,
    ) -> Result<(), Lauffehler> {
        let vorhanden = self.speicher.kontakt(&eintrag.key)?;

        let wege = wege_von(
            eintrag,
            vorhanden
                .as_ref()
                .map(|k| k.wege.clone())
                .unwrap_or_default(),
        );
        let anzeigename = eintrag
            .text("display_name")
            .unwrap_or("Kontakt")
            .to_string();

        let foto_vorher = vorhanden.as_ref().and_then(|k| k.foto.clone());
        let gleich = vorhanden.as_ref().is_some_and(|k| {
            k.anzeigename == anzeigename && k.wege == wege && k.papierkorb_at.is_none()
        });

        if !gleich {
            self.speicher.kontakt_schreiben(
                &Kontakt {
                    uuid: eintrag.key.clone(),
                    anzeigename,
                    wege,
                    foto: foto_vorher.clone(),
                    papierkorb_at: None,
                    geaendert_at: eintrag.text("updated_at").unwrap_or_default().to_string(),
                },
                Protokoll::Von(self.basis()),
            )?;
        }

        let foto_neu = self.kontaktfoto_ziehen(eintrag, foto_vorher).await?;

        if !gleich || foto_neu {
            bericht.gezogen += 1;
        }

        Ok(())
    }

    /// Das Foto, falls sich sein Abdruck geaendert hat.
    ///
    /// **Ein hier geaendertes Foto gewinnt**, solange es noch nicht hinueber
    /// ist: Gezogen wird vor dem Schieben, und sonst ueberschriebe das alte
    /// Bild von drueben das eben gewaehlte, bevor es je hinausging.
    ///
    /// Scheitert das Holen, bleibt der alte Abdruck stehen -- dann holt der
    /// naechste Lauf es erneut, statt ein Bild zu behaupten, das nicht da ist.
    async fn kontaktfoto_ziehen(
        &self,
        eintrag: &Eintrag,
        vorher: Option<String>,
    ) -> Result<bool, Lauffehler> {
        let Some(feld) = eintrag.felder.get("photo") else {
            return Ok(false); // Die Gegenstelle kennt keine Fotos.
        };
        if self.speicher.kontaktfoto_hinaus(&eintrag.key)?.is_some() {
            return Ok(false);
        }

        let drueben = feld.as_str().filter(|s| !s.is_empty());
        if drueben == vorher.as_deref() {
            return Ok(false);
        }

        match drueben {
            None => self.speicher.kontaktfoto_uebernehmen(
                &eintrag.key,
                None,
                None,
                Protokoll::Von(self.basis()),
            )?,
            Some(abdruck) => {
                let Ok(bytes) = self.gegenstelle.kontaktfoto(&eintrag.key).await else {
                    return Ok(false);
                };
                self.speicher.kontaktfoto_uebernehmen(
                    &eintrag.key,
                    Some(abdruck),
                    Some(&bytes),
                    Protokoll::Von(self.basis()),
                )?;
            }
        }

        Ok(true)
    }

    /// Der gemeinsame Stand -- aus dem eigenen Gedaechtnis und dem
    /// `base_hash` im Eintrag gewaehlt ([`ursprung_waehlen`]).
    ///
    /// **Ein Server schickt keinen `base_hash`, ein Geraet schon.** Hat das
    /// Tablet eine Notiz vom Telefon geholt und dann geaendert, kennt das
    /// Telefon keinen Ursprung dafuer -- es hat ja nur ausgeliefert. Tippt
    /// jetzt das Telefon auf "abgleichen", saehe es sonst zwei verschiedene
    /// Staende ohne gemeinsamen und legte eine Konfliktkopie an, obwohl nur
    /// eine Seite geschrieben hat.
    fn ursprung(
        &self,
        eintrag: &Eintrag,
        hier: Option<&str>,
        drueben: &str,
    ) -> Result<Option<String>, Lauffehler> {
        let eigener = self
            .speicher
            .ursprung(self.basis(), &eintrag.art, &eintrag.key)?;
        let genannt = eintrag.text("base_hash").map(str::to_string);

        Ok(ursprung_waehlen(&[eigener, genannt], hier, drueben))
    }

    // --- Schieben --------------------------------------------------------

    async fn schieben(&self, bericht: &mut Bericht) -> Result<(), Lauffehler> {
        loop {
            let marke = self.speicher.marke(self.basis())?.eigene;
            let (aenderungen, marke_danach, mehr) = self.speicher.aenderungen_seit(marke)?;

            let mut stapel = Vec::new();

            for aenderung in &aenderungen {
                // Was von dieser Gegenstelle kam, kennt sie schon.
                if aenderung.herkunft.as_deref() == Some(self.basis()) {
                    continue;
                }
                if ist_speicherart(&aenderung.art) && !self.gegenstelle.traegt_dateien() {
                    continue;
                }
                match hinausbringen(
                    self.speicher,
                    self.basis(),
                    &self.herkunft,
                    aenderung,
                    self.gegenstelle.fotos_mitschicken(),
                )? {
                    Some(eintrag) => stapel.push(eintrag),
                    None => bericht.ueberspringen(format!(
                        "outgoing: \"{}\" cannot be sent",
                        aenderung.art.ueber_die_leitung()
                    )),
                }
            }

            // Die Seite ist schon auf `SEITE` begrenzt und `SEITE` ist
            // dieselbe Zahl wie `MAX_EINTRAEGE` -- aber das ist ein Zufall
            // zweier Konstanten in zwei Repos. Hier wird trotzdem geteilt,
            // damit aus dem Zufall keine 422 wird, sobald eine der beiden
            // Zahlen sich aendert.
            for teil in stapel.chunks(MAX_EINTRAEGE) {
                let ergebnisse = self.gegenstelle.anwenden(teil).await?;
                // Was drueben schon so dastand, ist nicht geschoben worden --
                // sonst meldete der Lauf nach jedem Gegenbesuch Arbeit, die
                // keine war.
                bericht.geschoben += ergebnisse
                    .iter()
                    .filter(|e| !matches!(e.action.as_str(), "unchanged" | "gone"))
                    .count();

                for eintrag in teil {
                    self.ursprung_nach_dem_schieben(eintrag)?;
                    self.nachreichen_wenn_uebergangen(eintrag, &ergebnisse, bericht)?;
                }
            }

            self.speicher
                .eigene_marke_setzen(self.basis(), marke_danach)?;

            if !mehr {
                return Ok(());
            }

            if marke_danach <= marke {
                return Err(Lauffehler::TrittAufDerStelle { marke });
            }
        }
    }

    /// Ein Anhang, den die Gegenstelle uebergangen hat, kommt wieder vor.
    ///
    /// **Die Reihenfolge ist das Problem, nicht der Anhang.** Ein hier
    /// angelegter Anhang zeigt auf ein Bild, das hier auch eben erst
    /// entstand. Der Lauf schiebt die ZEILEN (Album, Bild, Anhang); die
    /// BYTES des Bildes gehen erst danach hinueber, in Stuecken. Der Server
    /// legt das Bild aber erst mit den Bytes an -- und einen Anhang, dessen
    /// Ziel fehlt, nimmt er nicht ("skipped"), sonst staende im Notiztext ein
    /// totes Bild.
    ///
    /// Also noch einmal vormerken: Die Zeile wird unveraendert neu ins
    /// Protokoll geschrieben und geht beim naechsten Schieben wieder hinaus.
    /// Wer die Bytes hinueberbringt, laesst danach einen zweiten Lauf folgen
    /// (`Bericht::offen`), und der bringt sie.
    fn nachreichen_wenn_uebergangen(
        &self,
        eintrag: &Eintrag,
        ergebnisse: &[openany_client::Ergebnis],
        bericht: &mut Bericht,
    ) -> Result<(), Lauffehler> {
        if eintrag.art != Art::Notizanhang || eintrag.was != Was::Da {
            return Ok(());
        }
        let uebergangen = ergebnisse
            .iter()
            .any(|e| e.key == eintrag.key && e.action == "skipped");
        if !uebergangen {
            return Ok(());
        }
        let Some((mappe, pfad)) = eintrag.key.split_once('|') else {
            return Ok(());
        };
        // Nur, wenn es den Anhang hier noch gibt -- sonst kaeme eine
        // Zuordnung, die schon weg ist, bei jedem Lauf wieder.
        if let Some(a) = self.speicher.anhang(mappe, pfad)? {
            self.speicher.anhang_schreiben(&a, Protokoll::Merken)?;
            bericht.offen += 1;
        }
        Ok(())
    }

    /// Nach dem Schieben ist der geschobene Stand der beidseitig bekannte.
    ///
    /// **Nur fuer die beiden Arten, die dreiseitig entschieden werden.** Ein
    /// Ursprung fuer einen Kalender waere eine Zahl, die niemand liest.
    fn ursprung_nach_dem_schieben(&self, eintrag: &Eintrag) -> Result<(), Lauffehler> {
        // Kein Ursprung, aber auch ein "ist drueben": Das Foto muss nicht noch
        // einmal hinaus.
        if eintrag.art == Art::Kontakt && eintrag.felder.contains_key("photo_bytes") {
            self.speicher.kontaktfoto_hinaus_erledigt(&eintrag.key)?;
        }

        match (&eintrag.art, eintrag.was) {
            (Art::Notiz | Art::Termin, Was::Papierkorb | Was::Fort) => self
                .speicher
                .ursprung_vergessen(self.basis(), &eintrag.art, &eintrag.key)?,
            (Art::Notiz | Art::Termin, Was::Da) => {
                // Der Abdruck steht schon im Eintrag -- ihn hier neu zu
                // rechnen waere dieselbe Rechnung zweimal, und die zweite
                // koennte anders ausfallen.
                if let Some(hash) = eintrag.text("hash") {
                    self.speicher.ursprung_merken(
                        self.basis(),
                        &eintrag.art,
                        &eintrag.key,
                        hash,
                    )?;
                }
            }
            _ => {}
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use openany_client::{Delta, Ergebnis};
    use openany_store::Weg;
    use serde_json::Value;
    use std::collections::{HashMap, VecDeque};
    use std::sync::Mutex;

    /// Eine Gegenstelle aus Pappe.
    ///
    /// Sie kann drei Dinge, die ein echter Server nur mit Muehe kann: eine
    /// bestimmte Seitenfolge liefern, mitten im Lauf abbrechen und auf der
    /// Stelle treten.
    struct Pappserver {
        basis: String,
        seiten: Mutex<VecDeque<Delta>>,
        texte: Mutex<HashMap<String, String>>,
        empfangen: Mutex<Vec<Eintrag>>,
        /// Wenn gesetzt, schlaegt das naechste `delta` fehl.
        reisst_ab: Mutex<bool>,
        /// Wenn gesetzt, nimmt er Anhaenge nicht an -- wie der echte Server,
        /// solange das Ziel drueben fehlt.
        uebergeht_anhaenge: Mutex<bool>,
    }

    impl Pappserver {
        fn neu() -> Self {
            Self {
                basis: "https://openany.de".into(),
                seiten: Mutex::new(VecDeque::new()),
                texte: Mutex::new(HashMap::new()),
                empfangen: Mutex::new(Vec::new()),
                reisst_ab: Mutex::new(false),
                uebergeht_anhaenge: Mutex::new(false),
            }
        }

        fn ohne_ziel_fuer_anhaenge(self) -> Self {
            *self.uebergeht_anhaenge.lock().unwrap() = true;
            self
        }

        fn seite(self, seite: Delta) -> Self {
            self.seiten.lock().unwrap().push_back(seite);
            self
        }

        fn text(self, zk_id: &str, inhalt: &str) -> Self {
            self.texte
                .lock()
                .unwrap()
                .insert(zk_id.into(), inhalt.into());
            self
        }

        fn geschoben(&self) -> Vec<Eintrag> {
            self.empfangen.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl Gegenstelle for Pappserver {
        fn basis(&self) -> &str {
            &self.basis
        }

        async fn delta(&self, _seit: i64) -> Result<Delta, OpenanyError> {
            if *self.reisst_ab.lock().unwrap() {
                return Err(OpenanyError::SchluesselUngueltig);
            }

            Ok(self.seiten.lock().unwrap().pop_front().unwrap_or(Delta {
                cursor: 0,
                more: false,
                eintraege: vec![],
                speicher: None,
            }))
        }

        async fn notiztext(&self, zk_id: &str) -> Result<String, OpenanyError> {
            self.texte
                .lock()
                .unwrap()
                .get(zk_id)
                .cloned()
                .ok_or(OpenanyError::SchluesselUngueltig)
        }

        async fn anwenden(&self, eintraege: &[Eintrag]) -> Result<Vec<Ergebnis>, OpenanyError> {
            self.empfangen.lock().unwrap().extend_from_slice(eintraege);
            let uebergeht = *self.uebergeht_anhaenge.lock().unwrap();

            Ok(eintraege
                .iter()
                .map(|e| Ergebnis {
                    art: e.art.ueber_die_leitung().to_string(),
                    key: e.key.clone(),
                    action: if uebergeht && e.art == Art::Notizanhang {
                        "skipped".into()
                    } else {
                        "updated".into()
                    },
                    conflict: false,
                })
                .collect())
        }
    }

    fn seite(cursor: i64, more: bool, eintraege: Vec<Eintrag>) -> Delta {
        Delta {
            cursor,
            more,
            eintraege,
            speicher: None,
        }
    }

    fn speicher() -> Speicher {
        Speicher::im_arbeitsspeicher().unwrap()
    }

    fn notiz_eintrag(zk_id: &str, titel: &str) -> Eintrag {
        Eintrag::neu(Art::Notiz, zk_id, Was::Da)
            .mit("title", titel)
            .mit("updated_at", "2026-09-06T10:00:00+02:00")
    }

    fn anhang_eintrag(mappe: &str, pfad: &str, ziel: &str) -> Eintrag {
        Eintrag::neu(Art::Notizanhang, format!("{mappe}|{pfad}"), Was::Da)
            .mit("folder", mappe)
            .mit("path", pfad)
            .mit("target_type", "media")
            .mit("target_uuid", ziel)
    }

    fn hiesiger_anhang(s: &Speicher) -> openany_store::Anhang {
        let a = openany_store::Anhang {
            mappe: "Reise".into(),
            pfad: "strand.jpg".into(),
            ziel_art: "media".into(),
            ziel_uuid: "bild-1".into(),
            groesse: 1234,
            abdruck: Some("abc".into()),
            vorschau: Some("def".into()),
        };
        s.anhang_schreiben(&a, Protokoll::Merken).unwrap();
        a
    }

    /// EIN HIER ANGELEGTER ANHANG GEHT HINAUS -- mit Ziel und beiden
    /// Abdruecken, in denselben Feldnamen, in denen er auch ankommt.
    ///
    /// Bis zum 17.09.2026 hatte `hinausbringen` keinen Zweig dafuer: Der
    /// Eintrag fiel unter "laesst sich nicht versenden", und ein Bild, das
    /// jemand in der App in eine Notiz setzte, blieb fuer immer nur dort.
    #[tokio::test]
    async fn ein_hiesiger_anhang_geht_mit_ziel_und_abdruecken_hinaus() {
        let s = speicher();
        hiesiger_anhang(&s);
        let server = Pappserver::neu();

        let bericht = Laeufer::neu(&s, &server, "Tablet").lauf().await;

        assert_eq!(bericht.geschoben, 1);
        assert_eq!(bericht.uebersprungen, 0);
        assert_eq!(bericht.offen, 0);
        let hinaus = server.geschoben();
        assert_eq!(hinaus.len(), 1);
        let e = &hinaus[0];
        assert_eq!(e.art, Art::Notizanhang);
        assert_eq!(e.key, "Reise|strand.jpg");
        assert_eq!(e.text("folder"), Some("Reise"));
        assert_eq!(e.text("path"), Some("strand.jpg"));
        assert_eq!(e.text("target_type"), Some("media"));
        assert_eq!(e.text("target_uuid"), Some("bild-1"));
        assert_eq!(e.text("hash"), Some("abc"));
        assert_eq!(e.text("thumb_hash"), Some("def"));
        assert_eq!(e.felder.get("size").and_then(Value::as_u64), Some(1234));
    }

    /// Ein Anhang, den die Gegenstelle uebergeht, weil sein Ziel dort noch
    /// fehlt, wird NACHGEREICHT: Er steht als offen im Bericht, und der
    /// naechste Lauf schiebt ihn wieder. Ohne das ginge er still verloren --
    /// die Marke ist schon weiter, niemand fragte je nach ihm.
    #[tokio::test]
    async fn ein_drueben_uebergangener_anhang_kommt_beim_naechsten_lauf_wieder() {
        let s = speicher();
        hiesiger_anhang(&s);
        let server = Pappserver::neu().ohne_ziel_fuer_anhaenge();

        let erster = Laeufer::neu(&s, &server, "Tablet").lauf().await;
        assert_eq!(erster.offen, 1, "der Anhang ist offen, nicht verloren");
        assert_eq!(server.geschoben().len(), 1);

        // Jetzt "sind die Bytes drueben": Der Server nimmt Anhaenge an.
        *server.uebergeht_anhaenge.lock().unwrap() = false;
        let zweiter = Laeufer::neu(&s, &server, "Tablet").lauf().await;

        assert_eq!(zweiter.offen, 0);
        assert_eq!(zweiter.geschoben, 1);
        assert_eq!(server.geschoben().len(), 2, "derselbe Anhang, noch einmal");

        // Und danach ist Ruhe: Ein dritter Lauf schiebt nichts mehr.
        let dritter = Laeufer::neu(&s, &server, "Tablet").lauf().await;
        assert_eq!(dritter.geschoben, 0);
        assert_eq!(server.geschoben().len(), 2);
    }

    /// Ein Anhang, den es hier nicht mehr gibt, wird nicht nachgereicht --
    /// sonst kaeme bei jedem Lauf eine Zuordnung wieder, die schon weg ist.
    #[tokio::test]
    async fn ein_hier_geloeschter_anhang_wird_nicht_nachgereicht() {
        let s = speicher();
        hiesiger_anhang(&s);
        s.anhang_entfernen("Reise", "strand.jpg", Protokoll::Still)
            .unwrap();
        let server = Pappserver::neu().ohne_ziel_fuer_anhaenge();

        let bericht = Laeufer::neu(&s, &server, "Tablet").lauf().await;

        // Was hinausgeht, ist ein Grabstein -- und der wird nicht "skipped".
        assert_eq!(bericht.offen, 0);
        assert_eq!(server.geschoben()[0].was, Was::Fort);
    }

    /// EIN ANHANG KOMMT AN, statt uebersprungen zu werden.
    ///
    /// Bis zum 16.09.2026 fiel `Art::Notizanhang` in den Sammelzweig fuer
    /// unbekannte Arten. Bei Tiffys Konto waren das 26 Eintraege -- Bilder in
    /// Notizen, die in der App als roher Markdown-Text standen, waehrend der
    /// Bericht eine Zahl bei "uebersprungen" nannte, die niemand deuten konnte.
    #[tokio::test]
    async fn ein_notiz_anhang_kommt_an() {
        let s = speicher();
        let server = Pappserver::neu().seite(seite(
            1,
            false,
            vec![anhang_eintrag("UniStuff/BWL", "bild.png", "ab16147a")],
        ));

        let bericht = Laeufer::neu(&s, &server, "Tablet").lauf().await;

        assert_eq!(bericht.gezogen, 1);
        assert_eq!(bericht.uebersprungen, 0, "er faellt nicht mehr durch");

        let anhang = s.anhang("UniStuff/BWL", "bild.png").unwrap().unwrap();
        assert_eq!(anhang.ziel_uuid, "ab16147a");
        assert_eq!(anhang.ziel_art, "media");
    }

    /// Ohne Ziel ist ein Anhang ein Verweis ins Leere -- drueben kann
    /// `target()` null sein, wenn das Bild geloescht wurde.
    #[tokio::test]
    async fn ein_anhang_ohne_ziel_wird_uebergangen() {
        let s = speicher();
        let ohne = Eintrag::neu(Art::Notizanhang, "Mappe|bild.png", Was::Da)
            .mit("folder", "Mappe")
            .mit("path", "bild.png")
            .mit("target_type", "media");
        let server = Pappserver::neu().seite(seite(1, false, vec![ohne]));

        let bericht = Laeufer::neu(&s, &server, "Tablet").lauf().await;

        assert_eq!(bericht.gezogen, 0);
        assert_eq!(bericht.uebersprungen, 1);
        assert_eq!(
            bericht.gruende.get("note attachment without target"),
            Some(&1)
        );
        assert!(s.anhang("Mappe", "bild.png").unwrap().is_none());
    }

    /// Ein zweiter Lauf meldet ihn nicht noch einmal.
    #[tokio::test]
    async fn derselbe_anhang_zweimal_ist_keine_aenderung() {
        let s = speicher();
        let bauen = || {
            Pappserver::neu().seite(seite(
                1,
                false,
                vec![anhang_eintrag("Mappe", "bild.png", "ziel")],
            ))
        };

        Laeufer::neu(&s, &bauen(), "Tablet").lauf().await;
        let zweiter = Laeufer::neu(&s, &bauen(), "Tablet").lauf().await;

        assert_eq!(zweiter.gezogen, 0);
        assert_eq!(zweiter.uebersprungen, 0);
    }

    /// Und ein Grabstein raeumt ihn weg -- `loeschen` braucht dafuer den
    /// zweiteiligen Schluessel.
    #[tokio::test]
    async fn ein_grabstein_raeumt_den_anhang_weg() {
        let s = speicher();
        Laeufer::neu(
            &s,
            &Pappserver::neu().seite(seite(
                1,
                false,
                vec![anhang_eintrag("Mappe", "bild.png", "ziel")],
            )),
            "Tablet",
        )
        .lauf()
        .await;

        let fort = Eintrag::neu(Art::Notizanhang, "Mappe|bild.png", Was::Fort);
        Laeufer::neu(
            &s,
            &Pappserver::neu().seite(seite(2, false, vec![fort])),
            "Tablet",
        )
        .lauf()
        .await;

        assert!(s.anhang("Mappe", "bild.png").unwrap().is_none());
    }

    fn termin_eintrag(uuid: &str, kalender: &str, titel: &str) -> Eintrag {
        let mut e = Eintrag::neu(Art::Termin, uuid, Was::Da);
        e.felder = Terminfelder {
            titel: titel.into(),
            beginn: "2026-09-10T09:00:00".into(),
            ende: "2026-09-10T09:30:00".into(),
            ..Default::default()
        }
        .als_eintrag();
        e.mit("calendar", kalender)
    }

    // --- Ziehen ----------------------------------------------------------

    #[tokio::test]
    async fn ein_erstabgleich_holt_kalender_termin_und_notiz() {
        let s = speicher();
        let server = Pappserver::neu()
            .text("20260906100000", "Milch\nBrot")
            .seite(seite(
                17,
                false,
                vec![
                    Eintrag::neu(Art::Kalender, "k-1", Was::Da)
                        .mit("name", "Privat")
                        .mit("color", "#005F60"),
                    termin_eintrag("t-1", "k-1", "Zahnarzt"),
                    notiz_eintrag("20260906100000", "Einkauf"),
                    Eintrag::neu(Art::Kontakt, "c-1", Was::Da).mit("display_name", "Hannah"),
                ],
            ));

        let bericht = Laeufer::neu(&s, &server, "Hannahs Telefon").lauf().await;

        assert!(bericht.durchgelaufen(), "{:?}", bericht.fehler);
        assert_eq!(bericht.gezogen, 4);
        assert_eq!(s.kalender("k-1").unwrap().unwrap().name, "Privat");
        assert_eq!(s.termin("t-1").unwrap().unwrap().felder.titel, "Zahnarzt");
        assert_eq!(
            s.notiz("20260906100000").unwrap().unwrap().inhalt,
            "Milch\nBrot"
        );
        assert_eq!(s.kontakt("c-1").unwrap().unwrap().anzeigename, "Hannah");
        assert_eq!(s.marke(server.basis()).unwrap().fremde, 17);
    }

    // --- Die Abo-Adresse ---------------------------------------------

    /// Eine Adresse, die drueben eingetragen wurde, kommt hier an -- und ab
    /// da holt DIESES Geraet den Feed selbst.
    #[tokio::test]
    async fn eine_abo_adresse_kommt_an() {
        let s = speicher();
        let server = Pappserver::neu().seite(seite(
            1,
            false,
            vec![Eintrag::neu(Art::Kalender, "k-1", Was::Da)
                .mit("name", "Familie")
                .mit("sync_url", "https://api.familywall.example/feed.ics")],
        ));

        Laeufer::neu(&s, &server, "Tablet").lauf().await;

        let k = s.kalender("k-1").unwrap().unwrap();
        assert_eq!(
            k.abo_url.as_deref(),
            Some("https://api.familywall.example/feed.ics")
        );
        assert!(k.ist_abo());
    }

    /// ABWESEND HEISST "UNBEKANNT". Eine aeltere Gegenstelle, die das Feld
    /// nicht kennt, darf kein Abo loeschen -- sie taete es bei JEDEM Lauf,
    /// und der Kalender bliebe stehen und hoerte nur auf, sich zu fuellen.
    #[tokio::test]
    async fn ein_eintrag_ohne_das_feld_laesst_das_abo_stehen() {
        let s = speicher();
        s.kalender_schreiben(
            &Kalender {
                uuid: "k-1".into(),
                name: "Familie".into(),
                farbe: "#005F60".into(),
                abo_url: Some("https://api.familywall.example/feed.ics".into()),
                zuletzt_geholt: Some("2026-09-17T08:00:00".into()),
                papierkorb_at: None,
                geaendert_at: String::new(),
            },
            Protokoll::Still,
        )
        .unwrap();

        let server = Pappserver::neu().seite(seite(
            1,
            false,
            vec![Eintrag::neu(Art::Kalender, "k-1", Was::Da).mit("name", "Familie, neu benannt")],
        ));

        Laeufer::neu(&s, &server, "Tablet").lauf().await;

        let k = s.kalender("k-1").unwrap().unwrap();
        assert_eq!(k.name, "Familie, neu benannt");
        assert!(
            k.abo_url.is_some(),
            "das Abo wurde stillschweigend entfernt"
        );
        // Und die Stunde bleibt: Es ist dasselbe Abo.
        assert!(k.zuletzt_geholt.is_some());
    }

    /// `null` dagegen heisst ausdruecklich "kein Abo mehr".
    #[tokio::test]
    async fn ein_ausdrueckliches_null_gibt_das_abo_auf() {
        let s = speicher();
        s.kalender_schreiben(
            &Kalender {
                uuid: "k-1".into(),
                name: "Familie".into(),
                farbe: "#005F60".into(),
                abo_url: Some("https://api.familywall.example/feed.ics".into()),
                zuletzt_geholt: Some("2026-09-17T08:00:00".into()),
                papierkorb_at: None,
                geaendert_at: String::new(),
            },
            Protokoll::Still,
        )
        .unwrap();

        let server = Pappserver::neu().seite(seite(
            1,
            false,
            vec![Eintrag::neu(Art::Kalender, "k-1", Was::Da)
                .mit("name", "Familie")
                .mit("sync_url", serde_json::Value::Null)],
        ));

        Laeufer::neu(&s, &server, "Tablet").lauf().await;

        let k = s.kalender("k-1").unwrap().unwrap();
        assert!(k.abo_url.is_none());
        assert!(!k.ist_abo());
    }

    /// Eine NEUE Adresse heisst: noch nie geholt. Sonst hielte der
    /// Auffrischer die Stunde der alten fuer die der neuen und liesse den
    /// frischen Feed eine Weile ungeholt.
    #[tokio::test]
    async fn eine_neue_adresse_setzt_die_stunde_zurueck() {
        let s = speicher();
        s.kalender_schreiben(
            &Kalender {
                uuid: "k-1".into(),
                name: "Familie".into(),
                farbe: "#005F60".into(),
                abo_url: Some("https://alt.example/feed.ics".into()),
                zuletzt_geholt: Some("2026-09-17T08:00:00".into()),
                papierkorb_at: None,
                geaendert_at: String::new(),
            },
            Protokoll::Still,
        )
        .unwrap();

        let server = Pappserver::neu().seite(seite(
            1,
            false,
            vec![Eintrag::neu(Art::Kalender, "k-1", Was::Da)
                .mit("name", "Familie")
                .mit("sync_url", "https://neu.example/feed.ics")],
        ));

        Laeufer::neu(&s, &server, "Tablet").lauf().await;

        let k = s.kalender("k-1").unwrap().unwrap();
        assert_eq!(k.abo_url.as_deref(), Some("https://neu.example/feed.ics"));
        assert!(k.zuletzt_geholt.is_none(), "die alte Stunde blieb stehen");
    }

    /// Und andersherum: Was hier eingetragen wurde, geht hinaus.
    #[tokio::test]
    async fn die_eigene_abo_adresse_geht_hinaus() {
        let s = speicher();
        s.kalender_schreiben(
            &Kalender {
                uuid: "k-1".into(),
                name: "Familie".into(),
                farbe: "#005F60".into(),
                abo_url: Some("https://api.familywall.example/feed.ics".into()),
                zuletzt_geholt: None,
                papierkorb_at: None,
                geaendert_at: String::new(),
            },
            Protokoll::Merken,
        )
        .unwrap();

        let server = Pappserver::neu().seite(seite(0, false, vec![]));
        Laeufer::neu(&s, &server, "Tablet").lauf().await;

        let hinaus = server.empfangen.lock().unwrap().clone();
        let kalender = hinaus
            .iter()
            .find(|e| e.art == Art::Kalender)
            .expect("der Kalender ging gar nicht hinaus");
        assert_eq!(
            kalender.text("sync_url"),
            Some("https://api.familywall.example/feed.ics")
        );
    }

    #[tokio::test]
    async fn die_marke_rueckt_seitenweise_vor() {
        let s = speicher();
        let server = Pappserver::neu()
            .seite(seite(
                10,
                true,
                vec![Eintrag::neu(Art::Kalender, "k-1", Was::Da).mit("name", "Erste Seite")],
            ))
            .seite(seite(
                20,
                false,
                vec![Eintrag::neu(Art::Kalender, "k-2", Was::Da).mit("name", "Zweite Seite")],
            ));

        let bericht = Laeufer::neu(&s, &server, "Telefon").lauf().await;

        assert_eq!(bericht.gezogen, 2);
        assert_eq!(s.marke(server.basis()).unwrap().fremde, 20);
    }

    #[tokio::test]
    async fn eine_gegenstelle_die_auf_der_stelle_tritt_wird_abgebrochen() {
        // Ohne diese Pruefung liefe der Laeufer ewig -- auf einem Telefon
        // faellt das als leerer Akku auf, nicht als Serverlast.
        let s = speicher();
        let server = Pappserver::neu()
            .seite(seite(5, true, vec![]))
            .seite(seite(5, true, vec![]))
            .seite(seite(5, true, vec![]));

        let bericht = Laeufer::neu(&s, &server, "Telefon").lauf().await;

        assert!(!bericht.durchgelaufen());
        assert!(
            bericht.fehler[0].contains("not making progress"),
            "{:?}",
            bericht.fehler
        );
    }

    #[tokio::test]
    async fn eine_unbekannte_art_wird_uebergangen_und_die_marke_rueckt_trotzdem() {
        let s = speicher();
        let server = Pappserver::neu().seite(seite(
            9,
            false,
            vec![
                Eintrag::neu(Art::Unbekannt("rezept".into()), "r-1", Was::Da),
                Eintrag::neu(Art::Album, "a-1", Was::Da),
                Eintrag::neu(Art::Kalender, "k-1", Was::Da).mit("name", "Privat"),
            ],
        ));

        let bericht = Laeufer::neu(&s, &server, "Telefon").lauf().await;

        assert_eq!(bericht.gezogen, 1);
        assert_eq!(bericht.uebersprungen, 2);
        // Und JEDER der beiden mit Namen -- eine "2" allein sagte am
        // 16.09.2026 niemandem, ob sie harmlos ist.
        assert_eq!(
            bericht
                .gruende
                .get("unknown kind \"rezept\" (newer counterpart)"),
            Some(&1)
        );
        // Der Pappserver traegt keine Dateien -- das Album faellt schon an
        // der Vereinbarung, nicht erst an den Feldern.
        assert_eq!(
            bericht
                .gruende
                .get("files and pictures: not agreed with this counterpart"),
            Some(&1)
        );
        assert_eq!(
            bericht.gruende.values().sum::<usize>(),
            bericht.uebersprungen
        );
        assert_eq!(
            s.marke(server.basis()).unwrap().fremde,
            9,
            "sonst haenge der Abgleich fuer immer an dieser Seite"
        );
    }

    #[tokio::test]
    async fn papierkorb_und_loeschen_sind_nicht_dasselbe() {
        let s = speicher();
        let server = Pappserver::neu()
            .text("zk-1", "bleibt")
            .text("zk-2", "geht")
            .seite(seite(
                1,
                true,
                vec![notiz_eintrag("zk-1", "Eins"), notiz_eintrag("zk-2", "Zwei")],
            ))
            .seite(seite(
                2,
                false,
                vec![
                    Eintrag::neu(Art::Notiz, "zk-1", Was::Papierkorb),
                    Eintrag::neu(Art::Notiz, "zk-2", Was::Fort),
                ],
            ));

        Laeufer::neu(&s, &server, "Telefon").lauf().await;

        assert!(
            s.notiz("zk-1").unwrap().unwrap().im_papierkorb(),
            "im Papierkorb, also zurueckholbar"
        );
        assert!(s.notiz("zk-2").unwrap().is_none(), "endgueltig fort");
    }

    #[tokio::test]
    async fn was_gezogen_wurde_wird_nicht_zurueckgeschoben() {
        // Der Pingpong-Test. Ohne `Protokoll::Still` beim Anwenden schoebe
        // jeder Lauf alles zurueck, was er gerade geholt hat.
        let s = speicher();
        let server = Pappserver::neu().text("zk-1", "Milch").seite(seite(
            7,
            false,
            vec![
                Eintrag::neu(Art::Kalender, "k-1", Was::Da).mit("name", "Privat"),
                termin_eintrag("t-1", "k-1", "Zahnarzt"),
                notiz_eintrag("zk-1", "Einkauf"),
            ],
        ));

        let bericht = Laeufer::neu(&s, &server, "Telefon").lauf().await;

        assert_eq!(bericht.gezogen, 3);
        assert_eq!(bericht.geschoben, 0);
        assert!(server.geschoben().is_empty());
    }

    #[tokio::test]
    async fn derselbe_stand_zweimal_gezogen_aendert_nichts() {
        let s = speicher();
        let eintraege = vec![
            termin_eintrag("t-1", "k-1", "Zahnarzt"),
            notiz_eintrag("zk-1", "Einkauf"),
        ];
        let server = Pappserver::neu()
            .text("zk-1", "Milch")
            .seite(seite(1, false, eintraege.clone()))
            .seite(seite(2, false, eintraege));

        Laeufer::neu(&s, &server, "Telefon").lauf().await;
        let zweiter = Laeufer::neu(&s, &server, "Telefon").lauf().await;

        assert_eq!(zweiter.konflikte, 0, "ein zweiter Lauf ist kein Streitfall");
        assert_eq!(s.notizen().unwrap().len(), 1);
        assert!(server.geschoben().is_empty(), "und schiebt nichts zurueck");
    }

    #[tokio::test]
    async fn beidseitig_geaendert_ergibt_eine_kopie_und_laesst_das_original_stehen() {
        let s = speicher();
        let server = Pappserver::neu()
            .text("zk-1", "Der Stand vom Server")
            .seite(seite(1, false, vec![notiz_eintrag("zk-1", "Einkauf")]))
            .seite(seite(2, false, vec![notiz_eintrag("zk-1", "Einkauf")]));

        // Erster Lauf: beide Seiten einig.
        Laeufer::neu(&s, &server, "Telefon").lauf().await;

        // Hier wird geaendert ...
        s.notiz_schreiben(
            &Notiz {
                zk_id: "zk-1".into(),
                titel: "Einkauf".into(),
                inhalt: "Mein eigener Stand".into(),
                ..Default::default()
            },
            Protokoll::Merken,
        )
        .unwrap();

        // ... und drueben auch.
        server
            .texte
            .lock()
            .unwrap()
            .insert("zk-1".into(), "Der geaenderte Stand vom Server".into());

        let bericht = Laeufer::neu(&s, &server, "Telefon").lauf().await;

        assert_eq!(bericht.konflikte, 1);
        assert_eq!(
            s.notiz("zk-1").unwrap().unwrap().inhalt,
            "Mein eigener Stand",
            "das Original bleibt unangetastet"
        );

        let kopien: Vec<_> = s
            .notizen()
            .unwrap()
            .into_iter()
            .filter(|n| n.titel.contains("Konflikt"))
            .collect();

        assert_eq!(kopien.len(), 1);
        assert_eq!(kopien[0].inhalt, "Der geaenderte Stand vom Server");
        assert!(
            kopien[0].titel.contains("openany.de"),
            "die Kopie sagt, woher sie kommt: {}",
            kopien[0].titel
        );
    }

    #[tokio::test]
    async fn nur_hier_geaendert_behaelt_den_hiesigen_stand() {
        let s = speicher();
        let server = Pappserver::neu()
            .text("zk-1", "Gemeinsamer Stand")
            .seite(seite(1, false, vec![notiz_eintrag("zk-1", "Einkauf")]))
            .seite(seite(2, false, vec![notiz_eintrag("zk-1", "Einkauf")]));

        Laeufer::neu(&s, &server, "Telefon").lauf().await;

        s.notiz_schreiben(
            &Notiz {
                zk_id: "zk-1".into(),
                titel: "Einkauf".into(),
                inhalt: "Nur hier geaendert".into(),
                ..Default::default()
            },
            Protokoll::Merken,
        )
        .unwrap();

        // Drueben unveraendert -- das Fremde ist schlicht aelter.
        let bericht = Laeufer::neu(&s, &server, "Telefon").lauf().await;

        assert_eq!(bericht.konflikte, 0, "kein Streit, nur ein alter Ausdruck");
        assert_eq!(
            s.notiz("zk-1").unwrap().unwrap().inhalt,
            "Nur hier geaendert"
        );
        assert_eq!(s.notizen().unwrap().len(), 1, "keine Kopie");
    }

    #[tokio::test]
    async fn ein_termin_darf_vor_seinem_kalender_ankommen() {
        // Hier weicht das Programm bewusst vom Server ab -- siehe den
        // Kommentar in `termin_ziehen`.
        let s = speicher();
        let server = Pappserver::neu().seite(seite(
            1,
            false,
            vec![termin_eintrag("t-1", "kommt-noch", "Zahnarzt")],
        ));

        let bericht = Laeufer::neu(&s, &server, "Telefon").lauf().await;

        assert_eq!(bericht.gezogen, 1);
        assert_eq!(
            s.termin("t-1").unwrap().unwrap().kalender_uuid,
            "kommt-noch",
            "mit seiner echten Zuordnung, nicht geraten"
        );
    }

    #[tokio::test]
    async fn der_speicherstand_landet_im_bericht() {
        let s = speicher();
        let mut voll = seite(1, false, vec![]);
        voll.speicher = Some(Speicherstand {
            belegt: 1_024_500,
            grenze: Some(10_485_760),
        });
        let server = Pappserver::neu().seite(voll);

        let bericht = Laeufer::neu(&s, &server, "Telefon").lauf().await;

        let stand = bericht.speicher.unwrap();

        assert_eq!(stand.belegt, 1_024_500);
        assert!(
            stand.passt(1_000),
            "damit kann gewarnt werden, bevor es klemmt"
        );
    }

    // --- Schieben --------------------------------------------------------

    #[tokio::test]
    async fn eine_hiesige_notiz_geht_mit_allem_hinaus_was_drueben_gebraucht_wird() {
        let s = speicher();
        let server = Pappserver::neu();

        s.notiz_schreiben(
            &Notiz {
                zk_id: "20260906120000".into(),
                titel: "Einkauf".into(),
                inhalt: "Milch".into(),
                mappe: Some("Haushalt".into()),
                ..Default::default()
            },
            Protokoll::Merken,
        )
        .unwrap();

        let bericht = Laeufer::neu(&s, &server, "Hannahs Telefon").lauf().await;

        assert_eq!(bericht.geschoben, 1);

        let hinaus = server.geschoben();
        let e = &hinaus[0];

        assert_eq!(e.was, Was::Da);
        assert_eq!(e.text("content"), Some("Milch"), "der Text reist mit");
        assert_eq!(e.text("folder"), Some("Haushalt"));
        assert_eq!(e.text("origin"), Some("Hannahs Telefon"));
        assert_eq!(
            e.felder.get("base_hash"),
            Some(&Value::Null),
            "es gab noch nie einen gemeinsamen Stand -- null, nicht \"\""
        );
        assert_eq!(e.text("hash"), Some(notizabdruck("Milch").as_str()));
    }

    #[tokio::test]
    async fn nach_dem_schieben_ist_der_stand_beidseitig_bekannt() {
        let s = speicher();
        let server = Pappserver::neu();

        s.notiz_schreiben(
            &Notiz {
                zk_id: "zk-1".into(),
                inhalt: "Milch".into(),
                ..Default::default()
            },
            Protokoll::Merken,
        )
        .unwrap();

        Laeufer::neu(&s, &server, "Telefon").lauf().await;

        assert_eq!(
            s.ursprung(server.basis(), &Art::Notiz, "zk-1").unwrap(),
            Some(notizabdruck("Milch")),
            "sonst waere derselbe Stand beim naechsten Lauf ein Konflikt"
        );

        // Und ein zweiter Lauf schiebt nichts mehr.
        let zweiter = Laeufer::neu(&s, &server, "Telefon").lauf().await;

        assert_eq!(zweiter.geschoben, 0);
    }

    #[tokio::test]
    async fn der_zustand_kommt_aus_der_sache_und_nicht_aus_dem_protokoll() {
        // Die Notiz wurde angelegt (`Da`) und dann geloescht. Das Protokoll
        // traegt beide Zeilen; hinaus geht der ZUSTAND -- also `delete`.
        let s = speicher();
        let server = Pappserver::neu();

        s.notiz_schreiben(&Notiz::mit_id("zk-1"), Protokoll::Merken)
            .unwrap();
        s.loeschen(&Art::Notiz, "zk-1", Protokoll::Merken).unwrap();

        Laeufer::neu(&s, &server, "Telefon").lauf().await;

        let hinaus = server.geschoben();

        assert_eq!(hinaus.len(), 1, "zwei Protokollzeilen, ein Eintrag");
        assert_eq!(hinaus[0].was, Was::Fort);
    }

    #[tokio::test]
    async fn eine_hiesige_notiz_im_papierkorb_geht_als_papierkorb_hinaus() {
        let s = speicher();
        let server = Pappserver::neu();

        s.notiz_schreiben(&Notiz::mit_id("zk-1"), Protokoll::Merken)
            .unwrap();
        s.papierkorb(&Art::Notiz, "zk-1", Protokoll::Merken)
            .unwrap();

        Laeufer::neu(&s, &server, "Telefon").lauf().await;

        let hinaus = server.geschoben();

        assert_eq!(hinaus[0].was, Was::Papierkorb);
        assert!(
            hinaus[0].text("deleted_at").is_some(),
            "damit drueben derselbe Papierkorb entsteht und keine Vernichtung"
        );
    }

    #[tokio::test]
    async fn ein_hiesiger_termin_geht_mit_seinen_neun_feldern_und_dem_abdruck_hinaus() {
        let s = speicher();
        let server = Pappserver::neu();

        s.termin_schreiben(
            &Termin {
                uuid: "t-1".into(),
                kalender_uuid: "k-1".into(),
                felder: Terminfelder {
                    titel: "Zahnarzt".into(),
                    beginn: "2026-09-10T09:00:00".into(),
                    ende: "2026-09-10T09:30:00".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            Protokoll::Merken,
        )
        .unwrap();

        Laeufer::neu(&s, &server, "Telefon").lauf().await;

        let hinaus = server.geschoben();
        let e = &hinaus[0];

        assert_eq!(e.text("title"), Some("Zahnarzt"));
        assert_eq!(e.text("start"), Some("2026-09-10T09:00:00"));
        assert_eq!(e.felder.get("all_day"), Some(&Value::Bool(false)));
        assert_eq!(e.text("calendar"), Some("k-1"));
        assert_eq!(
            e.text("hash"),
            Some("8a25a4ca136e584ecf499150247f7ae606621c87ad4d2d01f9d4636082c969a9"),
            "derselbe Abdruck, den PHP rechnet"
        );
    }

    #[tokio::test]
    async fn ein_fehler_beim_ziehen_haelt_das_schieben_an() {
        // Wer auf einem veralteten Stand schiebt, erzeugt Konfliktkopien
        // drueben -- dort, wo niemand sitzt, der sie aufloest.
        let s = speicher();
        let server = Pappserver::neu();
        *server.reisst_ab.lock().unwrap() = true;

        s.notiz_schreiben(&Notiz::mit_id("zk-1"), Protokoll::Merken)
            .unwrap();

        let bericht = Laeufer::neu(&s, &server, "Telefon").lauf().await;

        assert!(!bericht.durchgelaufen());
        assert_eq!(bericht.geschoben, 0);
        assert!(server.geschoben().is_empty());
        assert!(
            s.marke(server.basis()).unwrap().letzter_fehler.is_some(),
            "und der Grund bleibt lesbar stehen"
        );
    }

    #[tokio::test]
    async fn ein_kontakt_geht_mit_seinen_wegen_hinaus() {
        let s = speicher();
        let server = Pappserver::neu();

        s.kontakt_schreiben(
            &Kontakt {
                uuid: "c-1".into(),
                anzeigename: "Hannah".into(),
                wege: vec![
                    Weg {
                        art: "phone".into(),
                        beschriftung: Some("mobil".into()),
                        wert: "0171".into(),
                    },
                    Weg {
                        art: "matrix".into(),
                        beschriftung: None,
                        wert: "@hannah:matrix.org".into(),
                    },
                ],
                ..Default::default()
            },
            Protokoll::Merken,
        )
        .unwrap();

        Laeufer::neu(&s, &server, "Telefon").lauf().await;

        let hinaus = server.geschoben();

        assert_eq!(hinaus[0].text("display_name"), Some("Hannah"));
        assert_eq!(
            hinaus[0].felder.get("channels"),
            Some(&serde_json::json!([
                { "kind": "phone", "label": "mobil", "value": "0171" },
                { "kind": "matrix", "label": null, "value": "@hannah:matrix.org" }
            ]))
        );
        // Fuer einen Server vor den Wegen: die beiden alten Felder, abgeleitet.
        assert_eq!(hinaus[0].text("matrix_id"), Some("@hannah:matrix.org"));
        assert_eq!(
            hinaus[0].felder.get("meshtastic_id"),
            Some(&Value::Null),
            "keine Kennung heisst null -- nicht eine leere"
        );
        assert!(
            !hinaus[0].felder.contains_key("photo_bytes"),
            "ohne hier geaendertes Foto laesst der Server das Bild in Ruhe"
        );
    }

    #[tokio::test]
    async fn wege_von_drueben_kommen_vollstaendig_an() {
        let s = speicher();
        let server = Pappserver::neu().seite(seite(
            1,
            false,
            vec![Eintrag::neu(Art::Kontakt, "c-1", Was::Da)
                .mit("display_name", "Hannah")
                .mit(
                    "channels",
                    serde_json::json!([
                        { "kind": "address", "label": "privat", "value": "Weg 7" },
                        { "kind": "phone", "label": null, "value": "030" },
                        { "kind": "phone", "label": null, "value": "  " }
                    ]),
                )],
        ));

        Laeufer::neu(&s, &server, "Telefon").lauf().await;

        let k = s.kontakt("c-1").unwrap().unwrap();
        let werte: Vec<_> = k.wege.iter().map(|w| w.wert.as_str()).collect();
        assert_eq!(
            werte,
            vec!["Weg 7", "030"],
            "leere Zeilen fallen weg, die Reihenfolge bleibt"
        );
    }

    #[tokio::test]
    async fn eine_alte_gegenstelle_ersetzt_nur_ihre_beiden_kennungen() {
        let s = speicher();
        s.kontakt_schreiben(
            &Kontakt {
                uuid: "c-1".into(),
                anzeigename: "Hannah".into(),
                wege: vec![
                    Weg {
                        art: "phone".into(),
                        beschriftung: None,
                        wert: "0171".into(),
                    },
                    Weg {
                        art: "matrix".into(),
                        beschriftung: None,
                        wert: "@alt:m.org".into(),
                    },
                    Weg {
                        art: "meshtastic".into(),
                        beschriftung: None,
                        wert: "!1".into(),
                    },
                ],
                ..Default::default()
            },
            Protokoll::Still,
        )
        .unwrap();

        // Ohne `channels`: ein Server vor den Wegen.
        let server = Pappserver::neu().seite(seite(
            1,
            false,
            vec![Eintrag::neu(Art::Kontakt, "c-1", Was::Da)
                .mit("display_name", "Hannah")
                .mit("matrix_id", "@neu:m.org")
                .mit("meshtastic_id", Value::Null)],
        ));

        Laeufer::neu(&s, &server, "Telefon").lauf().await;

        let k = s.kontakt("c-1").unwrap().unwrap();
        let wege: Vec<_> = k
            .wege
            .iter()
            .map(|w| (w.art.as_str(), w.wert.as_str()))
            .collect();
        assert_eq!(
            wege,
            vec![("phone", "0171"), ("matrix", "@neu:m.org")],
            "die Telefonnummer bleibt, Matrix ersetzt, Meshtastic genannt-und-leer geloescht"
        );
    }

    #[tokio::test]
    async fn ein_hier_gesetztes_foto_geht_einmal_hinaus() {
        let s = speicher();
        let server = Pappserver::neu();
        s.kontakt_schreiben(
            &Kontakt {
                uuid: "c-1".into(),
                anzeigename: "Hannah".into(),
                ..Default::default()
            },
            Protokoll::Still,
        )
        .unwrap();
        s.kontaktfoto_setzen("c-1", b"bild").unwrap();

        Laeufer::neu(&s, &server, "Telefon").lauf().await;
        assert_eq!(server.geschoben()[0].text("photo_bytes"), Some("YmlsZA=="));

        // Beim zweiten Lauf nicht noch einmal.
        s.kontakt_schreiben(&s.kontakt("c-1").unwrap().unwrap(), Protokoll::Merken)
            .unwrap();
        Laeufer::neu(&s, &server, "Telefon").lauf().await;
        assert!(!server.geschoben()[1].felder.contains_key("photo_bytes"));
    }
}
