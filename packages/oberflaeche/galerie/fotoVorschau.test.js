import { describe, it, expect } from 'vitest';
import { vorschauMasse, brauchtVorschau, VORSCHAU_KANTE } from './fotoVorschau';

describe('vorschauMasse', () => {
  it('bringt die längste Kante auf 480 – quer wie hochkant', () => {
    expect(vorschauMasse(4000, 3000)).toEqual({ breite: 480, hoehe: 360 });
    expect(vorschauMasse(3000, 4000)).toEqual({ breite: 360, hoehe: 480 });
  });

  it('vergrößert nie', () => {
    expect(vorschauMasse(200, 100)).toEqual({ breite: 200, hoehe: 100 });
  });

  it('macht aus einem sehr schmalen Bild keine Null', () => {
    expect(vorschauMasse(10000, 2)).toEqual({ breite: 480, hoehe: 1 });
  });

  it('gibt bei fehlenden Maßen nichts', () => {
    expect(vorschauMasse(0, 100)).toBeNull();
    expect(vorschauMasse(undefined, 100)).toBeNull();
  });

  it('dieselbe Kante wie Server und App', () => {
    expect(VORSCHAU_KANTE).toBe(480);
  });
});

describe('brauchtVorschau', () => {
  it('Rasterbilder ja, SVG und Nicht-Bilder nein', () => {
    expect(brauchtVorschau({ type: 'image/jpeg' })).toBe(true);
    expect(brauchtVorschau({ type: 'image/heic' })).toBe(true);
    expect(brauchtVorschau({ type: 'image/svg+xml' })).toBe(false);
    expect(brauchtVorschau({ type: 'application/pdf' })).toBe(false);
    expect(brauchtVorschau(null)).toBe(false);
  });
});
