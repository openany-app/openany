// Die Geometrie der Entzerrung: aus vier Ecken eines schief fotografierten
// Blattes ein sauberes Rechteck rechnen.
//
// Alles hier ist reine Rechnerei ohne Canvas – die Pixelarbeit steht daneben.
// Das ist kein Ordnungsfimmel: Eine Projektivtransformation, die um eine Ecke
// verrutscht, sieht auf einem Foto genauso aus wie eine richtige. Nur an
// synthetischen Faellen mit bekanntem Ergebnis faellt so etwas auf.
//
// Warum das den bisherigen Deskew ersetzt: Eine Drehung ist ein Sonderfall
// dieser Abbildung. Wer die vier Ecken kennt, richtet gerade UND schneidet zu
// UND entzerrt in einem Schritt. Der Deskew bleibt nur als Rueckfallebene fuer
// den Fall, dass keine Ecken zu finden sind.

/**
 * Ecken in eine feste Reihenfolge bringen: oben-links, oben-rechts,
 * unten-rechts, unten-links.
 *
 * Der Trick mit den Summen und Differenzen: Oben-links hat die kleinste Summe
 * x+y, unten-rechts die groesste; oben-rechts die groesste Differenz x-y,
 * unten-links die kleinste. Das gilt fuer jedes halbwegs achsennahe Viereck
 * und kommt ohne Winkelrechnung aus.
 */
export function orderCorners(punkte) {
  if (punkte.length !== 4) throw new Error('Vier Ecken erwartet.');

  const summe = punkte.map((p) => p.x + p.y);
  const diff = punkte.map((p) => p.x - p.y);
  const kleinstes = (a) => a.indexOf(Math.min(...a));
  const groesstes = (a) => a.indexOf(Math.max(...a));

  return [
    punkte[kleinstes(summe)],   // oben links
    punkte[groesstes(diff)],    // oben rechts
    punkte[groesstes(summe)],   // unten rechts
    punkte[kleinstes(diff)],    // unten links
  ];
}

/**
 * Loest das Gleichungssystem A·x = b nach x (Gauss mit Spaltenpivotierung).
 * Gibt `null` zurueck, wenn das System entartet ist – vier Ecken, die auf
 * einer Linie liegen, haben keine Loesung, und die will man als solche
 * erkennen und nicht als Zahlensalat weiterreichen.
 */
function loese(A, b) {
  const n = b.length;
  const M = A.map((zeile, i) => [...zeile, b[i]]);

  for (let s = 0; s < n; s += 1) {
    let pivot = s;
    for (let i = s + 1; i < n; i += 1) {
      if (Math.abs(M[i][s]) > Math.abs(M[pivot][s])) pivot = i;
    }
    if (Math.abs(M[pivot][s]) < 1e-10) return null;
    [M[s], M[pivot]] = [M[pivot], M[s]];

    for (let i = s + 1; i < n; i += 1) {
      const f = M[i][s] / M[s][s];
      for (let j = s; j <= n; j += 1) M[i][j] -= f * M[s][j];
    }
  }

  const x = new Array(n).fill(0);
  for (let i = n - 1; i >= 0; i -= 1) {
    let summe = M[i][n];
    for (let j = i + 1; j < n; j += 1) summe -= M[i][j] * x[j];
    x[i] = summe / M[i][i];
  }
  return x;
}

/**
 * Projektivtransformation, die `von` (4 Punkte) auf `nach` (4 Punkte) abbildet.
 *
 * Rueckgabe sind acht Zahlen h0..h7; die neunte ist per Normierung 1. Fuer
 * das Entzerren wird sie in der Richtung ZIEL -> QUELLE gebraucht: Man laeuft
 * ueber die Pixel des fertigen Rechtecks und fragt, wo sie im Foto herkommen.
 * Andersherum blieben Loecher.
 *
 * @returns {number[]|null} null, wenn die Punkte entartet liegen
 */
export function solveHomography(von, nach) {
  const A = [];
  const b = [];

  for (let i = 0; i < 4; i += 1) {
    const { x, y } = von[i];
    const { x: u, y: v } = nach[i];
    A.push([x, y, 1, 0, 0, 0, -x * u, -y * u]);
    b.push(u);
    A.push([0, 0, 0, x, y, 1, -x * v, -y * v]);
    b.push(v);
  }

  return loese(A, b);
}

/** Einen Punkt durch die Transformation schicken. */
export function applyHomography(h, x, y) {
  const nenner = h[6] * x + h[7] * y + 1;
  return {
    x: (h[0] * x + h[1] * y + h[2]) / nenner,
    y: (h[3] * x + h[4] * y + h[5]) / nenner,
  };
}

const abstand = (a, b) => Math.hypot(a.x - b.x, a.y - b.y);

/**
 * Groesse des entzerrten Rechtecks.
 *
 * Sauber waere das nur mit der Brennweite der Kamera zu rechnen. Praktisch
 * nimmt man je Achse die LAENGERE der beiden gegenueberliegenden Kanten: Die
 * kuerzere ist die weiter entfernte und perspektivisch gestaucht – nach ihr zu
 * gehen hiesse, den entfernten Teil des Blattes dauerhaft kleiner zu lassen.
 */
export function targetSize(ecken) {
  const [ol, or_, ur, ul] = ecken;
  return {
    width: Math.round(Math.max(abstand(ol, or_), abstand(ul, ur))),
    height: Math.round(Math.max(abstand(ol, ul), abstand(or_, ur))),
  };
}

/**
 * Taugt dieses Viereck als Blatt?
 *
 * Diese Pruefung ist die wichtigste Zeile Sicherheit im ganzen Vorhaben. Das
 * Originalfoto wird nach der Umwandlung nicht aufgehoben – ein Viereck, das
 * daneben liegt, schneidet unwiederbringlich einen Teil des Dokuments ab.
 * Deshalb im Zweifel ABLEHNEN: Ein Foto mit Rand ist ein kleiner Schoenheits-
 * fehler, ein Foto mit abgeschnittener Zeile ist ein Datenverlust.
 */
export function plausibleQuad(ecken, breite, hoehe, {
  minFlaeche = 0.25,   // weniger als ein Viertel des Bildes: eher ein Logo als ein Blatt
  maxFlaeche = 0.995,  // praktisch das ganze Bild: dann gibt es nichts zu schneiden
  minWinkel = 55,      // eine Blattecke ist rechtwinklig; so schief kann keine Aufnahme sein
  maxWinkel = 125,
} = {}) {
  if (!ecken || ecken.length !== 4) return false;
  if (ecken.some((p) => !Number.isFinite(p.x) || !Number.isFinite(p.y))) return false;

  // Flaeche nach der Trapezformel (Gauss).
  let flaeche = 0;
  for (let i = 0; i < 4; i += 1) {
    const a = ecken[i];
    const b = ecken[(i + 1) % 4];
    flaeche += a.x * b.y - b.x * a.y;
  }
  flaeche = Math.abs(flaeche) / 2;
  const anteil = flaeche / (breite * hoehe);
  if (anteil < minFlaeche || anteil > maxFlaeche) return false;

  // Konvex UND im Uhrzeigersinn geordnet: Alle Kreuzprodukte gleiches
  // Vorzeichen. Eine eingedrueckte Ecke bedeutet, dass die Erkennung etwas
  // anderes als ein Blatt gefunden hat.
  let vorzeichen = 0;
  for (let i = 0; i < 4; i += 1) {
    const a = ecken[i];
    const b = ecken[(i + 1) % 4];
    const c = ecken[(i + 2) % 4];
    const kreuz = (b.x - a.x) * (c.y - b.y) - (b.y - a.y) * (c.x - b.x);
    if (kreuz === 0) return false;
    const v = Math.sign(kreuz);
    if (vorzeichen === 0) vorzeichen = v;
    else if (v !== vorzeichen) return false;

    // Innenwinkel an Ecke b.
    const w = Math.abs((Math.atan2(a.y - b.y, a.x - b.x) - Math.atan2(c.y - b.y, c.x - b.x)) * 180 / Math.PI);
    const winkel = w > 180 ? 360 - w : w;
    if (winkel < minWinkel || winkel > maxWinkel) return false;
  }

  return true;
}
