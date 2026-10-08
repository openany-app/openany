// Auswahl für die [[-Vorschlagsliste im Projekt-Chat: aus allen verlinkbaren
// Zielen die wenigen heraussuchen, die angezeigt werden.
//
// Frei von Vue und DOM, damit es in der node-Umgebung testbar ist (wie
// chatText.js und folderTree.js).

/**
 * Wie viele Vorschläge höchstens. Mehr als eine Handvoll überblickt niemand
 * beim Tippen; die Liste scrollt zwar, aber gescrollt wird hier nicht.
 */
export const SUGGEST_MAX = 10;

/**
 * Die anzuzeigenden Ziele zu einer Eingabe.
 *
 * Zwei Regeln, und die zweite ist der eigentliche Grund für diese Datei:
 *
 *  1. GÜTE zuerst – was mit dem Getippten beginnt, meint man eher als etwas,
 *     das es nur irgendwo in der Mitte enthält.
 *  2. REIHUM je Art. Vorher wurden schlicht die ersten acht Treffer genommen,
 *     und weil die Liste vom Server nach Arten sortiert kommt (Notizen
 *     zuerst), füllten in einem Projekt mit fünfzig Notizen ebendiese jeden
 *     Platz. Dass sich auch ein Board, eine Datei oder ein Bild verlinken
 *     lässt, erfuhr so niemand – die Funktion war da und blieb unsichtbar.
 *
 * Der beste Treffer bleibt dabei oben: Die Arten kommen in der Reihenfolge
 * ihres jeweils besten Treffers dran, also führt dessen Art die Runde an.
 *
 * @param {Array<{kind: string, id: number, title: string}>} kandidaten
 * @param {string} query  das hinter „[[" Getippte, klein oder groß
 * @param {number} [max]
 */
export function suggestTargets(kandidaten, query, max = SUGGEST_MAX) {
  const q = String(query ?? '').toLowerCase();

  const treffer = (kandidaten || [])
    .filter((k) => k && typeof k.title === 'string' && k.title.toLowerCase().includes(q));

  treffer.sort((a, b) => {
    const av = a.title.toLowerCase().startsWith(q) ? 0 : 1;
    const bv = b.title.toLowerCase().startsWith(q) ? 0 : 1;

    return av - bv || a.title.localeCompare(b.title, 'de');
  });

  // Map hält die Einfügereihenfolge – die ist hier die Güte-Reihenfolge.
  const nachArt = new Map();
  for (const item of treffer) {
    if (! nachArt.has(item.kind)) nachArt.set(item.kind, []);
    nachArt.get(item.kind).push(item);
  }

  const aus = [];
  const listen = [...nachArt.values()];
  for (let runde = 0; aus.length < max; runde++) {
    let etwasGenommen = false;
    for (const liste of listen) {
      if (runde >= liste.length) continue;
      aus.push(liste[runde]);
      etwasGenommen = true;
      if (aus.length >= max) break;
    }
    if (! etwasGenommen) break;
  }

  return aus;
}
