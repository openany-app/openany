// Die Druckfassung einer Mappe als PDF -- auf dem Gerät gesetzt.
//
// Jede Notiz der Mappe wird ein Kapitel, alphabetisch nach Titel. Steuern
// lässt sich die Reihenfolge über Vorsilben („01 Einleitung", „02 …"); Zahlen
// werden dabei als Zahlen verglichen, „2" kommt vor „10".
//
// UNTERMAPPEN (festgelegt mit Tiffy, 14.10.2026; abschaltbar im Dialog):
// Eine Untermappe wird eine Überschrift mit ihrem Namen, ihre Notizen rücken
// eine Ebene darunter -- „2 Grundlagen", „2.1 Begriffe". Notizen und
// Untermappen werden gemeinsam sortiert. Eine Notiz, die heißt wie ihre
// Untermappe, ist kein eigener Abschnitt, sondern der Text unter deren
// Überschrift. Leere Untermappen fallen weg. Kein Seitenumbruch zwischen
// Kapiteln: Der Text läuft durch.
//
// Wie notizAlsPdf.js nur über einen dynamischen Import (notesSources.js).
import { markdownZuTypst, bildpfade, ohneTitelzeile } from './markdownZuTypst';
import { bilderHolen } from './bilder';
import { druckfassungVorlage, SATZBREITE_CM } from './druckfassungVorlage';
import { setzen } from './typstSatz';
import { quellenHolen, bibBeilegen } from './quellenZuBib';

/** Ein Fehler, den die Oberfläche in eigenen Worten erklären kann. */
export class DruckfassungFehler extends Error {
  constructor(grund) {
    super(grund);
    this.grund = grund;
  }
}

/** So tief wird höchstens gelesen -- gegen Kreise in kaputten Daten. */
const HOECHSTE_TIEFE = 8;

/**
 * „01 Einleitung" -> „Einleitung": Die Vorsilbe ist nur zum Sortieren da;
 * die Nummer setzt Typst selbst. Bleibt nichts übrig, bleibt der Titel.
 *
 * HÖCHSTENS DREI ZIFFERN: „2026 Rückblick" ist ein Titel mit Jahreszahl,
 * keine Vorsilbe.
 */
export function ohneSortiervorsilbe(titel) {
  const ohne = String(titel ?? '').replace(/^\s*\d{1,3}(?:[.)_-]+\s*|\s+)/, '').trim();
  return ohne || String(titel ?? '').trim();
}

/** Titel alphabetisch, Zahlen als Zahlen, ohne Rücksicht auf Groß/klein. */
export function nachTitel(a, b, sprache = 'de') {
  return String(a.title ?? '').localeCompare(String(b.title ?? ''), sprache, { numeric: true, sensitivity: 'base' });
}

/** Heißt die Notiz wie die Mappe? („02 Grundlagen" und „Grundlagen" auch.) */
const gleicherName = (a, b) => {
  const n = (x) => ohneSortiervorsilbe(x).toLocaleLowerCase();
  return n(a) === n(b);
};

/**
 * Baum -> Abschnitte in Lesereihenfolge. Eine reine Funktion (prüfbar ohne
 * Browser).
 *
 * @param {{id, name, notizen: {titel, inhalt}[], unter?: object[]}} mappe
 * @param {string} [sprache]
 * @param {number} [ebene]  die Ebene der Abschnitte darin (1 = Kapitel)
 * @returns {{titel: string, roherTitel: string, ebene: number, inhalt: string, mappeId}[]}
 *          Untermappen-Überschriften tragen als `inhalt` ihren Einleitungstext
 *          (oder '')
 */
export function gliederung(mappe, sprache = 'de', ebene = 1) {
  const eintraege = [
    ...mappe.notizen.map((n) => ({ title: n.titel, notiz: n })),
    ...(mappe.unter ?? []).map((u) => ({ title: u.name, mappe: u })),
  ].sort((a, b) => nachTitel(a, b, sprache));

  const abschnitte = [];
  for (const e of eintraege) {
    if (e.notiz) {
      abschnitte.push({ titel: ohneSortiervorsilbe(e.notiz.titel), roherTitel: e.notiz.titel, ebene, inhalt: e.notiz.inhalt, mappeId: mappe.id });
      continue;
    }
    const u = e.mappe;
    const einleitung = u.notizen.find((n) => gleicherName(n.titel, u.name)) ?? null;
    const darunter = gliederung({ ...u, notizen: u.notizen.filter((n) => n !== einleitung) }, sprache, ebene + 1);
    if (!einleitung && !darunter.length) continue; // leer
    abschnitte.push({
      titel: ohneSortiervorsilbe(u.name),
      roherTitel: einleitung?.titel ?? u.name,
      ebene,
      inhalt: einleitung?.inhalt ?? '',
      mappeId: u.id,
    });
    abschnitte.push(...darunter);
  }
  return abschnitte;
}

async function alleNotizen(quelle, mappeId) {
  const liste = [];
  let seite = 1;
  // Die Liste ist seitenweise (50 je Seite); eine Mappe kann mehr haben.
  for (let schutz = 0; schutz < 200; schutz++) {
    const { data } = await quelle.getNotes(mappeId, seite);
    liste.push(...(data.items ?? []));
    // Die Webapp sagt `has_more`, die App nur `next_page`.
    if (!(data.has_more ?? data.next_page != null)) break;
    seite = data.next_page ?? seite + 1;
  }
  // Die Inhalte -- die Liste bringt nur Titel mit.
  const notizen = [];
  for (const eintrag of liste) {
    const { data } = await quelle.getNote(eintrag.id);
    notizen.push({ titel: data.title || eintrag.title || '', inhalt: data.content ?? '' });
  }
  return notizen;
}

/** Mappe samt (auf Wunsch) Untermappen lesen. */
async function baumLesen(quelle, mappe, alleMappen, tiefe = 0) {
  const unter = tiefe < HOECHSTE_TIEFE ? alleMappen.filter((f) => (f.parent_id ?? null) === mappe.id) : [];
  const baum = { id: mappe.id, name: mappe.name, notizen: await alleNotizen(quelle, mappe.id), unter: [] };
  for (const u of unter) baum.unter.push(await baumLesen(quelle, u, alleMappen, tiefe + 1));
  return baum;
}

/** Alle Mappen der Quelle (für die Untermappen); scheitert das, ohne. */
async function mappenListe(quelle) {
  try {
    const { data } = await quelle.getNoteFolders();
    return data.folders ?? [];
  } catch {
    return [];
  }
}

/**
 * @param {object} quelle   eine Notiz-Quelle aus notesSources.js
 * @param {{id: number, name: string}} mappe
 * @param {object} angaben  titel, untertitel, name, einrichtung, datum, zusatz,
 *                          inhaltsverzeichnis, untermappen
 * @returns {Promise<{data: Blob}>}
 * @throws {DruckfassungFehler} `leer`, wenn die Mappe keine Notizen hat
 */
export async function druckfassungAlsPdf(quelle, mappe, angaben) {
  const sprache = document.documentElement.lang || navigator.language || 'de';

  const alleMappen = angaben.untermappen === false ? [] : await mappenListe(quelle);
  const abschnitte = gliederung(await baumLesen(quelle, mappe, alleMappen), sprache);
  if (!abschnitte.length) throw new DruckfassungFehler('leer');

  // Bilder je Mappe: Anhänge gehören zu ihrer Mappe, und gleiche Pfade in
  // zwei Mappen sind zwei verschiedene Bilder.
  const bilder = new Map(); // mappeId -> (Pfad -> Datei)
  const dateien = new Map();
  for (const mappeId of new Set(abschnitte.map((a) => a.mappeId))) {
    const pfade = [...new Set(abschnitte.filter((a) => a.mappeId === mappeId).flatMap((a) => bildpfade(a.inhalt)))];
    const geholt = await bilderHolen(quelle, mappeId, pfade, SATZBREITE_CM, `m${mappeId}-`);
    bilder.set(mappeId, geholt.zuordnung);
    for (const [name, bytes] of geholt.dateien) dateien.set(name, bytes);
  }

  // Zitate: nur Schlüssel aus der Quellenliste der Mappe (QuellenDialog.vue).
  // Untermappen erben sie; eigene Listen von Untermappen gelten hier nicht.
  const { bib, schluessel } = await quellenHolen(quelle, mappe.id);
  const zitiert = new Set();

  const kapitel = abschnitte.map((a) => ({
    titel: a.titel,
    ebene: a.ebene,
    rumpf: a.inhalt
      ? markdownZuTypst(ohneTitelzeile(a.inhalt, a.roherTitel), {
        bild: (src) => bilder.get(a.mappeId)?.get(src) ?? null,
        obersteEbene: a.ebene + 1,
        quellen: schluessel,
        zitiert,
      })
      : '',
  }));

  const quelltext = druckfassungVorlage({
    angaben: { ...angaben, titel: angaben.titel || mappe.name },
    kapitel,
    sprache,
    literatur: bibBeilegen(dateien, bib, zitiert, sprache),
  });

  const pdf = await setzen(quelltext, dateien);
  return { data: new Blob([pdf], { type: 'application/pdf' }) };
}
