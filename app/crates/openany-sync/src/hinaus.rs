//! Was beide Richtungen teilen: der [`crate::Laeufer`], der schiebt, und die
//! [`crate::Auskunft`], die einem anderen Geraet antwortet.
//!
//! **Eine Hand fuer beide Wege.** Ein Termin, der beim Schieben neun Felder
//! traegt und in der Auskunft acht, waere die Stelle, an der zwei Geraete
//! bei jedem Lauf dieselbe Aenderung meldeten und keine anwendeten.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{Local, Utc};
use openany_client::{notizabdruck, Art, Eintrag, Was};
use openany_store::{Aenderung, FotoHinaus, Speicher, SpeicherFehler, Weg};
use serde_json::Value;

/// Einen eigenen Protokolleintrag versandfertig machen.
///
/// **Der Zustand kommt aus der Sache, nicht aus dem Protokoll.** Das
/// Protokoll sagt, DASS sich etwas geaendert hat; wie es jetzt dasteht,
/// weiss nur die Sache selbst. Eine Notiz, die geloescht und wieder
/// hergestellt wurde, ist da -- ganz gleich, welche Zeile zuletzt
/// geschrieben wurde. So kann kein ueberholter Eintrag eine falsche
/// Auskunft geben.
///
/// * `basis` -- die Gegenstelle, deren Ursprung als `base_hash` mitgeht.
/// * `herkunft` -- wie dieses Geraet heisst, falls drueben eine
///   Konfliktkopie entsteht.
/// * `fotos_immer` -- Kontaktfotos auch dann als Bytes mitschicken, wenn sie
///   nicht hier gesetzt wurden (siehe [`crate::Gegenstelle::fotos_mitschicken`]).
pub(crate) fn hinausbringen(
    speicher: &Speicher,
    basis: &str,
    herkunft: &str,
    aenderung: &Aenderung,
    fotos_immer: bool,
) -> Result<Option<Eintrag>, SpeicherFehler> {
    let art = aenderung.art.clone();
    let key = aenderung.schluessel.clone();

    match art {
        Art::Notiz => {
            let Some(notiz) = speicher.notiz(&key)? else {
                return Ok(Some(Eintrag::neu(art, key, Was::Fort)));
            };

            if let Some(seit) = &notiz.papierkorb_at {
                return Ok(Some(
                    Eintrag::neu(art, key, Was::Papierkorb).mit("deleted_at", seit.clone()),
                ));
            }

            let hash = notizabdruck(&notiz.inhalt);
            let ursprung = mitzuschickender_ursprung(speicher, basis, &art, &key)?;

            Ok(Some(
                Eintrag::neu(art, key, Was::Da)
                    .mit("title", notiz.titel)
                    .mit("folder", nullbar(notiz.mappe))
                    .mit("content", notiz.inhalt.clone())
                    .mit("hash", hash)
                    .mit("size", notiz.inhalt.len() as u64)
                    .mit("updated_at", notiz.geaendert_at)
                    .mit("base_hash", nullbar(ursprung))
                    .mit("origin", herkunft),
            ))
        }
        Art::Termin => {
            let Some(termin) = speicher.termin(&key)? else {
                return Ok(Some(Eintrag::neu(art, key, Was::Fort)));
            };

            if let Some(seit) = &termin.papierkorb_at {
                return Ok(Some(
                    Eintrag::neu(art, key, Was::Papierkorb).mit("deleted_at", seit.clone()),
                ));
            }

            let ursprung = mitzuschickender_ursprung(speicher, basis, &art, &key)?;
            let mut eintrag = Eintrag::neu(art, key, Was::Da);

            // Die neun Felder aus EINER Hand -- sie hier von Hand
            // abzuschreiben waere die Stelle, an der ein Feld zwar in den
            // Abdruck eingeht, aber nie hinausgeht.
            eintrag.felder = termin.felder.als_eintrag();

            Ok(Some(
                eintrag
                    .mit("calendar", termin.kalender_uuid)
                    .mit("hash", termin.felder.abdruck())
                    .mit("updated_at", termin.geaendert_at)
                    .mit("base_hash", nullbar(ursprung))
                    .mit("origin", herkunft),
            ))
        }
        Art::Kalender => {
            let Some(kalender) = speicher.kalender(&key)? else {
                return Ok(Some(Eintrag::neu(art, key, Was::Fort)));
            };

            if let Some(seit) = &kalender.papierkorb_at {
                return Ok(Some(
                    Eintrag::neu(art, key, Was::Papierkorb).mit("deleted_at", seit.clone()),
                ));
            }

            Ok(Some(
                Eintrag::neu(art, key, Was::Da)
                    .mit("name", kalender.name)
                    .mit("color", kalender.farbe)
                    // Die Abo-Adresse reist mit, der INHALT des Abos nicht:
                    // Die Termine daraus stehen mit `Protokoll::Still` im
                    // Speicher und kommen hier nie vorbei. Damit traegt man
                    // einen Feed einmal ein und hat ihn auf allen Geraeten --
                    // jedes holt ihn dann selbst.
                    //
                    // `null` heisst ausdruecklich "kein Abo" und ist etwas
                    // anderes als ein fehlendes Feld. Die Gegenseite
                    // unterscheidet beides (`SyncApply::aboAdresse`).
                    .mit("sync_url", nullbar(kalender.abo_url))
                    .mit("updated_at", kalender.geaendert_at),
            ))
        }
        Art::Kontakt => {
            let Some(kontakt) = speicher.kontakt(&key)? else {
                return Ok(Some(Eintrag::neu(art, key, Was::Fort)));
            };

            if let Some(seit) = &kontakt.papierkorb_at {
                return Ok(Some(
                    Eintrag::neu(art, key, Was::Papierkorb).mit("deleted_at", seit.clone()),
                ));
            }

            // `channels` UND die beiden alten Felder: Ein Server vor den
            // Wegen liest nur diese beiden, ein neuer nimmt `channels` und
            // uebergeht sie. So verliert keine Gegenstelle etwas.
            let mut hinaus = Eintrag::neu(art, key.clone(), Was::Da)
                .mit("display_name", kontakt.anzeigename.clone())
                .mit(
                    "channels",
                    serde_json::to_value(&kontakt.wege).unwrap_or(Value::Array(vec![])),
                )
                .mit(
                    "matrix_id",
                    nullbar(kontakt.kennung("matrix").map(str::to_string)),
                )
                .mit(
                    "meshtastic_id",
                    nullbar(kontakt.kennung("meshtastic").map(str::to_string)),
                )
                .mit("photo", nullbar(kontakt.foto.clone()))
                .mit("updated_at", kontakt.geaendert_at);

            // Das Foto nur, wenn es HIER geaendert wurde. Fehlt
            // `photo_bytes`, laesst der Server das Bild in Ruhe.
            match speicher.kontaktfoto_hinaus(&key)? {
                Some(FotoHinaus::Setzen(bytes)) => {
                    hinaus = hinaus.mit("photo_bytes", BASE64.encode(bytes));
                }
                Some(FotoHinaus::Entfernen) => {
                    hinaus = hinaus.mit("photo_bytes", Value::Null);
                }
                None if fotos_immer && kontakt.foto.is_some() => {
                    if let Some(bytes) = speicher.kontaktfoto(&key)? {
                        hinaus = hinaus.mit("photo_bytes", BASE64.encode(bytes));
                    }
                }
                None => {}
            }

            Ok(Some(hinaus))
        }
        Art::Datei => {
            let Some(d) = speicher.datei(&key)? else {
                return Ok(Some(Eintrag::neu(art, key, Was::Fort)));
            };

            if let Some(seit) = &d.papierkorb_at {
                return Ok(Some(
                    Eintrag::neu(art, key, Was::Papierkorb).mit("deleted_at", seit.clone()),
                ));
            }

            // Dieselben Namen wie drueben im DeltaFeed, dazu `hash` und
            // `mime_type`: Ein Geraet holt den Inhalt nach Abdruck.
            Ok(Some(
                Eintrag::neu(art, key, Was::Da)
                    .mit("name", d.name)
                    .mit("zone", d.zone)
                    .mit("is_folder", d.ist_ordner)
                    .mit("parent", nullbar(d.eltern))
                    .mit("size", d.groesse)
                    .mit("mime_type", d.mime)
                    .mit("hash", nullbar(d.abdruck))
                    .mit("updated_at", d.geaendert_at),
            ))
        }
        Art::Album => {
            let Some(a) = speicher.album(&key)? else {
                return Ok(Some(Eintrag::neu(art, key, Was::Fort)));
            };
            if let Some(seit) = &a.papierkorb_at {
                return Ok(Some(
                    Eintrag::neu(art, key, Was::Papierkorb).mit("deleted_at", seit.clone()),
                ));
            }
            Ok(Some(
                Eintrag::neu(art, key, Was::Da)
                    .mit("name", a.name)
                    .mit("description", a.beschreibung)
                    .mit("parent", nullbar(a.eltern))
                    .mit("updated_at", a.geaendert_at),
            ))
        }
        Art::Medium => {
            let Some(b) = speicher.bild(&key)? else {
                return Ok(Some(Eintrag::neu(art, key, Was::Fort)));
            };
            if let Some(seit) = &b.papierkorb_at {
                return Ok(Some(
                    Eintrag::neu(art, key, Was::Papierkorb).mit("deleted_at", seit.clone()),
                ));
            }
            let exif = serde_json::from_str::<Value>(&b.exif).unwrap_or(Value::Null);
            Ok(Some(
                Eintrag::neu(art, key, Was::Da)
                    .mit("name", b.name)
                    .mit("album", nullbar(b.album))
                    .mit("mime_type", b.mime)
                    .mit("size", b.groesse)
                    .mit("hash", nullbar(b.abdruck))
                    .mit("thumb_hash", nullbar(b.vorschau))
                    .mit("exif", exif)
                    .mit("updated_at", b.geaendert_at),
            ))
        }
        Art::Notizanhang => {
            // Der Schluessel ist zweiteilig (Mappenpfad|Pfad), wie er ueber
            // die Leitung kommt -- die Tabelle kennt zwei Spalten.
            let (mappe, pfad) = match key.split_once('|') {
                Some((m, p)) => (m.to_string(), p.to_string()),
                None => (String::new(), key.clone()),
            };
            let Some(a) = speicher.anhang(&mappe, &pfad)? else {
                return Ok(Some(Eintrag::neu(art, key, Was::Fort)));
            };

            // Dieselben Namen wie im DeltaFeed drueben, samt BEIDEN
            // Abdruecken des Ziels: Ohne sie kann die Gegenseite die Bytes
            // nicht anfordern (16.09.2026: 26 Zuordnungen, kein Bild).
            Ok(Some(
                Eintrag::neu(art, key, Was::Da)
                    .mit("folder", a.mappe)
                    .mit("path", a.pfad)
                    .mit("target_type", a.ziel_art)
                    .mit("target_uuid", a.ziel_uuid)
                    .mit("size", a.groesse)
                    .mit("hash", nullbar(a.abdruck))
                    .mit("thumb_hash", nullbar(a.vorschau)),
            ))
        }
        // Eine Art, die dieser Stand nicht kennt, kann er auch nicht schieben.
        Art::Unbekannt(_) => Ok(None),
    }
}

/// Der `base_hash` fuer eine Gegenstelle: der gemeinsame Stand mit ihr, sonst
/// der juengste mit irgendeiner (siehe `Speicher::juengster_ursprung`).
fn mitzuschickender_ursprung(
    speicher: &Speicher,
    basis: &str,
    art: &Art,
    key: &str,
) -> Result<Option<String>, SpeicherFehler> {
    match speicher.ursprung(basis, art, key)? {
        Some(u) => Ok(Some(u)),
        None => speicher.juengster_ursprung(art, key),
    }
}

/// Eine Datei aus einem Eintrag -- `None`, wenn Pflichtfelder fehlen.
///
/// Das letzte Schreiben gewinnt, wie bei Kalendern: Eine Datei wird in
/// diesem Programm nicht bearbeitet, sondern neu hochgeladen (neue uuid).
/// Was sich an einer vorhandenen aendert, sind Name und Ort.
pub(crate) fn datei_aus_eintrag(eintrag: &Eintrag) -> Option<openany_store::Datei> {
    Some(openany_store::Datei {
        uuid: eintrag.key.clone(),
        zone: eintrag.text("zone").unwrap_or("files").to_string(),
        ist_ordner: eintrag
            .felder
            .get("is_folder")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        eltern: eintrag.text("parent").map(str::to_string),
        name: eintrag.text("name")?.to_string(),
        groesse: eintrag
            .felder
            .get("size")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        mime: eintrag.text("mime_type").unwrap_or_default().to_string(),
        abdruck: eintrag.text("hash").map(str::to_string),
        papierkorb_at: None,
        geaendert_at: eintrag.text("updated_at").unwrap_or_default().to_string(),
    })
}

/// Steht dasselbe schon da? Dann nichts schreiben -- sonst liefe es zwischen
/// drei Geraeten im Kreis (siehe `Protokoll::Von`).
pub(crate) fn datei_gleich(a: &openany_store::Datei, b: &openany_store::Datei) -> bool {
    a.zone == b.zone
        && a.ist_ordner == b.ist_ordner
        && a.eltern == b.eltern
        && a.name == b.name
        && a.groesse == b.groesse
        && a.mime == b.mime
        && a.abdruck == b.abdruck
        && a.papierkorb_at.is_none()
}

/// Dateien, Alben und Bilder -- was nur zwischen Geraeten reist
/// ([`crate::Gegenstelle::traegt_dateien`]).
/// Ein Anhang aus einem Delta-Eintrag.
///
/// **`target_uuid` darf fehlen** -- drueben steht dort `$sache->target()?->uuid`,
/// und das Ziel kann geloescht sein. Ein Anhang ohne Ziel ist keine
/// Zuordnung, sondern ein Verweis ins Leere; er wird uebergangen.
pub(crate) fn anhang_aus_eintrag(eintrag: &Eintrag) -> Option<openany_store::Anhang> {
    Some(openany_store::Anhang {
        // Die Wurzelmappe heisst drueben leer, nicht `null`.
        mappe: eintrag.text("folder").unwrap_or_default().to_string(),
        pfad: eintrag.text("path")?.to_string(),
        ziel_art: eintrag.text("target_type")?.to_string(),
        ziel_uuid: eintrag.text("target_uuid")?.to_string(),
        groesse: eintrag
            .felder
            .get("size")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        abdruck: eintrag.text("hash").map(str::to_string),
        vorschau: eintrag.text("thumb_hash").map(str::to_string),
    })
}

pub(crate) fn ist_speicherart(art: &Art) -> bool {
    matches!(art, Art::Datei | Art::Album | Art::Medium)
}

pub(crate) fn album_aus_eintrag(eintrag: &Eintrag) -> Option<openany_store::Album> {
    Some(openany_store::Album {
        uuid: eintrag.key.clone(),
        name: eintrag.text("name")?.to_string(),
        beschreibung: eintrag.text("description").unwrap_or_default().to_string(),
        eltern: eintrag.text("parent").map(str::to_string),
        papierkorb_at: None,
        geaendert_at: eintrag.text("updated_at").unwrap_or_default().to_string(),
    })
}

pub(crate) fn album_gleich(a: &openany_store::Album, b: &openany_store::Album) -> bool {
    a.name == b.name
        && a.beschreibung == b.beschreibung
        && a.eltern == b.eltern
        && a.papierkorb_at.is_none()
}

pub(crate) fn bild_aus_eintrag(eintrag: &Eintrag) -> Option<openany_store::Bild> {
    Some(openany_store::Bild {
        uuid: eintrag.key.clone(),
        album: eintrag.text("album").map(str::to_string),
        name: eintrag.text("name")?.to_string(),
        mime: eintrag.text("mime_type").unwrap_or_default().to_string(),
        groesse: eintrag
            .felder
            .get("size")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        abdruck: eintrag.text("hash").map(str::to_string),
        vorschau: eintrag.text("thumb_hash").map(str::to_string),
        exif: match eintrag.felder.get("exif") {
            Some(v @ Value::Object(_)) => v.to_string(),
            _ => String::new(),
        },
        papierkorb_at: None,
        geaendert_at: eintrag.text("updated_at").unwrap_or_default().to_string(),
    })
}

pub(crate) fn bild_gleich(a: &openany_store::Bild, b: &openany_store::Bild) -> bool {
    let exif = |t: &str| serde_json::from_str::<Value>(t).unwrap_or(Value::Null);
    a.album == b.album
        && a.name == b.name
        && a.mime == b.mime
        && a.groesse == b.groesse
        && a.abdruck == b.abdruck
        && a.vorschau == b.vorschau
        && exif(&a.exif) == exif(&b.exif)
        && a.papierkorb_at.is_none()
}

/// Der Zusatz im Titel einer Konfliktkopie.
///
/// Fest und nicht uebersetzt -- er steht dauerhaft im Titel, und ein
/// Sprachwechsel darf einen Titel nicht nachtraeglich veraendern.
pub(crate) fn konflikttitel(titel: &str, wer: &str) -> String {
    format!(
        "{titel} (Konflikt {wer} {})",
        Local::now().format("%Y-%m-%d")
    )
    .trim()
    .to_string()
}

/// Eine freie `zk_id` im Zettlr-Format `YYYYMMDDHHMMSS`.
///
/// Bei Kollision je eine Sekunde weiter -- dieselbe Regel wie drueben in
/// `Note::generateZkId()`. Die Sekunde ist grob genug, dass zwei Geraete
/// im selben Moment dieselbe Id vergeben koennen; dann trifft drueben der
/// Drei-Seiten-Vergleich zu und legt eine Konfliktkopie an. Unschoen,
/// aber nichts geht verloren.
pub(crate) fn freie_zk_id(speicher: &Speicher) -> Result<String, SpeicherFehler> {
    let mut moment = Local::now();

    for _ in 0..86_400 {
        let vorschlag = moment.format("%Y%m%d%H%M%S").to_string();

        if speicher.notiz(&vorschlag)?.is_none() {
            return Ok(vorschlag);
        }

        moment += chrono::Duration::seconds(1);
    }

    // Praktisch unerreichbar; ein Zeitstempel mit Zufall ist immer noch
    // besser als ein Abbruch, der eine fremde Fassung verwirft.
    Ok(format!(
        "{}{}",
        Utc::now().format("%Y%m%d%H%M%S"),
        &uuid::Uuid::new_v4().simple().to_string()[..4]
    ))
}

/// Die Wege eines ankommenden Kontakts.
///
/// **Abwesend heisst "unbekannt", leer heisst "keine"** -- dieselbe Regel
/// wie drueben in `SyncApply::wege()`. Eine Gegenstelle ohne `channels`
/// (ein alter Server) nennt nur `matrix_id`/`meshtastic_id`;
/// dann werden NUR diese beiden Arten ersetzt und alle anderen Wege
/// bleiben stehen. Ohne diese Unterscheidung loeschte der erste Abgleich
/// mit einer alten Gegenstelle jede Telefonnummer.
pub(crate) fn wege_von(eintrag: &Eintrag, vorhanden: Vec<Weg>) -> Vec<Weg> {
    match eintrag.felder.get("channels") {
        Some(Value::Array(liste)) => liste
            .iter()
            .filter_map(|w| serde_json::from_value::<Weg>(w.clone()).ok())
            .filter(|w| !w.wert.trim().is_empty())
            .collect(),
        _ => alte_kennungen(vorhanden, eintrag),
    }
}

/// `matrix_id`/`meshtastic_id` einer Gegenstelle ohne `channels` in die Wege
/// einsortieren: je genannte Art ersetzen, alles andere stehen lassen.
/// Genannt und leer heisst "geloescht"; gar nicht genannt heisst "unbekannt".
fn alte_kennungen(mut wege: Vec<Weg>, eintrag: &Eintrag) -> Vec<Weg> {
    for (feld, art) in [("matrix_id", "matrix"), ("meshtastic_id", "meshtastic")] {
        let Some(wert) = eintrag.felder.get(feld) else {
            continue;
        };
        let wert = wert.as_str().unwrap_or("").trim().to_string();
        let platz = wege.iter().position(|w| w.art == art);
        wege.retain(|w| w.art != art);

        if !wert.is_empty() {
            let neu = Weg {
                art: art.into(),
                beschriftung: None,
                wert,
            };
            match platz {
                Some(i) => wege.insert(i.min(wege.len()), neu),
                None => wege.push(neu),
            }
        }
    }

    wege
}

/// `Option<String>` als JSON: `null` statt einer leeren Zeichenkette.
///
/// Der Unterschied ist nicht kosmetisch. Drueben heisst `null` bei
/// `base_hash` "es gab nie einen gemeinsamen Stand" -- und das schaltet den
/// Vergleich in den vorsichtigen Modus. Eine leere Zeichenkette waere ein
/// Hash, der zu nichts passt, und ergaebe dasselbe Ergebnis aus dem falschen
/// Grund. Bei `folder` heisst `null` die Wurzel; `""` waere eine Mappe ohne
/// Namen.
pub(crate) fn nullbar(wert: Option<String>) -> Value {
    match wert {
        Some(s) => Value::String(s),
        None => Value::Null,
    }
}
