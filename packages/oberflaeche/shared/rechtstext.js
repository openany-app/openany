/*
 * Das kleine Markdown der Rechtstexte (backend/resources/rechtstexte/README.md)
 * in Blöcke zerlegen. Die Anzeige baut daraus Elemente statt HTML, so kann auch
 * eine verfälschte Antwort kein Skript in die Seite bringen.
 */

/** „ein **fettes** Wort" → [{ text: 'ein ' }, { text: 'fettes', fett: true }, { text: ' Wort' }] */
export function inline(zeile) {
  return zeile.split(/\*\*/).map((text, i) => ({ text, fett: i % 2 === 1 })).filter((t) => t.text);
}

/** @returns {Array<{art: 'h1'|'h2'|'p'|'ul', zeilen: Array}>} */
export function rechtstextBloecke(markdown) {
  const bloecke = [];
  for (const roh of String(markdown ?? '').replace(/\r\n/g, '\n').split(/\n\s*\n/)) {
    if (!roh.trim()) continue;
    const zeilen = roh.trim().split('\n');
    if (zeilen[0].startsWith('## ')) bloecke.push({ art: 'h2', zeilen: [inline(zeilen[0].slice(3).trim())] });
    else if (zeilen[0].startsWith('# ')) bloecke.push({ art: 'h1', zeilen: [inline(zeilen[0].slice(2).trim())] });
    else if (zeilen.every((z) => z.startsWith('- '))) bloecke.push({ art: 'ul', zeilen: zeilen.map((z) => inline(z.slice(2).trim())) });
    else bloecke.push({ art: 'p', zeilen: zeilen.map((z) => inline(z.trim())) });
  }
  return bloecke;
}
