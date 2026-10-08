<script setup>
// Kalender importieren: entweder eine .ics-Datei einlesen oder eine
// WebCal-URL abonnieren. Ziel-Kalender wählbar; die Import-Aufrufe macht
// der Eltern-View.
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import BaseModal from '@oberflaeche/base/BaseModal.vue';
import { File, Globe } from 'lucide-vue-next';
import BaseButton from '@oberflaeche/base/BaseButton.vue';

const { t } = useI18n();

const props = defineProps({
  calendars: { type: Array, required: true },
  loading: { type: Boolean, default: false },
});
const emit = defineEmits(['import-file', 'import-url', 'close']);

const importType = ref('url');
const importUrl = ref('');
// Standardmäßig der erste Kalender.
const selectedCalendarId = ref(props.calendars.length ? props.calendars[0].id : '');

const onFile = (e) => {
  const file = e.target.files[0];
  if (!file) return;
  emit('import-file', { calendarId: selectedCalendarId.value, file });
};

const onUrl = () => {
  if (!importUrl.value || !selectedCalendarId.value) return;
  emit('import-url', { calendarId: selectedCalendarId.value, url: importUrl.value });
};
</script>

<template>
  <!-- persistent: schließt nicht per Overlay-Klick (Auswahl bleibt erhalten) -->
  <BaseModal :title="t('calendar.importModal.title')" size="sm" persistent @close="emit('close')">
    <div class="flex gap-4 mb-4 border-b border-linie">
      <button @click="importType = 'url'" class="pb-2 text-sm font-bold border-b-2 transition-colors flex items-center gap-1.5" :class="importType === 'url' ? 'border-marke text-marke' : 'border-transparent text-slate-500'"><Globe class="w-4 h-4" /> {{ t('calendar.importModal.tabUrl') }}</button>
      <button @click="importType = 'file'" class="pb-2 text-sm font-bold border-b-2 transition-colors flex items-center gap-1.5" :class="importType === 'file' ? 'border-marke text-marke' : 'border-transparent text-slate-500'"><File class="w-4 h-4" /> {{ t('calendar.importModal.tabFile') }}</button>
    </div>

    <div class="mb-4">
      <label class="block text-sm font-semibold mb-1 dark:text-slate-300">{{ t('calendar.importModal.targetCalendar') }}</label>
      <select v-model="selectedCalendarId" class="w-full rounded-xl border border-linie p-2 dark:bg-slate-800 dark:text-white">
        <option v-for="c in calendars" :key="c.id" :value="c.id">{{ c.name }}</option>
      </select>
    </div>

    <div v-if="importType === 'file'" class="mb-2">
      <input type="file" accept=".ics" @change="onFile" class="block w-full text-sm text-slate-500 file:mr-4 file:py-2 file:px-4 file:rounded-xl file:border-0 file:text-sm file:font-semibold file:bg-marke-leise file:text-marke hover:file:bg-marke-leise">
    </div>

    <div v-else class="mb-2 space-y-4">
      <div>
        <label class="block text-sm font-semibold mb-1 dark:text-slate-300">{{ t('calendar.importModal.webcalUrl') }}</label>
        <input v-model="importUrl" type="url" placeholder="https://..." class="w-full rounded-xl border border-linie p-2 dark:bg-slate-800 dark:text-white">
      </div>
      <BaseButton @click="onUrl" :disabled="loading || !importUrl" class="w-full" groesse="normal">{{ t('calendar.importModal.subscribeUrl') }}</BaseButton>
    </div>

    <template #footer>
      <button @click="emit('close')" class="px-4 py-2 text-sm font-semibold text-fliess bg-auflage rounded-xl hover:bg-slate-200 dark:hover:bg-slate-700">{{ t('common.close') }}</button>
    </template>
  </BaseModal>
</template>
