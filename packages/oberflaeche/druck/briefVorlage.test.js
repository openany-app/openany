import { describe, it, expect } from 'vitest';
import { briefVorlage } from './briefVorlage';

const brief = (mehr = {}) => briefVorlage({
  titel: 'Kündigung #1',
  empfaenger: 'Beispiel GmbH\nMusterstraße 1\n12345 Beispielstadt',
  absender: 'Erika Beispiel\nWeg 2\n54321 Probedorf',
  ortDatum: 'Probedorf, 1. November 2026',
  rumpf: 'Sehr geehrte Damen und Herren,',
  sprache: 'de-DE',
  ...mehr,
});

describe('briefVorlage', () => {
  it('setzt Anschriftfeld und Absender nach DIN 5008 Form B', () => {
    const t = brief();
    // Anschriftfeld 20 mm von links, 45 mm von oben -- relativ zum Rand (25/27 mm).
    expect(t).toContain('dx: -5mm, dy: 18mm');
    // Absenderblock 125 mm von links, 50 mm von oben.
    expect(t).toContain('dx: 100mm, dy: 23mm');
    expect(t).toContain('Beispiel GmbH \\\nMusterstraße 1 \\\n12345 Beispielstadt');
  });

  it('nennt den Absender klein über der Anschrift, in einer Zeile', () => {
    expect(brief()).toContain('Erika Beispiel · Weg 2 · 54321 Probedorf');
  });

  it('maskiert Betreff und Datum, endet mit dem Text', () => {
    const t = brief();
    expect(t).toContain('#text(weight: "bold")[Kündigung \\#1]');
    expect(t).toContain('Probedorf, 1\\. November 2026');
    expect(t).toContain('lang: "de"');
    expect(t.trim().endsWith('Sehr geehrte Damen und Herren,')).toBe(true);
  });

  it('lässt leere Zeilen weg', () => {
    expect(brief({ empfaenger: '\nA\n\n  \nB\n' })).toContain('#text(size: 11pt)[A \\\nB]');
  });
});
