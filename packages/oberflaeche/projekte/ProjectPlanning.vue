<script setup>
// Der Reiter „Planung" eines Projekts: EINE Liste über alles, was dort
// nebeneinander liegt, plus der Vorlagen-Picker („+ Neu → Was möchtest du
// starten?"), über den alles angelegt wird. Bewusst keine zweite
// Navigationsebene: Ein neuer Planungs-Typ wird eine Zeile in dieser Liste,
// kein eigener Reiter.
//
// Abstimmungen laufen über einen generischen Motor mit einem type-Feld
// (Terminfindung, Umfrage, Schichtplan, Mitbringliste, Einladungsliste). Ihre
// Formulare und Detailansichten leben hier bzw. unter polls/. Die Behälter
// (Board, Roadmap, Orte, Schulplanung) sind seit dem 16.10.2026 zurückgebaut
// (docs/plan-ende-zu-ende.md, Schritt 0).
//
// Die Datei hieß bis zum Container-Umbau ProjectPolls.vue – zu einer Zeit,
// als der Reiter wirklich nur Abstimmungen enthielt.
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from './umgebung';
import ClampText from '@oberflaeche/base/ClampText.vue';
import BaseModal from '@oberflaeche/base/BaseModal.vue';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import PollSchedule from './polls/PollSchedule.vue';
import PollSurvey from './polls/PollSurvey.vue';
import PollSignup from './polls/PollSignup.vue';
import PollInvite from './polls/PollInvite.vue';
import ProjektKalender from './ProjektKalender.vue';
import WhiteboardAnsicht from './whiteboard/WhiteboardAnsicht.vue';
import {
  Plus, RefreshCcw, Trash2, ArrowLeft, Loader2, CalendarClock, X, Vote,
  CheckCircle2, Lock, BarChart3, ListChecks, ShoppingBasket, Copy, Clock,
  PartyPopper, ChevronLeft, ChevronRight, Presentation,
} from 'lucide-vue-next';
import { useToast } from '@oberflaeche/composables/useToast';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import { kann } from './umgebung';
import { formatTime, formatWeekdayTime } from '@oberflaeche/shared/date';

const toast = useToast();
const { t } = useI18n();
const { confirmDialog, confirmDelete } = useConfirm();
const props = defineProps({
  projectId: { type: [Number, String], required: true },
  meId: { type: Number, default: null },
  isOwner: { type: Boolean, default: false },
  members: { type: Array, default: () => [] },
  // Was dieser Rahmen hier nicht anbietet -- 'abstimmungen': keine neuen
  // (Projekte vor Ort, docs/plan-abstimmungen-e2e.md); 'whiteboards': keine
  // (dto., docs/plan-whiteboard.md).
  ohne: { type: Array, default: () => [] },
});

// Der Bestand dieser Liste, für das Zahlen-Abzeichen am Reiter. Wer ihn
// zeigt, lädt ihn nicht – wer ihn kennt, zeigt ihn nicht.
const emit = defineEmits(['count']);

// Vorlagen: bestimmen type + settings-Defaults des generischen Motors.
//
// SEIT DEM 16.10.2026 NUR NOCH ABSTIMMUNGEN (docs/plan-ende-zu-ende.md,
// Schritt 0): Kanban, Roadmaps, Orte und Schulplanung sind zurückgebaut –
// sie liessen mehrere Leute dasselbe bearbeiten, und das verträgt sich nicht
// mit Ende-zu-Ende-Verschlüsselung. Hier bleibt, was je Eintrag einen
// Schreiber hat: die Frage dem Ersteller, jede Stimme ihrer Person.
const POLL_TEMPLATES = [
  { key: 'schedule', type: 'schedule', label: t('planning.templates.schedule.label'), icon: CalendarClock, hint: t('planning.templates.schedule.hint') },
  { key: 'survey', type: 'survey', label: t('planning.templates.survey.label'), icon: BarChart3, hint: t('planning.templates.survey.hint') },
  { key: 'shift', type: 'signup', label: t('planning.templates.shift.label'), icon: ListChecks, hint: t('planning.templates.shift.hint') },
  { key: 'bring', type: 'signup', label: t('planning.templates.bring.label'), icon: ShoppingBasket, hint: t('planning.templates.bring.hint') },
  { key: 'invite', type: 'invite', label: t('planning.templates.invite.label'), icon: PartyPopper, hint: t('planning.templates.invite.hint') },
];


// EIN Regal, alphabetisch nach dem ANGEZEIGTEN Namen – also je Sprache
// verschieden, mit `localeCompare('de')`, damit Umlaute dort einsortieren,
// wo man sie sucht (Ä bei A und nicht hinter Z).
//
// Sortiert werden KOPIEN, und das ist kein Stilzwang: `typeMeta()` greift
// POLL_TEMPLATES über den Index an (`[0]` = Terminfindung, `[4]` =
// Einladungsliste). In-place sortiert bekäme jede Abstimmung in der Liste
// stillschweigend das falsche Symbol und den falschen Typnamen.
// Was der Rahmen nicht kann, steht gar nicht erst zur Wahl (umgebung.js).
// Das Whiteboard (docs/plan-whiteboard.md) ist keine Abstimmung: eigener
// Speicher, eigene Ansicht -- aber dieselbe Kachelreihe und dieselbe Liste.
const WHITEBOARD_TEMPLATE = { key: 'whiteboard', label: t('planning.templates.whiteboard.label'), icon: Presentation, hint: t('planning.templates.whiteboard.hint') };
const mitWhiteboards = kann('whiteboards') && !props.ohne.includes('whiteboards');

const PICKER_TEMPLATES = [
  ...(kann('abstimmungenVerwalten') && !props.ohne.includes('abstimmungen') ? POLL_TEMPLATES : []),
  ...(mitWhiteboards ? [WHITEBOARD_TEMPLATE] : []),
]
  .sort((a, b) => a.label.localeCompare(b.label, 'de'));

// Waagerechtes Scrollen versteckt, was rechts aus dem Bild ragt – in einem
// Dialog, der die Frage „was kann ich hier überhaupt?" beantwortet, ist das
// die eigentliche Gefahr. Drei Gegenmittel, und keins davon ist Zierrat:
//
//   Pfeile    auf dem Zeigegerät gibt es keine Wischgeste. Sie erscheinen
//             nur, wenn es in die Richtung wirklich weitergeht.
//   Kante     eine ausblendende Fläche am Rand zeigt, DASS da noch etwas
//             ist – Pfeile allein sagen es zu leise.
//   Tastatur  das Regal ist fokussierbar und mit den Pfeiltasten scrollbar.
//
// Der Element-Bezug liegt bewusst in einer einfachen Variablen und nicht in
// einem ref: Ein DOM-Knoten muss nicht reaktiv sein, und Vue tief in ihn
// hineinzuschauen wäre reine Arbeit ohne Nutzen.
let regal = null;
const regalStand = ref({ links: false, rechts: false });

const pruefeRegal = () => {
  if (!regal) return;
  // Vier Pixel Toleranz: Bruchteile aus der Skalierung ließen den Pfeil
  // sonst am Anschlag stehen bleiben.
  const links = regal.scrollLeft > 4;
  const rechts = regal.scrollLeft + regal.clientWidth < regal.scrollWidth - 4;

  // NUR bei echter Änderung schreiben. Sonst dreht sich das hier im Kreis:
  // Der Element-Bezug wird bei jedem Rendern neu aufgerufen, prüfte, schriebe
  // ein neues Objekt, löste das nächste Rendern aus – und so fort.
  if (regalStand.value.links === links && regalStand.value.rechts === rechts) return;

  regalStand.value = { links, rechts };
};

const merkeRegal = (el) => {
  regal = el || null;
  if (el) nextTick(pruefeRegal);
};

// Eine Bildbreite abzüglich eines Streifens: Die angeschnittene Kachel am
// Rand bleibt sichtbar und zeigt, dass man in derselben Reihe weiterliest.
const schiebeRegal = (richtung) => {
  regal?.scrollBy({ left: richtung * (regal.clientWidth * 0.8), behavior: 'smooth' });
};

const typeMeta = (p) => {
  if (p.type === 'schedule') return POLL_TEMPLATES[0];
  if (p.type === 'survey') return POLL_TEMPLATES[1];
  if (p.type === 'invite') return POLL_TEMPLATES[4];
  return p.settings?.variant === 'bring' ? POLL_TEMPLATES[3] : POLL_TEMPLATES[2];
};

const loading = ref(true);
const error = ref('');
const polls = ref([]);
const poll = ref(null); // geöffnete Abstimmung (Detail)
const pollLoading = ref(false);
const whiteboards = ref([]);
const whiteboard = ref(null); // geöffnetes Whiteboard (ganze Fläche)
const whiteboardModal = ref(null); // { title }


const pickerOpen = ref(false);
const createModal = ref(null); // { template, title, description, expires_at, options, settings }
const creating = ref(false);
const closeModal = ref(null); // schedule: { finalOptionId, addToCalendar }
const closing = ref(false);

const canManage = (p) => kann('abstimmungenVerwalten') && p && (p.created_by === props.meId || props.isOwner);

const fmtScheduleOption = (o) => {
  const start = formatWeekdayTime(o.starts_at);
  return o.ends_at ? `${start}–${formatTime(o.ends_at)}` : start;
};

const loadPolls = async () => {
  loading.value = true; error.value = '';
  try {
    const [p, w] = await Promise.all([
      api.getPolls(props.projectId),
      mitWhiteboards ? api.getWhiteboards(props.projectId) : { data: [] },
    ]);
    polls.value = p.data;
    whiteboards.value = w.data;
    meldeBestand();
  }
  catch (e) { error.value = t('planning.loadFailed'); }
  finally { loading.value = false; }
};

// Die Planungs-Liste: Abstimmungen, Listen und Whiteboards, neueste zuerst.
const planItems = computed(() => [
  ...polls.value.map((p) => ({ kind: 'poll', sortKey: p.created_at, item: p })),
  ...whiteboards.value.map((w) => ({ kind: 'whiteboard', sortKey: w.created_at, item: { ...w, id: w.uuid, title: w.titel } })),
]
  .sort((a, b) => new Date(b.sortKey) - new Date(a.sortKey)));

// Der Zähler am Reiter „Planung" gehört der Projektansicht, der Bestand
// steht aber hier: Diese Liste IST, was Project::planningCount() zählt. Bisher kam die Zahl nur mit dem
// Projekt-Payload und blieb danach stehen; wer einen Behälter löschte, sah
// zwei Zahlen auf einem Schirm, die sich widersprachen.
//
// Gemeldet wird ausdrücklich am Ende jedes Ladens und nicht über einen
// Watcher auf der Länge: Der verpasste den Fall, dass eine leere Liste leer
// bleibt (0 → 0 ist keine Änderung), und hinge an der Reihenfolge, in der
// Vue seine Effekte abarbeitet.
const meldeBestand = () => emit('count', planItems.value.length);

const openPoll = async (id) => {
  pollLoading.value = true;
  try { poll.value = (await api.getPoll(props.projectId, id)).data; }
  catch (e) { error.value = t('planning.loadPollFailed'); }
  finally { pollLoading.value = false; }
};
const reloadPoll = () => poll.value && openPoll(poll.value.id);
const backToList = () => { poll.value = null; loadPolls(); };


// --- Erstellen ---

const openCreate = (template) => {
  pickerOpen.value = false;
  if (template.key === 'whiteboard') {
    whiteboardModal.value = { title: '' };
    return;
  }
  const base = { template, title: '', description: '', expires_at: '', notifyMembers: false };
  if (template.key === 'schedule') {
    createModal.value = { ...base, options: [{ starts_at: '', ends_at: '' }, { starts_at: '', ends_at: '' }] };
  } else if (template.key === 'survey') {
    createModal.value = { ...base, options: [{ label: '' }, { label: '' }], settings: { multiple: false, anonymous: false } };
  } else if (template.key === 'invite') {
    // Gäste: Projektmitglieder ankreuzen + freie Namen (Menschen ohne Konto).
    createModal.value = { ...base, memberIds: [], options: [{ label: '' }, { label: '' }] };
  } else {
    createModal.value = {
      ...base,
      options: [{ label: '', capacity: '' }, { label: '', capacity: '' }],
      settings: template.key === 'bring'
        ? { member_options: true, done_marking: true, variant: 'bring' }
        : { member_options: false, done_marking: false, variant: 'shift' },
    };
  }
};
const addCreateOption = () => {
  const t = createModal.value.template.key;
  createModal.value.options.push(
    t === 'schedule' ? { starts_at: '', ends_at: '' }
    : t === 'survey' || t === 'invite' ? { label: '' }
    : { label: '', capacity: '' },
  );
};
const invitableMembers = computed(() => props.members);
const toggleInviteMember = (userId) => {
  const sel = createModal.value.memberIds;
  const i = sel.indexOf(userId);
  if (i === -1) sel.push(userId);
  else sel.splice(i, 1);
};
const removeCreateOption = (i) => createModal.value.options.splice(i, 1);
const presetYesNo = () => { createModal.value.options = [{ label: 'Ja' }, { label: 'Nein' }]; };

const submitCreate = async () => {
  const m = createModal.value;
  const tpl = m.template;

 
  let options;
  if (tpl.key === 'schedule') {
    options = m.options.filter((o) => o.starts_at).map((o) => ({ starts_at: o.starts_at, ends_at: o.ends_at || null }));
  } else if (tpl.key === 'invite') {
    // Angekreuzte Mitglieder (verknüpft, sagen selbst zu) + freie Namen.
    options = [
      ...invitableMembers.value.filter((mem) => m.memberIds.includes(mem.user_id)).map((mem) => ({ label: mem.name, user_id: mem.user_id })),
      ...m.options.filter((o) => o.label.trim()).map((o) => ({ label: o.label.trim() })),
    ];
  } else {
    options = m.options.filter((o) => o.label.trim()).map((o) => ({
      label: o.label.trim(),
      ...(tpl.type === 'signup' ? { capacity: o.capacity ? Number(o.capacity) : null } : {}),
    }));
  }
  if (!m.title.trim() || options.length === 0) {
    toast.error(tpl.key === 'schedule' ? t('planning.createMissingScheduleFields') : tpl.key === 'invite' ? t('planning.createMissingInviteFields') : t('planning.createMissingEntryFields'));
    return;
  }
  creating.value = true;
  try {
    const res = await api.createPoll(props.projectId, {
      type: tpl.type,
      title: m.title.trim(),
      description: m.description.trim() || null,
      expires_at: m.expires_at || null,
      options,
      settings: m.settings || undefined,
      notify_members: m.notifyMembers,
    });
    createModal.value = null;
    poll.value = res.data;
    // Die verdeckte Liste nachziehen, damit der gemeldete Bestand die neue
    // Abstimmung mitzählt.
    loadPolls();
  } catch (e) {
    toast.error(e.response?.data?.message || t('planning.createFailed'));
  } finally {
    creating.value = false;
  }
};

// --- Whiteboards ---
const openWhiteboard = async (id) => {
  pollLoading.value = true;
  try { whiteboard.value = (await api.getWhiteboard(props.projectId, id)).data; }
  catch (e) { toast.error(t('planning.whiteboard.ladenFehler')); }
  finally { pollLoading.value = false; }
};
const closeWhiteboard = () => { whiteboard.value = null; loadPolls(); };

const submitWhiteboard = async () => {
  const titel = whiteboardModal.value.title.trim();
  if (!titel) return;
  creating.value = true;
  try {
    whiteboard.value = (await api.createWhiteboard(props.projectId, { titel })).data;
    whiteboardModal.value = null;
    loadPolls();
  } catch (e) {
    toast.error(e.response?.data?.message || t('planning.createFailed'));
  } finally {
    creating.value = false;
  }
};

const deleteWhiteboard = async (w) => {
  if (!(await confirmDelete(t('planning.whiteboard.confirmDelete', { title: w.title })))) return;
  try {
    await api.deleteWhiteboard(props.projectId, w.id);
    await loadPolls();
  } catch (e) {
    toast.error(t('planning.deleteFailed'));
  }
};

// --- Löschen / Abschließen / Duplizieren ---
const deletePoll = async (p) => {
  if (!(await confirmDelete(t('planning.confirmDeletePoll', { title: p.title })))) return;
  try {
    await api.deletePoll(props.projectId, p.id);
    if (poll.value?.id === p.id) poll.value = null;
    await loadPolls();
  } catch (e) {
    toast.error(e.response?.status === 403 ? t('planning.deleteForbidden') : t('planning.deleteFailed'));
  }
};

const startClose = async () => {
  if (poll.value.type === 'schedule') {
    closeModal.value = { finalOptionId: poll.value.best_option_id || poll.value.options[0]?.id, addToCalendar: false };
  } else if (await confirmDialog(t('planning.confirmClose', { title: poll.value.title }), { title: t('planning.closeTitle'), confirmLabel: t('planning.finish') })) {
    submitClose({});
  }
};
const submitClose = async (payload) => {
  closing.value = true;
  try {
    const res = await api.closePoll(props.projectId, poll.value.id, payload);
    closeModal.value = null;
    poll.value = res.data;
  } catch (e) {
    toast.error(t('planning.closeFailed'));
  } finally {
    closing.value = false;
  }
};

const duplicatePoll = async (p) => {
  try {
    const res = await api.duplicatePoll(props.projectId, p.id);
    poll.value = res.data;
  } catch (e) {
    toast.error(t('planning.duplicateFailed'));
  }
};

const onPollUpdate = (data) => { poll.value = data; };

onMounted(() => {
  loadPolls();
  // Wird das Fenster schmaler, passen weniger Kacheln nebeneinander – dann
  // gehört der rechte Pfeil da hin, wo vorher keiner nötig war.
  window.addEventListener('resize', pruefeRegal);
});
onUnmounted(() => {
  window.removeEventListener('resize', pruefeRegal);
});
</script>

<template>
  <div>
    <!-- ===== DETAIL ===== -->
    <div v-if="poll" class="karte shadow-sm p-6 space-y-5">
      <div class="flex items-start gap-3 flex-wrap">
        <button @click="backToList" class="p-2 text-leise hover:bg-auflage rounded-xl" :title="t('common.back')"><ArrowLeft class="w-4 h-4" /></button>
        <div class="min-w-0">
          <h3 class="font-extrabold text-lg text-schrift flex items-center gap-2 flex-wrap">
            <component :is="typeMeta(poll).icon" class="w-5 h-5 text-marke shrink-0" />
            {{ poll.title }}
            <span class="text-xs font-bold px-2 py-0.5 rounded-full bg-marke-leise text-marke">{{ typeMeta(poll).label }}</span>
            <span v-if="poll.status === 'closed'" class="text-xs font-bold px-2 py-0.5 rounded-full bg-auflage text-slate-500 flex items-center gap-1"><Lock class="w-3 h-3" /> {{ t('planning.closed') }}</span>
            <span v-else-if="poll.expires_at" class="text-xs font-bold px-2 py-0.5 rounded-full bg-amber-50 dark:bg-amber-900/30 text-amber-600 dark:text-amber-400 flex items-center gap-1"><Clock class="w-3 h-3" /> {{ t('planning.until', { date: formatWeekdayTime(poll.expires_at) }) }}</span>
          </h3>
          <div v-if="poll.description" class="text-sm text-leise mt-1"><ClampText :text="poll.description" :lines="2" /></div>
        </div>
        <div class="ml-auto flex items-center gap-2">
          <button @click="reloadPoll" class="p-2 text-leise hover:bg-auflage rounded-xl" :title="t('planning.refresh')"><RefreshCcw class="w-4 h-4" :class="pollLoading ? 'animate-spin text-marke' : ''" /></button>
          <button v-if="kann('abstimmungenVerwalten')" @click="duplicatePoll(poll)" class="p-2 text-leise hover:bg-auflage rounded-xl" :title="t('planning.duplicate')"><Copy class="w-4 h-4" /></button>
          <BaseButton v-if="canManage(poll) && poll.status === 'open'" @click="startClose" groesse="normal"><CheckCircle2 class="w-4 h-4" /> {{ t('planning.finish') }}</BaseButton>
        </div>
      </div>

      <PollSchedule v-if="poll.type === 'schedule'" :poll="poll" :project-id="projectId" :me-id="meId" @update:poll="onPollUpdate" />
      <PollSurvey v-else-if="poll.type === 'survey'" :poll="poll" :project-id="projectId" :me-id="meId" @update:poll="onPollUpdate" />
      <PollInvite v-else-if="poll.type === 'invite'" :poll="poll" :project-id="projectId" :me-id="meId" :can-manage="canManage(poll)" :members="members" @update:poll="onPollUpdate" />
      <PollSignup v-else :poll="poll" :project-id="projectId" :me-id="meId" :can-manage="canManage(poll)" @update:poll="onPollUpdate" />
    </div>


    <!-- ===== LISTE ===== -->
    <div v-else class="karte shadow-sm p-6 space-y-5">
      <div class="flex items-center gap-2">
        <Vote class="w-5 h-5 text-marke" />
        <h3 class="font-extrabold text-lg text-schrift">{{ t('planning.title') }}</h3>
        <BaseButton v-if="PICKER_TEMPLATES.length" @click="pickerOpen = true" groesse="kopf" class="ml-auto"><Plus class="w-4 h-4" /> {{ t('planning.newPoll') }}</BaseButton>
        <button @click="loadPolls" class="p-2 text-slate-400 hover:bg-auflage rounded-xl" :title="t('planning.refresh')"><RefreshCcw class="w-4 h-4" :class="loading ? 'animate-spin text-marke' : ''" /></button>
      </div>

      <div v-if="loading" class="py-10 text-center text-slate-400"><Loader2 class="w-6 h-6 animate-spin mx-auto" /></div>
      <p v-else-if="error" class="text-sm text-rose-500 font-medium">{{ error }}</p>
      <p v-else-if="planItems.length === 0" class="py-8 text-center text-sm text-slate-400 dark:text-slate-500 font-medium">{{ PICKER_TEMPLATES.length ? t('planning.emptyList') : t('planning.emptyListOhneNeu') }}</p>
      <ul v-else class="divide-y divide-linie">
        <li v-for="entry in planItems" :key="`${entry.kind}-${entry.item.id}`" class="py-3 flex items-center gap-3">
          <template v-if="entry.kind === 'whiteboard'">
            <button @click="openWhiteboard(entry.item.id)" class="flex items-center gap-3 flex-1 min-w-0 text-left group">
              <Presentation class="w-4 h-4 shrink-0 text-marke" />
              <span class="font-bold text-schrift truncate group-hover:text-marke">{{ entry.item.title }}</span>
              <span class="text-xs font-medium text-slate-400 shrink-0">{{ WHITEBOARD_TEMPLATE.label }}</span>
            </button>
            <button v-if="entry.item.eigen || isOwner" @click="deleteWhiteboard(entry.item)" class="p-1.5 text-slate-400 hover:text-rose-500 hover:bg-rose-50 dark:hover:bg-rose-900/20 rounded-xl" :title="t('planning.whiteboard.loeschen')"><Trash2 class="w-4 h-4" /></button>
          </template>
          <template v-else>
            <button @click="openPoll(entry.item.id)" class="flex items-center gap-3 flex-1 min-w-0 text-left group">
              <component :is="typeMeta(entry.item).icon" class="w-4 h-4 shrink-0" :class="entry.item.status === 'closed' ? 'text-slate-400' : 'text-marke'" />
              <span class="font-bold text-schrift truncate group-hover:text-marke">{{ entry.item.title }}</span>
              <span class="text-xs font-medium text-slate-400 shrink-0">{{ typeMeta(entry.item).label }}</span>
              <span v-if="entry.item.status === 'closed'" class="text-xs font-bold px-2 py-0.5 rounded-full bg-auflage text-slate-500 shrink-0">{{ t('planning.closedBadge') }}</span>
              <span v-else-if="entry.item.expires_at" class="text-xs font-bold text-amber-500 shrink-0 flex items-center gap-1"><Clock class="w-3 h-3" /> {{ formatWeekdayTime(entry.item.expires_at) }}</span>
            </button>
            <button v-if="kann('abstimmungenVerwalten')" @click="duplicatePoll(entry.item)" class="p-1.5 text-slate-400 hover:text-marke hover:bg-marke-leise rounded-xl" :title="t('planning.duplicate')"><Copy class="w-4 h-4" /></button>
            <button v-if="canManage(entry.item)" @click="deletePoll(entry.item)" class="p-1.5 text-slate-400 hover:text-rose-500 hover:bg-rose-50 dark:hover:bg-rose-900/20 rounded-xl" :title="t('planning.deletePollTitle')"><Trash2 class="w-4 h-4" /></button>
          </template>
        </li>
      </ul>
    </div>

    <!-- Geteilte Kalender (4c) -- nur in der Liste, nicht in einer offenen Abstimmung. -->
    <ProjektKalender v-if="!poll" :project-id="projectId" />

    <!-- ===== VORLAGEN-AUSWAHL ===== -->
    <div v-if="pickerOpen" class="fixed inset-0 bg-slate-900/60 backdrop-blur-sm flex items-center justify-center p-4 z-[100]" @click.self="pickerOpen = false">
      <!-- max-h/overflow fehlten hier als Einzige: Der Dialog wuchs mit jeder
           Vorlage und lief bei elf aus dem Bild, ohne dass man ihn scrollen
           konnte. -->
      <div class="karte w-full max-w-2xl shadow-2xl p-6 space-y-5 max-h-[90vh] overflow-y-auto">
        <div class="flex items-center gap-2">
          <h3 class="font-extrabold text-schrift">{{ t('planning.pickerTitle') }}</h3>
          <!-- Pfeile nur, wenn es in die Richtung wirklich weitergeht. -->
          <div class="ml-auto flex items-center gap-1">
            <button v-if="regalStand.links" @click="schiebeRegal(-1)"
              :aria-label="t('planning.scrollBack')" :title="t('planning.scrollBack')"
              class="p-1 text-slate-400 hover:text-marke hover:bg-auflage rounded-lg"><ChevronLeft class="w-4 h-4" /></button>
            <button v-if="regalStand.rechts" @click="schiebeRegal(1)"
              :aria-label="t('planning.scrollForward')" :title="t('planning.scrollForward')"
              class="p-1 text-slate-400 hover:text-marke hover:bg-auflage rounded-lg"><ChevronRight class="w-4 h-4" /></button>
          </div>
          <button @click="pickerOpen = false" class="p-1.5 text-slate-400 hover:bg-auflage rounded-xl"><X class="w-5 h-5" /></button>
        </div>

        <div class="relative">
          <div
            :ref="merkeRegal"
            @scroll="pruefeRegal"
            tabindex="0"
            role="group"
            :aria-label="t('planning.pickerTitle')"
            class="flex gap-3 overflow-x-auto snap-x snap-mandatory pb-1 focus:outline-none focus:ring-2 focus:ring-marke rounded-xl"
          >
            <!-- `tpl`, nicht `t`: das würde die Übersetzungsfunktion überdecken. -->
            <button v-for="tpl in PICKER_TEMPLATES" :key="tpl.key" @click="openCreate(tpl)"
              class="snap-start shrink-0 w-44 text-left p-4 rounded-xl border border-linie hover:border-marke hover:bg-marke-leise/50 transition-all group">
              <component :is="tpl.icon" class="w-6 h-6 text-marke mb-2 group-hover:scale-110 transition-transform" />
              <p class="font-extrabold text-schrift text-sm">{{ tpl.label }}</p>
              <p class="text-xs text-leise mt-1">{{ tpl.hint }}</p>
            </button>
          </div>
          <!-- Die ausblendende Kante sagt „da kommt noch was", wo ein Pfeil
               allein zu leise ist. pointer-events-none, sonst schluckte sie
               den Klick auf die angeschnittene Kachel darunter. -->
          <div v-if="regalStand.rechts"
            class="pointer-events-none absolute inset-y-0 right-0 w-10 bg-gradient-to-l from-white dark:from-slate-900 to-transparent rounded-r-xl"></div>
        </div>
      </div>
    </div>

    <!-- ===== ERSTELLEN-MODAL ===== -->
    <div v-if="createModal" class="fixed inset-0 bg-slate-900/60 backdrop-blur-sm flex items-center justify-center p-4 z-[100]" @click.self="createModal = null">
      <div class="karte w-full max-w-lg shadow-2xl max-h-[90vh] overflow-y-auto">
        <div class="flex items-center justify-between px-6 py-4 border-b border-linie">
          <h3 class="font-extrabold text-schrift flex items-center gap-2">
            <component :is="createModal.template.icon" class="w-5 h-5 text-marke" />
            {{ t('planning.newTemplateTitle', { label: createModal.template.label }) }}
          </h3>
          <button @click="createModal = null" class="p-1.5 text-slate-400 hover:bg-auflage rounded-xl"><X class="w-5 h-5" /></button>
        </div>
        <div class="p-6 space-y-4">
          <input v-model="createModal.title" type="text" :placeholder="t(`planning.titlePlaceholder.${createModal.template.key}`)" maxlength="255" class="w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift" />
          <textarea v-model="createModal.description" rows="2" :placeholder="t('planning.descriptionPlaceholder')" class="w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift resize-none"></textarea>


          <!-- Optionen: Terminfindung -->
          <div v-if="createModal.template.key === 'schedule'" class="space-y-2">
            <label class="text-xs font-bold text-leise uppercase">{{ t('planning.scheduleOptionsLabel') }}</label>
            <div v-for="(o, i) in createModal.options" :key="i" class="flex items-center gap-2">
              <input v-model="o.starts_at" type="datetime-local" class="flex-1 px-3 py-2 bg-vertieft border border-linie rounded-xl text-sm text-schrift" />
              <span class="text-slate-400 text-xs">{{ t('planning.to') }}</span>
              <input v-model="o.ends_at" type="datetime-local" class="flex-1 px-3 py-2 bg-vertieft border border-linie rounded-xl text-sm text-schrift" />
              <button @click="removeCreateOption(i)" :disabled="createModal.options.length <= 1" class="p-1.5 text-slate-400 hover:text-rose-500 disabled:opacity-30 rounded-xl"><Trash2 class="w-4 h-4" /></button>
            </div>
          </div>

          <!-- Optionen: Umfrage -->
          <div v-else-if="createModal.template.key === 'survey'" class="space-y-2">
            <div class="flex items-center justify-between">
              <label class="text-xs font-bold text-leise uppercase">{{ t('planning.answersLabel') }}</label>
              <button @click="presetYesNo" class="text-xs font-bold text-marke hover:underline">{{ t('planning.useYesNo') }}</button>
            </div>
            <div v-for="(o, i) in createModal.options" :key="i" class="flex items-center gap-2">
              <input v-model="o.label" type="text" maxlength="255" :placeholder="t('planning.answerPlaceholder', { n: i + 1 })" class="flex-1 px-3 py-2 bg-vertieft border border-linie rounded-xl text-sm text-schrift" />
              <button @click="removeCreateOption(i)" :disabled="createModal.options.length <= 1" class="p-1.5 text-slate-400 hover:text-rose-500 disabled:opacity-30 rounded-xl"><Trash2 class="w-4 h-4" /></button>
            </div>
          </div>

          <!-- Optionen: Einladungsliste (Mitglieder ankreuzen + freie Namen) -->
          <div v-else-if="createModal.template.key === 'invite'" class="space-y-3">
            <div v-if="invitableMembers.length" class="space-y-2">
              <label class="text-xs font-bold text-leise uppercase">{{ t('planning.inviteMembersLabel') }}</label>
              <div class="flex flex-wrap gap-2">
                <label v-for="m in invitableMembers" :key="m.user_id"
                  class="flex items-center gap-1.5 px-3 py-1.5 rounded-full border text-sm font-bold cursor-pointer select-none transition-all"
                  :class="createModal.memberIds.includes(m.user_id)
                    ? 'border-marke bg-marke-leise text-marke'
                    : 'border-linie text-fliess hover:border-marke'"
                  @click.prevent="toggleInviteMember(m.user_id)">
                  <input type="checkbox" class="hidden" :checked="createModal.memberIds.includes(m.user_id)" />
                  {{ m.name }}
                </label>
              </div>
            </div>
            <div class="space-y-2">
              <label class="text-xs font-bold text-leise uppercase">{{ t('planning.inviteFreeLabel') }}</label>
              <div v-for="(o, i) in createModal.options" :key="i" class="flex items-center gap-2">
                <input v-model="o.label" type="text" maxlength="255" :placeholder="t('planning.inviteFreePlaceholder')" class="flex-1 px-3 py-2 bg-vertieft border border-linie rounded-xl text-sm text-schrift" />
                <button @click="removeCreateOption(i)" :disabled="createModal.options.length <= 1" class="p-1.5 text-slate-400 hover:text-rose-500 disabled:opacity-30 rounded-xl"><Trash2 class="w-4 h-4" /></button>
              </div>
            </div>
          </div>

          <!-- Optionen: Schichtplan / Mitbringliste -->
          <div v-else class="space-y-2">
            <label class="text-xs font-bold text-leise uppercase">{{ createModal.template.key === 'shift' ? t('planning.entriesLabelShift') : t('planning.entriesLabelBring') }}</label>
            <div v-for="(o, i) in createModal.options" :key="i" class="flex items-center gap-2">
              <input v-model="o.label" type="text" maxlength="255" :placeholder="createModal.template.key === 'shift' ? t('planning.entryPlaceholderShift', { n: i + 1 }) : t('planning.entryPlaceholderBring', { n: i + 1 })" class="flex-1 px-3 py-2 bg-vertieft border border-linie rounded-xl text-sm text-schrift" />
              <input v-model="o.capacity" type="number" min="1" max="999" :placeholder="createModal.template.key === 'shift' ? t('planning.capacityPlaceholderShift') : t('planning.capacityPlaceholderOther')" class="w-24 px-3 py-2 bg-vertieft border border-linie rounded-xl text-sm text-schrift" />
              <button @click="removeCreateOption(i)" :disabled="createModal.options.length <= 1" class="p-1.5 text-slate-400 hover:text-rose-500 disabled:opacity-30 rounded-xl"><Trash2 class="w-4 h-4" /></button>
            </div>
          </div>

          <button @click="addCreateOption" class="flex items-center gap-1.5 text-sm font-bold text-marke hover:underline"><Plus class="w-4 h-4" /> {{ createModal.template.key === 'invite' ? t('planning.addName') : t('planning.addOption') }}</button>

          <!-- Einstellungen (die Einladungsliste hat kein Ablaufdatum) -->
          <div v-if="createModal.template.key !== 'invite'" class="space-y-2 pt-2 border-t border-linie">
            <template v-if="createModal.template.key === 'survey'">
              <label class="flex items-center gap-2 text-sm font-medium text-fliess">
                <input v-model="createModal.settings.multiple" type="checkbox" class="rounded border-slate-300 text-marke focus:ring-marke" />
                {{ t('planning.allowMultiple') }}
              </label>
              <label class="flex items-center gap-2 text-sm font-medium text-fliess">
                <input v-model="createModal.settings.anonymous" type="checkbox" class="rounded border-slate-300 text-marke focus:ring-marke" />
                {{ t('planning.anonymousVoting') }}
              </label>
            </template>
            <template v-else-if="createModal.settings">
              <label class="flex items-center gap-2 text-sm font-medium text-fliess">
                <input v-model="createModal.settings.member_options" type="checkbox" class="rounded border-slate-300 text-marke focus:ring-marke" />
                {{ t('planning.memberOptionsAllowed') }}
              </label>
              <label class="flex items-center gap-2 text-sm font-medium text-fliess">
                <input v-model="createModal.settings.done_marking" type="checkbox" class="rounded border-slate-300 text-marke focus:ring-marke" />
                {{ t('planning.doneMarkingAllowed') }}
              </label>
            </template>
            <label class="flex items-center gap-2 text-sm font-medium text-fliess">
              <span class="shrink-0">{{ t('planning.expiresAtLabel') }}</span>
              <input v-model="createModal.expires_at" type="datetime-local" class="px-3 py-1.5 bg-vertieft border border-linie rounded-xl text-sm text-schrift" />
            </label>
          </div>
        </div>
        <div class="flex justify-end gap-2 px-6 py-4 border-t border-linie">
          <button @click="createModal = null" class="px-4 py-2 text-fliess hover:bg-auflage rounded-xl text-sm font-bold">{{ t('common.cancel') }}</button>
          <!-- Einladungsliste benachrichtigt verknüpfte Mitglieder automatisch. -->
          <label v-if="createModal.template.key !== 'invite'" class="mr-auto flex items-center gap-2 text-xs font-semibold text-leise cursor-pointer select-none">
            <input type="checkbox" v-model="createModal.notifyMembers" class="accent-marke" />
            {{ t('projects.notifyMembers') }}
          </label>
          <span v-else class="mr-auto"></span>
          <BaseButton @click="submitCreate" :loading="creating">{{ t('planning.create') }}</BaseButton>
        </div>
      </div>
    </div>

    <!-- ===== NEUES WHITEBOARD: nur ein Titel ===== -->
    <BaseModal v-if="whiteboardModal" :title="t('planning.whiteboard.neu')" @close="whiteboardModal = null">
      <input v-model="whiteboardModal.title" type="text" maxlength="255" :placeholder="t('planning.titlePlaceholder.whiteboard')"
        @keydown.enter="submitWhiteboard"
        class="w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift" />
      <template #footer>
        <BaseButton variant="ghost" @click="whiteboardModal = null">{{ t('common.cancel') }}</BaseButton>
        <BaseButton :loading="creating" :disabled="!whiteboardModal.title.trim()" @click="submitWhiteboard">{{ t('planning.create') }}</BaseButton>
      </template>
    </BaseModal>

    <!-- ===== GEÖFFNETES WHITEBOARD: ganze Fläche ===== -->
    <WhiteboardAnsicht v-if="whiteboard" :key="whiteboard.uuid" :project-id="projectId" :start="whiteboard" @zurueck="closeWhiteboard" />

    <!-- ===== ABSCHLIESSEN-MODAL (nur Terminfindung) ===== -->
    <BaseModal v-if="closeModal" :title="t('planning.closeModalTitle')" @close="closeModal = null">
      <div class="space-y-4">
        <p class="text-sm text-leise">{{ t('planning.closeModalHint') }}</p>
        <select v-model="closeModal.finalOptionId" class="w-full px-3 py-2.5 bg-vertieft border border-linie rounded-xl text-sm text-schrift">
          <option v-for="o in poll.options" :key="o.id" :value="o.id">{{ fmtScheduleOption(o) }} — {{ t('planning.yesCount', { count: o.tally.yes }) }}</option>
        </select>
        <!-- Das Gerät trägt den Termin selbst ein (docs/plan-abstimmungen-e2e.md). -->
        <label class="flex items-center gap-2 text-sm font-medium text-fliess">
          <input v-model="closeModal.addToCalendar" type="checkbox" class="rounded border-slate-300 text-marke focus:ring-marke" />
          {{ t('planning.addToCalendar') }}
        </label>
      </div>
      <template #footer>
        <BaseButton variant="ghost" @click="closeModal = null">{{ t('common.cancel') }}</BaseButton>
        <BaseButton :loading="closing" @click="submitClose({ final_option_id: closeModal.finalOptionId, add_to_calendar: closeModal.addToCalendar })">{{ t('planning.finish') }}</BaseButton>
      </template>
    </BaseModal>
  </div>
</template>
