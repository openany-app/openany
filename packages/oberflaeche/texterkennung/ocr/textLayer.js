// Eine unsichtbare Textebene in ein BESTEHENDES PDF schreiben.
//
// Das ist der entscheidende Unterschied zum Foto-Weg. Dort baut jsPDF ein
// neues Dokument aus einem Bild. Hier waere derselbe Weg ein Verlust: Ein
// gescanntes PDF ist bereits ein fertiges Dokument, oft mit hoeherer
// Aufloesung und besserer Kompression, als wir sie mit einem neu gerenderten
// JPEG je erreichen. Es neu zu rastern, nur um Text darueberzulegen, hiesse
// die Vorlage zu verschlechtern, um sie zu verbessern.
//
// Also bleibt das Original Byte fuer Byte, und wir haengen an jede Seite einen
// zusaetzlichen Inhaltsstrom mit dem Text im PDF-Textrendermodus 3
// („invisible"): Er faerbt kein Pixel, laesst sich aber suchen, markieren,
// kopieren und vorlesen. Genau so arbeiten die etablierten OCR-Werkzeuge.
// Die Datei waechst dabei um die Groesse des Textes, also um wenige Kilobyte.

import { sanitizeWinAnsi, horizontalScaleFor } from './buildPdf';

/**
 * @param {ArrayBuffer|Uint8Array} pdfBytes  Das unveraenderte Original
 * @param {Array<{nummer: number, woerter: Array}>} seiten
 *        `woerter` sind Platzierungen in PDF-Punkten (pdfPages.platziereWoerter)
 * @returns {Promise<{blob: Blob, woerter: number}>}
 *          `woerter` = wie viele tatsaechlich geschrieben wurden. 0 heisst:
 *          es gab nichts zu erkennen – der Aufrufer soll dann nicht so tun,
 *          als waere das Dokument jetzt durchsuchbar.
 */
export async function schreibeTextebene(pdfBytes, seiten) {
  const {
    PDFDocument, StandardFonts, TextRenderingMode,
    pushGraphicsState, popGraphicsState,
    beginText, endText,
    setTextRenderingMode, setFontAndSize, setTextMatrix,
    setCharacterSqueeze, showText,
  } = await import('pdf-lib');

  const doc = await PDFDocument.load(pdfBytes, { updateMetadata: false });
  // Helvetica ist eine der 14 Standardschriften: Sie muss nicht eingebettet
  // werden und kostet die Datei damit nichts. Sichtbar wird sie ohnehin nie –
  // welche Glyphen sie zeichnen wuerde, ist gleichgueltig, solange die
  // Zeichencodes stimmen.
  const font = await doc.embedFont(StandardFonts.Helvetica);
  const seitenObjekte = doc.getPages();

  let geschrieben = 0;

  for (const { nummer, woerter } of seiten) {
    const seite = seitenObjekte[nummer - 1];
    if (!seite || !woerter?.length) continue;

    // Registriert die Schrift in den Ressourcen der Seite und liefert den
    // Namen, unter dem der Inhaltsstrom sie ansprechen kann (/Helvetica-…).
    seite.setFont(font);
    const [, fontName] = seite.getFont();

    const ops = [
      pushGraphicsState(),
      beginText(),
      setTextRenderingMode(TextRenderingMode.Invisible),
    ];

    for (const wort of woerter) {
      const text = sanitizeWinAnsi(wort.text);
      if (!text) continue;

      let kodiert;
      try {
        kodiert = font.encodeText(text);
      } catch {
        // WinAnsi kann ein Zeichen nicht: Ein Wort weniger in der Suche ist
        // besser als ein Dokument, das nicht entsteht.
        continue;
      }

      const natuerlich = font.widthOfTextAtSize(text, wort.size);

      ops.push(
        setFontAndSize(fontName, wort.size),
        // Tz statt Tc – die Begruendung steht in buildPdf.horizontalScaleFor:
        // Zeichenabstaende zerlegen kurze Woerter beim Kopieren in einzelne
        // Buchstaben, eine Glyphenstauchung tut das nicht. Tz zaehlt in
        // Prozent.
        setCharacterSqueeze(100 * horizontalScaleFor(wort.breite, natuerlich)),
        // Die Textmatrix traegt Ort UND Schreibrichtung. Der Richtungsvektor
        // kommt aus der Grundlinie des Wortes, deshalb sitzt der Text auch auf
        // gedrehten Seiten richtig herum.
        setTextMatrix(wort.ux, wort.uy, -wort.uy, wort.ux, wort.x, wort.y),
        showText(kodiert),
      );
      geschrieben += 1;
    }

    ops.push(endText(), popGraphicsState());
    seite.pushOperators(...ops);
  }

  const bytes = await doc.save();

  return { blob: new Blob([bytes], { type: 'application/pdf' }), woerter: geschrieben };
}
