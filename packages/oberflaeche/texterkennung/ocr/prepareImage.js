// Canvas-Anbindung der Foto-Aufbereitung. Die Rechnerei steht daneben in
// imageMath.js und ist dort ohne DOM getestet; hier geht es nur darum, Pixel
// von A nach B zu bringen.
//
// Zwei Bilder verlassen diese Datei, mit demselben Ausschnitt und demselben
// Drehwinkel, aber ABSICHTLICH unterschiedlicher Aufloesung:
//
//   - ein graues fuer Tesseract, bei ~300 dpi – so viele Pixel wie die
//     Erkennung braucht,
//   - ein farbiges fuer die sichtbare PDF-Ebene, bei ~150 dpi – so wenige,
//     wie fuer das Auge reichen.
//
// Die Trennung ist der Grund, warum ein einseitiges PDF nicht mehrere Megabyte
// wiegt, ohne dass die Erkennung etwas abgibt. Sie hat aber einen Preis, den
// man kennen muss: Die Wortkaesten von Tesseract zaehlen in Pixeln des GRAUEN
// Bildes. buildPdf bekommt dessen Masse deshalb ausdruecklich mitgeteilt
// (`wordsWidth`/`wordsHeight`) – wer stattdessen die Masse des sichtbaren
// Bildes nimmt, legt die Textebene um den Faktor der Aufloesungsdifferenz
// daneben.
//
// Die Trennung traegt inzwischen eine zweite Sache: Das GLAETTEN bekommt nur
// die sichtbare Ebene. An drei echten Handyfotos nachgemessen, aendert es die
// Zahl der sicheren Treffer kaum, laesst aber die unsicheren um die Haelfte
// steigen – es hellt Rauschen auf, und Tesseract liest daraus Buchstaben. Die
// Erkennung arbeitet deshalb weiter auf dem unveraenderten Bild.
//
// Der ZUSCHNITT dagegen trifft beide gleich: Er passiert vor allem anderen,
// also sehen Erkennung und Anzeige denselben Ausschnitt.

import { toGray, estimateSkew, targetScale, displayScale, rotatedBounds } from './imageMath';
import { detectPageQuad } from './detectPage';
import { plausibleQuad } from './pageGeometry';
import { flatten } from './flatten';

/**
 * Der Nutzer sieht am Ende dieses Bild – nicht das graue Arbeitsbild. Weil das
 * Original nach der Umwandlung nicht aufgehoben wird, ist das die einzige
 * verbleibende Fassung.
 *
 * 0.75 zusammen mit den 150 dpi aus imageMath: an einem echten Handyfoto
 * durchgemessen, ergibt rund 40 % der Dateigroesse von 200 dpi / 0.85. Wer
 * hier weiter runter geht, sieht es zuerst an den Kanten kleiner Schrift –
 * die Aufloesung ist der schonendere Regler als die Qualitaet.
 */
const JPEG_QUALITAET = 0.75;

/** Kantenlaenge der Hilfskopie, auf der der Schiefwinkel gemessen wird. */
const MESSKANTE = 800;

/**
 * Foto als Bitmap, EXIF-Drehung bereits angewandt.
 *
 * `imageOrientation: 'from-image'` nimmt uns genau die Arbeit ab, die der
 * Server bisher mit exif_read_data und imagerotate von Hand gemacht hat
 * (FileNodeController::exportPdf). Handyfotos speichern die Drehung meist nur
 * als EXIF-Marke; ohne diesen Schalter liegt das Blatt quer.
 */
export async function loadOriginal(file) {
  return createImageBitmap(file, { imageOrientation: 'from-image' });
}

/** Canvas ohne DOM-Anhang. Wirft, wenn der Browser keinen 2D-Kontext gibt. */
function leinwand(width, height) {
  const canvas = document.createElement('canvas');
  canvas.width = width;
  canvas.height = height;
  const ctx = canvas.getContext('2d', { willReadFrequently: false });
  if (!ctx) throw new Error('Kein 2D-Kontext verfügbar.');
  return { canvas, ctx };
}

/**
 * Schiefwinkel des Blattes. Gemessen wird auf einer kleinen Kopie – bei 800 px
 * ist der Winkel genauso gut ablesbar wie bei 4000, kostet aber ein Fuenfzigstel
 * der Rechenzeit.
 *
 * Rueckgabe ist der Korrekturwinkel (siehe imageMath.estimateSkew): direkt an
 * renderRotated weiterreichen.
 */
export function measureSkew(bitmap, quelle = null) {
  const q = quelle ?? { x: 0, y: 0, width: bitmap.width, height: bitmap.height };
  const skala = Math.min(1, MESSKANTE / Math.max(q.width, q.height));
  const width = Math.max(1, Math.round(q.width * skala));
  const height = Math.max(1, Math.round(q.height * skala));

  const { canvas, ctx } = leinwand(width, height);
  ctx.drawImage(bitmap, q.x, q.y, q.width, q.height, 0, 0, width, height);
  const { data } = ctx.getImageData(0, 0, width, height);
  const grad = estimateSkew(toGray(data, width, height), width, height);

  freigeben(canvas);
  return grad;
}

/**
 * Zeichnet das Foto gedreht und skaliert neu.
 *
 * Der Hintergrund wird weiss gefuellt: Beim Drehen entstehen an den Ecken
 * Zwickel, die sonst schwarz oder durchsichtig blieben – auf einem
 * Dokumentenfoto sieht beides nach Fehler aus, und Tesseract haelt schwarze
 * Flaechen fuer Inhalt.
 *
 * @param {ImageBitmap} bitmap  Vorlage
 * @param {number} grad         Korrekturwinkel aus measureSkew
 * @param {number} skala        Faktor aus imageMath.targetScale
 * @param {boolean} grau        true fuer das Arbeitsbild, false fuers PDF
 */
export function renderRotated(bitmap, grad, skala, { grau = false, quelle = null } = {}) {
  const q = quelle ?? { x: 0, y: 0, width: bitmap.width, height: bitmap.height };
  const gedreht = rotatedBounds(q.width, q.height, grad);
  const width = Math.max(1, Math.round(gedreht.width * skala));
  const height = Math.max(1, Math.round(gedreht.height * skala));

  const { canvas, ctx } = leinwand(width, height);
  ctx.fillStyle = '#ffffff';
  ctx.fillRect(0, 0, width, height);

  ctx.imageSmoothingEnabled = true;
  ctx.imageSmoothingQuality = 'high';
  if (grau) ctx.filter = 'grayscale(1)';

  ctx.translate(width / 2, height / 2);
  ctx.rotate((grad * Math.PI) / 180);
  ctx.drawImage(
    bitmap,
    q.x, q.y, q.width, q.height,
    (-q.width * skala) / 2,
    (-q.height * skala) / 2,
    q.width * skala,
    q.height * skala,
  );
  ctx.setTransform(1, 0, 0, 1, 0, 0);
  ctx.filter = 'none';

  return canvas;
}

/**
 * Ausschnitt, der das Blatt enthaelt – oder `null`, wenn keiner sicher genug
 * gefunden wurde.
 *
 * ACHSENPARALLELE HUELLBOX, bewusst keine perspektivische Entzerrung. An drei
 * echten Fotos gemessen lag bei zweien eine Ecke daneben: einmal frass ein
 * Schatten die obere rechte Ecke, einmal zaehlte ein weisses Kabel als Teil
 * des Blattes. Eine Huellbox nimmt je Achse den aeussersten der vier Punkte –
 * eine zu weit innen liegende Ecke fuehrt also nur dazu, dass WENIGER
 * weggeschnitten wird. Eine Entzerrung rechnet dagegen DURCH die Ecken und
 * schneidet bei einer falschen Ecke Inhalt weg. Das Original ist danach fort;
 * dieser Handel lohnt sich nicht.
 */
export function detectCropRect(bitmap, { messkante = 600, rand = 0.01 } = {}) {
  const skala = Math.min(1, messkante / Math.max(bitmap.width, bitmap.height));
  const width = Math.max(1, Math.round(bitmap.width * skala));
  const height = Math.max(1, Math.round(bitmap.height * skala));

  const { canvas, ctx } = leinwand(width, height);
  ctx.drawImage(bitmap, 0, 0, width, height);
  const { data } = ctx.getImageData(0, 0, width, height);
  const quad = detectPageQuad(toGray(data, width, height), width, height);
  freigeben(canvas);

  if (!quad || !plausibleQuad(quad, width, height)) return null;

  // Zurueck auf Originalmasse, mit etwas Luft nach aussen.
  const zurueck = 1 / skala;
  const luft = Math.round(rand * Math.max(bitmap.width, bitmap.height));
  const x0 = Math.max(0, Math.round(Math.min(...quad.map((p) => p.x)) * zurueck) - luft);
  const y0 = Math.max(0, Math.round(Math.min(...quad.map((p) => p.y)) * zurueck) - luft);
  const x1 = Math.min(bitmap.width, Math.round(Math.max(...quad.map((p) => p.x)) * zurueck) + luft);
  const y1 = Math.min(bitmap.height, Math.round(Math.max(...quad.map((p) => p.y)) * zurueck) + luft);

  const w = x1 - x0;
  const h = y1 - y0;
  // Weniger als ein Prozent gespart: den Aufwand nicht wert, und jeder
  // Zuschnitt ist ein Risiko, das sich lohnen muss.
  if (w <= 0 || h <= 0 || (w * h) / (bitmap.width * bitmap.height) > 0.99) return null;

  return { x: x0, y: y0, width: w, height: h };
}

/**
 * Beide Fassungen eines Fotos in einem Rutsch – die einzige Stelle, an der
 * Winkel und Skalen festgelegt werden. Wer sie getrennt aufruft, riskiert zwei
 * Bilder mit unterschiedlichem Winkel oder Ausschnitt, und damit eine
 * Textebene, die neben den Woertern liegt.
 *
 * Gibt neben den beiden Leinwaenden `wordsWidth`/`wordsHeight` zurueck: die
 * Masse, auf die sich die Wortkaesten beziehen.
 */
export async function prepareForOcr(file) {
  const bitmap = await loadOriginal(file);
  try {
    // Zuschnitt zuerst: Was danach kommt – Winkelmessung, Skalierung – soll
    // das Blatt sehen und nicht den Tisch darum herum.
    const quelle = detectCropRect(bitmap)
      ?? { x: 0, y: 0, width: bitmap.width, height: bitmap.height };

    const grad = measureSkew(bitmap, quelle);
    const grau = renderRotated(bitmap, grad, targetScale(quelle.width, quelle.height, grad), { grau: true, quelle });
    const farbe = renderRotated(bitmap, grad, displayScale(quelle.width, quelle.height, grad), { grau: false, quelle });

    // Nur die sichtbare Ebene wird geglaettet – siehe Kopf dieser Datei.
    glaetten(farbe);

    return {
      grau,
      farbe,
      grad,
      // Bezugsrahmen der Wortkaesten. Muss mitreisen, weil `farbe` kleiner ist.
      wordsWidth: grau.width,
      wordsHeight: grau.height,
    };
  } finally {
    // Ein 12-Megapixel-Foto haengt sonst bis zur naechsten
    // Garbage-Collection im Speicher – bei zehn Fotos hintereinander macht
    // das mehrere hundert Megabyte aus.
    bitmap.close?.();
  }
}

/** Canvas als JPEG-Datenzeile fuer jsPDF. */
export function toJpegDataUrl(canvas) {
  return canvas.toDataURL('image/jpeg', JPEG_QUALITAET);
}

/**
 * Gibt den Speicher eines Canvas frei. `width = 0` ist der einzige Weg, der
 * auch auf iOS wirklich greift; die Referenz allein fallen zu lassen reicht
 * dort nicht zuverlaessig.
 */
export function freigeben(canvas) {
  if (!canvas) return;
  canvas.width = 0;
  canvas.height = 0;
}

/**
 * Beleuchtung der sichtbaren Ebene ausgleichen. Kapselt den Zugriff auf die
 * Bilddaten, damit flatten() selbst ohne Canvas testbar bleibt.
 *
 * Der Weisspunkt sammelt das Rauschen im Papier ein: Ohne ihn kostet das
 * Glaetten rund ein Drittel mehr Dateigroesse, weil das Aufhellen die Koernung
 * mit aufhellt – und Koernung ist genau das, was JPEG teuer macht.
 */
export function glaetten(canvas, optionen = { weisspunkt: 230 }) {
  const ctx = canvas.getContext('2d');
  if (!ctx || !canvas.width || !canvas.height) return canvas;

  const bild = ctx.getImageData(0, 0, canvas.width, canvas.height);
  flatten(bild.data, canvas.width, canvas.height, optionen);
  ctx.putImageData(bild, 0, 0);

  return canvas;
}
