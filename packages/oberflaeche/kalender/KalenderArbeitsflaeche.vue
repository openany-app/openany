<script setup>
// Kalender: lädt Kalender/Termine, rahmt Toolbar und Raster und koordiniert
// die Modals. Die Rechnerei steckt in useCalendarGrid (Monat) bzw.
// useCalendarWeek (Woche), die Ansichten liegen daneben in kalender/.
//
// SEIT DEM 15.09.2026 IM GEMEINSAMEN PAKET, mit einer Datenquelle statt der
// API – dasselbe Muster wie NotesWorkspace. Die Webapp reicht ihren
// api-Service hinein (views/Calendar.vue), das Programm seine lokale SQLite.
//
// Was nur mit Server geht, ist OPTIONAL und fehlt mit seinem Knopf, wenn die
// Datenquelle es nicht anbietet: Abo-URL abgleichen (`syncCalendar`),
// ICS-Import/-Export (`importICS`, `exportICS`), Einblendungen aus Projekten
// (`getCalendarOverlay`), die Tagesansicht der Schulplanung
// (`getCalendarDay`, `createCard`) und der Abo-Link (`erweiterungen.feed`).
import { ref, computed, watch, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { Calendar, ChevronLeft, ChevronRight, Plus, CalendarDays, AlertCircle, X } from 'lucide-vue-next';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import { useToast } from '@oberflaeche/composables/useToast';
import { useCalendarGrid } from '@oberflaeche/composables/useCalendarGrid';
import { useCalendarWeek } from '@oberflaeche/composables/useCalendarWeek';
import { useCalendarDay } from '@oberflaeche/composables/useCalendarDay';
import { formatDateRangeShort, formatDayRangeShort, getLocalDateString } from '@oberflaeche/shared/date';
import ModuleHeader from '@oberflaeche/base/ModuleHeader.vue';
import ModulePage from '@oberflaeche/base/ModulePage.vue';
import CalendarSidebar from './CalendarSidebar.vue';
import CalendarMonthGrid from './CalendarMonthGrid.vue';
import CalendarWeekGrid from './CalendarWeekGrid.vue';
import CalendarDayView from './CalendarDayView.vue';
import CalendarEventModal from './CalendarEventModal.vue';
import CalendarEventDetail from './CalendarEventDetail.vue';
import CalendarCreateModal from './CalendarCreateModal.vue';
import CalendarEditModal from './CalendarEditModal.vue';
import CalendarImportModal from './CalendarImportModal.vue';
import BaseButton from '@oberflaeche/base/BaseButton.vue';

const props = defineProps({
  // Datenquelle mit den Methodennamen des api-Service (getCalendars,
  // getEvents, createEvent …). Siehe Kopf: einige sind optional.
  dataSource: { type: Object, required: true },
  // { feed, overlay } – Abo-Link und das Detail einer Projekt-Einblendung
  // (mit Karte); beides nur mit Server.
  erweiterungen: { type: Object, default: () => ({}) },
  // Wer schaut: die Tagesansicht zeigt die eigenen Hefte zuerst.
  meId: { type: [Number, String], default: null },
});
const emit = defineEmits(['open-in-project']);
const api = props.dataSource;

// Was diese Datenquelle kann – die Seitenleiste blendet danach aus.
/*
 * IN EIN ABO WIRD NICHT GESCHRIEBEN.
 *
 * `performSync()` loescht drueben bei jedem Abruf ALLE Termine des Kalenders
 * und legt sie aus dem Feed neu an -- die fremde Quelle gewinnt, und das ist
 * richtig so. Ein hier angelegter Termin waere beim naechsten Abgleich
 * spurlos fort, und der kommt, sobald jemand die Kalenderliste oeffnet und
 * die letzte Pruefung ueber eine Stunde her ist.
 *
 * Bis zum 17.09.2026 bot die Oberflaeche Abo-Kalender wie jeden anderen an.
 * Der Server weist das inzwischen ab (422) -- aber ein Nein, das erst nach
 * dem Speichern kommt, ist die schlechtere Haelfte. Was nicht geht, soll
 * nicht angeboten werden.
 */
const beschreibbare = computed(() => calendars.value.filter((c) => !c.sync_url));

/** Der Kalender eines Termins -- fuer die Frage, ob er aus einem Abo stammt. */
const kalenderVon = (termin) => calendars.value.find((c) => c.id === termin?.calendar_id);

const kann = {
  sync: Boolean(api.syncCalendar),
  import: Boolean(api.importICS),
  export: Boolean(api.exportICS),
  feed: Boolean(props.erweiterungen.feed),
};

const { t, tm } = useI18n();
const { confirmDelete } = useConfirm();
const toast = useToast();

const MONTH_NAMES = tm('calendar.calendar.months');

const focusedDate = ref(new Date());
// 'month' oder 'week'. Die Woche ist keine zweite Seite, sondern dieselbe mit
// anderer Auflösung: derselbe fokussierte Tag, dieselben Daten, dasselbe
// Zeitfenster beim Laden.
const viewMode = ref('month');
const calendars = ref([]);
const visibleCalendarIds = ref(new Set());
const events = ref([]);
const isLoading = ref(false);
const errorMessage = ref('');

// Overlay: Stundenplan, Betreuung, Fälligkeiten aus Projekten – read-only,
// eigener Endpunkt.
//
// Gespeichert wird die AUSGEBLENDETE Menge, nicht die sichtbare – anders als
// bei den eigenen Kalendern (`openany_visible_calendars`). Eine neue Quelle
// (ein Wochenplan kam gerade in einem Projekt dazu) taucht so von selbst
// sichtbar auf, ohne dass diese Ansicht wüsste, dass sie neu ist: „unbekannt"
// bedeutet automatisch „nicht ausgeblendet".
const overlaySources = ref([]);
const overlayItems = ref([]);
const hiddenOverlayKeys = ref(new Set());
const overlayDetail = ref(null); // der geöffnete Overlay-Eintrag

const visibleOverlayKeys = computed(() => new Set(
  overlaySources.value.map((s) => s.key).filter((k) => !hiddenOverlayKeys.value.has(k)),
));

// UI/Modal-Zustand
const isSidebarOpen = ref(false);
const eventModal = ref(null);   // { initialDate, editEvent }
const detailEvent = ref(null);
const createOpen = ref(false);
const editCal = ref(null);
const feedCal = ref(null); // Kalender, dessen Abo-Link verwaltet wird
const importOpen = ref(false);
// Lade-/Speicher-Anzeigen
const isSaving = ref(false);
const isCreating = ref(false);
const isUpdatingColor = ref(false);
const isSyncing = ref(false);
const isImporting = ref(false);

// Die Tagesansicht ist kein dritter Ansichtsmodus, sondern eine Durchsicht
// ÜBER dem Raster: Sie beantwortet „was braucht das Kind morgen?", und dafür
// ist ein Raster die falsche Form. Deshalb ein eigener Endpunkt, ein eigenes
// Datum und ein Fenster darüber – der Monat bleibt stehen, wo er stand.
const dayDate = ref(null);      // YYYY-MM-DD, null = geschlossen
const dayData = ref(null);
const dayLoading = ref(false);
const daySaving = ref(false);
const meId = computed(() => props.meId);
const { bloecke: dayBloecke, istLeer: dayLeer } = useCalendarDay(dayData, meId);

const { calendarGrid, gridEventsMap } = useCalendarGrid(focusedDate, events, visibleCalendarIds);
const { weekDays, hours, allDayMap, timedMap } = useCalendarWeek(
  focusedDate, events, visibleCalendarIds, overlayItems, visibleOverlayKeys,
);

const viewHeaderLabel = computed(() => (viewMode.value === 'week'
  ? formatDateRangeShort(weekDays.value[0].date, weekDays.value[6].date)
  : `${MONTH_NAMES[focusedDate.value.getMonth()]} ${focusedDate.value.getFullYear()}`));

/*
 * Dieselbe Angabe, kürzer – für Telefone.
 *
 * Auf 390 Pixeln stehen im Kopf vier Bedienelemente nebeneinander, und der
 * Platz ist knapp. GEKÜRZT WIRD ABER NUR DIE JAHRESZAHL, NICHT DER
 * MONATSNAME: „September" ist die Angabe, wegen der die Zeile da ist, das
 * Jahr ist Beiwerk – und in neun von zehn Fällen ohnehin das laufende.
 *
 * Kein Sonderfall für „nur im fremden Jahr anzeigen": Ein Kopf, der mal drei
 * und mal acht Zeichen breit ist, springt beim Umblättern, und die Breite ist
 * genau das, was hier knapp ist. Entweder es passt, oder es passt nicht.
 *
 * Dieselbe Regel gilt für die WOCHENANGABE: „01.–07. Sept." statt
 * „01.–07. Sept. 2026". Sie ist die längste Fassung dieser Beschriftung und
 * damit die, an der sich der Platz entscheidet – dass der Monatsname dort
 * abgekürzt steht, macht die Sprache selbst so, sobald zwei Daten
 * zusammengezogen werden.
 *
 * Wer das Jahr sehen will, dreht das Gerät quer: ab `sm` steht es in beiden
 * Ansichten wieder da.
 *
 * Beide Fassungen stehen im Markup und werden per `hidden`/`sm:inline`
 * umgeschaltet – kein JavaScript, das die Fensterbreite beobachtet.
 */
const viewHeaderLabelShort = computed(() => {
  if (viewMode.value === 'week') {
    return formatDayRangeShort(weekDays.value[0].date, weekDays.value[6].date);
  }

  return MONTH_NAMES[focusedDate.value.getMonth()];
});

// dateString -> sichtbare Overlay-Einträge dieses Tages. Eigene, schlanke
// Ableitung statt Wiederverwendung von useCalendarGrid: Ein Overlay-Eintrag
// ist bereits serverseitig auf einen einzelnen Tag aufgefaltet (kein
// Zeitraum, keine RRULE) – die Balken-Packung des Monatsrasters wäre hier
// reiner Aufwand ohne Nutzen.
const overlayEventsMap = computed(() => {
  const map = new Map();
  for (const item of overlayItems.value) {
    if (!visibleOverlayKeys.value.has(item.source)) continue;
    if (!map.has(item.date)) map.set(item.date, []);
    map.get(item.date).push(item);
  }
  return map;
});

// ---------- Laden ----------
async function loadData() {
  isLoading.value = true;
  errorMessage.value = '';
  try {
    const calRes = await api.getCalendars();
    calendars.value = calRes.data;

    const stored = localStorage.getItem('openany_visible_calendars');
    if (stored) {
      try {
        visibleCalendarIds.value = new Set(JSON.parse(stored));
      } catch (e) {
        calendars.value.forEach(c => visibleCalendarIds.value.add(c.id));
      }
    } else if (visibleCalendarIds.value.size === 0) {
      calendars.value.forEach(c => visibleCalendarIds.value.add(c.id));
    }
    await loadEvents();
  } catch (err) {
    errorMessage.value = t('calendar.calendar.loadDataFailed');
    console.error(err);
  } finally {
    isLoading.value = false;
  }
}

// Sichtbares Fenster: Monat des fokussierten Tages ± 1. Das deckt die im
// Monatsraster gezeigten Nachbarmonats-Tage ab – und jede Woche, die diesen
// Tag enthält, ohne dafür einen zweiten Fall zu brauchen. Die Grenzen laufen
// über getLocalDateString: toISOString hätte den Monatsersten östlich von
// Greenwich auf den Vortag gezogen.
const ladeFenster = computed(() => {
  const y = focusedDate.value.getFullYear();
  const m = focusedDate.value.getMonth();
  return {
    from: getLocalDateString(new Date(y, m - 1, 1)),
    to: getLocalDateString(new Date(y, m + 2, 0)),
  };
});

async function loadEvents() {
  try {
    const { from, to } = ladeFenster.value;
    const [evtRes, overlayRes] = await Promise.all([
      api.getEvents(from, to),
      api.getCalendarOverlay ? api.getCalendarOverlay(from, to) : { data: { sources: [], items: [] } },
    ]);
    events.value = evtRes.data;
    overlaySources.value = overlayRes.data.sources;
    overlayItems.value = overlayRes.data.items;
  } catch (err) {
    console.error(err);
  }
}

// Nachgeladen wird, wenn sich das FENSTER ändert – nicht bei jedem
// fokussierten Tag. Sieben Schritte durch die Woche innerhalb desselben
// Monats wären sonst sieben Anfragen für dieselben Daten.
watch(() => `${ladeFenster.value.from}|${ladeFenster.value.to}`, () => loadEvents());
onMounted(() => {
  const gemerkteAnsicht = localStorage.getItem('openany_calendar_view');
  if (gemerkteAnsicht === 'week' || gemerkteAnsicht === 'month') viewMode.value = gemerkteAnsicht;

  const stored = localStorage.getItem('openany_hidden_overlay');
  if (stored) {
    try { hiddenOverlayKeys.value = new Set(JSON.parse(stored)); }
    catch (e) { /* verworfen, Vorgabe bleibt leer (alles sichtbar) */ }
  }
  loadData();
});

// ---------- Tagesansicht ----------
async function openDay(dateStr) {
  // Ohne Tagesansicht (Programm): Ein Tipp auf den Tag legt einen Termin an,
  // statt eine leere Durchsicht zu öffnen.
  if (!api.getCalendarDay) { openAdd(dateStr); return; }
  dayDate.value = dateStr;
  dayLoading.value = true;
  try {
    dayData.value = (await api.getCalendarDay(dateStr)).data;
  } catch (err) {
    console.error(err);
    dayData.value = null;
  } finally {
    dayLoading.value = false;
  }
}

function shiftDay(delta) {
  const d = new Date(`${dayDate.value}T00:00:00`);
  d.setDate(d.getDate() + delta);
  openDay(getLocalDateString(d));
}

// Ein Tipp, nicht ein Formular: Fach, Fälligkeit und Zielspalte stehen schon
// fest. Angelegt wird in EINEM Aufruf – ein POST und ein nachgeschobenes
// PATCH wären zwei Rundreisen und zwei Broadcasts für eine Zeile.
async function addHomework({ projectId, target, subjectId, title }) {
  daySaving.value = true;
  try {
    await api.createCard(projectId, target.board_id, target.column_id, title, {
      subject_id: subjectId,
      due_date: dayDate.value,
    });
    await Promise.all([openDay(dayDate.value), loadEvents()]);
  } catch (err) {
    console.error(err);
    toast.error(t('calendar.day.addFailed'));
  } finally {
    daySaving.value = false;
  }
}

// Sprung ins Projekt, dieselbe Zielform wie ein Chat-Verweis. Das Heft ist
// die Mappe selbst (kein bestimmter Eintrag darin) – ProjectShares kennt
// `note_folder` als Zielart.
function openFromDay({ link, projectId }) {
  dayDate.value = null;
  emit('open-in-project', { link, projectId });
}

function openHeft({ folderId, projectId }) {
  openFromDay({ link: { kind: 'note_folder', id: folderId, share_root_id: folderId }, projectId });
}

function toggleOverlaySource(key) {
  const s = hiddenOverlayKeys.value;
  if (s.has(key)) s.delete(key); else s.add(key);
  hiddenOverlayKeys.value = new Set(s);
  localStorage.setItem('openany_hidden_overlay', JSON.stringify(Array.from(hiddenOverlayKeys.value)));
}

// ---------- Navigation ----------
// Ein Pfeilpaar, zwei Schrittweiten: In der Wochenansicht wäre ein
// Monatssprung ein Verlust der Stelle, an der man gerade liest.
function changeFocus(delta) {
  const d = new Date(focusedDate.value);
  if (viewMode.value === 'week') d.setDate(d.getDate() + delta * 7);
  else d.setMonth(d.getMonth() + delta);
  focusedDate.value = d;
}

function setViewMode(modus) {
  viewMode.value = modus;
  localStorage.setItem('openany_calendar_view', modus);
}
// ---------- Kalender-Sichtbarkeit ----------
function persistVisible() {
  localStorage.setItem('openany_visible_calendars', JSON.stringify(Array.from(visibleCalendarIds.value)));
}
function toggleCalendarVisibility(id) {
  const s = visibleCalendarIds.value;
  if (s.has(id)) s.delete(id); else s.add(id);
  visibleCalendarIds.value = new Set(s); // Ersatz statt Mutation für Reaktivität
  persistVisible();
}

// ---------- Kalender verwalten ----------
async function syncAllCalendars() {
  const syncable = calendars.value.filter(c => c.sync_url);
  isSyncing.value = true;
  try {
    for (const c of syncable) await api.syncCalendar(c.id);
    await loadData();
  } catch (err) {
    toast.error(t('calendar.calendar.syncFailed'));
  } finally {
    isSyncing.value = false;
  }
}

function openImport() {
  if (calendars.value.length === 0) {
    toast.error(t('calendar.calendar.createCalendarFirst'));
    return;
  }
  importOpen.value = true;
}

async function importFile({ calendarId, file }) {
  isImporting.value = true;
  try {
    await api.importICS(calendarId, file);
    importOpen.value = false;
    await loadEvents();
  } catch (err) {
    toast.error(t('calendar.calendar.importFileFailed'));
  } finally {
    isImporting.value = false;
  }
}

async function importUrl({ calendarId, url }) {
  isImporting.value = true;
  try {
    await api.updateCalendar(calendarId, { sync_url: url });
    await api.syncCalendar(calendarId);
    await loadData();
    importOpen.value = false;
  } catch (err) {
    toast.error(t('calendar.calendar.importUrlFailed'));
  } finally {
    isImporting.value = false;
  }
}

async function createCalendar(form) {
  isCreating.value = true;
  try {
    const res = await api.createCalendar(form);
    createOpen.value = false;
    await loadData();
    if (!visibleCalendarIds.value.has(res.data.id)) {
      visibleCalendarIds.value.add(res.data.id);
      visibleCalendarIds.value = new Set(visibleCalendarIds.value);
      persistVisible();
    }
  } catch (err) {
    toast.error(t('calendar.calendar.createCalendarFailed'));
  } finally {
    isCreating.value = false;
  }
}

/*
 * Name, Farbe und Abo-Adresse in einem Zug -- der Dialog schickt alle drei.
 *
 * Vorher ging nur die Farbe. Die Abo-Adresse liess sich einmal eintragen und
 * danach nie wieder ansehen; bei einem Tippfehler blieb nur, den Kalender zu
 * loeschen und neu anzulegen -- mit allen Terminen darin.
 */
async function updateCalendar(felder) {
  if (!editCal.value) return;
  isUpdatingColor.value = true;
  try {
    await api.updateCalendar(editCal.value.id, felder);
    editCal.value = null;
    await loadData();
  } catch (err) {
    // Die Meldung des Servers, wenn er eine hat: Bei einer abgelehnten
    // Adresse ("Nur HTTPS-URLs sind erlaubt") ist sie die eigentliche
    // Auskunft, und "Speichern fehlgeschlagen" verschweigt sie.
    const fehler = err?.response?.data;
    const genauer = fehler?.errors?.sync_url?.[0] ?? fehler?.message;
    toast.error(genauer || t('calendar.calendar.updateColorFailed'));
  } finally {
    isUpdatingColor.value = false;
  }
}

async function deleteCalendar(cal) {
  try {
    await api.deleteCalendar(cal.id);
    await loadData();
  } catch (err) {
    toast.error(t('calendar.calendar.deleteCalendarFailed'));
  }
}

function exportCal(cal) { api.exportICS(cal.id); }

// Abo-Link geändert (erzeugt/erneuert/widerrufen): Kalenderliste frisch
// ziehen, damit das Token im Sidebar-Zustand stimmt.
function feedUpdated() { loadData(); }

// ---------- Termine ----------
function openAdd(dateStr = '') {
  eventModal.value = { initialDate: dateStr, editEvent: null };
}
function openDetail(event) { detailEvent.value = event; }
function openOverlayDetail(item) { overlayDetail.value = item; }

// Sprung ins Projekt: dieselbe Zielform, die ein Chat-Verweis auflöst
// (kind, id, Feld des Behälters) – ProjectDetail liest sie aus der URL und
// öffnet damit genau das, was auch ein Klick im Chat öffnen würde.
function openInProject() {
  const { link, project_id } = overlayDetail.value;
  overlayDetail.value = null;
  emit('open-in-project', { link, projectId: project_id });
}

function editFromDetail(event) {
  detailEvent.value = null;
  eventModal.value = { initialDate: '', editEvent: event };
}

async function saveEvent({ payload, eventId }) {
  isSaving.value = true;
  try {
    if (eventId) await api.updateEvent(eventId, payload);
    else await api.createEvent(payload);
    eventModal.value = null;
    await loadEvents();
  } catch (err) {
    console.error(err);
    toast.error(t('calendar.calendar.saveEventFailed'));
  } finally {
    isSaving.value = false;
  }
}

async function deleteEvent(event) {
  if (!(await confirmDelete(t('calendar.calendar.confirmDeleteEvent')))) return;
  try {
    const eventId = event.real_id || event.id;
    await api.deleteEvent(eventId);
    detailEvent.value = null;
    await loadEvents();
  } catch (err) {
    toast.error(t('calendar.calendar.deleteEventFailed'));
  }
}
</script>

<template>
  <ModulePage>

    <!-- Titel + Toolbar -->
    <ModuleHeader :icon="Calendar">
      <!-- Monatsauswahl, Kalender-Umschalter und Termin – in einer Zeile (auch mobil) -->
      <!--
        EINE ZEILE, UND DER PLATZ KOMMT VON DEN KNÖPFEN.

        Auf 390 Pixeln stehen hier vier Gruppen nebeneinander. Vorher gewann
        in dieser Enge immer dasselbe: Die Beschriftung schrumpfte oder
        verschwand hinter den Knöpfen, obwohl sie die Angabe ist, wegen der
        die Zeile da ist.

        Drei Stellen geben den Platz her, und alle drei kosten nichts:
        „Termin" wird auf dem Telefon zum reinen Plus (der Knopf bleibt in
        seiner Farbe erkennbar), die Innenabstände der Knöpfe schrumpfen um
        ein paar Pixel, und die Beschriftung verliert ihre MINDESTBREITE –
        die war es, die den Abstand zwischen Pfeil und Monatsname erzeugte,
        weil der Text darin zentriert stand statt zu passen.

        `shrink-0` überall außer an der Beschriftung: Wenn es doch einmal
        eng wird (eine lange Wochenangabe auf einem schmalen Telefon), gibt
        genau eine Stelle nach, und zwar die mit den drei Punkten – nicht
        ein Knopf, der dann halb abgeschnitten dasteht.
      -->
      <div class="flex items-center gap-1.5 sm:gap-3 min-w-0">
        <div class="flex items-center h-9 bg-auflage rounded-xl p-0.5 sm:p-1 border border-slate-200/60 dark:border-slate-700 min-w-0">
          <button @click="changeFocus(-1)" class="p-0.5 sm:p-1 hover:bg-white dark:hover:bg-slate-900 rounded-xl text-fliess shrink-0"><ChevronLeft class="w-4 h-4" /></button>
          <span class="px-1 sm:px-4 text-xs font-extrabold text-schrift text-center min-w-0 sm:min-w-[150px] truncate">
            <span class="sm:hidden">{{ viewHeaderLabelShort }}</span>
            <span class="hidden sm:inline">{{ viewHeaderLabel }}</span>
          </span>
          <button @click="changeFocus(1)" class="p-0.5 sm:p-1 hover:bg-white dark:hover:bg-slate-900 rounded-xl text-fliess shrink-0"><ChevronRight class="w-4 h-4" /></button>
        </div>

        <!-- Monat/Woche: zwei Knöpfe statt eines Umschalters, damit die
             gezeigte Auflösung ablesbar ist, ohne sie zu betätigen. -->
        <div class="flex items-center h-9 bg-auflage rounded-xl p-0.5 sm:p-1 border border-slate-200/60 dark:border-slate-700 shrink-0">
          <button v-for="modus in ['month', 'week']" :key="modus" @click="setViewMode(modus)"
            class="px-1.5 sm:px-3 h-7 text-xs font-extrabold rounded-lg"
            :class="viewMode === modus
              ? 'bg-flaeche text-marke shadow-sm'
              : 'text-leise'">
            {{ modus === 'month' ? t('calendar.calendar.viewMonth') : t('calendar.calendar.viewWeek') }}
          </button>
        </div>

        <button @click="isSidebarOpen = !isSidebarOpen" :title="t('calendar.calendar.calendars')" class="flex items-center justify-center h-9 gap-1.5 px-2 sm:px-4 py-2 border border-linie bg-flaeche text-fliess text-sm font-bold rounded-xl shadow-sm hover:bg-auflage shrink-0" :class="isSidebarOpen ? 'ring-2 ring-marke' : ''"><CalendarDays class="w-4 h-4 shrink-0" /> <span class="hidden sm:inline">{{ t('calendar.calendar.calendars') }}</span></button>
        <BaseButton @click="openAdd()" :title="t('calendar.calendar.newEvent')" class="shrink-0" groesse="kopf"><Plus class="w-4 h-4 shrink-0" /> <span class="hidden sm:inline">{{ t('calendar.calendar.newEvent') }}</span></BaseButton>
      </div>
    </ModuleHeader>

    <div v-if="errorMessage" class="flex items-start gap-3 bg-rose-50 border border-rose-200 text-rose-700 rounded-xl px-4 py-3 text-sm font-medium">
      <AlertCircle class="w-5 h-5 text-rose-500" /> <span>{{ errorMessage }}</span>
      <button @click="errorMessage = ''" class="ml-auto"><X class="w-4 h-4" /></button>
    </div>

    <CalendarSidebar
      v-if="isSidebarOpen"
      :calendars="calendars"
      :visible-calendar-ids="visibleCalendarIds"
      :is-syncing="isSyncing"
      :is-loading="isLoading"
      :overlay-sources="overlaySources"
      :visible-overlay-keys="visibleOverlayKeys"
      :kann="kann"
      @toggle="toggleCalendarVisibility"
      @sync="syncAllCalendars"
      @import="openImport"
      @create="createOpen = true"
      @edit-color="editCal = $event"
      @export="exportCal"
      @feed="feedCal = $event"
      @delete="deleteCalendar"
      @toggle-overlay="toggleOverlaySource"
    />

    <CalendarWeekGrid
      v-if="viewMode === 'week'"
      :week-days="weekDays"
      :hours="hours"
      :all-day-map="allDayMap"
      :timed-map="timedMap"
      @add="openAdd"
      @open="openDetail"
      @open-overlay="openOverlayDetail"
      @open-day="openDay"
    />
    <CalendarMonthGrid v-else :grid="calendarGrid" :events-map="gridEventsMap" :overlay-map="overlayEventsMap" @add="openAdd" @open="openDetail" @open-overlay="openOverlayDetail" @open-day="openDay" />
  </ModulePage>

  <CalendarDayView
    v-if="dayDate"
    :date="dayDate"
    :bloecke="dayBloecke"
    :ist-leer="dayLeer"
    :loading="dayLoading"
    :saving="daySaving"
    @close="dayDate = null"
    @shift="shiftDay"
    @open-in-project="openFromDay"
    @open-heft="openHeft"
    @add-homework="addHomework"
  />

  <!-- Modals -->
  <CalendarEventModal
    v-if="eventModal"
    :calendars="beschreibbare"
    :initial-date="eventModal.initialDate"
    :edit-event="eventModal.editEvent"
    :saving="isSaving"
    @save="saveEvent"
    @close="eventModal = null"
  />
  <CalendarEventDetail
    v-if="detailEvent"
    :event="detailEvent"
    :abo="kalenderVon(detailEvent)?.sync_url ? kalenderVon(detailEvent).name : null"
    @close="detailEvent = null"
    @edit="editFromDetail"
    @delete="deleteEvent"
  />
  <component
    :is="erweiterungen.overlay"
    v-if="overlayDetail && erweiterungen.overlay"
    :item="overlayDetail"
    @close="overlayDetail = null"
    @open-in-project="openInProject"
  />
  <CalendarCreateModal
    v-if="createOpen"
    :saving="isCreating"
    @save="createCalendar"
    @close="createOpen = false"
  />
  <CalendarEditModal
    v-if="editCal"
    :calendar="editCal"
    :saving="isUpdatingColor"
    :kann-abo="Boolean(kann.sync)"
    @save="updateCalendar"
    @close="editCal = null"
  />
  <component
    :is="erweiterungen.feed"
    v-if="feedCal && erweiterungen.feed"
    :calendar="feedCal"
    @updated="feedUpdated"
    @close="feedCal = null"
  />
  <CalendarImportModal
    v-if="importOpen"
    :calendars="calendars"
    :loading="isImporting"
    @import-file="importFile"
    @import-url="importUrl"
    @close="importOpen = false"
  />
</template>
