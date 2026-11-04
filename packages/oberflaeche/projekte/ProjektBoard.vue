<script setup>
// Der Reiter „Board": das eine Board des Hubs. Es gibt
// keines anzulegen und keines zu löschen -- jeder Hub hat genau eines, und
// alle Mitglieder zeichnen darauf (projektwhiteboard.rs führt ihre
// Fassungen zusammen). Hier steht nur der Einstieg; gezeichnet wird auf der
// ganzen Fläche (WhiteboardAnsicht).
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { Presentation, Loader2 } from 'lucide-vue-next';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import { useToast } from '@oberflaeche/composables/useToast';
import { formatDateTime } from '@oberflaeche/shared/date';
import WhiteboardAnsicht from './whiteboard/WhiteboardAnsicht.vue';
import { api } from './umgebung';

const props = defineProps({
  projectId: { type: [Number, String], required: true },
});

const { t } = useI18n();
const toast = useToast();

const offen = ref(null); // das geladene Board, solange es offen ist
const laedt = ref(false);
const geaendert = ref('');

async function oeffnen() {
  laedt.value = true;
  try {
    offen.value = (await api.getBoard(props.projectId)).data;
  } catch {
    toast.error(t('planning.whiteboard.ladenFehler'));
  } finally {
    laedt.value = false;
  }
}

async function stand() {
  try {
    geaendert.value = (await api.getBoard(props.projectId)).data.geaendert ?? '';
  } catch {
    geaendert.value = '';
  }
}

function zu() {
  offen.value = null;
  stand();
}

stand();
</script>

<template>
  <div class="karte shadow-sm p-6 flex flex-col items-center text-center gap-3">
    <div class="w-14 h-14 rounded-2xl bg-marke-leise flex items-center justify-center">
      <Presentation class="w-7 h-7 text-marke" />
    </div>
    <h3 class="font-extrabold text-lg text-schrift">{{ t('planning.board.titel') }}</h3>
    <p class="text-sm text-leise max-w-md">{{ t('planning.board.hinweis') }}</p>
    <p v-if="geaendert" class="text-xs text-leise">{{ t('planning.board.geaendert', { zeit: formatDateTime(geaendert) }) }}</p>
    <BaseButton @click="oeffnen" :disabled="laedt">
      <Loader2 v-if="laedt" class="w-4 h-4 animate-spin" />
      <Presentation v-else class="w-4 h-4" />
      {{ t('planning.board.oeffnen') }}
    </BaseButton>
  </div>
  <WhiteboardAnsicht v-if="offen" :project-id="projectId" :start="offen" @zurueck="zu" />
</template>
