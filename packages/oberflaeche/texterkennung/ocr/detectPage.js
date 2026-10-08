// Das Blatt im Foto finden: vier Ecken, oder ehrlich gesagt gar nichts.
//
// Reine Rechnerei auf einem Graustufen-Feld, ohne Canvas – damit sie ohne
// Browser pruefbar ist. Die Geometrie danach steht in pageGeometry.js.
//
// Das Verfahren ist bewusst schlicht: Ein Blatt Papier ist die grosse helle
// Flaeche im Bild. Also Schwelle nach Otsu, groesste zusammenhaengende helle
// Flaeche suchen, deren aeusserste Ecken nehmen. Kantendetektoren waeren
// eleganter, aber auch empfindlicher gegen Tischkanten, Kabel und Schatten –
// und das Risiko liegt hier nicht darin, ein Blatt zu verpassen, sondern
// darin, das FALSCHE Viereck fuer eines zu halten.
//
// Der dunkle Text auf dem Papier stoert nicht: Buchstaben sind kleine dunkle
// Inseln, das Papier flieszt drumherum und bleibt zusammenhaengend.

import { histogram, otsuThreshold } from './imageMath';
import { orderCorners } from './pageGeometry';

/**
 * Groesste zusammenhaengende helle Flaeche als Maske (4er-Nachbarschaft).
 * Iterativ mit eigenem Stapel – eine rekursive Variante sprengt bei einem
 * bildfuellenden Blatt den Aufrufstapel.
 */
function groessteFlaeche(hell, width, height) {
  const marke = new Int32Array(width * height).fill(-1);
  const stapel = new Int32Array(width * height);
  let besteMarke = -1;
  let besteGroesse = 0;
  let naechste = 0;

  for (let start = 0; start < hell.length; start += 1) {
    if (!hell[start] || marke[start] !== -1) continue;

    const m = naechste++;
    let oben = 0;
    let groesse = 0;
    stapel[oben++] = start;
    marke[start] = m;

    while (oben > 0) {
      const p = stapel[--oben];
      groesse += 1;
      const x = p % width;
      const y = (p / width) | 0;

      if (x > 0 && hell[p - 1] && marke[p - 1] === -1) { marke[p - 1] = m; stapel[oben++] = p - 1; }
      if (x < width - 1 && hell[p + 1] && marke[p + 1] === -1) { marke[p + 1] = m; stapel[oben++] = p + 1; }
      if (y > 0 && hell[p - width] && marke[p - width] === -1) { marke[p - width] = m; stapel[oben++] = p - width; }
      if (y < height - 1 && hell[p + width] && marke[p + width] === -1) { marke[p + width] = m; stapel[oben++] = p + width; }
    }

    if (groesse > besteGroesse) { besteGroesse = groesse; besteMarke = m; }
  }

  return { marke, besteMarke, besteGroesse };
}

/**
 * Vier Ecken des Blattes, in Bildkoordinaten – oder `null`.
 *
 * Der Trick fuer die Ecken: Innerhalb der Flaeche hat die obere linke Ecke die
 * kleinste Summe x+y, die untere rechte die groesste; die obere rechte die
 * groesste Differenz x-y, die untere linke die kleinste. Fuer ein Viereck,
 * das nicht voellig verdreht liegt, trifft das genau die vier Ecken – ohne
 * Konturverfolgung und ohne Polygon-Vereinfachung.
 *
 * @param {Uint8Array} gray Graustufen, Zeile fuer Zeile
 * @returns {{x:number,y:number}[]|null} vier Ecken oder null
 */
export function detectPageQuad(gray, width, height, { minAnteil = 0.25 } = {}) {
  if (width < 40 || height < 40) return null;

  const schwelle = otsuThreshold(histogram(gray));
  const hell = new Uint8Array(gray.length);
  for (let i = 0; i < gray.length; i += 1) hell[i] = gray[i] > schwelle ? 1 : 0;

  const { marke, besteMarke, besteGroesse } = groessteFlaeche(hell, width, height);
  if (besteMarke < 0 || besteGroesse < minAnteil * width * height) return null;

  let minSumme = Infinity, maxSumme = -Infinity, minDiff = Infinity, maxDiff = -Infinity;
  let ol = null, ur = null, ul = null, or_ = null;

  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < width; x += 1) {
      if (marke[y * width + x] !== besteMarke) continue;
      const s = x + y;
      const d = x - y;
      if (s < minSumme) { minSumme = s; ol = { x, y }; }
      if (s > maxSumme) { maxSumme = s; ur = { x, y }; }
      if (d > maxDiff) { maxDiff = d; or_ = { x, y }; }
      if (d < minDiff) { minDiff = d; ul = { x, y }; }
    }
  }

  if (!ol || !or_ || !ur || !ul) return null;

  return orderCorners([ol, or_, ur, ul]);
}
