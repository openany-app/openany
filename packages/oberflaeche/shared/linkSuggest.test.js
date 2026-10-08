import { describe, it, expect } from 'vitest';
import { suggestTargets, SUGGEST_MAX } from '@oberflaeche/shared/linkSuggest';

/** Kurzschreibweise für ein Ziel. */
const z = (kind, id, title) => ({ kind, id, title });

/** Wie der Server liefert: nach Arten sortiert, Notizen zuerst. */
const vieleNotizen = (n) => Array.from({ length: n }, (_, i) => z('note', i + 1, `Notiz ${String(i + 1).padStart(2, '0')}`));

describe('suggestTargets', () => {
  it('lässt jede Art zu Wort kommen, auch neben vielen Notizen', () => {
    // Der Fall aus der Praxis: 52 Notizen, dann Dateien und ein Album.
    // Vorher wurden schlicht die ersten acht Treffer genommen – es kamen
    // ausschließlich Notizen, und dass sich eine Datei verlinken lässt,
    // erfuhr niemand.
    const kandidaten = [
      ...vieleNotizen(52),
      z('file', 1, 'Konzern.pdf'),
      z('folder', 2, 'UniStuff'),
      z('album', 3, 'Aufbau'),
    ];

    const arten = new Set(suggestTargets(kandidaten, '').map((s) => s.kind));

    expect(arten).toEqual(new Set(['note', 'file', 'folder', 'album']));
  });

  it('stellt den besten Treffer nach vorn', () => {
    const kandidaten = [
      z('note', 1, 'Einkaufsliste Wochenende'),
      z('board', 2, 'Einkauf'),
      z('file', 3, 'Beim Einkauf beachten.pdf'),
    ];

    // „Einkauf" beginnt mit dem Getippten, die Datei enthält es nur mittig.
    const [erster] = suggestTargets(kandidaten, 'einkauf');

    expect(erster.title).toBe('Einkauf');
  });

  it('sucht ohne Rücksicht auf Groß- und Kleinschreibung', () => {
    const kandidaten = [z('note', 1, 'Getränkeliste')];

    expect(suggestTargets(kandidaten, 'GETRÄNK')).toHaveLength(1);
    expect(suggestTargets(kandidaten, 'getränk')).toHaveLength(1);
  });

  it('gibt bei leerer Eingabe einen Querschnitt statt der ersten Art', () => {
    // Das ist der Fall des Verweis-Knopfes: Es wurde nichts getippt, und
    // die Liste soll zeigen, WAS sich überhaupt verlinken lässt.
    const kandidaten = [...vieleNotizen(20), z('board', 1, 'Sprint'), z('photo', 2, 'IMG_1234')];

    const arten = suggestTargets(kandidaten, '').map((s) => s.kind);

    expect(arten).toContain('board');
    expect(arten).toContain('photo');
  });

  it('hält die Obergrenze ein', () => {
    expect(suggestTargets(vieleNotizen(99), '')).toHaveLength(SUGGEST_MAX);
    expect(suggestTargets(vieleNotizen(99), '', 3)).toHaveLength(3);
  });

  it('nimmt weniger, wenn es weniger gibt – ohne Leerplätze', () => {
    const aus = suggestTargets([z('note', 1, 'Eins'), z('board', 2, 'Zwei')], '');

    expect(aus).toHaveLength(2);
    expect(aus.every((s) => s != null)).toBe(true);
  });

  it('liefert nichts bei fehlender Übereinstimmung', () => {
    expect(suggestTargets([z('note', 1, 'Getränke')], 'xyz')).toEqual([]);
  });

  it('verträgt leere und unvollständige Eingaben', () => {
    // Ein Ziel ohne Titel (etwa ein Ort ohne Namen) darf die Liste nicht
    // sprengen – auffindbar wäre es ohnehin nicht.
    expect(suggestTargets([], '')).toEqual([]);
    expect(suggestTargets(null, '')).toEqual([]);
    expect(suggestTargets([{ kind: 'place', id: 1, title: null }], '')).toEqual([]);
    expect(suggestTargets([z('note', 1, 'Eins')], null)).toHaveLength(1);
  });
});
