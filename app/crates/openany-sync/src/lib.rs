//! Der Abgleich: die Reihenfolge, in der die Dinge geschehen.
//!
//! Drei Crates, drei Fragen:
//!
//! | | |
//! |---|---|
//! | `openany-client` | **was** geht ueber die Leitung |
//! | `openany-store`  | **wo** liegt es hier |
//! | `openany-sync`   | **in welcher Reihenfolge** -- und wer gewinnt |
//!
//! ## Warum das ein eigenes Crate ist
//!
//! Weil hier alles steht, was man pruefen will, ohne einen Server zu haben --
//! und weil man genau die Faelle pruefen will, die ein echter Server nur
//! schwer herstellt: eine Leitung, die mitten in der zweiten Seite abreisst;
//! ein Eintrag mit einer Art aus der Zukunft; beide Seiten haben dieselbe
//! Notiz geaendert. Deshalb spricht der [`Laeufer`] nicht mit einem Client,
//! sondern mit einer [`Gegenstelle`] -- einem Trait mit drei Methoden.
//!
//! ## Die Marke bleibt hier
//!
//! openanys Abgleich ist so gebaut, dass der Server **nichts** ueber den
//! Fortschritt einer Gegenstelle weiss. Er entscheidet Konflikte mit dem
//! `base_hash`, den wir mitschicken, und merkt sich nichts. Das ist der
//! Grund, warum mehrere Geraete sich nicht in die Quere kommen -- und
//! warum Multidevice an openany nichts gekostet hat.
//!
//! Serverseitiger Fortschrittszustand je Geraet waere deshalb kein
//! Fortschritt, sondern ein Rueckschritt: Er fuehrte den Fehlerfall ein, den
//! es jetzt nicht gibt -- die Marke rueckt vor, das Anwenden auf dem Geraet
//! scheitert, der Eintrag ist verloren.

mod abo;
mod auskunft;
mod entscheidung;
mod gegenstelle;
mod hinaus;
mod ics;
mod inhalte_holen;
mod laeufer;
mod projektlauf;

pub use abo::{adresse_pruefen, auffrischen, Abobericht, Abofehler, Feedquelle, UeberDasNetz};
pub use auskunft::Auskunft;
pub use entscheidung::{entscheiden, ursprung_waehlen, Entscheidung};
pub use gegenstelle::Gegenstelle;
pub use inhalte_holen::{
    fehlende_inhalte_holen, inhalt_holen, reserve, InhaltFehler, InhalteBericht, STUECK,
};
pub use laeufer::{Bericht, Laeufer, Lauffehler};
pub use projektlauf::{projekt_lauf, Projektbericht, Projektgegenstelle};
