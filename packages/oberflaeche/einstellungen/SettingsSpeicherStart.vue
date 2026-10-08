<script setup>
// Mit welchem Reiter Speicher aufgeht. Drei Kacheln, sofortiges Speichern --
// wie beim Design nebenan.
import { useI18n } from 'vue-i18n';
import { HardDrive, Check, Image as ImageIcon, Folder, FileText } from 'lucide-vue-next';
import { useSpeicherStart } from '../composables/useSpeicherStart';
import { useToast } from '../composables/useToast';

const { t } = useI18n();
const toast = useToast();
const { speicherStart, setSpeicherStart } = useSpeicherStart();

// Dieselbe Reihenfolge und dieselben Namen wie die Reiter unter Speicher.
const REITER = [
  { id: 'gallery', icon: ImageIcon, label: () => t('albums.albums.title') },
  { id: 'files', icon: Folder, label: () => t('files.title') },
  { id: 'documents', icon: FileText, label: () => t('documents.title') },
];

const waehlen = async (id) => {
  try {
    await setSpeicherStart(id);
  } catch (e) {
    toast.error(t('settings.speicherStart.saveFailed'));
  }
};
</script>

<template>
  <div class="karte p-6 shadow-sm space-y-6">
    <h3 class="text-lg font-extrabold text-schrift flex items-center gap-2">
      <HardDrive class="w-5 h-5 text-marke" /> {{ t('settings.speicherStart.title') }}
    </h3>
    <p class="text-sm text-leise font-medium -mt-3">
      {{ t('settings.speicherStart.description') }}
    </p>

    <div class="space-y-3" role="radiogroup" :aria-label="t('settings.speicherStart.title')">
      <button
        v-for="r in REITER"
        :key="r.id"
        type="button"
        role="radio"
        :aria-checked="speicherStart === r.id"
        @click="waehlen(r.id)"
        class="w-full flex items-center justify-between gap-4 p-4 border rounded-xl text-left transition-all duration-300 cursor-pointer"
        :class="speicherStart === r.id
          ? 'bg-marke-leise border-marke shadow-sm'
          : 'border-linie hover:bg-slate-50 dark:hover:bg-slate-800/50'"
      >
        <span class="flex items-center gap-3 min-w-0">
          <component :is="r.icon" class="w-5 h-5 shrink-0" :class="speicherStart === r.id ? 'text-marke' : 'text-leise'" />
          <span class="font-bold transition-colors" :class="speicherStart === r.id ? 'text-marke' : 'text-fliess'">
            {{ r.label() }}
          </span>
        </span>
        <Check v-if="speicherStart === r.id" class="w-5 h-5 text-marke shrink-0" />
      </button>
    </div>
  </div>
</template>
