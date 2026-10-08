<script setup>
// Termin-Details (Nur-Lesen) mit Bearbeiten/Löschen. Eigene Modal-Hülle
// wegen des farbigen Kopfbalkens in der Kalenderfarbe.
import { useI18n } from 'vue-i18n';
import { Clock, Trash2, Edit3, X } from 'lucide-vue-next';
import { formatDate, formatDateTime } from '@oberflaeche/shared/date';

defineProps({
  event: { type: Object, required: true },
  /*
   * Der NAME des Abo-Kalenders, wenn dieser Termin aus einem stammt --
   * sonst `null`.
   *
   * Aus einem Abo laesst sich nichts aendern: Drueben loescht `performSync()`
   * bei jedem Abruf alle Termine und legt sie aus dem Feed neu an. Was hier
   * geaendert wuerde, waere beim naechsten Abgleich fort -- ohne Fehler und
   * ohne Meldung. Der Server weist es seit dem 17.09.2026 ab; hier steht,
   * WARUM, statt nur die Knoepfe zu verstecken.
   */
  abo: { type: String, default: null },
});
const emit = defineEmits(['close', 'edit', 'delete']);

const { t } = useI18n();

</script>

<template>
  <div class="fixed inset-0 z-[100] flex items-center justify-center p-4 bg-slate-900/40 backdrop-blur-sm" @click.self="emit('close')">
    <div class="bg-flaeche w-full max-w-md rounded-xl overflow-hidden shadow-2xl">
      <div class="h-3 w-full" :style="{ backgroundColor: event.color }"></div>
      <div class="p-6">
        <div class="flex justify-between items-start mb-4">
          <h2 class="text-2xl font-bold dark:text-white">{{ event.title }}</h2>
          <button @click="emit('close')" class="p-1 rounded-full bg-auflage dark:text-white"><X class="w-5 h-5"/></button>
        </div>
        <div class="space-y-3 text-sm text-fliess">
          <p v-if="!event.is_all_day"><Clock class="w-4 h-4 inline mr-2 text-marke"/> {{ formatDateTime(event.start_date) }} - {{ formatDateTime(event.end_date) }}</p>
          <p v-else><Clock class="w-4 h-4 inline mr-2 text-marke"/> {{ formatDate(event.start_date) }} - {{ formatDate(event.end_date) }}</p>
          <p v-if="event.is_all_day" class="text-xs text-slate-400 ml-6">{{ t('calendar.eventDetail.allDay') }}</p>
          <p v-if="event.description" class="bg-slate-50 dark:bg-slate-800 p-3 rounded-xl">{{ event.description }}</p>
        </div>
        <div class="mt-6 flex justify-end gap-2 border-t border-linie pt-4">
          <p v-if="abo" class="px-1 py-2 text-sm text-leise">{{ t('calendar.detail.fromSubscription', { name: abo }) }}</p>
          <template v-else>
            <button @click="emit('delete', event)" class="px-4 py-2 text-red-600 font-semibold hover:bg-red-50 dark:hover:bg-red-900/30 rounded-xl"><Trash2 class="w-4 h-4 inline mr-1"/> {{ t('common.delete') }}</button>
            <button @click="emit('edit', event)" class="px-4 py-2 text-marke bg-marke-leise rounded-xl font-semibold"><Edit3 class="w-4 h-4 inline mr-1"/> {{ t('common.edit') }}</button>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>
