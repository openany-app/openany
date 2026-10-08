//! Lesen über IMAP: Posteingang und „Gesendet", sonst nichts.
//!
//! **Jeder Aufruf baut seine Verbindung selbst auf** und schließt sie wieder.
//! Das kostet eine Anmeldung je Abholen, hält aber keinen Zustand, der nach
//! einem Netzwechsel still kaputt wäre. Die dauerhafte Verbindung (IDLE)
//! kommt mit dem Wachdienst (Schritt 4).
//!
//! **Was schon da ist, merkt sich der Aufrufer** als [`Stand`] je Ordner:
//! `UIDVALIDITY` und die höchste gesehene UID. Ändert der Server die
//! UIDVALIDITY (Ordner neu angelegt), gilt der Stand nicht mehr, und der
//! Ordner wird wie beim ersten Mal gelesen.

use crate::{tls, Ergebnis, Konto, PostFehler, Server, Sicherheit};
use async_imap::types::{Flag, NameAttribute};
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;

/// Die Verbindung: TLS, oder -- nur im Test -- Klartext.
#[derive(Debug)]
enum Strom {
    Tls(Box<TlsStream<TcpStream>>),
    #[cfg(feature = "testserver")]
    Klar(TcpStream),
}

impl tokio::io::AsyncRead for Strom {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            Strom::Tls(s) => std::pin::Pin::new(s.as_mut()).poll_read(cx, buf),
            #[cfg(feature = "testserver")]
            Strom::Klar(s) => std::pin::Pin::new(s).poll_read(cx, buf),
        }
    }
}

impl tokio::io::AsyncWrite for Strom {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        match self.get_mut() {
            Strom::Tls(s) => std::pin::Pin::new(s.as_mut()).poll_write(cx, buf),
            #[cfg(feature = "testserver")]
            Strom::Klar(s) => std::pin::Pin::new(s).poll_write(cx, buf),
        }
    }
    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            Strom::Tls(s) => std::pin::Pin::new(s.as_mut()).poll_flush(cx),
            #[cfg(feature = "testserver")]
            Strom::Klar(s) => std::pin::Pin::new(s).poll_flush(cx),
        }
    }
    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            Strom::Tls(s) => std::pin::Pin::new(s.as_mut()).poll_shutdown(cx),
            #[cfg(feature = "testserver")]
            Strom::Klar(s) => std::pin::Pin::new(s).poll_shutdown(cx),
        }
    }
}

type Sitzung = async_imap::Session<Strom>;

/// Die Ordner, die openany kennt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Ordner {
    pub eingang: String,
    /// `\Sent` -- `None`, wenn der Server keinen so kennzeichnet und auch
    /// keiner „Sent"/„Gesendet" heißt.
    pub gesendet: Option<String>,
    /// `\Trash` -- wohin „auch auf dem Mailserver löschen" verschiebt.
    pub papierkorb: Option<String>,
    /// `\Junk` -- wo der Anbieter Spamverdacht ablegt. Nicht im Verlauf;
    /// die App zeigt ihn als eigene Liste. (Seit 29.09.2026; ältere
    /// Einträge im Tresor haben ihn noch nicht.)
    #[serde(default)]
    pub spam: Option<String>,
}

/// Wie weit ein Ordner gelesen ist.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Stand {
    pub ordner: String,
    pub uidvalidity: u32,
    pub letzte_uid: u32,
}

/// Eine abgeholte Mail, noch roh.
#[derive(Debug, Clone)]
pub struct Roh {
    pub ordner: String,
    pub uidvalidity: u32,
    pub uid: u32,
    /// `\Seen` auf dem Server -- im Mailprogramm schon gelesen.
    pub gelesen: bool,
    pub daten: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Abgeholt {
    pub ordner: Ordner,
    pub mails: Vec<Roh>,
    /// Der neue Stand je gelesenem Ordner -- nach dem Ablegen der Mails
    /// speichern, nicht vorher.
    pub staende: Vec<Stand>,
    /// Wie viele Mails im Spam-Ordner liegen -- `None` ohne einen.
    pub spam: Option<u32>,
}

async fn tls_strom(host: &str, tcp: TcpStream) -> Ergebnis<TlsStream<TcpStream>> {
    let name = rustls::pki_types::ServerName::try_from(host.to_string())
        .map_err(|e| PostFehler::Verbindung(host.into(), e.to_string()))?;
    tokio_rustls::TlsConnector::from(tls())
        .connect(name, tcp)
        .await
        .map_err(|e| PostFehler::Verbindung(host.into(), e.to_string()))
}

async fn verbinden(konto: &Konto) -> Ergebnis<Sitzung> {
    let Server {
        host,
        port,
        sicherheit,
    } = &konto.imap;
    let fehler = |e: &dyn std::fmt::Display| PostFehler::Verbindung(host.clone(), e.to_string());

    let tcp = tokio::time::timeout(
        std::time::Duration::from_secs(20),
        TcpStream::connect((host.as_str(), *port)),
    )
    .await
    .map_err(|e| fehler(&e))?
    .map_err(|e| fehler(&e))?;

    let client = match sicherheit {
        Sicherheit::Ssl => {
            let mut c = async_imap::Client::new(Strom::Tls(Box::new(tls_strom(host, tcp).await?)));
            c.read_response()
                .await
                .ok_or_else(|| fehler(&"no greeting"))?
                .map_err(|e| fehler(&e))?;
            c
        }
        Sicherheit::Starttls => {
            let mut c = async_imap::Client::new(tcp);
            c.read_response()
                .await
                .ok_or_else(|| fehler(&"no greeting"))?
                .map_err(|e| fehler(&e))?;
            c.run_command_and_check_ok("STARTTLS", None)
                .await
                .map_err(|e| fehler(&e))?;
            async_imap::Client::new(Strom::Tls(Box::new(tls_strom(host, c.into_inner()).await?)))
        }
        #[cfg(feature = "testserver")]
        Sicherheit::Klartext => {
            let mut c = async_imap::Client::new(Strom::Klar(tcp));
            c.read_response()
                .await
                .ok_or_else(|| fehler(&"no greeting"))?
                .map_err(|e| fehler(&e))?;
            c
        }
    };

    client
        .login(&konto.benutzer, &konto.passwort)
        .await
        .map_err(|_| PostFehler::Anmeldung)
}

fn server(e: impl std::fmt::Display) -> PostFehler {
    PostFehler::Server(e.to_string())
}

/// Anmelden, Ordner finden, abmelden -- für „Verbinden" in den Einstellungen.
pub async fn pruefen(konto: &Konto) -> Ergebnis<Ordner> {
    let mut s = verbinden(konto).await?;
    let ordner = ordner_finden_in(&mut s).await?;
    let _ = s.logout().await;
    Ok(ordner)
}

/// Die Ordner, wie der Server sie nennt.
pub async fn ordner_finden(konto: &Konto) -> Ergebnis<Ordner> {
    pruefen(konto).await
}

async fn ordner_finden_in(s: &mut Sitzung) -> Ergebnis<Ordner> {
    let namen: Vec<_> = s
        .list(Some(""), Some("*"))
        .await
        .map_err(server)?
        .try_collect()
        .await
        .map_err(server)?;

    let mit = |attr: NameAttribute<'static>| {
        namen
            .iter()
            .find(|n| n.attributes().contains(&attr))
            .map(|n| n.name().to_string())
    };
    // Ohne RFC-6154-Kennzeichen: an den üblichen Namen erkennen.
    let heisst = |kandidaten: &[&str]| {
        namen
            .iter()
            .map(|n| n.name())
            .find(|name| {
                let letzter = name.rsplit(['/', '.']).next().unwrap_or(name);
                kandidaten.iter().any(|k| letzter.eq_ignore_ascii_case(k))
            })
            .map(str::to_string)
    };

    Ok(Ordner {
        eingang: "INBOX".into(),
        gesendet: mit(NameAttribute::Sent).or_else(|| {
            heisst(&[
                "Sent",
                "Gesendet",
                "Sent Items",
                "Sent Messages",
                "Gesendete Objekte",
                "Gesendete Elemente",
            ])
        }),
        papierkorb: mit(NameAttribute::Trash).or_else(|| {
            heisst(&[
                "Trash",
                "Papierkorb",
                "Deleted Items",
                "Deleted Messages",
                "Gelöschte Elemente",
            ])
        }),
        spam: mit(NameAttribute::Junk).or_else(|| {
            heisst(&[
                "Spam",
                "Junk",
                "Junk-E-Mail",
                "Junk E-Mail",
                "Junk Email",
                "Spamverdacht",
                "Unerwünscht",
                "Bulk Mail",
            ])
        }),
    })
}

/// Neue Mails aus Posteingang und „Gesendet".
///
/// `erstmals`: Wie viele der neuesten Mails ein Ordner beim ersten Lesen
/// liefert -- ein Postfach mit 20 000 Mails soll nicht komplett herunter.
pub async fn abholen(konto: &Konto, staende: &[Stand], erstmals: usize) -> Ergebnis<Abgeholt> {
    let mut s = verbinden(konto).await?;
    let ordner = ordner_finden_in(&mut s).await?;

    let mut mails = Vec::new();
    let mut neu_staende = Vec::new();
    let mut zu_lesen = vec![ordner.eingang.clone()];
    zu_lesen.extend(ordner.gesendet.clone());

    for name in zu_lesen {
        let postfach = s.select(&name).await.map_err(server)?;
        let uidvalidity = postfach.uid_validity.unwrap_or(0);
        let alt = staende
            .iter()
            .find(|st| st.ordner == name && st.uidvalidity == uidvalidity);

        let (bereich, ab) = match alt {
            Some(st) => (format!("{}:*", st.letzte_uid + 1), st.letzte_uid),
            None => {
                if postfach.exists == 0 {
                    neu_staende.push(Stand {
                        ordner: name,
                        uidvalidity,
                        letzte_uid: 0,
                    });
                    continue;
                }
                let mut alle: Vec<u32> = s
                    .uid_search("ALL")
                    .await
                    .map_err(server)?
                    .into_iter()
                    .collect();
                alle.sort_unstable();
                let neueste: Vec<String> = alle
                    .iter()
                    .rev()
                    .take(erstmals)
                    .map(u32::to_string)
                    .collect();
                if neueste.is_empty() {
                    neu_staende.push(Stand {
                        ordner: name,
                        uidvalidity,
                        letzte_uid: 0,
                    });
                    continue;
                }
                (neueste.join(","), 0)
            }
        };

        let abrufe: Vec<_> = s
            .uid_fetch(&bereich, "(UID FLAGS BODY.PEEK[])")
            .await
            .map_err(server)?
            .try_collect()
            .await
            .map_err(server)?;

        let mut hoechste = ab;
        for f in abrufe {
            // `n:*` liefert immer mindestens die letzte Mail -- auch wenn sie
            // schon bekannt ist.
            let Some(uid) = f.uid.filter(|u| *u > ab) else {
                continue;
            };
            let Some(daten) = f.body() else {
                continue;
            };
            hoechste = hoechste.max(uid);
            mails.push(Roh {
                ordner: name.clone(),
                uidvalidity,
                uid,
                gelesen: f.flags().any(|fl| matches!(fl, Flag::Seen)),
                daten: daten.to_vec(),
            });
        }
        neu_staende.push(Stand {
            ordner: name,
            uidvalidity,
            letzte_uid: hoechste,
        });
    }

    // Den Spam-Ordner nur zählen (EXAMINE ändert nichts, auch kein \Seen).
    let spam = match ordner.spam.as_deref() {
        Some(name) => s.examine(name).await.ok().map(|p| p.exists),
        None => None,
    };

    let _ = s.logout().await;
    Ok(Abgeholt {
        ordner,
        mails,
        staende: neu_staende,
        spam,
    })
}

/// Die neuesten `hoechstens` Mails eines Ordners, ohne sie als gelesen zu
/// markieren und ohne Stand -- für den Spamverdacht, der nicht in den
/// Verlauf gehört und bei jedem Öffnen frisch gelesen wird.
pub async fn neueste(konto: &Konto, ordner: &str, hoechstens: usize) -> Ergebnis<Vec<Roh>> {
    let mut s = verbinden(konto).await?;
    let postfach = s.examine(ordner).await.map_err(server)?;
    let uidvalidity = postfach.uid_validity.unwrap_or(0);
    let mut alle: Vec<u32> = s
        .uid_search("ALL")
        .await
        .map_err(server)?
        .into_iter()
        .collect();
    alle.sort_unstable();
    let neueste: Vec<String> = alle
        .iter()
        .rev()
        .take(hoechstens)
        .map(u32::to_string)
        .collect();
    let mut mails = Vec::new();
    if !neueste.is_empty() {
        let abrufe: Vec<_> = s
            .uid_fetch(neueste.join(","), "(UID FLAGS BODY.PEEK[])")
            .await
            .map_err(server)?
            .try_collect()
            .await
            .map_err(server)?;
        for f in abrufe {
            let (Some(uid), Some(daten)) = (f.uid, f.body()) else {
                continue;
            };
            mails.push(Roh {
                ordner: ordner.to_string(),
                uidvalidity,
                uid,
                gelesen: f.flags().any(|fl| matches!(fl, Flag::Seen)),
                daten: daten.to_vec(),
            });
        }
    }
    let _ = s.logout().await;
    mails.sort_by_key(|a| std::cmp::Reverse(a.uid));
    Ok(mails)
}

/// Eine Mail in einen anderen Ordner verschieben (UID MOVE; kann der Server
/// das nicht, COPY und dann `\Deleted` + EXPUNGE). Für „Kein Spam".
pub async fn verschieben(konto: &Konto, von: &str, uid: u32, nach: &str) -> Ergebnis<()> {
    let mut s = verbinden(konto).await?;
    s.select(von).await.map_err(server)?;
    let uid = uid.to_string();
    if s.uid_mv(&uid, nach).await.is_err() {
        s.uid_copy(&uid, nach).await.map_err(server)?;
        let _: Vec<_> = s
            .uid_store(&uid, "+FLAGS.SILENT (\\Deleted)")
            .await
            .map_err(server)?
            .try_collect()
            .await
            .map_err(server)?;
        let _: Vec<_> = s
            .expunge()
            .await
            .map_err(server)?
            .try_collect()
            .await
            .map_err(server)?;
    }
    let _ = s.logout().await;
    Ok(())
}

/// `\Seen` setzen -- im Mailprogramm steht die Mail dann auch als gelesen.
pub async fn als_gelesen(konto: &Konto, ordner: &str, uid: u32) -> Ergebnis<()> {
    let mut s = verbinden(konto).await?;
    s.select(ordner).await.map_err(server)?;
    let _: Vec<_> = s
        .uid_store(uid.to_string(), "+FLAGS.SILENT (\\Seen)")
        .await
        .map_err(server)?
        .try_collect()
        .await
        .map_err(server)?;
    let _ = s.logout().await;
    Ok(())
}

/// Auf dem Mailserver löschen: in den Papierkorb-Ordner verschieben, wenn es
/// einen gibt (und die Mail nicht schon darin liegt), sonst `\Deleted` und
/// EXPUNGE.
pub async fn loeschen(
    konto: &Konto,
    ordner: &str,
    uid: u32,
    papierkorb: Option<&str>,
) -> Ergebnis<()> {
    let mut s = verbinden(konto).await?;
    s.select(ordner).await.map_err(server)?;
    weg(&mut s, ordner, &uid.to_string(), papierkorb).await?;
    let _ = s.logout().await;
    Ok(())
}

/// Wie [`loeschen`], aber die Mail wird in `ordner` an ihrer Message-ID
/// gesucht (ohne `<>`). Für eine gesendete, deren Stelle in „Gesendet" die
/// App noch nicht kennt, und für die zweite Kopie einer Mail an sich selbst.
/// Gibt zurück, wie viele es waren.
pub async fn loeschen_nach_id(
    konto: &Konto,
    ordner: &str,
    message_id: &str,
    papierkorb: Option<&str>,
) -> Ergebnis<usize> {
    // Keine Anführungszeichen oder Zeilenumbrüche in die Suche lassen.
    if message_id.is_empty() || message_id.contains(['"', '\\', '\r', '\n']) {
        return Ok(0);
    }
    let mut s = verbinden(konto).await?;
    s.select(ordner).await.map_err(server)?;
    let uids = s
        .uid_search(format!("HEADER Message-ID \"<{message_id}>\""))
        .await
        .map_err(server)?;
    if !uids.is_empty() {
        let liste = uids
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(",");
        weg(&mut s, ordner, &liste, papierkorb).await?;
    }
    let _ = s.logout().await;
    Ok(uids.len())
}

/// Die UIDs im gewählten Ordner in den Papierkorb, sonst endgültig.
async fn weg(s: &mut Sitzung, ordner: &str, uids: &str, papierkorb: Option<&str>) -> Ergebnis<()> {
    let verschoben = match papierkorb.filter(|p| *p != ordner) {
        Some(p) => s.uid_mv(uids, p).await.is_ok(),
        None => false,
    };
    if !verschoben {
        let _: Vec<_> = s
            .uid_store(uids, "+FLAGS.SILENT (\\Deleted)")
            .await
            .map_err(server)?
            .try_collect()
            .await
            .map_err(server)?;
        let _: Vec<_> = s
            .expunge()
            .await
            .map_err(server)?
            .try_collect()
            .await
            .map_err(server)?;
    }
    Ok(())
}

/// Eine gesendete Mail in „Gesendet" ablegen -- SMTP tut das nicht von
/// selbst, und ohne sie fehlte die eigene Hälfte im Mailprogramm.
pub async fn ablegen(konto: &Konto, ordner: &str, daten: &[u8]) -> Ergebnis<()> {
    let mut s = verbinden(konto).await?;
    s.append(ordner, Some("(\\Seen)"), None, daten)
        .await
        .map_err(server)?;
    let _ = s.logout().await;
    Ok(())
}

/// Auf neue Mails warten (IMAP IDLE, RFC 2177) -- für den Wachdienst
/// (docs/plan-email-pgp.md, Schritt 4).
///
/// Kehrt zurück, sobald der Server etwas meldet (`true`), oder nach
/// `hoechstens` ohne Meldung (`false`). Danach ist die Verbindung zu: Der
/// Aufrufer holt mit [`abholen`] ab und ruft wieder. Neu verbinden nach
/// spätestens `hoechstens` ist Absicht -- viele Server (und Router) werfen
/// eine stille Verbindung nach 30 Minuten hinaus, und eine tote merkt man
/// sonst nicht.
///
/// **Ohne IDLE beim Server** wird `hoechstens` gewartet und dann `true`
/// gemeldet: einmal abholen je Runde, wie ein langsames Nachsehen.
pub async fn warten(
    konto: &Konto,
    ordner: &str,
    hoechstens: std::time::Duration,
) -> Ergebnis<bool> {
    let mut s = verbinden(konto).await?;
    let kann_idle = s
        .capabilities()
        .await
        .map(|c| c.has_str("IDLE"))
        .unwrap_or(false);
    s.select(ordner).await.map_err(server)?;
    if !kann_idle {
        let _ = s.logout().await;
        tokio::time::sleep(hoechstens).await;
        return Ok(true);
    }
    let mut idle = s.idle();
    idle.init().await.map_err(server)?;
    let (warte, _stopp) = idle.wait_with_timeout(hoechstens);
    let antwort = warte.await.map_err(server)?;
    let neu = matches!(
        antwort,
        async_imap::extensions::idle::IdleResponse::NewData(_)
    );
    if let Ok(mut s) = idle.done().await {
        let _ = s.logout().await;
    }
    Ok(neu)
}
