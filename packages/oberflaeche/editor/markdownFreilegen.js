// Zitate und Fußnoten so speichern, wie Zettlr und Pandoc sie lesen.
//
// ANLASS (15.10.2026): Der Editor (tiptap-markdown, prosemirror-markdown)
// maskiert beim Speichern JEDE eckige Klammer im Text. Aus `[@mueller2020]`
// wurde `\[@mueller2020\]`, aus `^[Anmerkung]` wurde `^\[Anmerkung\]`.
// openanys eigenes PDF las beides trotzdem, Zettlr/Pandoc aber sehen darin
// nur Text -- kein Zitat, keine Fußnote.
//
// Hier wird genau das zurückgenommen: `\[@…\]` und `^\[…\]` (mit
// verschachtelten Klammern, auch über Formatierung hinweg). Code -- Blöcke
// und `…` -- bleibt unberührt; dort ist `\[` wörtlich gemeint.

const ZAUN = /^ {0,3}(`{3,}|~{3,})/;

/** Die Länge eines Code-Abschnitts `…` ab i (0, wenn keiner beginnt). */
function codeSpanne(text, i) {
  if (text[i] !== '`') return 0;
  let n = 0;
  while (text[i + n] === '`') n++;
  const ende = text.indexOf('`'.repeat(n), i + n);
  return ende < 0 ? n : ende + n - i;
}

/** Wo die Fußnote endet, die bei `start` (nach `^\[`) beginnt -- Index von `\]`, oder -1. */
function fussnotenEnde(text, start) {
  let tiefe = 1;
  for (let j = start; j < text.length; j++) {
    if (text[j] === '\n' && text[j + 1] === '\n') return -1; // nicht über den Absatz hinaus
    const code = codeSpanne(text, j);
    if (code) { j += code - 1; continue; }
    if (text[j] === '\\') {
      if (text[j + 1] === '[') tiefe++;
      else if (text[j + 1] === ']' && --tiefe === 0) return j;
      j++;
      continue;
    }
    if (text[j] === '[') tiefe++;
    else if (text[j] === ']') tiefe--;
  }
  return -1;
}

/** Eine Zeile (oder mehrere, ohne Code-Block) freilegen. */
function absatzFreilegen(text) {
  let aus = '';
  for (let i = 0; i < text.length; i++) {
    const code = codeSpanne(text, i);
    if (code) {
      aus += text.slice(i, i + code);
      i += code - 1;
      continue;
    }
    // ^\[ … \]  ->  ^[ … ]   (das Innere wird selbst wieder freigelegt)
    if (text.startsWith('^\\[', i)) {
      const ende = fussnotenEnde(text, i + 3);
      if (ende > 0) {
        aus += `^[${absatzFreilegen(text.slice(i + 3, ende))}]`;
        i = ende + 1;
        continue;
      }
    }
    // \[@ … \]  ->  [@ … ]   (ohne weitere Klammern darin, in einer Zeile)
    if (text.startsWith('\\[@', i)) {
      const m = /^\\\[(@[^[\]\n\\]*(?:\\[^[\]\n][^[\]\n\\]*)*)\\\]/.exec(text.slice(i));
      if (m) {
        aus += `[${m[1]}]`;
        i += m[0].length - 1;
        continue;
      }
    }
    aus += text[i];
  }
  return aus;
}

/** Das ganze Markdown einer Notiz; Code-Blöcke bleiben, wie sie sind. */
export function zitateUndFussnotenFreilegen(markdown) {
  const zeilen = String(markdown ?? '').split('\n');
  const aus = [];
  let zaun = null;
  let puffer = [];
  const leeren = () => {
    if (puffer.length) aus.push(absatzFreilegen(puffer.join('\n')));
    puffer = [];
  };
  for (const zeile of zeilen) {
    const m = ZAUN.exec(zeile);
    if (zaun) {
      aus.push(zeile);
      if (m && m[1][0] === zaun[0] && m[1].length >= zaun.length) zaun = null;
    } else if (m) {
      leeren();
      zaun = m[1];
      aus.push(zeile);
    } else {
      puffer.push(zeile);
    }
  }
  leeren();
  return aus.join('\n');
}
