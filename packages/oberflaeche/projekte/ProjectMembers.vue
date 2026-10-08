<script setup>
// Tab „Mitglieder": Liste mit Rollen, Einladen/Entfernen/Owner-Übertragung.
import { ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from './umgebung';
import { debounce } from '@oberflaeche/shared/debounce';
import { UserPlus, LogOut, Search, Crown, Check, Copy } from 'lucide-vue-next';
import { useToast } from '@oberflaeche/composables/useToast';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import { formatDateShort } from '@oberflaeche/shared/date';
import BaseButton from '@oberflaeche/base/BaseButton.vue';

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
// Ob ICH einladen darf, entscheidet der Server (project.can_invite): Owner
// immer, sonstige Mitglieder nur bei gesetztem Schalter members_can_invite.
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
    await api.addProjectMember(props.projectId, user.id);
    memberSearch.value = '';
    searchResults.value = [];
    await reloadProject();
  } catch (err) {
    toast.error(err.response?.data?.message || t('projects.members.inviteFailed'));
  }
};

// ---------- Neuen Menschen anlegen ----------
// ENGER ALS `canInvite`, und das ist die eigentliche Entscheidung: Ein
// BESTEHENDES Mitglied aufzunehmen ist eine Sache dieses Projekts, einen
// NEUEN anzulegen vergibt einen Namen im ganzen Verbund. Der Server
// entscheidet es (`can_create_accounts` = Owner UND Verwalter); hier geht es
// nur darum, den Knopf gar nicht erst anzubieten.
const canCreateAccounts = computed(() => !!props.project?.can_create_accounts);
const neuOffen = ref(false);
const neuName = ref('');
const neuEmail = ref('');
const neuBusy = ref(false);
// Die Adresse fürs erste Passwort — das Einzige, was weitergegeben wird.
const passwortLink = ref(null);
const linkKopiert = ref(false);

const kontoAnlegen = async () => {
  if (!neuName.value.trim() || neuBusy.value) return;
  neuBusy.value = true;
  try {
    const res = await api.createProjectMemberAccount(props.projectId, {
      name: neuName.value.trim(),
      email: neuEmail.value.trim(),
    });
    passwortLink.value = res.data.passwort_setzen;
    neuName.value = '';
    neuEmail.value = '';
    linkKopiert.value = false;
    await reloadProject();
  } catch (err) {
    // Feldfehler (Name vergeben) und Klartext-Meldungen kommen beide als
    // `message` — mehr braucht es für ein Formular mit zwei Feldern nicht.
    toast.error(err.response?.data?.message || t('projects.members.newAccountFailed'));
  } finally { neuBusy.value = false; }
};

const linkKopieren = async () => {
  try {
    await navigator.clipboard.writeText(passwortLink.value);
    linkKopiert.value = true;
  } catch (e) { /* Ohne Zwischenablage bleibt das Feld zum Markieren da. */ }
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

// Einladerecht an alle Mitglieder weitergeben (nur der Owner sieht den
// Schalter; der Server erzwingt das über die Owner-only-Ability 'update').
const invitePermBusy = ref(false);
const setMembersCanInvite = async (value) => {
  invitePermBusy.value = true;
  try {
    const res = await api.updateProject(props.projectId, { members_can_invite: value });
    emit('update:project', res.data);
  } catch (err) {
    toast.error(err.response?.data?.message || t('projects.members.actionFailed'));
  } finally { invitePermBusy.value = false; }
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
    <!-- Einladen (Owner immer, Mitglieder nur bei freigegebenem Einladerecht) -->
    <div v-if="canInvite || isOwner || canCreateAccounts" class="p-5 border-b border-linie space-y-2">
      <label v-if="canInvite" class="text-sm font-bold text-fliess flex items-center gap-2"><UserPlus class="w-4 h-4 text-marke" /> {{ t('projects.members.inviteLabel') }}</label>
      <div v-if="canInvite" class="relative max-w-md">
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

      <!-- Neuen Menschen anlegen (nur Owner, die zugleich Verwalter sind) -->
      <div v-if="canCreateAccounts" class="pt-1">
        <button v-if="!neuOffen" @click="neuOffen = true" type="button"
          class="inline-flex items-center gap-1.5 text-sm font-bold text-marke hover:text-marke">
          <UserPlus class="w-4 h-4" /> {{ t('projects.members.newAccountLabel') }}
        </button>
        <div v-else class="max-w-md space-y-2 p-4 bg-marke-leise/60 border border-marke rounded-xl">
          <p class="text-xs text-fliess leading-relaxed">{{ t('projects.members.newAccountHint') }}</p>
          <input v-model="neuName" type="text" :placeholder="t('projects.members.newAccountName')"
            @keyup.enter="kontoAnlegen"
            class="w-full px-3.5 py-2.5 bg-white dark:bg-slate-950 border border-linie rounded-xl focus:outline-none focus:ring-2 focus:ring-marke text-sm font-medium text-schrift" />
          <input v-model="neuEmail" type="email" :placeholder="t('projects.members.newAccountEmail')"
            @keyup.enter="kontoAnlegen"
            class="w-full px-3.5 py-2.5 bg-white dark:bg-slate-950 border border-linie rounded-xl focus:outline-none focus:ring-2 focus:ring-marke text-sm font-medium text-schrift" />
          <p class="text-xs text-leise leading-relaxed">{{ t('projects.members.newAccountEmailHint') }}</p>
          <BaseButton @click="kontoAnlegen" type="button" :disabled="neuBusy || !neuName.trim()"
            groesse="normal">
            {{ t('projects.members.newAccountAction') }}
          </BaseButton>

          <div v-if="passwortLink" class="pt-2 space-y-1.5">
            <p class="text-xs font-bold text-emerald-700 dark:text-emerald-400">{{ t('projects.members.newAccountCreated') }}</p>
            <div class="flex items-center gap-2">
              <input type="text" readonly :value="passwortLink"
                class="flex-grow min-w-0 px-3 py-2 bg-white dark:bg-slate-950 border border-linie rounded-xl text-xs text-fliess font-mono" />
              <button @click="linkKopieren" type="button"
                class="px-3 py-2 bg-slate-100 hover:bg-slate-200 dark:bg-slate-800 dark:hover:bg-slate-700 text-fliess text-xs font-bold rounded-xl transition-colors flex items-center gap-1.5 shrink-0">
                <Check v-if="linkKopiert" class="w-3.5 h-3.5 text-emerald-500" />
                <Copy v-else class="w-3.5 h-3.5" />
                {{ linkKopiert ? t('projects.members.newAccountCopied') : t('projects.members.newAccountCopy') }}
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Einladerecht weitergeben (nur Owner) -->
      <label v-if="isOwner" class="flex items-start gap-2.5 pt-1 cursor-pointer select-none">
        <input type="checkbox" :checked="!!project.members_can_invite" :disabled="invitePermBusy"
          @change="setMembersCanInvite($event.target.checked)"
          class="mt-0.5 w-4 h-4 shrink-0 rounded border-linie text-marke focus:ring-marke disabled:opacity-50" />
        <span class="min-w-0">
          <span class="block text-sm font-bold text-fliess">{{ t('projects.members.membersCanInviteLabel') }}</span>
          <span class="block text-xs text-leise leading-relaxed">{{ t('projects.members.membersCanInviteHint') }}</span>
        </span>
      </label>
    </div>

    <div class="divide-y divide-slate-100 dark:divide-slate-800/50">
      <div v-for="member in project.members" :key="member.user_id" class="flex items-center gap-4 px-6 py-4">
        <div class="w-10 h-10 rounded-full flex items-center justify-center font-extrabold shrink-0 bg-marke-leise text-marke">
          {{ (member.name || '?').charAt(0).toUpperCase() }}
        </div>
        <div class="flex-1 min-w-0">
          <div class="text-sm font-extrabold text-schrift truncate flex items-center gap-2">
            {{ member.name }} <span v-if="member.user_id === meId" class="text-xs font-medium text-slate-400">{{ t('projects.members.me') }}</span>
          </div>
          <div class="text-xs text-slate-400 font-medium">{{ t('projects.members.joinedSince', { date: formatDateShort(member.joined_at) }) }}</div>
        </div>
        <span v-if="member.role === 'owner'" class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-extrabold uppercase tracking-wider bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-400 shrink-0">
          <Crown class="w-3 h-3" /> {{ t('projects.members.ownerBadge') }}
        </span>
        <div class="flex items-center gap-2 shrink-0">
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
