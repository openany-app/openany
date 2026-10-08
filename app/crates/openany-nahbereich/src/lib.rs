//! Geraete in der Naehe -- finden, ohne Server.
//!
//! **Was dieses Crate kann:** eine eigene Geraeteidentitaet halten
//! ([`Identitaet`]), sich im lokalen Netz ankuendigen und andere Geraete
//! finden ([`Nahbereich`]). Es spricht dafuer das offene LocalSend-Protokoll
//! (v2.2) ueber das `localsend`-Crate: Multicast, und wo ein Netz das nicht
//! durchlaesst, eine Suche im Subnetz.
//!
//! **Und mit gepaarten Geraeten:** paaren ([`anruf`], [`dienst`]) und
//! abgleichen ([`abgleich`]) -- die Schritte 4 und 5 von APK 0.1
//! (docs/plan-app-neuaufsatz.md).
//!
//! **Kein Plattform-Code.** Was Android dafuer zusaetzlich braucht (die
//! Multicast-Sperre), steht in `MainActivity.kt`; was iOS braucht, kommt mit
//! der iOS-Phase. Dieses Crate laeuft auf dem Schreibtisch genauso.

pub mod abgleich;
pub mod anruf;
pub mod chat;
pub mod dienst;
pub mod direkt;
pub mod einladen;
pub mod freigaben;
mod identitaet;
pub mod mitglieder;
mod nahbereich;
pub mod paaren;
pub mod person;
pub mod projektnah;
pub mod sperren;
mod tls;

pub use abgleich::{basis_fuer, GemeinsamerSpeicher, NahGegenstelle};
pub use dienst::{Dienst, Gastgeber, GemeinsamePaarungen, DIENST_PORT};
pub use identitaet::{Identitaet, IdentitaetFehler};
pub use mitglieder::{Mitglied, MitgliedFehler, Mitgliederliste};
pub use nahbereich::{GeraetInDerNaehe, Nahbereich, NahbereichFehler, KENNUNG_MODELL};
pub use person::{Geraet, Geraeteeintrag, Person, PersonFehler};
