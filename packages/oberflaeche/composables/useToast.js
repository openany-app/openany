import { ref } from 'vue';

// Globaler Toast-Zustand (Singleton): kurze Erfolgs-/Fehlermeldungen unten
// rechts statt blockierender alert()-Dialoge. Gerendert von
// components/base/ToastHost.vue (einmal in App.vue eingebunden).

const toasts = ref([]);
let nextId = 0;

const dismiss = (id) => {
  toasts.value = toasts.value.filter((t) => t.id !== id);
};

// `aktion` (optional): { label, onClick } -- ein Knopf in der Meldung,
// etwa „Öffnen" nach dem Ablegen eines PDFs. Mit Knopf bleibt sie länger.
const push = (type, message, timeout, aktion = null) => {
  const id = ++nextId;
  toasts.value.push({ id, type, message, aktion });
  if (timeout) setTimeout(() => dismiss(id), timeout);
  return id;
};

export function useToast() {
  return {
    toasts,
    dismiss,
    success: (message, aktion = null) => push('success', message, aktion ? 9000 : 4000, aktion),
    // Fehler bleiben etwas länger stehen.
    error: (message) => push('error', message, 7000),
    info: (message) => push('info', message, 5000),
  };
}
