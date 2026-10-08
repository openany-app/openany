/*
 * Farben und Stärken der PDF-Werkzeuge (PdfWerkzeuge.vue), an einer Stelle,
 * weil auch der PdfBetrachter die Vorgaben braucht.
 *
 * Dieselben Töne wie in Firefox: Markieren hell und durchscheinend, Stift
 * und Text kräftig. Das erste Element ist jeweils die Vorgabe.
 */
export const FARBEN = {
  markieren: ['#FFFF98', '#53FFBC', '#80EBFF', '#FFCBE6', '#FF4F5F'],
  stift: ['#000000', '#E11D48', '#2563EB', '#16A34A', '#F59E0B'],
  text: ['#000000', '#E11D48', '#2563EB', '#16A34A'],
};

// Stift: Strichstärke; Text: Schriftgröße. Vorgabe ist die mittlere.
export const STAERKEN = {
  stift: [2, 4, 8],
  text: [10, 14, 20],
};

// Für pdf.js: „name=#farbe,…" -- die Farben, die eine Markierung haben kann.
// Die Namen sind die von pdf.js (es sucht damit seine eigenen Beschriftungen).
export const MARKIER_FARBEN_PDFJS = ['yellow', 'green', 'blue', 'pink', 'red']
  .map((name, i) => `${name}=${FARBEN.markieren[i]}`)
  .join(',');
