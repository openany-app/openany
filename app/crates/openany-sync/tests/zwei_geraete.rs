//! Zwei Geraete, kein Server: Tablet und Telefon gleichen sich gegenseitig
//! ab -- jedes mit seinem eigenen Speicher, jedes mal als Laeufer, mal als
//! Auskunft.
//!
//! Die Leitung fehlt hier absichtlich (die steht in `openany-nahbereich`).
//! Was hier geprueft wird, sind die Regeln: dass es gleichgueltig ist, wer
//! tippt, dass nichts im Kreis laeuft, und dass beidseitig Geschriebenes
//! nicht verlorengeht.

mod gemeinsam;

use gemeinsam::{abgleichen, inhalt, nichts, notiz, Geraet};
use openany_client::{Art, Terminfelder};
use openany_store::{Kontakt, Protokoll, Termin, Weg};

#[tokio::test]
async fn notiz_termin_und_kontakt_kommen_an_und_der_zweite_lauf_meldet_nichts() {
    let telefon = Geraet::neu("Telefon");
    let tablet = Geraet::neu("Tablet");

    notiz(&telefon, "20260915100000", "Milch");
    telefon
        .speicher
        .termin_schreiben(
            &Termin {
                uuid: "t-1".into(),
                kalender_uuid: "k-1".into(),
                felder: Terminfelder {
                    titel: "Zahnarzt".into(),
                    beginn: "2026-09-20T09:00:00".into(),
                    ende: "2026-09-20T09:30:00".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            Protokoll::Merken,
        )
        .unwrap();
    tablet
        .speicher
        .kontakt_schreiben(
            &Kontakt {
                uuid: "c-1".into(),
                anzeigename: "Hannah".into(),
                wege: vec![Weg {
                    art: "phone".into(),
                    beschriftung: Some("mobil".into()),
                    wert: "0171".into(),
                }],
                ..Default::default()
            },
            Protokoll::Merken,
        )
        .unwrap();

    let erster = abgleichen(&tablet, &telefon).await;
    assert_eq!((erster.gezogen, erster.geschoben), (2, 1));

    assert_eq!(inhalt(&tablet, "20260915100000"), "Milch");
    assert_eq!(
        tablet.speicher.termin("t-1").unwrap().unwrap().felder.titel,
        "Zahnarzt"
    );
    let k = telefon.speicher.kontakt("c-1").unwrap().unwrap();
    assert_eq!(k.wege[0].wert, "0171");

    assert!(nichts(&abgleichen(&tablet, &telefon).await), "zweiter Lauf");
    assert!(
        nichts(&abgleichen(&telefon, &tablet).await),
        "und auch vom anderen Geraet aus nichts -- die Marken wurden gemeldet"
    );
}

#[tokio::test]
async fn beidseitig_geaendert_ergibt_eine_kopie_und_das_original_bleibt() {
    let telefon = Geraet::neu("Telefon");
    let tablet = Geraet::neu("Tablet");

    notiz(&telefon, "zk-1", "Gemeinsam");
    abgleichen(&tablet, &telefon).await;

    notiz(&telefon, "zk-1", "Vom Telefon");
    notiz(&tablet, "zk-1", "Vom Tablet");

    let bericht = abgleichen(&tablet, &telefon).await;
    assert_eq!(bericht.konflikte, 1);
    assert_eq!(
        inhalt(&tablet, "zk-1"),
        "Vom Tablet",
        "Original unangetastet"
    );

    let kopie = tablet
        .speicher
        .notizen()
        .unwrap()
        .into_iter()
        .find(|n| n.titel.contains("Konflikt"))
        .expect("die Kopie");
    assert_eq!(kopie.inhalt, "Vom Telefon");
    assert!(
        kopie.titel.contains("Konflikt Telefon"),
        "nach dem Namen, nicht dem Fingerabdruck: {}",
        kopie.titel
    );

    // Die Kopie wandert zum Telefon, und danach ist Ruhe.
    abgleichen(&tablet, &telefon).await;
    abgleichen(&telefon, &tablet).await;
    assert_eq!(telefon.speicher.notizen().unwrap().len(), 2);
    assert!(nichts(&abgleichen(&telefon, &tablet).await));
    assert!(nichts(&abgleichen(&tablet, &telefon).await));
}

#[tokio::test]
async fn geholt_und_geaendert_ist_kein_konflikt_egal_wer_tippt() {
    // Der Fall, fuer den es `ursprung_waehlen` gibt: Wer nur ausgeliefert
    // hat, weiss nicht, dass der Stand jetzt ein gemeinsamer ist.
    let telefon = Geraet::neu("Telefon");
    let tablet = Geraet::neu("Tablet");

    notiz(&telefon, "zk-1", "Erster Stand");
    abgleichen(&tablet, &telefon).await;

    // Das Tablet aendert -- und dieses Mal tippt das TELEFON.
    notiz(&tablet, "zk-1", "Auf dem Tablet weitergeschrieben");
    let bericht = abgleichen(&telefon, &tablet).await;

    assert_eq!(bericht.konflikte, 0);
    assert_eq!(inhalt(&telefon, "zk-1"), "Auf dem Tablet weitergeschrieben");

    // Und zurueck: das Telefon aendert, das Tablet tippt.
    notiz(&telefon, "zk-1", "Wieder am Telefon");
    let bericht = abgleichen(&tablet, &telefon).await;
    assert_eq!(bericht.konflikte, 0);
    assert_eq!(inhalt(&tablet, "zk-1"), "Wieder am Telefon");
    assert_eq!(tablet.speicher.notizen().unwrap().len(), 1);
}

#[tokio::test]
async fn ein_alter_kontakt_ueberschreibt_keinen_neueren() {
    // Ohne gemeldete Marken holte das Telefon den Kontakt, den das Tablet
    // eben erst hinuebergeschoben hat, spaeter noch einmal -- und
    // ueberschriebe damit, was es seither selbst geaendert hat.
    let telefon = Geraet::neu("Telefon");
    let tablet = Geraet::neu("Tablet");

    let kontakt = |name: &str| Kontakt {
        uuid: "c-1".into(),
        anzeigename: name.into(),
        ..Default::default()
    };

    tablet
        .speicher
        .kontakt_schreiben(&kontakt("Hanna"), Protokoll::Merken)
        .unwrap();
    abgleichen(&tablet, &telefon).await;

    telefon
        .speicher
        .kontakt_schreiben(&kontakt("Hannah"), Protokoll::Merken)
        .unwrap();
    abgleichen(&telefon, &tablet).await;

    assert_eq!(
        telefon
            .speicher
            .kontakt("c-1")
            .unwrap()
            .unwrap()
            .anzeigename,
        "Hannah"
    );
    assert_eq!(
        tablet.speicher.kontakt("c-1").unwrap().unwrap().anzeigename,
        "Hannah"
    );
}

#[tokio::test]
async fn ein_foto_reist_einmal_und_nicht_im_kreis() {
    let telefon = Geraet::neu("Telefon");
    let tablet = Geraet::neu("Tablet");

    telefon
        .speicher
        .kontakt_schreiben(
            &Kontakt {
                uuid: "c-1".into(),
                anzeigename: "Hannah".into(),
                ..Default::default()
            },
            Protokoll::Merken,
        )
        .unwrap();
    telefon.speicher.kontaktfoto_setzen("c-1", b"bild").unwrap();

    abgleichen(&tablet, &telefon).await;
    assert_eq!(
        tablet.speicher.kontaktfoto("c-1").unwrap().as_deref(),
        Some(&b"bild"[..])
    );
    assert!(nichts(&abgleichen(&telefon, &tablet).await));

    // Andersherum ueber das Schieben: das Tablet setzt ein neues Bild.
    tablet.speicher.kontaktfoto_setzen("c-1", b"neu").unwrap();
    abgleichen(&tablet, &telefon).await;
    assert_eq!(
        telefon.speicher.kontaktfoto("c-1").unwrap().as_deref(),
        Some(&b"neu"[..])
    );
    assert_eq!(
        telefon.speicher.kontakt("c-1").unwrap().unwrap().foto,
        tablet.speicher.kontakt("c-1").unwrap().unwrap().foto,
        "derselbe Abdruck auf beiden Seiten -- sonst holte jeder Lauf neu"
    );
    assert!(nichts(&abgleichen(&telefon, &tablet).await));
    assert!(nichts(&abgleichen(&tablet, &telefon).await));
}

#[tokio::test]
async fn papierkorb_bleibt_papierkorb_in_beide_richtungen() {
    let telefon = Geraet::neu("Telefon");
    let tablet = Geraet::neu("Tablet");

    notiz(&telefon, "zk-1", "Weg damit");
    abgleichen(&tablet, &telefon).await;

    tablet
        .speicher
        .papierkorb(&Art::Notiz, "zk-1", Protokoll::Merken)
        .unwrap();
    abgleichen(&tablet, &telefon).await;
    assert!(telefon
        .speicher
        .notiz("zk-1")
        .unwrap()
        .unwrap()
        .im_papierkorb());

    assert!(nichts(&abgleichen(&telefon, &tablet).await));
}
