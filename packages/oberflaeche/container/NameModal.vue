<script setup>
// Anlegen-/Umbenennen-Dialog für Behälter: ein Namensfeld, Fehlerzeile,
// Abbrechen/Bestätigen. Validierung und Speichern bleiben bei der Ansicht
// (submit liefert den getrimmten Namen).
import { ref } from 'vue';
import { Folder, X } from 'lucide-vue-next';
import BaseButton from '@oberflaeche/base/BaseButton.vue';

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
</script>

<template>
  <div
    class="fixed inset-0 bg-slate-900/60 backdrop-blur-sm flex items-center justify-center p-4 z-[100] animate-modal-in"
    @click.self="$emit('close')"
  >
    <div class="karte p-6 w-full max-w-md shadow-2xl space-y-6">
      <div class="flex items-center justify-between border-b border-linie pb-4">
        <div class="flex items-center gap-3">
          <Folder class="w-6 h-6 text-marke" />
          <h3 class="font-extrabold text-xl text-schrift">{{ title }}</h3>
        </div>
        <button @click="$emit('close')" class="p-1.5 text-slate-400 hover:bg-auflage rounded-xl transition-colors">
          <X class="w-5 h-5" />
        </button>
      </div>

      <form @submit.prevent="$emit('submit', name.trim())" class="space-y-5">
        <div class="space-y-2">
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
        </div>

        <div class="pt-4 flex items-center justify-end gap-3">
          <button
            type="button"
            @click="$emit('close')"
            class="px-5 py-2.5 text-fliess hover:bg-auflage rounded-xl text-sm font-bold transition-colors"
          >
            {{ cancelLabel }}
          </button>
          <BaseButton
            type="submit"
            groesse="normal"
          >
            {{ submitLabel }}
          </BaseButton>
        </div>
      </form>
    </div>
  </div>
</template>
