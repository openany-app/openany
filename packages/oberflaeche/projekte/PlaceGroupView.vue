<script setup>
// Detailansicht EINER Orte-Gruppe (Container, angelegt über den +Neu-Picker
// in der Planungsliste – wie ein Board). Lädt sich selbst über containerId und
// zeigt die Kartenpunkte als Raster mit je einem kleinen, nicht bedienbaren
// OSM-Ausschnitt. Anlegen über einen Dialog mit Pin-Setzen. Jedes Mitglied
// darf anlegen/bearbeiten; löschen nur Ersteller/Owner. Live via
// ProjectPlanningChanged. Kacheln kommen von OpenStreetMap (Hinweis unten).
import { ref, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from './umgebung';
import ClampText from '@oberflaeche/base/ClampText.vue';
import LeafletMap from './LeafletMap.vue';
import { Plus, RefreshCcw, Trash2, ArrowLeft, Loader2, MapPin, Pencil, X, Info } from 'lucide-vue-next';
import { useToast } from '@oberflaeche/composables/useToast';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import { initEcho, onEchoReconnect } from './umgebung';
import { debounce } from '@oberflaeche/shared/debounce';
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
  // Ort aus einem [[Verweis]] im Chat: hervorheben und ins Bild rollen.
  highlightId: { type: Number, default: null },
});
const emit = defineEmits(['close']);

const loading = ref(true);
const error = ref('');
const group = ref(null);
const places = ref([]);
const modal = ref(null); // { id?, name, note, coords: { lat, lng } | null }
const saving = ref(false);

const { highlighted, merkeEintrag } = useHighlight(
  () => props.highlightId,
  () => places.value.length,
);

const canDelete = (p) => p.created_by === props.meId || props.isOwner;

// Externer Link auf OpenStreetMap: mlat/mlon zeigt dort einen Marker
// am Ort, der Hash steuert Ausschnitt und Zoom.
const osmUrl = (p) => `https://www.openstreetmap.org/?mlat=${p.lat}&mlon=${p.lng}#map=16/${p.lat}/${p.lng}`;

const load = async () => {
  loading.value = true; error.value = '';
  try {
    group.value = (await api.getPlaceGroup(props.projectId, props.containerId)).data;
    places.value = group.value.places || [];
  } catch (e) { error.value = t('projects.places.loadFailed'); }
  finally { loading.value = false; }
};

const openCreate = () => { modal.value = { name: '', note: '', coords: null }; };
const openEdit = (p) => { modal.value = { id: p.id, name: p.name || '', note: p.note || '', coords: { lat: p.lat, lng: p.lng } }; };

const submit = async () => {
  const m = modal.value;
  if (!m.coords || !Number.isFinite(m.coords.lat)) { toast.error(t('projects.places.noPin')); return; }
  saving.value = true;
  const payload = { name: m.name.trim() || null, note: m.note.trim() || null, lat: m.coords.lat, lng: m.coords.lng };
  try {
    if (m.id) await api.updatePlace(props.projectId, props.containerId, m.id, payload);
    else await api.createPlace(props.projectId, props.containerId, payload);
    modal.value = null;
    await load();
  } catch (e) {
    toast.error(e.response?.data?.message || t('projects.places.saveFailed'));
  } finally { saving.value = false; }
};

const remove = async (p) => {
  if (!(await confirmDelete(t('projects.places.deletePlaceConfirm', { name: p.name || t('projects.places.unnamed') })))) return;
  try {
    await api.deletePlace(props.projectId, props.containerId, p.id);
    await load();
  } catch (e) {
    toast.error(e.response?.status === 403 ? t('projects.places.deleteForbidden') : t('projects.places.deleteFailed'));
  }
};

const debouncedReload = debounce(() => load(), 400);
const onLive = (payload) => { if (payload?.kind === 'place') debouncedReload(); };
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
      <MapPin class="w-5 h-5 text-marke" />
      <h3 class="font-extrabold text-lg text-schrift truncate">{{ group?.name || t('projects.places.title') }}</h3>
      <BaseButton @click="openCreate" groesse="kopf" class="ml-auto"><Plus class="w-4 h-4" /> {{ t('projects.places.add') }}</BaseButton>
      <button @click="load" class="p-2 text-slate-400 hover:bg-auflage rounded-xl" :title="t('projects.places.title')"><RefreshCcw class="w-4 h-4" :class="loading ? 'animate-spin text-marke' : ''" /></button>
    </div>

    <div v-if="loading" class="py-10 text-center text-slate-400"><Loader2 class="w-6 h-6 animate-spin mx-auto" /></div>
    <p v-else-if="error" class="text-sm text-rose-500 font-medium">{{ error }}</p>
    <p v-else-if="places.length === 0" class="py-8 text-center text-sm text-slate-400 dark:text-slate-500 font-medium">{{ t('projects.places.empty') }}</p>

    <div v-else class="grid sm:grid-cols-2 lg:grid-cols-3 gap-4">
      <div
        v-for="p in places" :key="p.id"
        :ref="(el) => merkeEintrag(p.id, el)"
        class="rounded-xl border overflow-hidden bg-vertieft group transition-colors"
        :class="p.id === highlighted
          ? 'border-marke ring-1 ring-marke'
          : 'border-linie'"
      >
        <div class="h-40 relative">
          <LeafletMap :model-value="{ lat: p.lat, lng: p.lng }" :zoom="15" />
        </div>
        <div class="p-3 flex items-start gap-2">
          <div class="min-w-0 flex-1">
            <a :href="osmUrl(p)" target="_blank" rel="noopener" :title="t('projects.places.openOsm')"
              class="flex items-start gap-2 group/link">
              <MapPin class="w-4 h-4 text-marke shrink-0 mt-0.5 group-hover/link:text-marke transition-colors" />
              <p class="font-bold text-schrift truncate group-hover/link:text-marke group-hover/link:underline transition-colors">{{ p.name || t('projects.places.unnamed') }}</p>
            </a>
            <div v-if="p.note" class="text-xs text-leise mt-0.5 ml-6"><ClampText :text="p.note" :lines="2" /></div>
          </div>
          <button @click="openEdit(p)" class="p-1.5 text-slate-400 hover:text-marke hover:bg-marke-leise rounded-xl" :title="t('projects.places.editTitle')"><Pencil class="w-4 h-4" /></button>
          <button v-if="canDelete(p)" @click="remove(p)" class="p-1.5 text-slate-400 hover:text-rose-500 hover:bg-rose-50 dark:hover:bg-rose-900/20 rounded-xl" :title="t('projects.places.deletePlaceTitle')"><Trash2 class="w-4 h-4" /></button>
        </div>
      </div>
    </div>

    <p class="flex items-center gap-1.5 text-xs text-slate-400 dark:text-slate-500"><Info class="w-3.5 h-3.5 shrink-0" /> {{ t('projects.places.externalHint') }}</p>

    <!-- Anlegen/Bearbeiten -->
    <div v-if="modal" class="fixed inset-0 bg-slate-900/60 backdrop-blur-sm flex items-center justify-center p-4 z-[100]" @click.self="modal = null">
      <div class="karte w-full max-w-lg shadow-2xl max-h-[90vh] overflow-y-auto">
        <div class="flex items-center justify-between px-6 py-4 border-b border-linie">
          <h3 class="font-extrabold text-schrift flex items-center gap-2"><MapPin class="w-5 h-5 text-marke" /> {{ modal.id ? t('projects.places.editTitle') : t('projects.places.newTitle') }}</h3>
          <button @click="modal = null" class="p-1.5 text-slate-400 hover:bg-auflage rounded-xl"><X class="w-5 h-5" /></button>
        </div>
        <div class="p-6 space-y-4">
          <p class="text-xs font-medium text-leise">{{ t('projects.places.pinHint') }}</p>
          <div class="h-64 rounded-xl overflow-hidden border border-linie">
            <LeafletMap v-model="modal.coords" :interactive="true" :zoom="15" />
          </div>
          <div>
            <label class="text-xs font-bold text-leise uppercase">{{ t('projects.places.nameLabel') }}</label>
            <input v-model="modal.name" type="text" maxlength="255" :placeholder="t('projects.places.namePlaceholder')" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift" />
          </div>
          <div>
            <label class="text-xs font-bold text-leise uppercase">{{ t('projects.places.noteLabel') }}</label>
            <textarea v-model="modal.note" rows="2" :placeholder="t('projects.places.notePlaceholder')" class="mt-1 w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift resize-none"></textarea>
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
