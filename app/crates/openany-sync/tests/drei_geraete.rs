//! Drei eigene Geraete -- Telefon, Tablet, PC --, die sich nie alle
//! gleichzeitig treffen.
//!
//! Die Fragen: Kommt an, was nur ueber einen Umweg reisen kann? Kommt das
//! Ganze zur Ruhe, oder laeuft etwas im Kreis? Und geht bei gleichzeitigem
//! Schreiben nichts verloren?

mod gemeinsam;

use gemeinsam::{abgleichen, inhalt, nichts, notiz, Geraet};
use openany_client::Art;
use openany_store::{Kontakt, Protokoll};

/// Alle Paare in beiden Richtungen, bis eine ganze Runde nichts mehr tut.
/// Gibt die Zahl der Runden zurueck; bricht nach zehn ab.
async fn bis_zur_ruhe(geraete: &[&Geraet]) -> usize {
    for runde in 1..=10 {
        let mut ruhig = true;
        for a in geraete {
            for b in geraete {
                if a.name != b.name && !nichts(&abgleichen(a, b).await) {
                    ruhig = false;
                }
            }
        }
        if ruhig {
            return runde;
        }
    }
    panic!("Nach zehn Runden noch nicht zur Ruhe gekommen -- etwas laeuft im Kreis.");
}

#[tokio::test]
async fn was_nur_ueber_einen_umweg_reisen_kann_kommt_an() {
    let telefon = Geraet::neu("Telefon");
    let tablet = Geraet::neu("Tablet");
    let pc = Geraet::neu("PC");

    notiz(&telefon, "zk-1", "Vom Telefon");

    // Das Telefon trifft nur das Tablet, das Tablet spaeter nur den PC.
    abgleichen(&tablet, &telefon).await;
    abgleichen(&pc, &tablet).await;

    assert_eq!(inhalt(&pc, "zk-1"), "Vom Telefon");
}

#[tokio::test]
async fn auch_wenn_das_mittlere_geraet_nur_schiebt() {
    // Diesmal tippt jeweils das Geraet, das etwas HAT -- der Weg geht also
    // ueber Schieben und Auskunft, nicht ueber Ziehen.
    let telefon = Geraet::neu("Telefon");
    let tablet = Geraet::neu("Tablet");
    let pc = Geraet::neu("PC");

    notiz(&telefon, "zk-1", "Vom Telefon");
    abgleichen(&telefon, &tablet).await;
    abgleichen(&tablet, &pc).await;

    assert_eq!(inhalt(&pc, "zk-1"), "Vom Telefon");
}

#[tokio::test]
async fn nichts_laeuft_im_kreis() {
    let telefon = Geraet::neu("Telefon");
    let tablet = Geraet::neu("Tablet");
    let pc = Geraet::neu("PC");

    notiz(&telefon, "zk-1", "Eins");
    notiz(&tablet, "zk-2", "Zwei");
    notiz(&pc, "zk-3", "Drei");

    let runden = bis_zur_ruhe(&[&telefon, &tablet, &pc]).await;
    assert!(runden <= 3, "{runden} Runden");

    for g in [&telefon, &tablet, &pc] {
        assert_eq!(g.speicher.notizen().unwrap().len(), 3, "{}", g.name);
    }
}

#[tokio::test]
async fn ein_grabstein_laeuft_nicht_im_kreis() {
    let telefon = Geraet::neu("Telefon");
    let tablet = Geraet::neu("Tablet");
    let pc = Geraet::neu("PC");

    notiz(&telefon, "zk-1", "Weg damit");
    notiz(&telefon, "zk-2", "Ganz weg");
    bis_zur_ruhe(&[&telefon, &tablet, &pc]).await;

    telefon
        .speicher
        .papierkorb(&Art::Notiz, "zk-1", Protokoll::Merken)
        .unwrap();
    telefon
        .speicher
        .loeschen(&Art::Notiz, "zk-2", Protokoll::Merken)
        .unwrap();

    abgleichen(&tablet, &telefon).await;
    abgleichen(&pc, &tablet).await;
    assert!(pc.speicher.notiz("zk-1").unwrap().unwrap().im_papierkorb());
    assert!(pc.speicher.notiz("zk-2").unwrap().is_none());

    let runden = bis_zur_ruhe(&[&telefon, &tablet, &pc]).await;
    assert!(runden <= 3, "{runden} Runden");
}

#[tokio::test]
async fn ueber_den_umweg_bekommen_und_dann_direkt_ist_kein_konflikt() {
    // Der PC hat die Notiz nur vom Tablet. Das Telefon schreibt weiter und
    // trifft den PC dann zum ersten Mal direkt -- gemeinsam hatten die beiden
    // nie etwas, und doch stammt die neue Fassung von dem ab, was der PC hat.
    let telefon = Geraet::neu("Telefon");
    let tablet = Geraet::neu("Tablet");
    let pc = Geraet::neu("PC");

    notiz(&telefon, "zk-1", "Erster Stand");
    abgleichen(&tablet, &telefon).await;
    abgleichen(&pc, &tablet).await;

    notiz(&telefon, "zk-1", "Am Telefon weitergeschrieben");
    let bericht = abgleichen(&telefon, &pc).await;

    assert_eq!(bericht.konflikte, 0);
    assert_eq!(inhalt(&pc, "zk-1"), "Am Telefon weitergeschrieben");
    assert_eq!(pc.speicher.notizen().unwrap().len(), 1);
}

#[tokio::test]
async fn gleichzeitig_geschrieben_geht_nichts_verloren() {
    let telefon = Geraet::neu("Telefon");
    let tablet = Geraet::neu("Tablet");
    let pc = Geraet::neu("PC");

    notiz(&telefon, "zk-1", "Gemeinsam");
    bis_zur_ruhe(&[&telefon, &tablet, &pc]).await;

    notiz(&telefon, "zk-1", "Telefon-Fassung");
    notiz(&pc, "zk-1", "PC-Fassung");

    bis_zur_ruhe(&[&telefon, &tablet, &pc]).await;

    for g in [&telefon, &tablet, &pc] {
        let inhalte: Vec<String> = g
            .speicher
            .notizen()
            .unwrap()
            .into_iter()
            .map(|n| n.inhalt)
            .collect();
        assert!(
            inhalte.iter().any(|i| i == "Telefon-Fassung")
                && inhalte.iter().any(|i| i == "PC-Fassung"),
            "{}: {inhalte:?}",
            g.name
        );
    }
}

#[tokio::test]
async fn ein_foto_kommt_auch_ueber_das_schieben_eines_dritten_an() {
    let telefon = Geraet::neu("Telefon");
    let tablet = Geraet::neu("Tablet");
    let pc = Geraet::neu("PC");

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

    abgleichen(&telefon, &tablet).await;
    // Das Tablet hat das Foto nicht selbst gesetzt -- und schickt es trotzdem
    // weiter, denn der PC kann es sonst nirgends holen.
    abgleichen(&tablet, &pc).await;

    assert_eq!(
        pc.speicher.kontaktfoto("c-1").unwrap().as_deref(),
        Some(&b"bild"[..])
    );
    let runden = bis_zur_ruhe(&[&telefon, &tablet, &pc]).await;
    assert!(runden <= 3, "{runden} Runden");
}
