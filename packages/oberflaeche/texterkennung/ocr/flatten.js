// Ungleiche Ausleuchtung herausrechnen: Das Papier wird gleichmaessig hell,
// Schrift und Farben bleiben.
//
// Das Verfahren ist eine Division durch den geschaetzten Hintergrund. Man
// nimmt an, dass jedes Pixel das Produkt aus "was dort gedruckt ist" und "wie
// hell es dort beleuchtet wird" ist. Kennt man die Beleuchtung, teilt man sie
// heraus und behaelt den Druck.
//
// Zwei Entscheidungen, die den Unterschied machen:
//
// 1. Der Hintergrund wird ueber das MAXIMUM je Kachel geschaetzt, nicht ueber
//    den Mittelwert. In einer Kachel mit Text zoege der Mittelwert die
//    Schaetzung nach unten – und weil anschliessend geteilt wird, wuerde
//    ausgerechnet der Text aufgehellt und damit blasser. Das Maximum trifft
//    das unbedruckte Papier daneben.
//
// 2. Der Faktor wird auf ALLE drei Kanaele gleich angewandt. Damit bleiben die
//    Verhaeltnisse zwischen Rot, Gruen und Blau erhalten – ein rotes Logo
//    bleibt rot und wird nur heller. Kanalweise zu normieren wuerde Farbstiche
//    wegrechnen, aber eben auch die Farben selbst.
//
// Anders als eine Binarisierung wirft das keine Information weg: Grauwerte
// bleiben Grauwerte, sie liegen nur wieder dort, wo sie ohne Schatten laegen.

/** Ziel-Helligkeit des Papiers. Nicht 255 – sonst kippt jedes Rauschen auf Weiss. */
const ZIEL = 244;

/** Kantenlaenge einer Kachel in Pixeln, bezogen auf die lange Bildkante. */
const KACHELN_PRO_KANTE = 48;

/**
 * Ein zu grosser Faktor macht aus Bildrauschen in dunklen Ecken (Tisch neben
 * dem Blatt) helles Gekrissel. Deckeln kostet dort etwas Aufhellung und
 * erspart das.
 */
const MAX_FAKTOR = 3.5;

/**
 * Hintergrund-Schaetzung als grobes Raster: je Kachel die groesste Helligkeit.
 * @returns {{werte: Float32Array, gw: number, gh: number}}
 */
export function backgroundGrid(rgba, width, height, kachel) {
  const gw = Math.max(1, Math.ceil(width / kachel));
  const gh = Math.max(1, Math.ceil(height / kachel));
  const werte = new Float32Array(gw * gh);

  for (let ky = 0; ky < gh; ky += 1) {
    for (let kx = 0; kx < gw; kx += 1) {
      const x1 = Math.min(width, (kx + 1) * kachel);
      const y1 = Math.min(height, (ky + 1) * kachel);
      let max = 0;

      for (let y = ky * kachel; y < y1; y += 1) {
        for (let x = kx * kachel; x < x1; x += 1) {
          const i = (y * width + x) * 4;
          const luma = rgba[i] * 0.299 + rgba[i + 1] * 0.587 + rgba[i + 2] * 0.114;
          if (luma > max) max = luma;
        }
      }

      werte[ky * gw + kx] = max;
    }
  }

  return { werte, gw, gh };
}

/**
 * Raster gleitend mitteln (3x3). Ohne diesen Schritt zeichnen sich die
 * Kachelgrenzen als Karomuster im fertigen Bild ab – der Faktor springt sonst
 * an jeder Kante.
 */
export function smoothGrid({ werte, gw, gh }) {
  const out = new Float32Array(werte.length);

  for (let y = 0; y < gh; y += 1) {
    for (let x = 0; x < gw; x += 1) {
      let summe = 0;
      let anzahl = 0;
      for (let dy = -1; dy <= 1; dy += 1) {
        for (let dx = -1; dx <= 1; dx += 1) {
          const nx = x + dx;
          const ny = y + dy;
          if (nx < 0 || ny < 0 || nx >= gw || ny >= gh) continue;
          summe += werte[ny * gw + nx];
          anzahl += 1;
        }
      }
      out[y * gw + x] = summe / anzahl;
    }
  }

  return { werte: out, gw, gh };
}

/** Zwischenwert aus dem Raster an einer Pixelposition (bilinear). */
export function sampleGrid({ werte, gw, gh }, x, y, kachel) {
  // Rastermitten liegen bei (k + 0.5) * kachel.
  const fx = Math.min(gw - 1, Math.max(0, x / kachel - 0.5));
  const fy = Math.min(gh - 1, Math.max(0, y / kachel - 0.5));
  const x0 = Math.floor(fx);
  const y0 = Math.floor(fy);
  const x1 = Math.min(gw - 1, x0 + 1);
  const y1 = Math.min(gh - 1, y0 + 1);
  const tx = fx - x0;
  const ty = fy - y0;

  const oben = werte[y0 * gw + x0] * (1 - tx) + werte[y0 * gw + x1] * tx;
  const unten = werte[y1 * gw + x0] * (1 - tx) + werte[y1 * gw + x1] * tx;

  return oben * (1 - ty) + unten * ty;
}

/**
 * Lohnt sich das Aufhellen auf DIESER Seite ueberhaupt?
 *
 * Gebraucht wird das nur auf dem PDF-Weg, und dort ist es die ganze
 * Entscheidung. Beim Foto ist die Antwort immer ja: Das Bild wird ohnehin neu
 * gezeichnet, das Glaetten ist ein Zwischenschritt auf einem Weg, den wir
 * sowieso gehen. Bei einem fertigen PDF ist es umgekehrt – aufhellen heisst
 * neu rastern, und das ist ein Preis. Er lohnt sich bei einem Handy-„Scan"
 * mit Schatten quer ueber der Seite, und er waere reiner Verlust bei einem
 * Flachbett-Scan, der schon sauber ist, oder bei einer Seite, die als reines
 * Schwarz-Weiss vorliegt.
 *
 * Gemessen wird auf demselben Kachelraster, das flatten() selbst benutzt: je
 * Kachel die groesste Helligkeit, also das unbedruckte Papier daneben.
 * Daraus zwei Zahlen:
 *
 *   weisspunkt  wie hell das Papier an seinen hellsten Stellen ist. 255 heisst
 *               reinweiss, 203 (an einem echten Handyfoto gemessen) heisst
 *               deutlich grau.
 *   spanne      der Unterschied zwischen den hellsten und den dunkelsten
 *               Papierstellen. Das ist die Zahl, die den flachen Schleier vom
 *               ungleichmaessigen Schatten trennt – und nur gegen den zweiten
 *               hilft eine kachelweise Division.
 *
 * Beide als Perzentil statt als Extremwert: Ein schwarzer Stempel oder ein
 * heller Kratzer darf nicht ueber die ganze Seite entscheiden.
 *
 * Eine bereits binarisierte Seite faellt hier von selbst durch – ihr Papier
 * ist ueberall 255, die Spanne ist 0 –, ohne dass es dafuer einen Sonderfall
 * braeuchte.
 */
export function beurteileAusleuchtung(rgba, width, height, {
  minWeisspunkt = 240,
  maxSpanne = 20,
} = {}) {
  const kachel = Math.max(8, Math.round(Math.max(width, height) / KACHELN_PRO_KANTE));
  const { werte } = backgroundGrid(rgba, width, height, kachel);
  if (!werte.length) return { weisspunkt: 255, spanne: 0, lohnt: false };

  const sortiert = Float32Array.from(werte).sort();
  const bei = (anteil) => sortiert[Math.min(sortiert.length - 1,
    Math.max(0, Math.round(anteil * (sortiert.length - 1))))];

  const weisspunkt = bei(0.9);
  const spanne = weisspunkt - bei(0.1);

  return {
    weisspunkt,
    spanne,
    lohnt: weisspunkt < minWeisspunkt || spanne > maxSpanne,
  };
}

/**
 * Beleuchtung herausrechnen. Arbeitet IN PLACE auf den Bilddaten – bei einem
 * 12-Megapixel-Bild waere eine zweite Kopie 48 MB, die es nicht braucht.
 *
 * @param {Uint8ClampedArray} rgba
 * @returns {Uint8ClampedArray} dasselbe Feld, veraendert
 */
export function flatten(rgba, width, height, {
  ziel = ZIEL,
  maxFaktor = MAX_FAKTOR,
  weisspunkt = 0,
} = {}) {
  // Weisspunkt: Alles oberhalb dieser Helligkeit wird auf reines Weiss
  // gezogen, darunter linear gestreckt.
  //
  // Wozu: Nach dem Glaetten ist das Papier zwar gleichmaessig hell, aber immer
  // noch koernig – und Koernung ist genau das, was JPEG teuer macht. Der
  // Weisspunkt sammelt dieses Rauschen ein.
  //
  // Wogegen abzuwaegen ist: Hellgraue INHALTE verschwinden mit. Tabellen-
  // schattierungen und blasse Stempel liegen genau in diesem Bereich. Deshalb
  // standardmaessig aus (0) – wer ihn setzt, sollte wissen, dass er etwas
  // wegwirft und nicht nur aufraeumt.
  const streckung = weisspunkt > 0 ? 255 / weisspunkt : 0;
  const kachel = Math.max(8, Math.round(Math.max(width, height) / KACHELN_PRO_KANTE));
  const raster = smoothGrid(backgroundGrid(rgba, width, height, kachel));

  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < width; x += 1) {
      const hintergrund = sampleGrid(raster, x, y, kachel);
      // Eine praktisch schwarze Kachel gibt keinen brauchbaren Bezug her.
      if (hintergrund < 1) continue;

      const i = (y * width + x) * 4;
      const r = rgba[i];
      const g = rgba[i + 1];
      const b = rgba[i + 2];

      // Zweiter Deckel, und der ist der wichtigere: So viel, dass kein Kanal
      // ueber 255 laeuft. Sonst schneidet die Ablage den ueberstehenden Kanal
      // ab, waehrend die anderen weiterwachsen – und ein saettiges Rot wird
      // beim Aufhellen zu Rosa. Lieber ein Logo etwas dunkler als in der
      // falschen Farbe.
      const hellster = Math.max(r, g, b);
      const faktor = Math.min(
        maxFaktor,
        ziel / hintergrund,
        hellster > 0 ? 255 / hellster : maxFaktor,
      );

      let nr = r * faktor;
      let ng = g * faktor;
      let nb = b * faktor;

      if (streckung > 0) {
        nr = nr >= weisspunkt ? 255 : nr * streckung;
        ng = ng >= weisspunkt ? 255 : ng * streckung;
        nb = nb >= weisspunkt ? 255 : nb * streckung;
      }

      rgba[i] = nr;
      rgba[i + 1] = ng;
      rgba[i + 2] = nb;
    }
  }

  return rgba;
}
