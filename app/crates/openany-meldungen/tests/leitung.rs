//! Die Leitung gegen einen kleinen Server, der ntfy spielt.
//!
//! Kein echtes ntfy: Die Faelle, um die es geht -- eine Zeile, die mitten
//! durchgeschnitten ankommt; ein Server, der schweigt; ein Netz, das
//! wechselt --, stellt ein echtes ntfy nicht auf Bestellung her.

use std::time::Duration;

use openany_meldungen::{halten, Leitung, Leitungsfehler, Signal, Vorgang};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, watch};

/// Nimmt Verbindungen an, meldet die Anfragezeile und gibt die Verbindung
/// an den Test heraus, der dann Zeilen schreibt.
async fn server() -> (String, mpsc::UnboundedReceiver<(String, TcpStream)>) {
    let horcher = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let basis = format!("http://{}", horcher.local_addr().unwrap());
    let (tx, rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        loop {
            let (mut s, _) = horcher.accept().await.unwrap();
            let mut kopf = Vec::new();
            let mut b = [0u8; 1];
            while !kopf.ends_with(b"\r\n\r\n") {
                if s.read(&mut b).await.unwrap() == 0 {
                    break;
                }
                kopf.push(b[0]);
            }
            let zeile = String::from_utf8_lossy(&kopf)
                .lines()
                .next()
                .unwrap_or("")
                .to_owned();
            s.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/x-ndjson\r\nConnection: close\r\n\r\n")
                .await
                .unwrap();
            if tx.send((zeile, s)).is_err() {
                return;
            }
        }
    });
    (basis, rx)
}

fn starten(
    leitung: Leitung,
    netz: watch::Receiver<bool>,
) -> (
    mpsc::UnboundedReceiver<Vorgang>,
    tokio::task::JoinHandle<()>,
) {
    let (tx, rx) = mpsc::unbounded_channel();
    let h = tokio::spawn(halten(leitung, netz, move |v| {
        let _ = tx.send(v);
    }));
    (rx, h)
}

async fn naechster(rx: &mut mpsc::UnboundedReceiver<Vorgang>) -> Vorgang {
    tokio::time::timeout(Duration::from_secs(10), rx.recv())
        .await
        .expect("nichts kam")
        .unwrap()
}

#[tokio::test]
async fn liest_zerschnittene_zeilen_und_holt_nach_einem_abriss_mit_since_nach() {
    let (basis, mut verbindungen) = server().await;
    let (_netz_tx, netz) = watch::channel(true);
    let (mut vorgaenge, h) = starten(Leitung::new(&basis, "upGeheim"), netz);

    let (anfrage, mut s) = verbindungen.recv().await.unwrap();
    assert_eq!(
        anfrage, "GET /upGeheim/json HTTP/1.1",
        "ohne letzte Kennung kein since"
    );
    s.write_all(b"{\"id\":\"o1\",\"event\":\"open\"}\n{\"id\":\"k\",\"event\":\"keep")
        .await
        .unwrap();
    s.flush().await.unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;
    s.write_all(b"alive\"}\n{\"id\":\"m7\",\"event\":\"message\",\"message\":\"{\\\"art\\\":\\\"nachricht\\\",\\\"projekt\\\":3}\"}\n")
        .await
        .unwrap();

    assert!(matches!(
        naechster(&mut vorgaenge).await,
        Vorgang::Verbunden
    ));
    match naechster(&mut vorgaenge).await {
        Vorgang::Ereignis(e) => {
            assert_eq!(e.id, "m7");
            assert_eq!(e.signal, Signal::Nachricht { projekt: Some(3) });
        }
        anders => panic!("{anders:?}"),
    }

    drop(s); // Server beendet
    match naechster(&mut vorgaenge).await {
        Vorgang::Abriss {
            grund: Leitungsfehler::Beendet,
            pause,
        } => {
            // Die Leitung stand (open kam) -- also von vorne: 1–2 s.
            assert!(pause <= Duration::from_secs(2), "{pause:?}");
        }
        anders => panic!("{anders:?}"),
    }

    let (anfrage, _s) = verbindungen.recv().await.unwrap();
    assert_eq!(anfrage, "GET /upGeheim/json?since=m7 HTTP/1.1");
    h.abort();
}

#[tokio::test]
async fn stille_gilt_als_tote_leitung() {
    let (basis, mut verbindungen) = server().await;
    let (_netz_tx, netz) = watch::channel(true);
    let mut leitung = Leitung::new(&basis, "upStill");
    leitung.lese_frist = Duration::from_millis(200);
    let (mut vorgaenge, h) = starten(leitung, netz);

    let (_, mut s) = verbindungen.recv().await.unwrap();
    s.write_all(b"{\"id\":\"o\",\"event\":\"open\"}\n")
        .await
        .unwrap();
    assert!(matches!(
        naechster(&mut vorgaenge).await,
        Vorgang::Verbunden
    ));
    assert!(matches!(
        naechster(&mut vorgaenge).await,
        Vorgang::Abriss {
            grund: Leitungsfehler::Stille(_),
            ..
        }
    ));
    h.abort();
}

#[tokio::test]
async fn ohne_netz_wird_nicht_gewaehlt_und_ein_wechsel_verbindet_sofort_neu() {
    let (basis, mut verbindungen) = server().await;
    let (netz_tx, netz) = watch::channel(false);
    let (mut vorgaenge, h) = starten(Leitung::new(&basis, "upNetz"), netz);

    let verfrueht = tokio::time::timeout(Duration::from_millis(300), verbindungen.recv()).await;
    assert!(verfrueht.is_err(), "ohne Netz darf niemand anrufen");

    netz_tx.send(true).unwrap();
    let (_, mut s) = verbindungen.recv().await.unwrap();
    s.write_all(b"{\"id\":\"o\",\"event\":\"open\"}\n")
        .await
        .unwrap();
    assert!(matches!(
        naechster(&mut vorgaenge).await,
        Vorgang::Verbunden
    ));

    // WLAN -> Mobilfunk: dieselbe Meldung „Netz da", aber ein Wechsel.
    netz_tx.send(true).unwrap();
    match naechster(&mut vorgaenge).await {
        Vorgang::Abriss {
            grund: Leitungsfehler::Netzwechsel,
            pause,
        } => assert_eq!(pause, openany_meldungen::NACH_NETZWECHSEL),
        anders => panic!("{anders:?}"),
    }
    let neu = tokio::time::timeout(Duration::from_secs(2), verbindungen.recv()).await;
    assert!(neu.is_ok(), "nach dem Wechsel sofort neu verbinden");
    h.abort();
}

#[tokio::test]
async fn fehlerstatus_wird_als_grund_gemeldet() {
    let horcher = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let basis = format!("http://{}", horcher.local_addr().unwrap());
    tokio::spawn(async move {
        let (mut s, _) = horcher.accept().await.unwrap();
        let mut b = [0u8; 1024];
        let _ = s.read(&mut b).await;
        s.write_all(b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\n\r\n")
            .await
            .unwrap();
    });
    let (_netz_tx, netz) = watch::channel(true);
    let (mut vorgaenge, h) = starten(Leitung::new(&basis, "fremd"), netz);
    assert!(matches!(
        naechster(&mut vorgaenge).await,
        Vorgang::Abriss {
            grund: Leitungsfehler::Status(403),
            ..
        }
    ));
    h.abort();
}

#[tokio::test]
async fn der_grund_eines_abrisses_nennt_das_thema_nicht() {
    // Niemand horcht auf Port 1: Die Verbindung scheitert sofort, und
    // reqwest haengt von sich aus die ganze URL an den Fehler.
    let (_netz_tx, netz) = watch::channel(true);
    let (mut vorgaenge, h) = starten(Leitung::new("http://127.0.0.1:1", "upGanzGeheim"), netz);
    match naechster(&mut vorgaenge).await {
        Vorgang::Abriss { grund, .. } => {
            let text = grund.to_string();
            assert!(!text.contains("upGanzGeheim"), "Thema im Protokoll: {text}");
        }
        anders => panic!("{anders:?}"),
    }
    h.abort();
}
