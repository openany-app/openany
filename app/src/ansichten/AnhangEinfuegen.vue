<script setup>
/*
 * Einfügen in eine Notiz — die App-Fassung des Dialogs.
 *
 * Die Webapp hat zwei Reiter: „Aus openany" (Galerie, Dokumente, Dateien
 * durchsehen und verknüpfen) und „Hochladen". Hier gibt es nur den zweiten:
 * eine Datei vom Gerät, aufs Tablet fotografiert oder aus dem Download-
 * Ordner. Verknüpfen aus dem eigenen Speicher käme später — der Rust-Weg
 * dafür (`link`) fehlt noch, und ein Reiter, der nichts kann, ist schlechter
 * als keiner.
 *
 * Das Ergebnis hat dieselbe Form wie drüben ({id, path, target_type, name},
 * dazu `pfad`); `applyInsert` in der Arbeitsfläche setzt es in den Editor.
 */
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { Upload, Loader2 } from 'lucide-vue-next';
import BaseModal from '@oberflaeche/base/BaseModal.vue';
import { lokaleNotizenQuelle } from '../quellen/notizen';

const props = defineProps({
    folderId: { type: [Number, String], default: null },
});
const emit = defineEmits(['close', 'insert']);

const { t } = useI18n();
const quelle = lokaleNotizenQuelle();

const laeuft = ref(false);
const fehler = ref('');
const feld = ref(null);

async function gewaehlt(e) {
    const datei = e.target.files?.[0];
    if (!datei) return;
    fehler.value = '';
    laeuft.value = true;
    try {
        const { data } = await quelle.uploadNoteAsset(props.folderId, datei);
        emit('insert', data);
    } catch (err) {
        fehler.value = err?.response?.data?.message || String(err);
    } finally {
        laeuft.value = false;
        // Dieselbe Datei noch einmal wählen soll wieder ein `change` sein.
        if (feld.value) feld.value.value = '';
    }
}
</script>

<template>
  <BaseModal :title="t('notes.insertTitle')" @close="emit('close')">
    <div class="space-y-4">
      <!-- Ein Feld, kein Ziehen: Auf dem Tablet gibt es nichts, was man
           hierher ziehen könnte. Der Knopf öffnet die Auswahl des Systems,
           auf Android samt Kamera. -->
      <label class="flex flex-col items-center justify-center gap-2 rounded-xl border-2 border-dashed border-linie p-8 text-center cursor-pointer hover:bg-auflage transition-colors"
             :class="laeuft ? 'opacity-60 pointer-events-none' : ''">
        <Loader2 v-if="laeuft" class="w-6 h-6 animate-spin text-marke" />
        <Upload v-else class="w-6 h-6 text-marke" />
        <span class="text-sm font-bold text-fliess">{{ laeuft ? t('common.loading') : t('notes.insertUpload') }}</span>
        <input ref="feld" type="file" class="sr-only" :disabled="laeuft" @change="gewaehlt">
      </label>

      <p class="text-xs text-leise">{{ t('app.speicherAnsicht.anhangHinweis') }}</p>

      <p v-if="fehler" class="text-sm text-rose-600 dark:text-rose-400">{{ fehler }}</p>
    </div>
  </BaseModal>
</template>
