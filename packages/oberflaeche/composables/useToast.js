import { ref } from 'vue';

// Globaler Toast-Zustand (Singleton): kurze Erfolgs-/Fehlermeldungen unten
// rechts statt blockierender alert()-Dialoge. Gerendert von
// components/base/ToastHost.vue (einmal in App.vue eingebunden).

const toasts = ref([]);
let nextId = 0;

const dismiss = (id) => {
  toasts.value = toasts.value.filter((t) => t.id !== id);
};

const push = (type, message, timeout) => {
  const id = ++nextId;
  toasts.value.push({ id, type, message });
  if (timeout) setTimeout(() => dismiss(id), timeout);
  return id;
};

export function useToast() {
  return {
    toasts,
    dismiss,
    success: (message) => push('success', message, 4000),
    // Fehler bleiben etwas länger stehen.
    error: (message) => push('error', message, 7000),
    info: (message) => push('info', message, 5000),
  };
}
