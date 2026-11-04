import { ref } from 'vue';

// Mit welchem Reiter Speicher aufgeht: Galerie, Dateien oder Dokumente.
// Nur auf dem Geraet (localStorage).

const SCHLUESSEL = 'openany_speicher_start';
export const SPEICHER_REITER = ['gallery', 'files', 'documents'];
const VORGABE = 'documents';

const bereinigen = (wert) => (SPEICHER_REITER.includes(wert) ? wert : VORGABE);

const gemerkt = (() => {
  try {
    return localStorage.getItem(SCHLUESSEL);
  } catch (e) {
    return null;
  }
})();

const speicherStart = ref(bereinigen(gemerkt));

const merken = (wert) => {
  speicherStart.value = wert;
  try {
    localStorage.setItem(SCHLUESSEL, wert);
  } catch (e) { /* Privater Modus o. ae. -- dann gilt es bis zum Neuladen. */ }
};

/** Wahl durch den Nutzer. */
const setSpeicherStart = (wert) => {
  merken(bereinigen(wert));
};

export function useSpeicherStart() {
  return { speicherStart, SPEICHER_REITER, setSpeicherStart };
}
