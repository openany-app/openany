<script setup>
// Rendert den globalen Bestätigungs-Dialog aus useConfirm (einmal in
// App.vue eingebunden). Bei danger liegt der Fokus auf „Abbrechen",
// damit Enter nichts versehentlich löscht.
import { ref, watch, nextTick } from 'vue';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import BaseModal from './BaseModal.vue';
import BaseButton from './BaseButton.vue';
import { AlertTriangle, HelpCircle } from 'lucide-vue-next';

const { active, respond } = useConfirm();

const cancelBtn = ref(null);
const confirmBtn = ref(null);
watch(active, async (a) => {
  if (!a) return;
  await nextTick();
  const target = a.danger ? cancelBtn.value : confirmBtn.value;
  target?.$el?.focus();
});
</script>

<template>
  <BaseModal v-if="active" :title="active.title" @close="respond(false)">
    <div class="flex items-start gap-3">
      <div class="w-10 h-10 rounded-xl flex items-center justify-center shrink-0"
        :class="active.danger ? 'bg-rose-50 dark:bg-rose-900/30 text-rose-500' : 'bg-marke-leise text-marke'">
        <component :is="active.danger ? AlertTriangle : HelpCircle" class="w-5 h-5" />
      </div>
      <p class="text-sm text-fliess font-medium leading-relaxed whitespace-pre-line pt-2">{{ active.message }}</p>
    </div>
    <template #footer>
      <BaseButton ref="cancelBtn" variant="secondary" @click="respond(false)">{{ active.cancelLabel }}</BaseButton>
      <BaseButton ref="confirmBtn" :variant="active.danger ? 'danger' : 'primary'" @click="respond(true)">{{ active.confirmLabel }}</BaseButton>
    </template>
  </BaseModal>
</template>
