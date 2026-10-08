<script setup>
// Detailansicht EINES Schuljahres (Container, angelegt über den +Neu-Picker
// in der Planungsliste – wie eine Roadmap). Zeitraum oben, darunter die
// freien Abschnitte als Liste.
//
// Zwei Arten, und der Unterschied ist keine Kosmetik: Ferien schneiden
// später den Unterricht UND die Schulzeit-Betreuung heraus, ein einzelner
// freier Tag nur den Unterricht. Deshalb steht die Art an jeder Zeile und
// nicht bloß im Formular.
//
// Anlegen/Bearbeiten/Löschen von Abschnitten darf jedes Mitglied – der
// größte Teil kommt aus dem ICS-Import und trägt dessen Ersteller; wäre
// Löschen an ihn gebunden, käme der zweite Elternteil an einen falsch
// eingelesenen Termin nicht heran.
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { api, kann } from './umgebung';
import { Plus, RefreshCcw, Trash2, ArrowLeft, Loader2, CalendarRange, Pencil, X, Loader, Upload, Sun, CalendarOff } from 'lucide-vue-next';
import { useToast } from '@oberflaeche/composables/useToast';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import { initEcho, onEchoReconnect } from './umgebung';
import { debounce } from '@oberflaeche/shared/debounce';
import { formatDateLong } from '@oberflaeche/shared/date';
import { useHighlight } from '@oberflaeche/composables/useHighlight';
import BaseButton from '@oberflaeche/base/BaseButton.vue';

const { t } = useI18n();
const toast = useToast();
const { confirmDelete } = useConfirm();
const props = defineProps({
  projectId: { type: [Number, String], required: true },
  containerId: { type: [Number, String], required: true },
  meId: { type: Number, default: null },
  isOwner: { type: Boolean, default: false },
  highlightId: { type: Number, default: null },
});
const emit = defineEmits(['close']);

const loading = ref(true);
const error = ref('');
const jahr = ref(null);
const breaks = ref([]);
const modal = ref(null); // { id?, name, starts_on, ends_on, kind }
const saving = ref(false);
const importing = ref(false);
const dateiFeld = ref(null);
// Die mitgelieferten Ferien: Bundesland wählen statt Datei suchen. Die Liste
// kommt vom Server – dieselbe Tabelle, die auch importiert wird.
const laender = ref([]);
const landWahl = ref('');
const uebernehmend = ref(false);

const { highlighted, merkeEintrag } = useHighlight(
  () => props.highlightId,
  () => breaks.value.length,
);

// Ein eintägiger Abschnitt braucht keine Spanne: „24.12.2026" liest sich
// besser als „24.12.2026 – 24.12.2026".
const zeitraum = (b) => (b.starts_on === b.ends_on
  ? formatDateLong(b.starts_on)
  : `${formatDateLong(b.starts_on)} – ${formatDateLong(b.ends_on)}`);

const tage = (b) => {
  const von = new Date(b.starts_on);
  const bis = new Date(b.ends_on);
  return Math.round((bis - von) / 86400000) + 1;
};

// Was außerhalb des Schuljahres liegt, wirkt später nirgends – der Expander
// fragt nur innerhalb. Ein Abschnitt, der still nichts tut, soll das sagen.
const liegtDraussen = (b) => jahr.value
  && (b.ends_on < jahr.value.starts_on || b.starts_on > jahr.value.ends_on);

const summe = computed(() => breaks.value
  .filter((b) => !liegtDraussen(b))
  .reduce((n, b) => n + tage(b), 0));

const load = async () => {
  loading.value = true; error.value = '';
  try {
    jahr.value = (await api.getSchoolYear(props.projectId, props.containerId)).data;
    breaks.value = jahr.value.breaks || [];
  } catch (e) { error.value = t('projects.schoolYear.loadFailed'); }
  finally { loading.value = false; }
};

const openCreate = () => {
  modal.value = {
    name: '', kind: 'holiday',
    starts_on: jahr.value?.starts_on || '', ends_on: jahr.value?.starts_on || '',
  };
};
const openEdit = (b) => {
  modal.value = { id: b.id, name: b.name, starts_on: b.starts_on, ends_on: b.ends_on, kind: b.kind };
};

const submit = async () => {
  const m = modal.value;
  if (!m.name.trim() || !m.starts_on || !m.ends_on) { toast.error(t('projects.schoolYear.missingFields')); return; }
  if (m.ends_on < m.starts_on) { toast.error(t('projects.schoolYear.endBeforeStart')); return; }
  saving.value = true;
  const payload = { name: m.name.trim(), starts_on: m.starts_on, ends_on: m.ends_on, kind: m.kind };
  try {
    if (m.id) await api.updateSchoolYearBreak(props.projectId, props.containerId, m.id, payload);
    else await api.createSchoolYearBreak(props.projectId, props.containerId, payload);
    modal.value = null;
    await load();
  } catch (e) {
    toast.error(e.response?.data?.message || t('projects.schoolYear.saveFailed'));
  } finally { saving.value = false; }
};

const remove = async (b) => {
  if (!(await confirmDelete(t('projects.schoolYear.deleteBreakConfirm', { name: b.name })))) return;
  try {
    await api.deleteSchoolYearBreak(props.projectId, props.containerId, b.id);
    await load();
  } catch (e) { toast.error(t('projects.schoolYear.deleteFailed')); }
};

// Ferien-Import: Die Länder veröffentlichen ihre Termine als ICS. Ergänzt,
// ersetzt nicht – von Hand eingetragene Studientage bleiben stehen.
const importieren = async (event) => {
  const datei = event.target.files?.[0];
  if (!datei) return;
  importing.value = true;
  try {
    const res = await api.importSchoolYearBreaks(props.projectId, props.containerId, datei);
    toast.success(res.data.message);
    if (res.data.skipped_outside > 0) {
      toast.info(t('projects.schoolYear.importOutside', { count: res.data.skipped_outside }));
    }
    await load();
  } catch (e) {
    toast.error(e.response?.data?.message || t('projects.schoolYear.importFailed'));
  } finally {
    importing.value = false;
    // Zurücksetzen, sonst löst dieselbe Datei kein change mehr aus.
    if (dateiFeld.value) dateiFeld.value.value = '';
  }
};

const ferienUebernehmen = async () => {
  if (!landWahl.value) return;
  uebernehmend.value = true;
  try {
    const res = await api.importPresetBreaks(props.projectId, props.containerId, landWahl.value);
    toast.success(res.data.message);
    await load();
  } catch (e) {
    toast.error(e.response?.data?.message || t('projects.schoolYear.importFailed'));
  } finally {
    uebernehmend.value = false;
  }
};

const debouncedReload = debounce(() => load(), 400);
const onLive = (payload) => { if (payload?.kind === 'school_year') debouncedReload(); };
const channel = `project.${props.projectId}`;
let offReconnect = null;

onMounted(() => {
  load();
  api.getSchoolHolidayStates().then((r) => { laender.value = r.data; }).catch(() => {});
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
      <CalendarRange class="w-5 h-5 text-marke" />
      <h3 class="font-extrabold text-lg text-schrift truncate">{{ jahr?.name || t('projects.schoolYear.title') }}</h3>
      <BaseButton @click="openCreate" groesse="kopf" class="ml-auto"><Plus class="w-4 h-4" /> {{ t('projects.schoolYear.add') }}</BaseButton>
      <template v-if="kann('ferienImport')">
      <button @click="dateiFeld?.click()" :disabled="importing" class="flex items-center gap-1.5 px-3 py-2 text-fliess hover:bg-auflage rounded-xl text-sm font-bold disabled:opacity-50" :title="t('projects.schoolYear.importHint')">
        <Loader v-if="importing" class="w-4 h-4 animate-spin" /><Upload v-else class="w-4 h-4" /> {{ t('projects.schoolYear.import') }}
      </button>
      <input ref="dateiFeld" type="file" accept=".ics,text/calendar" class="hidden" @change="importieren" />

      <!-- Der kürzere Weg: Bundesland wählen, fertig. Die Datei-Einfuhr
           daneben bleibt für alles, was die Tabelle nicht kennt (freie
           Schulen, Auslandsschulen, ein neues Jahr vor dem Nachtragen). -->
      <div class="flex items-center gap-1.5">
        <select v-model="landWahl" class="h-9 px-2 bg-vertieft border border-linie rounded-xl text-sm font-semibold text-fliess max-w-[11rem]">
          <option value="">{{ t('projects.schoolYear.statePlaceholder') }}</option>
          <option v-for="l in laender" :key="l.code" :value="l.code">{{ l.name }}</option>
        </select>
        <button @click="ferienUebernehmen" :disabled="!landWahl || uebernehmend"
          class="flex items-center gap-1.5 px-3 py-2 text-fliess hover:bg-auflage rounded-xl text-sm font-bold disabled:opacity-40">
          <Loader v-if="uebernehmend" class="w-4 h-4 animate-spin" /><CalendarOff v-else class="w-4 h-4" /> {{ t('projects.schoolYear.usePreset') }}
        </button>
      </div>
      </template>
      <button @click="load" class="p-2 text-slate-400 hover:bg-auflage rounded-xl" :title="t('projects.schoolYear.title')"><RefreshCcw class="w-4 h-4" :class="loading ? 'animate-spin text-marke' : ''" /></button>
    </div>

    <p v-if="jahr" class="text-sm font-semibold text-leise">
      {{ formatDateLong(jahr.starts_on) }} – {{ formatDateLong(jahr.ends_on) }}
      <span v-if="summe > 0" class="ml-2 text-slate-400">· {{ t('projects.schoolYear.freeDays', { count: summe }) }}</span>
    </p>

    <div v-if="loading" class="py-10 text-center text-slate-400"><Loader2 class="w-6 h-6 animate-spin mx-auto" /></div>
    <p v-else-if="error" class="text-sm text-rose-500 font-medium">{{ error }}</p>
    <p v-else-if="breaks.length === 0" class="py-8 text-center text-sm text-slate-400 dark:text-slate-500 font-medium">{{ t('projects.schoolYear.empty') }}</p>

    <ul v-else class="space-y-2">
      <li v-for="b in breaks" :key="b.id"
        :ref="(el) => merkeEintrag(b.id, el)"
        class="flex items-center gap-3 px-3 py-2.5 rounded-xl border transition-colors"
        :class="b.id === highlighted
          ? 'bg-marke-leise border-marke'
          : 'bg-vertieft border-linie'">
        <Sun v-if="b.kind === 'holiday'" class="w-4 h-4 text-amber-500 shrink-0" />
        <CalendarOff v-else class="w-4 h-4 text-slate-400 shrink-0" />
        <div class="min-w-0 flex-1">
          <div class="flex items-center gap-2 flex-wrap">
            <span class="font-bold text-schrift truncate">{{ b.name }}</span>
            <span class="text-xs font-bold px-2 py-0.5 rounded-full"
              :class="b.kind === 'holiday'
                ? 'bg-amber-50 dark:bg-amber-900/30 text-amber-600 dark:text-amber-400'
                : 'bg-auflage text-leise'">
              {{ t(`projects.schoolYear.kind.${b.kind}`) }}
            </span>
            <!-- Ein Abschnitt außerhalb des Schuljahres wirkt nirgends. Ohne
                 diesen Hinweis sucht man den Fehler später im Stundenplan. -->
            <span v-if="liegtDraussen(b)" class="text-xs font-bold px-2 py-0.5 rounded-full bg-rose-50 dark:bg-rose-900/30 text-rose-600 dark:text-rose-400">{{ t('projects.schoolYear.outside') }}</span>
          </div>
          <p class="text-xs font-semibold text-leise mt-0.5">
            {{ zeitraum(b) }}<span v-if="tage(b) > 1" class="text-slate-400"> · {{ t('projects.schoolYear.dayCount', { count: tage(b) }) }}</span>
          </p>
        </div>
        <button @click="openEdit(b)" class="p-1.5 text-slate-400 hover:text-marke hover:bg-marke-leise rounded-xl" :title="t('projects.schoolYear.editTitle')"><Pencil class="w-4 h-4" /></button>
        <button @click="remove(b)" class="p-1.5 text-slate-400 hover:text-rose-500 hover:bg-rose-50 dark:hover:bg-rose-900/20 rounded-xl" :title="t('projects.schoolYear.deleteBreakTitle')"><Trash2 class="w-4 h-4" /></button>
      </li>
    </ul>

    <!-- Anlegen/Bearbeiten -->
    <div v-if="modal" class="fixed inset-0 bg-slate-900/60 backdrop-blur-sm flex items-center justify-center p-4 z-[100]" @click.self="modal = null">
      <div class="karte w-full max-w-lg shadow-2xl max-h-[90vh] overflow-y-auto">
        <div class="flex items-center justify-between px-6 py-4 border-b border-linie">
          <h3 class="font-extrabold text-schrift flex items-center gap-2"><CalendarRange class="w-5 h-5 text-marke" /> {{ modal.id ? t('projects.schoolYear.editTitle') : t('projects.schoolYear.newTitle') }}</h3>
          <button @click="modal = null" class="p-1.5 text-slate-400 hover:bg-auflage rounded-xl"><X class="w-5 h-5" /></button>
        </div>
        <div class="p-6 space-y-4">
          <div>
            <label class="text-xs font-bold text-leise uppercase">{{ t('projects.schoolYear.nameLabel') }}</label>
            <input v-model="modal.name" type="text" maxlength="255" :placeholder="t('projects.schoolYear.namePlaceholder')" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift" />
          </div>
          <div class="grid grid-cols-2 gap-3">
            <div>
              <label class="text-xs font-bold text-leise uppercase">{{ t('planning.fields.starts_on') }}</label>
              <input v-model="modal.starts_on" type="date" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift" />
            </div>
            <div>
              <label class="text-xs font-bold text-leise uppercase">{{ t('planning.fields.ends_on') }}</label>
              <input v-model="modal.ends_on" type="date" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift" />
            </div>
          </div>
          <div>
            <label class="text-xs font-bold text-leise uppercase">{{ t('projects.schoolYear.kindLabel') }}</label>
            <select v-model="modal.kind" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift">
              <option value="holiday">{{ t('projects.schoolYear.kind.holiday') }}</option>
              <option value="free_day">{{ t('projects.schoolYear.kind.free_day') }}</option>
            </select>
            <p class="text-xs text-leise mt-1.5">{{ t(`projects.schoolYear.kindHint.${modal.kind}`) }}</p>
          </div>
        </div>
        <div class="flex justify-end gap-2 px-6 py-4 border-t border-linie">
          <button @click="modal = null" class="px-4 py-2 text-fliess hover:bg-auflage rounded-xl text-sm font-bold">{{ t('common.cancel') }}</button>
          <BaseButton @click="submit" :loading="saving">{{ t('common.save') }}</BaseButton>
        </div>
      </div>
    </div>
  </div>
</template>
