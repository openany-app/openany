<script setup>
// Das kleine ? an einer Überschrift: springt zum passenden Abschnitt der Hilfe.
// Wie der Sprung geht, sagt die Anwendung (`oberflaeche:hilfe-oeffnen`).
// helpOutline.test.js prüft, dass jedes `ziel` in der Hilfe steht.
import { inject } from 'vue';
import { useI18n } from 'vue-i18n';
import { CircleHelp } from 'lucide-vue-next';

defineProps({ ziel: { type: String, required: true } });

const { t } = useI18n();
const oeffnen = inject('oberflaeche:hilfe-oeffnen', null);
</script>

<template>
  <button
    v-if="oeffnen"
    type="button"
    class="ml-auto shrink-0 p-1.5 -my-1.5 text-leise hover:text-marke hover:bg-auflage rounded-xl transition-colors cursor-pointer"
    :title="t('shell.footer.help')"
    :aria-label="t('shell.footer.help')"
    @click="oeffnen(ziel)"
  >
    <CircleHelp class="w-5 h-5" />
  </button>
</template>
