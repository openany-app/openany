<script setup>
// Geteilte Kalender im Reiter „Planung" (docs/plan-kalender-e2e.md, 4c).
//
// Jedes Mitglied kann einen eigenen Kalender teilen; die anderen sehen die
// nächsten Termine daraus -- nur lesen, geschrieben wird beim Besitzer.
// Verschlüsselt mit dem Projektschlüssel; das erledigt die App. Bietet die
// Datenquelle das nicht an (lokale Projekte), bleibt der Abschnitt weg.
import { computed, onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { CalendarDays, Plus, Trash2, Loader2 } from 'lucide-vue-next';
import BaseModal from '@oberflaeche/base/BaseModal.vue';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import { useToast } from '@oberflaeche/composables/useToast';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import { expandRecurringEvents } from '@oberflaeche/shared/expandEvents';
import { formatDate, formatWeekdayTime } from '@oberflaeche/shared/date';
import { api } from './umgebung';

const props = defineProps({
  projectId: { type: [Number, String], required: true },
});

const { t } = useI18n();
const toast = useToast();
const { confirmDialog } = useConfirm();

const da = Boolean(api.getProjectCalendars);
const sichtbar = ref(da);
const laedt = ref(false);
const kalender = ref([]);

const teilenOffen = ref(false);
const eigene = ref([]);
const gewaehlt = ref('');
const teilt = ref(false);

/** So viele kommende Termine je Kalender. */
const NAECHSTE = 5;

async function laden() {
  laedt.value = true;
  try {
    kalender.value = (await api.getProjectCalendars(props.projectId)).data.kalender ?? [];
  } catch {
    // Kein Server-Projekt (oder offline ohne Ablage): Abschnitt weglassen.
    sichtbar.value = false;
  } finally {
    laedt.value = false;
  }
}

/** Die nächsten Termine eines Kalenders, Serien ausgefaltet. */
function naechste(k) {
  const jetzt = new Date();
  return expandRecurringEvents(k.termine ?? [])
    .filter((e) => new Date(e.end_date || e.start_date) >= jetzt)
    .sort((a, b) => new Date(a.start_date) - new Date(b.start_date))
    .slice(0, NAECHSTE);
}

const mitTerminen = computed(() => kalender.value.map((k) => ({ ...k, naechste: naechste(k) })));

/** Schon geteilte eigene Kalender stehen nicht noch einmal zur Wahl. */
async function teilenOeffnen() {
  try {
    eigene.value = (await api.getOwnCalendars()).data;
  } catch {
    eigene.value = [];
  }
  gewaehlt.value = eigene.value[0]?.id ?? '';
  teilenOffen.value = true;
}

async function teilen() {
  if (!gewaehlt.value) return;
  teilt.value = true;
  try {
    await api.shareCalendar(props.projectId, gewaehlt.value);
    teilenOffen.value = false;
    await laden();
  } catch {
    toast.error(t('planning.kalender.teilenFehler'));
  } finally {
    teilt.value = false;
  }
}

async function beenden(k) {
  if (!(await confirmDialog(t('planning.kalender.beendenFrage', { name: k.name }), {
    title: t('planning.kalender.beenden'), confirmLabel: t('planning.kalender.beenden'),
  }))) return;
  try {
    await api.unshareCalendar(props.projectId, k.uuid);
    await laden();
  } catch {
    toast.error(t('planning.kalender.beendenFehler'));
  }
}

const zeit = (e) => (e.is_all_day ? formatDate(e.start_date) : formatWeekdayTime(e.start_date));

onMounted(() => { if (da) laden(); });
</script>

<template>
  <div v-if="sichtbar" class="karte shadow-sm p-6 space-y-4 mt-6">
    <div class="flex items-center gap-2">
      <CalendarDays class="w-5 h-5 text-marke" />
      <h3 class="font-extrabold text-lg text-schrift">{{ t('planning.kalender.titel') }}</h3>
      <BaseButton @click="teilenOeffnen" groesse="kopf" class="ml-auto"><Plus class="w-4 h-4" /> {{ t('planning.kalender.teilen') }}</BaseButton>
    </div>

    <div v-if="laedt && !kalender.length" class="py-6 text-center text-slate-400"><Loader2 class="w-5 h-5 animate-spin mx-auto" /></div>
    <p v-else-if="!kalender.length" class="text-sm text-leise">{{ t('planning.kalender.leer') }}</p>

    <div v-for="k in mitTerminen" :key="k.uuid" class="space-y-2">
      <div class="flex items-center gap-2">
        <span class="w-3 h-3 rounded-full shrink-0" :style="{ backgroundColor: k.color }"></span>
        <span class="font-bold text-schrift truncate">{{ k.name }}</span>
        <span class="text-xs text-leise shrink-0">{{ k.eigen ? t('planning.kalender.vonDir') : t('planning.kalender.von', { name: k.von_name }) }}</span>
        <button v-if="k.eigen" @click="beenden(k)" class="ml-auto p-1.5 text-slate-400 hover:text-rose-500 hover:bg-rose-50 dark:hover:bg-rose-900/20 rounded-xl" :title="t('planning.kalender.beenden')"><Trash2 class="w-4 h-4" /></button>
      </div>
      <p v-if="!k.naechste.length" class="pl-5 text-xs text-leise">{{ t('planning.kalender.keineTermine') }}</p>
      <ul v-else class="pl-5 space-y-1">
        <li v-for="e in k.naechste" :key="e.id" class="text-sm flex gap-3 min-w-0">
          <span class="text-leise shrink-0 tabular-nums">{{ zeit(e) }}</span>
          <span class="text-fliess truncate">{{ e.title }}</span>
        </li>
      </ul>
    </div>

    <BaseModal v-if="teilenOffen" :title="t('planning.kalender.teilenTitel')" size="sm" @close="teilenOffen = false">
      <p class="text-sm text-leise mb-3">{{ t('planning.kalender.teilenHinweis') }}</p>
      <select v-if="eigene.length" v-model="gewaehlt" class="w-full rounded-xl border border-linie p-2 dark:bg-slate-800 dark:text-white">
        <option v-for="c in eigene" :key="c.id" :value="c.id">{{ c.name }}</option>
      </select>
      <p v-else class="text-sm text-leise">{{ t('planning.kalender.keinEigener') }}</p>
      <template #footer>
        <BaseButton variant="ghost" @click="teilenOffen = false">{{ t('common.cancel') }}</BaseButton>
        <BaseButton :loading="teilt" :disabled="!gewaehlt" @click="teilen">{{ t('planning.kalender.teilen') }}</BaseButton>
      </template>
    </BaseModal>
  </div>
</template>
