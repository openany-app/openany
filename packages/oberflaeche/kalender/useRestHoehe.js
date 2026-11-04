// Wie hoch darf ein Raster sein, damit es genau bis über die untere Leiste
// reicht? Gemessen wird vom oberen Rand des Elements bis zum Fensterende,
// abzüglich der Leiste unten (nur unter `md` da) und des unteren Rands der
// Seite (`.modul-seite`). So füllt das Raster den Platz zwischen Kopfzeile
// und Menü, ohne dass die Seite scrollt.
import { ref, onMounted, onUnmounted, nextTick, inject, watch } from 'vue';

/**
 * Steht etwas zeitweise über dem Raster (die Liste „Meine Kalender“), misst
 * es nicht neu: Es behält seine Höhe, die Seite scrollt. Geht die Liste zu,
 * wird gleich gemessen -- sonst bliebe das Raster gestaucht, bis der
 * Beobachter es merkt. Die Arbeitsfläche stellt den Schalter bereit.
 */
export const RASTER_FEST = Symbol('kalender:raster-fest');

export function useRestHoehe(elRef) {
  const rest = ref(null);

  const fest = inject(RASTER_FEST, ref(false));

  const messen = () => {
    const el = elRef.value;
    if (!el || fest.value) return;
    const oben = el.getBoundingClientRect().top + window.scrollY;
    const leiste = document.querySelector('[data-leiste-unten]');
    // Ab `md` ist die Leiste ausgeblendet und misst 0.
    const leisteHoehe = leiste ? leiste.getBoundingClientRect().height : 0;
    const seite = el.closest('.modul-seite');
    const seiteUnten = seite ? parseFloat(getComputedStyle(seite).paddingBottom) || 0 : 0;
    rest.value = Math.max(0, Math.floor(window.innerHeight - oben - leisteHoehe - seiteUnten));
  };

  let beobachter = null;
  watch(fest, async (jetzt) => {
    if (jetzt) return;
    await nextTick();
    messen();
  });

  onMounted(async () => {
    await nextTick();
    messen();
    window.addEventListener('resize', messen);
    // Kopf und Hinweise darüber können ihre Höhe ändern (Fehlerzeile,
    // Offline-Streifen); dann rückt das Raster und muss neu messen.
    if (typeof ResizeObserver !== 'undefined') {
      beobachter = new ResizeObserver(messen);
      beobachter.observe(document.body);
    }
  });
  onUnmounted(() => {
    window.removeEventListener('resize', messen);
    beobachter?.disconnect();
  });

  return { rest, messen };
}
