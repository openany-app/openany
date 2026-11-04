<script setup>
// Tab „Mitglieder": Liste mit Rollen, Einladen/Entfernen/Owner-Übertragung.
import { ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from './umgebung';
import { debounce } from '@oberflaeche/shared/debounce';
import { UserPlus, LogOut, Search, Crown } from 'lucide-vue-next';
import { useToast } from '@oberflaeche/composables/useToast';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import { formatDateShort } from '@oberflaeche/shared/date';

const toast = useToast();
const { confirmDialog, confirmDelete } = useConfirm();
const { t } = useI18n();

const props = defineProps({
  projectId: { type: [Number, String], required: true },
  project: { type: Object, required: true },
  meId: { type: Number, default: null },
  isOwner: { type: Boolean, default: false },
});
// Das Projekt gehört dem Eltern-View; Änderungen (Mitglieder, Owner,
// KI-Status) gehen als komplettes Payload zurück.
// `verlassen`: Man ist selbst ausgetreten -- wohin es dann geht, weiss
// der Rahmen (die Webapp hat einen Router, die App nicht).
const emit = defineEmits(['update:project', 'verlassen']);

const reloadProject = async () => {
  const res = await api.getProject(props.projectId);
  emit('update:project', res.data);
};

// ---------- Einladen ----------
// Ob ICH einladen darf, entscheidet der Server (project.can_invite): nur
// Owner -- wie in der unterschriebenen Mitgliederliste
// (docs/plan-projekte-eine-art.md, E2). Mit dem Namen: Die App sucht damit
// die Person hinter dem Konto und unterschreibt den Beitritt.
const canInvite = computed(() => !!props.project?.can_invite);
const memberSearch = ref('');
const searchResults = ref([]);
const searchUsers = debounce(async () => {
  const q = memberSearch.value.trim();
  if (!q) { searchResults.value = []; return; }
  try {
    const res = await api.searchProjectUsers(q);
    const memberIds = new Set(props.project.members.map((m) => m.user_id));
    searchResults.value = res.data.filter((u) => !memberIds.has(u.id));
  } catch (e) { searchResults.value = []; }
}, 300);

const addMember = async (user) => {
  try {
    await api.addProjectMember(props.projectId, user.id, user.name);
    memberSearch.value = '';
    searchResults.value = [];
    await reloadProject();
  } catch (err) {
    toast.error(err.response?.data?.message || t('projects.members.inviteFailed'));
  }
};

const removeMember = async (member) => {
  const isSelf = member.user_id === props.meId;
  const msg = isSelf ? t('projects.members.leaveConfirm') : t('projects.members.removeConfirm', { name: member.name });
  if (!(await confirmDelete(msg, { title: isSelf ? t('projects.members.leaveTitle') : t('projects.members.removeTitle'), confirmLabel: isSelf ? t('projects.members.leaveAction') : t('projects.members.removeAction') }))) return;
  try {
    await api.removeProjectMember(props.projectId, member.user_id);
    if (isSelf) { emit('verlassen'); return; }
    emit('update:project', { ...props.project, members: props.project.members.filter((m) => m.user_id !== member.user_id) });
  } catch (err) {
    toast.error(err.response?.data?.message || t('projects.members.actionFailed'));
  }
};

// Wie viele Owner das Projekt hat. Zwei Knöpfe hängen daran: Der letzte
// Owner kann weder herabgestuft werden noch gehen — ein Projekt ohne Owner
// ließe sich nicht mehr verwalten. Der Server weist beides ab; hier geht es
// darum, den Knopf gar nicht erst anzubieten.
const ownerCount = computed(() => props.project?.owner_count
  ?? (props.project?.members || []).filter((m) => m.role === 'owner').length);

// Zum Owner MACHEN, nicht übertragen: Der bisherige Owner behält seine
// Rolle. Genau das ist der Punkt dieser Stufe.
const promote = async (member) => {
  if (!(await confirmDialog(t('projects.members.promoteConfirm', { name: member.name }), {
    title: t('projects.members.promoteTitle'),
    confirmLabel: t('projects.members.promoteAction'),
  }))) return;
  await setRole(member, 'owner');
};

const demote = async (member) => {
  const selbst = member.user_id === props.meId;
  if (!(await confirmDialog(
    t(selbst ? 'projects.members.demoteSelfConfirm' : 'projects.members.demoteConfirm', { name: member.name }),
    { title: t('projects.members.demoteTitle'), confirmLabel: t('projects.members.demoteAction'), danger: true },
  ))) return;
  await setRole(member, 'member');
};

const setRole = async (member, role) => {
  try {
    const res = await api.setProjectMemberRole(props.projectId, member.user_id, role);
    emit('update:project', res.data);
  } catch (err) {
    toast.error(err.response?.data?.message || t('projects.members.roleFailed'));
  }
};

</script>

<template>
  <div class="karte shadow-sm overflow-hidden">
    <!-- Einladen (nur Owner) -->
    <div v-if="canInvite" class="p-5 border-b border-linie space-y-2">
      <label class="text-sm font-bold text-fliess flex items-center gap-2"><UserPlus class="w-4 h-4 text-marke" /> {{ t('projects.members.inviteLabel') }}</label>
      <div class="relative max-w-md">
        <Search class="w-4 h-4 absolute left-3.5 top-1/2 -translate-y-1/2 text-slate-400" />
        <input v-model="memberSearch" @input="searchUsers" type="search" :placeholder="t('projects.members.searchPlaceholder')"
          class="w-full pl-10 pr-4 py-2.5 bg-vertieft border border-linie rounded-xl focus:outline-none focus:ring-2 focus:ring-marke text-sm font-medium text-schrift" />
        <div v-if="searchResults.length" class="absolute z-20 mt-1 w-full karte shadow-xl overflow-hidden">
          <button v-for="u in searchResults" :key="u.id" @click="addMember(u)"
            class="w-full text-left px-4 py-2.5 text-sm font-semibold text-fliess hover:bg-marke-leise flex items-center gap-2">
            <UserPlus class="w-3.5 h-3.5 text-marke" /> {{ u.name }}
          </button>
        </div>
      </div>
    </div>

    <div class="divide-y divide-slate-100 dark:divide-slate-800/50">
      <!-- Umbrechen statt quetschen: Auf schmalen Schirmen gehen die Knöpfe
           in eine eigene Zeile unter den Namen – sonst stünde „dabei seit"
           Wort für Wort untereinander, und der Name verschwände ganz. -->
      <div v-for="member in project.members" :key="member.user_id" class="flex flex-wrap items-center gap-x-4 gap-y-2 px-6 py-4">
        <div class="w-10 h-10 rounded-full flex items-center justify-center font-extrabold shrink-0 bg-marke-leise text-marke">
          {{ (member.name || '?').charAt(0).toUpperCase() }}
        </div>
        <div class="flex-1 min-w-[9rem]">
          <div class="text-sm font-extrabold text-schrift truncate flex items-center gap-2">
            {{ member.name }} <span v-if="member.user_id === meId" class="text-xs font-medium text-slate-400">{{ t('projects.members.me') }}</span>
          </div>
          <div class="text-xs text-slate-400 font-medium">{{ t('projects.members.joinedSince', { date: formatDateShort(member.joined_at) }) }}</div>
        </div>
        <span v-if="member.role === 'owner'" class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-extrabold uppercase tracking-wider bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-400 shrink-0">
          <Crown class="w-3 h-3" /> {{ t('projects.members.ownerBadge') }}
        </span>
        <div class="flex flex-wrap items-center gap-2 w-full pl-14 sm:w-auto sm:pl-0 empty:hidden">
          <button v-if="isOwner && member.role !== 'owner'" @click="promote(member)"
            class="px-3 py-1.5 text-xs font-bold text-amber-600 dark:text-amber-400 bg-amber-50 dark:bg-amber-900/20 hover:bg-amber-100 dark:hover:bg-amber-900/40 rounded-xl transition-colors">
            {{ t('projects.members.promoteButton') }}
          </button>
          <!-- Herabstufen erst ab zwei Ownern: Der letzte kann es nicht, und
               ein Knopf, der immer scheitert, ist schlechter als keiner. -->
          <button v-if="isOwner && member.role === 'owner' && ownerCount > 1" @click="demote(member)"
            class="px-3 py-1.5 text-xs font-bold text-fliess bg-auflage hover:bg-slate-200 dark:hover:bg-slate-700 rounded-xl transition-colors">
            {{ t('projects.members.demoteButton') }}
          </button>
          <!-- Owner lassen sich nicht entfernen – nur herabstufen oder selbst
               gehen. Ohne diese Schranke wären zwei Owner ein Wettrennen. -->
          <button v-if="isOwner && member.role !== 'owner'" @click="removeMember(member)"
            class="px-3 py-1.5 text-xs font-bold text-rose-600 dark:text-rose-400 bg-rose-50 dark:bg-rose-900/20 hover:bg-rose-100 dark:hover:bg-rose-900/40 rounded-xl transition-colors">
            {{ t('projects.members.removeButton') }}
          </button>
          <!-- Verlassen darf jetzt auch ein Owner – nur nicht der letzte. -->
          <button v-if="member.user_id === meId && (!isOwner || ownerCount > 1)" @click="removeMember(member)"
            class="flex items-center gap-1.5 px-3 py-1.5 text-xs font-bold text-rose-600 dark:text-rose-400 bg-rose-50 dark:bg-rose-900/20 hover:bg-rose-100 dark:hover:bg-rose-900/40 rounded-xl transition-colors">
            <LogOut class="w-3.5 h-3.5" /> {{ t('projects.members.leaveButton') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
