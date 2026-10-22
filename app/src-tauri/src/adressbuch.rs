//! Das Adressbuch Ende-zu-Ende (docs/plan-adressbuch-e2e.md, Schritt 2b).
//!
//! **Uebersetzt wird an der Grenze zum Server, sonst nirgends.** Speicher,
//! Laeufer und der Abgleich mit Geraeten in der Naehe kennen den Kontakt wie
//! bisher im Klartext (`display_name`, `channels`, `photo`). Erst die
//! [`ServerGegenstelle`](crate::servergegenstelle::ServerGegenstelle) macht
//! daraus beim Schieben ein Chiffrat ([`Adressbuch::verschliessen`]) und beim
//! Ziehen wieder Klartext ([`Adressbuch::oeffnen`]).
//!
//! Verschluesselt wird an den Konto-Schluessel der Direktnachrichten: Jedes
//! freigegebene eigene Geraet liest, der Server nicht. Das Foto bekommt
//! einen eigenen Schluessel wie ein Anhang; der steht mit seiner Pruefsumme
//! im Kontakt.

use std::collections::HashMap;

use base64::{engine::general_purpose::STANDARD as B64, Engine};
use openany_client::{Art, Eintrag, Was};
use openany_direktpost::{AnhangInfo, Kontoschluessel, Schluessel, Verschluesselt};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Mehr nimmt der Server fuer ein Foto-Chiffrat nicht an (Base64,
/// `SyncApplyController`). Die Oberflaeche verkleinert vorher; was trotzdem
/// groesser ist, reist ohne Bild.
const FOTO_GRENZE: usize = 300_000;

/// Was ein Geraet zum Verschluesseln und Pruefen braucht -- nur, wenn es den
/// Konto-Schluessel schon hat (also freigegeben ist).
pub(crate) struct Adressbuch {
    pub ich: Schluessel,
    pub konto: Kontoschluessel,
    /// Der eigene Eintrag im Verzeichnis -- er unterschreibt.
    pub geraet: i64,
    /// Ed25519 aller eigenen Geraete, nach Eintrag im Verzeichnis.
    pub geraete: HashMap<i64, String>,
}

/// Was im Chiffrat steht.
#[derive(Debug, Serialize, Deserialize)]
struct Inhalt {
    v: u8,
    /// Auch drinnen: Der Klartext bindet sich selbst an seine Zeile.
    uuid: String,
    display_name: String,
    #[serde(default)]
    channels: Value,
    /// Abdruck des Fotos (unverschluesselt) -- so vergleicht der Laeufer wie
    /// bisher. `None`: kein Foto.
    #[serde(default)]
    photo: Option<String>,
    /// Wie das Foto-Chiffrat aufgeht. Fehlt es bei gesetztem `photo`, kennt
    /// dieser Stand das Foto nicht (es lag beim Schreiben nicht vor).
    #[serde(default)]
    foto: Option<AnhangInfo>,
    /// Per Sicherheitsnummer gepruefte Konten (2c). Fehlt in aelteren
    /// Staenden -- dann bleibt, was hier steht.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    geprueft: Option<Value>,
}

impl Adressbuch {
    /// Einen Kontakt zum Hinausschicken verschliessen. `bekannt` ist, was
    /// diese Seite schon zum Foto weiss (aus dem Ziehen eben).
    ///
    /// Nur `Da` -- Papierkorb und Grabstein tragen nichts Lesbares.
    pub fn verschliessen(
        &self,
        e: &Eintrag,
        bekannt: Option<&AnhangInfo>,
    ) -> Result<Eintrag, String> {
        let mut photo = e
            .text("photo")
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        let mut foto = None;
        let mut chiffrat = None;

        match e.felder.get("photo_bytes") {
            Some(Value::String(b)) => {
                let bytes = B64.decode(b.trim()).map_err(|e| e.to_string())?;
                let (c, info) = openany_direktpost::anhang_verschluesseln("foto", "image", &bytes)
                    .map_err(|e| e.to_string())?;
                let c = B64.encode(c);
                if c.len() <= FOTO_GRENZE {
                    chiffrat = Some(Value::String(c));
                    foto = Some(info);
                }
            }
            Some(_) => {
                photo = None;
                chiffrat = Some(Value::Null);
            }
            None => {
                foto = bekannt.filter(|_| photo.is_some()).cloned();
            }
        }

        let klartext = serde_json::to_vec(&Inhalt {
            v: 1,
            uuid: e.key.clone(),
            display_name: e.text("display_name").unwrap_or("Kontakt").to_string(),
            channels: e
                .felder
                .get("channels")
                .cloned()
                .unwrap_or(Value::Array(vec![])),
            photo,
            foto,
            geprueft: e.felder.get("geprueft").cloned(),
        })
        .map_err(|e| e.to_string())?;

        let (v, version) =
            openany_direktpost::kontakt_verschluesseln(&self.ich, &self.konto, &e.key, &klartext)
                .map_err(|e| e.to_string())?;

        let mut aus = Eintrag::neu(Art::Kontakt, e.key.clone(), Was::Da)
            .mit("inhalt", v.inhalt)
            .mit("signatur", v.signatur)
            .mit("geraet", self.geraet)
            .mit("version", version);
        if let Some(t) = e.felder.get("updated_at") {
            aus = aus.mit("updated_at", t.clone());
        }
        if let Some(c) = chiffrat {
            aus = aus.mit("foto_chiffrat", c);
        }
        Ok(aus)
    }

    /// Einen gezogenen Kontakt oeffnen -- `None`, wenn die Unterschrift nicht
    /// passt, das Geraet unbekannt ist oder das Chiffrat nicht aufgeht. Dazu,
    /// wie das Foto aufgeht.
    pub fn oeffnen(&self, e: &Eintrag) -> Option<(Eintrag, Option<AnhangInfo>)> {
        let von = e.felder.get("geraet").and_then(Value::as_i64)?;
        let ed25519 = self.geraete.get(&von)?;
        let version = e.felder.get("version").and_then(Value::as_u64)? as u32;
        let v = Verschluesselt {
            inhalt: e.text("inhalt")?.to_string(),
            signatur: e.text("signatur")?.to_string(),
        };
        let klartext =
            openany_direktpost::kontakt_oeffnen(&self.konto, &e.key, version, &v, ed25519).ok()?;
        let inhalt: Inhalt = serde_json::from_slice(&klartext).ok()?;
        if inhalt.uuid != e.key {
            return None;
        }

        let mut aus = Eintrag::neu(Art::Kontakt, e.key.clone(), Was::Da)
            .mit("display_name", inhalt.display_name)
            .mit("channels", inhalt.channels);
        if let Some(g) = inhalt.geprueft {
            aus = aus.mit("geprueft", g);
        }
        if let Some(t) = e.felder.get("updated_at") {
            aus = aus.mit("updated_at", t.clone());
        }
        // Ohne Schluessel zum Foto bleibt `photo` weg -- dann laesst der
        // Laeufer das Bild hier in Ruhe, statt es zu loeschen.
        match (&inhalt.photo, &inhalt.foto) {
            (None, _) => aus = aus.mit("photo", Value::Null),
            (Some(abdruck), Some(_)) => aus = aus.mit("photo", abdruck.clone()),
            (Some(_), None) => {}
        }
        Some((aus, inhalt.foto))
    }
}

/// Ein Eintrag, den der Server verschluesselt liefert?
pub(crate) fn ist_verschluesselt(e: &Eintrag) -> bool {
    e.art == Art::Kontakt && e.was == Was::Da && e.felder.contains_key("inhalt")
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn kontakt() -> Eintrag {
        Eintrag::neu(Art::Kontakt, "k-1", Was::Da)
            .mit("display_name", "Beispiel")
            .mit(
                "channels",
                serde_json::json!([{ "art": "email", "wert": "a@beispiel.test" }]),
            )
            .mit("photo", "abc")
            .mit("photo_bytes", B64.encode(b"bild"))
            .mit(
                "geprueft",
                serde_json::json!([{ "name": "beispiel", "age": "age1x" }]),
            )
            .mit("updated_at", "2026-10-17T10:00:00+00:00")
    }

    #[test]
    fn hin_und_zurueck() {
        let b = buch();
        let zu = b.verschliessen(&kontakt(), None).unwrap();
        assert!(ist_verschluesselt(&zu));
        assert!(zu.felder.get("display_name").is_none());
        assert!(!zu.text("inhalt").unwrap().contains("Beispiel"));

        let (auf, foto) = b.oeffnen(&zu).unwrap();
        assert_eq!(auf.text("display_name"), Some("Beispiel"));
        assert_eq!(auf.text("photo"), Some("abc"));
        assert_eq!(auf.felder["channels"], kontakt().felder["channels"]);
        assert_eq!(auf.felder["geprueft"], kontakt().felder["geprueft"]);

        let chiffrat = B64.decode(zu.text("foto_chiffrat").unwrap()).unwrap();
        assert_eq!(
            openany_direktpost::anhang_oeffnen(&chiffrat, &foto.unwrap()).unwrap(),
            b"bild"
        );
    }

    #[test]
    fn vertauscht_oder_fremd_geht_nicht_auf() {
        let b = buch();
        let mut zu = b.verschliessen(&kontakt(), None).unwrap();
        zu.key = "k-2".into();
        assert!(b.oeffnen(&zu).is_none());

        let zu = b.verschliessen(&kontakt(), None).unwrap();
        assert!(buch().oeffnen(&zu).is_none());
    }

    /// Ohne Bytes und ohne Wissen ueber das Foto: Es bleibt drueben, wie es
    /// ist, und hier sagt der Kontakt nichts darueber.
    #[test]
    fn ohne_bytes_bleibt_das_foto_unberuehrt() {
        let b = buch();
        let mut e = kontakt();
        e.felder.remove("photo_bytes");
        let zu = b.verschliessen(&e, None).unwrap();
        assert!(zu.felder.get("foto_chiffrat").is_none());
        let (auf, _) = b.oeffnen(&zu).unwrap();
        assert!(auf.felder.get("photo").is_none());

        e.felder.insert("photo_bytes".into(), Value::Null);
        let (auf, _) = b.oeffnen(&b.verschliessen(&e, None).unwrap()).unwrap();
        assert_eq!(auf.felder.get("photo"), Some(&Value::Null));
    }
}
