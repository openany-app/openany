<script setup>
// Der Abo-Link eines Wochenplan-TYPS: erzeugen, kopieren, erneuern
// (der alte Link ist im selben Moment tot) und widerrufen.
//
// Der Link hängt am Typ, nicht am geöffneten Plan. Das ist der einzige
// Punkt, den dieser Dialog erklären muss: Wer die Ferienbetreuung als
// zweiten Plan führt, hätte sonst zwei Links – und wer nur den ersten
// abonniert, sähe ab Ferienbeginn eine leere Betreuung, ohne dass ihm
// etwas fehlte.
//
// Der Orte-Schalter ist bewusst KEIN Nebensatz in der Beschriftung. Der
// Feed ist ein öffentlicher, nicht angemeldeter Link; mit Koordinaten
// stünden die Wohnanschriften beider Eltern dahinter, und eine einmal
// weitergeleitete URL ist nicht zurückzuholen. Deshalb steht die Warnung
// beim Schalter und nicht in einer Hilfeseite.
import { ref, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from './umgebung';
import { Rss, Copy, Check, RefreshCcw, Trash2, AlertTriangle, X, MapPin, Loader2 } from 'lucide-vue-next';
import { useToast } from '@oberflaeche/composables/useToast';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import BaseButton from '@oberflaeche/base/BaseButton.vue';

const props = defineProps({
  projectId: { type: [Number, String], required: true },
  // 'timetable' | 'care'
  type: { type: String, required: true },
});
const emit = defineEmits(['close']);

const { t } = useI18n();
const toast = useToast();
const { confirmDialog } = useConfirm();

const loading = ref(true);
const busy = ref(false);
const url = ref(null);
const mitOrten = ref(false);
const copied = ref(false);

const istBetreuung = computed(() => props.type === 'care');

const load = async () => {
  loading.value = true;
  try {
    const { data } = await api.getPlanFeeds(props.projectId);
    const feed = (data || []).find((f) => f.type === props.type);
    url.value = feed?.feed_url || null;
    mitOrten.value = !!feed?.with_places;
  } catch (e) {
    toast.error(t('projects.planFeed.loadFailed'));
  } finally {
    loading.value = false;
  }
};

const create = async () => {
  busy.value = true;
  try {
    const { data } = await api.createPlanFeedToken(props.projectId, props.type, mitOrten.value);
    url.value = data.feed_url;
    mitOrten.value = !!data.with_places;
    copied.value = false;
  } catch (e) {
    toast.error(t('projects.planFeed.createFailed'));
  } finally { busy.value = false; }
};

// Erneuern trennt jeden bestehenden Abonnenten – auch den anderen
// Elternteil. Das ist gewollt und muss deshalb einmal nachgefragt werden.
const rotate = async () => {
  const ok = await confirmDialog(t('projects.planFeed.rotateConfirm'), {
    title: t('projects.planFeed.rotate'),
    confirmLabel: t('projects.planFeed.rotate'),
  });
  if (ok) await create();
};

const remove = async () => {
  const ok = await confirmDialog(t('projects.planFeed.removeConfirm'), {
    title: t('projects.planFeed.remove'),
    confirmLabel: t('projects.planFeed.remove'),
    danger: true,
  });
  if (!ok) return;

  busy.value = true;
  try {
    await api.deletePlanFeedToken(props.projectId, props.type);
    url.value = null;
    mitOrten.value = false;
  } catch (e) {
    toast.error(t('projects.planFeed.removeFailed'));
  } finally { busy.value = false; }
};

// Das Umlegen lässt das Token in Ruhe – niemand muss seinen Kalender neu
// einrichten, nur weil die Adressen doch nicht mitgehen sollen.
const toggleOrte = async () => {
  const neu = !mitOrten.value;
  if (!url.value) { mitOrten.value = neu; return; }

  busy.value = true;
  try {
    const { data } = await api.setPlanFeedPlaces(props.projectId, props.type, neu);
    mitOrten.value = !!data.with_places;
  } catch (e) {
    toast.error(t('projects.planFeed.placesFailed'));
  } finally { busy.value = false; }
};

const copy = async () => {
  try {
    await navigator.clipboard.writeText(url.value);
    copied.value = true;
    setTimeout(() => { copied.value = false; }, 2000);
  } catch (e) { /* Zwischenablage evtl. gesperrt – die URL lässt sich markieren */ }
};

onMounted(load);
</script>

<template>
  <div class="fixed inset-0 bg-slate-900/60 backdrop-blur-sm flex items-center justify-center p-4 z-[100]" @click.self="emit('close')">
    <div class="bg-flaeche rounded-xl shadow-xl w-full max-w-lg max-h-[85vh] overflow-y-auto">
      <div class="flex items-center justify-between p-5 border-b border-linie">
        <h3 class="font-extrabold text-schrift flex items-center gap-2">
          <Rss class="w-5 h-5 text-marke" />
          {{ t(`projects.planFeed.title.${type}`) }}
        </h3>
        <button @click="emit('close')" class="p-1.5 text-slate-400 hover:bg-auflage rounded-xl"><X class="w-5 h-5" /></button>
      </div>

      <div class="p-5 space-y-4">
        <p class="text-sm text-leise leading-relaxed">{{ t('projects.planFeed.intro') }}</p>
        <!-- Warum EIN Link für beide Pläne eines Typs. Ohne diesen Satz
             sucht jemand mit zwei Betreuungsplänen den zweiten Link. -->
        <p class="text-sm text-leise leading-relaxed">{{ t('projects.planFeed.bundleHint') }}</p>

        <div v-if="loading" class="flex justify-center py-6"><Loader2 class="w-6 h-6 animate-spin text-marke" /></div>

        <template v-else-if="url">
          <div class="flex items-center gap-2">
            <input :value="url" readonly @focus="$event.target.select()"
              class="flex-1 min-w-0 px-3 py-2 text-xs font-mono bg-vertieft border border-linie rounded-xl text-schrift focus:outline-none" />
            <BaseButton @click="copy" class="shrink-0" groesse="klein">
              <Check v-if="copied" class="w-3.5 h-3.5" /><Copy v-else class="w-3.5 h-3.5" />
              {{ copied ? t('projects.planFeed.copied') : t('projects.planFeed.copy') }}
            </BaseButton>
          </div>

          <!-- Beim Betreuungsplan steht hier die schärfere Warnung: Der Link
               sagt, wo ein Kind schläft. -->
          <div class="flex items-start gap-2 rounded-xl border p-3 text-xs font-medium"
            :class="istBetreuung
              ? 'border-rose-200 dark:border-rose-800/60 bg-rose-50 dark:bg-rose-900/20 text-rose-800 dark:text-rose-300'
              : 'border-amber-200 dark:border-amber-800/60 bg-amber-50 dark:bg-amber-900/20 text-amber-800 dark:text-amber-300'">
            <AlertTriangle class="w-4 h-4 shrink-0 mt-0.5" />
            <span>{{ t(`projects.planFeed.secretHint.${type}`) }} {{ t('projects.planFeed.delayHint') }}</span>
          </div>

          <label class="flex items-start gap-3 rounded-xl border border-linie p-3 cursor-pointer">
            <input type="checkbox" :checked="mitOrten" :disabled="busy" @change="toggleOrte"
              class="mt-0.5 w-4 h-4 rounded accent-marke" />
            <span class="min-w-0">
              <span class="flex items-center gap-1.5 text-sm font-bold text-schrift">
                <MapPin class="w-4 h-4 text-marke" /> {{ t('projects.planFeed.withPlaces') }}
              </span>
              <span class="block text-xs text-leise leading-relaxed mt-1">{{ t('projects.planFeed.withPlacesHint') }}</span>
            </span>
          </label>

          <div class="flex flex-wrap gap-2">
            <button @click="rotate" :disabled="busy"
              class="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-bold text-marke bg-marke-leise hover:bg-marke-leise rounded-xl transition-colors cursor-pointer">
              <RefreshCcw class="w-3.5 h-3.5" /> {{ t('projects.planFeed.rotate') }}
            </button>
            <button @click="remove" :disabled="busy"
              class="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-bold text-rose-600 dark:text-rose-400 bg-rose-50 dark:bg-rose-900/20 hover:bg-rose-100 dark:hover:bg-rose-900/40 rounded-xl transition-colors cursor-pointer">
              <Trash2 class="w-3.5 h-3.5" /> {{ t('projects.planFeed.remove') }}
            </button>
          </div>
        </template>

        <template v-else>
          <label class="flex items-start gap-3 rounded-xl border border-linie p-3 cursor-pointer">
            <input type="checkbox" v-model="mitOrten" class="mt-0.5 w-4 h-4 rounded accent-marke" />
            <span class="min-w-0">
              <span class="flex items-center gap-1.5 text-sm font-bold text-schrift">
                <MapPin class="w-4 h-4 text-marke" /> {{ t('projects.planFeed.withPlaces') }}
              </span>
              <span class="block text-xs text-leise leading-relaxed mt-1">{{ t('projects.planFeed.withPlacesHint') }}</span>
            </span>
          </label>

          <BaseButton @click="create" :disabled="busy"
            groesse="normal">
            <Rss class="w-4 h-4" /> {{ t('projects.planFeed.create') }}
          </BaseButton>
        </template>
      </div>

      <div class="flex justify-end gap-2 p-5 border-t border-linie">
        <button @click="emit('close')" class="px-4 py-2 text-fliess hover:bg-auflage rounded-xl text-sm font-bold">{{ t('common.close') }}</button>
      </div>
    </div>
  </div>
</template>
