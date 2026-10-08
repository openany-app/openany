<script setup>
// Detailansicht EINER Roadmap (Container, angelegt über den +Neu-Picker in
// der Planungsliste – wie ein Board). Lädt sich selbst über containerId und
// zeigt die Meilensteine als vertikale Zeitachse mit „Heute"-Markierung.
// Jedes Mitglied darf Meilensteine anlegen/bearbeiten/Status setzen; löschen
// nur der Ersteller oder Projekt-Owner. Live-Sync via ProjectPlanningChanged.
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from './umgebung';
import ClampText from '@oberflaeche/base/ClampText.vue';
import PlacePicker from './PlacePicker.vue';
import SubjectPicker from './SubjectPicker.vue';
import { Plus, RefreshCcw, Trash2, ArrowLeft, Loader2, Flag, CheckCircle2, Circle, Pencil, X, MapPin } from 'lucide-vue-next';
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
  // Meilenstein aus einem [[Verweis]] im Chat: hervorheben und ins Bild
  // rollen. Nicht zum Bearbeiten aufschlagen – ein Verweis führt zum
  // Nachsehen, nicht in ein Formular.
  highlightId: { type: Number, default: null },
});
const emit = defineEmits(['close']);

const loading = ref(true);
const error = ref('');
const roadmap = ref(null);
const milestones = ref([]);
const modal = ref(null); // { id?, title, description, due_date, status }
const saving = ref(false);

const { highlighted, merkeEintrag } = useHighlight(
  () => props.highlightId,
  () => milestones.value.length,
);

const todayStr = () => new Date().toISOString().slice(0, 10);

const canDelete = (m) => m.created_by === props.meId || props.isOwner;
const isOverdue = (m) => m.status === 'open' && m.due_date < todayStr();

// Zeitleiste: Meilensteine (nach Datum sortiert vom Server) plus eine
// „Heute"-Marke an der Stelle, wo das heutige Datum einsortiert wäre.
const timeline = computed(() => {
  const today = todayStr();
  const out = [];
  let inserted = false;
  for (const m of milestones.value) {
    if (!inserted && m.due_date >= today) { out.push({ type: 'today' }); inserted = true; }
    out.push({ type: 'milestone', item: m });
  }
  if (!inserted) out.push({ type: 'today' });
  return out;
});

const load = async () => {
  loading.value = true; error.value = '';
  try {
    roadmap.value = (await api.getRoadmap(props.projectId, props.containerId)).data;
    milestones.value = roadmap.value.milestones || [];
  } catch (e) { error.value = t('projects.roadmap.loadFailed'); }
  finally { loading.value = false; }
};

const openCreate = () => { modal.value = { title: '', description: '', due_date: todayStr(), status: 'open', place_id: null, subject_id: null }; };
const openEdit = (m) => { modal.value = { id: m.id, title: m.title, description: m.description || '', due_date: m.due_date, status: m.status, place_id: m.place_id ?? null, subject_id: m.subject_id ?? null }; };

const submit = async () => {
  const m = modal.value;
  if (!m.title.trim() || !m.due_date) { toast.error(t('projects.roadmap.missingFields')); return; }
  saving.value = true;
  const payload = {
    title: m.title.trim(), description: m.description.trim() || null, due_date: m.due_date,
    status: m.status, place_id: m.place_id, subject_id: m.subject_id,
  };
  try {
    if (m.id) await api.updateMilestone(props.projectId, props.containerId, m.id, payload);
    else await api.createMilestone(props.projectId, props.containerId, payload);
    modal.value = null;
    await load();
  } catch (e) {
    toast.error(e.response?.data?.message || t('projects.roadmap.saveFailed'));
  } finally { saving.value = false; }
};

const toggleStatus = async (m) => {
  const next = m.status === 'reached' ? 'open' : 'reached';
  try {
    await api.updateMilestone(props.projectId, props.containerId, m.id, {
      title: m.title, description: m.description, due_date: m.due_date, status: next,
    });
    m.status = next;
  } catch (e) { toast.error(t('projects.roadmap.saveFailed')); }
};

const remove = async (m) => {
  if (!(await confirmDelete(t('projects.roadmap.deleteMilestoneConfirm', { title: m.title })))) return;
  try {
    await api.deleteMilestone(props.projectId, props.containerId, m.id);
    await load();
  } catch (e) {
    toast.error(e.response?.status === 403 ? t('projects.roadmap.deleteForbidden') : t('projects.roadmap.deleteFailed'));
  }
};

// Live: fremde Änderungen an dieser Roadmap nachziehen. Der project.{id}-Kanal
// gehört ProjectDetail – hier nur Listener.
const debouncedReload = debounce(() => load(), 400);
const onLive = (payload) => { if (payload?.kind === 'milestone') debouncedReload(); };
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
      <Flag class="w-5 h-5 text-marke" />
      <h3 class="font-extrabold text-lg text-schrift truncate">{{ roadmap?.name || t('projects.roadmap.title') }}</h3>
      <BaseButton @click="openCreate" groesse="kopf" class="ml-auto"><Plus class="w-4 h-4" /> {{ t('projects.roadmap.add') }}</BaseButton>
      <button @click="load" class="p-2 text-slate-400 hover:bg-auflage rounded-xl" :title="t('projects.roadmap.title')"><RefreshCcw class="w-4 h-4" :class="loading ? 'animate-spin text-marke' : ''" /></button>
    </div>

    <div v-if="loading" class="py-10 text-center text-slate-400"><Loader2 class="w-6 h-6 animate-spin mx-auto" /></div>
    <p v-else-if="error" class="text-sm text-rose-500 font-medium">{{ error }}</p>
    <p v-else-if="milestones.length === 0" class="py-8 text-center text-sm text-slate-400 dark:text-slate-500 font-medium">{{ t('projects.roadmap.empty') }}</p>

    <ol v-else class="relative border-l-2 border-linie ml-3 space-y-6">
      <li v-for="(row, i) in timeline" :key="i" class="relative pl-6">
        <template v-if="row.type === 'today'">
          <span class="absolute -left-[9px] top-1.5 w-4 h-4 rounded-full bg-rose-500 ring-4 ring-rose-100 dark:ring-rose-900/40"></span>
          <span class="text-xs font-extrabold uppercase tracking-wide text-rose-500">{{ t('projects.roadmap.today') }}</span>
        </template>
        <template v-else>
          <button @click="toggleStatus(row.item)"
            class="absolute -left-[11px] top-0.5 w-5 h-5 rounded-full flex items-center justify-center transition-colors"
            :class="row.item.status === 'reached'
              ? 'bg-emerald-500 text-white'
              : isOverdue(row.item) ? 'bg-flaeche text-rose-500 ring-2 ring-rose-400' : 'bg-flaeche text-marke ring-2 ring-marke'"
            :title="row.item.status === 'reached' ? t('projects.roadmap.markOpen') : t('projects.roadmap.markReached')">
            <CheckCircle2 v-if="row.item.status === 'reached'" class="w-4 h-4" />
            <Circle v-else class="w-3 h-3" />
          </button>
          <!-- Polster immer gesetzt (nicht nur beim Hervorheben), sonst
               ruckte die Zeitachse beim Ankommen um zwei Pixel. -->
          <div
            :ref="(el) => merkeEintrag(row.item.id, el)"
            class="flex items-start gap-2 rounded-xl px-2 py-1 -mx-2 transition-colors"
            :class="row.item.id === highlighted ? 'bg-marke-leise ring-1 ring-marke' : ''"
          >
            <div class="min-w-0 flex-1">
              <div class="flex items-center gap-2 flex-wrap">
                <span class="font-bold text-schrift" :class="row.item.status === 'reached' ? 'line-through decoration-slate-300' : ''">{{ row.item.title }}</span>
                <span v-if="row.item.status === 'reached'" class="text-xs font-bold px-2 py-0.5 rounded-full bg-emerald-50 dark:bg-emerald-900/30 text-emerald-600 dark:text-emerald-400">{{ t('projects.roadmap.statusReached') }}</span>
                <span v-else-if="isOverdue(row.item)" class="text-xs font-bold px-2 py-0.5 rounded-full bg-rose-50 dark:bg-rose-900/30 text-rose-600 dark:text-rose-400">{{ t('projects.roadmap.overdue') }}</span>
                <span v-if="row.item.subject" class="text-xs font-bold px-2 py-0.5 rounded-full bg-marke-leise text-marke">{{ row.item.subject.short || row.item.subject.name }}</span>
              </div>
              <p class="text-xs font-semibold text-leise mt-0.5">
                {{ formatDateLong(row.item.due_date) }}
                <span v-if="row.item.place" class="inline-flex items-center gap-1 ml-2 text-slate-400">
                  <MapPin class="w-3 h-3 shrink-0" />{{ row.item.place.name }}
                </span>
              </p>
              <div v-if="row.item.description" class="text-sm text-fliess mt-1"><ClampText :text="row.item.description" :lines="3" /></div>
            </div>
            <button @click="openEdit(row.item)" class="p-1.5 text-slate-400 hover:text-marke hover:bg-marke-leise rounded-xl" :title="t('projects.roadmap.editTitle')"><Pencil class="w-4 h-4" /></button>
            <button v-if="canDelete(row.item)" @click="remove(row.item)" class="p-1.5 text-slate-400 hover:text-rose-500 hover:bg-rose-50 dark:hover:bg-rose-900/20 rounded-xl" :title="t('projects.roadmap.deleteMilestoneTitle')"><Trash2 class="w-4 h-4" /></button>
          </div>
        </template>
      </li>
    </ol>

    <!-- Anlegen/Bearbeiten -->
    <div v-if="modal" class="fixed inset-0 bg-slate-900/60 backdrop-blur-sm flex items-center justify-center p-4 z-[100]" @click.self="modal = null">
      <div class="karte w-full max-w-lg shadow-2xl max-h-[90vh] overflow-y-auto">
        <div class="flex items-center justify-between px-6 py-4 border-b border-linie">
          <h3 class="font-extrabold text-schrift flex items-center gap-2"><Flag class="w-5 h-5 text-marke" /> {{ modal.id ? t('projects.roadmap.editTitle') : t('projects.roadmap.newTitle') }}</h3>
          <button @click="modal = null" class="p-1.5 text-slate-400 hover:bg-auflage rounded-xl"><X class="w-5 h-5" /></button>
        </div>
        <div class="p-6 space-y-4">
          <div>
            <label class="text-xs font-bold text-leise uppercase">{{ t('projects.roadmap.titleLabel') }}</label>
            <input v-model="modal.title" type="text" maxlength="255" :placeholder="t('projects.roadmap.titlePlaceholder')" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift" />
          </div>
          <div>
            <label class="text-xs font-bold text-leise uppercase">{{ t('projects.roadmap.dateLabel') }}</label>
            <input v-model="modal.due_date" type="date" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift" />
          </div>
          <div>
            <label class="text-xs font-bold text-leise uppercase">{{ t('projects.roadmap.descriptionLabel') }}</label>
            <textarea v-model="modal.description" rows="3" :placeholder="t('projects.roadmap.descriptionPlaceholder')" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift resize-none"></textarea>
          </div>
          <PlacePicker v-model="modal.place_id" :project-id="projectId" />
          <!-- Verbindet die Klassenarbeit mit der Stunde, in der sie
               geschrieben wird – die Tagesansicht liest genau das. -->
          <SubjectPicker v-model="modal.subject_id" :project-id="projectId" />
          <label class="flex items-center gap-2 text-sm font-medium text-fliess">
            <input v-model="modal.status" type="checkbox" true-value="reached" false-value="open" class="rounded border-slate-300 text-emerald-600 focus:ring-emerald-500" />
            {{ t('projects.roadmap.statusReached') }}
          </label>
        </div>
        <div class="flex justify-end gap-2 px-6 py-4 border-t border-linie">
          <button @click="modal = null" class="px-4 py-2 text-fliess hover:bg-auflage rounded-xl text-sm font-bold">{{ t('common.cancel') }}</button>
          <BaseButton @click="submit" :loading="saving">{{ t('common.save') }}</BaseButton>
        </div>
      </div>
    </div>
  </div>
</template>
