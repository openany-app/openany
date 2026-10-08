<script setup>
/*
 * Ein Album — Unteralben, Bilder, Kopf mit „+ Album" und Hinzufügen. Dieselbe
 * Ansicht für Webapp und Programm (bis 15.09.2026: frontend/src/views/AlbumDetail.vue).
 *
 * Alles Server- bzw. Gerätespezifische kommt über `dataSource`, siehe
 * GalerieArbeitsflaeche.vue.
 */
import { ref, computed, onMounted, watch } from 'vue';
import { istVideo } from './videoStandbild';
import { useI18n } from 'vue-i18n';
import ClampText from '../base/ClampText.vue';
import BaseModal from '../base/BaseModal.vue';
import KameraKnopf from '../base/KameraKnopf.vue';
import BaseButton from '../base/BaseButton.vue';
import ContainerBreadcrumbs from '../container/ContainerBreadcrumbs.vue';
import MoveTargetModal from '../container/MoveTargetModal.vue';
import AlbumCard from './AlbumCard.vue';
import MediaGrid from './MediaGrid.vue';
import { folderTreeOptions } from '../shared/folderTree';
import { UploadCloud, Loader2, Image as ImageIcon, Info, Plus } from 'lucide-vue-next';
import { useToast } from '../composables/useToast';
import { useConfirm } from '../composables/useConfirm';

const props = defineProps({
  albumId: { type: [Number, String], required: true },
  dataSource: { type: Object, required: true },
  teilen: { type: Boolean, default: false },
  // (album, neuLaden) => weitere Kachel-Aktionen (Programm: „behalten").
  albumAktionen: { type: Function, default: null },
  hinzufuegenText: { type: String, required: true },
});
const emit = defineEmits(['open', 'root', 'teilen']);

const { t } = useI18n();
const toast = useToast();
const { confirmDialog, confirmDelete } = useConfirm();
const quelle = props.dataSource;

const album = ref(null);
const isLoading = ref(true);
const isUploading = ref(false);
const fortschritt = ref(null);
const errorMessage = ref('');

const fileInput = ref(null);
const MAX_DEPTH = 7;

const fetchAlbum = async () => {
  isLoading.value = true;
  errorMessage.value = '';
  try {
    album.value = (await quelle.getAlbum(props.albumId)).data;
  } catch (err) {
    errorMessage.value = t('albums.detail.loadFailed');
    console.error(err);
  } finally {
    isLoading.value = false;
  }
};

onMounted(fetchAlbum);
watch(() => props.albumId, () => fetchAlbum());

const crumbs = computed(() => [
  { id: null, name: t('albums.detail.allAlbums') },
  ...(album.value?.breadcrumb ?? []),
  { id: album.value?.id, name: album.value?.name ?? '' },
]);
const navigateCrumb = (id) => {
  if (id === null) emit('root');
  else if (id !== album.value?.id) emit('open', id);
};
const goUp = () => {
  const crumb = album.value?.breadcrumb ?? [];
  if (crumb.length > 0) emit('open', crumb[crumb.length - 1].id);
  else emit('root');
};

const triggerUpload = () => fileInput.value.click();
const handleFileChange = async (event) => {
  const files = event.target.files;
  if (!files.length) return;

  isUploading.value = true;
  try {
    for (let i = 0; i < files.length; i++) {
      // Der Picker filtert nicht auf Bilder (siehe Kommentar am Input).
      // Videos gehen durch (seit 26.09.2026) -- wie sie hochkommen, weiß die
      // Datenquelle (in der Webapp: in Stücken, mit Standbild).
      if (!files[i].type.startsWith('image/') && !istVideo(files[i])) {
        toast.error(t('albums.detail.skippedNonImage', { name: files[i].name }));
        continue;
      }
      fortschritt.value = null;
      const res = await quelle.uploadMedia(album.value.id, files[i], {
        fortschritt: (anteil) => { fortschritt.value = anteil; },
      });
      if (!album.value.media) album.value.media = [];
      album.value.media.push(res.data);
    }
  } catch (err) {
    console.error('Upload error:', err);
    toast.error(err?.response?.data?.message || (typeof err === 'string' ? err : '') || t('albums.detail.uploadFailed'));
  } finally {
    isUploading.value = false;
    fortschritt.value = null;
    event.target.value = null;
  }
};

// Nur auf Android relevant: der Foto-Picker schwärzt dort GPS-EXIF,
// der Weg über die Dateien-App nicht (siehe Kommentar am file-Input).
const isAndroid = /android/i.test(navigator.userAgent);

const mediaUrl = (image) => quelle.bildUrl(image, album.value.id);
const thumbUrl = (image) => quelle.vorschauUrl(image, album.value.id);
const downloadImage = (image) => quelle.bildHerunterladen?.(image, album.value.id);
const vorbereiten = quelle.bildVorbereiten ? (image) => quelle.bildVorbereiten(image) : null;
const standbildNachreichen = quelle.videoStandbildNachreichen ? (image, e) => quelle.videoStandbildNachreichen(image, e) : null;
const deleteImage = async (image) => {
  if (!(await confirmDelete(t('albums.detail.confirmDeleteImage')))) return false;
  try {
    await quelle.deleteMedia(album.value.id, image.id);
    album.value.media = album.value.media.filter((m) => m.id !== image.id);
    return true;
  } catch (err) {
    toast.error(t('albums.detail.deleteImageFailed'));
    return false;
  }
};

const downloadAlbumZip = (target) => quelle.albumHerunterladen?.(target);

const trashAlbum = async (target) => {
  if (!(await confirmDialog(t('albums.albums.confirmMoveToTrash'), { title: t('albums.albums.moveToTrashTitle'), confirmLabel: t('albums.albums.moveAction') }))) return;
  try {
    await quelle.deleteAlbum(target.id);
    album.value.children = album.value.children.filter((a) => a.id !== target.id);
  } catch (err) {
    toast.error(t('albums.albums.deleteFailed'));
  }
};

const moveModal = ref(null);
const albumTree = ref([]);
const openMoveModal = async (target) => {
  try {
    albumTree.value = (await quelle.getAlbumTree()).data.albums;
    moveModal.value = { album: target };
  } catch (e) {
    toast.error(t('albums.detail.moveFailed'));
  }
};
const moveOptions = computed(() => {
  const target = moveModal.value?.album;
  if (!target) return [];
  const excluded = new Set([target.id]);
  let grew = true;
  while (grew) {
    grew = false;
    for (const a of albumTree.value) {
      if (excluded.has(a.parent_id) && !excluded.has(a.id)) {
        excluded.add(a.id);
        grew = true;
      }
    }
  }
  return folderTreeOptions(albumTree.value.filter((a) => !excluded.has(a.id)));
});
const moveAlbum = async (parentId) => {
  try {
    await quelle.updateAlbum(moveModal.value.album.id, { parent_id: parentId });
    moveModal.value = null;
    await fetchAlbum();
  } catch (e) {
    toast.error(e?.response?.data?.message || (typeof e === 'string' ? e : '') || t('albums.detail.moveFailed'));
  }
};

const isSubAlbumModalOpen = ref(false);
const isCreatingSubAlbum = ref(false);
const subAlbumForm = ref({ name: '', description: '' });
const submitSubAlbum = async () => {
  if (!subAlbumForm.value.name.trim()) return;
  isCreatingSubAlbum.value = true;
  try {
    const res = await quelle.createAlbum({ ...subAlbumForm.value, parent_id: album.value.id });
    if (!album.value.children) album.value.children = [];
    album.value.children.push({ ...res.data, media_count: 0, cover_media_id: null });
    album.value.children.sort((a, b) => (a.name || '').localeCompare(b.name || ''));
    isSubAlbumModalOpen.value = false;
    subAlbumForm.value = { name: '', description: '' };
  } catch (err) {
    console.error('Failed to create sub-album:', err);
    toast.error(err?.response?.data?.message || (typeof err === 'string' ? err : '') || t('albums.detail.createSubAlbumFailed'));
  } finally {
    isCreatingSubAlbum.value = false;
  }
};
</script>

<template>
  <div class="space-y-1 sm:space-y-6">
    <div v-if="isLoading" class="flex justify-center py-20">
      <Loader2 class="w-10 h-10 text-marke animate-spin" />
    </div>

    <div v-else-if="errorMessage" class="bg-rose-50 border border-rose-200 text-rose-700 p-4 rounded-xl font-medium">
      {{ errorMessage }}
    </div>

    <template v-else-if="album">
      <div class="flex flex-row items-center justify-between gap-2 sm:gap-4 bg-flaeche border border-linie p-3 sm:p-6 rounded-xl shadow-sm transition-colors duration-300">
        <div class="flex items-center gap-4 min-w-0">
          <div class="min-w-0">
            <h2 class="text-lg sm:text-2xl font-extrabold text-schrift tracking-tight truncate">{{ album.name }}</h2>
            <div class="hidden sm:block text-sm text-leise font-medium"><ClampText :text="album.description || t('albums.detail.noDescription')" :lines="1" /></div>
          </div>
        </div>

        <div class="flex items-center gap-2 sm:gap-3 shrink-0">
          <button
            v-if="(album.depth ?? 1) < MAX_DEPTH"
            @click="isSubAlbumModalOpen = true"
            class="flex items-center justify-center h-9 gap-1.5 px-2.5 sm:px-4 py-2 text-sm bg-slate-100 hover:bg-slate-200 dark:bg-slate-800 dark:hover:bg-slate-700 text-fliess font-bold rounded-xl transition-all cursor-pointer"
          >
            <Plus class="w-4 h-4" />
            <span>{{ t('albums.albums.album') }}</span>
          </button>

          <!-- Bewusst OHNE accept="image/*": damit öffnet Android den vollen
               Auswahldialog (inkl. Dateien-App) statt des Foto-Pickers, der
               GPS-Daten aus den EXIF schwärzt. Nicht-Bilder filtert
               handleFileChange. -->
          <input type="file" ref="fileInput" class="hidden" multiple @change="handleFileChange" />
          <KameraKnopf arten="foto-video" :disabled="isUploading" @aufgenommen="(dateien) => handleFileChange({ target: { files: dateien } })" />
          <BaseButton @click="triggerUpload" :disabled="isUploading" groesse="kopf">
            <Loader2 v-if="isUploading" class="w-4 h-4 animate-spin" />
            <UploadCloud v-else class="w-4 h-4" />
            <span>{{ hinzufuegenText }}<template v-if="isUploading && fortschritt !== null"> {{ Math.round(fortschritt * 100) }} %</template></span>
          </BaseButton>
        </div>
      </div>

      <ContainerBreadcrumbs :crumbs="crumbs" :show-back="true" @back="goUp" @navigate="navigateCrumb" />

      <p v-if="isAndroid" class="flex items-start gap-2 text-xs text-leise px-2 -mt-2">
        <Info class="w-4 h-4 shrink-0 mt-0.5" />
        <span>{{ t('albums.detail.gpsHint') }}</span>
      </p>

      <div v-if="album.children && album.children.length > 0" class="space-y-3">
        <h3 class="text-sm font-bold uppercase tracking-wider text-slate-400 dark:text-slate-500 px-1">{{ t('albums.detail.subAlbums') }}</h3>
        <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3 sm:gap-6">
          <AlbumCard
            v-for="child in album.children" :key="child.id"
            :album="child"
            :cover-url="quelle.albumTitelbildUrl"
            :show-download="!!quelle.albumHerunterladen"
          :zusatz-aktionen="albumAktionen ? (a) => albumAktionen(a, fetchAlbum) : null"
            :show-share="teilen"
            @open="emit('open', child.id)"
            @download="downloadAlbumZip(child)"
            @move="openMoveModal(child)"
            @trash="trashAlbum(child)"
            @share="emit('teilen', child)"
          />
        </div>
      </div>

      <MediaGrid
        v-if="album.media && album.media.length > 0"
        :images="album.media"
        :media-url="mediaUrl"
        :thumb-url="thumbUrl"
        :download-image="downloadImage"
        :delete-image="deleteImage"
        :vorbereiten="vorbereiten"
        :standbild-nachreichen="standbildNachreichen"
        :video-oeffnen="quelle.videoOeffnen ?? null"
      />

      <div v-else-if="!album.children || album.children.length === 0" class="flex-1 flex flex-col items-center justify-center py-20 text-center karte shadow-sm">
        <div class="w-20 h-20 rounded-full bg-marke-leise flex items-center justify-center text-marke mb-6">
          <ImageIcon class="w-10 h-10" />
        </div>
        <h3 class="text-xl font-extrabold text-schrift mb-2">{{ t('albums.detail.noImagesTitle') }}</h3>
        <p class="text-leise max-w-sm mb-6">{{ t('albums.detail.noImagesHint') }}</p>
        <button @click="triggerUpload" class="flex items-center justify-center gap-2 px-6 py-3 bg-marke-leise hover:bg-marke-leise text-marke font-bold rounded-xl transition-all cursor-pointer">
          <UploadCloud class="w-5 h-5" /> {{ hinzufuegenText }}
        </button>
      </div>
    </template>

    <MoveTargetModal
      v-if="moveModal"
      :title="t('albums.detail.moveAlbumTitle', { name: moveModal.album.name })"
      :options="moveOptions"
      :current-id="moveModal.album.parent_id ?? null"
      :top-label="t('albums.detail.topLevel')"
      @select="moveAlbum"
      @close="moveModal = null"
    />

    <BaseModal v-if="isSubAlbumModalOpen" :title="t('albums.detail.createSubAlbumTitle')" @close="isSubAlbumModalOpen = false">
      <form @submit.prevent="submitSubAlbum" class="space-y-4">
        <div>
          <label class="block text-sm font-bold text-fliess mb-1.5">{{ t('albums.albums.name') }}</label>
          <input v-model="subAlbumForm.name" type="text" required autofocus :placeholder="t('albums.albums.namePlaceholder')"
                 class="w-full bg-vertieft border border-linie rounded-xl px-4 py-2.5 text-sm focus:ring-2 focus:ring-marke focus:border-marke outline-none transition-all dark:text-white">
        </div>
        <div>
          <label class="block text-sm font-bold text-fliess mb-1.5">{{ t('albums.albums.description') }} <span class="text-slate-400 font-normal">{{ t('albums.albums.optional') }}</span></label>
          <textarea v-model="subAlbumForm.description" :placeholder="t('albums.albums.descriptionPlaceholder')" rows="3"
                    class="w-full bg-vertieft border border-linie rounded-xl px-4 py-2.5 text-sm focus:ring-2 focus:ring-marke focus:border-marke outline-none transition-all resize-none dark:text-white"></textarea>
        </div>
        <div class="pt-2 flex justify-end gap-3">
          <BaseButton variant="ghost" @click="isSubAlbumModalOpen = false">{{ t('albums.albums.cancel') }}</BaseButton>
          <BaseButton type="submit" :loading="isCreatingSubAlbum">{{ t('albums.albums.create') }}</BaseButton>
        </div>
      </form>
    </BaseModal>
  </div>
</template>
