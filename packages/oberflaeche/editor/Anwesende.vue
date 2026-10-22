<script setup>
/*
 * Wer gerade mitschreibt (docs/plan-gemeinsam-schreiben.md): oben im Editor,
 * Farbpunkt und Name je Person. Allein zeigt sie nichts.
 */
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { Users } from 'lucide-vue-next';

const props = defineProps({
  // Teilnehmer der Sitzung: [{ kennung, name, farbe }]
  teilnehmer: { type: Array, default: () => [] },
  // Die eigene Kennung -- wird als „du“ gezeigt.
  ich: { type: String, default: '' },
  // Ohne führendes Gerät: nur lesen, mit Grund.
  aus: { type: String, default: '' },
});
const { t } = useI18n();

const andere = computed(() => props.teilnehmer.filter((p) => p.kennung !== props.ich));
</script>

<template>
  <div v-if="aus" class="flex items-center gap-2 rounded-xl bg-vertieft px-3 py-2 text-sm text-fliess">
    <Users class="w-4 h-4 shrink-0 text-leise" /> {{ t('editor.gemeinsam.aus') }}
  </div>
  <div v-else-if="andere.length" class="flex flex-wrap items-center gap-x-3 gap-y-1 text-sm text-fliess">
    <span class="flex items-center gap-1.5 text-leise"><Users class="w-4 h-4" /> {{ t('editor.gemeinsam.mit') }}</span>
    <span v-for="p in andere" :key="p.kennung" class="flex items-center gap-1.5 font-semibold text-schrift">
      <span class="w-2.5 h-2.5 rounded-full" :style="{ backgroundColor: p.farbe }"></span>
      {{ p.name || t('editor.gemeinsam.jemand') }}
    </span>
  </div>
</template>
