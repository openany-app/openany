import { describe, it, expect } from 'vitest';
import { schreibeTextebene } from './textLayer';

async function leerseite(breite, hoehe, rotate = 0) {
  const { PDFDocument, degrees, rgb } = await import('pdf-lib');
  const doc = await PDFDocument.create();
  const seite = doc.addPage([breite, hoehe]);
  if (rotate) seite.setRotation(degrees(rotate));
  // Etwas sichtbaren Inhalt, damit die Seite nicht leer ist.
  seite.drawRectangle({ x: 10, y: 10, width: 50, height: 20, color: rgb(0.9, 0.9, 0.9) });
  return new Uint8Array(await doc.save());
}

async function leseText(bytes) {
  const pdfjs = await import('pdfjs-dist/legacy/build/pdf.mjs');
  const pdf = await pdfjs.getDocument({ data: bytes, useSystemFonts: false }).promise;
  const seite = await pdf.getPage(1);
  const inhalt = await seite.getTextContent();
  return inhalt.items.map((i) => ({ str: i.str, transform: i.transform, width: i.width }));
}

describe('schreibeTextebene', () => {
  it('legt Text an die angegebene Stelle und laesst ihn wieder auslesen', async () => {
    const bytes = await leerseite(595, 842);
    const { blob, woerter } = await schreibeTextebene(bytes, [{
      nummer: 1,
      woerter: [
        { text: 'Rechnung', x: 100, y: 700, ux: 1, uy: 0, breite: 60, size: 12 },
        { text: 'Zahnarzt', x: 170, y: 700, ux: 1, uy: 0, breite: 55, size: 12 },
      ],
    }]);

    expect(woerter).toBe(2);

    const items = await leseText(new Uint8Array(await blob.arrayBuffer()));
    const text = items.map((i) => i.str).join(' ');
    expect(text).toContain('Rechnung');
    expect(text).toContain('Zahnarzt');

    // Ort: die Textmatrix traegt x/y an Position 4 und 5.
    const ersteres = items.find((i) => i.str.includes('Rechnung'));
    expect(ersteres.transform[4]).toBeCloseTo(100, 1);
    expect(ersteres.transform[5]).toBeCloseTo(700, 1);

    // Breite: Tz muss das Wort auf den Kasten gezogen haben.
    expect(ersteres.width).toBeCloseTo(60, 0);
  });

  it('schreibt nichts, wenn es nichts zu schreiben gibt', async () => {
    const bytes = await leerseite(595, 842);
    const { woerter } = await schreibeTextebene(bytes, [{ nummer: 1, woerter: [] }]);
    expect(woerter).toBe(0);
  });

  it('ueberspringt Zeichen, die Helvetica nicht kodieren kann', async () => {
    const bytes = await leerseite(595, 842);
    const { woerter } = await schreibeTextebene(bytes, [{
      nummer: 1,
      woerter: [
        { text: '漢字', x: 50, y: 50, ux: 1, uy: 0, breite: 20, size: 10 },
        { text: 'Grüße', x: 50, y: 80, ux: 1, uy: 0, breite: 30, size: 10 },
      ],
    }]);
    // Nur das deutsche Wort ueberlebt – und das Dokument entsteht trotzdem.
    expect(woerter).toBe(1);
  });

  it('setzt Text auf einer um 90 Grad gedrehten Seite in Schreibrichtung', async () => {
    const bytes = await leerseite(595, 842, 90);
    // Auf einer /Rotate-90-Seite laeuft die sichtbare Zeile im Nutzerraum
    // senkrecht: Richtungsvektor (0, 1) statt (1, 0).
    const { blob } = await schreibeTextebene(bytes, [{
      nummer: 1,
      woerter: [{ text: 'Quer', x: 100, y: 100, ux: 0, uy: 1, breite: 40, size: 12 }],
    }]);
    const items = await leseText(new Uint8Array(await blob.arrayBuffer()));
    const eintrag = items.find((i) => i.str.includes('Quer'));
    expect(eintrag).toBeTruthy();
    // Die Drehung ist angekommen: keine waagerechte Komponente, dafuer eine
    // senkrechte. Der Betrag ist NICHT die Schriftgroesse – pdf.js rechnet die
    // Tz-Stauchung mit in die Matrix hinein, hier also 12 pt mal dem Faktor,
    // der „Quer" auf 40 pt zieht.
    expect(eintrag.transform[0]).toBeCloseTo(0, 5);
    expect(eintrag.transform[1]).toBeGreaterThan(0);
    expect(eintrag.width).toBeCloseTo(40, 0);
  });
});
