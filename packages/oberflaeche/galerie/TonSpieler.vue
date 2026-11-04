<script setup>
/*
 * Der Player für Ton in der großen Ansicht: Wellenform der ganzen Datei, die
 * abgespielte Strecke eingefärbt, ein Tipp springt an die Stelle.
 *
 * Ohne Wellenform (Datei zu groß, Format hier nicht dekodierbar) bleiben die
 * Balken flach und bedienen sich genauso.
 */
import { ref, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { Play, Pause } from 'lucide-vue-next';
import { dauerText } from './videoStandbild';
import { wellenform } from './ton';

const props = defineProps({
  url: { type: String, required: true },
  blob: { type: Object, default: null },
  // Für Formate, die ihre Länge nicht nennen (aufgenommenes WebM).
  dauer: { type: Number, default: null },
});

const { t } = useI18n();

const ANZAHL = 72;
const balken = ref(null);
const ton = ref(null);
const laeuft = ref(false);
const position = ref(0);
const laenge = ref(props.dauer ?? 0);

const anteil = computed(() => (laenge.value ? Math.min(1, position.value / laenge.value) : 0));
const hoehen = computed(() => (balken.value ?? new Array(ANZAHL).fill(0)).map((w) => Math.max(8, w * 100)));

function gelesen() {
  const d = ton.value.duration;
  if (Number.isFinite(d) && d > 0) laenge.value = d;
}

const umschalten = () => {
  if (ton.value.paused) ton.value.play().catch(() => {});
  else ton.value.pause();
};

function springen(event) {
  if (!laenge.value) return;
  const r = event.currentTarget.getBoundingClientRect();
  ton.value.currentTime = Math.min(1, Math.max(0, (event.clientX - r.left) / r.width)) * laenge.value;
}

function schieben(sekunden) {
  ton.value.currentTime = Math.min(laenge.value, Math.max(0, ton.value.currentTime + sekunden));
}

onMounted(async () => {
  balken.value = await wellenform(props.blob, ANZAHL);
});
</script>

<template>
  <div class="w-full space-y-4">
    <audio
      ref="ton"
      :src="url"
      autoplay
      @loadedmetadata="gelesen"
      @durationchange="gelesen"
      @timeupdate="position = ton.currentTime"
      @play="laeuft = true"
      @pause="laeuft = false"
    />

    <div
      class="flex items-center gap-[2px] h-24 w-full cursor-pointer touch-manipulation"
      role="slider"
      tabindex="0"
      :aria-label="t('albums.ton.position')"
      :aria-valuemin="0"
      :aria-valuemax="Math.round(laenge)"
      :aria-valuenow="Math.round(position)"
      :aria-valuetext="dauerText(position)"
      @click="springen"
      @keydown.left.prevent="schieben(-5)"
      @keydown.right.prevent="schieben(5)"
    >
      <span
        v-for="(h, i) in hoehen" :key="i"
        class="flex-1 min-w-0 rounded-full transition-colors"
        :class="(i + 0.5) / ANZAHL <= anteil ? 'bg-marke' : 'bg-slate-600'"
        :style="{ height: `${h}%` }"
      ></span>
    </div>

    <div class="flex items-center justify-between">
      <span class="w-12 text-left text-xs font-bold tabular-nums text-slate-300">{{ dauerText(position) }}</span>
      <button
        type="button"
        class="flex items-center justify-center w-14 h-14 rounded-full bg-marke text-white"
        :aria-label="laeuft ? t('albums.ton.pause') : t('albums.ton.abspielen')"
        @click="umschalten"
      >
        <Pause v-if="laeuft" class="w-6 h-6" />
        <Play v-else class="w-6 h-6 ml-0.5" />
      </button>
      <span class="w-12 text-right text-xs font-bold tabular-nums text-slate-300">{{ dauerText(laenge) }}</span>
    </div>
  </div>
</template>
