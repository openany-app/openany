// Eine Notiz als Brief -- auf dem Gerät gesetzt (typstSatz.js), mit
// Anschriftfeld und Absender nach DIN 5008 (briefVorlage.js).
//
// Wie notizAlsPdf.js nur über einen dynamischen Import: Wer nie einen Brief
// schreibt, lädt weder diesen Code noch den Satz-Kern.
import { markdownZuTypst, bildpfade, ohneTitelzeile } from './markdownZuTypst';
import { bilderHolen } from './bilder';
import { briefVorlage } from './briefVorlage';
import { setzen } from './typstSatz';

/**
 * @param {object} quelle  eine Notiz-Quelle
 * @param {number} id      die Notiz
 * @param {{empfaenger: string, absender: string, ortDatum: string}} angaben
 * @returns {Promise<{data: Blob}>}
 */
export async function briefAlsPdf(quelle, id, angaben) {
  const { data: notiz } = await quelle.getNote(id);
  const inhalt = notiz.content ?? '';

  // Bilder wie in der Notiz -- schmaler, der Satzspiegel ist 16,5 cm breit.
  const { zuordnung, dateien } = await bilderHolen(quelle, notiz.note_folder_id ?? null, bildpfade(inhalt), 16.5);

  const titel = notiz.title || '';
  const rumpf = markdownZuTypst(ohneTitelzeile(inhalt, titel), {
    bild: (src) => zuordnung.get(src) ?? null,
  });
  const quelltext = briefVorlage({
    titel,
    empfaenger: angaben.empfaenger,
    absender: angaben.absender,
    ortDatum: angaben.ortDatum,
    rumpf,
    sprache: document.documentElement.lang || navigator.language || 'de',
  });

  const pdf = await setzen(quelltext, dateien);
  return { data: new Blob([pdf], { type: 'application/pdf' }) };
}
