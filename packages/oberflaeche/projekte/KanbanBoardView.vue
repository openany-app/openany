<script setup>
// Detailansicht EINES Kanban-Boards (aus ProjectBoards.vue extrahiert –
// die Board-Liste lebt jetzt im Planung-Reiter, ProjectPlanning.vue). Lädt sich
// selbst über die containerId, hält Spalten/Karten inkl. Drag & Drop und
// zieht fremde Änderungen live nach (KanbanBoardChanged, debounced).
import { ref, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import draggable from 'vuedraggable';
import { api, kann } from './umgebung';
import MarkdownEditor from '@oberflaeche/editor/MarkdownEditor.vue';
import PlacePicker from './PlacePicker.vue';
import SubjectPicker from './SubjectPicker.vue';
import {
  Plus, RefreshCcw, Trash2, ArrowLeft, Loader2, X, Pencil,
  CalendarClock, UserCircle, MoreVertical, MapPin,
} from 'lucide-vue-next';
import { useToast } from '@oberflaeche/composables/useToast';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import { initEcho, onEchoReconnect } from './umgebung';
import { debounce } from '@oberflaeche/shared/debounce';
import BaseButton from '@oberflaeche/base/BaseButton.vue';

const toast = useToast();
const { confirmDelete } = useConfirm();
const { t } = useI18n();
const props = defineProps({
  projectId: { type: [Number, String], required: true },
  containerId: { type: [Number, String], required: true },
  members: { type: Array, default: () => [] },
  // Karte aus einem [[Verweis]] im Chat: Sie wird nach dem Laden aufgeschlagen.
  // Eine Karte hat keine eigene Ansicht – das Modal IST sie.
  openCardId: { type: [Number, String], default: null },
});
const emit = defineEmits(['close']);

const board = ref(null);
const boardLoading = ref(false);
const error = ref('');

// Karten-Detail-Modal
const cardModal = ref(null); // { ...card, _columnId }
const savingCard = ref(false);

// Spalten-Aktionen
const columnMenu = ref(null); // columnId, dessen Menü offen ist
const deleteColumnModal = ref(null); // { column, targetColumnId }

const today = new Date().toISOString().slice(0, 10);
const isOverdue = (due) => due && due < today;

const memberName = (id) => props.members.find((m) => m.user_id === id)?.name || '—';

const loadBoard = async () => {
  boardLoading.value = true;
  try {
    board.value = (await api.getBoard(props.projectId, props.containerId)).data;
  } catch (e) {
    error.value = t('projects.boards.boardLoadError');
  } finally {
    boardLoading.value = false;
  }
};
const reloadBoard = () => loadBoard();

// --- Spalten ---
const addColumn = async () => {
  const name = prompt(t('projects.boards.newColumnPrompt'));
  if (!name?.trim()) return;
  await api.createColumn(props.projectId, board.value.id, name.trim());
  reloadBoard();
};

const renameColumn = async (col) => {
  const name = prompt(t('projects.boards.renameColumnPrompt'), col.name);
  columnMenu.value = null;
  if (!name?.trim() || name === col.name) return;
  await api.renameColumn(props.projectId, board.value.id, col.id, name.trim());
  reloadBoard();
};

const askDeleteColumn = (col) => {
  columnMenu.value = null;
  if (col.cards.length === 0) {
    doDeleteColumn(col, null);
  } else {
    deleteColumnModal.value = { column: col, targetColumnId: otherColumns(col.id)[0]?.id ?? null };
  }
};

const otherColumns = (excludeId) => (board.value?.columns || []).filter((c) => c.id !== excludeId);

const doDeleteColumn = async (col, targetColumnId) => {
  try {
    await api.deleteColumn(props.projectId, board.value.id, col.id, targetColumnId);
    deleteColumnModal.value = null;
    reloadBoard();
  } catch (e) {
    toast.error(t('projects.boards.columnDeleteFailed'));
  }
};

// --- Karten ---
const quickAdd = ref({}); // columnId -> Titel
const addCard = async (col) => {
  const title = (quickAdd.value[col.id] || '').trim();
  if (!title) return;
  await api.createCard(props.projectId, board.value.id, col.id, title);
  quickAdd.value[col.id] = '';
  reloadBoard();
};

const openCard = (card, columnId) => {
  cardModal.value = { ...card, _columnId: columnId };
};

/**
 * Karte aus dem Chat aufschlagen. In welcher Spalte sie liegt, weiß hier
 * niemand – sie wird gesucht. Gibt zurück, ob es sie noch gibt: Zwischen dem
 * Schreiben der Nachricht und dem Klick kann sie gelöscht worden sein, und
 * dann soll etwas dastehen statt dass nichts geschieht.
 */
const schlageKarteAuf = (id) => {
  for (const col of board.value?.columns || []) {
    const karte = (col.cards || []).find((c) => c.id === id);
    if (karte) {
      openCard(karte, col.id);

      return true;
    }
  }

  return false;
};

const saveCard = async () => {
  savingCard.value = true;
  try {
    await api.updateCard(props.projectId, board.value.id, cardModal.value.id, {
      title: cardModal.value.title,
      description: cardModal.value.description,
      assigned_to: cardModal.value.assigned_to,
      due_date: cardModal.value.due_date || null,
      place_id: cardModal.value.place_id ?? null,
      subject_id: cardModal.value.subject_id ?? null,
    });
    cardModal.value = null;
    reloadBoard();
  } catch (e) {
    toast.error(t('projects.boards.cardSaveFailed'));
  } finally {
    savingCard.value = false;
  }
};

const deleteCard = async () => {
  if (!(await confirmDelete(t('projects.boards.deleteCardConfirm')))) return;
  await api.deleteCard(props.projectId, board.value.id, cardModal.value.id);
  cardModal.value = null;
  reloadBoard();
};

// --- Drag & Drop (vuedraggable / SortableJS: robuste Touch-Unterstützung) ---
// Karten: pro Spalte ein Draggable mit gemeinsamer group; wir persistieren
// genau EINEN Move (added = in diese Spalte gezogen, moved = innerhalb).
const onCardChange = async (col, evt) => {
  const change = evt.added || evt.moved;
  if (!change) return; // 'removed' behandelt die Zielspalte
  try {
    await api.moveCard(props.projectId, board.value.id, change.element.id, col.id, change.newIndex);
  } catch (e) {
    reloadBoard(); // bei Konflikt/Fehler resynchronisieren
  }
};

const onColumnChange = async (evt) => {
  if (!evt.moved) return;
  try {
    await api.reorderColumns(props.projectId, board.value.id, board.value.columns.map((c) => c.id));
  } catch (e) {
    reloadBoard();
  }
};

// Echtzeit: fremde Änderungen an DIESEM Board nachladen (toOthers).
// Der project.{id}-Kanal gehört ProjectDetail – hier nur Listener.
const debouncedBoardReload = debounce(() => reloadBoard(), 400);
const onKanbanChange = (payload) => {
  if (payload.board_id === props.containerId) debouncedBoardReload();
};

const echoChannelName = `project.${props.projectId}`;
let offReconnect = null;
onMounted(async () => {
  await loadBoard();
  if (props.openCardId && ! schlageKarteAuf(props.openCardId)) {
    toast.error(t('projects.chat.targetNotFound'));
  }
  if (initEcho()) {
    initEcho().private(echoChannelName).listen('KanbanBoardChanged', onKanbanChange);
    offReconnect = onEchoReconnect(reloadBoard);
  }
});
onUnmounted(() => {
  initEcho()?.private(echoChannelName).stopListening('KanbanBoardChanged', onKanbanChange);
  offReconnect?.();
});
</script>

<template>
  <div class="space-y-4">
    <div class="flex items-center gap-3 flex-wrap">
      <button @click="emit('close')" class="p-2 text-leise hover:bg-auflage rounded-xl" :title="t('projects.boards.backToList')"><ArrowLeft class="w-4 h-4" /></button>
      <h3 class="font-extrabold text-lg text-schrift">{{ board?.name }}</h3>
      <div class="ml-auto flex items-center gap-2">
        <button @click="reloadBoard" class="flex items-center gap-1.5 px-3 py-2 text-sm font-bold text-fliess hover:bg-auflage rounded-xl" :title="t('projects.boards.refresh')"><RefreshCcw class="w-4 h-4" :class="boardLoading ? 'animate-spin text-marke' : ''" /></button>
        <button v-if="board" @click="addColumn" class="flex items-center gap-1.5 px-3 py-2 text-sm font-bold text-marke hover:bg-marke-leise rounded-xl"><Plus class="w-4 h-4" /> {{ t('projects.boards.column') }}</button>
      </div>
    </div>

    <div v-if="!board && boardLoading" class="py-10 text-center text-slate-400"><Loader2 class="w-6 h-6 animate-spin mx-auto" /></div>
    <p v-else-if="error" class="text-sm text-rose-500 font-medium">{{ error }}</p>

    <!-- Spalten (horizontal scrollbar), per DnD sortierbar -->
    <div v-else-if="board" class="overflow-x-auto pb-2">
      <draggable :list="board.columns" item-key="id" group="columns" handle=".col-handle" @change="onColumnChange" class="flex gap-4 items-start min-h-[12rem]">
        <template #item="{ element: col }">
          <div class="w-72 shrink-0 bg-slate-50 dark:bg-slate-950/40 border border-linie rounded-xl flex flex-col max-h-[calc(100vh-20rem)]">
            <div class="flex items-center gap-2 px-3 py-2.5 border-b border-linie">
              <span class="col-handle cursor-grab text-slate-300 dark:text-slate-600 select-none">⠿</span>
              <span class="font-bold text-sm text-schrift truncate">{{ col.name }}</span>
              <span class="text-xs font-semibold text-slate-400">{{ col.cards.length }}</span>
              <div class="ml-auto relative">
                <button @click="columnMenu = columnMenu === col.id ? null : col.id" class="p-1 text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 rounded-xl"><MoreVertical class="w-4 h-4" /></button>
                <div v-if="columnMenu === col.id" class="absolute right-0 top-8 z-20 w-40 karte shadow-lg py-1 text-sm">
                  <button @click="renameColumn(col)" class="w-full text-left px-3 py-2 hover:bg-auflage flex items-center gap-2"><Pencil class="w-3.5 h-3.5" /> {{ t('common.rename') }}</button>
                  <button @click="askDeleteColumn(col)" class="w-full text-left px-3 py-2 hover:bg-rose-50 dark:hover:bg-rose-900/20 text-rose-600 flex items-center gap-2"><Trash2 class="w-3.5 h-3.5" /> {{ t('common.delete') }}</button>
                </div>
              </div>
            </div>

            <!-- Karten -->
            <div class="p-2 flex-1 overflow-y-auto">
              <draggable :list="col.cards" item-key="id" group="cards" @change="(e) => onCardChange(col, e)" class="space-y-2 min-h-[2rem]">
                <template #item="{ element: card }">
                  <div @click="openCard(card, col.id)"
                    class="karte p-3 shadow-sm cursor-pointer hover:border-marke transition-colors">
                    <p class="text-sm font-semibold text-schrift">{{ card.title }}</p>
                    <span v-if="card.subject" class="inline-block mt-1.5 text-xs font-bold px-1.5 py-0.5 rounded"
                      :style="card.subject.color ? { backgroundColor: card.subject.color + '20', color: card.subject.color } : null"
                      :class="!card.subject.color ? 'bg-marke-leise text-marke' : ''">
                      {{ card.subject.short || card.subject.name }}
                    </span>
                    <div v-if="card.assigned_to || card.due_date" class="flex items-center gap-3 mt-2 text-xs">
                      <span v-if="card.assigned_to" class="flex items-center gap-1 text-leise"><UserCircle class="w-3.5 h-3.5" /> {{ card.assignee_name || memberName(card.assigned_to) }}</span>
                      <span v-if="card.due_date" class="flex items-center gap-1 font-semibold" :class="isOverdue(card.due_date) ? 'text-rose-500' : 'text-leise'"><CalendarClock class="w-3.5 h-3.5" /> {{ card.due_date }}</span>
                      <span v-if="card.place" class="flex items-center gap-1 min-w-0 text-leise"><MapPin class="w-3.5 h-3.5 shrink-0" /> <span class="truncate">{{ card.place.name }}</span></span>
                    </div>
                  </div>
                </template>
              </draggable>
            </div>

            <!-- Schnelleingabe -->
            <div class="p-2 border-t border-linie">
              <input v-model="quickAdd[col.id]" @keyup.enter="addCard(col)" type="text" :placeholder="t('projects.boards.quickAddPlaceholder')"
                class="w-full px-3 py-2 karte text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift" />
            </div>
          </div>
        </template>
      </draggable>
    </div>

    <!-- ===== KARTEN-MODAL ===== -->
    <div v-if="cardModal" class="fixed inset-0 bg-slate-900/60 backdrop-blur-sm flex items-center justify-center p-4 z-[100]" @click.self="cardModal = null">
      <div class="karte w-full max-w-lg shadow-2xl max-h-[90vh] overflow-y-auto">
        <div class="flex items-center justify-between px-6 py-4 border-b border-linie">
          <h3 class="font-extrabold text-schrift">{{ t('projects.boards.cardModalTitle') }}</h3>
          <button @click="cardModal = null" class="p-1.5 text-slate-400 hover:bg-auflage rounded-xl"><X class="w-5 h-5" /></button>
        </div>
        <div class="p-6 space-y-4">
          <div class="space-y-1.5">
            <label class="text-xs font-bold text-leise uppercase">{{ t('projects.boards.titleLabel') }}</label>
            <input v-model="cardModal.title" type="text" maxlength="255" class="w-full px-4 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift" />
          </div>
          <div class="space-y-1.5">
            <label class="text-xs font-bold text-leise uppercase">{{ t('projects.boards.descriptionLabel') }}</label>
            <div class="border border-linie rounded-xl overflow-hidden h-56">
              <MarkdownEditor v-model="cardModal.description" class="h-full" />
            </div>
          </div>
          <div class="grid grid-cols-2 gap-4">
            <div v-if="kann('zuweisen')" class="space-y-1.5">
              <label class="text-xs font-bold text-leise uppercase">{{ t('projects.boards.assignmentLabel') }}</label>
              <select v-model="cardModal.assigned_to" class="w-full px-3 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift">
                <option :value="null">{{ t('projects.boards.nobody') }}</option>
                <option v-for="m in members" :key="m.user_id" :value="m.user_id">{{ m.name }}</option>
              </select>
            </div>
            <div class="space-y-1.5">
              <label class="text-xs font-bold text-leise uppercase">{{ t('projects.boards.dueDateLabel') }}</label>
              <input v-model="cardModal.due_date" type="date" class="w-full px-3 py-2.5 bg-vertieft border border-linie rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-marke text-schrift" />
            </div>
          </div>
          <PlacePicker v-model="cardModal.place_id" :project-id="projectId" />
          <!-- Verbindet die Hausaufgabe mit der Stunde, in der sie abzugeben
               ist – die Tagesansicht liest genau das. -->
          <SubjectPicker v-model="cardModal.subject_id" :project-id="projectId" />
        </div>
        <div class="flex items-center justify-between px-6 py-4 border-t border-linie">
          <button @click="deleteCard" class="flex items-center gap-1.5 px-4 py-2 text-rose-600 hover:bg-rose-50 dark:hover:bg-rose-900/20 rounded-xl text-sm font-bold"><Trash2 class="w-4 h-4" /> {{ t('common.delete') }}</button>
          <div class="flex gap-2">
            <button @click="cardModal = null" class="px-4 py-2 text-fliess hover:bg-auflage rounded-xl text-sm font-bold">{{ t('common.cancel') }}</button>
            <BaseButton @click="saveCard" :loading="savingCard">{{ t('common.save') }}</BaseButton>
          </div>
        </div>
      </div>
    </div>

    <!-- ===== SPALTE LÖSCHEN (mit Karten) ===== -->
    <div v-if="deleteColumnModal" class="fixed inset-0 bg-slate-900/60 backdrop-blur-sm flex items-center justify-center p-4 z-[100]" @click.self="deleteColumnModal = null">
      <div class="karte p-6 w-full max-w-md shadow-2xl space-y-4">
        <h3 class="font-extrabold text-lg text-schrift">{{ t('projects.boards.deleteColumnTitle', { name: deleteColumnModal.column.name }) }}</h3>
        <p class="text-sm text-leise">{{ t('projects.boards.deleteColumnText', { count: deleteColumnModal.column.cards.length }) }}</p>
        <select v-model="deleteColumnModal.targetColumnId" class="w-full px-3 py-2.5 bg-vertieft border border-linie rounded-xl text-sm text-schrift">
          <option v-for="c in otherColumns(deleteColumnModal.column.id)" :key="c.id" :value="c.id">{{ c.name }}</option>
        </select>
        <div class="flex justify-end gap-2">
          <button @click="deleteColumnModal = null" class="px-4 py-2 text-fliess hover:bg-auflage rounded-xl text-sm font-bold">{{ t('common.cancel') }}</button>
          <button @click="doDeleteColumn(deleteColumnModal.column, deleteColumnModal.targetColumnId)" :disabled="!deleteColumnModal.targetColumnId" class="px-5 py-2 bg-rose-600 hover:bg-rose-700 disabled:opacity-50 text-white rounded-xl text-sm font-bold">{{ t('projects.boards.moveAndDelete') }}</button>
        </div>
      </div>
    </div>
  </div>
</template>
