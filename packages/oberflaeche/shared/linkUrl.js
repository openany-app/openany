/**
 * Ergänzt ein fehlendes Schema in einer eingegebenen Link-Adresse.
 *
 * Der Vorgänger war ein `window.prompt`, das mit 'https://' VORBELEGT war.
 * Wer eine kopierte Adresse einfügte, bekam sie doppelt
 * ('https://https://example.de') – und musste das Präfix jedes Mal von Hand
 * löschen. Das Feld startet jetzt leer, und was fehlt, ergänzt diese
 * Funktion beim Speichern.
 *
 * Unangetastet bleiben:
 *   - alles mit eigenem Schema (https:, mailto:, tel:, …)
 *   - absolute Pfade (/…) – so liegen eingefügte Anhänge in Notizen vor
 *   - Anker (#…)
 */
export function normalizeLinkUrl(wert) {
  const v = String(wert ?? '').trim();
  if (!v) return '';

  // Schema, absoluter Pfad oder Anker: unverändert übernehmen.
  if (/^[a-z][a-z0-9+.-]*:/i.test(v) || v.startsWith('/') || v.startsWith('#')) return v;

  return `https://${v}`;
}
