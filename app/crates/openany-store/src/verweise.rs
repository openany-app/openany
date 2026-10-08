//! `[[Verweise]]` und `#Tags` im Text einer Notiz -- dieselbe Regel wie in
//! openany (`Note::syncLinksAndTags`).
//!
//! **Zwei Rechenwege fuer dieselbe Sache.** Der Server fuehrt Verweise und Tags
//! als Tabellen; das Programm rechnet sie beim Lesen aus dem Text. Rechnen die
//! beiden verschieden, zeigt dieselbe Notiz in der App andere Rueckverweise
//! als in der Webapp -- ohne Fehler, nur falsch. Deshalb stehen unten die
//! Faelle, die PHP am 15.09.2026 fuer genau diese Texte ausgegeben hat.
//!
//! **Kein Lookbehind.** PHP schreibt `(?<=^|\s)#…`; Rusts `regex` kennt kein
//! Lookbehind. `(?:^|\s)#(…)` verbraucht das Leerzeichen davor mit -- das
//! aendert nichts, weil ein Tag nie auf `#` endet und zwei Tags dazwischen
//! immer ein eigenes Leerzeichen brauchen.

use regex::Regex;
use std::sync::OnceLock;

fn wikilink() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"\[\[([^\[\]\n]+)\]\]").expect("Muster"))
}

fn tag() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"(?:^|\s)#(\p{L}[\p{L}\p{N}_/-]{0,49})").expect("Muster"))
}

fn ziel_mit_id() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^(\d{14})(?:\s+.*)?$").expect("Muster"))
}

/// Die Ziele der `[[Verweise]]`: nur der Teil vor `|`, getrimmt, ohne leere,
/// ohne Doppel (Gross/Klein egal, der erste gewinnt), hoechstens 200.
pub fn verweise(text: &str) -> Vec<String> {
    let mut gesehen = std::collections::HashSet::new();

    wikilink()
        .captures_iter(text)
        .filter_map(|c| {
            let roh = c.get(1)?.as_str();
            let ziel = roh.split('|').next().unwrap_or("").trim().to_string();
            (!ziel.is_empty() && gesehen.insert(ziel.to_lowercase())).then_some(ziel)
        })
        .take(200)
        .collect()
}

/// Die `#Tags`: klein geschrieben, `//` zusammengefasst, `/` an den Raendern
/// weg, ohne leere und Doppel, hoechstens 100.
pub fn tags(text: &str) -> Vec<String> {
    let mut gesehen = std::collections::HashSet::new();

    tag()
        .captures_iter(text)
        .filter_map(|c| {
            let klein = c.get(1)?.as_str().to_lowercase();
            let mut sauber = String::with_capacity(klein.len());
            for zeichen in klein.chars() {
                if !(zeichen == '/' && sauber.ends_with('/')) {
                    sauber.push(zeichen);
                }
            }
            let sauber = sauber.trim_matches('/').to_string();
            (!sauber.is_empty() && gesehen.insert(sauber.clone())).then_some(sauber)
        })
        .take(100)
        .collect()
}

/// Traegt das Ziel eine vorangestellte `zk_id` (`20260731153042 Titel`)?
pub fn ziel_id(ziel: &str) -> Option<&str> {
    ziel_mit_id()
        .captures(ziel.trim())
        .and_then(|c| c.get(1))
        .map(|m| m.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ausgabe von PHP (Note::WIKILINK_PATTERN, TAG_PATTERN und die
    // Nachbearbeitung aus syncLinksAndTags), am 15.09.2026 im lokalen
    // Container gerechnet. Nicht von Hand "verbessern" -- wer hier etwas
    // aendert, aendert es zuerst in PHP.

    #[test]
    fn tags_wie_php() {
        assert_eq!(
            tags("#Anfang mitten #zwei und#kein #Über/Ärger/ #3zahl #a//b #x_y-z"),
            vec!["anfang", "zwei", "über/ärger", "a/b", "x_y-z"]
        );
        assert_eq!(tags("Zeile\n#tag\n[[mehr\nzeilen]] [[a[b]]"), vec!["tag"]);
        assert_eq!(
            tags(&format!("#{} #Straße #日本 #café", "a".repeat(60))),
            vec![&"a".repeat(50) as &str, "straße", "日本", "café"]
        );
    }

    #[test]
    fn verweise_wie_php() {
        assert_eq!(
            verweise("Siehe [[Einkauf]] und [[20260731153042 Titel|Anzeige]] und [[  Leer  |x]] [[Einkauf]] [[einkauf]]"),
            vec!["Einkauf", "20260731153042 Titel", "Leer"]
        );
        assert!(verweise("Zeile\n#tag\n[[mehr\nzeilen]] [[a[b]]").is_empty());
        assert!(verweise("#Anfang mitten #zwei").is_empty());
    }

    #[test]
    fn ziel_mit_kennung() {
        assert_eq!(ziel_id("20260731153042 Titel"), Some("20260731153042"));
        assert_eq!(ziel_id("20260731153042"), Some("20260731153042"));
        assert_eq!(ziel_id("2026073115304 kurz"), None);
        assert_eq!(ziel_id("Einkauf"), None);
    }
}
