<script setup>
// Der Knopf „Aktionen" im Kopf einer Ansicht und sein Menü.
//
// Was er anbietet, hängt davon ab, wo man steht: in einer geöffneten Mappe
// deren Aktionen, auf der obersten Ebene das, was für alles gilt. Die Zeilen
// der Liste tragen dann keine Mappen-Knöpfe mehr (Tiffy, 14.10.2026: zu
// viele Knöpfe in der Spalte).
//
// Aktionen wie bei ActionButtons: { key, icon, title, onClick, danger?,
// disabled?, loading? }. Ist die Liste leer, zeigt der Aufrufer den Knopf
// gar nicht erst.
import { ref, computed, onBeforeUnmount, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { ChevronDown, Loader2, MoreHorizontal } from 'lucide-vue-next';
import BaseButton from '@oberflaeche/base/BaseButton.vue';

const props = defineProps({
  actions: { type: Array, required: true },
  // Worauf das Menü wirkt: „Mappe", „Ordner", „Akte", „Album" (16.10.2026,
  // Tiffy: „Aktionen" stand schon über der Spalte der Zeilen-Knöpfe und
  // verwirrte). Ohne Angabe bleibt es bei „Aktionen".
  label: { type: String, default: '' },
});

const { t } = useI18n();
const offen = ref(false);
const wurzel = ref(null);
const arbeitet = computed(() => props.actions.some((a) => a.loading));
const beschriftung = computed(() => props.label || t('common.aktionen'));

const schliessen = () => { offen.value = false; };
const aussen = (e) => { if (wurzel.value && !wurzel.value.contains(e.target)) schliessen(); };
const taste = (e) => { if (e.key === 'Escape') schliessen(); };

watch(offen, (jetzt) => {
  const an = jetzt ? 'addEventListener' : 'removeEventListener';
  document[an]('pointerdown', aussen);
  document[an]('keydown', taste);
});
onBeforeUnmount(() => {
  document.removeEventListener('pointerdown', aussen);
  document.removeEventListener('keydown', taste);
});

function waehlen(a) {
  schliessen();
  a.onClick();
}
</script>

<template>
  <div ref="wurzel" class="relative">
    <!-- Wie die übrigen Kopf-Knöpfe; offen in der Themenfarbe. Die Punkte
         nur, wo die Beschriftung fehlt (schmale Bildschirme). -->
    <BaseButton
      groesse="kopf"
      :variant="offen ? 'primary' : 'secondary'"
      :title="beschriftung"
      :aria-expanded="offen"
      aria-haspopup="menu"
      @click="offen = !offen"
    >
      <Loader2 v-if="arbeitet" class="w-4 h-4 animate-spin" />
      <MoreHorizontal v-else class="w-4 h-4 sm:hidden" />
      <span class="hidden sm:inline">{{ beschriftung }}</span>
      <ChevronDown class="hidden sm:block w-3.5 h-3.5 opacity-70" />
    </BaseButton>

    <div v-if="offen" role="menu" class="absolute right-0 top-11 z-30 min-w-56 karte shadow-lg py-1 text-sm">
      <button
        v-for="a in actions"
        :key="a.key"
        type="button"
        role="menuitem"
        :disabled="a.disabled || a.loading"
        class="w-full text-left px-3 py-2 flex items-center gap-2.5 font-semibold disabled:opacity-50 cursor-pointer"
        :class="a.danger
          ? 'text-rose-600 hover:bg-rose-50 dark:hover:bg-rose-900/20'
          : 'text-fliess hover:bg-auflage'"
        @click="waehlen(a)"
      >
        <Loader2 v-if="a.loading" class="w-4 h-4 animate-spin shrink-0" />
        <component :is="a.icon" v-else class="w-4 h-4 shrink-0" :class="a.danger ? '' : 'text-slate-400'" />
        <span>{{ a.title }}</span>
      </button>
    </div>
  </div>
</template>
