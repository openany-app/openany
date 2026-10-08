<script setup>
// Der Reiter „Planung" eines Projekts: EINE Liste über alles, was dort
// nebeneinander liegt, plus der Vorlagen-Picker („+ Neu → Was möchtest du
// starten?"), über den alles angelegt wird. Bewusst keine zweite
// Navigationsebene: Ein neuer Planungs-Typ wird eine Zeile in dieser Liste,
// kein eigener Reiter.
//
// Zwei Arten von Eintrag, und der Unterschied ist echt:
//
//   Behälter (Board, Roadmap, Orte) haben eigene Tabellen und Endpunkte.
//     Was es gibt, steht in shared/planningTypes.js und
//     planningContainers.js – nicht hier.
//   Abstimmungen laufen über einen generischen Motor mit einem type-Feld
//     (Terminfindung, Umfrage, Schichtplan, Mitbringliste, Einladungsliste).
//     Ihre Formulare und Detailansichten leben hier bzw. unter
//     components/project/polls/.
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
import {
  PLANNING_CONTAINERS, containerByTemplateKey, containerByKind, containerByItemKind, rowTemplate,
} from './planningContainers';
import { CONTAINER_SIGNALS } from './planningTypes';
import { STUNDENRASTER, RASTER_KEYS } from '@oberflaeche/shared/stundenraster';
import {
  Plus, RefreshCcw, Trash2, ArrowLeft, Loader2, CalendarClock, X, Vote,
  CheckCircle2, Lock, BarChart3, ListChecks, ShoppingBasket, Copy, Clock,
  PartyPopper, ChevronLeft, ChevronRight, GraduationCap,
} from 'lucide-vue-next';
import { useToast } from '@oberflaeche/composables/useToast';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import { initEcho, onEchoReconnect, kann } from './umgebung';
import { debounce } from '@oberflaeche/shared/debounce';
import { formatTime, formatWeekdayTime } from '@oberflaeche/shared/date';

const toast = useToast();
const { t } = useI18n();
const { confirmDialog, confirmDelete } = useConfirm();
const props = defineProps({
  projectId: { type: [Number, String], required: true },
  meId: { type: Number, default: null },
  isOwner: { type: Boolean, default: false },
  members: { type: Array, default: () => [] },
  // Ziel eines [[Verweises]] aus dem Chat: ein Behälter (board, roadmap,
  // places) oder ein Einzelpunkt darin (card, milestone, place) samt der Id
  // seines Behälters. Wirkt beim Aufbau – ProjectDetail baut den Reiter
  // dafür eigens neu auf.
  openTarget: { type: Object, default: null },
});

// Der Bestand dieser Liste, für das Zahlen-Abzeichen am Reiter. Wer ihn
// zeigt, lädt ihn nicht – wer ihn kennt, zeigt ihn nicht.
const emit = defineEmits(['count']);

// Vorlagen: bestimmen type + settings-Defaults des generischen Motors. Die
// Behälter-Vorlagen kommen aus planningContainers – ein neuer Planungs-Typ
// erscheint hier, ohne dass diese Datei etwas davon wissen muss.
const POLL_TEMPLATES = [
  { key: 'schedule', type: 'schedule', label: t('planning.templates.schedule.label'), icon: CalendarClock, hint: t('planning.templates.schedule.hint') },
  { key: 'survey', type: 'survey', label: t('planning.templates.survey.label'), icon: BarChart3, hint: t('planning.templates.survey.hint') },
  { key: 'shift', type: 'signup', label: t('planning.templates.shift.label'), icon: ListChecks, hint: t('planning.templates.shift.hint') },
  { key: 'bring', type: 'signup', label: t('planning.templates.bring.label'), icon: ShoppingBasket, hint: t('planning.templates.bring.hint') },
  { key: 'invite', type: 'invite', label: t('planning.templates.invite.label'), icon: PartyPopper, hint: t('planning.templates.invite.hint') },
];

// `flatMap`, weil ein Behälter mehrere Kacheln anbieten darf: Stundenplan und
// Betreuungsplan sind derselbe Typ mit verschiedener Vorbelegung. Die Kachel
// entscheidet, womit angelegt wird – nicht, was danach verwaltet wird.
const CONTAINER_TEMPLATES = PLANNING_CONTAINERS.flatMap((c) => c.templateKeys.map((key) => ({
  key,
  type: key,
  label: t(`planning.templates.${key}.label`),
  hint: t(`planning.templates.${key}.hint`),
  icon: c.templateIcons?.[key] || c.icon,
})));

// Kacheln, die KEINEN Behälter anlegen, sondern mehrere auf einmal.
//
// Bis hierher gilt: eine Kachel, ein Behälter. „Preset: Schule" bricht das
// bewusst – wer ein Schulprojekt aufsetzt, braucht vier Zeilen, die
// aufeinander zeigen (der Stundenplan muss das Schuljahr kennen, sonst zeigt
// er Unterricht in den Ferien). Vier Kacheln nacheinander zu klicken hieße,
// diese Verbindung von Hand herzustellen und dabei genau die eine
// Einstellung zu vergessen, die man nicht sieht.
//
// Ohne Betreuungsplan: Viele Familien haben gar keine Aufteilung, und eine
// Kachel, die ungefragt eine anlegt, macht aus einem Angebot eine
// Unterstellung. Wer eine hat, legt sie mit ihrer eigenen Kachel dazu.
const SETUP_TEMPLATES = [
  {
    key: 'schulsetup',
    type: 'schulsetup',
    label: t('planning.templates.schulsetup.label'),
    hint: t('planning.templates.schulsetup.hint'),
    icon: GraduationCap,
    istEinrichtung: true,
  },
];

// EIN Regal, alphabetisch nach dem ANGEZEIGTEN Namen – also je Sprache
// verschieden, mit `localeCompare('de')`, damit Umlaute dort einsortieren,
// wo man sie sucht (Ä bei A und nicht hinter Z).
//
// Vorher standen hier zwei Regale, getrennt nach Behältern und Abstimmungen.
// Der Unterschied ist echt (eigene Tabellen gegen einen generischen Motor),
// aber bei neun Einträgen half die Trennung beim Suchen nicht – sie zwang
// nur dazu, erst die richtige Reihe zu finden. Wer den Namen kennt, ist mit
// einer einzigen Ordnung schneller.
//
// Sortiert werden KOPIEN, und das ist kein Stilzwang: `typeMeta()` greift
// POLL_TEMPLATES über den Index an (`[0]` = Terminfindung, `[4]` =
// Einladungsliste). In-place sortiert bekäme jede Abstimmung in der Liste
// stillschweigend das falsche Symbol und den falschen Typnamen. Ebenso
// PLANNING_CONTAINERS: Dessen Reihenfolge trägt in loadPolls() die Zuordnung
// der parallelen Antworten (`containerRes[i]`).
// Was der Rahmen nicht kann, steht gar nicht erst zur Wahl (umgebung.js).
const PICKER_TEMPLATES = [
  ...(kann('abstimmungenVerwalten') ? POLL_TEMPLATES : []),
  ...CONTAINER_TEMPLATES,
  ...(kann('schulPaket') ? SETUP_TEMPLATES : []),
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

// Container-Vorlagen (Board/Roadmap/Orte) haben keine Poll-Optionen/-Settings,
// nur einen Namen.
const isContainer = (k) => containerByTemplateKey(k) !== null;

// Das Symbol der Zeile folgt der Vorlage, nicht dem Behälter: Stundenplan und
// Betreuungsplan sind derselbe Typ und sollen trotzdem auseinanderzuhalten
// sein, ohne den Namen zu lesen.
const zeilenSymbol = (entry) => entry.container.templateIcons?.[rowTemplate(entry.container, entry.item)]
  || entry.container.icon;

const typeMeta = (p) => {
  if (p.type === 'schedule') return POLL_TEMPLATES[0];
  if (p.type === 'survey') return POLL_TEMPLATES[1];
  if (p.type === 'invite') return POLL_TEMPLATES[4];
  return p.settings?.variant === 'bring' ? POLL_TEMPLATES[3] : POLL_TEMPLATES[2];
};

const loading = ref(true);
const error = ref('');
const polls = ref([]);
// Je Behälter-Art ihre geladene Liste – statt einer eigenen ref je Art.
const containerLists = ref(Object.fromEntries(PLANNING_CONTAINERS.map((c) => [c.kind, []])));
const poll = ref(null); // geöffnete Abstimmung (Detail)
// Welcher Behälter offen ist und welcher Einzelpunkt darin hervorzuheben ist –
// vorher drei Paare aus openXId und openYId.
const openContainer = ref(null); // { kind, id, highlightId } | null

// Aus dem Chat heraus direkt ins Ziel springen. Die Ansichten laden sich
// selbst über die Id – hier ist nur zu entscheiden, WELCHE aufgeht.
// Läuft vor dem ersten Laden der Liste, was nichts ausmacht: Die Detailsicht
// verdeckt sie ohnehin, und beim Schließen ist die Liste da.
//
// Ein Einzelpunkt (Karte, Meilenstein, Ort) hat keine eigene Ansicht: Zeigen
// lässt er sich nur, indem sein Behälter aufgeht. Dessen Id kommt deshalb aus
// der Auflösung mit – hier ist sie nicht zu ermitteln.
if (props.openTarget) {
  const ziel = props.openTarget;
  const alsBehaelter = containerByKind(ziel.kind);
  const alsPunkt = containerByItemKind(ziel.kind);

  if (alsBehaelter) openContainer.value = { kind: alsBehaelter.kind, id: ziel.id, highlightId: null };
  else if (alsPunkt) openContainer.value = { kind: alsPunkt.kind, id: ziel[alsPunkt.parentField], highlightId: ziel.id };
}
const pollLoading = ref(false);

// Ist gerade irgendein Detail (Poll oder Behälter) offen?
const anyDetailOpen = () => poll.value || openContainer.value;

// Ansicht und Eigenschaften des offenen Behälters. Welche Eigenschaften eine
// Ansicht braucht, weiß ihr Registry-Eintrag – die drei erwarten nicht
// dasselbe.
const openContainerMeta = computed(() => (openContainer.value ? containerByKind(openContainer.value.kind) : null));
const openContainerProps = computed(() => (openContainerMeta.value
  ? openContainerMeta.value.props(props, openContainer.value)
  : {}));

const pickerOpen = ref(false);
// Bundesländer für die Einrichtung – erst geladen, wenn jemand sie braucht.
const laender = ref([]);
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
    const [pollsRes, ...containerRes] = await Promise.all([
      api.getPolls(props.projectId),
      ...PLANNING_CONTAINERS.map((c) => c.list(props.projectId)),
    ]);
    polls.value = pollsRes.data;
    PLANNING_CONTAINERS.forEach((c, i) => { containerLists.value[c.kind] = containerRes[i].data; });
    meldeBestand();
  }
  catch (e) { error.value = t('planning.loadFailed'); }
  finally { loading.value = false; }
};

// Gemeinsame Planungs-Liste: Abstimmungen/Listen und alle Behälter, neueste
// zuerst.
const planItems = computed(() => [
  ...polls.value.map((p) => ({ kind: 'poll', container: null, sortKey: p.created_at, item: p })),
  ...PLANNING_CONTAINERS.flatMap((c) => containerLists.value[c.kind]
    .map((item) => ({ kind: c.kind, container: c, sortKey: item.created_at, item }))),
].sort((a, b) => new Date(b.sortKey) - new Date(a.sortKey)));

// Der Zähler am Reiter „Planung" gehört der Projektansicht, der Bestand
// steht aber hier: Diese Liste IST, was Project::planningCount() zählt –
// Abstimmungen plus alle Behälter. Bisher kam die Zahl nur mit dem
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

// --- Behälter (Board, Roadmap, Orte) ---
const closeContainer = () => { openContainer.value = null; loadPolls(); };

const deleteContainer = async (container, item) => {
  if (!(await confirmDelete(t(`${container.i18n}.deleteConfirm`, { name: item.name })))) return;
  try {
    await container.remove(props.projectId, item.id);
    await loadPolls();
  } catch (e) {
    toast.error(e.response?.status === 403
      ? t(`${container.i18n}.deleteForbidden`)
      : t(`${container.i18n}.deleteFailed`));
  }
};

// Echtzeit: fremde Board-Änderungen halten die Liste frisch, solange kein
// Board offen ist (das offene Board lädt KanbanBoardView selbst nach).
// Der project.{id}-Kanal gehört ProjectDetail – hier nur Listener.
const debouncedListReload = debounce(() => loadPolls(), 400);
const onKanbanChange = () => {
  if (!anyDetailOpen()) debouncedListReload();
};
// Behälter angelegt/gelöscht → Liste frisch halten. Änderungen an
// Einzelpunkten (Meilenstein, Ort) betreffen nur die jeweilige offene
// Detailansicht und kommen mit einer anderen `kind`-Angabe.
const onPlanningChange = (payload) => {
  if (CONTAINER_SIGNALS.includes(payload?.kind) && !anyDetailOpen()) debouncedListReload();
};
const echoChannelName = `project.${props.projectId}`;
let offReconnect = null;

// --- Erstellen ---
// Vorbelegung der Zeitraum-Felder: das laufende SCHULJAHR, nicht „heute bis
// heute in einem Jahr". Ein Schuljahr fängt im August an; wer im März ein
// Schuljahr anlegt, meint das laufende und nicht eines, das im März beginnt.
// Vor August gehört man noch ins vorige.
const jahresVorgabe = (feld) => {
  const heute = new Date();
  const startjahr = heute.getMonth() >= 7 ? heute.getFullYear() : heute.getFullYear() - 1;

  return feld === 'ends_on' ? `${startjahr + 1}-07-31` : `${startjahr}-08-01`;
};

const openCreate = (template) => {
  pickerOpen.value = false;
  const base = { template, title: '', description: '', expires_at: '', notifyMembers: false };
  if (template.key === 'schedule') {
    createModal.value = { ...base, options: [{ starts_at: '', ends_at: '' }, { starts_at: '', ends_at: '' }] };
  } else if (template.key === 'survey') {
    createModal.value = { ...base, options: [{ label: '' }, { label: '' }], settings: { multiple: false, anonymous: false } };
  } else if (template.key === 'invite') {
    // Gäste: Projektmitglieder ankreuzen + freie Namen (Menschen ohne Konto).
    createModal.value = { ...base, memberIds: [], options: [{ label: '' }, { label: '' }] };
  } else if (template.istEinrichtung) {
    // Kein Name: Die vier Behälter heißen, wie sie heißen sollen („Stundenplan",
    // „Klassenarbeiten und Tests"). Zu fragen gibt es nur den Zeitraum – und
    // den braucht das Schuljahr, sonst könnte es seine eigene Frage nicht
    // beantworten.
    createModal.value = {
      ...base,
      options: [],
      istEinrichtung: true,
      extra: { starts_on: jahresVorgabe('starts_on'), ends_on: jahresVorgabe('ends_on') },
      // Das Stundenraster gleich mit: Ein Stundenplan ohne Raster fragt beim
      // Eintragen nach zwei Uhrzeiten, einer mit Raster nach der 3. Stunde.
      // Vorbelegt mit dem verbreitetsten – geändert wird es hier oder später
      // in den Einstellungen des Plans.
      rasterKey: 'klassisch',
      // Bundesland: Damit stehen die Ferien sofort im Schuljahr, statt dass
      // jemand sie als Datei suchen muss. Leer bleibt zulässig – wer
      // außerhalb Deutschlands zur Schule geht, trägt sie von Hand ein.
      state: '',
    };
    if (laender.value.length === 0) {
      api.getSchoolHolidayStates().then((r) => { laender.value = r.data; }).catch(() => {});
    }
  } else if (isContainer(template.key)) {
    // Container brauchen meist nur einen Namen. Das Schuljahr braucht dazu
    // seinen Zeitraum – welche Felder ein Behälter beim Anlegen mitbringt,
    // sagt er selbst über `createFields`, statt dass diese Datei die Arten
    // einzeln kennt. Vorgabe: ein Jahr ab heute, damit die Felder nicht leer
    // dastehen; ein Schuljahr fängt selten am 1. Januar an.
    // `extra` bleibt undefined, wenn der Behälter keine Felder nennt – ein
    // leeres Objekt wäre truthy und ließe ein leeres Raster im Formular stehen.
    //
    // `vorgaben` ist etwas anderes als `extra`, und die Trennung ist nötig:
    // `extra` sind Felder, die im Formular STEHEN (und dort als Datumsfelder
    // gezeichnet werden), `vorgaben` ist, was die gewählte Kachel still
    // mitbringt – beim Wochenplan `type` und `applies_to`. Beides zusammen
    // geht ans Anlegen, aber nur `extra` wird gezeigt.
    const behaelter = containerByTemplateKey(template.key);
    const felder = behaelter.createFields || [];
    createModal.value = {
      ...base,
      options: [],
      vorgaben: behaelter.templateDefaults?.[template.key] || {},
      ...(felder.length > 0
        ? { extra: Object.fromEntries(felder.map((f) => [f, jahresVorgabe(f)])) }
        : {}),
    };
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

  // Die Einrichtung legt VIER Behälter an – in einer Transaktion auf dem
  // Server, nicht in vier Anfragen von hier. Danach wird keiner davon
  // geöffnet: Welcher von vieren wäre schon der richtige? Die Liste zeigt
  // sie alle.
  if (m.istEinrichtung) {
    creating.value = true;
    try {
      const res = await api.setUpSchool(props.projectId, {
        ...m.extra,
        state: m.state || null,
        periods: m.rasterKey ? STUNDENRASTER[m.rasterKey]() : [],
      }, m.notifyMembers);
      createModal.value = null;
      await loadPolls();
      const angelegt = res.data.created.length;
      toast.success(angelegt > 0
        ? t('planning.setupDone', { count: angelegt })
        : t('planning.setupNothingToDo'));
    } catch (e) {
      toast.error(e.response?.data?.message || t('planning.createFailed'));
    } finally {
      creating.value = false;
    }
    return;
  }

  // Container (Board/Roadmap/Orte) laufen nicht über den Poll-Motor: eigener
  // Endpoint, danach direkt öffnen.
  const container = containerByTemplateKey(tpl.key);
  if (container) {
    if (!m.title.trim()) {
      toast.error(t('planning.createMissingContainerTitle'));
      return;
    }
    creating.value = true;
    try {
      const res = await container.create(props.projectId, m.title.trim(), m.notifyMembers, {
        ...(m.vorgaben || {}),
        ...(m.extra || {}),
      });
      openContainer.value = { kind: container.kind, id: res.data.id, highlightId: null };
      createModal.value = null;
      // Der neue Behälter geht sofort auf, die Liste dahinter kennt ihn aber
      // noch nicht – und damit wäre auch der gemeldete Bestand um eins zu
      // niedrig, bis man zurückkommt. Nachladen ohne Warten: Die Liste ist
      // hier ohnehin verdeckt.
      loadPolls();
    } catch (e) {
      toast.error(e.response?.data?.message || t('planning.createFailed'));
    } finally {
      creating.value = false;
    }
    return;
  }

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
    // Wie beim Behälter: Die verdeckte Liste nachziehen, damit der gemeldete
    // Bestand die neue Abstimmung mitzählt.
    loadPolls();
  } catch (e) {
    toast.error(e.response?.data?.message || t('planning.createFailed'));
  } finally {
    creating.value = false;
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
  if (initEcho()) {
    initEcho().private(echoChannelName).listen('KanbanBoardChanged', onKanbanChange);
    initEcho().private(echoChannelName).listen('ProjectPlanningChanged', onPlanningChange);
    offReconnect = onEchoReconnect(() => { if (!anyDetailOpen()) loadPolls(); });
  }
});
onUnmounted(() => {
  window.removeEventListener('resize', pruefeRegal);
  initEcho()?.private(echoChannelName).stopListening('KanbanBoardChanged', onKanbanChange);
  initEcho()?.private(echoChannelName).stopListening('ProjectPlanningChanged', onPlanningChange);
  offReconnect?.();
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

    <!-- ===== BEHÄLTER-DETAIL (Board, Roadmap, Orte) ===== -->
    <component
      :is="openContainerMeta?.view"
      v-else-if="openContainer"
      v-bind="openContainerProps"
      :project-id="projectId"
      :container-id="openContainer.id"
      @close="closeContainer"
    />

    <!-- ===== LISTE ===== -->
    <div v-else class="karte shadow-sm p-6 space-y-5">
      <div class="flex items-center gap-2">
        <Vote class="w-5 h-5 text-marke" />
        <h3 class="font-extrabold text-lg text-schrift">{{ t('planning.title') }}</h3>
        <BaseButton @click="pickerOpen = true" groesse="kopf" class="ml-auto"><Plus class="w-4 h-4" /> {{ t('planning.newPoll') }}</BaseButton>
        <button @click="loadPolls" class="p-2 text-slate-400 hover:bg-auflage rounded-xl" :title="t('planning.refresh')"><RefreshCcw class="w-4 h-4" :class="loading ? 'animate-spin text-marke' : ''" /></button>
      </div>

      <div v-if="loading" class="py-10 text-center text-slate-400"><Loader2 class="w-6 h-6 animate-spin mx-auto" /></div>
      <p v-else-if="error" class="text-sm text-rose-500 font-medium">{{ error }}</p>
      <p v-else-if="planItems.length === 0" class="py-8 text-center text-sm text-slate-400 dark:text-slate-500 font-medium">{{ t('planning.emptyList') }}</p>
      <ul v-else class="divide-y divide-linie">
        <li v-for="entry in planItems" :key="`${entry.kind}-${entry.item.id}`" class="py-3 flex items-center gap-3">
          <!-- Behälter-Zeile: Board, Roadmap, Orte – eine Zeile für alle -->
          <template v-if="entry.container">
            <button @click="openContainer = { kind: entry.container.kind, id: entry.item.id, highlightId: null }" class="flex items-center gap-3 flex-1 min-w-0 text-left group">
              <component :is="zeilenSymbol(entry)" class="w-4 h-4 text-marke shrink-0" />
              <span class="font-bold text-schrift truncate group-hover:text-marke">{{ entry.item.name }}</span>
              <span class="text-xs font-medium text-slate-400 shrink-0">{{ t(`planning.templates.${rowTemplate(entry.container, entry.item)}.label`) }}</span>
              <span class="text-xs font-medium text-slate-400 shrink-0">{{ entry.item[entry.container.countAttr] }} {{ entry.item[entry.container.countAttr] === 1 ? t(`${entry.container.i18n}.countSingular`) : t(`${entry.container.i18n}.countPlural`) }}</span>
            </button>
            <button v-if="entry.item.created_by === meId || isOwner" @click="deleteContainer(entry.container, entry.item)" class="p-1.5 text-slate-400 hover:text-rose-500 hover:bg-rose-50 dark:hover:bg-rose-900/20 rounded-xl" :title="t(`${entry.container.i18n}.deleteTitle`)"><Trash2 class="w-4 h-4" /></button>
          </template>

          <!-- Abstimmungs-/Listen-Zeile -->
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
          <!-- Die Einrichtung fragt keinen Namen: Ihre vier Behälter heißen
               bereits so, wie sie heißen sollen. -->
          <input v-if="!createModal.istEinrichtung" v-model="createModal.title" type="text" :placeholder="t(`planning.titlePlaceholder.${createModal.template.key}`)" maxlength="255" class="w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift" />
          <textarea v-if="!isContainer(createModal.template.key) && !createModal.istEinrichtung" v-model="createModal.description" rows="2" :placeholder="t('planning.descriptionPlaceholder')" class="w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift resize-none"></textarea>

          <!-- Zeitraum-Felder eines Behälters (heute nur das Schuljahr). Was
               hier steht, sagt der Behälter über `createFields`. -->
          <div v-if="createModal.extra" class="grid grid-cols-2 gap-3">
            <div v-for="feld in Object.keys(createModal.extra)" :key="feld">
              <label class="text-xs font-bold text-leise uppercase">{{ t(`planning.fields.${feld}`) }}</label>
              <input v-model="createModal.extra[feld]" type="date" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift" />
            </div>
          </div>

          <div v-if="createModal.istEinrichtung">
            <label class="text-xs font-bold text-leise uppercase">{{ t('planning.setupStateLabel') }}</label>
            <select v-model="createModal.state" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift">
              <option value="">{{ t('planning.setupStateNone') }}</option>
              <option v-for="l in laender" :key="l.code" :value="l.code">{{ l.name }}</option>
            </select>
            <p class="text-xs text-leise mt-1.5">{{ t('planning.setupStateHint') }}</p>
          </div>

          <!-- Stundenraster der Einrichtung. Nur die Wahl, nicht die
               Bearbeitung: Wer einzelne Zeiten verschieben will, tut das
               danach im Stundenplan selbst – hier wäre es eine Tabelle in
               einem Dialog, der eine Frage beantworten soll. -->
          <div v-if="createModal.istEinrichtung">
            <label class="text-xs font-bold text-leise uppercase">{{ t('planning.setupGridLabel') }}</label>
            <div class="flex flex-wrap gap-1.5 mt-1.5">
              <button v-for="key in RASTER_KEYS" :key="key" type="button" @click="createModal.rasterKey = key"
                class="px-2.5 py-1.5 text-xs font-bold rounded-lg border"
                :class="createModal.rasterKey === key
                  ? 'bg-marke-leise border-marke text-marke'
                  : 'bg-vertieft border-linie text-fliess'">
                {{ t(`projects.weekPlan.grids.${key}`) }}
              </button>
              <button type="button" @click="createModal.rasterKey = null"
                class="px-2.5 py-1.5 text-xs font-bold rounded-lg border"
                :class="createModal.rasterKey === null
                  ? 'bg-marke-leise border-marke text-marke'
                  : 'bg-vertieft border-linie text-fliess'">
                {{ t('planning.setupGridNone') }}
              </button>
            </div>
            <p class="text-xs text-leise mt-1.5">{{ t('planning.setupGridHint') }}</p>
          </div>

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

          <!-- Optionen: Schichtplan / Mitbringliste (Boards haben keine) -->
          <div v-else-if="!isContainer(createModal.template.key)" class="space-y-2">
            <label class="text-xs font-bold text-leise uppercase">{{ createModal.template.key === 'shift' ? t('planning.entriesLabelShift') : t('planning.entriesLabelBring') }}</label>
            <div v-for="(o, i) in createModal.options" :key="i" class="flex items-center gap-2">
              <input v-model="o.label" type="text" maxlength="255" :placeholder="createModal.template.key === 'shift' ? t('planning.entryPlaceholderShift', { n: i + 1 }) : t('planning.entryPlaceholderBring', { n: i + 1 })" class="flex-1 px-3 py-2 bg-vertieft border border-linie rounded-xl text-sm text-schrift" />
              <input v-model="o.capacity" type="number" min="1" max="999" :placeholder="createModal.template.key === 'shift' ? t('planning.capacityPlaceholderShift') : t('planning.capacityPlaceholderOther')" class="w-24 px-3 py-2 bg-vertieft border border-linie rounded-xl text-sm text-schrift" />
              <button @click="removeCreateOption(i)" :disabled="createModal.options.length <= 1" class="p-1.5 text-slate-400 hover:text-rose-500 disabled:opacity-30 rounded-xl"><Trash2 class="w-4 h-4" /></button>
            </div>
          </div>

          <button v-if="!isContainer(createModal.template.key)" @click="addCreateOption" class="flex items-center gap-1.5 text-sm font-bold text-marke hover:underline"><Plus class="w-4 h-4" /> {{ createModal.template.key === 'invite' ? t('planning.addName') : t('planning.addOption') }}</button>

          <!-- Einstellungen (Boards + Einladungsliste haben kein Ablaufdatum) -->
          <div v-if="!isContainer(createModal.template.key) && createModal.template.key !== 'invite'" class="space-y-2 pt-2 border-t border-linie">
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

    <!-- ===== ABSCHLIESSEN-MODAL (nur Terminfindung) ===== -->
    <BaseModal v-if="closeModal" :title="t('planning.closeModalTitle')" @close="closeModal = null">
      <div class="space-y-4">
        <p class="text-sm text-leise">{{ t('planning.closeModalHint') }}</p>
        <select v-model="closeModal.finalOptionId" class="w-full px-3 py-2.5 bg-vertieft border border-linie rounded-xl text-sm text-schrift">
          <option v-for="o in poll.options" :key="o.id" :value="o.id">{{ fmtScheduleOption(o) }} — {{ t('planning.yesCount', { count: o.tally.yes }) }}</option>
        </select>
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
