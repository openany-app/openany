// Welches Anführungszeichen beim Tippen von " gesetzt wird.
//
// Im Deutschen steht das öffnende Zeichen UNTEN („), das schließende oben (“)
// – die englische Form (“ … ”) ist beides oben und sieht im deutschen Satz
// falsch aus. Welches von beiden gemeint ist, verrät nur das Zeichen davor:
// nach einem Leerzeichen oder am Absatzanfang fängt ein Zitat an, hinter einem
// Wort hört es auf.
//
// Frei von Vue, Tiptap und DOM, damit es in der node-Umgebung testbar ist
// (wie linkSuggest.js und chatText.js). Die Eingaberegel im Editor liegt in
// components/editor/DeutscheAnfuehrungszeichen.js und tut nichts weiter, als
// diese Funktion zu fragen.

export const ANFUEHRUNG_AUF = '„';
export const ANFUEHRUNG_ZU = '“';

/**
 * Zeichen, nach denen ein Zitat BEGINNT. Neben Leerraum sind das öffnende
 * Klammern (ein Zitat kann in einer Klammer stehen), Gedankenstriche und ein
 * bereits gesetztes öffnendes Zeichen – „‚Zitat im Zitat‘“ ist gültiger Satz.
 */
const BEGINNT_DANACH = /[\s([{<„‚\-–—/]/u;

/**
 * Das passende Zeichen zu dem, was unmittelbar davor steht.
 *
 * @param {string} davor Das Zeichen links vom Cursor; '' am Absatzanfang.
 * @returns {'„'|'“'}
 */
export function anfuehrungszeichenFuer(davor) {
  if (!davor || BEGINNT_DANACH.test(davor)) return ANFUEHRUNG_AUF;
  return ANFUEHRUNG_ZU;
}
