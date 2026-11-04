/*
 * Der Inhaltsleser: liest im Hintergrund den Text der Dateien aus, damit die
 * Suche auch im Inhalt findet.
 *
 * WARUM HIER UND NICHT AUF DEM SERVER: siehe textauszug/textAuszug.js. Die
 * App legt den Text in ihrem eigenen Speicher ab.
 *
 * Die Quelle sagt, was offen ist (`textOffen(zone)` → { offen, files }) und
 * nimmt den Text (`textSetzen(id, abdruck, stand, text)`); die Bytes kommen
 * über `downloadFileContent(id)`. Eine Datei nach der anderen, mit Pause --
 * der Leser soll nie spürbar bremsen. Bricht die Leitung ab, hört er auf;
 * beim nächsten Öffnen geht es weiter.
 */
import { ref } from 'vue';
import { formatVon, GROESSTE } from '../shared/textauszug/format';

const PAUSE_MS = 150;

/**
 * Die Fassung dieses Lesers. Hochzählen, wenn ein Fehler behoben ist, an dem
 * Dateien als `unlesbar` hängen blieben -- dann versucht er sie noch einmal.
 *   2  pdf.js 6: `destroy()` am Dokument gibt es nicht mehr
 */
export const LESER_VERSION = 2;
const warten = (ms) => new Promise((r) => setTimeout(r, ms));

export function inhaltsleser(quelle, zone) {
  const offen = ref(0);
  const laeuft = ref(false);
  let angehalten = false;

  const kann = typeof quelle.textOffen === 'function'
    && typeof quelle.textSetzen === 'function'
    && typeof quelle.downloadFileContent === 'function';

  async function eine(datei) {
    const format = formatVon(datei.name, datei.mime);
    const setzen = (stand, text = null) => quelle.textSetzen(datei.id, datei.abdruck, stand, text, LESER_VERSION);
    if (!format) return setzen('format');
    if (datei.bytes > GROESSTE) return setzen('zuGross');
    const res = await quelle.downloadFileContent(datei.id);
    const { textFuerSuche } = await import('../shared/textauszug/textAuszug');
    try {
      const text = await textFuerSuche(format, res.data);
      return setzen('ok', text);
    } catch (e) {
      if (e?.grund) return setzen(e.grund);
      throw e;
    }
  }

  async function starten() {
    if (!kann || laeuft.value) return;
    laeuft.value = true;
    angehalten = false;
    try {
      while (!angehalten) {
        const { offen: rest, files } = await quelle.textOffen(zone, LESER_VERSION);
        offen.value = rest ?? 0;
        if (!files?.length) break;
        let geschafft = 0;
        for (const datei of files) {
          if (angehalten) break;
          try {
            await eine(datei);
            geschafft += 1;
            offen.value = Math.max(0, offen.value - 1);
          } catch {
            // Eine Datei, die sich nicht holen ließ (Netz, gerade gelöscht):
            // beim nächsten Mal wieder.
          }
          await warten(PAUSE_MS);
        }
        // Nichts ging -- dann ist es die Leitung, nicht die Datei.
        if (geschafft === 0) break;
      }
    } catch {
      // Die Liste kam nicht -- später wieder.
    } finally {
      laeuft.value = false;
    }
  }

  const anhalten = () => { angehalten = true; };

  return { kann, offen, laeuft, starten, anhalten };
}
