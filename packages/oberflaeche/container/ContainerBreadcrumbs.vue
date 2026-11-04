<script setup>
// Breadcrumb-Kachel für Behälter-Ansichten (Mappen/Ordner/Akten/Alben):
// klickbare Pfad-Glieder, optional ein nicht-klickbares letztes Glied (tail,
// z. B. die offene Notiz) mit Kreuz zum Schließen und ein Slot rechts
// (z. B. das Suchfeld der Notizen).
import { useI18n } from 'vue-i18n';
import { ChevronRight, X } from 'lucide-vue-next';

defineProps({
  crumbs: { type: Array, required: true }, // [{ id, name }] – erstes Glied = Wurzel (id null)
  tail: { type: String, default: null },
  schliessbar: { type: Boolean, default: false },
  // Direkt an die Kachel darunter angeflanscht (offene Notiz): flacher, unten
  // eckig/randlos/ohne Schatten, damit Breadcrumbs + Notiz wie eine Kachel wirken.
  attached: { type: Boolean, default: false },
});

defineEmits(['navigate', 'schliessen']);

const { t } = useI18n();
</script>

<template>
  <div
    class="flex items-center justify-between bg-flaeche border border-linie transition-colors duration-300"
    :class="attached
      ? 'p-2 sm:p-2.5 rounded-t-xl rounded-b-none border-b-0'
      : 'p-2.5 sm:p-4 rounded-xl shadow-sm'"
  >
    <div class="flex items-center gap-2 text-sm font-bold overflow-x-auto">
      <div
        v-for="(bc, idx) in crumbs"
        :key="idx"
        class="flex items-center gap-2 shrink-0"
      >
        <button
          @click="$emit('navigate', bc.id)"
          class="px-1.5 py-2 -my-1 hover:text-marke transition-colors cursor-pointer"
          :class="idx === crumbs.length - 1 && !tail ? 'text-schrift' : 'text-leise'"
        >
          {{ bc.name }}
        </button>
        <ChevronRight v-if="idx < crumbs.length - 1 || tail" class="w-4 h-4 text-slate-300 dark:text-slate-600" />
      </div>

      <span v-if="tail" class="shrink-0 text-schrift truncate max-w-[16rem]">{{ tail }}</span>
    </div>

    <div class="flex items-center gap-1 shrink-0">
      <slot name="right" />
      <button
        v-if="schliessbar"
        @click="$emit('schliessen')"
        class="p-1.5 ml-2 text-slate-400 hover:bg-auflage rounded-xl transition-colors cursor-pointer"
        :aria-label="t('common.close')"
      >
        <X class="w-5 h-5" />
      </button>
    </div>
  </div>
</template>
