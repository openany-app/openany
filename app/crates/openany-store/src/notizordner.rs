//! Notizen als Dateien in einem Ordner -- fuer Zettlr, Obsidian & Co.
//! (docs/plan-notizen-e2e.md, 6d). Nur in der Desktop-App.
//!
//! **Je Mappe verbunden.** Eine eigene Mappe wird mit einem Ordner auf diesem
//! Computer verbunden; der Ordner IST die Mappe, ihre Untermappen sind
//! Unterordner, jede Notiz ist `{zk_id} {Titel}.md` -- die Form, unter der
//! Zettlr `[[zk_id Titel]]` aufloest und die frueher der Server schrieb.
//!
//! **Ein Spiegel, kein Speicherort.** Die Notiz in der Datenbank bleibt die
//! Quelle fuer den Abgleich mit openany.de. Je Datei merkt sich der Spiegel,
//! wie sie beim letzten Mal aussah (Abdruck des Texts, Pfad) und wie die
//! Notiz dazu stand. Daraus folgt bei jedem Durchlauf, wer sich bewegt hat:
//!
//! | Datei        | Notiz        | was geschieht                              |
//! |--------------|--------------|--------------------------------------------|
//! | gleich       | gleich       | nichts                                     |
//! | geaendert    | gleich       | Text/Titel/Mappe aus der Datei uebernehmen |
//! | gleich       | geaendert    | Datei neu schreiben (und umbenennen)       |
//! | geaendert    | geaendert    | Datei als Konfliktkopie, Notiz gewinnt      |
//! | geloescht    | gleich       | Notiz in den Papierkorb                    |
//! | neu, ohne Id | --           | neue Notiz, Datei bekommt die Id           |
//!
//! **Ein fehlender Ordner ist kein leerer.** Ist er nicht da (ausgehaengt,
//! umbenannt), wird nichts gespiegelt -- sonst saehe er aus wie lauter
//! geloeschte Dateien, und alle Notizen der Mappe landeten im Papierkorb.
//!
//! Gespiegelt werden nur eigene Notizen ausserhalb des Papierkorbs. Ihre
//! Bilder liegen daneben ([`crate::Speicher::notizordner_bilder`]).

use crate::{Ergebnis, Notiz, Protokoll, Speicher, SpeicherFehler};
use openany_client::Art;
use rusqlite::{params, OptionalExtension};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Eine Mappe, verbunden mit einem Ordner auf diesem Computer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verbindung {
    pub mappe: String,
    pub ordner: PathBuf,
}

/// Was ein Durchlauf getan hat.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Spiegelbericht {
    /// Dateien neu geschrieben oder umbenannt.
    pub geschrieben: usize,
    /// Aenderungen aus Dateien in Notizen uebernommen.
    pub uebernommen: usize,
    /// Neue Notizen aus neuen Dateien.
    pub neu: usize,
    /// Konfliktkopien.
    pub konflikte: usize,
    /// Notizen in den Papierkorb, weil ihre Datei fort ist.
    pub papierkorb: usize,
    /// Dateien entfernt, weil die Notiz fort ist.
    pub entfernt: usize,
}

impl Spiegelbericht {
    pub fn etwas_getan(&self) -> bool {
        *self != Self::default()
    }
}

/// Wie eine Datei beim letzten Durchlauf aussah.
#[derive(Debug, Clone)]
struct Stand {
    relpfad: String,
    datei_abdruck: String,
    notiz_abdruck: String,
    groesse: i64,
    mtime: i64,
}

/// Eine Datei im Ordner, wie sie jetzt ist.
#[derive(Debug, Clone)]
struct Fund {
    relpfad: String,
    abdruck: String,
    groesse: i64,
    mtime: i64,
    titel: String,
}

/// Zeichen, die Dateisysteme nicht vertragen -- dieselbe Regel, mit der der
/// Server bis Oktober 2026 seine Notizdateien benannte (`NoteTitle::sanitize`).
pub fn sauberer_name(titel: &str) -> String {
    use std::sync::OnceLock;
    static R: OnceLock<[regex::Regex; 5]> = OnceLock::new();
    let r = R.get_or_init(|| {
        [
            regex::Regex::new(r"\s*:\s+").unwrap(),
            regex::Regex::new(r"[:/\\]").unwrap(),
            regex::Regex::new(r#"["*?<>|\x00-\x1F]"#).unwrap(),
            regex::Regex::new(r"-{2,}").unwrap(),
            regex::Regex::new(r"\s+").unwrap(),
        ]
    });
    let s = r[0].replace_all(titel, " – ");
    let s = r[1].replace_all(&s, "-");
    let s = r[2].replace_all(&s, "");
    let s = r[3].replace_all(&s, "-");
    let s = r[4].replace_all(&s, " ");
    let mut s = s.trim().trim_end_matches([' ', '.']).to_string();
    if s.chars().count() > 100 {
        s = s.chars().take(100).collect::<String>();
        s = s.trim_end_matches([' ', '.', '-', '–']).to_string();
    }
    s
}

/// Der Name eines Unterordners fuer eine Mappe. Was mit einem Punkt beginnt,
/// verstecken Dateimanager und Zettlr -- dann bekommt es ein Wort davor.
pub(crate) fn ordnername(mappe: &str) -> String {
    let n = sauberer_name(mappe);
    if n.is_empty() || n.starts_with('.') {
        format!("Mappe {n}").trim().to_string()
    } else {
        n
    }
}

/// `{zk_id} {Titel}.md`, ohne Titel nur die Id.
pub fn dateiname(zk_id: &str, titel: &str) -> String {
    let t = sauberer_name(titel);
    if t.is_empty() {
        format!("{zk_id}.md")
    } else {
        format!("{zk_id} {t}.md")
    }
}

/// Id und Titel aus einem Dateinamen; `None` ohne Id davor.
pub fn aus_dateiname(name: &str) -> Option<(String, String)> {
    let stamm = name.strip_suffix(".md")?;
    let (id, rest) = match stamm.split_once(' ') {
        Some((id, rest)) => (id, rest.trim()),
        None => (stamm, ""),
    };
    (id.len() == 14 && id.bytes().all(|b| b.is_ascii_digit()))
        .then(|| (id.to_string(), rest.to_string()))
}

pub(crate) fn abdruck(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// Was an einer Notiz in die Datei geht: Titel und Text. Wo sie liegt,
/// vergleicht der Spiegel ueber den Pfad -- so bleibt eine umbenannte
/// verbundene Mappe ohne Folgen fuer die Dateien.
fn notizabdruck(n: &Notiz) -> String {
    abdruck(format!("{}\n{}", n.titel, n.inhalt).as_bytes())
}

pub(crate) fn liegt_in(mappe: Option<&str>, wurzel: &str) -> bool {
    mappe.is_some_and(|m| m == wurzel || m.starts_with(&format!("{wurzel}/")))
}

/// Der Unterordner einer Mappe im Ordner der Verbindung `verbunden`: ihre
/// Stufen unterhalb der verbundenen, je bereinigt. Leer fuer die Mappe selbst.
pub(crate) fn unterordner(verbunden: &str, mappe: &str) -> String {
    if mappe.len() <= verbunden.len() {
        return String::new();
    }
    mappe[verbunden.len() + 1..]
        .split('/')
        .map(ordnername)
        .collect::<Vec<_>>()
        .join("/")
}

fn verbindet(a: &str, b: &str) -> bool {
    liegt_in(Some(a), b) || liegt_in(Some(b), a)
}

pub(crate) fn mtime(meta: &std::fs::Metadata) -> i64 {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_millis() as i64)
}

pub(crate) fn pfad_von(ordner: &Path, relpfad: &str) -> PathBuf {
    relpfad
        .split('/')
        .fold(ordner.to_path_buf(), |p, teil| p.join(teil))
}

/// Erst daneben schreiben, dann umbenennen -- Zettlr sieht nie eine halbe Datei.
fn schreiben(ziel: &Path, inhalt: &str) -> std::io::Result<()> {
    if let Some(e) = ziel.parent() {
        std::fs::create_dir_all(e)?;
    }
    let tmp = ziel.with_extension("md.openany-tmp");
    std::fs::write(&tmp, inhalt)?;
    std::fs::rename(&tmp, ziel)
}

/// Eine freie `zk_id` -- dieselbe Regel wie beim Anlegen in der App.
fn freie_zk_id(speicher: &Speicher, vergeben: &BTreeSet<String>) -> Ergebnis<String> {
    let mut moment = chrono::Local::now();
    loop {
        let v = moment.format("%Y%m%d%H%M%S").to_string();
        if !vergeben.contains(&v) && speicher.notiz(&v)?.is_none() {
            return Ok(v);
        }
        moment += chrono::Duration::seconds(1);
    }
}

impl Speicher {
    /// Alle Verbindungen dieses Computers.
    pub fn notizordner(&self) -> Ergebnis<Vec<Verbindung>> {
        let db = self.db();
        let mut a = db.prepare("SELECT mappe, ordner FROM notizordner ORDER BY mappe")?;
        let liste = a
            .query_map([], |z| {
                Ok(Verbindung {
                    mappe: z.get(0)?,
                    ordner: PathBuf::from(z.get::<_, String>(1)?),
                })
            })?
            .collect::<Result<_, _>>()?;
        Ok(liste)
    }

    /// Darf diese Mappe verbunden werden? Nein, wenn sie selbst, eine Mappe
    /// darueber oder darunter schon verbunden ist -- eine Notiz gehoert
    /// hoechstens zu einem Ordner.
    pub fn notizordner_moeglich(&self, mappe: &str) -> Ergebnis<bool> {
        Ok(!self
            .notizordner()?
            .iter()
            .any(|v| verbindet(&v.mappe, mappe)))
    }

    /// Verbinden. Der erste Durchlauf entscheidet dann, was schon im Ordner
    /// liegt: Dateien mit bekannter Id werden verglichen, andere werden neue
    /// Notizen in der Mappe.
    pub fn notizordner_verbinden(&self, mappe: &str, ordner: &Path) -> Ergebnis<bool> {
        if !self.notizordner_moeglich(mappe)? {
            return Ok(false);
        }
        self.db().execute(
            "INSERT INTO notizordner (mappe, ordner, verbunden_at) VALUES (?1, ?2, ?3)",
            params![
                mappe,
                ordner.to_string_lossy(),
                chrono::Utc::now().to_rfc3339()
            ],
        )?;
        self.db().execute(
            "DELETE FROM notizordner_dateien WHERE mappe = ?1",
            params![mappe],
        )?;
        self.db().execute(
            "DELETE FROM notizordner_bilder WHERE mappe = ?1",
            params![mappe],
        )?;
        Ok(true)
    }

    /// Trennen. Die Dateien bleiben liegen, die Notizen auch.
    pub fn notizordner_trennen(&self, mappe: &str) -> Ergebnis<()> {
        self.db()
            .execute("DELETE FROM notizordner WHERE mappe = ?1", params![mappe])?;
        self.db().execute(
            "DELETE FROM notizordner_dateien WHERE mappe = ?1",
            params![mappe],
        )?;
        self.db().execute(
            "DELETE FROM notizordner_bilder WHERE mappe = ?1",
            params![mappe],
        )?;
        Ok(())
    }

    /// Eine Mappe wurde umbenannt: Die Verbindung zieht mit.
    pub(crate) fn notizordner_umziehen(&self, alt: &str, neu: &str) -> Ergebnis<()> {
        for v in self.notizordner()? {
            if liegt_in(Some(&v.mappe), alt) {
                let umbenannt = format!("{neu}{}", &v.mappe[alt.len()..]);
                self.db().execute(
                    "UPDATE notizordner SET mappe = ?1 WHERE mappe = ?2",
                    params![umbenannt, v.mappe],
                )?;
                self.db().execute(
                    "UPDATE notizordner_dateien SET mappe = ?1 WHERE mappe = ?2",
                    params![umbenannt, v.mappe],
                )?;
                self.db().execute(
                    "UPDATE notizordner_bilder SET mappe = ?1 WHERE mappe = ?2",
                    params![umbenannt, v.mappe],
                )?;
            }
        }
        Ok(())
    }

    /// Eine Mappe wurde geloescht: Verbindungen darin enden.
    pub(crate) fn notizordner_mappe_fort(&self, pfad: &str) -> Ergebnis<()> {
        for v in self.notizordner()? {
            if liegt_in(Some(&v.mappe), pfad) {
                self.notizordner_trennen(&v.mappe)?;
            }
        }
        Ok(())
    }

    fn spiegelstaende(&self, mappe: &str) -> Ergebnis<BTreeMap<String, Stand>> {
        let db = self.db();
        let mut a = db.prepare(
            "SELECT zk_id, relpfad, datei_abdruck, notiz_abdruck, groesse, mtime
             FROM notizordner_dateien WHERE mappe = ?1",
        )?;
        let liste = a
            .query_map(params![mappe], |z| {
                Ok((
                    z.get::<_, String>(0)?,
                    Stand {
                        relpfad: z.get(1)?,
                        datei_abdruck: z.get(2)?,
                        notiz_abdruck: z.get(3)?,
                        groesse: z.get(4)?,
                        mtime: z.get(5)?,
                    },
                ))
            })?
            .collect::<Result<_, _>>()?;
        Ok(liste)
    }

    fn stand_merken(&self, mappe: &str, zk_id: &str, fund: &Fund, notiz: &Notiz) -> Ergebnis<()> {
        self.db().execute(
            "INSERT INTO notizordner_dateien (zk_id, mappe, relpfad, datei_abdruck, notiz_abdruck, groesse, mtime)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT (zk_id) DO UPDATE SET
                mappe = excluded.mappe, relpfad = excluded.relpfad,
                datei_abdruck = excluded.datei_abdruck, notiz_abdruck = excluded.notiz_abdruck,
                groesse = excluded.groesse, mtime = excluded.mtime",
            params![zk_id, mappe, fund.relpfad, fund.abdruck, notizabdruck(notiz), fund.groesse, fund.mtime],
        )?;
        Ok(())
    }

    fn stand_vergessen(&self, zk_id: &str) -> Ergebnis<()> {
        self.db().execute(
            "DELETE FROM notizordner_dateien WHERE zk_id = ?1",
            params![zk_id],
        )?;
        Ok(())
    }

    /// Einen Durchlauf fuer eine Verbindung.
    pub fn notizordner_spiegeln(&self, v: &Verbindung) -> Ergebnis<Spiegelbericht> {
        if !v.ordner.is_dir() {
            return Err(SpeicherFehler::OrdnerFehlt(
                v.ordner.to_string_lossy().to_string(),
            ));
        }
        let mut bericht = Spiegelbericht::default();
        let staende = self.spiegelstaende(&v.mappe)?;

        // Der Unterordner einer Mappe: ihre Stufen unterhalb der verbundenen.
        let ordner_von = |mappe: &str| unterordner(&v.mappe, mappe);
        // Und zurueck: welche Mappe meint ein Unterordner? Bekannte Mappen
        // ueber ihren (bereinigten) Namen, unbekannte werden neue Mappen.
        let mut zu_mappe: BTreeMap<String, String> = BTreeMap::new();
        for m in self.mappen()? {
            if liegt_in(Some(&m.pfad), &v.mappe) && m.pfad != v.mappe {
                let o = ordner_von(&m.pfad);
                std::fs::create_dir_all(pfad_von(&v.ordner, &o))?;
                zu_mappe.insert(o, m.pfad);
            }
        }

        let soll = |n: &Notiz| -> String {
            let o = ordner_von(n.mappe.as_deref().unwrap_or(""));
            let name = dateiname(&n.zk_id, &n.titel);
            if o.is_empty() {
                name
            } else {
                format!("{o}/{name}")
            }
        };
        let mappe_von = |relpfad: &str| -> String {
            match relpfad.rsplit_once('/') {
                None => v.mappe.clone(),
                Some((o, _)) => zu_mappe
                    .get(o)
                    .cloned()
                    .unwrap_or_else(|| format!("{}/{o}", v.mappe)),
            }
        };

        // Die Dateien, wie sie jetzt sind.
        let mut mit_id: BTreeMap<String, Fund> = BTreeMap::new();
        let mut ohne_id: Vec<Fund> = Vec::new();
        let mut gelesen: BTreeMap<String, String> = BTreeMap::new();
        let mut stapel = vec![(v.ordner.clone(), String::new())];
        while let Some((dir, rel)) = stapel.pop() {
            for e in std::fs::read_dir(&dir)? {
                let e = e?;
                let name = e.file_name().to_string_lossy().to_string();
                if name.starts_with('.') {
                    continue;
                }
                let r = if rel.is_empty() {
                    name.clone()
                } else {
                    format!("{rel}/{name}")
                };
                let meta = e.metadata()?;
                if meta.is_dir() {
                    stapel.push((e.path(), r));
                    continue;
                }
                if !name.ends_with(".md") {
                    continue;
                }
                let (groesse, zeit) = (meta.len() as i64, mtime(&meta));
                let id_titel = aus_dateiname(&name);
                // Unveraendert seit dem letzten Mal (Groesse und Zeit)? Dann
                // nicht neu lesen -- der Waechter laeuft oft.
                let bekannt = id_titel
                    .as_ref()
                    .and_then(|(id, _)| staende.get(id))
                    .filter(|s| s.relpfad == r && s.groesse == groesse && s.mtime == zeit);
                let abdruck_jetzt = match bekannt {
                    Some(s) => s.datei_abdruck.clone(),
                    None => {
                        let text = std::fs::read_to_string(e.path())?;
                        let a = abdruck(text.as_bytes());
                        gelesen.insert(r.clone(), text);
                        a
                    }
                };
                let fund = Fund {
                    relpfad: r.clone(),
                    abdruck: abdruck_jetzt,
                    groesse,
                    mtime: zeit,
                    titel: id_titel.as_ref().map_or_else(
                        || name.trim_end_matches(".md").to_string(),
                        |(_, t)| t.clone(),
                    ),
                };
                match id_titel {
                    Some((id, _)) if !mit_id.contains_key(&id) => {
                        mit_id.insert(id, fund);
                    }
                    // Eine zweite Datei mit derselben Id (kopiert): wie neu.
                    _ => ohne_id.push(fund),
                }
            }
        }
        let text_von = |relpfad: &str, gelesen: &BTreeMap<String, String>| -> Ergebnis<String> {
            match gelesen.get(relpfad) {
                Some(t) => Ok(t.clone()),
                None => Ok(std::fs::read_to_string(pfad_von(&v.ordner, relpfad))?),
            }
        };

        // Eine Notiz in die Datei schreiben (und eine alte Datei fortraeumen).
        let notiz_hinaus = |speicher: &Speicher, n: &Notiz, alt: Option<&str>| -> Ergebnis<()> {
            let rel = soll(n);
            let ziel = pfad_von(&v.ordner, &rel);
            schreiben(&ziel, &n.inhalt)?;
            if let Some(a) = alt.filter(|a| *a != rel) {
                let _ = std::fs::remove_file(pfad_von(&v.ordner, a));
            }
            let meta = std::fs::metadata(&ziel)?;
            let fund = Fund {
                relpfad: rel,
                abdruck: abdruck(n.inhalt.as_bytes()),
                groesse: meta.len() as i64,
                mtime: mtime(&meta),
                titel: n.titel.clone(),
            };
            speicher.stand_merken(&v.mappe, &n.zk_id, &fund, n)
        };

        let mut ids: BTreeSet<String> = mit_id.keys().cloned().collect();
        ids.extend(staende.keys().cloned());
        for n in self.notizen()? {
            if liegt_in(n.mappe.as_deref(), &v.mappe) {
                ids.insert(n.zk_id.clone());
            }
        }
        let mut vergeben: BTreeSet<String> = ids.clone();

        for id in &ids {
            let notiz = self.notiz(id)?;
            let lebt = notiz
                .as_ref()
                .filter(|n| !n.im_papierkorb() && liegt_in(n.mappe.as_deref(), &v.mappe));
            let datei = mit_id.get(id);
            let stand = staende.get(id);

            match (lebt, datei, stand) {
                // Beide da.
                (Some(n), Some(f), s) => {
                    let datei_neu =
                        s.is_none_or(|s| s.datei_abdruck != f.abdruck || s.relpfad != f.relpfad);
                    let notiz_neu = s
                        .is_none_or(|s| s.notiz_abdruck != notizabdruck(n) || s.relpfad != soll(n));
                    if !datei_neu && !notiz_neu {
                        continue;
                    }
                    let text = text_von(&f.relpfad, &gelesen)?;
                    if s.is_none() && text == n.inhalt {
                        // Zum ersten Mal gesehen, schon gleich: nur merken.
                        if f.relpfad != soll(n) {
                            notiz_hinaus(self, n, Some(&f.relpfad))?;
                            bericht.geschrieben += 1;
                        } else {
                            self.stand_merken(&v.mappe, id, f, n)?;
                        }
                        continue;
                    }
                    let inhalt_drauszen = s.is_none_or(|s| s.datei_abdruck != f.abdruck);
                    if datei_neu && !notiz_neu {
                        // Draussen bearbeitet, umbenannt oder verschoben.
                        let mut neu = n.clone();
                        neu.inhalt = text;
                        if sauberer_name(&n.titel) != f.titel && !f.titel.is_empty() {
                            neu.titel = f.titel.clone();
                        }
                        neu.mappe = Some(mappe_von(&f.relpfad));
                        neu.geaendert_at = String::new();
                        self.notiz_schreiben(&neu, Protokoll::Merken)?;
                        let neu = self.notiz(id)?.unwrap_or(neu);
                        if f.relpfad != soll(&neu) {
                            notiz_hinaus(self, &neu, Some(&f.relpfad))?;
                        } else {
                            self.stand_merken(&v.mappe, id, f, &neu)?;
                        }
                        bericht.uebernommen += 1;
                    } else if notiz_neu && !(datei_neu && inhalt_drauszen) {
                        // In der App geaendert (oder nur draussen verschoben).
                        notiz_hinaus(self, n, Some(&f.relpfad))?;
                        bericht.geschrieben += 1;
                    } else {
                        // Beide Seiten: die Datei wird zur Kopie, die Notiz gewinnt.
                        let kopie_id = freie_zk_id(self, &vergeben)?;
                        vergeben.insert(kopie_id.clone());
                        let kopie = Notiz {
                            zk_id: kopie_id.clone(),
                            titel: format!(
                                "{} (Konflikt Ordner {})",
                                if f.titel.is_empty() {
                                    &n.titel
                                } else {
                                    &f.titel
                                },
                                chrono::Local::now().format("%Y-%m-%d")
                            ),
                            inhalt: text,
                            mappe: Some(mappe_von(&f.relpfad)),
                            papierkorb_at: None,
                            geaendert_at: String::new(),
                        };
                        self.notiz_schreiben(&kopie, Protokoll::Merken)?;
                        notiz_hinaus(self, n, Some(&f.relpfad))?;
                        if let Some(k) = self.notiz(&kopie_id)? {
                            notiz_hinaus(self, &k, None)?;
                        }
                        bericht.konflikte += 1;
                    }
                }
                // Notiz da, Datei fehlt.
                (Some(n), None, s) => {
                    match s {
                        // War gespiegelt und ist draussen geloescht, hier unveraendert.
                        Some(s) if s.notiz_abdruck == notizabdruck(n) && s.relpfad == soll(n) => {
                            self.papierkorb(&Art::Notiz, id, Protokoll::Merken)?;
                            self.stand_vergessen(id)?;
                            bericht.papierkorb += 1;
                        }
                        // Neu in der App oder hier geaendert: schreiben.
                        _ => {
                            notiz_hinaus(self, n, None)?;
                            bericht.geschrieben += 1;
                        }
                    }
                }
                // Datei da, Notiz fort (Papierkorb, geloescht, hinausverschoben).
                (None, Some(f), Some(s)) => {
                    let draussen_geaendert = s.datei_abdruck != f.abdruck;
                    if draussen_geaendert {
                        // Draussen weitergeschrieben, waehrend sie hier ging:
                        // nichts verlieren -- als neue Notiz behalten.
                        ohne_id.push(f.clone());
                    } else {
                        let _ = std::fs::remove_file(pfad_von(&v.ordner, &f.relpfad));
                        bericht.entfernt += 1;
                    }
                    self.stand_vergessen(id)?;
                }
                // Datei mit Id, die hier niemand kennt: neue Notiz unter dieser Id.
                (None, Some(f), None) => {
                    if notiz.is_some() {
                        // Die Id ist vergeben (Papierkorb, andere Mappe):
                        // als neue Notiz mit eigener Id.
                        ohne_id.push(f.clone());
                        continue;
                    }
                    let n = Notiz {
                        zk_id: id.clone(),
                        titel: f.titel.clone(),
                        inhalt: text_von(&f.relpfad, &gelesen)?,
                        mappe: Some(mappe_von(&f.relpfad)),
                        papierkorb_at: None,
                        geaendert_at: String::new(),
                    };
                    self.notiz_schreiben(&n, Protokoll::Merken)?;
                    let n = self.notiz(id)?.unwrap_or(n);
                    if f.relpfad != soll(&n) {
                        notiz_hinaus(self, &n, Some(&f.relpfad))?;
                    } else {
                        self.stand_merken(&v.mappe, id, f, &n)?;
                    }
                    bericht.neu += 1;
                }
                (None, None, Some(_)) => self.stand_vergessen(id)?,
                (None, None, None) => {}
            }
        }

        // Neue Dateien ohne (eindeutige) Id: neue Notizen, Datei bekommt die Id.
        for f in ohne_id {
            let id = freie_zk_id(self, &vergeben)?;
            vergeben.insert(id.clone());
            let n = Notiz {
                zk_id: id.clone(),
                titel: f.titel.clone(),
                inhalt: text_von(&f.relpfad, &gelesen)?,
                mappe: Some(mappe_von(&f.relpfad)),
                papierkorb_at: None,
                geaendert_at: String::new(),
            };
            self.notiz_schreiben(&n, Protokoll::Merken)?;
            let n = self.notiz(&id)?.unwrap_or(n);
            let alt = pfad_von(&v.ordner, &f.relpfad);
            notiz_hinaus(self, &n, None)?;
            if alt != pfad_von(&v.ordner, &soll(&n)) && alt.exists() {
                let _ = std::fs::remove_file(alt);
            }
            bericht.neu += 1;
        }

        Ok(bericht)
    }

    /// Wo eine Notiz dieser Mappe im Ordner liegt -- fuer „Ordner zeigen".
    pub fn notizordner_von(&self, mappe: &str) -> Ergebnis<Option<Verbindung>> {
        Ok(self
            .db()
            .query_row(
                "SELECT mappe, ordner FROM notizordner WHERE mappe = ?1",
                params![mappe],
                |z| {
                    Ok(Verbindung {
                        mappe: z.get(0)?,
                        ordner: PathBuf::from(z.get::<_, String>(1)?),
                    })
                },
            )
            .optional()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn notiz(id: &str, titel: &str, mappe: &str, inhalt: &str) -> Notiz {
        Notiz {
            zk_id: id.into(),
            titel: titel.into(),
            inhalt: inhalt.into(),
            mappe: Some(mappe.into()),
            papierkorb_at: None,
            geaendert_at: String::new(),
        }
    }

    fn aufbau() -> (Speicher, tempfile::TempDir, Verbindung) {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let v = Verbindung {
            mappe: "Uni".into(),
            ordner: dir.path().to_path_buf(),
        };
        assert!(s.notizordner_verbinden(&v.mappe, &v.ordner).unwrap());
        (s, dir, v)
    }

    fn lesen(v: &Verbindung, rel: &str) -> String {
        std::fs::read_to_string(pfad_von(&v.ordner, rel)).unwrap()
    }

    /// Zwischen zwei Durchlaeufen muss sich die Zeit sichtbar bewegen,
    /// sonst haelt der Groesse-und-Zeit-Vergleich eine Aenderung fuer keine.
    fn spaeter() {
        std::thread::sleep(std::time::Duration::from_millis(15));
    }

    #[test]
    fn namen_wie_frueher_auf_dem_server() {
        assert_eq!(
            dateiname("20261019120000", "Vorlage: Fehlerbericht"),
            "20261019120000 Vorlage – Fehlerbericht.md"
        );
        assert_eq!(
            dateiname("20261019120000", "Um 12:30 / heute?"),
            "20261019120000 Um 12-30 - heute.md"
        );
        assert_eq!(dateiname("20261019120000", ""), "20261019120000.md");
        assert_eq!(
            aus_dateiname("20261019120000 Beispiel.md"),
            Some(("20261019120000".into(), "Beispiel".into()))
        );
        assert_eq!(
            aus_dateiname("20261019120000.md"),
            Some(("20261019120000".into(), String::new()))
        );
        assert_eq!(aus_dateiname("Beispiel.md"), None);
    }

    #[test]
    fn notizen_gehen_als_dateien_hinaus_mit_untermappen() {
        let (s, _d, v) = aufbau();
        s.notiz_schreiben(
            &notiz("20261019120000", "Beispiel", "Uni", "Text"),
            Protokoll::Still,
        )
        .unwrap();
        s.notiz_schreiben(
            &notiz("20261019120001", "Zweite", "Uni/Kurs: A", "Mehr"),
            Protokoll::Still,
        )
        .unwrap();
        s.notiz_schreiben(
            &notiz("20261019120002", "Draussen", "Privat", "Nein"),
            Protokoll::Still,
        )
        .unwrap();

        let b = s.notizordner_spiegeln(&v).unwrap();

        assert_eq!(b.geschrieben, 2);
        assert_eq!(lesen(&v, "20261019120000 Beispiel.md"), "Text");
        assert_eq!(lesen(&v, "Kurs – A/20261019120001 Zweite.md"), "Mehr");
        // Ein zweiter Lauf tut nichts.
        assert!(!s.notizordner_spiegeln(&v).unwrap().etwas_getan());
    }

    #[test]
    fn draussen_bearbeitet_kommt_herein() {
        let (s, _d, v) = aufbau();
        s.notiz_schreiben(
            &notiz("20261019120000", "Beispiel", "Uni", "Alt"),
            Protokoll::Still,
        )
        .unwrap();
        s.notizordner_spiegeln(&v).unwrap();
        spaeter();
        std::fs::write(
            pfad_von(&v.ordner, "20261019120000 Beispiel.md"),
            "Neu aus Zettlr",
        )
        .unwrap();

        let b = s.notizordner_spiegeln(&v).unwrap();

        assert_eq!(b.uebernommen, 1);
        assert_eq!(
            s.notiz("20261019120000").unwrap().unwrap().inhalt,
            "Neu aus Zettlr"
        );
        assert!(!s.notizordner_spiegeln(&v).unwrap().etwas_getan());
    }

    #[test]
    fn draussen_umbenannt_und_verschoben() {
        let (s, _d, v) = aufbau();
        s.notiz_schreiben(
            &notiz("20261019120000", "Beispiel", "Uni", "Text"),
            Protokoll::Still,
        )
        .unwrap();
        s.notizordner_spiegeln(&v).unwrap();
        std::fs::create_dir_all(pfad_von(&v.ordner, "Neu")).unwrap();
        std::fs::rename(
            pfad_von(&v.ordner, "20261019120000 Beispiel.md"),
            pfad_von(&v.ordner, "Neu/20261019120000 Anders.md"),
        )
        .unwrap();

        s.notizordner_spiegeln(&v).unwrap();

        let n = s.notiz("20261019120000").unwrap().unwrap();
        assert_eq!(n.titel, "Anders");
        assert_eq!(n.mappe.as_deref(), Some("Uni/Neu"));
        assert!(!s.notizordner_spiegeln(&v).unwrap().etwas_getan());
    }

    #[test]
    fn in_der_app_geaendert_schreibt_und_benennt_um() {
        let (s, _d, v) = aufbau();
        s.notiz_schreiben(
            &notiz("20261019120000", "Beispiel", "Uni", "Text"),
            Protokoll::Still,
        )
        .unwrap();
        s.notizordner_spiegeln(&v).unwrap();
        s.notiz_schreiben(
            &notiz("20261019120000", "Umbenannt", "Uni", "Geändert"),
            Protokoll::Still,
        )
        .unwrap();

        s.notizordner_spiegeln(&v).unwrap();

        assert_eq!(lesen(&v, "20261019120000 Umbenannt.md"), "Geändert");
        assert!(!pfad_von(&v.ordner, "20261019120000 Beispiel.md").exists());
    }

    #[test]
    fn beide_geaendert_ergibt_eine_kopie() {
        let (s, _d, v) = aufbau();
        s.notiz_schreiben(
            &notiz("20261019120000", "Beispiel", "Uni", "Alt"),
            Protokoll::Still,
        )
        .unwrap();
        s.notizordner_spiegeln(&v).unwrap();
        spaeter();
        std::fs::write(
            pfad_von(&v.ordner, "20261019120000 Beispiel.md"),
            "Draussen",
        )
        .unwrap();
        s.notiz_schreiben(
            &notiz("20261019120000", "Beispiel", "Uni", "Drinnen"),
            Protokoll::Still,
        )
        .unwrap();

        let b = s.notizordner_spiegeln(&v).unwrap();

        assert_eq!(b.konflikte, 1);
        assert_eq!(
            s.notiz("20261019120000").unwrap().unwrap().inhalt,
            "Drinnen"
        );
        assert_eq!(lesen(&v, "20261019120000 Beispiel.md"), "Drinnen");
        let kopie = s
            .notizen()
            .unwrap()
            .into_iter()
            .find(|n| n.inhalt == "Draussen")
            .unwrap();
        assert!(kopie.titel.starts_with("Beispiel (Konflikt Ordner "));
        assert!(!s.notizordner_spiegeln(&v).unwrap().etwas_getan());
    }

    #[test]
    fn draussen_geloescht_geht_in_den_papierkorb() {
        let (s, _d, v) = aufbau();
        s.notiz_schreiben(
            &notiz("20261019120000", "Beispiel", "Uni", "Text"),
            Protokoll::Still,
        )
        .unwrap();
        s.notizordner_spiegeln(&v).unwrap();
        std::fs::remove_file(pfad_von(&v.ordner, "20261019120000 Beispiel.md")).unwrap();

        assert_eq!(s.notizordner_spiegeln(&v).unwrap().papierkorb, 1);
        assert!(s.notiz("20261019120000").unwrap().unwrap().im_papierkorb());
    }

    #[test]
    fn in_der_app_weggelegt_verschwindet_die_datei() {
        let (s, _d, v) = aufbau();
        s.notiz_schreiben(
            &notiz("20261019120000", "Beispiel", "Uni", "Text"),
            Protokoll::Still,
        )
        .unwrap();
        s.notizordner_spiegeln(&v).unwrap();
        s.papierkorb(&Art::Notiz, "20261019120000", Protokoll::Still)
            .unwrap();

        assert_eq!(s.notizordner_spiegeln(&v).unwrap().entfernt, 1);
        assert!(!pfad_von(&v.ordner, "20261019120000 Beispiel.md").exists());
    }

    #[test]
    fn neue_datei_ohne_id_wird_eine_notiz() {
        let (s, _d, v) = aufbau();
        std::fs::create_dir_all(pfad_von(&v.ordner, "Kurs")).unwrap();
        std::fs::write(pfad_von(&v.ordner, "Kurs/Idee.md"), "Aus Zettlr").unwrap();
        std::fs::write(pfad_von(&v.ordner, "bild.png"), "kein text").unwrap();

        let b = s.notizordner_spiegeln(&v).unwrap();

        assert_eq!(b.neu, 1);
        let n = s
            .notizen()
            .unwrap()
            .into_iter()
            .find(|n| n.inhalt == "Aus Zettlr")
            .unwrap();
        assert_eq!(n.titel, "Idee");
        assert_eq!(n.mappe.as_deref(), Some("Uni/Kurs"));
        assert!(pfad_von(&v.ordner, &format!("Kurs/{} Idee.md", n.zk_id)).exists());
        assert!(!pfad_von(&v.ordner, "Kurs/Idee.md").exists());
        assert!(!s.notizordner_spiegeln(&v).unwrap().etwas_getan());
    }

    #[test]
    fn ein_fehlender_ordner_raeumt_nichts_ab() {
        let (s, d, v) = aufbau();
        s.notiz_schreiben(
            &notiz("20261019120000", "Beispiel", "Uni", "Text"),
            Protokoll::Still,
        )
        .unwrap();
        s.notizordner_spiegeln(&v).unwrap();
        drop(d);

        assert!(matches!(
            s.notizordner_spiegeln(&v),
            Err(SpeicherFehler::OrdnerFehlt(_))
        ));
        assert!(!s.notiz("20261019120000").unwrap().unwrap().im_papierkorb());
    }

    #[test]
    fn eine_notiz_gehoert_hoechstens_zu_einem_ordner() {
        let (s, _d, _v) = aufbau();
        assert!(!s.notizordner_moeglich("Uni").unwrap());
        assert!(!s.notizordner_moeglich("Uni/Kurs").unwrap());
        assert!(s.notizordner_moeglich("Privat").unwrap());
        assert!(s.notizordner_moeglich("Unikat").unwrap());
    }

    #[test]
    fn umbenennen_der_mappe_behaelt_die_verbindung() {
        let (s, _d, v) = aufbau();
        s.notiz_schreiben(
            &notiz("20261019120000", "Beispiel", "Uni", "Text"),
            Protokoll::Still,
        )
        .unwrap();
        s.notizordner_spiegeln(&v).unwrap();

        s.mappe_umbenennen("Uni", "Studium").unwrap();

        let v2 = s.notizordner().unwrap().pop().unwrap();
        assert_eq!(v2.mappe, "Studium");
        assert!(!s.notizordner_spiegeln(&v2).unwrap().etwas_getan());
    }
}
