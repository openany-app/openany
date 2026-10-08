//! Der Aufbau der Datei -- und wie sie nachgezogen wird.
//!
//! **`PRAGMA user_version` und keine eigene Tabelle.** SQLite fuehrt die Zahl
//! ohnehin im Kopf der Datei; eine `schema_migrations`-Tabelle daneben waere
//! eine zweite Buchfuehrung ueber dieselbe Sache -- und die erste Abweichung
//! zwischen beiden ein Programm, das eine Wanderung zweimal laufen laesst.
//!
//! **Vorwaerts, nie zurueck.** Es gibt kein `down`. Ein ausgeliefertes
//! Programm rollt nicht zurueck: Auf dem Telefon liegt eine Datei, und wer
//! sie herunterstuft, verliert alles, was die neuere Fassung hineingeschrieben
//! hat. Deshalb bricht [`nachziehen`] bei einer unbekannt hohen Zahl ab,
//! statt zu raten.

use crate::{Ergebnis, SpeicherFehler};
use rusqlite::Connection;

/// Der Stand, den dieses Programm kennt. Jede Aenderung am Aufbau erhoeht ihn
/// und bekommt einen Schritt in [`SCHRITTE`].
pub const STAND: i64 = 22;

/// Ein Schritt je Stand. Der Index ist der Stand davor: `SCHRITTE[0]` fuehrt
/// von 0 nach 1.
const SCHRITTE: [&str; STAND as usize] = [
    SCHRITT_1, SCHRITT_2, SCHRITT_3, SCHRITT_4, SCHRITT_5, SCHRITT_6, SCHRITT_7, SCHRITT_8,
    SCHRITT_9, SCHRITT_10, SCHRITT_11, SCHRITT_12, SCHRITT_13, SCHRITT_14, SCHRITT_15, SCHRITT_16,
    SCHRITT_17, SCHRITT_18, SCHRITT_19, SCHRITT_20, SCHRITT_21, SCHRITT_22,
];

pub fn stand(db: &Connection) -> Ergebnis<i64> {
    Ok(db.query_row("PRAGMA user_version", [], |z| z.get(0))?)
}

/// Die Datei auf [`STAND`] bringen.
pub fn nachziehen(db: &Connection) -> Ergebnis<()> {
    let gefunden = stand(db)?;

    if gefunden > STAND {
        return Err(SpeicherFehler::ZuNeu {
            gefunden,
            bekannt: STAND,
        });
    }

    for (i, schritt) in SCHRITTE.iter().enumerate().skip(gefunden as usize) {
        // In EINER Transaktion je Schritt: Ein Abbruch mittendrin -- Akku
        // leer, Programm weggewischt -- darf keine halb umgebaute Datei
        // hinterlassen, die beim naechsten Start weder das eine noch das
        // andere ist.
        db.execute_batch(&format!(
            "BEGIN; {schritt} PRAGMA user_version = {}; COMMIT;",
            i + 1
        ))?;
    }

    Ok(())
}

/// Stand 1 -- Notizen, Kalender, Termine, Kontakte, Protokoll, Marken,
/// Urspruenge.
///
/// **Zeiten als ISO-8601-Text und nicht als Zahl.** Dieselbe Schreibweise wie
/// ueber der Leitung; ein Umrechnen an jeder Grenze ist eine Gelegenheit,
/// sich um eine Stunde zu irren. Und Text laesst sich in der Datei lesen,
/// wenn jemand einmal nachsehen muss, was das Programm wirklich gespeichert
/// hat.
///
/// **Papierkorb als Spalte und nicht als Loeschung.** Der Server unterscheidet
/// `trash` und `delete`, und die Verwechslung ist die teuerste an der ganzen
/// Strecke: Ein Griff in den Papierkorb auf der einen Seite wuerde auf der
/// anderen zur endgueltigen Vernichtung.
const SCHRITT_1: &str = r#"
CREATE TABLE notizen (
    zk_id         TEXT PRIMARY KEY,
    titel         TEXT NOT NULL DEFAULT '',
    inhalt        TEXT NOT NULL DEFAULT '',
    -- Der Mappenpfad, wie er ueber die Leitung geht -- NICHT eine lokale Id.
    -- NULL heisst Wurzel.
    mappe         TEXT,
    papierkorb_at TEXT,
    geaendert_at  TEXT NOT NULL
);

CREATE TABLE kalender (
    uuid          TEXT PRIMARY KEY,
    name          TEXT NOT NULL DEFAULT '',
    farbe         TEXT NOT NULL DEFAULT '',
    papierkorb_at TEXT,
    geaendert_at  TEXT NOT NULL
);

CREATE TABLE termine (
    uuid          TEXT PRIMARY KEY,
    -- OHNE Fremdschluessel, und das ist Absicht: Beim Ziehen kann der Termin
    -- vor seinem Kalender ankommen. Ein Fremdschluessel wuerde ihn dann
    -- abweisen; so wird er abgelegt und beim naechsten Lauf sichtbar. Was er
    -- NICHT darf, ist in irgendeinen Kalender wandern -- das entscheidet der
    -- Laeufer, nicht die Tabelle.
    kalender_uuid TEXT NOT NULL,
    titel         TEXT NOT NULL DEFAULT '',
    beschreibung  TEXT NOT NULL DEFAULT '',
    ort           TEXT NOT NULL DEFAULT '',
    beginn        TEXT NOT NULL DEFAULT '',
    ende          TEXT NOT NULL DEFAULT '',
    ganztags      INTEGER NOT NULL DEFAULT 0,
    rrule         TEXT NOT NULL DEFAULT '',
    rrule_bis     TEXT NOT NULL DEFAULT '',
    exdates       TEXT NOT NULL DEFAULT '',
    papierkorb_at TEXT,
    geaendert_at  TEXT NOT NULL
);
CREATE INDEX termine_nach_kalender ON termine (kalender_uuid);
CREATE INDEX termine_nach_beginn   ON termine (beginn);

CREATE TABLE kontakte (
    uuid          TEXT PRIMARY KEY,
    anzeigename   TEXT NOT NULL DEFAULT '',
    -- Beide NULL-bar: Ein Kontakt kann ueber genau einen Weg erreichbar sein.
    -- Was hier NICHT steht, ist die im lokalen Netz gefundene Adresse -- sie
    -- gilt nur hier und jetzt, und in einer abgeglichenen Tabelle wanderte
    -- sie als Muell auf die anderen Geraete. Erreichbarkeit ist
    -- Laufzeitzustand, kein Stammdatum.
    matrix_id     TEXT,
    meshtastic_id TEXT,
    papierkorb_at TEXT,
    geaendert_at  TEXT NOT NULL
);

-- Was sich HIER geaendert hat. Das Gegenstueck zu openanys `change_log`.
CREATE TABLE aenderungen (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    art         TEXT NOT NULL,
    schluessel  TEXT NOT NULL,
    was         TEXT NOT NULL,
    at          TEXT NOT NULL
);
CREATE INDEX aenderungen_nach_sache ON aenderungen (art, schluessel);

-- Wie weit ist der Abgleich mit einer Gegenstelle?
--
-- ZWEI Marken und nicht eine: Was von dort gelesen und was dorthin geschickt
-- wurde, gehen unterschiedlich weit. Mit einer gemeinsamen Marke risse ein
-- abgebrochener Lauf beide Richtungen mit.
CREATE TABLE marken (
    gegenstelle    TEXT PRIMARY KEY,
    fremde_marke   INTEGER NOT NULL DEFAULT 0,
    eigene_marke   INTEGER NOT NULL DEFAULT 0,
    letzter_lauf   TEXT,
    letzter_fehler TEXT
);

-- Der zuletzt beidseitig bekannte Stand je Sache.
--
-- DIE GEGENSTELLE STEHT IM SCHLUESSEL -- siehe den Kopf von lib.rs. Sie
-- spaeter nachzuziehen entwertete jeden vorhandenen Ursprung, und der erste
-- Abgleich danach machte aus jeder Notiz eine Konfliktkopie.
CREATE TABLE ursprung (
    gegenstelle TEXT NOT NULL,
    art         TEXT NOT NULL,
    schluessel  TEXT NOT NULL,
    abdruck     TEXT NOT NULL,
    PRIMARY KEY (gegenstelle, art, schluessel)
);
"#;

/// Stand 2 -- leer angelegte Notiz-Mappen (15.09.2026).
///
/// **Nur hier, nicht im Abgleich.** Eine Mappe ist drueben kein eigener
/// Eintrag im Delta, sondern der Pfad an der Notiz. Ohne diese Tabelle haette
/// eine eben angelegte, noch leere Mappe keinen Ort und waere beim naechsten
/// Oeffnen verschwunden. Sobald eine Notiz darin liegt, traegt ihr Pfad sie.
const SCHRITT_2: &str = r#"
CREATE TABLE mappen (
    pfad        TEXT PRIMARY KEY,
    angelegt_at TEXT NOT NULL
);
"#;

/// Stand 3 -- vollstaendige Kontakte (15.09.2026).
///
/// Wege als JSON-Liste in `kontakte.wege`, der Abdruck des Fotos, und die
/// Foto-Bytes in einer eigenen Tabelle -- sie sind hundertmal groesser als
/// der Rest und sollen nicht bei jeder Liste mitgelesen werden.
///
/// **Die beiden alten Kennungen wandern mit**, sonst verloere ein Programm
/// beim Nachziehen jede Matrix- und Meshtastic-Kennung, die es schon kannte.
/// Die Spalten bleiben stehen (SQLite entfernt sie nicht gern), werden aber
/// nicht mehr geschrieben.
///
/// `hinaus` in `kontaktfotos`: 1 = hier geaendert, beim naechsten Abgleich
/// hinueberzuschicken; `bytes` NULL mit `hinaus` 1 = hier entfernt.
const SCHRITT_3: &str = r#"
ALTER TABLE kontakte ADD COLUMN wege TEXT NOT NULL DEFAULT '[]';
ALTER TABLE kontakte ADD COLUMN foto_abdruck TEXT;

UPDATE kontakte SET wege = CASE
    WHEN coalesce(matrix_id, '') <> '' AND coalesce(meshtastic_id, '') <> '' THEN json_array(
        json_object('kind', 'matrix', 'label', NULL, 'value', matrix_id),
        json_object('kind', 'meshtastic', 'label', NULL, 'value', meshtastic_id))
    WHEN coalesce(matrix_id, '') <> '' THEN json_array(
        json_object('kind', 'matrix', 'label', NULL, 'value', matrix_id))
    WHEN coalesce(meshtastic_id, '') <> '' THEN json_array(
        json_object('kind', 'meshtastic', 'label', NULL, 'value', meshtastic_id))
    ELSE '[]'
END;

CREATE TABLE kontaktfotos (
    kontakt_uuid TEXT PRIMARY KEY,
    abdruck      TEXT,
    bytes        BLOB,
    hinaus       INTEGER NOT NULL DEFAULT 0
);
"#;

/// Stand 4 -- mehr als zwei eigene Geraete (15.09.2026).
///
/// `aenderungen.herkunft`: von welcher Gegenstelle eine angenommene Aenderung
/// kam (NULL = hier entstanden). Sie geht an alle anderen weiter, nur nicht
/// dorthin zurueck.
///
/// `ursprung.at`: wann ein Ursprung zuletzt gemerkt wurde -- fuer
/// `juengster_ursprung`. Vorhandene bleiben ohne Zeit und zaehlen als die
/// aeltesten.
const SCHRITT_4: &str = r#"
ALTER TABLE aenderungen ADD COLUMN herkunft TEXT;
ALTER TABLE ursprung ADD COLUMN at TEXT;
"#;

/// Stand 5 -- Dateien und Ordner (15.09.2026, Phase 3 Stufe A).
///
/// Nur, WAS es gibt; die Bytes liegen nach Abdruck in `inhalte/`
/// (`Inhalte`). `eltern` ist die uuid des Ordners, NULL die oberste Ebene.
const SCHRITT_5: &str = r#"
CREATE TABLE dateien (
    uuid          TEXT PRIMARY KEY,
    zone          TEXT NOT NULL DEFAULT 'files',
    ist_ordner    INTEGER NOT NULL DEFAULT 0,
    eltern        TEXT,
    name          TEXT NOT NULL,
    groesse       INTEGER NOT NULL DEFAULT 0,
    mime          TEXT NOT NULL DEFAULT '',
    abdruck       TEXT,
    papierkorb_at TEXT,
    geaendert_at  TEXT NOT NULL
);
CREATE INDEX dateien_im_ordner ON dateien (zone, eltern);
CREATE INDEX dateien_nach_abdruck ON dateien (abdruck);
"#;

/// Stand 6 -- die Galerie (15.09.2026, Phase 3 Stufe C).
///
/// `medien.album` NULL = Bild auf der obersten Ebene. `exif` wie drueben in
/// `custom_properties.exif`, als JSON-Text. `vorschau` ist der Abdruck des
/// Vorschaubilds -- ein eigener Inhalt, der immer mitreist.
const SCHRITT_6: &str = r#"
CREATE TABLE alben (
    uuid          TEXT PRIMARY KEY,
    name          TEXT NOT NULL,
    beschreibung  TEXT NOT NULL DEFAULT '',
    eltern        TEXT,
    papierkorb_at TEXT,
    geaendert_at  TEXT NOT NULL
);
CREATE INDEX alben_unter ON alben (eltern);

CREATE TABLE medien (
    uuid          TEXT PRIMARY KEY,
    album         TEXT,
    name          TEXT NOT NULL,
    mime          TEXT NOT NULL DEFAULT '',
    groesse       INTEGER NOT NULL DEFAULT 0,
    abdruck       TEXT,
    vorschau      TEXT,
    exif          TEXT NOT NULL DEFAULT '',
    papierkorb_at TEXT,
    geaendert_at  TEXT NOT NULL
);
CREATE INDEX medien_im_album ON medien (album);
CREATE INDEX medien_nach_abdruck ON medien (abdruck);
"#;

/// Stand 7 -- wer welche Inhalte hat (16.09.2026, Phase 3 Stufe D).
///
/// **Die Frage, die der Server nicht beantworten kann.** Zwei Geraete
/// benennen Inhalte nach ihrem Abdruck und koennen einander deshalb fragen:
/// "welche dieser sha256 hast du?" (`inhalte_da`). Der Server kennt diese
/// Sicht gar nicht -- bei ihm haengen Bytes an einer Datei oder einem Bild,
/// und einen Abdruck fuehrt er nirgends. Gefragt werden kann er also nicht.
///
/// Also wird gemerkt, was man selbst weiss: Ein Inhalt liegt drueben, wenn
/// er von dort kam oder erfolgreich dorthin ging. Mehr behauptet diese
/// Tabelle nicht.
///
/// **Sie ist ein Gedaechtnis, kein Stammdatum.** Geht sie verloren, wird
/// hoechstens etwas ein zweites Mal geschickt -- der Server erkennt dieselbe
/// Datei am Abdruck wieder und nimmt sie nicht doppelt an. Sie reist deshalb
/// auch nicht im Abgleich mit: Was Geraet A ueber den Server weiss, gilt
/// nicht fuer Geraet B.
///
/// **Ein Eintrag verschwindet nur mit seiner Gegenstelle.** Dass drueben
/// jemand loescht, erfaehrt diese Seite ueber das Aenderungsprotokoll --
/// und raeumt den Vermerk dann selbst weg.
const SCHRITT_7: &str = r#"
CREATE TABLE inhalt_dort (
    basis   TEXT NOT NULL,
    abdruck TEXT NOT NULL,
    seit    TEXT NOT NULL,
    PRIMARY KEY (basis, abdruck)
);
"#;

/// Stand 8 -- Notiz-Anhaenge (16.09.2026).
///
/// Ein Anhang ist eine ZUORDNUNG, keine Datei: Der Notiztext bleibt
/// Standard-Markdown (`![](bild.png)`), und diese Tabelle sagt, worauf der
/// relative Pfad zeigt. Die Bytes gehoeren dem Ziel -- einem Bild oder einer
/// Datei -- und reisen einmal, ueber deren eigenen Eintrag.
///
/// **Geschluesselt nach MAPPENPFAD, nicht nach einer Mappen-Id.** So kommt er
/// ueber die Leitung (`DeltaFeed` macht ihn ausdruecklich portabel), und so
/// heisst die Mappe hier ohnehin: `mappen.pfad` ist der Primaerschluessel.
/// Eine Id waere eine zweite Wahrheit ueber denselben Ort.
///
/// **Ohne Papierkorb.** Drueben hat `note_assets` keinen: Die Zuordnung ist da
/// oder sie ist weg. `DeltaFeed::imPapierkorb` haelt das ausdruecklich aus.
const SCHRITT_8: &str = r#"
CREATE TABLE anhaenge (
    mappe     TEXT NOT NULL,
    pfad      TEXT NOT NULL,
    ziel_art  TEXT NOT NULL,
    ziel_uuid TEXT NOT NULL,
    groesse   INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (mappe, pfad)
);
"#;

/// Stand 9 -- die Abdruecke des Anhang-ZIELS (16.09.2026).
///
/// Ein Anhang traegt keine Bytes, er zeigt auf welche. Fuer ein BILD ist das
/// besonders bitter: Drueben haelt "Regel 1" (jede Datei genau einmal) es aus
/// dem `media`-Strom heraus -- es reist als Anhang. Dieses Programm kennt die
/// Sache also gar nicht und haette ohne Abdruck keine Adresse dafuer.
///
/// Beim ersten Lauf kamen 26 Zuordnungen an und kein einziges Bild: Die Notiz
/// wusste, dass dort etwas hingehoert, und konnte es nicht zeigen.
///
/// **Zwei Abdruecke, und das ist kein Luxus.** Im Flusstext gehoert die
/// Vorschau -- klein, und ein Geraet "bei Bedarf" hat sie immer. Das Original
/// erst beim Oeffnen.
///
/// Als eigener Schritt und nicht in SCHRITT_8 hinein: Der stand schon auf
/// einem Geraet, und eine Datei, die sich fuer Stand 8 haelt, muss den Weg
/// nach 9 gehen duerfen.
const SCHRITT_9: &str = r#"
ALTER TABLE anhaenge ADD COLUMN abdruck TEXT;
ALTER TABLE anhaenge ADD COLUMN vorschau TEXT;
"#;

/// Stand 10 -- abonnierte Kalender (17.09.2026).
///
/// **Ein Abo ist eine ADRESSE, kein Inhalt.** So steht es seit jeher drueben
/// in `Calendar::shouldRecordChanges`: Ein abonnierter Kalender bleibt aus dem
/// Abgleich heraus, weil `performSync()` bei jedem Abruf alle seine Termine
/// loescht und aus dem Feed neu anlegt -- ihn durch die Leitung zu schieben
/// waere bei jedem Abruf der ganze Kalender.
///
/// Der Satz dort endet mit: "Das Programm auf dem Telefon abonniert den Feed
/// SELBST, wenn es ihn will." Gebaut war das nie. Ein Termin aus einem
/// FamilyWall-Kalender erreichte deshalb kein Geraet -- mit openany-Konto so
/// wenig wie ohne, denn ohne Konto gab es das Feld gar nicht.
///
/// **Und es ist der Fall, fuer den sich regelmaessiges Nachfragen ueberhaupt
/// lohnt.** Notizen und Dateien aendert der Mensch selbst; ein fremder
/// Kalender aendert sich, waehrend er schlaeft.
const SCHRITT_10: &str = r#"
ALTER TABLE kalender ADD COLUMN abo_url TEXT;
ALTER TABLE kalender ADD COLUMN zuletzt_geholt TEXT;
"#;

/// Stand 11 -- Nachrichten (Phase 4, 17.09.2026).
///
/// **Kein Eintrag im Protokoll, keine Marke, kein Ursprung.** Nachrichten
/// reisen nicht ueber den Abgleich: Sie kommen vom Homeserver, und jedes
/// Geraet ist dort ein eigenes Matrix-Geraet mit eigenen Schluesseln. Sie
/// ueber openany.de zu schieben hiesse, sie dort lesbar abzulegen -- genau
/// das, was die App anders macht als die Webapp.
///
/// **Die Ereignis-Id ist der Schluessel.** Das eigene Echo kommt mit
/// derselben Id zurueck, unter der das Senden die Zeile schon angelegt hat;
/// `INSERT OR IGNORE` macht daraus keine zweite.
///
/// **Geloescht heisst hier: nur auf diesem Geraet weg** (`geloescht_at`).
/// Ein Matrix-Raum vergisst nichts, und der naechste Abgleich brachte die
/// Zeile sonst zurueck.
const SCHRITT_11: &str = r#"
CREATE TABLE nachrichten (
    event_id     TEXT PRIMARY KEY,
    raum         TEXT NOT NULL,
    gegenueber   TEXT,
    absender     TEXT NOT NULL,
    von_mir      INTEGER NOT NULL DEFAULT 0,
    text         TEXT NOT NULL,
    zeit         TEXT NOT NULL,
    gelesen_at   TEXT,
    geloescht_at TEXT
);
CREATE INDEX nachrichten_zeit ON nachrichten (zeit);
"#;

/// Stand 12 -- Projekte und ihre Sachen (22.09.2026).
///
/// **EINE Tabelle fuer alles, was in einem Projekt liegt.** Drueben stehen
/// Boards, Spalten und Karten in eigenen Tabellen -- dort haengen
/// Fremdschluessel, Zaehler und Abfragen der Webapp daran. Hier zeichnet und
/// aendert das Programm nur; zehn Schema-Staende fuer zehn Planungstypen
/// waeren zehnmal derselbe Handgriff. Was eine Art ausmacht, sagt der Server
/// in seiner Karte (`Abgleichsarten`), und die Felder reisen als JSON.
///
/// **Das Projekt ist eine Gegenstelle.** Sein Strom hat eigene Marken --
/// dafuer braucht es keine neue Tabelle: `marken` ist nach der Gegenstelle
/// geschluesselt, und die heisst hier `<basis>#projekt:<uuid>`.
///
/// **Das Protokoll bekommt eine Spalte.** Ohne sie schoebe der persoenliche
/// Lauf Karten an openany.de und der Projektlauf Notizen an das Projekt --
/// beide Male an die falsche Adresse. `projekt IS NULL` heisst: gehoert mir,
/// nicht einem Projekt.
const SCHRITT_12: &str = r#"
CREATE TABLE projekte (
    uuid        TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    rolle       TEXT NOT NULL DEFAULT 'member',
    geaendert_at TEXT NOT NULL DEFAULT ''
);

CREATE TABLE projektsachen (
    projekt      TEXT NOT NULL,
    art          TEXT NOT NULL,
    uuid         TEXT NOT NULL,
    eltern       TEXT,
    felder       TEXT NOT NULL DEFAULT '{}',
    geaendert_at TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (projekt, uuid)
);
CREATE INDEX projektsachen_nach_art ON projektsachen (projekt, art);
CREATE INDEX projektsachen_nach_eltern ON projektsachen (projekt, eltern);

ALTER TABLE aenderungen ADD COLUMN projekt TEXT;
CREATE INDEX aenderungen_nach_projekt ON aenderungen (projekt, id);
"#;

/// Stand 13 -- Anhänge an Nachrichten (28.09.2026, docs/plan-email-pgp.md).
///
/// **Eine Spalte mit JSON, keine eigene Tabelle.** Eine Nachricht trägt
/// höchstens einen Anhang (so schickt Matrix sie), und was darin steht --
/// Name, Typ, Größe, Quelle -- liest nur die App selbst. Die Bytes liegen
/// nicht hier: Das SDK holt sie bei Bedarf und hält sie in seinem eigenen,
/// verschlüsselten Zwischenspeicher.
const SCHRITT_13: &str = r#"
ALTER TABLE nachrichten ADD COLUMN anhang TEXT;
"#;

/// Stand 14 -- Mails (29.09.2026, docs/plan-email-pgp.md, Schritt 2).
///
/// **Eine eigene Tabelle, nicht `nachrichten`.** Eine Mail hat Betreff,
/// Ordner, UID und Message-ID; eine Matrix-Nachricht nichts davon. Beides in
/// eine Tabelle zu zwingen hiesse, die Hälfte der Spalten je Zeile leer zu
/// lassen. Der Verlauf fügt beide erst in der Anzeige zusammen.
///
/// Die Anhänge liegen in der Inhaltsablage; die Zeile nennt ihren Abdruck.
/// `mail_staende` merkt sich je Ordner, bis wohin gelesen ist.
const SCHRITT_14: &str = r#"
CREATE TABLE mails (
    id              TEXT PRIMARY KEY,
    ordner          TEXT NOT NULL DEFAULT '',
    uidvalidity     INTEGER NOT NULL DEFAULT 0,
    uid             INTEGER NOT NULL DEFAULT 0,
    message_id      TEXT NOT NULL DEFAULT '',
    in_reply_to     TEXT,
    referenzen      TEXT NOT NULL DEFAULT '[]',
    von_mir         INTEGER NOT NULL DEFAULT 0,
    gegenueber      TEXT NOT NULL DEFAULT '',
    gegenueber_name TEXT NOT NULL DEFAULT '',
    betreff         TEXT NOT NULL DEFAULT '',
    text            TEXT NOT NULL DEFAULT '',
    zeit            TEXT NOT NULL,
    gelesen_at      TEXT,
    geloescht_at    TEXT,
    anhaenge        TEXT NOT NULL DEFAULT '[]'
);
CREATE INDEX mails_zeit ON mails (zeit);
CREATE INDEX mails_message_id ON mails (message_id);
CREATE INDEX mails_ort ON mails (ordner, uidvalidity, uid);

CREATE TABLE mail_staende (
    ordner      TEXT PRIMARY KEY,
    uidvalidity INTEGER NOT NULL,
    letzte_uid  INTEGER NOT NULL
);
"#;

/// Stand 15 -- mehrere Postfächer. Mails und Lesestände gehören jetzt zu
/// einem Postfach (seiner Adresse). Was schon da ist, bekommt `''` und wird
/// beim ersten Start dem bisher einzigen Postfach zugeordnet
/// (`mails_zuordnen`).
const SCHRITT_15: &str = r#"
ALTER TABLE mails ADD COLUMN postfach TEXT NOT NULL DEFAULT '';
DROP INDEX mails_ort;
CREATE INDEX mails_ort ON mails (postfach, ordner, uidvalidity, uid);

CREATE TABLE mail_staende_neu (
    postfach    TEXT NOT NULL DEFAULT '',
    ordner      TEXT NOT NULL,
    uidvalidity INTEGER NOT NULL,
    letzte_uid  INTEGER NOT NULL,
    PRIMARY KEY (postfach, ordner)
);
INSERT INTO mail_staende_neu (ordner, uidvalidity, letzte_uid)
    SELECT ordner, uidvalidity, letzte_uid FROM mail_staende;
DROP TABLE mail_staende;
ALTER TABLE mail_staende_neu RENAME TO mail_staende;
"#;

/// Stand 16 -- mehrere Matrix-Konten. Eine Nachricht gehört zu einem Konto
/// (seiner Kennung), und der Schlüssel wird `(konto, event_id)`: Schreiben
/// sich zwei Konten auf demselben Gerät, kommt DASSELBE Ereignis bei beiden
/// an -- einmal als eigenes, einmal als fremdes. Was schon da ist, bekommt
/// `''` und wird beim Start dem bisher einzigen Konto zugeordnet.
const SCHRITT_16: &str = r#"
CREATE TABLE nachrichten_neu (
    konto        TEXT NOT NULL DEFAULT '',
    event_id     TEXT NOT NULL,
    raum         TEXT NOT NULL,
    gegenueber   TEXT,
    absender     TEXT NOT NULL,
    von_mir      INTEGER NOT NULL DEFAULT 0,
    text         TEXT NOT NULL,
    zeit         TEXT NOT NULL,
    gelesen_at   TEXT,
    geloescht_at TEXT,
    anhang       TEXT,
    PRIMARY KEY (konto, event_id)
);
INSERT INTO nachrichten_neu
    (event_id, raum, gegenueber, absender, von_mir, text, zeit, gelesen_at, geloescht_at, anhang)
    SELECT event_id, raum, gegenueber, absender, von_mir, text, zeit, gelesen_at, geloescht_at, anhang
    FROM nachrichten;
DROP TABLE nachrichten;
ALTER TABLE nachrichten_neu RENAME TO nachrichten;
CREATE INDEX nachrichten_zeit ON nachrichten (zeit);
"#;

/// Stand 17 -- öffentliche OpenPGP-Schlüssel der Gegenüber (Schritt 3a).
///
/// **Nur auf diesem Gerät, nicht im Abgleich** (Tiffy, 29.09.2026): Sonst
/// sähe openany.de, mit wem verschlüsselt geschrieben wird. Einer je
/// Adresse; kommt ein anderer, ersetzt er den alten, und `vorher` behält
/// dessen Fingerabdruck, damit die Oberfläche den Wechsel zeigen kann.
const SCHRITT_17: &str = r#"
CREATE TABLE pgp_schluessel (
    adresse         TEXT PRIMARY KEY,
    fingerabdruck   TEXT NOT NULL,
    oeffentlich     TEXT NOT NULL,
    quelle          TEXT NOT NULL,
    zuerst_at       TEXT NOT NULL,
    aktualisiert_at TEXT NOT NULL,
    vorher          TEXT,
    geaendert_at    TEXT
);
"#;

/// Stand 18 -- verschlüsselte Mails (Schritt 3b). `pgp`: `verschluesselt`,
/// oder `unlesbar`, solange der passende Schlüssel fehlt (dann steht die
/// OpenPGP-Nachricht in `text`, damit sie sich später noch entschlüsseln
/// lässt). `signatur`: `gueltig`, `ungueltig`, `unbekannt` oder leer.
const SCHRITT_18: &str = r#"
ALTER TABLE mails ADD COLUMN pgp TEXT;
ALTER TABLE mails ADD COLUMN signatur TEXT;
"#;

/// Stand 19 -- lokale Projekte (01.10.2026,
/// docs/konzept-lokale-mitgliedschaften.md). Ein Projekt, das auf diesem
/// Gerät entstand, trägt seine unterschriebene Mitgliederliste als JSON;
/// bei einem Projekt vom Server bleibt die Spalte leer -- dort entscheidet
/// der Server, wer dabei ist.
const SCHRITT_19: &str = r#"
ALTER TABLE projekte ADD COLUMN mitgliederliste TEXT;
"#;

/// Stand 20 -- Freigaben in lokale Projekte (01.10.2026): die
/// unterschriebenen Eintraege als JSON. Was andere freigegeben haben, liegt
/// als Projektsache der Art `geteilte_notiz` daneben.
const SCHRITT_20: &str = r#"
ALTER TABLE projekte ADD COLUMN freigaben TEXT;
"#;

/// Stand 21 -- der Text der Dateien, fuer die Suche im Inhalt (Speicher-Suche
/// Stufe 2, 02.10.2026). Ausgelesen in der Oberflaeche (pdf.js), hier nur
/// abgelegt -- zum Abdruck der Datei, aus der er stammt: Aendert sie sich,
/// passt er nicht mehr und sie wird neu gelesen. Ein Index, kein Inhalt: Er
/// reist nicht im Abgleich, jedes Geraet liest selbst.
const SCHRITT_21: &str = r#"
CREATE TABLE dateitexte (
    uuid    TEXT PRIMARY KEY,
    abdruck TEXT NOT NULL,
    stand   TEXT NOT NULL,
    text    TEXT
);
"#;

/// Stand 22 -- die Fassung des Lesers: Was ein aelterer als `unlesbar`
/// ablegte, liest ein neuerer noch einmal (02.10.2026: pdf.js 6 hatte die
/// Auslese jedes PDFs scheitern lassen).
const SCHRITT_22: &str = r#"
ALTER TABLE dateitexte ADD COLUMN version INTEGER NOT NULL DEFAULT 1;
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn es_gibt_zu_jedem_stand_einen_schritt() {
        // Wer STAND erhoeht und den Schritt vergisst, bekommt sonst eine
        // Datei, die sich fuer neu haelt und leer ist.
        assert_eq!(SCHRITTE.len(), STAND as usize);
    }

    #[test]
    fn die_alten_kennungen_wandern_in_die_wege() {
        let db = Connection::open_in_memory().unwrap();
        db.execute_batch(&format!("{SCHRITT_1} {SCHRITT_2} PRAGMA user_version = 2;"))
            .unwrap();
        db.execute_batch(
            "INSERT INTO kontakte (uuid, anzeigename, matrix_id, meshtastic_id, geaendert_at)
             VALUES ('a', 'Beide', '@a:m.org', '!1234', 'x'),
                    ('b', 'Nur Funk', NULL, '!5678', 'x'),
                    ('c', 'Keiner', '', NULL, 'x');",
        )
        .unwrap();

        nachziehen(&db).unwrap();

        let wege = |uuid: &str| -> String {
            db.query_row("SELECT wege FROM kontakte WHERE uuid = ?1", [uuid], |z| {
                z.get(0)
            })
            .unwrap()
        };
        assert_eq!(
            wege("a"),
            r#"[{"kind":"matrix","label":null,"value":"@a:m.org"},{"kind":"meshtastic","label":null,"value":"!1234"}]"#
        );
        assert_eq!(
            wege("b"),
            r#"[{"kind":"meshtastic","label":null,"value":"!5678"}]"#
        );
        assert_eq!(wege("c"), "[]");
    }

    #[test]
    fn nachziehen_ist_wiederholbar() {
        let db = Connection::open_in_memory().unwrap();

        nachziehen(&db).unwrap();
        nachziehen(&db).unwrap();

        assert_eq!(stand(&db).unwrap(), STAND);
    }

    #[test]
    fn alle_vierzehn_tabellen_stehen() {
        let db = Connection::open_in_memory().unwrap();
        nachziehen(&db).unwrap();

        let mut abfrage = db
            .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
            .unwrap();
        let namen: Vec<String> = abfrage
            .query_map([], |z| z.get::<_, String>(0))
            .unwrap()
            .map(Result::unwrap)
            .filter(|n| !n.starts_with("sqlite_"))
            .collect();

        assert_eq!(
            namen,
            vec![
                "aenderungen",
                "alben",
                "anhaenge",
                "dateien",
                "dateitexte",
                "inhalt_dort",
                "kalender",
                "kontakte",
                "kontaktfotos",
                "mail_staende",
                "mails",
                "mappen",
                "marken",
                "medien",
                "nachrichten",
                "notizen",
                "pgp_schluessel",
                "projekte",
                "projektsachen",
                "termine",
                "ursprung",
            ]
        );
    }
}
