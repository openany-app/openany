/*
 * Welche fremden Pakete dieses Paket importiert.
 *
 * WARUM ES DIESE LISTE GIBT: Das Paket hat keine eigenen node_modules. Die
 * Anwendung, die es einbindet (frontend/, app/), bringt die Pakete mit, und
 * Vite muss sie ausdrücklich von DORT auflösen (`resolve.dedupe`). Fehlt ein
 * Name hier, bricht der Build mit „failed to resolve import" — oder, bei Vue,
 * es entsteht eine zweite Instanz, und gemeinsame refs greifen nicht.
 *
 * Beide vite.config.js lesen diese Liste. `fremde-pakete.test.js` prüft, dass
 * jeder Import im Paket hier steht — ein neuer Import ohne Eintrag fällt im
 * Test auf, nicht erst beim Bauen der anderen Anwendung.
 *
 * Die ANWENDUNG muss jedes davon in ihrer package.json haben.
 */
export const FREMDE_PAKETE = [
  'vue',
  'vue-i18n',
  'lucide-vue-next',
  '@tiptap/core',
  '@tiptap/extension-image',
  '@tiptap/extension-link',
  '@tiptap/extension-list',
  '@tiptap/extension-table',
  '@tiptap/starter-kit',
  '@tiptap/vue-3',
  'tiptap-markdown',
  // Die Texterkennung (texterkennung/, seit 26.09.2026 im Paket): Tesseract
  // als WebAssembly, pdf.js zum Aufschlagen, jsPDF und pdf-lib zum Bauen.
  'tesseract.js',
  'pdfjs-dist',
  'jspdf',
  'pdf-lib',
  // Die Projekt-Ansichten (projekte/, seit 01.10.2026 im Paket): Karte der
  // Orte und Ziehen von Karten und Spalten.
  'leaflet',
  'vuedraggable',
];

/** Nur für die Tests des Pakets, nicht für den Build. */
export const NUR_TESTS = ['vitest'];
