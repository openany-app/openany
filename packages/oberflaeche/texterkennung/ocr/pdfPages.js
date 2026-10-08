// Ein vorhandenes PDF fuer die Texterkennung aufschlagen.
//
// Der Unterschied zum Foto-Weg ist grundsaetzlich und bestimmt alles Weitere:
// Beim Foto BAUEN wir das PDF (buildPdf) – hier gibt es schon eins, und es
// bleibt. Wir rendern die Seiten nur, um Tesseract etwas zum Lesen zu geben;
// das Ergebnis ist eine Wortliste, die anschliessend in das ORIGINAL
// geschrieben wird (textLayer.js). Kein Pixel des Scans wird angefasst.
//
// Deshalb fehlt hier auch alles, was prepareImage.js tut: kein Zuschneiden,
// kein Ausgleichen der Beleuchtung, kein Geraderuecken. Nicht aus Bequem-
// lichkeit – wir DUERFEN das Bild nicht veraendern, denn die Wortkaesten
// muessen auf die Seite passen, wie sie ist. Ein schief eingescanntes Blatt
// bekommt eine schief liegende Textebene, und das ist richtig so: Sie deckt
// dann genau die Woerter, die dort stehen.

import { MAX_CANVAS_PIXELS, DISPLAY_LONG_EDGE } from './imageMath';
import { beurteileAusleuchtung } from './flatten';

/** 300 dpi ist die Schwelle, ab der Tesseract zuverlaessig liest. */
export const OCR_DPI = 300;

/** PDF-Punkte sind 1/72 Zoll – daraus wird der Renderfaktor. */
const PUNKTE_JE_ZOLL = 72;

/**
 * Ab wie vielen Zeichen je Seite wir annehmen, dass das PDF bereits eine
 * Textebene hat. Ein reiner Scan liefert null; ein PDF aus einem Textprogramm
 * liefert Hunderte. Der Wert liegt bewusst tief genug, um auch eine
 * halbleere Seite zu erkennen, und hoch genug, um an einer eingebetteten
 * Seitenzahl oder einem Wasserzeichen nicht haengenzubleiben.
 */
const TEXT_SCHWELLE = 40;

let pdfjsPromise = null;

/**
 * pdf.js lazy holen und ihm einen selbst gebauten Worker geben.
 *
 * Der Worker MUSS von hier kommen und nicht von einem CDN: Unsere CSP erlaubt
 * `worker-src 'self' blob:`, und ein externer Abruf je Dokument waere
 * ausserdem eine Datenspur.
 *
 * Warum `?worker` und nicht `?url` mit `GlobalWorkerOptions.workerSrc`: Die
 * Datei heisst im Paket `pdf.worker.min.mjs`. Mit `?url` landet genau diese
 * Endung im Build, und ob der Webserver `.mjs` mit einem JavaScript-MIME-Type
 * ausliefert, ist nicht garantiert. Zusammen mit `X-Content-Type-Options:
 * nosniff` (docker/php/Caddyfile) waere das kein Randfall, sondern ein
 * sicherer Fehlschlag – und zwar erst in Produktion, weil der Vite-Dev-Server
 * die Endung von sich aus richtig setzt. `?worker` laesst Vite den Worker
 * selbst buendeln und instanziieren; die Endung ist dann seine Sache.
 *
 * Auch der PDF-Betrachter (speicher/PdfBetrachter.vue) holt pdf.js hier:
 * ein Worker fuer beide, nicht zwei.
 */
export async function holePdfJs() {
  if (pdfjsPromise) return pdfjsPromise;

  pdfjsPromise = (async () => {
    const [pdfjs, worker] = await Promise.all([
      // LEGACY-FASSUNG, seit 27.09.2026. pdf.js 6 nutzt `Map.getOrInsertComputed`,
      // das Firefox 139 auf Tiffys Handy noch nicht kennt: „this[#e].getOrInsertComputed
      // is not a function". Die legacy-Fassung bringt Ersatz dafuer mit, in
      // Bibliothek UND Worker. Betrachter und Texterkennung teilen sich beides.
      import('pdfjs-dist/legacy/build/pdf.mjs'),
      import('pdfjs-dist/legacy/build/pdf.worker.min.mjs?worker'),
    ]);
    pdfjs.GlobalWorkerOptions.workerPort = new worker.default();
    return pdfjs;
  })().catch((e) => {
    pdfjsPromise = null;
    throw e;
  });

  return pdfjsPromise;
}

/**
 * PDF oeffnen. `daten` ist ein ArrayBuffer; pdf.js uebernimmt ihn und leert
 * ihn dabei – wer die Bytes spaeter noch braucht (und das tun wir, fuer die
 * Textebene), muss vorher eine Kopie behalten.
 */
/**
 * Ein geöffnetes Dokument wieder freigeben (Worker, entpackte Bilder).
 *
 * Seit pdf.js 6 hat das Dokument selbst kein `destroy()` mehr, nur seine
 * Ladeaufgabe. Der alte Aufruf warf -- in der Text-Auslese (textAuszug.js)
 * NACH dem Lesen, sodass jedes PDF als „unlesbar" galt (02.10.2026 beim
 * Bau der Inhaltssuche gefunden; die KI-Auslese der Webapp war genauso
 * betroffen).
 */
export function pdfFreigeben(pdf) {
  if (!pdf) return undefined;
  if (typeof pdf.loadingTask?.destroy === 'function') return pdf.loadingTask.destroy();
  if (typeof pdf.destroy === 'function') return pdf.destroy();
  return undefined;
}

export async function oeffnePdf(daten) {
  const pdfjs = await holePdfJs();
  return pdfjs.getDocument({
    data: daten,
    // Kein eval fuer eingebettete Schriften – unsere CSP verbietet es, und
    // gebraucht wird es nur beim Darstellen von Text, den ein Scan nicht hat.
    isEvalSupported: false,
    // Die Bilddecoder. Ohne diese Angabe holt pdf.js sie vom CDN, was an
    // unserer CSP scheitert – und dann rendert ausgerechnet der Normalfall
    // nicht: JBIG2 ist das Format, in dem Scanner ihre Seiten packen.
    // Die Dateien legt scripts/fetch-tesseract-assets.sh dorthin.
    wasmUrl: '/pdfjs/',
    // standardFontDataUrl fehlt bewusst. Die Standardschriften braucht nur,
    // wer eingebetteten TEXT darstellen will – ein Scan hat keinen, und ein
    // PDF, das welchen hat, lehnen wir vorher ab (hatSchonText). Ein weiteres
    // Megabyte mitzuschleppen, das nie geladen wird, waere Ballast.
  }).promise;
}

/**
 * Hat dieses PDF schon eine Textebene?
 *
 * Der Grund, warum das VOR der Erkennung geprueft wird: Eine zweite Textebene
 * ueber einer vorhandenen ist nicht neutral. Jedes Wort stuende doppelt im
 * Dokument, die Suche faende alles zweimal, und beim Markieren kaeme der Text
 * doppelt in die Zwischenablage. Ein PDF, das schon durchsuchbar ist, braucht
 * uns nicht – und ein Dokument zu verschlechtern, weil der Nutzer nicht wissen
 * konnte, dass es schon in Ordnung war, waere das schlechteste Ergebnis.
 *
 * Geprueft werden nur die ersten Seiten: Ein PDF mischt Scan und Digitalsatz
 * praktisch nie, und jede Seite kostet Zeit.
 */
export async function hatSchonText(pdf, maxSeiten = 3) {
  const bis = Math.min(pdf.numPages, maxSeiten);
  for (let n = 1; n <= bis; n += 1) {
    const seite = await pdf.getPage(n);
    const inhalt = await seite.getTextContent();
    const zeichen = inhalt.items.reduce((summe, i) => summe + (i.str?.trim().length ?? 0), 0);
    seite.cleanup();
    if (zeichen >= TEXT_SCHWELLE) return true;
  }
  return false;
}

/**
 * Eine Seite in ein Canvas zeichnen, gross genug fuer die Erkennung.
 *
 * Zurueck kommt neben dem Canvas der `viewport` – er ist der Schluessel zum
 * Rueckweg: Er weiss, wie Bildpixel wieder zu PDF-Punkten werden, samt
 * Seitendrehung und verschobenem Seitenrahmen. Diese Rechnung selbst
 * nachzubauen ist genau die Stelle, an der eine Textebene um Millimeter
 * verrutscht.
 */
export async function renderSeite(pdf, nummer) {
  const seite = await pdf.getPage(nummer);

  const roh = seite.getViewport({ scale: 1 });
  let skala = OCR_DPI / PUNKTE_JE_ZOLL;

  // Grosse Formate (A3-Plaene, Poster) sprengen sonst das Canvas-Limit; auf
  // iOS gibt der Kontext dann kommentarlos null zurueck.
  const flaeche = roh.width * roh.height * skala * skala;
  if (flaeche > MAX_CANVAS_PIXELS) skala *= Math.sqrt(MAX_CANVAS_PIXELS / flaeche);

  const viewport = seite.getViewport({ scale: skala });
  const canvas = document.createElement('canvas');
  canvas.width = Math.max(1, Math.floor(viewport.width));
  canvas.height = Math.max(1, Math.floor(viewport.height));

  const ctx = canvas.getContext('2d');
  if (!ctx) throw new Error('Canvas-Kontext nicht verfügbar.');
  // Weisser Grund: PDF-Seiten sind transparent, und Tesseract auf schwarzem
  // Grund liest nichts.
  ctx.fillStyle = '#ffffff';
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  await seite.render({ canvas, canvasContext: ctx, viewport }).promise;
  seite.cleanup();

  return { canvas, viewport };
}

/**
 * Wortkaesten aus Bildpixeln in PDF-Punkte umrechnen.
 *
 * Statt Winkel und Drehung selbst zu rechnen, werden ZWEI Punkte umgerechnet:
 * Anfang und Ende der Grundlinie. Ihr Abstand ist die Wortbreite in Punkten,
 * ihre Richtung ist die Schreibrichtung. Damit stimmt das Ergebnis auch auf
 * einer Seite mit /Rotate 90 oder 270 – und zwar ohne dass hier irgendwo eine
 * Fallunterscheidung nach Drehwinkel stuende, die man beim naechsten Sonderfall
 * wieder anfassen muesste.
 *
 * `ux`/`uy` ist der Richtungsvektor (Laenge 1); er geht spaeter unveraendert
 * als Textmatrix ins PDF.
 */
export function platziereWoerter(viewport, woerter) {
  const platz = [];

  for (const wort of woerter ?? []) {
    const [ax, ay] = viewport.convertToPdfPoint(wort.x0, wort.y1);
    const [bx, by] = viewport.convertToPdfPoint(wort.x1, wort.y1);

    const dx = bx - ax;
    const dy = by - ay;
    const breite = Math.hypot(dx, dy);
    if (!(breite > 0)) continue;

    platz.push({
      text: wort.text,
      x: ax,
      y: ay,
      ux: dx / breite,
      uy: dy / breite,
      breite,
      // Die Kastenhoehe in Punkten. Ein Betrag bleibt unter Drehung erhalten,
      // deshalb genuegt hier der Faktor.
      size: Math.max(1, (wort.y1 - wort.y0) / viewport.scale),
    });
  }

  return platz;
}

/**
 * Seitengroesse in PDF-Punkten, so wie sie im Original steht.
 *
 * Wichtig beim Ersetzen: Ein Brief auf Letter (612 x 792 pt) darf nicht als
 * A4 zurueckkommen. Der viewport traegt die Drehung schon mit, deshalb genuegt
 * es, seine Masse durch den Renderfaktor zu teilen.
 */
export function seitenFormat(viewport) {
  return {
    width: viewport.width / viewport.scale,
    height: viewport.height / viewport.scale,
  };
}

/**
 * Aus dem grossen Arbeitsbild eine kleinere, sichtbare Fassung machen.
 *
 * Nur noetig, wenn die Seite neu aufgebaut wird (der aufbereitende Weg).
 * Dieselbe Trennung wie beim Foto: Die Erkennung will 300 dpi, das Auge kommt
 * mit 150 aus, und die Datei dankt es. Verkleinert wird aus dem bereits
 * gerenderten Canvas statt mit einem zweiten Durchlauf durch pdf.js – das
 * spart die halbe Zeit und sieht genauso aus.
 */
export function anzeigeFassung(canvas, langeKante = DISPLAY_LONG_EDGE) {
  const skala = Math.min(1, langeKante / Math.max(canvas.width, canvas.height));
  if (skala >= 1) return canvas;

  const ziel = document.createElement('canvas');
  ziel.width = Math.max(1, Math.round(canvas.width * skala));
  ziel.height = Math.max(1, Math.round(canvas.height * skala));

  const ctx = ziel.getContext('2d');
  if (!ctx) return canvas;
  ctx.imageSmoothingEnabled = true;
  ctx.imageSmoothingQuality = 'high';
  ctx.drawImage(canvas, 0, 0, ziel.width, ziel.height);

  return ziel;
}

/**
 * Wie steht es um die Ausleuchtung dieser Seite? Liest die Pixel des
 * gerenderten Canvas und reicht sie an die Beurteilung weiter.
 */
export function beurteileSeite(canvas) {
  const ctx = canvas.getContext('2d');
  if (!ctx || !canvas.width || !canvas.height) {
    return { weisspunkt: 255, spanne: 0, lohnt: false };
  }
  const bild = ctx.getImageData(0, 0, canvas.width, canvas.height);
  return beurteileAusleuchtung(bild.data, canvas.width, canvas.height);
}

/** Canvas freigeben – ein 300-dpi-A4-Bild sind 35 MB. */
export function gibSeiteFrei(canvas) {
  if (!canvas) return;
  canvas.width = 0;
  canvas.height = 0;
}
