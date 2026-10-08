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
import { X, Upload, Loader2 } from 'lucide-vue-next';
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
  <div class="fixed inset-0 bg-slate-900/60 backdrop-blur-sm flex items-end sm:items-center justify-center p-0 sm:p-4 z-[100] ueber-tastatur" @click.self="emit('close')">
    <div role="dialog" aria-modal="true" aria-labelledby="anhang-titel"
         class="bg-flaeche w-full sm:max-w-md rounded-t-xl sm:rounded-xl shadow-2xl border border-linie p-6 space-y-4 animate-modal-in mb-16 sm:mb-0">
      <div class="flex items-center justify-between gap-4">
        <h3 id="anhang-titel" class="font-extrabold text-xl text-schrift">{{ t('notes.insertTitle') }}</h3>
        <button type="button" @click="emit('close')" :aria-label="t('common.close')"
                class="p-2 -mr-2 text-slate-400 hover:bg-auflage rounded-xl transition-colors cursor-pointer">
          <X class="w-5 h-5" />
        </button>
      </div>

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
  </div>
</template>
