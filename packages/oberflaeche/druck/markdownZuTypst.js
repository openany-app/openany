// Markdown einer Notiz -> Typst-Quelltext.
//
// WARUM SELBST ÜBERSETZT und nicht über ein Typst-Paket (cmarker): Notizen
// haben Eigenheiten, die kein Standard-Markdown kennt -- Wikilinks mit
// Zettel-Kennung, Bilder als Pfade in die Notiz-Anhänge, später für die
// Druckfassung Fußnoten und Zitate (Schreibweise wie Pandoc und Zettlr:
// `^[Anmerkung]`, `[@schluessel, S. 12]`). Hier liegt das in der eigenen Hand, und
// ein fremdes Paket müsste openany ohnehin selbst ausliefern.
//
// Gelesen wird mit markdown-it, demselben Parser, mit dem der Editor
// (tiptap-markdown) die Notiz versteht. Was er als Token liefert, wird hier
// zu Typst-Markup; Text wird dabei so maskiert, dass nichts darin als Typst
// gelesen wird.
import MarkdownIt from 'markdown-it';
import aufgabenlisten from 'markdown-it-task-lists';

const md = new MarkdownIt({ html: false, linkify: true, breaks: false }).use(aufgabenlisten);

// Zeichen, die in Typst-Markup etwas bedeuten. `/` wegen `//` (Kommentar)
// und `/ Begriff:` (Begriffsliste), `-`/`+`/`=` am Zeilenanfang (Liste,
// Überschrift), `~` (geschütztes Leerzeichen), `'`/`"` bleiben: Typst macht
// daraus typografische Anführungszeichen, und genau das soll es.
const MASKIEREN = /[\\#$*_`<>@[\]~=\-+/]/g;

/**
 * Text so, dass Typst ihn wörtlich setzt.
 *
 * DER PUNKT NACH EINER ZIFFER auch: „14. Oktober" am Anfang einer Zeile
 * oder Tabellenzelle läse Typst sonst als nummerierte Liste.
 */
export function textMaskieren(text) {
  return String(text).replace(MASKIEREN, (z) => `\\${z}`).replace(/(\d)\./g, '$1\\.');
}

// ENDE EINES AUSDRUCKS. Hinter `#strong[…]`, `#cite(…)` usw. liest Typst
// weiter: `;` beendet den Ausdruck und verschwindet, `(` und `.` setzen ihn
// fort (`#strong[a](b)` bricht ab). Jeder hier erzeugte Ausdruck bekommt
// deshalb diese Marke; inline() maskiert am Ende das Zeichen dahinter und
// streicht sie. Aus dem Text der Notiz wird sie vorher entfernt.
const ENDE = '\uE000';

/** Eine Zeichenkette als Typst-String-Literal ("…"). */
export function stringLiteral(text) {
  return `"${String(text).replace(/\\/g, '\\\\').replace(/"/g, '\\"').replace(/\n/g, '\\n').replace(/\r/g, '')}"`;
}

// [[20260726180305 Titel]], [[Titel]], [[Titel|Anzeige]] -> was der Leser sieht.
const WIKILINK = /\[\[([^\]|]+?)(?:\|([^\]]+))?\]\]/g;
const ZETTELKENNUNG = /^\d{12,14}\s+/;

/** Wikilinks im Fließtext: Anzeige statt Klammern und Kennung. */
export function wikilinksAufloesen(text) {
  return text.replace(WIKILINK, (_, ziel, anzeige) => (anzeige ?? ziel.replace(ZETTELKENNUNG, '')).trim());
}

/**
 * @param {string} markdown
 * @param {object} [o]
 * @param {(src: string) => ({datei: string, breiteCm?: number}|null)} [o.bild]
 *        Bildpfad -> Datei im Typst-Projekt und Breite auf dem Papier, oder
 *        null (dann steht statt des Bildes sein Alt-Text)
 * @param {number} [o.obersteEbene]  auf welche Ebene die oberste Überschrift
 *        der Notiz kommt -- in der Druckfassung 2, denn jede Notiz ist dort
 *        selbst ein Kapitel. Gleich, ob die Notiz mit `#` oder `##` beginnt:
 *        Ihre Gliederung rückt als Ganzes darunter. Ohne Angabe bleibt sie.
 * @param {Set<string>} [o.quellen]  Schlüssel, die im Literaturverzeichnis
 *        stehen. Nur diese werden zu Zitaten -- Typst bräche bei einem
 *        unbekannten ab. Alles andere bleibt als Text stehen, wie geschrieben.
 * @param {Set<string>} [o.zitiert]  wird gefüllt: welche Schlüssel vorkommen
 *        (ohne Zitat kein Verzeichnis)
 * @returns {string} Typst-Markup (ohne Seiteneinrichtung)
 */
export function markdownZuTypst(markdown, { bild = () => null, obersteEbene = null, quellen = new Set(), zitiert = new Set() } = {}) {
  const tokens = md.parse(String(markdown ?? ''), {});
  const ebenen = tokens.filter((t) => t.type === 'heading_open').map((t) => Number(t.tag.slice(1)));
  const ueberschriftVersatz = obersteEbene && ebenen.length ? obersteEbene - Math.min(...ebenen) : 0;
  return block(tokens, 0, tokens.length, { bild, ueberschriftVersatz, quellen, zitiert }).trim() + '\n';
}

// [@a], [@a, S. 12], [@a; @b, S. 3] -- Pandoc-Schreibweise. Ein Teil vor dem
// @ („siehe @a") kennt Typst nicht; solche Klammern bleiben Text.
const ZITATKLAMMER = /\[(@[^[\]]*)\]/y;
const ZITATTEIL = /^\s*@([^\s,;[\]@]+)\s*(?:,\s*(.*?))?\s*$/s;
// @a im Satz („wie @a zeigt") -> „wie Müller (2020) zeigt".
const ERZAEHLEND = /@([^\s,;[\]@]+)/y;

/** `[@…]` ab Position i -> Typst, oder null, wenn es kein (bekanntes) Zitat ist. */
function zitatklammer(text, i, ctx) {
  ZITATKLAMMER.lastIndex = i;
  const m = ZITATKLAMMER.exec(text);
  if (!m) return null;
  const teile = m[1].split(';').map((teil) => ZITATTEIL.exec(teil));
  if (teile.some((t) => !t || !ctx.quellen.has(t[1]))) return null;
  // Nebeneinander stehende #cite fasst Typst zu einer Klammer zusammen:
  // „(Weber, 2019; Klein, 2021, S. 50)".
  const aus = teile.map(([, schluessel, seite]) => {
    ctx.zitiert.add(schluessel);
    return `#cite(label(${stringLiteral(schluessel)})${seite ? `, supplement: [${textMaskieren(seite)}]` : ''})${ENDE}`;
  });
  return { typst: aus.join(' '), laenge: m[0].length };
}

/** `@schluessel` ab Position i (erzählend) -> Typst, oder null. */
function erzaehlendesZitat(text, i, ctx) {
  // Nicht mitten in einem Wort oder einer Adresse (name@beispiel.test).
  if (i > 0 && /[\p{L}\p{N}._@-]/u.test(text[i - 1])) return null;
  ERZAEHLEND.lastIndex = i;
  const m = ERZAEHLEND.exec(text);
  if (!m) return null;
  // Satzzeichen am Ende gehört zum Satz, nicht zum Schlüssel: „… bei @a."
  let schluessel = m[1];
  while (schluessel && !ctx.quellen.has(schluessel) && /[.:!?)'"“”»]$/.test(schluessel)) schluessel = schluessel.slice(0, -1);
  if (!ctx.quellen.has(schluessel)) return null;
  ctx.zitiert.add(schluessel);
  return { typst: `#cite(label(${stringLiteral(schluessel)}), form: "prose")${ENDE}`, laenge: schluessel.length + 1 };
}

/**
 * Ein Text-Token -> Typst, mit Zitaten und Fußnoten.
 *
 * FUSSNOTEN reichen über Formatierung hinweg (`^[mit **fett** darin]` sind
 * drei Token). Eine offene Fußnote liegt deshalb auf demselben Stapel wie
 * `**` und `*`; geschlossen wird sie nur, wenn sie oben liegt -- sonst
 * entstünde Typst mit verschränkten Klammern. Eckige Klammern darin zählen
 * mit, damit `^[siehe [[Notiz]]]` erst am Ende schließt.
 */
function text(inhalt, ctx, stapel, fussnoten) {
  let aus = '';
  let puffer = '';
  const leeren = () => {
    aus += textMaskieren(wikilinksAufloesen(puffer.replaceAll(ENDE, '')));
    puffer = '';
  };
  for (let i = 0; i < inhalt.length; i++) {
    const z = inhalt[i];
    const offen = fussnoten[fussnoten.length - 1];
    if (z === '^' && inhalt[i + 1] === '[') {
      leeren();
      aus += '#footnote[';
      stapel.push(`]${ENDE}`);
      fussnoten.push({ ebene: stapel.length, tiefe: 1 });
      i += 1;
      continue;
    }
    if (z === '[') {
      const zitat = zitatklammer(inhalt, i, ctx);
      if (zitat) {
        leeren();
        aus += zitat.typst;
        i += zitat.laenge - 1;
        continue;
      }
      if (offen && offen.ebene === stapel.length) offen.tiefe++;
    } else if (z === '@') {
      const zitat = erzaehlendesZitat(inhalt, i, ctx);
      if (zitat) {
        leeren();
        aus += zitat.typst;
        i += zitat.laenge - 1;
        continue;
      }
    } else if (z === ']' && offen && offen.ebene === stapel.length && --offen.tiefe === 0) {
      leeren();
      aus += stapel.pop();
      fussnoten.pop();
      continue;
    }
    puffer += z;
  }
  leeren();
  return aus;
}

/** Blockebene: tokens[von..bis) */
function block(tokens, von, bis, ctx) {
  let aus = '';
  for (let i = von; i < bis; i++) {
    const t = tokens[i];
    switch (t.type) {
      case 'heading_open': {
        const ebene = Math.max(1, Math.min(6, Number(t.tag.slice(1)) + ctx.ueberschriftVersatz));
        aus += `${'='.repeat(ebene)} ${inline(tokens[i + 1], ctx)}\n\n`;
        i += 2;
        break;
      }
      case 'paragraph_open': {
        // In einer engen Liste ist der Absatz versteckt -- kein Leerraum danach.
        const inhalt = inline(tokens[i + 1], ctx);
        aus += t.hidden ? inhalt : `${inhalt}\n\n`;
        i += 2;
        break;
      }
      case 'bullet_list_open':
      case 'ordered_list_open': {
        const ende = schliessendes(tokens, i);
        aus += liste(tokens, i, ende, ctx) + '\n';
        i = ende;
        break;
      }
      case 'blockquote_open': {
        const ende = schliessendes(tokens, i);
        aus += `#quote(block: true)[\n${block(tokens, i + 1, ende, ctx).trim()}\n]\n\n`;
        i = ende;
        break;
      }
      case 'fence':
      case 'code_block': {
        const sprache = (t.info || '').trim().split(/\s+/)[0];
        aus += `#raw(block: true${sprache ? `, lang: ${stringLiteral(sprache)}` : ''}, ${stringLiteral(t.content.replace(/\n$/, ''))})\n\n`;
        break;
      }
      case 'hr':
        aus += '#line(length: 100%, stroke: 0.5pt + luma(180))\n\n';
        break;
      case 'table_open': {
        const ende = schliessendes(tokens, i);
        aus += tabelle(tokens, i, ende, ctx) + '\n\n';
        i = ende;
        break;
      }
      default:
        break;
    }
  }
  return aus;
}

/** Eine Liste samt Unterlisten; Einrückung zwei Leerzeichen je Ebene. */
function liste(tokens, von, bis, ctx, tiefe = 0) {
  const nummeriert = tokens[von].type === 'ordered_list_open';
  const einzug = '  '.repeat(tiefe);
  let aus = '';
  for (let i = von + 1; i < bis; i++) {
    if (tokens[i].type !== 'list_item_open') continue;
    const ende = schliessendes(tokens, i);
    let text = '';
    let unter = '';
    for (let j = i + 1; j < ende; j++) {
      const t = tokens[j];
      if (t.type === 'bullet_list_open' || t.type === 'ordered_list_open') {
        const uende = schliessendes(tokens, j);
        unter += liste(tokens, j, uende, ctx, tiefe + 1);
        j = uende;
      } else if (t.type === 'inline') {
        text += (text ? ' ' : '') + inline(t, ctx);
      }
    }
    aus += `${einzug}${nummeriert ? '+' : '-'} ${text}\n${unter}`;
    i = ende;
  }
  return aus;
}

function tabelle(tokens, von, bis, ctx) {
  const zeilen = [];
  let kopfzeilen = 0;
  let imKopf = false;
  for (let i = von; i < bis; i++) {
    const t = tokens[i];
    if (t.type === 'thead_open') imKopf = true;
    if (t.type === 'thead_close') imKopf = false;
    if (t.type === 'tr_open') {
      zeilen.push([]);
      if (imKopf) kopfzeilen++;
    }
    if (t.type === 'th_open' || t.type === 'td_open') {
      zeilen[zeilen.length - 1].push(inline(tokens[i + 1], ctx));
    }
  }
  const spalten = Math.max(1, ...zeilen.map((z) => z.length));
  const zelle = (z) => `[${z}]`;
  // Der Kopf fett, wie im Editor -- Typst hebt ihn von sich aus nicht hervor.
  const kopf = zeilen.slice(0, kopfzeilen).map((z) => z.map((c) => `[#strong[${c}]]`).join(', '));
  const rumpf = zeilen.slice(kopfzeilen).map((z) => z.map(zelle).join(', '));
  return `#table(\n  columns: ${spalten},\n${kopf.length ? `  table.header(${kopf.join(', ')}),\n` : ''}${rumpf.map((r) => `  ${r},`).join('\n')}\n)`;
}

/** Inline-Inhalt eines `inline`-Tokens. */
function inline(t, ctx) {
  if (!t || !t.children) return '';
  let aus = '';
  const stapel = [];
  const fussnoten = []; // offene Fußnoten: {ebene im Stapel, Klammertiefe}
  for (const k of t.children) {
    switch (k.type) {
      case 'text':
        aus += text(k.content, ctx, stapel, fussnoten);
        break;
      case 'html_inline':
        // Das Kästchen einer Aufgabenliste kommt von markdown-it-task-lists
        // als <input>; alles andere HTML bleibt draußen (html: false).
        if (k.content.includes('task-list-item-checkbox')) aus += k.content.includes('checked') ? '☑' : '☐';
        break;
      case 'softbreak':
        aus += ' ';
        break;
      case 'hardbreak':
        aus += ' \\\n';
        break;
      case 'code_inline':
        aus += `#raw(${stringLiteral(k.content)})${ENDE}`;
        break;
      case 'strong_open':
        aus += '#strong[';
        stapel.push(`]${ENDE}`);
        break;
      case 'em_open':
        aus += '#emph[';
        stapel.push(`]${ENDE}`);
        break;
      case 's_open':
        aus += '#strike[';
        stapel.push(`]${ENDE}`);
        break;
      case 'link_open': {
        const ziel = k.attrGet('href') ?? '';
        aus += `#link(${stringLiteral(ziel)})[`;
        stapel.push(`]${ENDE}`);
        break;
      }
      case 'strong_close':
      case 'em_close':
      case 's_close':
      case 'link_close':
        // Eine darin offen gebliebene Fußnote endet mit der Formatierung.
        while (fussnoten.length && fussnoten[fussnoten.length - 1].ebene === stapel.length) {
          aus += stapel.pop();
          fussnoten.pop();
        }
        aus += stapel.pop() ?? '';
        break;
      case 'image': {
        const b = ctx.bild(k.attrGet('src') ?? '');
        const alt = k.content || k.attrGet('alt') || '';
        // In seiner natürlichen Größe (der Aufrufer rechnet sie aus), nicht
        // auf Satzbreite gezogen: Ein quadratisches Foto über die volle
        // Breite füllte sonst eine ganze Seite.
        const breite = b?.breiteCm ? `${b.breiteCm.toFixed(2)}cm` : '100%';
        aus += b
          ? `#box(image(${stringLiteral(b.datei)}, width: ${breite}))${ENDE}`
          : textMaskieren(alt);
        break;
      }
      case 'html_block':
        break;
      default:
        if (k.content) aus += textMaskieren(k.content);
        break;
    }
  }
  return (aus + stapel.reverse().join(''))
    .replace(new RegExp(`${ENDE}([;(.])`, 'g'), '\\$1')
    .replaceAll(ENDE, '');
}

/** Index des passenden `_close` zu tokens[i] (gleiche Verschachtelung). */
function schliessendes(tokens, i) {
  const art = tokens[i].type.replace(/_open$/, '');
  let tiefe = 0;
  for (let j = i; j < tokens.length; j++) {
    if (tokens[j].type === `${art}_open`) tiefe++;
    if (tokens[j].type === `${art}_close`) {
      tiefe--;
      if (tiefe === 0) return j;
    }
  }
  return tokens.length - 1;
}

/**
 * Viele Notizen beginnen mit `# Titel` -- derselbe Titel, den die Vorlage
 * schon setzt (oben auf der Seite bzw. als Kapitelüberschrift). Diese erste
 * Zeile fällt dann weg, sonst stünde er zweimal da.
 */
export function ohneTitelzeile(inhalt, titel) {
  return String(inhalt ?? '').replace(/^\s*#[ \t]+(.+?)[ \t]*(\r?\n|$)/, (zeile, h1) => (h1.trim() === String(titel).trim() ? '' : zeile));
}

/** Alle Bildpfade der Notiz, in der Reihenfolge ihres Auftretens. */
export function bildpfade(markdown) {
  const pfade = [];
  const besuchen = (liste) => {
    for (const t of liste ?? []) {
      if (t.type === 'image') pfade.push(t.attrGet('src') ?? '');
      if (t.children) besuchen(t.children);
    }
  };
  besuchen(md.parse(String(markdown ?? ''), {}));
  return [...new Set(pfade.filter(Boolean))];
}
