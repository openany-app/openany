// Einen einzelnen Eintrag aus einem ZIP-Archiv lesen – ohne Bibliothek.
//
// Wozu: Word- (.docx) und OpenDocument-Dateien (.odt) sind ZIP-Archive mit
// XML darin. Für eine KI-Anfrage braucht es genau eine Datei daraus. Dafür
// eine Bibliothek ins Bundle zu nehmen, hieße, für zwei Funktionen ein Paket
// zu pflegen; der Browser kann das Entpacken selbst (`DecompressionStream`).
//
// Was bewusst fehlt: ZIP64 (Archive über 4 GB), Verschlüsselung, mehrteilige
// Archive. Nichts davon kommt bei einer Word-Datei vor.
//
// Der Aufbau, soweit er hier gebraucht wird: Am Ende des Archivs steht ein
// Verzeichnis aller Einträge. Jeder Eintrag darin nennt Namen, Größe,
// Kompressionsart und die Stelle, an der seine Daten beginnen.

const ENDE_SIGNATUR = 0x06054b50;
const VERZEICHNIS_SIGNATUR = 0x02014b50;

// Schutz gegen „ZIP-Bomben": wenige Kilobyte, die zu Gigabytes entpacken.
// Der Text einer Word-Datei ist weit darunter.
const MAX_ENTPACKT = 50 * 1024 * 1024;

export class ZipFehler extends Error {}

/**
 * @param {ArrayBuffer} buffer
 * @param {string} name z. B. 'word/document.xml'
 * @returns {Promise<Uint8Array|null>} null, wenn es den Eintrag nicht gibt
 */
export async function zipEintrag(buffer, name) {
  const bytes = new Uint8Array(buffer);
  const view = new DataView(buffer);
  const ende = findeEnde(view, bytes.length);

  const anzahl = view.getUint16(ende + 10, true);
  let pos = view.getUint32(ende + 16, true);
  const decoder = new TextDecoder();

  for (let n = 0; n < anzahl; n += 1) {
    if (pos + 46 > bytes.length || view.getUint32(pos, true) !== VERZEICHNIS_SIGNATUR) {
      throw new ZipFehler('Verzeichnis beschädigt');
    }

    const methode = view.getUint16(pos + 10, true);
    const komprimiert = view.getUint32(pos + 20, true);
    const entpackt = view.getUint32(pos + 24, true);
    const nameLaenge = view.getUint16(pos + 28, true);
    const extraLaenge = view.getUint16(pos + 30, true);
    const kommentarLaenge = view.getUint16(pos + 32, true);
    const lokal = view.getUint32(pos + 42, true);
    const eintrag = decoder.decode(bytes.subarray(pos + 46, pos + 46 + nameLaenge));

    if (eintrag === name) {
      if (entpackt > MAX_ENTPACKT) throw new ZipFehler('Eintrag zu groß');

      // Die Längen im lokalen Kopf können von denen im Verzeichnis abweichen
      // (das Extra-Feld zumal) – deshalb dort nachlesen, wo die Daten beginnen.
      const start = lokal + 30 + view.getUint16(lokal + 26, true) + view.getUint16(lokal + 28, true);
      const daten = bytes.subarray(start, start + komprimiert);

      if (methode === 0) return daten;
      if (methode === 8) return entpacken(daten);
      throw new ZipFehler(`Kompressionsart ${methode} wird nicht unterstützt`);
    }

    pos += 46 + nameLaenge + extraLaenge + kommentarLaenge;
  }

  return null;
}

// Das Verzeichnisende liegt in den letzten 22 Byte – plus höchstens einem
// Kommentar von 65535 Byte davor.
function findeEnde(view, laenge) {
  for (let i = laenge - 22; i >= Math.max(0, laenge - 22 - 65535); i -= 1) {
    if (view.getUint32(i, true) === ENDE_SIGNATUR) return i;
  }
  throw new ZipFehler('Kein ZIP-Archiv');
}

async function entpacken(daten) {
  const strom = new Blob([daten]).stream().pipeThrough(new DecompressionStream('deflate-raw'));
  const ergebnis = new Uint8Array(await new Response(strom).arrayBuffer());
  if (ergebnis.length > MAX_ENTPACKT) throw new ZipFehler('Eintrag zu groß');
  return ergebnis;
}
