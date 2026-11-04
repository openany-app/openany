/*
 * Kalender unter „+ Teilen": eigene Kalender, die in
 * diesem Hub noch nicht geteilt sind, und das Teilen selbst
 * (projektkalender.rs -- vor Ort immer, mit Wolke auch über openany.de).
 * Geteilte Kalender stehen in der Freigaben-Liste wie alles andere; ein
 * Antippen öffnet sie in einem Rahmen (KalenderRahmen.vue). Die Listen
 * hören auf GETEILT.
 */
import { invoke } from '@tauri-apps/api/core';

// Ein Ereignis für jede Änderung unter Freigaben (Teilen, Beenden) -- die
// Liste lädt neu, der Zähler am Reiter frischt auf.
export const GETEILT = 'openany:kalender-geteilt';

/**
 * Die geteilten Kalender eines Hubs, in der Form der Freigaben-Liste:
 * { art: 'kalender', uuid, kalender, name, color, eigen, von_name }.
 */
export async function geteilteKalender(projekt) {
  const k = await invoke('projekt_kalender', { projekt }).catch(() => ({ kalender: [] }));
  return (k.kalender ?? []).map((x) => ({
    art: 'kalender', uuid: x.uuid, kalender: x.kalender, name: x.name, color: x.color, eigen: x.eigen, von_name: x.von_name, stufe: x.stufe,
  }));
}

/** Teilen beenden -- nach Rückfrage. Antwort: ob beendet wurde. */
export async function kalenderBeenden(projekt, k, confirmDialog, t) {
  const frage = t('planning.kalender.beendenFrage', { name: k.name });
  if (!(await confirmDialog(frage, { title: t('planning.kalender.beenden'), confirmLabel: t('planning.kalender.beenden') }))) return false;
  await invoke('projekt_kalender_beenden', { projekt, freigabe: k.uuid });
  window.dispatchEvent(new Event(GETEILT));
  return true;
}

/**
 * Eigene Kalender, die hier noch nicht geteilt sind: [{ id, name }].
 * `ohneAbos`: in Hubs auf openany.de -- Abos teilen sich vorerst nur vor Ort.
 */
export async function teilbareKalender(projekt, { ohneAbos = false } = {}) {
  const [eigene, geteilt] = await Promise.all([
    invoke('kalenderbuch_kalender').catch(() => []),
    invoke('projekt_kalender', { projekt }).catch(() => ({ kalender: [] })),
  ]);
  const schon = new Set((geteilt.kalender ?? []).filter((k) => k.eigen).map((k) => k.kalender));
  return eigene
    .filter((k) => !k.nur_lesen && !schon.has(k.id) && !(ohneAbos && k.sync_url))
    .map((k) => ({ id: k.id, name: k.name }))
    .sort((a, b) => a.name.localeCompare(b.name, 'de'));
}

/** `stufe`: `view` oder `edit` (nur auf openany.de; vor Ort wird gelesen). */
export async function kalenderTeilen(projekt, kalender, stufe = 'view') {
  await invoke('projekt_kalender_teilen', { projekt, kalender, stufe });
  window.dispatchEvent(new Event(GETEILT));
}
