//! Der Abdruck eines Termins -- dieselbe Rechnung wie in openanys
//! `App\Support\EventFingerprint`.
//!
//! **Warum das hier ueberhaupt noch einmal steht.** Eine Notiz hat einen
//! Inhalt, ueber den beide Seiten `sha256(text)` bilden und dabei auf
//! dasselbe kommen. Ein Termin hat keinen Inhalt, sondern neun Felder -- und
//! ueber Felder laesst sich nur hashen, wenn ihre Reihenfolge UND ihre
//! Schreibweise verabredet sind. Die Verabredung steht drueben in PHP; hier
//! steht die zweite Haelfte derselben Verabredung in Rust.
//!
//! **Was auf dem Spiel steht, wenn die beiden auseinanderlaufen.** Nichts
//! schlaegt fehl. Der Vergleich haelt jede Fassung der Gegenseite fuer
//! geaendert, und aus dem haeufigsten Fall ueberhaupt -- "drueben geaendert,
//! hier unangetastet" -- wird eine Konfliktkopie. Bei jedem Lauf, fuer jeden
//! Termin. Der Fehler zeigt sich als Unordnung, nicht als Fehlermeldung.
//!
//! **Deshalb wird das JSON hier von Hand geschrieben und nicht von
//! `serde_json`.** Die Zusage lautet: genau die Bytes, die PHPs
//! `json_encode($felder, JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES)`
//! erzeugt. `serde_json` trifft dieselbe Wahl an fast allen Stellen -- aber
//! nicht an allen: **die beiden Zeilentrenner U+2028 und U+2029 schreibt
//! PHP lang aus, `serde_json` laesst sie roh stehen.** Ein Text, der aus
//! einer Webseite kopiert wurde, kann sie enthalten; dann haette derselbe
//! Termin auf beiden Seiten verschiedene Abdruecke. Eine Bibliothek, die uns
//! diese Wahl abnimmt, ist hier die falsche Bibliothek -- sie darf ihre
//! Ausgabe naemlich jederzeit aendern, ohne dass sie damit etwas bricht.

use sha2::{Digest, Sha256};

/// Die neun Felder in der verabredeten Reihenfolge.
///
/// Als Struktur und nicht als Abbildung: So ist die Reihenfolge eine
/// Eigenschaft des Typs und nicht der Einfuegereihenfolge des Aufrufers --
/// genau der Punkt, an dem drueben `hash()` ausdruecklich ueber die
/// Verabredung laeuft statt ueber die uebergebenen Felder.
///
/// Wer ein Feld ergaenzt, entwertet damit **jeden vorhandenen
/// Ursprungsstand** auf beiden Seiten. Das ist kein Grund, es nie zu tun --
/// aber einer, es zu wissen.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Terminfelder {
    pub titel: String,
    pub beschreibung: String,
    pub ort: String,
    /// `Y-m-d\TH:i:s`, ohne Zone -- die Wanduhrzeit, wie sie drueben im Cast
    /// steht. Leer heisst "nicht gesetzt" und ist nicht dasselbe wie 1970.
    pub beginn: String,
    pub ende: String,
    pub ganztags: bool,
    pub rrule: String,
    pub rrule_bis: String,
    pub exdates: String,
}

/// Die Namen ueber der Leitung -- englisch, wie das ganze Delta-Protokoll.
///
/// Sie stehen absichtlich getrennt von den Feldnamen der Struktur: Innen wird
/// deutsch geredet wie im ganzen Haus, aussen steht der Vertrag. Wer die
/// Struktur umbenennt, aendert damit nicht versehentlich das Protokoll.
const NAMEN: [&str; 9] = [
    "title",
    "description",
    "location",
    "start",
    "end",
    "all_day",
    "rrule",
    "rrule_until",
    "exdates",
];

impl Terminfelder {
    /// Die Felder aus einem Delta-Eintrag der Gegenseite.
    ///
    /// Fehlende Felder werden zur leeren Zeichenkette und **nicht
    /// uebersprungen**: Ein Abdruck ueber acht Felder waere ein anderer als
    /// ueber neun. Eine aeltere Gegenstelle, die ein Feld nicht kennt,
    /// ergaebe sonst bei jedem Lauf einen Konflikt statt einer
    /// Uebereinstimmung.
    ///
    /// Fremde Felder im Eintrag (`calendar`, `hash`, `updated_at`) gehen
    /// nicht ein -- der Abdruck kennt nur die neun.
    pub fn aus_eintrag(eintrag: &serde_json::Map<String, serde_json::Value>) -> Self {
        let text = |name: &str| -> String {
            match eintrag.get(name) {
                Some(serde_json::Value::String(s)) => s.clone(),
                // Eine Zahl oder ein `null` an einer Textstelle ist ein
                // kaputter Eintrag, kein Grund zum Abbruch: PHPs `(string)`
                // macht daraus ebenfalls etwas Leeres beziehungsweise die
                // Ziffern. Hier reicht "leer" -- der Abdruck weicht dann ab
                // und die Sache wird als geaendert behandelt, was sie ist.
                Some(serde_json::Value::Number(n)) => n.to_string(),
                _ => String::new(),
            }
        };

        Self {
            titel: text("title"),
            beschreibung: text("description"),
            ort: text("location"),
            beginn: text("start"),
            ende: text("end"),
            // `(bool)` drueben: alles Wahrhaftige zaehlt. Hier eng gefasst
            // auf das, was PHP tatsaechlich schickt -- `true`/`false`.
            ganztags: eintrag
                .get("all_day")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            rrule: text("rrule"),
            rrule_bis: text("rrule_until"),
            exdates: text("exdates"),
        }
    }

    /// Dieselben Felder wieder als Eintragsfelder -- zum Hinausschicken.
    pub fn als_eintrag(&self) -> serde_json::Map<String, serde_json::Value> {
        let mut m = serde_json::Map::new();

        m.insert(NAMEN[0].into(), self.titel.clone().into());
        m.insert(NAMEN[1].into(), self.beschreibung.clone().into());
        m.insert(NAMEN[2].into(), self.ort.clone().into());
        m.insert(NAMEN[3].into(), self.beginn.clone().into());
        m.insert(NAMEN[4].into(), self.ende.clone().into());
        m.insert(NAMEN[5].into(), self.ganztags.into());
        m.insert(NAMEN[6].into(), self.rrule.clone().into());
        m.insert(NAMEN[7].into(), self.rrule_bis.clone().into());
        m.insert(NAMEN[8].into(), self.exdates.clone().into());

        m
    }

    /// Genau die Bytes, die PHP erzeugt.
    ///
    /// Oeffentlich, obwohl nur [`Self::abdruck`] sie braucht: Wenn zwei Seiten
    /// verschiedene Abdruecke haben, ist die erste Frage "welche Bytes hast
    /// du gehasht?", und eine Antwort darauf soll nicht erst eingebaut werden
    /// muessen.
    pub fn json(&self) -> String {
        let werte: [&str; 9] = [
            &self.titel,
            &self.beschreibung,
            &self.ort,
            &self.beginn,
            &self.ende,
            "", // Platzhalter: all_day ist ein Schalter, kein Text.
            &self.rrule,
            &self.rrule_bis,
            &self.exdates,
        ];

        let mut aus = String::from("{");

        for (i, name) in NAMEN.iter().enumerate() {
            if i > 0 {
                aus.push(',');
            }

            aus.push('"');
            aus.push_str(name);
            aus.push_str("\":");

            if i == 5 {
                aus.push_str(if self.ganztags { "true" } else { "false" });
            } else {
                aus.push('"');
                schreiben(&mut aus, werte[i]);
                aus.push('"');
            }
        }

        aus.push('}');
        aus
    }

    /// Der Abdruck: SHA-256 ueber [`Self::json`], hexadezimal in Kleinschrift
    /// -- so gibt PHPs `hash('sha256', …)` ihn aus.
    pub fn abdruck(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.json().as_bytes());

        hasher
            .finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }
}

/// Der Abdruck ueber die neun Felder eines Delta-Eintrags.
pub fn abdruck_von_eintrag(eintrag: &serde_json::Map<String, serde_json::Value>) -> String {
    Terminfelder::aus_eintrag(eintrag).abdruck()
}

/// Eine Zeichenkette so escapen, wie PHP es mit `JSON_UNESCAPED_UNICODE |
/// JSON_UNESCAPED_SLASHES` tut. Gemessen mit PHP 8.3, nicht aus der
/// Dokumentation abgeschrieben:
///
/// ```text
/// "  -> \"        \  -> \\        /  -> / (roh, wegen des Schalters)
/// 08 -> \b   09 -> \t   0A -> \n   0C -> \f   0D -> \r
/// sonstige < 0x20 -> \u00xx, Hexziffern KLEIN  (auch 0x0B -- kein \v)
/// 0x7F -> roh          Umlaute, Emoji -> roh (wegen des Schalters)
/// U+2028, U+2029 -> lang ausgeschrieben  << die EINZIGE Stelle, an der
///                                          PHP und serde_json auseinandergehen
/// ```
fn schreiben(aus: &mut String, wert: &str) {
    for c in wert.chars() {
        match c {
            '"' => aus.push_str("\\\""),
            '\\' => aus.push_str("\\\\"),
            '\u{08}' => aus.push_str("\\b"),
            '\t' => aus.push_str("\\t"),
            '\n' => aus.push_str("\\n"),
            '\u{0c}' => aus.push_str("\\f"),
            '\r' => aus.push_str("\\r"),
            // Die beiden Zeilentrenner. Sie sind der Grund, warum diese
            // Funktion ueberhaupt von Hand geschrieben ist.
            '\u{2028}' => aus.push_str("\\u2028"),
            '\u{2029}' => aus.push_str("\\u2029"),
            c if (c as u32) < 0x20 => aus.push_str(&format!("\\u{:04x}", c as u32)),
            c => aus.push(c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Die Abdruecke stammen aus PHP 8.3, gerechnet ueber
    /// `EventFingerprint::ausEintrag()` + `::hash()` am 06.09.2026.
    ///
    /// **Feste Werte und keine nachgerechneten**: Ein Test, der beide Seiten
    /// mit derselben Rust-Funktion rechnet, prueft nur, dass die Funktion mit
    /// sich selbst uebereinstimmt. Diese Zahlen sind das, was der Server
    /// wirklich sagt.
    fn php(hex: &str, felder: Terminfelder) {
        assert_eq!(
            felder.abdruck(),
            hex,
            "\nabweichende Bytes: {}\n",
            felder.json()
        );
    }

    #[test]
    fn leerer_termin() {
        php(
            "060835fa478d1deafe23b5864f1cb6769cf9f814d1a7b754bc833f8c3d34bfbd",
            Terminfelder::default(),
        );
    }

    #[test]
    fn schlichter_termin() {
        php(
            "8a25a4ca136e584ecf499150247f7ae606621c87ad4d2d01f9d4636082c969a9",
            Terminfelder {
                titel: "Zahnarzt".into(),
                beginn: "2026-09-10T09:00:00".into(),
                ende: "2026-09-10T09:30:00".into(),
                ..Default::default()
            },
        );
    }

    #[test]
    fn ganztags_ist_ein_schalter_und_keine_zeichenkette() {
        // `"all_day":true` und nicht `"all_day":"1"`. Der Unterschied kostet
        // nichts, solange niemand ihn macht -- und alles, sobald doch.
        php(
            "6b8bead5298693283aead69ecd1d78569af3f3efa2556038299c8bdf3bc01276",
            Terminfelder {
                titel: "Urlaub".into(),
                beginn: "2026-09-10T00:00:00".into(),
                ende: "2026-09-17T00:00:00".into(),
                ganztags: true,
                ..Default::default()
            },
        );
    }

    #[test]
    fn umlaute_schraegstriche_und_zeilenumbrueche() {
        // Drei Zusagen auf einmal: `ü` bleibt roh (UNESCAPED_UNICODE), `/`
        // bleibt roh (UNESCAPED_SLASHES), `\n` und `\t` werden kurz gesetzt.
        php(
            "6a5ec1dcd3b954e2f3aa4ca47c30609a98e2ba153ab5f48bd087b433528ddb91",
            Terminfelder {
                titel: "Grillen bei Müller & Söhne".into(),
                beschreibung: "Zeile 1\nZeile 2\tEnde".into(),
                ort: "Straße/Weg 7".into(),
                ..Default::default()
            },
        );
    }

    #[test]
    fn eine_serie_mit_ausnahmen() {
        php(
            "52fe5372cbcfe6103ed47995b37daf0264d59256b662544322fc7b7172a5b917",
            Terminfelder {
                titel: "Standup".into(),
                beginn: "2026-09-07T09:00:00".into(),
                ende: "2026-09-07T09:15:00".into(),
                rrule: "FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR".into(),
                rrule_bis: "2026-12-31T00:00:00".into(),
                exdates: "2026-10-03,2026-12-25".into(),
                ..Default::default()
            },
        );
    }

    #[test]
    fn anfuehrungszeichen_und_backslash() {
        php(
            "019086597aeb70462817cac62368e1a63407e0284773a168543a0a1236c11bd3",
            Terminfelder {
                titel: r#"Sie sagte "ja" \ nein"#.into(),
                ..Default::default()
            },
        );
    }

    #[test]
    fn emoji_bleibt_roh() {
        php(
            "bc5516bbe143420d3d3d7b7663b481a329682908771803aae3c6e8c938af1d87",
            Terminfelder {
                titel: "Geburtstag 🎂".into(),
                ..Default::default()
            },
        );
    }

    #[test]
    fn steuerzeichen_wird_lang_gesetzt_mit_kleinen_hexziffern() {
        php(
            "5639102f7f6d82aeb1cab8e745e5e6eac5cd43a7138a16f01637f42942875e34",
            Terminfelder {
                titel: "a\u{01}b".into(),
                ..Default::default()
            },
        );
    }

    #[test]
    fn del_bleibt_roh() {
        // 0x7F ist kein Steuerzeichen im Sinne von JSON -- PHP schreibt es
        // roh, und wer hier `< 0x20` zu `< 0x80` verallgemeinert, bricht das.
        php(
            "639fed39f9606e661f69dd43b90ad7baf58180df59d96b5876e3d112e3bbe90f",
            Terminfelder {
                titel: "a\u{7f}b".into(),
                ..Default::default()
            },
        );
    }

    /// Der Test, um dessentwillen der Kodierer von Hand geschrieben ist.
    #[test]
    fn zeilentrenner_werden_geflohen_wie_bei_php() {
        php(
            "8ef4a6207c616fabfa6d86e2fa61a928d12f2835bcb9a6803f99179d05ae5ace",
            Terminfelder {
                titel: "a\u{2028}b".into(),
                ..Default::default()
            },
        );
        php(
            "3d35dc8338bca2546c0440a43867f4caa2f15467f9feeef6c618ee668e4cd145",
            Terminfelder {
                titel: "a\u{2029}b".into(),
                ..Default::default()
            },
        );
    }

    /// Und der Beleg, dass es nicht ohne ginge.
    #[test]
    fn serde_json_wuerde_hier_andere_bytes_schreiben() {
        let felder = Terminfelder {
            titel: "a\u{2028}b".into(),
            ..Default::default()
        };

        let ueber_serde =
            serde_json::to_string(&serde_json::Value::Object(felder.als_eintrag())).unwrap();

        assert!(
            felder.json().contains("\\u2028"),
            "PHP schreibt den Zeilentrenner lang"
        );
        assert!(
            ueber_serde.contains('\u{2028}'),
            "serde_json schreibt ihn roh -- deshalb der eigene Kodierer"
        );
    }

    #[test]
    fn fremde_felder_gehen_nicht_in_den_abdruck_ein() {
        // Ein Delta-Eintrag traegt mehr, als der Abdruck kennt: `calendar`,
        // `hash`, `updated_at`. Ginge davon etwas ein, waere ein von einem
        // Kalender in den anderen verschobener Termin ein Konflikt.
        let mut mit = serde_json::Map::new();
        mit.insert("title".into(), "Zahnarzt".into());
        mit.insert("start".into(), "2026-09-10T09:00:00".into());
        mit.insert("end".into(), "2026-09-10T09:30:00".into());
        mit.insert("calendar".into(), "irgendeine-uuid".into());
        mit.insert("hash".into(), "egal".into());
        mit.insert("updated_at".into(), "2026-09-06T10:00:00+02:00".into());

        assert_eq!(
            abdruck_von_eintrag(&mit),
            "8a25a4ca136e584ecf499150247f7ae606621c87ad4d2d01f9d4636082c969a9",
        );
    }

    #[test]
    fn fehlende_felder_zaehlen_als_leer_und_nicht_als_nicht_da() {
        // Eine aeltere Gegenstelle kennt `exdates` nicht. Wuerde das Feld
        // dann fehlen statt leer zu sein, ergaebe jeder Lauf einen Konflikt.
        let mut ohne = serde_json::Map::new();
        ohne.insert("title".into(), "Zahnarzt".into());
        ohne.insert("start".into(), "2026-09-10T09:00:00".into());
        ohne.insert("end".into(), "2026-09-10T09:30:00".into());

        assert_eq!(
            abdruck_von_eintrag(&ohne),
            "8a25a4ca136e584ecf499150247f7ae606621c87ad4d2d01f9d4636082c969a9",
        );
    }

    #[test]
    fn hin_und_zurueck_aendert_den_abdruck_nicht() {
        let felder = Terminfelder {
            titel: "Grillen bei Müller".into(),
            beschreibung: "mit\nUmbruch".into(),
            ort: "Straße/Weg 7".into(),
            beginn: "2026-09-10T09:00:00".into(),
            ende: "2026-09-10T09:30:00".into(),
            ganztags: true,
            rrule: "FREQ=WEEKLY".into(),
            rrule_bis: "2026-12-31T00:00:00".into(),
            exdates: "2026-10-03".into(),
        };

        assert_eq!(
            Terminfelder::aus_eintrag(&felder.als_eintrag()),
            felder,
            "was hinausgeht, muss unveraendert zurueckkommen"
        );
    }
}
