<script setup>
// Fotos in ein anderes Album legen: aus der großen Ansicht einzeln, aus der
// Auswahl im Raster mehrere.
import { computed, onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import MoveTargetModal from '../container/MoveTargetModal.vue';
import { folderTreeOptions } from '../shared/folderTree';
import { useToast } from '../composables/useToast';

const props = defineProps({
  bilder: { type: Array, required: true },
  quelle: { type: Object, required: true },
  aktuell: { type: [Number, String], default: null }, // das Album, in dem sie liegen
});
const emit = defineEmits(['close', 'verschoben']);

const { t } = useI18n();
const toast = useToast();

const baum = ref(null);
const optionen = computed(() => folderTreeOptions(baum.value ?? []));
const titel = computed(() => (props.bilder.length === 1
  ? t('albums.detail.moveAlbumTitle', { name: props.bilder[0].name })
  : t('albums.detail.moveImagesTitle', { count: props.bilder.length })));

onMounted(async () => {
  try {
    baum.value = (await props.quelle.getAlbumTree()).data.albums;
  } catch {
    toast.error(t('albums.detail.moveImageFailed'));
    emit('close');
  }
});

const legen = async (album) => {
  try {
    for (const bild of props.bilder) await props.quelle.moveMedia(bild.id, album);
  } catch {
    toast.error(t('albums.detail.moveImageFailed'));
  }
  emit('verschoben');
};
</script>

<template>
  <MoveTargetModal
    v-if="baum"
    :title="titel"
    :options="optionen"
    :current-id="aktuell"
    :top-label="t('albums.detail.topLevel')"
    @select="legen"
    @close="emit('close')"
  />
</template>
