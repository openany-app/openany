<script setup>
/*
 * Eine Karte im Whiteboard antippen (docs/plan-whiteboard.md, W2): den
 * eingebundenen Inhalt ÜBER dem Whiteboard öffnen; Schließen führt zurück an
 * dieselbe Stelle. Die App reicht diese Ansicht dem Paket herein
 * (main.js → umgebung `WHITEBOARD_VERWEIS`).
 *
 *   notiz  → die Mappe, gleich an der Notiz (MappeBetrachter)
 *   datei  → PDF im PdfBetrachter, ein Bild groß, sonst übernehmen
 *
 * Ist die Freigabe beendet: Hinweis und zurück.
 */
import { ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { convertFileSrc } from '@tauri-apps/api/core';
import { Loader2 } from 'lucide-vue-next';
import PdfBetrachter from '@oberflaeche/speicher/PdfBetrachter.vue';
import { useToast } from '@oberflaeche/composables/useToast';
import MappeBetrachter from './MappeBetrachter.vue';
import { projekteQuelle as quelle } from '../../quellen/projekte';
import { einsortieren, mimeAus } from '../../quellen/ablage';

const props = defineProps({
  projekt: { type: String, required: true },
  // { art: 'notiz'|'datei'|'bild', freigabe, uuid, name, mime? }
  verweis: { type: Object, required: true },
});
const emit = defineEmits(['close']);
const { t } = useI18n();
const toast = useToast();

const mappe = ref(null);
const pdf = ref(false);
const bild = ref(null); // Adresse
const mime = props.verweis.mime || mimeAus(props.verweis.name ?? '');

function weg(meldung) {
  toast.info(meldung);
  emit('close');
}

onMounted(async () => {
  const freigabe = ((await quelle.mappen(props.projekt).catch(() => null))?.mappen ?? [])
    .find((m) => m.uuid === props.verweis.freigabe);
  if (!freigabe?.name) return weg(t('planning.whiteboard.nichtMehrGeteilt'));

  if (props.verweis.art === 'notiz') {
    mappe.value = freigabe;
  } else if (mime === 'application/pdf') {
    pdf.value = true;
  } else if (mime.startsWith('image/')) {
    try {
      bild.value = convertFileSrc(await quelle.freigabeHolen(props.projekt, freigabe.uuid, props.verweis.uuid, false));
    } catch {
      weg(t('planning.whiteboard.nichtMehrGeteilt'));
    }
  } else {
    // Was die App nicht anzeigen kann: eine eigene Kopie ablegen, wie in
    // GeteilteSache.vue.
    try {
      const bytes = await quelle.freigabeBytes(props.projekt, freigabe.uuid, props.verweis.uuid);
      const ort = await einsortieren(new File([bytes], props.verweis.name, { type: mime }), 'Aus Projekten');
      toast.success(t('app.teilen.uebernommen', { ort }));
    } catch {
      toast.info(t('planning.whiteboard.nichtMehrGeteilt'));
    }
    emit('close');
  }
});
</script>

<template>
  <MappeBetrachter v-if="mappe" :projekt="projekt" :mappe="mappe" :notiz="verweis.uuid" @close="emit('close')" />
  <PdfBetrachter v-else-if="pdf" :name="verweis.name" :laden="() => quelle.freigabeBytes(projekt, verweis.freigabe, verweis.uuid)" @close="emit('close')" />
  <div v-else class="fixed inset-0 z-[110] bg-black/90 flex flex-col" @click.self="emit('close')">
    <div class="flex items-center justify-between gap-2 p-3 text-white" :style="{ paddingTop: 'calc(0.75rem + var(--rand-oben, 0px))' }">
      <span class="text-sm truncate">{{ verweis.name }}</span>
      <button type="button" class="text-sm font-bold cursor-pointer" @click="emit('close')">{{ t('common.close') }}</button>
    </div>
    <img v-if="bild" :src="bild" :alt="verweis.name" class="flex-1 min-h-0 object-contain" />
    <div v-else class="flex-1 flex items-center justify-center"><Loader2 class="w-6 h-6 animate-spin text-white" /></div>
  </div>
</template>
