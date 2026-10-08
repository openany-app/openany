<script setup>
// „Freigaben"-Reiter eines Projekts: Landeliste aller in dieses Projekt
// freigegebenen Behälter (Notiz-Mappe, Album, Datei-Ordner/Akte). Ein Klick
// öffnet den passenden Betrachter, verwurzelt an der Freigabe. Die
// Freigabe-Daten liefert der schon vorhandene, typ-generische /shares-Endpoint.
import { computed, ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from './umgebung';
import { createProjectNotesSource, notizErweiterungen } from './umgebung';
import NotesWorkspace from '@oberflaeche/notizen/NotesWorkspace.vue';
import ProjectAlbumView from './ProjectAlbumView.vue';
import ProjectFileView from './ProjectFileView.vue';
import ProjectPackage from './ProjectPackage.vue';
import {
  FileText, Image as ImageIcon, Folder, Archive, Eye, Pencil, Share2, Trash2,
} from 'lucide-vue-next';
import { useToast } from '@oberflaeche/composables/useToast';
import { useConfirm } from '@oberflaeche/composables/useConfirm';

const props = defineProps({
  projectId: { type: [Number, String], required: true },
  meId: { type: [Number, String], default: null },      // eigene User-Id
  isOwner: { type: Boolean, default: false },            // Projekt-Owner?
  // Ziel eines [[Verweises]] aus dem Chat: { kind, id, share_root_id, … }.
  // Gesetzt heißt: nicht die Liste zeigen, sondern gleich dorthin springen.
  openTarget: { type: Object, default: null },
  projectName: { type: String, default: '' },
});
const { t } = useI18n();
const toast = useToast();
const { confirmDialog } = useConfirm();

const shares = ref([]);
const isLoading = ref(true);
const active = ref(null); // ausgewählte Freigabe oder null (Liste)

const notesSource = createProjectNotesSource(api, props.projectId);

// Der Paket-Rundlauf greift auf die freigegebenen NOTIZ-Mappen zu: ohne eine
// solche gibt es nichts zu exportieren, und ohne Schreibrecht darauf lehnt
// das Backend das Zurückspielen ohnehin ab.
const noteShares = computed(() => shares.value.filter((s) => s.type === 'note_folder' && !s.trashed));
const canWritePackage = computed(() => noteShares.value.every((s) => s.permission === 'edit'));

// Sprungziel aus dem Chat: Ebene und Element, an denen die jeweilige Ansicht
// starten soll. Nur beim Aufbau gesetzt – ProjectDetail erzwingt für jeden
// Klick einen Remount, damit auch ein zweiter Sprung greift.
const startFolderId = ref(null);    // Notizen: Mappe
const startNoteId = ref(null);
const startFileId = ref(null);      // Dateien: Ordner, dessen Inhalt zu zeigen ist
const highlightFileId = ref(null);  // … und die Zeile, um die es geht
const startAlbumId = ref(null);     // Bilder: Album
const startPhotoId = ref(null);     // … und das Bild, das aufgehen soll

const zurueckAufListe = () => {
  startFolderId.value = null;
  startNoteId.value = null;
  startFileId.value = null;
  highlightFileId.value = null;
  startAlbumId.value = null;
  startPhotoId.value = null;
};

const load = async () => {
  isLoading.value = true;
  try {
    shares.value = (await api.getProjectShares(props.projectId)).data;
    if (props.openTarget) springeZumZiel(props.openTarget);
  } finally {
    isLoading.value = false;
  }
};
onMounted(load);

/** Art des Ziels => Art der Freigabe, über die es erreichbar ist. */
const FREIGABE_ART = {
  note: 'note_folder',
  // Die MAPPE selbst als Ziel, ohne eine bestimmte Notiz darin: Das „Heft ›"
  // der Tagesansicht (Stufe 4b) führt an ein Fach-Heft, nicht an eine Seite.
  note_folder: 'note_folder',
  folder: 'file_folder',
  file: 'file_folder',
  album: 'album',
  photo: 'album',
};

/**
 * Die Freigabe öffnen, über die das Ziel erreichbar ist. Der Chat liefert die
 * Freigabe-Wurzel mit, weil hier nur die Wurzeln bekannt sind und nicht der
 * Baum darunter – eine Datei drei Ebenen tiefer wäre sonst nicht zuzuordnen.
 */
const springeZumZiel = (ziel) => {
  const share = shares.value.find(
    (s) => s.type === FREIGABE_ART[ziel.kind] && !s.trashed && s.shareable_id === ziel.share_root_id,
  );
  if (!share) {
    // Freigabe seit dem Schreiben der Nachricht entfernt: Liste zeigen
    // statt ins Leere springen.
    toast.error(t('projects.chat.noteGone'));
    return;
  }

  if (ziel.kind === 'note' || ziel.kind === 'note_folder') {
    startFolderId.value = ziel.kind === 'note' ? ziel.folder_id : ziel.id;
    startNoteId.value = ziel.kind === 'note' ? ziel.id : null;
  } else if (ziel.kind === 'file' || ziel.kind === 'folder') {
    // Die Ansicht zeigt Ordnerinhalte: Bei einer Datei ist das Ziel ihr
    // Ordner, hervorgehoben wird dann die Zeile.
    startFileId.value = ziel.folder_id;
    highlightFileId.value = ziel.kind === 'file' ? ziel.id : null;
  } else {
    startAlbumId.value = ziel.kind === 'photo' ? ziel.album_id : ziel.id;
    startPhotoId.value = ziel.kind === 'photo' ? ziel.id : null;
  }

  active.value = share;
};

const isFile = (s) => s.type === 'file_folder';
const isDocuments = (s) => isFile(s) && s.zone === 'documents';
const typeLabel = (s) => {
  if (s.type === 'album') return t('shares.projectShares.typeAlbum');
  if (s.type === 'note_folder') return t('shares.projectShares.typeNoteFolder');
  return isDocuments(s) ? t('shares.projectShares.typeFile') : t('shares.projectShares.typeFolder');
};
const typeIcon = (s) => {
  if (s.type === 'album') return ImageIcon;
  if (s.type === 'note_folder') return FileText;
  return isDocuments(s) ? Archive : Folder;
};
const typeColor = (s) => {
  if (s.type === 'album') return 'text-sky-600 dark:text-sky-400';
  if (s.type === 'note_folder') return 'text-marke';
  return 'text-amber-600 dark:text-amber-400';
};

const openShare = (s) => {
  if (s.trashed) return;
  // Ein Klick in der Liste meint die Freigabe selbst – ein zuvor gesetztes
  // Sprungziel aus dem Chat darf hier nicht nachwirken.
  zurueckAufListe();
  active.value = s;
};
const back = () => { active.value = null; };

// Entfernen darf: der Projekt-Owner (jede Freigabe seines Projekts, z. B. die
// des KI-Users), der Ressourcen-Eigentümer und wer die Freigabe angelegt hat.
const canRemove = (s) => props.isOwner
  || (props.meId != null && (s.owner_id === props.meId || s.shared_by?.id === props.meId));

const removeShare = async (s) => {
  const ok = await confirmDialog(t('shares.projectShares.confirmRemove', { name: s.name }), {
    title: t('shares.projectShares.removeTitle'),
    confirmLabel: t('shares.projectShares.removeConfirmLabel'),
  });
  if (!ok) return;
  try {
    await api.deleteProjectShare(props.projectId, s.id);
    if (active.value?.id === s.id) active.value = null;
    load();
  } catch (e) {
    toast.error(t('shares.projectShares.removeFailed'));
  }
};
</script>

<template>
  <!-- Detailansicht einer geöffneten Freigabe. Zurück zur Liste: erneut auf
       den „Freigaben"-Reiter klicken (setzt die Ansicht zurück); bei
       Alben/Dateien führt zusätzlich der Breadcrumb-Zurück-Knopf auf der
       Wurzelebene hierher zurück. -->
  <div v-if="active">
    <!-- key: Wechselt man ohne Umweg über die Liste auf eine ANDERE Mappe,
         muss die Arbeitsfläche neu starten – die Startebene wirkt nur beim
         Aufbau. -->
    <NotesWorkspace
      v-if="active.type === 'note_folder'"
      :key="active.id"
      :data-source="notesSource"
      :erweiterungen="notizErweiterungen()"
      :initial-folder-id="startFolderId ?? active.shareable_id"
      :initial-note-id="startNoteId"
      embedded
    />
    <ProjectAlbumView
      v-else-if="active.type === 'album'"
      :project-id="projectId"
      :album-id="active.shareable_id"
      :start-album-id="startAlbumId"
      :start-photo-id="startPhotoId"
      @back="back"
    />
    <ProjectFileView
      v-else
      :project-id="projectId"
      :root-id="active.shareable_id"
      :zone="active.zone || 'files'"
      :start-folder-id="startFileId"
      :highlight-id="highlightFileId"
      @back="back"
    />
  </div>

  <!-- Landeliste -->
  <div v-else class="space-y-1 sm:space-y-6">
    <!-- Rundlauf über Zettlr: herunterladen, dort arbeiten, zurückspielen.
         Nur sichtbar, wenn überhaupt eine Notiz-Mappe freigegeben ist. -->
    <div v-if="!isLoading && noteShares.length" class="flex justify-end">
      <ProjectPackage
        :project-id="projectId"
        :project-name="projectName"
        :can-write="canWritePackage"
      />
    </div>

    <div v-if="isLoading" class="flex justify-center py-20">
      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-marke"></div>
    </div>

    <div v-else-if="!shares.length" class="py-20 text-center space-y-3">
      <div class="w-16 h-16 rounded-full bg-slate-50 dark:bg-slate-800 text-slate-400 flex items-center justify-center mx-auto">
        <Share2 class="w-8 h-8" />
      </div>
      <h4 class="text-lg font-extrabold text-schrift">{{ t('shares.projectShares.emptyTitle') }}</h4>
      <p class="text-sm text-leise max-w-sm mx-auto">{{ t('shares.projectShares.emptyHint') }}</p>
    </div>

    <div v-else class="karte shadow-sm overflow-hidden divide-y divide-slate-100 dark:divide-slate-800/50">
      <div
        v-for="s in shares" :key="s.id"
        class="flex items-center gap-3 sm:gap-4 px-3 sm:px-6 py-3 sm:py-4 hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors group"
        :class="s.trashed ? 'opacity-60' : ''"
      >
        <button
          @click="openShare(s)"
          :disabled="s.trashed"
          class="flex items-center gap-3 sm:gap-4 min-w-0 flex-1 text-left disabled:cursor-not-allowed cursor-pointer"
        >
          <component :is="typeIcon(s)" class="w-6 h-6 shrink-0" :class="typeColor(s)" />
          <div class="min-w-0 flex-1">
            <div class="text-sm font-extrabold text-schrift truncate group-hover:text-marke transition-colors">{{ s.name }}</div>
            <div class="text-xs font-medium text-slate-400 truncate">
              {{ typeLabel(s) }} · {{ t('shares.projectShares.sharedBy', { name: s.shared_by?.name || '?' }) }}
            </div>
          </div>
        </button>
        <span
          class="shrink-0 hidden sm:inline-flex items-center gap-1 px-2 py-1 rounded-lg text-[11px] font-bold"
          :class="s.permission === 'edit' ? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-400' : 'bg-slate-100 text-slate-500 dark:bg-slate-800 dark:text-slate-400'"
        >
          <component :is="s.permission === 'edit' ? Pencil : Eye" class="w-3 h-3" />
          {{ s.permission === 'edit' ? t('shares.projectShares.edit') : t('shares.projectShares.read') }}
        </span>
        <span v-if="s.trashed" class="shrink-0 text-[11px] font-medium text-slate-400">{{ t('shares.projectShares.trashed') }}</span>
        <button
          v-if="canRemove(s)"
          @click="removeShare(s)"
          :title="t('shares.projectShares.removeShare')"
          class="shrink-0 p-1.5 text-slate-400 hover:text-rose-500 hover:bg-rose-50 dark:hover:bg-rose-500/10 rounded-xl transition-colors cursor-pointer"
        >
          <Trash2 class="w-4 h-4" />
        </button>
      </div>
    </div>
  </div>
</template>
