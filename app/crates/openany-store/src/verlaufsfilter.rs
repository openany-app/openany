//! Filter und Suche im Verlauf (Tiffy, 01.10.2026) -- für Matrix-Nachrichten
//! und Mails gleich.
//!
//! **Hier und nicht in der Ansicht**, weil der Verlauf seitenweise kommt:
//! „Ungelesen" über die erste Seite allein wäre eine halbe Antwort.
//!
//! **Welcher Weg gefragt ist, entscheidet der Aufrufer** -- er fragt die
//! Tabelle eines abgewählten Weges gar nicht erst.

use rusqlite::types::Value;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Verlaufsfilter {
    /// Nur empfangene, die noch niemand gelesen hat.
    pub ungelesen: bool,
    /// Nur solche mit Anhang.
    pub anhang: bool,
    /// Ein Stück Text; leer heißt: keine Suche.
    pub suche: String,
    /// Gegenüber, deren Name im Adressbuch auf die Suche passt -- deren
    /// Nachrichten treffen auch, wenn im Text nichts davon steht.
    pub suche_gegenueber: Vec<String>,
    /// Nur diese Gegenüber (eine Unterhaltung); leer heißt: alle.
    pub mit: Vec<String>,
    /// Nur dieses eigene Konto (`Some`) -- oder alle außer ihm (`ohne`).
    /// Die Direktnachrichten vor Ort liegen in der Tabelle der
    /// Matrix-Nachrichten unter dem Konto `nah` (02.10.2026).
    pub nur_konto: Option<String>,
    pub ohne_konto: Option<String>,
}

/// Wie eine Tabelle die Felder nennt, nach denen gefiltert wird.
pub(crate) struct Spalten {
    pub gegenueber: &'static str,
    /// Bedingung „hat einen Anhang".
    pub anhang: &'static str,
    /// Worin gesucht wird, außer im Gegenüber.
    pub suchfelder: &'static [&'static str],
    /// Die Spalte des eigenen Kontos, wo die Tabelle eine führt.
    pub konto: Option<&'static str>,
}

impl Verlaufsfilter {
    /// `AND …`-Bedingungen und ihre Werte, nach `geloescht_at IS NULL`.
    pub(crate) fn sql(&self, s: &Spalten) -> (String, Vec<Value>) {
        let mut sql = String::new();
        let mut werte: Vec<Value> = Vec::new();
        if self.ungelesen {
            sql.push_str(" AND von_mir = 0 AND gelesen_at IS NULL");
        }
        if let Some(spalte) = s.konto {
            if let Some(k) = &self.nur_konto {
                sql.push_str(&format!(" AND {spalte} = ?"));
                werte.push(Value::Text(k.clone()));
            }
            if let Some(k) = &self.ohne_konto {
                sql.push_str(&format!(" AND {spalte} <> ?"));
                werte.push(Value::Text(k.clone()));
            }
        }
        if self.anhang {
            sql.push_str(&format!(" AND {}", s.anhang));
        }
        if !self.mit.is_empty() {
            sql.push_str(&format!(
                " AND {} IN ({})",
                s.gegenueber,
                platzhalter(self.mit.len())
            ));
            werte.extend(self.mit.iter().cloned().map(Value::Text));
        }
        let suche = self.suche.trim().to_lowercase();
        if !suche.is_empty() {
            // LIKE ist in SQLite ohne Rücksicht auf Groß/klein -- aber nur
            // für ASCII. Deshalb zusätzlich LOWER() auf beiden Seiten; für
            // Umlaute bleibt es dabei, dass „Ä" nur „Ä" findet.
            let muster = format!("%{suche}%");
            let mut oder: Vec<String> = s
                .suchfelder
                .iter()
                .chain(std::iter::once(&s.gegenueber))
                .map(|f| format!("LOWER({f}) LIKE ?"))
                .collect();
            for _ in 0..oder.len() {
                werte.push(Value::Text(muster.clone()));
            }
            if !self.suche_gegenueber.is_empty() {
                oder.push(format!(
                    "{} IN ({})",
                    s.gegenueber,
                    platzhalter(self.suche_gegenueber.len())
                ));
                werte.extend(self.suche_gegenueber.iter().cloned().map(Value::Text));
            }
            sql.push_str(&format!(" AND ({})", oder.join(" OR ")));
        }
        (sql, werte)
    }
}

fn platzhalter(n: usize) -> String {
    vec!["?"; n].join(", ")
}
