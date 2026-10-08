<script setup>
// Wochenansicht: sieben Spalten Mo–So über einer Stundenachse, darüber das
// Band für Ganztägiges. Wo das Monatsraster zählt („da ist etwas"), zeigt die
// Woche die Lage im Tag – und damit das, was sich mit dem Stundenplan
// überschneidet. Die Rechnerei steckt in useCalendarWeek; hier wird nur
// gezeichnet.
//
// Overlay-Einträge (Stundenplan, Betreuung, Fälligkeiten) bleiben auch hier
// erkennbar anders: gestrichelter Rand, eigenes Ereignis beim Klick. Sie
// gehören einem Projekt und lassen sich hier nicht bearbeiten.
import { onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { formatTime, formatWeekday } from '@oberflaeche/shared/date';

const props = defineProps({
  weekDays: { type: Array, required: true },   // Tageszellen Mo–So
  hours: { type: Array, required: true },      // sichtbare Stunden, z. B. 7…18
  allDayMap: { type: Map, required: true },    // dateString -> Einträge
  timedMap: { type: Map, required: true },     // dateString -> Einträge mit Lage
});
const emit = defineEmits(['add', 'open', 'open-overlay', 'open-day']);

const { t } = useI18n();

// Höhe einer Stundenzeile in Pixeln. Die Lage der Einträge kommt in Prozent
// aus dem Composable – hier wird nur festgelegt, wovon.
const STUNDE_PX = 48;

// DIE ACHSE TRAEGT 24 STUNDEN, ZU SEHEN SIND ZWOELF.
//
// Der Tag steht vollstaendig im Raster (useCalendarWeek), aber ein Kasten von
// 1152 px Hoehe fuellt jeden Telefonschirm und schiebt alles andere aus dem
// Bild. Sichtbar ist deshalb ein Ausschnitt von zwoelf Stunden, der bei 7 Uhr
// beginnt -- dem Fenster, das die Achse frueher fest hatte. Wer den
// Frühdienst um fuenf oder den Abend um dreiundzwanzig braucht, scrollt
// hin; verschluckt wird nichts mehr.
const SICHTBARE_STUNDEN = 12;
const START_STUNDE = 7;

const stundenBereich = ref(null);

// Nach dem Zeichnen an den Anfang des Tages ruecken, den man ueblicherweise
// sucht. Nicht per CSS zu machen: Ein scrollbarer Kasten beginnt immer oben.
onMounted(() => {
  if (stundenBereich.value) stundenBereich.value.scrollTop = START_STUNDE * STUNDE_PX;
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

const oeffne = (e) => emit(e.kind === 'overlay' ? 'open-overlay' : 'open', e.eintrag);

const hoehe = () => `${props.hours.length * STUNDE_PX}px`;
</script>

<template>
  <div class="karte shadow-md overflow-hidden">
    <!-- DIE MINDESTBREITE STEHT AN DER SPALTE, NICHT AN DER SUMME.
         Bis zum 08.09.2026 trug dieser Kasten eine feste Mindestbreite von
         560 Pixeln: eine von Hand gerechnete Summe aus Stundenachse plus
         sieben Spalten. (Ausgeschrieben und nicht als Klasse notiert --
         Tailwind liest auch Kommentare und legte fuer die Schreibweise
         tatsaechlich eine Regel an, die niemand benutzt.)
         Jede Aenderung an der Achse machte die Zahl still falsch, und was
         sie eigentlich sagen wollte -- "eine Tagesspalte soll nicht unter X
         fallen" -- stand nirgends.
         Jetzt sagt das Raster es selbst: `minmax(48px, 1fr)`. Die Spalten
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

        <!-- Kopfzeile: Wochentag und Datum -->
        <div class="flex border-b border-linie bg-marke-leise/30 dark:bg-slate-800/50">
          <div class="w-10 shrink-0"></div>
          <div class="flex-1 grid grid-cols-[repeat(7,minmax(48px,1fr))]">
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
          <div class="flex-1 grid grid-cols-[repeat(7,minmax(48px,1fr))] divide-x divide-linie">
            <div v-for="cell in weekDays" :key="cell.dateString"
              @click="emit('add', cell.dateString)"
              class="min-w-0 min-h-[28px] p-1 space-y-1 cursor-pointer hover:bg-slate-50 dark:hover:bg-slate-800/50">
              <div v-for="entry in allDayMap.get(cell.dateString) || []" :key="entry.id"
                @click.stop="oeffne(entry)"
                class="h-[20px] flex items-center rounded-lg px-1.5 text-[10px] font-bold text-white truncate cursor-pointer hover:brightness-95 shadow-sm"
                :class="[entry.kind === 'overlay' ? 'border border-dashed border-white/40' : '', istVergangen(entry) ? 'opacity-40' : '']"
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
             :style="{ maxHeight: `${SICHTBARE_STUNDEN * STUNDE_PX}px` }">
          <div class="w-10 shrink-0">
            <!-- Die Beschriftung sitzt an der oberen Kante ihrer Zeile, damit
                 sie mit der Linie fluchtet, an der die Stunde beginnt. -->
            <div v-for="h in hours" :key="h" class="relative" :style="{ height: `${STUNDE_PX}px` }">
              <span class="absolute -top-1.5 right-1 text-[10px] font-bold text-slate-400 tabular-nums">{{ stundenLabel(h) }}</span>
            </div>
          </div>

          <div class="flex-1 grid grid-cols-[repeat(7,minmax(48px,1fr))] divide-x divide-linie">
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
                :class="[entry.kind === 'overlay' ? 'border border-dashed border-white/40' : '', istVergangen(entry) ? 'opacity-40' : '']"
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
