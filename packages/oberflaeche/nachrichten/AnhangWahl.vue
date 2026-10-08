<script setup>
/*
 * Einen Anhang an eine Nachricht hängen (docs/plan-email-pgp.md, Schritt 1).
 *
 * Gemeinsam für jeden Weg, der Anhänge kennt -- heute Matrix, später E-Mail.
 * Was mit der Datei geschieht, weiß der Weg; hier wird nur gewählt, gezeigt
 * und VOR dem Senden geprüft, ob sie unter der Grenze liegt. Eine zu große
 * Datei erst nach dem Hochladen abzulehnen, hieße Warten für nichts.
 *
 * Eine Datei je Nachricht: So schickt Matrix sie (ein Ereignis, ein
 * Anhang), und eine Liste, von der der Weg nur den ersten nimmt, wäre eine
 * falsche Zusage.
 */
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { Paperclip, X } from 'lucide-vue-next';
import KameraKnopf from '../base/KameraKnopf.vue';
import { fileIcon } from '../shared/fileIcons';
import { formatBytes } from '../shared/format';

const props = defineProps({
  modelValue: { type: Object, default: null }, // File | null
  // Höchstens so viele Bytes; 0 = unbekannt (dann prüft der Weg selbst).
  grenze: { type: Number, default: 0 },
  disabled: { type: Boolean, default: false },
});
const emit = defineEmits(['update:modelValue']);
const { t } = useI18n();

const feld = ref(null);
const fehler = ref('');

// Für das Symbol: dieselbe Einteilung wie in Dateien und Akten.
const art = (datei) => {
  const mime = datei?.type ?? '';
  if (mime.includes('pdf')) return 'pdf';
  for (const a of ['image', 'video', 'audio', 'text']) if (mime.startsWith(a)) return a;
  return 'file';
};

function nehmen(dateien) {
  const datei = dateien?.[0];
  if (!datei) return;
  if (props.grenze && datei.size > props.grenze) {
    fehler.value = t('settings.messages.anhangZuGross', { name: datei.name, grenze: formatBytes(props.grenze) });
    return;
  }
  fehler.value = '';
  emit('update:modelValue', datei);
}

function gewaehlt(e) {
  nehmen(e.target.files);
  // Zurücksetzen: sonst löst dieselbe Datei kein zweites Mal aus.
  e.target.value = null;
}
</script>

<template>
  <div class="space-y-2">
    <div v-if="modelValue" class="flex items-center gap-3 px-3 py-2 rounded-xl border border-linie bg-vertieft">
      <component :is="fileIcon(art(modelValue))" class="w-5 h-5 text-leise shrink-0" />
      <span class="min-w-0 flex-1">
        <span class="block text-sm font-bold text-schrift truncate">{{ modelValue.name }}</span>
        <span class="block text-xs text-leise">{{ formatBytes(modelValue.size) }}</span>
      </span>
      <button type="button" class="p-1.5 rounded-lg text-leise hover:bg-auflage cursor-pointer"
              :aria-label="t('settings.messages.anhangEntfernen')" :title="t('settings.messages.anhangEntfernen')"
              :disabled="disabled" @click="emit('update:modelValue', null)">
        <X class="w-4 h-4" />
      </button>
    </div>

    <div v-else class="flex items-center gap-2">
      <button type="button" :disabled="disabled"
              class="flex items-center gap-2 h-9 px-3 rounded-xl bg-slate-100 hover:bg-slate-200 dark:bg-slate-800 dark:hover:bg-slate-700 text-fliess text-sm font-bold cursor-pointer disabled:opacity-50"
              @click="feld?.click()">
        <Paperclip class="w-4 h-4" />
        <span>{{ t('settings.messages.anhangWaehlen') }}</span>
      </button>
      <input ref="feld" type="file" class="hidden" @change="gewaehlt" />
      <KameraKnopf arten="foto" :disabled="disabled" @aufgenommen="nehmen" />
      <span v-if="grenze" class="text-xs text-leise">{{ t('settings.messages.anhangGrenze', { grenze: formatBytes(grenze) }) }}</span>
    </div>

    <p v-if="fehler" class="text-xs font-bold text-rose-600">{{ fehler }}</p>
  </div>
</template>
