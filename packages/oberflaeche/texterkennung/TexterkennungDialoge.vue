<script setup>
/*
 * Die beiden Fortschritts- und Frage-Dialoge der Texterkennung, für jede
 * Hülle der Dateifläche gleich. Der Zustand kommt aus
 * `useTexterkennungImSpeicher` -- siehe dort.
 */
import DocumentIntakeModal from './DocumentIntakeModal.vue';

defineProps({
  texterkennung: { type: Object, required: true },
});
</script>

<template>
  <DocumentIntakeModal
    v-if="texterkennung.frage.value || texterkennung.intakeStand.value"
    :frage="texterkennung.frage.value"
    :vorschlag="texterkennung.vorschlag.value"
    :stand="texterkennung.intakeStand.value"
    @einzeln="texterkennung.einzeln()"
    @zusammen="(name) => texterkennung.zusammen(name)"
    @abbrechen="texterkennung.umwandlungAbbrechen()"
  />

  <DocumentIntakeModal
    v-if="texterkennung.ocrStand.value"
    modus="pdf"
    :stand="texterkennung.ocrStand.value"
    @abbrechen="texterkennung.ocrAbbrechen()"
  />
</template>
