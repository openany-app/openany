//! Dateien zwischen eigenen Geraeten: erst die Uebersicht, dann -- nach der
//! Regel des Geraets -- der Inhalt.

mod gemeinsam;

use gemeinsam::{abgleichen, gegenueber, nichts, Geraet};
use openany_client::{Delta, Eintrag, Ergebnis, OpenanyError};
use openany_store::{Datei, Protokoll};
use openany_sync::{fehlende_inhalte_holen, inhalt_holen, Gegenstelle, InhaltFehler};
use sha2::Digest;

const VIEL: u64 = u64::MAX / 4;

fn ablegen(g: &Geraet, bytes: &[u8]) -> (String, u64) {
    let mut l = g.inhalte.ladung().unwrap();
    l.schreiben(bytes).unwrap();
    g.inhalte.ablegen(l).unwrap()
}

fn ordner(g: &Geraet, uuid: &str, name: &str) {
    g.speicher
        .datei_schreiben(
            &Datei {
                uuid: uuid.into(),
                zone: "documents".into(),
                ist_ordner: true,
                name: name.into(),
                ..Default::default()
            },
            Protokoll::Merken,
        )
        .unwrap();
}

fn datei(g: &Geraet, uuid: &str, eltern: &str, name: &str, bytes: &[u8]) -> String {
    let (abdruck, groesse) = ablegen(g, bytes);
    g.speicher
        .datei_schreiben(
            &Datei {
                uuid: uuid.into(),
                zone: "documents".into(),
                eltern: Some(eltern.into()),
                name: name.into(),
                groesse,
                mime: "application/pdf".into(),
                abdruck: Some(abdruck.clone()),
                ..Default::default()
            },
            Protokoll::Merken,
        )
        .unwrap();
    abdruck
}

fn fehlend(g: &Geraet) -> Vec<(String, u64)> {
    g.speicher
        .lebendige_dateien()
        .unwrap()
        .into_iter()
        .filter_map(|d| Some((d.abdruck?, d.groesse)))
        .filter(|(a, _)| !g.inhalte.hat(a))
        .collect()
}

#[tokio::test]
async fn erst_die_uebersicht_dann_der_inhalt() {
    let telefon = Geraet::neu("Telefon");
    let tablet = Geraet::neu("Tablet");

    ordner(&telefon, "o-1", "Steuer");
    let abdruck = datei(&telefon, "d-1", "o-1", "Beleg.pdf", b"%PDF-1.7 Beleg");

    let bericht = abgleichen(&tablet, &telefon).await;
    assert_eq!(bericht.gezogen, 2);

    let d = tablet.speicher.datei("d-1").unwrap().unwrap();
    assert_eq!(d.name, "Beleg.pdf");
    assert_eq!(d.eltern.as_deref(), Some("o-1"));
    assert!(
        !tablet.inhalte.hat(&abdruck),
        "der Inhalt kommt nicht von selbst"
    );

    let geholt = fehlende_inhalte_holen(
        &tablet.inhalte,
        &gegenueber(&tablet, &telefon),
        &fehlend(&tablet),
        || VIEL,
        VIEL,
    )
    .await;
    assert_eq!(geholt.geholt, 1, "{geholt:?}");
    assert_eq!(
        tablet.inhalte.lesen(&abdruck, 0, 100).unwrap(),
        b"%PDF-1.7 Beleg"
    );

    assert!(nichts(&abgleichen(&tablet, &telefon).await));
    assert!(nichts(&abgleichen(&telefon, &tablet).await));
}

#[tokio::test]
async fn gross_und_stueckweise() {
    let telefon = Geraet::neu("Telefon");
    let tablet = Geraet::neu("Tablet");

    // Groesser als ein Stueck (4 MB), mit einem Muster, das jeder Versatzfehler zerstoert.
    let bytes: Vec<u8> = (0..9_500_000u32).map(|i| (i % 251) as u8).collect();
    let (abdruck, groesse) = ablegen(&telefon, &bytes);

    inhalt_holen(
        &tablet.inhalte,
        &gegenueber(&tablet, &telefon),
        &abdruck,
        groesse,
    )
    .await
    .unwrap();
    assert!(tablet.inhalte.hat(&abdruck));
}

#[tokio::test]
async fn ohne_platz_wird_nichts_geholt() {
    let telefon = Geraet::neu("Telefon");
    let tablet = Geraet::neu("Tablet");
    ordner(&telefon, "o-1", "Fotos");
    datei(&telefon, "d-1", "o-1", "gross.bin", &[7u8; 2000]);
    abgleichen(&tablet, &telefon).await;

    // 10 GB Datentraeger, 1,5 GB frei: Die Reserve (1 GB) passt, aber nur,
    // wenn die Datei klein ist -- hier ist "frei" kuenstlich knapp.
    let gb = 1024 * 1024 * 1024;
    let bericht = fehlende_inhalte_holen(
        &tablet.inhalte,
        &gegenueber(&tablet, &telefon),
        &fehlend(&tablet),
        || gb + 1000,
        10 * gb,
    )
    .await;
    assert_eq!(bericht.zu_wenig_platz, 1);
    assert_eq!(bericht.geholt, 0);
}

/// Eine Gegenstelle, die unterwegs ein Byte verdreht.
struct Faelscher;

#[async_trait::async_trait]
impl Gegenstelle for Faelscher {
    fn basis(&self) -> &str {
        "nah:Faelscher"
    }
    async fn delta(&self, _: i64) -> Result<Delta, OpenanyError> {
        unreachable!()
    }
    async fn notiztext(&self, _: &str) -> Result<String, OpenanyError> {
        unreachable!()
    }
    async fn anwenden(&self, _: &[Eintrag]) -> Result<Vec<Ergebnis>, OpenanyError> {
        unreachable!()
    }
    async fn inhalt(&self, _: &str, _: u64, _: u64) -> Result<Vec<u8>, OpenanyError> {
        Ok(b"echT".to_vec())
    }
}

#[tokio::test]
async fn ein_inhalt_der_nicht_passt_wird_verworfen() {
    let tablet = Geraet::neu("Tablet");
    let echt = sha2::Sha256::digest(b"echt")
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();

    let e = inhalt_holen(&tablet.inhalte, &Faelscher, &echt, 4)
        .await
        .unwrap_err();

    assert!(matches!(e, InhaltFehler::FalscherAbdruck), "{e:?}");
    assert!(!tablet.inhalte.hat(&echt));
    assert_eq!(
        tablet.inhalte.belegt(),
        0,
        "auch die verdrehte Fassung ist fort"
    );
}

#[tokio::test]
async fn ein_ordner_im_papierkorb_nimmt_seinen_inhalt_mit() {
    let telefon = Geraet::neu("Telefon");
    let tablet = Geraet::neu("Tablet");
    ordner(&telefon, "o-1", "Alt");
    datei(&telefon, "d-1", "o-1", "a.pdf", b"a");
    abgleichen(&tablet, &telefon).await;

    telefon
        .speicher
        .datei_papierkorb("o-1", Protokoll::Merken)
        .unwrap();
    abgleichen(&tablet, &telefon).await;

    assert!(tablet
        .speicher
        .datei("o-1")
        .unwrap()
        .unwrap()
        .papierkorb_at
        .is_some());
    assert!(tablet
        .speicher
        .datei("d-1")
        .unwrap()
        .unwrap()
        .papierkorb_at
        .is_some());
    assert!(nichts(&abgleichen(&telefon, &tablet).await));
}

#[tokio::test]
async fn ueber_ein_drittes_geraet() {
    let telefon = Geraet::neu("Telefon");
    let tablet = Geraet::neu("Tablet");
    let pc = Geraet::neu("PC");
    ordner(&telefon, "o-1", "Urlaub");
    let abdruck = datei(&telefon, "d-1", "o-1", "Plan.pdf", b"Plan");

    abgleichen(&telefon, &pc).await;
    fehlende_inhalte_holen(
        &pc.inhalte,
        &gegenueber(&pc, &telefon),
        &fehlend(&pc),
        || VIEL,
        VIEL,
    )
    .await;
    abgleichen(&pc, &tablet).await;
    let bericht = fehlende_inhalte_holen(
        &tablet.inhalte,
        &gegenueber(&tablet, &pc),
        &fehlend(&tablet),
        || VIEL,
        VIEL,
    )
    .await;

    assert_eq!(bericht.geholt, 1);
    assert!(tablet.inhalte.hat(&abdruck));
}

#[tokio::test]
async fn alben_und_bilder_reisen_mit_vorschau() {
    let telefon = Geraet::neu("Telefon");
    let tablet = Geraet::neu("Tablet");
    let (original, groesse) = ablegen(&telefon, b"JPEG-Original");
    let (vorschau, _) = ablegen(&telefon, b"JPEG-klein");
    telefon
        .speicher
        .album_schreiben(
            &openany_store::Album {
                uuid: "a-1".into(),
                name: "Urlaub".into(),
                ..Default::default()
            },
            Protokoll::Merken,
        )
        .unwrap();
    telefon
        .speicher
        .bild_schreiben(
            &openany_store::Bild {
                uuid: "b-1".into(),
                album: Some("a-1".into()),
                name: "strand.jpg".into(),
                mime: "image/jpeg".into(),
                groesse,
                abdruck: Some(original),
                vorschau: Some(vorschau.clone()),
                exif: r#"{"date":"2026:08:01 10:00:00","gps":{"lat":54.1,"lng":7.9}}"#.into(),
                ..Default::default()
            },
            Protokoll::Merken,
        )
        .unwrap();

    assert_eq!(abgleichen(&tablet, &telefon).await.gezogen, 2);
    let b = tablet.speicher.bild("b-1").unwrap().unwrap();
    assert_eq!(b.album.as_deref(), Some("a-1"));
    assert!(b.exif.contains("54.1"), "EXIF reist mit: {}", b.exif);

    // Die Vorschau holt jedes Geraet -- hier stellvertretend direkt.
    fehlende_inhalte_holen(
        &tablet.inhalte,
        &gegenueber(&tablet, &telefon),
        &[(vorschau.clone(), 10)],
        || VIEL,
        VIEL,
    )
    .await;
    assert!(tablet.inhalte.hat(&vorschau));

    assert!(nichts(&abgleichen(&tablet, &telefon).await));
    assert!(nichts(&abgleichen(&telefon, &tablet).await));

    telefon
        .speicher
        .album_papierkorb("a-1", Protokoll::Merken)
        .unwrap();
    abgleichen(&tablet, &telefon).await;
    assert!(tablet
        .speicher
        .bild("b-1")
        .unwrap()
        .unwrap()
        .papierkorb_at
        .is_some());
}
