//! Verschluesselte Inhalte zwischen Geraet und openany.de
//! (docs/plan-dateien-e2e.md, 7b).
//!
//! **Holen:** Das Chiffrat kommt ganz (in Stuecken) in eine Zwischendatei,
//! wird gegen den Abdruck im Kopf geprueft, entschluesselt und unter dem
//! Abdruck des Klartexts abgelegt -- dort, wo jeder andere Teil des
//! Programms Inhalte sucht. Videos und grosse Dateien gehen so nie ganz
//! durch den Speicher.
//!
//! **Hochladen:** Der Klartext aus der Ablage wird in eine Zwischendatei
//! verschluesselt und mit seinem Kopf hochgeladen; die Vorschau eines Bilds
//! geht mit demselben Schluessel hinterher. Erst nach dem Gelingen merkt sich
//! das Geraet Schluessel und „liegt dort".

use openany_client::{Hochzuladen, OpenanyError, STUECK_ZUM_SERVER};
use openany_direktpost::dateischloss;
use openany_store::Inhaltsschluessel;
use std::path::Path;

use crate::adressbuch::Adressbuch;
use crate::servergegenstelle::ServerGegenstelle;
use crate::Zustand;

/// So viel je Anfrage beim Holen.
const STUECK_VOM_SERVER: u64 = 8 * 1024 * 1024;

/// Raeumt eine Zwischendatei weg, wie auch immer es ausgeht.
struct Weg<'a>(&'a Path);
impl Drop for Weg<'_> {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(self.0);
    }
}

/// Einen verschluesselten Inhalt holen und ablegen.
pub(crate) async fn holen(
    zustand: &Zustand,
    gegenstelle: &ServerGegenstelle,
    k: &Inhaltsschluessel,
) -> Result<(), String> {
    let Some((art, uuid)) = gegenstelle.sache(&k.abdruck).cloned() else {
        return Err(format!("No file is known here for {}.", k.abdruck));
    };
    let chiffrat = zustand.inhalte.zwischenpfad().map_err(|e| e.to_string())?;
    let _weg = Weg(&chiffrat);
    {
        use std::io::Write;
        let mut f = std::fs::File::create(&chiffrat).map_err(|e| e.to_string())?;
        let mut von = 0u64;
        loop {
            let stueck = gegenstelle
                .client()
                .inhalt_bereich(&art, &uuid, von, STUECK_VOM_SERVER)
                .await
                .map_err(|e| e.to_string())?;
            if stueck.is_empty() {
                break;
            }
            f.write_all(&stueck).map_err(|e| e.to_string())?;
            von += stueck.len() as u64;
            if (stueck.len() as u64) < STUECK_VOM_SERVER || (k.groesse > 0 && von >= k.groesse) {
                break;
            }
        }
        f.sync_all().map_err(|e| e.to_string())?;
    }
    let klar = zustand.inhalte.zwischenpfad().map_err(|e| e.to_string())?;
    let _weg2 = Weg(&klar);
    // Den Chiffrat-Abdruck nicht verlangen: Haben zwei Sachen denselben
    // Inhalt, haengt drueben an jeder ein eigenes Chiffrat (age verschluesselt
    // nie zweimal gleich), gemerkt ist aber nur eines. Der Klartext-Abdruck
    // darunter entscheidet.
    let abdruck = dateischloss::datei_oeffnen(&chiffrat, &klar, &k.schluessel, None)
        .map_err(|e| e.to_string())?;
    if abdruck != k.abdruck {
        return Err("The decrypted content does not match its header.".into());
    }
    zustand
        .inhalte
        .uebernehmen(&klar)
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Was zum verschluesselten Hochladen einer Datei oder eines Bilds gehoert.
pub(crate) struct Hinauf<'a> {
    /// `file_node` oder `media`.
    pub art: &'a str,
    pub uuid: &'a str,
    /// Klartext-Abdruck des Inhalts.
    pub abdruck: &'a str,
    /// Der Eintrag, wie ihn der Abgleich schickte -- er wird zum Kopf.
    pub eintrag: openany_client::Eintrag,
    /// Klartext-Abdruck der Vorschau (Bilder), falls hier vorhanden.
    pub vorschau: Option<&'a str>,
    /// Der bisherige Schluessel dieses Inhalts, falls er schon einen hat.
    pub schluessel: Option<Inhaltsschluessel>,
}

/// Verschluesseln, hochladen, die Vorschau hinterher. Gibt die gemerkten
/// Schluessel (Inhalt, Vorschau) zurueck.
pub(crate) async fn hochladen(
    zustand: &Zustand,
    gegenstelle: &ServerGegenstelle,
    buch: &Adressbuch,
    h: Hinauf<'_>,
) -> Result<Vec<Inhaltsschluessel>, OpenanyError> {
    let fehler = |e: String| OpenanyError::Unlesbar(e);
    let Some(quelle) = zustand.inhalte.pfad(h.abdruck).filter(|p| p.is_file()) else {
        return Err(fehler("The content is not on this device.".into()));
    };
    let schluessel = h
        .schluessel
        .as_ref()
        .map(|k| k.schluessel.clone())
        .unwrap_or_else(dateischloss::neuer_schluessel);

    // Die Vorschau zuerst verschluesseln: Ihr Abdruck gehoert in den Kopf.
    let vorschau = match h
        .vorschau
        .and_then(|v| zustand.inhalte.pfad(v).map(|p| (v, p)))
    {
        Some((v, p)) if p.is_file() => {
            let klar = std::fs::read(&p).map_err(|e| fehler(e.to_string()))?;
            let c = dateischloss::bytes_verschluesseln(&schluessel, &klar)
                .map_err(|e| fehler(e.to_string()))?;
            Some((v.to_string(), c))
        }
        _ => None,
    };
    let vorschau_abdruck = vorschau.as_ref().map(|(_, c)| {
        use sha2::{Digest, Sha256};
        Sha256::digest(c)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    });

    let chiffrat = zustand
        .inhalte
        .zwischenpfad()
        .map_err(|e| fehler(e.to_string()))?;
    let _weg = Weg(&chiffrat);
    let info = dateischloss::datei_verschluesseln(&quelle, &chiffrat, &schluessel)
        .map_err(|e| fehler(e.to_string()))?;
    let k = Inhaltsschluessel {
        abdruck: h.abdruck.to_string(),
        schluessel: schluessel.clone(),
        chiffrat: info.chiffrat_abdruck.clone(),
        groesse: info.chiffrat_groesse,
    };
    let kopf = buch
        .upload_kopf(
            &h.eintrag,
            &k,
            vorschau_abdruck.as_deref(),
            gegenstelle.freigabe_fuer(h.uuid),
        )
        .map_err(fehler)?;

    let client = gegenstelle.client();
    let ziel = Hochzuladen {
        art: h.art,
        schluessel: Some(h.uuid),
        eltern: None,
        zone: None,
        // Der Name steht im Kopf; der Server bekommt keinen.
        name: h.uuid,
        groesse: info.chiffrat_groesse,
        abdruck: &info.chiffrat_abdruck,
        herkunft: None,
    };
    let uebertragung = client.upload_beginnen_mit_kopf(&ziel, Some(&kopf)).await?;
    let mut versatz = uebertragung.angekommen;
    {
        use std::io::{Read, Seek, SeekFrom};
        let mut f = std::fs::File::open(&chiffrat).map_err(|e| fehler(e.to_string()))?;
        while versatz < info.chiffrat_groesse {
            f.seek(SeekFrom::Start(versatz))
                .map_err(|e| fehler(e.to_string()))?;
            let mut puffer = vec![0u8; STUECK_ZUM_SERVER as usize];
            let n = f.read(&mut puffer).map_err(|e| fehler(e.to_string()))?;
            if n == 0 {
                let _ = client.upload_aufgeben(&uebertragung.token).await;
                return Err(fehler(
                    "The encrypted file is shorter than announced.".into(),
                ));
            }
            puffer.truncate(n);
            versatz = client
                .stueck_schicken(&uebertragung.token, versatz, puffer)
                .await?;
        }
    }
    client.upload_abschliessen(&uebertragung.token).await?;

    let mut gemerkt = vec![k];
    if let (Some((v, c)), Some(va)) = (vorschau, vorschau_abdruck) {
        // Die Vorschau gehoert zum Bild; scheitert sie, bleibt das Bild
        // trotzdem oben -- der naechste Lauf reicht sie nach.
        if client.media_standbild(h.uuid, c, None).await.is_ok() {
            gemerkt.push(Inhaltsschluessel {
                abdruck: v,
                schluessel,
                chiffrat: va,
                groesse: 0,
            });
        }
    }
    Ok(gemerkt)
}
