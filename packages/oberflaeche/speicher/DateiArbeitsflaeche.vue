<script setup>
/*
 * Die Arbeitsfläche für Dateien (zone="files") und Dokumente
 * (zone="documents") — dieselbe für Webapp und Programm.
 *
 * Bis zum 15.09.2026 war das `frontend/src/components/files/FileExplorer.vue`
 * und sprach direkt mit dem api-Service. Jetzt bekommt sie eine
 * `dataSource` mit denselben Methodennamen (siehe unten), so wie die
 * Notizen-Arbeitsfläche. Die Webapp reicht ihren api-Service durch, das
 * Programm eine Quelle, die `invoke()` ruft.
 *
 * WAS NUR DIE WEBAPP KANN, KOMMT ALS `erweiterungen`: Fotos beim Hochladen in
 * ein durchsuchbares PDF wandeln, nachträgliche Texterkennung, Freigabe in
 * ein Projekt. Die Arbeitsfläche kennt davon nur die Haken:
 *
 *   hochladenVorbereiten(dateien, kontext) → Dateien, die unverändert
 *                                            hochgeladen werden sollen
 *   nachHochladen(kontext)                 → danach (etwa: Fotos umwandeln)
 *   aktionen(eintrag, kontext)             → zusätzliche Knöpfe je Zeile
 *
 * `kontext` = { zone, ordnerId, neuLaden, zeigeFehler }.
 *
 * dataSource:
 *   getFiles(ordnerId, seite, zone)       → { data: { files, next_page, breadcrumbs } }
 *   createFolder(ordnerId, name, zone)
 *   renameFileNode(id, name)
 *   uploadFile(ordnerId, datei, zone, { fortschritt })
 *   deleteFileOrFolder(id)
 *   getFileTree(zone)                     → { data: { folders } }
 *   moveFileNode(id, ordnerId)
 *   herunterladen(datei)          optional — sonst kein Download-Knopf
 *   ordnerHerunterladen(ordner)   optional — ZIP eines Ordners
 *   oeffnen(datei)                optional — statt des Download-Dialogs
 *   downloadFileContent(id)       → { data: ArrayBuffer } — für den PDF-Betrachter
 *   replaceFileContent(id, datei, { zone, vorfassung })
 *                                 optional — ausgefüllte PDF-Formulare speichern
 *   externOeffnen(datei)          optional — „Mit anderem Programm öffnen"
 *   sucheDateien(q, zone)         optional — Suche nach Namen über alle
 *                                 Ordner → { data: { results } }, je Treffer
 *                                 wie in `files` plus `parent_id` und `pfad`
 *                                 (Namen von oben), bei Inhaltstreffern
 *                                 `ausschnitt`; ohne sie keine Lupe
 *   textOffen(zone, version), textSetzen(id, abdruck, stand, text, version)
 *                                 optional — der Inhaltsleser (inhaltsleser.js)
 *
 * PDFs zeigt die Arbeitsfläche selbst (PdfBetrachter, seit 27.09.2026), vor
 * `oeffnen` und vor dem Download-Dialog: in beiden Anwendungen gleich.
 */
import { ref, computed, watch, nextTick, onMounted, onUnmounted, inject } from 'vue';
import { useI18n } from 'vue-i18n';
import ActionButtons from '../container/ActionButtons.vue';
import ModuleHeader from '../base/ModuleHeader.vue';
import ContainerBreadcrumbs from '../container/ContainerBreadcrumbs.vue';
import ExplorerTable from '../container/ExplorerTable.vue';
import MoveTargetModal from '../container/MoveTargetModal.vue';
import NameModal from '../container/NameModal.vue';
import KameraKnopf from '../base/KameraKnopf.vue';
import PdfBetrachter from './PdfBetrachter.vue';
import { inhaltsleser } from './inhaltsleser';
import { trefferTeile } from '../nachrichten/treffer';
import BaseButton from '../base/BaseButton.vue';
import {
  Folder, Upload, Plus, Trash2, X, Loader2, Download, Archive,
  CheckCircle, AlertCircle, FolderInput, Pencil, CloudOff, Pin, ExternalLink, Search,
} from 'lucide-vue-next';
import { useToast } from '../composables/useToast';
import { useConfirm } from '../composables/useConfirm';
import { folderTreeOptions } from '../shared/folderTree';
import { fileIcon, fileIconColor } from '../shared/fileIcons';
import { formatDateShort } from '../shared/date';

const props = defineProps({
  zone: { type: String, default: 'files' }, // 'files' | 'documents'
  dataSource: { type: Object, required: true },
  erweiterungen: { type: Object, default: () => ({}) },
  hilfeZiel: { type: String, default: '' },
  // Im Programm: „Speichern" statt „Hochladen" — die Datei geht auf dieses
  // Gerät, nicht auf einen Server.
  lokal: { type: Boolean, default: false },
});
const emit = defineEmits(['hilfe']);

const Link = inject('oberflaeche:link', null);
const { t } = useI18n();
const toast = useToast();
const { confirmDialog } = useConfirm();
const quelle = props.dataSource;

const isDocuments = props.zone === 'documents';
// Texte je Zone: documents.* spiegelt die files.*-Schlüssel mit Akten-Wortlaut.
const L = (key, params) => t(`${isDocuments ? 'documents' : 'files'}.${key}`, params ?? {});
const LOKAL = ['upload', 'uploadSuccess', 'uploadFailed', 'emptyHint'];
const LA = (key) => L(props.lokal && LOKAL.includes(key) ? `lokal.${key}` : key);

const currentFolderId = ref(null);
const filesList = ref([]);
const breadcrumbs = ref([]);
const isLoading = ref(false);
const isUploading = ref(false);
const uploadFortschritt = ref(null); // 0..1, wenn die Quelle ihn meldet
const uploadStatus = ref('');
const uploadErrorMsg = ref('');
const isDragOver = ref(false);
const filesNextPage = ref(null);
const loadingMore = ref(false);
const nameModal = ref(null);
const folderError = ref('');
const activeViewerFile = ref(null);
const pdfDatei = ref(null);
const fileInput = ref(null);

const loadFiles = async ({ reset = true } = {}) => {
  const page = reset ? 1 : filesNextPage.value;
  if (page == null) return;
  if (reset) isLoading.value = true;
  try {
    const res = await quelle.getFiles(currentFolderId.value, page, props.zone);
    filesList.value = reset ? res.data.files : [...filesList.value, ...res.data.files];
    filesNextPage.value = res.data.next_page;
    if (reset) breadcrumbs.value = res.data.breadcrumbs;
  } catch (e) {
    console.error(L('loadFilesFailed'), e);
  } finally {
    isLoading.value = false;
  }
};
const loadMoreFiles = async () => {
  loadingMore.value = true;
  try { await loadFiles({ reset: false }); } finally { loadingMore.value = false; }
};

const displayBreadcrumbs = computed(() => [
  { id: null, name: L('home') },
  ...breadcrumbs.value.map((bc) => ({ id: bc.id, name: bc.name })),
]);

const navigateTo = (id) => {
  currentFolderId.value = id;
  loadFiles();
};

const goUp = () => {
  if (!breadcrumbs.value.length) return;
  currentFolderId.value = breadcrumbs.value.length > 1
    ? breadcrumbs.value[breadcrumbs.value.length - 2].id
    : null;
  loadFiles();
};

const openFolderModal = () => {
  folderError.value = '';
  nameModal.value = { mode: 'create', item: null };
};

const openRenameModal = (item) => {
  folderError.value = '';
  nameModal.value = { mode: 'rename', item };
};

/**
 * Die Endung einer Datei („.pdf"), oder '' bei Ordnern und Namen ohne Punkt.
 * Sie steht beim Umbenennen nicht im Feld und wird beim Speichern wieder
 * angehängt — ein „Rechnung" ohne .pdf öffnet auf der Platte kein Programm.
 */
const endungVon = (item) => (item.type === 'folder' ? '' : (item.name.match(/\.[^.]+$/)?.[0] ?? ''));

const nameModalInitial = computed(() => {
  const m = nameModal.value;
  if (!m || m.mode === 'create') return '';
  const endung = endungVon(m.item);
  return endung ? m.item.name.slice(0, -endung.length) : m.item.name;
});

const fehlertext = (err, sonst) => err?.response?.data?.message
  || (typeof err === 'string' ? err : '')
  || sonst;

const submitName = async (rawName) => {
  const name = rawName.trim().replace(/[/\\?%*:|"<>]/g, '');
  if (!name) {
    folderError.value = L('invalidFolderName');
    return;
  }
  const { mode, item } = nameModal.value;
  try {
    if (mode === 'create') {
      await quelle.createFolder(currentFolderId.value, name, props.zone);
    } else {
      await quelle.renameFileNode(item.id, name + endungVon(item));
    }
    nameModal.value = null;
    loadFiles();
  } catch (err) {
    folderError.value = fehlertext(err, mode === 'create' ? L('folderCreateFailed') : L('renameFailed'));
  }
};

const MAX_UPLOAD_BYTES = 1024 * 1024 * 1024; // 1 GB
const MAX_UPLOAD_LABEL = '1 GB';

const zeigeFehler = (text) => {
  uploadErrorMsg.value = text;
  uploadStatus.value = 'error';
  setTimeout(() => { uploadStatus.value = ''; uploadErrorMsg.value = ''; }, 6000);
};

const kontext = () => ({
  zone: props.zone,
  ordnerId: currentFolderId.value,
  neuLaden: loadFiles,
  zeigeFehler,
});

const uploadRoh = async (files) => {
  isUploading.value = true;
  uploadStatus.value = '';
  try {
    let uploaded = 0;
    let tooLargeName = '';
    for (const file of files) {
      if (file.size > MAX_UPLOAD_BYTES) { tooLargeName = tooLargeName || file.name; continue; }
      uploadFortschritt.value = null;
      await quelle.uploadFile(currentFolderId.value, file, props.zone, {
        fortschritt: (anteil) => { uploadFortschritt.value = anteil; },
      });
      uploaded++;
    }
    if (uploaded) loadFiles();
    if (tooLargeName) {
      zeigeFehler(L('uploadTooLarge', { name: tooLargeName, max: MAX_UPLOAD_LABEL }));
    } else {
      uploadStatus.value = 'success';
      setTimeout(() => { uploadStatus.value = ''; }, 3000);
    }
  } catch (err) {
    console.error('Dateiupload-Fehler:', err);
    zeigeFehler(fehlertext(err, ''));
  } finally {
    isUploading.value = false;
    uploadFortschritt.value = null;
  }
};

const handleFileUpload = async (event) => {
  const files = Array.from(event.target.files || event.dataTransfer?.files || []);
  // Sofort zurücksetzen: sonst löst dieselbe Datei beim zweiten Mal kein
  // change-Ereignis aus.
  if (fileInput.value) fileInput.value.value = null;
  if (!files.length) return;

  const roh = props.erweiterungen.hochladenVorbereiten
    ? await props.erweiterungen.hochladenVorbereiten(files, kontext())
    : files;
  if (roh?.length) await uploadRoh(roh);
  if (props.erweiterungen.nachHochladen) await props.erweiterungen.nachHochladen(kontext());
};

const handleDelete = async (item) => {
  const confirmed = await confirmDialog(L('confirmMoveToTrash', { name: item.name }), { title: L('moveToTrashTitle'), confirmLabel: L('moveAction') });
  if (!confirmed) return;
  try {
    await quelle.deleteFileOrFolder(item.id);
    loadFiles();
  } catch (err) {
    console.error(L('deleteFailed'), err);
    toast.error(L('deleteFileFailed'));
  }
};

const downloadFile = (file) => {
  if (file && quelle.herunterladen) quelle.herunterladen(file);
};

// --- Verschieben ---
const moveModal = ref(null);
const folderTree = ref([]);
const openMoveModal = async (item) => {
  try {
    folderTree.value = (await quelle.getFileTree(props.zone)).data.folders;
    moveModal.value = { item };
  } catch (e) {
    toast.error(L('moveFailed'));
  }
};
// Bei Ordnern ohne sich selbst und die eigenen Nachfahren.
const moveOptions = computed(() => {
  const item = moveModal.value?.item;
  if (!item) return [];
  if (item.type !== 'folder') return folderTreeOptions(folderTree.value);
  const excluded = new Set([item.id]);
  let grew = true;
  while (grew) {
    grew = false;
    for (const f of folderTree.value) {
      if (excluded.has(f.parent_id) && !excluded.has(f.id)) {
        excluded.add(f.id);
        grew = true;
      }
    }
  }
  return folderTreeOptions(folderTree.value.filter((f) => !excluded.has(f.id)));
});
const moveItem = async (parentId) => {
  try {
    await quelle.moveFileNode(moveModal.value.item.id, parentId);
    moveModal.value = null;
    loadFiles();
  } catch (e) {
    toast.error(fehlertext(e, L('moveFailed')));
  }
};

const itemActions = (item) => {
  const actions = [];
  if (item.type === 'folder' && quelle.ordnerHerunterladen) {
    actions.push({ key: 'zip', icon: Download, title: L('download'), onClick: () => quelle.ordnerHerunterladen(item) });
  }
  if (props.erweiterungen.aktionen) actions.push(...props.erweiterungen.aktionen(item, kontext()));
  actions.push({ key: 'rename', icon: Pencil, title: L('rename'), onClick: () => openRenameModal(item) });
  actions.push({ key: 'move', icon: FolderInput, title: L('move'), onClick: () => openMoveModal(item) });
  actions.push({ key: 'trash', icon: Trash2, title: L('delete'), onClick: () => handleDelete(item), danger: true });
  return actions;
};

const getFileIcon = (type) => fileIcon(type, isDocuments);
const getIconColorClass = fileIconColor;

// Die Bytes für den PDF-Betrachter. Im Programm holt die Quelle sie bei
// Bedarf erst von einem anderen Gerät.
const pdfLaden = async () => (await quelle.downloadFileContent(pdfDatei.value.id)).data;

// Ausgefüllte Formulare zurückschreiben: dieselbe Datei, gleiche ID. Beim
// ersten Speichern je Öffnen kommt die Fassung davor in den Papierkorb
// (docs/plan-pdf-bearbeiten.md). Ohne `replaceFileContent` bleibt es Ansicht.
const pdfSpeichern = quelle.replaceFileContent
  ? async (bytes, { erstes }) => {
    const datei = pdfDatei.value;
    const neu = new File([bytes], datei.name, { type: 'application/pdf' });
    await quelle.replaceFileContent(datei.id, neu, {
      zone: props.zone,
      vorfassung: erstes ? t('common.pdf.vorfassung') : null,
    });
    loadFiles();
  }
  : null;

const extern = async (datei) => {
  try {
    if (!(await quelle.externOeffnen(datei))) toast.error(t('common.pdf.fehler'));
  } catch (e) {
    toast.error(fehlertext(e, t('common.pdf.fehler')));
  }
};

const openFileViewer = (file) => {
  if (file.type === 'folder') {
    navigateTo(file.id);
  } else if (file.type === 'pdf' && quelle.downloadFileContent) {
    pdfDatei.value = file;
  } else if (quelle.oeffnen) {
    quelle.oeffnen(file).catch?.((e) => toast.error(fehlertext(e, L('downloadFailed'))));
  } else {
    activeViewerFile.value = file;
  }
};

/*
 * SUCHE NACH NAMEN (Tiffy, 02.10.2026, Stufe 1): über ALLE Ordner des
 * Bereichs, nicht nur im offenen -- wer sucht, weiß meist nicht mehr, wo es
 * liegt. Darum steht bei jedem Treffer sein Pfad, und der führt in den
 * Ordner. Gesucht wird in der Quelle: in der Webapp auf dem Server, im
 * Programm im eigenen Speicher (auch ohne Netz).
 */
const kannSuchen = typeof quelle.sucheDateien === 'function';
const sucheOffen = ref(false);
const suchText = ref('');
const treffer = ref([]);
const sucht = ref(false);
const suchFeld = ref(null);
let suchUhr = null;
let suchNr = 0;

const sucheOeffnen = async () => {
  sucheOffen.value = true;
  await nextTick();
  suchFeld.value?.focus();
};
const sucheSchliessen = () => {
  sucheOffen.value = false;
  suchText.value = '';
  treffer.value = [];
};
watch(suchText, (q) => {
  clearTimeout(suchUhr);
  if (q.trim().length < 2) { treffer.value = []; return; }
  suchUhr = setTimeout(async () => {
    const nr = ++suchNr;
    sucht.value = true;
    try {
      const res = await quelle.sucheDateien(q.trim(), props.zone);
      if (nr === suchNr) treffer.value = res.data.results ?? [];
    } catch (e) {
      if (nr === suchNr) toast.error(fehlertext(e, L('loadFilesFailed')));
    } finally {
      if (nr === suchNr) sucht.value = false;
    }
  }, 250);
});
const pfadText = (item) => [L('home'), ...(item.pfad ?? [])].join(' › ');
const markiert = (text) => trefferTeile(text, suchText.value);

// Stufe 2: den Inhalt der Dateien im Hintergrund lesen, solange die Fläche
// offen ist (inhaltsleser.js). Die Suche sagt, solange noch etwas offen ist.
const leser = inhaltsleser(quelle, props.zone);
onUnmounted(() => leser.anhalten());
const trefferOeffnen = (item) => {
  if (item.type === 'folder') {
    sucheSchliessen();
    navigateTo(item.id);
  } else {
    openFileViewer(item);
  }
};
const zumOrdner = (item) => {
  sucheSchliessen();
  navigateTo(item.parent_id ?? null);
};

defineExpose({ neuLaden: loadFiles, ordnerId: currentFolderId });

onMounted(() => {
  loadFiles();
  // Erst nach der Liste -- der Leser soll sie nicht aufhalten.
  setTimeout(() => leser.starten(), 1500);
});
</script>

<template>
  <div class="space-y-1 sm:space-y-6">
    <ModuleHeader :icon="isDocuments ? Archive : Folder" :sticky="false">
      <div class="flex items-center gap-2 sm:gap-3">
        <button
          v-if="kannSuchen"
          @click="sucheOffen ? sucheSchliessen() : sucheOeffnen()"
          :title="L('search')"
          :aria-label="L('search')"
          :aria-pressed="sucheOffen"
          class="flex items-center justify-center h-9 w-9 rounded-xl transition-all cursor-pointer"
          :class="sucheOffen ? 'bg-marke-leise text-marke' : 'bg-slate-100 hover:bg-slate-200 dark:bg-slate-800 dark:hover:bg-slate-700 text-fliess'"
        >
          <Search class="w-4 h-4" />
        </button>
        <button
          @click="openFolderModal"
          class="flex items-center justify-center h-9 gap-1.5 px-2.5 sm:px-4 py-2 bg-slate-100 hover:bg-slate-200 dark:bg-slate-800 dark:hover:bg-slate-700 text-fliess text-sm font-bold rounded-xl transition-all cursor-pointer"
        >
          <Plus class="w-4 h-4" />
          <span>{{ L('newFolder') }}</span>
        </button>

        <!-- In Akten: ein Foto, das durch die Texterkennung zum PDF wird. -->
        <KameraKnopf v-if="isDocuments" arten="foto" @aufgenommen="(dateien) => handleFileUpload({ target: { files: dateien } })" />
        <BaseButton @click="fileInput.click()" groesse="kopf">
          <Upload class="w-4 h-4" />
          <span>{{ LA('upload') }}</span>
        </BaseButton>
        <input type="file" ref="fileInput" class="hidden" @change="handleFileUpload" multiple />
      </div>
    </ModuleHeader>

    <!-- Die Suche steht dort, wo sonst der Pfad steht: Sie ersetzt für den
         Moment die Ordneransicht. -->
    <div v-if="sucheOffen" class="karte p-3 sm:p-4 space-y-3">
      <div class="relative">
        <Search class="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-leise pointer-events-none" />
        <input ref="suchFeld" v-model="suchText" type="text" inputmode="search" enterkeyhint="search" autocomplete="off" autocorrect="off"
               :placeholder="L('searchPlaceholder')" :aria-label="L('search')"
               @keydown.esc="sucheSchliessen"
               class="w-full pl-9 pr-10 py-2.5 bg-vertieft border border-linie rounded-xl text-sm font-medium focus:outline-none focus:ring-2 focus:ring-marke text-schrift" />
        <button type="button" @click="sucheSchliessen" :aria-label="L('searchClose')" :title="L('searchClose')"
                class="absolute right-2 top-1/2 -translate-y-1/2 p-1.5 text-leise hover:bg-auflage rounded-lg cursor-pointer">
          <X class="w-4 h-4" />
        </button>
      </div>
      <p v-if="leser.offen.value > 0" class="flex items-center gap-1.5 text-xs text-leise px-1">
        <Loader2 v-if="leser.laeuft.value" class="w-3.5 h-3.5 animate-spin" />
        {{ L('searchLiest', { n: leser.offen.value }) }}
      </p>
      <div v-if="sucht" class="flex justify-center py-6"><Loader2 class="w-5 h-5 animate-spin text-leise" /></div>
      <p v-else-if="suchText.trim().length >= 2 && !treffer.length" class="text-sm text-leise px-1">{{ L('searchNone') }}</p>
      <ul v-else-if="treffer.length" class="divide-y divide-linie">
        <li v-for="item in treffer" :key="item.id" class="flex items-center gap-3 py-2">
          <button type="button" class="flex items-center gap-3 min-w-0 flex-1 text-left cursor-pointer" @click="trefferOeffnen(item)">
            <component :is="getFileIcon(item.type)" class="w-6 h-6 shrink-0" :class="getIconColorClass(item.type)" />
            <span class="min-w-0">
              <span class="block font-bold text-schrift truncate"><template v-for="(h, k) in markiert(item.name)" :key="k"><mark v-if="h.treffer" class="treffer">{{ h.t }}</mark><template v-else>{{ h.t }}</template></template></span>
              <!-- Im Inhalt gefunden: die Stelle, sonst die Größe. -->
              <span v-if="item.ausschnitt" class="block text-xs text-fliess line-clamp-2"><template v-for="(h, k) in markiert(item.ausschnitt)" :key="k"><mark v-if="h.treffer" class="treffer">{{ h.t }}</mark><template v-else>{{ h.t }}</template></template></span>
              <span v-else class="block text-xs text-leise">{{ item.size || '' }}</span>
            </span>
          </button>
          <button type="button" class="shrink-0 max-w-[45%] text-xs font-bold text-marke hover:underline truncate cursor-pointer"
                  :title="pfadText(item)" @click="zumOrdner(item)">
            {{ pfadText(item) }}
          </button>
        </li>
      </ul>
    </div>

    <ContainerBreadcrumbs
      v-if="!sucheOffen"
      :crumbs="displayBreadcrumbs"
      :show-back="currentFolderId !== null"
      :attached="true"
      @back="goUp"
      @navigate="navigateTo"
    />

    <div
      v-show="!sucheOffen"
      class="relative group/dropzone -mt-1 sm:-mt-6"
      @dragenter.prevent="isDragOver = true"
      @dragover.prevent="isDragOver = true"
      @dragleave.prevent="isDragOver = false"
      @drop.prevent="isDragOver = false; handleFileUpload($event)"
    >
      <div
        v-if="isDragOver"
        class="absolute inset-0 z-50 border-2 border-dashed border-marke bg-marke-leise/90 rounded-xl p-8 text-center flex flex-col items-center justify-center backdrop-blur-sm transition-all"
        :class="isUploading ? 'pointer-events-none opacity-60' : ''"
      >
        <div class="w-16 h-16 rounded-full bg-marke-leise flex items-center justify-center text-marke mb-4 shadow-lg">
          <Upload class="w-8 h-8 animate-bounce" />
        </div>
        <p class="text-xl font-extrabold text-marke">{{ L('dropHere') }}</p>
      </div>

      <div v-if="uploadStatus === 'success'" class="flex items-center gap-3 bg-emerald-50 border border-emerald-200 text-emerald-700 p-4 rounded-xl font-bold">
        <CheckCircle class="w-5 h-5 text-emerald-600" />
        <span>{{ LA('uploadSuccess') }}</span>
      </div>
      <div v-if="uploadStatus === 'error'" class="flex items-center gap-3 bg-rose-50 border border-rose-200 text-rose-700 p-4 rounded-xl font-bold">
        <AlertCircle class="w-5 h-5 text-rose-600" />
        <span>{{ uploadErrorMsg || LA('uploadFailed') }}</span>
      </div>
      <div v-if="isUploading" class="flex items-center justify-center py-10">
        <div class="flex items-center gap-3 bg-marke-leise text-marke p-4 rounded-xl font-bold border border-marke">
          <Loader2 class="animate-spin h-5 w-5 text-marke" />
          <span>{{ L('uploading') }}<template v-if="uploadFortschritt !== null"> {{ Math.round(uploadFortschritt * 100) }} %</template></span>
        </div>
      </div>

      <div v-if="isLoading && !isUploading" class="flex justify-center py-20">
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-marke"></div>
      </div>
      <ExplorerTable
        v-else
        :columns="{ name: L('columnName'), size: L('columnSize'), meta: L('columnChanged'), actions: L('columnActions') }"
        :items="filesList"
        :mobile-actions="true"
        :attached-top="true"
        @open="openFileViewer"
      >
        <template #icon="{ item }">
          <div class="relative shrink-0">
            <component :is="getFileIcon(item.type)" class="w-6 h-6" :class="getIconColorClass(item.type)" />
            <!-- Nur im Programm: Der Inhalt liegt (noch) nicht auf diesem Gerät. -->
            <CloudOff v-if="item.vorhanden === false" class="absolute -bottom-1 -right-1 w-3.5 h-3.5 text-leise bg-flaeche rounded-full" />
            <!-- Nur im Programm: Dieser Ordner bleibt auf diesem Gerät. -->
            <Pin v-if="item.behalten" class="absolute -bottom-1 -right-1 w-3.5 h-3.5 text-marke bg-flaeche rounded-full" />
          </div>
        </template>

        <template #size="{ item }">
          {{ item.size || '--' }}
        </template>

        <template #meta="{ item }">
          {{ formatDateShort(item.updatedAt) }}
        </template>

        <template #actions="{ item }">
          <ActionButtons :actions="itemActions(item)" />
        </template>

        <template #footer>
          <div v-if="filesNextPage" class="p-4 text-center">
            <button @click="loadMoreFiles" :disabled="loadingMore" class="px-5 py-2 text-sm font-bold text-marke hover:bg-marke-leise rounded-xl disabled:opacity-50">{{ L('loadMore') }}</button>
          </div>
        </template>

        <template #empty>
          <div class="w-16 h-16 rounded-full bg-slate-50 dark:bg-slate-800 text-slate-400 flex items-center justify-center mx-auto">
            <component :is="isDocuments ? Archive : Folder" class="w-8 h-8" />
          </div>
          <h4 class="text-lg font-extrabold text-schrift">{{ L('emptyTitle') }}</h4>
          <p class="text-sm text-leise">{{ LA('emptyHint') }}</p>
          <component
            :is="Link && hilfeZiel ? Link : 'button'"
            :to="Link && hilfeZiel ? hilfeZiel : undefined"
            class="inline-block text-sm font-bold text-marke hover:underline underline-offset-2 cursor-pointer"
            @click="!(Link && hilfeZiel) && emit('hilfe', isDocuments ? 'files-dokumente' : 'files-dateien')"
          >
            {{ t('common.howItWorks') }}
          </component>
        </template>
      </ExplorerTable>
    </div>

    <NameModal
      v-if="nameModal"
      :key="nameModal.mode + '-' + (nameModal.item?.id ?? 'new')"
      :title="nameModal.mode === 'create' ? L('createFolderTitle') : L('renameTitle', { name: nameModal.item.name })"
      :label="nameModal.mode === 'create' ? L('folderName') : L('newName')"
      :placeholder="nameModal.mode === 'create' ? L('folderNamePlaceholder') : ''"
      :submit-label="nameModal.mode === 'create' ? L('createFolder') : L('save')"
      :cancel-label="L('cancel')"
      :initial="nameModalInitial"
      :error="folderError"
      @submit="submitName"
      @close="nameModal = null"
    />

    <MoveTargetModal
      v-if="moveModal"
      :title="L('moveTitle', { name: moveModal.item.name })"
      :options="moveOptions"
      :current-id="currentFolderId"
      :top-label="L('home')"
      @select="moveItem"
      @close="moveModal = null"
    />

    <PdfBetrachter v-if="pdfDatei" :key="pdfDatei.id" :name="pdfDatei.name" :laden="pdfLaden" :speichern="pdfSpeichern" @close="pdfDatei = null">
      <template #aktionen>
        <button
          v-if="quelle.herunterladen"
          class="p-2 rounded-xl text-fliess hover:bg-auflage transition-colors cursor-pointer"
          :title="L('download')"
          :aria-label="L('download')"
          @click="downloadFile(pdfDatei)"
        >
          <Download class="w-5 h-5" />
        </button>
        <button
          v-if="quelle.externOeffnen"
          class="p-2 rounded-xl text-fliess hover:bg-auflage transition-colors cursor-pointer"
          :title="t('common.pdf.extern')"
          :aria-label="t('common.pdf.extern')"
          @click="extern(pdfDatei)"
        >
          <ExternalLink class="w-5 h-5" />
        </button>
      </template>
    </PdfBetrachter>

    <!-- Der Download-Dialog der Webapp. Im Programm öffnet `oeffnen` direkt. -->
    <div
      v-if="activeViewerFile !== null"
      class="fixed inset-0 bg-slate-900/80 backdrop-blur-md flex items-center justify-center p-4 z-[100] animate-modal-in"
      @click.self="activeViewerFile = null"
    >
      <div class="karte w-full max-w-4xl shadow-2xl overflow-hidden flex flex-col max-h-[85vh]">
        <div class="px-6 py-5 flex items-center justify-between border-b border-linie shrink-0">
          <div class="flex items-center gap-3">
            <component :is="getFileIcon(activeViewerFile.type)" class="w-6 h-6" :class="getIconColorClass(activeViewerFile.type)" />
            <h3 class="font-extrabold text-lg text-schrift truncate max-w-[200px] sm:max-w-lg">{{ activeViewerFile.name }}</h3>
          </div>
          <div class="flex items-center gap-4">
            <button
              v-if="quelle.herunterladen && activeViewerFile.url !== null"
              @click="downloadFile(activeViewerFile)"
              class="text-sm font-bold text-marke hover:text-marke flex items-center gap-1.5 px-4 py-2 hover:bg-marke-leise rounded-xl transition-all cursor-pointer"
              :title="L('download')"
            >
              <Download class="w-4 h-4" />
              <span class="hidden sm:inline">{{ L('download') }}</span>
            </button>
            <button @click="activeViewerFile = null" class="p-2 text-slate-400 hover:bg-auflage rounded-xl transition-colors cursor-pointer">
              <X class="w-5 h-5" />
            </button>
          </div>
        </div>

        <div class="p-6 overflow-y-auto flex-grow bg-vertieft flex items-center justify-center">
          <div class="text-center space-y-6 max-w-sm mx-auto py-12">
            <div class="w-24 h-24 rounded-xl bg-marke-leise text-marke flex items-center justify-center mx-auto shadow-inner border border-marke">
              <component :is="getFileIcon(activeViewerFile.type)" class="w-12 h-12" />
            </div>
            <div class="space-y-2">
              <h4 class="font-extrabold text-2xl text-schrift">{{ L('readyToDownload') }}</h4>
              <p class="text-sm font-medium text-leise leading-relaxed">{{ L('downloadHint') }}</p>
            </div>
            <BaseButton v-if="quelle.herunterladen" @click="downloadFile(activeViewerFile)" class="w-full hover:shadow-xl hover:-translate-y-0.5 mt-4" groesse="gross">
              <Download class="w-5 h-5" />
              {{ L('downloadFile') }}
            </BaseButton>
            <slot name="betrachter-zusatz" :datei="activeViewerFile" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Ein Suchtreffer im Text -- wie im Nachrichtenmodul. */
.treffer {
  background: color-mix(in srgb, var(--color-marke, #0d9488) 25%, transparent);
  color: inherit;
  border-radius: 0.2em;
  padding: 0 0.05em;
}
</style>
