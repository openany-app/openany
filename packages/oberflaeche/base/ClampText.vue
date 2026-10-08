<script setup>
// Einheitliche Kürzung langer Beschreibungen in Kacheln/Kopfzeilen:
// klemmt auf N Zeilen (inkl. Umbruch langer Wörter/URLs, damit nichts
// horizontal aus der Kachel läuft) und blendet nur bei tatsächlich
// gekürztem Text ein „weiterlesen"/„weniger" ein. Der Toggle stoppt die
// Klick-Weitergabe, damit klickbare Kacheln nicht navigieren.
import { ref, computed, onMounted, onBeforeUnmount, watch, nextTick } from 'vue';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

const props = defineProps({
  text: { type: String, default: '' },
  lines: { type: Number, default: 2 },
});

const el = ref(null);
const expanded = ref(false);
const clamped = ref(false);

// Statische Klassen, damit Tailwind sie beim Build sieht.
//
// NICHT zusammen mit `block` verwenden: line-clamp-* setzt selbst
// `display:-webkit-box`, und `.block` steht im erzeugten Stylesheet dahinter.
// Bei gleicher Spezifität gewinnt die spätere Regel – `display:block` schlug
// die Klemmung also tot, an jeder Aufrufstelle und ohne jede Fehlermeldung.
// Deshalb wird `block` nur im ausgeklappten Zustand gesetzt; -webkit-box ist
// ohnehin ein Block-Element.
const lineClass = computed(() => ({
  1: 'line-clamp-1', 2: 'line-clamp-2', 3: 'line-clamp-3',
  4: 'line-clamp-4', 5: 'line-clamp-5', 6: 'line-clamp-6',
}[props.lines] || 'line-clamp-2'));

const measure = () => {
  if (!expanded.value) clamped.value = !!el.value && el.value.scrollHeight > el.value.clientHeight + 1;
};
onMounted(() => { nextTick(measure); window.addEventListener('resize', measure); });
onBeforeUnmount(() => window.removeEventListener('resize', measure));
watch(() => props.text, () => { expanded.value = false; nextTick(measure); });
</script>

<template>
  <span class="block min-w-0">
    <span ref="el" class="break-words" :class="expanded ? 'block' : lineClass">{{ text }}</span>
    <button
      v-if="clamped || expanded"
      @click.stop.prevent="expanded = !expanded"
      class="text-xs font-bold text-marke hover:underline"
    >{{ expanded ? t('home.base.clampText.less') : t('home.base.clampText.more') }}</button>
  </span>
</template>
