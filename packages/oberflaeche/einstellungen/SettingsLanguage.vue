<script setup>
// Sprache der Oberfläche. Gilt sofort und nur in diesem Browser
// (localStorage, siehe i18n/index.js) – wie Hell/Dunkel, und anders als das
// Design, das am Konto hängt. Der Server bekommt die Sprache mit jeder
// Anfrage im Accept-Language-Header und antwortet in ihr.
import { Languages } from 'lucide-vue-next';
import { useLocale } from '@oberflaeche/composables/useLocale';
import { useToast } from '@oberflaeche/composables/useToast';

const { locale, supportedLocales, localeNames, setLocale, t } = useLocale();
const toast = useToast();

const waehlen = async (event) => {
  const ok = await setLocale(event.target.value);
  if (!ok) {
    // Die Auswahl springt sonst nicht zurück: Der Wert im <select> ist schon
    // umgestellt, die Sprache aber nicht.
    event.target.value = locale.value;
    toast.error(t('settings.language.loadFailed'));
  }
};
</script>

<template>
  <div class="karte p-6 shadow-sm space-y-6">
    <h3 class="text-lg font-extrabold text-schrift flex items-center gap-2">
      <Languages class="w-5 h-5 text-marke" /> {{ t('settings.language.title') }}
    </h3>
    <p class="text-sm text-leise font-medium -mt-3">
      {{ t('settings.language.description') }}
    </p>
    <select :value="locale" @change="waehlen" :aria-label="t('settings.language.title')"
      class="w-full px-4 py-3 bg-vertieft border border-linie rounded-xl focus:ring-2 focus:ring-marke text-schrift font-medium">
      <!-- lang je Eintrag: Screenreader lesen „Français" dann französisch vor. -->
      <option v-for="code in supportedLocales" :key="code" :value="code" :lang="code">
        {{ localeNames[code] }}
      </option>
    </select>
  </div>
</template>
