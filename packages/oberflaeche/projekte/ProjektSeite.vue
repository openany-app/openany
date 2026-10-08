<script setup>
// Die Seite eines Projekts: Kopf, Tab-Leiste mit Zählern und die Modals zum
// Bearbeiten/Löschen. Die Tab-Inhalte sind eigene Komponenten daneben –
// diese Seite hält nur das Projekt selbst.
//
// SEIT 02.10.2026 IM PAKET. Sie war die Ansicht ProjectDetail der Webapp;
// jetzt zeigen Webapp UND App dieselbe Seite. Was nur einer von beiden hat,
// kommt von außen:
//   - Router und eigene Kennung: Die Seite meldet `zurueck`, und wer sie
//     einbindet, sagt mit `meinIdLaden`, wer hier schaut.
//   - api und Live-Kanal: aus ./umgebung (die App hat keinen Kanal).
//   - Slots `chat`, `mitglieder`, `inhalte`: ersetzen den Reiter-Inhalt –
//     die App setzt dort für Projekte ohne Server ihre Teile vor Ort ein.
//     `kopf` steht vor den Knöpfen der Titelzeile, `hinweis` über den
//     Reitern. Ohne Slot gilt, was die Webapp immer hatte.
import { ref, computed, onMounted, onUnmounted, defineAsyncComponent } from 'vue';
import { useI18n } from 'vue-i18n';
import { api, initEcho, leaveEchoChannel, onEchoReconnect } from './umgebung';
import BaseModal from '@oberflaeche/base/BaseModal.vue';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import ProjectShares from './ProjectShares.vue';
import {
  ArrowLeft, MessageSquare, Share2, Users,
  Trash2, Pencil, CalendarClock, RefreshCcw,
} from 'lucide-vue-next';
import { useToast } from '@oberflaeche/composables/useToast';
import { debounce } from '@oberflaeche/shared/debounce';
import { CONTAINER_SIGNALS } from './planningTypes';
import ModulePage from '@oberflaeche/base/ModulePage.vue';

const toast = useToast();
const { t } = useI18n();
const props = defineProps({
  projectId: { type: [Number, String], required: true },
  // Wer hier schaut. Die Webapp holt die Id aus dem geteilten
  // Session-Singleton (der Header lädt sie beim Start) – das spart den
  // zweiten /user/settings-Aufruf. Die App kennt keine Konto-Id: null.
  meinIdLaden: { type: Function, default: async () => null },
  // Ein Ziel, das gleich beim Öffnen aufgehen soll (dieselbe Form wie ein
  // Chat-Verweis) – in der Webapp der Sprung aus dem Kalender.
  startZiel: { type: Object, default: null },
  // Was dieser Rahmen hier nicht anbietet (local-first, 02.10.2026):
  // Reiter (`chat`, `members`) und Knöpfe (`umbenennen`, `loeschen`). Die
  // Webapp nennt nichts.
  ohne: { type: Array, default: () => [] },
});
const emit = defineEmits(['zurueck']);
// Alle Tabs außer der Standardansicht „Inhalte" als eigene, lazy geladene
// Chunks – halten die Projekt-Grundansicht schlank.
const ProjectChat = defineAsyncComponent(() => import('./ProjectChat.vue'));
const ProjectPlanning = defineAsyncComponent(() => import('./ProjectPlanning.vue'));
const ProjectMembers = defineAsyncComponent(() => import('./ProjectMembers.vue'));
const ProjectAi = defineAsyncComponent(() => import('./ProjectAi.vue'));

const projectId = props.projectId;

const project = ref(null);
const meId = ref(null);
const isLoading = ref(true);
const errorMsg = ref('');
// Standard: Chat – das Projekt öffnet dort, wo es sich bewegt. Was seit dem
// letzten Besuch geschrieben wurde, sieht man dann sofort, statt es erst über
// ein Zahlen-Abzeichen erahnen zu müssen.
const REITER = ['chat', 'planning', 'members', 'contents'];
const sichtbareReiter = REITER.filter((r) => !props.ohne.includes(r));
const activeTab = ref(sichtbareReiter[0] ?? 'planning'); // 'chat' | 'planning' | 'members' | 'contents'
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
//   Planung     meldet die Planungs-Ansicht selbst (@count), solange sie
//               gemountet ist: Ihre Liste IST, was planningCount() zählt.
//               Bei geschlossenem Reiter zieht ein fremder Wechsel die Zahl
//               über ProjectPlanningChanged nach – dann gibt es die Ansicht
//               nicht, die es melden könnte.
//   Inhalte     hat KEINEN Live-Weg. Legt jemand eine Notiz in eine
//               freigegebene Mappe, gibt es dafür kein Ereignis.
//   Mitglieder  kommt aus der geladenen Mitgliederliste.
//
// Für alles, was danach noch stehenbleibt – vor allem „Inhalte", und bei
// abgerissener Echo-Verbindung sämtliche –, gibt es den Knopf in der
// Titelzeile (zaehlerAktualisieren).
const unreadChat = ref(0);

// Eigene Ablage statt project.counts.planning: Diese Zahl hat mehrere
// Schreiber, und die Planungs-Ansicht weiß es zwischendurch besser als der
// zuletzt geladene Payload. Vorher blieb sie stehen – wer einen Behälter
// löschte, sah die alte Zahl über der neuen Liste.
const planningCount = ref(0);

const tabCounts = computed(() => project.value ? {
  contents: project.value.counts?.contents ?? 0,
  chat: unreadChat.value,
  // Planung: Abstimmungen/Listen, Boards, Roadmaps, Orte und Schuljahre. Was
  // dazugehört, entscheidet das Backend (Project::planningCount) – hier
  // summiert stand die Zahl auf polls + boards und zählte den Rest nicht mit.
  planning: planningCount.value,
  members: project.value.members.length,
} : {});
// Chat zeigt nur echte Ungelesene, alle anderen Tabs ihren Bestand.
const tabBadge = (id) => {
  const c = tabCounts.value[id];
  if (id === 'chat') return c > 0 ? c : null;
  return c ?? null;
};

// Echtzeit: Der project.{id}-Kanal wird HIER abonniert und beim Verlassen
// der Projektansicht verlassen (die Tab-Komponenten hängen nur eigene
// Listener an). Neue Chat-Nachrichten erhöhen das Tab-Badge, solange der
// Chat-Tab nicht offen ist (dort markiert der Abruf sie als gelesen).
const onLiveChatMessage = () => {
  if (activeTab.value !== 'chat') unreadChat.value += 1;
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
  planningCount.value = daten.counts?.planning ?? 0;
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

// Remount-Schlüssel für den Freigaben-Reiter: erneuter Klick auf den bereits
// aktiven „Freigaben"-Reiter setzt die eingebettete Detailansicht zurück auf
// die Freigaben-Liste (statt eines eigenen „Zurück"-Links).
const sharesKey = ref(0);
const switchTab = (tab) => {
  if (tab === 'contents' && activeTab.value === 'contents') sharesKey.value++;
  activeTab.value = tab;
  // Ansehen = gelesen (die Lesemarke setzt das Backend beim Nachrichtenabruf).
  if (tab === 'chat') unreadChat.value = 0;
};

// Klick auf einen [[Verweis]] im Chat: in den Reiter wechseln, der zur Art
// des Ziels gehört, und es dort öffnen. Der Reiter ist bis dahin nicht
// gemountet – das Ziel wird deshalb hier gehalten und als Prop übergeben,
// statt es hineinzurufen.
//
// Zwei getrennte Ablagen, weil es zwei Reiter sind: Was an einer Freigabe
// hängt (Notiz, Datei, Ordner, Album, Bild), liegt hinter dem
// Freigaben-Reiter; alles Geplante im Planungs-Reiter.
//
// Steht eine Art hier NICHT, landet sie in der Planung. Das ist die
// verträglichere Vorgabe: Eine unbekannte Art hat der Server aufgelöst, also
// gibt es sie – nur diese Frontend-Fassung kennt sie noch nicht.
const FREIGABE_ARTEN = ['note', 'note_folder', 'folder', 'file', 'album', 'photo'];

const pendingShare = ref(null);     // { kind, id, folder_id?, album_id?, share_root_id }
const pendingPlanning = ref(null);  // { kind, id, board_id?, roadmap_id?, group_id? }
const planningKey = ref(0);

const openTargetFromChat = (ziel) => {
  // Remount erzwingen, damit die Startebene auch dann greift, wenn der
  // Reiter schon einmal offen war (Startwerte wirken nur beim Aufbau).
  if (FREIGABE_ARTEN.includes(ziel.kind)) {
    pendingShare.value = ziel;
    sharesKey.value++;
    switchTab('contents');

    return;
  }

  pendingPlanning.value = ziel;
  planningKey.value++;
  switchTab('planning');
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

// Alle Zahlen-Abzeichen neu holen: Inhalte, Planung, Ungelesene, Mitglieder.
//
// Es gibt sie als Knopf, weil nicht jeder Zähler von selbst nachzieht. Der
// Chat zählt live hoch, die Planung meldet sich, solange ihr Reiter offen
// ist – aber **Inhalte** wurde bisher überhaupt nur beim Öffnen des Projekts
// gelesen: Legt ein anderes Mitglied eine Notiz in eine freigegebene Mappe,
// steht die alte Zahl bis zum nächsten Aufruf da. Und ohne laufende
// Echo-Verbindung (offline, Reverb weg) steht ohnehin alles still. Ein Klick
// ist die ehrliche Antwort darauf – und die Zahlen sind eine Anfrage wert,
// keine Dauerabfrage.
//
// Nichts davon zählt selbst: Die Zahlen kommen fertig aus `counts`, die
// Ungelesenen weiterhin über die Lesemarke (project_chat_reads). Diese
// Funktion holt sie nur neu – ein zweiter Zählweg neben dem bestehenden wäre
// genau die Sorte Verdopplung, die später auseinanderläuft.
//
// `still` trennt die beiden Aufrufer: Der Knopf zeigt einen Spinner und sagt
// Bescheid, wenn es schiefging; das Live-Ereignis läuft lautlos. Ein
// Abzeichen ist kein Grund, die Projektansicht durch eine Fehlermeldung zu
// ersetzen – deshalb hier auch kein `errorMsg` wie in loadProject().
const zaehlerLaden = ref(false);

const zaehlerAktualisieren = async ({ still = false } = {}) => {
  if (!still) zaehlerLaden.value = true;
  try {
    uebernehmeProjekt((await api.getProject(projectId)).data);
  } catch (e) {
    if (!still) toast.error(t('projects.detail.countsRefreshFailed'));
  } finally {
    zaehlerLaden.value = false;
  }
};

// Legt jemand anderes einen Planungs-Behälter an oder weg, während der
// Reiter zu ist, kann die Planungs-Ansicht nichts melden – sie ist nicht
// gemountet. Dann holt die Projektansicht die Zahl selbst nach.
const zaehlerNachziehen = debounce(() => zaehlerAktualisieren({ still: true }), 500);

const onLivePlanningChange = (payload) => {
  if (CONTAINER_SIGNALS.includes(payload?.kind) && activeTab.value !== 'planning') {
    zaehlerNachziehen();
  }
};

const echoChannelName = `project.${projectId}`;
let offReconnect = null;

// Für Rahmen ohne Live-Kanal (die App): Sie meldet selbst, wenn sich etwas
// getan hat, und die Zahlen werden still nachgezogen.
defineExpose({ zaehlerAktualisieren });

onMounted(() => {
  loadProject();
  if (props.startZiel) openTargetFromChat(props.startZiel);
  const echo = initEcho()?.private(echoChannelName);
  if (!echo) return;

  echo.listen('ProjectMessageSent', onLiveChatMessage);
  echo.listen('ProjectPlanningChanged', onLivePlanningChange);

  // Ein WebSocket reißt im Alltag ab – Laptop zu, WLAN-Wechsel, Reverb neu
  // gestartet –, und verpasste Ereignisse werden beim Neuverbinden NICHT
  // nachgeliefert (siehe services/echo.js). Ohne dieses Refetch stünden die
  // Abzeichen danach still, ohne dass irgendwo etwas kaputt aussähe: Die
  // Verbindung ist ja wieder da.
  //
  // Genau deshalb reicht „auf das Ereignis warten" allein nie. Jede Ansicht
  // mit Live-Daten hängt sich hier ein; diese hier war die letzte, die es
  // nicht tat – ausgerechnet die, der der Kanal gehört.
  offReconnect = onEchoReconnect(() => zaehlerAktualisieren({ still: true }));
});

onUnmounted(() => {
  leaveEchoChannel(echoChannelName);
  offReconnect?.();
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
        class="sticky top-0 z-20 md:static -mx-1 sm:-mx-6 -mt-1 sm:-mt-4 md:mx-0 md:mt-0 bg-flaeche border-b border-linie md:border md:rounded-xl md:shadow-sm md:overflow-hidden transition-colors"
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
            <!-- Zähler auffrischen: für jedes Mitglied, nicht nur den Owner.
                 Nur ein Symbol – der Knopf holt Zahlen, keine Inhalte, und
                 soll die beiden Verwaltungsknöpfe daneben nicht überstrahlen. -->
            <button @click="zaehlerAktualisieren()" :disabled="zaehlerLaden" :title="t('projects.detail.refreshCounts')"
              class="flex items-center justify-center h-8 w-8 text-leise hover:bg-auflage rounded-xl transition-colors disabled:opacity-50">
              <RefreshCcw class="w-4 h-4" :class="zaehlerLaden ? 'animate-spin text-marke' : ''" />
            </button>

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
              { id: 'chat', label: t('projects.detail.tabs.chat'), icon: MessageSquare },
              { id: 'planning', label: t('projects.detail.tabs.planning'), icon: CalendarClock },
              { id: 'members', label: t('projects.detail.tabs.members'), icon: Users },
              { id: 'contents', label: t('projects.detail.tabs.contents'), icon: Share2 },
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
        @open-target="openTargetFromChat" />
      <!-- „Freigaben": Landeliste aller in dieses Projekt geteilten Behälter
           (Mappe/Album/Ordner/Akte); Klick öffnet den passenden Betrachter. -->
      <template v-else-if="activeTab === 'contents' && $slots.inhalte">
        <slot name="inhalte" :project="project" :is-owner="isOwner" />
      </template>
      <ProjectShares v-else-if="activeTab === 'contents'" :key="sharesKey" :project-id="projectId" :me-id="meId" :is-owner="isOwner"
        :project-name="project.name" :open-target="pendingShare" />
      <!-- „Planung" enthält Terminfindung, Umfragen, Listen UND Kanban-Boards. -->
      <ProjectPlanning v-else-if="activeTab === 'planning'" :key="planningKey" :project-id="projectId" :me-id="meId" :is-owner="isOwner" :members="project.members"
        :open-target="pendingPlanning" @count="planningCount = $event" />
      <!-- Mitglieder – und darunter, wer außer ihnen mitliest: die KI. -->
      <template v-else-if="activeTab === 'members' && $slots.mitglieder">
        <slot name="mitglieder" :project="project" :is-owner="isOwner" />
      </template>
      <div v-else-if="activeTab === 'members'" class="space-y-4">
        <ProjectMembers :project-id="projectId" :project="project" :me-id="meId" :is-owner="isOwner"
          @update:project="project = $event" @verlassen="emit('zurueck')" />
        <ProjectAi :project-id="projectId" />
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
  </ModulePage>
</template>
