<script setup>
// Umfrage (Ja/Nein oder eigene Antworten, einfach/mehrfach, optional anonym):
// Auswahl + Ergebnisbalken. Anonym = nur Balken und „X von Y haben
// abgestimmt", keine Namen.
import { ref, computed, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from '../umgebung';
import { Loader2, EyeOff, Users } from 'lucide-vue-next';
import { useToast } from '@oberflaeche/composables/useToast';
import BaseButton from '@oberflaeche/base/BaseButton.vue';

const toast = useToast();
const { t } = useI18n();
const props = defineProps({
  poll: { type: Object, required: true },
  projectId: { type: [Number, String], required: true },
  meId: { type: Number, default: null },
});
const emit = defineEmits(['update:poll']);

const isMultiple = computed(() => props.poll.settings?.multiple);
const isAnonymous = computed(() => props.poll.settings?.anonymous);
const isOpen = computed(() => props.poll.status === 'open');

const selection = ref([...(props.poll.my_option_ids || [])]);
watch(() => props.poll.my_option_ids, (v) => { selection.value = [...(v || [])]; });

const dirty = computed(() => {
  const a = [...selection.value].sort();
  const b = [...(props.poll.my_option_ids || [])].sort();
  return a.length !== b.length || a.some((x, i) => x !== b[i]);
});

const toggle = (optionId) => {
  if (!isOpen.value) return;
  if (isMultiple.value) {
    selection.value = selection.value.includes(optionId)
      ? selection.value.filter((id) => id !== optionId)
      : [...selection.value, optionId];
  } else {
    selection.value = [optionId];
  }
};

const saving = ref(false);
const submit = async () => {
  if (selection.value.length === 0) return;
  saving.value = true;
  try {
    const res = await api.respondPoll(props.projectId, props.poll.id, { option_ids: selection.value });
    emit('update:poll', res.data);
  } catch (e) {
    toast.error(t('polls.survey.saveFailed'));
  } finally {
    saving.value = false;
  }
};

const maxCount = computed(() => Math.max(1, ...props.poll.options.map((o) => o.count)));
const votersFor = (optionId) => {
  if (isAnonymous.value || !props.poll.votes) return '';
  const ids = props.poll.votes.filter((v) => v.option_id === optionId).map((v) => v.user_id);
  const names = (props.poll.members || []).filter((m) => ids.includes(m.user_id)).map((m) => m.name);
  return names.join(', ');
};
const pending = computed(() => isAnonymous.value ? [] : (props.poll.members || []).filter((m) => !m.voted));
</script>

<template>
  <div class="space-y-5">
    <div class="flex items-center gap-2 text-xs font-bold text-leise">
      <span class="flex items-center gap-1"><Users class="w-3.5 h-3.5" /> {{ t('polls.survey.votedCount', { voted: poll.voted_count, total: poll.members_count }) }}</span>
      <span v-if="isAnonymous" class="flex items-center gap-1 px-2 py-0.5 rounded-full bg-auflage"><EyeOff class="w-3 h-3" /> {{ t('polls.survey.anonymous') }}</span>
      <span v-if="isMultiple" class="px-2 py-0.5 rounded-full bg-auflage">{{ t('polls.survey.multipleChoice') }}</span>
    </div>

    <ul class="space-y-2">
      <li v-for="o in poll.options" :key="o.id">
        <button @click="toggle(o.id)" :disabled="!isOpen"
          class="w-full text-left p-3 rounded-xl border transition-all relative overflow-hidden"
          :class="selection.includes(o.id)
            ? 'border-marke bg-marke-leise/60 ring-1 ring-marke'
            : 'border-linie hover:border-marke'"
          :style="!isOpen ? 'cursor: default' : ''">
          <!-- Ergebnisbalken als Hintergrund -->
          <div class="absolute inset-y-0 left-0 bg-marke-leise/70 transition-all" :style="{ width: `${(o.count / maxCount) * 100}%` }"></div>
          <div class="relative flex items-center gap-3">
            <span class="w-4 h-4 shrink-0 border-2 flex items-center justify-center"
              :class="[isMultiple ? 'rounded' : 'rounded-full', selection.includes(o.id) ? 'border-marke bg-marke' : 'border-slate-300 dark:border-slate-600']">
              <span v-if="selection.includes(o.id)" class="w-1.5 h-1.5 rounded-full bg-white"></span>
            </span>
            <span class="font-bold text-schrift flex-1 min-w-0 truncate">{{ o.label }}</span>
            <span class="text-sm font-extrabold text-marke shrink-0">{{ o.count }}</span>
          </div>
          <p v-if="votersFor(o.id)" class="relative text-xs text-leise mt-1 ml-7">{{ votersFor(o.id) }}</p>
        </button>
      </li>
    </ul>

    <div class="flex items-center gap-3 flex-wrap" v-if="isOpen">
      <BaseButton @click="submit" :disabled="saving || selection.length === 0 || !dirty"
        groesse="normal">
        <Loader2 v-if="saving" class="w-4 h-4 animate-spin" />
        {{ (poll.my_option_ids || []).length ? t('polls.survey.changeSelection') : t('polls.survey.vote') }}
      </BaseButton>
      <span v-if="pending.length" class="text-xs font-medium text-amber-500">{{ t('polls.survey.pending', { names: pending.map((m) => m.name).join(', ') }) }}</span>
    </div>
  </div>
</template>
