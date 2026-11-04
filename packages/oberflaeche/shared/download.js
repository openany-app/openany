// Macht aus einem beliebigen Titel/Namen einen sicheren Dateinamen: entfernt
// Pfad-/Steuerzeichen, kürzt Whitespace und begrenzt die Länge. Fällt auf
// `fallback` zurück, wenn nichts Brauchbares übrig bleibt.
export function sanitizeFilename(name, fallback) {
  // Die Steuerzeichen sind hier der Zweck, nicht ein Versehen: Ein Dateiname
  // mit \x00 oder einem Zeilenumbruch darin ist genau das, was hier weg soll.
  // eslint-disable-next-line no-control-regex
  const clean = (name || '').trim().replace(/[/\\?%*:|"<>\x00-\x1f]/g, '').replace(/\s+/g, ' ').slice(0, 120);
  return clean || fallback;
}

/*
 * WOHIN EIN BLOB GEHT — von der App beim Start gesetzt.
 *
 * In der Webansicht der App führt ein `<a download>` ins Leere (Android) bzw.
 * nicht in einen Speichern-Dialog. Die App reicht deshalb eine Funktion
 * herein, die die Datei selbst ablegt (Android: „Download", Schreibtisch:
 * Speichern-Dialog). Ohne sie bleibt es beim Browser-Download.
 */
let ablage = null;

export function setBlobAblage(fn) {
  ablage = fn;
}

// Stößt einen Browser-Download für einen per API geladenen Blob an. Mit
// gesetzter Ablage (App) geht er dorthin; die Rückgabe ist dann ein Promise.
export function downloadBlob(blob, filename) {
  if (ablage) return ablage(blob, filename);
  const url = window.URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.setAttribute('download', filename);
  document.body.appendChild(link);
  link.click();
  link.remove();
  window.URL.revokeObjectURL(url);
}
