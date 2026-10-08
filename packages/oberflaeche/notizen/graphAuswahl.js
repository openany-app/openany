/*
 * Was die Graphenansicht zeigt (NotesGraph.vue) -- ohne Vue, damit es sich
 * prüfen lässt. Entschieden mit Tiffy am 30.09.2026:
 *
 * - KEINE EBENE GEWÄHLT: Die Leiste nennt alle Tags. Gezeigt werden die
 *   Notizen, die einen gewählten Tag tragen -- ein Tag ist dann der Einstieg.
 *   Ohne Tag und ohne Ebene: nichts (die Ansicht sagt, was zu tun ist).
 * - EBENEN GEWÄHLT: Gezeigt werden deren Notizen. Die Leiste nennt nur Tags,
 *   die in diesen Notizen vorkommen, mit deren Zahl.
 * - NUR EINE EBENE: Es gibt nichts zu wählen, sie steht gleich da.
 */

/** Schlüssel der Ebene einer Notiz: ihre oberste Mappe, sonst 'root'. */
export const ebeneVon = (n) => (n.group == null ? 'root' : String(n.group));

/** Sind Ebenen im Spiel (mehr als eine, und mindestens eine gewählt)? */
export const nachEbenen = (gewaehlteEbenen, anzahlEbenen) => anzahlEbenen <= 1 || gewaehlteEbenen.size > 0;

/** Die gezeigten Notizen. */
export function gezeigteNotizen(notizen, gewaehlteEbenen, anzahlEbenen, tagKanten = [], gewaehlteTags = new Set()) {
  if (anzahlEbenen <= 1) return notizen;
  if (gewaehlteEbenen.size > 0) return notizen.filter((n) => gewaehlteEbenen.has(ebeneVon(n)));
  if (!gewaehlteTags.size) return [];
  const mitTag = new Set(tagKanten.filter((e) => gewaehlteTags.has(e.to.tag)).map((e) => e.from));
  return notizen.filter((n) => mitTag.has(n));
}

/**
 * Die Tags der Leiste: Tag -> Zahl. Nach Ebenen nur die der gezeigten
 * Notizen, sonst alle mit ihrer ganzen Zahl.
 */
export function leistenTags(tags, tagKanten, gezeigt, ebenenImSpiel) {
  if (!ebenenImSpiel) return new Map(tags.map((tg) => [tg.tag, tg.count]));
  const zaehler = new Map();
  for (const e of tagKanten) {
    if (gezeigt.has(e.from)) zaehler.set(e.to.tag, (zaehler.get(e.to.tag) || 0) + 1);
  }
  return zaehler;
}
