//! Bilder im Notizordner (docs/plan-dateien-e2e.md, 7c-3). Nur in der
//! Desktop-App, nur fuer eigene Notizen.
//!
//! **Neben den Notizen** (Tiffy, 20.10.2026): Ein Bild liegt genau dort,
//! wohin der Text der Notiz zeigt -- `![](bild.png)` neben der `.md`-Datei,
//! im Unterordner ihrer Mappe. So zeigt Zettlr es ohne Umweg, und kein Link
//! wird umgeschrieben.
//!
//! **Die App ist die Quelle**, wie bei den Notizen ein Spiegel mit
//! Gedaechtnis (`notizordner_bilder`: Abdruck, Groesse, Zeit beim letzten
//! Mal). Daraus folgt je Bild:
//!
//! | draussen             | in der App   | was geschieht                         |
//! |----------------------|--------------|---------------------------------------|
//! | fehlt                | --           | Bild schreiben (liegt es hier)        |
//! | gleich               | gleich       | nichts                                |
//! | gleich               | geaendert    | Bild neu schreiben                    |
//! | geaendert            | gleich       | Inhalt fuer den Anhang uebernehmen    |
//! | geaendert            | geaendert    | App gewinnt, draussen bleibt als Kopie|
//! | neu, im Text genannt | --           | neuer Anhang (Galerie, wie in der App)|
//! | Anhang fort          | --           | Datei fort, wenn draussen unveraendert|
//!
//! Draussen geloeschte Bilder kommen wieder: Solange die Notiz auf sie zeigt,
//! gehoeren sie dazu. Neue und geaenderte Bilder legt dieser Teil nicht
//! selbst an -- dafuer braucht es Vorschau und Galerie, also die App
//! (`notizordnerbefehle.rs`); hier steht nur, WAS zu tun ist.

use crate::notizordner::{abdruck, liegt_in, mtime, pfad_von, unterordner};
use crate::{Anhang, Ergebnis, Inhalte, Speicher, Verbindung};
use rusqlite::params;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Endungen, die als Bild zurueckgelesen werden.
const BILDENDUNGEN: [&str; 6] = ["png", "jpg", "jpeg", "gif", "webp", "svg"];

/// Ein Bild im Ordner, das die App uebernehmen soll.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bildfund {
    /// Relativ zum verbundenen Ordner.
    pub relpfad: String,
    /// Die Mappe, in deren Unterordner es liegt.
    pub mappe: String,
    /// Der Name, unter dem die Notiz es nennt.
    pub name: String,
    /// Wo es auf diesem Computer liegt.
    pub datei: PathBuf,
}

/// Was ein Durchlauf ueber Bilder ergab.
#[derive(Debug, Clone, Default)]
pub struct Bilderlauf {
    /// Bilder hinausgeschrieben oder fortgeraeumt.
    pub geschrieben: usize,
    /// Neue Bilder, die eine Notiz nennt -- werden Anhaenge.
    pub neu: Vec<Bildfund>,
    /// Draussen geaendert -- der Anhang bekommt den neuen Inhalt.
    pub geaendert: Vec<(Anhang, Bildfund)>,
}

#[derive(Debug, Clone)]
struct Stand {
    abdruck: String,
    groesse: i64,
    mtime: i64,
}

pub fn ist_bild(name: &str) -> bool {
    let endung = name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    name.contains('.') && BILDENDUNGEN.contains(&endung.as_str())
}

/// Nennt dieser Text das Bild `name` (Markdown-Link oder Wikilink)?
fn nennt(text: &str, name: &str) -> bool {
    let kodiert = name.replace(' ', "%20");
    [
        format!("]({name})"),
        format!("](<{name}>)"),
        format!("](./{name})"),
        format!("]({kodiert})"),
        format!("](./{kodiert})"),
        format!("[[{name}]]"),
        format!("[[{name}|"),
    ]
    .iter()
    .any(|m| text.contains(m.as_str()))
}

/// Ein Pfad aus dem Text, der im Ordner bleibt (kein `..`, nicht absolut).
fn harmlos(pfad: &str) -> bool {
    !pfad.is_empty()
        && !pfad.starts_with('/')
        && !pfad.contains('\\')
        && pfad
            .split('/')
            .all(|t| !t.is_empty() && t != "." && t != "..")
}

/// Erst daneben schreiben, dann umbenennen -- Zettlr sieht nie ein halbes Bild.
fn kopieren(von: &Path, ziel: &Path) -> std::io::Result<()> {
    if let Some(e) = ziel.parent() {
        std::fs::create_dir_all(e)?;
    }
    let tmp = ziel.with_extension("openany-tmp");
    std::fs::copy(von, &tmp)?;
    std::fs::rename(&tmp, ziel)
}

/// `bild.png` → `bild (Konflikt Ordner 2026-10-20).png`
fn konfliktname(pfad: &Path) -> PathBuf {
    let stamm = pfad
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let endung = pfad
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    pfad.with_file_name(format!(
        "{stamm} (Konflikt Ordner {}){endung}",
        chrono::Local::now().format("%Y-%m-%d")
    ))
}

impl Speicher {
    fn bildstaende(&self, mappe: &str) -> Ergebnis<BTreeMap<String, Stand>> {
        let db = self.db();
        let mut a = db.prepare(
            "SELECT relpfad, abdruck, groesse, mtime FROM notizordner_bilder WHERE mappe = ?1",
        )?;
        let liste = a
            .query_map(params![mappe], |z| {
                Ok((
                    z.get::<_, String>(0)?,
                    Stand {
                        abdruck: z.get(1)?,
                        groesse: z.get(2)?,
                        mtime: z.get(3)?,
                    },
                ))
            })?
            .collect::<Result<_, _>>()?;
        Ok(liste)
    }

    /// Vermerken, wie ein Bild im Ordner jetzt aussieht -- nach dem Schreiben
    /// oder Uebernehmen (auch aus der App: `notizordnerbefehle.rs`).
    pub fn notizordner_bild_merken(
        &self,
        v: &Verbindung,
        relpfad: &str,
        abdruck: &str,
    ) -> Ergebnis<()> {
        let meta = std::fs::metadata(pfad_von(&v.ordner, relpfad))?;
        self.db().execute(
            "INSERT INTO notizordner_bilder (mappe, relpfad, abdruck, groesse, mtime)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT (mappe, relpfad) DO UPDATE SET abdruck = excluded.abdruck,
                groesse = excluded.groesse, mtime = excluded.mtime",
            params![v.mappe, relpfad, abdruck, meta.len() as i64, mtime(&meta)],
        )?;
        Ok(())
    }

    fn bild_vergessen(&self, v: &Verbindung, relpfad: &str) -> Ergebnis<()> {
        self.db().execute(
            "DELETE FROM notizordner_bilder WHERE mappe = ?1 AND relpfad = ?2",
            params![v.mappe, relpfad],
        )?;
        Ok(())
    }

    /// Ein Durchlauf fuer die Bilder einer Verbindung -- nach dem der Notizen
    /// (dann stehen ihre Mappen und Texte schon).
    pub fn notizordner_bilder(&self, v: &Verbindung, inhalte: &Inhalte) -> Ergebnis<Bilderlauf> {
        let mut lauf = Bilderlauf::default();
        if !v.ordner.is_dir() {
            return Ok(lauf);
        }
        let staende = self.bildstaende(&v.mappe)?;
        let relpfad_von = |a: &Anhang| -> String {
            let o = unterordner(&v.mappe, &a.mappe);
            if o.is_empty() {
                a.pfad.clone()
            } else {
                format!("{o}/{}", a.pfad)
            }
        };

        // 1. Was die App kennt: die Bilder der Notizen in der Mappe.
        let mut bekannt: BTreeSet<String> = BTreeSet::new();
        for a in self.anhaenge_alle()? {
            if a.ziel_art != "media" || !liegt_in(Some(&a.mappe), &v.mappe) || !harmlos(&a.pfad) {
                continue;
            }
            let rel = relpfad_von(&a);
            bekannt.insert(rel.clone());
            let datei = pfad_von(&v.ordner, &rel);
            let hier = a
                .abdruck
                .as_deref()
                .and_then(|x| inhalte.pfad(x))
                .filter(|p| p.is_file());
            let stand = staende.get(&rel);

            let Ok(meta) = std::fs::metadata(&datei) else {
                // Draussen fehlt es: schreiben, wenn es hier liegt.
                if let (Some(quelle), Some(x)) = (&hier, &a.abdruck) {
                    kopieren(quelle, &datei)?;
                    self.notizordner_bild_merken(v, &rel, x)?;
                    lauf.geschrieben += 1;
                }
                continue;
            };
            let unberuehrt =
                stand.is_some_and(|s| s.groesse == meta.len() as i64 && s.mtime == mtime(&meta));
            let draussen = if unberuehrt {
                stand.map(|s| s.abdruck.clone()).unwrap_or_default()
            } else {
                abdruck(&std::fs::read(&datei)?)
            };
            let in_der_app = a.abdruck.clone().unwrap_or_default();
            if draussen == in_der_app {
                if !unberuehrt {
                    self.notizordner_bild_merken(v, &rel, &draussen)?;
                }
                continue;
            }
            let draussen_geaendert = stand.is_none_or(|s| s.abdruck != draussen);
            let app_geaendert = stand.is_none_or(|s| s.abdruck != in_der_app);
            if draussen_geaendert && !app_geaendert {
                lauf.geaendert.push((
                    a.clone(),
                    Bildfund {
                        relpfad: rel.clone(),
                        mappe: a.mappe.clone(),
                        name: a.pfad.clone(),
                        datei: datei.clone(),
                    },
                ));
                continue;
            }
            // In der App geaendert (oder beide): die App gewinnt. Was draussen
            // anders war, bleibt als Kopie daneben.
            let Some(quelle) = hier else {
                continue;
            };
            if draussen_geaendert {
                std::fs::rename(&datei, konfliktname(&datei))?;
            }
            kopieren(&quelle, &datei)?;
            self.notizordner_bild_merken(v, &rel, &in_der_app)?;
            lauf.geschrieben += 1;
        }

        // 2. Vermerkt, aber kein Anhang mehr (in der App entfernt): fort,
        //    wenn draussen unveraendert.
        for (rel, s) in &staende {
            if bekannt.contains(rel) {
                continue;
            }
            let datei = pfad_von(&v.ordner, rel);
            if let Ok(meta) = std::fs::metadata(&datei) {
                if s.groesse == meta.len() as i64 && s.mtime == mtime(&meta) {
                    std::fs::remove_file(&datei)?;
                    lauf.geschrieben += 1;
                }
            }
            self.bild_vergessen(v, rel)?;
        }

        // 3. Neue Bilder im Ordner, die eine Notiz daneben nennt.
        let mut zu_mappe: BTreeMap<String, String> = BTreeMap::new();
        zu_mappe.insert(String::new(), v.mappe.clone());
        for m in self.mappen()? {
            if liegt_in(Some(&m.pfad), &v.mappe) {
                zu_mappe.insert(unterordner(&v.mappe, &m.pfad), m.pfad);
            }
        }
        let notizen: Vec<_> = self
            .notizen()?
            .into_iter()
            .filter(|n| !n.im_papierkorb())
            .collect();
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
                if e.file_type()?.is_dir() {
                    stapel.push((e.path(), r));
                    continue;
                }
                if !ist_bild(&name) || bekannt.contains(&r) {
                    continue;
                }
                let Some(mappe) = zu_mappe.get(&rel) else {
                    continue;
                };
                let genannt = notizen
                    .iter()
                    .any(|n| n.mappe.as_deref() == Some(mappe) && nennt(&n.inhalt, &name));
                if genannt {
                    lauf.neu.push(Bildfund {
                        relpfad: r,
                        mappe: mappe.clone(),
                        name,
                        datei: e.path(),
                    });
                }
            }
        }
        Ok(lauf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Bild, Notiz, Protokoll};

    fn aufbau() -> (
        Speicher,
        Inhalte,
        tempfile::TempDir,
        tempfile::TempDir,
        Verbindung,
    ) {
        let s = Speicher::im_arbeitsspeicher().unwrap();
        let ablage = tempfile::tempdir().unwrap();
        let inhalte = Inhalte::oeffnen(ablage.path()).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let v = Verbindung {
            mappe: "Uni".into(),
            ordner: dir.path().to_path_buf(),
        };
        assert!(s.notizordner_verbinden(&v.mappe, &v.ordner).unwrap());
        (s, inhalte, ablage, dir, v)
    }

    fn ablegen(i: &Inhalte, bytes: &[u8]) -> String {
        let mut l = i.ladung().unwrap();
        l.schreiben(bytes).unwrap();
        i.ablegen(l).unwrap().0
    }

    fn anhang(s: &Speicher, mappe: &str, pfad: &str, abdruck: &str) -> Anhang {
        s.bild_schreiben(
            &Bild {
                uuid: format!("b-{pfad}"),
                name: pfad.into(),
                abdruck: Some(abdruck.into()),
                ..Default::default()
            },
            Protokoll::Still,
        )
        .unwrap();
        let a = Anhang {
            mappe: mappe.into(),
            pfad: pfad.into(),
            ziel_art: "media".into(),
            ziel_uuid: format!("b-{pfad}"),
            groesse: 1,
            abdruck: Some(abdruck.into()),
            vorschau: None,
        };
        s.anhang_schreiben(&a, Protokoll::Still).unwrap();
        a
    }

    fn spaeter() {
        std::thread::sleep(std::time::Duration::from_millis(15));
    }

    #[test]
    fn bilder_liegen_neben_ihrer_notiz() {
        let (s, i, _a, _d, v) = aufbau();
        let x = ablegen(&i, b"PNG-Beispiel");
        anhang(&s, "Uni", "bild.png", &x);
        anhang(&s, "Uni/Kapitel 1", "skizze.png", &x);

        let lauf = s.notizordner_bilder(&v, &i).unwrap();
        assert_eq!(lauf.geschrieben, 2);
        assert_eq!(
            std::fs::read(v.ordner.join("bild.png")).unwrap(),
            b"PNG-Beispiel"
        );
        assert!(v.ordner.join("Kapitel 1").join("skizze.png").is_file());

        // Nichts geaendert: nichts zu tun.
        let lauf = s.notizordner_bilder(&v, &i).unwrap();
        assert_eq!(lauf.geschrieben, 0);
        assert!(lauf.neu.is_empty() && lauf.geaendert.is_empty());

        // Draussen geloescht: kommt wieder.
        std::fs::remove_file(v.ordner.join("bild.png")).unwrap();
        assert_eq!(s.notizordner_bilder(&v, &i).unwrap().geschrieben, 1);
        assert!(v.ordner.join("bild.png").is_file());
    }

    #[test]
    fn draussen_geaendert_wird_uebernommen_beide_geaendert_gewinnt_die_app() {
        let (s, i, _a, _d, v) = aufbau();
        let x = ablegen(&i, b"PNG-alt");
        let a = anhang(&s, "Uni", "bild.png", &x);
        s.notizordner_bilder(&v, &i).unwrap();

        spaeter();
        std::fs::write(v.ordner.join("bild.png"), b"PNG-draussen").unwrap();
        let lauf = s.notizordner_bilder(&v, &i).unwrap();
        assert_eq!(lauf.geaendert.len(), 1);
        assert_eq!(lauf.geaendert[0].0.pfad, "bild.png");

        // Jetzt aendert sich auch die App: Sie gewinnt, draussen bleibt als Kopie.
        let y = ablegen(&i, b"PNG-app");
        s.anhang_schreiben(
            &Anhang {
                abdruck: Some(y),
                ..a
            },
            Protokoll::Still,
        )
        .unwrap();
        let lauf = s.notizordner_bilder(&v, &i).unwrap();
        assert_eq!(lauf.geschrieben, 1);
        assert_eq!(
            std::fs::read(v.ordner.join("bild.png")).unwrap(),
            b"PNG-app"
        );
        let kopien: Vec<_> = std::fs::read_dir(&v.ordner)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|n| n.contains("Konflikt"))
            .collect();
        assert_eq!(kopien.len(), 1);
    }

    #[test]
    fn neue_bilder_nur_wenn_eine_notiz_sie_nennt() {
        let (s, i, _a, _d, v) = aufbau();
        s.notiz_schreiben(
            &Notiz {
                zk_id: "20261020120000".into(),
                titel: "Beispiel".into(),
                inhalt: "Text ![](neu%20bild.png) und mehr".into(),
                mappe: Some("Uni".into()),
                papierkorb_at: None,
                geaendert_at: String::new(),
            },
            Protokoll::Still,
        )
        .unwrap();
        std::fs::write(v.ordner.join("neu bild.png"), b"PNG").unwrap();
        std::fs::write(v.ordner.join("fremd.png"), b"PNG").unwrap();
        std::fs::write(v.ordner.join("text.txt"), b"x").unwrap();

        let lauf = s.notizordner_bilder(&v, &i).unwrap();
        assert_eq!(lauf.neu.len(), 1);
        assert_eq!(lauf.neu[0].name, "neu bild.png");
        assert_eq!(lauf.neu[0].mappe, "Uni");
    }

    #[test]
    fn ein_entfernter_anhang_raeumt_seine_datei_fort() {
        let (s, i, _a, _d, v) = aufbau();
        let x = ablegen(&i, b"PNG");
        anhang(&s, "Uni", "bild.png", &x);
        s.notizordner_bilder(&v, &i).unwrap();
        s.anhang_entfernen("Uni", "bild.png", Protokoll::Still)
            .unwrap();
        assert_eq!(s.notizordner_bilder(&v, &i).unwrap().geschrieben, 1);
        assert!(!v.ordner.join("bild.png").exists());
    }

    #[test]
    fn nur_harmlose_pfade() {
        assert!(harmlos("bild.png"));
        assert!(harmlos("bilder/bild.png"));
        assert!(!harmlos("../bild.png"));
        assert!(!harmlos("/etc/bild.png"));
        assert!(!harmlos("a//b.png"));
        assert!(ist_bild("x.JPG"));
        assert!(!ist_bild("png"));
    }
}
