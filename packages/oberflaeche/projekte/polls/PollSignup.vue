<script setup>
// Eintrage-Liste (Schichtplan / Mitbringliste): Einträge mit Kapazität,
// „Übernehmen"/„Austragen" mit optionalem Kommentar, Erledigt-Häkchen und
// (wenn erlaubt) von Mitgliedern ergänzbare Einträge.
import { ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { api, kann } from '../umgebung';
import { Loader2, Plus, CheckCircle2, Circle, UserPlus, UserMinus } from 'lucide-vue-next';
import { useToast } from '@oberflaeche/composables/useToast';
import BaseButton from '@oberflaeche/base/BaseButton.vue';

const toast = useToast();
const { t } = useI18n();
const props = defineProps({
  poll: { type: Object, required: true },
  projectId: { type: [Number, String], required: true },
  meId: { type: Number, default: null },
  canManage: { type: Boolean, default: false },
});
const emit = defineEmits(['update:poll']);

const isOpen = computed(() => props.poll.status === 'open');
const doneMarking = computed(() => props.poll.settings?.done_marking);
const memberOptions = computed(() => props.poll.settings?.member_options);

const myClaim = (o) => o.claims.find((c) => c.user_id === props.meId) || null;
const isFull = (o) => o.capacity !== null && o.claims.length >= o.capacity;

// Übernehmen mit optionalem Kommentar (kleines Inline-Formular je Eintrag)
const claimFor = ref(null); // option id mit offenem Kommentar-Feld
const claimComment = ref('');
const busy = ref(false);

const startClaim = (o) => { claimFor.value = o.id; claimComment.value = myClaim(o)?.comment || ''; };

const submitClaim = async (optionId) => {
  busy.value = true;
  try {
    const res = await api.respondPoll(props.projectId, props.poll.id, {
      option_id: optionId, action: 'claim', comment: claimComment.value.trim() || null,
    });
    claimFor.value = null;
    emit('update:poll', res.data);
  } catch (e) {
    toast.error(e.response?.data?.message || t('polls.signup.claimFailed'));
  } finally {
    busy.value = false;
  }
};

const unclaim = async (optionId) => {
  busy.value = true;
  try {
    const res = await api.respondPoll(props.projectId, props.poll.id, { option_id: optionId, action: 'unclaim' });
    emit('update:poll', res.data);
  } catch (e) {
    toast.error(t('polls.signup.unclaimFailed'));
  } finally {
    busy.value = false;
  }
};

const toggleDone = async (o) => {
  try {
    const res = await api.togglePollOptionDone(props.projectId, props.poll.id, o.id, !o.done);
    emit('update:poll', res.data);
  } catch (e) {
    toast.error(e.response?.status === 403 ? t('polls.signup.doneToggleForbidden') : t('polls.signup.actionFailed'));
  }
};

const mayToggleDone = (o) => kann('abstimmungenVerwalten') && doneMarking.value && (props.canManage || !!myClaim(o));

// Eintrag ergänzen
const addForm = ref(null); // { label, capacity }
const adding = ref(false);
const submitAdd = async () => {
  if (!addForm.value.label.trim()) return;
  adding.value = true;
  try {
    const res = await api.addPollOption(props.projectId, props.poll.id, {
      label: addForm.value.label.trim(),
      capacity: addForm.value.capacity ? Number(addForm.value.capacity) : null,
    });
    addForm.value = null;
    emit('update:poll', res.data);
  } catch (e) {
    toast.error(t('polls.signup.addFailed'));
  } finally {
    adding.value = false;
  }
};
</script>

<template>
  <div class="space-y-4">
    <ul class="space-y-2">
      <li v-for="o in poll.options" :key="o.id"
        class="p-3 rounded-xl border border-linie"
        :class="o.done ? 'opacity-70 bg-emerald-50/50 dark:bg-emerald-950/20 border-emerald-200 dark:border-emerald-900' : ''">
        <div class="flex items-center gap-3 flex-wrap">
          <!-- Erledigt-Häkchen -->
          <button v-if="doneMarking" @click="mayToggleDone(o) && toggleDone(o)" :title="o.done ? t('polls.signup.markOpen') : t('polls.signup.markDone')"
            class="shrink-0" :class="mayToggleDone(o) ? 'cursor-pointer' : 'cursor-default'">
            <CheckCircle2 v-if="o.done" class="w-5 h-5 text-emerald-500" />
            <Circle v-else class="w-5 h-5" :class="mayToggleDone(o) ? 'text-slate-400 hover:text-emerald-500' : 'text-slate-300 dark:text-slate-600'" />
          </button>

          <span class="font-bold text-schrift" :class="o.done ? 'line-through' : ''">{{ o.label }}</span>

          <span v-if="o.capacity" class="text-xs font-bold px-2 py-0.5 rounded-full"
            :class="isFull(o) ? 'bg-rose-50 dark:bg-rose-900/30 text-rose-500' : 'bg-auflage text-slate-500'">
            {{ o.claims.length }}/{{ o.capacity }}
          </span>

          <div class="ml-auto flex items-center gap-2">
            <template v-if="isOpen">
              <button v-if="myClaim(o)" @click="unclaim(o.id)" :disabled="busy"
                class="flex items-center gap-1 px-2.5 py-1.5 text-xs font-bold text-rose-500 hover:bg-rose-50 dark:hover:bg-rose-900/20 rounded-xl">
                <UserMinus class="w-3.5 h-3.5" /> {{ t('polls.signup.unclaim') }}
              </button>
              <BaseButton v-else-if="!isFull(o)" @click="startClaim(o)" :disabled="busy"
                groesse="klein">
                <UserPlus class="w-3.5 h-3.5" /> {{ t('polls.signup.claim') }}
              </BaseButton>
              <span v-else class="text-xs font-bold text-slate-400">{{ t('polls.signup.full') }}</span>
            </template>
          </div>
        </div>

        <!-- Wer hat übernommen -->
        <ul v-if="o.claims.length" class="mt-2 ml-1 space-y-0.5">
          <li v-for="c in o.claims" :key="c.user_id" class="text-sm text-fliess">
            <span class="font-semibold">{{ c.name }}</span>
            <span v-if="c.user_id === meId" class="text-xs text-marke"> {{ t('polls.signup.you') }}</span>
            <span v-if="c.comment" class="text-slate-400"> – {{ c.comment }}</span>
          </li>
        </ul>

        <!-- Kommentar-Feld beim Übernehmen -->
        <div v-if="claimFor === o.id" class="mt-2 flex items-center gap-2">
          <input v-model="claimComment" type="text" maxlength="500" :placeholder="t('polls.signup.commentPlaceholder')"
            class="flex-1 px-3 py-2 bg-vertieft border border-linie rounded-xl text-sm text-schrift"
            @keyup.enter="submitClaim(o.id)" />
          <BaseButton @click="submitClaim(o.id)" :disabled="busy" groesse="klein">
            <Loader2 v-if="busy" class="w-3.5 h-3.5 animate-spin" /> {{ t('polls.signup.ok') }}
          </BaseButton>
          <button @click="claimFor = null" class="px-2 py-2 text-xs font-bold text-slate-500 hover:bg-auflage rounded-xl">{{ t('common.cancel') }}</button>
        </div>
      </li>
    </ul>

    <!-- Eintrag ergänzen -->
    <div v-if="isOpen && kann('abstimmungenVerwalten') && (memberOptions || canManage)">
      <button v-if="!addForm" @click="addForm = { label: '', capacity: '' }" class="flex items-center gap-1.5 text-sm font-bold text-marke hover:underline">
        <Plus class="w-4 h-4" /> {{ t('polls.signup.addEntry') }}
      </button>
      <div v-else class="flex items-center gap-2 flex-wrap">
        <input v-model="addForm.label" type="text" maxlength="255" :placeholder="t('polls.signup.entryPlaceholder')"
          class="flex-1 min-w-[10rem] px-3 py-2 bg-vertieft border border-linie rounded-xl text-sm text-schrift"
          @keyup.enter="submitAdd" />
        <input v-model="addForm.capacity" type="number" min="1" max="999" :placeholder="t('polls.signup.maxPeoplePlaceholder')"
          class="w-32 px-3 py-2 bg-vertieft border border-linie rounded-xl text-sm text-schrift" />
        <BaseButton @click="submitAdd" :disabled="adding" groesse="klein">
          <Loader2 v-if="adding" class="w-3.5 h-3.5 animate-spin" /> {{ t('polls.signup.add') }}
        </BaseButton>
        <button @click="addForm = null" class="px-2 py-2 text-xs font-bold text-slate-500 hover:bg-auflage rounded-xl">{{ t('common.cancel') }}</button>
      </div>
    </div>
  </div>
</template>
