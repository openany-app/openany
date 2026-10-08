<script setup>
// Monatsraster: Tageszellen mit gepackten Termin-Balken. Klick auf eine
// Zelle legt einen Termin an, Klick auf einen Balken öffnet die Details.
// Die Belegung (eventsMap) berechnet useCalendarGrid.
import { useI18n } from 'vue-i18n';
import { Plus } from 'lucide-vue-next';
import { getLocalDateString } from '@oberflaeche/shared/date';
import { formatTime } from '@oberflaeche/shared/date';

defineProps({
  grid: { type: Array, required: true },       // Tageszellen
  eventsMap: { type: Map, required: true },     // dateString -> Slot-Array
  // dateString -> Overlay-Einträge (Stundenplan/Betreuung/Fälligkeiten aus
  // Projekten). Eigene Zeile statt in dieselben Balken gemischt: Ein
  // Overlay-Eintrag lässt sich nicht bearbeiten/verschieben wie ein
  // Termin – @open-overlay statt @open macht den Unterschied auch im Klick
  // sichtbar, nicht nur im Detail-Dialog dahinter.
  overlayMap: { type: Map, default: () => new Map() },
});
const emit = defineEmits(['add', 'open', 'open-overlay', 'open-day']);

const { t, tm } = useI18n();

const WEEKDAYS = tm('calendar.monthGrid.weekdays');

const isToday = (date) => {
  const today = new Date();
  return date.getDate() === today.getDate() && date.getMonth() === today.getMonth() && date.getFullYear() === today.getFullYear();
};

// Verstrichene Termine (Ende liegt in der Vergangenheit) werden ausgegraut.
const isPast = (event) => new Date(event.end_date).getTime() < Date.now();

// Mehrtägige Balken bekommen an Anfang/Ende gerundete Ecken; Zwischentage
// überlappen leicht (negative Ränder), damit der Balken durchgehend wirkt.
const getEventClasses = (event, cell) => {
  const startStr = getLocalDateString(new Date(event.start_date));
  const endStr = getLocalDateString(new Date(event.end_date));
  const isMulti = endStr !== startStr;

  let classes = '';
  if (!isMulti) return classes + 'mx-1 rounded-xl px-1.5';

  const isStart = cell.dateString === startStr;
  const isEnd = cell.dateString === endStr;

  if (isStart) {
    classes += 'ml-1 rounded-l-xl pl-1.5 ';
  } else {
    classes += '-ml-[6px] rounded-l-none pl-2 relative z-10 ';
  }

  if (isEnd) {
    classes += 'mr-1 rounded-r-xl pr-1.5';
  } else {
    classes += '-mr-[6px] rounded-r-none pr-2 relative z-10 ';
  }
  return classes;
};

// Titel nur am Starttag und montags wiederholen (Balken laufen sonst
// wortlos über die Woche).
const showEventText = (event, cell) => {
  const startStr = getLocalDateString(new Date(event.start_date));
  const endStr = getLocalDateString(new Date(event.end_date));
  const isMulti = endStr !== startStr;
  if (!isMulti) return true;
  return (cell.dateString === startStr) || (cell.date.getDay() === 1);
};
</script>

<template>
  <div class="karte shadow-md overflow-hidden">
    <div class="grid grid-cols-7 border-b border-linie bg-marke-leise/30 dark:bg-slate-800/50 text-center py-3">
      <div v-for="day in WEEKDAYS" :key="day" class="text-xs font-extrabold text-marke uppercase">{{ day }}</div>
    </div>
    <!-- Zeilenzahl ergibt sich aus dem Monat (4–6 Wochen) – keine feste
         grid-rows-6, sonst bleibt bei 5-Wochen-Monaten eine leere Zeile. -->
    <div class="grid grid-cols-7 divide-x divide-y divide-linie bg-vertieft">
      <div
        v-for="(cell, idx) in grid" :key="idx"
        @click="emit('add', cell.dateString)"
        class="bg-flaeche min-h-[120px] flex flex-col p-1 hover:bg-slate-50 dark:hover:bg-slate-800/50 cursor-pointer group"
      >
        <div class="flex justify-between items-center p-1">
          <!-- Die Tageszahl öffnet die Durchsicht dieses Tages („was braucht
               das Kind morgen?"), die Zelle daneben weiterhin einen neuen
               Termin. Zwei Ziele in einer Zelle, aber das kleinere Ziel ist
               das seltenere. -->
          <button @click.stop="emit('open-day', cell.dateString)" :title="t('calendar.monthGrid.openDayTitle')"
            class="text-xs font-bold w-6 h-6 flex items-center justify-center rounded-full hover:ring-2 hover:ring-marke"
            :class="isToday(cell.date) ? 'bg-marke text-white' : 'text-slate-500'">{{ cell.date.getDate() }}</button>
          <!-- Bleibt beim Überfahren verborgen, anders als die Knöpfe im Chat
               und in der Kalenderliste: Die ganze Zelle löst dasselbe aus
               (@click am Zellen-div), das Plus ist nur ein Hinweis darauf.
               Ein Dutzend dauerhaft sichtbarer Pluszeichen je Monat wäre Lärm
               für eine Funktion, die ohnehin erreichbar ist. Verstecken ist
               vertretbar für einen HINWEIS, nicht für den einzigen Weg. -->
          <button @click.stop="emit('add', cell.dateString)" class="opacity-0 group-hover:opacity-100 p-0.5 hover:bg-marke-leise rounded-xl transition-all text-slate-400 hover:text-marke cursor-pointer" :title="t('calendar.monthGrid.addEventTitle')"><Plus class="w-3.5 h-3.5" /></button>
        </div>
        <div class="mt-1 flex-1 overflow-visible space-y-1">
          <template v-for="(event, i) in eventsMap.get(cell.dateString) || []" :key="event ? event.id : 'empty-'+i">
            <div
              v-if="event"
              @click.stop="emit('open', event)"
              class="h-[22px] sm:h-[24px] flex items-center text-[10px] sm:text-xs font-bold text-white truncate cursor-pointer hover:brightness-95 transition-all shadow-sm"
              :class="[getEventClasses(event, cell), isPast(event) ? 'opacity-40' : '']"
              :style="{ backgroundColor: event.color }"
            >
              <span :class="{'opacity-0': !showEventText(event, cell)}" class="flex items-center gap-1 min-w-0">
                <span v-if="!event.is_all_day" class="font-medium opacity-85 shrink-0 text-[9px] sm:text-[10px]">
                  {{ formatTime(event.start_date) }}
                </span>
                <span class="truncate">{{ event.title }}</span>
              </span>
            </div>
            <div v-else class="h-[22px] sm:h-[24px]"></div>
          </template>

          <!-- Overlay: Stundenplan, Betreuung, Fälligkeiten aus Projekten.
               Schlanker als ein Termin-Balken (kein Zeitraum-Layout nötig,
               jeder Eintrag gehört sich selbst dem Tag), damit die eigenen
               Termine optisch führend bleiben. -->
            <div v-for="entry in overlayMap.get(cell.dateString) || []" :key="entry.link.kind + '-' + entry.link.id + '-' + cell.dateString"
              @click.stop="emit('open-overlay', entry)"
              class="mx-1 h-[20px] sm:h-[22px] flex items-center gap-1 rounded-lg px-1.5 text-[10px] sm:text-xs font-bold text-white truncate cursor-pointer hover:brightness-95 transition-all border border-dashed border-white/40"
              :style="{ backgroundColor: entry.color || '#6366f1' }">
              <span v-if="!entry.is_all_day" class="font-medium opacity-85 shrink-0 text-[9px] sm:text-[10px]">{{ entry.starts_at }}</span>
              <span class="truncate">{{ entry.title }}</span>
            </div>
        </div>
      </div>
    </div>
  </div>
</template>
