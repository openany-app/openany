//! Die Weckleitung: eine dauerhafte, sparsame Verbindung zu openanys ntfy.
//!
//! Plan: `docs/plan-app-neuaufsatz.md`, Phase 6. Laravel und Matrix-Homeserver
//! schicken ein **Signal ohne Inhalt** an das geheime Thema dieses Geraets;
//! diese Kiste haelt die Leitung offen, liest die Signale und reicht sie
//! weiter. Den Inhalt holt danach der Abgleich -- hier geht nichts ueber die
//! Leitung, was man lesen muesste.
//!
//! ## Plattformfrei
//!
//! Kein JNI, kein Android. Auf dem Schreibtisch laeuft [`halten`] einfach in
//! der Laufzeit des Programms; auf Android startet der Vordergrunddienst eine
//! eigene und ruft dieselbe Funktion. Was je Plattform verschieden ist -- ob
//! Netz da ist, wie eine Benachrichtigung aussieht --, kommt von aussen herein.
//!
//! ## Sparsam, weil der Funk zaehlt und nicht die CPU
//!
//! * **Keine eigenen Timer, die senden.** Das Lebenszeichen schickt der
//!   Server (`NTFY_KEEPALIVE_INTERVAL`, 3 min). Hier laeuft nur eine
//!   Lese-Frist knapp darueber: Bleibt es laenger still, ist die Leitung tot.
//! * **Kein Versuch ohne Netz.** Meldet die Schale „kein Netz", wartet die
//!   Schleife auf die naechste Meldung statt blind neu zu waehlen.
//! * **Wachsende Wartezeit mit Zufall.** Faellt ntfy aus, sollen nicht alle
//!   Geraete in derselben Sekunde wiederkommen.
//! * **Nichts verlieren:** Beim Neuverbinden geht `since=<letzte ID>` mit;
//!   ntfy liefert nach, was in der Luecke kam (bis 12 h).

mod leitung;
mod signal;
mod warten;

pub use leitung::{halten, Leitung, Leitungsfehler, Vorgang, NACH_NETZWECHSEL};
pub use signal::{Ereignis, Signal};
pub use warten::Wartezeit;
