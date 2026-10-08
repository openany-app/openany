//! Die andere Haelfte: was ein Geraet einem gepaarten Geraet anbietet.
//!
//! Das Gegenstueck zu openanys `DeltaController` und `SyncApply` -- nur dass
//! hier kein Server antwortet, sondern das Telefon oder das Tablet selbst.
//! Das andere Geraet spricht mit dieser Auskunft ueber seinen ganz normalen
//! [`crate::Laeufer`]; fuer ihn ist sie eine [`crate::Gegenstelle`] wie jede
//! andere.
//!
//! **Kein Netz, kein HTTP.** Wie die Anfragen hereinkommen, entscheidet
//! `openany-nahbereich`. Hier stehen nur die Regeln -- und die sind dieselben
//! wie beim Ziehen im Laeufer, damit es gleichgueltig ist, welches der beiden
//! Geraete gerade auf "Jetzt abgleichen" getippt hat.
//!
//! **Ein Unterschied zum Server, und er ist gewollt:** Neben dem `base_hash`
//! im Eintrag zaehlt auch der eigene Ursprung fuer das anrufende Geraet. Ein
//! Server kennt keinen; ein Geraet schon, denn es hat selbst schon von dem
//! anderen gezogen. Welcher der beiden der frische ist, klaert
//! [`crate::ursprung_waehlen`].

use crate::entscheidung::{entscheiden, ursprung_waehlen, Entscheidung};
use crate::hinaus::{
    album_aus_eintrag, album_gleich, anhang_aus_eintrag, bild_aus_eintrag, bild_gleich,
    datei_aus_eintrag, datei_gleich, freie_zk_id, hinausbringen, konflikttitel, wege_von,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use openany_client::{
    abdruck_von_eintrag, notizabdruck, Art, Delta, Eintrag, Ergebnis, Terminfelder, Was,
};
use openany_store::{
    fotoabdruck, Inhalte, Kalender, Kontakt, Notiz, Protokoll, Speicher, SpeicherFehler, Termin,
};
use serde_json::Value;

/// Die Auskunft fuer genau ein anrufendes Geraet.
pub struct Auskunft<'a> {
    speicher: &'a Speicher,
    /// Wie das anrufende Geraet HIER heisst -- dieselbe Basis, unter der
    /// dieses Geraet es selbst anspricht (`nah:<fingerabdruck>`). Nur so
    /// teilen sich beide Richtungen Marken und Urspruenge.
    basis: String,
    /// Wie DIESES Geraet heisst.
    herkunft: String,
    /// Die Inhaltsablage, falls dieses Geraet Inhalte ausliefert.
    inhalte: Option<&'a Inhalte>,
}

impl<'a> Auskunft<'a> {
    pub fn neu(
        speicher: &'a Speicher,
        basis: impl Into<String>,
        herkunft: impl Into<String>,
    ) -> Self {
        Self {
            speicher,
            basis: basis.into(),
            herkunft: herkunft.into(),
            inhalte: None,
        }
    }

    /// Mit Inhaltsablage: Dann liefert die Auskunft auch Dateiinhalte.
    pub fn mit_inhalten(mut self, inhalte: &'a Inhalte) -> Self {
        self.inhalte = Some(inhalte);
        self
    }

    /// Ein Stueck eines Inhalts; `None`, wenn er hier nicht liegt.
    pub fn inhalt(&self, abdruck: &str, von: u64, laenge: u64) -> Option<Vec<u8>> {
        let inhalte = self.inhalte?;
        if !inhalte.hat(abdruck) {
            return None;
        }
        inhalte
            .lesen(abdruck, von, laenge.min(crate::STUECK) as usize)
            .ok()
    }

    /// Welche dieser Abdruecke liegen hier als Inhalt?
    pub fn inhalte_da(&self, abdruecke: &[String]) -> Vec<String> {
        let Some(inhalte) = self.inhalte else {
            return Vec::new();
        };
        abdruecke
            .iter()
            .filter(|a| inhalte.hat(a))
            .cloned()
            .collect()
    }

    /// Eine Seite aus dem eigenen Protokoll -- in derselben Form, die der
    /// Server ausgibt.
    ///
    /// **Ohne Notiztext und ohne Fotobytes.** Die holt der Laeufer einzeln,
    /// und nur, wenn er sie braucht; eine Seite mit 500 Notizen soll nicht
    /// 500 Texte tragen.
    pub fn delta(&self, seit: i64) -> Result<Delta, SpeicherFehler> {
        let (aenderungen, cursor, more) = self.speicher.aenderungen_seit(seit)?;
        let mut eintraege = Vec::with_capacity(aenderungen.len());

        for aenderung in &aenderungen {
            // Was vom anrufenden Geraet kam, kennt es schon.
            if aenderung.herkunft.as_deref() == Some(self.basis.as_str()) {
                continue;
            }
            if let Some(mut eintrag) =
                hinausbringen(self.speicher, &self.basis, &self.herkunft, aenderung, false)?
            {
                eintrag.felder.remove("content");
                eintrag.felder.remove("photo_bytes");
                eintraege.push(eintrag);
            }
        }

        Ok(Delta {
            cursor,
            more,
            eintraege,
            speicher: None,
        })
    }

    pub fn notiztext(&self, zk_id: &str) -> Result<Option<String>, SpeicherFehler> {
        Ok(self.speicher.notiz(zk_id)?.map(|n| n.inhalt))
    }

    pub fn kontaktfoto(&self, uuid: &str) -> Result<Option<Vec<u8>>, SpeicherFehler> {
        self.speicher.kontaktfoto(uuid)
    }

    /// Das andere Geraet meldet, wie weit beide sind (siehe
    /// [`crate::Gegenstelle::marken_melden`]) -- aus seiner Sicht, also
    /// ueber Kreuz.
    ///
    /// **Nur vorwaerts.** Eine Meldung, die aelter ist als der eigene Stand,
    /// holte Erledigtes zurueck. Und nie ueber das eigene Protokoll hinaus:
    /// Eine Marke, die es hier gar nicht gibt, uebersprange kuenftige
    /// Aenderungen.
    pub fn marken_gemeldet(&self, eigene: i64, fremde: i64) -> Result<(), SpeicherFehler> {
        let marke = self.speicher.marke(&self.basis)?;

        // Was das andere Geraet von sich hierher geschoben hat, braucht
        // dieses Geraet nicht mehr bei ihm zu holen.
        if eigene > marke.fremde {
            self.speicher.fremde_marke_setzen(&self.basis, eigene)?;
        }

        // Was es von hier geholt hat, braucht nicht mehr hinueber.
        let fremde = fremde.min(self.speicher.eigene_marke_jetzt()?);
        if fremde > marke.eigene {
            self.speicher.eigene_marke_setzen(&self.basis, fremde)?;
        }

        Ok(())
    }

    /// Geschobene Eintraege annehmen -- je Eintrag ein Ergebnis, in der
    /// Reihenfolge der Eintraege.
    ///
    /// **Angenommenes mit Herkunft ins Protokoll** (`Protokoll::Von`), damit
    /// es an weitere eigene Geraete weitergeht -- aber nie an das anrufende
    /// zurueck. Konfliktkopien ohne Herkunft: Die kennt drueben niemand.
    pub fn anwenden(&self, eintraege: &[Eintrag]) -> Result<Vec<Ergebnis>, SpeicherFehler> {
        eintraege.iter().map(|e| self.eines(e)).collect()
    }

    fn eines(&self, eintrag: &Eintrag) -> Result<Ergebnis, SpeicherFehler> {
        let aktion = match eintrag.was {
            Was::Papierkorb | Was::Fort => self.fortnehmen(eintrag)?,
            Was::Da => match eintrag.art {
                Art::Notiz => self.notiz(eintrag)?,
                Art::Termin => self.termin(eintrag)?,
                Art::Kalender => self.kalender(eintrag)?,
                Art::Kontakt => self.kontakt(eintrag)?,
                Art::Datei => self.datei(eintrag)?,
                Art::Album => self.album(eintrag)?,
                Art::Medium => self.bild(eintrag)?,
                Art::Notizanhang => self.anhang(eintrag)?,
                Art::Unbekannt(_) => "ignored",
            },
        };

        Ok(Ergebnis {
            art: eintrag.art.ueber_die_leitung().to_string(),
            key: eintrag.key.clone(),
            action: aktion.to_string(),
            conflict: aktion == "conflict",
        })
    }

    /// Der gemeinsame Stand -- aus dem `base_hash` im Eintrag und dem eigenen
    /// Gedaechtnis gewaehlt ([`ursprung_waehlen`]).
    fn ursprung(
        &self,
        eintrag: &Eintrag,
        hier: Option<&str>,
        drueben: &str,
    ) -> Result<Option<String>, SpeicherFehler> {
        let genannt = eintrag.text("base_hash").map(str::to_string);
        let eigener = self
            .speicher
            .ursprung(&self.basis, &eintrag.art, &eintrag.key)?;

        Ok(ursprung_waehlen(&[genannt, eigener], hier, drueben))
    }

    fn wer(&self, eintrag: &Eintrag) -> String {
        eintrag.text("origin").unwrap_or("Abgleich").to_string()
    }

    fn fortnehmen(&self, eintrag: &Eintrag) -> Result<&'static str, SpeicherFehler> {
        let getan = if eintrag.was == Was::Papierkorb {
            self.speicher
                .papierkorb(&eintrag.art, &eintrag.key, Protokoll::Von(&self.basis))?
        } else {
            self.speicher
                .loeschen(&eintrag.art, &eintrag.key, Protokoll::Von(&self.basis))?
        };

        self.speicher
            .ursprung_vergessen(&self.basis, &eintrag.art, &eintrag.key)?;

        Ok(match (getan, eintrag.was) {
            (false, _) => "gone",
            (true, Was::Papierkorb) => "trashed",
            (true, _) => "deleted",
        })
    }

    fn notiz(&self, eintrag: &Eintrag) -> Result<&'static str, SpeicherFehler> {
        let inhalt = eintrag.text("content").unwrap_or_default().to_string();
        let drueben = notizabdruck(&inhalt);
        let hiesige = self.speicher.notiz(&eintrag.key)?;
        let hier = hiesige.as_ref().map(|n| notizabdruck(&n.inhalt));
        let ursprung = self.ursprung(eintrag, hier.as_deref(), &drueben)?;
        let titel = eintrag.text("title").unwrap_or_default();
        let mappe = eintrag.text("folder").map(str::to_string);

        let aktion = match entscheiden(ursprung.as_deref(), hier.as_deref(), &drueben) {
            Entscheidung::Unveraendert | Entscheidung::HierBleibt => "unchanged",
            e @ (Entscheidung::Anlegen | Entscheidung::Uebernehmen) => {
                self.speicher.notiz_schreiben(
                    &Notiz {
                        zk_id: eintrag.key.clone(),
                        titel: titel.to_string(),
                        inhalt,
                        mappe,
                        papierkorb_at: None,
                        geaendert_at: eintrag.text("updated_at").unwrap_or_default().to_string(),
                    },
                    Protokoll::Von(&self.basis),
                )?;
                if e == Entscheidung::Anlegen {
                    "created"
                } else {
                    "updated"
                }
            }
            // Der hiesige Stand bleibt unangetastet; die fremde Fassung kommt
            // als eigene Notiz daneben -- und ins Protokoll, denn drueben
            // kennt sie niemand.
            Entscheidung::Konflikt => {
                self.speicher.notiz_schreiben(
                    &Notiz {
                        zk_id: freie_zk_id(self.speicher)?,
                        titel: konflikttitel(titel, &self.wer(eintrag)),
                        inhalt,
                        mappe,
                        papierkorb_at: None,
                        geaendert_at: String::new(),
                    },
                    Protokoll::Merken,
                )?;
                "conflict"
            }
        };

        // Wie beim Ziehen: Der angekommene Stand ist ab jetzt der gemeinsame,
        // auch nach einem Konflikt -- sonst faende der naechste Lauf ihn noch
        // einmal.
        self.speicher
            .ursprung_merken(&self.basis, &Art::Notiz, &eintrag.key, &drueben)?;

        Ok(aktion)
    }

    /// Ein Termin -- dreiseitig. Ohne Kalender-uuid nicht zuzuordnen; mit
    /// einer, deren Kalender hier noch fehlt, trotzdem angenommen (dieselbe
    /// bewusste Abweichung vom Server wie im Laeufer).
    fn termin(&self, eintrag: &Eintrag) -> Result<&'static str, SpeicherFehler> {
        let Some(kalender_uuid) = eintrag.text("calendar").map(str::to_string) else {
            return Ok("skipped");
        };

        let drueben = abdruck_von_eintrag(&eintrag.felder);
        let hier = self.speicher.termin(&eintrag.key)?.map(|t| t.abdruck());
        let ursprung = self.ursprung(eintrag, hier.as_deref(), &drueben)?;
        let felder = Terminfelder::aus_eintrag(&eintrag.felder);

        let aktion = match entscheiden(ursprung.as_deref(), hier.as_deref(), &drueben) {
            Entscheidung::Unveraendert | Entscheidung::HierBleibt => "unchanged",
            e @ (Entscheidung::Anlegen | Entscheidung::Uebernehmen) => {
                self.speicher.termin_schreiben(
                    &Termin {
                        uuid: eintrag.key.clone(),
                        kalender_uuid,
                        felder,
                        papierkorb_at: None,
                        geaendert_at: eintrag.text("updated_at").unwrap_or_default().to_string(),
                    },
                    Protokoll::Von(&self.basis),
                )?;
                if e == Entscheidung::Anlegen {
                    "created"
                } else {
                    "updated"
                }
            }
            Entscheidung::Konflikt => {
                let mut kopie = felder;
                kopie.titel = konflikttitel(&kopie.titel, &self.wer(eintrag));
                self.speicher.termin_schreiben(
                    &Termin {
                        uuid: uuid::Uuid::new_v4().to_string(),
                        kalender_uuid,
                        felder: kopie,
                        papierkorb_at: None,
                        geaendert_at: String::new(),
                    },
                    Protokoll::Merken,
                )?;
                "conflict"
            }
        };

        self.speicher
            .ursprung_merken(&self.basis, &Art::Termin, &eintrag.key, &drueben)?;

        Ok(aktion)
    }

    /// Eine Datei oder ein Ordner -- das letzte Schreiben gewinnt.
    fn datei(&self, eintrag: &Eintrag) -> Result<&'static str, SpeicherFehler> {
        let Some(neu) = datei_aus_eintrag(eintrag) else {
            return Ok("skipped");
        };
        let da = self.speicher.datei(&eintrag.key)?;
        if da.as_ref().is_some_and(|d| datei_gleich(d, &neu)) {
            return Ok("unchanged");
        }
        self.speicher
            .datei_schreiben(&neu, Protokoll::Von(&self.basis))?;
        Ok(if da.is_none() { "created" } else { "updated" })
    }

    fn album(&self, eintrag: &Eintrag) -> Result<&'static str, SpeicherFehler> {
        let Some(neu) = album_aus_eintrag(eintrag) else {
            return Ok("skipped");
        };
        let da = self.speicher.album(&eintrag.key)?;
        if da.as_ref().is_some_and(|a| album_gleich(a, &neu)) {
            return Ok("unchanged");
        }
        self.speicher
            .album_schreiben(&neu, Protokoll::Von(&self.basis))?;
        Ok(if da.is_none() { "created" } else { "updated" })
    }

    fn bild(&self, eintrag: &Eintrag) -> Result<&'static str, SpeicherFehler> {
        let Some(neu) = bild_aus_eintrag(eintrag) else {
            return Ok("skipped");
        };
        let da = self.speicher.bild(&eintrag.key)?;
        if da.as_ref().is_some_and(|b| bild_gleich(b, &neu)) {
            return Ok("unchanged");
        }
        self.speicher
            .bild_schreiben(&neu, Protokoll::Von(&self.basis))?;
        Ok(if da.is_none() { "created" } else { "updated" })
    }

    /// Ein Notiz-Anhang -- eine Zuordnung, keine Bytes. Sie wird auch
    /// angenommen, wenn das Ziel hier (noch) fehlt: Die Bytes holt sich
    /// dieses Geraet nach seiner Regel, und eine Zuordnung ohne Bild ist
    /// besser als ein Bild, dem beim naechsten Lauf die Zuordnung fehlt.
    ///
    /// BIS ZUM 17.09.2026 FIEL DIE ART IN DEN SAMMELZWEIG ("ignored"): Ein
    /// Anhang, den ein Geraet anlegte und hierher schob, kam nie an.
    fn anhang(&self, eintrag: &Eintrag) -> Result<&'static str, SpeicherFehler> {
        let Some(neu) = anhang_aus_eintrag(eintrag) else {
            return Ok("skipped");
        };
        let da = self.speicher.anhang(&neu.mappe, &neu.pfad)?;
        if da.as_ref() == Some(&neu) {
            return Ok("unchanged");
        }
        self.speicher
            .anhang_schreiben(&neu, Protokoll::Von(&self.basis))?;
        Ok(if da.is_none() { "created" } else { "updated" })
    }

    /// Ein Kalender -- das letzte Schreiben gewinnt.
    fn kalender(&self, eintrag: &Eintrag) -> Result<&'static str, SpeicherFehler> {
        let da = self.speicher.kalender(&eintrag.key)?;
        let name = eintrag
            .text("name")
            .map(str::to_string)
            .or_else(|| da.as_ref().map(|k| k.name.clone()))
            .unwrap_or_else(|| "Kalender".into());
        let farbe = eintrag
            .text("color")
            .map(str::to_string)
            .or_else(|| da.as_ref().map(|k| k.farbe.clone()))
            .unwrap_or_default();

        if let Some(k) = &da {
            if k.name == name && k.farbe == farbe && k.papierkorb_at.is_none() {
                return Ok("unchanged");
            }
        }

        self.speicher.kalender_schreiben(
            &Kalender {
                uuid: eintrag.key.clone(),
                name,
                farbe,
                // Die Abo-Adresse reist mit, der Inhalt nicht -- die
                // Begruendung steht ausfuehrlich an `Laeufer::kalender_ziehen`.
                abo_url: match eintrag.hat("sync_url") {
                    true => eintrag.text("sync_url").map(str::to_string),
                    false => da.as_ref().and_then(|k| k.abo_url.clone()),
                },
                zuletzt_geholt: da.as_ref().and_then(|k| k.zuletzt_geholt.clone()),
                papierkorb_at: None,
                geaendert_at: eintrag.text("updated_at").unwrap_or_default().to_string(),
            },
            Protokoll::Von(&self.basis),
        )?;

        Ok(if da.is_none() { "created" } else { "updated" })
    }

    /// Ein Kontakt -- das letzte Schreiben gewinnt, keine Konfliktkopie
    /// (Begruendung am Laeufer und an `SyncApply::kontakt`).
    ///
    /// Das Foto kommt hier als Bytes: abwesend heisst "unveraendert", `null`
    /// heisst "entfernt".
    fn kontakt(&self, eintrag: &Eintrag) -> Result<&'static str, SpeicherFehler> {
        let da = self.speicher.kontakt(&eintrag.key)?;
        let wege = wege_von(
            eintrag,
            da.as_ref().map(|k| k.wege.clone()).unwrap_or_default(),
        );
        let anzeigename = eintrag
            .text("display_name")
            .unwrap_or("Kontakt")
            .to_string();

        let gleich = da.as_ref().is_some_and(|k| {
            k.anzeigename == anzeigename && k.wege == wege && k.papierkorb_at.is_none()
        });

        if !gleich {
            self.speicher.kontakt_schreiben(
                &Kontakt {
                    uuid: eintrag.key.clone(),
                    anzeigename,
                    wege,
                    foto: da.as_ref().and_then(|k| k.foto.clone()),
                    papierkorb_at: None,
                    geaendert_at: eintrag.text("updated_at").unwrap_or_default().to_string(),
                },
                Protokoll::Von(&self.basis),
            )?;
        }

        let foto_neu = match eintrag.felder.get("photo_bytes") {
            None => false,
            Some(Value::String(roh)) if !roh.is_empty() => match BASE64.decode(roh) {
                // Ein unlesbares Bild reisst den Kontakt nicht mit.
                Err(_) => false,
                Ok(bytes) => {
                    let abdruck = fotoabdruck(&bytes);
                    let vorher = da.as_ref().and_then(|k| k.foto.as_deref());
                    if vorher == Some(abdruck.as_str()) {
                        false
                    } else {
                        self.speicher.kontaktfoto_uebernehmen(
                            &eintrag.key,
                            Some(&abdruck),
                            Some(&bytes),
                            Protokoll::Von(&self.basis),
                        )?;
                        true
                    }
                }
            },
            Some(_) => {
                let hatte = da.as_ref().is_some_and(|k| k.foto.is_some());
                if hatte {
                    self.speicher.kontaktfoto_uebernehmen(
                        &eintrag.key,
                        None,
                        None,
                        Protokoll::Von(&self.basis),
                    )?;
                }
                hatte
            }
        };

        Ok(match (da.is_some(), gleich && !foto_neu) {
            (false, _) => "created",
            (true, true) => "unchanged",
            (true, false) => "updated",
        })
    }
}
