<script setup>
// Einladungsliste: Einträge sind Personen. Verknüpfte Projektmitglieder
// sagen selbst zu/ab (nur beim eigenen Eintrag); Ersteller/Owner pflegen
// jeden Eintrag inklusive „zurück auf eingeladen" und ergänzen Gäste.
import { ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from '../umgebung';
import { Loader2, Plus, Check, X, RotateCcw, UserRound, Users, Send, Trash2 } from 'lucide-vue-next';
import { useToast } from '@oberflaeche/composables/useToast';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import BaseButton from '@oberflaeche/base/BaseButton.vue';

const toast = useToast();
const { confirmDelete } = useConfirm();
const { t } = useI18n();
const props = defineProps({
  poll: { type: Object, required: true },
  projectId: { type: [Number, String], required: true },
  meId: { type: Number, default: null },
  canManage: { type: Boolean, default: false },
  members: { type: Array, default: () => [] },
});
const emit = defineEmits(['update:poll']);

const isOpen = computed(() => props.poll.status === 'open');
const savingId = ref(null);

const setStatus = async (option, status) => {
  savingId.value = option.id;
  try {
    const res = await api.respondPoll(props.projectId, props.poll.id, { option_id: option.id, status });
    emit('update:poll', res.data);
  } catch (e) {
    toast.error(t('polls.invite.saveFailed'));
  } finally {
    savingId.value = null;
  }
};

// Eintrag entfernen (nur Organisation).
const removeEntry = async (option) => {
  if (!(await confirmDelete(t('polls.invite.removeConfirm', { name: option.label })))) return;
  savingId.value = option.id;
  try {
    const res = await api.removePollOption(props.projectId, props.poll.id, option.id);
    emit('update:poll', res.data);
  } catch (e) {
    toast.error(t('polls.invite.removeFailed'));
  } finally {
    savingId.value = null;
  }
};

// Gast ergänzen (nur Organisation): freier Name oder verknüpftes Mitglied.
const addName = ref('');
const addMemberId = ref('');
const adding = ref(false);
// Mitglieder, die noch keinen Eintrag haben (KI-Mitglieder feiern nicht mit).
const addableMembers = computed(() => props.members.filter(
  (m) => !props.poll.options.some((o) => o.user_id === m.user_id),
));
const onPickMember = () => {
  const m = props.members.find((x) => x.user_id === Number(addMemberId.value));
  if (m) addName.value = m.name;
};
const addGuest = async () => {
  if (!addName.value.trim()) return;
  adding.value = true;
  try {
    const res = await api.addPollOption(props.projectId, props.poll.id, {
      label: addName.value.trim(),
      user_id: addMemberId.value ? Number(addMemberId.value) : null,
    });
    addName.value = '';
    addMemberId.value = '';
    emit('update:poll', res.data);
  } catch (e) {
    toast.error(t('polls.invite.addFailed'));
  } finally {
    adding.value = false;
  }
};

const statusMeta = (s) => ({
  yes: { label: t('polls.invite.statusYes'), cls: 'bg-emerald-50 dark:bg-emerald-900/20 text-emerald-600 dark:text-emerald-400' },
  no: { label: t('polls.invite.statusNo'), cls: 'bg-rose-50 dark:bg-rose-900/20 text-rose-600 dark:text-rose-400' },
  invited: { label: t('polls.invite.statusInvited'), cls: 'bg-amber-50 dark:bg-amber-900/20 text-amber-600 dark:text-amber-400' },
  pending: { label: t('polls.invite.statusPending'), cls: 'bg-auflage text-leise' },
}[s] ?? { label: s, cls: 'bg-auflage text-slate-500' });
</script>

<template>
  <div class="space-y-5">
    <!-- Zählerzeile -->
    <div class="flex items-center gap-2 text-xs font-bold text-leise">
      <Users class="w-3.5 h-3.5" />
      {{ t('polls.invite.counts', { total: poll.counts.total, yes: poll.counts.yes, no: poll.counts.no }) }}
    </div>

    <ul class="space-y-2">
      <li v-for="o in poll.options" :key="o.id"
        class="flex items-center gap-3 p-3 rounded-xl border border-linie"
        :class="o.user_id === meId ? 'bg-marke-leise/40 border-marke' : ''">
        <UserRound class="w-4 h-4 shrink-0" :class="o.user_id ? 'text-marke' : 'text-slate-400'" />
        <span class="font-bold text-schrift flex-1 min-w-0 truncate">
          {{ o.label }} <span v-if="o.user_id === meId" class="text-xs font-medium text-marke">{{ t('polls.invite.you') }}</span>
        </span>

        <span class="text-xs font-bold px-2 py-0.5 rounded-full shrink-0" :class="statusMeta(o.invite_status).cls">
          {{ statusMeta(o.invite_status).label }}
        </span>

        <!-- Aktionen: eigener Eintrag → zu-/absagen; Organisation → alles -->
        <div v-if="isOpen && (o.user_id === meId || canManage)" class="flex items-center gap-1 shrink-0">
          <Loader2 v-if="savingId === o.id" class="w-4 h-4 animate-spin text-marke" />
          <template v-else>
            <!-- Noch offen: die Organisation lädt ausdrücklich ein. -->
            <button v-if="canManage && o.invite_status === 'pending'" @click="setStatus(o, 'invited')"
              class="p-1.5 text-amber-600 hover:bg-amber-50 dark:hover:bg-amber-900/20 rounded-xl" :title="t('polls.invite.markInvited')">
              <Send class="w-4 h-4" />
            </button>
            <button v-if="o.invite_status !== 'yes'" @click="setStatus(o, 'yes')"
              class="p-1.5 text-emerald-600 hover:bg-emerald-50 dark:hover:bg-emerald-900/20 rounded-xl" :title="t('polls.invite.accept')">
              <Check class="w-4 h-4" />
            </button>
            <button v-if="o.invite_status !== 'no'" @click="setStatus(o, 'no')"
              class="p-1.5 text-rose-500 hover:bg-rose-50 dark:hover:bg-rose-900/20 rounded-xl" :title="t('polls.invite.decline')">
              <X class="w-4 h-4" />
            </button>
            <button v-if="canManage && (o.invite_status === 'yes' || o.invite_status === 'no')" @click="setStatus(o, 'invited')"
              class="p-1.5 text-slate-400 hover:bg-auflage rounded-xl" :title="t('polls.invite.reset')">
              <RotateCcw class="w-4 h-4" />
            </button>
            <button v-if="canManage" @click="removeEntry(o)"
              class="p-1.5 text-slate-400 hover:text-rose-500 hover:bg-rose-50 dark:hover:bg-rose-900/20 rounded-xl" :title="t('polls.invite.remove')">
              <Trash2 class="w-4 h-4" />
            </button>
          </template>
        </div>
      </li>
    </ul>

    <!-- Gast ergänzen (nur Organisation, solange offen) -->
    <div v-if="isOpen && canManage" class="flex items-center gap-2 flex-wrap pt-2 border-t border-linie">
      <input v-model="addName" type="text" maxlength="255" :placeholder="t('polls.invite.entryPlaceholder')"
        class="flex-1 min-w-[140px] px-3 py-2 bg-vertieft border border-linie rounded-xl text-sm text-schrift" />
      <select v-model="addMemberId" @change="onPickMember"
        class="px-3 py-2 bg-vertieft border border-linie rounded-xl text-sm text-schrift">
        <option value="">{{ t('polls.invite.linkMemberPlaceholder') }}</option>
        <option v-for="m in addableMembers" :key="m.user_id" :value="m.user_id">{{ m.name }}</option>
      </select>
      <BaseButton @click="addGuest" :disabled="adding || !addName.trim()"
        groesse="normal">
        <Loader2 v-if="adding" class="w-4 h-4 animate-spin" /><Plus v-else class="w-4 h-4" /> {{ t('polls.invite.add') }}
      </BaseButton>
    </div>
  </div>
</template>
