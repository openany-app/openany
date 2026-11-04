// Eine Notiz als Brief -- nach DIN 5008, Form B, für den Fensterumschlag
// (DL oder C6/5). Eine reine Funktion wie notizVorlage.js.
//
// DIE MASSE STEHEN AUF DEM PAPIER, nicht im Satzspiegel: Das Anschriftfeld
// beginnt 20 mm von links und 45 mm von oben (85 × 45 mm, oben 17,7 mm
// Rücksendeangabe, darunter die Anschrift), der Absenderblock 125 mm von
// links und 50 mm von oben. Typst platziert relativ zum Satzspiegel; die
// Abstände unten rechnen die Ränder heraus.
//
// Ort und Datum stehen unter beiden Blöcken rechtsbündig, darunter der
// Betreff (der Notiztitel) und der Text.
import { stringLiteral, textMaskieren } from './markdownZuTypst';

const RAND_LINKS = 25; // mm
const RAND_OBEN = 27; // mm

/** Mehrzeiliger Text als Typst-Zeilen: maskiert, mit Zeilenumbrüchen. */
function zeilen(text) {
  return String(text ?? '')
    .split(/\r?\n/)
    .map((z) => z.trim())
    .filter(Boolean)
    .map(textMaskieren)
    .join(' \\\n');
}

/** Die Rücksendeangabe: der Absender in einer Zeile, klein über der Anschrift. */
function ruecksendezeile(absender) {
  return String(absender ?? '')
    .split(/\r?\n/)
    .map((z) => z.trim())
    .filter(Boolean)
    .join(' · ');
}

/**
 * @param {object} p
 * @param {string} p.titel       Betreff (der Notiztitel)
 * @param {string} p.empfaenger  mehrzeilig
 * @param {string} p.absender    mehrzeilig
 * @param {string} p.ortDatum    z. B. „Berlin, 1. November 2026"
 * @param {string} p.rumpf       der Text, schon in Typst
 * @param {string} p.sprache
 */
export function briefVorlage({ titel, empfaenger, absender, ortDatum, rumpf, sprache }) {
  const lang = String(sprache || 'de').slice(0, 2).toLowerCase();
  const mm = (wert) => `${wert}mm`;
  return `#set document(title: ${stringLiteral(titel)})
#set page(paper: "a4", margin: (left: ${mm(RAND_LINKS)}, right: 20mm, top: ${mm(RAND_OBEN)}, bottom: 25mm), footer: context {
  let gesamt = counter(page).final().first()
  if gesamt > 1 { align(right, text(size: 8pt)[#counter(page).display() / #gesamt]) }
})
#set text(font: "Libertinus Serif", size: 11pt, lang: ${stringLiteral(lang)}, hyphenate: true)
#set par(justify: false, leading: 0.65em, spacing: 1.1em)
#show heading: set block(above: 1.3em, below: 0.6em)
#show raw: set text(font: "DejaVu Sans Mono", size: 9pt)
#show link: set text(fill: rgb("#1f4fb8"))
#set table(stroke: 0.5pt + luma(170), inset: 5pt)

#place(top + left, dx: ${mm(20 - RAND_LINKS)}, dy: ${mm(45 - RAND_OBEN)})[
  #block(width: 85mm, height: 45mm)[
    #block(height: 17.7mm, width: 100%)[
      #align(bottom)[#text(size: 7pt, fill: luma(90))[${textMaskieren(ruecksendezeile(absender))}]]
    ]
    #text(size: 11pt)[${zeilen(empfaenger)}]
  ]
]
#place(top + left, dx: ${mm(125 - RAND_LINKS)}, dy: ${mm(50 - RAND_OBEN)})[
  #block(width: 65mm)[#text(size: 9.5pt)[${zeilen(absender)}]]
]
#place(top + right, dy: ${mm(93 - RAND_OBEN)})[${textMaskieren(ortDatum)}]

#v(${mm(103.4 - RAND_OBEN)})
#text(weight: "bold")[${textMaskieren(titel)}]
#v(0.8em)

${rumpf}`;
}
