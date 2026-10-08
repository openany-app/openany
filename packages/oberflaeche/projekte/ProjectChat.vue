<script setup>
// Projekt-Chat: Verlauf mit Nachladen älterer Nachrichten, optimistischem
// Senden (negative Temp-Id, bei Antwort ersetzt) und Löschen eigener
// Nachrichten (Owner: aller). Lädt beim Mount; Ansehen setzt die Lesemarke.
import { ref, watch, nextTick, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from './umgebung';
import {
  Send, RefreshCcw, Trash2, Loader2, Pin, PinOff, ChevronDown, ChevronUp,
  FileText, LayoutGrid, Flag, MapPin, SquareKanban, Milestone, MapPinned,
  Folder, File as FileIcon, Images, Image as ImageIcon, Link2,
} from 'lucide-vue-next';
import { useToast } from '@oberflaeche/composables/useToast';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import { initEcho, onEchoReconnect } from './umgebung';
import { formatDateTime } from '@oberflaeche/shared/date';
import ChatMessageText from './ChatMessageText.vue';
import { linkMarkup } from './chatText';
import { suggestTargets } from '@oberflaeche/shared/linkSuggest';

const toast = useToast();
const { confirmDelete } = useConfirm();
const { t } = useI18n();

const props = defineProps({
  projectId: { type: [Number, String], required: true },
  meId: { type: Number, default: null },
  isOwner: { type: Boolean, default: false },
});

// Klick auf einen [[Verweis]] – die Projektansicht wechselt in den Reiter,
// der zur Art des Ziels gehört, und öffnet es dort.
const emit = defineEmits(['open-target']);

const messages = ref([]);
const pinned = ref([]); // Pin-Leiste (kommt mit dem Verlauf, live per Event)
const pinnedOpen = ref(false);
const hasMoreOlder = ref(false);
const chatLoading = ref(false);
const chatInput = ref('');
const notifyMembers = ref(false);
const chatBox = ref(null);
let tempIdCounter = -1;

// Aufgelöste [[Verweise]] aller bisher geladenen Nachrichten: Linkziel
// (klein) => { kind, label, … }. Wird beim Nachladen ERGÄNZT statt ersetzt –
// ältere Nachrichten bleiben sonst klickbar leer, sobald jemand eine neue
// schreibt.
const linkTargets = ref({});
const mergeLinkTargets = (karte) => {
  if (karte) linkTargets.value = { ...linkTargets.value, ...karte };
};

const scrollChatToBottom = async () => {
  await nextTick();
  if (chatBox.value) chatBox.value.scrollTop = chatBox.value.scrollHeight;
};

const loadChat = async () => {
  chatLoading.value = true;
  try {
    const res = await api.getProjectMessages(props.projectId);
    messages.value = res.data.messages;
    pinned.value = res.data.pinned || [];
    hasMoreOlder.value = res.data.has_more;
    mergeLinkTargets(res.data.link_targets);
    scrollChatToBottom();
  } catch (e) { console.error(e); } finally { chatLoading.value = false; }
};

const loadOlder = async () => {
  if (!messages.value.length) return;
  try {
    const res = await api.getProjectMessages(props.projectId, { before_id: messages.value[0].id });
    messages.value = [...res.data.messages, ...messages.value];
    hasMoreOlder.value = res.data.has_more;
    mergeLinkTargets(res.data.link_targets);
  } catch (e) { console.error(e); }
};

// Aktualisieren: lädt nur Nachrichten seit der letzten bekannten Id nach.
const refreshChat = async () => {
  const lastKnown = [...messages.value].reverse().find((m) => m.id > 0);
  if (!lastKnown) return loadChat();
  try {
    const res = await api.getProjectMessages(props.projectId, { after_id: lastKnown.id });
    mergeLinkTargets(res.data.link_targets);
    // Doppelte ausfiltern (z. B. eigene, schon optimistisch angezeigte).
    const known = new Set(messages.value.map((m) => m.id));
    const fresh = res.data.messages.filter((m) => !known.has(m.id));
    if (fresh.length) {
      messages.value = [...messages.value, ...fresh];
      scrollChatToBottom();
    }
  } catch (e) { console.error(e); }
};

const sendMessage = async () => {
  const body = chatInput.value.trim();
  if (!body) return;
  chatInput.value = '';
  // Optimistisch anzeigen (negative Temp-Id), bei Antwort ersetzen.
  const temp = { id: tempIdCounter--, body, created_at: new Date().toISOString(), user: { id: props.meId, name: t('projects.chat.me') }, pending: true };
  messages.value.push(temp);
  scrollChatToBottom();
  try {
    const res = await api.sendProjectMessage(props.projectId, body, notifyMembers.value);
    notifyMembers.value = false;
    mergeLinkTargets(res.data.link_targets);
    const idx = messages.value.findIndex((m) => m.id === temp.id);
    if (idx !== -1) messages.value[idx] = res.data;
  } catch (err) {
    messages.value = messages.value.filter((m) => m.id !== temp.id);
    chatInput.value = body;
    toast.error(err.response?.data?.message || t('projects.chat.sendFailed'));
  }
};

const canDeleteMessage = (msg) => msg.id > 0 && (msg.user?.id === props.meId || props.isOwner);
const deleteMessage = async (msg) => {
  if (!(await confirmDelete(t('projects.chat.deleteConfirm')))) return;
  try {
    await api.deleteProjectMessage(props.projectId, msg.id);
    messages.value = messages.value.filter((m) => m.id !== msg.id);
  } catch (err) {
    toast.error(t('projects.chat.deleteFailed'));
  }
};


// --- Anheften (darf jedes Mitglied) ---
// Lokalen Zustand aus der aktualisierten Nachricht nachziehen: Verlauf
// ersetzt die Nachricht, die Pin-Leiste nimmt sie auf bzw. wirft sie raus.
const applyPinnedMessage = (msg) => {
  const idx = messages.value.findIndex((m) => m.id === msg.id);
  if (idx !== -1) messages.value[idx] = { ...messages.value[idx], ...msg };
  pinned.value = pinned.value.filter((m) => m.id !== msg.id);
  if (msg.pinned) pinned.value = [msg, ...pinned.value];
};

const togglePin = async (msg) => {
  try {
    const res = await api.pinProjectMessage(props.projectId, msg.id, !msg.pinned);
    mergeLinkTargets(res.data.link_targets);
    applyPinnedMessage(res.data);
  } catch (err) {
    toast.error(t('projects.chat.pinFailed'));
  }
};

// --- [[-Vorschlagsliste im Eingabefeld ---
// Dieselbe Geste wie im Notiz-Editor, aber eigene Umsetzung: Dessen Version
// hängt an ProseMirror (state.selection, coordsAtPos) und trägt für ein
// <textarea> nicht.
const inputEl = ref(null);
const linkCandidates = ref([]);       // [{ kind, id, title }]
const suggest = ref(null);            // { items, active, from }
const closeSuggest = () => { suggest.value = null; };

const loadLinkCandidates = async () => {
  try {
    linkCandidates.value = (await api.getProjectLinkTargets(props.projectId)).data.items;
  } catch (e) {
    // Ohne Vorschläge geht Tippen und Senden weiter – von Hand geschriebene
    // Titel-Verweise löst der Server ohnehin auf.
    console.error('Verlinkbare Ziele konnten nicht geladen werden', e);
  }
};

// Art in der Vorschlagsliste kenntlich machen. Ohne den Hinweis stünden
// „Einkauf" (Notiz) und „Einkauf" (Board) ununterscheidbar untereinander –
// genau die Mehrdeutigkeit, wegen der die Ziele getippt sind. Dieselben
// Symbole wie im Planungs-Reiter, damit man sie nicht neu lernen muss.
const kindLabel = (kind) => t(`projects.chat.kinds.${kind}`);
const KIND_ICON = {
  note: FileText, board: LayoutGrid, roadmap: Flag, places: MapPin,
  card: SquareKanban, milestone: Milestone, place: MapPinned,
  folder: Folder, file: FileIcon, album: Images, photo: ImageIcon,
};

// --- Eingabefeld wächst mit ---
// Mit rows="1" sah man immer nur eine Zeile: Wer einen längeren Beitrag
// schreibt (Begrüßung, Absprache, eingefügter Text), tippte durch ein
// Guckloch und konnte nicht überblicken, was er gerade verfasst.
//
// Ab MAX_HOEHE wird gescrollt statt weiter zu wachsen – sonst schöbe eine
// lange Nachricht den Verlauf nach und nach ganz aus dem Bild.
const MAX_HOEHE = 192; // 12rem, rund acht Zeilen

const passeHoeheAn = () => {
  const el = inputEl.value;
  if (!el) return;
  // Erst zurücksetzen: scrollHeight schrumpft sonst nie wieder, und das
  // Feld bliebe nach dem Löschen eines Absatzes zu hoch.
  el.style.height = 'auto';
  el.style.height = `${Math.min(el.scrollHeight, MAX_HOEHE)}px`;
};

// Am Wert hängen statt am Tippen: Das erfasst auch, was das Feld nicht
// selbst geändert hat – eingefügter Vorschlag, Leeren nach dem Senden,
// Zurückschreiben nach einem gescheiterten Versuch.
watch(chatInput, () => nextTick(passeHoeheAn));

const checkSuggest = () => {
  const el = inputEl.value;
  if (!el || !linkCandidates.value.length) return closeSuggest();
  const vorCursor = el.value.slice(0, el.selectionStart);
  const m = /\[\[([^[\]\n]*)$/.exec(vorCursor);
  if (!m) return closeSuggest();
  const query = m[1];
  // Auswahl und Reihenfolge stehen in shared/linkSuggest.js: Sie lassen jede
  // Art zu Wort kommen, statt die Plätze nach der Sortierung des Servers zu
  // vergeben (Notizen zuerst) – sonst blieben Boards, Dateien und Bilder in
  // einem notizreichen Projekt unsichtbar.
  const items = suggestTargets(linkCandidates.value, query);
  if (!items.length) return closeSuggest();
  suggest.value = { items, active: 0, from: el.selectionStart - m[1].length - 2 };
};

/**
 * Verweis-Knopf: setzt „[[" an der Schreibmarke ein und klappt damit die
 * Vorschlagsliste auf.
 *
 * Die Geste gab es nur über die Tastatur, und auf einem Handy liegen zwei
 * eckige Klammern zwei Ebenen tief – wer nicht weiß, dass es sie gibt, findet
 * die Verweise nie. Der Knopf macht dieselbe Sache sichtbar.
 *
 * Ein Leerzeichen davor, wenn der Text unmittelbar weitergeht: „steht auf[["
 * läse sich sonst wie ein Tippfehler.
 */
const fuegeVerweisEin = () => {
  const el = inputEl.value;
  if (!el) return;
  const anfang = el.selectionStart ?? chatInput.value.length;
  const davor = chatInput.value.slice(0, anfang);
  const danach = chatInput.value.slice(el.selectionEnd ?? anfang);
  const trenner = davor && ! /\s$/u.test(davor) ? ' ' : '';
  chatInput.value = `${davor}${trenner}[[${danach}`;

  nextTick(() => {
    const pos = davor.length + trenner.length + 2;
    el.focus();
    el.setSelectionRange(pos, pos);
    // Ohne leere Suche bliebe die Liste zu: Sie hängt am Tippen, und hier
    // hat niemand getippt.
    checkSuggest();
  });
};

const waehleVorschlag = (item) => {
  const el = inputEl.value;
  const s = suggest.value;
  if (!el || !s) return;
  const einsatz = linkMarkup(item);
  const davor = el.value.slice(0, s.from);
  const danach = el.value.slice(el.selectionStart);
  chatInput.value = davor + einsatz + danach;
  closeSuggest();
  nextTick(() => {
    const pos = davor.length + einsatz.length;
    el.focus();
    el.setSelectionRange(pos, pos);
  });
};

// Enter sendet – AUSSER die Vorschlagsliste ist offen, dann fügt es ein.
// Umschalt+Enter bleibt der Zeilenumbruch (wie vorher über .exact).
const onKeydown = (e) => {
  const s = suggest.value;
  if (s) {
    const n = s.items.length;
    if (e.key === 'ArrowDown') { e.preventDefault(); s.active = (s.active + 1) % n; return; }
    if (e.key === 'ArrowUp') { e.preventDefault(); s.active = (s.active - 1 + n) % n; return; }
    if (e.key === 'Enter' || e.key === 'Tab') { e.preventDefault(); waehleVorschlag(s.items[s.active]); return; }
    if (e.key === 'Escape') { e.preventDefault(); closeSuggest(); return; }
  }
  if (e.key === 'Enter' && !e.shiftKey && !e.ctrlKey && !e.metaKey && !e.altKey) {
    e.preventDefault();
    sendMessage();
  }
};

// Echtzeit (Reverb): neue Chat-Nachrichten kommen als Event (dank
// toOthers nur fremde); refreshChat lädt ab der letzten bekannten Id nach.
// Der project.{id}-Kanal GEHÖRT ProjectDetail (Badge) – hier nur den
// eigenen Listener an-/abmelden (stopListening), nicht den Kanal verlassen:
// der Chat-Tab wird per v-if unmountet, der Kanal muss weiterleben.
const echoChannelName = `project.${props.projectId}`;
const onLiveMessage = () => refreshChat();
const onLivePin = (payload) => applyPinnedMessage(payload);
let offReconnect = null;
onMounted(() => {
  loadChat();
  loadLinkCandidates();
  if (initEcho()) {
    initEcho().private(echoChannelName).listen('ProjectMessageSent', onLiveMessage);
    initEcho().private(echoChannelName).listen('ProjectMessagePinned', onLivePin);
    // Nach Verbindungsabriss (Laptop zu, WLAN-Wechsel) verpasste
    // Nachrichten nachladen – Events werden nicht nachgeliefert.
    offReconnect = onEchoReconnect(refreshChat);
  }
});
onUnmounted(() => {
  initEcho()?.private(echoChannelName).stopListening('ProjectMessageSent', onLiveMessage);
  initEcho()?.private(echoChannelName).stopListening('ProjectMessagePinned', onLivePin);
  offReconnect?.();
});
</script>

<template>
  <!-- Höhe: bis kurz über die untere Leiste. Vorher standen hier 24rem
       Abzug – das ergab ein Fenster, in dem kaum ein Dutzend Zeilen Platz
       fanden, obwohl darunter der halbe Bildschirm frei blieb.
       Abgezogen wird jetzt, was tatsächlich darüber und darunter steht:
       mobil Projektkopf und Bottom-Navigation, am Desktop zusätzlich die
       Kopfleiste und das größere Seitenpolster.
       dvh statt vh, weil 100vh auf dem Handy die eingeklappte Adressleiste
       mitzählt und das Fenster damit unten aus dem Bild liefe. -->
  <div class="karte shadow-sm overflow-hidden flex flex-col h-[calc(100dvh-11rem)] md:h-[calc(100dvh-16rem)] min-h-[24rem]">
    <div class="shrink-0 px-5 py-3 border-b border-linie flex items-center justify-between">
      <h3 class="font-extrabold text-schrift text-sm">{{ t('projects.chat.title') }}</h3>
      <button @click="refreshChat" :title="t('projects.chat.loadNewTitle')"
        class="flex items-center gap-1.5 px-3 py-1.5 text-xs font-bold text-marke bg-marke-leise hover:bg-marke-leise rounded-xl transition-colors">
        <RefreshCcw class="w-3.5 h-3.5" /> {{ t('projects.chat.refresh') }}
      </button>
    </div>

    <!-- Pin-Leiste: aufklappbar, für Adressen/Codes/wichtige Absprachen.
         shrink-0 ist wesentlich: Ohne das quetschte der Flex-Container die
         aufgeklappte Leiste zusammen, und von einer längeren angehefteten
         Nachricht war nur ein Ausschnitt zu sehen.

         Angeheftete Nachrichten stehen hier VOLLSTÄNDIG. Eine Kürzung mit
         „weiterlesen" war kurz eingebaut und wieder entfernt: Die Nachricht
         steht ohnehin auch im Verlauf (Anheften nimmt sie dort nicht weg),
         eine zweite Klickstufe brächte also nichts, was nicht schon eine
         Bildschirmlänge tiefer ungekürzt zu lesen wäre. -->
    <div v-if="pinned.length" class="shrink-0 border-b border-amber-100 dark:border-amber-900/40 bg-amber-50/60 dark:bg-amber-900/10">
      <button @click="pinnedOpen = !pinnedOpen" class="w-full px-5 py-2 flex items-center gap-2 text-xs font-bold text-amber-700 dark:text-amber-400">
        <Pin class="w-3.5 h-3.5" />
        {{ t('projects.chat.pinnedCount', { count: pinned.length }) }}
        <component :is="pinnedOpen ? ChevronUp : ChevronDown" class="w-3.5 h-3.5 ml-auto" />
      </button>
      <ul v-if="pinnedOpen" class="px-5 pb-3 space-y-2 max-h-72 overflow-y-auto">
        <li v-for="p in pinned" :key="p.id" class="flex items-start gap-2 text-sm bg-flaeche border border-amber-100 dark:border-amber-900/40 rounded-xl px-3 py-2">
          <div class="flex-1 min-w-0">
            <span class="text-xs font-bold text-leise">{{ p.user?.id === meId ? t('projects.chat.me') : (p.user?.name || '?') }}</span>
            <p class="text-schrift font-medium whitespace-pre-wrap break-words">
              <ChatMessageText :body="p.body" :link-targets="linkTargets" @open-target="emit('open-target', $event)" />
            </p>
          </div>
          <button @click="togglePin(p)" :title="t('projects.chat.unpin')" class="p-1 text-amber-500 hover:text-rose-500 shrink-0">
            <PinOff class="w-3.5 h-3.5" />
          </button>
        </li>
      </ul>
    </div>

    <div ref="chatBox" class="flex-1 overflow-y-auto p-5 space-y-3">
      <div v-if="hasMoreOlder" class="text-center">
        <button @click="loadOlder" class="text-xs font-bold text-marke hover:underline">{{ t('projects.chat.loadOlder') }}</button>
      </div>
      <div v-if="chatLoading" class="flex justify-center py-8"><Loader2 class="w-6 h-6 text-marke animate-spin" /></div>
      <p v-else-if="!messages.length" class="text-center text-sm text-slate-400 py-10">{{ t('projects.chat.emptyState') }}</p>

      <div v-for="msg in messages" :key="msg.id" class="flex" :class="msg.user?.id === meId ? 'justify-end' : 'justify-start'">
        <div class="max-w-[80%] group" :class="msg.pending ? 'opacity-60' : ''">
          <div class="flex items-baseline gap-2 mb-0.5" :class="msg.user?.id === meId ? 'flex-row-reverse' : ''">
            <span class="text-xs font-bold text-fliess">{{ msg.user?.id === meId ? t('projects.chat.me') : (msg.user?.name || '?') }}</span>
            <span class="text-[10px] font-medium text-slate-400">{{ formatDateTime(msg.created_at) }}</span>
            <Pin v-if="msg.pinned" class="w-3 h-3 text-amber-500 shrink-0" />
            <!-- Immer sichtbar, nicht erst beim Überfahren mit der Maus.
                 Vorher standen hier opacity-0 + group-hover: Der Chat wirkte
                 dadurch ruhiger, aber das Anheften war schlicht nicht zu
                 finden – man muss zufällig über die Nachricht fahren, um zu
                 erfahren, dass es die Funktion gibt. Auf dem Handy gab es sie
                 gar nicht, weil es dort kein Überfahren gibt.
                 Die Ruhe stellt jetzt die Farbe her (blass, erst beim
                 Anfassen bunt), nicht das Verstecken. -->
            <button v-if="msg.id > 0" @click="togglePin(msg)" :title="msg.pinned ? t('projects.chat.unpin') : t('projects.chat.pin')"
              class="p-0.5 text-slate-300 dark:text-slate-600 hover:text-amber-500 dark:hover:text-amber-400 transition-colors">
              <component :is="msg.pinned ? PinOff : Pin" class="w-3 h-3" />
            </button>
            <button v-if="canDeleteMessage(msg)" @click="deleteMessage(msg)" :title="t('common.delete')"
              class="p-0.5 text-slate-300 dark:text-slate-600 hover:text-rose-500 dark:hover:text-rose-400 transition-colors"><Trash2 class="w-3 h-3" /></button>
          </div>
          <div class="px-4 py-2.5 rounded-xl text-sm font-medium whitespace-pre-wrap break-words"
            :class="msg.user?.id === meId
              ? 'bg-marke text-white rounded-tr-xl'
              : 'bg-auflage text-slate-800 dark:text-slate-100 rounded-tl-xl'"><ChatMessageText
              :body="msg.body" :link-targets="linkTargets" @open-target="emit('open-target', $event)" /></div>
        </div>
      </div>
    </div>

    <form @submit.prevent="sendMessage" class="shrink-0 relative p-4 border-t border-linie flex items-end gap-3">
      <!-- [[-Vorschläge: über dem Eingabefeld verankert statt am Cursor –
           Cursor-Koordinaten in einem textarea bräuchten einen Spiegel-Div,
           und der Gewinn wäre gering. mousedown.prevent hält den Fokus im
           Feld, sonst schlösse der Blur die Liste vor dem Klick. -->
      <ul v-if="suggest" class="absolute bottom-full left-4 right-4 mb-1 z-20 max-h-56 overflow-y-auto karte shadow-lg py-1">
        <li class="px-3 py-1 text-[10px] font-bold uppercase tracking-wide text-slate-400">{{ t('projects.chat.suggestHint') }}</li>
        <!-- Schlüssel aus Art UND Id: Notiz 5 und Board 5 gibt es beide,
             eine blosse Id liesse Vue die Eintraege verwechseln. -->
        <li v-for="(item, i) in suggest.items" :key="`${item.kind}:${item.id}`">
          <button type="button" @mousedown.prevent="waehleVorschlag(item)"
            class="w-full flex items-center gap-2 px-3 py-2 text-left text-sm font-medium text-fliess cursor-pointer"
            :class="i === suggest.active ? 'bg-marke-leise' : 'hover:bg-auflage'">
            <component :is="KIND_ICON[item.kind]" class="w-3.5 h-3.5 shrink-0 text-amber-600 dark:text-amber-400" />
            <span class="truncate">{{ item.title }}</span>
            <span class="ml-auto shrink-0 text-[10px] font-bold uppercase tracking-wide text-slate-400">{{ kindLabel(item.kind) }}</span>
          </button>
        </li>
      </ul>
      <!-- rows="2" ist die Untergrenze; die Höhe stellt passeHoeheAn nach dem
           Inhalt ein und deckelt bei max-h-48. resize-none bleibt: Ein von
           Hand gezogener Griff und die automatische Höhe würden sich
           gegenseitig überschreiben. -->
      <textarea ref="inputEl" v-model="chatInput" rows="2" maxlength="5000" :placeholder="t('projects.chat.inputPlaceholder')"
        @keydown="onKeydown" @input="checkSuggest" @click="checkSuggest" @blur="closeSuggest"
        class="flex-1 max-h-48 overflow-y-auto px-4 py-2.5 bg-vertieft border border-linie rounded-xl focus:outline-none focus:ring-2 focus:ring-marke text-sm font-medium leading-relaxed text-schrift resize-none"></textarea>
      <!-- mousedown.prevent haelt den Fokus im Feld: Sonst schloesse der Blur
           die Vorschlagsliste in dem Moment, in dem der Klick sie oeffnet.
           Gearbeitet wird trotzdem auf click – der kommt auf jedem Geraet an,
           und genau dort (Handy) wird der Knopf gebraucht. -->
      <button
        type="button"
        @mousedown.prevent
        @click="fuegeVerweisEin"
        :title="t('projects.chat.suggestHint')"
        :aria-label="t('projects.chat.suggestHint')"
        class="p-3 bg-slate-100 hover:bg-slate-200 dark:bg-slate-800 dark:hover:bg-slate-700 text-fliess rounded-xl transition-colors cursor-pointer"
      >
        <Link2 class="w-4 h-4" />
      </button>
      <button type="submit" :disabled="!chatInput.trim()"
        class="p-3 bg-marke hover:bg-marke-satt disabled:opacity-40 text-white rounded-xl shadow-md transition-all">
        <Send class="w-4 h-4" />
      </button>
    </form>
    <label class="shrink-0 px-4 pb-3 -mt-1 flex items-center gap-2 text-xs font-semibold text-leise cursor-pointer select-none">
      <input type="checkbox" v-model="notifyMembers" class="accent-marke" />
      {{ t('projects.notifyMembers') }}
    </label>
  </div>
</template>
