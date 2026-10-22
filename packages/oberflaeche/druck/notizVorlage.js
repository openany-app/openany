// Die Seite um eine Notiz herum -- als reine Funktion, damit sie sich ohne
// Browser prüfen lässt (und mit echtem Typst, siehe notizVorlage.test.js).
import { stringLiteral, textMaskieren } from './markdownZuTypst';
import { literaturverzeichnis } from './quellenZuBib';

/** Die Seite um den Inhalt herum: schlicht, gut lesbar, mit Silbentrennung. */
export function vorlage({ titel, stand, rumpf, sprache, literatur = false }) {
  const lang = String(sprache || 'de').slice(0, 2).toLowerCase();
  return `#set document(title: ${stringLiteral(titel)})
#set page(paper: "a4", margin: (x: 2.3cm, y: 2.4cm), numbering: "1", number-align: center)
#set text(font: "Libertinus Serif", size: 11.5pt, lang: ${stringLiteral(lang)}, hyphenate: true)
#set par(justify: true, leading: 0.7em, spacing: 1.15em)
#show heading: set block(above: 1.5em, below: 0.75em)
#show raw: set text(font: "DejaVu Sans Mono", size: 9pt)
#show raw.where(block: true): block.with(fill: luma(244), inset: 8pt, radius: 3pt, width: 100%)
#show link: set text(fill: rgb("#1f4fb8"))
#set table(stroke: 0.5pt + luma(170), inset: 5pt)
#show quote.where(block: true): it => block(stroke: (left: 2pt + luma(190)), inset: (left: 10pt, y: 3pt), it.body)
#show table: set par(justify: false)

#block(below: 1.2em)[
  #text(size: 19pt, weight: "bold", hyphenate: false)[${textMaskieren(titel)}] \\
  #v(0.2em)
  #text(size: 9pt, fill: luma(120))[${textMaskieren(stand)}]
]

${rumpf}${literatur ? `\n${literaturverzeichnis(lang)}\n` : ''}`;
}

