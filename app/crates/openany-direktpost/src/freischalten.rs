//! Ein Geraet schaltet ein Geraet frei (Z4, docs/plan-zero-knowledge.md).
//!
//! Das NEUE Geraet wuerfelt 6 Ziffern und zeigt sie an. Das ALTE (schon
//! freigegebene) tippt sie ein. Beide fuehren damit einen SPAKE2 ueber den
//! Briefkasten bei anyid. Die Ziffern sind das Passwort und gehen nie an den
//! Server. Am Ende kennt das alte Geraet die oeffentlichen Schluessel des
//! neuen -- genau die, die auf dem Bildschirm daneben erzeugt wurden -- und
//! legt ihm danach die Kiste mit dem Kontoschluessel.
//!
//! WARUM NICHT MEHR DER ZIFFERNVERGLEICH (bis Z4 `code`). Dort entstanden
//! die Ziffern aus den Schluesseln des neuen Geraets. Ein Server, der luegt,
//! erzeugte sich eigene Schluessel, bis die Ziffern passten -- im Mittel eine
//! halbe Million Versuche, also Sekunden. Beim SPAKE2 hat er je Anfrage
//! EINEN Versuch: Er muss die Ziffern raten, bevor er etwas sieht, und ein
//! Fehlschlag beendet die Anfrage.
//!
//! Der Ablauf, drei Nachrichten:
//!
//! 1. neu → alt: SPAKE2-Nachricht A.
//! 2. alt → neu: SPAKE2-Nachricht B und der Beweis des alten Geraets, dass
//!    es dieselben Ziffern hat.
//! 3. neu → alt: die oeffentlichen Schluessel des neuen Geraets und sein
//!    Beweis, gebunden an diese Schluessel.
//!
//! Beide Beweise haengen an der Kennung der Anfrage und an beiden
//! SPAKE2-Nachrichten. Eine Nachricht aus einer anderen Anfrage passt nicht.

use base64::{engine::general_purpose::STANDARD as B64, Engine};
use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use spake2::{Ed25519Group, Identity, Password, Spake2};

const FASSUNG: u32 = 1;
const KENNUNG_NEU: &[u8] = b"openany-freischalten-v1-neu";
const KENNUNG_ALT: &[u8] = b"openany-freischalten-v1-alt";

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Freischaltfehler {
    /// Eingetippt wurden nicht genau 6 Ziffern.
    #[error("enter exactly 6 digits")]
    Ziffern,
    /// Die Nachricht ist kaputt oder gehoert nicht an diese Stelle.
    #[error("unexpected message: {0}")]
    Nachricht(String),
    /// Die Ziffern stimmen nicht ueberein -- oder jemand dazwischen hat
    /// geraten. Beides heisst: Anfrage verwerfen, neu anfangen.
    #[error("the digits do not match")]
    StimmtNicht,
}

/// Die oeffentlichen Schluessel eines Geraets, wie [`crate::Schluessel::oeffentlich`]
/// sie liefert.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Geraeteschluessel {
    pub age: String,
    pub ed25519: String,
}

#[derive(Serialize, Deserialize)]
struct Nachricht {
    v: u32,
    schritt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pake: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    beweis: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    schluessel: Option<Geraeteschluessel>,
}

impl Nachricht {
    fn neu(schritt: &str) -> Self {
        Self {
            v: FASSUNG,
            schritt: schritt.to_string(),
            pake: None,
            beweis: None,
            schluessel: None,
        }
    }

    fn als_text(&self) -> String {
        serde_json::to_string(self).expect("Nachricht laesst sich immer schreiben")
    }

    fn lesen(text: &str, schritt: &str) -> Result<Self, Freischaltfehler> {
        let n: Nachricht =
            serde_json::from_str(text).map_err(|e| Freischaltfehler::Nachricht(e.to_string()))?;
        if n.v != FASSUNG || n.schritt != schritt {
            return Err(Freischaltfehler::Nachricht(format!(
                "expected {schritt} v{FASSUNG}"
            )));
        }
        Ok(n)
    }
}

/// 6 Ziffern, gleichverteilt, zum Anzeigen: `"123 456"`.
pub fn ziffern_wuerfeln() -> String {
    // Unterhalb eines Vielfachen von 1 000 000, sonst kaemen kleine Zahlen
    // ein wenig haeufiger vor.
    const GRENZE: u32 = 4_294_000_000;
    loop {
        let mut b = [0u8; 4];
        getrandom::fill(&mut b).expect("Zufall des Systems");
        let z = u32::from_be_bytes(b);
        if z < GRENZE {
            let z = z % 1_000_000;
            return format!("{:03} {:03}", z / 1000, z % 1000);
        }
    }
}

/// Getippte Ziffern auf die 6 Ziffern selbst bringen. Leerzeichen und
/// Bindestriche duerfen dabei sein, sonst nichts.
fn ziffern_lesen(eingabe: &str) -> Result<String, Freischaltfehler> {
    let mut ziffern = String::new();
    for c in eingabe.chars() {
        match c {
            '0'..='9' => ziffern.push(c),
            ' ' | '-' | '\u{a0}' => {}
            _ => return Err(Freischaltfehler::Ziffern),
        }
    }
    if ziffern.len() == 6 {
        Ok(ziffern)
    } else {
        Err(Freischaltfehler::Ziffern)
    }
}

fn b64_lesen(feld: Option<String>, name: &str) -> Result<Vec<u8>, Freischaltfehler> {
    let text = feld.ok_or_else(|| Freischaltfehler::Nachricht(format!("{name} missing")))?;
    B64.decode(text)
        .map_err(|e| Freischaltfehler::Nachricht(e.to_string()))
}

/// Die beiden Schluessel fuer die Beweise und das Protokoll, an dem sie
/// haengen.
struct Sitzung {
    alt: [u8; 32],
    neu: [u8; 32],
    protokoll: [u8; 32],
}

impl Sitzung {
    fn ableiten(geheim: &[u8], anfrage: &str, a: &[u8], b: &[u8]) -> Self {
        let hk = Hkdf::<Sha256>::new(Some(b"openany-freischalten-v1"), geheim);
        let mut alt = [0u8; 32];
        let mut neu = [0u8; 32];
        hk.expand(b"beweis-alt", &mut alt)
            .expect("32 Bytes passen immer");
        hk.expand(b"beweis-neu", &mut neu)
            .expect("32 Bytes passen immer");

        let mut h = Sha256::new();
        h.update(b"openany-freischalten-v1\n");
        h.update((anfrage.len() as u64).to_be_bytes());
        h.update(anfrage.as_bytes());
        h.update((a.len() as u64).to_be_bytes());
        h.update(a);
        h.update((b.len() as u64).to_be_bytes());
        h.update(b);

        Self {
            alt,
            neu,
            protokoll: h.finalize().into(),
        }
    }

    fn beweis_alt(&self) -> HmacSha256 {
        let mut m = HmacSha256::new_from_slice(&self.alt).expect("HMAC nimmt jede Laenge");
        m.update(&self.protokoll);
        m
    }

    fn beweis_neu(&self, schluessel: &Geraeteschluessel) -> HmacSha256 {
        let mut m = HmacSha256::new_from_slice(&self.neu).expect("HMAC nimmt jede Laenge");
        m.update(&self.protokoll);
        m.update(&(schluessel.age.len() as u64).to_be_bytes());
        m.update(schluessel.age.as_bytes());
        m.update(&(schluessel.ed25519.len() as u64).to_be_bytes());
        m.update(schluessel.ed25519.as_bytes());
        m
    }
}

fn kennungen() -> (Identity, Identity) {
    (Identity::new(KENNUNG_NEU), Identity::new(KENNUNG_ALT))
}

// --- Das neue Geraet -----------------------------------------------------------

/// Die Seite des NEUEN Geraets, zwischen Nachricht 1 und 3.
pub struct NeuesGeraet {
    pake: Spake2<Ed25519Group>,
    anfrage: String,
    a: Vec<u8>,
    schluessel: Geraeteschluessel,
}

impl NeuesGeraet {
    /// Anfangen: mit den eigenen (angezeigten) Ziffern und der Kennung der
    /// Anfrage, die anyid vergeben hat. Liefert Nachricht 1.
    pub fn beginnen(
        anfrage: &str,
        ziffern: &str,
        schluessel: Geraeteschluessel,
    ) -> Result<(Self, String), Freischaltfehler> {
        let passwort = Password::new(ziffern_lesen(ziffern)?);
        let (neu, alt) = kennungen();
        let (pake, a) = Spake2::<Ed25519Group>::start_a(&passwort, &neu, &alt);

        let mut n = Nachricht::neu("neu-1");
        n.pake = Some(B64.encode(&a));

        Ok((
            Self {
                pake,
                anfrage: anfrage.to_string(),
                a,
                schluessel,
            },
            n.als_text(),
        ))
    }

    /// Nachricht 2 lesen und Nachricht 3 liefern -- aber nur, wenn das alte
    /// Geraet bewiesen hat, dass es dieselben Ziffern hat. Sonst gehen die
    /// Schluessel gar nicht erst hinaus.
    pub fn antworten(self, nachricht: &str) -> Result<String, Freischaltfehler> {
        let n = Nachricht::lesen(nachricht, "alt-1")?;
        let b = b64_lesen(n.pake, "pake")?;
        let beweis = b64_lesen(n.beweis, "beweis")?;

        let geheim = self
            .pake
            .finish(&b)
            .map_err(|e| Freischaltfehler::Nachricht(format!("{e:?}")))?;
        let sitzung = Sitzung::ableiten(&geheim, &self.anfrage, &self.a, &b);

        sitzung
            .beweis_alt()
            .verify_slice(&beweis)
            .map_err(|_| Freischaltfehler::StimmtNicht)?;

        let mut antwort = Nachricht::neu("neu-2");
        antwort.beweis =
            Some(B64.encode(sitzung.beweis_neu(&self.schluessel).finalize().into_bytes()));
        antwort.schluessel = Some(self.schluessel);
        Ok(antwort.als_text())
    }
}

// --- Das alte Geraet -----------------------------------------------------------

/// Die Seite des ALTEN Geraets, zwischen Nachricht 2 und 3.
pub struct AltesGeraet {
    sitzung: Sitzung,
}

impl AltesGeraet {
    /// Nachricht 1 lesen, mit den eingetippten Ziffern antworten. Liefert
    /// Nachricht 2.
    pub fn antworten(
        anfrage: &str,
        getippt: &str,
        nachricht: &str,
    ) -> Result<(Self, String), Freischaltfehler> {
        let passwort = Password::new(ziffern_lesen(getippt)?);
        let n = Nachricht::lesen(nachricht, "neu-1")?;
        let a = b64_lesen(n.pake, "pake")?;

        let (neu, alt) = kennungen();
        let (pake, b) = Spake2::<Ed25519Group>::start_b(&passwort, &neu, &alt);
        let geheim = pake
            .finish(&a)
            .map_err(|e| Freischaltfehler::Nachricht(format!("{e:?}")))?;
        let sitzung = Sitzung::ableiten(&geheim, anfrage, &a, &b);

        let mut antwort = Nachricht::neu("alt-1");
        antwort.pake = Some(B64.encode(&b));
        antwort.beweis = Some(B64.encode(sitzung.beweis_alt().finalize().into_bytes()));

        Ok((Self { sitzung }, antwort.als_text()))
    }

    /// Nachricht 3 lesen. Liefert die Schluessel des neuen Geraets -- nur,
    /// wenn sein Beweis zu genau diesen Schluesseln passt.
    pub fn abschliessen(self, nachricht: &str) -> Result<Geraeteschluessel, Freischaltfehler> {
        let n = Nachricht::lesen(nachricht, "neu-2")?;
        let beweis = b64_lesen(n.beweis, "beweis")?;
        let schluessel = n
            .schluessel
            .ok_or_else(|| Freischaltfehler::Nachricht("schluessel missing".into()))?;

        self.sitzung
            .beweis_neu(&schluessel)
            .verify_slice(&beweis)
            .map_err(|_| Freischaltfehler::StimmtNicht)?;

        Ok(schluessel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Schluessel;

    const ANFRAGE: &str = "beispiel-anfrage-0001";

    fn schluessel() -> Geraeteschluessel {
        let (age, ed25519) = Schluessel::erzeugen().unwrap().oeffentlich();
        Geraeteschluessel { age, ed25519 }
    }

    #[test]
    fn mit_denselben_ziffern_bekommt_das_alte_genau_die_schluessel_des_neuen() {
        let ziffern = ziffern_wuerfeln();
        let eigene = schluessel();

        let (neu, n1) = NeuesGeraet::beginnen(ANFRAGE, &ziffern, eigene.clone()).unwrap();
        let (alt, a1) = AltesGeraet::antworten(ANFRAGE, &ziffern, &n1).unwrap();
        let n2 = neu.antworten(&a1).unwrap();

        assert_eq!(alt.abschliessen(&n2).unwrap(), eigene);
    }

    #[test]
    fn getippt_darf_anders_geschrieben_sein() {
        let (neu, n1) = NeuesGeraet::beginnen(ANFRAGE, "123 456", schluessel()).unwrap();
        let (alt, a1) = AltesGeraet::antworten(ANFRAGE, " 123-456 ", &n1).unwrap();
        let n2 = neu.antworten(&a1).unwrap();
        assert!(alt.abschliessen(&n2).is_ok());
    }

    #[test]
    fn falsche_ziffern_das_neue_gibt_seine_schluessel_nicht_heraus() {
        let (neu, n1) = NeuesGeraet::beginnen(ANFRAGE, "123 456", schluessel()).unwrap();
        let (_alt, a1) = AltesGeraet::antworten(ANFRAGE, "123 457", &n1).unwrap();

        assert_eq!(
            neu.antworten(&a1).unwrap_err(),
            Freischaltfehler::StimmtNicht
        );
    }

    #[test]
    fn eine_andere_anfrage_passt_nicht() {
        let (neu, n1) = NeuesGeraet::beginnen(ANFRAGE, "123 456", schluessel()).unwrap();
        let (_alt, a1) = AltesGeraet::antworten("andere-anfrage", "123 456", &n1).unwrap();

        assert_eq!(
            neu.antworten(&a1).unwrap_err(),
            Freischaltfehler::StimmtNicht
        );
    }

    /// DER FALL, UM DEN ES GEHT: Der Briefkasten luegt. Er spielt dem alten
    /// Geraet ein eigenes "neues" vor, mit eigenen Schluesseln und
    /// geratenen Ziffern. Ohne die richtigen Ziffern kommt er nicht durch.
    #[test]
    fn ein_luegender_briefkasten_mit_eigenem_neuen_geraet_kommt_nicht_durch() {
        let echte = "482 913";
        let (_echt, _n1) = NeuesGeraet::beginnen(ANFRAGE, echte, schluessel()).unwrap();

        let (falsch, n1_falsch) = NeuesGeraet::beginnen(ANFRAGE, "000 000", schluessel()).unwrap();
        let (alt, a1) = AltesGeraet::antworten(ANFRAGE, echte, &n1_falsch).unwrap();

        // Das untergeschobene Geraet merkt es selbst ...
        assert_eq!(
            falsch.antworten(&a1).unwrap_err(),
            Freischaltfehler::StimmtNicht
        );

        // ... und wer den Beweis einfach weglaesst oder erfindet, kommt beim
        // alten Geraet nicht durch.
        let mut erfunden = Nachricht::neu("neu-2");
        erfunden.beweis = Some(B64.encode([0u8; 32]));
        erfunden.schluessel = Some(schluessel());
        assert_eq!(
            alt.abschliessen(&erfunden.als_text()).unwrap_err(),
            Freischaltfehler::StimmtNicht
        );
    }

    /// Der Briefkasten laesst den echten Ablauf durch, tauscht aber in der
    /// letzten Nachricht die Schluessel gegen seine eigenen.
    #[test]
    fn ausgetauschte_schluessel_in_der_letzten_nachricht_fallen_auf() {
        let ziffern = "482 913";
        let (neu, n1) = NeuesGeraet::beginnen(ANFRAGE, ziffern, schluessel()).unwrap();
        let (alt, a1) = AltesGeraet::antworten(ANFRAGE, ziffern, &n1).unwrap();
        let n2 = neu.antworten(&a1).unwrap();

        let mut n: Nachricht = serde_json::from_str(&n2).unwrap();
        n.schluessel = Some(schluessel());

        assert_eq!(
            alt.abschliessen(&n.als_text()).unwrap_err(),
            Freischaltfehler::StimmtNicht
        );
    }

    /// Der Briefkasten spielt dem neuen Geraet ein eigenes "altes" vor. Es
    /// gibt seine Schluessel nicht heraus -- die Ziffern kennt der
    /// Briefkasten nicht.
    #[test]
    fn ein_luegender_briefkasten_mit_eigenem_alten_geraet_bekommt_nichts() {
        let (neu, n1) = NeuesGeraet::beginnen(ANFRAGE, "482 913", schluessel()).unwrap();
        let (_falsch, a1_falsch) = AltesGeraet::antworten(ANFRAGE, "111 111", &n1).unwrap();

        assert_eq!(
            neu.antworten(&a1_falsch).unwrap_err(),
            Freischaltfehler::StimmtNicht
        );
    }

    #[test]
    fn kaputte_nachrichten_sind_fehler_und_keine_panik() {
        assert!(matches!(
            AltesGeraet::antworten(ANFRAGE, "123456", "kein json"),
            Err(Freischaltfehler::Nachricht(_))
        ));
        assert!(matches!(
            AltesGeraet::antworten(
                ANFRAGE,
                "123456",
                r#"{"v":1,"schritt":"neu-1","pake":"AAAA"}"#
            ),
            Err(Freischaltfehler::Nachricht(_))
        ));
        assert!(matches!(
            AltesGeraet::antworten(
                ANFRAGE,
                "123456",
                r#"{"v":2,"schritt":"neu-1","pake":"AAAA"}"#
            ),
            Err(Freischaltfehler::Nachricht(_))
        ));

        let (neu, n1) = NeuesGeraet::beginnen(ANFRAGE, "123456", schluessel()).unwrap();
        // Die eigene Nachricht zurueckgespiegelt: falscher Schritt.
        assert!(matches!(
            neu.antworten(&n1),
            Err(Freischaltfehler::Nachricht(_))
        ));
    }

    #[test]
    fn nur_genau_sechs_ziffern() {
        for falsch in ["12345", "1234567", "12a456", "", "123 45x"] {
            assert_eq!(
                NeuesGeraet::beginnen(ANFRAGE, falsch, schluessel()).err(),
                Some(Freischaltfehler::Ziffern),
                "{falsch:?}"
            );
        }
    }

    #[test]
    fn gewuerfelte_ziffern_sehen_aus_wie_angezeigt() {
        for _ in 0..200 {
            let z = ziffern_wuerfeln();
            assert_eq!(z.len(), 7);
            assert_eq!(&z[3..4], " ");
            assert!(ziffern_lesen(&z).is_ok());
        }
    }
}
