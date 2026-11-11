<script setup>
// Termin-Details (Nur-Lesen) mit Bearbeiten/Löschen, oben ein Balken in der
// Kalenderfarbe.
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { Clock, MapPin, Trash2, Edit3, Bell, Repeat } from 'lucide-vue-next';
import BaseModal from '@oberflaeche/base/BaseModal.vue';
import { formatDate, formatDateTime } from '@oberflaeche/shared/date';
import { regelText } from '@oberflaeche/shared/regelText';

const props = defineProps({
  event: { type: Object, required: true },
  /*
   * Der NAME des Abo-Kalenders, wenn dieser Termin aus einem stammt --
   * sonst `null`.
   *
   * Aus einem Abo laesst sich nichts aendern: Jeder Abruf schreibt alle
   * Termine aus dem Feed neu. Was hier geaendert wuerde, waere beim naechsten
   * Abruf fort -- ohne Fehler und ohne Meldung. Hier steht, WARUM, statt nur
   * die Knoepfe zu verstecken.
   */
  abo: { type: String, default: null },
  // Aus einem in einem Projekt geteilten Kalender (4c): { projekt, von } --
  // nur lesen, geschrieben wird beim Besitzer.
  geteilt: { type: Object, default: null },
});
const emit = defineEmits(['close', 'edit', 'delete']);

const { t, locale } = useI18n();

// Die Wiederholung in Worten, auch was ein anderes Programm genauer sagt.
const wiederholung = computed(() => regelText(props.event.original_event || props.event, t, locale.value));

// Die Erinnerung (K3) in Worten -- dieselben wie im Termin-Dialog.
const erinnerung = computed(() => {
  const m = props.event.reminder;
  if (m === undefined || m === null || m === '') return '';
  if (m === '0') return t('calendar.eventModal.reminderAtStart');
  if (m === '60') return t('calendar.eventModal.reminderHour');
  if (m === '1440') return t('calendar.eventModal.reminderDay');
  return t('calendar.eventModal.reminderMinutes', { n: m });
});

</script>

<template>
  <BaseModal :title="event.title" @close="emit('close')">
    <template #oben>
      <div class="h-3 w-full shrink-0" :style="{ backgroundColor: event.color }"></div>
    </template>

    <div class="space-y-3 text-sm text-fliess">
      <p v-if="!event.is_all_day"><Clock class="w-4 h-4 inline mr-2 text-marke"/> {{ formatDateTime(event.start_date) }} - {{ formatDateTime(event.end_date) }}</p>
      <p v-else><Clock class="w-4 h-4 inline mr-2 text-marke"/> {{ formatDate(event.start_date) }} - {{ formatDate(event.end_date) }}</p>
      <p v-if="event.is_all_day" class="text-xs text-slate-400 ml-6">{{ t('calendar.eventDetail.allDay') }}</p>
      <p v-if="event.location" class="break-words"><MapPin class="w-4 h-4 inline mr-2 text-marke"/> {{ event.location }}</p>
      <p v-if="wiederholung"><Repeat class="w-4 h-4 inline mr-2 text-marke"/> {{ wiederholung }}</p>
      <p v-else-if="event.series && event.occurrence" class="text-xs text-leise"><Repeat class="w-4 h-4 inline mr-2"/> {{ t('calendar.recurrence.moved') }}</p>
      <p v-if="erinnerung"><Bell class="w-4 h-4 inline mr-2 text-marke"/> {{ erinnerung }}</p>
      <!-- pre-line: Zeilenumbrüche der Beschreibung (aus ICS oder dem Dialog) bleiben stehen. -->
      <p v-if="event.description" class="bg-slate-50 dark:bg-slate-800 p-3 rounded-xl whitespace-pre-line break-words">{{ event.description }}</p>
    </div>

    <template #footer>
      <p v-if="abo" class="px-1 py-2 text-sm text-leise">{{ t('calendar.detail.fromSubscription', { name: abo }) }}</p>
      <p v-else-if="geteilt" class="px-1 py-2 text-sm text-leise">{{ t('calendar.detail.geteilt', { projekt: geteilt.projekt, von: geteilt.von }) }}</p>
      <template v-else>
        <button @click="emit('delete', event)" class="px-4 py-2 text-red-600 font-semibold hover:bg-red-50 dark:hover:bg-red-900/30 rounded-xl"><Trash2 class="w-4 h-4 inline mr-1"/> {{ t('common.delete') }}</button>
        <button @click="emit('edit', event)" class="px-4 py-2 text-marke bg-marke-leise rounded-xl font-semibold"><Edit3 class="w-4 h-4 inline mr-1"/> {{ t('common.edit') }}</button>
      </template>
    </template>
  </BaseModal>
</template>
