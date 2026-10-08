<script setup>
// Neuen Kalender anlegen (Name + Farbe). Die Anlage übernimmt der Eltern-View.
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import BaseModal from '@oberflaeche/base/BaseModal.vue';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import { PRESET_COLORS, DEFAULT_CALENDAR_COLOR } from '@oberflaeche/shared/calendarColors';
import { FolderPlus } from 'lucide-vue-next';

defineProps({
  saving: { type: Boolean, default: false },
});
const emit = defineEmits(['save', 'close']);

const { t } = useI18n();

const form = ref({ name: '', color: DEFAULT_CALENDAR_COLOR });

const submit = () => {
  if (!form.value.name.trim()) return;
  emit('save', { ...form.value });
};
</script>

<template>
  <BaseModal size="sm" @close="emit('close')">
    <template #header>
      <div class="flex items-center gap-2">
        <FolderPlus class="w-5 h-5 text-marke" />
        <h3 class="text-lg font-bold text-schrift">{{ t('calendar.createModal.title') }}</h3>
      </div>
    </template>
    <form @submit.prevent="submit" class="space-y-4">
      <div class="space-y-1">
        <label class="block text-xs font-bold uppercase tracking-wider text-leise">{{ t('calendar.createModal.name') }}</label>
        <input v-model="form.name" type="text" :placeholder="t('calendar.createModal.namePlaceholder')" required class="w-full px-3 py-2 bg-slate-50 dark:bg-slate-800 border border-linie rounded-xl focus:outline-hidden focus:ring-2 focus:ring-marke text-sm font-semibold text-schrift">
      </div>
      <div class="space-y-1">
        <label class="block text-xs font-bold uppercase tracking-wider text-leise">{{ t('calendar.createModal.color') }}</label>
        <div class="flex gap-1.5 h-8">
          <button v-for="c in PRESET_COLORS" :key="c.value" type="button" @click="form.color = c.value" class="flex-1 rounded-xl border-2 transition-all cursor-pointer" :style="{ backgroundColor: c.value, borderColor: form.color === c.value ? 'var(--color-marke-satt)' : 'transparent' }" :class="form.color === c.value ? 'ring-2 ring-marke scale-110' : 'hover:scale-105'"></button>
        </div>
      </div>
      <div class="flex justify-end gap-2 pt-3 border-t border-linie">
        <BaseButton type="button" variant="ghost" @click="emit('close')">{{ t('common.cancel') }}</BaseButton>
        <BaseButton type="submit" :loading="saving">{{ t('common.save') }}</BaseButton>
      </div>
    </form>
  </BaseModal>
</template>
