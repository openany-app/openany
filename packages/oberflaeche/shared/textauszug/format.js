// Welche Dateien die Suche im Inhalt lesen kann. Nach Typ, sonst nach
// Endung -- Geräte melden den Typ nicht immer gleich (die App kennt manchmal
// nur den Namen).

const NACH_TYP = {
  'application/pdf': 'pdf',
  'application/vnd.openxmlformats-officedocument.wordprocessingml.document': 'docx',
  'application/vnd.oasis.opendocument.text': 'odt',
  'text/plain': 'text',
  'text/csv': 'text',
  'text/markdown': 'text',
};

const NACH_ENDUNG = {
  pdf: 'pdf', docx: 'docx', odt: 'odt', txt: 'text', csv: 'text', md: 'text', markdown: 'text',
};

/** Größer wird nicht gelesen -- das hieße, sie ganz über die Leitung zu holen. */
export const GROESSTE = 50 * 1024 * 1024;

/** @returns {'pdf'|'docx'|'odt'|'text'|null} */
export function formatVon(name, mime) {
  const typ = String(mime ?? '').toLowerCase().split(';')[0].trim();
  if (NACH_TYP[typ]) return NACH_TYP[typ];
  const endung = String(name ?? '').toLowerCase().match(/\.([a-z0-9]+)$/)?.[1];
  return NACH_ENDUNG[endung] ?? null;
}
