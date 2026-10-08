<script setup>
// Kalender-Panel: Sichtbarkeit umschalten, Sync/Import/Neu, je Kalender
// Farbe/Export/Löschen. Löschen bestätigt in zwei Klicks (kein Dialog);
// die eigentlichen Aktionen führt der Eltern-View aus.
import { computed, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { useConfirm } from '../composables/useConfirm';
import {
  RefreshCcw, Download, FolderPlus, Check, Palette, Share2, Rss, Trash2,
  CalendarDays, Users, ListChecks, ChevronRight,
} from 'lucide-vue-next';

const props = defineProps({
  calendars: { type: Array, required: true },
  visibleCalendarIds: { type: Set, required: true },
  isSyncing: { type: Boolean, default: false },
  isLoading: { type: Boolean, default: false },
  // Quellen des Kalender-Overlays: Stundenplan/Betreuung/Fälligkeiten je
  // Projekt, in dem der Nutzer Mitglied ist (siehe CalendarOverlayController).
  overlaySources: { type: Array, default: () => [] },
  visibleOverlayKeys: { type: Set, default: () => new Set() },
  // Was die Datenquelle kann; fehlt etwas, fehlt sein Knopf.
  kann: { type: Object, default: () => ({ sync: true, import: true, export: true, feed: true }) },
});
const emit = defineEmits(['toggle', 'sync', 'import', 'create', 'edit-color', 'export', 'feed', 'delete', 'toggle-overlay']);

// Je Projekt gruppiert, damit „Schule – Lena" als EINE Überschrift mit
// darunterliegenden Schaltern erscheint statt als flache Liste.
const overlayByProject = computed(() => {
  const gruppen = new Map();
  for (const quelle of props.overlaySources) {
    if (!gruppen.has(quelle.project_id)) {
      gruppen.set(quelle.project_id, { project_id: quelle.project_id, project_name: quelle.project_name, quellen: [] });
    }
    gruppen.get(quelle.project_id).quellen.push(quelle);
  }
  return Array.from(gruppen.values());
});

const OVERLAY_ICON = { timetable: CalendarDays, care: Users, due: ListChecks };

/*
 * „AUS PROJEKTEN" STEHT EINGEKLAPPT DA — ALS EIN BLOCK.
 *
 * Es ist eine Auskunft über anderer Leute Arbeit: Stundenplan, Betreuung,
 * Fälligkeiten – je Projekt drei Schalter, bei vier Projekten zwölf. Offen
 * gezeigt nähme das den EIGENEN Kalendern darüber die Sichtbarkeit, und die
 * sind der Grund, aus dem jemand dieses Panel öffnet.
 *
 * EIN SCHALTER, NICHT EINER JE PROJEKT. Bis zum 08.09.2026 hatte jedes
 * Projekt sein eigenes Dreieck: Wer den ganzen Abschnitt wegräumen wollte,
 * klickte viermal, und vier eingeklappte Zeilen brauchen kaum weniger Platz
 * als eine aufgeklappte. Die Projektnamen stehen weiterhin da, aber als
 * Überschrift — man sieht, woher eine Quelle kommt, ohne sie zu suchen.
 *
 * WAS DAS EINKLAPPEN NICHT TUT: die Quellen abschalten. Ein zugeklappter
 * Abschnitt, dessen Stundenplan angehakt ist, zeigt seine Stunden weiter im
 * Raster — sonst verschwänden mit einem Klick auf ein Dreieck Termine, und
 * niemand fände den Zusammenhang wieder. Deshalb steht die Zahl der aktiven
 * Quellen in der Zeile.
 */
const projekteOffen = ref(false);

// Wie viele Quellen aus ALLEN Projekten gerade im Raster stehen.
const aktiveQuellen = computed(
  () => props.overlaySources.filter((q) => props.visibleOverlayKeys.has(q.key)).length,
);

const { t } = useI18n();

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
    <div v-if="calendars.length === 0 && !isLoading" class="text-xs text-slate-400 italic">{{ t('calendar.sidebar.empty') }}</div>
    <div v-else class="space-y-1">
      <div v-for="cal in calendars" :key="cal.id" class="flex items-center justify-between group">
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
          <button v-if="kann.feed" @click="emit('feed', cal)" class="p-1.5 hover:text-marke" :class="cal.ics_feed_token ? 'text-marke' : 'text-slate-400'" :title="t('calendar.sidebar.feed')"><Rss class="w-4 h-4" /></button>
          <button @click="requestDelete(cal)" class="p-1.5 text-slate-400 hover:text-rose-500" :title="t('common.delete')">
            <Trash2 class="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>

    <!-- Aus Projekten: EINE Quelle je (Projekt, Typ), nicht je Wochenplan –
         sonst könnte der zweite Betreuungsplan (`applies_to = breaks`)
         übersehen werden, und der Kalender wäre ab Ferienbeginn
         stillschweigend leer. -->
    <div v-if="overlayByProject.length > 0" class="pt-4 border-t border-linie space-y-3">
      <!-- Die Überschrift IST der Schalter – ein Dreieck für den ganzen
           Abschnitt, nicht eines je Projekt. -->
      <button @click="projekteOffen = !projekteOffen" :aria-expanded="projekteOffen"
        class="w-full flex items-center gap-1.5 rounded-xl hover:bg-auflage cursor-pointer">
        <ChevronRight class="w-4 h-4 shrink-0 text-slate-400 transition-transform"
          :class="projekteOffen ? 'rotate-90' : ''" />
        <h3 class="min-w-0 flex-1 text-left text-sm font-extrabold text-schrift truncate">{{ t('calendar.overlay.title') }}</h3>
        <!-- Wie viele Quellen gerade im Raster stehen. Ohne die Zahl wäre
             nicht zu sehen, dass ein zugeklappter Abschnitt weiter Termine
             beisteuert. -->
        <span v-if="aktiveQuellen > 0"
          class="shrink-0 px-1.5 rounded-full bg-marke-leise text-[10px] font-extrabold text-marke">
          {{ aktiveQuellen }}
        </span>
      </button>

      <div v-if="projekteOffen" class="space-y-3">
        <div v-for="gruppe in overlayByProject" :key="gruppe.project_id" class="space-y-1">
          <!-- Der Projektname ordnet die Schalter zu, ist aber nichts zum
               Anklicken mehr: Es gibt nur noch das eine Dreieck oben. -->
          <p class="px-3 text-xs font-bold text-slate-400 dark:text-slate-500 truncate">{{ gruppe.project_name }}</p>
          <button v-for="quelle in gruppe.quellen" :key="quelle.key" @click="emit('toggle-overlay', quelle.key)"
            class="w-full flex items-center gap-3 px-3 py-2 rounded-xl hover:bg-auflage">
            <span class="w-4 h-4 rounded-xl shrink-0 border-2 border-marke flex items-center justify-center"
              :class="visibleOverlayKeys.has(quelle.key) ? 'bg-marke' : 'bg-transparent'">
              <Check v-if="visibleOverlayKeys.has(quelle.key)" class="w-2.5 h-2.5 text-white" />
            </span>
            <component :is="OVERLAY_ICON[quelle.type]" class="w-4 h-4 text-marke shrink-0" />
            <span class="text-sm font-semibold text-fliess">{{ t(`calendar.overlay.types.${quelle.type}`) }}</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
