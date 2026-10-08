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
 * WER EINEN LINK SIGNIERT — von der Webapp beim Start gesetzt.
 *
 * Das Paket kennt keinen API-Dienst; deshalb reicht die Webapp eine Funktion
 * herein, die aus einem Pfad einen Link macht, der seine Berechtigung selbst
 * mitbringt (`POST /api/download-links`, siehe Downloadlink.php).
 *
 * ANLASS (17.09.2026): Firefox auf Android übergibt große Downloads seinem
 * Download-Manager, und der fragt neu an, ohne `Referer`. Ohne den erkennt
 * der Server keine Browser-Sitzung und antwortet 401 — eine 62-MB-APK
 * scheiterte so dreimal, bei 17 und bei 22 MB.
 */
let signierer = null;

export function setDownloadSigner(fn) {
  signierer = fn;
}

// Direkter Browser-Download per Link: der Browser streamt die Antwort direkt
// auf die Platte. Anders als der Blob-Weg wird die Datei NICHT komplett in den
// Arbeitsspeicher geladen – nötig für große Dateien (bis 1 GB). Nur für
// GET-Downloads verwenden.
//
// Mit Signierer geht der Link signiert hinaus. Scheitert das Signieren, geht
// er wie bisher: Auf dem Schreibtisch trägt die Sitzung ihn ohnehin, und ein
// Download, der vielleicht nicht fortgesetzt werden kann, ist besser als gar
// keiner.
export async function triggerDownload(url, filename) {
  let ziel = url;
  if (signierer) {
    try {
      ziel = (await signierer(url)) || url;
    } catch {
      ziel = url;
    }
  }
  const link = document.createElement('a');
  link.href = ziel;
  if (filename) link.setAttribute('download', filename);
  document.body.appendChild(link);
  link.click();
  link.remove();
}

// Stößt einen Browser-Download für einen per API geladenen Blob an.
export function downloadBlob(blob, filename) {
  const url = window.URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.setAttribute('download', filename);
  document.body.appendChild(link);
  link.click();
  link.remove();
  window.URL.revokeObjectURL(url);
}
