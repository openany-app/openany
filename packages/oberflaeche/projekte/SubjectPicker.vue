<script setup>
// Ein Fach des Projekts auswählen – für alles, was auf ein Fach zeigen darf
// (Kanban-Karte, Meilenstein, Wochenplan-Slot). Angelegt werden Fächer
// weiterhin nur im Seitenpanel der Stundenplan-Ansicht (SubjectsPanel.vue);
// ein zweiter Anlegeweg hier wäre eine zweite Vorstellung davon, was ein Fach
// ist – dieselbe Überlegung wie bei PlacePicker.vue.
import { ref, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from './umgebung';
import { BookOpen, X, Loader2 } from 'lucide-vue-next';

const { t } = useI18n();
const props = defineProps({
  projectId: { type: [Number, String], required: true },
  modelValue: { type: [Number, String], default: null },
});
const emit = defineEmits(['update:modelValue']);

const faecher = ref([]);
const laedt = ref(true);
const fehler = ref('');

onMounted(async () => {
  try {
    faecher.value = (await api.getSubjects(props.projectId)).data;
  } catch (e) {
    fehler.value = t('projects.subjects.pickerLoadFailed');
  } finally {
    laedt.value = false;
  }
});

const gewaehlt = computed(() => faecher.value.find((f) => f.id === props.modelValue) || null);

// Wie bei PlacePicker: ein gesetztes, aber nicht mehr gelistetes Fach
// (gelöscht) ausdrücklich zeigen statt stumm zu verschlucken.
const verwaist = computed(() => props.modelValue !== null && !laedt.value && !fehler.value && gewaehlt.value === null);

const waehle = (event) => {
  const wert = event.target.value;
  // Die Kennung so, wie sie in der Liste steht: in der Webapp eine Zahl, in
  // der App eine uuid (projekte/umgebung.js).
  emit('update:modelValue', wert === '' ? null : (faecher.value.find((x) => String(x.id) === wert)?.id ?? null));
};
</script>

<template>
  <div>
    <label class="block text-xs font-bold text-leise uppercase tracking-wide">
      {{ t('projects.subjects.pickerLabel') }}
    </label>

    <p v-if="laedt" class="mt-1 flex items-center gap-2 text-sm text-slate-400">
      <Loader2 class="w-4 h-4 animate-spin" /> {{ t('common.loading') }}
    </p>
    <p v-else-if="fehler" class="mt-1 text-sm font-medium text-rose-500">{{ fehler }}</p>
    <p v-else-if="faecher.length === 0 && !verwaist" class="mt-1 text-sm text-slate-400">
      {{ t('projects.subjects.pickerEmpty') }}
    </p>

    <div v-else class="mt-1 flex items-center gap-2">
      <select
        :value="modelValue ?? ''"
        @change="waehle"
        class="flex-1 min-w-0 px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift"
      >
        <option value="">{{ t('projects.subjects.pickerNone') }}</option>
        <option v-if="verwaist" :value="modelValue">{{ t('projects.subjects.pickerUnavailable') }}</option>
        <option v-for="f in faecher" :key="f.id" :value="f.id">{{ f.name }}</option>
      </select>
      <button
        v-if="modelValue"
        type="button"
        @click="emit('update:modelValue', null)"
        class="p-2 text-slate-400 hover:text-rose-500 hover:bg-rose-50 dark:hover:bg-rose-900/20 rounded-xl shrink-0"
        :title="t('projects.subjects.pickerClear')"
      >
        <X class="w-4 h-4" />
      </button>
    </div>

    <div v-if="gewaehlt" class="mt-2 flex items-center gap-1.5 text-xs font-medium text-leise">
      <BookOpen class="w-3.5 h-3.5 shrink-0" :style="gewaehlt.color ? { color: gewaehlt.color } : null" />
      <span class="truncate">{{ gewaehlt.teacher || t('projects.subjects.noTeacher') }}</span>
    </div>
  </div>
</template>
