<script setup>
// Wochenansicht: sieben Spalten Mo–So über einer Stundenachse, darüber das
// Band für Ganztägiges. Wo das Monatsraster zählt („da ist etwas"), zeigt die
// Woche die Lage im Tag – und damit das, was sich mit dem Stundenplan
// überschneidet. Die Rechnerei steckt in useCalendarWeek; hier wird nur
// gezeichnet.
import { computed, onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { formatTime, formatWeekday, isoWoche } from '@oberflaeche/shared/date';
import { useRestHoehe } from './useRestHoehe';

const props = defineProps({
  weekDays: { type: Array, required: true },   // Tageszellen Mo–So, in der Tagesansicht einer
  hours: { type: Array, required: true },      // sichtbare Stunden, z. B. 7…18
  allDayMap: { type: Map, required: true },    // dateString -> Einträge
  timedMap: { type: Map, required: true },     // dateString -> Einträge mit Lage
});
const emit = defineEmits(['add', 'open', 'open-day']);

const { t } = useI18n();

// Höhe einer Stundenzeile in Pixeln. Die Lage der Einträge kommt in Prozent
// aus dem Composable – hier wird nur festgelegt, wovon.
const STUNDE_PX = 48;

// DIE ACHSE TRAEGT 24 STUNDEN, ZU SEHEN IST EIN AUSSCHNITT.
//
// Der Tag steht vollstaendig im Raster (useCalendarWeek), aber ein Kasten von
// 1152 px Hoehe fuellt jeden Schirm und schiebt alles andere aus dem Bild.
// Der Ausschnitt reicht deshalb bis zum Menü (useRestHoehe) und beginnt bei
// 7 Uhr; wer früher oder später braucht, scrollt hin. Ohne Messung (noch
// nicht gezeichnet) sind es zwölf Stunden.
const SICHTBARE_STUNDEN = 12;
const START_STUNDE = 7;
// Unter diese Höhe schrumpft der Ausschnitt nicht, auch wenn wenig Platz ist.
const MINDEST_PX = 4 * STUNDE_PX;

const stundenBereich = ref(null);
const { rest } = useRestHoehe(stundenBereich);
// Abzüglich des unteren Rands der Karte (eine Linie), höchstens der ganze Tag.
const ausschnitt = computed(() => Math.min(
  props.hours.length * STUNDE_PX,
  rest.value ? Math.max(MINDEST_PX, rest.value - 1) : SICHTBARE_STUNDEN * STUNDE_PX,
));

// Nach dem Zeichnen an den Anfang des Tages ruecken, den man ueblicherweise
// sucht. Nicht per CSS zu machen: Ein scrollbarer Kasten beginnt immer oben.
onMounted(() => {
  // Ein paar Pixel früher, damit die Beschriftung „7:00“ nicht angeschnitten ist.
  if (stundenBereich.value) stundenBereich.value.scrollTop = START_STUNDE * STUNDE_PX - 8;
});

const isToday = (date) => {
  const heute = new Date();
  return date.getDate() === heute.getDate()
    && date.getMonth() === heute.getMonth()
    && date.getFullYear() === heute.getFullYear();
};

// Verstrichenes wird ausgegraut – wie im Monatsraster. Der Tag kommt aus dem
// Eintrag, die Minute aus seiner Endzeit; das trägt beide Quellen.
const istVergangen = (e) => {
  const d = new Date(`${e.startTag}T00:00:00`);
  d.setMinutes(e.bisMinute);
  return d.getTime() < Date.now();
};

const stundenLabel = (h) => formatTime(new Date(2024, 0, 1, h, 0));

const oeffne = (e) => emit('open', e.eintrag);

// Sieben Spalten in der Woche, eine am Tag.
const spalten = computed(() => ({ gridTemplateColumns: `repeat(${props.weekDays.length}, minmax(48px, 1fr))` }));

const hoehe = () => `${props.hours.length * STUNDE_PX}px`;
</script>

<template>
  <div class="karte shadow-md overflow-hidden">
    <!-- DIE MINDESTBREITE STEHT AN DER SPALTE, NICHT AN DER SUMME.
         Eine von Hand gerechnete Summe aus Stundenachse plus sieben Spalten
         würde bei jeder Änderung an der Achse still falsch. Das Raster sagt
         es selbst: `minmax(48px, 1fr)`. Die Spalten
         fuellen, was da ist, und unterschreiten 48 px nicht; darunter setzt
         `overflow-x-auto` das Wischen fort. Der Browser rechnet, nicht ich.

         WARUM 48. Abzueglich Polster (p-1) und Innenabstand der
         Termin-Kaestchen (px-1.5) bleiben rund 36 px Text, bei 10 px Schrift
         also etwa sechs Zeichen. Das ist die Untergrenze, ab der ein Titel
         noch etwas aussagt. Auf einem Telefon mit rund 410 CSS-Pixeln
         (nicht zu verwechseln mit der Display-Aufloesung -- ein Geraet mit
         1220 Bildpunkten rechnet bei Pixelverhaeltnis 3 mit 407) passen
         damit alle sieben Tage ohne Wischen, und die Spalten werden 53 px
         breit statt der geforderten 48. -->
    <div class="overflow-x-auto">
      <!-- `min-w-max`: Der Kasten ist mindestens so breit wie das Raster
           darin verlangt. Ohne ihn naehmen die drei Zeilen nur die Breite des
           Fensters ein, waehrend ihr Inhalt darueber hinausragt -- beim
           Wischen brechen dann Hintergrund und Trennlinien mittendrin ab. -->
      <div class="min-w-max">

        <!-- Kopfzeile: Wochentag und Datum. In der Tagesansicht nicht -- dort
             steht das ganze Datum schon im Kopf der Seite. -->
        <div v-if="weekDays.length > 1" class="flex border-b border-linie bg-marke-leise/30 dark:bg-slate-800/50">
          <!-- Die Ecke über der Stundenachse trägt die Kalenderwoche. -->
          <div class="w-10 shrink-0 py-2 flex flex-col items-center justify-center text-marke">
            <span class="text-[10px] font-extrabold uppercase">{{ t('calendar.weekGrid.kw') }}</span>
            <span class="text-xs font-bold tabular-nums">{{ weekDays.length ? isoWoche(weekDays[0].date) : '' }}</span>
          </div>
          <div class="flex-1 grid" :style="spalten">
            <div v-for="cell in weekDays" :key="cell.dateString" class="py-2 text-center">
              <div class="text-[10px] font-extrabold text-marke uppercase">{{ formatWeekday(cell.date) }}</div>
              <!-- Wie im Monatsraster: Die Tageszahl führt in die Durchsicht
                   dieses Tages. -->
              <button @click="emit('open-day', cell.dateString)" :title="t('calendar.monthGrid.openDayTitle')"
                class="mt-0.5 mx-auto text-xs font-bold w-6 h-6 flex items-center justify-center rounded-full hover:ring-2 hover:ring-marke"
                :class="isToday(cell.date) ? 'bg-marke text-white' : 'text-leise'">
                {{ cell.date.getDate() }}
              </button>
            </div>
          </div>
        </div>

        <!-- Band für Ganztägiges und Mehrtägiges -->
        <div class="flex border-b border-linie">
          <!-- Die Spalte bleibt leer, aber sie bleibt: Sie haelt die Flucht
               mit der Stundenachse darunter, die ebenfalls w-10 breit ist.
               Die Beschriftung steht nur noch fuer Screenreader da - im Bild
               sagt die Lage ueber der Stundenachse, worum es geht, und der
               Platz gehoert den sieben Spalten. -->
          <div class="w-10 shrink-0 py-1.5">
            <span class="sr-only">{{ t('calendar.weekGrid.allDay') }}</span>
          </div>
          <div class="flex-1 grid divide-x divide-linie" :style="spalten">
            <div v-for="cell in weekDays" :key="cell.dateString"
              @click="emit('add', cell.dateString)"
              class="min-w-0 min-h-[28px] p-1 space-y-1 cursor-pointer hover:bg-slate-50 dark:hover:bg-slate-800/50">
              <div v-for="entry in allDayMap.get(cell.dateString) || []" :key="entry.id"
                @click.stop="oeffne(entry)"
                class="h-[20px] flex items-center rounded-lg px-1.5 text-[10px] font-bold text-white truncate cursor-pointer hover:brightness-95 shadow-sm"
                :class="istVergangen(entry) ? 'opacity-40' : ''"
                :style="{ backgroundColor: entry.color || '#6366f1' }">
                <span class="truncate">{{ entry.title }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Stundenachse: der ganze Tag, sichtbar ein Ausschnitt.
             `overscroll-contain` haelt den Wisch hier fest -- sonst scrollt am
             Ende der Achse die ganze Seite weiter, und man verliert die
             Wochenansicht aus dem Blick. -->
        <div ref="stundenBereich" class="flex bg-vertieft overflow-y-auto overscroll-contain ohne-scrollbalken"
             :style="{ height: `${ausschnitt}px` }">
          <div class="w-10 shrink-0">
            <!-- Die Beschriftung sitzt an der oberen Kante ihrer Zeile, damit
                 sie mit der Linie fluchtet, an der die Stunde beginnt. -->
            <div v-for="h in hours" :key="h" class="relative" :style="{ height: `${STUNDE_PX}px` }">
              <span class="absolute -top-1.5 right-1 text-[10px] font-bold text-slate-400 tabular-nums">{{ stundenLabel(h) }}</span>
            </div>
          </div>

          <div class="flex-1 grid divide-x divide-linie" :style="spalten">
            <div v-for="cell in weekDays" :key="cell.dateString" class="relative bg-flaeche" :style="{ height: hoehe() }">
              <!-- Stundenzeilen: gleichzeitig Trennlinie und Klickfläche für
                   einen neuen Termin an diesem Tag. -->
              <div v-for="h in hours" :key="h"
                @click="emit('add', cell.dateString)"
                class="border-t border-linie hover:bg-slate-50 dark:hover:bg-slate-800/50 cursor-pointer"
                :style="{ height: `${STUNDE_PX}px` }"></div>

              <div v-for="entry in timedMap.get(cell.dateString) || []" :key="entry.id"
                @click.stop="oeffne(entry)"
                class="absolute px-1.5 py-0.5 rounded-lg text-[10px] font-bold text-white overflow-hidden cursor-pointer hover:brightness-95 shadow-sm"
                :class="istVergangen(entry) ? 'opacity-40' : ''"
                :style="{
                  top: `${entry.top}%`,
                  height: `${entry.height}%`,
                  left: `calc(${entry.left}% + 2px)`,
                  width: `calc(${entry.width}% - 4px)`,
                  backgroundColor: entry.color || '#6366f1',
                }">
                <div class="truncate">{{ entry.title }}</div>
                <div class="truncate font-medium opacity-85 text-[9px]">{{ formatTime(entry.zeitQuelle) }}</div>
              </div>
            </div>
          </div>
        </div>

      </div>
    </div>
  </div>
</template>
