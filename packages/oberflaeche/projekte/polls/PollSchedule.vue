<script setup>
// Terminfindung: Matrix Optionen × Mitglieder mit ja/nein/vielleicht.
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from '../umgebung';
import { Check, Minus, HelpCircle, CheckCircle2 } from 'lucide-vue-next';
import { useToast } from '@oberflaeche/composables/useToast';
import { formatTime, formatWeekdayTime } from '@oberflaeche/shared/date';

const toast = useToast();
const { t } = useI18n();
const props = defineProps({
  poll: { type: Object, required: true },
  projectId: { type: [Number, String], required: true },
  meId: { type: Number, default: null },
});
const emit = defineEmits(['update:poll']);

const ANSWERS = [
  { value: 'yes', icon: Check, label: t('polls.schedule.yes'), cls: 'text-emerald-600 bg-emerald-50 dark:bg-emerald-900/30 dark:text-emerald-400' },
  { value: 'maybe', icon: HelpCircle, label: t('polls.schedule.maybe'), cls: 'text-amber-600 bg-amber-50 dark:bg-amber-900/30 dark:text-amber-400' },
  { value: 'no', icon: Minus, label: t('polls.schedule.no'), cls: 'text-rose-600 bg-rose-50 dark:bg-rose-900/30 dark:text-rose-400' },
];

const fmtOption = (o) => {
  const start = formatWeekdayTime(o.starts_at);
  return o.ends_at ? `${start}–${formatTime(o.ends_at)}` : start;
};

const voteMap = computed(() => {
  const m = {};
  (props.poll.votes || []).forEach((v) => { m[`${v.option_id}:${v.user_id}`] = v.answer; });
  return m;
});
const myAnswer = (optionId) => voteMap.value[`${optionId}:${props.meId}`] || null;
const answerFor = (optionId, userId) => voteMap.value[`${optionId}:${userId}`] || null;
const answerMeta = (a) => ANSWERS.find((x) => x.value === a);
const finalOption = computed(() => props.poll.options.find((o) => o.id === props.poll.final_option_id) || null);

const vote = async (optionId, answer) => {
  if (props.poll.status === 'closed') return;
  try {
    const res = await api.respondPoll(props.projectId, props.poll.id, { option_id: optionId, answer });
    emit('update:poll', res.data);
  } catch (e) {
    toast.error(t('polls.schedule.voteSaveFailed'));
  }
};
</script>

<template>
  <div class="space-y-5">
    <!-- Finaler Termin -->
    <div v-if="finalOption" class="flex items-center gap-3 p-4 bg-emerald-50 dark:bg-emerald-900/20 border border-emerald-200 dark:border-emerald-800 rounded-xl">
      <CheckCircle2 class="w-5 h-5 text-emerald-600 dark:text-emerald-400 shrink-0" />
      <div>
        <p class="text-xs font-bold text-emerald-700 dark:text-emerald-400 uppercase">{{ t('polls.schedule.finalOptionLabel') }}</p>
        <p class="font-bold text-schrift">{{ fmtOption(finalOption) }}</p>
      </div>
    </div>

    <!-- Matrix: Optionen als Spalten, Mitglieder als Zeilen -->
    <div class="overflow-x-auto">
      <table class="w-full border-collapse text-sm">
        <thead>
          <tr>
            <th class="text-left p-2 font-bold text-leise sticky left-0 bg-flaeche">{{ t('polls.schedule.member') }}</th>
            <th v-for="o in poll.options" :key="o.id" class="p-2 text-center font-bold min-w-[9rem]"
              :class="o.id === poll.best_option_id ? 'text-marke' : 'text-fliess'">
              <div class="flex flex-col items-center gap-0.5">
                <span>{{ fmtOption(o) }}</span>
                <span v-if="o.id === poll.best_option_id" class="text-[10px] uppercase font-extrabold">{{ t('polls.schedule.bestOption') }}</span>
              </div>
            </th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="m in poll.members" :key="m.user_id" class="border-t border-linie">
            <td class="p-2 font-semibold text-fliess sticky left-0 bg-flaeche whitespace-nowrap">
              {{ m.name }}
              <span v-if="m.user_id === meId" class="text-xs text-marke">{{ t('polls.schedule.you') }}</span>
              <span v-if="!m.voted" class="ml-1 text-[10px] font-bold text-amber-500 uppercase">{{ t('polls.schedule.pending') }}</span>
            </td>
            <td v-for="o in poll.options" :key="o.id" class="p-2 text-center">
              <!-- Eigene Zeile & offen: klickbare Buttons -->
              <div v-if="m.user_id === meId && poll.status === 'open'" class="flex items-center justify-center gap-1">
                <button v-for="a in ANSWERS" :key="a.value" @click="vote(o.id, a.value)"
                  class="p-1.5 rounded-xl transition-all" :class="myAnswer(o.id) === a.value ? a.cls + ' ring-2 ring-offset-1 ring-current dark:ring-offset-slate-900' : 'text-slate-300 dark:text-slate-600 hover:bg-auflage'"
                  :title="a.label"><component :is="a.icon" class="w-4 h-4" /></button>
              </div>
              <!-- Sonst: nur Anzeige -->
              <span v-else-if="answerFor(o.id, m.user_id)" class="inline-flex items-center justify-center w-7 h-7 rounded-xl" :class="answerMeta(answerFor(o.id, m.user_id)).cls">
                <component :is="answerMeta(answerFor(o.id, m.user_id)).icon" class="w-4 h-4" />
              </span>
              <span v-else class="text-slate-300 dark:text-slate-600">·</span>
            </td>
          </tr>
          <!-- Summenzeile -->
          <tr class="border-t-2 border-linie font-bold">
            <td class="p-2 text-leise sticky left-0 bg-flaeche">{{ t('polls.schedule.yesVotes') }}</td>
            <td v-for="o in poll.options" :key="o.id" class="p-2 text-center"
              :class="o.id === poll.best_option_id ? 'text-marke' : 'text-fliess'">
              {{ o.tally.yes }}
              <span class="block text-[10px] font-medium text-slate-400">{{ t('polls.schedule.maybeCount', { count: o.tally.maybe }) }}</span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
