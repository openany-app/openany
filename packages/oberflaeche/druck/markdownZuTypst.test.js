import { describe, it, expect } from 'vitest';
import { markdownZuTypst, textMaskieren, wikilinksAufloesen, bildpfade } from './markdownZuTypst';

describe('textMaskieren', () => {
  it('lässt nichts als Typst gelten', () => {
    expect(textMaskieren('#tag $5 *fett* a_b `c` @x <y> [z] ~ = - + //'))
      .toBe('\\#tag \\$5 \\*fett\\* a\\_b \\`c\\` \\@x \\<y\\> \\[z\\] \\~ \\= \\- \\+ \\/\\/');
  });
  it('lässt Anführungszeichen für die Typografie stehen', () => {
    expect(textMaskieren('„so“ "auch"')).toBe('„so“ "auch"');
  });
});

describe('wikilinksAufloesen', () => {
  it('zeigt den Titel ohne Zettel-Kennung, oder die Anzeige', () => {
    expect(wikilinksAufloesen('siehe [[20260726180305 Klage]] und [[Frist|die Frist]]')).toBe('siehe Klage und die Frist');
  });
});

describe('markdownZuTypst', () => {
  it('übersetzt Überschriften, Absätze und Hervorhebungen', () => {
    const t = markdownZuTypst('# Titel\n\nEin **fetter** und *kursiver* Satz mit `code`.');
    expect(t).toContain('= Titel');
    expect(t).toContain('#strong[fetter]');
    expect(t).toContain('#emph[kursiver]');
    expect(t).toContain('#raw("code")');
  });

  it('übersetzt verschachtelte Listen', () => {
    const t = markdownZuTypst('- eins\n  - zwei\n- drei\n\n1. erstens\n2. zweitens');
    expect(t).toContain('- eins\n  - zwei\n- drei');
    expect(t).toContain('+ erstens\n+ zweitens');
  });

  it('übersetzt Aufgabenlisten in Kästchen', () => {
    const t = markdownZuTypst('- [ ] offen\n- [x] erledigt');
    expect(t).toContain('☐ offen');
    expect(t).toContain('☑ erledigt');
  });

  it('übersetzt Tabellen mit Kopf', () => {
    const t = markdownZuTypst('| A | B |\n|---|---|\n| 1 | 2 |');
    expect(t).toContain('columns: 2');
    expect(t).toContain('table.header([#strong[A]], [#strong[B]])');
    expect(t).toContain('[1], [2],');
  });

  it('setzt Code-Blöcke als String, damit nichts darin Typst wird', () => {
    const t = markdownZuTypst('```js\nconst a = "#x" // y\n```');
    expect(t).toContain('#raw(block: true, lang: "js", "const a = \\"#x\\" // y")');
  });

  it('macht aus Links echte Links', () => {
    expect(markdownZuTypst('[openany](https://openany.de)')).toContain('#link("https://openany.de")[openany]');
  });

  it('bettet bekannte Bilder ein und setzt bei unbekannten den Alt-Text', () => {
    const t = markdownZuTypst('![Skizze](a.png) ![Fehlt](b.png)', { bild: (src) => (src === 'a.png' ? { datei: 'bild-1.jpg', breiteCm: 8 } : null) });
    expect(t).toContain('image("bild-1.jpg", width: 8.00cm)');
    expect(t).toContain('Fehlt');
  });

  it('setzt rohes HTML als Text und führt nichts davon aus', () => {
    expect(markdownZuTypst('Text <script>x</script>')).toContain('\\<script\\>x\\<\\/script\\>');
  });

  it('ein Zitat wird ein Zitatblock', () => {
    expect(markdownZuTypst('> zitiert')).toContain('#quote(block: true)[\nzitiert\n]');
  });
});

describe('bildpfade', () => {
  it('sammelt jedes Bild einmal, in Reihenfolge', () => {
    expect(bildpfade('![a](x.png)\n\n- ![b](y.jpg) ![c](x.png)')).toEqual(['x.png', 'y.jpg']);
  });
});

describe('Zitate und Fußnoten', () => {
  const quellen = new Set(['mueller2020', 'weber2019']);
  const setzen = (md, zitiert = new Set()) => markdownZuTypst(md, { quellen, zitiert }).trim();

  it('[@a, S. 12] wird ein Zitat mit Seitenangabe -- auch so, wie der Editor es speichert', () => {
    const zitiert = new Set();
    expect(setzen('Text \\[@mueller2020, S. 12\\].', zitiert))
      .toBe('Text #cite(label("mueller2020"), supplement: [S. 12])\\.');
    expect([...zitiert]).toEqual(['mueller2020']);
  });

  it('[@a; @b] werden zwei Zitate nebeneinander (Typst fasst sie zusammen)', () => {
    expect(setzen('[@weber2019; @mueller2020, S. 3]'))
      .toBe('#cite(label("weber2019")) #cite(label("mueller2020"), supplement: [S. 3])');
  });

  it('@a im Satz wird erzählend zitiert, Satzzeichen bleiben draußen', () => {
    expect(setzen('Wie @mueller2020. zeigt')).toBe('Wie #cite(label("mueller2020"), form: "prose")\\. zeigt');
  });

  it('Unbekanntes und Adressen bleiben Text -- Typst bräche sonst ab', () => {
    const zitiert = new Set();
    expect(setzen('[@unbekannt] [siehe @mueller2020] a@mueller2020 @fremd', zitiert))
      .toBe('\\[\\@unbekannt\\] \\[siehe #cite(label("mueller2020"), form: "prose")\\] a\\@mueller2020 \\@fremd');
    expect(setzen('Ohne Liste [@mueller2020]', undefined).includes('#cite')).toBe(true);
    expect(markdownZuTypst('Ohne Liste [@mueller2020]')).not.toContain('#cite');
  });

  it('^[…] wird eine Fußnote, mit Formatierung, Zitat und Wikilink darin', () => {
    expect(setzen('Satz.\\^\\[Eine **fette** Anmerkung [@weber2019] zu [[Notiz|dort]].\\] Weiter.'))
      .toBe('Satz.#footnote[Eine #strong[fette] Anmerkung #cite(label("weber2019")) zu dort.] Weiter.');
  });

  it('was hinter einem Ausdruck steht, liest Typst nicht als dessen Fortsetzung', () => {
    // `;` verschwände, `(` und `.` hängte Typst an den Ausdruck an.
    expect(setzen('**fett**; danach')).toBe('#strong[fett]\\; danach');
    expect(setzen('*BAG*(Urteil) `x`.y')).toBe('#emph[BAG]\\(Urteil) #raw("x")\\.y');
    expect(setzen('Siehe [@weber2019]; ferner ^[a].')).toBe('Siehe #cite(label("weber2019"))\\; ferner #footnote[a]\\.');
    // Die Marke selbst kommt aus einer Notiz nicht durch.
    expect(setzen('a\uE000;b')).toBe('a;b');
  });

  it('verschränkte Klammern ergeben trotzdem gültiges Typst', () => {
    // Die Fußnote endet mit dem Fettdruck, in dem sie begann.
    expect(setzen('**fett ^[offen** danach]')).toBe('#strong[fett #footnote[offen]] danach\\]');
    // Eine ] im Fettdruck schließt die äußere Fußnote nicht.
    expect(setzen('^[a **b] c** d]')).toBe('#footnote[a #strong[b\\] c] d]');
    // Nie geschlossen: endet am Absatzende.
    expect(setzen('^[offen')).toBe('#footnote[offen]');
  });
});
