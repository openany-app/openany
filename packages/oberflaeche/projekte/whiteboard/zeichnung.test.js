import { describe, it, expect } from 'vitest';
import { zumSpeichern, fassung, excalidrawSprache, alsLink, istVerweisLink, bildVerweise, dateiId, platzhalter, eingepasst } from './zeichnung';

describe('Whiteboard-Zeichnung', () => {
  it('speichert nur lebende Elemente, Bilder nur als Verweis', () => {
    const z = zumSpeichern([
      { id: 'a', type: 'rectangle', version: 2 },
      { id: 'b', type: 'arrow', version: 5, isDeleted: true },
      { id: 'c', type: 'image', version: 1, fileId: 'f' },
      { id: 'd', type: 'image', version: 1, fileId: 'g', customData: { verweis: { art: 'bild', freigabe: 'F', uuid: 'U' } } },
    ]);
    expect(z.elements.map((e) => e.id)).toEqual(['a', 'd']);
    expect(z).not.toHaveProperty('files');
  });

  it('Verweise: Link, Datei-Kennung, Bilder je Datei einmal', () => {
    const v = { art: 'notiz', freigabe: 'F1', uuid: '20261022 1' };
    expect(alsLink(v)).toBe('openany:verweis/notiz/F1/20261022%201');
    expect(istVerweisLink(alsLink(v))).toBe(true);
    expect(istVerweisLink('https://beispiel.test')).toBe(false);

    const b = { art: 'bild', freigabe: 'F', uuid: 'U' };
    const liste = bildVerweise([
      { type: 'image', fileId: dateiId(b), customData: { verweis: b } },
      { type: 'image', fileId: dateiId(b), customData: { verweis: b } },
      { type: 'image', fileId: 'x', isDeleted: true, customData: { verweis: b } },
      { type: 'rectangle', customData: { verweis: b } },
    ]);
    expect(liste).toEqual([{ fileId: 'openany-F-U', verweis: b }]);
  });

  it('Platzhalter und Größe', () => {
    expect(decodeURIComponent(platzhalter('<weg>'))).toContain('&lt;weg&gt;');
    expect(eingepasst(1200, 600)).toEqual({ width: 360, height: 180 });
    expect(eingepasst(100, 50)).toEqual({ width: 100, height: 50 });
    expect(eingepasst(0, 0).width).toBe(360);
  });

  it('der Fingerabdruck ändert sich nur mit den Elementen', () => {
    const a = [{ id: 'a', version: 1 }];
    expect(fassung(a)).toBe(fassung([{ id: 'a', version: 1 }]));
    expect(fassung(a)).not.toBe(fassung([{ id: 'a', version: 2 }]));
    expect(fassung(a)).not.toBe(fassung([{ id: 'a', version: 1, isDeleted: true }]));
    expect(fassung([])).toBe(fassung(undefined));
  });

  it('kennt die Sprachen der App', () => {
    expect(excalidrawSprache('de')).toBe('de-DE');
    expect(excalidrawSprache('uk')).toBe('uk-UA');
    expect(excalidrawSprache('xx')).toBe('en');
  });
});
