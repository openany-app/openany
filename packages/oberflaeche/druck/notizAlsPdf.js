// Eine Notiz als PDF -- auf dem Gerät gesetzt (typstSatz.js).
//
// Kommt nur über einen dynamischen Import herein (notesSources.js): Wer nie
// ein PDF macht, lädt weder diesen Code noch den Satz-Kern.
//
// Seit 15.10.2026 im gemeinsamen Paket: Webapp und App setzen dasselbe PDF.
// Die App bringt dafür die Typst-Pakete selbst mit (fremde-pakete.js).
import { markdownZuTypst, bildpfade, ohneTitelzeile } from './markdownZuTypst';
import { bilderHolen } from './bilder';
import { vorlage } from './notizVorlage';
import { setzen } from './typstSatz';
import { quellenHolen, bibBeilegen } from './quellenZuBib';

/** Datum und Uhrzeit wie im Rest der Oberfläche, in der Sprache der Seite. */
function standText(zeitpunkt, sprache) {
  const d = new Date(zeitpunkt);
  if (Number.isNaN(d.getTime())) return '';
  return new Intl.DateTimeFormat(sprache, { dateStyle: 'medium', timeStyle: 'short' }).format(d);
}

/**
 * @param {object} quelle  eine Notiz-Quelle aus notesSources.js
 * @param {number} id      die Notiz
 * @returns {Promise<{data: Blob}>} wie die frühere Server-Antwort
 */
export async function notizAlsPdf(quelle, id) {
  const { data: notiz } = await quelle.getNote(id);
  const inhalt = notiz.content ?? '';
  const ordner = notiz.note_folder_id ?? null;

  // Bilder: relative Pfade aus der Notiz -> Anhänge der Mappe -> Bytes.
  const { zuordnung, dateien } = await bilderHolen(quelle, ordner, bildpfade(inhalt), 16.4);

  // Zitate aus der Quellenliste ihrer Mappe -- wie in der Druckfassung.
  const { bib, schluessel } = await quellenHolen(quelle, ordner);
  const zitiert = new Set();

  const sprache = document.documentElement.lang || navigator.language || 'de';
  const titel = notiz.title || 'Notiz';
  const rumpf = markdownZuTypst(ohneTitelzeile(inhalt, titel), {
    bild: (src) => zuordnung.get(src) ?? null,
    quellen: schluessel,
    zitiert,
  });
  const quelltext = vorlage({
    titel,
    stand: standText(notiz.updated_at, sprache),
    rumpf,
    sprache,
    literatur: bibBeilegen(dateien, bib, zitiert, sprache),
  });

  const pdf = await setzen(quelltext, dateien);
  return { data: new Blob([pdf], { type: 'application/pdf' }) };
}
