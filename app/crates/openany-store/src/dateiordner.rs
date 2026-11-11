//! Einen Ordner auf diesem Computer mit Dateien bzw. Dokumenten abgleichen,
//! wie Nextcloud (docs/plan-viele-dateien.md, D4). Nur in der Desktop-App.
//!
//! **Ein Paar** ist ein Ordner auf diesem Computer und ein Bereich in openany:
//! eine Zone (`documents`, `files`) ab einem Ordner oder ihrer obersten Ebene.
//! Unterordner werden Ordner bzw. Akten. Die Paare gelten je Gerät und gehen
//! nicht in den Abgleich.
//!
//! **Zwei Schritte je Durchlauf.** [`scannen`] liest den Ordner, ohne den
//! Speicher zu sperren -- neue und geänderte Dateien kommen dabei schon in
//! den Inhaltsspeicher, als Klon, wo das Dateisystem es kann (APFS, btrfs,
//! XFS: kein zweiter Platz), sonst als Kopie. Danach entscheidet
//! [`Speicher::dateiordner_abgleichen`] unter der Sperre, was geschieht.
//!
//! Je Datei merkt sich das Paar, wie sie beim letzten Mal aussah (Pfad,
//! Abdruck, Größe, Änderungszeit). Daraus folgt, wer sich bewegt hat:
//!
//! | Ordner       | openany            | was geschieht                         |
//! |--------------|--------------------|---------------------------------------|
//! | gleich       | gleich             | nichts                                |
//! | geändert     | gleich             | neuer Inhalt in openany               |
//! | gleich       | geändert/verschoben| Datei neu schreiben bzw. umbenennen   |
//! | geändert     | geändert           | beide bleiben, die aus dem Ordner als „Name (Konflikt, Gerät, Datum)“ |
//! | gelöscht     | gleich             | openany: Papierkorb                   |
//! | gleich       | gelöscht           | Ordner: Papierkorb des Systems        |
//! | neu          | --                 | neu in openany (umbenannt: verschoben)|
//! | --           | neu                | neu im Ordner                         |
//!
//! Geschrieben wird nie halb: erst in eine Zwischendatei daneben, dann
//! umbenannt. **Ein fehlender Ordner ist kein leerer** -- ist er nicht da
//! (Stick ab, umbenannt), wird nichts abgeglichen.

use crate::{Datei, Ergebnis, Inhalte, Protokoll, Speicher, SpeicherFehler};
use rusqlite::params;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Ein Ordner auf diesem Computer, verbunden mit einem Bereich.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ordnerpaar {
    pub id: i64,
    pub zone: String,
    /// Der Ordner in openany; `None` = oberste Ebene der Zone.
    pub wurzel: Option<String>,
    pub ordner: PathBuf,
}

/// Wie eine Datei oder ein Ordner beim letzten Mal aussah.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stand {
    pub relpfad: String,
    pub uuid: String,
    pub ist_ordner: bool,
    /// Leer bei Ordnern.
    pub abdruck: String,
    pub groesse: u64,
    pub mtime: i64,
}

/// Etwas im Ordner, wie es jetzt ist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fund {
    pub relpfad: String,
    pub ist_ordner: bool,
    /// Bei Dateien: liegt im Inhaltsspeicher.
    pub abdruck: String,
    pub groesse: u64,
    pub mtime: i64,
}

/// Was [`scannen`] fand.
#[derive(Debug, Default)]
pub struct Scan {
    pub funde: Vec<Fund>,
    /// Gerade in Arbeit (eben geändert) oder nicht lesbar: in diesem
    /// Durchlauf unberührt, auch ihr Stand.
    pub beschaeftigt: BTreeSet<String>,
}

/// Was ein Durchlauf getan hat.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Bericht {
    /// In openany neu, geändert, verschoben oder in den Papierkorb.
    pub herein: usize,
    /// Im Ordner geschrieben, umbenannt oder weggeworfen.
    pub hinaus: usize,
    pub konflikte: usize,
    /// Neu angelegte Ordner in openany (für „auf diesem Gerät gewählt“).
    pub neue_ordner: Vec<(String, Option<String>)>,
    /// Inhalte, die hier fehlen und erst geholt werden müssen: (Abdruck, Größe).
    pub fehlt: Vec<(String, u64)>,
}

impl Bericht {
    pub fn etwas_getan(&self) -> bool {
        self.herein + self.hinaus + self.konflikte > 0
    }
}

/// Was nie abgeglichen wird: Systemkram, halbe Downloads, Sperrdateien von
/// Office -- und alles Versteckte. Dazu `.openanyignore` im Ordner, ein Muster
/// je Zeile (`*` und `?`, mit `/` am Ende nur Ordner).
#[derive(Debug, Clone)]
pub struct Ausnahmen {
    muster: Vec<(regex::Regex, bool)>,
}

const IMMER: &[&str] = &[
    ".*",
    "~$*",
    "*.tmp",
    "*.part",
    "*.crdownload",
    "Thumbs.db",
    "desktop.ini",
    "*.openany-tmp",
];

impl Ausnahmen {
    pub fn lesen(ordner: &Path) -> Self {
        let eigene = std::fs::read_to_string(ordner.join(".openanyignore")).unwrap_or_default();
        Self::aus(eigene.lines())
    }

    fn aus<'a>(eigene: impl Iterator<Item = &'a str>) -> Self {
        let muster = IMMER
            .iter()
            .copied()
            .chain(eigene)
            .map(str::trim)
            .filter(|z| !z.is_empty() && !z.starts_with('#'))
            .filter_map(|z| {
                let nur_ordner = z.ends_with('/');
                let z = z.trim_end_matches('/');
                let mut r = String::from("(?i)^");
                for c in z.chars() {
                    match c {
                        '*' => r.push_str("[^/]*"),
                        '?' => r.push_str("[^/]"),
                        c => r.push_str(&regex::escape(&c.to_string())),
                    }
                }
                r.push('$');
                regex::Regex::new(&r).ok().map(|r| (r, nur_ordner))
            })
            .collect();
        Self { muster }
    }

    /// Fällt der Pfad (oder ein Ordner darüber) unter eine Ausnahme? Muster
    /// mit `/` gelten für den Pfad vom Ordner an, andere für jeden Namen.
    pub fn trifft(&self, relpfad: &str, ist_ordner: bool) -> bool {
        let teile: Vec<&str> = relpfad.split('/').collect();
        teile.iter().enumerate().any(|(i, name)| {
            let ordner_hier = i + 1 < teile.len() || ist_ordner;
            let bis_hier = teile[..=i].join("/");
            self.muster.iter().any(|(r, nur_ordner)| {
                (!nur_ordner || ordner_hier)
                    && (r.is_match(name) || (r.as_str().contains('/') && r.is_match(&bis_hier)))
            })
        })
    }
}

fn mtime_ms(meta: &std::fs::Metadata) -> i64 {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_millis() as i64)
}

/// Eine Datei aus dem Ordner in den Inhaltsspeicher -- als Klon, wo es geht.
fn einlesen(pfad: &Path, inhalte: &Inhalte) -> std::io::Result<(String, u64)> {
    let zwischen = inhalte.zwischenpfad()?;
    if let Err(e) = reflink_copy::reflink_or_copy(pfad, &zwischen) {
        let _ = std::fs::remove_file(&zwischen);
        return Err(e);
    }
    inhalte.uebernehmen(&zwischen)
}

/// Den Ordner lesen. Dateien, die seit dem letzten Mal gleich groß und gleich
/// alt sind, werden nicht neu gelesen. Was jünger als `ruhe` ist, schreibt
/// womöglich gerade jemand -- es wartet bis zum nächsten Durchlauf.
pub fn scannen(
    paar: &Ordnerpaar,
    stand: &[Stand],
    inhalte: &Inhalte,
    ruhe: Duration,
) -> Ergebnis<Scan> {
    if !paar.ordner.is_dir() {
        return Err(SpeicherFehler::OrdnerFehlt(
            paar.ordner.to_string_lossy().to_string(),
        ));
    }
    let ausnahmen = Ausnahmen::lesen(&paar.ordner);
    let bekannt: HashMap<&str, &Stand> = stand.iter().map(|s| (s.relpfad.as_str(), s)).collect();
    let grenze = SystemTime::now()
        .checked_sub(ruhe)
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(i64::MAX, |d| d.as_millis() as i64);

    let mut scan = Scan::default();
    let mut offen = vec![(paar.ordner.clone(), String::new())];
    while let Some((dir, rel)) = offen.pop() {
        let Ok(eintraege) = std::fs::read_dir(&dir) else {
            if !rel.is_empty() {
                scan.beschaeftigt.insert(rel);
            }
            continue;
        };
        for e in eintraege.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            let relpfad = if rel.is_empty() {
                name
            } else {
                format!("{rel}/{name}")
            };
            // Verknüpfungen nicht folgen: eine auf einen Elternordner liefe
            // im Kreis.
            let Ok(meta) = std::fs::symlink_metadata(e.path()) else {
                continue;
            };
            if meta.is_dir() {
                if ausnahmen.trifft(&relpfad, true) {
                    continue;
                }
                scan.funde.push(Fund {
                    relpfad: relpfad.clone(),
                    ist_ordner: true,
                    abdruck: String::new(),
                    groesse: 0,
                    mtime: 0,
                });
                offen.push((e.path(), relpfad));
            } else if meta.is_file() {
                if ausnahmen.trifft(&relpfad, false) {
                    continue;
                }
                let (groesse, mtime) = (meta.len(), mtime_ms(&meta));
                if mtime > grenze {
                    scan.beschaeftigt.insert(relpfad);
                    continue;
                }
                let abdruck = match bekannt.get(relpfad.as_str()) {
                    Some(s) if !s.ist_ordner && s.groesse == groesse && s.mtime == mtime => {
                        s.abdruck.clone()
                    }
                    _ => match einlesen(&e.path(), inhalte) {
                        Ok((a, _)) => a,
                        Err(_) => {
                            scan.beschaeftigt.insert(relpfad);
                            continue;
                        }
                    },
                };
                scan.funde.push(Fund {
                    relpfad,
                    ist_ordner: false,
                    abdruck,
                    groesse,
                    mtime,
                });
            }
        }
    }
    Ok(scan)
}

/// `Rechnung.pdf` → `Rechnung (Konflikt, Mac, 2026-11-10 21-30).pdf`
pub fn konfliktname(name: &str, geraet: &str, jetzt: chrono::DateTime<chrono::Local>) -> String {
    let zusatz = format!("(Konflikt, {geraet}, {})", jetzt.format("%Y-%m-%d %H-%M"));
    match name.rsplit_once('.') {
        Some((stamm, endung)) if !stamm.is_empty() => format!("{stamm} {zusatz}.{endung}"),
        _ => format!("{name} {zusatz}"),
    }
}

fn eltern_von(relpfad: &str) -> &str {
    relpfad.rsplit_once('/').map_or("", |(e, _)| e)
}

fn name_von(relpfad: &str) -> &str {
    relpfad.rsplit_once('/').map_or(relpfad, |(_, n)| n)
}

/// Ein Durchlauf unter der Sperre: hält Ordner und openany gleich.
struct Lauf<'a> {
    speicher: &'a Speicher,
    paar: &'a Ordnerpaar,
    inhalte: &'a Inhalte,
    wegwerfen: &'a dyn Fn(&Path) -> std::io::Result<()>,
    mime: &'a dyn Fn(&str) -> String,
    geraet: &'a str,
    bericht: Bericht,
    /// relpfad → uuid der Ordner in openany.
    ordner: HashMap<String, Option<String>>,
    neu_stand: BTreeMap<String, Stand>,
}

impl Lauf<'_> {
    fn ziel(&self, relpfad: &str) -> PathBuf {
        relpfad
            .split('/')
            .fold(self.paar.ordner.clone(), |p, t| p.join(t))
    }

    /// Den Ordner in openany für diesen Pfad -- angelegt, wo er fehlt.
    fn app_ordner(&mut self, relpfad: &str) -> Ergebnis<Option<String>> {
        if let Some(u) = self.ordner.get(relpfad) {
            return Ok(u.clone());
        }
        let eltern = self.app_ordner(eltern_von(relpfad))?;
        let name = name_von(relpfad);
        let vorhanden = self
            .speicher
            .dateien_im_ordner(&self.paar.zone, eltern.as_deref())?
            .into_iter()
            .find(|d| d.ist_ordner && d.name.to_lowercase() == name.to_lowercase());
        let uuid = match vorhanden {
            Some(d) => d.uuid,
            None => {
                let uuid = uuid::Uuid::new_v4().to_string();
                self.speicher.datei_schreiben(
                    &Datei {
                        uuid: uuid.clone(),
                        zone: self.paar.zone.clone(),
                        ist_ordner: true,
                        eltern: eltern.clone(),
                        name: name.to_string(),
                        ..Default::default()
                    },
                    Protokoll::Merken,
                )?;
                self.bericht.herein += 1;
                self.bericht.neue_ordner.push((uuid.clone(), eltern));
                uuid
            }
        };
        self.ordner.insert(relpfad.to_string(), Some(uuid.clone()));
        self.neu_stand.insert(
            relpfad.to_string(),
            Stand {
                relpfad: relpfad.to_string(),
                uuid: uuid.clone(),
                ist_ordner: true,
                abdruck: String::new(),
                groesse: 0,
                mtime: 0,
            },
        );
        Ok(Some(uuid))
    }

    /// Ein freier Name in openany neben `relpfad`.
    fn freier_name(&self, eltern: Option<&str>, name: &str) -> Ergebnis<String> {
        let mut kandidat = name.to_string();
        let mut n = 2;
        while self
            .speicher
            .dateiname_vergeben(&self.paar.zone, eltern, &kandidat, None)?
        {
            kandidat = match name.rsplit_once('.') {
                Some((s, e)) if !s.is_empty() => format!("{s} ({n}).{e}"),
                _ => format!("{name} ({n})"),
            };
            n += 1;
        }
        Ok(kandidat)
    }

    /// Eine Datei aus dem Ordner neu in openany.
    fn neu_herein(&mut self, f: &Fund) -> Ergebnis<String> {
        let eltern = self.app_ordner(eltern_von(&f.relpfad))?;
        let name = name_von(&f.relpfad);
        let uuid = uuid::Uuid::new_v4().to_string();
        self.speicher.datei_schreiben(
            &Datei {
                uuid: uuid.clone(),
                zone: self.paar.zone.clone(),
                eltern,
                name: name.to_string(),
                groesse: f.groesse,
                mime: (self.mime)(name),
                abdruck: Some(f.abdruck.clone()),
                ..Default::default()
            },
            Protokoll::Merken,
        )?;
        self.bericht.herein += 1;
        self.merken(f, &uuid);
        Ok(uuid)
    }

    fn merken(&mut self, f: &Fund, uuid: &str) {
        self.neu_stand.insert(
            f.relpfad.clone(),
            Stand {
                relpfad: f.relpfad.clone(),
                uuid: uuid.to_string(),
                ist_ordner: false,
                abdruck: f.abdruck.clone(),
                groesse: f.groesse,
                mtime: f.mtime,
            },
        );
    }

    /// Den Inhalt einer Datei aus openany in den Ordner schreiben -- erst
    /// daneben, dann umbenannt. Fehlt der Inhalt hier, steht er im Bericht.
    fn hinaus(&mut self, d: &Datei, relpfad: &str) -> Ergebnis<()> {
        let abdruck = d.abdruck.clone().unwrap_or_default();
        let quelle = self.inhalte.pfad(&abdruck).filter(|p| p.is_file());
        let Some(quelle) = quelle else {
            self.bericht.fehlt.push((abdruck, d.groesse));
            return Ok(());
        };
        let ziel = self.ziel(relpfad);
        let dir = ziel.parent().expect("below the folder").to_path_buf();
        std::fs::create_dir_all(&dir)?;
        let zwischen = dir.join(format!(".{}.openany-tmp", name_von(relpfad)));
        let _ = std::fs::remove_file(&zwischen);
        reflink_copy::reflink_or_copy(&quelle, &zwischen)?;
        std::fs::rename(&zwischen, &ziel)?;
        let meta = std::fs::metadata(&ziel)?;
        self.bericht.hinaus += 1;
        self.neu_stand.insert(
            relpfad.to_string(),
            Stand {
                relpfad: relpfad.to_string(),
                uuid: d.uuid.clone(),
                ist_ordner: false,
                abdruck,
                groesse: meta.len(),
                mtime: mtime_ms(&meta),
            },
        );
        Ok(())
    }

    fn umbenennen(&mut self, von: &str, nach: &str) -> Ergebnis<()> {
        let ziel = self.ziel(nach);
        std::fs::create_dir_all(ziel.parent().expect("below the folder"))?;
        std::fs::rename(self.ziel(von), &ziel)?;
        self.bericht.hinaus += 1;
        Ok(())
    }

    /// Im Ordner in den Papierkorb des Systems.
    fn weg(&mut self, relpfad: &str) {
        if (self.wegwerfen)(&self.ziel(relpfad)).is_ok() {
            self.bericht.hinaus += 1;
        }
    }

    /// Die Fassung aus dem Ordner als Konfliktkopie: im Ordner umbenannt und
    /// als eigene Datei in openany.
    fn konflikt(&mut self, f: &Fund) -> Ergebnis<()> {
        let name = konfliktname(name_von(&f.relpfad), self.geraet, chrono::Local::now());
        let eltern = eltern_von(&f.relpfad);
        let eltern_uuid = self.app_ordner(eltern)?;
        let name = self.freier_name(eltern_uuid.as_deref(), &name)?;
        let neu = if eltern.is_empty() {
            name
        } else {
            format!("{eltern}/{name}")
        };
        self.umbenennen(&f.relpfad, &neu)?;
        let kopie = Fund {
            relpfad: neu,
            ..f.clone()
        };
        self.neu_herein(&kopie)?;
        self.bericht.konflikte += 1;
        Ok(())
    }
}

impl Speicher {
    pub fn dateiordner(&self) -> Ergebnis<Vec<Ordnerpaar>> {
        let db = self.db();
        let mut abfrage =
            db.prepare("SELECT id, zone, wurzel, ordner FROM dateiordner ORDER BY id")?;
        let liste = abfrage
            .query_map([], |z| {
                Ok(Ordnerpaar {
                    id: z.get(0)?,
                    zone: z.get(1)?,
                    wurzel: z.get(2)?,
                    ordner: PathBuf::from(z.get::<_, String>(3)?),
                })
            })?
            .collect::<Result<_, _>>()?;
        Ok(liste)
    }

    /// Verbinden. `None`, wenn derselbe Bereich oder ein Ordner darüber oder
    /// darunter schon verbunden ist -- zwei Spiegel über denselben Dateien
    /// schrieben sich gegenseitig um.
    pub fn dateiordner_verbinden(
        &self,
        zone: &str,
        wurzel: Option<&str>,
        ordner: &Path,
    ) -> Ergebnis<Option<i64>> {
        for p in self.dateiordner()? {
            let gleicher_bereich = p.zone == zone && p.wurzel.as_deref() == wurzel;
            if gleicher_bereich || p.ordner.starts_with(ordner) || ordner.starts_with(&p.ordner) {
                return Ok(None);
            }
        }
        self.db().execute(
            "INSERT INTO dateiordner (zone, wurzel, ordner, verbunden_at) VALUES (?1, ?2, ?3, ?4)",
            params![
                zone,
                wurzel,
                ordner.to_string_lossy(),
                chrono::Utc::now().to_rfc3339()
            ],
        )?;
        Ok(Some(self.db().last_insert_rowid()))
    }

    /// Trennen. Dateien bleiben, wo sie sind -- im Ordner und in openany.
    pub fn dateiordner_trennen(&self, id: i64) -> Ergebnis<()> {
        self.db()
            .execute("DELETE FROM dateiordner WHERE id = ?1", params![id])?;
        self.db()
            .execute("DELETE FROM dateiordner_stand WHERE paar = ?1", params![id])?;
        Ok(())
    }

    pub fn dateiordner_stand(&self, paar: i64) -> Ergebnis<Vec<Stand>> {
        let db = self.db();
        let mut abfrage = db.prepare(
            "SELECT relpfad, uuid, ist_ordner, abdruck, groesse, mtime
             FROM dateiordner_stand WHERE paar = ?1",
        )?;
        let liste = abfrage
            .query_map(params![paar], |z| {
                Ok(Stand {
                    relpfad: z.get(0)?,
                    uuid: z.get(1)?,
                    ist_ordner: z.get::<_, i64>(2)? != 0,
                    abdruck: z.get(3)?,
                    groesse: z.get::<_, i64>(4)?.max(0) as u64,
                    mtime: z.get(5)?,
                })
            })?
            .collect::<Result<_, _>>()?;
        Ok(liste)
    }

    /// Was in openany im Bereich liegt, nach Pfad vom Bereich an -- ohne
    /// Papierkorb und ohne, was unter eine Ausnahme fällt.
    fn bereich(
        &self,
        paar: &Ordnerpaar,
        ausnahmen: &Ausnahmen,
    ) -> Ergebnis<BTreeMap<String, Datei>> {
        let mut alle = BTreeMap::new();
        let mut offen = vec![(paar.wurzel.clone(), String::new())];
        while let Some((eltern, rel)) = offen.pop() {
            for d in self.dateien_im_ordner(&paar.zone, eltern.as_deref())? {
                let relpfad = if rel.is_empty() {
                    d.name.clone()
                } else {
                    format!("{rel}/{}", d.name)
                };
                if ausnahmen.trifft(&relpfad, d.ist_ordner) {
                    continue;
                }
                if d.ist_ordner {
                    offen.push((Some(d.uuid.clone()), relpfad.clone()));
                }
                alle.insert(relpfad, d);
            }
        }
        Ok(alle)
    }

    /// Ordner und openany gleichziehen, nach dem, was [`scannen`] fand.
    ///
    /// `wegwerfen` legt eine Datei im Ordner in den Papierkorb des Systems,
    /// `mime` nennt den Typ zu einem Dateinamen, `geraet` steht in
    /// Konfliktkopien.
    pub fn dateiordner_abgleichen(
        &self,
        paar: &Ordnerpaar,
        scan: &Scan,
        inhalte: &Inhalte,
        wegwerfen: &dyn Fn(&Path) -> std::io::Result<()>,
        mime: &dyn Fn(&str) -> String,
        geraet: &str,
    ) -> Ergebnis<Bericht> {
        if !paar.ordner.is_dir() {
            return Err(SpeicherFehler::OrdnerFehlt(
                paar.ordner.to_string_lossy().to_string(),
            ));
        }
        if let Some(w) = paar.wurzel.as_deref() {
            if !self
                .datei(w)?
                .is_some_and(|d| d.ist_ordner && d.papierkorb_at.is_none())
            {
                return Err(SpeicherFehler::OrdnerFehlt(w.to_string()));
            }
        }
        let ausnahmen = Ausnahmen::lesen(&paar.ordner);
        let stand = self.dateiordner_stand(paar.id)?;
        let app = self.bereich(paar, &ausnahmen)?;
        let app_uuid: HashMap<&str, (&str, &Datei)> = app
            .iter()
            .map(|(r, d)| (d.uuid.as_str(), (r.as_str(), d)))
            .collect();
        let disk: BTreeMap<&str, &Fund> =
            scan.funde.iter().map(|f| (f.relpfad.as_str(), f)).collect();

        let mut lauf = Lauf {
            speicher: self,
            paar,
            inhalte,
            wegwerfen,
            mime,
            geraet,
            bericht: Bericht::default(),
            ordner: HashMap::from([(String::new(), paar.wurzel.clone())]),
            neu_stand: BTreeMap::new(),
        };
        for (r, d) in &app {
            if d.ist_ordner {
                lauf.ordner.insert(r.clone(), Some(d.uuid.clone()));
            }
        }
        // Was dieser Durchlauf schon erledigt hat -- auf beiden Seiten.
        let mut disk_erledigt: BTreeSet<String> = BTreeSet::new();
        let mut app_erledigt: BTreeSet<String> = BTreeSet::new();
        // Im Ordner verschwunden, in openany unverändert: vielleicht nur
        // umbenannt (dann taucht der Abdruck anderswo neu auf).
        let mut verschwunden: Vec<&Stand> = Vec::new();

        let beschaeftigt = |r: &str| {
            scan.beschaeftigt
                .iter()
                .any(|b| r == b || r.starts_with(&format!("{b}/")))
        };

        // 0. Neu im Ordner, aber am Pfad liegt in openany eine bekannte Datei,
        // die eben dorthin umbenannt wurde: die neue zuerst als Konfliktkopie
        // beiseite, sonst überschriebe das Umbenennen sie.
        let im_stand: BTreeSet<&str> = stand.iter().map(|s| s.relpfad.as_str()).collect();
        let bekannte: BTreeSet<&str> = stand.iter().map(|s| s.uuid.as_str()).collect();
        for f in scan.funde.iter().filter(|f| !f.ist_ordner) {
            if im_stand.contains(f.relpfad.as_str()) || beschaeftigt(&f.relpfad) {
                continue;
            }
            if let Some(a) = app.get(&f.relpfad) {
                if !a.ist_ordner && bekannte.contains(a.uuid.as_str()) {
                    disk_erledigt.insert(f.relpfad.clone());
                    if a.abdruck.as_deref() != Some(f.abdruck.as_str()) {
                        lauf.konflikt(f)?;
                    }
                }
            }
        }

        // 1. Dateien, die beim letzten Mal auf beiden Seiten gleich waren.
        for s in stand.iter().filter(|s| !s.ist_ordner) {
            if beschaeftigt(&s.relpfad) {
                lauf.neu_stand.insert(s.relpfad.clone(), s.clone());
                disk_erledigt.insert(s.relpfad.clone());
                app_erledigt.insert(s.uuid.clone());
                continue;
            }
            let d = disk.get(s.relpfad.as_str()).copied();
            let a = app_uuid.get(s.uuid.as_str()).copied();
            if d.is_some() {
                disk_erledigt.insert(s.relpfad.clone());
            }
            if a.is_some() {
                app_erledigt.insert(s.uuid.clone());
            }
            let disk_neu = d.is_some_and(|d| d.abdruck != s.abdruck);
            let app_neu = a.is_some_and(|(_, a)| a.abdruck.as_deref() != Some(s.abdruck.as_str()));
            let verschoben = a.is_some_and(|(r, _)| r != s.relpfad);
            match (d, a) {
                (None, None) => {}
                (None, Some((r, a))) => {
                    if app_neu {
                        lauf.hinaus(a, r)?;
                    } else {
                        verschwunden.push(s);
                    }
                }
                (Some(d), None) => {
                    if disk_neu {
                        lauf.neu_herein(d)?;
                    } else {
                        lauf.weg(&s.relpfad);
                    }
                }
                (Some(d), Some((r, a))) => {
                    if disk_neu && app_neu {
                        lauf.konflikt(d)?;
                        lauf.hinaus(a, r)?;
                    } else if disk_neu {
                        let mut neu = a.clone();
                        neu.abdruck = Some(d.abdruck.clone());
                        neu.groesse = d.groesse;
                        neu.geaendert_at = String::new();
                        self.datei_schreiben(&neu, Protokoll::Merken)?;
                        lauf.bericht.herein += 1;
                        if verschoben {
                            lauf.umbenennen(&s.relpfad, r)?;
                            let meta = std::fs::metadata(lauf.ziel(r))?;
                            lauf.merken(
                                &Fund {
                                    relpfad: r.to_string(),
                                    mtime: mtime_ms(&meta),
                                    ..d.clone()
                                },
                                &a.uuid,
                            );
                        } else {
                            lauf.merken(d, &a.uuid);
                        }
                    } else if app_neu {
                        if verschoben {
                            lauf.weg(&s.relpfad);
                        }
                        lauf.hinaus(a, r)?;
                    } else if verschoben {
                        lauf.umbenennen(&s.relpfad, r)?;
                        let meta = std::fs::metadata(lauf.ziel(r))?;
                        lauf.merken(
                            &Fund {
                                relpfad: r.to_string(),
                                mtime: mtime_ms(&meta),
                                ..d.clone()
                            },
                            &a.uuid,
                        );
                    } else {
                        lauf.merken(d, &a.uuid);
                    }
                }
            }
        }

        // 2. Neu im Ordner.
        for f in scan.funde.iter().filter(|f| !f.ist_ordner) {
            if disk_erledigt.contains(&f.relpfad) {
                continue;
            }
            // Am selben Pfad liegt in openany schon eine Datei, die der Spiegel
            // noch nicht kennt (erstes Verbinden).
            if let Some(a) = app.get(&f.relpfad).filter(|a| !a.ist_ordner) {
                if !app_erledigt.contains(&a.uuid) {
                    app_erledigt.insert(a.uuid.clone());
                    if a.abdruck.as_deref() == Some(f.abdruck.as_str()) {
                        lauf.merken(f, &a.uuid);
                    } else {
                        lauf.konflikt(f)?;
                        lauf.hinaus(a, &f.relpfad)?;
                    }
                    continue;
                }
            }
            // Umbenannt oder verschoben: derselbe Inhalt verschwand anderswo.
            if let Some(i) = verschwunden.iter().position(|s| s.abdruck == f.abdruck) {
                let s = verschwunden.remove(i);
                let (_, a) = app_uuid[s.uuid.as_str()];
                let mut neu = a.clone();
                neu.eltern = lauf.app_ordner(eltern_von(&f.relpfad))?;
                neu.name = lauf.freier_name(neu.eltern.as_deref(), name_von(&f.relpfad))?;
                neu.geaendert_at = String::new();
                self.datei_schreiben(&neu, Protokoll::Merken)?;
                lauf.bericht.herein += 1;
                lauf.merken(f, &a.uuid);
                continue;
            }
            lauf.neu_herein(f)?;
        }

        // 3. Im Ordner gelöscht, in openany unverändert: Papierkorb.
        for s in verschwunden {
            if self.datei_papierkorb(&s.uuid, Protokoll::Merken)? {
                lauf.bericht.herein += 1;
            }
        }

        // 4. Neu in openany: hinaus.
        for (r, a) in &app {
            if a.ist_ordner || app_erledigt.contains(&a.uuid) || beschaeftigt(r) {
                continue;
            }
            lauf.hinaus(a, r)?;
        }

        // 5. Ordner. Neue im Ordner kommen nach openany, neue in openany in
        // den Ordner. Verschwundene: in openany in den Papierkorb, wenn nichts
        // mehr darin liegt; im Ordner entfernt, wenn er leer ist.
        for f in scan.funde.iter().filter(|f| f.ist_ordner) {
            if !beschaeftigt(&f.relpfad) {
                lauf.app_ordner(&f.relpfad)?;
            }
        }
        for (r, a) in &app {
            if a.ist_ordner && !beschaeftigt(r) {
                std::fs::create_dir_all(lauf.ziel(r))?;
                lauf.neu_stand.entry(r.clone()).or_insert_with(|| Stand {
                    relpfad: r.clone(),
                    uuid: a.uuid.clone(),
                    ist_ordner: true,
                    abdruck: String::new(),
                    groesse: 0,
                    mtime: 0,
                });
            }
        }
        let mut alte_ordner: Vec<&Stand> = stand.iter().filter(|s| s.ist_ordner).collect();
        alte_ordner.sort_by_key(|s| std::cmp::Reverse(s.relpfad.matches('/').count()));
        for s in alte_ordner {
            if beschaeftigt(&s.relpfad) {
                lauf.neu_stand.insert(s.relpfad.clone(), s.clone());
                continue;
            }
            let auf_platte = disk.contains_key(s.relpfad.as_str());
            let in_app = app_uuid.contains_key(s.uuid.as_str());
            if !auf_platte
                && in_app
                && self
                    .dateien_im_ordner(&paar.zone, Some(&s.uuid))?
                    .is_empty()
            {
                self.datei_papierkorb(&s.uuid, Protokoll::Merken)?;
                lauf.bericht.herein += 1;
                lauf.neu_stand.remove(&s.relpfad);
            } else if auf_platte && !in_app && std::fs::remove_dir(lauf.ziel(&s.relpfad)).is_ok() {
                lauf.bericht.hinaus += 1;
                lauf.neu_stand.remove(&s.relpfad);
            }
        }

        // Der neue Stand.
        self.db().execute(
            "DELETE FROM dateiordner_stand WHERE paar = ?1",
            params![paar.id],
        )?;
        for s in lauf.neu_stand.values() {
            self.db().execute(
                "INSERT INTO dateiordner_stand (paar, relpfad, uuid, ist_ordner, abdruck, groesse, mtime)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    paar.id,
                    s.relpfad,
                    s.uuid,
                    s.ist_ordner as i64,
                    s.abdruck,
                    s.groesse as i64,
                    s.mtime
                ],
            )?;
        }
        Ok(lauf.bericht)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    struct Probe {
        _dir: tempfile::TempDir,
        ordner: PathBuf,
        papierkorb: PathBuf,
        inhalte: Inhalte,
        speicher: Speicher,
        paar: Ordnerpaar,
    }

    impl Probe {
        fn neu() -> Self {
            let dir = tempfile::tempdir().unwrap();
            let ordner = dir.path().join("Dokumente");
            let papierkorb = dir.path().join("Papierkorb");
            fs::create_dir_all(&ordner).unwrap();
            fs::create_dir_all(&papierkorb).unwrap();
            let inhalte = Inhalte::oeffnen(dir.path().join("inhalte")).unwrap();
            let speicher = Speicher::im_arbeitsspeicher().unwrap();
            let id = speicher
                .dateiordner_verbinden("documents", None, &ordner)
                .unwrap()
                .unwrap();
            let paar = Ordnerpaar {
                id,
                zone: "documents".into(),
                wurzel: None,
                ordner: ordner.clone(),
            };
            Self {
                _dir: dir,
                ordner,
                papierkorb,
                inhalte,
                speicher,
                paar,
            }
        }

        fn lauf(&self) -> Bericht {
            let stand = self.speicher.dateiordner_stand(self.paar.id).unwrap();
            let scan = scannen(&self.paar, &stand, &self.inhalte, Duration::ZERO).unwrap();
            let korb = self.papierkorb.clone();
            let wegwerfen = move |p: &Path| fs::rename(p, korb.join(p.file_name().unwrap()));
            self.speicher
                .dateiordner_abgleichen(
                    &self.paar,
                    &scan,
                    &self.inhalte,
                    &wegwerfen,
                    &|_| "application/pdf".into(),
                    "Mac",
                )
                .unwrap()
        }

        fn schreiben(&self, rel: &str, text: &str) {
            let p = self.ordner.join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(&p, text).unwrap();
            // Eine andere Änderungszeit, auch wenn der Test schneller ist als
            // die Uhr des Dateisystems.
            let t = SystemTime::now() - Duration::from_secs(60);
            fs::File::options()
                .write(true)
                .open(&p)
                .unwrap()
                .set_modified(t + Duration::from_millis(text.len() as u64))
                .unwrap();
        }

        /// Pfad → Text, wie openany es sieht.
        fn app(&self) -> BTreeMap<String, String> {
            let a = self
                .speicher
                .bereich(&self.paar, &Ausnahmen::aus(std::iter::empty()))
                .unwrap();
            a.into_iter()
                .filter(|(_, d)| !d.ist_ordner)
                .map(|(r, d)| {
                    let p = self.inhalte.pfad(d.abdruck.as_deref().unwrap()).unwrap();
                    (r, fs::read_to_string(p).unwrap())
                })
                .collect()
        }

        /// Pfad → Text, wie der Ordner es hat (ohne Versteckte).
        fn platte(&self) -> BTreeMap<String, String> {
            fn gang(wurzel: &Path, p: &Path, m: &mut BTreeMap<String, String>) {
                for e in fs::read_dir(p).unwrap().flatten() {
                    if e.file_name().to_string_lossy().starts_with('.') {
                        continue;
                    }
                    if e.path().is_dir() {
                        gang(wurzel, &e.path(), m);
                    } else {
                        let r = e
                            .path()
                            .strip_prefix(wurzel)
                            .unwrap()
                            .to_string_lossy()
                            .replace('\\', "/");
                        m.insert(r, fs::read_to_string(e.path()).unwrap());
                    }
                }
            }
            let mut m = BTreeMap::new();
            gang(&self.ordner, &self.ordner, &mut m);
            m
        }

        fn app_datei(&self, rel: &str) -> Datei {
            self.speicher
                .bereich(&self.paar, &Ausnahmen::aus(std::iter::empty()))
                .unwrap()
                .remove(rel)
                .unwrap()
        }

        fn app_neu(&self, eltern: Option<&str>, name: &str, text: &str) -> String {
            let mut l = self.inhalte.ladung().unwrap();
            l.schreiben(text.as_bytes()).unwrap();
            let (abdruck, groesse) = self.inhalte.ablegen(l).unwrap();
            let uuid = uuid::Uuid::new_v4().to_string();
            self.speicher
                .datei_schreiben(
                    &Datei {
                        uuid: uuid.clone(),
                        zone: "documents".into(),
                        eltern: eltern.map(str::to_string),
                        name: name.into(),
                        groesse,
                        abdruck: Some(abdruck),
                        ..Default::default()
                    },
                    Protokoll::Merken,
                )
                .unwrap();
            uuid
        }

        fn app_inhalt(&self, uuid: &str, text: &str) {
            let mut l = self.inhalte.ladung().unwrap();
            l.schreiben(text.as_bytes()).unwrap();
            let (abdruck, groesse) = self.inhalte.ablegen(l).unwrap();
            let mut d = self.speicher.datei(uuid).unwrap().unwrap();
            d.abdruck = Some(abdruck);
            d.groesse = groesse;
            self.speicher
                .datei_schreiben(&d, Protokoll::Merken)
                .unwrap();
        }
    }

    fn karte(paare: &[(&str, &str)]) -> BTreeMap<String, String> {
        paare
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect()
    }

    #[test]
    fn beim_ersten_mal_kommt_alles_herein_und_hinaus() {
        let p = Probe::neu();
        p.schreiben("Steuer/2026/Beleg.pdf", "beleg");
        p.schreiben("Brief.pdf", "brief");
        p.app_neu(None, "Vertrag.pdf", "vertrag");

        let b = p.lauf();
        assert!(b.etwas_getan());
        let alles = karte(&[
            ("Brief.pdf", "brief"),
            ("Steuer/2026/Beleg.pdf", "beleg"),
            ("Vertrag.pdf", "vertrag"),
        ]);
        assert_eq!(p.app(), alles);
        assert_eq!(p.platte(), alles);
        // Unterordner sind Akten geworden.
        assert!(p.app_datei("Steuer").ist_ordner);

        // Ein zweiter Lauf tut nichts.
        assert!(!p.lauf().etwas_getan());
    }

    #[test]
    fn aenderungen_gehen_in_beide_richtungen() {
        let p = Probe::neu();
        p.schreiben("a.pdf", "eins");
        let uuid_b = p.app_neu(None, "b.pdf", "zwei");
        p.lauf();

        p.schreiben("a.pdf", "eins, geändert");
        p.app_inhalt(&uuid_b, "zwei, geändert");
        p.lauf();
        let alles = karte(&[("a.pdf", "eins, geändert"), ("b.pdf", "zwei, geändert")]);
        assert_eq!(p.app(), alles);
        assert_eq!(p.platte(), alles);
    }

    #[test]
    fn beide_seiten_geaendert_gibt_eine_konfliktkopie() {
        let p = Probe::neu();
        p.schreiben("a.pdf", "eins");
        p.lauf();
        let uuid = p.app_datei("a.pdf").uuid;

        p.schreiben("a.pdf", "im Ordner");
        p.app_inhalt(&uuid, "in openany");
        let b = p.lauf();
        assert_eq!(b.konflikte, 1);

        let app = p.app();
        assert_eq!(app["a.pdf"], "in openany");
        let kopie = app.keys().find(|k| k.contains("(Konflikt, Mac, ")).unwrap();
        assert_eq!(app[kopie], "im Ordner");
        assert_eq!(p.platte(), app);
        assert!(!p.lauf().etwas_getan());
    }

    #[test]
    fn umbenennen_im_ordner_verschiebt_in_openany() {
        let p = Probe::neu();
        p.schreiben("a.pdf", "eins");
        p.lauf();
        let uuid = p.app_datei("a.pdf").uuid;

        fs::create_dir_all(p.ordner.join("Akte")).unwrap();
        fs::rename(p.ordner.join("a.pdf"), p.ordner.join("Akte/b.pdf")).unwrap();
        p.lauf();
        assert_eq!(
            p.app_datei("Akte/b.pdf").uuid,
            uuid,
            "dieselbe Datei, nicht neu"
        );
        assert_eq!(p.app(), karte(&[("Akte/b.pdf", "eins")]));
    }

    #[test]
    fn umbenennen_in_openany_benennt_im_ordner_um() {
        let p = Probe::neu();
        p.schreiben("a.pdf", "eins");
        p.lauf();
        let mut d = p.app_datei("a.pdf");
        d.name = "neu.pdf".into();
        p.speicher.datei_schreiben(&d, Protokoll::Merken).unwrap();
        p.lauf();
        assert_eq!(p.platte(), karte(&[("neu.pdf", "eins")]));
    }

    #[test]
    fn umbenennen_in_openany_ueberschreibt_keine_neue_datei_im_ordner() {
        let p = Probe::neu();
        p.schreiben("alt.pdf", "bekannt");
        p.lauf();
        let mut d = p.app_datei("alt.pdf");
        d.name = "a.pdf".into();
        p.speicher.datei_schreiben(&d, Protokoll::Merken).unwrap();
        p.schreiben("a.pdf", "neu im Ordner");

        let b = p.lauf();
        assert_eq!(b.konflikte, 1);
        let platte = p.platte();
        assert_eq!(platte["a.pdf"], "bekannt");
        assert!(platte.values().any(|t| t == "neu im Ordner"));
        assert_eq!(p.app(), platte);
        assert!(!p.lauf().etwas_getan());
    }

    #[test]
    fn loeschen_geht_auf_beiden_seiten_in_den_papierkorb() {
        let p = Probe::neu();
        p.schreiben("weg-im-ordner.pdf", "eins");
        let uuid = p.app_neu(None, "weg-in-openany.pdf", "zwei");
        p.lauf();
        let im_ordner = p.app_datei("weg-im-ordner.pdf").uuid;

        fs::remove_file(p.ordner.join("weg-im-ordner.pdf")).unwrap();
        p.speicher
            .datei_papierkorb(&uuid, Protokoll::Merken)
            .unwrap();
        p.lauf();

        assert!(p.app().is_empty());
        assert!(p.platte().is_empty());
        // Nicht endgültig: in openany im Papierkorb, im Ordner beim System.
        assert!(p
            .speicher
            .datei(&im_ordner)
            .unwrap()
            .unwrap()
            .papierkorb_at
            .is_some());
        assert!(p.papierkorb.join("weg-in-openany.pdf").is_file());
    }

    #[test]
    fn ein_fehlender_ordner_loescht_nichts() {
        let p = Probe::neu();
        p.schreiben("a.pdf", "eins");
        p.lauf();
        fs::rename(&p.ordner, p.ordner.with_file_name("Anderswo")).unwrap();

        let stand = p.speicher.dateiordner_stand(p.paar.id).unwrap();
        assert!(matches!(
            scannen(&p.paar, &stand, &p.inhalte, Duration::ZERO),
            Err(SpeicherFehler::OrdnerFehlt(_))
        ));
        assert_eq!(p.app(), karte(&[("a.pdf", "eins")]));
    }

    #[test]
    fn ausnahmen_bleiben_draussen() {
        let p = Probe::neu();
        fs::write(p.ordner.join(".openanyignore"), "*.bak\nAlt/\n").unwrap();
        p.schreiben("a.pdf", "eins");
        p.schreiben("a.pdf.bak", "alt");
        p.schreiben("~$Brief.docx", "sperre");
        p.schreiben("Alt/x.pdf", "x");
        p.schreiben("Thumbs.db", "x");
        p.lauf();
        assert_eq!(p.app(), karte(&[("a.pdf", "eins")]));
    }

    #[test]
    fn was_gerade_geschrieben_wird_wartet() {
        let p = Probe::neu();
        p.schreiben("a.pdf", "eins");
        p.lauf();
        // Frisch geändert: in diesem Lauf unberührt, nicht für gelöscht gehalten.
        fs::write(p.ordner.join("a.pdf"), "halb").unwrap();
        let stand = p.speicher.dateiordner_stand(p.paar.id).unwrap();
        let scan = scannen(&p.paar, &stand, &p.inhalte, Duration::from_secs(30)).unwrap();
        assert!(scan.beschaeftigt.contains("a.pdf"));
        let b = p
            .speicher
            .dateiordner_abgleichen(
                &p.paar,
                &scan,
                &p.inhalte,
                &|_| Ok(()),
                &|_| String::new(),
                "Mac",
            )
            .unwrap();
        assert!(!b.etwas_getan());
        assert_eq!(p.app(), karte(&[("a.pdf", "eins")]));
    }

    #[test]
    fn verbinden_verweigert_ueberlappende_ordner() {
        let p = Probe::neu();
        let s = &p.speicher;
        assert!(s
            .dateiordner_verbinden("documents", None, &p.ordner.join("Unter"))
            .unwrap()
            .is_none());
        assert!(s
            .dateiordner_verbinden("files", None, p.ordner.parent().unwrap())
            .unwrap()
            .is_none());
        assert!(s
            .dateiordner_verbinden("files", None, &p.papierkorb)
            .unwrap()
            .is_some());
    }

    #[test]
    fn konfliktnamen_behalten_die_endung() {
        let t = chrono::Local::now();
        let n = konfliktname("Rechnung.pdf", "Mac", t);
        assert!(n.starts_with("Rechnung (Konflikt, Mac, ") && n.ends_with(").pdf"));
        assert!(konfliktname("LIESMICH", "Mac", t).starts_with("LIESMICH (Konflikt"));
    }
}
