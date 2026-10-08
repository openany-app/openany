<script setup>
/*
 * Freigaben eines Server-Projekts (Weg 1, 30.09.2026). Eigene liegen ganz
 * auf diesem Gerät (das Programm behält sie von selbst) und öffnen sich im
 * Speicher; fremde holt der Betrachter vom Server, nur lesend. Ohne Netz
 * kommen nur die eigenen.
 */
import { ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { Folder, FileText, Image as ImageIcon, NotebookText, Smartphone, Cloud, Loader2 } from 'lucide-vue-next';
import { useToast } from '@oberflaeche/composables/useToast';
import { projekteQuelle as quelle } from '../../quellen/projekte';
import FreigabeBetrachter from '../FreigabeBetrachter.vue';

const props = defineProps({ project: { type: Object, required: true } });
const emit = defineEmits(['oeffnen']);
const toast = useToast();
const { t } = useI18n();

const freigaben = ref(null);
const offen = ref(null);
onMounted(async () => {
  try {
    freigaben.value = await quelle.freigaben(props.project.id);
  } catch {
    freigaben.value = [];
  }
});

const icon = (f) => ({ album: ImageIcon, note_folder: NotebookText }[f.art] ?? (f.zone === 'documents' ? FileText : Folder));
const ortText = (f) => t(`app.projekte.ort.${{ album: 'galerie', note_folder: 'notizen' }[f.art] ?? (f.zone === 'documents' ? 'dokumente' : 'dateien')}`);

function tippen(f) {
  if (!f.eigen) {
    offen.value = f;
    return;
  }
  // Eigene: im Speicher, im passenden Reiter. Den Ordner selbst sucht man
  // dort unter seinem Namen.
  const tab = { album: 'gallery' }[f.art] ?? (f.zone === 'documents' ? 'documents' : 'files');
  toast.info(t('app.projekte.liegtHier', { name: f.name, ort: ortText(f) }));
  emit('oeffnen', f.art === 'note_folder' ? 'notes' : 'files', { tab });
}
</script>

<template>
  <div class="md:max-w-[720px]">
    <div v-if="freigaben === null" class="flex justify-center py-6"><Loader2 class="w-5 h-5 animate-spin text-leise" /></div>
    <p v-else-if="!freigaben.length" class="text-sm text-leise">{{ t('app.projekte.keineServerFreigaben') }}</p>
    <div v-else class="karte shadow-sm divide-y divide-linie">
      <button
        v-for="f in freigaben" :key="`${f.art}:${f.uuid ?? f.id}`" type="button"
        class="w-full flex items-start gap-3 p-3 text-left hover:bg-auflage transition-colors cursor-pointer"
        :disabled="f.im_papierkorb"
        @click="tippen(f)"
      >
        <component :is="icon(f)" class="w-5 h-5 mt-0.5 shrink-0 text-marke" />
        <span class="flex-1 min-w-0">
          <span class="block text-sm font-medium text-schrift truncate">{{ f.name }}</span>
          <span class="block text-xs text-leise">
            {{ f.eigen ? t('app.projekte.vonDir') : f.von ? t('app.projekte.von', { name: f.von }) : t('app.projekte.vonMitglied') }} · {{ ortText(f) }}<template v-if="f.im_papierkorb"> · {{ t('app.projekte.imPapierkorb') }}</template>
          </span>
        </span>
        <span class="shrink-0 flex items-center gap-1 text-xs text-leise">
          <template v-if="f.eigen"><Smartphone class="w-3.5 h-3.5" /> {{ t('app.projekte.aufGeraet') }}</template>
          <template v-else><Cloud class="w-3.5 h-3.5" /> {{ t('app.projekte.nurMitNetz') }}</template>
        </span>
      </button>
    </div>
    <FreigabeBetrachter v-if="offen" :projekt="project.id" :freigabe="offen" @close="offen = null" />
  </div>
</template>
