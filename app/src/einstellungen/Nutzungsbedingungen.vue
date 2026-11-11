<script setup>
/*
 * Den Nutzungsbedingungen zustimmen -- die erste Bestätigung eines neuen
 * Kontos (docs/plan-registrierung-app.md, A3). Erst danach gibt der Server
 * Speicher frei. Der Text kommt vom Server (rechtstext.js).
 */
import { ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import RechtstextAnzeige from '@oberflaeche/shared/RechtstextAnzeige.vue';
import { rechtstextHolen, gemerkterRechtstext } from '../quellen/rechtstext';

const emit = defineEmits(['zugestimmt']);
const { t } = useI18n();
const text = ref(gemerkterRechtstext('agb'));
const arbeitet = ref(false);
const fehler = ref('');

onMounted(async () => {
    try { text.value = await rechtstextHolen('agb'); } catch { /* der gemerkte bleibt */ }
});

async function zustimmen() {
    arbeitet.value = true;
    fehler.value = '';
    try {
        const fristen = await invoke('konto_bestaetigen', { was: 'erste' });
        emit('zugestimmt', fristen);
        window.dispatchEvent(new CustomEvent('openany-fristen'));
    } catch (e) {
        fehler.value = String(e);
    } finally {
        arbeitet.value = false;
    }
}
</script>

<template>
  <div class="space-y-3">
    <p class="text-sm text-fliess">{{ t('app.kontoAnlegen.agbHinweis') }}</p>
    <div v-if="text" class="max-h-72 overflow-y-auto rounded-xl border border-linie bg-flaeche p-4">
      <RechtstextAnzeige :text="text" />
    </div>
    <p v-else class="text-sm text-leise">{{ t('app.kontoAnlegen.agbFehlt') }}</p>
    <button class="knopf-wichtig" :disabled="arbeitet || !text" @click="zustimmen">{{ t('app.kontoAnlegen.zustimmen') }}</button>
    <p v-if="fehler" class="text-sm text-rose-600 dark:text-rose-400">{{ fehler }}</p>
  </div>
</template>
