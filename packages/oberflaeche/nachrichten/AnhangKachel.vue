<script setup>
/*
 * Ein Anhang im Verlauf (docs/plan-email-pgp.md, Schritt 1). Gemeinsam für
 * jeden Weg: Die Bytes holt `laden()`, woher auch immer -- aus der eigenen
 * Ablage, vom Homeserver, später aus dem Postfach.
 *
 * Bilder zeigen sich gleich als Vorschau; alles andere als Name und Größe.
 * Antippen meldet `oeffnen` -- was das heißt (Betrachter, Speichern), weiß
 * die Arbeitsfläche.
 */
import { ref, onMounted, onBeforeUnmount } from 'vue';
import { useI18n } from 'vue-i18n';
import { Download, Loader2, Smartphone } from 'lucide-vue-next';
import { fileIcon, fileIconColor } from '../shared/fileIcons';
import { formatBytes } from '../shared/format';

const props = defineProps({
  anhang: { type: Object, required: true }, // { name, mime, groesse }
  // () => Promise<ArrayBuffer | Uint8Array | Blob>
  laden: { type: Function, required: true },
  // Nur im Programm: ein zweiter Knopf „Aufs Gerät" (Download-Ordner, ohne
  // Abgleich).
  aufsGeraet: { type: Boolean, default: false },
});
const emit = defineEmits(['oeffnen', 'speichern', 'aufs-geraet']);
const { t } = useI18n();

const istBild = (props.anhang.mime ?? '').startsWith('image/');
const art = (() => {
  const mime = props.anhang.mime ?? '';
  if (mime.includes('pdf')) return 'pdf';
  for (const a of ['image', 'video', 'audio', 'text']) if (mime.startsWith(a)) return a;
  return 'file';
})();

const vorschau = ref(null);
const laedt = ref(false);
const kaputt = ref(false);

onMounted(async () => {
  if (!istBild) return;
  laedt.value = true;
  try {
    const daten = await props.laden();
    const blob = daten instanceof Blob ? daten : new Blob([daten], { type: props.anhang.mime });
    vorschau.value = URL.createObjectURL(blob);
  } catch {
    kaputt.value = true;
  } finally {
    laedt.value = false;
  }
});
onBeforeUnmount(() => { if (vorschau.value) URL.revokeObjectURL(vorschau.value); });

defineExpose({ vorschau });
</script>

<template>
  <div class="inline-flex max-w-full items-stretch rounded-xl border border-linie bg-flaeche overflow-hidden">
    <button type="button" class="flex items-center gap-3 min-w-0 text-left cursor-pointer hover:bg-auflage"
            :class="istBild && vorschau ? 'p-0' : 'px-3 py-2'"
            :title="anhang.name" @click="emit('oeffnen', vorschau)">
      <img v-if="istBild && vorschau" :src="vorschau" :alt="anhang.name" class="block max-h-48 max-w-full sm:max-w-xs object-contain" />
      <template v-else>
        <Loader2 v-if="laedt" class="w-5 h-5 animate-spin text-marke shrink-0" />
        <component :is="fileIcon(art)" v-else class="w-6 h-6 shrink-0" :class="fileIconColor(art)" />
        <span class="min-w-0">
          <span class="block text-sm font-bold text-schrift truncate max-w-[14rem] sm:max-w-xs">{{ anhang.name }}</span>
          <span class="block text-xs text-leise">{{ anhang.groesse ? formatBytes(anhang.groesse) : '' }}<template v-if="kaputt"> · {{ t('settings.messages.anhangNichtGeladen') }}</template></span>
        </span>
      </template>
    </button>
    <button type="button" class="px-2 border-l border-linie text-leise hover:bg-auflage hover:text-marke cursor-pointer"
            :title="t('settings.messages.anhangSpeichern')" :aria-label="t('settings.messages.anhangSpeichern')"
            @click="emit('speichern')">
      <Download class="w-4 h-4" />
    </button>
    <button v-if="aufsGeraet" type="button" class="px-2 border-l border-linie text-leise hover:bg-auflage hover:text-marke cursor-pointer"
            :title="t('settings.messages.anhangAufsGeraet')" :aria-label="t('settings.messages.anhangAufsGeraet')"
            @click="emit('aufs-geraet')">
      <Smartphone class="w-4 h-4" />
    </button>
  </div>
</template>
