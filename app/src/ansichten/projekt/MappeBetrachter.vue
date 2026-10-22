<script setup>
/*
 * Eine geteilte Notiz-Mappe im Projekt öffnen (docs/plan-notizen-e2e.md, 6c):
 * die gewohnte Notiz-Arbeitsfläche, mit der Mappe als Datenquelle. Mit
 * „bearbeiten" editierbar, sonst nur zu lesen.
 */
import { ArrowLeft } from 'lucide-vue-next';
import { useI18n } from 'vue-i18n';
import NotesWorkspace from '@oberflaeche/notizen/NotesWorkspace.vue';
import { useToast } from '@oberflaeche/composables/useToast';
import { geteilteMappeQuelle } from '../../quellen/geteilteMappe';

const props = defineProps({
  projekt: { type: String, required: true },
  mappe: { type: Object, required: true }, // aus projekt_mappen
  // Gleich diese Notiz öffnen (zk_id) -- eine Karte im Whiteboard.
  notiz: { type: String, default: null },
});
const emit = defineEmits(['close']);
const { t } = useI18n();
const toast = useToast();

const quelle = geteilteMappeQuelle(props.projekt, props.mappe.uuid, {
  beiKonflikt: () => toast.info(t('app.mappe.konflikt')),
  stufe: props.mappe.stufe,
});
</script>

<template>
  <div class="fixed inset-0 z-[90] bg-boden overflow-y-auto" :style="{ paddingBottom: 'var(--rand-unten)', paddingLeft: 'var(--rand-links)', paddingRight: 'var(--rand-rechts)' }">
    <div class="sticky top-0 z-10 flex items-center gap-2 px-4 py-3 bg-boden border-b border-linie" :style="{ paddingTop: 'calc(0.75rem + var(--rand-oben))' }">
      <button type="button" class="p-2 rounded-xl hover:bg-auflage" :title="t('common.back')" @click="emit('close')"><ArrowLeft class="w-5 h-5" /></button>
      <span class="font-bold text-schrift truncate">{{ mappe.name ?? '…' }}</span>
      <span class="text-xs text-leise shrink-0">{{ mappe.stufe === 'edit' ? t('app.mappe.bearbeiten') : t('app.mappe.ansehen') }} · {{ mappe.eigen ? t('app.projekte.vonDir') : t('app.projekte.von', { name: mappe.von_name }) }}</span>
    </div>
    <div class="p-4">
      <!-- Die Quelle gibt der Mappe selbst die Nummer 1: gleich darin öffnen. -->
      <NotesWorkspace :data-source="quelle" embedded :initial-folder-id="1" :initial-note-id="notiz" />
    </div>
  </div>
</template>
