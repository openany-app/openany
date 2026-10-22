// Die Druckfassung einer Mappe: Titelblatt, Inhaltsverzeichnis, Kapitel.
//
// Festgelegt mit Tiffy am 14.10.2026: TeX Gyre Termes 12 pt im Text,
// TeX Gyre Heros für Überschriften, 2,5 cm Ränder rundum, eineinhalbzeilig,
// Blocksatz mit Silbentrennung, Seitenzahlen ab Seite 2 (das Titelblatt
// zählt mit, zeigt aber keine), Inhaltsverzeichnis zum An- und Ausschalten,
// Kapitel alphabetisch nach Titel. Eine reine Funktion -- prüfbar ohne
// Browser (druckfassungVorlage.test.js).
import { stringLiteral, textMaskieren } from './markdownZuTypst';
import { literaturverzeichnis } from './quellenZuBib';

/** Satzbreite: A4 (21 cm) minus 2 × 2,5 cm. */
export const SATZBREITE_CM = 16;

const HEROS = '"TeX Gyre Heros"';

/** Mehrzeiliger Text -> Typst, Zeilen mit `\` getrennt. */
const zeilen = (text) => String(text ?? '').split(/\r?\n/).map((z) => textMaskieren(z.trim())).filter(Boolean).join(' \\\n');

/**
 * @param {object} o
 * @param {object} o.angaben  titel, untertitel, name, einrichtung, datum, zusatz, inhaltsverzeichnis
 * @param {{titel: string, rumpf: string, ebene?: number}[]} o.kapitel  schon in
 *        Typst übersetzt, in Reihenfolge; ohne Ebene ein Kapitel (1)
 * @param {string} o.sprache
 * @param {boolean} [o.literatur]  ob zitiert wird -- dann folgt das
 *        Literaturverzeichnis auf eigener Seite (die Datei legt der Aufrufer bei)
 */
export function druckfassungVorlage({ angaben = {}, kapitel = [], sprache = 'de', literatur = false }) {
  const lang = String(sprache || 'de').slice(0, 2).toLowerCase();
  const a = angaben;
  const titel = a.titel || '';

  // ZEILENABSTAND „EINEINHALBFACH" wie in einer Textverarbeitung, denn daran
  // messen Hochschulen: dort 1,5 × der einfache Abstand (≈ 1,15 × Schriftgröße),
  // also rund 20,7 pt von Grundlinie zu Grundlinie bei 12 pt. Typst misst
  // `leading` dagegen als Lücke zwischen den Zeilen (Versalhöhe bis Grundlinie
  // bleibt außen vor) -- bei Termes rund 0,66 em. Daher 1,06 em.
  const titelblatt = `#page(numbering: none)[
  #set align(center)
  #set par(justify: false, leading: 0.6em)
  #text(font: ${HEROS}, size: 12pt)[${zeilen(a.einrichtung)}]
  #v(1fr)
  #text(font: ${HEROS}, size: 20pt, weight: "bold", hyphenate: false)[${textMaskieren(titel)}]
${a.untertitel ? `  #v(0.8em)\n  #text(font: ${HEROS}, size: 14pt, hyphenate: false)[${textMaskieren(a.untertitel)}]\n` : ''}  #v(1fr)
  ${zeilen(a.zusatz)}
  #v(2.5em)
  ${[a.name, a.datum].filter(Boolean).map((z) => textMaskieren(z)).join(' \\\n  ')}
]`;

  const verzeichnis = a.inhaltsverzeichnis
    ? `#outline(depth: 3, indent: auto)
#pagebreak()`
    : '';

  // Jeder Abschnitt auf seiner Ebene: 1 für Kapitel, tiefer für Notizen in
  // Untermappen (druckfassungAlsPdf.js baut die Gliederung).
  const rumpf = kapitel.map((k) => `${'='.repeat(Math.max(1, k.ebene ?? 1))} ${textMaskieren(k.titel)}\n\n${k.rumpf ?? ''}`).join('\n');

  return `#set document(title: ${stringLiteral(titel)}${a.name ? `, author: ${stringLiteral(a.name)}` : ''})
#set page(paper: "a4", margin: 2.5cm, numbering: "1", number-align: center)
#set text(font: "TeX Gyre Termes", size: 12pt, lang: ${stringLiteral(lang)}, hyphenate: true)
#set par(justify: true, leading: 1.06em, spacing: 1.5em)
#set heading(numbering: "1.1")
#show heading: set text(font: ${HEROS}, weight: "bold", hyphenate: false)
#show heading: set par(justify: false)
#show heading.where(level: 1): set text(size: 15pt)
#show heading.where(level: 2): set text(size: 13pt)
#show heading: set block(above: 1.8em, below: 1em)
#show outline: set heading(numbering: none)
#show raw: set text(font: "DejaVu Sans Mono", size: 10pt)
#show raw.where(block: true): block.with(fill: luma(244), inset: 8pt, radius: 3pt, width: 100%)
#show quote.where(block: true): it => block(stroke: (left: 2pt + luma(190)), inset: (left: 10pt, y: 3pt), it.body)
#set table(stroke: 0.5pt + luma(170), inset: 5pt)
#show table: set par(justify: false)
#show footnote.entry: set text(size: 10pt)
#show footnote.entry: set par(leading: 0.65em)

${titelblatt}

${verzeichnis}

${rumpf}${literatur ? `\n#pagebreak()\n${literaturverzeichnis(lang)}\n` : ''}`;
}
