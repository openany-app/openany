//! Direktnachrichten vor Ort, ohne Server (Tiffy, 02.10.2026) -- die
//! App-Seite zu `openany_nahbereich::direkt`.
//!
//! **Wer schreiben darf (Tiffy, 06.10.2026).** Bekannte Personen immer:
//! eigene Geraete, Mitglieder eines gemeinsamen lokalen Projekts und wen man
//! vor Ort per 6 Ziffern bestaetigt hat. Alle anderen nur, wenn „Anfragen
//! erlauben" an ist (Vorgabe: aus) -- dann landen sie als Anfrage im eigenen
//! Ordner, hoechstens drei Nachrichten zu je 500 Zeichen. Ein „Annehmen" ohne
//! Treffen gibt es nicht (Tiffy: entweder Annehmen oder 6 Ziffern, und dann
//! die 6 Ziffern). Wer antworten oder selbst einen Unbekannten anschreiben
//! will, bestaetigt erst vor Ort.
//!
//! **Bestaetigt heisst: an diese Geraete gebunden** (`angenommen.json`). Die
//! Personen-Id eines Unbekannten ist ungeprueft -- jeder koennte sie nennen.
//! Nur von den bestaetigten Geraeten wird unter dieser Id angenommen. Sonst
//! schliche sich ein fremdes Geraet in eine Unterhaltung, und Antworten
//! gingen auch dorthin.
//!
//! **Postausgang:** Ist das andere Geraet nicht da, wartet die Nachricht und
//! geht beim naechsten Treffen hinaus -- `nah_lage` fragt alle drei Sekunden
//! und stoesst [`postausgang_leeren`] an.
//!
//! Was hier gemerkt wird, liegt neben `personen.json` im Ordner `nah/`:
//! `absender.json` (ueber welches Geraet eine Person erreichbar ist),
//! `blockiert.json`, `postausgang.json`, `angenommen.json` (Person ->
//! Geraete), `anfragen.json` (Person -> wie viele Nachrichten bisher),
//! `abgelehnt.json` (eigene Nachricht -> Grund) und `einstellungen.json`.

use crate::Zustand;
use openany_nahbereich::direkt::{DirektNachricht, Wer};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// Eine Person, die einmal geschrieben hat oder angeschrieben wurde.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Absender {
    pub name: String,
    pub geraete: BTreeSet<String>,
}

/// Eine Nachricht, die auf ihr Geraet wartet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wartend {
    pub nachricht: DirektNachricht,
    /// Die Geraete der Person, an die sie geht.
    pub ziele: Vec<String>,
}

/// Was man hier einstellt.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DirektEinstellungen {
    /// Duerfen Unbekannte Anfragen schicken? Vorgabe: nein.
    #[serde(default)]
    pub anfragen_erlauben: bool,
}

#[derive(Default)]
pub struct DirektStand {
    ordner: PathBuf,
    pub(crate) absender: Mutex<BTreeMap<String, Absender>>,
    pub(crate) blockiert: Mutex<BTreeSet<String>>,
    pub(crate) postausgang: Mutex<Vec<Wartend>>,
    /// Vor Ort bestaetigte Personen und die Geraete, an die sie gebunden sind.
    pub(crate) angenommen: Mutex<BTreeMap<String, BTreeSet<String>>>,
    /// Offene Anfragen: wie viele Nachrichten bisher.
    pub(crate) anfragen: Mutex<BTreeMap<String, u32>>,
    /// Eigene Nachrichten, die das andere Geraet abgelehnt hat: Id -> Grund.
    pub(crate) abgelehnt: Mutex<BTreeMap<String, String>>,
    pub(crate) einstellungen: Mutex<DirektEinstellungen>,
    /// Zaehler fuer die Oberflaeche: neue Direktnachrichten.
    pub(crate) zaehler: AtomicU64,
    /// Laeuft gerade ein Gang durch den Postausgang?
    leert: AtomicBool,
}

fn lesen<T: for<'a> Deserialize<'a> + Default>(pfad: &Path) -> T {
    std::fs::read_to_string(pfad)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

fn schreiben<T: Serialize>(pfad: &Path, wert: &T) {
    let _ = std::fs::write(pfad, serde_json::to_string_pretty(wert).unwrap_or_default());
}

impl DirektStand {
    pub fn laden(ordner: &Path) -> Self {
        Self {
            absender: Mutex::new(lesen(&ordner.join("absender.json"))),
            blockiert: Mutex::new(lesen(&ordner.join("blockiert.json"))),
            postausgang: Mutex::new(lesen(&ordner.join("postausgang.json"))),
            angenommen: Mutex::new(lesen(&ordner.join("angenommen.json"))),
            anfragen: Mutex::new(lesen(&ordner.join("anfragen.json"))),
            abgelehnt: Mutex::new(lesen(&ordner.join("abgelehnt.json"))),
            einstellungen: Mutex::new(lesen(&ordner.join("einstellungen.json"))),
            ordner: ordner.to_path_buf(),
            ..Default::default()
        }
    }

    pub fn absender_merken(&self, personen_id: &str, name: &str, geraet: &str) {
        if let Ok(mut a) = self.absender.lock() {
            let eintrag = a.entry(personen_id.to_string()).or_default();
            if !name.trim().is_empty() {
                eintrag.name = name.to_string();
            }
            eintrag.geraete.insert(geraet.to_string());
            schreiben(&self.ordner.join("absender.json"), &*a);
        }
    }

    pub fn ist_blockiert(&self, personen_id: &str, geraet: &str) -> bool {
        self.blockiert
            .lock()
            .map(|b| b.contains(personen_id) || b.contains(geraet))
            .unwrap_or(false)
    }

    pub fn anfragen_erlaubt(&self) -> bool {
        self.einstellungen
            .lock()
            .map(|e| e.anfragen_erlauben)
            .unwrap_or(false)
    }

    pub fn anfragen_erlauben(&self, an: bool) {
        if let Ok(mut e) = self.einstellungen.lock() {
            e.anfragen_erlauben = an;
            schreiben(&self.ordner.join("einstellungen.json"), &*e);
        }
    }

    /// Die Geraete, an die eine angenommene Person gebunden ist.
    pub fn angenommen_von(&self, personen_id: &str) -> Option<BTreeSet<String>> {
        self.angenommen
            .lock()
            .ok()
            .and_then(|a| a.get(personen_id).cloned())
    }

    /// Eine Person als bestaetigt merken (nach den 6 Ziffern) -- gebunden an
    /// die Geraete, die dabei bestaetigt wurden. Eine offene Anfrage ist damit
    /// erledigt.
    pub fn annehmen(&self, personen_id: &str, geraete: BTreeSet<String>) {
        if geraete.is_empty() {
            return;
        }
        if let Ok(mut a) = self.angenommen.lock() {
            a.entry(personen_id.to_string())
                .or_default()
                .extend(geraete);
            schreiben(&self.ordner.join("angenommen.json"), &*a);
        }
        self.anfrage_erledigt(personen_id);
    }

    fn anfrage_erledigt(&self, personen_id: &str) {
        if let Ok(mut a) = self.anfragen.lock() {
            if a.remove(personen_id).is_some() {
                schreiben(&self.ordner.join("anfragen.json"), &*a);
            }
        }
    }

    /// Wie viele Nachrichten eine offene Anfrage schon hat.
    fn anfrage_zahl(&self, personen_id: &str) -> u32 {
        self.anfragen
            .lock()
            .ok()
            .and_then(|a| a.get(personen_id).copied())
            .unwrap_or(0)
    }

    fn anfrage_zaehlen(&self, personen_id: &str) {
        if let Ok(mut a) = self.anfragen.lock() {
            *a.entry(personen_id.to_string()).or_default() += 1;
            schreiben(&self.ordner.join("anfragen.json"), &*a);
        }
    }

    /// Die offenen Anfragen: Personen-Id -> Zahl der Nachrichten.
    pub fn offene_anfragen(&self) -> BTreeMap<String, u32> {
        self.anfragen.lock().map(|a| a.clone()).unwrap_or_default()
    }

    /// Eine eigene Nachricht wurde abgelehnt: aus dem Postausgang, mit Grund.
    fn abgelehnt_merken(&self, id: &str, kennung: &str) {
        if let Ok(mut a) = self.abgelehnt.lock() {
            a.insert(id.to_string(), kennung.to_string());
            schreiben(&self.ordner.join("abgelehnt.json"), &*a);
        }
        self.zugestellt(id);
    }

    /// Warum eine eigene Nachricht abgelehnt wurde -- oder `None`.
    pub fn abgelehnt_weil(&self, id: &str) -> Option<String> {
        self.abgelehnt.lock().ok().and_then(|a| a.get(id).cloned())
    }

    fn blockieren(&self, personen_id: &str) {
        let geraete = self
            .absender
            .lock()
            .ok()
            .and_then(|a| a.get(personen_id).map(|x| x.geraete.clone()))
            .unwrap_or_default();
        if let Ok(mut b) = self.blockiert.lock() {
            b.insert(personen_id.to_string());
            b.extend(geraete);
            schreiben(&self.ordner.join("blockiert.json"), &*b);
        }
        // Was an diese Person noch wartet, geht nicht mehr hinaus.
        if let Ok(mut p) = self.postausgang.lock() {
            p.retain(|w| w.nachricht.an_person != personen_id);
            schreiben(&self.ordner.join("postausgang.json"), &*p);
        }
        // Und sie ist weder angenommen noch eine offene Anfrage.
        if let Ok(mut a) = self.angenommen.lock() {
            if a.remove(personen_id).is_some() {
                schreiben(&self.ordner.join("angenommen.json"), &*a);
            }
        }
        self.anfrage_erledigt(personen_id);
    }

    fn einreihen(&self, w: Wartend) {
        if let Ok(mut p) = self.postausgang.lock() {
            p.push(w);
            schreiben(&self.ordner.join("postausgang.json"), &*p);
        }
    }

    fn zugestellt(&self, id: &str) {
        if let Ok(mut p) = self.postausgang.lock() {
            p.retain(|w| w.nachricht.id != id);
            schreiben(&self.ordner.join("postausgang.json"), &*p);
        }
    }

    pub fn wartet(&self, id: &str) -> bool {
        self.postausgang
            .lock()
            .map(|p| p.iter().any(|w| w.nachricht.id == id))
            .unwrap_or(false)
    }
}

/// Darf eine Direktnachricht hier ankommen? (`Gastgeber::direkt_annehmen`)
/// Der Fehler ist eine Kennung aus `direkt::grund`.
pub fn annehmen(g: &crate::NahGastgeber, n: &DirektNachricht, anrufer: &str) -> Result<(), String> {
    use openany_nahbereich::direkt::{grund, ANFRAGEN_HOECHSTENS, ANFRAGE_LAENGE};
    use openany_nahbereich::Gastgeber;
    let nein = || Err(grund::NICHT_ANGENOMMEN.to_string());
    let Some(ich) = g.person() else {
        return nein();
    };
    if n.an_person != ich.personen_id && !ich.frueher.contains(&n.an_person) {
        return nein();
    }
    if g.direkt.ist_blockiert(&n.von_person, anrufer) {
        return nein();
    }
    // Wer sich als bekannte Person ausgibt, muss eines ihrer Geraete sein.
    if let Some(p) = g.bekannte_person(&n.von_person) {
        if !p.eintraege.iter().any(|e| e.fingerabdruck == anrufer) {
            return nein();
        }
        g.direkt
            .absender_merken(&n.von_person, &n.von_name, anrufer);
        return Ok(());
    }
    // Bestaetigt: nur von den Geraeten, an die die Bestaetigung gebunden ist.
    if let Some(geraete) = g.direkt.angenommen_von(&n.von_person) {
        if !geraete.contains(anrufer) {
            return nein();
        }
        g.direkt
            .absender_merken(&n.von_person, &n.von_name, anrufer);
        return Ok(());
    }
    // Eine Anfrage.
    if !g.direkt.anfragen_erlaubt() {
        return nein();
    }
    // Auch eine offene Anfrage bleibt bei ihrem ersten Geraet -- sonst
    // mischte sich ein fremdes unter derselben Kennung hinein.
    let bisher = g
        .direkt
        .absender
        .lock()
        .ok()
        .and_then(|a| a.get(&n.von_person).map(|x| x.geraete.clone()))
        .unwrap_or_default();
    if !bisher.is_empty() && !bisher.contains(anrufer) {
        return nein();
    }
    if n.text.chars().count() > ANFRAGE_LAENGE {
        return Err(grund::ZU_LANG.to_string());
    }
    if g.direkt.anfrage_zahl(&n.von_person) >= ANFRAGEN_HOECHSTENS {
        return Err(grund::ANFRAGEN_VOLL.to_string());
    }
    g.direkt.anfrage_zaehlen(&n.von_person);
    g.direkt
        .absender_merken(&n.von_person, &n.von_name, anrufer);
    Ok(())
}

/// Ist eine Nachricht dieser Person eine Anfrage -- weder bekannt noch
/// bestaetigt?
pub fn ist_anfrage(g: &crate::NahGastgeber, personen_id: &str) -> bool {
    !ist_bekannt(g, personen_id)
}

/// Ist diese Person bekannt -- ein Mitglied eines gemeinsamen lokalen
/// Projekts, die eigene oder vor Ort bestaetigt? Sonst heisst sie in der
/// Ansicht „unbekannt".
pub fn ist_bekannt(g: &crate::NahGastgeber, personen_id: &str) -> bool {
    g.bekannte_person(personen_id).is_some() || g.direkt.angenommen_von(personen_id).is_some()
}

/// Wie eine Person in der Auswahl heisst: ihr Name -- oder, hat sie keinen
/// gesetzt, der Name ihres zuletzt eingetragenen Geraets.
pub fn anzeigename(p: &openany_nahbereich::Person) -> String {
    if !p.name.trim().is_empty() {
        return p.name.clone();
    }
    p.eintraege
        .iter()
        .max_by(|a, b| a.at.cmp(&b.at))
        .map(|e| e.name.clone())
        .unwrap_or_default()
}

/// Wen man anschreiben kann.
#[derive(Serialize)]
pub struct Ziel {
    /// `person:<id>` oder `geraet:<fingerabdruck>`.
    ziel: String,
    name: String,
    /// Gerade in der Naehe?
    da: bool,
    unbekannt: bool,
}

fn eigene_geraete(g: &crate::NahGastgeber) -> BTreeSet<String> {
    use openany_nahbereich::Gastgeber;
    let mut eigene: BTreeSet<String> = g.gepaarte().into_iter().map(|(fp, _)| fp).collect();
    if let Some(p) = g.person() {
        eigene.extend(p.eintraege.iter().map(|e| e.fingerabdruck.clone()));
    }
    eigene
}

/// Die Geraete einer Person, so weit dieses Geraet sie kennt.
fn geraete_von(g: &crate::NahGastgeber, personen_id: &str) -> BTreeSet<String> {
    let mut fps: BTreeSet<String> = g
        .bekannte_person(personen_id)
        .map(|p| {
            p.eintraege
                .iter()
                .map(|e| e.fingerabdruck.clone())
                .collect()
        })
        .unwrap_or_default();
    if let Some(a) = g
        .direkt
        .absender
        .lock()
        .ok()
        .and_then(|a| a.get(personen_id).cloned())
    {
        fps.extend(a.geraete);
    }
    fps
}

async fn in_der_naehe(zustand: &Zustand) -> Vec<openany_nahbereich::GeraetInDerNaehe> {
    zustand
        .nah
        .lock()
        .await
        .as_ref()
        .map(|n| n.geraete())
        .unwrap_or_default()
        .into_iter()
        .filter(|g| g.openany)
        .collect()
}

/// Die Namen aus den Mitgliederlisten der lokalen Projekte: Personen-Id ->
/// Name. Hat eine Person selbst keinen Namen gesetzt, steht hier der, unter
/// dem sie eingeladen wurde -- besser als der Name ihres Geraets.
async fn mitgliedsnamen(zustand: &Zustand) -> BTreeMap<String, String> {
    let mut namen = BTreeMap::new();
    let speicher = zustand.speicher.lock().await;
    for p in speicher.projekte().unwrap_or_default() {
        let Some(liste) = p
            .mitgliederliste
            .as_deref()
            .and_then(|j| serde_json::from_str::<openany_nahbereich::Mitgliederliste>(j).ok())
        else {
            continue;
        };
        for e in liste.eintraege {
            if !e.name.trim().is_empty() {
                namen.insert(e.personen_id, e.name);
            }
        }
    }
    namen
}

/// Wie eine Person heisst: ihr eigener Name, sonst der aus einem
/// gemeinsamen Projekt, sonst der ihres Geraets.
fn name_fuer(
    g: &crate::NahGastgeber,
    mitglieder: &BTreeMap<String, String>,
    id: &str,
) -> Option<String> {
    let person = g.bekannte_person(id);
    if let Some(n) = person
        .as_ref()
        .map(|p| p.name.clone())
        .filter(|n| !n.trim().is_empty())
    {
        return Some(n);
    }
    mitglieder.get(id).cloned().or_else(|| {
        person
            .map(|p| anzeigename(&p))
            .filter(|n| !n.trim().is_empty())
    })
}

#[tauri::command]
pub async fn nah_nachricht_ziele(
    zustand: tauri::State<'_, Arc<Zustand>>,
) -> Result<Vec<Ziel>, String> {
    let g = &zustand.gastgeber;
    let eigene = eigene_geraete(g);
    let naehe: Vec<_> = in_der_naehe(&zustand)
        .await
        .into_iter()
        .filter(|d| !eigene.contains(&d.fingerabdruck))
        .collect();

    // Personen: bekannte und solche, mit denen schon geschrieben wurde.
    let mut personen: BTreeMap<String, String> = g.fremde_personen();
    if let Ok(a) = g.direkt.absender.lock() {
        for (id, x) in a.iter() {
            personen.entry(id.clone()).or_insert_with(|| x.name.clone());
        }
    }
    let blockiert = g
        .direkt
        .blockiert
        .lock()
        .map(|b| b.clone())
        .unwrap_or_default();

    let mitglieder = mitgliedsnamen(&zustand).await;
    let mut ziele = Vec::new();
    let mut vergeben: BTreeSet<String> = BTreeSet::new();
    for (id, name) in personen {
        let name = name_fuer(g, &mitglieder, &id).unwrap_or(name);
        if blockiert.contains(&id) {
            continue;
        }
        let fps = geraete_von(g, &id);
        let da = naehe.iter().any(|d| fps.contains(&d.fingerabdruck));
        vergeben.extend(fps);
        ziele.push(Ziel {
            unbekannt: !ist_bekannt(g, &id),
            ziel: format!("person:{id}"),
            name: if name.trim().is_empty() {
                "unnamed".into()
            } else {
                name
            },
            da,
        });
    }
    // Geraete in der Naehe, deren Person hier noch niemand kennt.
    for d in naehe {
        if vergeben.contains(&d.fingerabdruck) || blockiert.contains(&d.fingerabdruck) {
            continue;
        }
        ziele.push(Ziel {
            ziel: format!("geraet:{}", d.fingerabdruck),
            name: d.name,
            da: true,
            unbekannt: true,
        });
    }
    ziele.sort_by(|a, b| {
        b.da.cmp(&a.da)
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    Ok(ziele)
}

/// Senden: ablegen, und sofort zustellen, wenn ein Geraet der Person in der
/// Naehe ist -- sonst in den Postausgang. Gibt die abgelegte Nachricht
/// zurueck und ob sie schon drueben ist.
pub async fn senden(
    zustand: &Zustand,
    ziel: &str,
    text: &str,
) -> Result<(DirektNachricht, String), String> {
    use openany_nahbereich::Gastgeber;
    let g = &zustand.gastgeber;
    let ident = crate::geraet_identitaet(zustand)?;
    let ich = g.person().ok_or("This device has no person yet.")?;
    let naehe = in_der_naehe(zustand).await;

    let (an, name, ziele) = if let Some(fp) = ziel.strip_prefix("geraet:") {
        let d = naehe
            .iter()
            .find(|d| d.fingerabdruck == fp)
            .ok_or("This device is no longer nearby.")?;
        let wer =
            openany_nahbereich::anruf::wer(&ident, &d.adresse, openany_nahbereich::DIENST_PORT, fp)
                .await?;
        g.direkt.absender_merken(&wer.personen_id, &wer.name, fp);
        (wer.personen_id, wer.name, vec![fp.to_string()])
    } else if let Some(id) = ziel.strip_prefix("person:") {
        let mitglieder = mitgliedsnamen(zustand).await;
        let name = name_fuer(g, &mitglieder, id)
            .or_else(|| {
                g.direkt
                    .absender
                    .lock()
                    .ok()
                    .and_then(|a| a.get(id).map(|x| x.name.clone()))
            })
            .unwrap_or_default();
        let fps: Vec<String> = geraete_von(g, id).into_iter().collect();
        if fps.is_empty() {
            return Err("No device is known for this person.".into());
        }
        (id.to_string(), name, fps)
    } else {
        return Err("Please choose someone nearby.".into());
    };
    if g.direkt.ist_blockiert(&an, "") {
        return Err("This person is blocked.".into());
    }
    // Nur an Bekannte. Einem Unbekannten zu schreiben hiesse, dass seine
    // Antwort hier als Anfrage gilt oder gar nicht ankommt -- also erst vor
    // Ort per 6 Ziffern bestaetigen.
    if !ist_bekannt(g, &an) {
        return Err("Confirm this person nearby with the 6 digits first.".into());
    }

    let von = Wer {
        personen_id: ich.personen_id.clone(),
        name: ich.name.clone(),
    };
    let n = DirektNachricht::schreiben(&ident, &von, &an, text).map_err(|e| e.to_string())?;
    {
        let speicher = zustand.speicher.lock().await;
        openany_nahbereich::direkt::ablegen_mit(&speicher, &n, true, &name)?;
    }

    let mut erledigt = false;
    for d in naehe.iter().filter(|d| ziele.contains(&d.fingerabdruck)) {
        use openany_nahbereich::anruf::Zustellung;
        match openany_nahbereich::anruf::nachricht_zustellen(
            &ident,
            &d.adresse,
            openany_nahbereich::DIENST_PORT,
            &d.fingerabdruck,
            &n,
        )
        .await
        {
            Zustellung::Zugestellt => {
                erledigt = true;
                break;
            }
            // Abgelehnt: nicht in den Postausgang -- noch einmal hilft nicht.
            Zustellung::Abgelehnt(kennung) => {
                g.direkt.abgelehnt_merken(&n.id, &kennung);
                erledigt = true;
                break;
            }
            Zustellung::NichtErreichbar(_) => {}
        }
    }
    if !erledigt {
        g.direkt.einreihen(Wartend {
            nachricht: n.clone(),
            ziele,
        });
    }
    Ok((n, name))
}

/// Was im Postausgang wartet und dessen Geraet jetzt da ist, hinausschicken
/// -- im Hintergrund, damit `nah_lage` nicht wartet.
pub fn postausgang_leeren(zustand: Arc<Zustand>, naehe: Vec<openany_nahbereich::GeraetInDerNaehe>) {
    let stand = &zustand.gastgeber.direkt;
    let faellig: Vec<Wartend> = stand
        .postausgang
        .lock()
        .map(|p| {
            p.iter()
                .filter(|w| naehe.iter().any(|d| w.ziele.contains(&d.fingerabdruck)))
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    if faellig.is_empty() || stand.leert.swap(true, Ordering::SeqCst) {
        return;
    }
    tauri::async_runtime::spawn(async move {
        if let Ok(ident) = crate::geraet_identitaet(&zustand) {
            for w in faellig {
                for d in naehe.iter().filter(|d| w.ziele.contains(&d.fingerabdruck)) {
                    use openany_nahbereich::anruf::Zustellung;
                    let stand = &zustand.gastgeber.direkt;
                    match openany_nahbereich::anruf::nachricht_zustellen(
                        &ident,
                        &d.adresse,
                        openany_nahbereich::DIENST_PORT,
                        &d.fingerabdruck,
                        &w.nachricht,
                    )
                    .await
                    {
                        Zustellung::Zugestellt => stand.zugestellt(&w.nachricht.id),
                        Zustellung::Abgelehnt(kennung) => {
                            stand.abgelehnt_merken(&w.nachricht.id, &kennung)
                        }
                        Zustellung::NichtErreichbar(_) => continue,
                    }
                    stand.zaehler.fetch_add(1, Ordering::Relaxed);
                    break;
                }
            }
        }
        zustand
            .gastgeber
            .direkt
            .leert
            .store(false, Ordering::SeqCst);
    });
}

/// Anfragen: ob sie erlaubt sind, und welche offen sind.
#[derive(Serialize)]
pub struct AnfragenLage {
    erlaubt: bool,
    offen: Vec<OffeneAnfrage>,
}

#[derive(Serialize)]
pub struct OffeneAnfrage {
    /// `person:<id>` -- wie in `nah_nachricht_ziele`.
    person: String,
    name: String,
    /// Ist ein Geraet der Person gerade in der Naehe? Nur dann laesst sie sich
    /// per 6 Ziffern bestaetigen.
    da: bool,
    /// Die Nachrichten der Anfrage, aelteste zuerst (hoechstens drei).
    texte: Vec<AnfrageText>,
}

#[derive(Serialize)]
pub struct AnfrageText {
    text: String,
    zeit: String,
}

#[tauri::command]
pub async fn nah_anfragen(zustand: tauri::State<'_, Arc<Zustand>>) -> Result<AnfragenLage, String> {
    let g = &zustand.gastgeber;
    let namen = g
        .direkt
        .absender
        .lock()
        .map(|a| a.clone())
        .unwrap_or_default();
    let naehe: BTreeSet<String> = in_der_naehe(&zustand)
        .await
        .into_iter()
        .map(|d| d.fingerabdruck)
        .collect();
    // Die juengsten Nachrichten vor Ort; eine Anfrage hat hoechstens drei,
    // und sie sind neu -- die erste Seite reicht.
    let juengste = {
        let speicher = zustand.speicher.lock().await;
        speicher
            .nachrichten_gefiltert(
                1,
                &openany_store::Verlaufsfilter {
                    nur_konto: Some(openany_nahbereich::direkt::KONTO.into()),
                    ..Default::default()
                },
            )
            .map(|(l, _)| l)
            .unwrap_or_default()
    };
    let offen = g
        .direkt
        .offene_anfragen()
        .into_keys()
        .filter(|id| ist_anfrage(g, id) && !g.direkt.ist_blockiert(id, ""))
        .map(|id| {
            let mut texte: Vec<AnfrageText> = juengste
                .iter()
                .filter(|n| !n.von_mir && n.raum == id)
                .map(|n| AnfrageText {
                    text: n.text.clone(),
                    zeit: n.zeit.clone(),
                })
                .collect();
            texte.sort_by(|a, b| a.zeit.cmp(&b.zeit));
            let a = namen.get(&id);
            OffeneAnfrage {
                name: a.map(|a| a.name.clone()).unwrap_or_default(),
                da: a.is_some_and(|a| a.geraete.iter().any(|fp| naehe.contains(fp))),
                person: format!("person:{id}"),
                texte,
            }
        })
        .collect();
    Ok(AnfragenLage {
        erlaubt: g.direkt.anfragen_erlaubt(),
        offen,
    })
}

/// „Anfragen von Geraeten in der Naehe erlauben" -- an oder aus. Offene
/// Anfragen bleiben stehen, wenn man ausschaltet.
#[tauri::command]
pub async fn nah_anfragen_erlauben(
    zustand: tauri::State<'_, Arc<Zustand>>,
    an: bool,
) -> Result<(), String> {
    zustand.gastgeber.direkt.anfragen_erlauben(an);
    Ok(())
}

/// Einen vor Ort bestaetigten Kontakt merken: die Person, gebunden an ihre
/// gueltigen Geraete -- ausgehend von dem, das gerade bestaetigt hat.
pub fn kontakt_merken(g: &crate::NahGastgeber, person: &openany_nahbereich::Person, geraet: &str) {
    let geraete: BTreeSet<String> = person.gueltige(&[geraet]).into_keys().collect();
    let name = anzeigename(person);
    for fp in &geraete {
        g.direkt.absender_merken(&person.personen_id, &name, fp);
    }
    g.direkt.annehmen(&person.personen_id, geraete);
    g.direkt.zaehler.fetch_add(1, Ordering::Relaxed);
}

/// Einen Kontakt vor Ort per 6 Ziffern bestaetigen -- mit einem Geraet in
/// der Naehe (`geraet:<fp>`) oder einer Person, von der eines in der Naehe
/// ist (`person:<id>`). Gibt den Code zurueck; drueben zeigt das Geraet
/// dieselben Ziffern (einladen.rs, `KONTAKT`).
#[tauri::command]
pub async fn nah_kontakt_bestaetigen(
    zustand: tauri::State<'_, Arc<Zustand>>,
    ziel: String,
) -> Result<String, String> {
    use openany_nahbereich::Gastgeber;
    let g = &zustand.gastgeber;
    let ich = g.person().ok_or("This device has no person yet.")?;
    let naehe = in_der_naehe(&zustand).await;
    let fp = if let Some(fp) = ziel.strip_prefix("geraet:") {
        fp.to_string()
    } else if let Some(id) = ziel.strip_prefix("person:") {
        let fps = geraete_von(g, id);
        naehe
            .iter()
            .find(|d| fps.contains(&d.fingerabdruck))
            .map(|d| d.fingerabdruck.clone())
            .ok_or("This device is not nearby right now.")?
    } else {
        return Err("Please choose someone nearby.".into());
    };
    if !naehe.iter().any(|d| d.fingerabdruck == fp) {
        return Err("This device is not nearby right now.".into());
    }
    let ident = crate::ident_laden(&zustand).await?;
    let adresse = crate::adresse_von(&zustand, &fp).await?;
    let anfrage = openany_nahbereich::einladen::Anfrage {
        projekt: openany_nahbereich::einladen::KONTAKT.into(),
        projekt_name: String::new(),
        von: if ich.name.trim().is_empty() {
            g.mein_name()
        } else {
            ich.name
        },
        geraet: g.mein_name(),
        zufall: uuid::Uuid::new_v4().simple().to_string(),
    };
    openany_nahbereich::anruf::einladen(
        &ident,
        &g.einladungen,
        &adresse,
        openany_nahbereich::DIENST_PORT,
        &fp,
        &anfrage,
    )
    .await
}

/// Die anfragende Seite einer Kontakt-Bestaetigung: Hat drueben jemand
/// „Passt" gesagt? Dann die Person drueben merken und die eigene hinueber.
/// `true`: fertig.
pub async fn kontakt_abschliessen(
    zustand: &Zustand,
    fp: &str,
    offen: &openany_nahbereich::einladen::OffeneEinladung,
) -> Result<bool, String> {
    use openany_nahbereich::Gastgeber;
    let g = &zustand.gastgeber;
    let ident = crate::ident_laden(zustand).await?;
    let adresse = crate::adresse_von(zustand, fp).await?;
    if !offen.dort_bestaetigt
        && !openany_nahbereich::anruf::einladung_nachsehen(
            &ident,
            &g.einladungen,
            &adresse,
            openany_nahbereich::DIENST_PORT,
            fp,
        )
        .await?
    {
        return Ok(false);
    }
    let Some(person_b) = g
        .einladungen
        .lock()
        .ok()
        .and_then(|mut e| e.get(fp)?.person.clone())
    else {
        return Ok(false);
    };
    let ich = g.person().ok_or("This device has no person yet.")?;
    openany_nahbereich::anruf::kontakt_aufnahme_senden(
        &ident,
        &adresse,
        openany_nahbereich::DIENST_PORT,
        fp,
        &ich,
    )
    .await?;
    kontakt_merken(g, &person_b, fp);
    if let Ok(mut e) = g.einladungen.lock() {
        e.entfernen(fp);
    }
    Ok(true)
}

/// Eine Person blockieren: Von ihr kommt hier nichts mehr an, und an sie
/// geht nichts mehr hinaus.
#[tauri::command]
pub async fn nah_blockieren(
    zustand: tauri::State<'_, Arc<Zustand>>,
    person: String,
) -> Result<(), String> {
    let person = person
        .strip_prefix("person:")
        .unwrap_or(&person)
        .to_string();
    if person.is_empty() {
        return Err("Block whom?".into());
    }
    zustand.gastgeber.direkt.blockieren(&person);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use openany_nahbereich::{Gastgeber, Identitaet, Person};

    fn gastgeber() -> (crate::NahGastgeber, Identitaet, tempfile::TempDir) {
        let ordner = tempfile::tempdir().unwrap();
        let ident = Identitaet::erzeugen().unwrap();
        let g = crate::NahGastgeber::laden(
            &ordner.path().join("gepaart.json"),
            "Tablet".into(),
            Arc::new(tokio::sync::Mutex::new(
                openany_store::Speicher::im_arbeitsspeicher().unwrap(),
            )),
            Arc::new(openany_store::Inhalte::oeffnen(ordner.path().join("inhalte")).unwrap()),
        );
        g.person_merken(Person::neu(&ident, "Tablet").unwrap());
        (g, ident, ordner)
    }

    #[test]
    fn wer_sich_als_bekannte_person_ausgibt_muss_ihr_geraet_sein() {
        let (g, _ident, _o) = gastgeber();
        let ich = g.person().unwrap();
        let anna_tel = Identitaet::erzeugen().unwrap();
        let anna = Person::neu(&anna_tel, "Anna").unwrap();
        g.person_fremd_merken(anna.clone());
        let fremd = Identitaet::erzeugen().unwrap();

        let als_anna = |ident: &Identitaet| {
            DirektNachricht::schreiben(
                ident,
                &Wer {
                    personen_id: anna.personen_id.clone(),
                    name: "Anna".into(),
                },
                &ich.personen_id,
                "Hallo",
            )
            .unwrap()
        };
        assert!(annehmen(&g, &als_anna(&anna_tel), &anna_tel.fingerabdruck).is_ok());
        // Ein fremdes Geraet unter Annas Kennung: nein.
        assert!(annehmen(&g, &als_anna(&fremd), &fremd.fingerabdruck).is_err());

        // Eine unbekannte Kennung: nur mit erlaubten Anfragen.
        g.direkt.anfragen_erlauben(true);
        let unbekannt = DirektNachricht::schreiben(
            &fremd,
            &Wer {
                personen_id: "p-x".into(),
                name: "Mama".into(),
            },
            &ich.personen_id,
            "Hallo",
        )
        .unwrap();
        assert!(annehmen(&g, &unbekannt, &fremd.fingerabdruck).is_ok());
        assert!(!ist_bekannt(&g, "p-x"));
        assert!(ist_bekannt(&g, &anna.personen_id));

        // Blockiert: nichts mehr -- auch nicht unter neuer Kennung vom selben Geraet.
        g.direkt.blockieren("p-x");
        assert!(annehmen(&g, &unbekannt, &fremd.fingerabdruck).is_err());
        let neu = DirektNachricht::schreiben(
            &fremd,
            &Wer {
                personen_id: "p-y".into(),
                name: "Papa".into(),
            },
            &ich.personen_id,
            "Hallo",
        )
        .unwrap();
        assert!(annehmen(&g, &neu, &fremd.fingerabdruck).is_err());
    }

    fn von(ident: &Identitaet, id: &str, an: &str, text: &str) -> DirektNachricht {
        DirektNachricht::schreiben(
            ident,
            &Wer {
                personen_id: id.into(),
                name: "Fremd".into(),
            },
            an,
            text,
        )
        .unwrap()
    }

    #[test]
    fn anfragen_sind_aus_bis_man_sie_erlaubt() {
        use openany_nahbereich::direkt::grund;
        let (g, _ident, _o) = gastgeber();
        let ich = g.person().unwrap().personen_id;
        let fremd = Identitaet::erzeugen().unwrap();
        let n = von(&fremd, "p-x", &ich, "Hallo");
        assert_eq!(
            annehmen(&g, &n, &fremd.fingerabdruck),
            Err(grund::NICHT_ANGENOMMEN.to_string())
        );
        g.direkt.anfragen_erlauben(true);
        assert!(annehmen(&g, &n, &fremd.fingerabdruck).is_ok());
        assert!(ist_anfrage(&g, "p-x"));
    }

    #[test]
    fn eine_anfrage_hat_grenzen_und_bleibt_bei_ihrem_geraet() {
        use openany_nahbereich::direkt::grund;
        let (g, _ident, _o) = gastgeber();
        g.direkt.anfragen_erlauben(true);
        let ich = g.person().unwrap().personen_id;
        let fremd = Identitaet::erzeugen().unwrap();

        // Zu lang -- und das zaehlt nicht mit.
        let lang = von(&fremd, "p-x", &ich, &"a".repeat(501));
        assert_eq!(
            annehmen(&g, &lang, &fremd.fingerabdruck),
            Err(grund::ZU_LANG.into())
        );
        // Genau 500 Zeichen (auch mit Umlauten) gehen.
        assert!(annehmen(
            &g,
            &von(&fremd, "p-x", &ich, &"ä".repeat(500)),
            &fremd.fingerabdruck
        )
        .is_ok());
        assert!(annehmen(&g, &von(&fremd, "p-x", &ich, "zwei"), &fremd.fingerabdruck).is_ok());
        assert!(annehmen(&g, &von(&fremd, "p-x", &ich, "drei"), &fremd.fingerabdruck).is_ok());
        assert_eq!(
            annehmen(&g, &von(&fremd, "p-x", &ich, "vier"), &fremd.fingerabdruck),
            Err(grund::ANFRAGEN_VOLL.into())
        );

        // Ein zweites Geraet unter derselben Kennung mischt sich nicht ein.
        let zweit = Identitaet::erzeugen().unwrap();
        assert_eq!(
            annehmen(
                &g,
                &von(&zweit, "p-x", &ich, "ich auch"),
                &zweit.fingerabdruck
            ),
            Err(grund::NICHT_ANGENOMMEN.into())
        );
    }

    #[test]
    fn bestaetigt_heisst_bekannt_und_an_die_geraete_gebunden() {
        use openany_nahbereich::direkt::grund;
        let (g, _ident, _o) = gastgeber();
        let ich = g.person().unwrap().personen_id;
        let carla_tel = Identitaet::erzeugen().unwrap();
        let carla = Person::neu(&carla_tel, "Carla").unwrap();

        // Ohne erlaubte Anfragen kommt erst nach der Bestaetigung etwas an.
        let n = von(&carla_tel, &carla.personen_id, &ich, "Hallo");
        assert!(annehmen(&g, &n, &carla_tel.fingerabdruck).is_err());
        kontakt_merken(&g, &carla, &carla_tel.fingerabdruck);
        assert!(ist_bekannt(&g, &carla.personen_id));
        assert!(!ist_anfrage(&g, &carla.personen_id));
        // Keine Grenzen mehr: auch lang und viele.
        for _ in 0..5 {
            assert!(annehmen(
                &g,
                &von(&carla_tel, &carla.personen_id, &ich, &"x".repeat(900)),
                &carla_tel.fingerabdruck
            )
            .is_ok());
        }
        // Aber nur von ihren Geraeten.
        let fremd = Identitaet::erzeugen().unwrap();
        assert_eq!(
            annehmen(
                &g,
                &von(&fremd, &carla.personen_id, &ich, "ich bin Carla"),
                &fremd.fingerabdruck
            ),
            Err(grund::NICHT_ANGENOMMEN.into())
        );
        // Blockieren nimmt die Bestaetigung zurueck.
        g.direkt.blockieren(&carla.personen_id);
        assert!(!ist_bekannt(&g, &carla.personen_id));
    }
}
