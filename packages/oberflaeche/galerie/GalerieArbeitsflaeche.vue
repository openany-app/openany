<script setup>
/*
 * Die Galerie — Alben-Kacheln und Bilder ohne Album, darin eingebettet die
 * Albumansicht. Dieselbe Arbeitsfläche für Webapp und Programm (bis
 * 15.09.2026: frontend/src/views/Albums.vue und AlbumDetail.vue).
 *
 * dataSource (Namen wie der api-Service der Webapp):
 *   getAlbums(seite)                  → { data: { items, next_page } }
 *   getAlbum(id)                      → { data: { id, name, description, media, children, breadcrumb, depth } }
 *   createAlbum({ name, description, parent_id })
 *   updateAlbum(id, { parent_id })
 *   getAlbumTree()                    → { data: { albums } }
 *   deleteAlbum(id)
 *   uploadMedia(albumId, datei, { fortschritt })
 *   deleteMedia(albumId, bildId)
 *   getGalleryMedia(seite)            → { data: { items, next_page } }
 *   uploadGalleryMedia(datei, { fortschritt })
 *   deleteGalleryMedia(bildId)
 *   bildUrl(bild, albumId|null)       Adresse des Originals
 *   vorschauUrl(bild, albumId|null)   Adresse der Vorschau
 *   albumTitelbildUrl(album)          Adresse des Titelbilds oder null
 *   bildHerunterladen(bild, albumId)  optional
 *   albumHerunterladen(album)         optional — ZIP
 *   bildVorbereiten(bild)             optional — vor dem Zeigen des Originals
 *
 * `teilen`: Kachel-Aktion „In Projekt freigeben" zeigen; die Arbeitsfläche
 * meldet dann `teilen` mit dem Album, den Dialog stellt die Anwendung.
 */
import { ref, computed, onMounted, watch } from 'vue';
import { istVideo } from './videoStandbild';
import { useI18n } from 'vue-i18n';
import BaseModal from '../base/BaseModal.vue';
import KameraKnopf from '../base/KameraKnopf.vue';
import BaseButton from '../base/BaseButton.vue';
import ModuleHeader from '../base/ModuleHeader.vue';
import MoveTargetModal from '../container/MoveTargetModal.vue';
import AlbumCard from './AlbumCard.vue';
import MediaGrid from './MediaGrid.vue';
import AlbumAnsicht from './AlbumAnsicht.vue';
import { folderTreeOptions } from '../shared/folderTree';
import { Image as ImageIcon, Camera, Loader2, Plus, UploadCloud } from 'lucide-vue-next';
import { useToast } from '../composables/useToast';
import { useConfirm } from '../composables/useConfirm';

const props = defineProps({
  dataSource: { type: Object, required: true },
  // Ein Album, das beim Öffnen gleich aufgeschlagen ist (Webapp: ?album=).
  startAlbum: { type: [Number, String], default: null },
  teilen: { type: Boolean, default: false },
  // (album, neuLaden) => weitere Kachel-Aktionen (Programm: „behalten").
  albumAktionen: { type: Function, default: null },
  // Im Programm: „Hinzufügen" statt „Hochladen" — das Bild geht auf dieses Gerät.
  lokal: { type: Boolean, default: false },
});
const emit = defineEmits(['album', 'teilen']);

const { t } = useI18n();
const toast = useToast();
const { confirmDialog, confirmDelete } = useConfirm();
const quelle = props.dataSource;
const hinzufuegenText = computed(() => (props.lokal ? t('albums.detail.lokal.upload') : t('albums.detail.upload')));

const albums = ref([]);
const isLoading = ref(true);
const errorMessage = ref('');

const currentAlbumId = ref(props.startAlbum);
const openAlbum = (id) => { currentAlbumId.value = id; };
const closeAlbum = () => { currentAlbumId.value = null; fetchRoot(); };
watch(currentAlbumId, (id) => emit('album', id));

const albumsNextPage = ref(null);
const galleryNextPage = ref(null);
const loadingMore = ref(false);

const isCreateModalOpen = ref(false);
const isSubmitting = ref(false);
const newAlbumForm = ref({ name: '', description: '' });

const fehlertext = (err, sonst) => err?.response?.data?.message || (typeof err === 'string' ? err : '') || sonst;

const fetchAlbums = async ({ reset = true } = {}) => {
  const page = reset ? 1 : albumsNextPage.value;
  if (page == null) return;
  if (reset) isLoading.value = true;
  errorMessage.value = '';
  try {
    const response = await quelle.getAlbums(page);
    albums.value = reset ? response.data.items : [...albums.value, ...response.data.items];
    albumsNextPage.value = response.data.next_page;
  } catch (err) {
    console.error('Failed to load albums:', err);
    errorMessage.value = t('albums.albums.loadFailed');
  } finally {
    isLoading.value = false;
  }
};
const loadMoreAlbums = async () => {
  loadingMore.value = true;
  try { await fetchAlbums({ reset: false }); } finally { loadingMore.value = false; }
};

const rootImages = ref([]);
const fetchRootImages = async ({ reset = true } = {}) => {
  const page = reset ? 1 : galleryNextPage.value;
  if (page == null) return;
  try {
    const res = await quelle.getGalleryMedia(page);
    rootImages.value = reset ? res.data.items : [...rootImages.value, ...res.data.items];
    galleryNextPage.value = res.data.next_page;
  } catch (err) {
    console.error('Failed to load gallery media:', err);
  }
};
const loadMoreRootImages = async () => {
  loadingMore.value = true;
  try { await fetchRootImages({ reset: false }); } finally { loadingMore.value = false; }
};

const fetchRoot = () => Promise.all([fetchAlbums(), fetchRootImages()]);

onMounted(fetchRoot);

const rootMediaUrl = (image) => quelle.bildUrl(image, null);
const rootThumbUrl = (image) => quelle.vorschauUrl(image, null);
const downloadRootImage = (image) => quelle.bildHerunterladen?.(image, null);
const vorbereiten = quelle.bildVorbereiten ? (image) => quelle.bildVorbereiten(image) : null;
const standbildNachreichen = quelle.videoStandbildNachreichen ? (image, e) => quelle.videoStandbildNachreichen(image, e) : null;
const deleteRootImage = async (image) => {
  if (!(await confirmDelete(t('albums.detail.confirmDeleteImage')))) return false;
  try {
    await quelle.deleteGalleryMedia(image.id);
    rootImages.value = rootImages.value.filter((m) => m.id !== image.id);
    return true;
  } catch (err) {
    toast.error(t('albums.detail.deleteImageFailed'));
    return false;
  }
};

const fileInput = ref(null);
const isUploading = ref(false);
const fortschritt = ref(null);
const triggerUpload = () => fileInput.value.click();
const handleFileChange = async (event) => {
  const files = event.target.files;
  if (!files.length) return;
  isUploading.value = true;
  try {
    for (let i = 0; i < files.length; i++) {
      if (!files[i].type.startsWith('image/') && !istVideo(files[i])) {
        toast.error(t('albums.detail.skippedNonImage', { name: files[i].name }));
        continue;
      }
      fortschritt.value = null;
      const res = await quelle.uploadGalleryMedia(files[i], {
        fortschritt: (anteil) => { fortschritt.value = anteil; },
      });
      rootImages.value.unshift(res.data);
    }
  } catch (err) {
    toast.error(fehlertext(err, t('albums.detail.uploadFailed')));
  } finally {
    isUploading.value = false;
    fortschritt.value = null;
    event.target.value = null;
  }
};

const submitNewAlbum = async () => {
  if (!newAlbumForm.value.name.trim()) return;
  isSubmitting.value = true;
  try {
    const res = await quelle.createAlbum(newAlbumForm.value);
    albums.value.unshift(res.data);
    isCreateModalOpen.value = false;
    newAlbumForm.value = { name: '', description: '' };
  } catch (err) {
    console.error('Failed to create:', err);
    toast.error(t('albums.albums.createFailed'));
  } finally {
    isSubmitting.value = false;
  }
};

const downloadAlbumZip = (album) => quelle.albumHerunterladen?.(album);

const deleteAlbum = async (album) => {
  if (!(await confirmDialog(t('albums.albums.confirmMoveToTrash'), { title: t('albums.albums.moveToTrashTitle'), confirmLabel: t('albums.albums.moveAction') }))) return;
  try {
    await quelle.deleteAlbum(album.id);
    albums.value = albums.value.filter((a) => a.id !== album.id);
  } catch (err) {
    console.error('Failed to delete:', err);
    toast.error(t('albums.albums.deleteFailed'));
  }
};

const moveModal = ref(null);
const albumTree = ref([]);
const openMoveModal = async (album) => {
  try {
    albumTree.value = (await quelle.getAlbumTree()).data.albums;
    moveModal.value = { album };
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
    fetchAlbums();
  } catch (e) {
    toast.error(fehlertext(e, t('albums.detail.moveFailed')));
  }
};
</script>

<template>
  <AlbumAnsicht
    v-if="currentAlbumId"
    :album-id="currentAlbumId"
    :data-source="quelle"
    :teilen="teilen"
    :hinzufuegen-text="hinzufuegenText"
    :album-aktionen="albumAktionen"
    @open="openAlbum"
    @root="closeAlbum"
    @teilen="(a) => emit('teilen', a)"
  />

  <div v-else class="space-y-1 sm:space-y-6">
    <ModuleHeader :icon="ImageIcon" :sticky="false">
      <div class="flex items-center gap-2 sm:gap-3">
        <button
          @click="isCreateModalOpen = true"
          class="flex items-center justify-center h-9 gap-1.5 px-2.5 sm:px-4 py-2 bg-slate-100 hover:bg-slate-200 dark:bg-slate-800 dark:hover:bg-slate-700 text-fliess text-sm font-bold rounded-xl transition-all cursor-pointer"
        >
          <Plus class="w-4 h-4" />
          <span>{{ t('albums.albums.album') }}</span>
        </button>
        <input type="file" ref="fileInput" class="hidden" multiple @change="handleFileChange" />
          <KameraKnopf arten="foto-video" :disabled="isUploading" @aufgenommen="(dateien) => handleFileChange({ target: { files: dateien } })" />
        <BaseButton @click="triggerUpload" :disabled="isUploading" groesse="kopf">
          <Loader2 v-if="isUploading" class="w-4 h-4 animate-spin" />
          <UploadCloud v-else class="w-4 h-4" />
          <span>{{ hinzufuegenText }}<template v-if="isUploading && fortschritt !== null"> {{ Math.round(fortschritt * 100) }} %</template></span>
        </BaseButton>
      </div>
    </ModuleHeader>

    <div v-if="errorMessage" class="bg-rose-50 border border-rose-200 text-rose-700 p-4 rounded-xl text-sm font-medium">
      {{ errorMessage }}
    </div>

    <div v-if="isLoading" class="flex justify-center items-center py-20">
      <Loader2 class="w-10 h-10 text-marke animate-spin" />
    </div>

    <div v-else-if="albums.length === 0 && rootImages.length === 0 && !errorMessage" class="flex flex-col items-center justify-center py-20 text-center karte shadow-sm">
      <div class="w-20 h-20 rounded-full bg-auflage flex items-center justify-center text-slate-400 mb-6">
        <Camera class="w-10 h-10" />
      </div>
      <h3 class="text-xl font-extrabold text-schrift mb-2">{{ t('albums.albums.emptyTitle') }}</h3>
      <p class="text-leise max-w-sm mb-6">{{ t('albums.albums.emptyHint') }}</p>
      <BaseButton @click="isCreateModalOpen = true" groesse="gross">
        <Plus class="w-5 h-5" /> {{ t('albums.albums.createAlbum') }}
      </BaseButton>
    </div>

    <template v-else>
      <div v-if="albums.length > 0" class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3 sm:gap-6">
        <AlbumCard
          v-for="album in albums" :key="album.id"
          :album="album"
          :cover-url="quelle.albumTitelbildUrl"
          :show-download="!!quelle.albumHerunterladen"
          :zusatz-aktionen="albumAktionen ? (a) => albumAktionen(a, fetchAlbums) : null"
          :show-share="teilen"
          @open="openAlbum(album.id)"
          @download="downloadAlbumZip(album)"
          @move="openMoveModal(album)"
          @trash="deleteAlbum(album)"
          @share="emit('teilen', album)"
        />
        <div v-if="albumsNextPage" class="col-span-full text-center pt-2">
          <button @click="loadMoreAlbums" :disabled="loadingMore" class="px-5 py-2 text-sm font-bold text-marke hover:bg-marke-leise rounded-xl disabled:opacity-50">{{ t('albums.albums.loadMoreAlbums') }}</button>
        </div>
      </div>

      <div v-if="rootImages.length > 0" class="space-y-3">
        <h3 v-if="albums.length > 0" class="text-sm font-bold uppercase tracking-wider text-slate-400 dark:text-slate-500 px-1">{{ t('albums.albums.rootImages') }}</h3>
        <MediaGrid
          :images="rootImages"
          :media-url="rootMediaUrl"
          :thumb-url="rootThumbUrl"
          :download-image="downloadRootImage"
          :delete-image="deleteRootImage"
          :vorbereiten="vorbereiten"
          :standbild-nachreichen="standbildNachreichen"
          :video-oeffnen="quelle.videoOeffnen ?? null"
        />
        <div v-if="galleryNextPage" class="text-center">
          <button @click="loadMoreRootImages" :disabled="loadingMore" class="px-5 py-2 text-sm font-bold text-marke hover:bg-marke-leise rounded-xl disabled:opacity-50">{{ t('common.more') }}</button>
        </div>
      </div>
    </template>

    <BaseModal v-if="isCreateModalOpen" :title="t('albums.albums.newAlbumTitle')" @close="isCreateModalOpen = false">
      <form @submit.prevent="submitNewAlbum" class="space-y-4">
        <div>
          <label class="block text-sm font-bold text-fliess mb-1.5">{{ t('albums.albums.name') }}</label>
          <input v-model="newAlbumForm.name" type="text" required autofocus :placeholder="t('albums.albums.namePlaceholder')"
                 class="w-full bg-vertieft border border-linie rounded-xl px-4 py-2.5 text-sm focus:ring-2 focus:ring-marke focus:border-marke outline-none transition-all dark:text-white">
        </div>
        <div>
          <label class="block text-sm font-bold text-fliess mb-1.5">{{ t('albums.albums.description') }} <span class="text-slate-400 font-normal">{{ t('albums.albums.optional') }}</span></label>
          <textarea v-model="newAlbumForm.description" :placeholder="t('albums.albums.descriptionPlaceholder')" rows="3"
                    class="w-full bg-vertieft border border-linie rounded-xl px-4 py-2.5 text-sm focus:ring-2 focus:ring-marke focus:border-marke outline-none transition-all resize-none dark:text-white"></textarea>
        </div>
        <div class="pt-2 flex justify-end gap-3">
          <BaseButton variant="ghost" @click="isCreateModalOpen = false">{{ t('albums.albums.cancel') }}</BaseButton>
          <BaseButton type="submit" :loading="isSubmitting">{{ t('albums.albums.create') }}</BaseButton>
        </div>
      </form>
    </BaseModal>

    <MoveTargetModal
      v-if="moveModal"
      :title="t('albums.detail.moveAlbumTitle', { name: moveModal.album.name })"
      :options="moveOptions"
      :current-id="moveModal.album.parent_id ?? null"
      :top-label="t('albums.detail.topLevel')"
      @select="moveAlbum"
      @close="moveModal = null"
    />
  </div>
</template>
