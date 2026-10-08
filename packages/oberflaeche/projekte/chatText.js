// Chat-Nachrichten in Stücke zerlegen: Text, [[Wikilinks]] und Web-Adressen.
//
// Bewusst ein Zerleger und kein HTML-Erzeuger: Die Anzeige rendert die Stücke
// über v-for, nie über v-html. Damit bleibt die Maskierung Vues Aufgabe und
// XSS baulich ausgeschlossen – auch wenn hier künftig jemand etwas ergänzt.
//
// Frei von Vue und DOM, damit es in der node-Umgebung testbar ist (wie
// folderTree.js und date.js).

import { PLANNING_CHAT_KINDS } from './planningTypes';

// Dieselbe Wikilink-Form wie im Backend (Note::WIKILINK_PATTERN) und eine
// Web-Adresse. Als EIN Ausdruck, damit die Reihenfolge im Text erhalten
// bleibt, statt zweimal zu suchen und die Treffer zusammenzufädeln.
const STUECKE = /\[\[([^[\]\n]+)\]\]|(https?:\/\/[^\s<>"']+)/gu;

/**
 * Arten, deren Sichtbarkeit an einer FREIGABE hängt statt an der
 * Projekt-Mitgliedschaft. Der Unterschied ist im Chat sichtbar: Bleibt ein
 * Verweis tot, ist bei diesen Arten die wahrscheinliche Ursache eine fehlende
 * Freigabe, bei den übrigen ein gelöschtes oder fremdes Ziel. „Nicht
 * freigegeben" wäre an einem Board schlicht falsch.
 *
 * Steht vor GETIPPT, weil die Regex daraus gebaut wird.
 */
export const SHARED_KINDS = ['note', 'folder', 'file', 'album', 'photo'];

/** Kennung + Leerzeichen am Anfang eines Linkziels (kanonische Form). */
const KENNUNG_VORN = /^\d{14}\s+/u;

/**
 * Getipptes Linkziel: `art:id`, z. B. [[board:12|Einkauf]].
 *
 * Warum eine Id und nicht der Name: Mit mehreren Zielarten wird ein Name
 * mehrdeutig – heißen ein Board und eine Notiz beide „Einkauf", meint
 * [[Einkauf]] niemand weiß was. Und Umbenennen bräche jeden Verweis. Diese
 * Form schreibt niemand von Hand; sie entsteht aus der Vorschlagsliste.
 *
 * Die Artnamen sind englisch wie alle Bezeichner (siehe README) – es ist ein
 * Datenformat, das dauerhaft in den Nachrichten steht, kein Anzeigetext.
 *
 * Die Planungs-Arten kommen aus planningKinds.js, die freigabegebundenen aus
 * SHARED_KINDS weiter unten – zusammen sind das dieselben Arten, die
 * App\Support\PlanningTypes und ChatLinks im Backend kennen.
 *
 * Die längste Art steht vorn: Die Alternative greift von links, und `place`
 * würde `places:3` sonst nach dem Wort abschneiden, wo dann kein ':' mehr
 * folgt. Nach Länge zu sortieren erledigt das für jede künftige Art mit.
 */
// Ohne Rücksicht auf Groß-/Kleinschreibung, weil das Backend seine Karte
// kleingeschrieben führt (linkKey): Ein [[Board:12]] fände dort sonst ein
// Ziel, das die Anzeige hier nicht als getippt erkennt.
const GETIPPT = new RegExp(
  `^(${[...PLANNING_CHAT_KINDS, ...SHARED_KINDS].sort((a, b) => b.length - a.length).join('|')}):(\\d+)$`,
  'iu',
);

/**
 * Satzzeichen am Adressende gehören meist zum Satz, nicht zur Adresse:
 * „steht auf https://…/karte." – der Punkt ist Interpunktion. Ebenso eine
 * schließende Klammer ohne öffnendes Gegenstück („(siehe https://…)").
 */
function trimmeSatzzeichen(url) {
  let ende = url.length;
  while (ende > 0) {
    const z = url[ende - 1];
    if ('.,;:!?'.includes(z)) { ende -= 1; continue; }
    if (z === ')') {
      const teil = url.slice(0, ende);
      if ((teil.match(/\)/g) || []).length > (teil.match(/\(/g) || []).length) { ende -= 1; continue; }
    }
    break;
  }
  return url.slice(0, ende);
}

/** Nur http(s) – alles andere (javascript:, data:, …) bleibt Text. */
function istErlaubt(url) {
  try {
    return ['http:', 'https:'].includes(new URL(url).protocol);
  } catch {
    return false;
  }
}

/**
 * @param {string} body
 * @param {{wikilinks?: boolean}} [optionen] `wikilinks: false` lässt
 *   [[…]] unangetastet im Text stehen – für Direktnachrichten, die keine
 *   Notiz-Freigaben kennen und aus einem Verweis nichts machen könnten.
 *   Ihn trotzdem zu zerlegen würde nur die Klammern schlucken und einen
 *   Titel zurücklassen, der nach nichts verlinkt.
 * @returns {Array<{type:'text',value:string}
 *   | {type:'wikilink', target:string, label:string, kind:?string, id:?number}
 *   | {type:'url', href:string, label:string}>}
 *
 *   Bei getippten Zielen ([[board:12]]) ist `label` leer, wenn kein
 *   Anzeigetext dabeisteht: Wie das Ziel heißt, weiß nur der Server. Die
 *   Anzeige nimmt den Namen dann aus der aufgelösten Karte. „board:12" als
 *   Anzeigetext wäre schlicht falsch.
 */
export function parseChatText(body, { wikilinks = true } = {}) {
  const text = typeof body === 'string' ? body : '';
  const stuecke = [];
  let zuletzt = 0;

  const schiebeText = (bis) => {
    if (bis > zuletzt) stuecke.push({ type: 'text', value: text.slice(zuletzt, bis) });
  };

  STUECKE.lastIndex = 0;
  let m;
  while ((m = STUECKE.exec(text)) !== null) {
    const [ganzes, wiki, url] = m;

    if (wiki !== undefined) {
      if (! wikilinks) continue; // bleibt unverändert Text, inklusive Klammern
      const [ziel, anzeige] = wiki.split('|');
      const target = ziel.trim();
      if (target === '') continue; // [[|x]] ist kein Verweis
      schiebeText(m.index);
      const getippt = GETIPPT.exec(target);
      const eigener = (anzeige ?? '').trim();
      stuecke.push({
        type: 'wikilink',
        target,
        kind: getippt ? getippt[1].toLowerCase() : null,
        id: getippt ? Number(getippt[2]) : null,
        // Ohne eigenen Anzeigetext die Kennung abschneiden: Sonst stünde
        // „20260801182244 Getränkeliste" mitten im Chat, obwohl die vierzehn
        // Ziffern niemandem etwas sagen. Bei getippten Zielen bleibt das Feld
        // leer – „board:12" wäre als Anzeigetext schlicht falsch.
        label: getippt
          ? eigener
          : (eigener || target.replace(KENNUNG_VORN, '') || target),
      });
      zuletzt = m.index + ganzes.length;
      continue;
    }

    const adresse = trimmeSatzzeichen(url);
    if (adresse === '' || !istErlaubt(adresse)) continue; // bleibt Text
    schiebeText(m.index);
    stuecke.push({ type: 'url', href: adresse, label: adresse });
    zuletzt = m.index + adresse.length;
  }

  schiebeText(text.length);

  return stuecke;
}

/**
 * Der Text, den die Vorschlagsliste einsetzt: `[[art:id|Name]]`.
 *
 * Der Name kommt mit, damit im Eingabefeld lesbar steht, was man gerade
 * verlinkt hat – „[[board:12]]" mitten in einem längeren Beitrag könnte
 * niemand mehr überblicken.
 *
 * Klammern und Strich müssen dabei aus dem Namen heraus: Ein Board namens
 * „Einkauf | Getränke" ergäbe sonst [[board:12|Einkauf | Getränke]], und der
 * Zerleger schnitte am ERSTEN Strich – der Rest stünde als toter Text da.
 * Ein „]]" im Namen beendete den Verweis sogar mitten im Namen.
 *
 * @param {{kind: string, id: number, title: string}} ziel
 */
export function linkMarkup({ kind, id, title }) {
  const name = String(title ?? '').replace(/[[\]|]/gu, ' ').replace(/\s+/gu, ' ').trim();

  return name ? `[[${kind}:${id}|${name}]]` : `[[${kind}:${id}]]`;
}

/**
 * Schlüssel, unter dem das Backend ein Linkziel in seiner Karte führt:
 * das kleingeschriebene Rohziel. Eine Stelle für die Regel, damit Anzeige
 * und Nachschlagen nicht auseinanderlaufen können.
 */
export function linkKey(target) {
  return (target || '').toLowerCase();
}
