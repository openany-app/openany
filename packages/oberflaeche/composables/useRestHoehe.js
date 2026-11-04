/*
 * WIE HOCH DARF EIN KASTEN SEIN, DAMIT ER GENAU BIS UNTEN REICHT?
 *
 * Gebraucht vom Notiz-Editor auf Telefon und Tablet. Mit fester Höhe
 * (`100vh - 10rem`) liefe der Kasten unter die Leiste unten und unter die
 * Tastatur. Deshalb endet er genau dort, wo das Sichtbare endet:
 *
 *   Tastatur offen   über der Tastatur (und über der Leiste des Editors, die
 *                    auf dem Telefon dort klebt, siehe MarkdownEditor)
 *   Tastatur zu      über der Leiste unten (`data-leiste-unten`)
 *
 * Gescrollt wird dann nur noch IM Kasten; die Seite steht.
 *
 * DIE TASTATUR: Das Programm auf Android meldet sie selbst
 * (`window.openanyRaender`, app/src/raender.js) -- seine Webansicht
 * verkleinert dafür nichts. Im Browser schrumpft `visualViewport`.
 *
 * Am Schreibtisch ohne Tastatur gibt es keine Zahl (`null`): Dann gilt, was
 * im CSS steht.
 */
import { ref, watch, onMounted, onBeforeUnmount, nextTick } from 'vue';

const MOBIL = '(max-width: 767px)';
// Ab so viel fehlender Höhe ist es eine Tastatur und kein Adressbalken.
const TASTATUR_AB = 80;
// Die Leiste des Editors über der Tastatur (MarkdownEditor, `kbOpen`).
const EDITOR_LEISTE = 44;

/** Unterkante des Sichtbaren in Fenster-Koordinaten, falls eine Tastatur offen ist. */
export function ueberTastatur() {
  const bruecke = window.openanyRaender;
  if (bruecke?.tastatur) {
    const hoehe = bruecke.tastatur();
    return hoehe >= TASTATUR_AB ? window.innerHeight - hoehe : null;
  }
  const vv = window.visualViewport;
  if (!vv) return null;
  return window.innerHeight - vv.height >= TASTATUR_AB ? vv.height : null;
}

export function useRestHoehe(el) {
  const hoehe = ref(null);
  const tastaturOffen = ref(false);

  function messen() {
    const kasten = el.value;
    const mobil = window.matchMedia(MOBIL).matches;
    const tastatur = ueberTastatur();
    tastaturOffen.value = tastatur !== null;

    if (!kasten || (!mobil && tastatur === null)) {
      hoehe.value = null;
      return;
    }

    let unten;
    if (tastatur !== null) {
      // Die Leiste des Editors steht nur auf dem Telefon über der Tastatur,
      // und nur, solange er den Fokus hat.
      const imEditor = document.activeElement?.closest?.('.md-editor');
      unten = tastatur - (mobil && imEditor ? EDITOR_LEISTE : 8);
    } else {
      const leiste = document.querySelector('[data-leiste-unten]');
      const r = leiste?.getBoundingClientRect();
      unten = r && r.height ? r.top : window.innerHeight;
    }

    // Absolut gemessen: Ist die Seite noch ein Stück gescrollt, wird der
    // Kasten so bemessen, als stünde sie oben -- dann gibt es nichts mehr zu
    // scrollen, und sie springt dorthin zurück.
    const oben = kasten.getBoundingClientRect().top + window.scrollY;
    hoehe.value = Math.max(160, Math.floor(unten - oben));
  }

  // Die Tastatur klappt animiert auf; ihre Höhe steht erst danach fest.
  function spaeterMessen() {
    messen();
    setTimeout(messen, 300);
  }

  watch(el, () => nextTick(messen));

  onMounted(() => {
    window.addEventListener('resize', messen);
    window.addEventListener('openany-raender', messen);
    window.visualViewport?.addEventListener('resize', messen);
    document.addEventListener('focusin', spaeterMessen);
    document.addEventListener('focusout', spaeterMessen);
    nextTick(messen);
  });

  onBeforeUnmount(() => {
    window.removeEventListener('resize', messen);
    window.removeEventListener('openany-raender', messen);
    window.visualViewport?.removeEventListener('resize', messen);
    document.removeEventListener('focusin', spaeterMessen);
    document.removeEventListener('focusout', spaeterMessen);
  });

  return { hoehe, tastaturOffen, messen };
}
