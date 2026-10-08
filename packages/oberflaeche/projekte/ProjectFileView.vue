<script setup>
// Datei-/Akten-Ansicht einer in ein Projekt freigegebenen FileNode. Ansehen
// + Herunterladen für alle; bei Freigabe-Stufe „bearbeiten" zusätzlich
// Hochladen, Unterordner anlegen und Löschen (der Server schreibt alles dem
// Ordner-Eigentümer zu). Nutzt dieselben Bausteine wie der persönliche
// Speicher (ExplorerTable, ContainerBreadcrumbs), ohne die volle
// FileExplorer-Komponente zu berühren.
import { ref, computed, watch, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from './umgebung';
import ContainerBreadcrumbs from '@oberflaeche/container/ContainerBreadcrumbs.vue';
import ExplorerTable from '@oberflaeche/container/ExplorerTable.vue';
import ActionButtons from '@oberflaeche/container/ActionButtons.vue';
import NameModal from '@oberflaeche/container/NameModal.vue';
import {
  Folder, Download, Archive,
  Plus, Upload, Trash2, Sparkles,
} from 'lucide-vue-next';
import { triggerDownload } from '@oberflaeche/shared/download';
import { fileIcon, fileIconColor } from '@oberflaeche/shared/fileIcons';
import { useToast } from '@oberflaeche/composables/useToast';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import { formatDateShort } from '@oberflaeche/shared/date';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import BaseModal from '@oberflaeche/base/BaseModal.vue';
import { KiFrage } from './umgebung';
import { textAuslesen, istTextAuszugFehler } from './umgebung';

const props = defineProps({
  projectId: { type: [Number, String], required: true },
  rootId: { type: [Number, String], required: true },   // Freigabe-Wurzel
  zone: { type: String, default: 'files' },             // 'files' | 'documents'
  // Sprungziel aus dem Chat: Ordner, dessen Inhalt gleich zu sehen sein soll,
  // und die Zeile, um die es dabei geht. Beides wirkt nur beim Aufbau.
  startFolderId: { type: [Number, String], default: null },
  highlightId: { type: [Number, String], default: null },
});
const emit = defineEmits(['back']);

const { t } = useI18n();
const toast = useToast();
const { confirmDialog } = useConfirm();
const isDocuments = computed(() => props.zone === 'documents');
const L = (key, params) => t(`${isDocuments.value ? 'documents' : 'files'}.${key}`, params ?? {});

const currentId = ref(props.startFolderId ?? props.rootId);

// Hervorhebung der angesprungenen Datei. Sie erlischt, sobald man den Ordner
// wechselt: Ab da sucht man etwas anderes, und ein stehengebliebener Rahmen
// wäre nur noch ein Rätsel.
const highlighted = ref(props.highlightId);
const files = ref([]);
const breadcrumbs = ref([]);
const access = ref('read');
const isLoading = ref(false);
const downloadingId = ref(null);
const fileInput = ref(null);
const isUploading = ref(false);
const isFolderModalOpen = ref(false);
const folderError = ref('');

const canEdit = computed(() => access.value === 'edit' || access.value === 'owner');

// KI des Projekts – nur in Akten, und nur für Formate, die der Server als
// lesbar meldet (`ki_format`). Den Text liest der Browser beim Absenden aus.
const kiAnbindung = ref(null);
const kiDokument = ref(null);
const ladeKi = async () => {
  if (!isDocuments.value) return;
  try {
    kiAnbindung.value = (await api.getProjectAi(props.projectId)).data.anbindung;
  } catch {
    kiAnbindung.value = null;
  }
};
const kiHinweis = computed(() => kiAnbindung.value
  ? t('ai.destinationDocument', { anbieter: kiAnbindung.value.provider_name, modell: kiAnbindung.value.model })
  : '');
const KI_MELDUNGEN = { keinText: 'ai.noText', zuLang: 'ai.tooLong', unlesbar: 'ai.unreadable' };
const kiSenden = async (auftrag) => {
  const datei = kiDokument.value;
  let text;
  try {
    const { data } = await api.getProjectFileBytes(props.projectId, datei.id);
    text = await textAuslesen(datei.ki_format, data);
  } catch (e) {
    // Eine Meldung, die KiFrage so anzeigt – nichts ist hinausgegangen.
    const fehler = new Error('auslesen');
    fehler.meldung = t(istTextAuszugFehler(e) ? KI_MELDUNGEN[e.grund] : 'ai.unreadable');
    throw fehler;
  }
  return (await api.askProjectAiDocument(props.projectId, datei.id, auftrag, text)).data.anfrage;
};
const kiAbholen = async (anfrage) => (await api.getProjectAiRequest(props.projectId, anfrage)).data;

const load = async () => {
  isLoading.value = true;
  try {
    const res = await api.getProjectFiles(props.projectId, currentId.value);
    files.value = res.data.files;
    breadcrumbs.value = res.data.breadcrumbs;
    access.value = res.data.folder?.access ?? 'read';
  } catch (e) {
    toast.error(t('shares.projectShares.loadFailed'));
  } finally {
    isLoading.value = false;
  }
};
watch(currentId, () => { highlighted.value = null; load(); });
onMounted(() => { load(); ladeKi(); });

// Breadcrumbs kommen vom Backend bereits an der Freigabe-Wurzel gekappt.
const crumbs = computed(() => breadcrumbs.value.map((b) => ({ id: b.id, name: b.name })));
const navigate = (id) => { if (id != null) currentId.value = id; };
const goBack = () => {
  if (breadcrumbs.value.length > 1) currentId.value = breadcrumbs.value[breadcrumbs.value.length - 2].id;
  else emit('back');
};

const open = (item) => {
  if (item.type === 'folder') currentId.value = item.id;
  else download(item);
};
// Direkter Browser-Download (streamt auf die Platte) – für große Dateien nötig.
const download = (item) => {
  triggerDownload(`/api/projects/${props.projectId}/files/${item.id}/download`, item.name);
};

// --- Bearbeiten (nur bei edit-Stufe) ---
const submitFolder = async (rawName) => {
  const name = rawName.trim().replace(/[/\\?%*:|"<>]/g, '');
  if (!name) { folderError.value = L('invalidFolderName'); return; }
  try {
    await api.createProjectFileFolder(props.projectId, currentId.value, name);
    isFolderModalOpen.value = false;
    load();
  } catch (err) {
    folderError.value = err.response?.status === 422 ? L('folderAlreadyExists') : L('folderCreateFailed');
  }
};
const handleUpload = async (event) => {
  const list = event.target.files;
  if (!list || !list.length) return;
  isUploading.value = true;
  try {
    for (let i = 0; i < list.length; i++) {
      await api.uploadProjectFile(props.projectId, currentId.value, list[i]);
    }
    load();
  } catch (err) {
    toast.error(err.response?.data?.message || L('uploadFailed'));
  } finally {
    isUploading.value = false;
    if (fileInput.value) fileInput.value.value = null;
  }
};
const handleDelete = async (item) => {
  const ok = await confirmDialog(L('confirmMoveToTrash', { name: item.name }), { title: L('moveToTrashTitle'), confirmLabel: L('moveAction') });
  if (!ok) return;
  try {
    await api.deleteProjectFileNode(props.projectId, item.id);
    load();
  } catch (err) {
    toast.error(err.response?.data?.message || L('deleteFileFailed'));
  }
};

const itemActions = (item) => {
  const actions = [];
  if (item.type !== 'folder') {
    actions.push({ key: 'download', icon: Download, title: L('download'), onClick: () => download(item), loading: downloadingId.value === item.id });
  }
  if (kiAnbindung.value && item.ki_format) {
    actions.push({ key: 'ai', icon: Sparkles, title: t('ai.askAction'), onClick: () => { kiDokument.value = item; } });
  }
  if (canEdit.value) {
    actions.push({ key: 'trash', icon: Trash2, title: L('delete'), onClick: () => handleDelete(item), danger: true });
  }
  return actions;
};

// Symbol und Farbe je Dateityp – gemeinsam mit dem persönlichen Speicher.
const getFileIcon = (type) => fileIcon(type, isDocuments.value);
const getIconColorClass = fileIconColor;
</script>

<template>
  <div class="space-y-1 sm:space-y-6">
    <!-- Bearbeiten-Leiste (nur bei edit-Stufe) -->
    <div v-if="canEdit" class="flex items-center justify-end gap-2">
      <button @click="folderError = ''; isFolderModalOpen = true"
        class="flex items-center justify-center h-9 gap-1.5 px-2.5 sm:px-4 py-2 bg-slate-100 hover:bg-slate-200 dark:bg-slate-800 dark:hover:bg-slate-700 text-fliess text-sm font-bold rounded-xl transition-all cursor-pointer">
        <Plus class="w-4 h-4" /> <span>{{ L('newFolder') }}</span>
      </button>
      <BaseButton @click="$refs.fileInput.click()" :disabled="isUploading"
        groesse="kopf">
        <Upload class="w-4 h-4" /> <span>{{ L('upload') }}</span>
      </BaseButton>
      <input type="file" ref="fileInput" class="hidden" multiple @change="handleUpload" />
    </div>

    <ContainerBreadcrumbs
      :crumbs="crumbs"
      :show-back="true"
      :attached="true"
      @back="goBack"
      @navigate="navigate"
    />

    <div class="-mt-1 sm:-mt-6">
      <div v-if="isLoading" class="flex justify-center py-20">
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-marke"></div>
      </div>
      <ExplorerTable
        v-else
        :columns="{ name: L('columnName'), size: L('columnSize'), meta: L('columnChanged'), actions: L('columnActions') }"
        :items="files"
        :mobile-actions="true"
        :attached-top="true"
        :highlight-id="highlighted"
        @open="open"
      >
        <template #icon="{ item }">
          <component :is="getFileIcon(item.type)" class="w-6 h-6 shrink-0" :class="getIconColorClass(item.type)" />
        </template>
        <template #size="{ item }">{{ item.size || '--' }}</template>
        <template #meta="{ item }">
          {{ formatDateShort(item.updatedAt) }}
        </template>
        <template #actions="{ item }">
          <ActionButtons :actions="itemActions(item)" />
        </template>
        <template #empty>
          <div class="w-16 h-16 rounded-full bg-slate-50 dark:bg-slate-800 text-slate-400 flex items-center justify-center mx-auto">
            <component :is="isDocuments ? Archive : Folder" class="w-8 h-8" />
          </div>
          <h4 class="text-lg font-extrabold text-schrift">{{ L('emptyTitle') }}</h4>
        </template>
      </ExplorerTable>
    </div>

    <BaseModal v-if="kiDokument" :title="`${t('ai.askAction')}: ${kiDokument.name}`" size="lg" persistent @close="kiDokument = null">
      <KiFrage :key="kiDokument.id" :hinweis="kiHinweis" :senden="kiSenden" :abholen="kiAbholen"
        :vorbereitung-text="t('ai.reading')" />
    </BaseModal>

    <NameModal
      v-if="isFolderModalOpen"
      :title="L('createFolderTitle')"
      :label="L('folderName')"
      :placeholder="L('folderNamePlaceholder')"
      :submit-label="L('createFolder')"
      :cancel-label="L('cancel')"
      :error="folderError"
      @submit="submitFolder"
      @close="isFolderModalOpen = false"
    />
  </div>
</template>
