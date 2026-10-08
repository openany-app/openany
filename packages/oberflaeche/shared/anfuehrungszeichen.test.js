import { describe, it, expect } from 'vitest';
import { anfuehrungszeichenFuer, ANFUEHRUNG_AUF, ANFUEHRUNG_ZU } from './anfuehrungszeichen';

describe('anfuehrungszeichenFuer', () => {
  it('öffnet am Absatzanfang', () => {
    expect(anfuehrungszeichenFuer('')).toBe(ANFUEHRUNG_AUF);
  });

  it('öffnet nach Leerraum', () => {
    for (const davor of [' ', '\n', '\t']) {
      expect(anfuehrungszeichenFuer(davor)).toBe(ANFUEHRUNG_AUF);
    }
  });

  it('öffnet nach einer öffnenden Klammer', () => {
    for (const davor of ['(', '[', '{', '<']) {
      expect(anfuehrungszeichenFuer(davor)).toBe(ANFUEHRUNG_AUF);
    }
  });

  it('öffnet nach einem Gedankenstrich', () => {
    for (const davor of ['-', '–', '—']) {
      expect(anfuehrungszeichenFuer(davor)).toBe(ANFUEHRUNG_AUF);
    }
  });

  it('öffnet im Zitat im Zitat', () => {
    expect(anfuehrungszeichenFuer('„')).toBe(ANFUEHRUNG_AUF);
    expect(anfuehrungszeichenFuer('‚')).toBe(ANFUEHRUNG_AUF);
  });

  it('schließt hinter einem Wort', () => {
    for (const davor of ['t', 'ß', 'Z', '2']) {
      expect(anfuehrungszeichenFuer(davor)).toBe(ANFUEHRUNG_ZU);
    }
  });

  it('schließt hinter Satzzeichen', () => {
    // "Er kam." -- der Punkt steht im Zitat, danach wird geschlossen.
    for (const davor of ['.', ',', '!', '?', ':', ';']) {
      expect(anfuehrungszeichenFuer(davor)).toBe(ANFUEHRUNG_ZU);
    }
  });

  it('schließt hinter einer schließenden Klammer', () => {
    expect(anfuehrungszeichenFuer(')')).toBe(ANFUEHRUNG_ZU);
  });
});
