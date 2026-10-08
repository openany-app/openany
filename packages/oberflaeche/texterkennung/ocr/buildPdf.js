// Baut aus vorbereiteten Seiten ein PDF mit unsichtbarer Textebene.
//
// Der Aufbau je Seite ist zweischichtig:
//
//   unten  das farbige Foto des Nutzers – Logos, Stempel, Briefkopf, alles
//          so, wie es aufgenommen wurde (nur geradegerueckt)
//   oben   jedes erkannte Wort an seinem Platz, im Textrendermodus 3
//          („invisible"): es faerbt kein Pixel, laesst sich aber suchen,
//          markieren, kopieren und vorlesen
//
// Genau das kann der bisherige serverseitige Weg nicht: dompdf hat keinen
// Zugriff auf den Textrendermodus. Deshalb entsteht das PDF hier im Browser,
// dort, wo auch die Wortkoordinaten anfallen.

import { pdfPageSize } from './imageMath';
import { toJpegDataUrl } from './prepareImage';

/**
 * Der Standardfont Helvetica wird in WinAnsi kodiert. Umlaute und ß sind
 * darin enthalten, typografische Feinheiten nicht – und jsPDF wirft bei
 * Zeichen, die es nicht kodieren kann. Weil Tesseract genau solche Zeichen
 * gern erkennt (Gedankenstriche, Anfuehrungszeichen), werden sie hier auf ihre
 * schlichten Entsprechungen zurueckgeholt.
 */
const ERSATZ = {
  '‘': "'", '’': "'", '‚': "'", '‛': "'",
  '“': '"', '”': '"', '„': '"', '‟': '"',
  '–': '-', '—': '-', '−': '-', '‐': '-', '‑': '-',
  '…': '...', ' ': ' ', ' ': ' ', ' ': ' ',
  '˝': '"', '´': "'", '`': "'",
};

/** Text auf das reduzieren, was Helvetica/WinAnsi tragen kann. */
export function sanitizeWinAnsi(text) {
  let out = '';
  for (const zeichen of String(text)) {
    if (ERSATZ[zeichen] !== undefined) {
      out += ERSATZ[zeichen];
    } else if (zeichen.codePointAt(0) <= 0xff) {
      out += zeichen;
    }
    // Alles darueber (CJK, Emoji, Sonderzeichen) faellt weg. Ein Wort weniger
    // in der Suche ist besser als ein PDF, das gar nicht erst entsteht.
  }
  return out;
}

/**
 * Ort und Groesse eines Wortes in PDF-Punkten.
 *
 * `fx`/`fy` rechnen Pixel des ARBEITSBILDES in Punkte um – nicht Pixel des
 * sichtbaren Bildes. Die beiden sind verschieden gross (das sichtbare ist
 * kleiner, damit die Datei nicht aufblaeht), decken aber denselben Ausschnitt.
 * Deshalb genuegt ein Faktor je Achse; eine Ruecktransformation der Drehung
 * braucht es nach wie vor nicht.
 *
 * Zwei Faktoren statt einem, weil beide Leinwaende auf ganze Pixel gerundet
 * werden und ihre Seitenverhaeltnisse dadurch um Haaresbreite auseinander
 * liegen koennen.
 *
 * Die Grundlinie liegt auf der Unterkante des Kastens. Das ist fuer Woerter
 * mit Unterlaengen ein bis zwei Punkt zu tief, faellt aber bei unsichtbarem
 * Text nicht auf – waehrend ein Fehler in der Breite sofort auffaellt, weil
 * die Markierung dann neben dem Wort steht.
 */
export function wordPlacement(word, fx, fy = fx) {
  return {
    x: word.x0 * fx,
    y: word.y1 * fy,
    size: Math.max(1, (word.y1 - word.y0) * fy),
    breite: (word.x1 - word.x0) * fx,
  };
}

/**
 * Faktor, der ein Wort genau auf die Breite seines Kastens zieht – als
 * horizontale Stauchung/Streckung der Glyphen selbst (PDF-Operator Tz).
 *
 * Warum nicht ueber den Zeichenabstand (Tc), was naeher liegt: Tc schiebt
 * LUECKEN zwischen die Buchstaben. Ein Textextraktor – jeder PDF-Betrachter,
 * jede Vorlesehilfe – setzt ab einer gewissen Luecke eine Leerstelle. Und weil
 * sich die Differenz zur Eigenbreite bei einem kurzen Wort auf wenige Zeichen
 * verteilt, wird sie dort am groessten. Ergebnis beim Kopieren:
 *
 *   "Zwischen Druckdatum und V e r s a n d koennen a u s organisatorischen"
 *
 * Genau so ist es aufgefallen: lange Woerter heil, kurze in Einzelbuchstaben
 * zerfallen. Tz veraendert die Glyphenbreite statt der Abstaende – die
 * Textebene deckt weiterhin denselben Kasten, aber es entstehen keine Luecken,
 * die als Leerzeichen durchgehen.
 *
 * Geprueft mit pdftotext gegen echte Tesseract-Kaesten; Tc reproduziert den
 * Fehler, Tz liefert den Absatz fehlerfrei.
 */
export function horizontalScaleFor(zielBreite, natuerlicheBreite) {
  if (!Number.isFinite(zielBreite) || !Number.isFinite(natuerlicheBreite) || natuerlicheBreite <= 0) {
    return 1;
  }

  // Ein voellig unpassender Kasten (Erkennungsfehler, Bildrauschen) soll die
  // Glyphen nicht ins Absurde ziehen. Unsichtbar ist das zwar, beim Markieren
  // aber nicht.
  return Math.min(10, Math.max(0.1, zielBreite / natuerlicheBreite));
}

/**
 * @param {Array<{
 *   canvas: HTMLCanvasElement,
 *   words: Array|null,
 *   wordsWidth?: number,
 *   wordsHeight?: number,
 *   format?: {width: number, height: number},
 * }>} seiten
 *        `words: null` heisst: Erkennung ist ausgefallen, Seite bekommt nur
 *        das Bild. Das PDF entsteht trotzdem.
 *
 *        `wordsWidth`/`wordsHeight` sind die Masse des Bildes, auf das sich
 *        die Wortkaesten beziehen – das Arbeitsbild der Erkennung, das groesser
 *        ist als `canvas`. Fehlen sie, wird `canvas` selbst angenommen.
 *
 *        `format` ist die Seitengroesse in Punkten. Fehlt sie, wird sie aus dem
 *        Seitenverhaeltnis abgeleitet (lange Kante A4). Das stimmt fuer ein
 *        Foto, dessen wahre Groesse niemand kennt – nicht aber fuer eine Seite
 *        aus einem PDF, die ihre eigene, oft abweichende Groesse mitbringt
 *        (Letter ist 792 pt lang, nicht 842). Wird ein solches Dokument
 *        ersetzt, muss es hinterher dieselben Masse haben wie vorher.
 * @returns {Promise<Blob>}
 */
export async function buildPdf(seiten) {
  if (!seiten.length) throw new Error('Kein Bild für das PDF.');

  const { jsPDF } = await import('jspdf');
  let doc = null;

  for (const { canvas, words, wordsWidth, wordsHeight, format: vorgabe } of seiten) {
    const format = vorgabe ?? pdfPageSize(canvas.width, canvas.height);
    const ausrichtung = format.width >= format.height ? 'landscape' : 'portrait';

    if (!doc) {
      doc = new jsPDF({ unit: 'pt', format: [format.width, format.height], orientation: ausrichtung });
    } else {
      doc.addPage([format.width, format.height], ausrichtung);
    }

    // Untere Ebene: das Foto, seitenfüllend und randlos.
    doc.addImage(toJpegDataUrl(canvas), 'JPEG', 0, 0, format.width, format.height);

    if (!words?.length) continue;

    // Obere Ebene. Schwarz gesetzt, aber im Modus „invisible" – die Farbe
    // zaehlt nur, falls ein Betrachter den Modus ignoriert.
    doc.setFont('helvetica', 'normal');
    doc.setTextColor(0, 0, 0);

    const fx = format.width / (wordsWidth || canvas.width);
    const fy = format.height / (wordsHeight || canvas.height);

    for (const word of words) {
      const text = sanitizeWinAnsi(word.text);
      if (!text) continue;

      const { x, y, size, breite } = wordPlacement(word, fx, fy);
      doc.setFontSize(size);
      doc.text(text, x, y, {
        renderingMode: 'invisible',
        baseline: 'alphabetic',
        horizontalScale: horizontalScaleFor(breite, doc.getTextWidth(text)),
      });
    }
  }

  return doc.output('blob');
}
