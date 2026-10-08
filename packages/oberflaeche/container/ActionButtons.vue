<script setup>
// Gemeinsamer Kern der Aktions-Darstellung (ItemActions/ContainerActions):
// rendert eine Reihe Icon-Buttons einheitlich. Jede Aktion:
// { key, icon, title, onClick, danger?, disabled?, loading? }
// Die Definition der Aktionen (welche, was sie tun) bleibt bei der Ansicht.
import { Loader2 } from 'lucide-vue-next';

defineProps({
  actions: { type: Array, required: true },
});
</script>

<template>
  <button
    v-for="a in actions"
    :key="a.key"
    @click.stop="a.onClick()"
    :disabled="a.disabled || a.loading"
    :title="a.title"
    class="p-2 text-slate-400 dark:text-slate-500 rounded-xl transition-all cursor-pointer disabled:opacity-50"
    :class="a.danger
      ? 'hover:text-rose-600 hover:bg-rose-50 dark:hover:bg-rose-900/30'
      : 'hover:text-marke hover:bg-marke-leise'"
  >
    <Loader2 v-if="a.loading" class="w-4 h-4 animate-spin" />
    <component v-else :is="a.icon" class="w-4 h-4" />
  </button>
</template>
