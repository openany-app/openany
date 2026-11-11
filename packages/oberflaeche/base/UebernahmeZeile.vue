<script setup>
/*
 * Die schmale Zeile über Mediathek und Dateien, solange viele Dateien
 * übernommen werden (docs/plan-viele-dateien.md, Skizze D):
 *
 *   ⟳ 237 von 4.515 übernommen · noch ca. 2 Std.          [Pausieren]
 *   ✓ 4.509 übernommen · 6 nicht übernommen ›                       ×
 *
 * Der Pfeil öffnet die Liste mit Gründen, das Kreuz räumt die Zeile ab.
 * Ohne Schlange (`uebernahme` null) zeigt sie nichts.
 *
 * `neu` meldet, dass Dateien dazugekommen sind — höchstens alle paar
 * Sekunden und einmal am Ende, damit die Ansicht nicht bei jeder Datei
 * neu lädt.
 */
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { Loader2, Pause, Check, X, ChevronRight } from 'lucide-vue-next';
import BaseButton from './BaseButton.vue';
import BaseModal from './BaseModal.vue';
import { grundText } from '../composables/useUebernahme';

const props = defineProps({
  uebernahme: { type: Object, default: null },
});
const emit = defineEmits(['neu']);

const { t, te, locale } = useI18n();
const stand = ref(null);
const liste = ref(null);

const zahl = (n) => Number(n).toLocaleString(locale.value);
const erledigt = computed(() => (stand.value ? stand.value.fertig + stand.value.uebersprungen + stand.value.fehler : 0));
const gesamt = computed(() => (stand.value ? erledigt.value + stand.value.wartet : 0));
const offen = computed(() => (stand.value?.wartet ?? 0) > 0);

/*
 * Restzeit aus dem, was seit dem ersten Blick abgearbeitet wurde (Bytes,
 * nicht Dateien: ein Video wiegt hundert Fotos). Erst nach ein paar
 * Sekunden — vorher ist jede Schätzung geraten.
 */
let messung = null;
const restzeit = computed(() => {
  const s = stand.value;
  if (!s || !offen.value || s.pausiert || !messung) return '';
  const sekunden = (Date.now() - messung.zeit) / 1000;
  const geschafft = messung.bytes - s.bytes_offen;
  if (sekunden < 10 || geschafft <= 0) return '';
  const rest = s.bytes_offen / (geschafft / sekunden);
  if (rest < 60) return t('common.uebernahme.restKurz');
  if (rest < 3600) return t('common.uebernahme.restMinuten', { n: Math.round(rest / 60) });
  return t('common.uebernahme.restStunden', { n: Math.round(rest / 3600) });
});

let zuletztFertig = null;
let neuZeit = 0;
async function lesen() {
  if (!props.uebernahme) return;
  try {
    const s = await props.uebernahme.stand();
    if (s.wartet > 0 && s.laeuft && !s.pausiert) {
      if (!messung) messung = { zeit: Date.now(), bytes: s.bytes_offen };
    } else {
      messung = null;
    }
    const fertig = s.fertig;
    const dazu = zuletztFertig !== null && fertig > zuletztFertig;
    if (dazu && (!s.laeuft || Date.now() - neuZeit > 3000)) {
      neuZeit = Date.now();
      emit('neu');
    }
    zuletztFertig = fertig;
    stand.value = s;
  } catch {
    // Der Stand ist eine Anzeige; fehlt er einmal, kommt er mit der nächsten Meldung.
  }
}

async function pausieren(an) {
  stand.value = await props.uebernahme.pausieren(an);
  messung = null;
}
async function abbrechen() {
  await props.uebernahme.abbrechen();
  await lesen();
}
async function schliessen() {
  await props.uebernahme.abschliessen();
  zuletztFertig = null;
  await lesen();
}
async function listeZeigen() {
  liste.value = await props.uebernahme.nichtUebernommen();
}

let abmelden = null;
onMounted(() => {
  if (!props.uebernahme) return;
  abmelden = props.uebernahme.beobachten(lesen);
  lesen();
});
onUnmounted(() => abmelden?.());
</script>

<template>
  <div
    v-if="uebernahme && stand && gesamt > 0"
    class="flex flex-wrap items-center gap-x-3 gap-y-1 karte px-4 py-2 text-sm text-fliess"
    role="status"
  >
    <template v-if="offen">
      <Pause v-if="stand.pausiert" class="w-4 h-4 text-leise shrink-0" />
      <Loader2 v-else class="w-4 h-4 text-marke animate-spin shrink-0" />
      <span class="flex-1 min-w-0">
        <template v-if="stand.pausiert">{{ t('common.uebernahme.pausiert') }} · </template>
        {{ t('common.uebernahme.vonGesamt', { n: zahl(erledigt), gesamt: zahl(gesamt) }) }}
        <template v-if="restzeit"> · {{ restzeit }}</template>
      </span>
      <template v-if="stand.pausiert">
        <BaseButton groesse="klein" variant="secondary" @click="abbrechen">{{ t('common.cancel') }}</BaseButton>
        <BaseButton groesse="klein" @click="pausieren(false)">{{ t('common.uebernahme.fortsetzen') }}</BaseButton>
      </template>
      <BaseButton v-else groesse="klein" variant="secondary" @click="pausieren(true)">{{ t('common.uebernahme.pausieren') }}</BaseButton>
    </template>

    <template v-else>
      <Check class="w-4 h-4 text-emerald-600 shrink-0" />
      <span class="flex-1 min-w-0">
        {{ t('common.uebernahme.fertig', { n: zahl(stand.fertig) }) }}
        <template v-if="stand.uebersprungen"> · {{ t('common.uebernahme.schonDa', { n: zahl(stand.uebersprungen) }, stand.uebersprungen) }}</template>
        <template v-if="stand.fehler"> ·
          <button type="button" class="inline-flex items-center font-semibold text-marke hover:underline" @click="listeZeigen">
            {{ t('common.uebernahme.nichtUebernommen', { n: zahl(stand.fehler) }) }}
            <ChevronRight class="w-4 h-4" />
          </button>
        </template>
      </span>
      <button type="button" class="p-1 rounded-lg text-leise hover:bg-auflage" :aria-label="t('common.close')" @click="schliessen">
        <X class="w-4 h-4" />
      </button>
    </template>
  </div>

  <BaseModal v-if="liste" :title="t('common.uebernahme.listeTitel')" size="lg" @close="liste = null">
    <ul class="divide-y divide-slate-100 dark:divide-slate-800 max-h-[60vh] overflow-y-auto text-sm">
      <li v-for="(eintrag, i) in liste" :key="i" class="py-2">
        <div class="font-semibold text-schrift break-all">{{ eintrag.name }}</div>
        <div class="text-fliess">{{ grundText(t, te, eintrag.grund) }}</div>
        <div class="text-xs text-leise break-all">{{ eintrag.quelle }}</div>
      </li>
    </ul>
  </BaseModal>
</template>
