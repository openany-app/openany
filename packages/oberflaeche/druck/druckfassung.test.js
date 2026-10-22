import { describe, it, expect } from 'vitest';
import { ohneSortiervorsilbe, nachTitel, gliederung } from './druckfassungAlsPdf';
import { druckfassungVorlage } from './druckfassungVorlage';
import { markdownZuTypst, textMaskieren } from './markdownZuTypst';

describe('ohneSortiervorsilbe', () => {
  it('nimmt Vorsilben zum Sortieren weg', () => {
    expect(ohneSortiervorsilbe('01 Einleitung')).toBe('Einleitung');
    expect(ohneSortiervorsilbe('2. Fristen')).toBe('Fristen');
    expect(ohneSortiervorsilbe('3) Ergebnis')).toBe('Ergebnis');
    expect(ohneSortiervorsilbe('04_Anhang')).toBe('Anhang');
  });
  it('lässt Jahreszahlen und reine Zahlen stehen', () => {
    expect(ohneSortiervorsilbe('2026 Rückblick')).toBe('2026 Rückblick');
    expect(ohneSortiervorsilbe('42')).toBe('42');
    expect(ohneSortiervorsilbe('Einleitung')).toBe('Einleitung');
  });
});

describe('nachTitel', () => {
  it('sortiert Zahlen als Zahlen und ohne Rücksicht auf Groß/klein', () => {
    const titel = ['10 Schluss', '2 Hauptteil', '1 Einleitung', 'anhang', 'Ausblick'];
    expect(titel.map((title) => ({ title })).sort((a, b) => nachTitel(a, b)).map((n) => n.title))
      .toEqual(['1 Einleitung', '2 Hauptteil', '10 Schluss', 'anhang', 'Ausblick']);
  });
});

describe('obersteEbene', () => {
  it('rückt die Gliederung einer Notiz als Ganzes auf Ebene 2', () => {
    const t = markdownZuTypst('## Abschnitt\n\n### Unterabschnitt', { obersteEbene: 2 });
    expect(t).toContain('== Abschnitt');
    expect(t).toContain('=== Unterabschnitt');
  });
  it('auch wenn die Notiz mit # beginnt', () => {
    expect(markdownZuTypst('# Teil', { obersteEbene: 2 })).toContain('== Teil');
  });
});

describe('textMaskieren', () => {
  it('maskiert den Punkt nach einer Ziffer, damit keine Liste daraus wird', () => {
    expect(textMaskieren('14. Oktober')).toBe('14\\. Oktober');
  });
});

describe('druckfassungVorlage', () => {
  const angaben = { titel: 'Arbeit #1', untertitel: 'Unter', name: 'Tiffy', einrichtung: 'Uni\nFakultät', datum: '14. Oktober 2026', zusatz: 'Seminar', inhaltsverzeichnis: true };

  it('setzt die Vorgaben: Termes 12 pt, Heros für Überschriften, 2,5 cm, Silbentrennung', () => {
    const t = druckfassungVorlage({ angaben, kapitel: [], sprache: 'de-DE' });
    expect(t).toContain('font: "TeX Gyre Termes", size: 12pt, lang: "de", hyphenate: true');
    expect(t).toContain('#show heading: set text(font: "TeX Gyre Heros"');
    expect(t).toContain('margin: 2.5cm');
    expect(t).toContain('justify: true');
  });

  it('Titelblatt ohne Seitenzahl, mit maskierten Angaben', () => {
    const t = druckfassungVorlage({ angaben, kapitel: [], sprache: 'de' });
    expect(t).toContain('#page(numbering: none)[');
    expect(t).toContain('Arbeit \\#1');
    expect(t).toContain('Uni \\\nFakultät');
    expect(t).toContain('14\\. Oktober 2026');
  });

  it('Inhaltsverzeichnis nur, wenn eingeschaltet', () => {
    expect(druckfassungVorlage({ angaben, kapitel: [] })).toContain('#outline(');
    expect(druckfassungVorlage({ angaben: { ...angaben, inhaltsverzeichnis: false }, kapitel: [] })).not.toContain('#outline(');
  });

  it('jede Notiz wird ein Kapitel', () => {
    const t = druckfassungVorlage({ angaben, kapitel: [{ titel: 'Einleitung', rumpf: 'Text' }, { titel: 'Schluss', rumpf: 'Ende' }] });
    expect(t).toContain('= Einleitung\n\nText');
    expect(t).toContain('= Schluss\n\nEnde');
  });
});

describe('gliederung (Untermappen)', () => {
  const n = (titel, inhalt = `Text ${titel}`) => ({ titel, inhalt });
  const kurz = (abschnitte) => abschnitte.map((a) => `${a.ebene} ${a.titel}${a.inhalt ? '' : ' (leer)'} @${a.mappeId}`);

  it('Untermappe = Überschrift, ihre Notizen eine Ebene darunter; alles gemeinsam sortiert', () => {
    const baum = {
      id: 1, name: 'Arbeit',
      notizen: [n('03 Fazit'), n('01 Einleitung')],
      unter: [{ id: 2, name: '02 Grundlagen', notizen: [n('02 Fristen'), n('01 Begriffe')], unter: [] }],
    };
    expect(kurz(gliederung(baum))).toEqual([
      '1 Einleitung @1',
      '1 Grundlagen (leer) @2',
      '2 Begriffe @2',
      '2 Fristen @2',
      '1 Fazit @1',
    ]);
  });

  it('eine Notiz mit dem Namen der Untermappe ist deren Einleitungstext', () => {
    const baum = {
      id: 1, name: 'Arbeit', notizen: [],
      unter: [{ id: 2, name: '02 Grundlagen', notizen: [n('Grundlagen', 'Vorweg.'), n('Begriffe')], unter: [] }],
    };
    const [kopf, ...rest] = gliederung(baum);
    expect(kopf).toMatchObject({ titel: 'Grundlagen', ebene: 1, inhalt: 'Vorweg.', roherTitel: 'Grundlagen' });
    expect(kurz(rest)).toEqual(['2 Begriffe @2']);
  });

  it('leere Untermappen fallen weg, tiefe rücken weiter ein', () => {
    const baum = {
      id: 1, name: 'A', notizen: [n('x')],
      unter: [
        { id: 2, name: 'Leer', notizen: [], unter: [{ id: 3, name: 'Auch leer', notizen: [], unter: [] }] },
        { id: 4, name: 'B', notizen: [], unter: [{ id: 5, name: 'C', notizen: [n('tief')], unter: [] }] },
      ],
    };
    expect(kurz(gliederung(baum))).toEqual(['1 B (leer) @4', '2 C (leer) @5', '3 tief @5', '1 x @1']);
  });
});

describe('druckfassungVorlage: Ebenen', () => {
  it('setzt jeden Abschnitt auf seine Ebene, ohne Angabe als Kapitel', () => {
    const q = druckfassungVorlage({ kapitel: [{ titel: 'K', rumpf: '' }, { titel: 'U', ebene: 2, rumpf: 'x' }] });
    expect(q).toContain('\n= K\n');
    expect(q).toContain('\n== U\n\nx');
    expect(q).not.toContain('pagebreak()\n= U');
  });
});
