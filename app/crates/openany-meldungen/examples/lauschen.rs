//! Haelt die Leitung und schreibt jeden Vorgang auf die Konsole.
//!
//!     cargo run -p openany-meldungen --example lauschen -- https://ntfy.openany.de upGeheim
//!
//! Zum Pruefen am Schreibtisch, bevor ein Geraet im Spiel ist: In einem
//! zweiten Fenster `curl -d '{"art":"nachricht","projekt":1}' <basis>/<thema>`.

use openany_meldungen::{halten, Leitung, Vorgang};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(basis), Some(thema)) = (args.next(), args.next()) else {
        eprintln!("Aufruf: lauschen <basis> <thema>");
        std::process::exit(2);
    };
    // Der Schreibtisch meldet kein Netz: Sender fallen lassen heisst
    // „immer da".
    let (_, netz) = tokio::sync::watch::channel(true);
    halten(Leitung::new(basis, thema), netz, |v| match v {
        Vorgang::Verbunden => println!("verbunden"),
        Vorgang::Ereignis(e) => println!("{}  {:?}", e.id, e.signal),
        Vorgang::Abriss { grund, pause } => println!("abgerissen: {grund} -- neu in {pause:?}"),
    })
    .await;
}
