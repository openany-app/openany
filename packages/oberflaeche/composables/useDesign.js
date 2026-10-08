import { ref, watchEffect } from 'vue';

// Gemeinsamer Design-Zustand (Singleton auf Modul-Ebene) nach dem Vorbild von
// src-admin/composables/useTheme.js.
//
// NICHT ZU VERWECHSELN MIT useTheme/'openany_theme': Das ist hell/dunkel. Hier
// geht es um das Aussehen – 'klar' (Vorgabe: Petrol, kantiger, ohne Tapete)
// oder 'flieder' (das verspielte, bis 05.09.2026 einzige Design). Beide Achsen
// haengen unabhaengig voneinander als Klasse am <html>-Element.
//
// WO DIE WAHL GESPEICHERT WIRD, ENTSCHEIDET DIE ANWENDUNG, nicht dieses Paket.
//
// - Webapp: zusaetzlich im Konto (users.design), damit das Design ueber
//   Browser hinweg folgt. Sie haengt dafuer mit `designSpeichernMit()` einen
//   API-Aufruf an (frontend/src/composables/useDesign.js).
// - Programm (app/): nur auf dem Geraet. Wie dieses Geraet aussieht, geht
//   keine andere Gegenstelle etwas an -- es haengt nichts an.
//
// localStorage haelt den Wert in beiden Faellen, damit beim Laden nichts
// aufblitzt (Webapp: public/theme-init.js).

const DESIGN_KEY = 'openany_design';
export const DESIGNS = ['klar', 'flieder'];
const VORGABE = 'klar';

const bereinigen = (wert) => (DESIGNS.includes(wert) ? wert : VORGABE);

// Gleiche Vorsicht wie in public/theme-init.js: Ist localStorage gesperrt,
// gilt eben die Vorgabe – kein Grund, die App nicht starten zu lassen.
const gemerkt = (() => {
  try {
    return localStorage.getItem(DESIGN_KEY);
  } catch (e) {
    return null;
  }
})();

const design = ref(bereinigen(gemerkt));

// Klasse am <html>-Element immer synchron halten. 'klar' traegt keine Klasse –
// was in style.css unter @theme steht, IST das klare Design.
//
// Die Wache auf `document` ist nicht theoretisch: useCurrentUser importiert
// dieses Modul, und dessen Unit-Tests laufen in der Node-Umgebung ohne DOM.
// Ohne sie riss schon der blosse Import die ganze Testdatei mit
// ('document is not defined'), bevor ein einziger Test lief.
watchEffect(() => {
  if (typeof document === 'undefined') return;
  document.documentElement.classList.toggle('design-flieder', design.value === 'flieder');
});

const merken = (wert) => {
  design.value = wert;
  try {
    localStorage.setItem(DESIGN_KEY, wert);
  } catch (e) { /* Privater Modus o. ae. – dann eben mit kurzem Aufblitzen. */ }
};

let speichern = null;

/**
 * Wo die Wahl ausser auf dem Geraet noch hin soll. Der Rueckruf bekommt den
 * neuen Wert und darf den verbindlichen zurueckgeben (etwa den, den ein
 * Server bestaetigt hat). Ohne Aufruf bleibt es beim Geraet.
 */
export function designSpeichernMit(rueckruf) {
  speichern = rueckruf;
}

/**
 * Umschalten durch den Nutzer: sofort anwenden, dann speichern. Schlaegt das
 * Speichern fehl, faellt die Anzeige auf den vorherigen Wert zurueck – sonst
 * saehe man ein Design, das beim naechsten Anmelden wieder weg waere.
 */
const setDesign = async (wert) => {
  const neu = bereinigen(wert);
  const vorher = design.value;
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

/**
 * Der Wert aus /user/settings. Setzt ohne zurueckzuschreiben – sonst schriebe
 * jedes Laden der Sitzung das zurueck, was es gerade gelesen hat.
 */
const uebernehmeVomServer = (wert) => {
  merken(bereinigen(wert));
};

export function useDesign() {
  return { design, DESIGNS, setDesign, uebernehmeVomServer };
}
