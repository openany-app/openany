// Die Geometrie der Blatterkennung: vier gefundene Ecken ordnen und prüfen, ob
// sie als Blatt taugen.
//
// Reine Rechnerei ohne Canvas, damit sie sich an synthetischen Fällen mit
// bekanntem Ergebnis prüfen lässt; die Pixelarbeit steht daneben.

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
