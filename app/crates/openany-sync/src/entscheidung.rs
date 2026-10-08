//! Der Drei-Seiten-Vergleich -- die Frage, die aus einem Abrufen einen
//! ABGLEICH macht: Wurde hier geaendert, drueben, oder auf beiden Seiten?
//!
//! Wortgleich zu openanys `PackageMerge`, und das ist kein Zufall, sondern
//! Bedingung: Beide Seiten treffen dieselbe Entscheidung ueber dieselben drei
//! Hashes. Ein Laeufer, der anders entschiede als der Server, produzierte je
//! nach Richtung andere Ergebnisse -- und niemand koennte sagen, welche die
//! richtige war.
//!
//! **Drei Hashes rein, eine Entscheidung raus.** Keine Datenbank, keine
//! Dateien, kein Netz. Was danach geschieht, entscheidet der Aufrufer.

/// Was mit einem Eintrag zu geschehen hat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Entscheidung {
    /// Gibt es hier noch nicht -- anlegen.
    Anlegen,
    /// Beide Seiten gleich -- nichts tun. Die haeufigste Antwort.
    Unveraendert,
    /// Nur drueben geaendert -- uebernehmen.
    Uebernehmen,
    /// Nur hier geaendert, das Fremde ist aelter -- den hiesigen Stand
    /// behalten.
    HierBleibt,
    /// Beidseitig geaendert -- Konfliktkopie, das Original unangetastet.
    Konflikt,
}

/// * `ursprung` -- der zuletzt beidseitig bekannte Stand; `None` heisst: es
///   gab nie einen.
/// * `hier` -- der jetzige Stand hier; `None` heisst: gibt es hier nicht.
/// * `drueben` -- der Stand, der angekommen ist.
///
/// **Ohne Ursprung ist der Vergleich zweiseitig**, und zwei abweichende
/// Staende sehen genauso aus, ob nun eine Seite geaendert hat oder beide.
/// Dann gilt jeder Unterschied als Konflikt. Das ist unbequem und
/// absichtlich so: Ein Abgleich, der im Zweifel ueberschreibt, verliert Text.
pub fn entscheiden(ursprung: Option<&str>, hier: Option<&str>, drueben: &str) -> Entscheidung {
    let Some(hier) = hier else {
        return Entscheidung::Anlegen;
    };

    // Gleicher Inhalt -- ZUERST geprueft, damit ein zweiter Durchlauf
    // derselben Seite wirklich null Aenderungen meldet, auch ohne Ursprung.
    if hier == drueben {
        return Entscheidung::Unveraendert;
    }

    let Some(ursprung) = ursprung else {
        return Entscheidung::Konflikt;
    };

    // Hier unangetastet seit dem gemeinsamen Stand -> die Aenderung kam von
    // drueben.
    if hier == ursprung {
        return Entscheidung::Uebernehmen;
    }

    // Drueben unangetastet -> das Fremde ist schlicht aelter als der hiesige
    // Stand. Kein Konflikt, sondern ein alter Ausdruck.
    if drueben == ursprung {
        return Entscheidung::HierBleibt;
    }

    Entscheidung::Konflikt
}

/// Aus mehreren Kandidaten den gemeinsamen Stand waehlen.
///
/// **Zwischen zwei Geraeten gibt es zwei Gedaechtnisse.** Jedes merkt sich,
/// welcher Stand zuletzt beiden bekannt war -- aber nur, wenn es selbst
/// gezogen oder angenommen hat. Wer nur ausgeliefert hat, weiss nichts
/// davon, und sein Gedaechtnis veraltet. Welches der beiden das frische ist,
/// haengt davon ab, wer zuletzt getippt hat.
///
/// Jeder Kandidat war aber irgendwann wirklich beiden bekannt. Stimmt der
/// hiesige Stand mit einem von ihnen ueberein, hat hier seither niemand
/// geschrieben; stimmt der angekommene mit einem ueberein, drueben nicht.
/// Erst wenn keiner passt, haben beide geschrieben -- dann der erste
/// Kandidat, und [`entscheiden`] findet den Konflikt.
pub fn ursprung_waehlen(
    kandidaten: &[Option<String>],
    hier: Option<&str>,
    drueben: &str,
) -> Option<String> {
    let bekannt = || kandidaten.iter().flatten();

    if let Some(h) = hier {
        if let Some(k) = bekannt().find(|k| k.as_str() == h) {
            return Some(k.clone());
        }
    }

    bekannt()
        .find(|k| k.as_str() == drueben)
        .or_else(|| bekannt().next())
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::Entscheidung::*;
    use super::*;

    #[test]
    fn was_es_hier_nicht_gibt_wird_angelegt() {
        assert_eq!(entscheiden(None, None, "neu"), Anlegen);
        // Auch mit Ursprung: Die Sache ist hier fort, der Ursprung ist ein
        // Ueberbleibsel. Anlegen ist richtig -- geloescht wird ueber einen
        // eigenen Eintrag, nicht durch Abwesenheit.
        assert_eq!(entscheiden(Some("alt"), None, "neu"), Anlegen);
    }

    #[test]
    fn gleich_ist_gleich_auch_ohne_ursprung() {
        assert_eq!(entscheiden(None, Some("a"), "a"), Unveraendert);
        assert_eq!(entscheiden(Some("frueher"), Some("a"), "a"), Unveraendert);
    }

    #[test]
    fn nur_drueben_geaendert_wird_uebernommen() {
        assert_eq!(entscheiden(Some("a"), Some("a"), "b"), Uebernehmen);
    }

    #[test]
    fn nur_hier_geaendert_bleibt_hier() {
        assert_eq!(entscheiden(Some("a"), Some("b"), "a"), HierBleibt);
    }

    #[test]
    fn beidseitig_geaendert_ist_ein_konflikt() {
        assert_eq!(entscheiden(Some("a"), Some("b"), "c"), Konflikt);
    }

    #[test]
    fn ohne_ursprung_ist_jeder_unterschied_ein_konflikt() {
        // Der degradierte Modus. Er ist der Grund, warum der Ursprung ueber
        // die Gegenstelle geschluesselt sein MUSS: Wer ihn nachtraeglich
        // einzieht, wirft alle Urspruenge weg und landet fuer einen Lauf
        // genau hier -- fuer jede Notiz.
        assert_eq!(entscheiden(None, Some("a"), "b"), Konflikt);
    }

    #[test]
    fn ein_veraltetes_gedaechtnis_macht_keinen_konflikt() {
        let k =
            |a: Option<&str>, b: Option<&str>| vec![a.map(str::to_string), b.map(str::to_string)];

        // Hier veraltet (v1), der Eintrag weiss es besser (v2 = hier).
        let u = ursprung_waehlen(&k(Some("v1"), Some("v2")), Some("v2"), "v3");
        assert_eq!(entscheiden(u.as_deref(), Some("v2"), "v3"), Uebernehmen);

        // Andersherum: der Eintrag veraltet, hier frisch.
        let u = ursprung_waehlen(&k(Some("v2"), Some("v1")), Some("v2"), "v3");
        assert_eq!(entscheiden(u.as_deref(), Some("v2"), "v3"), Uebernehmen);

        // Drueben unveraendert seit einem bekannten Stand.
        let u = ursprung_waehlen(&k(Some("v1"), Some("v2")), Some("v4"), "v2");
        assert_eq!(entscheiden(u.as_deref(), Some("v4"), "v2"), HierBleibt);

        // Beide geschrieben -- bleibt ein Konflikt.
        let u = ursprung_waehlen(&k(Some("v1"), Some("v2")), Some("x"), "y");
        assert_eq!(entscheiden(u.as_deref(), Some("x"), "y"), Konflikt);

        // Gar kein Kandidat -- der vorsichtige Modus.
        let u = ursprung_waehlen(&k(None, None), Some("x"), "y");
        assert_eq!(entscheiden(u.as_deref(), Some("x"), "y"), Konflikt);
    }

    #[test]
    fn der_haeufigste_fall_ist_kein_konflikt() {
        // "Drueben geaendert, hier unangetastet" -- das passiert bei jedem
        // Lauf. Ergaebe es eine Konfliktkopie, waere der Abgleich unbenutzbar.
        let ursprung = "stand-vom-letzten-lauf";

        assert_eq!(
            entscheiden(Some(ursprung), Some(ursprung), "der neue Stand"),
            Uebernehmen
        );
    }
}
