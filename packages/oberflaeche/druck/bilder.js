// Bilder aus den Notiz-Anhängen für ein PDF holen -- gemeinsam für das
// Notiz-PDF und die Druckfassung.

/** So groß dürfen Bilder im PDF höchstens werden (längste Kante, Pixel). */
const BILD_KANTE = 1600;
const HOECHSTENS_HOCH_CM = 12;

/**
 * Wie breit ein Bild auf dem Papier wird: seine natürliche Größe bei 150 dpi,
 * höchstens Satzbreite und höchstens 12 cm hoch -- ein quadratisches Foto
 * über die volle Breite füllte sonst eine ganze Seite.
 */
export function breiteAufPapier(breitePx, hoehePx, satzbreiteCm = 16.4) {
  const natuerlich = (breitePx / 150) * 2.54;
  return Math.min(natuerlich, satzbreiteCm, HOECHSTENS_HOCH_CM * (breitePx / hoehePx));
}

/**
 * Ein Bild für das PDF holen: neu gezeichnet als JPEG, nach der EXIF-Angabe
 * gedreht, höchstens 1600 px. Das gleicht auch Formate aus, die Typst nicht
 * liest (HEIC; die Typst-Endung muss ohnehin zum Inhalt passen), und hält das
 * PDF klein. `null`, wenn das Bild nicht zu holen oder zu lesen ist.
 */
async function bildVorbereiten(url, satzbreiteCm) {
  try {
    const antwort = await fetch(url, { credentials: 'same-origin' });
    if (!antwort.ok) return null;
    const bild = await createImageBitmap(await antwort.blob(), { imageOrientation: 'from-image' });
    const faktor = Math.min(1, BILD_KANTE / Math.max(bild.width, bild.height));
    const canvas = document.createElement('canvas');
    canvas.width = Math.max(1, Math.round(bild.width * faktor));
    canvas.height = Math.max(1, Math.round(bild.height * faktor));
    const ctx = canvas.getContext('2d');
    // Transparenz (PNG) wird weiß -- JPEG kennt keine.
    ctx.fillStyle = '#fff';
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.drawImage(bild, 0, 0, canvas.width, canvas.height);
    const breiteCm = breiteAufPapier(bild.width, bild.height, satzbreiteCm);
    bild.close?.();
    const blob = await new Promise((fertig) => canvas.toBlob(fertig, 'image/jpeg', 0.85));
    if (!blob) return null;
    return { bytes: new Uint8Array(await blob.arrayBuffer()), breiteCm };
  } catch {
    return null;
  }
}

/**
 * Bildpfade einer Mappe -> Dateien fürs Typst-Projekt.
 *
 * @param {object} quelle   eine Notiz-Quelle aus notesSources.js
 * @param {number|null} ordner  die Mappe, deren Anhänge gelten
 * @param {string[]} pfade  relative Bildpfade aus den Notizen
 * @param {number} [satzbreiteCm]
 * @param {string} [vorsilbe]  vor den Dateinamen -- wer mehrere Mappen in ein
 *        Dokument setzt, braucht verschiedene („m12-bild-1.jpg")
 * @returns {Promise<{zuordnung: Map<string, {datei: string, breiteCm: number}>, dateien: Map<string, Uint8Array>}>}
 */
export async function bilderHolen(quelle, ordner, pfade, satzbreiteCm, vorsilbe = '') {
  const zuordnung = new Map();
  const dateien = new Map();
  if (!pfade.length) return { zuordnung, dateien };

  let anhaenge = new Map();
  try {
    const { data } = await quelle.getNoteAssets(ordner);
    anhaenge = new Map((data.items ?? []).map((a) => [a.path, a]));
  } catch {
    // Ohne Anhänge steht statt jedes Bildes sein Alt-Text.
  }

  let n = 0;
  for (const pfad of pfade) {
    const anhang = anhaenge.get(pfad);
    if (!anhang || anhang.target_type !== 'media') continue;
    // Das Original, nicht die 480-px-Vorschau: Gedruckt ist das zu grob.
    const url = quelle.assetUrl(anhang, ordner).replace(/\?thumb=1$/, '');
    const vorbereitet = await bildVorbereiten(url, satzbreiteCm);
    if (!vorbereitet) continue;
    const name = `${vorsilbe}bild-${++n}.jpg`;
    dateien.set(name, vorbereitet.bytes);
    zuordnung.set(pfad, { datei: name, breiteCm: vorbereitet.breiteCm });
  }
  return { zuordnung, dateien };
}
