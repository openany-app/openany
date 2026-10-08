//! Trifft jeder Klick einen Befehl?
//!
//! **Die eine Stelle, die kein anderer Test erreicht.** Die Fachlogik ist in
//! `crates/` gepruefft, 99 Tests ohne Server und ohne Emulator. Was dort nicht
//! vorkommt, ist die Naht zwischen Oberflaeche und Schale: ein `invoke()` mit
//! einem Tippfehler im Namen, oder mit einem Argument, das die Signatur nicht
//! kennt.
//!
//! So ein Fehler schlaegt **nirgends** fehl, wo jemand hinsieht. Er kompiliert,
//! das Fenster geht auf, und erst wer auf genau diesen Knopf drueckt, bekommt
//! eine Fehlermeldung im Webview -- die in keinem Terminal und in keinem Log
//! steht. Auf Android sieht sie ueberhaupt niemand.
//!
//! Deshalb liest dieser Test die Vue-Dateien und die Befehlsliste und haelt sie
//! nebeneinander. Kein Webview, keine Anzeige, kein Klick.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

fn wurzel() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Alle Rust-Dateien der Schale zusammen -- Befehle stehen seit Phase 3 auch
/// in eigenen Modulen (`dateibefehle.rs`).
fn schale() -> String {
    let mut alles = String::new();
    for eintrag in fs::read_dir(wurzel().join("src")).expect("src/") {
        let pfad = eintrag.expect("Eintrag").path();
        if pfad.extension().is_some_and(|e| e == "rs") {
            alles.push_str(&fs::read_to_string(&pfad).expect("Quelldatei"));
            alles.push('\n');
        }
    }
    alles
}

/// Die Namen aus `generate_handler![...]`.
fn registrierte_befehle(quelle: &str) -> BTreeSet<String> {
    let ab = quelle
        .find("generate_handler![")
        .expect("generate_handler! fehlt")
        + "generate_handler![".len();
    let bis = ab
        + quelle[ab..]
            .find(']')
            .expect("generate_handler! nicht geschlossen");

    quelle[ab..bis]
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        // `dateibefehle::dateien_liste` heisst in der Oberflaeche `dateien_liste`.
        .map(|s| s.rsplit("::").next().unwrap_or(s).to_string())
        .collect()
}

/// `snake_case` -> `camelCase`, wie Tauri es zur Oberflaeche hin uebersetzt.
fn camel(name: &str) -> String {
    let mut teile = name.split('_');
    let mut aus = teile.next().unwrap_or_default().to_string();

    for teil in teile {
        let mut zeichen = teil.chars();

        if let Some(erstes) = zeichen.next() {
            aus.push(erstes.to_ascii_uppercase());
            aus.push_str(zeichen.as_str());
        }
    }

    aus
}

/// Die Argumente eines Befehls -- ohne den Zustand, den Tauri selbst stellt.
fn argumente(quelle: &str, befehl: &str) -> BTreeSet<String> {
    let marke = format!("async fn {befehl}(");
    let ab = quelle
        .find(&marke)
        .unwrap_or_else(|| panic!("{befehl} nicht gefunden"))
        + marke.len();
    let bis = ab + quelle[ab..].find(')').expect("Signatur nicht geschlossen");

    quelle[ab..bis]
        .split(',')
        .map(str::trim)
        // `State<'_, Arc<Zustand>>` zerfaellt am Komma in zwei Teile -- keiner
        // davon ist ein Argument der Oberflaeche.
        .filter(|t| !t.is_empty() && !t.contains("tauri::State") && !t.contains("Zustand>"))
        .filter_map(|t| t.split(':').next())
        .map(|n| camel(n.trim()))
        .collect()
}

/// Jedes `invoke('name', { … })` in den Vue- und JS-Dateien.
///
/// JS gehoert seit dem 15.09.2026 dazu: Die Datenquellen der Arbeitsflaechen
/// (`src/quellen/`) sind reine JS-Dateien -- ohne sie saehe dieser Test die
/// meisten Aufrufe gar nicht.
fn aufrufe() -> Vec<(String, String, String)> {
    let mut gefunden = Vec::new();

    sammeln(&wurzel().join("../src"), &mut |datei, text| {
        let mut rest = text.as_str();

        while let Some(i) = rest.find("invoke('") {
            rest = &rest[i + "invoke('".len()..];

            let Some(ende) = rest.find('\'') else { break };
            let name = rest[..ende].to_string();
            let nach = &rest[ende..];

            // EIN AUFRUF OHNE ARGUMENTE IST HIER ZU ENDE, und das muss
            // ausdruecklich dastehen. Die Suche nach `{` unten nimmt sonst
            // die naechste Klammer, die ihr begegnet -- und nach
            // `invoke('kopplung_laeuft');` ist das der `if`-Block darunter.
            // Am 16.09.2026 meldete der Test daraufhin einen Kommentar als
            // Argument: "die Signatur kennt nur {}" ueber zwei Zeilen Prosa.
            //
            // Die Vierzig-Zeichen-Grenze darunter war der Versuch, genau das
            // zu verhindern. Sie ist eine Schaetzung; dies ist die Frage.
            let hinter = nach[1..].trim_start();

            // Die Argumente bis zur schliessenden Klammer des Objekts --
            // grob, aber genau genug: Es geht um Schluesselnamen.
            let roh = if hinter.starts_with(')') {
                String::new()
            } else {
                match (nach.find('{'), nach.find('}')) {
                    (Some(a), Some(z)) if a < z && a < 40 => nach[a + 1..z].to_string(),
                    _ => String::new(),
                }
            };

            gefunden.push((datei.to_string(), name, roh));
        }
    });

    gefunden
}

fn sammeln(ordner: &Path, tun: &mut impl FnMut(&str, String)) {
    for eintrag in fs::read_dir(ordner).expect("src/ fehlt") {
        let pfad = eintrag.expect("Eintrag").path();

        if pfad.is_dir() {
            sammeln(&pfad, tun);
        } else if pfad.extension().is_some_and(|e| e == "vue" || e == "js") {
            let name = pfad.file_name().unwrap().to_string_lossy().to_string();
            tun(&name, fs::read_to_string(&pfad).expect("Quelldatei"));
        }
    }
}

#[test]
fn jeder_aufruf_trifft_einen_befehl() {
    let quelle = schale();
    let befehle = registrierte_befehle(&quelle);

    for (datei, name, _) in aufrufe() {
        assert!(
            befehle.contains(&name),
            "{datei}: invoke('{name}') -- diesen Befehl gibt es nicht. \
             Registriert sind: {befehle:?}"
        );
    }
}

#[test]
fn jedes_argument_trifft_ein_feld_der_signatur() {
    let quelle = schale();

    for (datei, name, roh) in aufrufe() {
        // `{ ...objekt }` -- die Schluessel stehen woanders; hier ist nichts
        // zu pruefen, ohne den Rest des Programms zu verstehen.
        if roh.contains("...") {
            continue;
        }

        let erlaubt = argumente(&quelle, &name);

        for zeile in roh.split(',') {
            let Some(schluessel) = zeile.split(':').next() else {
                continue;
            };
            let schluessel = schluessel.trim();

            if schluessel.is_empty() {
                continue;
            }

            assert!(
                erlaubt.contains(schluessel),
                "{datei}: {name}(… {schluessel} …) -- die Signatur kennt nur {erlaubt:?}"
            );
        }
    }
}

#[test]
fn kein_befehl_steht_ungenutzt_herum() {
    // Ein registrierter Befehl, den niemand ruft, ist entweder ein Rest oder
    // eine vergessene Ansicht. Beides will man sehen.
    let quelle = schale();
    let gerufen: BTreeSet<String> = aufrufe().into_iter().map(|(_, name, _)| name).collect();

    let ungenutzt: Vec<_> = registrierte_befehle(&quelle)
        .into_iter()
        .filter(|b| !gerufen.contains(b))
        .collect();

    assert!(ungenutzt.is_empty(), "nie gerufen: {ungenutzt:?}");
}
