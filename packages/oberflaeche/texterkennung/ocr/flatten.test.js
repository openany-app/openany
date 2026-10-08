import { describe, it, expect } from 'vitest';
import { backgroundGrid, smoothGrid, sampleGrid, flatten, beurteileAusleuchtung } from './flatten';

/**
 * Ein Blatt mit Schatten: Die Helligkeit fällt von links (240) nach rechts
 * (90) ab, darauf sitzt dunkle Schrift (Wert 30) in regelmäßigen Balken.
 * Genau der Fall, den ein abfotografiertes Dokument zeigt.
 */
function blattMitSchatten(width, height, { schrift = true } = {}) {
  const rgba = new Uint8ClampedArray(width * height * 4);
  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < width; x += 1) {
      const beleuchtung = 240 - (150 * x) / width;
      // Schriftbalken: 6 px hoch alle 20 px, mit Rand.
      const istSchrift = schrift && y % 20 < 6 && x > width * 0.1 && x < width * 0.9;
      const wert = istSchrift ? beleuchtung * 0.15 : beleuchtung;
      const i = (y * width + x) * 4;
      rgba[i] = rgba[i + 1] = rgba[i + 2] = wert;
      rgba[i + 3] = 255;
    }
  }
  return rgba;
}

const lumaAt = (rgba, width, x, y) => rgba[(y * width + x) * 4];

describe('backgroundGrid', () => {
  it('nimmt je Kachel das Maximum, nicht den Mittelwert', () => {
    // Eine Kachel, darin ein heller und viele dunkle Werte.
    const rgba = new Uint8ClampedArray(4 * 4 * 4);
    for (let i = 0; i < 16; i += 1) {
      rgba[i * 4] = rgba[i * 4 + 1] = rgba[i * 4 + 2] = 20;
      rgba[i * 4 + 3] = 255;
    }
    rgba[0] = rgba[1] = rgba[2] = 200;

    const g = backgroundGrid(rgba, 4, 4, 4);
    expect(g.gw).toBe(1);
    expect(g.werte[0]).toBeCloseTo(200, 0);
  });

  it('folgt dem Helligkeitsverlauf über die Kacheln', () => {
    const rgba = blattMitSchatten(200, 100);
    const g = backgroundGrid(rgba, 200, 100, 25);
    // Links hell, rechts dunkel – monoton fallend in der obersten Zeile.
    expect(g.werte[0]).toBeGreaterThan(g.werte[g.gw - 1]);
  });
});

describe('smoothGrid', () => {
  it('mittelt Ausreißer weg, die sonst als Karomuster durchschlagen', () => {
    const werte = new Float32Array([100, 100, 100, 100, 200, 100, 100, 100, 100]);
    const g = smoothGrid({ werte, gw: 3, gh: 3 });
    expect(g.werte[4]).toBeLessThan(200);
    expect(g.werte[4]).toBeGreaterThan(100);
  });

  it('lässt ein gleichmäßiges Raster unverändert', () => {
    const werte = new Float32Array(9).fill(150);
    expect([...smoothGrid({ werte, gw: 3, gh: 3 }).werte]).toEqual(new Array(9).fill(150));
  });
});

describe('sampleGrid', () => {
  const raster = { werte: new Float32Array([0, 100, 200, 300]), gw: 2, gh: 2 };

  it('trifft die Rastermitten exakt', () => {
    expect(sampleGrid(raster, 5, 5, 10)).toBeCloseTo(0, 5);
    expect(sampleGrid(raster, 15, 15, 10)).toBeCloseTo(300, 5);
  });

  it('blendet dazwischen über', () => {
    expect(sampleGrid(raster, 10, 5, 10)).toBeCloseTo(50, 5);
  });

  it('läuft am Rand nicht aus dem Raster', () => {
    expect(sampleGrid(raster, 0, 0, 10)).toBeCloseTo(0, 5);
    expect(sampleGrid(raster, 1000, 1000, 10)).toBeCloseTo(300, 5);
  });
});

describe('flatten', () => {
  const W = 300, H = 200;

  it('gleicht den Helligkeitsverlauf des Papiers aus', () => {
    const rgba = blattMitSchatten(W, H);
    // Papierzeile ohne Schrift (y % 20 >= 6).
    const linksVorher = lumaAt(rgba, W, 10, 10);
    const rechtsVorher = lumaAt(rgba, W, W - 10, 10);
    expect(linksVorher - rechtsVorher).toBeGreaterThan(100); // deutlicher Verlauf

    flatten(rgba, W, H);

    const links = lumaAt(rgba, W, 10, 10);
    const rechts = lumaAt(rgba, W, W - 10, 10);
    expect(Math.abs(links - rechts)).toBeLessThan(20);
  });

  it('macht das Papier hell, ohne es auf reines Weiß zu drücken', () => {
    const rgba = blattMitSchatten(W, H);
    flatten(rgba, W, H);
    const papier = lumaAt(rgba, W, W / 2, 10);
    expect(papier).toBeGreaterThan(200);
    expect(papier).toBeLessThanOrEqual(255);
  });

  // Der Kern: Die Schrift muss dunkel BLEIBEN. Würde der Hintergrund über den
  // Mittelwert geschätzt, zöge der Text die Schätzung nach unten und würde
  // beim Teilen selbst aufgehellt – das Dokument verblasste.
  it('lässt die Schrift dunkel und hebt den Kontrast', () => {
    // In der Bildmitte messen: Die Schriftbalken der Vorlage enden bei 90 %
    // der Breite, weiter rechts steht nur Papier.
    const rgba = blattMitSchatten(W, H);
    const schriftVorher = lumaAt(rgba, W, W / 2, 2);
    const papierVorher = lumaAt(rgba, W, W / 2, 10);

    flatten(rgba, W, H);

    const schrift = lumaAt(rgba, W, W / 2, 2);
    const papier = lumaAt(rgba, W, W / 2, 10);
    expect(schrift).toBeLessThan(100);
    expect(papier - schrift).toBeGreaterThan(papierVorher - schriftVorher);
  });

  it('erhält Farben – ein rotes Logo bleibt rot', () => {
    const rgba = new Uint8ClampedArray(W * H * 4);
    for (let i = 0; i < W * H; i += 1) {
      // Gleichmäßig dunkel beleuchtetes Rot (2:1:1).
      rgba[i * 4] = 120; rgba[i * 4 + 1] = 60; rgba[i * 4 + 2] = 60; rgba[i * 4 + 3] = 255;
    }
    flatten(rgba, W, H);
    const [r, g, b] = [rgba[0], rgba[1], rgba[2]];
    expect(r).toBeGreaterThan(120);          // aufgehellt
    expect(r / g).toBeCloseTo(2, 1);         // Verhältnis erhalten
    expect(g).toBeCloseTo(b, 0);
  });

  it('deckelt den Faktor, statt Rauschen in dunklen Ecken aufzublasen', () => {
    const rgba = new Uint8ClampedArray(W * H * 4);
    for (let i = 0; i < W * H; i += 1) {
      rgba[i * 4] = rgba[i * 4 + 1] = rgba[i * 4 + 2] = 10;
      rgba[i * 4 + 3] = 255;
    }
    flatten(rgba, W, H);
    // Ohne Deckel wäre 10 * (244/10) = 244; mit Deckel 3.5 höchstens 35.
    expect(rgba[0]).toBeLessThanOrEqual(36);
  });

  it('kommt mit einem völlig schwarzen Bild klar, statt zu werfen', () => {
    const rgba = new Uint8ClampedArray(40 * 40 * 4);
    for (let i = 0; i < 40 * 40; i += 1) rgba[i * 4 + 3] = 255;
    expect(() => flatten(rgba, 40, 40)).not.toThrow();
  });
});

// Hilfsbild: weisses Papier mit schwarzer „Schrift", optional abgedunkelt und
// optional mit einem Helligkeitsverlauf von links nach rechts.
function seiteMitSchleier(w, h, { grundhelligkeit = 255, verlauf = 0 } = {}) {
  const rgba = new Uint8ClampedArray(w * h * 4);
  for (let y = 0; y < h; y += 1) {
    for (let x = 0; x < w; x += 1) {
      const i = (y * w + x) * 4;
      const abfall = verlauf * (x / (w - 1));
      // Jede achte Zeile ist Schrift – sie darf die Messung nicht bestimmen.
      const wert = (y % 8 === 0) ? 20 : grundhelligkeit - abfall;
      rgba[i] = rgba[i + 1] = rgba[i + 2] = Math.max(0, wert);
      rgba[i + 3] = 255;
    }
  }
  return rgba;
}

describe('beurteileAusleuchtung', () => {
  it('laesst eine saubere Seite in Ruhe', () => {
    const u = beurteileAusleuchtung(seiteMitSchleier(400, 560), 400, 560);
    expect(u.weisspunkt).toBeGreaterThanOrEqual(250);
    expect(u.spanne).toBeLessThan(5);
    expect(u.lohnt).toBe(false);
  });

  it('erkennt einen flachen Grauschleier', () => {
    const u = beurteileAusleuchtung(seiteMitSchleier(400, 560, { grundhelligkeit: 205 }), 400, 560);
    expect(u.weisspunkt).toBeLessThan(215);
    // Flach heisst: gleichmaessig grau, keine Spanne.
    expect(u.spanne).toBeLessThan(5);
    expect(u.lohnt).toBe(true);
  });

  it('erkennt einen Schattenverlauf, auch wenn das Papier hell genug waere', () => {
    // Rechts 70 Stufen dunkler als links, links aber noch fast weiss – der
    // Weisspunkt allein wuerde das durchgehen lassen.
    const u = beurteileAusleuchtung(seiteMitSchleier(400, 560, { grundhelligkeit: 252, verlauf: 70 }), 400, 560);
    expect(u.weisspunkt).toBeGreaterThan(240);
    expect(u.spanne).toBeGreaterThan(20);
    expect(u.lohnt).toBe(true);
  });

  it('haelt eine bereits binarisierte Seite fuer sauber', () => {
    // Reines Schwarz-Weiss, wie es JBIG2 und CCITT liefern.
    const w = 400; const h = 560;
    const rgba = new Uint8ClampedArray(w * h * 4);
    for (let y = 0; y < h; y += 1) {
      for (let x = 0; x < w; x += 1) {
        const i = (y * w + x) * 4;
        const wert = (y % 8 === 0 && x % 3 !== 0) ? 0 : 255;
        rgba[i] = rgba[i + 1] = rgba[i + 2] = wert;
        rgba[i + 3] = 255;
      }
    }
    const u = beurteileAusleuchtung(rgba, w, h);
    expect(u.weisspunkt).toBe(255);
    expect(u.spanne).toBe(0);
    expect(u.lohnt).toBe(false);
  });
});
