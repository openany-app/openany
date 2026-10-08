import { describe, it, expect } from 'vitest';
import { sanitizeWinAnsi, wordPlacement, horizontalScaleFor, buildPdf } from './buildPdf';

/**
 * Canvas-Attrappe: buildPdf braucht von einem Canvas nur Maße und ein Bild
 * als Datenzeile. Ein 1x1-JPEG reicht – geprüft wird die Struktur des PDFs,
 * nicht die Bildqualität.
 */
const JPEG_1PX = 'data:image/jpeg;base64,/9j/4AAQSkZJRgABAQEAYABgAAD/2wBDAAgGBgcGBQgHBwcJCQgKDBQNDAsLDBkSEw8UHRofHh0aHBwgJC4nICIsIxwcKDcpLDAxNDQ0Hyc5PTgyPC4zNDL/wAALCAABAAEBAREA/8QAFAABAAAAAAAAAAAAAAAAAAAACf/EABQQAQAAAAAAAAAAAAAAAAAAAAD/2gAIAQEAAD8AKp//2Q==';

function canvasAttrappe(width, height) {
  return { width, height, toDataURL: () => JPEG_1PX };
}

/**
 * Die Textbefehle aus dem PDF: Setzen der Schriftgröße (Tf), Positionieren
 * (Td/Tm) und die horizontale Streckung (Tz). Genau das, woran sich ablesen
 * lässt, WO die unsichtbare Textebene liegt – ohne die Datei ganz zu
 * vergleichen, in der auch Zeitstempel und Datei-Kennungen stecken.
 */
function textOperatoren(buffer) {
  const inhalt = new TextDecoder('latin1').decode(buffer);
  return inhalt.match(/[-\d.]+ [-\d.]+ Td|[\d.]+ Tz|\/F\d+ [\d.]+ Tf/g) ?? [];
}

describe('sanitizeWinAnsi', () => {
  it('lässt Umlaute und ß in Ruhe – das ist der halbe deutsche Wortschatz', () => {
    expect(sanitizeWinAnsi('Größenmaß Ärger Übung')).toBe('Größenmaß Ärger Übung');
  });

  it('holt typografische Zeichen auf ihre schlichten Entsprechungen zurück', () => {
    expect(sanitizeWinAnsi('„Rechnung“ – Pos. 3 …')).toBe('"Rechnung" - Pos. 3 ...');
    expect(sanitizeWinAnsi('Müller’s')).toBe("Müller's");
  });

  it('wirft nicht kodierbare Zeichen weg, statt das ganze PDF zu verlieren', () => {
    expect(sanitizeWinAnsi('Rechnung 東京 ok')).toBe('Rechnung  ok');
  });

  it('kommt mit leerem Text klar', () => {
    expect(sanitizeWinAnsi('')).toBe('');
  });
});

describe('wordPlacement', () => {
  // Arbeitsbild 2000 px breit, Seite 800 pt → Faktor 0,4.
  const wort = { text: 'Rechnung', x0: 100, y0: 200, x1: 300, y1: 240 };

  it('rechnet Canvas-Pixel in PDF-Punkte um', () => {
    const p = wordPlacement(wort, 0.4);
    expect(p.x).toBeCloseTo(40, 6);
    expect(p.breite).toBeCloseTo(80, 6);
  });

  it('setzt die Grundlinie auf die Unterkante des Kastens', () => {
    expect(wordPlacement(wort, 0.4).y).toBeCloseTo(96, 6);
  });

  it('leitet die Schriftgröße aus der Kastenhöhe ab', () => {
    expect(wordPlacement(wort, 0.4).size).toBeCloseTo(16, 6);
  });

  it('lässt die Schriftgröße nie auf 0 fallen', () => {
    expect(wordPlacement({ ...wort, y1: 200.0001 }, 0.0001).size).toBeGreaterThan(0);
  });

  it('nimmt für die Höhe den y-Faktor, wenn die Achsen auseinanderliegen', () => {
    const p = wordPlacement(wort, 0.4, 0.5);
    expect(p.x).toBeCloseTo(40, 6);      // x über fx
    expect(p.breite).toBeCloseTo(80, 6); // Breite über fx
    expect(p.y).toBeCloseTo(120, 6);     // Grundlinie über fy
    expect(p.size).toBeCloseTo(20, 6);   // Schriftgröße über fy
  });

  it('nimmt ohne zweiten Faktor den ersten für beide Achsen', () => {
    expect(wordPlacement(wort, 0.4)).toEqual(wordPlacement(wort, 0.4, 0.4));
  });
});

describe('horizontalScaleFor', () => {
  it('zieht zu schmalen Text auf die Kastenbreite auf', () => {
    expect(horizontalScaleFor(60, 50)).toBeCloseTo(1.2, 6);
  });

  it('staucht zu breiten Text', () => {
    expect(horizontalScaleFor(40, 50)).toBeCloseTo(0.8, 6);
  });

  it('lässt passenden Text unangetastet', () => {
    expect(horizontalScaleFor(60, 60)).toBe(1);
  });

  it('fällt bei unbrauchbaren Maßen auf 1 zurück statt zu verzerren', () => {
    expect(horizontalScaleFor(60, 0)).toBe(1);
    expect(horizontalScaleFor(60, NaN)).toBe(1);
    expect(horizontalScaleFor(NaN, 50)).toBe(1);
    expect(horizontalScaleFor(60, -5)).toBe(1);
  });

  it('deckelt absurde Kästen aus Erkennungsfehlern', () => {
    expect(horizontalScaleFor(10000, 5)).toBe(10);
    expect(horizontalScaleFor(1, 5000)).toBe(0.1);
  });
});

describe('buildPdf', () => {
  const woerter = [{ text: 'Rechnung', x0: 100, y0: 200, x1: 300, y1: 240 }];

  it('erzeugt ein PDF', async () => {
    const blob = await buildPdf([{ canvas: canvasAttrappe(2000, 1500), words: woerter }]);
    const kopf = new TextDecoder().decode((await blob.arrayBuffer()).slice(0, 5));
    expect(kopf).toBe('%PDF-');
  });

  it('kommt ohne Erkennung aus – dann eben ohne Textebene', async () => {
    const ohne = await buildPdf([{ canvas: canvasAttrappe(2000, 1500), words: null }]);
    const mit = await buildPdf([{ canvas: canvasAttrappe(2000, 1500), words: woerter }]);
    expect(ohne.size).toBeGreaterThan(0);
    // Die Textebene muss sich in der Datei niederschlagen, sonst wäre sie
    // stillschweigend gar nicht drin.
    expect(mit.size).toBeGreaterThan(ohne.size);
  });

  it('macht aus mehreren Fotos ein mehrseitiges Dokument', async () => {
    const eine = await buildPdf([{ canvas: canvasAttrappe(1500, 2000), words: null }]);
    const drei = await buildPdf([
      { canvas: canvasAttrappe(1500, 2000), words: null },
      { canvas: canvasAttrappe(1500, 2000), words: null },
      { canvas: canvasAttrappe(1500, 2000), words: null },
    ]);
    expect(drei.size).toBeGreaterThan(eine.size);
  });

  // Der gemeldete Fehler: Die Textebene lag zwar richtig, liess sich aber nicht
  // sauber kopieren - kurze Woerter kamen als Einzelbuchstaben heraus
  // ("V e r s a n d"). Ursache war das Strecken ueber den Zeichenabstand (Tc),
  // dessen Luecken jeder Textextraktor als Leerzeichen liest. Tz streckt die
  // Glyphen selbst. Der Test haelt fest, dass Tc nicht zurueckkehrt.
  it('streckt über Tz und nicht über Zeichenabstände (sonst zerfallen Wörter beim Kopieren)', async () => {
    const blob = await buildPdf([{ canvas: canvasAttrappe(2000, 1500), words: woerter }]);
    const inhalt = new TextDecoder('latin1').decode(await blob.arrayBuffer());

    expect(inhalt).toMatch(/[\d.]+ Tz/);
    expect(inhalt).not.toMatch(/[\d.]+ Tc/);
  });

  // Sichtbare Ebene und Arbeitsbild haben ABSICHTLICH verschiedene Auflösungen
  // (~200 vs. ~300 dpi), damit die Datei klein bleibt. Die Wortkästen zählen
  // aber in Pixeln des Arbeitsbildes. Wer hier die Maße des sichtbaren Bildes
  // nimmt, legt die ganze Textebene um den Faktor der Differenz daneben – und
  // zwar unsichtbar, weil das Bild ja richtig aussieht.
  describe('getrennte Auflösungen', () => {
    // Dasselbe Wort, einmal in einem 1000er und einmal in einem 2000er
    // Arbeitsbild beschrieben. Beide zeigen auf dieselbe Stelle der Seite,
    // also muss dasselbe PDF herauskommen.
    it('bezieht die Wortkästen auf das Arbeitsbild, nicht auf das sichtbare Bild', async () => {
      const klein = await buildPdf([{
        canvas: canvasAttrappe(500, 700),
        words: [{ text: 'Rechnung', x0: 100, y0: 200, x1: 300, y1: 240 }],
        wordsWidth: 1000, wordsHeight: 1400,
      }]);
      const gross = await buildPdf([{
        canvas: canvasAttrappe(500, 700),
        words: [{ text: 'Rechnung', x0: 200, y0: 400, x1: 600, y1: 480 }],
        wordsWidth: 2000, wordsHeight: 2800,
      }]);

      expect(textOperatoren(await gross.arrayBuffer()))
        .toEqual(textOperatoren(await klein.arrayBuffer()));
    });

    it('fällt ohne Angabe auf die Maße des sichtbaren Bildes zurück', async () => {
      const ohne = await buildPdf([{
        canvas: canvasAttrappe(1000, 1400),
        words: [{ text: 'Rechnung', x0: 100, y0: 200, x1: 300, y1: 240 }],
      }]);
      const mit = await buildPdf([{
        canvas: canvasAttrappe(1000, 1400),
        words: [{ text: 'Rechnung', x0: 100, y0: 200, x1: 300, y1: 240 }],
        wordsWidth: 1000, wordsHeight: 1400,
      }]);

      expect(textOperatoren(await ohne.arrayBuffer()))
        .toEqual(textOperatoren(await mit.arrayBuffer()));
    });

    it('würde eine falsche Zuordnung bemerken', async () => {
      const richtig = await buildPdf([{
        canvas: canvasAttrappe(500, 700),
        words: [{ text: 'Rechnung', x0: 100, y0: 200, x1: 300, y1: 240 }],
        wordsWidth: 1000, wordsHeight: 1400,
      }]);
      const falsch = await buildPdf([{
        canvas: canvasAttrappe(500, 700),
        words: [{ text: 'Rechnung', x0: 100, y0: 200, x1: 300, y1: 240 }],
        wordsWidth: 500, wordsHeight: 700,
      }]);

      expect(textOperatoren(await falsch.arrayBuffer()))
        .not.toEqual(textOperatoren(await richtig.arrayBuffer()));
    });
  });

  it('verweigert eine leere Seitenliste, statt ein leeres PDF abzulegen', async () => {
    await expect(buildPdf([])).rejects.toThrow();
  });

  it('überspringt Wörter, von denen nach der Bereinigung nichts übrig bleibt', async () => {
    const nurCjk = await buildPdf([{ canvas: canvasAttrappe(2000, 1500), words: [{ text: '東京', x0: 1, y0: 1, x1: 9, y1: 9 }] }]);
    expect(nurCjk.size).toBeGreaterThan(0);
  });
});

describe('buildPdf mit vorgegebenem Seitenformat', () => {
  // Gebraucht auf dem PDF-Weg: Wird ein vorhandenes Dokument ersetzt, muss es
  // hinterher dieselben Masse haben wie vorher. Ohne Vorgabe leitet buildPdf
  // die Groesse aus dem Seitenverhaeltnis ab und legt die lange Kante auf A4 –
  // aus einem Letter-Brief (792 pt) wuerde so ein 842 pt langer.
  async function seitenmasse(blob) {
    const pdfjs = await import('pdfjs-dist/legacy/build/pdf.mjs');
    const pdf = await pdfjs.getDocument({ data: new Uint8Array(await blob.arrayBuffer()) }).promise;
    const [, , breite, hoehe] = (await pdf.getPage(1)).view;
    return { breite, hoehe };
  }

  it('uebernimmt die Vorgabe unveraendert', async () => {
    const letter = { width: 612, height: 792 };
    const blob = await buildPdf([{ canvas: canvasAttrappe(1700, 2338), words: null, format: letter }]);

    const { breite, hoehe } = await seitenmasse(blob);
    expect(breite).toBeCloseTo(612, 0);
    expect(hoehe).toBeCloseTo(792, 0);
  });

  it('leitet ohne Vorgabe weiterhin aus dem Seitenverhaeltnis ab', async () => {
    const blob = await buildPdf([{ canvas: canvasAttrappe(1700, 2338), words: null }]);

    const { hoehe } = await seitenmasse(blob);
    // Lange Kante auf A4-Laenge – das bisherige Verhalten, unveraendert.
    expect(hoehe).toBeCloseTo(842, 0);
  });
});
