// Den Text eines Dokuments für eine KI-Anfrage auslesen – im Browser.
//
// WARUM HIER UND NICHT AUF DEM SERVER: pdf.js ist für die Texterkennung schon
// da, und Word-Dateien sind ZIP-Archive, die der Browser selbst entpacken
// kann. Der Server bräuchte für PDFs ein zusätzliches Systempaket. Dieselbe
// Auslese läuft später unverändert in der WebView der Tauri-App.
//
// Welches Verfahren eine Datei bekommt (`format`), entscheidet der Server
// (KiDokument::FORMATE) – hier gibt es keine zweite Liste.
//
// WARUM KEIN DOMParser: Die Tests laufen in Node, und dort gibt es keinen.
// Ein kleiner Tokenizer reicht für die paar Elemente, die Text tragen, und
// läuft überall gleich.

import { zipEintrag } from './zipLesen';

/** Muss zu ProjectAiController::MAX_ZEICHEN passen – dort wird es durchgesetzt. */
export const MAX_ZEICHEN = 200_000;

/**
 * Unter so vielen Zeichen gilt ein PDF als Scan ohne Textebene. Dieselbe
 * Schwelle wie bei der Texterkennung, nur übers ganze Dokument.
 */
const PDF_MINDESTENS = 40;

export class TextAuszugFehler extends Error {
  /** @param {'keinText'|'zuLang'|'unlesbar'} grund */
  constructor(grund) {
    super(grund);
    this.grund = grund;
  }
}

/**
 * @param {'pdf'|'docx'|'odt'|'text'} format
 * @param {ArrayBuffer} buffer
 * @returns {Promise<string>}
 */
export async function textAuslesen(format, buffer) {
  let text;
  try {
    text = await roh(format, buffer);
  } catch (e) {
    if (e instanceof TextAuszugFehler) throw e;
    throw new TextAuszugFehler('unlesbar');
  }

  text = text.replace(/\n{3,}/g, '\n\n').trim();

  if (text.length < (format === 'pdf' ? PDF_MINDESTENS : 1)) throw new TextAuszugFehler('keinText');
  // Ablehnen statt abschneiden: Eine KI, die still die halbe Datei gelesen
  // hat, antwortet überzeugt über etwas, das sie nicht kennt.
  if (text.length > MAX_ZEICHEN) throw new TextAuszugFehler('zuLang');

  return text;
}

/**
 * Für die Suche im Speicher: Hier wird ABGESCHNITTEN statt abgelehnt. Wer
 * sucht, findet lieber in den ersten 200 000 Zeichen als gar nicht -- anders
 * als eine KI, die über den Rest überzeugt Falsches sagen würde.
 */
export async function textFuerSuche(format, buffer) {
  try {
    const text = await textAuslesen(format, buffer);
    return text;
  } catch (e) {
    if (e?.grund !== 'zuLang') throw e;
    const ganz = (await roh(format, buffer)).replace(/\n{3,}/g, '\n\n').trim();
    return ganz.slice(0, MAX_ZEICHEN);
  }
}

async function roh(format, buffer) {
  switch (format) {
    case 'pdf': return pdfText(buffer);
    case 'docx': return docxText(await xmlAus(buffer, 'word/document.xml'));
    case 'odt': return odtText(await xmlAus(buffer, 'content.xml'));
    case 'text': return new TextDecoder().decode(buffer);
    default: throw new TextAuszugFehler('unlesbar');
  }
}

async function xmlAus(buffer, name) {
  const bytes = await zipEintrag(buffer, name);
  if (!bytes) throw new TextAuszugFehler('unlesbar');
  return new TextDecoder().decode(bytes);
}

async function pdfText(buffer) {
  // Erst hier laden: pdf.js ist groß, und die meisten Aufrufe brauchen es nie.
  const { oeffnePdf, pdfFreigeben } = await import('@oberflaeche/texterkennung/ocr/pdfPages');
  const pdf = await oeffnePdf(buffer);
  const seiten = [];
  try {
    for (let n = 1; n <= pdf.numPages; n += 1) {
      const seite = await pdf.getPage(n);
      const inhalt = await seite.getTextContent();
      seiten.push(inhalt.items.map((i) => (i.str ?? '') + (i.hasEOL ? '\n' : '')).join(''));
      seite.cleanup();
    }
  } finally {
    pdfFreigeben(pdf);
  }
  return seiten.join('\n\n');
}

// ---------------------------------------------------------------------------
// XML

const TOKEN = /<(\/)?([\w.-]+:)?([\w.-]+)([^>]*?)(\/)?>|([^<]+)/g;

function* tokens(xml) {
  for (const t of xml.matchAll(TOKEN)) {
    if (t[6] !== undefined) {
      yield { text: entschluesseln(t[6]) };
    } else if (!t[0].startsWith('<?') && !t[0].startsWith('<!')) {
      yield { name: (t[2] ?? '') + t[3], schliesst: !!t[1], leer: !!t[5], attribute: t[4] };
    }
  }
}

function attribut(attribute, name) {
  const m = attribute.match(new RegExp(`${name}="([^"]*)"`));
  return m ? m[1] : null;
}

function entschluesseln(s) {
  return s
    .replace(/&#x([0-9a-f]+);/gi, (_, h) => String.fromCodePoint(parseInt(h, 16)))
    .replace(/&#(\d+);/g, (_, d) => String.fromCodePoint(Number(d)))
    .replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&quot;/g, '"').replace(/&apos;/g, "'")
    .replace(/&amp;/g, '&');
}

/**
 * Sammelt Absätze und Tabellenzeilen zu Text. Tabellenzeilen werden als
 * Markdown-Zeilen geschrieben und ohne Leerzeile aneinandergehängt, damit
 * eine Tabelle als Tabelle erkennbar bleibt.
 */
function sammler() {
  const bloecke = [];
  const zeilen = [];
  const zellen = [];
  return {
    absatz(text) {
      if (zellen.length) zellen[zellen.length - 1].push(text);
      else if (text.trim()) bloecke.push({ text, zeile: false });
    },
    zeileAuf() { zeilen.push([]); },
    zelleAuf() { zellen.push([]); },
    zelleZu() {
      const inhalt = (zellen.pop() ?? []).join(' ').trim();
      if (zeilen.length) zeilen[zeilen.length - 1].push(inhalt);
    },
    zeileZu() {
      const zellenDerZeile = zeilen.pop() ?? [];
      const text = `| ${zellenDerZeile.join(' | ')} |`;
      // Eine Zeile in einer Zelle (verschachtelte Tabelle) bleibt Text der Zelle.
      if (zellen.length) zellen[zellen.length - 1].push(text);
      else bloecke.push({ text, zeile: true });
    },
    ergebnis() {
      return bloecke.reduce((aus, b, i) => {
        if (i === 0) return b.text;
        return aus + (b.zeile && bloecke[i - 1].zeile ? '\n' : '\n\n') + b.text;
      }, '');
    },
  };
}

/** Text aus `word/document.xml`. */
export function docxText(xml) {
  const s = sammler();
  let absatz = null;
  let stufe = 0;
  let liste = false;
  let imText = false;

  for (const t of tokens(xml)) {
    if (t.text !== undefined) {
      if (imText && absatz !== null) absatz += t.text;
      continue;
    }
    switch (t.name) {
      case 'w:p':
        if (t.schliesst) {
          if (absatz !== null) {
            const praefix = stufe ? `${'#'.repeat(stufe)} ` : (liste ? '- ' : '');
            s.absatz(praefix + absatz);
          }
          absatz = null;
        } else if (!t.leer) {
          absatz = ''; stufe = 0; liste = false;
        }
        break;
      case 'w:pStyle': {
        // Englisch „Heading2", deutsches Word „berschrift2" (das Ü fällt aus der Kennung).
        const m = (attribut(t.attribute, 'w:val') ?? '').match(/(?:heading|berschrift)\s*(\d)/i);
        if (m) stufe = Math.min(Number(m[1]), 6);
        break;
      }
      case 'w:numPr': liste = true; break;
      case 'w:t': imText = !t.schliesst && !t.leer; break;
      case 'w:tab': if (absatz !== null) absatz += '\t'; break;
      case 'w:br': case 'w:cr': if (absatz !== null) absatz += '\n'; break;
      case 'w:tr': if (t.schliesst) s.zeileZu(); else if (!t.leer) s.zeileAuf(); break;
      case 'w:tc': if (t.schliesst) s.zelleZu(); else if (!t.leer) s.zelleAuf(); break;
      default: break;
    }
  }
  return s.ergebnis();
}

/** Text aus `content.xml` eines OpenDocument-Textes. */
export function odtText(xml) {
  const s = sammler();
  const offen = []; // verschachtelte Absätze (z. B. Fußnoten im Absatz)
  let listenTiefe = 0;

  const anhaengen = (text) => { if (offen.length) offen[offen.length - 1].text += text; };

  for (const t of tokens(xml)) {
    if (t.text !== undefined) {
      anhaengen(t.text);
      continue;
    }
    switch (t.name) {
      case 'text:p':
      case 'text:h':
        if (t.schliesst) {
          const a = offen.pop();
          if (!a) break;
          if (offen.length) { offen[offen.length - 1].text += ` ${a.text}`; break; }
          const praefix = a.stufe ? `${'#'.repeat(a.stufe)} ` : (listenTiefe ? '- ' : '');
          s.absatz(praefix + a.text);
        } else if (!t.leer) {
          const stufe = t.name === 'text:h' ? Math.min(Number(attribut(t.attribute, 'text:outline-level') ?? 1), 6) : 0;
          offen.push({ text: '', stufe });
        }
        break;
      case 'text:s': anhaengen(' '.repeat(Number(attribut(t.attribute, 'text:c') ?? 1))); break;
      case 'text:tab': anhaengen('\t'); break;
      case 'text:line-break': anhaengen('\n'); break;
      case 'text:list-item': listenTiefe += t.schliesst ? -1 : (t.leer ? 0 : 1); break;
      case 'table:table-row': if (t.schliesst) s.zeileZu(); else if (!t.leer) s.zeileAuf(); break;
      case 'table:table-cell': if (t.schliesst) s.zelleZu(); else if (!t.leer) s.zelleAuf(); break;
      default: break;
    }
  }
  return s.ergebnis();
}
