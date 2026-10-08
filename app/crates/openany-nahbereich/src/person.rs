//! Die Person hinter den Geraeten (docs/konzept-lokale-mitgliedschaften.md, §3).
//!
//! **Eine Person ist die Menge ihrer Geraete.** Mitglied eines Projekts wird
//! spaeter die Person, nicht das Geraet -- sonst muesste Anna fuer Telefon und
//! Tablet zweimal eingeladen werden, und ein neues Telefon haette keinen
//! Zugang.
//!
//! **Woran ein Dritter erkennt, dass ein Geraet zu Anna gehoert:** an einer
//! Kette von Buergschaften. Jedes Geraet traegt einen Eintrag, den ein anderes
//! Geraet derselben Person unterschrieben hat ("dieses Geraet gehoert zu
//! mir"). Wer Anna vor Ort getroffen hat, kennt EIN Geraet von ihr sicher --
//! und von dort aus traegt die Kette zu allen anderen. Eine Unterschrift von
//! einem Geraet, das nicht in der Kette steht, zaehlt nicht.
//!
//! **Unterschrieben wird mit dem Schluessel, der auch das TLS-Zertifikat
//! traegt.** Kein zweiter Schluessel, den man verlieren oder verwechseln
//! koennte; der Fingerabdruck des Zertifikats ist die Geraete-Id.
//!
//! **Die Personen-Id ist ein Name, keine Sicherheit.** Sie haelt die Geraete
//! eines Menschen zusammen. Was gilt, entscheidet die Kette. Paaren sich zwei
//! eigene Geraete mit verschiedenen Ids, nehmen beide die kleinere -- beide
//! rechnen dasselbe, ohne sich zu einigen, wer "erster" ist. Die andere steht
//! danach unter `frueher`, damit spaetere Verweise auf sie nicht ins Leere
//! gehen.

use crate::tls::fingerabdruck;
use crate::Identitaet;
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::SignatureScheme;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// "Dieses Geraet gehoert zur selben Person wie der Buerge."
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Geraeteeintrag {
    pub fingerabdruck: String,
    pub name: String,
    /// Das Zertifikat des Geraets -- aus ihm folgen Fingerabdruck und
    /// oeffentlicher Schluessel.
    pub zertifikat_pem: String,
    /// Wer buergt (Fingerabdruck). Gleich `fingerabdruck`: das Geraet
    /// stellt sich selbst vor -- das zaehlt nur, wo man es schon kennt.
    pub buerge: String,
    /// ISO-8601. Bei mehreren Eintraegen fuer dasselbe Geraet gilt fuer den
    /// Namen der juengste.
    pub at: String,
    /// RSA-PKCS#1-SHA256 des Buergen, hexadezimal.
    pub signatur: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Person {
    pub personen_id: String,
    /// Wie diese Person sich nennt (darf leer sein).
    #[serde(default)]
    pub name: String,
    /// Ids, die diese Person frueher trug (siehe oben).
    #[serde(default)]
    pub frueher: Vec<String>,
    pub eintraege: Vec<Geraeteeintrag>,
}

/// Ein Geraet, wie die Oberflaeche es zeigt.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Geraet {
    pub fingerabdruck: String,
    pub name: String,
}

#[derive(Debug, thiserror::Error)]
pub enum PersonFehler {
    #[error("The device key cannot be used for signing: {0}")]
    Schluessel(String),
}

fn nachricht(fingerabdruck: &str, name: &str, buerge: &str, at: &str) -> Vec<u8> {
    let mut n = b"openany-geraet-v1".to_vec();
    for teil in [fingerabdruck, name, buerge, at] {
        n.push(0x1f);
        n.extend_from_slice(teil.as_bytes());
    }
    n
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn aus_hex(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok())
        .collect()
}

/// Mit dem Schluessel dieses Geraets unterschreiben.
pub(crate) fn unterschreiben(ident: &Identitaet, nachricht: &[u8]) -> Result<String, PersonFehler> {
    let fehler = |e: &dyn std::fmt::Display| PersonFehler::Schluessel(e.to_string());
    let schluessel =
        PrivateKeyDer::from_pem_slice(ident.schluessel_pem.as_bytes()).map_err(|e| fehler(&e))?;
    let signierer = rustls::crypto::ring::sign::any_supported_type(&schluessel)
        .map_err(|e| fehler(&e))?
        .choose_scheme(&[SignatureScheme::RSA_PKCS1_SHA256])
        .ok_or_else(|| fehler(&"not RSA-PKCS#1-SHA256"))?;
    Ok(hex(&signierer.sign(nachricht).map_err(|e| fehler(&e))?))
}

/// Das Zertifikat als DER -- `None`, wenn es keines ist.
pub(crate) fn der(zertifikat_pem: &str) -> Option<CertificateDer<'static>> {
    CertificateDer::from_pem_slice(zertifikat_pem.as_bytes()).ok()
}

/// Traegt das Zertifikat diese Unterschrift?
pub(crate) fn pruefen(zertifikat_pem: &str, nachricht: &[u8], signatur: &str) -> bool {
    let (Some(zert), Some(sig)) = (der(zertifikat_pem), aus_hex(signatur)) else {
        return false;
    };
    let Ok(ende) = webpki::EndEntityCert::try_from(&zert) else {
        return false;
    };
    ende.verify_signature(webpki::ring::RSA_PKCS1_2048_8192_SHA256, nachricht, &sig)
        .is_ok()
}

impl Geraeteeintrag {
    /// `ident` buergt fuer das Geraet mit diesem Zertifikat.
    pub fn buergen(
        ident: &Identitaet,
        zertifikat_pem: &str,
        name: &str,
    ) -> Result<Self, PersonFehler> {
        let fp = der(zertifikat_pem)
            .map(|d| fingerabdruck(&d))
            .ok_or_else(|| PersonFehler::Schluessel("certificate unreadable".into()))?;
        let at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let signatur = unterschreiben(ident, &nachricht(&fp, name, &ident.fingerabdruck, &at))?;
        Ok(Self {
            fingerabdruck: fp,
            name: name.to_string(),
            zertifikat_pem: zertifikat_pem.to_string(),
            buerge: ident.fingerabdruck.clone(),
            at,
            signatur,
        })
    }

    /// Passt das Zertifikat zum Fingerabdruck? (Ohne Blick auf den Buergen.)
    fn zertifikat_passt(&self) -> bool {
        der(&self.zertifikat_pem).is_some_and(|d| fingerabdruck(&d) == self.fingerabdruck)
    }

    fn signatur_von(&self, buerge_zertifikat_pem: &str) -> bool {
        pruefen(
            buerge_zertifikat_pem,
            &nachricht(&self.fingerabdruck, &self.name, &self.buerge, &self.at),
            &self.signatur,
        )
    }
}

impl Person {
    /// Eine neue Person mit diesem Geraet als erstem.
    pub fn neu(ident: &Identitaet, geraetename: &str) -> Result<Self, PersonFehler> {
        Ok(Self {
            personen_id: uuid::Uuid::new_v4().to_string(),
            name: String::new(),
            frueher: Vec::new(),
            eintraege: vec![Geraeteeintrag::buergen(
                ident,
                &ident.zertifikat_pem,
                geraetename,
            )?],
        })
    }

    /// Die Geraete, fuer die die Kette traegt -- ausgehend von denen, die
    /// man sicher kennt (`vertraut`: Fingerabdruecke, etwa das Geraet, das
    /// man vor Ort getroffen hat). Fingerabdruck -> Zertifikat.
    pub fn gueltige(&self, vertraut: &[&str]) -> BTreeMap<String, String> {
        let mut gueltig: BTreeMap<String, String> = BTreeMap::new();
        // Die Vertrauten mit ihrem eigenen Zertifikat, so es hier steht.
        for e in &self.eintraege {
            if vertraut.contains(&e.fingerabdruck.as_str()) && e.zertifikat_passt() {
                gueltig.insert(e.fingerabdruck.clone(), e.zertifikat_pem.clone());
            }
        }
        // Dann so lange weiter, wie neue dazukommen.
        loop {
            let vorher = gueltig.len();
            for e in &self.eintraege {
                if gueltig.contains_key(&e.fingerabdruck) || !e.zertifikat_passt() {
                    continue;
                }
                if let Some(zert) = gueltig.get(&e.buerge) {
                    if e.signatur_von(zert) {
                        gueltig.insert(e.fingerabdruck.clone(), e.zertifikat_pem.clone());
                    }
                }
            }
            if gueltig.len() == vorher {
                return gueltig;
            }
        }
    }

    /// Nur die Eintraege, die von `vertraut` aus tragen -- was ein anderes
    /// Geraet schickt, wird so gefiltert, bevor es hier landet.
    pub fn nur_gueltige(&self, vertraut: &[&str]) -> Person {
        let g = self.gueltige(vertraut);
        let mut p = self.clone();
        p.eintraege.retain(|e| {
            g.contains_key(&e.fingerabdruck)
                && (e.buerge == e.fingerabdruck || g.contains_key(&e.buerge))
                && g.get(&e.buerge).is_some_and(|z| e.signatur_von(z))
        });
        p
    }

    /// Die Geraete, mit dem juengsten Namen je Geraet.
    pub fn geraete(&self, vertraut: &[&str]) -> Vec<Geraet> {
        let g = self.gueltige(vertraut);
        let mut namen: BTreeMap<&str, (&str, &str)> = BTreeMap::new();
        for e in &self.eintraege {
            if !g.contains_key(&e.fingerabdruck) {
                continue;
            }
            let neuer = namen
                .get(e.fingerabdruck.as_str())
                .is_none_or(|(at, _)| e.at.as_str() > *at);
            if neuer {
                namen.insert(&e.fingerabdruck, (&e.at, &e.name));
            }
        }
        namen
            .into_iter()
            .map(|(fp, (_, name))| Geraet {
                fingerabdruck: fp.to_string(),
                name: name.to_string(),
            })
            .collect()
    }

    /// Zwei Staende DERSELBEN Person zusammenlegen (eigene, gepaarte Geraete).
    /// Die kleinere Id gilt, mit ihrem Namen; die andere wird `frueher`.
    pub fn zusammenlegen(&mut self, andere: &Person) {
        // Der Name der Id, die gilt -- und ist der leer, der andere. So
        // kommen beide Seiten auf denselben.
        if andere.personen_id < self.personen_id {
            self.frueher.push(std::mem::replace(
                &mut self.personen_id,
                andere.personen_id.clone(),
            ));
            if !andere.name.is_empty() {
                self.name = andere.name.clone();
            }
        } else if andere.personen_id > self.personen_id {
            self.frueher.push(andere.personen_id.clone());
        }
        let mut frueher: BTreeSet<String> = self.frueher.drain(..).collect();
        frueher.extend(andere.frueher.iter().cloned());
        frueher.remove(&self.personen_id);
        self.frueher = frueher.into_iter().collect();
        if self.name.is_empty() {
            self.name = andere.name.clone();
        }
        for e in &andere.eintraege {
            if !self.eintraege.contains(e) {
                self.eintraege.push(e.clone());
            }
        }
    }

    /// Hat `buerge` schon fuer `fingerabdruck` gebuergt?
    pub fn hat_buergschaft(&self, fingerabdruck: &str, buerge: &str) -> bool {
        self.eintraege
            .iter()
            .any(|e| e.fingerabdruck == fingerabdruck && e.buerge == buerge)
    }

    /// Den eigenen Geraetenamen neu eintragen (neuer Eintrag, der juengste
    /// gilt).
    pub fn umbenennen(&mut self, ident: &Identitaet, name: &str) -> Result<(), PersonFehler> {
        let schon = self
            .geraete(&[&ident.fingerabdruck])
            .into_iter()
            .any(|g| g.fingerabdruck == ident.fingerabdruck && g.name == name);
        if !schon {
            self.eintraege
                .push(Geraeteeintrag::buergen(ident, &ident.zertifikat_pem, name)?);
        }
        Ok(())
    }
}

/// Was ein gepaartes EIGENES Geraet (`fp_anderer`) von seiner Person
/// schickt, hier aufnehmen -- beim Paaren und bei jedem Abgleich danach.
///
/// Aufgenommen wird nur, was von diesem Geraet aus traegt; die Personen
/// werden eine (kleinere Id gilt), und dieses Geraet buergt fuer das andere,
/// falls es das noch nicht tat. Beide Seiten tun dasselbe -- danach buergen
/// sie fuereinander, und die Kette traegt in beide Richtungen.
pub fn aufnehmen(
    ident: &Identitaet,
    eigene: &mut Person,
    fremde: &Person,
    fp_anderer: &str,
) -> Result<(), PersonFehler> {
    let gut = fremde.nur_gueltige(&[fp_anderer]);
    let gueltig = gut.gueltige(&[fp_anderer]);
    let Some(zertifikat) = gueltig.get(fp_anderer) else {
        return Err(PersonFehler::Schluessel(
            "The other device does not introduce itself.".into(),
        ));
    };
    eigene.zusammenlegen(&gut);
    if !eigene.hat_buergschaft(fp_anderer, &ident.fingerabdruck) {
        let name = gut
            .geraete(&[fp_anderer])
            .into_iter()
            .find(|g| g.fingerabdruck == fp_anderer)
            .map(|g| g.name)
            .unwrap_or_default();
        eigene
            .eintraege
            .push(Geraeteeintrag::buergen(ident, zertifikat, &name)?);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn geraet() -> Identitaet {
        Identitaet::erzeugen().unwrap()
    }

    #[test]
    fn die_kette_traegt_von_einem_bekannten_geraet_zu_allen() {
        let telefon = geraet();
        let tablet = geraet();
        let laptop = geraet();

        let mut anna = Person::neu(&telefon, "Telefon").unwrap();
        // Telefon buergt fuer Tablet, Tablet fuer Laptop.
        anna.eintraege
            .push(Geraeteeintrag::buergen(&telefon, &tablet.zertifikat_pem, "Tablet").unwrap());
        anna.eintraege
            .push(Geraeteeintrag::buergen(&tablet, &laptop.zertifikat_pem, "Laptop").unwrap());

        // NUR IN BUERGSCHAFTSRICHTUNG: Dass das Telefon fuer das Tablet
        // buergt, macht das Telefon nicht glaubwuerdig -- sonst koennte sich
        // jeder in Annas Kette haengen, indem er fuer eines ihrer Geraete
        // buergt.
        assert_eq!(anna.gueltige(&[&tablet.fingerabdruck]).len(), 2);

        // Beim Paaren buergen beide fuereinander. Dann kennt, wer Annas
        // Tablet vor Ort getroffen hat, alle drei.
        anna.eintraege
            .push(Geraeteeintrag::buergen(&tablet, &telefon.zertifikat_pem, "Telefon").unwrap());
        let g = anna.gueltige(&[&tablet.fingerabdruck]);
        assert_eq!(g.len(), 3);
        let namen: Vec<String> = anna
            .geraete(&[&tablet.fingerabdruck])
            .into_iter()
            .map(|g| g.name)
            .collect();
        assert_eq!(namen.len(), 3);
        assert!(namen.contains(&"Laptop".to_string()));

        // Wer keines kennt, kennt keines.
        assert!(anna.gueltige(&[]).is_empty());
    }

    #[test]
    fn ein_fremder_buerge_oder_eine_faelschung_zaehlt_nicht() {
        let telefon = geraet();
        let mallory = geraet();
        let mallorys_laptop = geraet();
        let mut anna = Person::neu(&telefon, "Telefon").unwrap();

        // Mallory buergt fuer sich selbst und seinen Laptop -- er steht
        // nicht in Annas Kette.
        anna.eintraege
            .push(Geraeteeintrag::buergen(&mallory, &mallory.zertifikat_pem, "Ich").unwrap());
        anna.eintraege.push(
            Geraeteeintrag::buergen(&mallory, &mallorys_laptop.zertifikat_pem, "Laptop").unwrap(),
        );
        // Und faelscht einen Eintrag im Namen von Annas Telefon.
        // Mallory buergt fuer Annas Telefon -- das macht ihn nicht zu Anna.
        anna.eintraege
            .push(Geraeteeintrag::buergen(&mallory, &telefon.zertifikat_pem, "Telefon").unwrap());
        let mut falsch =
            Geraeteeintrag::buergen(&mallory, &mallorys_laptop.zertifikat_pem, "Tablet").unwrap();
        falsch.buerge = telefon.fingerabdruck.clone();
        anna.eintraege.push(falsch);

        let g = anna.gueltige(&[&telefon.fingerabdruck]);
        assert_eq!(g.keys().collect::<Vec<_>>(), [&telefon.fingerabdruck]);
        assert_eq!(
            anna.nur_gueltige(&[&telefon.fingerabdruck]).eintraege.len(),
            1
        );
    }

    #[test]
    fn zusammenlegen_nimmt_die_kleinere_id_auf_beiden_seiten() {
        let telefon = geraet();
        let tablet = geraet();
        let mut a = Person::neu(&telefon, "Telefon").unwrap();
        let mut b = Person::neu(&tablet, "Tablet").unwrap();
        a.name = "Anna".into();
        let (ida, idb) = (a.personen_id.clone(), b.personen_id.clone());

        let (a0, b0) = (a.clone(), b.clone());
        a.zusammenlegen(&b0);
        b.zusammenlegen(&a0);
        assert_eq!(a.personen_id, b.personen_id);
        assert_eq!(a.personen_id, ida.clone().min(idb.clone()));
        assert_eq!(a.frueher, [ida.max(idb)]);
        assert_eq!(a.eintraege.len(), 2);
        assert_eq!(a.name, "Anna");
        assert_eq!(b.name, "Anna");
    }

    #[test]
    fn umbenennen_gilt_mit_dem_juengsten_eintrag() {
        let telefon = geraet();
        let mut p = Person::neu(&telefon, "Telefon").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(5));
        p.umbenennen(&telefon, "Annas Telefon").unwrap();
        p.umbenennen(&telefon, "Annas Telefon").unwrap();
        assert_eq!(p.eintraege.len(), 2);
        assert_eq!(
            p.geraete(&[&telefon.fingerabdruck])[0].name,
            "Annas Telefon"
        );
    }

    #[test]
    fn nach_dem_aufnehmen_buergen_beide_fuereinander() {
        let telefon = geraet();
        let tablet = geraet();
        let mut a = Person::neu(&telefon, "Telefon").unwrap();
        let mut b = Person::neu(&tablet, "Tablet").unwrap();

        // Tablet fragt Telefon: Telefon nimmt auf, antwortet; Tablet nimmt
        // die Antwort auf und schickt noch einmal.
        aufnehmen(&telefon, &mut a, &b.clone(), &tablet.fingerabdruck).unwrap();
        aufnehmen(&tablet, &mut b, &a.clone(), &telefon.fingerabdruck).unwrap();
        aufnehmen(&telefon, &mut a, &b.clone(), &tablet.fingerabdruck).unwrap();

        assert_eq!(a.personen_id, b.personen_id);
        // Wer nur das Telefon kennt, kennt beide -- und umgekehrt.
        assert_eq!(a.gueltige(&[&telefon.fingerabdruck]).len(), 2);
        assert_eq!(a.gueltige(&[&tablet.fingerabdruck]).len(), 2);
        assert_eq!(b.gueltige(&[&telefon.fingerabdruck]).len(), 2);

        // Ein Fremder ohne eigenen Eintrag wird abgewiesen.
        let mallory = geraet();
        let leer = Person {
            eintraege: vec![],
            ..b.clone()
        };
        assert!(aufnehmen(&telefon, &mut a, &leer, &mallory.fingerabdruck).is_err());
    }
}
