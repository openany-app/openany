<script setup>
/*
 * Das Inhaltsverzeichnis der Einstellungen -- gebaut wie das der Hilfe
 * (HilfeSeite.vue): links, klebt beim Scrollen, je Gruppe aufklappbar mit
 * ihren Kacheln; auf dem Telefon steht es über den Kacheln.
 *
 * GELESEN AUS DER SEITE, NICHT AUS EINER LISTE. Welche Kacheln es gibt,
 * entscheidet jede Kachel selbst (sie blendet sich aus, wo sie nichts zu
 * bieten hat -- etwa die Aktualisierung unter Android), und manche erscheinen
 * erst, wenn ihre Daten da sind. Eine feste Liste zeigte dann Einträge ins
 * Leere. Darum: Gruppen sind die `section[data-gruppe]` unter `wurzel`, ihre
 * Kacheln die `.karte` direkt im Raster darunter, der Name deren erste
 * Überschrift.
 */
import { ref, watch, onBeforeUnmount } from 'vue';
import { useI18n } from 'vue-i18n';
import { ChevronRight } from 'lucide-vue-next';

const props = defineProps({
  wurzel: { type: Object, default: null }, // das Element, das die Gruppen trägt
});
const { t } = useI18n();

const gruppen = ref([]); // [{ id, titel, kacheln: [{ id, titel }] }]
const offen = ref(new Set());

function umschalten(id) {
  const neu = new Set(offen.value);
  if (neu.has(id)) neu.delete(id); else neu.add(id);
  offen.value = neu;
}

function lesen() {
  const el = props.wurzel;
  if (!el) return;
  const neu = [];
  el.querySelectorAll('section[data-gruppe]').forEach((s) => {
    const kacheln = [];
    s.querySelectorAll(':scope > div > .karte').forEach((k, i) => {
      const titel = k.querySelector('h2, h3')?.textContent?.trim();
      if (!titel) return;
      if (!k.id) {
        k.id = `einstellung-${s.dataset.gruppe}-${i}`;
        // Beim Springen nicht unter der Kopfzeile verschwinden.
        k.style.scrollMarginTop = '5rem';
      }
      kacheln.push({ id: k.id, titel });
    });
    if (kacheln.length) neu.push({ id: s.id, titel: s.dataset.titel, kacheln });
  });
  // Nur zuweisen, wenn sich etwas geändert hat -- sonst rendert jede
  // Änderung irgendwo in einer Kachel das Verzeichnis neu.
  if (JSON.stringify(neu) !== JSON.stringify(gruppen.value)) gruppen.value = neu;
}

function springen(id) {
  document.getElementById(id)?.scrollIntoView({ behavior: 'smooth', block: 'start' });
}

// Das Element kommt erst, wenn die Seite steht -- darauf warten.
let beobachter = null;
watch(() => props.wurzel, (el) => {
  beobachter?.disconnect();
  beobachter = null;
  if (!el) return;
  lesen();
  beobachter = new MutationObserver(lesen);
  beobachter.observe(el, { childList: true, subtree: true, characterData: true });
}, { immediate: true });
onBeforeUnmount(() => beobachter?.disconnect());
</script>

<template>
  <nav v-if="gruppen.length" class="lg:sticky lg:top-20 karte shadow-sm p-5 max-h-[calc(100vh-7rem)] overflow-y-auto">
    <div class="text-xs font-extrabold uppercase tracking-wider text-slate-400 mb-3">{{ t('help.toc') }}</div>
    <ul class="space-y-1">
      <li v-for="g in gruppen" :key="g.id">
        <div class="flex items-center gap-1">
          <button type="button" @click="springen(g.id)"
            class="flex-1 min-w-0 px-2.5 py-1.5 rounded-xl text-left text-sm font-bold text-fliess hover:bg-auflage transition-colors cursor-pointer">
            {{ g.titel }}
          </button>
          <button type="button" @click="umschalten(g.id)"
            class="shrink-0 p-1.5 rounded-xl text-slate-400 hover:text-marke hover:bg-auflage transition-colors cursor-pointer"
            :aria-expanded="offen.has(g.id)" :aria-label="g.titel">
            <ChevronRight class="w-4 h-4 transition-transform" :class="offen.has(g.id) ? 'rotate-90' : ''" />
          </button>
        </div>
        <ul v-if="offen.has(g.id)" class="mt-1 mb-2 ml-4 pl-3 border-l border-linie space-y-0.5">
          <li v-for="k in g.kacheln" :key="k.id">
            <button type="button" @click="springen(k.id)"
              class="w-full px-2 py-1 rounded-xl text-left text-xs font-semibold text-leise hover:text-marke hover:bg-auflage transition-colors cursor-pointer">
              {{ k.titel }}
            </button>
          </li>
        </ul>
      </li>
    </ul>
  </nav>
</template>
