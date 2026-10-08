<script setup>
/*
 * Alle Kommentare im Dokument, im Werkzeug „Kommentare" (Schritt 3). Die
 * Einträge liefert pdf.js beim Umschalten in diesen Modus -- neue aus dieser
 * Sitzung wie die, die schon im PDF standen. Antippen springt hin.
 *
 * Breit: eine Spalte rechts. Schmal: ein Streifen unten, damit die Seite
 * darüber sichtbar bleibt.
 */
import { useI18n } from 'vue-i18n';
import { MessageSquare } from 'lucide-vue-next';

defineProps({
  // [{ id, seite, text, farbe, datum }]
  eintraege: { type: Array, default: () => [] },
});
const emit = defineEmits(['springen']);
const { t, locale } = useI18n();

const datum = (wert) => {
  const zeit = wert ? new Date(wert) : null;
  return zeit && !Number.isNaN(zeit.getTime()) ? zeit.toLocaleDateString(locale.value) : '';
};
</script>

<template>
  <aside class="absolute z-10 inset-x-0 bottom-0 max-h-[45%] sm:inset-x-auto sm:right-0 sm:top-0 sm:bottom-0 sm:max-h-none sm:w-80 flex flex-col bg-flaeche border-t sm:border-t-0 sm:border-l border-linie shadow-lg" :aria-label="t('common.pdf.werkzeug.kommentare')">
    <h4 class="px-4 py-3 text-sm font-extrabold text-schrift border-b border-linie flex items-center gap-2 shrink-0">
      <MessageSquare class="w-4 h-4 text-marke" />
      {{ t('common.pdf.werkzeug.kommentare') }}
      <span class="text-leise font-bold">{{ eintraege.length }}</span>
    </h4>
    <p v-if="!eintraege.length" class="px-4 py-6 text-sm text-leise">{{ t('common.pdf.keineKommentare') }}</p>
    <ul v-else class="overflow-y-auto divide-y divide-linie">
      <li v-for="e in eintraege" :key="e.id">
        <button type="button" class="w-full text-left px-4 py-3 hover:bg-auflage cursor-pointer flex gap-3" @click="emit('springen', e)">
          <span class="mt-1 w-3 h-3 rounded-full shrink-0 border border-linie" :style="{ backgroundColor: e.farbe || 'transparent' }"></span>
          <span class="min-w-0 flex-1">
            <span class="block text-sm text-schrift whitespace-pre-wrap break-words line-clamp-4">{{ e.text }}</span>
            <span class="block text-xs text-leise mt-1">{{ t('common.pdf.seite') }} {{ e.seite }}<template v-if="datum(e.datum)"> · {{ datum(e.datum) }}</template></span>
          </span>
        </button>
      </li>
    </ul>
  </aside>
</template>
