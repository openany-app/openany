// Die Rechenteile der Foto-Aufbereitung, bewusst ohne Canvas und ohne Vue:
// so laufen sie in der node-Umgebung von Vitest. Die Canvas-Anbindung liegt
// daneben in prepareImage.js.
//
// Die Reihenfolge ist Absicht und der Kern der Sache: Wir richten das Blatt
// gerade und vergroessern es, aber wir binarisieren das Bild NICHT, das an
// Tesseract geht. Die LSTM-Engine bringt ihre eigene, lokal arbeitende
// Aufbereitung mit und wird durch ein vorab plattgerechnetes Schwarz-Weiss-Bild
// eher schlechter. Otsu kommt hier trotzdem vor – aber nur auf der kleinen
// Hilfskopie, auf der wir den Schiefwinkel messen. Dort ist ein globaler
// Schwellenwert genau richtig, weil uns nur interessiert, wo ueberhaupt
// Schrift steht, nicht wie sie aussieht.

/** Tesseract will rund 300 dpi sehen; A4 laengs sind das ~3500 px. */
export const OCR_LONG_EDGE = 3500;

/**
 * Die SICHTBARE Ebene darf deutlich kleiner sein als das Arbeitsbild – rund
 * 150 dpi auf A4. Das ist der Grund, warum ein einseitiges PDF nicht mehrere
 * Megabyte wiegen muss: Die 300 dpi braucht Tesseract, nicht das Auge.
 *
 * Weil beide Ebenen getrennt gerechnet werden, kostet das die Erkennung
 * NICHTS – der durchsuchbare Text bleibt so genau wie bei voller Aufloesung.
 *
 * 150 statt der urspruenglichen 200 dpi, weil an einem echten Handyfoto
 * nachgemessen: 200 dpi ergaben rund 1,7 MB je Seite, 150 dpi bei
 * Qualitaet 0.75 noch gut 40 % davon. Am Bildschirm ist das nicht zu
 * unterscheiden, im Ausdruck minimal weicher.
 */
export const DISPLAY_LONG_EDGE = 1754;

/**
 * Obergrenze fuer die Canvas-Flaeche. Safari auf iOS gibt oberhalb von rund
 * 16,7 Mio. Pixeln kommentarlos `null` als Kontext zurueck – lieber ein etwas
 * kleineres Bild als gar keins.
 */
export const MAX_CANVAS_PIXELS = 16_000_000;

/** Mehr als das Vierfache hochzurechnen bringt keine Erkennung mehr dazu. */
const MAX_UPSCALE = 4;

/** Graustufen nach Luma (BT.601) – dieselbe Gewichtung, die auch Video nutzt. */
export function toGray(rgba, width, height) {
  const gray = new Uint8Array(width * height);
  for (let i = 0, p = 0; p < gray.length; i += 4, p += 1) {
    gray[p] = (rgba[i] * 0.299 + rgba[i + 1] * 0.587 + rgba[i + 2] * 0.114) | 0;
  }
  return gray;
}

/** 256 Faecher, wie die Graustufen selbst. */
export function histogram(gray) {
  const bins = new Uint32Array(256);
  for (let i = 0; i < gray.length; i += 1) bins[gray[i]] += 1;
  return bins;
}

/**
 * Otsu: der Schwellenwert, der die Varianz ZWISCHEN den beiden Klassen maximiert
 * – anschaulich der Schnitt, der die zwei Gipfel eines Dokument-Histogramms
 * (Papier hell, Schrift dunkel) am saubersten trennt.
 *
 * Der zurueckgegebene Wert gehoert zur DUNKLEN Klasse: dunkel ist `wert <= t`.
 * Bei einem sauber zweigipfligen Bild landet t deshalb auf dem dunklen Gipfel
 * selbst – wer mit `<` statt `<=` vergleicht, verliert die komplette Schrift.
 *
 * Gibt es nur einen Gipfel (leeres oder komplett ueberbelichtetes Bild), ist
 * jede Trennung gleich schlecht; dann faellt der Wert auf 127 zurueck.
 */
export function otsuThreshold(bins) {
  const gesamt = bins.reduce((a, b) => a + b, 0);
  if (gesamt === 0) return 127;

  let summe = 0;
  for (let i = 0; i < 256; i += 1) summe += i * bins[i];

  let summeHintergrund = 0;
  let gewichtHintergrund = 0;
  let besteVarianz = -1;
  let bester = 127;

  for (let t = 0; t < 256; t += 1) {
    gewichtHintergrund += bins[t];
    if (gewichtHintergrund === 0) continue;
    const gewichtVordergrund = gesamt - gewichtHintergrund;
    if (gewichtVordergrund === 0) break;

    summeHintergrund += t * bins[t];
    const mittelHintergrund = summeHintergrund / gewichtHintergrund;
    const mittelVordergrund = (summe - summeHintergrund) / gewichtVordergrund;
    const abstand = mittelHintergrund - mittelVordergrund;
    const varianz = gewichtHintergrund * gewichtVordergrund * abstand * abstand;

    if (varianz > besteVarianz) {
      besteVarianz = varianz;
      bester = t;
    }
  }

  return bester;
}

/**
 * Schiefwinkel eines abfotografierten Blattes, in Grad – und zwar bereits als
 * KORREKTURWINKEL: der Wert kann unveraendert an ctx.rotate() (bzw.
 * renderRotated) weitergereicht werden und stellt das Blatt gerade. Ein Blatt,
 * dessen Zeilen nach rechts unten abfallen, ergibt einen negativen Wert.
 * Diese Richtung steckt hier drin, damit sie nicht an jedem Aufrufort neu
 * gedreht – und irgendwann falsch gedreht – werden muss.
 *
 * Verfahren (Projektionsprofil): Fuer jeden Kandidatenwinkel zaehlen wir, wie
 * viele dunkle Pixel auf dieselbe gedachte Zeile fallen. Steht das Blatt
 * gerade, sammeln sich alle Buchstaben einer Textzeile in wenigen Zeilen und
 * dazwischen ist nichts – das Profil bekommt hohe Gipfel und tiefe Taeler.
 * Steht es schief, verschmiert alles. Die Summe der Quadrate misst genau das:
 * sie ist maximal, wenn die Masse auf wenige Zeilen konzentriert ist.
 *
 * Ausdruecklich KEINE Vorstufe zum Binarisieren des OCR-Bildes – das
 * Schwarz-Weiss-Bild entsteht hier, wird gemessen und weggeworfen.
 */
export function estimateSkew(gray, width, height, { maxDeg = 8, schritt = 0.5 } = {}) {
  if (width < 20 || height < 20) return 0;

  const schwelle = otsuThreshold(histogram(gray));

  // Nur die dunklen Pixel interessieren. Sie einmal einzusammeln spart bei
  // jedem der ~33 Winkel den Durchlauf durch das ganze Bild.
  const xs = [];
  const ys = [];
  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < width; x += 1) {
      if (gray[y * width + x] <= schwelle) {
        xs.push(x);
        ys.push(y);
      }
    }
  }

  // Ein leeres oder fast schwarzes Bild gibt keinen belastbaren Winkel her.
  const anteil = xs.length / (width * height);
  if (xs.length < 200 || anteil > 0.6) return 0;

  const mitteX = width / 2;
  let besterWinkel = 0;
  let bestesMass = -1;

  for (let grad = -maxDeg; grad <= maxDeg + 1e-9; grad += schritt) {
    const tan = Math.tan((grad * Math.PI) / 180);
    // Die Zeilenzahl waechst mit der Scherung; grosszuegig Platz lassen.
    const versatz = Math.ceil(Math.abs(tan) * mitteX) + 1;
    const zeilen = new Uint32Array(height + 2 * versatz);

    for (let i = 0; i < xs.length; i += 1) {
      const zeile = (ys[i] - (xs[i] - mitteX) * tan + versatz) | 0;
      if (zeile >= 0 && zeile < zeilen.length) zeilen[zeile] += 1;
    }

    let mass = 0;
    for (let i = 0; i < zeilen.length; i += 1) mass += zeilen[i] * zeilen[i];

    if (mass > bestesMass) {
      bestesMass = mass;
      besterWinkel = grad;
    }
  }

  // Unter einem drittel Grad ist die Drehung nicht zu sehen, kostet aber eine
  // zweite Interpolation des ganzen Bildes. Dann lieber gar nicht drehen.
  return Math.abs(besterWinkel) < 0.3 ? 0 : Math.round(besterWinkel * 100) / 100;
}

/** Kantenlaengen, die ein um `grad` gedrehtes Rechteck einnimmt. */
export function rotatedBounds(width, height, grad) {
  const bogen = (grad * Math.PI) / 180;
  const c = Math.abs(Math.cos(bogen));
  const s = Math.abs(Math.sin(bogen));
  return {
    width: Math.ceil(width * c + height * s),
    height: Math.ceil(width * s + height * c),
  };
}

/**
 * Faktor, mit dem das Foto fuer die Erkennung skaliert wird.
 *
 * Kleine Fotos werden hochgerechnet, bis die lange Kante ~300 dpi entspricht –
 * das ist der groesste einzelne Hebel auf die Trefferquote, weil Tesseract eine
 * x-Hoehe von rund 20 px braucht. Grosse Fotos werden NICHT verkleinert, ausser
 * die gedrehte Flaeche sprengt das Canvas-Limit.
 */
export function targetScale(width, height, grad = 0, {
  longEdge = OCR_LONG_EDGE,
  maxPixels = MAX_CANVAS_PIXELS,
} = {}) {
  const lang = Math.max(width, height);
  if (lang === 0) return 1;

  let skala = Math.min(Math.max(1, longEdge / lang), MAX_UPSCALE);

  const gedreht = rotatedBounds(width, height, grad);
  const flaeche = gedreht.width * gedreht.height * skala * skala;
  if (flaeche > maxPixels) {
    skala *= Math.sqrt(maxPixels / flaeche);
  }

  return skala;
}

/**
 * Faktor fuer die SICHTBARE Ebene – die Umkehrung von targetScale.
 *
 * Hier wird nur VERKLEINERT, nie vergroessert: Ein Foto ueber die eigene
 * Aufloesung hinaus aufzublasen kostet Bytes und bringt kein einziges Detail
 * dazu. Ist die Vorlage kleiner als das Ziel, bleibt sie wie sie ist.
 *
 * Getrennt von targetScale, weil die beiden Ebenen gegenlaeufige Interessen
 * haben: Die Erkennung will so viele Pixel wie moeglich, die Datei so wenige
 * wie noetig.
 */
export function displayScale(width, height, grad = 0, {
  longEdge = DISPLAY_LONG_EDGE,
  maxPixels = MAX_CANVAS_PIXELS,
} = {}) {
  const lang = Math.max(width, height);
  if (lang === 0) return 1;

  let skala = Math.min(1, longEdge / lang);

  const gedreht = rotatedBounds(width, height, grad);
  const flaeche = gedreht.width * gedreht.height * skala * skala;
  if (flaeche > maxPixels) {
    skala *= Math.sqrt(maxPixels / flaeche);
  }

  return skala;
}

/**
 * Seitenformat in PDF-Punkten. Die lange Kante liegt auf A4-Laenge, das
 * Seitenverhaeltnis folgt dem Foto – so bleibt ringsum kein weisser Rand.
 * Gleiche Regel wie der serverseitige Export in FileNodeController::exportPdf,
 * damit alte und neue PDFs nebeneinander gleich aussehen.
 */
export function pdfPageSize(width, height, longEdge = 842) {
  if (width >= height) {
    return { width: longEdge, height: (longEdge * height) / width };
  }
  return { width: (longEdge * width) / height, height: longEdge };
}
