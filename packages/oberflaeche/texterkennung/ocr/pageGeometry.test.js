import { describe, it, expect } from 'vitest';
import {
  orderCorners, plausibleQuad,
} from './pageGeometry';

const P = (x, y) => ({ x, y });
// Ein schräg fotografiertes Blatt: oben schmaler als unten (nach hinten gekippt).
const SCHRAEG = [P(120, 80), P(880, 140), P(940, 1180), P(60, 1100)];

describe('orderCorners', () => {
  it('sortiert nach oben-links, oben-rechts, unten-rechts, unten-links', () => {
    const gemischt = [P(940, 1180), P(120, 80), P(60, 1100), P(880, 140)];
    expect(orderCorners(gemischt)).toEqual(SCHRAEG);
  });

  it('ist gegen die Eingabereihenfolge unempfindlich', () => {
    const a = orderCorners([...SCHRAEG]);
    const b = orderCorners([...SCHRAEG].reverse());
    expect(a).toEqual(b);
  });

  it('verweigert alles, was keine vier Ecken sind', () => {
    expect(() => orderCorners([P(0, 0), P(1, 1), P(2, 2)])).toThrow();
  });
});

describe('plausibleQuad', () => {
  const B = 1000, H = 1300;

  it('nimmt ein sauber erkanntes Blatt an', () => {
    expect(plausibleQuad(SCHRAEG, B, H)).toBe(true);
  });

  it('lehnt einen zu kleinen Ausschnitt ab (eher ein Logo als ein Blatt)', () => {
    expect(plausibleQuad([P(400, 500), P(600, 500), P(600, 700), P(400, 700)], B, H)).toBe(false);
  });

  it('lehnt ab, wenn praktisch das ganze Bild umfasst wird – da gibt es nichts zu schneiden', () => {
    expect(plausibleQuad([P(0, 0), P(1000, 0), P(1000, 1300), P(0, 1300)], B, H)).toBe(false);
  });

  it('lehnt ein nicht konvexes Viereck ab (eingedrückte Ecke)', () => {
    expect(plausibleQuad([P(50, 50), P(950, 50), P(500, 650), P(50, 1250)], B, H)).toBe(false);
  });

  it('lehnt zu spitze Winkel ab – eine Blattecke ist rechtwinklig', () => {
    // Fast ein Dreieck: die obere Ecke läuft auf rund 45° zusammen. So sieht
    // kein Blatt aus, egal aus welchem Winkel fotografiert.
    expect(plausibleQuad([P(500, 100), P(950, 1200), P(500, 1250), P(50, 1200)], B, H)).toBe(false);
  });

  it('nimmt ein deutlich perspektivisch verzerrtes, aber echtes Blatt noch an', () => {
    // Von schräg oben fotografiert: oben schmal, unten breit – die Winkel
    // bleiben in der Nähe des rechten Winkels.
    expect(plausibleQuad([P(250, 120), P(760, 120), P(950, 1200), P(60, 1200)], B, H)).toBe(true);
  });

  it('lehnt entartete Eingaben ab, statt sie durchzureichen', () => {
    expect(plausibleQuad(null, B, H)).toBe(false);
    expect(plausibleQuad([P(0, 0), P(1, 1), P(2, 2)], B, H)).toBe(false);
    expect(plausibleQuad([P(0, 0), P(NaN, 0), P(1, 1), P(0, 1)], B, H)).toBe(false);
  });
});
