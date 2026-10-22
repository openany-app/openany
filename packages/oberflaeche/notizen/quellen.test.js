import { describe, it, expect } from 'vitest';
import { alsFormular, ausFormular, schluesselVorschlag, schluesselGueltig, kurzanzeige, vollstaendig } from './quellen';

const buch = {
  id: 'mueller2020',
  type: 'book',
  title: 'Beispielgesetz',
  author: [{ family: 'Müller', given: 'Hans' }, { literal: 'Institut Beispiel' }],
  issued: { 'date-parts': [[2020, 5]] },
  publisher: 'Beck',
  ISBN: '978-3-00-000000-0', // kennt das Formular nicht
};

describe('Formular <-> CSL', () => {
  it('kommt unverändert zurück, auch mit Feldern, die das Formular nicht zeigt', () => {
    expect(ausFormular(alsFormular(buch), buch)).toEqual(buch);
  });

  it('Namen: „Nachname, Vorname" je Zeile, ohne Komma eine Einrichtung', () => {
    const f = { ...alsFormular(buch), autoren: 'Weber, Anna\nBundesarbeitsgericht' };
    expect(ausFormular(f, buch).author).toEqual([{ family: 'Weber', given: 'Anna' }, { literal: 'Bundesarbeitsgericht' }]);
  });

  it('ein anderes Jahr ersetzt das Datum, leere Felder fallen weg', () => {
    const e = ausFormular({ ...alsFormular(buch), jahr: '2021', verlag: '' }, buch);
    expect(e.issued).toEqual({ 'date-parts': [[2021]] });
    expect(e).not.toHaveProperty('publisher');
  });

  it('wechselt die Art, fällt weg, was die neue nicht zeigt', () => {
    const e = ausFormular({ ...alsFormular(buch), typ: 'article-journal', enthalten: 'NZA' }, buch);
    expect(e.type).toBe('article-journal');
    expect(e['container-title']).toBe('NZA');
    expect(e).not.toHaveProperty('publisher');
    expect(e.ISBN).toBe(buch.ISBN);
  });

  it('ein Urteil hat ein Tagesdatum', () => {
    const f = { ...alsFormular({}), typ: 'legal_case', schluessel: 'bag2020', gericht: 'BAG', art: 'Urteil', datum: '2020-03-12', aktenzeichen: '2 AZR 123/19' };
    expect(ausFormular(f)).toEqual({
      id: 'bag2020', type: 'legal_case', authority: 'BAG', genre: 'Urteil', number: '2 AZR 123/19',
      issued: { 'date-parts': [[2020, 3, 12]] },
    });
    expect(alsFormular(ausFormular(f)).datum).toBe('2020-03-12');
  });
});

describe('Kurzname', () => {
  it('aus Verfasser und Jahr, bei Gleichstand mit a, b', () => {
    const f = { autoren: 'Müller, Hans', jahr: '2020' };
    expect(schluesselVorschlag(f)).toBe('muller2020');
    expect(schluesselVorschlag(f, new Set(['muller2020']))).toBe('muller2020a');
    expect(schluesselVorschlag(f, new Set(['muller2020', 'muller2020a']))).toBe('muller2020b');
  });

  it('ohne Verfasser aus Gericht oder Titel', () => {
    expect(schluesselVorschlag({ gericht: 'BAG', datum: '2020-03-12' })).toBe('bag2020');
    expect(schluesselVorschlag({ titel: 'Übungsgesetz', jahr: '1969' })).toBe('ubungsgesetz1969');
  });

  it('muss sich mit [@…] zitieren lassen', () => {
    expect(schluesselGueltig('mueller2020')).toBe(true);
    expect(schluesselGueltig('mit leer')).toBe(false);
    expect(schluesselGueltig('a;b')).toBe(false);
    expect(schluesselGueltig('')).toBe(false);
  });
});

describe('Anzeige', () => {
  it('zeigt Verfasser, Jahr und Titel', () => {
    expect(kurzanzeige(buch)).toBe('Müller und Institut Beispiel (2020): Beispielgesetz');
  });
  it('braucht Verfasser, Titel oder Gericht', () => {
    expect(vollstaendig({ autoren: '', titel: ' ' })).toBe(false);
    expect(vollstaendig({ gericht: 'BAG' })).toBe(true);
  });
});
