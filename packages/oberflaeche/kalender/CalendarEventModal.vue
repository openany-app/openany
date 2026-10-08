<script setup>
// Termin anlegen/bearbeiten. Das Formular initialisiert sich beim Öffnen
// aus editEvent (Bearbeiten) bzw. initialDate (Neu) und baut beim Speichern
// das Datum/Zeit-Payload; die API-Aufrufe macht der Eltern-View.
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import BaseModal from '@oberflaeche/base/BaseModal.vue';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import { getLocalDateString } from '@oberflaeche/shared/date';

const { t } = useI18n();

const props = defineProps({
  calendars: { type: Array, required: true },
  initialDate: { type: String, default: '' },   // Vorbelegung für neue Termine
  editEvent: { type: Object, default: null },    // gesetzt = Bearbeiten
  saving: { type: Boolean, default: false },
});
const emit = defineEmits(['save', 'close']);

const isEditing = !!props.editEvent;

// Formular initial aus editEvent bzw. initialDate füllen.
function buildForm() {
  if (props.editEvent) {
    const original = props.editEvent.original_event || props.editEvent;
    const startDt = new Date(original.start_date);
    const endDt = new Date(original.end_date);
    const pad = (n) => n.toString().padStart(2, '0');
    return {
      title: original.title,
      calendar_id: original.calendar_id,
      start_date: original.start_date.split('T')[0],
      end_date: original.end_date.split('T')[0],
      start_time: `${pad(startDt.getHours())}:${pad(startDt.getMinutes())}`,
      end_time: `${pad(endDt.getHours())}:${pad(endDt.getMinutes())}`,
      is_all_day: original.is_all_day,
      rrule: original.rrule || 'NONE',
      description: original.description || '',
      location: original.location || '',
    };
  }
  const dateVal = props.initialDate || getLocalDateString(new Date());
  return {
    title: '',
    calendar_id: props.calendars.length > 0 ? props.calendars[0].id : '',
    start_date: dateVal,
    end_date: dateVal,
    start_time: '12:00',
    end_time: '13:00',
    is_all_day: true,
    rrule: 'NONE',
    description: '',
    location: '',
  };
}

const form = ref(buildForm());

function submit() {
  if (!form.value.title.trim()) return;
  let start = form.value.start_date;
  let end = form.value.end_date;
  if (!form.value.is_all_day) {
    start = `${start} ${form.value.start_time}:00`;
    end = `${end} ${form.value.end_time}:00`;
  } else {
    start = `${start} 00:00:00`;
    end = `${end} 23:59:59`;
  }
  const payload = { ...form.value, start_date: start, end_date: end };
  const eventId = props.editEvent ? (props.editEvent.real_id || props.editEvent.id) : null;
  emit('save', { payload, eventId });
}
</script>

<template>
  <!-- persistent: kein versehentliches Schließen per Overlay-Klick im Formular -->
  <BaseModal :title="isEditing ? t('calendar.eventModal.editTitle') : t('calendar.eventModal.newTitle')" size="lg" persistent @close="emit('close')">
    <form @submit.prevent="submit" class="space-y-4">
      <div>
        <label class="block text-sm font-semibold mb-1 dark:text-slate-300">{{ t('calendar.eventModal.titleLabel') }}</label>
        <input v-model="form.title" required type="text" class="w-full rounded-xl border border-linie p-2 dark:bg-slate-800 dark:text-white" autofocus>
      </div>

      <label class="flex items-center gap-3 cursor-pointer select-none">
        <div class="relative">
          <input type="checkbox" v-model="form.is_all_day" class="sr-only" />
          <div class="w-10 h-5 rounded-full transition-colors" :class="form.is_all_day ? 'bg-marke' : 'bg-slate-300 dark:bg-slate-600'"></div>
          <div class="absolute top-0.5 left-0.5 w-4 h-4 bg-white rounded-full shadow transition-transform" :class="form.is_all_day ? 'translate-x-5' : 'translate-x-0'"></div>
        </div>
        <span class="text-sm font-semibold text-fliess">{{ t('calendar.eventModal.allDay') }}</span>
      </label>

      <div class="grid grid-cols-2 gap-4">
        <div>
          <label class="block text-sm font-semibold mb-1 dark:text-slate-300">{{ t('calendar.eventModal.startDate') }}</label>
          <input v-model="form.start_date" required type="date" class="w-full rounded-xl border border-linie p-2 dark:bg-slate-800 dark:text-white">
        </div>
        <div>
          <label class="block text-sm font-semibold mb-1 dark:text-slate-300">{{ t('calendar.eventModal.endDate') }}</label>
          <input v-model="form.end_date" required type="date" class="w-full rounded-xl border border-linie p-2 dark:bg-slate-800 dark:text-white">
        </div>
      </div>
      <div v-if="!form.is_all_day" class="grid grid-cols-2 gap-4">
        <div>
          <label class="block text-sm font-semibold mb-1 dark:text-slate-300">{{ t('calendar.eventModal.from') }}</label>
          <input v-model="form.start_time" required type="time" class="w-full rounded-xl border border-linie p-2 dark:bg-slate-800 dark:text-white">
        </div>
        <div>
          <label class="block text-sm font-semibold mb-1 dark:text-slate-300">{{ t('calendar.eventModal.to') }}</label>
          <input v-model="form.end_time" required type="time" class="w-full rounded-xl border border-linie p-2 dark:bg-slate-800 dark:text-white">
        </div>
      </div>

      <div>
        <label class="block text-sm font-semibold mb-1 dark:text-slate-300">{{ t('calendar.eventModal.location') }}</label>
        <input v-model="form.location" type="text" class="w-full rounded-xl border border-linie p-2 dark:bg-slate-800 dark:text-white" :placeholder="t('calendar.eventModal.locationPlaceholder')">
      </div>

      <div class="grid grid-cols-2 gap-4">
        <div>
          <label class="block text-sm font-semibold mb-1 dark:text-slate-300">{{ t('calendar.eventModal.calendar') }}</label>
          <select v-model="form.calendar_id" class="w-full rounded-xl border border-linie p-2 dark:bg-slate-800 dark:text-white">
            <option v-for="c in calendars" :key="c.id" :value="c.id">{{ c.name }}</option>
          </select>
        </div>
        <div>
          <label class="block text-sm font-semibold mb-1 dark:text-slate-300">{{ t('calendar.eventModal.recurrence') }}</label>
          <select v-model="form.rrule" class="w-full rounded-xl border border-linie p-2 dark:bg-slate-800 dark:text-white">
            <option value="NONE">{{ t('calendar.eventModal.recurrenceNone') }}</option>
            <option value="DAILY">{{ t('calendar.eventModal.recurrenceDaily') }}</option>
            <option value="WEEKLY">{{ t('calendar.eventModal.recurrenceWeekly') }}</option>
            <option value="MONTHLY">{{ t('calendar.eventModal.recurrenceMonthly') }}</option>
            <option value="YEARLY">{{ t('calendar.eventModal.recurrenceYearly') }}</option>
          </select>
        </div>
      </div>

      <div>
        <label class="block text-sm font-semibold mb-1 dark:text-slate-300">{{ t('calendar.eventModal.notes') }}</label>
        <textarea v-model="form.description" rows="3" class="w-full rounded-xl border border-linie p-2 dark:bg-slate-800 dark:text-white"></textarea>
      </div>
      <div class="flex justify-end gap-2 pt-4">
        <BaseButton type="button" variant="ghost" @click="emit('close')">{{ t('common.cancel') }}</BaseButton>
        <BaseButton type="submit" :loading="saving" :disabled="!form.title">{{ t('common.save') }}</BaseButton>
      </div>
    </form>
  </BaseModal>
</template>
