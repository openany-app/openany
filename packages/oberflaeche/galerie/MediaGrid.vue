<script setup>
// Bilder-Raster mit Lightbox (EXIF-Anzeige, Download, Löschen) – aus der
// Album-Detailansicht extrahiert, damit die Galerie-Wurzelebene dasselbe
// Raster nutzen kann. URL-Bau und Aktionen kommen als Props von außen.
//
// AUCH VIDEOS (docs/plan-videos-galerie.md): In der
// Kachel ihr Standbild mit Play-Zeichen und Dauer -- oder, wenn das Gerät
// keines ziehen konnte, ein Film-Symbol. Im Leuchtkasten ein <video>. Kann
// der Browser es nicht abspielen (oft HEVC), sagt er das, statt schwarz zu
// bleiben.
//
// AUCH TON: In der Kachel Symbol, Name und Dauer, im Leuchtkasten TonSpieler.vue.
import { ref, computed, watch, onBeforeUnmount } from 'vue';
import { istVideo, dauerText, standbildAusLaufendem } from './videoStandbild';
import { istTon } from './ton';
import TonSpieler from './TonSpieler.vue';
import { vorschauAusBild } from './fotoVorschau';
import { nachMonaten } from './aufnahme';
import ZurueckSchliesst from '../base/ZurueckSchliesst.vue';
import { useToast } from '@oberflaeche/composables/useToast';
import { useI18n } from 'vue-i18n';
import { formatDateTime } from '@oberflaeche/shared/date';
import {
  MapPin, Calendar as CalendarIcon, Loader2, X, ChevronLeft, ChevronRight, Download, Trash2, FolderInput, Check, Play, Film, Music,
} from 'lucide-vue-next';

const props = defineProps({
  images: { type: Array, required: true },
  mediaUrl: { type: Function, required: true },   // (image) => Original-URL
  thumbUrl: { type: Function, required: true },   // (image) => Thumbnail-URL
  bildAktion: { type: Function, default: null },    // (image) => {icon, title, onClick} | null
  downloadImage: { type: Function, default: null },  // async (image); null = kein Download
  deleteImage: { type: Function, default: null },    // async (image) => true; null = read-only
  verschieben: { type: Function, default: null },    // (images) => void; null = geht hier nicht
  loescheBilder: { type: Function, default: null },  // async (images) => true; für die Auswahl
  // true: Löschen ausblenden (z. B. in eine Projekt-Freigabe eingebettet).
  readonly: { type: Boolean, default: false },
  // async (image) => void — vor dem Zeigen des Originals. Im Programm holt es
  // das Original von einem anderen Gerät, wenn hier nur die Vorschau liegt.
  vorbereiten: { type: Function, default: null },
  // async (image, { bild, dauer }) — eine Vorschau nachreichen: für ein Video
  // ohne Standbild aus dem Betrachter, während es läuft; für ein Foto ohne
  // Vorschau aus der Kachel, sobald sie das Original zeigt. null = nicht
  // möglich (Projekt-Freigaben: Vorschauen fremder Fotos setzt man nicht).
  standbildNachreichen: { type: Function, default: null },
  // async (image) => void — ein Video statt im Leuchtkasten in einem anderen
  // Programm abspielen. Das Programm gibt es an den Player des Systems, weil
  // Videos aus der Ablage in der Webansicht nicht zuverlässig spielen.
  videoOeffnen: { type: Function, default: null },
  // async (image) => { url, blob } für den Player, oder null, wenn der Ton schon
  // anderswo spielt. Ohne: die Adresse des Originals, dann ohne Wellenform.
  tonLaden: { type: Function, default: null },
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
 * meldet ein <img src=""> keinen Ladefehler; die Kachel bliebe null Pixel
 * hoch, ohne Film-Symbol und ohne dass das Nachholen je anläuft.
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
/*
 * DASSELBE FÜR FOTOS. Der Server rechnet keine Vorschau; kam ein Foto ohne an (Projekt-Import, großes Foto in Stücken), zeigt
 * die Kachel das Original -- und aus genau diesem, schon dekodierten Bild
 * entsteht die Vorschau und geht hinauf. Ab dem nächsten Laden ist sie da.
 *
 * NUR BEI `thumb_hash === null`, nicht bei „fehlt": Wo ein Rahmen das Feld
 * gar nicht mitliefert, schickte sonst jede Kachel bei jedem Ansehen eine
 * Vorschau, die es längst gibt.
 */
const fotoNachreichen = async (image, img) => {
  if (!props.standbildNachreichen || istVideo(image) || image.thumb_hash !== null || nachgereicht.has(image.id)) return;
  nachgereicht.add(image.id);
  const bild = await vorschauAusBild(img);
  if (!bild) return;
  try {
    await props.standbildNachreichen(image, { bild });
  } catch {
    // Beim nächsten Ansehen noch einmal.
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

const ton = ref(null); // { url, blob }
const tonFreigeben = () => {
  if (ton.value?.url.startsWith('blob:')) URL.revokeObjectURL(ton.value.url);
  ton.value = null;
};
watch(selectedImage, async (bild) => {
  tonFreigeben();
  if (!bild || !istTon(bild)) return;
  try {
    const geladen = props.tonLaden ? await props.tonLaden(bild) : { url: props.mediaUrl(bild) };
    if (geladen === null) {
      closeLightbox();
    } else if (selectedImage.value?.id === bild.id) {
      ton.value = geladen;
    } else if (geladen.url.startsWith('blob:')) {
      URL.revokeObjectURL(geladen.url);
    }
  } catch (e) {
    toast.error(String(e?.message ?? e));
  }
});
onBeforeUnmount(tonFreigeben);
/*
 * MEHRFACHAUSWAHL: Gedrückt halten wählt ein Foto aus, danach ergänzt jeder
 * Tipp die Auswahl. Scrollen bricht das Halten ab (`pointercancel`).
 */
const auswahl = ref(new Set());
const waehlt = ref(false);
const kannWaehlen = computed(() => !props.readonly && Boolean(props.verschieben || props.loescheBilder));
const auswahlBilder = computed(() => props.images.filter((b) => auswahl.value.has(b.id)));
let halten = null;
let klickSchlucken = false;
const umschalten = (image) => {
  const neu = new Set(auswahl.value);
  if (neu.has(image.id)) neu.delete(image.id); else neu.add(image.id);
  auswahl.value = neu;
  if (!neu.size) waehlt.value = false;
};
const auswahlEnde = () => {
  waehlt.value = false;
  auswahl.value = new Set();
};
const haltenEnde = () => {
  clearTimeout(halten);
  halten = null;
};
const haltenBeginn = (image) => {
  // Android meldet nach dem Halten nicht immer einen Tipp: Der nächste Druck
  // ist eine neue Geste, und ihr Tipp zählt.
  klickSchlucken = false;
  if (!kannWaehlen.value || waehlt.value) return;
  halten = setTimeout(() => {
    klickSchlucken = true;
    waehlt.value = true;
    umschalten(image);
  }, 500);
};
const kachelTipp = (image) => {
  if (klickSchlucken) {
    klickSchlucken = false;
    return;
  }
  if (waehlt.value) umschalten(image);
  else openLightbox(image);
};
const alleWaehlen = () => { auswahl.value = new Set(props.images.map((b) => b.id)); };
const auswahlVerschieben = () => props.verschieben(auswahlBilder.value);
// Verschoben oder gelöscht: Was nicht mehr hier liegt, fällt aus Auswahl und
// großer Ansicht. Beim Abbrechen bleibt beides stehen.
watch(() => props.images, (bilder) => {
  const da = new Set(bilder.map((b) => b.id));
  if (selectedImage.value && !da.has(selectedImage.value.id)) closeLightbox();
  if ([...auswahl.value].some((id) => !da.has(id))) {
    auswahl.value = new Set([...auswahl.value].filter((id) => da.has(id)));
    if (!auswahl.value.size) waehlt.value = false;
  }
});
const auswahlLoeschen = async () => {
  if (await props.loescheBilder(auswahlBilder.value)) auswahlEnde();
};
const onMove = (image) => props.verschieben([image]);
const aktionFuerBild = computed(() => (props.bildAktion && selectedImage.value ? props.bildAktion(selectedImage.value) : null));
const aktionAusfuehren = async () => {
  const bild = selectedImage.value;
  await aktionFuerBild.value.onClick();
  bild.auf_server = !bild.auf_server;
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
  try {
    await props.downloadImage(image);
  } catch {
    toast.error(t('albums.detail.imageDownloadFailed'));
  } finally {
    isDownloadingImage.value = false;
  }
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

const leistenKnopf = 'flex flex-col items-center justify-center gap-1 min-w-0 px-1 py-2 rounded-xl text-[11px] sm:text-xs font-bold leading-tight text-center break-words text-slate-100 hover:bg-white/10 transition-colors disabled:opacity-50 disabled:cursor-not-allowed';

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
    <ZurueckSchliesst v-if="waehlt" @schliessen="auswahlEnde" />
    <div v-if="waehlt" class="fixed left-2 right-2 bottom-20 md:bottom-4 z-[60] karte shadow-lg px-3 py-2 flex items-center gap-2 max-w-xl mx-auto">
      <button type="button" class="p-1.5 text-slate-400 hover:bg-auflage rounded-xl shrink-0" :aria-label="t('common.close')" @click="auswahlEnde"><X class="w-5 h-5" /></button>
      <span class="text-sm font-bold text-schrift whitespace-nowrap">{{ t('albums.detail.selectedCount', { count: auswahl.size }) }}</span>
      <button type="button" class="text-sm font-bold text-marke px-2 py-1 hover:underline" @click="alleWaehlen">{{ t('albums.detail.selectAll') }}</button>
      <span class="flex-1"></span>
      <button v-if="verschieben" type="button" class="p-2 text-fliess hover:bg-auflage rounded-xl shrink-0" :title="t('albums.detail.moveAlbum')" :aria-label="t('albums.detail.moveAlbum')" @click="auswahlVerschieben"><FolderInput class="w-5 h-5" /></button>
      <button v-if="loescheBilder" type="button" class="p-2 text-rose-500 hover:bg-auflage rounded-xl shrink-0" :title="t('albums.detail.delete')" :aria-label="t('albums.detail.delete')" @click="auswahlLoeschen"><Trash2 class="w-5 h-5" /></button>
    </div>
    <!-- Je Monat eine Überschrift und darunter sein Raster. -->
    <section v-for="monat in monate" :key="monat.schluessel" class="pb-6 last:pb-10">
    <h3 v-if="monat.titel" class="sticky top-0 z-10 py-2 mb-2 text-sm font-extrabold text-schrift bg-flaeche/90 backdrop-blur-sm">{{ monat.titel }}</h3>
    <div class="columns-2 md:columns-3 lg:columns-4 gap-2 sm:gap-4 space-y-2 sm:space-y-4">
      <div
        v-for="image in monat.bilder" :key="image.id"
        @click="kachelTipp(image)"
        @pointerdown="haltenBeginn(image)"
        @pointerup="haltenEnde"
        @pointerleave="haltenEnde"
        @pointercancel="haltenEnde"
        @contextmenu="kannWaehlen && $event.preventDefault()"
        :class="auswahl.has(image.id) ? 'ring-4 ring-marke' : ''"
        class="select-none break-inside-avoid relative group rounded-xl overflow-hidden cursor-pointer shadow-sm hover:shadow-xl transition-all duration-300 bg-auflage border border-slate-200/50 dark:border-slate-700/50"
      >
        <div v-if="istTon(image)"
             class="w-full aspect-video flex flex-col items-center justify-center gap-1.5 px-3 bg-slate-800 text-slate-300">
          <Music class="w-8 h-8 shrink-0" />
          <span class="max-w-full truncate text-xs font-bold">{{ image.name }}</span>
          <span v-if="dauer(image)" class="absolute bottom-2 right-2 px-1.5 py-0.5 rounded-md bg-black/60 text-white text-[11px] font-bold">
            {{ dauer(image) }}
          </span>
        </div>
        <div v-else-if="ohneVorschau(image)"
             class="w-full aspect-video flex items-center justify-center bg-slate-800 text-slate-400">
          <Film class="w-10 h-10" />
        </div>
        <img v-else :src="lokaleStandbilder[image.id] ?? thumbUrl(image)" class="w-full h-auto object-cover group-hover:scale-105 transition-transform duration-700 ease-out" loading="lazy"
             @load="fotoNachreichen(image, $event.target)"
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
        <span v-if="waehlt" class="absolute top-2 left-2 z-10 w-6 h-6 rounded-full border-2 border-white flex items-center justify-center pointer-events-none"
              :class="auswahl.has(image.id) ? 'bg-marke' : 'bg-black/30'">
          <Check v-if="auswahl.has(image.id)" class="w-4 h-4 text-white" />
        </span>
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

        <div v-if="istTon(selectedImage)" class="w-full max-w-md text-center text-slate-200 space-y-5">
          <Music class="w-16 h-16 mx-auto text-slate-400" />
          <p class="text-sm font-bold break-words">{{ selectedImage.name }}</p>
          <TonSpieler v-if="ton" :key="selectedImage.id" :url="ton.url" :blob="ton.blob ?? null" :dauer="selectedImage.custom_properties?.dauer ?? null" />
          <Loader2 v-else class="w-8 h-8 mx-auto animate-spin" />
        </div>
        <template v-else-if="istVideo(selectedImage)">
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

      <!-- Unten: Fotodaten und die Aktionen, gleich breit über die ganze Zeile. -->
      <div class="w-full bg-slate-900 border-t border-slate-800 shadow-2xl shrink-0" :style="{ paddingBottom: 'var(--rand-unten, 0px)' }">

        <div v-if="selectedImage.custom_properties?.exif?.date || hasGps(selectedImage)" class="flex flex-wrap items-center gap-4 sm:gap-8 px-4 pt-3">
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
        <div v-else-if="!istTon(selectedImage)" class="px-4 pt-3 text-sm text-slate-500 font-medium">
          {{ t('albums.detail.noExif') }}
        </div>

        <div class="grid grid-flow-col auto-cols-fr gap-1 p-2 max-w-2xl mx-auto">
          <button v-if="downloadImage" @click="onDownload(selectedImage)" :disabled="isDownloadingImage" :class="leistenKnopf">
            <Loader2 v-if="isDownloadingImage" class="w-5 h-5 animate-spin" />
            <Download v-else class="w-5 h-5" />
            <span>{{ t('albums.detail.download') }}</span>
          </button>
          <button v-if="!readonly && aktionFuerBild" @click="aktionAusfuehren" :class="leistenKnopf">
            <component :is="aktionFuerBild.icon" class="w-5 h-5" />
            <span>{{ aktionFuerBild.title }}</span>
          </button>
          <button v-if="!readonly && verschieben" @click="onMove(selectedImage)" :class="leistenKnopf">
            <FolderInput class="w-5 h-5" />
            <span>{{ t('albums.detail.moveAlbum') }}</span>
          </button>
          <button v-if="!readonly && deleteImage" @click="onDelete(selectedImage)" :class="[leistenKnopf, '!text-rose-400']">
            <Trash2 class="w-5 h-5" />
            <span>{{ t('albums.detail.papierkorb') }}</span>
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
