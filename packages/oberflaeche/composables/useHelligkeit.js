import { ref, computed } from 'vue';
import { Sun, Moon, Sparkles } from 'lucide-vue-next';

/*
 * Hell, dunkel oder dem System folgen — die zweite Achse neben dem Design.
 *
 * Bis zum 15.09.2026 stand diese Logik in der Webapp-Schale (App.vue), und das
 * Programm hatte eine eigene Fassung mit ANDEREN Werten im selben Schluessel
 * (`hell`/`dunkel` statt `light`/`dark`). Wer beide nebeneinander las, sah
 * zwei Systeme. Jetzt ist es eines.
 *
 * Werte in localStorage 'openany_theme': 'auto' | 'light' | 'dark'.
 * Wirkung: Klasse `dark` am <html>-Element.
 */

const SCHLUESSEL = 'openany_theme';
const WERTE = ['auto', 'light', 'dark'];

const gelesen = (() => {
  try {
    return localStorage.getItem(SCHLUESSEL);
  } catch (e) {
    return null;
  }
})();

const modus = ref(WERTE.includes(gelesen) ? gelesen : 'auto');

const systemDunkel = () =>
  typeof window !== 'undefined' && window.matchMedia?.('(prefers-color-scheme: dark)').matches;

function anwenden() {
  if (typeof document === 'undefined') return;
  const dunkel = modus.value === 'dark' || (modus.value === 'auto' && systemDunkel());
  document.documentElement.classList.toggle('dark', dunkel);
}

// Bei „auto" muss ein Wechsel im Betriebssystem durchschlagen — sonst bleibt
// ein Fenster hell, das seit Sonnenuntergang offen steht.
if (typeof window !== 'undefined') {
  window.matchMedia?.('(prefers-color-scheme: dark)').addEventListener('change', () => {
    if (modus.value === 'auto') anwenden();
  });
}

/** auto → hell → dunkel → auto. */
function weiterschalten() {
  modus.value = WERTE[(WERTE.indexOf(modus.value) + 1) % WERTE.length];
  try {
    localStorage.setItem(SCHLUESSEL, modus.value);
  } catch (e) { /* dann eben nur fuer diese Sitzung */ }
  anwenden();
}

export function useHelligkeit(t) {
  const symbol = computed(() => (modus.value === 'light' ? Sun : modus.value === 'dark' ? Moon : Sparkles));
  const name = computed(() =>
    modus.value === 'auto' ? t('shell.header.themeAuto')
      : modus.value === 'light' ? t('shell.header.themeLight')
        : t('shell.header.themeDark'));

  return { modus, symbol, name, anwenden, weiterschalten };
}
