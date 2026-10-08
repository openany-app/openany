//! Das Notizbuch: was die Notizen-Arbeitsflaeche aus dem Speicher braucht.
//!
//! Die Arbeitsflaeche (`packages/oberflaeche/notizen/NotesWorkspace.vue`) ist
//! dieselbe wie in der Webapp. Sie fragt nach Mappen mit Ids, nach einer
//! Seite Notizen, nach Titeln, Tags und Rueckverweisen -- all das, was drueben
//! eigene Tabellen und Endpunkte sind. Hier wird es aus den Notizen gerechnet.
//!
//! **Mappen sind Pfade.** Ueber die Leitung geht `Uni/Hausarbeit`, keine Id;
//! so steht es auch in `notizen.mappe`. Die Arbeitsflaeche braucht aber Zahlen
//! (`{id, name, parent_id}`, strikt verglichen). Die Id ist deshalb ein
//! Abdruck des Pfades ([`mappen_id`]): gleich fuer denselben Pfad, bei jedem
//! Start, ohne eine zweite Buchfuehrung, die mit den Pfaden auseinanderliefe.
//!
//! **Leere Mappen** haetten sonst keinen Ort, denn eine Mappe entsteht aus den
//! Notizen darin. Sie stehen in `mappen` -- nur hier, nicht im Abgleich:
//! Drueben entsteht die Mappe, sobald die erste Notiz darin ankommt.
//!
//! Alles wird im Speicher gerechnet, nicht in SQL. Ein persoenliches
//! Notizbuch hat Hunderte Notizen, keine Millionen, und die Regeln fuer Tags
//! und Verweise stehen so an EINER Stelle ([`crate::verweise`]).

use crate::verweise;
use crate::{Ergebnis, Notiz, Protokoll, Speicher};
use openany_client::Art;
use std::collections::{BTreeMap, BTreeSet};

/// Eine Mappe, wie die Arbeitsflaeche sie sieht.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mappe {
    pub id: u32,
    pub name: String,
    pub pfad: String,
    pub eltern_id: Option<u32>,
}

/// Ein Knoten im Notiz-Graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graphknoten {
    pub id: String,
    pub titel: String,
    /// Die eigene Mappe (Id wie in `mappen`), `None` = Wurzel.
    pub mappe: Option<u32>,
    /// Die oberste Mappe -- Ebene und Farbe im Graphen.
    pub gruppe: Option<u32>,
    pub gruppenname: Option<String>,
}

/// Der ganze Notiz-Graph (`notizgraph`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Notizgraph {
    pub knoten: Vec<Graphknoten>,
    /// (von, nach) als `zk_id`.
    pub kanten: Vec<(String, String)>,
    /// (Tag, Zahl der Notizen).
    pub tags: Vec<(String, usize)>,
    /// (Notiz, Tag).
    pub tag_kanten: Vec<(String, String)>,
}

/// Was die Liste zeigen soll.
#[derive(Debug, Clone, Default)]
pub struct Filter {
    /// `None` = Wurzel.
    pub mappe: Option<String>,
    /// Ohne ausdrueckliche Mappe bei Suche/Tag ueber ALLE Mappen -- dieselbe
    /// Regel wie drueben (`NoteController::index`).
    pub ueberall: bool,
    pub tag: Option<String>,
    pub suche: Option<String>,
}

/// Eine Seite der Liste.
#[derive(Debug, Clone)]
pub struct Seite {
    pub notizen: Vec<Notiz>,
    pub weitere: bool,
}

/// Wie viele Notizen eine Seite traegt -- wie drueben (`PaginatesResults`).
pub const PRO_SEITE: usize = 50;

/// Die Id einer Mappe: FNV-1a (32 Bit) ueber den Pfad, nie 0.
///
/// 32 und nicht 64 Bit, weil die Zahl in JavaScript landet und dort oberhalb
/// von 2^53 still gerundet wuerde -- zwei Mappen mit derselben Id waeren die
/// Folge, ohne Fehler.
pub fn mappen_id(pfad: &str) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    for b in pfad.as_bytes() {
        h ^= u32::from(*b);
        h = h.wrapping_mul(0x0100_0193);
    }
    h.max(1)
}

fn eltern(pfad: &str) -> Option<&str> {
    pfad.rsplit_once('/').map(|(e, _)| e)
}

fn liegt_in(mappe: Option<&str>, wurzel: &str) -> bool {
    mappe.is_some_and(|m| m == wurzel || m.starts_with(&format!("{wurzel}/")))
}

impl Speicher {
    /// Alle Mappen: aus den Pfaden der Notizen (samt aller Zwischenstufen) und
    /// den leer angelegten.
    pub fn mappen(&self) -> Ergebnis<Vec<Mappe>> {
        let mut pfade = BTreeSet::new();

        for n in self.notizen()? {
            if let Some(m) = n.mappe.filter(|m| !m.is_empty()) {
                pfade.insert(m);
            }
        }
        for p in self.leere_mappen()? {
            pfade.insert(p);
        }

        // Zwischenstufen: `a/b/c` ergibt auch `a` und `a/b`.
        for p in pfade.clone() {
            let mut rest = p.as_str();
            while let Some(e) = eltern(rest) {
                pfade.insert(e.to_string());
                rest = e;
            }
        }

        Ok(pfade
            .into_iter()
            .map(|pfad| Mappe {
                id: mappen_id(&pfad),
                name: pfad.rsplit('/').next().unwrap_or(&pfad).to_string(),
                eltern_id: eltern(&pfad).map(mappen_id),
                pfad,
            })
            .collect())
    }

    /// Der Pfad zu einer Mappen-Id -- `None`, wenn es sie nicht (mehr) gibt.
    pub fn mappen_pfad(&self, id: u32) -> Ergebnis<Option<String>> {
        Ok(self
            .mappen()?
            .into_iter()
            .find(|m| m.id == id)
            .map(|m| m.pfad))
    }

    fn leere_mappen(&self) -> Ergebnis<Vec<String>> {
        let db = self.db();
        let mut abfrage = db.prepare("SELECT pfad FROM mappen ORDER BY pfad")?;
        let liste = abfrage
            .query_map([], |z| z.get::<_, String>(0))?
            .collect::<Result<_, _>>()?;
        Ok(liste)
    }

    /// Eine Mappe anlegen. `/` im Namen ist verboten, er trennt die Stufen.
    pub fn mappe_anlegen(&self, eltern: Option<&str>, name: &str) -> Ergebnis<String> {
        let name = name.trim().replace('/', "-");
        let pfad = match eltern {
            Some(e) if !e.is_empty() => format!("{e}/{name}"),
            _ => name,
        };
        self.db().execute(
            "INSERT OR IGNORE INTO mappen (pfad, angelegt_at) VALUES (?1, ?2)",
            rusqlite::params![pfad, chrono::Utc::now().to_rfc3339()],
        )?;
        Ok(pfad)
    }

    /// Umbenennen: Jede Notiz darin bekommt den neuen Pfad -- und ins
    /// Protokoll, damit der naechste Abgleich die Mappe drueben mitnimmt.
    pub fn mappe_umbenennen(&self, pfad: &str, name: &str) -> Ergebnis<String> {
        let name = name.trim().replace('/', "-");
        let neu = match eltern(pfad) {
            Some(e) => format!("{e}/{name}"),
            None => name,
        };
        if neu == pfad {
            return Ok(neu);
        }

        for mut n in self.notizen_mit_papierkorb()? {
            if liegt_in(n.mappe.as_deref(), pfad) {
                let rest = &n.mappe.as_deref().unwrap_or("")[pfad.len()..];
                n.mappe = Some(format!("{neu}{rest}"));
                n.geaendert_at = String::new();
                self.notiz_schreiben(&n, Protokoll::Merken)?;
            }
        }

        let alte: Vec<String> = self
            .leere_mappen()?
            .into_iter()
            .filter(|p| liegt_in(Some(p), pfad))
            .collect();
        for p in alte {
            let umbenannt = format!("{neu}{}", &p[pfad.len()..]);
            self.db()
                .execute("DELETE FROM mappen WHERE pfad = ?1", [&p])?;
            self.db().execute(
                "INSERT OR IGNORE INTO mappen (pfad, angelegt_at) VALUES (?1, ?2)",
                rusqlite::params![umbenannt, chrono::Utc::now().to_rfc3339()],
            )?;
        }
        Ok(neu)
    }

    /// Loeschen, wie drueben: `delete_notes` legt alle Notizen darunter in den
    /// Papierkorb, `move_to_parent` haengt sie eine Stufe hoeher.
    pub fn mappe_loeschen(&self, pfad: &str, notizen_loeschen: bool) -> Ergebnis<()> {
        let ziel = eltern(pfad).map(str::to_string);

        for mut n in self.notizen()? {
            if !liegt_in(n.mappe.as_deref(), pfad) {
                continue;
            }
            if notizen_loeschen {
                self.papierkorb(&Art::Notiz, &n.zk_id, Protokoll::Merken)?;
            } else {
                n.mappe = ziel.clone();
                n.geaendert_at = String::new();
                self.notiz_schreiben(&n, Protokoll::Merken)?;
            }
        }

        self.db().execute(
            "DELETE FROM mappen WHERE pfad = ?1 OR pfad LIKE ?2 ESCAPE '\\'",
            rusqlite::params![
                pfad,
                format!(
                    "{}/%",
                    pfad.replace('\\', "\\\\")
                        .replace('%', "\\%")
                        .replace('_', "\\_")
                )
            ],
        )?;
        Ok(())
    }

    fn notizen_mit_papierkorb(&self) -> Ergebnis<Vec<Notiz>> {
        let db = self.db();
        let mut abfrage = db.prepare(
            "SELECT zk_id, titel, inhalt, mappe, papierkorb_at, geaendert_at FROM notizen",
        )?;
        let liste = abfrage
            .query_map([], |z| {
                Ok(Notiz {
                    zk_id: z.get(0)?,
                    titel: z.get(1)?,
                    inhalt: z.get(2)?,
                    mappe: z.get(3)?,
                    papierkorb_at: z.get(4)?,
                    geaendert_at: z.get(5)?,
                })
            })?
            .collect::<Result<_, _>>()?;
        Ok(liste)
    }

    /// Eine Seite der Liste -- alphabetisch nach Titel, wie drueben.
    pub fn notizliste(&self, filter: &Filter, seite: usize) -> Ergebnis<Seite> {
        let suche = filter
            .suche
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_lowercase);
        let tag = filter.tag.as_deref().map(str::to_lowercase);
        let ueber_alle =
            filter.mappe.is_none() && (filter.ueberall || suche.is_some() || tag.is_some());

        let mut treffer: Vec<Notiz> = self
            .notizen()?
            .into_iter()
            .filter(|n| {
                ueber_alle
                    || match &filter.mappe {
                        None => n.mappe.as_deref().is_none_or(str::is_empty),
                        Some(m) => n.mappe.as_deref() == Some(m.as_str()),
                    }
            })
            .filter(|n| {
                suche.as_ref().is_none_or(|s| {
                    n.titel.to_lowercase().contains(s) || n.inhalt.to_lowercase().contains(s)
                })
            })
            .filter(|n| {
                // Verschachtelt: #projekt trifft auch #projekt/unterthema.
                tag.as_ref().is_none_or(|t| {
                    verweise::tags(&n.inhalt)
                        .iter()
                        .any(|x| x == t || x.starts_with(&format!("{t}/")))
                })
            })
            .collect();

        treffer.sort_by_key(|n| n.titel.to_lowercase());

        let seite = seite.max(1);
        let anfang = (seite - 1) * PRO_SEITE;
        let weitere = treffer.len() > anfang + PRO_SEITE;
        let notizen = treffer.into_iter().skip(anfang).take(PRO_SEITE).collect();

        Ok(Seite { notizen, weitere })
    }

    /// Alle Titel (fuer `[[`-Vorschlaege), alphabetisch, ohne leere.
    pub fn notiztitel(&self) -> Ergebnis<Vec<(String, String)>> {
        let mut liste: Vec<(String, String)> = self
            .notizen()?
            .into_iter()
            .filter(|n| !n.titel.trim().is_empty())
            .map(|n| (n.zk_id, n.titel))
            .collect();
        liste.sort_by_key(|x| x.1.to_lowercase());
        Ok(liste)
    }

    /// Wohin ein `[[Verweis]]` zeigt. Mit Kennung ueber die `zk_id`, sonst
    /// ueber den Titel (Gross/Klein egal); bei doppelten Titeln gewinnt die
    /// kleinste Kennung -- dieselbe Wahl wie drueben (`scopeForLinkTarget`).
    pub fn verweisziel(&self, ziel: &str) -> Ergebnis<Option<Notiz>> {
        if let Some(id) = verweise::ziel_id(ziel) {
            return self.notiz(id).map(|n| n.filter(|n| !n.im_papierkorb()));
        }

        let gesucht = ziel.trim().to_lowercase();
        let mut kandidaten: Vec<Notiz> = self
            .notizen()?
            .into_iter()
            .filter(|n| n.titel.to_lowercase() == gesucht)
            .collect();
        kandidaten.sort_by(|a, b| a.zk_id.cmp(&b.zk_id));
        Ok(kandidaten.into_iter().next())
    }

    /// Alle Tags mit ihrer Haeufigkeit, alphabetisch.
    pub fn notiztags(&self) -> Ergebnis<Vec<(String, usize)>> {
        let mut zaehler: BTreeMap<String, usize> = BTreeMap::new();
        for n in self.notizen()? {
            for t in verweise::tags(&n.inhalt) {
                *zaehler.entry(t).or_default() += 1;
            }
        }
        Ok(zaehler.into_iter().collect())
    }

    /// Der Notiz-Graph (30.09.2026): Knoten, Verweise zwischen Notizen, Tags
    /// und wer welchen Tag traegt -- in der Form, die drueben
    /// `NoteGraphService::build` liefert, damit dieselbe Ansicht beides
    /// zeichnet. `gruppe` ist die oberste Mappe (ihr Pfadanfang), `None` fuer
    /// Notizen der obersten Ebene.
    ///
    /// Verweise loesen sich wie `verweisziel` auf: mit Kennung ueber die
    /// `zk_id`, sonst ueber den Titel (Gross/Klein egal, kleinste Kennung
    /// gewinnt). Kanten auf sich selbst und doppelte fallen weg.
    pub fn notizgraph(&self) -> Ergebnis<Notizgraph> {
        let alle = self.notizen()?;
        let mut nach_titel: BTreeMap<String, &str> = BTreeMap::new();
        let ids: BTreeSet<&str> = alle.iter().map(|n| n.zk_id.as_str()).collect();
        for n in &alle {
            let t = n.titel.trim().to_lowercase();
            if t.is_empty() {
                continue;
            }
            let e = nach_titel.entry(t).or_insert(n.zk_id.as_str());
            if n.zk_id.as_str() < *e {
                *e = n.zk_id.as_str();
            }
        }

        let mut knoten = Vec::new();
        let mut kanten = Vec::new();
        let mut gesehen = BTreeSet::new();
        let mut tag_kanten = Vec::new();
        let mut tag_zahl: BTreeMap<String, usize> = BTreeMap::new();
        for n in &alle {
            let oberste = n
                .mappe
                .as_deref()
                .and_then(|m| m.split('/').next())
                .filter(|m| !m.is_empty());
            knoten.push(Graphknoten {
                id: n.zk_id.clone(),
                titel: n.titel.clone(),
                mappe: n.mappe.as_deref().map(mappen_id),
                gruppe: oberste.map(mappen_id),
                gruppenname: oberste.map(str::to_string),
            });
            for ziel in verweise::verweise(&n.inhalt) {
                let nach = match verweise::ziel_id(&ziel) {
                    Some(id) => ids.get(id).copied(),
                    None => nach_titel.get(&ziel.trim().to_lowercase()).copied(),
                };
                if let Some(nach) = nach.filter(|z| *z != n.zk_id) {
                    if gesehen.insert((n.zk_id.clone(), nach.to_string())) {
                        kanten.push((n.zk_id.clone(), nach.to_string()));
                    }
                }
            }
            for t in verweise::tags(&n.inhalt) {
                *tag_zahl.entry(t.clone()).or_default() += 1;
                tag_kanten.push((n.zk_id.clone(), t));
            }
        }
        Ok(Notizgraph {
            knoten,
            kanten,
            tags: tag_zahl.into_iter().collect(),
            tag_kanten,
        })
    }

    /// Welche Notizen per `[[Verweis]]` auf diese zeigen.
    pub fn rueckverweise(&self, zk_id: &str) -> Ergebnis<Vec<(String, String)>> {
        let alle = self.notizen()?;
        let mut liste = Vec::new();

        for quelle in &alle {
            if quelle.zk_id == zk_id {
                continue;
            }
            let trifft = verweise::verweise(&quelle.inhalt).iter().any(|ziel| {
                self.verweisziel(ziel)
                    .ok()
                    .flatten()
                    .is_some_and(|z| z.zk_id == zk_id)
            });
            if trifft {
                liste.push((quelle.zk_id.clone(), quelle.titel.clone()));
            }
        }

        liste.sort_by_key(|x| x.1.to_lowercase());
        Ok(liste)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn speicher() -> Speicher {
        Speicher::im_arbeitsspeicher().unwrap()
    }

    fn notiz(s: &Speicher, id: &str, titel: &str, inhalt: &str, mappe: Option<&str>) {
        s.notiz_schreiben(
            &Notiz {
                zk_id: id.into(),
                titel: titel.into(),
                inhalt: inhalt.into(),
                mappe: mappe.map(Into::into),
                ..Default::default()
            },
            Protokoll::Merken,
        )
        .unwrap();
    }

    #[test]
    fn der_graph_hat_ebenen_verweise_und_tags() {
        let s = speicher();
        notiz(
            &s,
            "20260101000001",
            "Kosten",
            "siehe [[Bilanz]] und [[kosten]] #bwl",
            Some("Uni/BWL"),
        );
        notiz(
            &s,
            "20260101000002",
            "Bilanz",
            "[[20260101000001]] #bwl #pruefung",
            Some("Uni"),
        );
        notiz(&s, "20260101000003", "Lose", "[[Gibt es nicht]]", None);

        let g = s.notizgraph().unwrap();
        let k1 = g.knoten.iter().find(|k| k.id == "20260101000001").unwrap();
        assert_eq!(k1.gruppe, Some(mappen_id("Uni")));
        assert_eq!(k1.gruppenname.as_deref(), Some("Uni"));
        assert_eq!(k1.mappe, Some(mappen_id("Uni/BWL")));
        let lose = g.knoten.iter().find(|k| k.id == "20260101000003").unwrap();
        assert_eq!((lose.gruppe, lose.mappe), (None, None));

        // Titel und Kennung loesen sich auf; auf sich selbst und ins Leere nicht.
        let mut kanten = g.kanten.clone();
        kanten.sort();
        assert_eq!(
            kanten,
            vec![
                ("20260101000001".to_string(), "20260101000002".to_string()),
                ("20260101000002".to_string(), "20260101000001".to_string()),
            ]
        );
        assert_eq!(
            g.tags,
            vec![("bwl".to_string(), 2), ("pruefung".to_string(), 1)]
        );
        assert_eq!(g.tag_kanten.len(), 3);
    }

    #[test]
    fn mappen_entstehen_aus_pfaden_mit_zwischenstufen() {
        let s = speicher();
        notiz(&s, "20260101000001", "a", "", Some("Uni/Hausarbeit"));
        s.mappe_anlegen(None, "Leer").unwrap();

        let m = s.mappen().unwrap();
        let namen: Vec<_> = m.iter().map(|x| x.pfad.as_str()).collect();
        assert_eq!(namen, vec!["Leer", "Uni", "Uni/Hausarbeit"]);

        let haus = m.iter().find(|x| x.pfad == "Uni/Hausarbeit").unwrap();
        assert_eq!(haus.name, "Hausarbeit");
        assert_eq!(haus.eltern_id, Some(mappen_id("Uni")));
    }

    #[test]
    fn die_mappen_id_ist_stabil_und_nie_null() {
        assert_eq!(mappen_id("Uni"), mappen_id("Uni"));
        assert_ne!(mappen_id("Uni"), mappen_id("uni"));
        assert!(mappen_id("") > 0);
    }

    #[test]
    fn liste_zeigt_die_ebene_und_sucht_ueber_alle() {
        let s = speicher();
        notiz(&s, "20260101000001", "Wurzel", "Brot", None);
        notiz(
            &s,
            "20260101000002",
            "Tief",
            "brot #einkauf/woche",
            Some("Haushalt"),
        );

        let wurzel = s.notizliste(&Filter::default(), 1).unwrap();
        assert_eq!(wurzel.notizen.len(), 1);

        let suche = Filter {
            suche: Some("BROT".into()),
            ..Default::default()
        };
        assert_eq!(s.notizliste(&suche, 1).unwrap().notizen.len(), 2);

        let tag = Filter {
            tag: Some("einkauf".into()),
            ..Default::default()
        };
        assert_eq!(s.notizliste(&tag, 1).unwrap().notizen[0].titel, "Tief");

        let in_mappe = Filter {
            mappe: Some("Haushalt".into()),
            suche: Some("brot".into()),
            ..Default::default()
        };
        assert_eq!(s.notizliste(&in_mappe, 1).unwrap().notizen.len(), 1);
    }

    #[test]
    fn rueckverweise_ueber_titel_und_kennung() {
        let s = speicher();
        notiz(&s, "20260101000001", "Einkauf", "", None);
        notiz(&s, "20260101000002", "Montag", "Siehe [[einkauf]]", None);
        notiz(
            &s,
            "20260101000003",
            "Dienstag",
            "[[20260101000001 Einkauf|hier]]",
            None,
        );
        notiz(&s, "20260101000004", "Mittwoch", "[[Anderes]]", None);

        let r = s.rueckverweise("20260101000001").unwrap();
        let titel: Vec<_> = r.iter().map(|x| x.1.as_str()).collect();
        assert_eq!(titel, vec!["Dienstag", "Montag"]);
    }

    #[test]
    fn umbenennen_nimmt_unterstufen_mit_und_merkt_es_sich() {
        let s = speicher();
        notiz(&s, "20260101000001", "a", "", Some("Uni/Hausarbeit"));
        let vorher = s.eigene_marke_jetzt().unwrap();

        s.mappe_umbenennen("Uni", "Studium").unwrap();

        assert_eq!(
            s.notiz("20260101000001").unwrap().unwrap().mappe.as_deref(),
            Some("Studium/Hausarbeit")
        );
        let (aenderungen, _, _) = s.aenderungen_seit(vorher).unwrap();
        assert_eq!(aenderungen.len(), 1);
    }

    #[test]
    fn loeschen_haengt_hoeher_oder_legt_in_den_papierkorb() {
        let s = speicher();
        notiz(&s, "20260101000001", "a", "", Some("Uni/Hausarbeit"));
        notiz(&s, "20260101000002", "b", "", Some("Alt"));

        s.mappe_loeschen("Uni/Hausarbeit", false).unwrap();
        assert_eq!(
            s.notiz("20260101000001").unwrap().unwrap().mappe.as_deref(),
            Some("Uni")
        );

        s.mappe_loeschen("Alt", true).unwrap();
        assert!(s.notiz("20260101000002").unwrap().unwrap().im_papierkorb());
    }
}
