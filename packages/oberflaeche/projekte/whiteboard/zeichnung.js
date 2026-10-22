/*
 * Was von einer Excalidraw-Zeichnung gespeichert wird (docs/plan-whiteboard.md).
 *
 * Nur die Elemente: Ansicht, Zoom und Werkzeug gehören dem Gerät. Gelöschte
 * Elemente fallen weg. Bilder bleiben nur, wenn sie auf einen geteilten
 * Inhalt verweisen (W2) -- ihre Bytes speichert das Whiteboard nie, sie
 * kommen beim Öffnen aus der Freigabe.
 *
 * EIN VERWEIS: { art: 'bild'|'notiz'|'datei', freigabe, uuid, name, mime? }.
 * Er steht in `customData.verweis`; Karten tragen ihn zusätzlich als Link
 * (`openany:verweis/…`), damit Excalidraw sie antippbar macht.
 */

export const LEER = Object.freeze({ elements: [] });

export function zumSpeichern(elemente) {
  return {
    elements: (elemente ?? [])
      .filter((e) => !e.isDeleted && (e.type !== 'image' || e.customData?.verweis))
      .map((e) => ({ ...e })),
  };
}

/**
 * Ein Fingerabdruck der Zeichnung: ändert sich, sobald ein Element sich
 * ändert. Excalidraw meldet jede Bewegung des Zeigers als Änderung -- damit
 * wird nur gespeichert, wenn wirklich gezeichnet wurde.
 */
export function fassung(elemente) {
  let n = 0;
  for (const e of elemente ?? []) n += (e.version ?? 0) + (e.isDeleted ? 1 : 0);
  return `${(elemente ?? []).length}:${n}`;
}

/** Die Sprache der Oberfläche in der Form, die Excalidraw kennt. */
const SPRACHEN = { de: 'de-DE', en: 'en', es: 'es-ES', fr: 'fr-FR', pl: 'pl-PL', pt: 'pt-PT', uk: 'uk-UA' };
export const excalidrawSprache = (locale) => SPRACHEN[String(locale ?? '').slice(0, 2)] ?? 'en';

const LINK = 'openany:verweis/';

export const alsLink = (v) => `${LINK}${v.art}/${v.freigabe}/${encodeURIComponent(v.uuid)}`;
export const istVerweisLink = (link) => typeof link === 'string' && link.startsWith(LINK);

/** Die Datei-Kennung eines eingebundenen Bildes in Excalidraw. */
export const dateiId = (v) => `openany-${v.freigabe}-${v.uuid}`;

/** Die eingebundenen Bilder einer Zeichnung, je Datei einmal. */
export function bildVerweise(elemente) {
  const da = new Map();
  for (const e of elemente ?? []) {
    const v = e.customData?.verweis;
    if (e.type === 'image' && !e.isDeleted && v && e.fileId && !da.has(e.fileId)) da.set(e.fileId, v);
  }
  return [...da].map(([fileId, verweis]) => ({ fileId, verweis }));
}

/** Ein Bild an der Stelle eines Bildes, dessen Freigabe endete. */
export function platzhalter(text) {
  const sicher = String(text).replace(/[<>&"]/g, (z) => ({ '<': '&lt;', '>': '&gt;', '&': '&amp;', '"': '&quot;' }[z]));
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="400" height="300" viewBox="0 0 400 300">`
    + '<rect x="2" y="2" width="396" height="296" rx="12" fill="#f1f5f9" stroke="#94a3b8" stroke-width="3" stroke-dasharray="10 8"/>'
    + `<text x="200" y="158" font-family="sans-serif" font-size="22" fill="#64748b" text-anchor="middle">${sicher}</text></svg>`;
  return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`;
}

/** Wie groß ein eingefügtes Bild wird: höchstens `rand` an der längeren Seite. */
export function eingepasst(breite, hoehe, rand = 360) {
  if (!breite || !hoehe) return { width: rand, height: rand * 0.75 };
  const f = Math.min(1, rand / Math.max(breite, hoehe));
  return { width: Math.round(breite * f), height: Math.round(hoehe * f) };
}
