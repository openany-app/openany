//! Die Kopplung von Hand -- gegen echte anyid- und openany-Instanzen.
//!
//! Die elf Tests im Crate laufen ohne beide Server; sie beweisen, dass die
//! Zustandsmaschine stimmt. **Dieses Beispiel beweist das andere**: dass die
//! vier Schritte ueber zwei echte Server zusammenpassen -- die Adressen, die
//! Kopfzeilen, der Name der Anwendung, die Lebensdauer des Tickets.
//!
//! ```bash
//! export ANYID_BASIS=https://id.anytail.localhost
//! export OPENANY_BASIS=https://openany.localhost
//! export OPENANY_CA=~/anyx/infra/caddy/lokale-ca.crt   # nur lokal noetig
//! export OPENANY_AUSWEISE=/tmp/openany-ausweise
//! cargo run -p openany-anmeldung --example koppeln
//! ```
//!
//! Das Programm zeigt einen Code und wartet. Im Browser anmelden, den Code
//! eintippen, bestaetigen -- den Rest holt es sich selbst.
//!
//! **Die Bestaetigung im Browser ist kein Umweg, sondern der
//! Sicherheitsanker.** Ein Programm, das sich selbst bestaetigen koennte,
//! waere kein Nachweis.

use anyid_client::{AnyidClient, Dateispeicher};
use openany_anmeldung::{Anmeldung, Kopplungsstand, OpenanySchalter};
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let anyid_basis = std::env::var("ANYID_BASIS")?;
    let openany_basis = std::env::var("OPENANY_BASIS")?;
    let ordner =
        PathBuf::from(std::env::var("OPENANY_AUSWEISE").unwrap_or_else(|_| "ausweise".into()));
    let geraetename = std::env::var("OPENANY_GERAET").unwrap_or_else(|_| "Probegeraet".into());

    let ca = match std::env::var("OPENANY_CA") {
        Ok(p) => Some(std::fs::read(p)?),
        Err(_) => None,
    };

    let anyid = AnyidClient::neu_mit_ca(&anyid_basis, ca.as_deref())?;
    let mut openany = OpenanySchalter::neu(&openany_basis);

    if let Some(pem) = &ca {
        openany = openany.mit_ca(pem.clone());
    }

    let anmeldung = Anmeldung::neu(
        anyid,
        openany,
        &geraetename,
        format!("{anyid_basis}/einstellungen/geraete"),
        Box::new(Dateispeicher::neu(ordner.join("anyid-ausweis"))),
        Box::new(Dateispeicher::neu(ordner.join("openany-schluessel"))),
    );

    // Schon gekoppelt? Dann nur den Schluessel erneuern -- kein Mensch noetig.
    if anmeldung.einsatzbereit()? {
        println!("Dieses Gerät ist gekoppelt. Ich hole einen frischen Schlüssel …");
        let name = anmeldung.schluessel_erneuern().await?;
        println!("  Konto:      {name}");
        println!(
            "  Schlüssel:  {}",
            ordner.join("openany-schluessel").display()
        );

        return Ok(());
    }

    let kopplung = anmeldung.koppeln_beginnen().await?;

    println!("\n╭──────────────────────────────────────────────");
    println!("│  Code:     {}", kopplung.code);
    println!("│  Browser:  {}", kopplung.browserziel);
    println!("│  gilt noch {} Sekunden", kopplung.ablauf_in);
    println!("╰──────────────────────────────────────────────\n");
    println!("Dort anmelden, den Code eintippen, bestätigen. Ich warte.\n");

    let frist = Instant::now() + Duration::from_secs(kopplung.ablauf_in);

    loop {
        match anmeldung.koppeln_abholen(&kopplung).await? {
            Kopplungsstand::Wartet { intervall } => {
                if Instant::now() > frist {
                    println!("Der Code ist abgelaufen. Noch einmal von vorn.");

                    return Ok(());
                }

                print!(".");
                use std::io::Write;
                std::io::stdout().flush()?;
                tokio::time::sleep(Duration::from_secs(intervall)).await;
            }
            Kopplungsstand::Ungueltig => {
                // Abgelaufen, erfunden, falscher Verifier oder schon
                // abgeholt -- anyid unterscheidet die vier nicht, und das ist
                // Absicht.
                println!("\nDieser Code gilt nicht (mehr). Noch einmal von vorn.");

                return Ok(());
            }
            Kopplungsstand::Gekoppelt { name } => {
                println!("\n\nGekoppelt.");
                println!("  Konto:      {name}");
                println!("  Gerät:      {geraetename}");
                println!("  Ausweise:   {}", ordner.display());
                println!(
                    "\nZum Abgleich:\n  OPENANY_SCHLUESSEL_DATEI={} \\\n    cargo run -p openany-sync --example probe",
                    ordner.join("openany-schluessel").display()
                );

                return Ok(());
            }
        }
    }
}
