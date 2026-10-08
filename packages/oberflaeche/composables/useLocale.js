import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { setLocale, SUPPORTED_LOCALES, LOCALE_NAMES } from '@oberflaeche/i18n';

// Dünner Wrapper um vue-i18n für Komponenten: aktuelle Sprache als computed
// + Setter, der nachlädt und speichert (siehe i18n/index.js setLocale).
export function useLocale() {
  const { locale, t } = useI18n();

  const currentLocale = computed({
    get: () => locale.value,
    set: (value) => setLocale(value),
  });

  return { locale: currentLocale, supportedLocales: SUPPORTED_LOCALES, localeNames: LOCALE_NAMES, setLocale, t };
}
