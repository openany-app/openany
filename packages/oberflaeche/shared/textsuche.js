/*
 * Ein Suchfeld, das findet, was der Mensch meint.
 *
 * DER FALL, UM DEN ES GEHT: Wer „muller" tippt, sucht „Müller". Ein blanker
 * `includes()` fände ihn nicht, und der Nutzer schlösse daraus, dass der
 * Kontakt nicht da ist – die schlechteste aller Antworten, weil sie falsch
 * ist und trotzdem endgültig aussieht.
 *
 * WIE: Beide Seiten werden in Kleinbuchstaben zerlegt (NFD) und um die
 * Zeichen erleichtert, die nur Betonung tragen. Aus „Müller" wird „muller",
 * aus „Ærø" wird „ærø" – das Æ ist ein eigener Buchstabe und keine Betonung,
 * deshalb bleibt es stehen. Das ist die Grenze dieses Verfahrens und
 * ausdrücklich in Ordnung: Es soll Tippfaulheit auffangen, nicht Sprachen
 * übersetzen.
 *
 * WAS ES NICHT TUT: die deutsche Ersatzschreibung (ue → ü). Wer „mueller"
 * tippt, findet „Müller" nicht. Das ließe sich ergänzen, verlangt aber eine
 * Regel je Sprache – und hier soll dieselbe Datei für de, en, es, pt und fr
 * gelten.
 */

/** Kleinbuchstaben, ohne Betonungszeichen, ohne Ränder. */
export function normal(text) {
  return String(text ?? '')
    .normalize('NFD')
    .replace(/\p{Diacritic}/gu, '')
    .toLowerCase()
    .trim();
}

/**
 * Passt `text` auf die Sucheingabe?
 *
 * Eine leere Suche passt auf alles – sonst verschwände die ganze Liste,
 * sobald jemand ins Feld klickt und wieder herausgeht.
 *
 * Mehrere Wörter müssen ALLE vorkommen, aber in beliebiger Reihenfolge:
 * „müller hannah" findet „Hannah Müller". Wer zwei Wörter tippt, will
 * einschränken, nicht eine Wortfolge suchen.
 */
export function passt(text, suche) {
  const worte = normal(suche).split(/\s+/).filter(Boolean);

  if (worte.length === 0) return true;

  const heuhaufen = normal(text);

  return worte.every((wort) => heuhaufen.includes(wort));
}
