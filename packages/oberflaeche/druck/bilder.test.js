import { describe, it, expect } from 'vitest';
import { breiteAufPapier } from './bilder';
import { ohneTitelzeile } from './markdownZuTypst';

describe('breiteAufPapier', () => {
  it('ein kleines Bild bleibt klein (natürliche Größe bei 150 dpi)', () => {
    expect(breiteAufPapier(300, 200)).toBeCloseTo(5.08, 2);
  });
  it('ein breites Foto endet an der Satzbreite', () => {
    expect(breiteAufPapier(4000, 3000)).toBeCloseTo(16, 0);
  });
  it('ein hohes oder quadratisches Foto wird höchstens 12 cm hoch', () => {
    expect(breiteAufPapier(1024, 1024)).toBeCloseTo(12, 5);
    expect(breiteAufPapier(1000, 2000)).toBeCloseTo(6, 5);
  });
});

describe('ohneTitelzeile', () => {
  it('nimmt die erste Überschrift weg, wenn sie der Titel ist', () => {
    expect(ohneTitelzeile('# Fristen\n\nText', 'Fristen')).toBe('\nText');
  });
  it('lässt eine andere Überschrift stehen', () => {
    expect(ohneTitelzeile('# Anders\n\nText', 'Fristen')).toBe('# Anders\n\nText');
  });
  it('lässt eine Unterüberschrift stehen', () => {
    expect(ohneTitelzeile('## Fristen\n\nText', 'Fristen')).toBe('## Fristen\n\nText');
  });
});
