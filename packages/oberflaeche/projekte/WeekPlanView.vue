<script setup>
// Detailansicht EINES Wochenplans – Stundenplan oder Betreuungsplan. Beide
// sind derselbe Behälter (`week_plans.type`), unterschieden durch das, was
// darin steht: Uhrzeiten und Fächer beim einen, ganze Tage und Menschen beim
// anderen.
//
// Das Raster ist ABSICHTLICH keine Zeitachse in Pixeln. Was hier gebraucht
// wird, ist Eingeben und Nachschlagen – „welche Stunden hat Lena montags?" –,
// und dafür liest sich eine Spalte je Wochentag mit ihren Einträgen der Reihe
// nach besser als Kästchen, die je nach Länge verrutschen. Die maßstäbliche
// Wochenansicht kommt später im Kalender (useCalendarWeek), wo sie hingehört:
// dort liegen Stundenplan, Betreuung und die eigenen Termine übereinander.
//
// Zwei Dinge, die man leicht für Zierrat hält und die es nicht sind:
//
//   Der A/B-Umschalter erscheint nur bei `cycle_weeks = 2`. Ohne
//     zweiwöchigen Rhythmus gäbe es nichts umzuschalten, und ein toter
//     Schalter lädt dazu ein, ihn zu suchen.
//   Die Kopfzeile nennt `applies_to` im Klartext. Ein Betreuungsplan mit
//     „nur Ferien" ist die halbe Wahrheit – ohne den zweiten Plan wäre der
//     Kalender die restliche Zeit leer, und das sieht man dem Raster nicht an.
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { api, kann } from './umgebung';
import PlacePicker from './PlacePicker.vue';
import SubjectPicker from './SubjectPicker.vue';
import SubjectsPanel from './SubjectsPanel.vue';
import PlanFeedModal from './PlanFeedModal.vue';
import {
  Plus, RefreshCcw, Trash2, ArrowLeft, Loader2, X, Pencil,
  CalendarDays, Users, Settings2, MapPin, Clock, CalendarRange, BookOpen, Rss,
} from 'lucide-vue-next';
import { useToast } from '@oberflaeche/composables/useToast';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import { initEcho, onEchoReconnect } from './umgebung';
import { debounce } from '@oberflaeche/shared/debounce';
import { formatDateLong } from '@oberflaeche/shared/date';
import { STUNDENRASTER, RASTER_KEYS, stundeZuZeit } from '@oberflaeche/shared/stundenraster';
import { useHighlight } from '@oberflaeche/composables/useHighlight';
import BaseButton from '@oberflaeche/base/BaseButton.vue';

const { t, locale } = useI18n();
const toast = useToast();
const { confirmDelete, confirmDialog } = useConfirm();
const props = defineProps({
  projectId: { type: [Number, String], required: true },
  containerId: { type: [Number, String], required: true },
  meId: { type: Number, default: null },
  isOwner: { type: Boolean, default: false },
  members: { type: Array, default: () => [] },
  highlightId: { type: Number, default: null },
});
const emit = defineEmits(['close']);

const loading = ref(true);
const error = ref('');
const plan = ref(null);
const slots = ref([]);
const perioden = ref([]);
const woche = ref(0);            // 0 = A, 1 = B
const slotModal = ref(null);
const periodModal = ref(null);
const settingsModal = ref(null);
const schuljahre = ref([]);
const saving = ref(false);
const subjectsPanelOpen = ref(false);
// Der Abo-Link hängt am TYP dieses Plans, nicht am Plan selbst – deshalb
// steht der Knopf hier, öffnet aber das Abo für Stundenplan bzw. Betreuung
// des ganzen Projekts.
const feedModalOpen = ref(false);

const { highlighted, merkeEintrag } = useHighlight(
  () => props.highlightId,
  () => slots.value.length,
);

// Sechs Farben plus „keine". Ein freier Farbwähler wäre hier kein Gewinn:
// Der Nutzen der Farbe ist, dass zwei Eltern auf einen Blick auseinandergehen
// – dafür braucht es unterscheidbare Töne, nicht beliebige.
const FARBEN = ['#6366f1', '#ec4899', '#f59e0b', '#10b981', '#0ea5e9', '#8b5cf6'];

// Die Wochentags-Namen kommen aus der Sprache des Browsers statt aus vierzehn
// Übersetzungsschlüsseln. Der 1.1.2024 war ein Montag – von da an sieben Tage.
const wochentage = computed(() => {
  const kurz = new Intl.DateTimeFormat(locale.value, { weekday: 'short' });
  const lang = new Intl.DateTimeFormat(locale.value, { weekday: 'long' });
  return [1, 2, 3, 4, 5, 6, 7].map((n) => {
    const tag = new Date(Date.UTC(2024, 0, n));
    return { n, kurz: kurz.format(tag), lang: lang.format(tag) };
  });
});

// Das Stundenraster des Plans: die Uhrzeiten, nach denen diese Schule ihren
// Tag teilt. Steht eins da, wird im Eintrags-Dialog nicht mehr nach „09:55
// bis 10:40" gefragt, sondern nach der 3. Stunde – dreißigmal im Schuljahr
// ist das der Unterschied zwischen Eintragen und Abtippen.
const stundenraster = computed(() => plan.value?.settings?.periods || []);

const stundenLabel = (i) => {
  const p = stundenraster.value[i];

  return p ? t('projects.weekPlan.lessonNumber', { n: i + 1, from: p.from, to: p.to }) : '';
};

const istBetreuung = computed(() => plan.value?.type === 'care');

// Wie die beiden Wochen heißen, hängt an der ZÄHLWEISE, nicht am Behälter:
// „A-Woche" ist Schulsprache für einen Rhythmus ab einem Stichtag; wo nach
// Kalenderwochen gezählt wird, heißen sie so, wie es in der Vereinbarung
// steht — gerade und ungerade.
const wochenNamen = computed(() => (plan.value?.cycle_mode === 'iso_week'
  ? [t('projects.weekPlan.weekEven'), t('projects.weekPlan.weekOdd')]
  : [t('projects.weekPlan.weekA'), t('projects.weekPlan.weekB')]));
const zweiWochen = computed(() => (plan.value?.cycle_weeks ?? 1) > 1);

// WELCHE der beiden Wochen gerade läuft. Ohne diese Auskunft sind „A-Woche"
// und „B-Woche" zwei Namen ohne Bezug – man sieht zwei Raster und weiß nicht,
// welches heute gilt. Die Rechnung ist dieselbe wie in
// `WeekPlan::wochenIndex()`: beide Daten auf ihren Montag ziehen, Abstand in
// Wochen, modulo Zykluslänge. Sie steht hier ein zweites Mal, weil es um eine
// BESCHRIFTUNG geht — die Auffaltung selbst bleibt allein beim Server.
const laufendeWoche = computed(() => {
  if (!zweiWochen.value) return null;

  // Nach Kalenderwochen gezählt: gerade = 0, ungerade = 1. Die ISO-Woche
  // hängt am Donnerstag derselben Woche – deshalb nicht der einfache
  // Tagesabstand, sondern der Umweg über ihn.
  if (plan.value.cycle_mode === 'iso_week') {
    const heute = new Date();
    const donnerstag = new Date(heute.getFullYear(), heute.getMonth(), heute.getDate());
    donnerstag.setDate(donnerstag.getDate() - ((donnerstag.getDay() + 6) % 7) + 3);
    const erster = new Date(donnerstag.getFullYear(), 0, 4);
    const kw = 1 + Math.round(((donnerstag - erster) / 86400000 - 3 + ((erster.getDay() + 6) % 7)) / 7);

    return kw % plan.value.cycle_weeks;
  }

  if (!plan.value?.cycle_anchor) return null;

  const montagVon = (d) => {
    const m = new Date(d.getFullYear(), d.getMonth(), d.getDate());
    m.setDate(m.getDate() - ((m.getDay() + 6) % 7));
    return m;
  };

  const anker = montagVon(new Date(`${plan.value.cycle_anchor.slice(0, 10)}T00:00:00`));
  const heute = montagVon(new Date());
  const wochen = Math.floor((heute - anker) / (7 * 24 * 60 * 60 * 1000));
  const zyklus = plan.value.cycle_weeks;

  return ((wochen % zyklus) + zyklus) % zyklus;
});

// Nur die Einträge der gezeigten Woche, je Wochentag. Sortiert wird schon
// serverseitig; hier wird nur verteilt.
// Ein STUNDENPLAN zeigt Mo–Fr, ein Betreuungsplan Mo–So. Das ist keine
// Annahme über die Woche, sondern über den Behälter: An Samstagen ist keine
// Schule, zwei leere Spalten kosten aber ein Fünftel der Breite – und in der
// Betreuung ist das Wochenende gerade das Herzstück („wo schläft das Kind
// Freitag?").
//
// Ausgeblendet wird nur, was leer IST: Steht doch ein Eintrag am Wochenende
// (Samstagsunterricht, eine AG), kommen die Spalten von selbst zurück. So
// verschwinden keine Daten hinter einer Vorgabe, und wer bewusst etwas
// eintragen will, schaltet das Wochenende mit einem Knopf dazu.
const wochenendeZeigen = ref(false);

const wochenendeBelegt = computed(() => slots.value.some((s) => s.weekday >= 6));

const sichtbareTage = computed(() => (
  istBetreuung.value || wochenendeZeigen.value || wochenendeBelegt.value
    ? wochentage.value
    : wochentage.value.filter((tag) => tag.n <= 5)
));

const raster = computed(() => sichtbareTage.value.map((tag) => ({
  ...tag,
  eintraege: slots.value.filter((s) => s.weekday === tag.n && (s.week_index ?? 0) === woche.value),
})));

const mitgliedName = (id) => props.members.find((m) => m.user_id === id)?.name || '';

const zeitraum = (p) => (p.starts_on === p.ends_on
  ? formatDateLong(p.starts_on)
  : `${formatDateLong(p.starts_on)} – ${formatDateLong(p.ends_on)}`);

const load = async () => {
  loading.value = true; error.value = '';
  try {
    plan.value = (await api.getWeekPlan(props.projectId, props.containerId)).data;
    slots.value = plan.value.slots || [];
    perioden.value = plan.value.periods || [];
    // Eine B-Woche, die es nicht mehr gibt, darf nicht offen bleiben.
    if (!zweiWochen.value) woche.value = 0;
    // Aufgeschlagen wird die Woche, die GERADE läuft – nicht immer A. Wer
    // nachsieht, will in aller Regel wissen, was diese Woche gilt.
    else woche.value = laufendeWoche.value ?? 0;
  } catch (e) { error.value = t('projects.weekPlan.loadFailed'); }
  finally { loading.value = false; }
};

// --- Raster-Einträge ---

const openSlotCreate = (weekday) => {
  slotModal.value = {
    weekday,
    week_index: woche.value,
    // Der Betreuungsplan besteht aus ganzen Tagen, der Stundenplan aus
    // Uhrzeiten. Die Vorbelegung nimmt dem häufigen Fall einen Klick ab.
    is_all_day: istBetreuung.value,
    starts_at: istBetreuung.value ? '' : '08:00',
    ends_at: istBetreuung.value ? '' : '08:45',
    subject_id: null, title: '', subtitle: '', place_id: null, color: null,
    assigned_to: istBetreuung.value ? (props.meId ?? null) : null,
  };
};

// Welche Stunde des Rasters gerade eingestellt ist – oder null („freie
// Zeit"). Abgeleitet statt gespeichert: Am Slot stehen Uhrzeiten, nicht
// Stundennummern. Verschiebt die Schule die dritte Stunde, verschieben sich
// alle Einträge mit, ohne dass etwas umgetragen werden müsste.
const gewaehlteStunde = computed({
  get: () => (slotModal.value ? stundeZuZeit(stundenraster.value, slotModal.value.starts_at) : null),
  set: (i) => {
    if (i === null || i === '') return;
    const p = stundenraster.value[Number(i)];
    if (!p) return;
    slotModal.value.starts_at = p.from;
    slotModal.value.ends_at = p.to;
  },
});

const openSlotEdit = (s) => {
  slotModal.value = {
    id: s.id,
    weekday: s.weekday,
    week_index: s.week_index ?? 0,
    is_all_day: s.is_all_day,
    starts_at: s.starts_at || '',
    ends_at: s.ends_at || '',
    subject_id: s.subject_id ?? null,
    title: s.title || '', subtitle: s.subtitle || '',
    place_id: s.place_id ?? null, color: s.color || null,
    assigned_to: s.assigned_to ?? null,
  };
};

const submitSlot = async () => {
  const m = slotModal.value;
  if (!m.is_all_day && !m.starts_at) { toast.error(t('projects.weekPlan.slotNeedsTime')); return; }
  saving.value = true;
  const payload = {
    weekday: m.weekday,
    week_index: m.week_index,
    is_all_day: m.is_all_day,
    // Ein ganztägiger Eintrag trägt keine Uhrzeit – sonst stünde sie in der
    // Datenbank und wirkte nirgends.
    starts_at: m.is_all_day ? null : (m.starts_at || null),
    ends_at: m.is_all_day ? null : (m.ends_at || null),
    subject_id: m.subject_id,
    title: m.title.trim() || null,
    subtitle: m.subtitle.trim() || null,
    place_id: m.place_id,
    color: m.color,
    assigned_to: m.assigned_to,
  };
  try {
    if (m.id) await api.updateWeekPlanSlot(props.projectId, props.containerId, m.id, payload);
    else await api.createWeekPlanSlot(props.projectId, props.containerId, payload);
    slotModal.value = null;
    await load();
  } catch (e) {
    toast.error(e.response?.data?.message || t('projects.weekPlan.saveFailed'));
  } finally { saving.value = false; }
};

const removeSlot = async (s) => {
  if (!(await confirmDelete(t('projects.weekPlan.deleteSlotConfirm', { name: slotName(s) })))) return;
  try {
    await api.deleteWeekPlanSlot(props.projectId, props.containerId, s.id);
    await load();
  } catch (e) { toast.error(t('projects.weekPlan.deleteFailed')); }
};

// Woran ein Eintrag zu erkennen ist: am Fach, sonst am Titel (Rückfall für
// AG, Mittagsband, Freistunde), beim Betreuungsplan an der Person. Fehlt
// alles, bleibt die Uhrzeit.
const slotName = (s) => s.subject?.name
  || s.title
  || (s.assigned_to ? mitgliedName(s.assigned_to) : '')
  || s.starts_at
  || t('projects.weekPlan.allDay');

// Der Ort am Slot schlägt den Ort am Fach: Der Raum stimmt meistens, aber
// nicht immer (Vertretungsraum, Sporthalle statt Klassenzimmer).
const slotPlace = (s) => s.place || s.subject?.place || null;

// --- Abschnitte ---

const heute = () => new Date().toISOString().slice(0, 10);

const openPeriodCreate = () => {
  periodModal.value = {
    title: '', starts_on: heute(), ends_on: heute(),
    assigned_to: istBetreuung.value ? (props.meId ?? null) : null,
    place_id: null, color: null, note: '',
  };
};
const openPeriodEdit = (p) => {
  periodModal.value = {
    id: p.id, title: p.title, starts_on: p.starts_on, ends_on: p.ends_on,
    assigned_to: p.assigned_to ?? null, place_id: p.place_id ?? null,
    color: p.color || null, note: p.note || '',
  };
};

const submitPeriod = async () => {
  const m = periodModal.value;
  if (!m.title.trim() || !m.starts_on || !m.ends_on) { toast.error(t('projects.weekPlan.missingFields')); return; }
  if (m.ends_on < m.starts_on) { toast.error(t('projects.weekPlan.endBeforeStart')); return; }
  saving.value = true;
  const payload = {
    title: m.title.trim(), starts_on: m.starts_on, ends_on: m.ends_on,
    assigned_to: m.assigned_to, place_id: m.place_id, color: m.color,
    note: m.note.trim() || null,
  };
  try {
    if (m.id) await api.updateWeekPlanPeriod(props.projectId, props.containerId, m.id, payload);
    else await api.createWeekPlanPeriod(props.projectId, props.containerId, payload);
    periodModal.value = null;
    await load();
  } catch (e) {
    toast.error(e.response?.data?.message || t('projects.weekPlan.saveFailed'));
  } finally { saving.value = false; }
};

const removePeriod = async (p) => {
  if (!(await confirmDelete(t('projects.weekPlan.deletePeriodConfirm', { name: p.title })))) return;
  try {
    await api.deleteWeekPlanPeriod(props.projectId, props.containerId, p.id);
    await load();
  } catch (e) { toast.error(t('projects.weekPlan.deleteFailed')); }
};

// --- Einstellungen ---

// Eine Vorlage ERSETZT das Raster, sie ergänzt es nicht: Zwei
// übereinandergelegte Raster wären eine Liste, die niemand mehr lesen kann.
const rasterVorlage = (key) => {
  settingsModal.value.stunden = STUNDENRASTER[key]().map((x) => ({ ...x }));
};

const stundeHinzufuegen = () => {
  const letzte = settingsModal.value.stunden.at(-1);
  settingsModal.value.stunden.push({ from: letzte?.to || '08:00', to: '' });
};

const stundeEntfernen = (i) => settingsModal.value.stunden.splice(i, 1);

const openSettings = async () => {
  const p = plan.value;
  settingsModal.value = {
    name: p.name, type: p.type, applies_to: p.applies_to,
    school_year_id: p.school_year_id ?? null,
    // Zwei Felder in der Datenbank, EINE Frage in der Oberfläche: „wie oft
    // wiederholt sich der Plan?" – die Aufteilung in Zykluslänge und
    // Zählweise ist eine Sache der Speicherung, keine, die jemand beim
    // Ausfüllen sortieren sollte.
    rhythmus: p.cycle_weeks > 1 ? (p.cycle_mode === 'iso_week' ? 'iso_week' : 'anchor') : 'einfach',
    cycle_anchor: p.cycle_anchor || '',
    stunden: (p.settings?.periods || []).map((x) => ({ ...x })),
    valid_from: p.valid_from || '', valid_to: p.valid_to || '',
    timezone: p.timezone,
  };
  // Die Ferienliste wird geteilt, nicht kopiert: Hier wird nur ausgewählt,
  // welches Schuljahr dieser Plan liest.
  try { schuljahre.value = (await api.getSchoolYears(props.projectId)).data; }
  catch (e) { schuljahre.value = []; }
};

const submitSettings = async () => {
  const m = settingsModal.value;
  if (!m.name.trim()) { toast.error(t('projects.weekPlan.missingFields')); return; }
  if (m.rhythmus === 'anchor' && !m.cycle_anchor) { toast.error(t('projects.weekPlan.anchorNeeded')); return; }

  // Zurück auf eine Woche heißt: Die Einträge der B-Woche stehen weiter in
  // der Datenbank, gelten aber nie mehr und sind auch nicht mehr zu sehen.
  // Sie wegzuwerfen wäre schlimmer, sie stillschweigend liegen zu lassen
  // auch – also wird gefragt.
  const inB = slots.value.filter((s) => (s.week_index ?? 0) === 1).length;
  if (m.rhythmus === 'einfach' && inB > 0
    && !(await confirmDialog(t('projects.weekPlan.dropBWeekConfirm', { count: inB }), {
      title: t('projects.weekPlan.dropBWeekTitle'),
    }))) return;

  saving.value = true;
  try {
    await api.updateWeekPlan(props.projectId, props.containerId, {
      name: m.name.trim(), type: m.type, applies_to: m.applies_to,
      school_year_id: m.school_year_id,
      cycle_weeks: m.rhythmus === 'einfach' ? 1 : 2,
      cycle_mode: m.rhythmus === 'iso_week' ? 'iso_week' : 'anchor',
      // Nach Kalenderwochen gezählt hätte ein Ankermontag nichts zu
      // bestimmen – die Woche trägt ihre Nummer selbst.
      cycle_anchor: m.rhythmus === 'anchor' ? m.cycle_anchor : null,
      // Das Raster liegt in `settings` – ein Feld, das die Migration von
      // Anfang an dafür vorgesehen hat. Leere Zeilen fallen raus, damit eine
      // halb ausgefüllte Zeile nicht als Stunde ohne Zeit zurückkommt.
      settings: {
        ...(plan.value?.settings || {}),
        periods: m.stunden.filter((x) => x.from && x.to),
      },
      valid_from: m.valid_from || null, valid_to: m.valid_to || null,
      timezone: m.timezone,
    });
    settingsModal.value = null;
    await load();
  } catch (e) {
    toast.error(e.response?.data?.message || t('projects.weekPlan.saveFailed'));
  } finally { saving.value = false; }
};

const debouncedReload = debounce(() => load(), 400);
const onLive = (payload) => { if (payload?.kind === 'week_plan') debouncedReload(); };
const channel = `project.${props.projectId}`;
let offReconnect = null;

onMounted(() => {
  load();
  if (initEcho()) {
    initEcho().private(channel).listen('ProjectPlanningChanged', onLive);
    offReconnect = onEchoReconnect(() => load());
  }
});
onUnmounted(() => {
  initEcho()?.private(channel).stopListening('ProjectPlanningChanged', onLive);
  offReconnect?.();
});
</script>

<template>
  <div class="karte shadow-sm p-6 space-y-5">
    <div class="flex items-center gap-2 flex-wrap">
      <button @click="emit('close')" class="p-2 text-leise hover:bg-auflage rounded-xl" :title="t('common.back')"><ArrowLeft class="w-4 h-4" /></button>
      <component :is="istBetreuung ? Users : CalendarDays" class="w-5 h-5 text-marke" />
      <h3 class="font-extrabold text-lg text-schrift truncate">{{ plan?.name || t('projects.weekPlan.title') }}</h3>
      <button @click="subjectsPanelOpen = true" v-if="plan && !istBetreuung" class="ml-auto flex items-center gap-1.5 px-3 py-2 text-fliess hover:bg-auflage rounded-xl text-sm font-bold" :title="t('projects.subjects.title')">
        <BookOpen class="w-4 h-4" /> {{ t('projects.subjects.title') }}
      </button>
      <button @click="feedModalOpen = true" v-if="plan && kann('planAbo')" class="flex items-center gap-1.5 px-3 py-2 text-fliess hover:bg-auflage rounded-xl text-sm font-bold" :class="{ 'ml-auto': istBetreuung }" :title="t(`projects.planFeed.title.${plan.type}`)">
        <Rss class="w-4 h-4" /> {{ t('projects.planFeed.button') }}
      </button>
      <button @click="openSettings" v-if="plan" class="flex items-center gap-1.5 px-3 py-2 text-fliess hover:bg-auflage rounded-xl text-sm font-bold" :class="{ 'ml-auto': istBetreuung && !kann('planAbo') }" :title="t('projects.weekPlan.settingsTitle')">
        <Settings2 class="w-4 h-4" /> {{ t('projects.weekPlan.settings') }}
      </button>
      <button @click="load" class="p-2 text-slate-400 hover:bg-auflage rounded-xl" :title="t('projects.weekPlan.title')"><RefreshCcw class="w-4 h-4" :class="loading ? 'animate-spin text-marke' : ''" /></button>
    </div>

    <!-- Was dieser Plan überhaupt abdeckt. Ohne diese Zeile sähe ein
         Ferien-Betreuungsplan aus wie ein leeres Raster. -->
    <div v-if="plan" class="flex items-center gap-2 flex-wrap text-xs font-semibold">
      <span class="px-2 py-0.5 rounded-full bg-marke-leise text-marke">
        {{ t(`planning.templates.${istBetreuung ? 'betreuungsplan' : 'stundenplan'}.label`) }}
      </span>
      <span class="px-2 py-0.5 rounded-full bg-auflage text-leise">
        {{ t(`projects.weekPlan.appliesTo.${plan.applies_to}`) }}
      </span>
      <span v-if="zweiWochen" class="px-2 py-0.5 rounded-full bg-auflage text-leise">
        {{ t('projects.weekPlan.abRhythm') }}
      </span>
      <span class="text-slate-400 dark:text-slate-500">{{ plan.timezone }}</span>
      <span v-if="plan.valid_from || plan.valid_to" class="text-slate-400 dark:text-slate-500">
        · {{ formatDateLong(plan.valid_from) }} – {{ formatDateLong(plan.valid_to) }}
      </span>
      <!-- Ein Plan, der auf Schultage oder Ferien zeigt, aber kein Schuljahr
           kennt, kann nicht wissen, wann das ist – er würde immer gelten. -->
      <span v-if="plan.applies_to !== 'all' && !plan.school_year_id"
        class="px-2 py-0.5 rounded-full bg-rose-50 dark:bg-rose-900/30 text-rose-600 dark:text-rose-400">
        {{ t('projects.weekPlan.noSchoolYear') }}
      </span>
    </div>

    <div v-if="loading" class="py-10 text-center text-slate-400"><Loader2 class="w-6 h-6 animate-spin mx-auto" /></div>
    <p v-else-if="error" class="text-sm text-rose-500 font-medium">{{ error }}</p>

    <template v-else-if="plan">
      <!-- A/B-Umschalter, nur wenn es zwei Wochen gibt -->
      <div v-if="zweiWochen" class="flex items-center gap-1 p-1 bg-auflage rounded-xl w-fit">
        <button v-for="(bez, i) in wochenNamen" :key="i"
          @click="woche = i"
          class="px-3 py-1.5 rounded-lg text-sm font-bold transition-colors"
          :class="woche === i ? 'bg-flaeche text-marke shadow-sm' : 'text-leise'">
          {{ bez }}<span v-if="laufendeWoche === i" class="font-medium opacity-70"> · {{ t('projects.weekPlan.currentWeek') }}</span>
        </button>
      </div>

      <!-- Das Raster. Waagerecht scrollbar statt gequetscht: Sieben Spalten
           auf einem Handy wären sonst je 45 Pixel breit. -->
      <!-- Nur beim Stundenplan und nur, solange das Wochenende wirklich leer
           ist – sonst steht es ohnehin da. -->
      <button v-if="!istBetreuung && !wochenendeBelegt" @click="wochenendeZeigen = !wochenendeZeigen"
        class="mb-2 text-xs font-bold text-leise hover:text-marke">
        {{ wochenendeZeigen ? t('projects.weekPlan.hideWeekend') : t('projects.weekPlan.showWeekend') }}
      </button>

      <div class="overflow-x-auto -mx-2 px-2">
        <div class="grid gap-2" :class="raster.length === 7 ? 'grid-cols-7 min-w-[52rem]' : 'grid-cols-5 min-w-[38rem]'">
          <div v-for="tag in raster" :key="tag.n" class="space-y-2">
            <p class="text-xs font-extrabold text-leise uppercase text-center py-1">{{ tag.kurz }}</p>

            <div v-for="s in tag.eintraege" :key="s.id"
              :ref="(el) => merkeEintrag(s.id, el)"
              class="group relative px-2.5 py-2 rounded-xl border text-left transition-colors"
              :class="s.id === highlighted
                ? 'bg-marke-leise border-marke'
                : 'bg-vertieft border-linie'"
              :style="(s.color || s.subject?.color) ? { borderLeftWidth: '4px', borderLeftColor: s.color || s.subject.color } : null">
              <p v-if="!s.is_all_day" class="text-[11px] font-bold text-leise flex items-center gap-1">
                <Clock class="w-3 h-3 shrink-0" />{{ s.starts_at }}<span v-if="s.ends_at">–{{ s.ends_at }}</span>
              </p>
              <p v-else class="text-[11px] font-bold text-slate-400">{{ t('projects.weekPlan.allDay') }}</p>
              <!-- Das Fach schlägt den freien Titel – der bleibt Rückfall
                   für AG, Mittagsband, Freistunde. -->
              <p v-if="s.subject" class="text-sm font-bold text-schrift truncate">{{ s.subject.short || s.subject.name }}</p>
              <p v-else-if="s.title" class="text-sm font-bold text-schrift truncate">{{ s.title }}</p>
              <p v-if="s.assigned_to" class="text-xs font-semibold text-marke truncate">{{ mitgliedName(s.assigned_to) }}</p>
              <p v-if="s.subject?.teacher" class="text-[11px] text-leise truncate">{{ s.subject.teacher }}</p>
              <p v-else-if="s.subtitle" class="text-[11px] text-leise truncate">{{ s.subtitle }}</p>
              <p v-if="slotPlace(s)" class="text-[11px] text-slate-400 flex items-center gap-1 truncate"><MapPin class="w-3 h-3 shrink-0" />{{ slotPlace(s).name }}</p>

              <div class="absolute top-1 right-1 hidden group-hover:flex items-center gap-0.5 bg-white/90 dark:bg-slate-900/90 rounded-lg">
                <button @click="openSlotEdit(s)" class="p-1 text-slate-400 hover:text-marke rounded-lg" :title="t('projects.weekPlan.editSlot')"><Pencil class="w-3.5 h-3.5" /></button>
                <button @click="removeSlot(s)" class="p-1 text-slate-400 hover:text-rose-500 rounded-lg" :title="t('projects.weekPlan.deleteSlot')"><Trash2 class="w-3.5 h-3.5" /></button>
              </div>
            </div>

            <button @click="openSlotCreate(tag.n)"
              class="w-full py-1.5 rounded-xl border border-dashed border-linie text-slate-400 hover:border-marke hover:text-marke flex items-center justify-center"
              :aria-label="t('projects.weekPlan.addSlotFor', { day: tag.lang })" :title="t('projects.weekPlan.addSlotFor', { day: tag.lang })">
              <Plus class="w-4 h-4" />
            </button>
          </div>
        </div>
      </div>

      <!-- Abschnitte: sie überschreiben das Raster in ihrem Zeitraum -->
      <div class="pt-4 border-t border-linie space-y-3">
        <div class="flex items-center gap-2">
          <CalendarRange class="w-4 h-4 text-marke" />
          <h4 class="font-extrabold text-schrift">{{ t('projects.weekPlan.periods') }}</h4>
          <button @click="openPeriodCreate" class="ml-auto flex items-center gap-1.5 px-3 py-1.5 text-marke hover:bg-marke-leise rounded-xl text-sm font-bold"><Plus class="w-4 h-4" /> {{ t('projects.weekPlan.addPeriod') }}</button>
        </div>
        <p class="text-xs text-leise">{{ t('projects.weekPlan.periodsHint') }}</p>

        <p v-if="perioden.length === 0" class="py-4 text-center text-sm text-slate-400 dark:text-slate-500 font-medium">{{ t('projects.weekPlan.noPeriods') }}</p>
        <ul v-else class="space-y-2">
          <li v-for="p in perioden" :key="p.id"
            class="flex items-center gap-3 px-3 py-2.5 rounded-xl bg-vertieft border border-linie"
            :style="p.color ? { borderLeftWidth: '4px', borderLeftColor: p.color } : null">
            <div class="min-w-0 flex-1">
              <div class="flex items-center gap-2 flex-wrap">
                <span class="font-bold text-schrift truncate">{{ p.title }}</span>
                <span v-if="p.assigned_to" class="text-xs font-bold px-2 py-0.5 rounded-full bg-marke-leise text-marke">{{ mitgliedName(p.assigned_to) }}</span>
              </div>
              <p class="text-xs font-semibold text-leise mt-0.5">
                {{ zeitraum(p) }}
                <span v-if="p.place" class="ml-2 text-slate-400 inline-flex items-center gap-1"><MapPin class="w-3 h-3" />{{ p.place.name }}</span>
              </p>
              <p v-if="p.note" class="text-xs text-leise mt-0.5">{{ p.note }}</p>
            </div>
            <button @click="openPeriodEdit(p)" class="p-1.5 text-slate-400 hover:text-marke hover:bg-marke-leise rounded-xl" :title="t('projects.weekPlan.editPeriod')"><Pencil class="w-4 h-4" /></button>
            <button @click="removePeriod(p)" class="p-1.5 text-slate-400 hover:text-rose-500 hover:bg-rose-50 dark:hover:bg-rose-900/20 rounded-xl" :title="t('projects.weekPlan.deletePeriod')"><Trash2 class="w-4 h-4" /></button>
          </li>
        </ul>
      </div>
    </template>

    <!-- ===== Raster-Eintrag ===== -->
    <div v-if="slotModal" class="fixed inset-0 bg-slate-900/60 backdrop-blur-sm flex items-center justify-center p-4 z-[100]" @click.self="slotModal = null">
      <div class="karte w-full max-w-lg shadow-2xl max-h-[90vh] overflow-y-auto">
        <div class="flex items-center justify-between px-6 py-4 border-b border-linie">
          <h3 class="font-extrabold text-schrift flex items-center gap-2">
            <component :is="istBetreuung ? Users : CalendarDays" class="w-5 h-5 text-marke" />
            {{ slotModal.id ? t('projects.weekPlan.editSlot') : t('projects.weekPlan.newSlot') }}
          </h3>
          <button @click="slotModal = null" class="p-1.5 text-slate-400 hover:bg-auflage rounded-xl"><X class="w-5 h-5" /></button>
        </div>
        <div class="p-6 space-y-4">
          <div class="grid grid-cols-2 gap-3">
            <div>
              <label class="text-xs font-bold text-leise uppercase">{{ t('projects.weekPlan.weekdayLabel') }}</label>
              <select v-model.number="slotModal.weekday" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift">
                <option v-for="tag in wochentage" :key="tag.n" :value="tag.n">{{ tag.lang }}</option>
              </select>
            </div>
            <div v-if="zweiWochen">
              <label class="text-xs font-bold text-leise uppercase">{{ t('projects.weekPlan.weekLabel') }}</label>
              <select v-model.number="slotModal.week_index" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift">
                <option :value="0">{{ t('projects.weekPlan.weekA') }}</option>
                <option :value="1">{{ t('projects.weekPlan.weekB') }}</option>
              </select>
            </div>
          </div>

          <label class="flex items-center gap-2 text-sm font-medium text-fliess">
            <input v-model="slotModal.is_all_day" type="checkbox" class="rounded border-slate-300 text-marke focus:ring-marke" />
            {{ t('projects.weekPlan.allDayLabel') }}
          </label>

          <!-- Führt der Plan ein Raster, wird nach der STUNDE gefragt. Die
               Uhrzeitfelder bleiben trotzdem stehen: Eine AG um halb vier
               passt in kein Raster, und sie soll trotzdem eintragbar sein. -->
          <div v-if="!slotModal.is_all_day && stundenraster.length">
            <label class="text-xs font-bold text-leise uppercase">{{ t('projects.weekPlan.lessonLabel') }}</label>
            <select v-model="gewaehlteStunde" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift">
              <option :value="null">{{ t('projects.weekPlan.lessonFree') }}</option>
              <option v-for="(p, i) in stundenraster" :key="i" :value="i">{{ stundenLabel(i) }}</option>
            </select>
          </div>

          <div v-if="!slotModal.is_all_day" class="grid grid-cols-2 gap-3">
            <div>
              <label class="text-xs font-bold text-leise uppercase">{{ t('projects.weekPlan.startsAt') }}</label>
              <input v-model="slotModal.starts_at" type="time" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift" />
            </div>
            <div>
              <label class="text-xs font-bold text-leise uppercase">{{ t('projects.weekPlan.endsAt') }}</label>
              <input v-model="slotModal.ends_at" type="time" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift" />
            </div>
          </div>

          <!-- Der Betreuungsplan kennt keine Fächer – nur der Stundenplan. -->
          <SubjectPicker v-if="!istBetreuung" v-model="slotModal.subject_id" :project-id="projectId" />

          <div>
            <label class="text-xs font-bold text-leise uppercase">{{ t('projects.weekPlan.slotTitle') }}</label>
            <input v-model="slotModal.title" type="text" maxlength="255" :placeholder="istBetreuung ? t('projects.weekPlan.slotTitlePlaceholderCare') : t('projects.weekPlan.slotTitlePlaceholder')" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift" />
            <p v-if="!istBetreuung && slotModal.subject_id" class="text-xs text-leise mt-1.5">{{ t('projects.weekPlan.slotTitleWithSubjectHint') }}</p>
          </div>
          <div>
            <label class="text-xs font-bold text-leise uppercase">{{ t('projects.weekPlan.slotSubtitle') }}</label>
            <input v-model="slotModal.subtitle" type="text" maxlength="255" :placeholder="t('projects.weekPlan.slotSubtitlePlaceholder')" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift" />
          </div>

          <div v-if="kann('zuweisen')">
            <label class="text-xs font-bold text-leise uppercase">{{ t('projects.weekPlan.assignedTo') }}</label>
            <select v-model="slotModal.assigned_to" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift">
              <option :value="null">{{ t('projects.weekPlan.nobody') }}</option>
              <option v-for="m in members" :key="m.user_id" :value="m.user_id">{{ m.name }}</option>
            </select>
          </div>

          <PlacePicker v-model="slotModal.place_id" :project-id="projectId" />

          <div>
            <label class="text-xs font-bold text-leise uppercase">{{ t('projects.weekPlan.colorLabel') }}</label>
            <div class="mt-1 flex items-center gap-2 flex-wrap">
              <button @click="slotModal.color = null" class="w-7 h-7 rounded-full border-2 flex items-center justify-center"
                :class="slotModal.color ? 'border-linie text-slate-400' : 'border-marke text-marke'"
                :title="t('projects.weekPlan.noColor')"><X class="w-3.5 h-3.5" /></button>
              <button v-for="f in FARBEN" :key="f" @click="slotModal.color = f"
                class="w-7 h-7 rounded-full border-2" :style="{ backgroundColor: f }"
                :class="slotModal.color === f ? 'border-slate-900 dark:border-white' : 'border-transparent'"></button>
            </div>
          </div>
        </div>
        <div class="flex justify-end gap-2 px-6 py-4 border-t border-linie">
          <button @click="slotModal = null" class="px-4 py-2 text-fliess hover:bg-auflage rounded-xl text-sm font-bold">{{ t('common.cancel') }}</button>
          <BaseButton @click="submitSlot" :loading="saving">{{ t('common.save') }}</BaseButton>
        </div>
      </div>
    </div>

    <!-- ===== Abschnitt ===== -->
    <div v-if="periodModal" class="fixed inset-0 bg-slate-900/60 backdrop-blur-sm flex items-center justify-center p-4 z-[100]" @click.self="periodModal = null">
      <div class="karte w-full max-w-lg shadow-2xl max-h-[90vh] overflow-y-auto">
        <div class="flex items-center justify-between px-6 py-4 border-b border-linie">
          <h3 class="font-extrabold text-schrift flex items-center gap-2"><CalendarRange class="w-5 h-5 text-marke" /> {{ periodModal.id ? t('projects.weekPlan.editPeriod') : t('projects.weekPlan.newPeriod') }}</h3>
          <button @click="periodModal = null" class="p-1.5 text-slate-400 hover:bg-auflage rounded-xl"><X class="w-5 h-5" /></button>
        </div>
        <div class="p-6 space-y-4">
          <div>
            <label class="text-xs font-bold text-leise uppercase">{{ t('projects.weekPlan.periodTitle') }}</label>
            <input v-model="periodModal.title" type="text" maxlength="255" :placeholder="t('projects.weekPlan.periodTitlePlaceholder')" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift" />
          </div>
          <div class="grid grid-cols-2 gap-3">
            <div>
              <label class="text-xs font-bold text-leise uppercase">{{ t('planning.fields.starts_on') }}</label>
              <input v-model="periodModal.starts_on" type="date" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift" />
            </div>
            <div>
              <label class="text-xs font-bold text-leise uppercase">{{ t('planning.fields.ends_on') }}</label>
              <input v-model="periodModal.ends_on" type="date" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift" />
            </div>
          </div>
          <div v-if="kann('zuweisen')">
            <label class="text-xs font-bold text-leise uppercase">{{ t('projects.weekPlan.assignedTo') }}</label>
            <select v-model="periodModal.assigned_to" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift">
              <option :value="null">{{ t('projects.weekPlan.nobody') }}</option>
              <option v-for="m in members" :key="m.user_id" :value="m.user_id">{{ m.name }}</option>
            </select>
          </div>
          <PlacePicker v-model="periodModal.place_id" :project-id="projectId" />
          <div>
            <label class="text-xs font-bold text-leise uppercase">{{ t('projects.weekPlan.noteLabel') }}</label>
            <textarea v-model="periodModal.note" rows="2" maxlength="2000" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift resize-none"></textarea>
          </div>
          <div>
            <label class="text-xs font-bold text-leise uppercase">{{ t('projects.weekPlan.colorLabel') }}</label>
            <div class="mt-1 flex items-center gap-2 flex-wrap">
              <button @click="periodModal.color = null" class="w-7 h-7 rounded-full border-2 flex items-center justify-center"
                :class="periodModal.color ? 'border-linie text-slate-400' : 'border-marke text-marke'"
                :title="t('projects.weekPlan.noColor')"><X class="w-3.5 h-3.5" /></button>
              <button v-for="f in FARBEN" :key="f" @click="periodModal.color = f"
                class="w-7 h-7 rounded-full border-2" :style="{ backgroundColor: f }"
                :class="periodModal.color === f ? 'border-slate-900 dark:border-white' : 'border-transparent'"></button>
            </div>
          </div>
        </div>
        <div class="flex justify-end gap-2 px-6 py-4 border-t border-linie">
          <button @click="periodModal = null" class="px-4 py-2 text-fliess hover:bg-auflage rounded-xl text-sm font-bold">{{ t('common.cancel') }}</button>
          <BaseButton @click="submitPeriod" :loading="saving">{{ t('common.save') }}</BaseButton>
        </div>
      </div>
    </div>

    <!-- ===== Einstellungen ===== -->
    <div v-if="settingsModal" class="fixed inset-0 bg-slate-900/60 backdrop-blur-sm flex items-center justify-center p-4 z-[100]" @click.self="settingsModal = null">
      <div class="karte w-full max-w-lg shadow-2xl max-h-[90vh] overflow-y-auto">
        <div class="flex items-center justify-between px-6 py-4 border-b border-linie">
          <h3 class="font-extrabold text-schrift flex items-center gap-2"><Settings2 class="w-5 h-5 text-marke" /> {{ t('projects.weekPlan.settingsTitle') }}</h3>
          <button @click="settingsModal = null" class="p-1.5 text-slate-400 hover:bg-auflage rounded-xl"><X class="w-5 h-5" /></button>
        </div>
        <div class="p-6 space-y-4">
          <div>
            <label class="text-xs font-bold text-leise uppercase">{{ t('projects.weekPlan.nameLabel') }}</label>
            <input v-model="settingsModal.name" type="text" maxlength="255" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift" />
          </div>
          <div>
            <label class="text-xs font-bold text-leise uppercase">{{ t('projects.weekPlan.typeLabel') }}</label>
            <select v-model="settingsModal.type" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift">
              <option value="timetable">{{ t('planning.templates.stundenplan.label') }}</option>
              <option value="care">{{ t('planning.templates.betreuungsplan.label') }}</option>
            </select>
          </div>
          <div>
            <label class="text-xs font-bold text-leise uppercase">{{ t('projects.weekPlan.appliesToLabel') }}</label>
            <select v-model="settingsModal.applies_to" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift">
              <option value="all">{{ t('projects.weekPlan.appliesTo.all') }}</option>
              <option value="school_days">{{ t('projects.weekPlan.appliesTo.school_days') }}</option>
              <option value="breaks">{{ t('projects.weekPlan.appliesTo.breaks') }}</option>
            </select>
            <p class="text-xs text-leise mt-1.5">{{ t(`projects.weekPlan.appliesToHint.${settingsModal.applies_to}`) }}</p>
          </div>
          <div>
            <label class="text-xs font-bold text-leise uppercase">{{ t('projects.weekPlan.schoolYearLabel') }}</label>
            <select v-model="settingsModal.school_year_id" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift">
              <option :value="null">{{ t('projects.weekPlan.noSchoolYearOption') }}</option>
              <option v-for="j in schuljahre" :key="j.id" :value="j.id">{{ j.name }}</option>
            </select>
            <p v-if="schuljahre.length === 0" class="text-xs text-leise mt-1.5">{{ t('projects.weekPlan.noSchoolYearsYet') }}</p>
          </div>
          <div class="grid grid-cols-2 gap-3">
            <div>
              <label class="text-xs font-bold text-leise uppercase">{{ t('projects.weekPlan.cycleLabel') }}</label>
              <select v-model="settingsModal.rhythmus" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift">
                <option value="einfach">{{ t('projects.weekPlan.cycleOne') }}</option>
                <option value="anchor">{{ t('projects.weekPlan.cycleAnchor') }}</option>
                <option value="iso_week">{{ t('projects.weekPlan.cycleIsoWeek') }}</option>
              </select>
            </div>
            <div v-if="settingsModal.rhythmus === 'anchor'">
              <label class="text-xs font-bold text-leise uppercase">{{ t('projects.weekPlan.anchorLabel') }}</label>
              <input v-model="settingsModal.cycle_anchor" type="date" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift" />
            </div>
          </div>
          <p v-if="settingsModal.rhythmus === 'anchor'" class="text-xs text-leise -mt-2">{{ t('projects.weekPlan.anchorHint') }}</p>
          <!-- Der Hinweis, an dem alles hängt: Steht die Regelung schriftlich
               als „gerade Kalenderwoche", dann IST der Sprung am
               Jahreswechsel Teil der Vereinbarung. -->
          <p v-if="settingsModal.rhythmus === 'iso_week'" class="text-xs text-leise -mt-2">{{ t('projects.weekPlan.isoWeekHint') }}</p>

          <div class="grid grid-cols-2 gap-3">
            <div>
              <label class="text-xs font-bold text-leise uppercase">{{ t('projects.weekPlan.validFrom') }}</label>
              <input v-model="settingsModal.valid_from" type="date" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift" />
            </div>
            <div>
              <label class="text-xs font-bold text-leise uppercase">{{ t('projects.weekPlan.validTo') }}</label>
              <input v-model="settingsModal.valid_to" type="date" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift" />
            </div>
          </div>

          <!-- Stundenraster: nur beim Stundenplan. Der Betreuungsplan
               besteht aus ganzen Tagen, dort gäbe es nichts zu takten. -->
          <div v-if="settingsModal.type !== 'care'" class="border-t border-linie pt-4">
            <div class="flex items-center justify-between gap-2">
              <label class="text-xs font-bold text-leise uppercase">{{ t('projects.weekPlan.gridLabel') }}</label>
              <button type="button" @click="stundeHinzufuegen" class="text-xs font-bold text-marke">+ {{ t('projects.weekPlan.gridAdd') }}</button>
            </div>
            <p class="text-xs text-leise mt-1">{{ t('projects.weekPlan.gridHint') }}</p>

            <!-- Vorlagen: ein Startpunkt, kein Gesetz. Jede Zeile bleibt
                 danach einzeln änderbar. -->
            <div class="flex flex-wrap gap-1.5 mt-2">
              <button v-for="key in RASTER_KEYS" :key="key" type="button" @click="rasterVorlage(key)"
                class="px-2.5 py-1 text-xs font-bold rounded-lg bg-auflage text-fliess hover:bg-marke-leise hover:text-marke">
                {{ t(`projects.weekPlan.grids.${key}`) }}
              </button>
            </div>

            <div v-if="settingsModal.stunden.length" class="mt-3 space-y-1.5 max-h-56 overflow-y-auto">
              <div v-for="(p, i) in settingsModal.stunden" :key="i" class="flex items-center gap-2">
                <span class="w-6 shrink-0 text-xs font-bold text-slate-400 tabular-nums">{{ i + 1 }}.</span>
                <input v-model="p.from" type="time" class="flex-1 px-3 py-2 bg-vertieft border border-linie rounded-xl text-sm text-schrift" />
                <span class="text-slate-400 text-xs">–</span>
                <input v-model="p.to" type="time" class="flex-1 px-3 py-2 bg-vertieft border border-linie rounded-xl text-sm text-schrift" />
                <button type="button" @click="stundeEntfernen(i)" class="p-1.5 text-slate-400 hover:text-rose-500 rounded-xl"><Trash2 class="w-4 h-4" /></button>
              </div>
            </div>
          </div>

          <div>
            <label class="text-xs font-bold text-leise uppercase">{{ t('projects.weekPlan.timezoneLabel') }}</label>
            <input v-model="settingsModal.timezone" type="text" maxlength="64" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift" />
            <p class="text-xs text-leise mt-1.5">{{ t('projects.weekPlan.timezoneHint') }}</p>
          </div>
        </div>
        <div class="flex justify-end gap-2 px-6 py-4 border-t border-linie">
          <button @click="settingsModal = null" class="px-4 py-2 text-fliess hover:bg-auflage rounded-xl text-sm font-bold">{{ t('common.cancel') }}</button>
          <BaseButton @click="submitSettings" :loading="saving">{{ t('common.save') }}</BaseButton>
        </div>
      </div>
    </div>

    <SubjectsPanel v-if="subjectsPanelOpen" :project-id="projectId" @close="subjectsPanelOpen = false; load()" />

    <PlanFeedModal v-if="feedModalOpen && plan" :project-id="projectId" :type="plan.type" @close="feedModalOpen = false" />
  </div>
</template>
