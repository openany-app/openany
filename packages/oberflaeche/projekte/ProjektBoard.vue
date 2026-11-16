<script setup>
// Das eine Board des Hubs. Es gibt keines anzulegen und keines zu löschen --
// jeder Hub hat genau eines, und alle Mitglieder zeichnen darauf
// (projektwhiteboard.rs führt ihre Fassungen zusammen). Der Reiter „Board"
// öffnet es gleich auf der ganzen Fläche (WhiteboardAnsicht); „Zurück" führt
// in den Reiter, aus dem man kam (ProjektSeite).
import { ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { Loader2 } from 'lucide-vue-next';
import { useToast } from '@oberflaeche/composables/useToast';
import WhiteboardAnsicht from './whiteboard/WhiteboardAnsicht.vue';
import { api } from './umgebung';

const props = defineProps({
  projectId: { type: [Number, String], required: true },
});
const emit = defineEmits(['zu']);

const { t } = useI18n();
const toast = useToast();

const offen = ref(null); // das geladene Board

onMounted(async () => {
  try {
    offen.value = (await api.getBoard(props.projectId)).data;
  } catch {
    toast.error(t('planning.whiteboard.ladenFehler'));
    emit('zu');
  }
});
</script>

<template>
  <WhiteboardAnsicht v-if="offen" :project-id="projectId" :start="offen" @zurueck="emit('zu')" />
  <div v-else class="rahmen-ueber-leiste z-[90] flex items-center justify-center bg-boden" role="status"
       :aria-label="t('planning.board.titel')">
    <Loader2 class="w-8 h-8 animate-spin text-marke" />
  </div>
</template>
