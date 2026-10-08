//! Der Vertrag zu openany -- und der Weg dorthin.
//!
//! Dieses Crate weiss, **was** ueber die Leitung geht und **wie** es
//! hinkommt. Es weiss nicht, wo etwas gespeichert wird (das ist
//! `openany-store`) und nicht, in welcher Reihenfolge etwas geschieht (das
//! ist `openany-sync`).
//!
//! ## Der Abgleich ist gebaut, nicht neu
//!
//! openany hat den Zweiwegabgleich fuer den verschluesselten Stick gebaut
//! (seit dem 08.10.2026 ausgebaut): ein Laeufer, der einmal laeuft und dann
//! fertig ist. Dieses Programm ist
//! der **dritte** Nutzer desselben Protokolls, nicht ein neues Vorhaben --
//! aber es ist ein anderer Nutzer: Ein Telefon gleicht dauernd und nebenbei
//! ab, auf einer Leitung, die abreisst.
//!
//! Daraus folgt fuer dieses Crate zweierlei, und beides steht ausdruecklich
//! im Bau statt in einer Notiz:
//!
//! * **Nichts reisst an einem einzelnen Eintrag.** Ein fremdes Feld, eine
//!   unbekannte Art, ein kaputter Eintrag -- die Seite bleibt gueltig und die
//!   Marke rueckt vor. Siehe [`Delta::lesen`].
//! * **Ein Fehler wird unterschieden.** "Spaeter nochmal" und "so wird das
//!   nie wieder etwas" sind nicht dasselbe. Siehe [`OpenanyError`].
//!
//! ## Der Abdruck ist die heikelste Stelle
//!
//! Ein Termin wird dreiseitig entschieden -- ueber einen Hash von neun
//! Feldern. Rechnen beide Seiten ihn verschieden, schlaegt **nichts** fehl:
//! Es entstehen nur Konfliktkopien, bei jedem Lauf. Deshalb ist
//! [`Terminfelder`] keine Bequemlichkeit, sondern die zweite Haelfte einer
//! Verabredung, deren erste in PHP steht -- mit festgeschriebenen
//! Vergleichswerten aus PHP als Test.

mod abdruck;
mod client;
mod vertrag;

pub use abdruck::{abdruck_von_eintrag, Terminfelder};
pub use client::{
    anyid_der_instanz, geraeteschluessel_holen, Angekommen, Geraeteschluessel, Hochzuladen,
    OpenanyClient, OpenanyError, Uebertragung, MAX_EINTRAEGE, STUECK_ZUM_SERVER,
};
pub use vertrag::{Art, Delta, Eintrag, Ergebnis, Speicherstand, Vertragsfehler, Was};

/// Der Hash eines Notiztextes -- die andere Sorte Abdruck.
///
/// Eine Notiz hat einen Inhalt, also reicht `sha256` darueber; die neun
/// Felder eines Termins haben das nicht, deshalb dort [`Terminfelder`].
/// Beide muessen mit PHP uebereinstimmen, aber nur einer davon ist heikel.
pub fn notizabdruck(inhalt: &str) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(inhalt.as_bytes());

    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn der_notizabdruck_stimmt_mit_php_ueberein() {
        // `php -r 'echo hash("sha256", "Hallo Welt");'` -- fest eingetragen,
        // damit ein Wechsel des Verfahrens auf einer Seite auffaellt.
        assert_eq!(
            notizabdruck("Hallo Welt"),
            "2d2da19605a34e037dbe82173f98a992a530a5fdd53dad882f570d4ba204ef30",
        );
    }

    #[test]
    fn ein_leerer_text_hat_den_bekannten_leer_hash() {
        assert_eq!(
            notizabdruck(""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        );
    }
}
