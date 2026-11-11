<script setup>
/*
 * Eine Fläche, auf der man Dateien und Ordner ablegen kann
 * (docs/plan-viele-dateien.md, D1, Skizze A):
 *
 *   ⤓  Hier ablegen
 *   → Album „Urlaub 2026“
 *   Ordner werden eigene Alben.
 *
 * Die Pfade kennt nur die Anwendung. Sie meldet über `uebernahme.beimZiehen`,
 * WO gezogen wird; liegt die Stelle auf dieser Fläche, zeigt sie den Rahmen,
 * beim Loslassen nennt sie mit `uebernahme.ablegen` das Ziel. Eine
 * verborgene Fläche (v-show) hat keine Ausdehnung und trifft nie.
 *
 * Ohne `uebernahme` (oder ohne `ablegen` darin) ist es eine schlichte Hülle.
 */
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { ArrowDownToLine } from 'lucide-vue-next';
import { useUebernahme } from '../composables/useUebernahme';

const props = defineProps({
  uebernahme: { type: Object, default: null },
  // `galerie` oder `files`
  zone: { type: String, required: true },
  // Album oder Ordner, `null` = oberste Ebene
  ziel: { type: String, default: null },
  zielName: { type: String, default: '' },
  // `false`, solange das Ziel nicht feststeht (Album lädt noch) oder hier
  // nichts hinein darf.
  bereit: { type: Boolean, default: true },
});

const { t } = useI18n();
const { ablegen } = useUebernahme(props.uebernahme);
const flaeche = ref(null);
const aktiv = ref(false);

const zieltext = computed(() => {
  if (props.zone === 'galerie') {
    return props.ziel ? t('common.uebernahme.nachAlbum', { name: props.zielName }) : t('common.uebernahme.nachMediathek');
  }
  return props.ziel ? t('common.uebernahme.nachOrdner', { name: props.zielName }) : t('common.uebernahme.nachDateien');
});
const hinweis = computed(() => t(props.zone === 'galerie' ? 'common.uebernahme.hinweisAlben' : 'common.uebernahme.hinweisOrdner'));

function trifft({ x, y }) {
  const r = flaeche.value?.getBoundingClientRect();
  return Boolean(r && r.width && r.height && x >= r.left && x <= r.right && y >= r.top && y <= r.bottom);
}

function ziehen({ art, x, y }) {
  if (!props.bereit) return;
  if (art === 'ueber') {
    aktiv.value = trifft({ x, y });
  } else if (art === 'ablegen') {
    const hier = trifft({ x, y });
    aktiv.value = false;
    if (hier) ablegen(props.zone, props.ziel);
  } else {
    aktiv.value = false;
  }
}

let abmelden = null;
onMounted(() => {
  if (props.uebernahme?.beimZiehen) abmelden = props.uebernahme.beimZiehen(ziehen);
});
onUnmounted(() => abmelden?.());
</script>

<template>
  <div ref="flaeche" class="relative">
    <slot />
    <div
      v-if="aktiv"
      class="absolute inset-0 z-50 pointer-events-none border-2 border-dashed border-marke bg-marke-leise/90 rounded-xl p-8 backdrop-blur-sm"
    >
      <!-- Mitten im Blick, auch wenn die Fläche länger ist als das Fenster. -->
      <div class="sticky top-[30vh] flex flex-col items-center text-center">
        <div class="w-16 h-16 rounded-full bg-marke-leise flex items-center justify-center text-marke mb-4 shadow-lg">
          <ArrowDownToLine class="w-8 h-8" />
        </div>
        <p class="text-xl font-extrabold text-marke">{{ t('common.uebernahme.hierAblegen') }}</p>
        <p class="mt-1 text-sm font-semibold text-schrift break-all">→ {{ zieltext }}</p>
        <p class="mt-1 text-sm text-fliess">{{ hinweis }}</p>
      </div>
    </div>
  </div>
</template>
