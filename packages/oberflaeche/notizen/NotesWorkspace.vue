<script setup>
// Geteilte Notiz-Arbeitsfläche (aus Notes.vue extrahiert): Explorer,
// Editor, Suche, Tags, Backlinks, Gliederung, Graph. Statt direkt den
// api-Service aufzurufen, nutzt sie eine injizierte Datenquelle
// (dataSource) mit denselben Methodennamen – so kann dieselbe Komponente
// persönliche UND (später) projekt-scoped Notizen bedienen.
import { ref, computed, onMounted, watch, defineAsyncComponent } from 'vue';
import { useI18n } from 'vue-i18n';
import MarkdownEditor from '@oberflaeche/editor/MarkdownEditor.vue';
import ActionButtons from '@oberflaeche/container/ActionButtons.vue';
import ContainerBreadcrumbs from '@oberflaeche/container/ContainerBreadcrumbs.vue';
import ExplorerTable from '@oberflaeche/container/ExplorerTable.vue';
import MoveTargetModal from '@oberflaeche/container/MoveTargetModal.vue';
import NameModal from '@oberflaeche/container/NameModal.vue';
import ModuleHeader from '@oberflaeche/base/ModuleHeader.vue';
import IconPdf from '@oberflaeche/icons/IconPdf.vue';
import IconMd from '@oberflaeche/icons/IconMd.vue';
import {
  FileText, Folder, FolderInput, ArrowLeft, Plus, Trash2, X,
  Loader2, Download, Pencil, Share2, Undo, Redo,
  Link2, Waypoints, Search, BookMarked, ListTree, ChevronRight, Tag, Users,
  AlertTriangle, RefreshCw, Check, AlertCircle
} from 'lucide-vue-next';

// Graph + Übersicht nur bei Bedarf laden (hält den Notizen-Chunk klein).
const NotesIndex = defineAsyncComponent(() => import('./NotesIndex.vue'));
import { useToast } from '@oberflaeche/composables/useToast';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import { debounce } from '@oberflaeche/shared/debounce';
import { useNoteAutosave } from '@oberflaeche/composables/useNoteAutosave';
import { useNoteAssets } from '@oberflaeche/composables/useNoteAssets';
import { useNoteExport } from '@oberflaeche/composables/useNoteExport';
import { childFolders, buildBreadcrumbs, folderTreeOptions } from '@oberflaeche/shared/folderTree';
import { formatDateShort, formatDateTimeShort } from '@oberflaeche/shared/date';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import ModulePage from '@oberflaeche/base/ModulePage.vue';

const props = defineProps({
  // Datenquelle mit denselben Methodennamen wie der api-Service
  // (getNotes, createNote, getNoteFolders, …) plus assetUrl/getGraph/
  // supportsLibrary. Persönlich = der echte api-Service, Projekt = Adapter.
  dataSource: { type: Object, required: true },
  // Eingebettet (z. B. im Projekt-Tab): ohne eigenen Seitenrahmen und mit
  // nicht-klebendem Kopf, da schon ein äußerer Rahmen/Tab existiert.
  embedded: { type: Boolean, default: false },
  // Startebene. Wer die Arbeitsfläche schon für eine BESTIMMTE Mappe öffnet
  // (Freigaben-Reiter: der Nutzer hat die Mappe gerade angeklickt), landet
  // direkt darin, statt sie über die Wurzel erneut heraussuchen zu müssen.
  // Der Pfad bleibt vollständig – über die Brotkrumen geht es nach oben.
  initialFolderId: { type: [Number, String], default: null },
  // Notiz, die beim Aufbau gleich geöffnet werden soll (Klick auf einen
  // [[Verweis]] im Projekt-Chat). Wie initialFolderId nur beim Aufbau wirksam.
  initialNoteId: { type: [Number, String], default: null },
  // WAS NUR MIT SERVER GEHT, REICHT DIE ANWENDUNG HEREIN (seit 15.09.2026,
  // gemeinsames Paket): { teilen, einfuegen, ki, graph } – je eine Komponente
  // oder nichts. Die Webapp gibt alle vier (notizErweiterungen.js), das
  // Programm keine. Fehlt eine, fehlt auch ihr Knopf: kein Knopf, der ins
  // Leere führt.
  erweiterungen: { type: Object, default: () => ({}) },
});
// Lokal als `api` gebunden, damit alle bestehenden Aufrufstellen
// (api.getNotes(), api.createNote(), …) unverändert bleiben.
const api = props.dataSource;

const { t } = useI18n();
const toast = useToast();
const { confirmDialog } = useConfirm();

// --- State (Explorer wie im Dokumente-Modul, nur ohne Größen-Spalte) ---
// Notizen werden pro Mappen-Ebene geladen und paginiert (Server:
// note_folder_id + page), damit auch tausende Notizen schlank bleiben.
const notes = ref([]);          // Notizen der AKTUELLEN Ebene (paginiert)
const notesNextPage = ref(null);
const loadingMore = ref(false);
const folders = ref([]);        // alle eigenen Mappen (flach: {id, name, parent_id})
// null = oberste Ebene (Home). Als Zahl halten: Vergleiche gegen Mappen-Ids
// und note_folder_id laufen strikt (===), eine Id als Zeichenkette träfe nie.
const currentFolderId = ref(props.initialFolderId != null ? Number(props.initialFolderId) : null);
const isLoading = ref(false);
const errorMsg = ref('');

// Papierkorb

// Geöffnete Notiz (Editor-Ansicht ersetzt den Explorer)
const openNote = ref(null);
const noteUpdatedAt = ref(null); // „Zuletzt geändert" separat (siehe openNoteEditor)
// Read-only, wenn die geöffnete Notiz nur mit Leserecht sichtbar ist (projekt-
// geteilte Mappe ohne Bearbeiten). Persönlich = 'owner' -> immer editierbar.
const noteReadOnly = computed(() => openNote.value?.access === 'read');

// KI-Antwort ans Ende der Notiz – über das v-model, damit der Autosave sie
// wie getippten Text speichert. Nur Projektnotizen haben ein KI-Feld
// (api.ai ist bei persönlichen Notizen null).
const insertAiAnswer = (text) => {
  if (!openNote.value || noteReadOnly.value) return;
  const bisher = (openNote.value.content || '').replace(/\s+$/, '');
  openNote.value.content = `${bisher}${bisher ? '\n\n' : ''}${text.trim()}\n`;
};

// Obsidian-Stil: Titel-Liste fürs [[-Autocomplete, Backlinks der offenen
// Notiz und #Tags (Filter-Chips, Mappen-übergreifend).
const noteTitles = ref([]);   // [{ id, title }]
const backlinks = ref([]);    // [{ id, title }] – wer verlinkt hierher?
const backlinksOpen = ref(false); // Chip-Liste eingeklappt (spart Editor-Höhe)
const tagsOpen = ref(false);  // #Tag-Chips der offenen Notiz eingeklappt
// #Tags der offenen Notiz, direkt aus dem Inhalt gelesen (gleiche Regel wie
// Note::TAG_PATTERN im Backend: beginnt mit Buchstabe, nur nach Zeilenanfang/
// Whitespace → Überschriften „# …" sind KEIN Tag), kleingeschrieben + dedupliziert.
const noteTags = computed(() => {
  const content = openNote.value?.content || '';
  const re = /(?<=^|\s)#([\p{L}][\p{L}\p{N}_/-]{0,49})/gu;
  const seen = new Set();
  let m;
  while ((m = re.exec(content)) !== null) seen.add(m[1].toLowerCase());
  return [...seen];
});
const allTags = ref([]);      // [{ tag, count }]
const activeTag = ref(null);  // gesetzter Tag-Filter (oder null)
const searchQuery = ref('');  // Volltextsuche (Titel + Inhalt, Mappen-übergreifend)
const showGraph = ref(false); // Graph-Ansicht (ersetzt den Explorer)
const showIndex = ref(false); // Übersicht/Index (ersetzt den Explorer)

const toggleGraph = () => {
  showGraph.value = !showGraph.value;
  if (showGraph.value) { showIndex.value = false; closeNoteEditor(); }
};

const toggleIndex = () => {
  showIndex.value = !showIndex.value;
  if (showIndex.value) { showGraph.value = false; closeNoteEditor(); }
};

// Titel/Tag aus der Übersicht angeklickt (analog zu den Graph-Handlern).
const openFromIndex = async (id) => {
  showIndex.value = false;
  try {
    openNoteEditor((await api.getNote(id)).data);
  } catch (e) {
    toast.error(t('notes.loadFailed'));
  }
};
const filterByTagFromIndex = (tag) => {
  showIndex.value = false;
  activeTag.value = tag;
  searchQuery.value = '';
  loadNotes({ reset: true });
};

// Knoten im Graph angeklickt: Notiz öffnen (verlässt die Graph-Ansicht).
const openFromGraph = async (node) => {
  showGraph.value = false;
  try {
    openNoteEditor((await api.getNote(node.id)).data);
  } catch (e) {
    toast.error(t('notes.loadFailed'));
  }
};

// Tag-Raute im Graph angeklickt: Graph verlassen, nach dem Tag filtern
// (wie ein Klick auf den Filter-Chip in der Liste).
const filterByTagFromGraph = (tag) => {
  showGraph.value = false;
  activeTag.value = tag;
  searchQuery.value = '';
  loadNotes({ reset: true });
};

// #Tag-Chip innerhalb der offenen Notiz angeklickt: Editor schließen und nach
// dem Tag filtern (wie ein Klick auf den Filter-Chip in der Liste/im Graph).
const filterByTagFromNote = (tag) => {
  closeNoteEditor();
  activeTag.value = tag;
  searchQuery.value = '';
  loadNotes({ reset: true });
};

// Modals
const folderModal = ref(null);      // { mode: 'create'|'rename', id, name }
const deleteFolderModal = ref(null); // { folder }
const moveNoteModal = ref(null);     // { note }
const shareModal = ref(null);        // { folder } → In Projekt freigeben
const folderError = ref('');

// --- Herausgelöste Zuständigkeiten ---
// Autosave, Anhänge/Zitate und Export haben mit dem Explorer drumherum wenig
// zu tun und stecken je in einem eigenen Composable. Was hier bleibt, ist das
// Zusammenspiel: welche Notiz offen ist, welche Ebene angezeigt wird.
const editorRef = ref(null);

const {
  noteCitations, noteContextFolderId, insertDialogOpen, libraryModal,
  loadNoteContext, assetResolver, applyInsert, onPasteImage, onAssetOpen,
  openLibraryModal, setLibrary,
} = useNoteAssets({ api, editorRef, reloadAll: () => loadAll() });

const {
  exportingPdfId, downloadingFolderId,
  exportNotePdf, exportNoteMd, downloadFolderZip,
} = useNoteExport({ api });

const {
  saveStatus, saveError, externalChange, reimporting,
  retrySave, reloadAfterExternalChange, reimportFromFiles,
} = useNoteAutosave({
  api,
  openNote,
  noteReadOnly,
  // Serverantwort in die Listen zurückspiegeln: Änderungsdatum, Zeile im
  // Explorer und der Titel im Wikilink-Autocomplete.
  applySaved: (data) => {
    noteUpdatedAt.value = data.updated_at;
    const idx = notes.value.findIndex((n) => n.id === data.id);
    if (idx !== -1) notes.value[idx] = { ...notes.value[idx], ...data };
    const tIdx = noteTitles.value.findIndex((n) => n.id === data.id);
    if (tIdx !== -1) noteTitles.value[tIdx] = { id: data.id, title: data.title };
  },
  reopenNote: (data) => openNoteEditor(data),
  reloadAll: () => loadAll(),
});

// --- Laden ---
// Mappen bilden den Baum (klein, komplett geladen); Notizen kommen pro
// Ebene und Seite. loadAll lädt die Mappen + erste Seite der Ebene.
const loadAll = async () => {
  isLoading.value = true;
  errorMsg.value = '';
  try {
    const [foldersRes, titlesRes, tagsRes] = await Promise.all([
      api.getNoteFolders(), api.getNoteTitles(), api.getNoteTags(),
    ]);
    folders.value = foldersRes.data.folders;
    noteTitles.value = titlesRes.data.items;
    allTags.value = tagsRes.data.items;
    await loadNotes({ reset: true });
  } catch (e) {
    errorMsg.value = t('notes.loadFailed');
    console.error(e);
  } finally {
    isLoading.value = false;
  }
};

// Beim Suchen gilt standardmäßig „überall": Man sucht meist etwas, dessen
// Mappe man gerade NICHT weiß. Wer eingrenzen will, schaltet um – der Server
// kann Suche, Tag- und Mappen-Filter seit Kurzem kombinieren.
const searchInFolder = ref(false);

const loadNotes = async ({ reset = false } = {}) => {
  const page = reset ? 1 : notesNextPage.value;
  if (page == null) return;
  if (reset) { notes.value = []; notesNextPage.value = null; }
  const suchend = searchQuery.value.trim() !== '';
  const folderId = (suchend && !searchInFolder.value) ? null : currentFolderId.value;
  const res = await api.getNotes(folderId, page, activeTag.value, searchQuery.value.trim());
  notes.value = reset ? res.data.items : [...notes.value, ...res.data.items];
  notesNextPage.value = res.data.next_page;
};

// Suche-während-des-Tippens: verzögert nachladen. Der Tag-Filter bleibt
// bestehen – Suche und Tag greifen gemeinsam.
const runSearch = debounce(() => loadNotes({ reset: true }), 300);
watch(searchQuery, runSearch);
watch(searchInFolder, () => loadNotes({ reset: true }));
const clearSearch = () => { searchQuery.value = ''; searchInFolder.value = false; };

const loadMoreNotes = async () => {
  loadingMore.value = true;
  try { await loadNotes(); } finally { loadingMore.value = false; }
};

// Ebene wechseln: aktuelle Mappe setzen und deren Notizen frisch laden
// (hebt einen aktiven Tag-Filter auf).
const goToFolder = (id) => {
  currentFolderId.value = id;
  activeTag.value = null;
  searchQuery.value = '';
  loadNotes({ reset: true });
};

// --- Aktuelle Ebene: Mappen zuerst, dann Notizen (alphabetisch) ---
// Mit aktivem Tag-Filter: flache Notiz-Liste über alle Mappen, keine Ordner.
const currentItems = computed(() => {
  if (activeTag.value || searchQuery.value.trim() !== '') {
    return notes.value
      .slice()
      .sort((a, b) => (a.title || '').localeCompare(b.title || '', 'de'))
      .map((n) => ({ ...n, type: 'note', name: n.title || t('notes.untitled') }));
  }
  const folderItems = childFolders(folders.value, currentFolderId.value)
    .map((f) => ({ ...f, type: 'folder' }));
  const noteItems = notes.value
    .filter((n) => (n.note_folder_id ?? null) === currentFolderId.value)
    .sort((a, b) => (a.title || '').localeCompare(b.title || '', 'de'))
    .map((n) => ({ ...n, type: 'note', name: n.title || t('notes.untitled') }));
  return [...folderItems, ...noteItems];
});

// --- Breadcrumbs (aus der flachen Mappen-Liste aufgebaut) ---
const displayBreadcrumbs = computed(() => buildBreadcrumbs(folders.value, currentFolderId.value));

// --- Navigation ---
// Pfad-Klick schließt eine offene Notiz (der Pfad bleibt beim Bearbeiten
// sichtbar) und wechselt dann in die Mappe.
const navigateTo = (id) => { closeNoteEditor(); goToFolder(id); };
const enterFolder = (id) => goToFolder(id);
const goUp = () => {
  const bc = displayBreadcrumbs.value;
  goToFolder(bc.length > 1 ? bc[bc.length - 2].id : null);
};

// --- Notiz öffnen / schließen ---
// Die Explorer-Liste kommt ohne content (Performance); beim Öffnen wird
// die volle Notiz nachgeladen.
const openNoteEditor = async (note) => {
  let full = note;
  if (note.content === undefined) {
    try {
      full = (await api.getNote(note.id)).data;
    } catch (e) {
      toast.error(t('notes.loadFailed'));
      return;
    }
  }
  // Anhänge + Zitier-Kandidaten der Mappe VOR dem Editor laden, damit
  // ![](pfad)-Bilder beim ersten Rendern aufgelöst werden können.
  await loadNoteContext(full.note_folder_id ?? null);
  openNote.value = { ...full };
  // Änderungsdatum separat halten (nicht in openNote), damit der Deep-Watch
  // beim Aktualisieren keine Save-Schleife auslöst.
  noteUpdatedAt.value = full.updated_at;
  saveStatus.value = null;
  saveError.value = '';
  loadBacklinks(full.id);
};

const closeNoteEditor = () => {
  const wasOpen = openNote.value != null;
  openNote.value = null;
  backlinks.value = [];
  // Tags können sich beim Bearbeiten geändert haben – Chips leise auffrischen.
  if (wasOpen) {
    api.getNoteTags().then((r) => { allTags.value = r.data.items; }).catch(() => {});
  }
};

const openItem = (item) => {
  if (item.type === 'folder') enterFolder(item.id);
  else openNoteEditor(item);
};

// --- Backlinks („Verlinkt von …") der offenen Notiz ---
const loadBacklinks = async (noteId) => {
  backlinks.value = [];
  backlinksOpen.value = false; // bei jeder neu geöffneten Notiz wieder zuklappen
  tagsOpen.value = false;      // Tag-Chips ebenfalls eingeklappt starten
  try {
    backlinks.value = (await api.getNoteBacklinks(noteId)).data.items;
  } catch (e) { /* Panel ist optional – Fehler nicht störend melden */ }
};

// --- [[Wikilink]] angeklickt: Notiz per Titel öffnen (oder anlegen) ---
// noteTitles ist nur ein schneller Weg: Die Liste ist serverseitig auf 2000
// Einträge gedeckelt. Ein Fehltreffer darin heißt also NICHT, dass die Notiz
// fehlt – deshalb vor dem Anlege-Angebot immer den Server fragen. Sonst legt
// der Editor oberhalb der Grenze Duplikate an, und doppelte Titel machen die
// Wikilink-Auflösung dauerhaft mehrdeutig.
const onWikilink = async (title) => {
  const openById = async (id) => {
    try {
      openNoteEditor((await api.getNote(id)).data);
    } catch (e) {
      toast.error(t('notes.loadFailed'));
    }
  };

  // Kanonische Ziele („20260731153042 Titel") sind über die Titel-Liste nicht
  // auflösbar. Die Id abzuschneiden und über den Rest zu suchen wäre FALSCH:
  // Bei zwei gleichnamigen Notizen unterscheidet genau sie die beiden. Also
  // fragt hier immer der Server, der zk_id kennt.
  const hatIdPraefix = /^\d{14}(\s|$)/.test(title.trim());

  const cached = hatIdPraefix
    ? null
    : noteTitles.value.find((n) => (n.title || '').toLowerCase() === title.toLowerCase());
  if (cached) { await openById(cached.id); return; }

  try {
    const found = (await api.resolveTitle(title)).data?.note;
    if (found) {
      // Vorhanden, nur außerhalb der gedeckelten Liste – für den nächsten Klick merken.
      noteTitles.value.push({ id: found.id, title: found.title });
      await openById(found.id);
      return;
    }
  } catch (e) {
    // Im Zweifel NICHT das Anlegen anbieten – ein Netzfehler darf kein Duplikat erzeugen.
    toast.error(t('notes.loadFailed'));
    return;
  }

  // Trägt das Ziel eine Kennung, hat die Notiz einmal existiert. Anlegen
  // repariert das NICHT: Die neue Notiz bekäme eine neue Kennung, der Verweis
  // zeigte weiter auf die alte. Also sagen, was ist, statt etwas anzubieten,
  // das den Klick beim nächsten Mal genauso enden lässt.
  if (hatIdPraefix) { toast.error(t('notes.wikiTargetGone')); return; }

  // Wie in Obsidian: unaufgelöster Link bietet das Anlegen an.
  if (!(await confirmDialog(t('notes.wikiCreateConfirm', { title }), { title: t('notes.wikiCreateTitle'), confirmLabel: t('notes.wikiCreateAction') }))) return;
  try {
    const res = await api.createNote({ title, content: '', note_folder_id: currentFolderId.value });
    noteTitles.value.push({ id: res.data.id, title: res.data.title });
    if ((res.data.note_folder_id ?? null) === currentFolderId.value) notes.value.unshift(res.data);
    openNoteEditor(res.data);
  } catch (e) {
    toast.error(t('notes.createNoteFailed'));
  }
};

// Backlink-Chip angeklickt: Quelle öffnen.
const openBacklink = async (link) => {
  try {
    openNoteEditor((await api.getNote(link.id)).data);
  } catch (e) {
    toast.error(t('notes.loadFailed'));
  }
};

// --- Notiz CRUD ---
const handleCreateNote = async () => {
  try {
    // Ohne Titel anlegen: Den bestimmt die erste Zeile, sobald etwas darin
    // steht (NoteTitle::fromContent). Vorher hieß jede neue Notiz „Neue
    // Notiz" – ein Dutzend gleichnamiger Einträge im Vault, und
    // `[[Neue Notiz]]` war mehrdeutig.
    //
    // Bewusst kein deutschsprachiger Vorgabewert, den das Backend erkennen
    // müsste: Auf Englisch hieße er „New note", und ein Vergleich gegen eine
    // übersetzte Zeichenkette wäre brüchig. Leer ist eindeutig.
    const res = await api.createNote({ title: '', content: '', note_folder_id: currentFolderId.value });
    notes.value.unshift(res.data);
    noteTitles.value.push({ id: res.data.id, title: res.data.title });
    openNoteEditor(res.data);
  } catch (e) {
    toast.error(t('notes.createNoteFailed'));
  }
};

const handleDeleteNote = async (note) => {
  if (!(await confirmDialog(t('notes.confirmMoveToTrash', { name: note.title || t('notes.untitled') }), { title: t('notes.moveToTrashTitle'), confirmLabel: t('notes.moveAction') }))) return;
  try {
    await api.deleteNote(note.id);
    notes.value = notes.value.filter((n) => n.id !== note.id);
    if (openNote.value?.id === note.id) closeNoteEditor();
  } catch (e) {
    toast.error(t('notes.deleteNoteFailed'));
  }
};

// --- Notiz verschieben ---
// Alle Mappen als eingerückte Liste (DFS) für den Verschieben-Dialog.
const folderOptions = computed(() => folderTreeOptions(folders.value));
// Synthetische Eigentümer-Gruppe („Mappen von X", nur im Projekt): reine
// Navigationsebene ohne echte Mappe -> hier nichts anlegen/keine Aktionen.
const currentFolderIsOwnerGroup = computed(() => folders.value.find((f) => f.id === currentFolderId.value)?.owner_group === true);

const moveNote = async (note, folderId) => {
  try {
    const res = await api.updateNote(note.id, { note_folder_id: folderId });
    // Verschobene Notiz verlässt die aktuelle Ebene → aus der Liste nehmen.
    if ((folderId ?? null) !== (currentFolderId.value ?? null)) {
      notes.value = notes.value.filter((n) => n.id !== note.id);
    } else {
      const idx = notes.value.findIndex((n) => n.id === note.id);
      if (idx !== -1) notes.value[idx] = { ...notes.value[idx], ...res.data };
    }
    if (openNote.value?.id === note.id) openNote.value = { ...openNote.value, note_folder_id: folderId };
    moveNoteModal.value = null;
  } catch (err) {
    toast.error(err.response?.data?.message || t('notes.moveFailed'));
  }
};

// --- Mappen CRUD ---
const openCreateFolder = () => {
  folderError.value = '';
  folderModal.value = { mode: 'create', id: null, name: '' };
};
const openRenameFolder = (folder) => {
  folderError.value = '';
  folderModal.value = { mode: 'rename', id: folder.id, name: folder.name };
};
const submitFolder = async (rawName) => {
  const name = rawName.trim().replace(/[/\\?%*:|"<>]/g, '');
  if (!name) {
    folderError.value = t('notes.invalidFolderName');
    return;
  }
  try {
    if (folderModal.value.mode === 'create') {
      await api.createNoteFolder(name, currentFolderId.value);
    } else {
      await api.updateNoteFolder(folderModal.value.id, { name });
    }
    folderModal.value = null;
    await loadAll();
  } catch (err) {
    folderError.value = err.response?.data?.message || t('notes.folderSaveFailed');
  }
};

const confirmDeleteFolder = async (mode) => {
  const folder = deleteFolderModal.value.folder;
  try {
    await api.deleteNoteFolder(folder.id, mode);
    deleteFolderModal.value = null;
    await loadAll();
  } catch (err) {
    toast.error(err.response?.data?.message || t('common.deleteFailed'));
  }
};

// Papierkorb: läuft zentral über /papierkorb (Trash.vue).

// --- Aktions-Definitionen für die ActionButtons-Komponente ---
// (Darstellung zentral in components/container, Definition bleibt hier.)
const folderActions = (item) => {
  if (item.owner_group) return [];
  // Umbenennen/Freigeben/Löschen nur für EIGENE Mappen (access 'owner'). Fremde
  // bzw. KI-Mappen (im Projekt mit Stufe read/edit sichtbar) darf man nicht
  // strukturell ändern – sonst liefe man in einen 403. Das Herauslösen aus dem
  // Projekt geschieht über den Freigaben-Reiter. Persönliche Ansicht: kein
  // access-Feld -> alles erlaubt.
  const owns = !item.access || item.access === 'owner';
  const actions = api.exportNoteFolder
    ? [{ key: 'zip', icon: Download, title: t('notes.downloadZip'), onClick: () => downloadFolderZip(item), loading: downloadingFolderId.value === item.id }]
    : [];
  if (owns) {
    actions.push({ key: 'rename', icon: Pencil, title: t('notes.rename'), onClick: () => openRenameFolder(item) });
    if (props.erweiterungen.teilen) {
      actions.push({ key: 'share', icon: Share2, title: t('notes.shareToProject'), onClick: () => { shareModal.value = { folder: item }; } });
    }
    // Bibliothek („Aus openany") ist eigentümer-lokal -> nur in der persönlichen Ansicht.
    if (api.supportsLibrary) {
      actions.push({ key: 'library', icon: BookMarked, title: t('notes.libraryAction'), onClick: () => openLibraryModal(item) });
    }
    actions.push({ key: 'trash', icon: Trash2, title: t('notes.delete'), onClick: () => { deleteFolderModal.value = { folder: item }; }, danger: true });
  }
  return actions;
};
const noteActions = (item) => ([
  { key: 'md', icon: IconMd, title: t('notes.downloadMd'), onClick: () => exportNoteMd(item) },
  ...(api.exportNotePdf
    ? [{ key: 'pdf', icon: IconPdf, title: t('notes.downloadPdf'), onClick: () => exportNotePdf(item), loading: exportingPdfId.value === item.id }]
    : []),
  { key: 'move', icon: FolderInput, title: t('notes.moveToFolder'), onClick: () => { moveNoteModal.value = { note: item }; } },
  { key: 'trash', icon: Trash2, title: t('notes.delete'), onClick: () => handleDeleteNote(item), danger: true },
]);


onMounted(async () => {
  await loadAll();
  // Notiz gleich öffnen: entweder als Prop (Verweis aus dem Projekt-Chat)
  // oder als Deep-Link von der Widget-Startseite (/notes?note=ID).
  const noteId = Number(props.initialNoteId)
    || Number(new URLSearchParams(window.location.search).get('note'));
  if (noteId) openNoteEditor({ id: noteId });
});
</script>

<template>
  <!-- Rahmen: eigenständige Seite (ModulePage-Maße) oder eingebettet ohne
       Rahmen (z. B. Projekt-Tab, der schon einen äußeren Rahmen mitbringt). -->
  <!-- Eingebettet (im Speicher-Rahmen) nur der vertikale Rhythmus, sonst der
       gemeinsame Seiten-Rahmen aller Modul-Seiten. Die volle Kette stand hier
       ausgeschrieben und war Zeichen für Zeichen ModulePage. -->
  <component :is="embedded ? 'div' : ModulePage" :class="embedded ? 'space-y-1 sm:space-y-6' : ''">

    <!-- Titel + Aktionen: Mobil ohne Icon-Kachel/Titel, die Buttons
         rücken in die Titelzeile. Der Papierkorb wohnt zentral unter
         /papierkorb (Profil-/Burgermenü). -->
    <ModuleHeader :icon="FileText" :sticky="!embedded">
      <div class="flex items-center gap-2 sm:gap-3">
        <!-- Nach einer Sitzung in Zettlr: Dateien wieder einlesen. Nur
             persönlich – im Projektkontext gibt es keinen Vault-Zweig. -->
        <button
          v-if="api.supportsLibrary"
          @click="reimportFromFiles"
          :disabled="reimporting"
          :title="t('notes.reimportTitle')"
          class="flex items-center justify-center h-9 gap-1.5 px-2.5 sm:px-4 py-2 text-sm font-bold rounded-xl transition-all cursor-pointer bg-slate-100 hover:bg-slate-200 dark:bg-slate-800 dark:hover:bg-slate-700 text-fliess disabled:opacity-50"
        >
          <RefreshCw class="w-4 h-4" :class="reimporting ? 'animate-spin' : ''" />
          <span class="hidden sm:inline">{{ t('notes.reimportButton') }}</span>
        </button>

        <button
          @click="toggleIndex"
          :title="t('notes.indexTitle')"
          class="flex items-center justify-center h-9 gap-1.5 px-2.5 sm:px-4 py-2 text-sm font-bold rounded-xl transition-all cursor-pointer"
          :class="showIndex
            ? 'bg-slate-800 dark:bg-slate-200 text-white dark:text-slate-900'
            : 'bg-slate-100 hover:bg-slate-200 dark:bg-slate-800 dark:hover:bg-slate-700 text-fliess'"
        >
          <ListTree class="w-4 h-4" />
          <span class="hidden sm:inline">{{ t('notes.indexButton') }}</span>
        </button>

        <button
          v-if="erweiterungen.graph"
          @click="toggleGraph"
          :title="t('notes.graphTitle')"
          class="flex items-center justify-center h-9 gap-1.5 px-2.5 sm:px-4 py-2 text-sm font-bold rounded-xl transition-all cursor-pointer"
          :class="showGraph
            ? 'bg-slate-800 dark:bg-slate-200 text-white dark:text-slate-900'
            : 'bg-slate-100 hover:bg-slate-200 dark:bg-slate-800 dark:hover:bg-slate-700 text-fliess'"
        >
          <Waypoints class="w-4 h-4" />
          <span class="hidden sm:inline">{{ t('notes.graphButton') }}</span>
        </button>

        <button
            v-if="!currentFolderIsOwnerGroup"
            @click="openCreateFolder"
            class="flex items-center justify-center h-9 gap-1.5 px-2.5 sm:px-4 py-2 bg-slate-100 hover:bg-slate-200 dark:bg-slate-800 dark:hover:bg-slate-700 text-fliess text-sm font-bold rounded-xl transition-all cursor-pointer"
          >
            <Plus class="w-4 h-4" />
            <span>{{ t('notes.newFolder') }}</span>
          </button>

          <BaseButton
            v-if="!currentFolderIsOwnerGroup"
            @click="handleCreateNote"
            groesse="kopf"
          >
            <Plus class="w-4 h-4" />
            <span>{{ t('notes.newNote') }}</span>
          </BaseButton>
      </div>
    </ModuleHeader>

    <!-- Breadcrumbs: bleiben auch bei geöffneter Notiz stehen – der
         Notiztitel hängt dann als letztes Glied am Pfad, ein Klick auf
         eine Mappe schließt den Editor und wechselt dorthin. -->
    <ContainerBreadcrumbs
      v-if="!showGraph && !showIndex"
      :crumbs="displayBreadcrumbs"
      :show-back="currentFolderId !== null || !!openNote"
      :tail="openNote ? (openNote.title || t('notes.untitled')) : null"
      :attached="true"
      @back="openNote ? closeNoteEditor() : goUp()"
      @navigate="navigateTo"
    >
      <template #right>
        <!-- Suche (Titel + Inhalt, Mappen-übergreifend); nur im Explorer -->
        <div v-if="!openNote" class="relative shrink-0 ml-3 w-40 sm:w-64">
          <Search class="w-4 h-4 text-slate-400 absolute left-3 top-1/2 -translate-y-1/2 pointer-events-none" />
          <input
            v-model="searchQuery"
            type="search"
            :placeholder="t('notes.searchPlaceholder')"
            class="w-full pl-9 pr-8 py-2 text-sm font-medium bg-auflage text-schrift placeholder:text-slate-400 rounded-xl border-none focus:outline-none focus:ring-2 focus:ring-slate-300 dark:focus:ring-slate-600 transition-all [&::-webkit-search-cancel-button]:hidden"
          />
          <button v-if="searchQuery" @click="clearSearch" class="absolute right-2 top-1/2 -translate-y-1/2 p-0.5 text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 rounded-xl cursor-pointer" :title="t('common.close')">
            <X class="w-4 h-4" />
          </button>
        </div>

        <!-- Nur sinnvoll, wenn man beim Suchen überhaupt in einer Mappe steht -->
        <label v-if="searchQuery && currentFolderId" class="flex items-center gap-1.5 text-xs font-bold text-leise cursor-pointer select-none shrink-0">
          <input type="checkbox" v-model="searchInFolder"
            class="w-3.5 h-3.5 rounded border-linie text-marke focus:ring-marke" />
          {{ t('notes.searchInFolder') }}
        </label>
      </template>
    </ContainerBreadcrumbs>

    <!-- Graph-Ansicht der Wikilinks (ersetzt den Explorer) -->
    <component :is="erweiterungen.graph" v-if="showGraph && erweiterungen.graph" :load-graph="api.getGraph" @open="openFromGraph" @tag="filterByTagFromGraph" />

    <!-- Übersicht/Index: Titel A–Z + Tag-Baum (ersetzt den Explorer) -->
    <NotesIndex v-else-if="showIndex" :titles="noteTitles" :tags="allTags" @open="openFromIndex" @tag="filterByTagFromIndex" />

    <!-- Editor-Ansicht (ersetzt den Explorer, solange eine Notiz offen ist) -->
    <div v-else-if="openNote" class="karte rounded-t-none -mt-1 sm:-mt-6 shadow-sm flex flex-col overflow-hidden h-[calc(100vh-10rem)] min-h-[24rem]">
      <div class="h-14 border-b border-linie px-4 sm:px-6 flex items-center justify-between bg-slate-50/50 dark:bg-slate-950/50 gap-3 shrink-0">
        <button @click="closeNoteEditor" :title="t('notes.back')" class="p-1.5 text-leise hover:bg-auflage rounded-xl transition-colors shrink-0">
          <ArrowLeft class="w-4 h-4" />
        </button>
        <input v-model="openNote.title" type="text" :readonly="noteReadOnly" :placeholder="t('notes.noteInput')"
          class="bg-transparent border-none text-lg font-bold text-schrift placeholder:text-slate-400 focus:outline-none focus:ring-0 w-full" />
        <!-- Gespeichert? AUF DEM TELEFON HIER, am Schreibtisch in der Leiste des
             Editors. Die Leiste steht unterhalb von `md` nur bei offener
             Tastatur da; war sie zu, sah man nirgends, ob der Text angekommen
             ist (am 15.09.2026 auf dem Tablet gefragt). Nur das Zeichen – der
             Platz neben dem Titel ist knapp, der Text steht im Tooltip. -->
        <span class="md:hidden shrink-0 flex items-center" aria-live="polite">
          <Loader2 v-if="saveStatus === 'saving'" class="w-4 h-4 animate-spin text-marke" :title="t('editor.saving')" />
          <Check v-else-if="saveStatus === 'saved'" class="w-4 h-4 text-emerald-500" :title="t('editor.saved')" />
          <button v-else-if="saveStatus === 'error'" type="button" @click="retrySave" :title="saveError || t('editor.saveError')"
            class="p-1 text-rose-500"><AlertCircle class="w-4 h-4" /></button>
        </span>
        <div class="flex items-center gap-2 shrink-0">
            <button @click="editorRef?.undo()" :title="t('notes.undo')"
              class="p-1.5 text-slate-400 hover:text-marke hover:bg-marke-leise rounded-xl transition-colors"><Undo class="w-4 h-4" /></button>
            <button @click="editorRef?.redo()" :title="t('notes.redo')"
              class="p-1.5 text-slate-400 hover:text-marke hover:bg-marke-leise rounded-xl transition-colors"><Redo class="w-4 h-4" /></button>
            <button @click="moveNoteModal = { note: openNote }" :title="t('notes.moveToFolder')"
              class="p-1.5 text-slate-400 hover:text-marke hover:bg-marke-leise rounded-xl transition-colors"><FolderInput class="w-4 h-4" /></button>
            <button @click="exportNoteMd(openNote)" :title="t('notes.downloadMd')"
              class="p-1.5 text-slate-400 hover:text-marke hover:bg-marke-leise rounded-xl transition-colors"><IconMd class="w-4 h-4" /></button>
            <button v-if="api.exportNotePdf" @click="exportNotePdf(openNote)" :disabled="exportingPdfId === openNote.id" :title="t('notes.downloadPdf')"
              class="p-1.5 text-slate-400 hover:text-marke hover:bg-marke-leise rounded-xl transition-colors disabled:opacity-50">
              <Loader2 v-if="exportingPdfId === openNote.id" class="w-4 h-4 animate-spin" />
              <IconPdf v-else class="w-4 h-4" />
            </button>
            <button @click="handleDeleteNote(openNote)" :title="t('notes.moveToTrash')"
              class="p-1.5 text-slate-400 hover:text-rose-500 hover:bg-rose-50 dark:hover:bg-rose-500/10 rounded-xl transition-colors"><Trash2 class="w-4 h-4" /></button>
        </div>
      </div>

      <!-- Von außen geändert (Zettlr, Sync-Client): Der Server hat das
           Überschreiben verweigert. Erst neu laden, dann weiterschreiben. -->
      <div v-if="externalChange" class="mx-4 mb-3 p-3 rounded-xl border border-amber-300 dark:border-amber-700/60 bg-amber-50 dark:bg-amber-900/20 flex items-start gap-3">
        <AlertTriangle class="w-5 h-5 shrink-0 text-amber-600 dark:text-amber-400 mt-0.5" />
        <div class="min-w-0 flex-1">
          <p class="text-sm font-bold text-amber-800 dark:text-amber-300">{{ t('notes.externalChangeTitle') }}</p>
          <p class="text-xs text-amber-700 dark:text-amber-400 mt-0.5">{{ saveError || t('notes.externalChangeHint') }}</p>
        </div>
        <button @click="reloadAfterExternalChange"
          class="shrink-0 px-3 py-1.5 text-xs font-bold text-white bg-amber-600 hover:bg-amber-700 rounded-lg transition-colors">
          {{ t('notes.reloadNote') }}
        </button>
      </div>

      <!-- WYSIWYG-Markdown-Editor (Quelle = Markdown) -->
      <MarkdownEditor
        ref="editorRef"
        :key="openNote.id"
        v-model="openNote.content"
        :read-only="noteReadOnly"
        :save-status="saveStatus"
        :save-error="saveError"
        :wikilinks="true"
        :wikilink-suggestions="noteTitles"
        :tag-suggestions="allTags"
        :asset-resolver="assetResolver"
        :citations="noteCitations"
        @wikilink="onWikilink"
        :einfuegbar="Boolean(erweiterungen.einfuegen)"
        @insert="insertDialogOpen = true"
        @paste-image="onPasteImage"
        @asset-open="onAssetOpen"
        @retry="retrySave"
        class="flex-1 min-h-0"
      />

      <!-- KI des Projekts: nur, wenn die Datenquelle eine anbietet. -->
      <component :is="erweiterungen.ki" v-if="api.ai && erweiterungen.ki" :ai="api.ai" :note-id="openNote.id" :read-only="noteReadOnly" @insert="insertAiAnswer" />

      <!-- Beziehungen UNTER der Notiz: „Verlinkt von" (wer verlinkt per
           [[Wikilink]] hierher?) und die #Tags der Notiz. Beide eingeklappt,
           damit die Chip-Listen wenig Platz brauchen. -->
      <div v-if="backlinks.length || noteTags.length" class="px-4 sm:px-6 py-2 border-t border-linie flex items-start gap-x-5 gap-y-2 flex-wrap shrink-0">
        <div v-if="backlinks.length" class="flex items-center gap-2 flex-wrap">
          <button @click="backlinksOpen = !backlinksOpen"
            class="flex items-center gap-1 text-xs font-bold text-slate-400 dark:text-slate-500 uppercase tracking-wider hover:text-slate-600 dark:hover:text-slate-300 transition-colors cursor-pointer">
            <ChevronRight class="w-3.5 h-3.5 transition-transform" :class="backlinksOpen ? 'rotate-90' : ''" />
            <Link2 class="w-3.5 h-3.5" /> {{ t('notes.backlinksLabel') }}
            <span class="text-slate-400 dark:text-slate-500 normal-case">({{ backlinks.length }})</span>
          </button>
          <template v-if="backlinksOpen">
            <button v-for="link in backlinks" :key="link.id" @click="openBacklink(link)"
              class="px-2.5 py-1 text-xs font-bold text-fliess bg-auflage hover:bg-slate-200 dark:hover:bg-slate-700 rounded-xl transition-colors">
              {{ link.title || t('notes.untitled') }}
            </button>
          </template>
        </div>
        <div v-if="noteTags.length" class="flex items-center gap-2 flex-wrap">
          <button @click="tagsOpen = !tagsOpen"
            class="flex items-center gap-1 text-xs font-bold text-slate-400 dark:text-slate-500 uppercase tracking-wider hover:text-slate-600 dark:hover:text-slate-300 transition-colors cursor-pointer">
            <ChevronRight class="w-3.5 h-3.5 transition-transform" :class="tagsOpen ? 'rotate-90' : ''" />
            <Tag class="w-3.5 h-3.5" /> {{ t('notes.noteTagsLabel') }}
            <span class="text-slate-400 dark:text-slate-500 normal-case">({{ noteTags.length }})</span>
          </button>
          <template v-if="tagsOpen">
            <button v-for="tg in noteTags" :key="tg" @click="filterByTagFromNote(tg)"
              class="px-2.5 py-1 text-xs font-bold text-fliess bg-auflage hover:bg-slate-200 dark:hover:bg-slate-700 rounded-xl transition-colors">
              #{{ tg }}
            </button>
          </template>
        </div>
      </div>

      <!-- Stand: Erstell- und Änderungsdatum unter der Notiz -->
      <div class="px-4 sm:px-6 py-2 border-t border-linie flex flex-wrap items-center gap-x-4 gap-y-0.5 text-xs text-slate-400 dark:text-slate-500 shrink-0">
        <span>{{ t('notes.createdAt') }}: {{ formatDateTimeShort(openNote.created_at) }}</span>
        <span>{{ t('notes.updatedAt') }}: {{ formatDateTimeShort(noteUpdatedAt) }}</span>
      </div>
    </div>

    <!-- Explorer (Tabelle wie im Dokumente-Modul, ohne Größen-Spalte) -->
    <template v-else>
      <div v-if="isLoading" class="flex justify-center py-20">
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-marke"></div>
      </div>
      <div v-else-if="errorMsg" class="karte shadow-sm py-16 text-center text-sm font-bold text-rose-500">{{ errorMsg }}</div>
      <ExplorerTable
        v-else
        class="-mt-1 sm:-mt-6"
        :columns="{ name: t('notes.columnName'), meta: t('notes.columnChanged'), actions: t('notes.columnActions') }"
        :items="currentItems"
        :mobile-actions="true"
        :attached-top="true"
        @open="openItem"
      >
        <template #icon="{ item }">
          <component
            :is="item.owner_group ? Users : (item.type === 'folder' ? Folder : FileText)"
            class="w-6 h-6 shrink-0"
            :class="item.owner_group ? 'text-slate-400 dark:text-slate-500' : (item.type === 'folder' ? 'text-marke' : 'text-amber-600 dark:text-amber-400')"
          />
        </template>

        <template #meta="{ item }">
          {{ formatDateShort(item.updated_at) }}
        </template>

        <template #actions="{ item }">
          <ActionButtons :actions="item.type === 'folder' ? folderActions(item) : noteActions(item)" />
        </template>

        <template #footer>
          <div v-if="notesNextPage" class="p-4 text-center">
            <button @click="loadMoreNotes" :disabled="loadingMore" class="px-5 py-2 text-sm font-bold text-marke hover:bg-marke-leise rounded-xl disabled:opacity-50">{{ t('notes.loadMoreNotes') }}</button>
          </div>
        </template>

        <!-- Leerzustand (Suche ohne Treffer bekommt eigenen Text) -->
        <template #empty>
          <div class="w-16 h-16 rounded-full bg-slate-50 dark:bg-slate-800 text-slate-400 flex items-center justify-center mx-auto">
            <component :is="searchQuery.trim() ? Search : FileText" class="w-8 h-8" />
          </div>
          <template v-if="searchQuery.trim()">
            <h4 class="text-lg font-extrabold text-schrift">{{ t('notes.searchEmptyTitle') }}</h4>
            <p class="text-sm text-leise">{{ t('notes.searchEmptyHint') }}</p>
          </template>
          <template v-else>
            <h4 class="text-lg font-extrabold text-schrift">{{ currentFolderId === null ? t('notes.emptyTitleRoot') : t('notes.emptyTitleFolder') }}</h4>
            <p class="text-sm text-leise">
              {{ t('notes.emptyHint') }}
            </p>
            <router-link to="/hilfe#notes" class="inline-block text-sm font-bold text-leise underline underline-offset-2 hover:text-slate-700 dark:hover:text-slate-200 transition-colors">
              {{ t('common.howItWorks') }}
            </router-link>
          </template>
        </template>
      </ExplorerTable>
    </template>

    <!-- MODAL: Mappe anlegen / umbenennen -->
    <NameModal
      v-if="folderModal"
      :key="folderModal.mode + '-' + (folderModal.id ?? 'new')"
      :title="folderModal.mode === 'create' ? t('notes.createFolderTitle') : t('notes.renameFolderTitle')"
      :label="t('notes.folderName')"
      :placeholder="t('notes.folderNamePlaceholder')"
      :submit-label="folderModal.mode === 'create' ? t('notes.createFolder') : t('notes.save')"
      :cancel-label="t('notes.cancel')"
      :initial="folderModal.name"
      :error="folderError"
      @submit="submitFolder"
      @close="folderModal = null"
    />

    <!-- MODAL: Mappe löschen (Modus wählen) -->
    <div v-if="deleteFolderModal" class="fixed inset-0 bg-slate-900/60 backdrop-blur-sm flex items-center justify-center p-4 z-[100]" @click.self="deleteFolderModal = null">
      <div class="bg-flaeche w-full max-w-md rounded-xl shadow-2xl p-6 space-y-4 border border-linie">
        <h3 class="text-lg font-extrabold text-schrift">{{ t('notes.deleteFolderTitle', { name: deleteFolderModal.folder.name }) }}</h3>
        <p class="text-sm text-leise">{{ t('notes.deleteFolderQuestion') }}</p>
        <div class="space-y-2">
          <button @click="confirmDeleteFolder('move_to_parent')" class="w-full text-left p-4 rounded-xl border border-linie hover:bg-auflage transition-colors">
            <div class="font-bold text-slate-800 dark:text-slate-100">{{ t('notes.keepNotesTitle') }}</div>
            <div class="text-xs text-leise">{{ t('notes.keepNotesHint') }}</div>
          </button>
          <button @click="confirmDeleteFolder('delete_notes')" class="w-full text-left p-4 rounded-xl border border-rose-200 dark:border-rose-900 hover:bg-rose-50 dark:hover:bg-rose-900/20 transition-colors">
            <div class="font-bold text-rose-600 dark:text-rose-400">{{ t('notes.deleteNotesTitle') }}</div>
            <div class="text-xs text-rose-500/80">{{ t('notes.deleteNotesHint') }}</div>
          </button>
        </div>
        <div class="flex justify-end"><button @click="deleteFolderModal = null" class="px-4 py-2 rounded-xl font-bold text-fliess hover:bg-auflage">{{ t('notes.cancel') }}</button></div>
      </div>
    </div>

    <!-- MODAL: Mappe in Projekt freigeben -->
    <component
      :is="erweiterungen.teilen"
      v-if="shareModal && erweiterungen.teilen"
      type="note_folder"
      :item-id="shareModal.folder.id"
      :item-name="shareModal.folder.name"
      @close="shareModal = null"
    />

    <!-- MODAL: Bibliothek (CSL-JSON) mit Mappe verknüpfen -->
    <div v-if="libraryModal" class="fixed inset-0 bg-slate-900/60 backdrop-blur-sm flex items-center justify-center p-4 z-[100]" @click.self="libraryModal = null">
      <div class="bg-flaeche w-full max-w-md rounded-xl shadow-2xl p-6 space-y-4 border border-linie">
        <div class="flex items-center justify-between border-b border-linie pb-4">
          <div class="flex items-center gap-3">
            <BookMarked class="w-6 h-6 text-leise" />
            <h3 class="font-extrabold text-xl text-schrift truncate">{{ t('notes.libraryTitle', { name: libraryModal.folder.name }) }}</h3>
          </div>
          <button @click="libraryModal = null" class="p-1.5 text-slate-400 hover:bg-auflage rounded-xl transition-colors shrink-0">
            <X class="w-5 h-5" />
          </button>
        </div>
        <p class="text-sm text-leise">{{ t('notes.libraryHint') }}</p>
        <div class="max-h-72 overflow-y-auto space-y-1">
          <button
            @click="setLibrary(null)"
            class="w-full text-left px-3 py-2.5 rounded-xl hover:bg-auflage flex items-center gap-2 text-sm font-semibold text-fliess"
          >
            <X class="w-4 h-4 text-slate-400" /> {{ t('notes.libraryNone') }}
          </button>
          <button
            v-for="f in libraryModal.candidates"
            :key="f.id"
            @click="setLibrary(f.id)"
            class="w-full text-left px-3 py-2.5 rounded-xl hover:bg-auflage flex items-center gap-2 text-sm font-semibold text-fliess truncate"
          >
            <FileText class="w-4 h-4 text-slate-400 shrink-0" /> {{ f.name }}
          </button>
          <p v-if="libraryModal.candidates.length === 0" class="px-3 py-2 text-xs text-slate-400">{{ t('notes.libraryEmpty') }}</p>
        </div>
      </div>
    </div>

    <!-- MODAL: Notiz in Mappe verschieben -->
    <MoveTargetModal
      v-if="moveNoteModal"
      :title="t('notes.moveNoteTitle', { name: moveNoteModal.note.title || t('notes.untitled') })"
      :options="folderOptions"
      :current-id="moveNoteModal.note.note_folder_id ?? null"
      :top-label="t('notes.topLevel')"
      :empty-label="t('notes.noFoldersYet')"
      @select="(id) => moveNote(moveNoteModal.note, id)"
      @close="moveNoteModal = null"
    />

    <!-- Einfüge-Dialog: openany-Inhalt verknüpfen oder hochladen -->
    <component
      :is="erweiterungen.einfuegen"
      v-if="insertDialogOpen && erweiterungen.einfuegen"
      :folder-id="noteContextFolderId"
      @insert="applyInsert"
      @close="insertDialogOpen = false"
    />
  </component>
</template>
