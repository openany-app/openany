<script setup>
// Bilder-Raster mit Lightbox (EXIF-Anzeige, Download, Löschen) – aus der
// Album-Detailansicht extrahiert, damit die Galerie-Wurzelebene dasselbe
// Raster nutzen kann. URL-Bau und Aktionen kommen als Props von außen.
//
// SEIT DEM 26.09.2026 AUCH VIDEOS (docs/plan-videos-galerie.md): In der
// Kachel ihr Standbild mit Play-Zeichen und Dauer -- oder, wenn das Gerät
// keines ziehen konnte, ein Film-Symbol. Im Leuchtkasten ein <video>. Kann
// der Browser es nicht abspielen (oft HEVC), sagt er das, statt schwarz zu
// bleiben.
import { ref, computed } from 'vue';
import { istVideo, dauerText, standbildAusLaufendem } from './videoStandbild';
import { nachMonaten } from './aufnahme';
import { useToast } from '@oberflaeche/composables/useToast';
import { useI18n } from 'vue-i18n';
import { formatDateTime } from '@oberflaeche/shared/date';
import {
  MapPin, Calendar as CalendarIcon, Loader2, X, ChevronLeft, ChevronRight, Download, Trash2, Play, Film,
} from 'lucide-vue-next';

const props = defineProps({
  images: { type: Array, required: true },
  mediaUrl: { type: Function, required: true },   // (image) => Original-URL
  thumbUrl: { type: Function, required: true },   // (image) => Thumbnail-URL
  downloadImage: { type: Function, default: null },  // async (image); null = kein Download
  deleteImage: { type: Function, default: null },    // async (image) => true; null = read-only
  // true: Löschen ausblenden (z. B. in eine Projekt-Freigabe eingebettet).
  readonly: { type: Boolean, default: false },
  // async (image) => void — vor dem Zeigen des Originals. Im Programm holt es
  // das Original von einem anderen Gerät, wenn hier nur die Vorschau liegt.
  vorbereiten: { type: Function, default: null },
  // async (image, { bild, dauer }) — ein Standbild für ein Video ohne eines
  // nachreichen. Aus dem Betrachter, während es läuft. null = nicht möglich.
  standbildNachreichen: { type: Function, default: null },
  // async (image) => void — ein Video statt im Leuchtkasten in einem anderen
  // Programm abspielen. Das Programm gibt es an den Player des Systems, weil
  // Videos aus der Ablage in der Webansicht nicht zuverlässig spielen.
  videoOeffnen: { type: Function, default: null },
});

const { t, locale } = useI18n();
const toast = useToast();

// Neueste zuerst nach Aufnahmedatum, in Monaten (aufnahme.js). Die Lightbox
// blättert in derselben Reihenfolge, wie das Raster sie zeigt.
const monate = computed(() => nachMonaten(props.images, locale.value));
const reihe = computed(() => monate.value.flatMap((m) => m.bilder));

const selectedImage = ref(null);
const isLightboxOpen = ref(false);
const isDownloadingImage = ref(false);

// Videos, zu denen es kein Standbild gibt (404), und Videos, die dieser
// Browser nicht abspielen kann. Je Sitzung gemerkt, nicht gespeichert.
const ohneStandbild = ref(new Set());
const nichtAbspielbar = ref(new Set());
const merken = (menge, id) => { menge.value = new Set(menge.value).add(id); };
/*
 * OHNE STANDBILD heißt zweierlei: Die Vorschau-Adresse antwortet 404 (Webapp)
 * -- oder es gibt gar keine (Programm, `vorschau_pfad` leer). Im zweiten Fall
 * meldet ein <img src=""> keinen Ladefehler; die Kachel war am 26.09.2026
 * am Tablet null Pixel hoch, ohne Film-Symbol und ohne dass das Nachholen
 * beim Ansehen je anlief.
 */
const ohneVorschau = (image) => istVideo(image)
  && (ohneStandbild.value.has(image.id) || (!lokaleStandbilder.value[image.id] && !props.thumbUrl(image)));

/*
 * NACHHOLEN BEIM ANSEHEN. Kam ein Video ohne Standbild an (älterer Browser,
 * oder die Firefox-Macke aus videoStandbild.js), nimmt der Betrachter eines
 * aus dem LAUFENDEN Video -- das Bild ist dort sicher dekodiert, weil man es
 * sieht -- und reicht es nach. Die Kachel zeigt es sofort (lokal), der
 * Server hat es beim nächsten Laden. Einmal je Video und Sitzung.
 */
const lokaleStandbilder = ref({});
const nachgereicht = new Set();
const vielleichtNachreichen = async (image, video) => {
  if (!props.standbildNachreichen || !ohneVorschau(image) || nachgereicht.has(image.id)) return;
  nachgereicht.add(image.id);
  const ergebnis = await standbildAusLaufendem(video);
  if (!ergebnis) return;
  try {
    await props.standbildNachreichen(image, ergebnis);
    lokaleStandbilder.value = { ...lokaleStandbilder.value, [image.id]: URL.createObjectURL(ergebnis.bild) };
    const rest = new Set(ohneStandbild.value);
    rest.delete(image.id);
    ohneStandbild.value = rest;
  } catch {
    // Nicht schlimm: Dann bleibt es beim Film-Symbol, und beim nächsten
    // Ansehen wird es wieder versucht.
    nachgereicht.delete(image.id);
  }
};
const dauer = (image) => {
  const d = image?.custom_properties?.dauer;
  return d ? dauerText(d) : '';
};

// Solange das Original vorbereitet wird, zeigt der Leuchtkasten die Vorschau.
const bereit = ref(true);
const vorbereitenFuer = async (image) => {
  if (!props.vorbereiten) return;
  bereit.value = false;
  try {
    await props.vorbereiten(image);
  } finally {
    if (selectedImage.value?.id === image.id) bereit.value = true;
  }
};

const openLightbox = (image) => {
  if (istVideo(image) && props.videoOeffnen) {
    // Scheitert das Holen oder Öffnen, soll man das sehen und nicht ins
    // Leere tippen.
    Promise.resolve(props.videoOeffnen(image))
      .catch((e) => toast.error(String(e?.message ?? e)));
    return;
  }
  selectedImage.value = image;
  isLightboxOpen.value = true;
  vorbereitenFuer(image);
};
const closeLightbox = () => {
  isLightboxOpen.value = false;
  selectedImage.value = null;
};
const navigateImage = (direction) => {
  const liste = reihe.value;
  if (!liste.length) return;
  const currentIndex = liste.findIndex((m) => m.id === selectedImage.value.id);
  let newIndex = currentIndex + direction;
  if (newIndex < 0) newIndex = liste.length - 1;
  if (newIndex >= liste.length) newIndex = 0;
  selectedImage.value = liste[newIndex];
  vorbereitenFuer(selectedImage.value);
};

/**
 * Ein bestimmtes Bild aufschlagen – für den Sprung aus einem [[Verweis]] im
 * Projekt-Chat. Gibt zurück, ob es in diesem Raster überhaupt vorkommt: Es
 * kann seit dem Schreiben der Nachricht gelöscht worden sein, und dann soll
 * der Aufrufer das sagen können, statt dass nichts geschieht.
 */
const openImage = (id) => {
  const bild = props.images.find((m) => m.id === id);
  if (bild) openLightbox(bild);

  return !! bild;
};
defineExpose({ openImage });

const onDownload = async (image) => {
  isDownloadingImage.value = true;
  try { await props.downloadImage(image); } finally { isDownloadingImage.value = false; }
};
const onDelete = async (image) => {
  if (await props.deleteImage(image)) closeLightbox();
};

// GPS nur werten, wenn echte Koordinaten vorliegen – (0,0) ist das
// "GPS-Feld ohne Fix"-Artefakt mancher Handys (Altbestand), kein Standort.
const hasGps = (img) => {
  const gps = img?.custom_properties?.exif?.gps;
  return !!gps && !(Number(gps.lat) === 0 && Number(gps.lng) === 0);
};
const osmUrl = (img) => {
  const { lat, lng } = img.custom_properties.exif.gps;
  return `https://www.openstreetmap.org/?mlat=${lat}&mlon=${lng}#map=16/${lat}/${lng}`;
};

const formatDate = (dateStr) => {
  if (!dateStr) return '';
  // EXIF-Datum kommt als "YYYY:MM:DD HH:MM:SS" – nur die Doppelpunkte
  // im Datumsteil ersetzen, die im Zeitteil müssen bleiben.
  const cleaned = dateStr.replace(/^(\d{4}):(\d{2}):(\d{2})/, '$1-$2-$3').replace(' ', 'T');
  // Unlesbares roh zeigen statt verschlucken – bei Fotos aus fremden Quellen
  // steht dort gelegentlich etwas, das kein Zeitstempel ist.
  return formatDateTime(cleaned, dateStr);
};
</script>

<template>
  <div>
    <!-- Je Monat eine Überschrift und darunter sein Raster. -->
    <section v-for="monat in monate" :key="monat.schluessel" class="pb-6 last:pb-10">
    <h3 v-if="monat.titel" class="sticky top-0 z-10 py-2 mb-2 text-sm font-extrabold text-schrift bg-flaeche/90 backdrop-blur-sm">{{ monat.titel }}</h3>
    <div class="columns-2 md:columns-3 lg:columns-4 gap-2 sm:gap-4 space-y-2 sm:space-y-4">
      <div
        v-for="image in monat.bilder" :key="image.id"
        @click="openLightbox(image)"
        class="break-inside-avoid relative group rounded-xl overflow-hidden cursor-pointer shadow-sm hover:shadow-xl transition-all duration-300 bg-auflage border border-slate-200/50 dark:border-slate-700/50"
      >
        <div v-if="ohneVorschau(image)"
             class="w-full aspect-video flex items-center justify-center bg-slate-800 text-slate-400">
          <Film class="w-10 h-10" />
        </div>
        <img v-else :src="lokaleStandbilder[image.id] ?? thumbUrl(image)" class="w-full h-auto object-cover group-hover:scale-105 transition-transform duration-700 ease-out" loading="lazy"
             @error="istVideo(image) && merken(ohneStandbild, image.id)" />
        <!-- Video: Play-Zeichen in der Mitte, Dauer unten rechts. -->
        <template v-if="istVideo(image)">
          <div class="absolute inset-0 flex items-center justify-center pointer-events-none">
            <span class="p-3 rounded-full bg-black/50 text-white backdrop-blur-sm"><Play class="w-6 h-6" /></span>
          </div>
          <span v-if="dauer(image)" class="absolute bottom-2 right-2 px-1.5 py-0.5 rounded-md bg-black/60 text-white text-[11px] font-bold">
            {{ dauer(image) }}
          </span>
        </template>
        <!-- Hover overlay -->
        <div class="absolute inset-0 bg-gradient-to-t from-black/60 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300 flex flex-col justify-end p-4">
          <div v-if="image.custom_properties?.exif" class="flex items-center gap-3 text-white/90">
            <span v-if="image.custom_properties.exif.date" class="flex items-center gap-1 text-[10px] font-bold"><CalendarIcon class="w-3 h-3" /> EXIF</span>
            <span v-if="hasGps(image)" class="flex items-center gap-1 text-[10px] font-bold"><MapPin class="w-3 h-3" /> GPS</span>
          </div>
        </div>
      </div>
    </div>
    </section>

    <!-- Lightbox Modal -->
    <div v-if="isLightboxOpen && selectedImage" class="fixed inset-0 z-[100] flex flex-col bg-black/95 backdrop-blur-xl animate-fade-in">

      <!-- Top controls -->
      <button @click="closeLightbox" class="absolute top-6 right-6 z-10 p-3 bg-white/10 hover:bg-white/20 text-white rounded-full backdrop-blur-md transition-colors">
        <X class="w-6 h-6" />
      </button>

      <!-- Main Image Area with Navigation -->
      <div class="flex-1 w-full flex items-center justify-center p-4 sm:p-8 relative min-h-0">
        <button @click.stop="navigateImage(-1)" class="absolute left-4 sm:left-8 p-3 bg-black/50 hover:bg-black/70 text-white rounded-full backdrop-blur-sm transition-all z-10">
          <ChevronLeft class="w-8 h-8" />
        </button>

        <template v-if="istVideo(selectedImage)">
          <div v-if="nichtAbspielbar.has(selectedImage.id)" class="max-w-md text-center text-slate-200 space-y-4">
            <Film class="w-12 h-12 mx-auto text-slate-400" />
            <p class="text-sm">{{ t('albums.detail.videoNotPlayable') }}</p>
            <button v-if="downloadImage" @click="onDownload(selectedImage)" class="inline-flex items-center gap-2 px-4 py-2 bg-marke/20 hover:bg-marke/30 text-white font-bold rounded-xl">
              <Download class="w-4 h-4" /> {{ t('albums.detail.download') }}
            </button>
          </div>
          <!-- `:key`, damit beim Blättern ein neues <video> entsteht statt
               eines, das noch das vorige abspielt. -->
          <!-- crossorigin: Im Programm kommt das Video über das Asset-Protokoll
               (andere Herkunft). Ohne die Angabe wäre das Canvas beim
               Nachholen des Standbilds gesperrt. -->
          <video v-else-if="bereit" :key="selectedImage.id" :src="mediaUrl(selectedImage)" controls autoplay playsinline crossorigin="anonymous"
                 class="max-w-full max-h-full rounded-xl drop-shadow-2xl"
                 @error="merken(nichtAbspielbar, selectedImage.id)"
                 @playing="(e) => vielleichtNachreichen(selectedImage, e.target)" />
        </template>
        <img v-else :src="bereit ? mediaUrl(selectedImage) : thumbUrl(selectedImage)" class="max-w-full max-h-full object-contain drop-shadow-2xl rounded-xl" />
        <Loader2 v-if="!bereit" class="absolute w-10 h-10 text-white animate-spin" />

        <button @click.stop="navigateImage(1)" class="absolute right-4 sm:right-8 p-3 bg-black/50 hover:bg-black/70 text-white rounded-full backdrop-blur-sm transition-all z-10">
          <ChevronRight class="w-8 h-8" />
        </button>
      </div>

      <!-- Bottom Info Area -->
      <div class="w-full bg-slate-900 border-t border-slate-800 p-4 sm:p-6 flex flex-col sm:flex-row items-center justify-between gap-4 shadow-2xl shrink-0">

        <div class="flex flex-wrap items-center gap-4 sm:gap-6 flex-1">
          <!-- EXIF Details -->
          <div v-if="selectedImage.custom_properties?.exif?.date || hasGps(selectedImage)" class="flex flex-wrap items-center gap-4 sm:gap-8">

            <div v-if="selectedImage.custom_properties.exif.date" class="flex items-center gap-3">
              <CalendarIcon class="w-5 h-5 text-marke" />
              <div>
                <div class="text-[10px] font-bold uppercase tracking-wider text-slate-400">{{ t('albums.detail.captureDate') }}</div>
                <div class="text-sm text-white font-medium">{{ formatDate(selectedImage.custom_properties.exif.date) }}</div>
              </div>
            </div>

            <div v-if="hasGps(selectedImage)" class="flex items-center gap-3">
              <MapPin class="w-5 h-5 text-marke" />
              <div>
                <div class="text-[10px] font-bold uppercase tracking-wider text-slate-400">{{ t('albums.detail.location') }}</div>
                <!-- OpenStreetMap statt Google: kein Tracking-Klick nach draußen -->
                <a :href="osmUrl(selectedImage)" target="_blank" rel="noopener" class="text-sm text-marke hover:text-marke font-medium">
                  {{ t('albums.detail.showOnMap') }}
                </a>
              </div>
            </div>

          </div>
          <div v-else class="text-sm text-slate-500 font-medium">
            {{ t('albums.detail.noExif') }}
          </div>
        </div>

        <div class="flex items-center gap-3 shrink-0">
          <button v-if="downloadImage" @click="onDownload(selectedImage)" :disabled="isDownloadingImage" class="shrink-0 flex items-center justify-center gap-2 px-4 py-2 bg-marke/10 hover:bg-marke/20 text-marke font-bold rounded-xl transition-colors border border-marke/20 disabled:opacity-50 disabled:cursor-not-allowed">
            <Loader2 v-if="isDownloadingImage" class="w-4 h-4 animate-spin" />
            <Download v-else class="w-4 h-4" /> {{ t('albums.detail.download') }}
          </button>
          <button v-if="!readonly && deleteImage" @click="onDelete(selectedImage)" class="shrink-0 flex items-center justify-center gap-2 px-4 py-2 bg-rose-500/10 hover:bg-rose-500/20 text-rose-500 font-bold rounded-xl transition-colors border border-rose-500/20">
            <Trash2 class="w-4 h-4" /> {{ t('albums.detail.delete') }}
          </button>
        </div>

      </div>

      <!-- Platz für Ergänzungen des Aufrufers zum offenen Bild – im Projekt
           das KI-Feld. Den Rahmen bringt der Aufrufer mit, damit bei einem
           Bild ohne Ergänzung kein leerer Streifen stehen bleibt. -->
      <slot name="lightbox" :image="selectedImage" />
    </div>
  </div>
</template>
