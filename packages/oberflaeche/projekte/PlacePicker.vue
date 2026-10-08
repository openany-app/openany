<script setup>
// Einen Ort des Projekts auswählen – für alles, was auf einen Ort zeigen darf
// (Meilenstein, Kanban-Karte).
//
// Die Orte kommen aus den Orte-Sammlungen des Projekts, quer über die Gruppen
// (GET projects/{p}/places). Angelegt werden sie weiterhin dort und nur dort:
// Ein zweiter Anlegeweg mit eigener Karte wäre eine zweite Vorstellung davon,
// was ein Ort ist. Ist noch keiner da, führt ein Hinweis dorthin.
//
// Ein Raum ist kein Ort: „R 204" gehört in die Beschreibung, nicht hierher.
import { ref, computed, onMounted, defineAsyncComponent } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from './umgebung';
import { MapPin, X, Loader2 } from 'lucide-vue-next';

// Erst laden, wenn wirklich ein Ort gewählt ist. Der Picker sitzt im
// Karten-Dialog des Boards, und das Board hängt fest am Planung-Reiter –
// mit einem gewöhnlichen Import käme Leaflet (rund 50 kB gepackt) bei
// jedem Öffnen der Planung mit, auch in einem Projekt ohne einen einzigen
// Ort. Dieselbe Überlegung wie bei den Detailansichten in
// planningContainers.js.
const LeafletMap = defineAsyncComponent(() => import('./LeafletMap.vue'));

const { t } = useI18n();
const props = defineProps({
  projectId: { type: [Number, String], required: true },
  // Ausgewählte Ort-Id oder null. Über v-model gebunden.
  modelValue: { type: [Number, String], default: null },
});
const emit = defineEmits(['update:modelValue']);

const orte = ref([]);
const laedt = ref(true);
const fehler = ref('');

onMounted(async () => {
  try {
    orte.value = (await api.getProjectPlaces(props.projectId)).data;
  } catch (e) {
    fehler.value = t('projects.places.pickerLoadFailed');
  } finally {
    laedt.value = false;
  }
});

const gewaehlt = computed(() => orte.value.find((o) => o.id === props.modelValue) || null);

// Der gesetzte Ort steht nicht in der Liste – seine Sammlung liegt im
// Papierkorb. Der Verweis besteht weiter (die Zeile steht ja noch), nur neu
// setzen lässt er sich nicht. Ihn stumm zu verschlucken wäre falsch: Beim
// Speichern ginge die alte Id mit, und die Prüfung lehnte sie ab – mit einem
// Fehler an einer Änderung, die mit dem Ort nichts zu tun hat. Also
// ausdrücklich anzeigen und wegräumbar machen.
const verwaist = computed(() => props.modelValue !== null && !laedt.value && !fehler.value && gewaehlt.value === null);

// Der Kartenausschnitt erwartet { lat, lng } – nicht den ganzen Ort.
const punkt = computed(() => (gewaehlt.value ? { lat: gewaehlt.value.lat, lng: gewaehlt.value.lng } : null));

const waehle = (event) => {
  const wert = event.target.value;
  // Die Kennung so, wie sie in der Liste steht: in der Webapp eine Zahl, in
  // der App eine uuid (projekte/umgebung.js).
  emit('update:modelValue', wert === '' ? null : (orte.value.find((x) => String(x.id) === wert)?.id ?? null));
};
</script>

<template>
  <div>
    <label class="block text-xs font-bold text-leise uppercase tracking-wide">
      {{ t('projects.places.pickerLabel') }}
    </label>

    <p v-if="laedt" class="mt-1 flex items-center gap-2 text-sm text-slate-400">
      <Loader2 class="w-4 h-4 animate-spin" /> {{ t('common.loading') }}
    </p>
    <p v-else-if="fehler" class="mt-1 text-sm font-medium text-rose-500">{{ fehler }}</p>
    <p v-else-if="orte.length === 0 && !verwaist" class="mt-1 text-sm text-slate-400">
      {{ t('projects.places.pickerEmpty') }}
    </p>

    <div v-else class="mt-1 flex items-center gap-2">
      <select
        :value="modelValue ?? ''"
        @change="waehle"
        class="flex-1 min-w-0 px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift"
      >
        <option value="">{{ t('projects.places.pickerNone') }}</option>
        <option v-if="verwaist" :value="modelValue">{{ t('projects.places.pickerUnavailable') }}</option>
        <option v-for="ort in orte" :key="ort.id" :value="ort.id">{{ ort.name }}</option>
      </select>
      <button
        v-if="modelValue"
        type="button"
        @click="emit('update:modelValue', null)"
        class="p-2 text-slate-400 hover:text-rose-500 hover:bg-rose-50 dark:hover:bg-rose-900/20 rounded-xl shrink-0"
        :title="t('projects.places.pickerClear')"
      >
        <X class="w-4 h-4" />
      </button>
    </div>

    <!-- Nicht bedienbarer Ausschnitt, wie in der Orte-Übersicht: Er zeigt,
         dass der richtige Punkt gemeint ist, ohne zum Verschieben einzuladen –
         verschoben wird ein Ort in seiner Sammlung. -->
    <div v-if="gewaehlt" class="mt-2 flex items-center gap-2">
      <!-- Nicht kleiner: Der OpenStreetMap-Hinweis unten rechts ist Pflicht
           und verdeckte bei 96×64 fast die halbe Karte. -->
      <div class="w-40 h-24 rounded-xl overflow-hidden border border-linie shrink-0">
        <LeafletMap :model-value="punkt" :zoom="14" />
      </div>
      <span class="flex items-center gap-1.5 text-xs font-medium text-leise min-w-0">
        <MapPin class="w-3.5 h-3.5 text-marke shrink-0" />
        <span class="truncate">{{ gewaehlt.name }}</span>
      </span>
    </div>
  </div>
</template>
