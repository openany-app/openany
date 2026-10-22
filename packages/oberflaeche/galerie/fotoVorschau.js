// Die Vorschau eines Fotos -- hier auf dem Gerät gemacht, nicht auf dem Server.
//
// WARUM HIER (seit Oktober 2026): Der Server dekodierte jedes Foto mit GD, um
// eine Kachel daraus zu machen -- ein 48-Megapixel-Foto brauchte dafür rund
// 200 MB Arbeitsspeicher, und über 30 Megapixel gab es gar keine. Das Gerät
// hat das Bild beim Hochladen ohnehin in der Hand, und der Browser dekodiert
// mit Hilfe der Grafikkarte. Der Server nimmt die Vorschau nur noch an und
// kodiert sie neu (MediaDeliveryService::vorschauAnnehmen).
//
// ZWEI EINGÄNGE:
//   fotoVorschau(datei)    vor dem Hochladen, aus der gewählten Datei
//   vorschauAusBild(img)   beim Ansehen, aus einem schon angezeigten <img> --
//                          für Bilder, die ohne ankamen (Projekt-Import,
//                          große Fotos in Stücken). Das Bild ist dort sicher
//                          dekodiert, weil man es sieht.
//
// SCHEITERN IST ERLAUBT. Kann der Browser ein Format nicht dekodieren (HEIC
// in Chrome und Firefox), kommt `null`, und das Bild geht ohne Vorschau
// hinauf. Wer es später in einem Browser ansieht, der es kann, reicht sie
// nach.
//
// 480 PX wie in der App (`vorschau_jpeg` in galeriebefehle.rs) und auf dem
// Server (`media.thumb_max`) -- eine Größe für alle Kacheln.

export const VORSCHAU_KANTE = 480;
const QUALITAET = 0.82;

/** Breite und Höhe der Vorschau: längste Kante höchstens 480, nie größer als das Bild. */
export function vorschauMasse(breite, hoehe, kante = VORSCHAU_KANTE) {
  if (!(breite > 0) || !(hoehe > 0)) return null;
  const faktor = Math.min(1, kante / Math.max(breite, hoehe));
  return {
    breite: Math.max(1, Math.round(breite * faktor)),
    hoehe: Math.max(1, Math.round(hoehe * faktor)),
  };
}

/** Ob aus dieser Datei überhaupt eine Vorschau werden kann. */
export function brauchtVorschau(datei) {
  const typ = String(datei?.type ?? '').toLowerCase();
  // SVG ist kein Rasterbild (und der Server zeigt es ohnehin nicht inline).
  return typ.startsWith('image/') && typ !== 'image/svg+xml';
}

function zeichnen(quelle, breite, hoehe) {
  const masse = vorschauMasse(breite, hoehe);
  if (!masse) return Promise.resolve(null);
  const canvas = document.createElement('canvas');
  canvas.width = masse.breite;
  canvas.height = masse.hoehe;
  const ctx = canvas.getContext('2d');
  if (!ctx) return Promise.resolve(null);
  ctx.drawImage(quelle, 0, 0, masse.breite, masse.hoehe);
  return new Promise((fertig) => canvas.toBlob((blob) => fertig(blob ?? null), 'image/jpeg', QUALITAET));
}

/**
 * Die Vorschau aus einer gewählten Datei, oder `null`.
 *
 * `imageOrientation: 'from-image'` dreht nach der EXIF-Angabe -- ohne läge
 * ein Hochkant-Foto vom Handy quer in der Kachel.
 *
 * NACHEINANDER aufrufen, nicht für viele Dateien gleichzeitig: Ein großes
 * Foto dekodiert belegt Hunderte MB, und Safari auf iOS beendet den ganzen
 * Tab, wenn es zu viel wird. Die Upload-Wege laden ohnehin Datei für Datei.
 */
export async function fotoVorschau(datei) {
  if (!brauchtVorschau(datei) || typeof createImageBitmap !== 'function') return null;
  let bild;
  try {
    bild = await createImageBitmap(datei, { imageOrientation: 'from-image' });
  } catch {
    return null;
  }
  try {
    return await zeichnen(bild, bild.width, bild.height);
  } catch {
    return null;
  } finally {
    bild.close?.();
  }
}

/** Die Vorschau aus einem schon geladenen <img>, oder `null`. */
export async function vorschauAusBild(img) {
  if (!img?.complete || !img.naturalWidth) return null;
  try {
    return await zeichnen(img, img.naturalWidth, img.naturalHeight);
  } catch {
    // Ein Bild fremder Herkunft „verunreinigt" das Canvas; das kommt hier
    // nicht vor (alles kommt von derselben Adresse), aber falls doch, ist
    // es kein Fehler -- dann eben keine Vorschau.
    return null;
  }
}
