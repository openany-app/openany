<script setup>
/*
 * Die Kachel „Aktualisierung" -- nur auf dem Schreibtisch (aktualisierung.rs,
 * docs/plan-desktop.md Schritt 3). Unter Android lehnt `aktualisierung_lage`
 * ab; dann bleibt die Kachel unsichtbar. Dort aktualisiert die neue APK.
 */
import { ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { Download, RefreshCw, Loader2, AlertTriangle } from 'lucide-vue-next';
import BaseButton from '@oberflaeche/base/BaseButton.vue';

const { t } = useI18n();

const da = ref(false);
const version = ref('');
const neu = ref(null);
const geprueft = ref(false);
const arbeitet = ref('');
const fehler = ref('');

async function suchen() {
    fehler.value = '';
    arbeitet.value = 'suchen';
    try {
        const l = await invoke('aktualisierung_lage');
        da.value = true;
        version.value = l.version;
        neu.value = l.neu;
        geprueft.value = true;
    } catch (e) {
        // Unter Android gibt es das nicht: keine Kachel. Auf dem Schreibtisch
        // heißt ein Fehler meist „kein Netz" oder „noch kein Release".
        if (da.value) fehler.value = String(e?.message ?? e);
    } finally {
        arbeitet.value = '';
    }
}

async function installieren() {
    fehler.value = '';
    arbeitet.value = 'installieren';
    try {
        // Startet das Programm neu; zurück kommt nur ein Fehler.
        await invoke('aktualisierung_installieren');
    } catch (e) {
        fehler.value = String(e?.message ?? e);
        arbeitet.value = '';
    }
}

onMounted(async () => {
    // Erst nur die eigene Version: Zeigt sich die Kachel, fragt sie nach.
    try {
        const l = await invoke('aktualisierung_lage');
        da.value = true;
        version.value = l.version;
        neu.value = l.neu;
        geprueft.value = true;
    } catch (e) {
        const text = String(e?.message ?? e);
        if (!text.includes('Not available on this platform')) {
            da.value = true;
            fehler.value = text;
        }
    }
});
</script>

<template>
  <div v-if="da" class="karte p-6 shadow-sm space-y-4">
    <h3 class="text-lg font-extrabold text-schrift flex items-center gap-2">
      <Download class="w-5 h-5 text-marke" /> {{ t('app.aktualisierung.titel') }}
    </h3>
    <p v-if="version" class="text-sm text-fliess">{{ t('app.aktualisierung.aktuell', { version }) }}</p>

    <template v-if="neu">
      <p class="text-sm font-bold text-schrift">{{ t('app.aktualisierung.neu', { version: neu.version }) }}</p>
      <p v-if="neu.notizen" class="text-sm text-fliess whitespace-pre-wrap">{{ neu.notizen }}</p>
      <BaseButton :disabled="arbeitet !== ''" @click="installieren">
        <Loader2 v-if="arbeitet === 'installieren'" class="w-4 h-4 animate-spin" /><Download v-else class="w-4 h-4" />
        {{ t('app.aktualisierung.installieren') }}
      </BaseButton>
      <p class="text-xs text-leise">{{ t('app.aktualisierung.neustartHinweis') }}</p>
    </template>
    <template v-else>
      <p v-if="geprueft" class="text-sm text-leise">{{ t('app.aktualisierung.keine') }}</p>
      <BaseButton variant="secondary" :disabled="arbeitet !== ''" @click="suchen">
        <Loader2 v-if="arbeitet === 'suchen'" class="w-4 h-4 animate-spin" /><RefreshCw v-else class="w-4 h-4" />
        {{ t('app.aktualisierung.suchen') }}
      </BaseButton>
    </template>

    <p v-if="fehler" class="text-sm text-warnung flex items-start gap-2">
      <AlertTriangle class="w-4 h-4 shrink-0 mt-0.5" /><span class="min-w-0 break-words">{{ t('app.aktualisierung.fehler') }} ({{ fehler }})</span>
    </p>
  </div>
</template>
