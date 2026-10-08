import { describe, it, expect } from 'vitest';
import { detectPageQuad } from './detectPage';

/**
 * Baut ein Testbild: dunkler Untergrund, darauf ein helles Rechteck als
 * "Blatt", darin dunkle Balken als "Schrift".
 */
function blattAuf(width, height, { x0, y0, x1, y1, schrift = true, papier = 230, tisch = 60 }) {
  const gray = new Uint8Array(width * height).fill(tisch);
  for (let y = y0; y < y1; y += 1) {
    for (let x = x0; x < x1; x += 1) {
      const istSchrift = schrift && (y - y0) % 14 < 4 && x > x0 + 8 && x < x1 - 8;
      gray[y * width + x] = istSchrift ? 40 : papier;
    }
  }
  return gray;
}

describe('detectPageQuad', () => {
  const W = 400, H = 500;

  it('findet ein rechteckiges Blatt', () => {
    const q = detectPageQuad(blattAuf(W, H, { x0: 40, y0: 60, x1: 360, y1: 440 }), W, H);
    expect(q).not.toBeNull();
    const [ol, or_, ur, ul] = q;
    expect(ol.x).toBeCloseTo(40, -1);
    expect(ol.y).toBeCloseTo(60, -1);
    expect(ur.x).toBeCloseTo(359, -1);
    expect(ur.y).toBeCloseTo(439, -1);
    expect(or_.y).toBeCloseTo(60, -1);
    expect(ul.x).toBeCloseTo(40, -1);
  });

  // Der Text auf dem Papier zerlegt die helle Fläche nicht: Buchstaben sind
  // dunkle Inseln, das Papier fließt drumherum. Ginge das schief, fände die
  // Erkennung statt des Blattes eine einzelne Textzeile.
  it('lässt sich von Schrift auf dem Papier nicht zerteilen', () => {
    const mit = detectPageQuad(blattAuf(W, H, { x0: 40, y0: 60, x1: 360, y1: 440, schrift: true }), W, H);
    const ohne = detectPageQuad(blattAuf(W, H, { x0: 40, y0: 60, x1: 360, y1: 440, schrift: false }), W, H);
    expect(mit).toEqual(ohne);
  });

  it('gibt null zurück, wenn nichts Blattartiges da ist', () => {
    const gray = new Uint8Array(W * H).fill(50);
    expect(detectPageQuad(gray, W, H)).toBeNull();
  });

  it('lehnt eine zu kleine helle Fläche ab', () => {
    const q = detectPageQuad(blattAuf(W, H, { x0: 150, y0: 200, x1: 250, y1: 300, schrift: false }), W, H);
    expect(q).toBeNull();
  });

  it('verweigert winzige Bilder, statt zu raten', () => {
    expect(detectPageQuad(new Uint8Array(100), 10, 10)).toBeNull();
  });

  it('findet das Blatt auch, wenn es bis an den Bildrand läuft', () => {
    const q = detectPageQuad(blattAuf(W, H, { x0: 0, y0: 30, x1: W, y1: 470, schrift: false }), W, H);
    expect(q).not.toBeNull();
    expect(q[0].x).toBe(0);
    expect(q[1].x).toBe(W - 1);
  });

  // Ein zweites helles Objekt neben dem Blatt (Kabel, Zettel) darf die
  // Erkennung nicht kapern – gewinnen soll die GRÖSSTE Fläche.
  it('nimmt die größte helle Fläche, nicht die erstbeste', () => {
    const gray = blattAuf(W, H, { x0: 120, y0: 60, x1: 380, y1: 440, schrift: false });
    for (let y = 10; y < 50; y += 1) for (let x = 10; x < 60; x += 1) gray[y * W + x] = 240;

    const q = detectPageQuad(gray, W, H);
    expect(q).not.toBeNull();
    expect(q[0].x).toBeGreaterThan(100);
  });
});
