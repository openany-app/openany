/*
 * Der Absender für Briefe (notizen/PdfAuswahlDialog.vue) und der Ort vor dem
 * Datum -- einmal in den Einstellungen abgelegt, auf diesem Gerät.
 *
 * Im Browser-Speicher und nicht im Abgleich: Es ist eine Vorgabe fürs
 * Ausfüllen, kein Inhalt. Fehlt sie (anderes Gerät, Speicher geleert), füllt
 * man sie im Dialog aus, und sie ist wieder da.
 */
const SCHLUESSEL = 'openany_brief_absender';

/** @returns {{absender: string, ort: string}} */
export function absenderLesen() {
  try {
    const w = JSON.parse(localStorage.getItem(SCHLUESSEL) ?? '{}');
    return { absender: String(w.absender ?? ''), ort: String(w.ort ?? '') };
  } catch {
    return { absender: '', ort: '' };
  }
}

export function absenderMerken({ absender, ort }) {
  try {
    localStorage.setItem(SCHLUESSEL, JSON.stringify({ absender: absender ?? '', ort: ort ?? '' }));
  } catch {
    // Ohne Speicher bleibt es beim nächsten Mal leer -- kein Grund zu scheitern.
  }
}

/** Eine Postanschrift aus dem Adressbuch als Zeilen: Kommas trennen. */
export function anschriftZeilen(name, adresse) {
  const teile = String(adresse ?? '').split(/\s*[,\n]\s*/).filter(Boolean);
  return [name, ...teile].filter(Boolean).join('\n');
}
