import { ref } from 'vue';

// Mit welchem Reiter Speicher aufgeht: Galerie, Dateien oder Dokumente
// (27.09.2026). Gebaut wie useDesign:
//
// - Webapp: zusaetzlich im Konto (users.speicher_start), damit die Wahl auf
//   jedem Browser gilt. Sie haengt dafuer mit `speicherStartSpeichernMit()`
//   einen API-Aufruf an.
// - Programm (app/): nur auf dem Geraet.
//
// localStorage haelt den Wert in beiden Faellen, damit Speicher schon vor der
// Sitzung mit dem richtigen Reiter aufgeht.

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

let speichern = null;

/** Wohin die Wahl ausser auf das Geraet noch soll (Webapp: ins Konto). */
export function speicherStartSpeichernMit(rueckruf) {
  speichern = rueckruf;
}

/** Wahl durch den Nutzer: sofort merken, dann speichern; bei Fehler zurück. */
const setSpeicherStart = async (wert) => {
  const neu = bereinigen(wert);
  const vorher = speicherStart.value;
  if (neu === vorher) return;

  merken(neu);
  if (!speichern) return;

  try {
    const bestaetigt = await speichern(neu);
    if (bestaetigt) merken(bereinigen(bestaetigt));
  } catch (e) {
    merken(vorher);
    throw e;
  }
};

/** Der Wert aus /user/settings, ohne ihn zurückzuschreiben. */
const uebernehmeVomServer = (wert) => {
  merken(bereinigen(wert));
};

export function useSpeicherStart() {
  return { speicherStart, SPEICHER_REITER, setSpeicherStart, uebernehmeVomServer };
}
