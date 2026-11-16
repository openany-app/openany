<script setup>
// Die Seite eines Projekts: Kopf, Tab-Leiste mit Zählern und die Modals zum
// Bearbeiten/Löschen. Die Tab-Inhalte sind eigene Komponenten daneben –
// diese Seite hält nur das Projekt selbst.
//
// IM PAKET: Webapp UND App zeigen dieselbe Seite. Was nur einer von beiden
// hat, kommt von außen:
//   - Router und eigene Kennung: Die Seite meldet `zurueck`, und wer sie
//     einbindet, sagt mit `meinIdLaden`, wer hier schaut.
//   - api: aus ./umgebung.
//   - Slots `chat`, `mitglieder`, `inhalte`: ersetzen den Reiter-Inhalt –
//     die App setzt dort für Projekte ohne Server ihre Teile vor Ort ein.
//     `kopf` steht vor den Knöpfen der Titelzeile, `hinweis` über den
//     Reitern. Ohne Slot gilt der Inhalt der Seite selbst.
import { ref, computed, onMounted, onUnmounted, defineAsyncComponent } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from './umgebung';
import BaseModal from '@oberflaeche/base/BaseModal.vue';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import {
  ArrowLeft, MessageSquare, Share2, Users,
  Trash2, Pencil, Presentation,
} from 'lucide-vue-next';
import { useToast } from '@oberflaeche/composables/useToast';
import ModulePage from '@oberflaeche/base/ModulePage.vue';

const toast = useToast();
const { t } = useI18n();
const props = defineProps({
  projectId: { type: [Number, String], required: true },
  // Wer hier schaut. Die Webapp holt die Id aus dem geteilten
  // Session-Singleton (der Header lädt sie beim Start) – das spart den
  // zweiten /user/settings-Aufruf. Die App kennt keine Konto-Id: null.
  meinIdLaden: { type: Function, default: async () => null },
  // Was dieser Rahmen hier nicht anbietet (local-first):
  // Reiter (`chat`, `members`) und Knöpfe (`umbenennen`, `loeschen`). Die
  // Webapp nennt nichts.
  ohne: { type: Array, default: () => [] },
});
const emit = defineEmits(['zurueck']);
// Alle Tabs außer der Standardansicht „Inhalte" als eigene, lazy geladene
// Chunks – halten die Projekt-Grundansicht schlank.
const ProjectChat = defineAsyncComponent(() => import('./ProjectChat.vue'));
const ProjektBoard = defineAsyncComponent(() => import('./ProjektBoard.vue'));
const ProjectMembers = defineAsyncComponent(() => import('./ProjectMembers.vue'));

const projectId = props.projectId;

const project = ref(null);
const meId = ref(null);
const isLoading = ref(true);
const errorMsg = ref('');
// Reihenfolge und Start: Freigaben, Board, Mitglieder,
// Chat -- ein Hub öffnet bei seinen Freigaben.
const REITER = ['contents', 'board', 'members', 'chat'];
const sichtbareReiter = REITER.filter((r) => !props.ohne.includes(r));
const activeTab = ref(sichtbareReiter[0] ?? 'contents'); // 'contents' | 'members' | 'chat'
// „Board" ist kein Reiter mit eigenem Inhalt: Es gibt dort nur das eine
// Board, also öffnet es gleich -- über dem Reiter, in dem man war. „Zurück"
// führt wieder dorthin.
const boardOffen = ref(false);
const darf = (was) => !props.ohne.includes(was);

const isOwner = computed(() => project.value?.my_role === 'owner');

// Tab-Badges: Die Anfangswerte kommen als `counts` mit dem Projekt-Payload.
// „Inhalte" zählt die Inhalte hinter den Freigaben (Notizen/Dateien/Bilder),
// nicht die Freigaben selbst.
//
// WIE SIE AKTUELL BLEIBEN, ist je Zähler verschieden – und das gehört
// hierher, weil „wird clientseitig aktuell gehalten" pauschal nicht stimmte:
//
//   Chat        zählt live hoch (ProjectMessageSent) und wird beim Ansehen
//               des Reiters genullt.
//   Inhalte     zählt die Einträge unter Freigaben (samt Kalendern) und
//               frischt auf, wenn hier geteilt oder beendet wird
//               (`openany:kalender-geteilt`). Was andere teilen, kommt
//               mit dem nächsten Laden.
//   Mitglieder  kommt aus der geladenen Mitgliederliste.
//
// Was danach noch stehenbleibt, holt der Abgleich: Er baut die Seite neu
// auf (ProjekteAnsicht, `stand`).
const unreadChat = ref(0);

const tabCounts = computed(() => project.value ? {
  contents: project.value.counts?.contents ?? 0,
  chat: unreadChat.value,
  members: project.value.members.length,
} : {});
// Chat zeigt nur echte Ungelesene, alle anderen Tabs ihren Bestand.
const tabBadge = (id) => {
  const c = tabCounts.value[id];
  if (id === 'chat') return c > 0 ? c : null;
  return c ?? null;
};

// Die EINE Stelle, an der aus einem Projekt-Payload die Anzeige wird.
//
// Sie steht hier für sich, weil es zwei Aufrufer gibt – das erste Laden und
// das Auffrischen der Zähler –, die sich nur im Verhalten beim Fehlschlag
// unterscheiden. Die Regeln, wie aus `counts` ein Abzeichen wird, dürfen
// nicht zweimal dastehen: Die Ungelesen-Fallunterscheidung stand kurzzeitig
// wortgleich in beiden, und wer sie ändert, hätte die zweite übersehen.
const uebernehmeProjekt = (daten) => {
  project.value = daten;
  // Die Ungelesen-Zahl selbst kommt fertig vom Server (Lesemarke in
  // project_chat_reads) – hier wird nichts nachgezählt, nur der eine Fall
  // abgefangen: Steht der Chat offen, hat sein Abruf die Marke schon
  // gesetzt. Ohne das zeigte das Abzeichen Ungelesene an, während man die
  // Nachrichten längst vor sich hat.
  unreadChat.value = activeTab.value === 'chat' ? 0 : (daten.counts?.unread_messages ?? 0);
};

const loadProject = async () => {
  try {
    // Projekt laden; parallel die Session sicherstellen (gecacht nach dem
    // App-Start, daher i. d. R. ohne zusätzliche Anfrage).
    const [projRes, ich] = await Promise.all([api.getProject(projectId), props.meinIdLaden()]);
    uebernehmeProjekt(projRes.data);
    meId.value = ich ?? null;
  } catch (e) {
    errorMsg.value = e.response?.status === 403 ? t('projects.detail.noAccess') : t('projects.detail.loadError');
  } finally {
    isLoading.value = false;
  }
};

const switchTab = (tab) => {
  if (tab === 'board') {
    boardOffen.value = true;
    return;
  }
  activeTab.value = tab;
  // Ansehen = gelesen (die Lesemarke setzt das Backend beim Nachrichtenabruf).
  if (tab === 'chat') unreadChat.value = 0;
};

// Klick auf einen [[Verweis]] im Chat: in den Freigaben-Reiter wechseln.
// Verweisbar ist nur, was an einer Freigabe hängt (Notiz, Datei, Ordner,
// Album, Bild).
const FREIGABE_ARTEN = ['note', 'note_folder', 'folder', 'file', 'album', 'photo'];

const openTargetFromChat = (ziel) => {
  if (!FREIGABE_ARTEN.includes(ziel.kind)) return;
  switchTab('contents');
};

// ---------- Projekt verwalten (Owner) ----------
const editModal = ref(null);  // { name }
const deleteModal = ref(false);
const deleting = ref(false);

const submitEdit = async () => {
  const name = editModal.value.name.trim();
  if (!name) return;
  try {
    const res = await api.updateProject(projectId, { name });
    project.value = res.data;
    editModal.value = null;
  } catch (err) {
    toast.error(err.response?.data?.message || t('projects.detail.saveError'));
  }
};

const confirmDeleteProject = async () => {
  deleting.value = true;
  try {
    await api.deleteProject(projectId);
    emit('zurueck');
  } catch (err) {
    toast.error(err.response?.data?.message || t('projects.detail.deleteError'));
  } finally {
    deleting.value = false;
  }
};

// Alle Zahlen-Abzeichen still neu holen: Inhalte, Ungelesene, Mitglieder.
// Nichts davon zählt selbst: Die Zahlen kommen fertig aus `counts`, die
// Ungelesenen über die Lesemarke (project_chat_reads). Ein Abzeichen ist
// kein Grund für eine Fehlermeldung, deshalb bleibt ein Fehler stumm.
const zaehlerAktualisieren = async () => {
  try {
    uebernehmeProjekt((await api.getProject(projectId)).data);
  } catch {
    // Beim nächsten Laden stimmt es wieder.
  }
};

// Die App meldet selbst, wenn sich etwas getan hat, und die Zahlen werden
// still nachgezogen.
defineExpose({ zaehlerAktualisieren });

// Unter Freigaben wurde geteilt oder beendet: den Zähler nachziehen.
const FREIGABEN_GEAENDERT = 'openany:kalender-geteilt';
const freigabenGeaendert = () => zaehlerAktualisieren();

onMounted(() => {
  loadProject();
  window.addEventListener(FREIGABEN_GEAENDERT, freigabenGeaendert);
});

onUnmounted(() => {
  window.removeEventListener(FREIGABEN_GEAENDERT, freigabenGeaendert);
});
</script>

<template>
  <ModulePage>

    <div v-if="isLoading" class="flex justify-center py-20">
      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-marke"></div>
    </div>
    <div v-else-if="errorMsg" class="karte shadow-sm py-16 text-center space-y-4">
      <p class="text-sm font-bold text-rose-500">{{ errorMsg }}</p>
      <button @click="emit('zurueck')" class="text-sm font-bold text-marke hover:underline">{{ t('projects.detail.backToList') }}</button>
    </div>

    <template v-else-if="project">
      <!-- Projekt-Kopf: Titelzeile + Reiter. Mobil eine randlose, klebende
           Leiste im Stil der Modul-Header (Zeilen durch Linien getrennt);
           ab md EINE Karte (Titel + Reiter, nur durch eine Linie getrennt).
           Kein Icon, keine Beschreibung mehr. -->
      <div
        class="sticky top-[var(--rand-oben)] z-20 md:static sm:-mt-4 md:mx-0 md:mt-0 bg-flaeche border-b border-linie md:border md:rounded-xl md:shadow-sm md:overflow-hidden transition-colors"
      >
        <!-- Titelzeile -->
        <div class="flex items-center justify-between gap-2 sm:gap-4 min-w-0 px-3 sm:px-6 py-1.5 sm:py-2.5 bg-slate-50 dark:bg-slate-800/40 border-b border-slate-100 dark:border-slate-800/70">
          <div class="flex items-center gap-2 sm:gap-3 min-w-0">
            <button @click="emit('zurueck')" :title="t('common.back')" class="p-1.5 text-leise hover:bg-auflage rounded-xl transition-colors shrink-0">
              <ArrowLeft class="w-5 h-5" />
            </button>
            <h2 class="text-base font-extrabold text-marke tracking-tight truncate">{{ project.name }}</h2>
          </div>

          <div class="flex items-center gap-2 shrink-0">
            <slot name="kopf" :project="project" :is-owner="isOwner" />

            <template v-if="isOwner">
            <button v-if="darf('umbenennen')" @click="editModal = { name: project.name }" :title="t('common.rename')"
              class="flex items-center justify-center h-8 gap-1.5 px-2.5 sm:px-4 py-1.5 bg-slate-100 hover:bg-slate-200 dark:bg-slate-800 dark:hover:bg-slate-700 text-fliess text-sm font-bold rounded-xl transition-colors">
              <Pencil class="w-4 h-4" /> <span class="hidden sm:inline">{{ t('common.rename') }}</span>
            </button>
            <button v-if="darf('loeschen')" @click="deleteModal = true" :title="t('common.delete')"
              class="flex items-center justify-center h-8 gap-1.5 px-2.5 sm:px-4 py-1.5 bg-rose-50 hover:bg-rose-100 dark:bg-rose-900/20 dark:hover:bg-rose-900/40 text-rose-600 dark:text-rose-400 text-sm font-bold rounded-xl transition-colors">
              <Trash2 class="w-4 h-4" /> <span class="hidden sm:inline">{{ t('common.delete') }}</span>
            </button>
            </template>
          </div>
        </div>

        <!-- Reiter (Projekt-Menü) -->
        <div class="flex items-center gap-1 sm:gap-2 px-2 sm:px-3 py-2">
          <button v-for="tab in [
              { id: 'contents', label: t('projects.detail.tabs.contents'), icon: Share2 },
              { id: 'board', label: t('projects.detail.tabs.board'), icon: Presentation },
              { id: 'members', label: t('projects.detail.tabs.members'), icon: Users },
              { id: 'chat', label: t('projects.detail.tabs.chat'), icon: MessageSquare },
            ].filter((r) => darf(r.id))" :key="tab.id"
            @click="switchTab(tab.id)"
            class="flex-1 flex items-center justify-center gap-2 px-2 sm:px-3 py-2 rounded-xl text-sm font-bold transition-colors"
            :class="activeTab === tab.id ? 'bg-marke text-white shadow-md' : 'text-fliess hover:bg-auflage'">
            <component :is="tab.icon" class="w-4 h-4" />
            <span class="hidden sm:inline">{{ tab.label }}</span>
            <!-- Ungelesen-Zähler am Chat, rot hervorgehoben -->
            <span v-if="tab.id === 'chat' && tabBadge('chat')"
              class="text-[10px] font-extrabold min-w-[1.1rem] px-1 py-0.5 rounded-full text-center"
              :class="activeTab === 'chat' ? 'bg-white/25 text-white' : 'bg-rose-500 text-white'">{{ tabBadge('chat') }}</span>
            <!-- Bestandszahlen ohne Klammern: Sie standen dort nur, um die
                 Zahl vom Wort abzusetzen – das tut jetzt die eigene Fläche,
                 wie beim Chat daneben. Nur ruhiger, weil ein Bestand keine
                 Aufforderung ist. -->
            <span v-else-if="tab.id !== 'chat' && tabBadge(tab.id) !== null"
              class="text-[10px] font-extrabold min-w-[1.1rem] px-1 py-0.5 rounded-full text-center"
              :class="activeTab === tab.id
                ? 'bg-white/25 text-white'
                : 'bg-slate-200 dark:bg-slate-700 text-fliess'">{{ tabBadge(tab.id) }}</span>
          </button>
        </div>
      </div>

      <slot name="hinweis" :project="project" :is-owner="isOwner" />

      <!-- Tab-Inhalte (v-if statt v-show: Wechsel lädt frisch) -->
      <template v-if="activeTab === 'chat' && $slots.chat">
        <slot name="chat" :project="project" :is-owner="isOwner" />
      </template>
      <ProjectChat v-else-if="activeTab === 'chat'" :project-id="projectId" :me-id="meId" :is-owner="isOwner"
        :mit-hinweis="!project?.lokal" @open-target="openTargetFromChat" />
      <!-- „Freigaben": Landeliste aller in dieses Projekt geteilten Behälter
           (Mappe/Album/Ordner/Akte); Klick öffnet den passenden Betrachter. -->
      <template v-else-if="activeTab === 'contents'">
        <slot name="inhalte" :project="project" :is-owner="isOwner" />
      </template>
      <!-- Mitglieder – und darunter, wer außer ihnen mitliest: die KI. -->
      <template v-else-if="activeTab === 'members' && $slots.mitglieder">
        <slot name="mitglieder" :project="project" :is-owner="isOwner" />
      </template>
      <div v-else-if="activeTab === 'members'" class="space-y-4">
        <ProjectMembers :project-id="projectId" :project="project" :me-id="meId" :is-owner="isOwner"
          @update:project="project = $event" @verlassen="emit('zurueck')" />
      </div>

      <!-- MODAL: Projekt umbenennen -->
      <BaseModal v-if="editModal" :title="t('projects.detail.editTitle')" @close="editModal = null">
        <form @submit.prevent="submitEdit" class="space-y-4">
          <input v-model="editModal.name" type="text" required :placeholder="t('projects.detail.namePlaceholder')"
            class="w-full px-4 py-3 bg-vertieft border border-linie rounded-xl focus:outline-none focus:ring-2 focus:ring-marke text-sm font-semibold text-schrift" />
          <div class="pt-2 flex items-center justify-end gap-3">
            <BaseButton variant="ghost" @click="editModal = null">{{ t('common.cancel') }}</BaseButton>
            <BaseButton type="submit">{{ t('common.save') }}</BaseButton>
          </div>
        </form>
      </BaseModal>

      <!-- MODAL: Projekt löschen (Warn-Dialog) -->
      <BaseModal v-if="deleteModal" @close="deleteModal = false">
        <template #header>
          <h3 class="font-extrabold text-schrift">{{ t('projects.detail.deleteTitle', { name: project.name }) }}</h3>
        </template>
        <p class="text-sm text-leise">
          {{ t('projects.detail.deleteWarningPart1') }} <strong>{{ t('projects.detail.deleteWarningStrong1') }}</strong> {{ t('projects.detail.deleteWarningPart2') }}
          {{ t('projects.detail.deleteWarningPart3') }} <strong>{{ t('projects.detail.deleteWarningStrong2') }}</strong> {{ t('projects.detail.deleteWarningPart4') }}
        </p>
        <div class="pt-5 flex items-center justify-end gap-3">
          <BaseButton variant="ghost" @click="deleteModal = false">{{ t('common.cancel') }}</BaseButton>
          <BaseButton variant="danger" :loading="deleting" @click="confirmDeleteProject">{{ t('projects.detail.deleteForever') }}</BaseButton>
        </div>
      </BaseModal>
    </template>
    <!-- Das Board des Hubs, über dem Reiter, in dem man war. -->
    <ProjektBoard v-if="boardOffen" :project-id="projectId" @zu="boardOffen = false" />
  </ModulePage>
</template>
