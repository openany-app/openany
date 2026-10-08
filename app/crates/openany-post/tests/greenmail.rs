//! Der ganze Weg gegen einen echten (Test-)Mailserver: senden, abholen,
//! gelesen markieren, ablegen, löschen.
//!
//! Braucht GreenMail auf localhost (SMTP 3025, IMAP 3143) und das Merkmal
//! `testserver`:
//!
//!   docker run -d --rm --name greenmail-test -p 127.0.0.1:3025:3025 \
//!     -p 127.0.0.1:3143:3143 -e GREENMAIL_OPTS='-Dgreenmail.setup.test.smtp \
//!     -Dgreenmail.setup.test.imap -Dgreenmail.auth.disabled \
//!     -Dgreenmail.hostname=0.0.0.0' greenmail/standalone:2.1.2
//!   cargo test -p openany-post --features testserver -- --ignored
#![cfg(feature = "testserver")]

use openany_post::*;

fn konto(name: &str) -> Konto {
    let server = |port| Server {
        host: "127.0.0.1".into(),
        port,
        sicherheit: Sicherheit::Klartext,
    };
    Konto {
        adresse: format!("{name}@localhost"),
        anzeigename: name.into(),
        imap: server(3143),
        smtp: server(3025),
        benutzer: format!("{name}@localhost"),
        passwort: "egal".into(),
    }
}

#[tokio::test]
#[ignore = "braucht GreenMail auf localhost"]
async fn senden_abholen_gelesen_loeschen() {
    let eindeutig = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let tiffy = konto(&format!("tiffy{eindeutig}"));
    let ferdinand = konto(&format!("ferdinand{eindeutig}"));

    // Anmelden und Ordner finden (GreenMail hat nur INBOX).
    let ordner = pruefen(&tiffy).await.expect("Anmeldung");
    assert_eq!(ordner.eingang, "INBOX");

    // Ferdinand schreibt Tiffy, mit Anhang.
    let gesendet = senden(
        &ferdinand,
        &Entwurf {
            an: vec![tiffy.adresse.clone()],
            betreff: "Grüße aus dem Test".into(),
            text: "Hallo Tiffy!".into(),
            anhaenge: vec![AnhangDaten {
                name: "plan.pdf".into(),
                mime: "application/pdf".into(),
                daten: b"%PDF-1.4\n".to_vec(),
            }],
            ..Default::default()
        },
    )
    .await
    .expect("senden");
    assert!(!gesendet.roh.is_empty());

    // Tiffy holt ab.
    let erst = abholen(&tiffy, &[], 50).await.expect("abholen");
    assert_eq!(erst.mails.len(), 1);
    let roh = &erst.mails[0];
    assert!(!roh.gelesen, "BODY.PEEK darf nicht als gelesen markieren");
    let mail = mail_lesen(&roh.daten).unwrap();
    assert_eq!(mail.betreff, "Grüße aus dem Test");
    assert_eq!(mail.message_id, gesendet.message_id);
    assert_eq!(mail.von.unwrap().adresse, ferdinand.adresse);
    assert_eq!(mail.anhaenge[0].daten, b"%PDF-1.4\n");

    // Mit dem Stand: nichts Neues.
    let staende: Vec<Stand> = erst.staende.clone();
    let zweit = abholen(&tiffy, &staende, 50).await.expect("abholen 2");
    assert!(zweit.mails.is_empty());

    // Gelesen markieren -- beim nächsten Lesen von vorn kommt sie als gelesen.
    als_gelesen(&tiffy, "INBOX", roh.uid)
        .await
        .expect("gelesen");
    let neu_gelesen = abholen(&tiffy, &[], 50).await.unwrap();
    assert!(neu_gelesen.mails[0].gelesen);

    // Ablegen (wie „Gesendet") -- kommt als neue Mail mit dem Stand.
    ablegen(&tiffy, "INBOX", &gesendet.roh)
        .await
        .expect("ablegen");
    let dritt = abholen(&tiffy, &staende, 50).await.unwrap();
    assert_eq!(dritt.mails.len(), 1);

    // Löschen ohne Papierkorb-Ordner: \Deleted + EXPUNGE.
    loeschen(&tiffy, "INBOX", roh.uid, None)
        .await
        .expect("loeschen");
    let rest = abholen(&tiffy, &[], 50).await.unwrap();
    assert_eq!(rest.mails.len(), 1, "nur die abgelegte bleibt");
    assert_ne!(rest.mails[0].uid, roh.uid);

    // Die abgelegte an ihrer Message-ID finden und löschen.
    let n = loeschen_nach_id(&tiffy, "INBOX", &gesendet.message_id, None)
        .await
        .expect("loeschen_nach_id");
    assert_eq!(n, 1);
    assert!(abholen(&tiffy, &[], 50).await.unwrap().mails.is_empty());
    let n = loeschen_nach_id(&tiffy, "INBOX", &gesendet.message_id, None)
        .await
        .unwrap();
    assert_eq!(n, 0, "nichts mehr da");
}

#[tokio::test]
#[ignore = "braucht GreenMail auf localhost"]
async fn warten_meldet_neue_mail_und_laeuft_sonst_ab() {
    let eindeutig = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let tiffy = konto(&format!("tiffy-idle{eindeutig}"));
    let ferdinand = konto(&format!("ferdinand-idle{eindeutig}"));
    pruefen(&tiffy).await.expect("Anmeldung");

    // Nichts kommt: nach der Frist `false`.
    let still = warten(&tiffy, "INBOX", std::time::Duration::from_secs(2))
        .await
        .expect("warten");
    assert!(!still);

    // Eine Mail kommt, während gewartet wird: `true`, lange vor der Frist.
    let wache = tokio::spawn({
        let tiffy = tiffy.clone();
        async move { warten(&tiffy, "INBOX", std::time::Duration::from_secs(30)).await }
    });
    tokio::time::sleep(std::time::Duration::from_millis(800)).await;
    let beginn = std::time::Instant::now();
    senden(
        &ferdinand,
        &Entwurf {
            an: vec![tiffy.adresse.clone()],
            betreff: "Weck mich".into(),
            text: "Hallo".into(),
            ..Default::default()
        },
    )
    .await
    .expect("senden");
    assert!(wache.await.unwrap().expect("warten"));
    assert!(beginn.elapsed() < std::time::Duration::from_secs(10));
}
