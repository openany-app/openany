<script setup>
// Breadcrumb-Kachel für Behälter-Ansichten (Mappen/Ordner/Akten/Alben):
// Zurück-Pfeil, klickbare Pfad-Glieder, optional ein nicht-klickbares
// letztes Glied (tail, z. B. die offene Notiz) und ein Slot rechts
// (z. B. das Suchfeld der Notizen).
import { ArrowLeft, ChevronRight } from 'lucide-vue-next';

defineProps({
  crumbs: { type: Array, required: true }, // [{ id, name }] – erstes Glied = Wurzel (id null)
  showBack: { type: Boolean, default: false },
  tail: { type: String, default: null },
  // Direkt an die Kachel darunter angeflanscht (offene Notiz): flacher, unten
  // eckig/randlos/ohne Schatten, damit Breadcrumbs + Notiz wie eine Kachel wirken.
  attached: { type: Boolean, default: false },
});

defineEmits(['navigate', 'back']);
</script>

<template>
  <div
    class="flex items-center justify-between bg-flaeche border border-linie transition-colors duration-300"
    :class="attached
      ? 'p-2 sm:p-2.5 rounded-t-xl rounded-b-none border-b-0'
      : 'p-2.5 sm:p-4 rounded-xl shadow-sm'"
  >
    <div class="flex items-center gap-2 text-sm font-bold overflow-x-auto">
      <button
        v-if="showBack"
        @click="$emit('back')"
        class="p-1.5 hover:bg-auflage rounded-xl transition-colors text-leise mr-2 cursor-pointer"
      >
        <ArrowLeft class="w-4 h-4" />
      </button>

      <div
        v-for="(bc, idx) in crumbs"
        :key="idx"
        class="flex items-center gap-2 shrink-0"
      >
        <button
          @click="$emit('navigate', bc.id)"
          class="hover:text-marke transition-colors cursor-pointer"
          :class="idx === crumbs.length - 1 && !tail ? 'text-schrift' : 'text-leise'"
        >
          {{ bc.name }}
        </button>
        <ChevronRight v-if="idx < crumbs.length - 1 || tail" class="w-4 h-4 text-slate-300 dark:text-slate-600" />
      </div>

      <span v-if="tail" class="shrink-0 text-schrift truncate max-w-[16rem]">{{ tail }}</span>
    </div>

    <slot name="right" />
  </div>
</template>
