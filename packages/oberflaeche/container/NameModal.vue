<script setup>
// Anlegen-/Umbenennen-Dialog für Behälter: ein Namensfeld, Fehlerzeile,
// Abbrechen/Bestätigen. Validierung und Speichern bleiben bei der Ansicht
// (submit liefert den getrimmten Namen).
import { ref, useId } from 'vue';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import BaseModal from '@oberflaeche/base/BaseModal.vue';

const props = defineProps({
  title: { type: String, required: true },
  label: { type: String, required: true },
  placeholder: { type: String, default: '' },
  submitLabel: { type: String, required: true },
  cancelLabel: { type: String, required: true },
  initial: { type: String, default: '' },
  error: { type: String, default: '' },
});

defineEmits(['submit', 'close']);
const name = ref(props.initial);
const formular = useId();
</script>

<template>
  <BaseModal :title="title" @close="$emit('close')">
    <form :id="formular" @submit.prevent="$emit('submit', name.trim())" class="space-y-2">
      <label class="block text-sm font-bold text-fliess">{{ label }}</label>
      <input
        v-model="name"
        type="text"
        :placeholder="placeholder"
        required
        autofocus
        class="w-full px-4 py-3 bg-vertieft border border-linie rounded-xl focus:outline-none focus:ring-2 focus:ring-marke text-sm font-semibold text-schrift transition-all"
      />
      <p v-if="error" class="text-sm text-rose-600 font-bold mt-1">{{ error }}</p>
    </form>

    <template #footer>
      <BaseButton variant="secondary" @click="$emit('close')">{{ cancelLabel }}</BaseButton>
      <BaseButton type="submit" :form="formular">{{ submitLabel }}</BaseButton>
    </template>
  </BaseModal>
</template>
