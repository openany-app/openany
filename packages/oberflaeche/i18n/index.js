import { createI18n } from 'vue-i18n';
import de from './locales/de';

// Reihenfolge = Reihenfolge in der Sprachauswahl (Einstellungen).
export const SUPPORTED_LOCALES = ['de', 'en', 'es', 'fr', 'pt', 'pl', 'uk'];

// Jede Sprache in ihrer eigenen Schreibweise: Wer die Oberfläche nicht lesen
// kann, muss seine Sprache trotzdem in der Liste finden.
export const LOCALE_NAMES = {
  de: 'Deutsch',
  en: 'English',
  es: 'Español',
  fr: 'Français',
  pt: 'Português',
  pl: 'Polski',
  uk: 'Українська',
};

const STORAGE_KEY = 'openany_locale';

// Kein DOM/localStorage in der reinen Logik-Testumgebung (vitest.config.js:
// environment 'node', siehe shared/*.test.js) – dort greifen die Fallbacks.
const hasDom = typeof window !== 'undefined' && typeof localStorage !== 'undefined';

// NUR DEUTSCH STECKT IM HAUPTBÜNDEL. Jede weitere Sprache sind gut 120 KB
// Text, die fast niemand braucht – mit sieben Sprachen trug jeder Start sonst
// 700 KB fremder Zeilen mit. Deutsch bleibt fest drin, weil es der Rückfall
// für fehlende Zeilen ist und ohne Netz immer da sein muss.
const loaders = import.meta.glob(['./locales/*/index.js', '!./locales/de/index.js']);
const loaded = new Set(['de']);

// Polnisch und Ukrainisch haben drei Mehrzahlformen: 1 Kontakt, 2–4 Kontakte,
// 5+ Kontakte (und 12–14 zählt zur dritten). Zeilen dort schreiben also
// `eins | wenige | viele`. Hat eine Zeile nur zwei Formen, gilt die
// Zweierregel wie im Deutschen.
//
// Der Unterschied zwischen beiden: Im Ukrainischen steht 21, 31, 101 in der
// Einzahl („21 контакт"), im Polnischen nur die 1 selbst („21 kontaktów").
function slavicPlural({ singularEndsInOne }) {
  return (choice, choicesLength) => {
    if (choicesLength < 3) return choice === 1 ? 0 : 1;
    const mod10 = choice % 10;
    const mod100 = choice % 100;
    if (choice === 1 || (singularEndsInOne && mod10 === 1 && mod100 !== 11)) return 0;
    if (mod10 >= 2 && mod10 <= 4 && (mod100 < 12 || mod100 > 14)) return 1;
    return 2;
  };
}

/**
 * Die Sprache beim Start: was jemand selbst gewählt hat, sonst die erste
 * unterstützte aus den Sprachen des Geräts bzw. Browsers (der Reihe nach --
 * „Italienisch, Deutsch" ergibt Deutsch), sonst Englisch.
 *
 * ENGLISCH ALS ERSATZ, NICHT DEUTSCH (Tiffy, 06.10.2026): Wer sein Gerät auf
 * Italienisch oder Türkisch gestellt hat, liest eher Englisch als Deutsch.
 * Bis dahin stand hier Deutsch, weil openany für den deutschen Markt begann.
 */
export function waehleSprache(gewaehlt, geraet) {
  if (gewaehlt && SUPPORTED_LOCALES.includes(gewaehlt)) return gewaehlt;
  for (const tag of geraet ?? []) {
    const code = String(tag ?? '').slice(0, 2).toLowerCase();
    if (SUPPORTED_LOCALES.includes(code)) return code;
  }
  return 'en';
}

export function detectInitialLocale() {
  if (!hasDom) return 'de';

  const geraet = navigator.languages?.length ? navigator.languages : [navigator.language];
  return waehleSprache(localStorage.getItem(STORAGE_KEY), geraet);
}

export const i18n = createI18n({
  legacy: false,
  locale: 'de',
  fallbackLocale: 'de',
  messages: { de },
  pluralRules: {
    pl: slavicPlural({ singularEndsInOne: false }),
    uk: slavicPlural({ singularEndsInOne: true }),
  },
});

// Lädt die Zeilen einer Sprache nach. Schlägt das fehl (kein Netz, und der
// Teil lag noch nicht im Speicher des Service Workers), bleibt die Sprache,
// die gerade gilt – lieber Deutsch als eine leere Oberfläche.
async function loadLocale(locale) {
  if (loaded.has(locale)) return;
  const modul = await loaders[`./locales/${locale}/index.js`]();
  i18n.global.setLocaleMessage(locale, modul.default);
  loaded.add(locale);
}

// Wechselt die Sprache zur Laufzeit und hält das <html lang> synchron
// (Barrierefreiheit/SEO). Einziger Schreibpfad für den Locale-State.
//
// `persist: false` nur beim Start: Die aus dem Browser erratene Sprache wird
// nicht gespeichert, sonst folgte openany einem späteren Wechsel der
// Browsersprache nicht mehr. Gespeichert wird erst, was jemand selbst wählt.
export async function setLocale(locale, { persist = true } = {}) {
  if (!SUPPORTED_LOCALES.includes(locale)) return false;

  try {
    await loadLocale(locale);
  } catch (e) {
    return false;
  }

  i18n.global.locale.value = locale;
  if (!hasDom) return true;
  if (persist) localStorage.setItem(STORAGE_KEY, locale);
  document.documentElement.setAttribute('lang', locale);
  return true;
}
