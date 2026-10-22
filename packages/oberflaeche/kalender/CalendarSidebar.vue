<script setup>
// Kalender-Panel: Sichtbarkeit umschalten, Sync/Import/Neu, je Kalender
// Farbe/Export/Löschen. Löschen bestätigt in zwei Klicks (kein Dialog);
// die eigentlichen Aktionen führt der Eltern-View aus.
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useConfirm } from '../composables/useConfirm';
import {
  RefreshCcw, Download, FolderPlus, Check, Palette, Share2, Trash2,
} from 'lucide-vue-next';

const props = defineProps({
  calendars: { type: Array, required: true },
  visibleCalendarIds: { type: Set, required: true },
  isSyncing: { type: Boolean, default: false },
  isLoading: { type: Boolean, default: false },
  // Was die Datenquelle kann; fehlt etwas, fehlt sein Knopf.
  kann: { type: Object, default: () => ({ sync: true, import: true, export: true }) },
});
const emit = defineEmits(['toggle', 'sync', 'import', 'create', 'edit-color', 'export', 'delete']);

const { t } = useI18n();

// Eigene oben, darunter, was andere in Projekten teilen (nur lesen,
// docs/plan-kalender-e2e.md, 4c) -- dort gibt es nur das Ein- und Ausblenden.
const eigene = computed(() => props.calendars.filter((c) => !c.nur_lesen));
const geteilte = computed(() => props.calendars.filter((c) => c.nur_lesen));

/*
 * LÖSCHEN MIT EINEM DIALOG, der sagt, was verschwindet.
 *
 * HIER STAND EINE ZWEI-KLICK-BESTÄTIGUNG mit drei Sekunden Fenster: Der
 * erste Klick tat nichts und machte aus dem Mülleimer einen Haken, der
 * zweite löschte — aber nur innerhalb von drei Sekunden. Danach bewaffnete
 * jeder weitere Klick bloß neu, beliebig oft.
 *
 * Am 17.09.2026 gemeldet als „einmal ging es, danach nie wieder". Genau so
 * sah es aus: Im Zugriffsprotokoll steht EIN DELETE und danach keins mehr —
 * die Oberfläche hat nie wieder gefragt. Wer zögert, wird bestraft, und das
 * Zögern ist bei einer zerstörenden Handlung das richtige Verhalten.
 *
 * Dazu kam: Das Symbol wechselte von Mülleimer zu Haken, 16 Pixel in einer
 * Reihe von vier. Als Warnung dafür, dass ALLE TERMINE mitgehen, ist das zu
 * wenig — dieser Kalender trug über tausend.
 *
 * `useConfirm` ist der Weg, den das Haus für so etwas schon hat.
 */
const { confirmDelete } = useConfirm();

const requestDelete = async (cal) => {
  const bestaetigt = await confirmDelete(
    t('calendar.sidebar.deleteWarning', { name: cal.name }),
    { title: t('calendar.sidebar.deleteTitle') },
  );

  if (bestaetigt) emit('delete', cal);
};
</script>

<template>
  <div class="bg-flaeche border border-slate-200/80 dark:border-slate-800 rounded-xl p-5 shadow-sm space-y-4">
    <div class="flex items-center justify-between">
      <h3 class="text-sm font-extrabold text-schrift">{{ t('calendar.sidebar.title') }}</h3>
      <div class="flex items-center gap-1">
        <button v-if="kann.sync" @click="emit('sync')" class="flex items-center gap-1 text-xs font-bold text-slate-600 hover:bg-slate-100 px-2 py-1.5 rounded-xl dark:text-slate-400 dark:hover:bg-slate-800"><RefreshCcw class="w-3.5 h-3.5" :class="isSyncing ? 'animate-spin text-marke' : ''" /> {{ t('calendar.sidebar.sync') }}</button>
        <button v-if="kann.import" @click="emit('import')" class="flex items-center gap-1 text-xs font-bold text-slate-600 hover:bg-slate-100 px-2 py-1.5 rounded-xl dark:text-slate-400 dark:hover:bg-slate-800"><Download class="w-3.5 h-3.5" /> {{ t('calendar.sidebar.import') }}</button>
        <button @click="emit('create')" class="flex items-center gap-1 text-xs font-bold text-marke hover:bg-marke-leise px-2 py-1.5 rounded-xl"><FolderPlus class="w-3.5 h-3.5" /> {{ t('calendar.sidebar.new') }}</button>
      </div>
    </div>
    <div v-if="eigene.length === 0 && !isLoading" class="text-xs text-slate-400 italic">{{ t('calendar.sidebar.empty') }}</div>
    <div v-else class="space-y-1">
      <div v-for="cal in eigene" :key="cal.id" class="flex items-center justify-between group">
        <button @click="emit('toggle', cal.id)" class="flex-1 flex items-center gap-3 px-3 py-2.5 rounded-xl hover:bg-auflage">
          <span class="w-4 h-4 rounded-xl shrink-0 border-2 flex items-center justify-center" :style="{ borderColor: cal.color, backgroundColor: visibleCalendarIds.has(cal.id) ? cal.color : 'transparent' }">
            <Check v-if="visibleCalendarIds.has(cal.id)" class="w-2.5 h-2.5 text-white" />
          </span>
          <span class="text-sm font-semibold text-fliess">{{ cal.name }}</span>
        </button>
        <!-- Immer sichtbar: Farbe, Export, Abo-Link und Löschen sind hier der
             EINZIGE Weg zu diesen Funktionen. Hinter dem Überfahren mit der
             Maus versteckt, erfährt niemand, dass es sie gibt – und auf dem
             Handy gibt es kein Überfahren. -->
        <div class="flex items-center gap-1">
          <button @click="emit('edit-color', cal)" class="p-1.5 text-slate-400 hover:text-marke" :title="t('calendar.sidebar.changeColor')"><Palette class="w-4 h-4" /></button>
          <button v-if="kann.export" @click="emit('export', cal)" class="p-1.5 text-slate-400 hover:text-marke" :title="t('calendar.sidebar.exportIcs')"><Share2 class="w-4 h-4" /></button>
          <button @click="requestDelete(cal)" class="p-1.5 text-slate-400 hover:text-rose-500" :title="t('common.delete')">
            <Trash2 class="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>

    <template v-if="geteilte.length">
      <h4 class="mt-4 mb-1 px-3 text-xs font-bold uppercase tracking-wider text-leise">{{ t('calendar.sidebar.ausProjekten') }}</h4>
      <div class="space-y-1">
        <button v-for="cal in geteilte" :key="cal.id" @click="emit('toggle', cal.id)" class="w-full flex items-center gap-3 px-3 py-2 rounded-xl hover:bg-auflage text-left">
          <span class="w-4 h-4 rounded-xl shrink-0 border-2 flex items-center justify-center" :style="{ borderColor: cal.color, backgroundColor: visibleCalendarIds.has(cal.id) ? cal.color : 'transparent' }">
            <Check v-if="visibleCalendarIds.has(cal.id)" class="w-2.5 h-2.5 text-white" />
          </span>
          <span class="min-w-0">
            <span class="block text-sm font-semibold text-fliess truncate">{{ cal.name }}</span>
            <span class="block text-xs text-leise truncate">{{ t('calendar.sidebar.geteiltVon', { projekt: cal.projekt, von: cal.von }) }}</span>
          </span>
        </button>
      </div>
    </template>
  </div>
</template>
