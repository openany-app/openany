import { ref } from 'vue';
import { i18n } from '@oberflaeche/i18n';

const t = (k) => i18n.global.t(k);

// Promise-basierter Ersatz für window.confirm(): `await confirmDialog(...)`
// bzw. `await confirmDelete(...)` liefert true/false. Gerendert von
// components/base/ConfirmHost.vue (einmal in App.vue eingebunden).

const active = ref(null); // { message, title, confirmLabel, cancelLabel, danger, resolve }

const confirmDialog = (message, opts = {}) => new Promise((resolve) => {
  active.value = {
    message,
    title: opts.title ?? t('common.confirmTitle'),
    confirmLabel: opts.confirmLabel ?? t('common.confirm'),
    cancelLabel: opts.cancelLabel ?? t('common.cancel'),
    danger: !!opts.danger,
    resolve,
  };
});

// Voreinstellung für destruktive Aktionen (roter Bestätigen-Button).
const confirmDelete = (message, opts = {}) => confirmDialog(message, {
  title: t('common.reallyDeleteTitle'),
  confirmLabel: t('common.delete'),
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
