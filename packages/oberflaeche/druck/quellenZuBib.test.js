import { describe, it, expect } from 'vitest';
import { quellenZuBib, datum, literaturverzeichnis, bibBeilegen, stilFuer, BIB_DATEI, STIL_DATEI } from './quellenZuBib';

describe('quellenZuBib', () => {
  it('ein Buch mit Personen und einer Einrichtung', () => {
    const { bib, schluessel } = quellenZuBib([{
      id: 'mueller2020', type: 'book', title: 'Recht & Praxis {neu}', edition: '3',
      author: [{ family: 'Müller', given: 'Hans' }, { literal: 'Institut Beispiel' }],
      issued: { 'date-parts': [[2020]] }, publisher: 'Beck', 'publisher-place': 'München',
    }]);
    expect(bib).toBe(`@book{mueller2020,
  author = {Müller, Hans and {Institut Beispiel}},
  title = {Recht \\& Praxis neu},
  date = {2020},
  publisher = {Beck},
  location = {München},
  edition = {3},
}
`);
    expect([...schluessel]).toEqual(['mueller2020']);
  });

  it('Aufsatz, Beitrag, Webseite bekommen ihre Felder', () => {
    const { bib } = quellenZuBib([
      { id: 'a', type: 'article-journal', title: 'T', 'container-title': 'NZA', volume: 36, issue: '4', page: '201-207' },
      { id: 'b', type: 'chapter', title: 'T', 'container-title': 'Handbuch', editor: [{ family: 'Groß', given: 'M' }] },
      { id: 'c', type: 'webpage', title: 'T', URL: 'https://beispiel.test/?a=1&b=2', accessed: { 'date-parts': [[2026, 10, 14]] } },
    ]);
    expect(bib).toContain('@article{a,');
    expect(bib).toContain('journaltitle = {NZA}');
    expect(bib).toContain('pages = {201--207}');
    expect(bib).toContain('@incollection{b,');
    expect(bib).toContain('booktitle = {Handbuch}');
    expect(bib).toContain('editor = {Groß, M}');
    expect(bib).toContain('url = {https://beispiel.test/?a=1&b=2}');
    expect(bib).toContain('urldate = {2026-10-14}');
  });

  it('ein Urteil: Gericht als Verfasser, Fundstelle als publisher (das zeigt Typst)', () => {
    const { bib } = quellenZuBib([{
      id: 'bag2020', type: 'legal_case', authority: 'BAG', genre: 'Urteil', number: '2 AZR 123/19',
      issued: { 'date-parts': [[2020, 3, 12]] }, 'container-title': 'NZA 2020, 1001',
    }]);
    expect(bib).toContain('author = {{BAG}}');
    expect(bib).toContain('title = {Urteil, 12.03.2020, 2 AZR 123/19}');
    expect(bib).toContain('publisher = {NZA 2020, 1001}');
    expect(bib).toContain('date = {2020-03-12}');
  });

  it('lässt aus, was sich nicht zitieren ließe oder doppelt ist', () => {
    const { schluessel } = quellenZuBib([
      { id: 'mit leer', type: 'book', title: 'x' },
      { id: 'ok', type: 'book', title: 'x' },
      { id: 'ok', type: 'book', title: 'y' },
      { id: 'leer', type: 'book' },
      null,
    ]);
    expect([...schluessel]).toEqual(['ok']);
  });

  it('Datum aus Teilen oder freiem Text', () => {
    expect(datum({ 'date-parts': [[2020, 3]] })).toBe('2020-03');
    expect(datum({ raw: 'Frühjahr 2019' })).toBe('2019');
    expect(datum(undefined)).toBe('');
  });
});

describe('Verzeichnis', () => {
  it('Harvard mit der Überschrift der Sprache', () => {
    expect(literaturverzeichnis('de')).toBe('#bibliography("quellen.bib", title: [Literaturverzeichnis], style: "quellen.csl")');
    expect(literaturverzeichnis('xx')).toBe('#bibliography("quellen.bib", style: "quellen.csl")');
  });

  it('die Datei kommt nur dazu, wenn zitiert wird', () => {
    const dateien = new Map();
    expect(bibBeilegen(dateien, '@book{a}', new Set())).toBe(false);
    expect(dateien.has(BIB_DATEI)).toBe(false);
    expect(bibBeilegen(dateien, '@book{a}', new Set(['a']))).toBe(true);
    expect(new TextDecoder().decode(dateien.get(BIB_DATEI))).toBe('@book{a}');
    expect(dateien.has(STIL_DATEI)).toBe(true);
  });

  it('der Stil bekommt nur auf Deutsch den Punkt nach dem Tag', () => {
    expect(stilFuer('de')).toContain('<locale xml:lang="de">');
    expect(stilFuer('de')).not.toContain('<date-part name="day" suffix=" "/>');
    expect(stilFuer('en')).not.toContain('<locale xml:lang="de">');
    expect(stilFuer('en')).toContain('<date-part name="day" suffix=" "/>');
  });
});
