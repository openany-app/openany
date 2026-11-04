<script setup>
/*
 * Ton aufnehmen: Knopf neben den Kamera-Knöpfen und das Aufnahmefenster.
 *
 * Wie aufgenommen wird, weiß die Anwendung (`aufnehmer`, Programm:
 * app/src/quellen/tonaufnahme.js); ohne `aufnehmer` fehlt der Knopf.
 *
 * Gibt es nur das Mikrofon, startet ein Tipp gleich die Aufnahme. Gibt es auch
 * Systemaudio, wird die Quelle vorher gewählt: Während der Aufnahme lässt sie
 * sich nicht mehr wechseln.
 *
 * DIE AUFNAHME KANN OHNE DIESES FENSTER WEITERGELAUFEN SEIN (Android: App
 * geschlossen, Beenden über die Benachrichtigung). Beim Laden wird sie deshalb
 * wieder aufgenommen, eine schon fertige gespeichert.
 */
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';
import { useI18n } from 'vue-i18n';
import { Mic, Volume2, Pause, Play, Loader2 } from 'lucide-vue-next';
import BaseModal from '../base/BaseModal.vue';
import BaseButton from '../base/BaseButton.vue';
import { dauerText } from './videoStandbild';
import { useToast } from '../composables/useToast';
import { useConfirm } from '../composables/useConfirm';

const props = defineProps({
  // Siehe app/src/quellen/tonaufnahme.js; null = geht hier nicht.
  aufnehmer: { type: Object, default: null },
  // Das geöffnete Album. Ohne: das Album „Aufnahmen", denn was in keinem Album
  // liegt, kommt in keinen Abgleich.
  albumId: { type: [Number, String], default: null },
  // (name) => id des Albums für Aufnahmen
  aufnahmeAlbum: { type: Function, default: null },
  disabled: { type: Boolean, default: false },
});
const emit = defineEmits(['gespeichert']);

const { t } = useI18n();
const toast = useToast();
const { confirmDelete } = useConfirm();

const phase = ref(null); // null | 'wahl' | 'startet' | 'laeuft' | 'speichert'
const stand = ref({});
const art = ref('mikrofon');
const fehler = ref('');
const pegelReihe = ref(new Array(24).fill(0));

let takt = null;
const taktAus = () => { clearInterval(takt); takt = null; };

function lesen() {
  const s = props.aufnehmer.stand();
  stand.value = s;
  if (s.laeuft) {
    phase.value = 'laeuft';
    pegelReihe.value = [...pegelReihe.value.slice(1), s.pausiert ? 0 : s.pegel];
    return;
  }
  if (phase.value === 'startet' && s.fehler) {
    fehler.value = t(`albums.ton.fehler.${s.fehler}`);
    phase.value = props.aufnehmer.kannSystem ? 'wahl' : null;
    if (!phase.value) toast.error(fehler.value);
    taktAus();
  } else if (phase.value === 'laeuft') {
    // Von außen beendet: über die Benachrichtigung oder die Einwilligung entzogen.
    taktAus();
    if (s.fertig) speichern();
    else {
      phase.value = null;
      if (s.fehler) toast.error(t(`albums.ton.fehler.${s.fehler}`));
    }
  }
}

const taktAn = () => { if (!takt) takt = setInterval(lesen, 200); };

async function starten() {
  fehler.value = '';
  phase.value = 'startet';
  await props.aufnehmer.starten(art.value);
  taktAn();
  lesen();
}

function oeffnen() {
  fehler.value = '';
  if (props.aufnehmer.kannSystem) phase.value = 'wahl';
  else starten();
}

const pausiert = computed(() => stand.value.pausiert);
const pauseUmschalten = () => {
  props.aufnehmer.pausieren(!pausiert.value);
  lesen();
};

function dateiname() {
  const d = new Date();
  const z = (n) => String(n).padStart(2, '0');
  return `${t('albums.ton.name')} ${d.getFullYear()}-${z(d.getMonth() + 1)}-${z(d.getDate())} ${z(d.getHours())}-${z(d.getMinutes())}`;
}

async function speichern() {
  taktAus();
  phase.value = 'speichert';
  try {
    const album = props.albumId ?? await props.aufnahmeAlbum(t('albums.ton.album'));
    const bild = await props.aufnehmer.speichern(album, dateiname());
    emit('gespeichert', bild);
    toast.success(t('albums.ton.gespeichert'));
    phase.value = null;
  } catch (e) {
    // Die fertige Datei bleibt liegen und wird beim nächsten Laden wieder versucht.
    console.error('Aufnahme nicht gespeichert:', e);
    toast.error(t('albums.ton.fehler.speichern'));
    phase.value = null;
  }
}

async function verwerfen() {
  if (!(await confirmDelete(t('albums.ton.verwerfenFrage')))) return;
  taktAus();
  await props.aufnehmer.verwerfen();
  phase.value = null;
}

const zeit = computed(() => dauerText((stand.value.ms ?? 0) / 1000));

onMounted(() => {
  if (!props.aufnehmer) return;
  const s = props.aufnehmer.stand();
  if (s.laeuft) {
    phase.value = 'laeuft';
    taktAn();
  } else if (s.fertig) {
    speichern();
  }
});
onBeforeUnmount(taktAus);

const knopf = 'flex items-center justify-center h-9 w-9 bg-slate-100 hover:bg-slate-200 dark:bg-slate-800 dark:hover:bg-slate-700 text-fliess rounded-xl transition-all cursor-pointer disabled:opacity-50';
const quellenKnopf = (an) => ['flex-1 flex items-center justify-center gap-2 px-3 py-2.5 rounded-xl text-sm font-bold transition-colors', an ? 'bg-marke text-white' : 'bg-auflage text-fliess'];
</script>

<template>
  <template v-if="aufnehmer">
    <button type="button" :class="knopf" :disabled="disabled || phase === 'speichert'" :title="t('albums.ton.aufnehmen')" :aria-label="t('albums.ton.aufnehmen')" @click="oeffnen">
      <Mic class="w-4 h-4" />
    </button>

    <BaseModal v-if="phase" :title="t('albums.ton.titel')" :fest="phase !== 'wahl'" @close="phase = null">
      <div v-if="phase === 'wahl'" class="space-y-4">
        <div class="flex gap-2">
          <button type="button" :class="quellenKnopf(art === 'mikrofon')" @click="art = 'mikrofon'"><Mic class="w-4 h-4" /> {{ t('albums.ton.mikrofon') }}</button>
          <button type="button" :class="quellenKnopf(art === 'system')" @click="art = 'system'"><Volume2 class="w-4 h-4" /> {{ t('albums.ton.system') }}</button>
        </div>
        <p v-if="art === 'system'" class="text-sm text-leise">{{ t('albums.ton.systemHinweis') }}</p>
        <p v-if="fehler" class="text-sm text-rose-500">{{ fehler }}</p>
      </div>

      <div v-else-if="phase === 'laeuft'" class="flex flex-col items-center gap-4 py-2">
        <p class="flex items-center gap-2 text-sm font-bold text-leise">
          <component :is="stand.art === 'system' ? Volume2 : Mic" class="w-4 h-4" />
          {{ stand.art === 'system' ? t('albums.ton.system') : t('albums.ton.mikrofon') }}
        </p>
        <p class="text-4xl font-extrabold tabular-nums text-schrift">
          <span class="inline-block w-3 h-3 mr-2 align-middle rounded-full" :class="pausiert ? 'bg-slate-400' : 'bg-rose-500 animate-pulse'"></span>{{ zeit }}
        </p>
        <div class="flex items-center gap-0.5 h-10" aria-hidden="true">
          <span v-for="(p, i) in pegelReihe" :key="i" class="w-1 rounded-full bg-marke" :style="{ height: `${Math.max(6, Math.sqrt(p) * 100)}%` }"></span>
        </div>
      </div>

      <div v-else class="flex justify-center py-8"><Loader2 class="w-8 h-8 text-marke animate-spin" /></div>

      <template v-if="phase === 'wahl' || phase === 'laeuft'" #footer>
        <template v-if="phase === 'wahl'">
          <BaseButton variant="ghost" @click="phase = null">{{ t('common.cancel') }}</BaseButton>
          <BaseButton @click="starten"><Mic class="w-4 h-4" /> {{ t('albums.ton.starten') }}</BaseButton>
        </template>
        <template v-else>
          <BaseButton variant="ghost" @click="verwerfen">{{ t('albums.ton.verwerfen') }}</BaseButton>
          <BaseButton variant="secondary" :aria-label="pausiert ? t('albums.ton.weiter') : t('albums.ton.pause')" @click="pauseUmschalten">
            <Play v-if="pausiert" class="w-4 h-4" /><Pause v-else class="w-4 h-4" />
            <span>{{ pausiert ? t('albums.ton.weiter') : t('albums.ton.pause') }}</span>
          </BaseButton>
          <BaseButton @click="speichern">{{ t('albums.ton.speichern') }}</BaseButton>
        </template>
      </template>
    </BaseModal>
  </template>
</template>
