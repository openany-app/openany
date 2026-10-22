/*
 * Geteilte Inhalte fürs Whiteboard (docs/plan-whiteboard.md, W2).
 *
 * Das Whiteboard bindet ein, was im Projekt geteilt ist: Bilder aus Alben
 * und Ordnern, Notizen aus Mappen, Dateien aus Ordnern. Hier steht, wie die
 * App das liest -- in EINER Form für den Einbinden-Dialog (Paket,
 * projekte/whiteboard/), gleich ob Album, Ordner oder Mappe:
 *
 *   { name, wurzel, ebenen: [{ uuid, name, eltern }],
 *     eintraege: [{ art: 'bild'|'datei'|'notiz', uuid, name, eltern, mime }] }
 *
 * Bilder kommen als Daten-URL: Excalidraw legt sie so in seine Dateien. Die
 * Vorschau reicht -- das Original bleibt in der Freigabe.
 *
 * JEDER `invoke` STEHT AUF EINER ZEILE (src-tauri/tests/verdrahtung.rs).
 */
import { invoke, convertFileSrc } from '@tauri-apps/api/core';
import { projekteQuelle as quelle } from './projekte';
import { mimeAus } from './ablage';

/** Die Freigaben des Projekts, die sich öffnen lassen. */
export async function freigaben(projekt) {
  const liste = (await quelle.mappen(projekt)).mappen ?? [];
  return liste.filter((m) => m.name).map((m) => ({ uuid: m.uuid, art: m.art, name: m.name }));
}

/** Was in einer Freigabe liegt, in der einen Form (siehe oben). */
export async function inhalt(projekt, freigabe, art) {
  if (art === 'mappe') {
    const s = await invoke('projekt_mappe_notizen', { projekt, freigabe });
    const ebenen = new Map();
    for (const n of s.notizen ?? []) {
      // Unterordner auf dem Weg anlegen, wie in geteilteMappe.js.
      let p = s.pfad;
      for (const teil of (n.folder ?? s.pfad).slice(s.pfad.length).split('/').filter(Boolean)) {
        ebenen.set(`${p}/${teil}`, { uuid: `${p}/${teil}`, name: teil, eltern: p });
        p = `${p}/${teil}`;
      }
    }
    return {
      name: s.name,
      wurzel: s.pfad,
      ebenen: [...ebenen.values()],
      eintraege: (s.notizen ?? []).map((n) => ({ art: 'notiz', uuid: n.zk_id, name: n.title || '…', eltern: n.folder ?? s.pfad })),
    };
  }
  const s = await quelle.freigabeSachen(projekt, freigabe);
  const album = art === 'album';
  return {
    name: s.name,
    wurzel: s.wurzel,
    ebenen: (album ? s.alben : s.dateien.filter((d) => d.ordner)).map((x) => ({ uuid: x.uuid, name: x.name, eltern: x.eltern })),
    eintraege: album
      ? s.bilder.map((b) => ({ art: 'bild', uuid: b.uuid, name: b.name, eltern: b.album, mime: b.mime || mimeAus(b.name) }))
      : s.dateien.filter((d) => !d.ordner).map((d) => {
        const mime = d.mime || mimeAus(d.name);
        return { art: mime.startsWith('image/') ? 'bild' : 'datei', uuid: d.uuid, name: d.name, eltern: d.eltern, mime };
      }),
  };
}

const alsDatenUrl = (blob) => new Promise((gut, schlecht) => {
  const r = new FileReader();
  r.onload = () => gut(r.result);
  r.onerror = () => schlecht(r.error);
  r.readAsDataURL(blob);
});

/** Ein Bild als Daten-URL -- die Vorschau, sonst das Original. */
export async function bild(projekt, freigabe, uuid) {
  const pfad = await quelle.freigabeHolen(projekt, freigabe, uuid, true)
    .catch(() => quelle.freigabeHolen(projekt, freigabe, uuid, false));
  const blob = await (await fetch(convertFileSrc(pfad))).blob();
  return { dataURL: await alsDatenUrl(blob), mimeType: blob.type || 'image/jpeg' };
}
