<script setup>
// Galerie-Ansicht eines in ein Projekt freigegebenen Albums. Ansehen +
// Herunterladen für alle; bei Freigabe-Stufe „bearbeiten" zusätzlich Fotos
// hochladen, Unteralben anlegen und Fotos löschen (der Server schreibt alles
// dem Album-Eigentümer zu). Nutzt dasselbe Foto-Raster (MediaGrid) wie die
// persönliche Galerie, ohne die volle AlbumDetail-Komponente zu berühren.
import { ref, computed, watch, nextTick, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from './umgebung';
import MediaGrid from '@oberflaeche/galerie/MediaGrid.vue';
import ContainerBreadcrumbs from '@oberflaeche/container/ContainerBreadcrumbs.vue';
import NameModal from '@oberflaeche/container/NameModal.vue';
import { Image as ImageIcon, Plus, Upload } from 'lucide-vue-next';
import { triggerDownload } from '@oberflaeche/shared/download';
import { useToast } from '@oberflaeche/composables/useToast';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import { KiFrage } from './umgebung';

const props = defineProps({
  projectId: { type: [Number, String], required: true },
  albumId: { type: [Number, String], required: true }, // Freigabe-Wurzel
  // Sprungziel aus dem Chat: Album, das gleich offen sein soll, und ein Bild
  // darin, das aufgeschlagen wird. Beides wirkt nur beim Aufbau.
  startAlbumId: { type: [Number, String], default: null },
  startPhotoId: { type: [Number, String], default: null },
});
const emit = defineEmits(['back']);

const { t } = useI18n();
const toast = useToast();
const { confirmDialog } = useConfirm();

const currentId = ref(props.startAlbumId ?? props.albumId);
const mediaGrid = ref(null);
const album = ref(null);
const isLoading = ref(false);
const isUploading = ref(false);
const fileInput = ref(null);
const isSubAlbumModalOpen = ref(false);
const subAlbumError = ref('');

const canEdit = computed(() => album.value?.access === 'edit' || album.value?.access === 'owner');

// KI des Projekts: Das Feld erscheint im Leuchtkasten nur, wenn eine
// Anbindung da ist, deren Modell Bilder liest, und das Bild eines ist, das
// die KI lesen kann (beides entscheidet der Server).
const kiAnbindung = ref(null);
const ladeKi = async () => {
  try {
    kiAnbindung.value = (await api.getProjectAi(props.projectId)).data.anbindung;
  } catch {
    kiAnbindung.value = null;
  }
};
const kiHinweis = computed(() => kiAnbindung.value
  ? t('ai.destinationPhoto', { anbieter: kiAnbindung.value.provider_name, modell: kiAnbindung.value.model })
  : '');
const kiSenden = (img) => async (auftrag) =>
  (await api.askProjectAiPhoto(props.projectId, currentId.value, img.id, auftrag)).data.anfrage;
const kiAbholen = async (anfrage) => (await api.getProjectAiRequest(props.projectId, anfrage)).data;

const load = async () => {
  isLoading.value = true;
  try {
    const res = await api.getProjectAlbum(props.projectId, currentId.value);
    album.value = res.data;
  } catch (e) {
    toast.error(t('shares.projectShares.loadFailed'));
  } finally {
    isLoading.value = false;
  }
};
watch(currentId, load);
onMounted(async () => {
  ladeKi();
  await load();
  // Erst nach dem Laden: Das Raster entsteht mit den Bildern, vorher gibt es
  // nichts aufzuschlagen.
  if (props.startPhotoId != null) {
    await nextTick();
    if (! mediaGrid.value?.openImage(props.startPhotoId)) {
      toast.error(t('projects.chat.targetNotFound'));
    }
  }
});

// Breadcrumbs kommen vom Backend an der Freigabe-Wurzel gekappt.
const crumbs = computed(() => (album.value?.breadcrumb || []).map((b) => ({ id: b.id, name: b.name })));
const navigate = (id) => { if (id != null) currentId.value = id; };
const goBack = () => {
  const bc = album.value?.breadcrumb || [];
  if (bc.length > 1) currentId.value = bc[bc.length - 2].id;
  else emit('back');
};
const openChild = (child) => { currentId.value = child.id; };

const mediaUrl = (img) => img.url;
const thumbUrl = (img) => img.thumb_url;
const downloadImage = (img) => {
  triggerDownload(`/api/projects/${props.projectId}/albums/${currentId.value}/media/${img.id}/download`, img.name);
};

// --- Bearbeiten (nur bei edit-Stufe) ---
const handleUpload = async (event) => {
  const list = event.target.files;
  if (!list || !list.length) return;
  isUploading.value = true;
  try {
    for (let i = 0; i < list.length; i++) {
      await api.uploadProjectAlbumMedia(props.projectId, currentId.value, list[i]);
    }
    load();
  } catch (err) {
    toast.error(err.response?.data?.message || t('albums.detail.uploadFailed'));
  } finally {
    isUploading.value = false;
    if (fileInput.value) fileInput.value.value = null;
  }
};
const submitSubAlbum = async (rawName) => {
  const name = rawName.trim();
  if (!name) return;
  try {
    await api.createProjectSubAlbum(props.projectId, currentId.value, name);
    isSubAlbumModalOpen.value = false;
    load();
  } catch (err) {
    subAlbumError.value = err.response?.data?.message || t('albums.albums.createFailed');
  }
};
const deleteMedia = async (img) => {
  const ok = await confirmDialog(t('albums.detail.confirmDeleteImage'), { title: t('albums.detail.delete'), confirmLabel: t('albums.detail.delete') });
  if (!ok) return false;
  try {
    await api.deleteProjectAlbumMedia(props.projectId, currentId.value, img.id);
    load();
    return true;
  } catch (err) {
    toast.error(err.response?.data?.message || t('albums.detail.deleteImageFailed'));
    return false;
  }
};
</script>

<template>
  <div class="space-y-1 sm:space-y-6">
    <!-- Bearbeiten-Leiste (nur bei edit-Stufe) -->
    <div v-if="canEdit" class="flex items-center justify-end gap-2">
      <button @click="subAlbumError = ''; isSubAlbumModalOpen = true"
        class="flex items-center justify-center h-9 gap-1.5 px-2.5 sm:px-4 py-2 bg-slate-100 hover:bg-slate-200 dark:bg-slate-800 dark:hover:bg-slate-700 text-fliess text-sm font-bold rounded-xl transition-all cursor-pointer">
        <Plus class="w-4 h-4" /> <span>{{ t('albums.detail.newSubAlbum') }}</span>
      </button>
      <BaseButton @click="$refs.fileInput.click()" :disabled="isUploading"
        groesse="kopf">
        <Upload class="w-4 h-4" /> <span>{{ t('albums.detail.upload') }}</span>
      </BaseButton>
      <input type="file" ref="fileInput" class="hidden" accept="image/*" multiple @change="handleUpload" />
    </div>

    <ContainerBreadcrumbs
      :crumbs="crumbs"
      :show-back="true"
      :attached="true"
      @back="goBack"
      @navigate="navigate"
    />

    <div class="-mt-1 sm:-mt-6 bg-flaeche border border-linie rounded-b-xl shadow-sm p-4 sm:p-6">
      <div v-if="isLoading" class="flex justify-center py-20">
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-marke"></div>
      </div>

      <template v-else-if="album">
        <!-- Unteralben -->
        <div v-if="album.children && album.children.length" class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-2 sm:gap-4 mb-6">
          <button
            v-for="child in album.children" :key="child.id"
            @click="openChild(child)"
            class="group text-left rounded-xl overflow-hidden border border-linie bg-slate-50 dark:bg-slate-800/50 hover:shadow-lg transition-all"
          >
            <div class="aspect-square bg-auflage overflow-hidden">
              <img v-if="child.cover_url" :src="child.cover_url" class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500" loading="lazy" />
              <div v-else class="w-full h-full flex items-center justify-center text-slate-300 dark:text-slate-600">
                <ImageIcon class="w-8 h-8" />
              </div>
            </div>
            <div class="px-3 py-2">
              <div class="text-sm font-bold text-slate-800 dark:text-slate-100 truncate">{{ child.name }}</div>
              <div class="text-[11px] font-medium text-slate-400">{{ t('shares.projectShares.photoCount', { count: child.media_count }) }}</div>
            </div>
          </button>
        </div>

        <!-- Fotos -->
        <MediaGrid
          v-if="album.media && album.media.length"
          ref="mediaGrid"
          :images="album.media"
          :media-url="mediaUrl"
          :thumb-url="thumbUrl"
          :download-image="downloadImage"
          :delete-image="canEdit ? deleteMedia : null"
          :readonly="!canEdit"
        >
          <template v-if="kiAnbindung?.kann_bilder" #lightbox="{ image }">
            <div v-if="image.ki_lesbar" class="w-full bg-slate-900 border-t border-slate-800 px-4 sm:px-6 pb-4 pt-3 max-h-[40vh] overflow-y-auto shrink-0">
              <div class="rounded-xl bg-grund p-3">
                <KiFrage :key="image.id" :hinweis="kiHinweis" :senden="kiSenden(image)" :abholen="kiAbholen" />
              </div>
            </div>
          </template>
        </MediaGrid>
        <div v-else-if="!album.children || !album.children.length" class="py-16 text-center text-slate-400">
          <ImageIcon class="w-10 h-10 mx-auto mb-3 opacity-60" />
          <p class="text-sm font-medium">{{ t('shares.projectShares.albumEmpty') }}</p>
        </div>
      </template>
    </div>

    <NameModal
      v-if="isSubAlbumModalOpen"
      :title="t('albums.detail.createSubAlbumTitle')"
      :label="t('albums.albums.name')"
      :placeholder="t('albums.albums.namePlaceholder')"
      :submit-label="t('albums.albums.create')"
      :cancel-label="t('albums.albums.cancel')"
      :error="subAlbumError"
      @submit="submitSubAlbum"
      @close="isSubAlbumModalOpen = false"
    />
  </div>
</template>
