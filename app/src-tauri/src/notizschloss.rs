//! Notizen Ende-zu-Ende (docs/plan-notizen-e2e.md, 6b).
//!
//! Dasselbe Muster wie [`crate::adressbuch`] und [`crate::kalenderschloss`]:
//! **uebersetzt wird an der Grenze zum Server.** Speicher, Laeufer und der
//! Abgleich vor Ort kennen Notizen, Mappenangaben und Anhang-Zuordnungen im
//! Klartext.
//!
//! - **Notiz:** Titel, Mappe und Text in einem Chiffrat. Der Abdruck geht
//!   verdeckt hinaus (wie beim Termin); beim Umstellen zusaetzlich offen
//!   (`base_klar`).
//! - **Mappenangaben, Anhang-Zuordnung:** Ihr Schluessel ist ein Pfad
//!   (`Uni/Hausarbeit`, `Uni/Hausarbeit|bild.png`) -- der verlaesst das
//!   Geraet nicht. Hinaus geht ein **verdeckter Schluessel** (aus dem Pfad
//!   und einem Geheimnis des Kontos), der Pfad steht im Chiffrat.

use openany_client::{Art, Eintrag, Was};
use openany_direktpost::{Kontoschluessel, Verschluesselt};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::adressbuch::Adressbuch;
use crate::mappenteilen::Freigabe;

#[derive(Serialize, Deserialize)]
struct NotizInhalt {
    v: u8,
    zk_id: String,
    title: String,
    #[serde(default)]
    folder: Option<String>,
    content: String,
}

#[derive(Serialize, Deserialize)]
struct AngabeInhalt {
    v: u8,
    /// Der echte Schluessel (Pfad bzw. `Pfad|Datei`).
    key: String,
    felder: Map<String, Value>,
}

/// Arten, deren Schluessel ein Pfad ist und verdeckt hinausgeht.
pub(crate) fn verdeckte_art(art: &Art) -> bool {
    matches!(art, Art::Mappe | Art::Notizanhang)
}

/// Ein Eintrag, den der Server verschluesselt liefert?
pub(crate) fn ist_verschluesselt(e: &Eintrag) -> bool {
    matches!(e.art, Art::Notiz | Art::Mappe | Art::Notizanhang)
        && e.was == Was::Da
        && e.felder.contains_key("inhalt")
}

fn chiffrat(e: &Eintrag) -> Option<(Verschluesselt, u32, i64)> {
    Some((
        Verschluesselt {
            inhalt: e.text("inhalt")?.to_string(),
            signatur: e.text("signatur")?.to_string(),
        },
        e.felder.get("version").and_then(Value::as_u64)? as u32,
        e.felder.get("geraet").and_then(Value::as_i64)?,
    ))
}

impl Adressbuch {
    /// Der verdeckte Schluessel einer Mappe oder Anhang-Zuordnung.
    pub fn schluessel_verdecken(&self, art: &Art, key: &str) -> String {
        self.konto
            .abdruck_verdecken(&format!("schluessel:{}", art.ueber_die_leitung()), key)
    }

    fn zu(&self, art: &Art, key: &str, klartext: &[u8]) -> Result<Eintrag, String> {
        self.zu_mit(&self.konto, art, key, klartext)
    }

    fn zu_mit(
        &self,
        schluessel: &Kontoschluessel,
        art: &Art,
        key: &str,
        klartext: &[u8],
    ) -> Result<Eintrag, String> {
        let (v, version) = openany_direktpost::sache_verschluesseln(
            &self.ich,
            schluessel,
            art.ueber_die_leitung(),
            key,
            klartext,
        )
        .map_err(|e| e.to_string())?;
        Ok(Eintrag::neu(art.clone(), key.to_string(), Was::Da)
            .mit("inhalt", v.inhalt)
            .mit("signatur", v.signatur)
            .mit("geraet", self.geraet)
            .mit("version", version))
    }

    fn auf(&self, e: &Eintrag) -> Option<Vec<u8>> {
        let (v, version, von) = chiffrat(e)?;
        let ed = self.geraete.get(&von)?;
        openany_direktpost::sache_oeffnen(
            &self.konto,
            e.art.ueber_die_leitung(),
            &e.key,
            version,
            &v,
            ed,
        )
        .ok()
    }

    /// Eine Notiz verschliessen. `klar`: die Basis zusaetzlich offen.
    pub fn notiz_verschliessen(&self, e: &Eintrag, klar: bool) -> Result<Eintrag, String> {
        let roh = serde_json::to_vec(&NotizInhalt {
            v: 1,
            zk_id: e.key.clone(),
            title: e.text("title").unwrap_or_default().to_string(),
            folder: e.text("folder").map(str::to_string),
            content: e.text("content").unwrap_or_default().to_string(),
        })
        .map_err(|e| e.to_string())?;
        let hash = e.text("hash").unwrap_or_default();
        let basis = e.text("base_hash").filter(|s| !s.is_empty());
        let mut aus = self
            .zu(&Art::Notiz, &e.key, &roh)?
            .mit("hash", self.konto.abdruck_verdecken(&e.key, hash))
            .mit(
                "base_hash",
                basis.map_or(Value::Null, |b| {
                    self.konto.abdruck_verdecken(&e.key, b).into()
                }),
            );
        for feld in ["size", "updated_at", "origin"] {
            if let Some(w) = e.felder.get(feld) {
                aus = aus.mit(feld, w.clone());
            }
        }
        if klar {
            aus = aus.mit("base_klar", basis.map_or(Value::Null, Value::from));
        }
        Ok(aus)
    }

    /// Eine gezogene Notiz oeffnen -- der Eintrag (Titel, Mappe, Klartext-
    /// Abdruck) und der Text.
    pub fn notiz_oeffnen(&self, e: &Eintrag) -> Option<(Eintrag, String)> {
        let n: NotizInhalt = serde_json::from_slice(&self.auf(e)?).ok()?;
        if n.zk_id != e.key {
            return None;
        }
        let mut aus = Eintrag::neu(Art::Notiz, e.key.clone(), Was::Da)
            .mit("title", n.title)
            .mit("folder", n.folder.map_or(Value::Null, Value::from))
            .mit("hash", openany_client::notizabdruck(&n.content))
            .mit("size", n.content.len() as u64);
        if let Some(t) = e.felder.get("updated_at") {
            aus = aus.mit("updated_at", t.clone());
        }
        Some((aus, n.content))
    }

    /// Mappenangaben oder Anhang-Zuordnung verschliessen -- unter
    /// verdecktem Schluessel. Grabsteine bekommen nur den neuen Schluessel.
    /// Eine Anhang-Zuordnung in einer geteilten Mappe (7c) mit deren
    /// Schluessel (`f`); verdeckt bleibt der Schluessel mit dem Konto.
    pub fn angabe_verschliessen(
        &self,
        e: &Eintrag,
        f: Option<&Freigabe>,
    ) -> Result<Eintrag, String> {
        let verdeckt = self.schluessel_verdecken(&e.art, &e.key);
        if e.was != Was::Da {
            let mut aus = e.clone();
            aus.key = verdeckt;
            return Ok(aus);
        }
        let roh = serde_json::to_vec(&AngabeInhalt {
            v: 1,
            key: e.key.clone(),
            felder: e.felder.clone(),
        })
        .map_err(|e| e.to_string())?;
        let mut aus = match f {
            Some(f) => self
                .zu_mit(&f.schluessel, &e.art, &verdeckt, &roh)?
                .mit("freigabe", f.uuid.clone()),
            None => self.zu(&e.art, &verdeckt, &roh)?,
        };
        if let Some(t) = e.felder.get("updated_at") {
            aus = aus.mit("updated_at", t.clone());
        }
        Ok(aus)
    }

    /// Eine gezogene Angabe oeffnen -- mit ihrem echten Schluessel. Liegt sie
    /// in einer eigenen geteilten Mappe, mit deren Schluessel (`f`).
    pub fn angabe_oeffnen(&self, e: &Eintrag, f: Option<&Freigabe>) -> Option<Eintrag> {
        let roh = match f {
            Some(f) => {
                let (v, version, von) = chiffrat(e)?;
                let ed = self.geraete.get(&von)?;
                openany_direktpost::sache_oeffnen(
                    &f.schluessel,
                    e.art.ueber_die_leitung(),
                    &e.key,
                    version,
                    &v,
                    ed,
                )
                .ok()?
            }
            None => self.auf(e)?,
        };
        let a: AngabeInhalt = serde_json::from_slice(&roh).ok()?;
        // Der Server darf eine Angabe nicht unter fremdem Schluessel ausgeben.
        if self.schluessel_verdecken(&e.art, &a.key) != e.key {
            return None;
        }
        let mut aus = Eintrag::neu(e.art.clone(), a.key, Was::Da);
        aus.felder = a.felder;
        Some(aus)
    }
}

/// Eine Anhang-Zuordnung einer geteilten Mappe oeffnen -- fuer Mitglieder:
/// mit dem Mappenschluessel, unterschrieben von einem Geraet des Besitzers.
/// Den verdeckten Schluessel koennen sie nicht nachrechnen (der ist mit dem
/// Konto des Besitzers verdeckt); die Unterschrift muss genuegen.
pub(crate) fn angabe_auf(
    schluessel: &Kontoschluessel,
    ed25519: &str,
    art: &Art,
    verdeckt: &str,
    version: u32,
    v: &Verschluesselt,
) -> Option<Eintrag> {
    let roh = openany_direktpost::sache_oeffnen(
        schluessel,
        art.ueber_die_leitung(),
        verdeckt,
        version,
        v,
        ed25519,
    )
    .ok()?;
    let a: AngabeInhalt = serde_json::from_slice(&roh).ok()?;
    let mut aus = Eintrag::neu(art.clone(), a.key, Was::Da);
    aus.felder = a.felder;
    Some(aus)
}

#[cfg(test)]
mod tests {
    use super::*;
    use openany_direktpost::{Kontoschluessel, Schluessel};
    use std::collections::HashMap;

    fn buch() -> Adressbuch {
        let ich = Schluessel::erzeugen().unwrap();
        let ed = ich.oeffentlich().1;
        Adressbuch {
            ich,
            konto: Kontoschluessel::neu(),
            geraet: 7,
            geraete: HashMap::from([(7, ed)]),
        }
    }

    #[test]
    fn notiz_hin_und_zurueck() {
        let b = buch();
        let e = Eintrag::neu(Art::Notiz, "20261019120000", Was::Da)
            .mit("title", "Beispiel")
            .mit("folder", "Uni/Hausarbeit")
            .mit("content", "Text")
            .mit("hash", openany_client::notizabdruck("Text"))
            .mit("base_hash", "alt");
        let zu = b.notiz_verschliessen(&e, true).unwrap();
        assert!(ist_verschluesselt(&zu));
        for feld in ["title", "folder", "content"] {
            assert!(zu.felder.get(feld).is_none(), "{feld}");
        }
        assert_ne!(zu.text("hash"), e.text("hash"));
        assert_eq!(zu.text("base_klar"), Some("alt"));

        let (auf, text) = b.notiz_oeffnen(&zu).unwrap();
        assert_eq!(text, "Text");
        assert_eq!(auf.text("title"), Some("Beispiel"));
        assert_eq!(auf.text("folder"), Some("Uni/Hausarbeit"));
        assert_eq!(auf.text("hash"), e.text("hash"));
        assert!(buch().notiz_oeffnen(&zu).is_none());
    }

    #[test]
    fn angaben_reisen_unter_verdecktem_schluessel() {
        let b = buch();
        let e = Eintrag::neu(Art::Notizanhang, "Uni|bild.png", Was::Da)
            .mit("folder", "Uni")
            .mit("path", "bild.png")
            .mit("target_uuid", "t-1");
        let zu = b.angabe_verschliessen(&e, None).unwrap();
        assert_eq!(zu.key.len(), 64);
        assert!(!zu.key.contains("Uni"));
        assert!(zu.felder.get("path").is_none());
        let auf = b.angabe_oeffnen(&zu, None).unwrap();
        assert_eq!(auf.key, "Uni|bild.png");
        assert_eq!(auf.text("target_uuid"), Some("t-1"));

        // Unter anderem Schluessel ausgegeben: geht nicht auf.
        let mut verschoben = zu.clone();
        verschoben.key = b.schluessel_verdecken(&Art::Notizanhang, "Uni|anderes.png");
        assert!(b.angabe_oeffnen(&verschoben, None).is_none());

        // Ein Grabstein bekommt nur den verdeckten Schluessel.
        let weg = b
            .angabe_verschliessen(
                &Eintrag::neu(Art::Notizanhang, "Uni|bild.png", Was::Fort),
                None,
            )
            .unwrap();
        assert_eq!(weg.key, zu.key);
        assert_eq!(weg.was, Was::Fort);
    }
}
