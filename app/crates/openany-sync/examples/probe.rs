//! Die Probe auf die ganze Strecke -- gegen eine echte openany-Instanz.
//!
//! Die 88 Tests im Workspace laufen ohne Server; sie beweisen, dass dieses
//! Programm mit sich selbst uebereinstimmt. **Diese Probe beweist das
//! andere**: dass es mit openany uebereinstimmt. Beides ist noetig, und das
//! zweite laesst sich nicht nachbauen -- ein Pappserver, den ich selbst
//! geschrieben habe, sagt genau das, was ich erwartet habe.
//!
//! ```bash
//! export OPENANY_BASIS=https://openany.localhost
//! export OPENANY_SCHLUESSEL='15|…'          # ein Geraeteschluessel
//! export OPENANY_CA=~/anyx/infra/caddy/lokale-ca.crt   # nur lokal noetig
//! export OPENANY_SPEICHER=/tmp/probe.sqlite
//! cargo run -p openany-sync --example probe
//! ```
//!
//! Zweimal hintereinander aufgerufen muss der zweite Lauf **nichts** ziehen
//! und **nichts** schieben. Tut er es doch, ist irgendwo eine Verabredung
//! zwischen den beiden Seiten gebrochen -- und genau das faellt sonst nirgends
//! auf, weil nichts fehlschlaegt.

use openany_client::OpenanyClient;
use openany_store::{Notiz, Protokoll, Speicher};
use openany_sync::Laeufer;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let basis = std::env::var("OPENANY_BASIS")?;
    // Entweder der Schluessel selbst -- oder die Datei, in die `koppeln`
    // ihn gelegt hat. Die zweite Form ist die interessantere: Sie schliesst
    // die Kette von der Kopplung bis zum Abgleich, ohne dass irgendwo ein
    // Geheimnis durch eine Kommandozeile geht (und damit in die
    // Shell-Geschichte).
    let schluessel = match std::env::var("OPENANY_SCHLUESSEL") {
        Ok(s) => s,
        Err(_) => std::fs::read_to_string(
            std::env::var("OPENANY_SCHLUESSEL_DATEI")
                .map_err(|_| "Weder OPENANY_SCHLUESSEL noch OPENANY_SCHLUESSEL_DATEI gesetzt.")?,
        )?
        .trim()
        .to_string(),
    };
    let pfad = std::env::var("OPENANY_SPEICHER").unwrap_or_else(|_| "probe.sqlite".into());

    let ca = match std::env::var("OPENANY_CA") {
        Ok(p) => Some(std::fs::read(p)?),
        Err(_) => None,
    };

    let client = OpenanyClient::neu_mit_ca(&basis, &schluessel, ca.as_deref())?;
    let speicher = Speicher::oeffnen(&pfad)?;

    // Eine hiesige Notiz, wenn sie verlangt wird -- fuer die Gegenrichtung.
    if let Ok(text) = std::env::var("OPENANY_SCHREIBE_NOTIZ") {
        let zk_id = std::env::var("OPENANY_SCHREIBE_ZK")
            .unwrap_or_else(|_| chrono::Local::now().format("%Y%m%d%H%M%S").to_string());

        speicher.notiz_schreiben(
            &Notiz {
                zk_id: zk_id.clone(),
                titel: "Von der Probe".into(),
                inhalt: text,
                ..Default::default()
            },
            Protokoll::Merken,
        )?;

        println!("hiesige Notiz angelegt: {zk_id}");
    }

    let bericht = Laeufer::neu(&speicher, &client, "Probegeraet").lauf().await;

    println!("\n── Bericht ──────────────────────────────");
    println!("  gezogen        {}", bericht.gezogen);
    println!("  geschoben      {}", bericht.geschoben);
    println!("  uebersprungen  {}", bericht.uebersprungen);
    println!("  konflikte      {}", bericht.konflikte);

    match &bericht.speicher {
        Some(s) => println!(
            "  speicher       {} von {}",
            s.belegt,
            s.grenze
                .map(|g| g.to_string())
                .unwrap_or_else(|| "unbegrenzt".into())
        ),
        None => println!("  speicher       (nicht gemeldet)"),
    }

    for fehler in &bericht.fehler {
        println!("  FEHLER         {fehler}");
    }

    println!("\n── Was hier liegt ───────────────────────");

    for k in speicher.kalender_alle()? {
        println!("  Kalender  {}  {}", &k.uuid[..8], k.name);

        for t in speicher.termine_im_kalender(&k.uuid)? {
            println!(
                "    Termin  {}  {}  {}  abdruck {}",
                &t.uuid[..8],
                t.felder.beginn,
                t.felder.titel,
                t.abdruck(),
            );
        }
    }

    for n in speicher.notizen()? {
        println!(
            "  Notiz     {}  {}  ({} Bytes)",
            n.zk_id,
            n.titel,
            n.inhalt.len()
        );
    }

    for k in speicher.kontakte()? {
        println!(
            "  Kontakt   {}  {}  {}",
            &k.uuid[..8],
            k.anzeigename,
            k.kennung("matrix").unwrap_or("—")
        );
    }

    let marke = speicher.marke(&basis)?;
    println!(
        "\n  Marken: fremd {}  eigen {}  Fehler {:?}",
        marke.fremde, marke.eigene, marke.letzter_fehler
    );

    Ok(())
}
