<script setup>
// Album-Kachel (Galerie-Wurzel UND Unteralben in der Detailansicht):
// Cover, Name, Beschreibung, Bildanzahl. Ein Klick öffnet das Album.
//
// SEIT 16.10.2026 OHNE KNÖPFE (Tiffy, wie die Mappen bei den Notizen): Was
// man mit einem Album tun kann -- herunterladen, freigeben, verschieben,
// löschen --, steht im geöffneten Album unter „Aktionen" (AlbumAnsicht).
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import ClampText from '../base/ClampText.vue';
import { Image as ImageIcon, Pin } from 'lucide-vue-next';

const props = defineProps({
  album: { type: Object, required: true },
  // (album) => Adresse des Titelbilds oder null. Von außen, weil das Paket
  // keine Server-Adressen kennt: Die Webapp nennt /api/…, das Programm die
  // Ablage auf dem Gerät.
  coverUrl: { type: Function, required: true },
});

defineEmits(['open']);
const { t } = useI18n();

// Scheitert das Titelbild (ein Video ohne Standbild antwortet 404), lieber
// der leere Platzhalter als ein kaputtes Bild.
const titelbildFehlt = ref(false);
const thumbnail = () => (titelbildFehlt.value ? null : props.coverUrl(props.album));
const count = () => props.album.media_count ?? props.album.media?.length ?? 0;
</script>

<template>
  <div
    @click="$emit('open')"
    class="group relative karte overflow-hidden shadow-sm hover:shadow-xl hover:-translate-y-1 transition-all duration-300 cursor-pointer flex flex-col"
  >
    <!-- Thumbnail -->
    <div class="aspect-square bg-auflage relative overflow-hidden">
      <img
        v-if="thumbnail()"
        :src="thumbnail()"
        class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500"
        alt="Album Cover"
        @error="titelbildFehlt = true"
      />
      <div v-else class="absolute inset-0 flex flex-col items-center justify-center text-slate-400">
        <ImageIcon class="w-12 h-12 mb-2 opacity-50" />
        <span class="text-xs font-bold opacity-50">{{ t('albums.albums.noCover') }}</span>
      </div>
    </div>

    <!-- Info -->
    <div class="p-5">
      <h3 class="font-extrabold text-lg text-schrift truncate flex items-center gap-1.5">
        <Pin v-if="album.behalten" class="w-4 h-4 text-marke shrink-0" />
        <span class="truncate">{{ album.name }}</span>
      </h3>
      <div class="text-sm text-leise mt-1"><ClampText :text="album.description || t('albums.albums.noDescription')" :lines="2" /></div>
      <div class="mt-4 flex items-center gap-2">
        <div class="text-xs font-bold text-marke bg-marke-leise px-3 py-1 rounded-full shrink-0">
          {{ count() }} {{ count() === 1 ? t('albums.albums.imageCountSingular') : t('albums.albums.imageCountPlural') }}
        </div>
      </div>
    </div>
  </div>
</template>
