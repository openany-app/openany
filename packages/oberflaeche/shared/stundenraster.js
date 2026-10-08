// Stundenraster: die Uhrzeiten, nach denen eine Schule ihren Tag teilt.
//
// Sie gehören dem einzelnen Plan (`week_plans.settings.periods`) und nicht
// der App: Zwei Schulen klingeln verschieden, und die Vorlagen hier sind ein
// Startpunkt, kein Gesetz. Wer eine Zeile ändert, ändert seinen Plan – die
// Vorlage bleibt, wie sie ist.
//
// Vue-frei, damit die Werte auch außerhalb einer Komponente zu prüfen sind.

/**
 * Baut ein Raster aus Startzeit, Länge und Pausen.
 *
 * Von Hand ausgeschriebene Uhrzeitpaare wären hier zwei Dutzend Zeilen, in
 * denen sich ein Zahlendreher gut versteckt.
 *
 * @param start   'HH:MM' der ersten Stunde
 * @param dauer   Minuten je Stunde
 * @param pausen  Minuten nach der n-ten Stunde (Index 0 = nach der ersten);
 *                fehlt ein Wert, gilt `pause`
 * @param pause   Vorgabe-Pause zwischen zwei Stunden
 * @param anzahl  Wie viele Stunden
 */
export function raster({ start, dauer, pause, pausen = {}, anzahl }) {
    const [h, m] = start.split(':').map(Number);
    let minute = h * 60 + m;
    const zeit = (min) => `${String(Math.floor(min / 60)).padStart(2, '0')}:${String(min % 60).padStart(2, '0')}`;

    return Array.from({ length: anzahl }, (_, i) => {
        const von = minute;
        const bis = von + dauer;
        minute = bis + (pausen[i] ?? pause);

        return { from: zeit(von), to: zeit(bis) };
    });
}

/**
 * Die mitgelieferten Vorlagen. Die Pausen sind nicht ausgedacht: Nach der
 * zweiten und der vierten Stunde liegt an den meisten Schulen die große
 * Pause, und ein Raster ohne sie verschiebt den ganzen Nachmittag.
 */
export const STUNDENRASTER = {
  // Das verbreitete 45-Minuten-Raster mit zwei großen Pausen.
  klassisch: () => raster({
    start: '08:00', dauer: 45, pause: 5, pausen: { 1: 20, 3: 20, 5: 15 }, anzahl: 8,
  }),
  // Blöcke à 60 Minuten, wie sie Ganztagsschulen oft fahren.
  blocks60: () => raster({
    start: '08:00', dauer: 60, pause: 15, pausen: { 3: 45 }, anzahl: 6,
  }),
  // Doppelstunden: weniger Fachwechsel, längere Pausen dazwischen.
  doppel: () => raster({
    start: '08:00', dauer: 90, pause: 20, pausen: { 1: 20, 2: 40 }, anzahl: 4,
  }),
};

export const RASTER_KEYS = Object.keys(STUNDENRASTER);

/** Die Nummer der Stunde, in der diese Zeit beginnt – oder null. */
export function stundeZuZeit(periods, starts_at) {
  if (!starts_at) return null;
  const i = (periods || []).findIndex((p) => p.from === starts_at.slice(0, 5));

  return i === -1 ? null : i;
}
