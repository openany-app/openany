<script setup>
/*
 * Oben in den Nachrichten, nur wenn dieses Gerät noch auf den Schlüssel
 * wartet (16.10.2026, seit Z4 am 21.10.2026 mit 6 Ziffern statt
 * Codevergleich). Anfragen ANDERER Geräte stehen oben in der App
 * (FreischaltAnfragen); die ganze Verwaltung in den Einstellungen →
 * Meine Geräte.
 */
import { ref, watch, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { KeyRound, Loader2 } from 'lucide-vue-next';
import FreischaltungNeu from '../einstellungen/FreischaltungNeu.vue';

const { t } = useI18n();
const lage = ref(null);

async function laden() {
    try {
        lage.value = await invoke('nachrichtenschluessel_lage');
    } catch {
        lage.value = null; // ohne Verbindung: nichts zu sagen
    }
}

// Kommt die Kiste gleich, alle fünf Sekunden nachsehen.
let takt = null;
watch(lage, (l) => {
    clearInterval(takt);
    takt = l?.stand === 'wartet' && l?.kiste_erwartet ? setInterval(laden, 5000) : null;
});

onMounted(laden);
onUnmounted(() => clearInterval(takt));
</script>

<template>
  <div v-if="lage?.stand === 'wartet'" class="mb-4 karte p-4 space-y-2 border-marke">
    <p class="text-sm font-bold text-schrift flex items-center gap-2"><KeyRound class="w-4 h-4 text-marke" /> {{ t('app.nachrichtenschluessel.wartet') }}</p>
    <p v-if="lage.kiste_erwartet" class="text-sm text-fliess flex items-center gap-2">
      <Loader2 class="w-4 h-4 animate-spin" /> {{ t('app.freischalten.kommtGleich') }}
    </p>
    <template v-else>
      <p class="text-sm text-fliess">{{ t('app.nachrichtenschluessel.wartetHinweis') }}</p>
      <FreischaltungNeu nur-schluessel @lage-geaendert="laden" />
    </template>
  </div>
</template>
