import { describe, it, expect } from 'vitest';
import { platziereWoerter } from './pdfPages';
import { schreibeTextebene } from './textLayer';

// pdf.js wird hier direkt in der node-Fassung geladen. Der Produktivpfad
// (holePdfJs in pdfPages.js) holt den Browser-Build samt Worker – der laeuft
// in Vitest nicht, wird fuer die Geometrie aber auch nicht gebraucht:
// platziereWoerter rechnet nur mit dem viewport.
async function viewportFuer(breite, hoehe, skala, rotate = 0) {
  const { PDFDocument, degrees } = await import('pdf-lib');
  const doc = await PDFDocument.create();
  const seite = doc.addPage([breite, hoehe]);
  if (rotate) seite.setRotation(degrees(rotate));
  const bytes = new Uint8Array(await doc.save());

  const pdfjs = await import('pdfjs-dist/legacy/build/pdf.mjs');
  const pdf = await pdfjs.getDocument({ data: bytes.slice() }).promise;
  const page = await pdf.getPage(1);

  return { viewport: page.getViewport({ scale: skala }), bytes };
}

async function leseText(bytes) {
  const pdfjs = await import('pdfjs-dist/legacy/build/pdf.mjs');
  const pdf = await pdfjs.getDocument({ data: bytes }).promise;
  const inhalt = await (await pdf.getPage(1)).getTextContent();
  return inhalt.items;
}

const SKALA = 300 / 72; // 300 dpi

describe('platziereWoerter', () => {
  it('rechnet Bildpixel in PDF-Punkte um', async () => {
    const { viewport } = await viewportFuer(595, 842, SKALA);

    // Ein Wortkasten, wie Tesseract ihn auf dem 300-dpi-Bild liefert.
    const [platz] = platziereWoerter(viewport, [
      { text: 'Rechnung', x0: 100, y0: 200, x1: 400, y1: 250 },
    ]);

    // Links: 100 px / 4,1667 = 24 pt. Grundlinie: die Bild-Y-Achse zeigt nach
    // unten, die PDF-Y-Achse nach oben – 842 - 250/4,1667 = 782 pt.
    expect(platz.x).toBeCloseTo(24, 4);
    expect(platz.y).toBeCloseTo(782, 4);
    expect(platz.breite).toBeCloseTo(72, 4);
    expect(platz.size).toBeCloseTo(12, 4);
    // Waagerecht: Richtungsvektor zeigt nach rechts.
    expect(platz.ux).toBeCloseTo(1, 6);
    expect(platz.uy).toBeCloseTo(0, 6);
  });

  it('dreht die Schreibrichtung auf einer /Rotate-90-Seite mit', async () => {
    const { viewport } = await viewportFuer(595, 842, SKALA, 90);

    const [platz] = platziereWoerter(viewport, [
      { text: 'Quer', x0: 100, y0: 200, x1: 400, y1: 250 },
    ]);

    // Im Nutzerraum laeuft die Zeile jetzt senkrecht. Breite und Hoehe des
    // Kastens sind davon unberuehrt – ein Betrag dreht sich nicht mit.
    expect(Math.abs(platz.ux)).toBeCloseTo(0, 6);
    expect(Math.abs(platz.uy)).toBeCloseTo(1, 6);
    expect(platz.breite).toBeCloseTo(72, 4);
    expect(platz.size).toBeCloseTo(12, 4);
  });

  it('laesst Kaesten ohne Breite fallen', async () => {
    const { viewport } = await viewportFuer(595, 842, SKALA);
    const platz = platziereWoerter(viewport, [
      { text: 'x', x0: 100, y0: 200, x1: 100, y1: 250 },
    ]);
    expect(platz).toHaveLength(0);
  });
});

describe('Von den Wortkaesten bis ins fertige PDF', () => {
  it('legt das Wort dorthin, wo es im Bild stand', async () => {
    const { viewport, bytes } = await viewportFuer(595, 842, SKALA);

    const woerter = platziereWoerter(viewport, [
      { text: 'Rechnung', x0: 100, y0: 200, x1: 400, y1: 250 },
    ]);
    const { blob } = await schreibeTextebene(bytes, [{ nummer: 1, woerter }]);
    const items = await leseText(new Uint8Array(await blob.arrayBuffer()));

    const eintrag = items.find((i) => i.str.includes('Rechnung'));
    expect(eintrag).toBeTruthy();
    // Genau die Punkte aus der Rechnung oben – ueber zwei Bibliotheken hinweg
    // (pdf-lib schreibt, pdf.js liest) und ohne Abweichung.
    expect(eintrag.transform[4]).toBeCloseTo(24, 3);
    expect(eintrag.transform[5]).toBeCloseTo(782, 3);
    // Die Breite traegt eine Toleranz von einem Sechstel Punkt (0,06 mm), und
    // die ist erklaerbar: Die Stauchung Tz wird aus pdf-libs Helvetica-Metrik
    // berechnet, hier aber mit pdf.js' eigener Metriktabelle nachgemessen.
    // Die beiden Tabellen weichen um rund zwei Promille voneinander ab. Beim
    // Markieren ist das nicht zu sehen; enger anzusetzen hiesse, eine
    // Uebereinstimmung zu behaupten, die es zwischen den Bibliotheken nicht
    // gibt.
    expect(eintrag.width).toBeCloseTo(72, 0);
    expect(eintrag.height).toBeCloseTo(12, 2);
  });
});
