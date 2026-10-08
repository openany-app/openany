<script setup>
// Die Fächer eines Projekts pflegen – wie die Spalten eines Kanban-Boards:
// kein Papierkorb, jedes Mitglied darf anlegen, ändern, löschen. Öffnet sich
// aus WeekPlanView.vue als Dialog; Fächer selbst haben keine eigene Zeile in
// der Planungsliste.
import { ref, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { api, kann } from './umgebung';
import PlacePicker from './PlacePicker.vue';
import {
  Plus, Trash2, X, Pencil, Loader, Loader2, BookOpen, NotebookText, FolderPlus, Users,
} from 'lucide-vue-next';
import { useToast } from '@oberflaeche/composables/useToast';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import { initEcho, onEchoReconnect } from './umgebung';
import { debounce } from '@oberflaeche/shared/debounce';
import BaseButton from '@oberflaeche/base/BaseButton.vue';

const { t } = useI18n();
const toast = useToast();
const { confirmDelete } = useConfirm();
const props = defineProps({
  projectId: { type: [Number, String], required: true },
});
const emit = defineEmits(['close']);

const FARBEN = ['#6366f1', '#ec4899', '#f59e0b', '#10b981', '#0ea5e9', '#8b5cf6'];

const loading = ref(true);
const error = ref('');
const faecher = ref([]);
const modal = ref(null);
const saving = ref(false);
const anlegendHeft = ref(null); // Id des Fachs, dessen Heft gerade entsteht

const load = async () => {
  loading.value = true; error.value = '';
  try { faecher.value = (await api.getSubjects(props.projectId)).data; }
  catch (e) { error.value = t('projects.subjects.loadFailed'); }
  finally { loading.value = false; }
};

const openCreate = () => {
  modal.value = { name: '', short: '', teacher: '', color: null, place_id: null };
};
const openEdit = (f) => {
  modal.value = {
    id: f.id, name: f.name, short: f.short || '', teacher: f.teacher || '',
    color: f.color || null, place_id: f.place_id ?? null,
  };
};

const submit = async () => {
  const m = modal.value;
  if (!m.name.trim()) { toast.error(t('projects.subjects.missingFields')); return; }
  saving.value = true;
  const payload = {
    name: m.name.trim(), short: m.short.trim() || null,
    teacher: m.teacher.trim() || null, color: m.color, place_id: m.place_id,
  };
  try {
    if (m.id) await api.updateSubject(props.projectId, m.id, payload);
    else await api.createSubject(props.projectId, payload);
    modal.value = null;
    await load();
  } catch (e) {
    toast.error(e.response?.data?.message || t('projects.subjects.saveFailed'));
  } finally { saving.value = false; }
};

const remove = async (f) => {
  if (!(await confirmDelete(t('projects.subjects.deleteConfirm', { name: f.name })))) return;
  try {
    await api.deleteSubject(props.projectId, f.id);
    await load();
  } catch (e) { toast.error(t('projects.subjects.deleteFailed')); }
};

// Mappe anlegen UND freigeben, in einem Schritt – der Picker bietet das nur
// an, wenn noch keine Mappe da ist. Zwei getrennte Wege über das
// Notizen-Modul wären Verwaltungsarbeit, und dann bliebe das Feld leer.
const heftAnlegen = async (f) => {
  anlegendHeft.value = f.id;
  try {
    await api.createSubjectNoteFolder(props.projectId, f.id);
    toast.success(t('projects.subjects.heftCreated'));
    await load();
  } catch (e) {
    toast.error(e.response?.data?.message || t('projects.subjects.heftCreateFailed'));
  } finally {
    anlegendHeft.value = null;
  }
};

const debouncedReload = debounce(() => load(), 400);
const onLive = (payload) => { if (payload?.kind === 'subject') debouncedReload(); };
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
  <div class="fixed inset-0 bg-slate-900/60 backdrop-blur-sm flex items-center justify-center p-4 z-[100]" @click.self="emit('close')">
    <div class="karte w-full max-w-lg shadow-2xl max-h-[90vh] overflow-y-auto">
      <div class="flex items-center justify-between px-6 py-4 border-b border-linie">
        <h3 class="font-extrabold text-schrift flex items-center gap-2"><BookOpen class="w-5 h-5 text-marke" /> {{ t('projects.subjects.title') }}</h3>
        <button @click="emit('close')" class="p-1.5 text-slate-400 hover:bg-auflage rounded-xl"><X class="w-5 h-5" /></button>
      </div>

      <div class="p-6 space-y-4">
        <p class="text-xs text-leise">{{ t('projects.subjects.hint') }}</p>

        <div v-if="loading" class="py-10 text-center text-slate-400"><Loader2 class="w-6 h-6 animate-spin mx-auto" /></div>
        <p v-else-if="error" class="text-sm text-rose-500 font-medium">{{ error }}</p>
        <p v-else-if="faecher.length === 0" class="py-4 text-center text-sm text-slate-400 dark:text-slate-500 font-medium">{{ t('projects.subjects.empty') }}</p>

        <ul v-else class="space-y-2">
          <li v-for="f in faecher" :key="f.id"
            class="flex items-center gap-3 px-3 py-2.5 rounded-xl bg-vertieft border border-linie"
            :style="f.color ? { borderLeftWidth: '4px', borderLeftColor: f.color } : null">
            <div class="min-w-0 flex-1">
              <div class="flex items-center gap-2 flex-wrap">
                <span class="font-bold text-schrift truncate">{{ f.name }}</span>
                <span v-if="f.short" class="text-xs font-bold px-1.5 py-0.5 rounded bg-auflage text-slate-500">{{ f.short }}</span>
              </div>
              <p class="text-xs font-semibold text-leise mt-0.5 flex items-center gap-2 flex-wrap">
                <span v-if="f.teacher" class="inline-flex items-center gap-1"><Users class="w-3 h-3" />{{ f.teacher }}</span>
                <span v-if="f.place" class="text-slate-400">{{ f.place.name }}</span>
              </p>
              <!-- Wem die Mappe gehört, muss zu sehen sein – sonst hält man
                   sie für Projektbesitz und wundert sich, wenn sie verschwindet. -->
              <p v-if="f.note_folder" class="text-xs font-semibold text-marke mt-1 flex items-center gap-1">
                <NotebookText class="w-3 h-3" /> {{ t('projects.subjects.folderOwnedBy', { name: f.note_folder.owner_name }) }}
              </p>
              <button v-else-if="kann('heft')" @click="heftAnlegen(f)" :disabled="anlegendHeft === f.id"
                class="mt-1 flex items-center gap-1 text-xs font-bold text-marke hover:underline disabled:opacity-50">
                <Loader v-if="anlegendHeft === f.id" class="w-3 h-3 animate-spin" /><FolderPlus v-else class="w-3 h-3" />
                {{ t('projects.subjects.createFolder') }}
              </button>
            </div>
            <button @click="openEdit(f)" class="p-1.5 text-slate-400 hover:text-marke hover:bg-marke-leise rounded-xl" :title="t('projects.subjects.editTitle')"><Pencil class="w-4 h-4" /></button>
            <button @click="remove(f)" class="p-1.5 text-slate-400 hover:text-rose-500 hover:bg-rose-50 dark:hover:bg-rose-900/20 rounded-xl" :title="t('projects.subjects.deleteTitle')"><Trash2 class="w-4 h-4" /></button>
          </li>
        </ul>

        <button @click="openCreate" class="flex items-center gap-1.5 text-sm font-bold text-marke hover:underline"><Plus class="w-4 h-4" /> {{ t('projects.subjects.add') }}</button>
      </div>
    </div>

    <!-- Anlegen/Bearbeiten -->
    <div v-if="modal" class="fixed inset-0 bg-slate-900/60 backdrop-blur-sm flex items-center justify-center p-4 z-[110]" @click.self="modal = null">
      <div class="karte w-full max-w-lg shadow-2xl max-h-[90vh] overflow-y-auto">
        <div class="flex items-center justify-between px-6 py-4 border-b border-linie">
          <h3 class="font-extrabold text-schrift flex items-center gap-2"><BookOpen class="w-5 h-5 text-marke" /> {{ modal.id ? t('projects.subjects.editTitle') : t('projects.subjects.newTitle') }}</h3>
          <button @click="modal = null" class="p-1.5 text-slate-400 hover:bg-auflage rounded-xl"><X class="w-5 h-5" /></button>
        </div>
        <div class="p-6 space-y-4">
          <div class="grid grid-cols-3 gap-3">
            <div class="col-span-2">
              <label class="text-xs font-bold text-leise uppercase">{{ t('projects.subjects.nameLabel') }}</label>
              <input v-model="modal.name" type="text" maxlength="255" :placeholder="t('projects.subjects.namePlaceholder')" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift" />
            </div>
            <div>
              <label class="text-xs font-bold text-leise uppercase">{{ t('projects.subjects.shortLabel') }}</label>
              <input v-model="modal.short" type="text" maxlength="16" :placeholder="t('projects.subjects.shortPlaceholder')" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift" />
            </div>
          </div>
          <div>
            <label class="text-xs font-bold text-leise uppercase">{{ t('projects.subjects.teacherLabel') }}</label>
            <input v-model="modal.teacher" type="text" maxlength="255" :placeholder="t('projects.subjects.teacherPlaceholder')" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift" />
          </div>
          <PlacePicker v-model="modal.place_id" :project-id="projectId" />
          <div>
            <label class="text-xs font-bold text-leise uppercase">{{ t('projects.weekPlan.colorLabel') }}</label>
            <div class="mt-1 flex items-center gap-2 flex-wrap">
              <button @click="modal.color = null" class="w-7 h-7 rounded-full border-2 flex items-center justify-center"
                :class="modal.color ? 'border-linie text-slate-400' : 'border-marke text-marke'"
                :title="t('projects.weekPlan.noColor')"><X class="w-3.5 h-3.5" /></button>
              <button v-for="c in FARBEN" :key="c" @click="modal.color = c"
                class="w-7 h-7 rounded-full border-2" :style="{ backgroundColor: c }"
                :class="modal.color === c ? 'border-slate-900 dark:border-white' : 'border-transparent'"></button>
            </div>
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
