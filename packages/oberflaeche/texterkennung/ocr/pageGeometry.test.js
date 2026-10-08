import { describe, it, expect } from 'vitest';
import {
  orderCorners, solveHomography, applyHomography, targetSize, plausibleQuad,
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

describe('solveHomography', () => {
  // Die Probe aufs Exempel: Die Transformation muss die Stützpunkte, aus denen
  // sie berechnet wurde, exakt auf ihre Ziele abbilden.
  it('bildet die vier Stützpunkte exakt ab', () => {
    const ziel = [P(0, 0), P(800, 0), P(800, 1000), P(0, 1000)];
    const h = solveHomography(SCHRAEG, ziel);

    SCHRAEG.forEach((q, i) => {
      const p = applyHomography(h, q.x, q.y);
      expect(p.x).toBeCloseTo(ziel[i].x, 6);
      expect(p.y).toBeCloseTo(ziel[i].y, 6);
    });
  });

  // Hin und zurück muss wieder am Ausgangspunkt landen – auch für Punkte, die
  // keine Stützpunkte waren. Genau da zeigt sich, ob die Abbildung stimmt oder
  // nur an den vier Ecken zufällig passt.
  it('ist umkehrbar, auch abseits der Stützpunkte', () => {
    const ziel = [P(0, 0), P(800, 0), P(800, 1000), P(0, 1000)];
    const hin = solveHomography(SCHRAEG, ziel);
    const zurueck = solveHomography(ziel, SCHRAEG);

    for (const [x, y] of [[300, 400], [500, 900], [700, 200]]) {
      const t = applyHomography(hin, x, y);
      const r = applyHomography(zurueck, t.x, t.y);
      expect(r.x).toBeCloseTo(x, 4);
      expect(r.y).toBeCloseTo(y, 4);
    }
  });

  it('bildet eine reine Verschiebung als solche ab', () => {
    const von = [P(0, 0), P(10, 0), P(10, 10), P(0, 10)];
    const nach = [P(5, 7), P(15, 7), P(15, 17), P(5, 17)];
    const p = applyHomography(solveHomography(von, nach), 5, 5);
    expect(p.x).toBeCloseTo(10, 6);
    expect(p.y).toBeCloseTo(12, 6);
  });

  it('gibt null zurück, wenn die Ecken auf einer Linie liegen', () => {
    const linie = [P(0, 0), P(10, 10), P(20, 20), P(30, 30)];
    expect(solveHomography(linie, [P(0, 0), P(1, 0), P(1, 1), P(0, 1)])).toBeNull();
  });
});

describe('targetSize', () => {
  it('nimmt je Achse die längere der gegenüberliegenden Kanten', () => {
    // Oberkante 762, Unterkante 884 (schräge Kanten zählen mit ihrer echten
    // Länge, nicht mit der x-Differenz). Die längere gewinnt, sonst bliebe der
    // entfernte Teil des Blattes dauerhaft gestaucht.
    const s = targetSize(SCHRAEG);
    expect(s.width).toBe(884);
    expect(s.height).toBeGreaterThan(1000);
  });

  it('folgt der längeren Kante auch dann, wenn sie die untere ist', () => {
    // Oben 200 breit, unten 800: Das Ergebnis muss 800 sein.
    const s = targetSize([P(300, 0), P(500, 0), P(900, 600), P(100, 600)]);
    expect(s.width).toBe(800);
  });

  it('lässt ein Rechteck unverändert', () => {
    expect(targetSize([P(0, 0), P(400, 0), P(400, 300), P(0, 300)]))
      .toEqual({ width: 400, height: 300 });
  });
});

// Diese Prüfung ist die Sicherung gegen Datenverlust: Das Originalfoto wird
// nach der Umwandlung nicht aufgehoben, ein falsches Viereck schneidet also
// unwiederbringlich ab. Im Zweifel muss sie ablehnen.
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
