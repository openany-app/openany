//! Der Laeufer fuer den Strom EINES PROJEKTS.
//!
//! Dieselbe Reihenfolge wie beim grossen [`crate::Laeufer`] -- erst ziehen,
//! dann schieben, die Marke seitenweise -- und aus demselben Grund: Wer auf
//! einem veralteten Stand schiebt, erzeugt Konflikte drueben, wo niemand
//! sitzt, der sie aufloest.
//!
//! **Warum ein eigener Laeufer und nicht derselbe.** Der grosse kennt jede
//! Art beim Namen: Er holt Notiztexte, Kontaktfotos und Dateiinhalte, rechnet
//! Abdruecke und entscheidet dreiseitig. Ein Projektstrom traegt nichts
//! davon -- keine Bytes, keine Texte ausser den Feldern selbst, und (bis auf
//! Weiteres) letztes Schreiben statt Drei-Seiten-Vergleich. Ihn durch den
//! grossen zu schleusen hiesse, dort ueberall `if projekt` zu schreiben; das
//! waere genau die Sorte Ausnahme, an der zwei Wege auseinanderlaufen.
//!
//! **Die Arten kennt hier niemand.** Was `board`, `column` oder `card`
//! ausmacht, steht in der Karte des Servers; hier reisen die Felder als JSON
//! und der Eintrag sagt selbst, woran er haengt (`parent`, `parent_type`).
//! Ein elfter Planungstyp kostet deshalb in dieser Datei nichts.

use crate::gegenstelle::Gegenstelle;
use openany_client::{Art, Delta, Eintrag, OpenanyError, Was};
use openany_store::{Projektsache, Protokoll, Speicher, SpeicherFehler};
use serde_json::Value;

/// Was ein Projektlauf getan hat.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Projektbericht {
    pub gezogen: usize,
    pub geschoben: usize,
    pub uebersprungen: usize,
    pub fehler: Vec<String>,
}

impl Projektbericht {
    pub fn durchgelaufen(&self) -> bool {
        self.fehler.is_empty()
    }
}

/// Was der Laeufer von einem Projektstrom braucht -- zwei Methoden.
///
/// `delta` und `anwenden` von [`Gegenstelle`] wuerden reichen; der Trait
/// verlangt aber auch `notiztext`, und eine Gegenstelle, die das nicht kann,
/// muesste luegen. Zwei Methoden, die es wirklich gibt, sind ehrlicher als
/// fuenf, von denen drei abwinken.
#[async_trait::async_trait]
pub trait Projektgegenstelle: Send + Sync {
    /// Der Name des Stroms -- `<basis>#projekt:<uuid>`. Marken haengen daran.
    fn basis(&self) -> &str;

    /// Die uuid des Projekts, dem dieser Strom gehoert.
    fn projekt(&self) -> &str;

    async fn delta(&self, seit: i64) -> Result<Delta, OpenanyError>;

    async fn anwenden(
        &self,
        eintraege: &[Eintrag],
    ) -> Result<Vec<openany_client::Ergebnis>, OpenanyError>;
}

/// Ein vollstaendiger Lauf gegen einen Projektstrom.
pub async fn projekt_lauf<G: Projektgegenstelle + ?Sized>(
    speicher: &Speicher,
    gegenstelle: &G,
) -> Projektbericht {
    let mut bericht = Projektbericht::default();

    if let Err(e) = ziehen(speicher, gegenstelle, &mut bericht).await {
        bericht.fehler.push(e.to_string());
    } else if let Err(e) = schieben(speicher, gegenstelle, &mut bericht).await {
        bericht.fehler.push(e.to_string());
    } else if bericht.geschoben > 0 {
        /*
         * WAS DRUEBEN AUS DEM GESCHOBENEN WURDE, GLEICH HOLEN.
         *
         * Eine Stimme aendert drueben die Zaehler aller Optionen, und die
         * kennt nur der Server. Ohne diesen zweiten Zug stuende nach dem
         * Abstimmen die eigene Antwort neben den alten Zahlen -- bis zum
         * naechsten Abgleich. Gezaehlt wird er nicht: Was hier ankommt, ist
         * das Echo des eben Geschobenen, kein Neues von anderen.
         */
        let mut echo = Projektbericht::default();

        if let Err(e) = ziehen(speicher, gegenstelle, &mut echo).await {
            bericht.fehler.push(e.to_string());
        }
    }

    let _ = speicher.lauf_beendet(
        gegenstelle.basis(),
        bericht.fehler.first().map(String::as_str),
    );

    bericht
}

async fn ziehen<G: Projektgegenstelle + ?Sized>(
    speicher: &Speicher,
    gegenstelle: &G,
    bericht: &mut Projektbericht,
) -> Result<(), Lauffehler> {
    let mut marke = speicher.marke(gegenstelle.basis())?.fremde;

    /*
     * WAS HIER NOCH NICHT HINAUS IST, UEBERSCHREIBT DAS ZIEHEN NICHT.
     *
     * Der Lauf zieht zuerst und schiebt dann. Ohne diese Menge ersetzte das
     * Ziehen eine eben verschobene Karte durch den alten Stand von drueben;
     * die neue Zeile im Protokoll truege dann die Herkunft der Gegenstelle,
     * und das Schieben liesse sie als "kennt sie schon" liegen. Die
     * Verschiebung waere spurlos fort. Am 22.09.2026 auf dem Tablet so
     * gesehen: Karte verschoben, Abgleichen getippt, Karte wieder zurueck.
     *
     * Das ist "letztes Schreiben gewinnt", ehrlich genommen: Die hiesige
     * Aenderung ist juenger als der Stand, den dieses Geraet von drueben
     * kannte. Sie geht gleich danach hinaus, und drueben gewinnt, was
     * zuletzt ankommt. Ein Drei-Seiten-Vergleich wie beim grossen Laeufer
     * bleibt eine spaetere Stufe.
     */
    let eigene = speicher.marke(gegenstelle.basis())?.eigene;
    let offene = speicher.projekt_offene(gegenstelle.projekt(), eigene, gegenstelle.basis())?;

    loop {
        let seite = gegenstelle.delta(marke).await?;

        for eintrag in &seite.eintraege {
            if offene.contains(&eintrag.key) {
                continue;
            }

            einziehen(speicher, gegenstelle, eintrag, bericht)?;
        }

        speicher.fremde_marke_setzen(gegenstelle.basis(), seite.cursor)?;

        if !seite.more {
            return Ok(());
        }

        // Dieselbe Sicherung wie im grossen Laeufer: Eine Gegenstelle, die
        // "es gibt mehr" sagt und die Marke nicht vorrueckt, liefe sonst
        // ewig -- auf einem Telefon als leerer Akku, den niemand erklaeren
        // kann.
        if seite.cursor <= marke {
            return Err(Lauffehler::TrittAufDerStelle { marke });
        }

        marke = seite.cursor;
    }
}

fn einziehen<G: Projektgegenstelle + ?Sized>(
    speicher: &Speicher,
    gegenstelle: &G,
    eintrag: &Eintrag,
    bericht: &mut Projektbericht,
) -> Result<(), Lauffehler> {
    let projekt = gegenstelle.projekt();
    let art = eintrag.art.ueber_die_leitung().to_string();

    // Das Projekt beschreibt sich selbst: Name ja, Mitgliedschaft nein --
    // die steht im persoenlichen Strom, und nur dort.
    if art == "project" {
        if eintrag.was == Was::Da {
            if let Some(mut p) = speicher.projekt(projekt)? {
                p.name = eintrag.text("name").unwrap_or(&p.name).to_string();
                p.geaendert_at = eintrag.text("updated_at").unwrap_or_default().to_string();
                speicher.projekt_schreiben(&p)?;
                bericht.gezogen += 1;

                return Ok(());
            }
        }

        bericht.uebersprungen += 1;

        return Ok(());
    }

    if eintrag.was != Was::Da {
        // `Papierkorb` und `Fort` sind hier dasselbe: Was das Programm nicht
        // zeigen soll, ist fort. Einen zweiten Papierkorb fuer Karten haelt
        // drueben niemand.
        if speicher.projektsache_entfernen(
            projekt,
            &eintrag.key,
            Protokoll::Von(gegenstelle.basis()),
        )? {
            bericht.gezogen += 1;
        } else {
            bericht.uebersprungen += 1;
        }

        return Ok(());
    }

    let neu = Projektsache {
        projekt: projekt.to_string(),
        art,
        uuid: eintrag.key.clone(),
        eltern: eintrag.text("parent").map(str::to_string),
        felder: felder_von(eintrag),
        geaendert_at: eintrag.text("updated_at").unwrap_or_default().to_string(),
    };

    // Derselbe Stand noch einmal ist keine Aenderung -- sonst meldete jeder
    // Lauf alle Karten als gezogen.
    if speicher.projektsache(projekt, &eintrag.key)?.as_ref() == Some(&neu) {
        return Ok(());
    }

    speicher.projektsache_schreiben(&neu, Protokoll::Von(gegenstelle.basis()))?;
    bericht.gezogen += 1;

    Ok(())
}

/// Die Felder der Art -- ohne das, was der Strom selbst ausmacht.
///
/// `type`, `key` und `action` stehen im Eintrag, `parent` und `parent_type`
/// in eigenen Spalten. Sie noch einmal in die Felder zu legen hiesse, zwei
/// Wahrheiten ueber dasselbe zu halten.
fn felder_von(eintrag: &Eintrag) -> Value {
    let mut felder = serde_json::Map::new();

    for (name, wert) in &eintrag.felder {
        if !matches!(name.as_str(), "parent" | "parent_type" | "updated_at") {
            felder.insert(name.clone(), wert.clone());
        }
    }

    Value::Object(felder)
}

async fn schieben<G: Projektgegenstelle + ?Sized>(
    speicher: &Speicher,
    gegenstelle: &G,
    bericht: &mut Projektbericht,
) -> Result<(), Lauffehler> {
    let projekt = gegenstelle.projekt();

    loop {
        let marke = speicher.marke(gegenstelle.basis())?.eigene;
        let (aenderungen, marke_danach, mehr) =
            speicher.projekt_aenderungen_seit(projekt, marke)?;

        let mut stapel = Vec::new();

        for aenderung in &aenderungen {
            // Was von dieser Gegenstelle kam, kennt sie schon.
            if aenderung.herkunft.as_deref() == Some(gegenstelle.basis()) {
                continue;
            }

            match hinaus(speicher, projekt, aenderung)? {
                Some(eintrag) => stapel.push(eintrag),
                None => bericht.uebersprungen += 1,
            }
        }

        for teil in stapel.chunks(openany_client::MAX_EINTRAEGE) {
            let ergebnisse = gegenstelle.anwenden(teil).await?;

            bericht.geschoben += ergebnisse
                .iter()
                .filter(|e| {
                    !matches!(
                        e.action.as_str(),
                        "unchanged" | "gone" | "skipped" | "rejected" | "ignored"
                    )
                })
                .count();

            // Abgelehnt heisst: drueben fehlt ein Pflichtfeld, oder die Art
            // ist unbekannt. Das kommt nicht von allein wieder -- zaehlen,
            // nicht erneut vormerken.
            bericht.uebersprungen += ergebnisse
                .iter()
                .filter(|e| matches!(e.action.as_str(), "rejected" | "ignored"))
                .count();

            // Was drueben uebergangen wurde, weil sein Elternteil noch fehlt,
            // kommt beim naechsten Lauf wieder -- dafuer wird es erneut
            // vorgemerkt. Dieselbe Lehre wie beim Notiz-Anhang (#224).
            for (eintrag, ergebnis) in teil.iter().zip(ergebnisse.iter()) {
                if ergebnis.action == "skipped" {
                    if let Some(sache) = speicher.projektsache(projekt, &eintrag.key)? {
                        speicher.projektsache_schreiben(&sache, Protokoll::Merken)?;
                    }
                }
            }
        }

        speicher.eigene_marke_setzen(gegenstelle.basis(), marke_danach)?;

        if !mehr {
            return Ok(());
        }

        if marke_danach <= marke {
            return Err(Lauffehler::TrittAufDerStelle { marke });
        }
    }
}

/// Einen eigenen Protokolleintrag versandfertig machen.
///
/// **Der Zustand kommt aus der Sache, nicht aus dem Protokoll** -- wie beim
/// grossen Laeufer. Das Protokoll sagt, DASS sich etwas geaendert hat; wie es
/// jetzt dasteht, weiss nur die Sache selbst.
fn hinaus(
    speicher: &Speicher,
    projekt: &str,
    aenderung: &openany_store::Projektaenderung,
) -> Result<Option<Eintrag>, SpeicherFehler> {
    let art = Art::aus(&aenderung.art);

    let Some(sache) = speicher.projektsache(projekt, &aenderung.schluessel)? else {
        return Ok(Some(Eintrag::neu(
            art,
            aenderung.schluessel.clone(),
            Was::Fort,
        )));
    };

    let mut eintrag = Eintrag::neu(art, sache.uuid.clone(), Was::Da);

    if let Value::Object(felder) = &sache.felder {
        for (name, wert) in felder {
            eintrag.felder.insert(name.clone(), wert.clone());
        }
    }

    if let Some(eltern) = &sache.eltern {
        eintrag = eintrag.mit("parent", eltern.clone());
    }

    Ok(Some(eintrag))
}

#[derive(Debug, thiserror::Error)]
enum Lauffehler {
    #[error(transparent)]
    Gegenstelle(#[from] OpenanyError),
    #[error(transparent)]
    Speicher(#[from] SpeicherFehler),
    #[error("The counterpart is not making progress (cursor stays at {marke}).")]
    TrittAufDerStelle { marke: i64 },
}

/// Damit `Gegenstelle` als Import nicht ungenutzt dasteht: Ein Projektstrom
/// ist bewusst KEINE `Gegenstelle` (siehe Kopf dieser Datei).
#[allow(dead_code)]
fn _kein_gegenstellen_trait<G: Gegenstelle>(_: &G) {}

#[cfg(test)]
mod tests {
    use super::*;
    use openany_client::Ergebnis;
    use std::collections::VecDeque;
    use std::sync::Mutex;

    const BASIS: &str = "https://openany.de#projekt:p1";

    /// Ein Projektstrom aus Pappe -- Seiten hinein, Empfangenes heraus.
    struct Pappstrom {
        seiten: Mutex<VecDeque<Delta>>,
        empfangen: Mutex<Vec<Eintrag>>,
        /// Diese Schluessel weist er ab, weil ihr Elternteil fehlt.
        uebergeht: Mutex<Vec<String>>,
    }

    impl Pappstrom {
        fn neu() -> Self {
            Self {
                seiten: Mutex::new(VecDeque::new()),
                empfangen: Mutex::new(Vec::new()),
                uebergeht: Mutex::new(Vec::new()),
            }
        }

        fn seite(self, cursor: i64, more: bool, eintraege: Vec<Eintrag>) -> Self {
            self.seiten.lock().unwrap().push_back(Delta {
                cursor,
                more,
                eintraege,
                speicher: None,
            });
            self
        }

        fn uebergeht(self, key: &str) -> Self {
            self.uebergeht.lock().unwrap().push(key.into());
            self
        }

        fn geschoben(&self) -> Vec<Eintrag> {
            self.empfangen.lock().unwrap().clone()
        }
    }

    #[async_trait::async_trait]
    impl Projektgegenstelle for Pappstrom {
        fn basis(&self) -> &str {
            BASIS
        }

        fn projekt(&self) -> &str {
            "p1"
        }

        async fn delta(&self, _seit: i64) -> Result<Delta, OpenanyError> {
            Ok(self.seiten.lock().unwrap().pop_front().unwrap_or(Delta {
                cursor: 0,
                more: false,
                eintraege: vec![],
                speicher: None,
            }))
        }

        async fn anwenden(&self, eintraege: &[Eintrag]) -> Result<Vec<Ergebnis>, OpenanyError> {
            self.empfangen.lock().unwrap().extend_from_slice(eintraege);
            let uebergeht = self.uebergeht.lock().unwrap().clone();

            Ok(eintraege
                .iter()
                .map(|e| Ergebnis {
                    art: e.art.ueber_die_leitung().to_string(),
                    key: e.key.clone(),
                    action: if uebergeht.contains(&e.key) {
                        "skipped".into()
                    } else {
                        "updated".into()
                    },
                    conflict: false,
                })
                .collect())
        }
    }

    fn speicher() -> Speicher {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        s.projekt_schreiben(&openany_store::Projekt {
            uuid: "p1".into(),
            name: "Haushalt".into(),
            rolle: "owner".into(),
            geaendert_at: String::new(),
            mitgliederliste: None,
        })
        .unwrap();
        s
    }

    fn karte(uuid: &str, spalte: &str, titel: &str) -> Eintrag {
        Eintrag::neu(Art::aus("card"), uuid, Was::Da)
            .mit("title", titel)
            .mit("position", 1u64)
            .mit("parent", spalte)
            .mit("parent_type", "column")
            .mit("updated_at", "2026-09-22T10:00:00Z")
    }

    /// EINE KARTE KOMMT AN -- mit ihren Feldern und an ihrer Spalte.
    #[tokio::test]
    async fn eine_karte_kommt_an() {
        let s = speicher();
        let strom = Pappstrom::neu().seite(7, false, vec![karte("k1", "s1", "Milch")]);

        let bericht = projekt_lauf(&s, &strom).await;

        assert_eq!(bericht.gezogen, 1);
        let sache = s.projektsache("p1", "k1").unwrap().unwrap();
        assert_eq!(sache.art, "card");
        assert_eq!(sache.eltern.as_deref(), Some("s1"));
        assert_eq!(sache.text("title"), "Milch");
        assert_eq!(s.marke(BASIS).unwrap().fremde, 7);
    }

    /// `parent` und `updated_at` stehen in eigenen Spalten -- nicht zweimal.
    #[tokio::test]
    async fn der_strom_wird_nicht_doppelt_gefuehrt() {
        let s = speicher();
        let strom = Pappstrom::neu().seite(1, false, vec![karte("k1", "s1", "Milch")]);

        projekt_lauf(&s, &strom).await;

        let felder = s.projektsache("p1", "k1").unwrap().unwrap().felder;
        assert!(felder.get("parent").is_none());
        assert!(felder.get("parent_type").is_none());
        assert!(felder.get("updated_at").is_none());
        assert_eq!(felder.get("title").and_then(|w| w.as_str()), Some("Milch"));
    }

    /// Derselbe Stand zweimal ist keine Aenderung.
    #[tokio::test]
    async fn derselbe_stand_zweimal_zaehlt_einmal() {
        let s = speicher();
        let strom = Pappstrom::neu()
            .seite(1, false, vec![karte("k1", "s1", "Milch")])
            .seite(2, false, vec![karte("k1", "s1", "Milch")]);

        assert_eq!(projekt_lauf(&s, &strom).await.gezogen, 1);
        assert_eq!(projekt_lauf(&s, &strom).await.gezogen, 0);
    }

    /// Ein Grabstein raeumt weg -- und der Behaelter nimmt seinen Inhalt mit.
    #[tokio::test]
    async fn ein_grabstein_raeumt_den_teilbaum_weg() {
        let s = speicher();
        let strom = Pappstrom::neu()
            .seite(
                1,
                false,
                vec![
                    Eintrag::neu(Art::aus("board"), "b1", Was::Da).mit("name", "Einkauf"),
                    Eintrag::neu(Art::aus("column"), "s1", Was::Da)
                        .mit("name", "Offen")
                        .mit("parent", "b1"),
                    karte("k1", "s1", "Milch"),
                ],
            )
            .seite(
                2,
                false,
                vec![Eintrag::neu(Art::aus("board"), "b1", Was::Fort)],
            );

        projekt_lauf(&s, &strom).await;
        projekt_lauf(&s, &strom).await;

        assert!(s.projektsache("p1", "b1").unwrap().is_none());
        assert!(
            s.projektsache("p1", "k1").unwrap().is_none(),
            "die Karte auch"
        );
    }

    /// Das Projekt benennt sich selbst -- die Rolle bleibt, wie sie war.
    #[tokio::test]
    async fn das_projekt_benennt_sich() {
        let s = speicher();
        let strom = Pappstrom::neu().seite(
            1,
            false,
            vec![Eintrag::neu(Art::aus("project"), "p1", Was::Da).mit("name", "Wohnung")],
        );

        projekt_lauf(&s, &strom).await;

        let p = s.projekt("p1").unwrap().unwrap();
        assert_eq!(p.name, "Wohnung");
        assert_eq!(p.rolle, "owner", "die Rolle steht im persoenlichen Strom");
    }

    /// Was hier entsteht, geht hinaus -- mit `parent` als eigenem Feld.
    #[tokio::test]
    async fn hiesiges_geht_hinaus() {
        let s = speicher();
        s.projektsache_schreiben(
            &Projektsache {
                projekt: "p1".into(),
                art: "card".into(),
                uuid: "k9".into(),
                eltern: Some("s1".into()),
                felder: serde_json::json!({ "title": "Brot", "position": 3 }),
                geaendert_at: String::new(),
            },
            Protokoll::Merken,
        )
        .unwrap();

        let strom = Pappstrom::neu();
        let bericht = projekt_lauf(&s, &strom).await;

        assert_eq!(bericht.geschoben, 1);
        let hinaus = strom.geschoben();
        assert_eq!(hinaus.len(), 1);
        assert_eq!(hinaus[0].art.ueber_die_leitung(), "card");
        assert_eq!(hinaus[0].text("title"), Some("Brot"));
        assert_eq!(hinaus[0].text("parent"), Some("s1"));
    }

    /// Was von DIESER Gegenstelle kam, geht nicht zu ihr zurueck.
    #[tokio::test]
    async fn gezogenes_geht_nicht_zurueck() {
        let s = speicher();
        let strom = Pappstrom::neu().seite(1, false, vec![karte("k1", "s1", "Milch")]);

        projekt_lauf(&s, &strom).await;

        assert!(strom.geschoben().is_empty());
    }

    /// Drueben uebergangen, weil die Spalte fehlt: kommt beim naechsten Lauf.
    #[tokio::test]
    async fn uebergangenes_kommt_wieder() {
        let s = speicher();
        s.projektsache_schreiben(
            &Projektsache {
                projekt: "p1".into(),
                art: "card".into(),
                uuid: "k9".into(),
                eltern: Some("s-fehlt".into()),
                felder: serde_json::json!({ "title": "Brot" }),
                geaendert_at: String::new(),
            },
            Protokoll::Merken,
        )
        .unwrap();

        let strom = Pappstrom::neu().uebergeht("k9");
        let erster = projekt_lauf(&s, &strom).await;
        assert_eq!(
            erster.geschoben, 0,
            "uebergangen zaehlt nicht als geschoben"
        );

        let zweiter = projekt_lauf(&s, &strom).await;
        assert_eq!(strom.geschoben().len(), 2, "derselbe Eintrag, noch einmal");
        assert_eq!(zweiter.geschoben, 0);
    }

    /// HIESIGES UEBERLEBT DAS ZIEHEN -- die Probe vom 22.09.2026.
    ///
    /// Karte auf dem Geraet in eine andere Spalte, dann Abgleichen: Drueben
    /// steht sie noch in der alten, und dieser Stand kommt zuerst herein. Er
    /// darf die Verschiebung nicht ueberschreiben, und sie muss hinaus.
    #[tokio::test]
    async fn eine_verschiebung_ueberlebt_das_ziehen() {
        let s = speicher();
        let erst = Pappstrom::neu().seite(1, false, vec![karte("k1", "s1", "Salz")]);
        projekt_lauf(&s, &erst).await;

        let mut karte_hier = s.projektsache("p1", "k1").unwrap().unwrap();
        karte_hier.eltern = Some("s2".into());
        s.projektsache_schreiben(&karte_hier, Protokoll::Merken)
            .unwrap();

        // Drueben unveraendert: derselbe alte Stand noch einmal.
        let dann = Pappstrom::neu().seite(1, false, vec![karte("k1", "s1", "Salz")]);
        let bericht = projekt_lauf(&s, &dann).await;

        assert_eq!(
            s.projektsache("p1", "k1")
                .unwrap()
                .unwrap()
                .eltern
                .as_deref(),
            Some("s2"),
            "die Verschiebung bleibt"
        );
        let hinaus = dann.geschoben();
        assert_eq!(hinaus.len(), 1, "und geht hinaus");
        assert_eq!(hinaus[0].text("parent"), Some("s2"));
        assert_eq!(bericht.geschoben, 1);
    }

    /// NACH DEM SCHIEBEN KOMMT GLEICH, WAS DRUEBEN DARAUS WURDE -- im selben
    /// Lauf. Bei einer Stimme sind das die neuen Zaehler; ohne diesen Zug
    /// stuende die eigene Antwort bis zum naechsten Abgleich neben alten
    /// Zahlen.
    #[tokio::test]
    async fn das_echo_kommt_im_selben_lauf() {
        let s = speicher();
        s.projektsache_schreiben(
            &Projektsache {
                projekt: "p1".into(),
                art: "poll_option".into(),
                uuid: "o1".into(),
                eltern: Some("u1".into()),
                felder: serde_json::json!({ "label": "Blau", "count": 0, "my_answer": "yes" }),
                geaendert_at: String::new(),
            },
            Protokoll::Merken,
        )
        .unwrap();

        let strom = Pappstrom::neu().seite(3, false, vec![]).seite(
            4,
            false,
            vec![Eintrag::neu(Art::aus("poll_option"), "o1", Was::Da)
                .mit("label", "Blau")
                .mit("count", 1u64)
                .mit("my_answer", "yes")
                .mit("parent", "u1")],
        );

        let bericht = projekt_lauf(&s, &strom).await;

        assert_eq!(bericht.geschoben, 1);
        assert_eq!(bericht.gezogen, 0, "das Echo zaehlt nicht als Neues");
        assert_eq!(
            s.projektsache("p1", "o1").unwrap().unwrap().zahl("count"),
            1,
            "die neuen Zaehler sind schon da"
        );
        assert_eq!(s.marke(BASIS).unwrap().fremde, 4);
    }

    /// Ist sie einmal hinaus, gilt wieder, was von drueben kommt.
    #[tokio::test]
    async fn nach_dem_schieben_zieht_das_naechste_wieder() {
        let s = speicher();
        let erst = Pappstrom::neu().seite(1, false, vec![karte("k1", "s1", "Salz")]);
        projekt_lauf(&s, &erst).await;

        let mut karte_hier = s.projektsache("p1", "k1").unwrap().unwrap();
        karte_hier.eltern = Some("s2".into());
        s.projektsache_schreiben(&karte_hier, Protokoll::Merken)
            .unwrap();
        projekt_lauf(&s, &Pappstrom::neu()).await;

        // Jemand in der Webapp legt sie danach in die dritte Spalte.
        let spaeter = Pappstrom::neu().seite(2, false, vec![karte("k1", "s3", "Salz")]);
        projekt_lauf(&s, &spaeter).await;

        assert_eq!(
            s.projektsache("p1", "k1")
                .unwrap()
                .unwrap()
                .eltern
                .as_deref(),
            Some("s3")
        );
        assert!(spaeter.geschoben().is_empty(), "nichts geht doppelt hinaus");
    }

    /// Eine Gegenstelle, die nicht vorankommt, wird abgebrochen.
    #[tokio::test]
    async fn eine_gegenstelle_auf_der_stelle_wird_abgebrochen() {
        let s = speicher();
        let strom = Pappstrom::neu()
            .seite(0, true, vec![])
            .seite(0, true, vec![]);

        let bericht = projekt_lauf(&s, &strom).await;

        assert!(!bericht.durchgelaufen());
        assert!(
            bericht.fehler[0].contains("not making progress"),
            "{:?}",
            bericht.fehler
        );
    }
}
