<script setup>
/*
 * Die Werkzeugleiste zum Bearbeiten eines PDFs: Lesen · Markieren · Stift ·
 * Text, dazu Farbe, Stärke und Rückgängig (docs/plan-pdf-bearbeiten.md,
 * Schritt 2).
 *
 * Nur Knöpfe. Was sie in pdf.js auslösen, macht der PdfBetrachter -- er hält
 * den EventBus, diese Leiste meldet nur, was gewählt wurde.
 *
 * MARKIEREN AUF DEM TABLET: pdf.js macht aus einer Textauswahl eine
 * Markierung, sobald der Finger losgelassen wird. Nach langem Drücken meldet
 * Android aber einen Abbruch statt eines Loslassens -- es entstand nichts
 * (27.09.2026). Deshalb der Knopf „Auswahl markieren": lange drücken, mit den
 * Griffen anpassen, Knopf. Mit der Maus markiert pdf.js wie gewohnt selbst.
 *
 * „Lesen" ist ein eigenes Werkzeug und kein Aus-Zustand: Auf dem Tablet
 * zeichnet der Finger im Stift-Modus, statt zu blättern. Wer blättern will,
 * schaltet sichtbar zurück -- das ist verlässlicher als jede Geste, die
 * erraten müsste, was gemeint war.
 */
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { Hand, Highlighter, PenLine, Type, MessageSquare, Undo2, Redo2 } from 'lucide-vue-next';
import { FARBEN, STAERKEN } from './pdfWerkzeuge';

const props = defineProps({
  werkzeug: { type: String, default: 'lesen' }, // lesen | markieren | stift | text
  farbe: { type: String, default: '' },
  staerke: { type: Number, default: 0 },
  kannZurueck: { type: Boolean, default: false },
  kannVor: { type: Boolean, default: false },
  // Ist gerade Text im Dokument ausgewählt?
  auswahl: { type: Boolean, default: false },
});
const emit = defineEmits(['werkzeug', 'farbe', 'staerke', 'zurueck', 'vor', 'auswahl-markieren']);

const beruehrbar = typeof window !== 'undefined'
  && typeof window.matchMedia === 'function'
  && window.matchMedia('(pointer: coarse)').matches;
const { t } = useI18n();

const WERKZEUGE = [
  { id: 'lesen', icon: Hand },
  { id: 'markieren', icon: Highlighter },
  { id: 'stift', icon: PenLine },
  { id: 'text', icon: Type },
  // Keine Zeichenfläche, sondern die Übersicht aller Kommentare (Schritt 3).
  { id: 'kommentare', icon: MessageSquare },
];

const farben = computed(() => FARBEN[props.werkzeug] ?? []);
const staerken = computed(() => STAERKEN[props.werkzeug] ?? []);

const knopf = 'p-2 rounded-xl transition-colors cursor-pointer disabled:opacity-40 disabled:cursor-default';
</script>

<template>
  <div class="flex items-center gap-1 sm:gap-2 px-2 sm:px-4 py-2 border-b border-linie bg-flaeche shrink-0 overflow-x-auto" role="toolbar" :aria-label="t('common.pdf.bearbeiten')">
    <!-- Werkzeuge -->
    <div class="flex items-center gap-0.5 p-0.5 rounded-xl bg-auflage shrink-0" role="radiogroup">
      <button
        v-for="w in WERKZEUGE"
        :key="w.id"
        type="button"
        role="radio"
        :aria-checked="werkzeug === w.id"
        :title="t(`common.pdf.werkzeug.${w.id}`)"
        :aria-label="t(`common.pdf.werkzeug.${w.id}`)"
        :class="[knopf, werkzeug === w.id ? 'bg-marke text-white shadow-sm' : 'text-fliess hover:bg-flaeche']"
        @click="emit('werkzeug', w.id)"
      >
        <component :is="w.icon" class="w-4 h-4" />
      </button>
    </div>

    <!-- Farben des Werkzeugs -->
    <div v-if="farben.length" class="flex items-center gap-1.5 px-1 shrink-0" role="radiogroup" :aria-label="t('common.pdf.farbe')">
      <button
        v-for="f in farben"
        :key="f"
        type="button"
        role="radio"
        :aria-checked="farbe === f"
        :aria-label="f"
        class="w-6 h-6 rounded-full border-2 transition-transform cursor-pointer"
        :class="farbe === f ? 'border-marke scale-110' : 'border-linie'"
        :style="{ backgroundColor: f }"
        @click="emit('farbe', f)"
      ></button>
    </div>

    <!-- Stärke: Strich beim Stift, Schriftgröße beim Text -->
    <div v-if="staerken.length" class="flex items-center gap-0.5 p-0.5 rounded-xl bg-auflage shrink-0" role="radiogroup" :aria-label="t(werkzeug === 'text' ? 'common.pdf.schriftgroesse' : 'common.pdf.staerke')">
      <button
        v-for="(s, i) in staerken"
        :key="s"
        type="button"
        role="radio"
        :aria-checked="staerke === s"
        :aria-label="String(s)"
        :class="[knopf, 'w-9 flex items-center justify-center', staerke === s ? 'bg-flaeche shadow-sm text-marke' : 'text-fliess']"
        @click="emit('staerke', s)"
      >
        <span v-if="werkzeug === 'stift'" class="block rounded-full bg-current" :style="{ width: `${6 + i * 5}px`, height: `${Math.min(2 + i * 2, 7)}px` }"></span>
        <span v-else class="font-bold leading-none" :style="{ fontSize: `${10 + i * 3}px` }">A</span>
      </button>
    </div>

    <!-- Markieren auf Touch: erst auswählen, dann bestätigen. `pointerdown`
         ohne Standard, sonst hebt das Tippen die Auswahl auf. -->
    <template v-if="werkzeug === 'markieren'">
      <button
        v-if="auswahl"
        type="button"
        class="px-3 py-1.5 rounded-xl bg-marke text-white text-xs font-bold whitespace-nowrap shrink-0 cursor-pointer"
        @pointerdown.prevent
        @mousedown.prevent
        @click="emit('auswahl-markieren')"
      >
        {{ t('common.pdf.auswahlMarkieren') }}
      </button>
      <span v-else-if="beruehrbar" class="text-xs text-leise whitespace-nowrap shrink-0">{{ t('common.pdf.auswahlHinweis') }}</span>
    </template>
    <span v-else-if="werkzeug === 'kommentare'" class="text-xs text-leise whitespace-nowrap shrink-0">{{ t('common.pdf.kommentarHinweis') }}</span>

    <span class="flex-1"></span>

    <button type="button" :class="[knopf, 'text-fliess hover:bg-auflage']" :disabled="!kannZurueck" :title="t('common.pdf.rueckgaengig')" :aria-label="t('common.pdf.rueckgaengig')" @click="emit('zurueck')">
      <Undo2 class="w-4 h-4" />
    </button>
    <button type="button" :class="[knopf, 'text-fliess hover:bg-auflage']" :disabled="!kannVor" :title="t('common.pdf.wiederholen')" :aria-label="t('common.pdf.wiederholen')" @click="emit('vor')">
      <Redo2 class="w-4 h-4" />
    </button>
  </div>
</template>
