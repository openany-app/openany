<script setup>
// Explorer-Tabelle für Behälter-Ansichten: Kopfzeile, Zeilen mit
// Icon/Name + optionaler Größen-Spalte + Meta-Spalte + Aktions-Spalte,
// Leerzustand und Fußzeile (Mehr-laden) als Slots. Die Darstellung des
// Inhalts (Icons, Aktionen) bleibt Sache der jeweiligen Ansicht.
import { computed } from 'vue';
import { useHighlight } from '@oberflaeche/composables/useHighlight';

const props = defineProps({
  columns: { type: Object, required: true }, // { name, meta, actions, size? }
  items: { type: Array, required: true },    // [{ id, type, name, ... }]
  // true: Aktionen auch auf Mobil sichtbar (rechtsbündig) statt sm-only.
  mobileActions: { type: Boolean, default: false },
  // true: oben eckig (an eine darüber angeflanschte Breadcrumb-Kachel).
  attachedTop: { type: Boolean, default: false },
  // Id einer Zeile, die hervorgehoben und ins Bild gerollt werden soll –
  // gesetzt, wenn man aus einem [[Verweis]] im Chat hierherkommt. In einem
  // langen Ordner steht die gemeinte Datei sonst irgendwo unterhalb.
  highlightId: { type: [Number, String], default: null },
});

defineEmits(['open']);

const { highlighted, merkeEintrag } = useHighlight(
  () => props.highlightId,
  () => props.items.length,
);

// Spaltenraster: mit Größen-Spalte rückt der Name enger zusammen.
const hasSize = computed(() => props.columns.size != null);
const nameClass = computed(() => (hasSize.value ? 'col-span-8 sm:col-span-6' : 'col-span-8 sm:col-span-7'));
// Meta („Geändert"): auf Mobil nur zeigen, wenn die Aktionen NICHT mobil
// gebraucht werden – sonst hat die Aktions-Spalte Vorrang (Meta erst ab sm),
// damit auf schmalen Schirmen nicht 16 Rasterspalten überlaufen.
const metaClass = computed(() => (props.mobileActions
  ? 'hidden sm:block sm:col-span-2'
  : 'col-span-4 sm:col-span-2'));
const actionsCellClass = computed(() => {
  const span = hasSize.value ? 'sm:col-span-2' : 'sm:col-span-3';
  return props.mobileActions
    ? `col-span-4 ${span} flex justify-end sm:justify-center gap-1`
    : `${span} hidden sm:flex justify-center gap-1`;
});
// Aktions-Überschrift: mit mobilen Aktionen auch auf Mobil zeigen (rechts-
// bündig wie die Aktions-Zellen), sonst erst ab sm (zentriert).
const actionsHeadClass = computed(() => {
  const span = hasSize.value ? 'sm:col-span-2' : 'sm:col-span-3';
  return props.mobileActions
    ? `col-span-4 ${span} text-right sm:text-center`
    : `${span} text-center hidden sm:block`;
});
</script>

<template>
  <div
    class="bg-flaeche border border-linie shadow-sm overflow-hidden transition-colors duration-300"
    :class="attachedTop ? 'rounded-b-xl' : 'rounded-xl'"
  >

    <!-- Tabellenkopf -->
    <div class="grid grid-cols-12 bg-slate-50 dark:bg-slate-800/40 border-b border-linie px-3 py-2.5 sm:px-6 sm:py-4 text-xs font-bold text-leise uppercase tracking-wider">
      <div :class="nameClass">{{ columns.name }}</div>
      <div v-if="hasSize" class="col-span-2 hidden sm:block text-center">{{ columns.size }}</div>
      <div :class="metaClass" class="text-center">{{ columns.meta }}</div>
      <div :class="actionsHeadClass">{{ columns.actions }}</div>
    </div>

    <!-- Zeilen -->
    <div v-if="items.length > 0" class="divide-y divide-slate-100 dark:divide-slate-800/50">
      <div
        v-for="item in items"
        :key="item.type + '-' + item.id"
        :ref="(el) => merkeEintrag(item.id, el)"
        @click="$emit('open', item)"
        class="grid grid-cols-12 px-3 py-2.5 sm:px-6 sm:py-4 items-center hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors cursor-pointer group"
        :class="item.id === highlighted ? 'bg-marke-leise ring-1 ring-inset ring-marke' : ''"
      >
        <div :class="nameClass" class="flex items-center gap-3 sm:gap-4 min-w-0">
          <slot name="icon" :item="item" />
          <span class="text-sm font-extrabold text-schrift truncate group-hover:text-marke transition-colors">
            {{ item.name }}
          </span>
        </div>

        <div v-if="hasSize" class="col-span-2 hidden sm:block text-center text-sm font-medium text-leise">
          <slot name="size" :item="item" />
        </div>

        <div :class="metaClass" class="text-center text-sm text-leise font-medium">
          <slot name="meta" :item="item" />
        </div>

        <div :class="actionsCellClass">
          <slot name="actions" :item="item" />
        </div>
      </div>
      <slot name="footer" />
    </div>

    <!-- Leerzustand -->
    <div v-else class="py-20 text-center space-y-4 max-w-sm mx-auto">
      <slot name="empty" />
    </div>
  </div>
</template>
