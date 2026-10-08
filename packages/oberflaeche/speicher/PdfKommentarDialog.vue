<script setup>
/*
 * Einen Kommentar an einer Anmerkung schreiben, ändern oder löschen
 * (docs/plan-pdf-bearbeiten.md, Schritt 3). Nur die Oberfläche: Was mit dem
 * Text geschieht, entscheidet der PdfBetrachter.
 */
import { ref, onMounted, nextTick } from 'vue';
import { useI18n } from 'vue-i18n';
import BaseModal from '../base/BaseModal.vue';
import BaseButton from '../base/BaseButton.vue';

const props = defineProps({
  text: { type: String, default: '' },
  // Gibt es schon einen Kommentar? Dann auch „Löschen".
  vorhanden: { type: Boolean, default: false },
});
const emit = defineEmits(['speichern', 'loeschen', 'close']);
const { t } = useI18n();

const eingabe = ref(props.text);
const feld = ref(null);
onMounted(async () => {
  await nextTick();
  feld.value?.focus();
});
</script>

<template>
  <BaseModal :title="t(vorhanden ? 'common.pdf.kommentarBearbeiten' : 'common.pdf.kommentarNeu')" @close="emit('close')">
    <textarea
      ref="feld"
      v-model="eingabe"
      rows="5"
      class="w-full px-3 py-2 bg-vertieft border border-linie rounded-xl focus:outline-none focus:ring-2 focus:ring-marke text-sm text-schrift resize-y"
      :aria-label="t('common.pdf.kommentar')"
      :placeholder="t('common.pdf.kommentarPlatzhalter')"
      @keydown.ctrl.enter="eingabe.trim() && emit('speichern', eingabe.trim())"
      @keydown.meta.enter="eingabe.trim() && emit('speichern', eingabe.trim())"
    ></textarea>
    <template #footer>
      <BaseButton v-if="vorhanden" variant="danger" class="mr-auto" @click="emit('loeschen')">{{ t('common.delete') }}</BaseButton>
      <BaseButton variant="secondary" @click="emit('close')">{{ t('common.cancel') }}</BaseButton>
      <BaseButton :disabled="!eingabe.trim()" @click="emit('speichern', eingabe.trim())">{{ t('common.save') }}</BaseButton>
    </template>
  </BaseModal>
</template>
