import { describe, it, expect } from 'vitest';
import {
  toGray, histogram, otsuThreshold, estimateSkew,
  rotatedBounds, targetScale, displayScale, pdfPageSize,
  OCR_LONG_EDGE, DISPLAY_LONG_EDGE,
} from './imageMath';

/**
 * Malt ein Streifenmuster wie Textzeilen: dunkle Balken im Abstand `abstand`,
 * das Ganze um `grad` geschert. Das ist genau das Signal, auf das der
 * Schiefwinkel-Schätzer anspringen soll.
 *
 * Achtung beim Vorzeichen: hier ist `grad` die Schieflage des Blattes,
 * estimateSkew liefert den Winkel, der sie AUFHEBT – also das Gegenteil.
 */
function textzeilen(width, height, grad, { abstand = 30, dicke = 6 } = {}) {
  const gray = new Uint8Array(width * height).fill(240);
  const tan = Math.tan((grad * Math.PI) / 180);
  const mitteX = width / 2;
  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < width; x += 1) {
      // Rand freilassen, damit die Zeilen nicht am Bildrand abgeschnitten
      // unterschiedlich lang werden.
      if (x < width * 0.1 || x > width * 0.9) continue;
      const zeile = y + (x - mitteX) * tan;
      if (((zeile % abstand) + abstand) % abstand < dicke) gray[y * width + x] = 20;
    }
  }
  return gray;
}

describe('toGray', () => {
  it('gewichtet die Kanäle nach Luma statt sie zu mitteln', () => {
    // Reines Grün ist deutlich heller als reines Blau – ein schlichter
    // Mittelwert würde beide gleich behandeln.
    const rgba = new Uint8ClampedArray([0, 255, 0, 255, 0, 0, 255, 255]);
    const gray = toGray(rgba, 2, 1);
    expect(gray[0]).toBe(149);
    expect(gray[1]).toBe(29);
  });
});

describe('otsuThreshold', () => {
  // Der Schwellenwert gehört zur dunklen Klasse: dunkel ist `wert <= t`.
  // Deshalb wird hier die Trennung geprüft, nicht eine Zahl – bei zwei
  // scharfen Gipfeln sind alle Schnitte dazwischen gleich gut, und welchen
  // davon die Schleife zuerst findet, ist beliebig.
  it('trennt die zwei Gipfel eines Dokument-Histogramms', () => {
    // Schrift bei 30, Papier bei 220.
    const bins = new Uint32Array(256);
    bins[30] = 1000;
    bins[220] = 9000;
    const t = otsuThreshold(bins);
    expect(30).toBeLessThanOrEqual(t);
    expect(220).toBeGreaterThan(t);
  });

  it('trennt ein echtes Zwei-Gipfel-Bild in Schrift und Papier', () => {
    const t = otsuThreshold(histogram(textzeilen(200, 200, 0)));
    expect(20).toBeLessThanOrEqual(t);  // Schrift zählt als dunkel …
    expect(240).toBeGreaterThan(t);     // … Papier nicht.
  });

  it('fällt bei leerem Histogramm auf die Mitte zurück statt zu werfen', () => {
    expect(otsuThreshold(new Uint32Array(256))).toBe(127);
  });
});

describe('estimateSkew', () => {
  it('findet einen um 3° geneigten Zeilensatz und liefert die Gegendrehung', () => {
    const korrektur = estimateSkew(textzeilen(400, 400, 3), 400, 400);
    expect(korrektur).toBeLessThan(-2.5);
    expect(korrektur).toBeGreaterThan(-3.5);
  });

  it('findet auch die Gegenrichtung', () => {
    const korrektur = estimateSkew(textzeilen(400, 400, -2.5), 400, 400);
    expect(korrektur).toBeGreaterThan(2);
    expect(korrektur).toBeLessThan(3);
  });

  // Der Rückgabewert geht unverändert an ctx.rotate(). Dreht man das Bild
  // damit, muss der Rest-Schiefwinkel verschwinden – das ist der eigentliche
  // Vertrag, und er hält nur, wenn das Vorzeichen stimmt.
  it('macht das Blatt gerade, wenn man mit dem Ergebnis zurückdreht', () => {
    const korrektur = estimateSkew(textzeilen(400, 400, 4), 400, 400);
    const geradegerueckt = textzeilen(400, 400, 4 + korrektur);
    expect(estimateSkew(geradegerueckt, 400, 400)).toBe(0);
  });

  it('lässt ein gerades Blatt in Ruhe (kein Drehen um Nullkommanichts)', () => {
    expect(estimateSkew(textzeilen(400, 400, 0), 400, 400)).toBe(0);
  });

  it('gibt bei einem leeren Bild 0 zurück statt zu raten', () => {
    expect(estimateSkew(new Uint8Array(400 * 400).fill(255), 400, 400)).toBe(0);
  });

  it('gibt bei einem durchgehend dunklen Bild 0 zurück', () => {
    expect(estimateSkew(new Uint8Array(400 * 400).fill(10), 400, 400)).toBe(0);
  });

  it('verweigert winzige Bilder, statt Unsinn zu melden', () => {
    expect(estimateSkew(new Uint8Array(100), 10, 10)).toBe(0);
  });
});

describe('rotatedBounds', () => {
  it('lässt ein ungedrehtes Rechteck unverändert', () => {
    expect(rotatedBounds(100, 50, 0)).toEqual({ width: 100, height: 50 });
  });

  it('wächst in beide Richtungen, wenn gedreht wird', () => {
    const b = rotatedBounds(100, 50, 45);
    expect(b.width).toBeGreaterThan(100);
    expect(b.height).toBeGreaterThan(50);
  });
});

describe('targetScale', () => {
  it('rechnet ein kleines Foto auf ~300 dpi hoch', () => {
    const s = targetScale(1000, 750);
    expect(s).toBeCloseTo(OCR_LONG_EDGE / 1000, 5);
  });

  it('verkleinert ein großes Foto nicht – Auflösung ist der Hebel', () => {
    expect(targetScale(4000, 3000)).toBe(1);
  });

  it('deckelt die Vergrößerung, statt ein Briefmarkenbild aufzublasen', () => {
    expect(targetScale(100, 100)).toBe(4);
  });

  it('bleibt unter der Canvas-Flächengrenze, auch wenn das Verkleinern bedeutet', () => {
    const s = targetScale(9000, 7000);
    expect(9000 * s * 7000 * s).toBeLessThanOrEqual(16_000_000 + 1);
    expect(s).toBeLessThan(1);
  });

  it('rechnet die Drehung in die Fläche mit ein', () => {
    const gerade = targetScale(5000, 3000, 0);
    const schief = targetScale(5000, 3000, 45);
    expect(schief).toBeLessThan(gerade);
  });
});

// Die sichtbare Ebene geht den umgekehrten Weg wie die Erkennung: Sie will so
// wenige Pixel wie moeglich, damit das PDF nicht aufblaeht.
describe('displayScale', () => {
  it('verkleinert ein großes Foto auf die Anzeige-Auflösung', () => {
    expect(displayScale(4000, 3000)).toBeCloseTo(DISPLAY_LONG_EDGE / 4000, 5);
  });

  it('bläst ein kleines Foto NICHT auf – das kostet Bytes ohne ein Detail zu gewinnen', () => {
    expect(displayScale(1000, 750)).toBe(1);
    expect(displayScale(100, 100)).toBe(1);
  });

  it('bleibt unter der Canvas-Flächengrenze', () => {
    const s = displayScale(30000, 20000);
    expect(30000 * s * 20000 * s).toBeLessThanOrEqual(16_000_000 + 1);
  });

  // Der eigentliche Zweck der Trennung: Die Erkennung bekommt mehr Pixel als
  // die Anzeige. Kippt das jemals um, waere die Trennung sinnlos geworden.
  it('liefert für ein großes Foto weniger Pixel als die Erkennung', () => {
    expect(displayScale(4000, 3000)).toBeLessThan(targetScale(4000, 3000));
    expect(displayScale(1200, 900)).toBeLessThan(targetScale(1200, 900));
  });
});

describe('pdfPageSize', () => {
  it('legt die lange Kante quer auf A4-Länge', () => {
    const s = pdfPageSize(2000, 1000);
    expect(s.width).toBe(842);
    expect(s.height).toBe(421);
  });

  it('legt die lange Kante hochkant auf A4-Länge', () => {
    const s = pdfPageSize(1000, 2000);
    expect(s.height).toBe(842);
    expect(s.width).toBe(421);
  });

  it('behält das Seitenverhältnis – kein weißer Rand im PDF', () => {
    const s = pdfPageSize(1600, 1200);
    expect(s.width / s.height).toBeCloseTo(4 / 3, 6);
  });
});
