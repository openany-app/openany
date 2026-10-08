//! Wie dieses Programm zu einem Zugang kommt -- vier Schritte ueber zwei
//! Server.
//!
//! ```text
//!   1  anyid     Kopplung beginnen        -> Code fuer den Menschen
//!   ~  Browser   der Mensch bestaetigt        (hier passiert das Eigentliche)
//!   2  anyid     abholen, mit dem Verifier -> Geraetetoken
//!   3  anyid     Ticket erbitten           -> Ticket, 60 Sekunden gueltig
//!   4  openany   Ticket einloesen          -> Geraeteschluessel
//! ```
//!
//! ## Warum es so herum laeuft und nicht andersherum
//!
//! Der naheliegende Weg waere eine Rueckrufadresse: `openany://auth?ticket=…`.
//! **Er ist bewusst nicht gebaut.** Unter Android kann sich jede beliebige App
//! fuer ein URL-Schema registrieren und das Ticket abfangen. Der gebaute Weg
//! hat gar keine Adresse, die sich entfuehren liesse: Das Programm faengt an,
//! der Mensch bestaetigt im Browser, das Programm **holt** ab -- und weist
//! sich dabei mit einem `verifier` aus, den nur es kennt.
//!
//! **Das Geheimnis liegt offen.** Dieses Programm soll quelloffen werden; was
//! im Binary steht, steht auch im Repo. Es bringt deshalb keinen
//! Anwendungsschluessel fuer anyid mit -- genau dafuer der PKCE-Beweis und der
//! Schluessel je Geraet. Taucht hier je ein festes Geheimnis auf, ist an
//! dieser Stelle etwas falsch.
//!
//! ## Zwei Ausweise, zwei Ablagen, zwei Widerrufswege
//!
//! Am Ende haelt das Programm **zwei** Dinge, und sie sind nicht dasselbe:
//!
//! | | wer stellt aus | wo widerrufen | was es kann |
//! |---|---|---|---|
//! | **Geraetetoken** | anyid | anyids Geraeteliste | Tickets erbitten, sonst nichts |
//! | **Geraeteschluessel** | openany | openanys App-Passwoerter | der Abgleich |
//!
//! Sie in eine Ablage zu legen waere der Fehler, der spaeter teuer wird: Wird
//! der openany-Schluessel widerrufen, ist das **kein** Grund, neu zu koppeln
//! -- solange das Geraetetoken lebt, holt [`Anmeldung::schluessel_erneuern`]
//! einen neuen, ohne dass ein Mensch etwas tun muss. Erst wenn **anyid**
//! widerruft, ist wirklich Schluss; dann werden beide vergessen.
//!
//! ## Wo der Ausweis liegt
//!
//! Hinter [`anyid_client::Tokenspeicher`] -- demselben Trait, an dem auch
//! anytail-app haengt. Die mitgelieferte Dateifassung ist **ausdruecklich
//! keine sichere Ablage**; ausgeliefert gehoeren beide Ausweise in den
//! Android-Keystore beziehungsweise in den Schluesselbund.

mod stellen;

pub use anyid_client::{Dateispeicher, Tokenspeicher};
pub use stellen::{Ausweisstelle, OpenanySchalter, Schluesselstelle};

use anyid_client::{Abholung, AnyidError, Kopplungsstart};
use openany_client::OpenanyError;
use std::io;
use thiserror::Error;

/// Wie dieses Programm bei anyid heisst. Muss in `config/anyid.php` unter
/// `apps` stehen, sonst weist anyid die Kopplung ab.
pub const ANWENDUNG: &str = "openany";

#[derive(Debug, Error)]
pub enum Anmeldefehler {
    #[error(transparent)]
    Anyid(#[from] AnyidError),

    #[error(transparent)]
    Openany(#[from] OpenanyError),

    #[error("The credentials could not be stored: {0}")]
    Ablage(#[from] io::Error),

    /// Es gibt kein Geraetetoken -- dieses Geraet war nie gekoppelt oder ist
    /// abgemeldet worden. Ein Erneuern hilft dann nicht; es muss gekoppelt
    /// werden, und dafuer braucht es einen Menschen vor einem Browser.
    #[error("This device is not paired.")]
    NichtGekoppelt,
}

/// Was dem Menschen zu zeigen ist, waehrend das Programm wartet.
#[derive(Debug, Clone)]
pub struct Kopplung {
    /// Der kurze Code zum Abtippen. **Anzeigen, nicht verschicken** --
    /// abgeholt wird nie mit ihm.
    pub code: String,
    /// Wohin der Mensch gehen soll.
    pub browserziel: String,
    /// Wie oft nachgefragt werden darf, in Sekunden.
    pub intervall: u64,
    /// Wie lange der Code gilt.
    pub ablauf_in: u64,
    /// Der Rest -- unter anderem der Verifier. Er verlaesst das Programm
    /// nicht; nach aussen ging nur sein Hash.
    start: Kopplungsstart,
}

/// Wie weit die Kopplung ist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kopplungsstand {
    /// Der Mensch hat noch nicht bestaetigt. Nach `intervall` erneut fragen.
    Wartet { intervall: u64 },
    /// Fertig -- beide Ausweise liegen. `name` ist, wem das Konto gehoert.
    Gekoppelt { name: String },
    /// Abgelaufen, erfunden, falscher Verifier oder schon abgeholt.
    ///
    /// **Die vier sind nicht zu unterscheiden, und das ist Absicht** -- wer
    /// raet, soll aus der Antwort nicht lernen, wie nah er war. Fuer den
    /// Menschen heisst es in allen vier Faellen dasselbe: noch einmal von
    /// vorn.
    Ungueltig,
}

/// Der Zugang dieses Geraets.
pub struct Anmeldung<A: Ausweisstelle, S: Schluesselstelle> {
    anyid: A,
    openany: S,
    geraetename: String,
    /// Wohin der Mensch geschickt wird, um zu bestaetigen -- anyids
    /// Geraeteliste. Oeffentlich, denn hier geht der Browser hin.
    browserziel: String,
    ausweis: Box<dyn Tokenspeicher + Send + Sync>,
    schluesselablage: Box<dyn Tokenspeicher + Send + Sync>,
}

impl<A: Ausweisstelle, S: Schluesselstelle> Anmeldung<A, S> {
    pub fn neu(
        anyid: A,
        openany: S,
        geraetename: impl Into<String>,
        browserziel: impl Into<String>,
        ausweis: Box<dyn Tokenspeicher + Send + Sync>,
        schluesselablage: Box<dyn Tokenspeicher + Send + Sync>,
    ) -> Self {
        Self {
            anyid,
            openany,
            geraetename: geraetename.into(),
            browserziel: browserziel.into(),
            ausweis,
            schluesselablage,
        }
    }

    /// Der Geraeteschluessel fuer den Abgleich -- oder `None`.
    pub fn schluessel(&self) -> io::Result<Option<String>> {
        self.schluesselablage.lesen()
    }

    /// Ist dieses Geraet einsatzbereit?
    ///
    /// **Beide** Ausweise, nicht einer: Mit dem Geraetetoken allein laesst sich
    /// nichts abgleichen, mit dem Schluessel allein nichts erneuern.
    pub fn einsatzbereit(&self) -> io::Result<bool> {
        Ok(self.ausweis.lesen()?.is_some() && self.schluesselablage.lesen()?.is_some())
    }

    /// Schritt 1: eine Kopplung beginnen.
    pub async fn koppeln_beginnen(&self) -> Result<Kopplung, Anmeldefehler> {
        let start = self
            .anyid
            .kopplung_beginnen(ANWENDUNG, &self.geraetename)
            .await?;

        Ok(Kopplung {
            code: start.code.clone(),
            browserziel: self.browserziel.clone(),
            intervall: start.intervall,
            ablauf_in: start.ablauf_in,
            start,
        })
    }

    /// Schritte 2 bis 4 -- einmal nachfragen.
    ///
    /// **Sie liegen zusammen, weil sie zusammengehoeren.** Zwischen dem
    /// Geraetetoken und dem Geraeteschluessel darf kein Zustand liegen, in dem
    /// das Programm haengenbleiben kann: Ein Geraet mit anyid-Ausweis und ohne
    /// openany-Schluessel sieht angemeldet aus und kann nichts. Faellt Schritt
    /// 3 oder 4 aus, wird deshalb auch der eben geholte Ausweis wieder
    /// vergessen und der Mensch faengt von vorn an -- er steht ohnehin noch
    /// davor.
    pub async fn koppeln_abholen(
        &self,
        kopplung: &Kopplung,
    ) -> Result<Kopplungsstand, Anmeldefehler> {
        match self.anyid.kopplung_abholen(&kopplung.start).await? {
            Abholung::Wartet { intervall } => Ok(Kopplungsstand::Wartet { intervall }),
            Abholung::Ungueltig => Ok(Kopplungsstand::Ungueltig),
            Abholung::Fertig { token, .. } => {
                self.ausweis.schreiben(&token)?;

                match self.schluessel_holen(&token).await {
                    Ok(name) => Ok(Kopplungsstand::Gekoppelt { name }),
                    Err(e) => {
                        // Halb gekoppelt ist schlimmer als gar nicht.
                        let _ = self.ausweis.vergessen();

                        Err(e)
                    }
                }
            }
        }
    }

    /// Einen neuen Geraeteschluessel holen, **ohne neu zu koppeln**.
    ///
    /// Der Fall: openany hat den Schluessel widerrufen -- oder er ist beim
    /// Ablegen verlorengegangen --, aber anyid kennt das Geraet noch. Dann
    /// braucht es keinen Menschen vor einem Browser; ein Ticket genuegt.
    ///
    /// Widerruft dagegen **anyid**, hilft nichts mehr: Beide Ausweise werden
    /// vergessen, und der Aufrufer bekommt [`AnyidError::GeraetWiderrufen`]
    /// -- der eine Fehler, auf den anders zu reagieren ist als mit "spaeter
    /// nochmal".
    pub async fn schluessel_erneuern(&self) -> Result<String, Anmeldefehler> {
        let Some(token) = self.ausweis.lesen()? else {
            return Err(Anmeldefehler::NichtGekoppelt);
        };

        match self.schluessel_holen(&token).await {
            Ok(name) => Ok(name),
            Err(e) => {
                if matches!(
                    e,
                    Anmeldefehler::Anyid(AnyidError::GeraetWiderrufen | AnyidError::KontoGesperrt)
                ) {
                    self.abmelden()?;
                }

                Err(e)
            }
        }
    }

    /// Schritte 3 und 4: Ticket erbitten, Schluessel loesen, ablegen.
    async fn schluessel_holen(&self, geraetetoken: &str) -> Result<String, Anmeldefehler> {
        let ticket = self.anyid.ticket(geraetetoken).await?;

        let schluessel = self
            .openany
            .schluessel_loesen(&ticket.ticket, &self.geraetename)
            .await?;

        // Genau einmal sichtbar -- danach steht drueben nur noch sein Hash.
        // Wer ihn hier nicht ablegt, muss neu koppeln.
        self.schluesselablage.schreiben(&schluessel.schluessel)?;

        Ok(schluessel.name)
    }

    /// Beide Ausweise vergessen.
    ///
    /// **Das ist kein Widerruf.** Drueben leben beide weiter, bis sie dort
    /// widerrufen werden -- in anyids Geraeteliste und in openanys
    /// App-Passwoertern. Ein Programm, das "abgemeldet" mit "widerrufen"
    /// verwechselt, laesst nach einem Geraeteverlust einen Zugang offen, von
    /// dem der Mensch glaubt, er sei zu.
    pub fn abmelden(&self) -> Result<(), Anmeldefehler> {
        self.ausweis.vergessen()?;
        self.schluesselablage.vergessen()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyid_client::{Geraet, Ticket, Verifier};
    use async_trait::async_trait;
    use openany_client::Geraeteschluessel;
    use std::collections::VecDeque;
    use std::sync::Mutex;

    /// Was anyid auf ein Ticket-Erbitten antwortet.
    ///
    /// Als Aufzaehlung und nicht als vorbereiteter `Result`, weil
    /// `AnyidError` nicht `Clone` ist -- und weil so beim Lesen dasteht,
    /// welche drei Faelle es ueberhaupt gibt.
    enum Ticketantwort {
        Gibt,
        Widerrufen,
        Gesperrt,
    }

    struct FakeAnyid {
        /// Womit gekoppelt wurde -- damit sich pruefen laesst, dass es
        /// "openany" ist und nicht versehentlich "anytail".
        gefragt: Mutex<Vec<(String, String)>>,
        abholungen: Mutex<VecDeque<Abholung>>,
        ticket: Mutex<Ticketantwort>,
        tickets_ausgegeben: Mutex<usize>,
    }

    impl FakeAnyid {
        fn neu() -> Self {
            Self {
                gefragt: Mutex::new(Vec::new()),
                abholungen: Mutex::new(VecDeque::new()),
                ticket: Mutex::new(Ticketantwort::Gibt),
                tickets_ausgegeben: Mutex::new(0),
            }
        }

        fn dann(self, abholung: Abholung) -> Self {
            self.abholungen.lock().unwrap().push_back(abholung);
            self
        }

        fn fertig() -> Abholung {
            Abholung::Fertig {
                token: "anyid-geraetetoken".into(),
                geraet: Geraet {
                    id: "g-1".into(),
                    name: "Hannahs Telefon".into(),
                    anwendung: "openany".into(),
                },
            }
        }
    }

    #[async_trait]
    impl Ausweisstelle for FakeAnyid {
        async fn kopplung_beginnen(
            &self,
            anwendung: &str,
            geraetename: &str,
        ) -> Result<Kopplungsstart, AnyidError> {
            self.gefragt
                .lock()
                .unwrap()
                .push((anwendung.into(), geraetename.into()));

            Ok(Kopplungsstart {
                code: "ABCD-1234".into(),
                geraetecode: "langer-code".into(),
                verifier: Verifier::aus("bleibt-im-programm"),
                intervall: 5,
                ablauf_in: 600,
            })
        }

        async fn kopplung_abholen(&self, _start: &Kopplungsstart) -> Result<Abholung, AnyidError> {
            Ok(self
                .abholungen
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or(Abholung::Ungueltig))
        }

        async fn ticket(&self, _geraetetoken: &str) -> Result<Ticket, AnyidError> {
            match *self.ticket.lock().unwrap() {
                Ticketantwort::Widerrufen => Err(AnyidError::GeraetWiderrufen),
                Ticketantwort::Gesperrt => Err(AnyidError::KontoGesperrt),
                Ticketantwort::Gibt => {
                    let mut n = self.tickets_ausgegeben.lock().unwrap();
                    *n += 1;

                    Ok(Ticket {
                        ticket: format!("ticket-{n}"),
                        anwendung: "openany".into(),
                    })
                }
            }
        }
    }

    struct FakeOpenany {
        eingeloest: Mutex<Vec<String>>,
        faellt_aus: Mutex<bool>,
        naechster: Mutex<usize>,
    }

    impl FakeOpenany {
        fn neu() -> Self {
            Self {
                eingeloest: Mutex::new(Vec::new()),
                faellt_aus: Mutex::new(false),
                naechster: Mutex::new(0),
            }
        }
    }

    #[async_trait]
    impl Schluesselstelle for FakeOpenany {
        async fn schluessel_loesen(
            &self,
            ticket: &str,
            geraetename: &str,
        ) -> Result<Geraeteschluessel, OpenanyError> {
            self.eingeloest.lock().unwrap().push(ticket.into());

            if *self.faellt_aus.lock().unwrap() {
                return Err(OpenanyError::SchluesselUngueltig);
            }

            let mut n = self.naechster.lock().unwrap();
            *n += 1;

            Ok(Geraeteschluessel {
                schluessel: format!("15|schluessel-{n}"),
                geraet: geraetename.into(),
                name: "Hannah".into(),
            })
        }
    }

    struct Aufbau {
        _ordner: tempfile::TempDir,
        anmeldung: Anmeldung<FakeAnyid, FakeOpenany>,
    }

    fn aufbauen(anyid: FakeAnyid, openany: FakeOpenany) -> Aufbau {
        let ordner = tempfile::tempdir().unwrap();

        Aufbau {
            anmeldung: Anmeldung::neu(
                anyid,
                openany,
                "Hannahs Telefon",
                "https://id.openany.de/einstellungen/geraete",
                // Die echte Dateifassung und keine dritte Attrappe: Dass zwei
                // getrennte Ablagen wirklich zwei getrennte Dateien sind, ist
                // genau das, was hier zu pruefen ist.
                Box::new(Dateispeicher::neu(ordner.path().join("anyid-ausweis"))),
                Box::new(Dateispeicher::neu(ordner.path().join("openany-schluessel"))),
            ),
            _ordner: ordner,
        }
    }

    #[tokio::test]
    async fn der_ganze_weg_von_vorn_bis_hinten() {
        let a = aufbauen(
            FakeAnyid::neu()
                .dann(Abholung::Wartet { intervall: 5 })
                .dann(FakeAnyid::fertig()),
            FakeOpenany::neu(),
        );

        assert!(!a.anmeldung.einsatzbereit().unwrap());

        let kopplung = a.anmeldung.koppeln_beginnen().await.unwrap();

        assert_eq!(kopplung.code, "ABCD-1234");
        assert!(kopplung.browserziel.contains("/einstellungen/geraete"));

        // Der Mensch hat noch nicht bestaetigt.
        assert_eq!(
            a.anmeldung.koppeln_abholen(&kopplung).await.unwrap(),
            Kopplungsstand::Wartet { intervall: 5 }
        );
        assert!(!a.anmeldung.einsatzbereit().unwrap());

        // Jetzt hat er.
        assert_eq!(
            a.anmeldung.koppeln_abholen(&kopplung).await.unwrap(),
            Kopplungsstand::Gekoppelt {
                name: "Hannah".into()
            }
        );

        assert!(a.anmeldung.einsatzbereit().unwrap());
        assert_eq!(
            a.anmeldung.schluessel().unwrap().as_deref(),
            Some("15|schluessel-1")
        );
    }

    #[tokio::test]
    async fn gekoppelt_wird_fuer_openany_und_nicht_fuer_irgendetwas() {
        // Dieselbe Schale koennte fuer anytail koppeln; anyid prueft die
        // Anwendung gegen seine Liste. Ein falscher Name hier ergaebe ein
        // Geraet, das Tickets fuer die falsche Anwendung erbittet.
        let a = aufbauen(FakeAnyid::neu(), FakeOpenany::neu());

        a.anmeldung.koppeln_beginnen().await.unwrap();

        let gefragt = a.anmeldung.anyid.gefragt.lock().unwrap();

        assert_eq!(gefragt[0], ("openany".into(), "Hannahs Telefon".into()));
        assert_eq!(ANWENDUNG, "openany");
    }

    #[tokio::test]
    async fn ein_ungueltiger_code_legt_nichts_ab() {
        let a = aufbauen(
            FakeAnyid::neu().dann(Abholung::Ungueltig),
            FakeOpenany::neu(),
        );

        let kopplung = a.anmeldung.koppeln_beginnen().await.unwrap();

        assert_eq!(
            a.anmeldung.koppeln_abholen(&kopplung).await.unwrap(),
            Kopplungsstand::Ungueltig
        );
        assert!(!a.anmeldung.einsatzbereit().unwrap());
        assert_eq!(a.anmeldung.schluessel().unwrap(), None);
    }

    #[tokio::test]
    async fn halb_gekoppelt_bleibt_niemand() {
        // anyid sagt ja, openany faellt aus. Ein Geraet mit anyid-Ausweis und
        // ohne openany-Schluessel saehe angemeldet aus und koennte nichts.
        let openany = FakeOpenany::neu();
        *openany.faellt_aus.lock().unwrap() = true;

        let a = aufbauen(FakeAnyid::neu().dann(FakeAnyid::fertig()), openany);

        let kopplung = a.anmeldung.koppeln_beginnen().await.unwrap();
        let fehler = a.anmeldung.koppeln_abholen(&kopplung).await.unwrap_err();

        assert!(matches!(fehler, Anmeldefehler::Openany(_)), "{fehler:?}");
        assert!(
            !a.anmeldung.einsatzbereit().unwrap(),
            "der eben geholte Ausweis muss wieder fort sein"
        );
        assert_eq!(a.anmeldung.schluessel().unwrap(), None);
    }

    #[tokio::test]
    async fn ein_widerrufener_schluessel_heisst_nicht_neu_koppeln() {
        // Der Fall, fuer den die zwei getrennten Ablagen da sind: openany hat
        // den Schluessel widerrufen, anyid kennt das Geraet noch. Kein Mensch
        // muss etwas tun.
        let a = aufbauen(
            FakeAnyid::neu().dann(FakeAnyid::fertig()),
            FakeOpenany::neu(),
        );

        let kopplung = a.anmeldung.koppeln_beginnen().await.unwrap();
        a.anmeldung.koppeln_abholen(&kopplung).await.unwrap();

        let name = a.anmeldung.schluessel_erneuern().await.unwrap();

        assert_eq!(name, "Hannah");
        assert_eq!(
            a.anmeldung.schluessel().unwrap().as_deref(),
            Some("15|schluessel-2"),
            "ein neuer Schluessel"
        );
        assert_eq!(
            *a.anmeldung.anyid.tickets_ausgegeben.lock().unwrap(),
            2,
            "zwei Tickets, eine Kopplung"
        );
    }

    #[tokio::test]
    async fn erneuern_ohne_kopplung_ist_ein_eigener_fehler() {
        // Nicht "Netzwerkfehler" und nicht "unberechtigt": Der Aufrufer muss
        // unterscheiden koennen, ob er es spaeter nochmal versuchen soll oder
        // einen Menschen vor einen Browser holen muss.
        let a = aufbauen(FakeAnyid::neu(), FakeOpenany::neu());

        let fehler = a.anmeldung.schluessel_erneuern().await.unwrap_err();

        assert!(
            matches!(fehler, Anmeldefehler::NichtGekoppelt),
            "{fehler:?}"
        );
    }

    #[tokio::test]
    async fn ein_bei_anyid_widerrufenes_geraet_vergisst_beide_ausweise() {
        let a = aufbauen(
            FakeAnyid::neu().dann(FakeAnyid::fertig()),
            FakeOpenany::neu(),
        );

        let kopplung = a.anmeldung.koppeln_beginnen().await.unwrap();
        a.anmeldung.koppeln_abholen(&kopplung).await.unwrap();
        assert!(a.anmeldung.einsatzbereit().unwrap());

        *a.anmeldung.anyid.ticket.lock().unwrap() = Ticketantwort::Widerrufen;

        let fehler = a.anmeldung.schluessel_erneuern().await.unwrap_err();

        assert!(
            matches!(fehler, Anmeldefehler::Anyid(AnyidError::GeraetWiderrufen)),
            "{fehler:?}"
        );
        assert!(
            !a.anmeldung.einsatzbereit().unwrap(),
            "hier hilft kein Wiederholen -- also darf auch nichts liegenbleiben"
        );
        assert_eq!(a.anmeldung.schluessel().unwrap(), None);
    }

    #[tokio::test]
    async fn ein_gesperrtes_konto_ebenso() {
        let a = aufbauen(
            FakeAnyid::neu().dann(FakeAnyid::fertig()),
            FakeOpenany::neu(),
        );

        let kopplung = a.anmeldung.koppeln_beginnen().await.unwrap();
        a.anmeldung.koppeln_abholen(&kopplung).await.unwrap();

        *a.anmeldung.anyid.ticket.lock().unwrap() = Ticketantwort::Gesperrt;
        let _ = a.anmeldung.schluessel_erneuern().await;

        assert!(!a.anmeldung.einsatzbereit().unwrap());
    }

    #[tokio::test]
    async fn abmelden_vergisst_beides_und_widerruft_nichts() {
        let a = aufbauen(
            FakeAnyid::neu().dann(FakeAnyid::fertig()),
            FakeOpenany::neu(),
        );

        let kopplung = a.anmeldung.koppeln_beginnen().await.unwrap();
        a.anmeldung.koppeln_abholen(&kopplung).await.unwrap();

        a.anmeldung.abmelden().unwrap();

        assert!(!a.anmeldung.einsatzbereit().unwrap());
        assert_eq!(a.anmeldung.schluessel().unwrap(), None);
        // Drueben lebt beides weiter. Wer "abgemeldet" mit "widerrufen"
        // verwechselt, laesst nach einem Geraeteverlust einen Zugang offen,
        // von dem der Mensch glaubt, er sei zu.
        assert_eq!(a.anmeldung.openany.eingeloest.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn zweimal_abmelden_beschwert_sich_nicht() {
        let a = aufbauen(FakeAnyid::neu(), FakeOpenany::neu());

        a.anmeldung.abmelden().unwrap();
        a.anmeldung.abmelden().unwrap();
    }

    #[tokio::test]
    async fn jedes_ticket_wird_nur_einmal_eingeloest() {
        // Ein Ticket lebt sechzig Sekunden und ist einmal einloesbar. Zweimal
        // dasselbe zu schicken hiesse, beim zweiten Mal eine 401 zu bekommen
        // und sie fuer einen Widerruf zu halten.
        let a = aufbauen(
            FakeAnyid::neu().dann(FakeAnyid::fertig()),
            FakeOpenany::neu(),
        );

        let kopplung = a.anmeldung.koppeln_beginnen().await.unwrap();
        a.anmeldung.koppeln_abholen(&kopplung).await.unwrap();
        a.anmeldung.schluessel_erneuern().await.unwrap();

        let eingeloest = a.anmeldung.openany.eingeloest.lock().unwrap();

        assert_eq!(*eingeloest, vec!["ticket-1", "ticket-2"]);
    }
}
