import { ref } from 'vue';

// Promise-basierter Ersatz für window.confirm(): `await confirmDialog(...)`
// bzw. `await confirmDelete(...)` liefert true/false. Gerendert von
// components/base/ConfirmHost.vue (einmal in App.vue eingebunden).

const active = ref(null); // { message, title, confirmLabel, cancelLabel, danger, resolve }

const confirmDialog = (message, opts = {}) => new Promise((resolve) => {
  active.value = {
    message,
    title: opts.title ?? 'Bist du sicher?',
    confirmLabel: opts.confirmLabel ?? 'Bestätigen',
    cancelLabel: opts.cancelLabel ?? 'Abbrechen',
    danger: !!opts.danger,
    resolve,
  };
});

// Voreinstellung für destruktive Aktionen (roter Bestätigen-Button).
const confirmDelete = (message, opts = {}) => confirmDialog(message, {
  title: 'Wirklich löschen?',
  confirmLabel: 'Löschen',
  danger: true,
  ...opts,
});

const respond = (value) => {
  active.value?.resolve(value);
  active.value = null;
};

export function useConfirm() {
  return { active, confirmDialog, confirmDelete, respond };
}
