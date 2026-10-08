//! Einen abonnierten Kalender auffrischen -- holen, lesen, einsortieren.
//!
//! # Wofuer das gut ist
//!
//! Drueben kann der Server einen Feed holen, weil er immer laeuft und immer
//! im Netz steht. Wer keinen openany-Zugang hat, hat keinen Server -- und
//! saehe den Termin, den jemand in FamilyWall eintraegt, nie. Deshalb holt
//! das Geraet selbst.
//!
//! # Die fremde Quelle gewinnt
//!
//! Ein Abo ist eine Kopie, kein zweiter Schreibplatz. Bei jedem Abruf
//! entscheidet die Datei, was im Kalender steht: Was fehlt, wird geloescht;
//! was neu ist, kommt dazu. Genau deshalb ist ein solcher Kalender in der
//! Oberflaeche schreibgeschuetzt -- ein Termin, den man hier eintraegt, waere
//! beim naechsten Abruf spurlos fort. Ohne Fehler, ohne Meldung.
//!
//! # Warum das GEHOLTE das Geraet nicht verlaesst -- die Adresse aber schon
//!
//! Die Termine aus dem Feed werden mit [`Protokoll::Still`] geschrieben. Sie
//! gehen NICHT in den Abgleich.
//!
//! Der Grund ist nicht Sparsamkeit. Die Quelle ist der Feed, und an den kommt
//! jedes Geraet selbst heran. Wer die Kopie stattdessen weiterreichte, haette
//! zwei Kopien, die verschieden alt sind -- und ein Geraet, das gerade nicht
//! auffrischen konnte, schriebe seinen veralteten Stand ueber den frischen
//! des anderen.
//!
//! Die ADRESSE dagegen reist mit, wie Name und Farbe. Hier stand einmal das
//! Gegenteil, mit derselben Begruendung -- aber sie gilt dem Inhalt, und ich
//! hatte sie auf den Behaelter mitangewendet. Das hiess, denselben Feed auf
//! jedem Geraet von Hand einzutragen. Am 17.09.2026 gefragt und geaendert.
//!
//! `zuletzt_geholt` bleibt still: Wann DIESES Geraet zuletzt geholt hat, geht
//! kein anderes etwas an -- und ein Abo, das reihum als "gerade erst geholt"
//! gilt, wird von niemandem mehr geholt.

use std::collections::HashSet;

use chrono::Local;
use openany_client::Art;
use openany_store::{Kalender, Protokoll, Speicher, Termin};

use crate::ics;

/// Groesser als das nimmt dieses Geraet nicht an.
///
/// Ein Jahreskalender einer Familie liegt bei einigen hundert Kilobyte. Fuenf
/// Megabyte sind reichlich Luft -- und eine Grenze, die verhindert, dass eine
/// Adresse, die auf etwas ganz anderes zeigt, den Speicher des Telefons
/// fuellt, bevor jemand es merkt.
const HOECHSTENS: usize = 5 * 1024 * 1024;

/// Wie viele Umleitungen mitgegangen werden.
const UMLEITUNGEN: usize = 5;

/// Was ein Auffrischen ergeben hat.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Abobericht {
    pub neu: usize,
    pub geaendert: usize,
    pub entfernt: usize,
    /// Unveraendert -- gelesen, aber nicht angefasst. Das ist der Normalfall
    /// und steht hier, damit "0 neu, 0 geaendert" nicht wie ein Fehlschlag
    /// aussieht.
    pub unveraendert: usize,
    /// Angaben, die nicht gelesen werden konnten (siehe [`ics::Gelesen`]).
    pub uebersprungen: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum Abofehler {
    #[error("This address points into the local network. A calendar feed lives on the internet.")]
    ImEigenenNetz,

    #[error("A feed address must start with https://.")]
    NichtHttps,

    #[error("This is not a valid address.")]
    KeineAdresse,

    #[error("The source does not respond: {0}")]
    Unerreichbar(String),

    #[error("The source responds with {0}.")]
    Abgelehnt(u16),

    #[error("The source sends too many redirects.")]
    ZuVieleUmleitungen,

    #[error("The file is larger than {}\u{a0}MB.", HOECHSTENS / 1024 / 1024)]
    ZuGross,

    /// Kein Fehler der Leitung, sondern des Inhalts: Was ankam, ist keine
    /// ICS-Datei. Meistens eine Anmeldeseite -- die Adresse ist abgelaufen
    /// oder war nie eine Feed-Adresse.
    #[error("There is no calendar at this address.")]
    KeinKalender,

    /// Die Datei ist in Ordnung, aber leer -- und hier stehen Termine.
    ///
    /// GEFUNDEN AM 17.09.2026 an einer echten Adresse: FamilyWall antwortet
    /// auf einen Schluessel, den es nicht (mehr) annimmt, nicht mit einem
    /// Fehler, sondern mit einem gueltigen, leeren Kalender --
    /// `PRODID:Invalid`, 90 Byte, Status 200. Zwei von drei Adressen taten
    /// das in diesem Augenblick.
    ///
    /// Wer das fuer einen Kalender haelt, aus dem alles abgesagt wurde,
    /// loescht beim naechsten Abruf alle Termine. Still, mit 200 OK, ohne
    /// dass irgendwo etwas schieflief.
    ///
    /// Ein Abo, das der Mensch WIRKLICH geleert hat, laeuft hier ebenfalls
    /// auf: Dann sagt er es einmal und loescht den Kalender selbst. Das ist
    /// die Seite, auf der man sich irren moechte.
    #[error("The source currently delivers not a single event. There are {0} here -- they stay. Usually the feed address has expired.")]
    QuelleLeer(usize),

    #[error("{0}")]
    Speicher(String),
}

impl From<openany_store::SpeicherFehler> for Abofehler {
    fn from(f: openany_store::SpeicherFehler) -> Self {
        Abofehler::Speicher(f.to_string())
    }
}

/// Eine Feed-Adresse, bevor das Geraet sie anfasst.
///
/// WARUM DAS SEIN MUSS: Ein Telefon steht in einem Heimnetz. Drueben prueft
/// `OutboundUrl::istOeffentlich`, dass eine Adresse nicht ins eigene
/// Rechenzentrum zeigt; hier ist die Nachbarschaft der Router, der Drucker
/// und die Fritzbox. Eine Adresse, die jemand eintraegt, darf dort nicht
/// anklopfen -- ein Feed waere sonst ein bequemer Weg, ein fremdes Heimnetz
/// abzutasten.
///
/// Nur https: Eine Feed-Adresse ist ein Passwort in Adressform, und ueber
/// http laege sie offen im selben WLAN.
///
/// WAS DAS NICHT LEISTET: Ein Name wird erst beim Verbinden aufgeloest, und
/// ein Name kann auf eine Adresse im Heimnetz zeigen. Dagegen hilft nur ein
/// eigener Aufloeser, der jede Antwort prueft. Das ist hier nicht gebaut --
/// die Huerde liegt also bei "wer einen Namen dafuer einrichtet", nicht bei
/// "wer eine Zahl eintippt". Fuer einen Feed, den der Mensch selbst von Hand
/// eintraegt, ist das der vertretbare Stand.
pub fn adresse_pruefen(url: &str) -> Result<(), Abofehler> {
    let adresse = url::Url::parse(url.trim()).map_err(|_| Abofehler::KeineAdresse)?;

    if adresse.scheme() != "https" {
        return Err(Abofehler::NichtHttps);
    }

    let wirt = adresse.host().ok_or(Abofehler::KeineAdresse)?;

    let privat = match wirt {
        url::Host::Ipv4(ip) => {
            ip.is_private() || ip.is_loopback() || ip.is_link_local() || ip.is_unspecified()
        }
        // fc00::/7 ist der private Bereich in IPv6, fe80::/10 das lokale Netz.
        url::Host::Ipv6(ip) => {
            let erstes = ip.segments()[0];
            ip.is_loopback()
                || ip.is_unspecified()
                || erstes & 0xfe00 == 0xfc00
                || erstes & 0xffc0 == 0xfe80
        }
        url::Host::Domain(d) => {
            let d = d.to_ascii_lowercase();
            d == "localhost" || d.ends_with(".local") || d.ends_with(".localhost")
        }
    };

    if privat {
        return Err(Abofehler::ImEigenenNetz);
    }

    Ok(())
}

/// Woher der Text kommt.
///
/// Als Trait, damit die Tests hier ohne Netz auskommen -- und weil ein
/// Abgleich, der zum Pruefen ins Internet muss, kein Test ist, sondern eine
/// Wettervorhersage.
#[async_trait::async_trait]
pub trait Feedquelle: Send + Sync {
    async fn holen(&self, url: &str) -> Result<String, Abofehler>;
}

/// Die echte Quelle: https, mit gepruefter Umleitung und Groessengrenze.
pub struct UeberDasNetz {
    klient: reqwest::Client,
}

impl UeberDasNetz {
    pub fn neu() -> Result<Self, Abofehler> {
        let klient = reqwest::Client::builder()
            // Umleitungen werden HIER von Hand gegangen und nicht von reqwest.
            //
            // WARUM: Eine oeffentliche Adresse, die auf 192.168.1.1 umleitet,
            // ist der uebliche Weg, eine Pruefung wie `adresse_pruefen` zu
            // umgehen -- sie sieht nur die erste Station. Jede weitere muss
            // durch dieselbe Pruefung.
            .redirect(reqwest::redirect::Policy::none())
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| Abofehler::Unerreichbar(e.to_string()))?;

        Ok(Self { klient })
    }
}

#[async_trait::async_trait]
impl Feedquelle for UeberDasNetz {
    async fn holen(&self, url: &str) -> Result<String, Abofehler> {
        let mut adresse = url.trim().to_string();

        for _ in 0..=UMLEITUNGEN {
            adresse_pruefen(&adresse)?;

            let antwort = self
                .klient
                .get(&adresse)
                .header("Accept", "text/calendar, text/plain")
                .send()
                .await
                .map_err(|e| Abofehler::Unerreichbar(e.to_string()))?;

            if antwort.status().is_redirection() {
                adresse = antwort
                    .headers()
                    .get(reqwest::header::LOCATION)
                    .and_then(|w| w.to_str().ok())
                    // Eine relative Umleitung ist erlaubt und kommt vor.
                    .and_then(|ziel| url::Url::parse(&adresse).ok()?.join(ziel).ok())
                    .ok_or(Abofehler::KeinKalender)?
                    .to_string();

                continue;
            }

            if !antwort.status().is_success() {
                return Err(Abofehler::Abgelehnt(antwort.status().as_u16()));
            }

            // Erst die angekuendigte Groesse -- damit bricht der Abruf ab,
            // bevor ein Byte durch die Mobilfunkrechnung geht.
            if antwort
                .content_length()
                .is_some_and(|l| l > HOECHSTENS as u64)
            {
                return Err(Abofehler::ZuGross);
            }

            let text = antwort
                .text()
                .await
                .map_err(|e| Abofehler::Unerreichbar(e.to_string()))?;

            // Und dann die tatsaechliche: Eine Quelle muss `Content-Length`
            // nicht mitschicken, und wer es nicht tut, kaeme sonst an der
            // Grenze vorbei.
            if text.len() > HOECHSTENS {
                return Err(Abofehler::ZuGross);
            }

            return Ok(text);
        }

        Err(Abofehler::ZuVieleUmleitungen)
    }
}

/// Einen Abo-Kalender auffrischen.
///
/// Der Kalender muss ein Abo sein; fuer einen eigenen tut diese Funktion
/// nichts (und nicht etwa: loescht seine Termine).
pub async fn auffrischen(
    speicher: &Speicher,
    kalender: &Kalender,
    quelle: &dyn Feedquelle,
) -> Result<Abobericht, Abofehler> {
    let Some(url) = kalender.abo_url.clone() else {
        return Ok(Abobericht::default());
    };

    let text = quelle.holen(&url).await?;

    // Ein `BEGIN:VCALENDAR` ist die einzige billige Art, "das ist ueberhaupt
    // eine ICS-Datei" zu pruefen. Ohne diese Zeile stuende sonst eine
    // Anmeldeseite als "0 Termine" da -- und der Mensch saehe einen Kalender,
    // der sich still geleert hat.
    if !text.contains("BEGIN:VCALENDAR") {
        return Err(Abofehler::KeinKalender);
    }

    let gelesen = ics::lesen(&text);

    let vorhanden = speicher.termine_im_kalender(&kalender.uuid)?.len();
    if gelesen.termine.is_empty() && vorhanden > 0 {
        return Err(Abofehler::QuelleLeer(vorhanden));
    }

    let bericht = einsortieren(speicher, kalender, gelesen)?;

    let mut kalender = kalender.clone();
    kalender.zuletzt_geholt = Some(Local::now().format("%Y-%m-%dT%H:%M:%S").to_string());
    speicher.kalender_schreiben(&kalender, Protokoll::Still)?;

    Ok(bericht)
}

/// Das Gelesene an die Stelle des Vorhandenen.
fn einsortieren(
    speicher: &Speicher,
    kalender: &Kalender,
    gelesen: ics::Gelesen,
) -> Result<Abobericht, Abofehler> {
    let mut bericht = Abobericht {
        uebersprungen: gelesen.uebersprungen,
        ..Abobericht::default()
    };

    let vorhanden = speicher.termine_im_kalender(&kalender.uuid)?;
    let mut bleibt: HashSet<String> = HashSet::new();

    for aus_dem_feed in gelesen.termine {
        let uuid = wiedererkennung(&kalender.uuid, &aus_dem_feed);
        bleibt.insert(uuid.clone());

        let alt = vorhanden.iter().find(|t| t.uuid == uuid);

        // Nur schreiben, was sich geaendert hat. Sonst traegt jeder Abruf
        // jeden Termin neu ein -- und die Oberflaeche zeigte bei jedem
        // Auffrischen "42 geaendert", wo sich nichts getan hat.
        if alt.is_some_and(|t| t.felder == aus_dem_feed.felder) {
            bericht.unveraendert += 1;
            continue;
        }

        if alt.is_some() {
            bericht.geaendert += 1;
        } else {
            bericht.neu += 1;
        }

        speicher.termin_schreiben(
            &Termin {
                uuid,
                kalender_uuid: kalender.uuid.clone(),
                felder: aus_dem_feed.felder,
                papierkorb_at: None,
                geaendert_at: String::new(),
            },
            Protokoll::Still,
        )?;
    }

    // Was nicht mehr im Feed steht, ist dort abgesagt worden.
    //
    // Endgueltig und nicht in den Papierkorb: Der Papierkorb ist fuer das,
    // was der Mensch selbst weggeworfen hat und zurueckholen koennte. Hier
    // koennte er nichts zurueckholen -- der naechste Abruf loeschte es
    // wieder. Ein Papierkorb, der sich von selbst fuellt und aus dem nichts
    // zurueckkommt, ist nur ein Ort, an dem Dinge liegen.
    for t in &vorhanden {
        if !bleibt.contains(&t.uuid) {
            speicher.loeschen(&Art::Termin, &t.uuid, Protokoll::Still)?;
            bericht.entfernt += 1;
        }
    }

    Ok(bericht)
}

/// Dieselbe Kennung fuer denselben Termin -- bei jedem Abruf.
///
/// Ohne das legte jeder Abruf jeden Termin neu an, und der Kalender wuechse
/// bei jedem Auffrischen um seinen eigenen Inhalt.
///
/// Aus dem `UID` der Datei und der Kennung DIESES Kalenders: Derselbe Feed,
/// zweimal abonniert, ergaebe sonst zweimal dieselben Kennungen und damit
/// zwei Kalender, die sich gegenseitig ueberschreiben.
///
/// Ohne `UID` (es kommt vor) tritt der Beginn samt Titel an seine Stelle. Das
/// ist schlechter -- wer den Titel im Feed aendert, bekommt hier einen neuen
/// Termin und der alte verschwindet --, aber es ist stabil, und mehr ist aus
/// einer Datei ohne UID nicht herauszuholen.
fn wiedererkennung(kalender_uuid: &str, termin: &ics::AusDemFeed) -> String {
    let kennzeichen = if termin.uid.is_empty() {
        format!("{}|{}", termin.felder.beginn, termin.felder.titel)
    } else {
        termin.uid.clone()
    };

    uuid::Uuid::new_v5(
        &uuid::Uuid::NAMESPACE_URL,
        format!("openany-abo:{kalender_uuid}:{kennzeichen}").as_bytes(),
    )
    .to_string()
}

#[cfg(test)]
mod abo_test {
    use super::*;

    struct Feste(String);

    #[async_trait::async_trait]
    impl Feedquelle for Feste {
        async fn holen(&self, _url: &str) -> Result<String, Abofehler> {
            Ok(self.0.clone())
        }
    }

    fn datei(vevents: &str) -> String {
        format!("BEGIN:VCALENDAR\r\nVERSION:2.0\r\n{vevents}END:VCALENDAR\r\n")
    }

    fn vevent(uid: &str, titel: &str) -> String {
        format!(
            "BEGIN:VEVENT\r\nUID:{uid}\r\nSUMMARY:{titel}\r\n\
             DTSTART:20260920T090000\r\nDTEND:20260920T100000\r\nEND:VEVENT\r\n"
        )
    }

    fn speicher_mit_abo() -> (Speicher, Kalender) {
        let speicher = Speicher::im_arbeitsspeicher().expect("Speicher");
        let kalender = Kalender {
            uuid: "kal-1".into(),
            name: "Familie".into(),
            farbe: "#4f46e5".into(),
            abo_url: Some("https://api.familywall.example/feed.ics".into()),
            zuletzt_geholt: None,
            papierkorb_at: None,
            geaendert_at: String::new(),
        };
        speicher
            .kalender_schreiben(&kalender, Protokoll::Still)
            .expect("anlegen");

        (speicher, kalender)
    }

    async fn frisch(speicher: &Speicher, kalender: &Kalender, inhalt: &str) -> Abobericht {
        auffrischen(speicher, kalender, &Feste(inhalt.to_string()))
            .await
            .expect("auffrischen")
    }

    #[tokio::test]
    async fn der_erste_abruf_bringt_die_termine() {
        let (speicher, kalender) = speicher_mit_abo();

        let bericht = frisch(&speicher, &kalender, &datei(&vevent("a", "Zahnarzt"))).await;

        assert_eq!(bericht.neu, 1);
        assert_eq!(speicher.termine_im_kalender("kal-1").unwrap().len(), 1);
    }

    /// Der Punkt, an dem ein naiver Abgleich den Kalender bei jedem Abruf
    /// verdoppelt.
    #[tokio::test]
    async fn derselbe_feed_zweimal_bleibt_derselbe_kalender() {
        let (speicher, kalender) = speicher_mit_abo();
        let inhalt = datei(&format!(
            "{}{}",
            vevent("a", "Zahnarzt"),
            vevent("b", "Sport")
        ));

        frisch(&speicher, &kalender, &inhalt).await;
        let kalender = speicher.kalender("kal-1").unwrap().unwrap();
        let zweiter = frisch(&speicher, &kalender, &inhalt).await;

        assert_eq!(zweiter.unveraendert, 2);
        assert_eq!(zweiter.neu, 0);
        assert_eq!(zweiter.geaendert, 0);
        assert_eq!(speicher.termine_im_kalender("kal-1").unwrap().len(), 2);
    }

    /// Die fremde Quelle gewinnt: umbenannt dort heisst umbenannt hier.
    #[tokio::test]
    async fn eine_aenderung_im_feed_kommt_an() {
        let (speicher, kalender) = speicher_mit_abo();

        frisch(&speicher, &kalender, &datei(&vevent("a", "Zahnarzt"))).await;
        let kalender = speicher.kalender("kal-1").unwrap().unwrap();
        let bericht = frisch(
            &speicher,
            &kalender,
            &datei(&vevent("a", "Zahnarzt, verlegt")),
        )
        .await;

        assert_eq!(bericht.geaendert, 1);
        assert_eq!(bericht.neu, 0);
        let termine = speicher.termine_im_kalender("kal-1").unwrap();
        assert_eq!(termine.len(), 1);
        assert_eq!(termine[0].felder.titel, "Zahnarzt, verlegt");
    }

    /// Was im Feed abgesagt wurde, verschwindet auch hier.
    #[tokio::test]
    async fn ein_abgesagter_termin_verschwindet() {
        let (speicher, kalender) = speicher_mit_abo();

        frisch(
            &speicher,
            &kalender,
            &datei(&format!(
                "{}{}",
                vevent("a", "Zahnarzt"),
                vevent("b", "Sport")
            )),
        )
        .await;
        let kalender = speicher.kalender("kal-1").unwrap().unwrap();
        let bericht = frisch(&speicher, &kalender, &datei(&vevent("a", "Zahnarzt"))).await;

        assert_eq!(bericht.entfernt, 1);
        let termine = speicher.termine_im_kalender("kal-1").unwrap();
        assert_eq!(termine.len(), 1);
        assert_eq!(termine[0].felder.titel, "Zahnarzt");
    }

    /// Eine Anmeldeseite statt einer Datei. Der Kalender darf sich davon
    /// NICHT leeren -- das ist der Fall, in dem eine abgelaufene Adresse
    /// still alle Termine mitnimmt.
    #[tokio::test]
    async fn eine_anmeldeseite_leert_den_kalender_nicht() {
        let (speicher, kalender) = speicher_mit_abo();
        frisch(&speicher, &kalender, &datei(&vevent("a", "Zahnarzt"))).await;

        let kalender = speicher.kalender("kal-1").unwrap().unwrap();
        let fehler = auffrischen(
            &speicher,
            &kalender,
            &Feste("<html><body>Bitte anmelden</body></html>".into()),
        )
        .await;

        assert!(matches!(fehler, Err(Abofehler::KeinKalender)));
        assert_eq!(speicher.termine_im_kalender("kal-1").unwrap().len(), 1);
    }

    /// Ein eigener Kalender wird nicht angefasst. Wer das verwechselt,
    /// loescht mit einem Griff alle selbst eingetragenen Termine.
    /// Eine gueltige, aber leere Datei -- das, was FamilyWall auf eine
    /// abgelaufene Adresse schickt. 200 OK, sauberes VCALENDAR, null
    /// Termine. Wer das einsortiert, loescht den ganzen Kalender.
    #[tokio::test]
    async fn eine_leere_quelle_loescht_nichts() {
        let (speicher, kalender) = speicher_mit_abo();
        frisch(&speicher, &kalender, &datei(&vevent("a", "Zahnarzt"))).await;

        let kalender = speicher.kalender("kal-1").unwrap().unwrap();
        let fehler = auffrischen(
            &speicher,
            &kalender,
            // Wortwoertlich die Antwort vom 17.09.2026.
            &Feste(
                "BEGIN:VCALENDAR\r\nPRODID:Invalid\r\nX-WR-CALNAME:Invalid\r\n\
                 CALSCALE:GREGORIAN\r\nEND:VCALENDAR\r\n"
                    .into(),
            ),
        )
        .await;

        assert!(matches!(fehler, Err(Abofehler::QuelleLeer(1))));
        assert_eq!(speicher.termine_im_kalender("kal-1").unwrap().len(), 1);
    }

    /// Ein NEUES Abo darf leer anfangen -- dann ist nichts zu verlieren.
    #[tokio::test]
    async fn ein_leeres_neues_abo_ist_kein_fehler() {
        let (speicher, kalender) = speicher_mit_abo();

        let bericht = frisch(&speicher, &kalender, &datei("")).await;

        assert_eq!(bericht.neu, 0);
    }

    #[tokio::test]
    async fn ein_eigener_kalender_bleibt_unberuehrt() {
        let speicher = Speicher::im_arbeitsspeicher().expect("Speicher");
        let kalender = Kalender {
            uuid: "kal-2".into(),
            name: "Meiner".into(),
            farbe: "#4f46e5".into(),
            abo_url: None,
            zuletzt_geholt: None,
            papierkorb_at: None,
            geaendert_at: String::new(),
        };
        speicher
            .kalender_schreiben(&kalender, Protokoll::Merken)
            .unwrap();
        speicher
            .termin_schreiben(
                &Termin {
                    uuid: "t-1".into(),
                    kalender_uuid: "kal-2".into(),
                    felder: Default::default(),
                    papierkorb_at: None,
                    geaendert_at: String::new(),
                },
                Protokoll::Merken,
            )
            .unwrap();

        let bericht = frisch(&speicher, &kalender, &datei("")).await;

        assert_eq!(bericht, Abobericht::default());
        assert_eq!(speicher.termine_im_kalender("kal-2").unwrap().len(), 1);
    }

    /// Nach einem Abruf steht die Stunde am Kalender -- daran erkennt der
    /// Auffrischer spaeter, ob es sich schon wieder lohnt.
    #[tokio::test]
    async fn die_stunde_des_abrufs_wird_vermerkt() {
        let (speicher, kalender) = speicher_mit_abo();

        frisch(&speicher, &kalender, &datei(&vevent("a", "Zahnarzt"))).await;

        let danach = speicher.kalender("kal-1").unwrap().unwrap();
        assert!(danach.zuletzt_geholt.is_some());
        // Und die Adresse steht noch da. Wer sie hier verloere, machte aus
        // dem Abo beim naechsten Mal einen gewoehnlichen Kalender.
        assert_eq!(danach.abo_url, kalender.abo_url);
    }

    /// Derselbe Feed in zwei Kalendern ergibt zwei getrennte Kalender und
    /// nicht zwei, die sich gegenseitig ueberschreiben.
    #[tokio::test]
    async fn zweimal_derselbe_feed_stoert_sich_nicht() {
        let (speicher, erster) = speicher_mit_abo();
        let zweiter = Kalender {
            uuid: "kal-2".into(),
            ..erster.clone()
        };
        speicher
            .kalender_schreiben(&zweiter, Protokoll::Still)
            .unwrap();

        let inhalt = datei(&vevent("a", "Zahnarzt"));
        frisch(&speicher, &erster, &inhalt).await;
        frisch(&speicher, &zweiter, &inhalt).await;

        assert_eq!(speicher.termine_im_kalender("kal-1").unwrap().len(), 1);
        assert_eq!(speicher.termine_im_kalender("kal-2").unwrap().len(), 1);
    }

    /// Eine Datei ohne UID. Schlechter, aber nicht kaputt: Beim zweiten
    /// Abruf darf der Termin sich nicht verdoppeln.
    #[tokio::test]
    async fn auch_ohne_uid_wird_wiedererkannt() {
        let (speicher, kalender) = speicher_mit_abo();
        let inhalt = datei(
            "BEGIN:VEVENT\r\nSUMMARY:Ohne Kennung\r\n\
             DTSTART:20260920T090000\r\nEND:VEVENT\r\n",
        );

        frisch(&speicher, &kalender, &inhalt).await;
        let kalender = speicher.kalender("kal-1").unwrap().unwrap();
        frisch(&speicher, &kalender, &inhalt).await;

        assert_eq!(speicher.termine_im_kalender("kal-1").unwrap().len(), 1);
    }

    #[test]
    fn adressen_ins_eigene_netz_gehen_nicht() {
        for adresse in [
            "https://192.168.1.1/feed.ics",
            "https://10.0.0.5/feed.ics",
            "https://127.0.0.1/feed.ics",
            "https://[::1]/feed.ics",
            "https://[fe80::1]/feed.ics",
            "https://[fd00::1]/feed.ics",
            "https://localhost/feed.ics",
            "https://fritz.box.local/feed.ics",
        ] {
            assert!(
                adresse_pruefen(adresse).is_err(),
                "durchgelassen: {adresse}"
            );
        }

        assert!(adresse_pruefen("http://example.com/feed.ics").is_err());
        assert!(adresse_pruefen("https://api.familywall.com/f.ics").is_ok());
    }
}
