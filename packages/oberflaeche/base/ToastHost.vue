<script setup>
// Rendert die Toasts aus useToast (einmal in App.vue eingebunden):
// unten rechts, stapelnd, Klick schließt.
import { useToast } from '@oberflaeche/composables/useToast';
import { CheckCircle2, AlertCircle, Info, X } from 'lucide-vue-next';

const { toasts, dismiss } = useToast();

const META = {
  success: { icon: CheckCircle2, cls: 'border-emerald-200 dark:border-emerald-900 text-emerald-600 dark:text-emerald-400' },
  error: { icon: AlertCircle, cls: 'border-rose-200 dark:border-rose-900 text-rose-600 dark:text-rose-400' },
  info: { icon: Info, cls: 'border-marke text-marke' },
};
</script>

<template>
  <div class="fixed bottom-4 right-4 z-[200] flex flex-col gap-2 w-[calc(100vw-2rem)] max-w-sm" aria-live="polite">
    <TransitionGroup
      enter-active-class="transition duration-200 ease-out"
      enter-from-class="opacity-0 translate-y-2"
      leave-active-class="transition duration-150 ease-in"
      leave-to-class="opacity-0"
    >
      <div v-for="t in toasts" :key="t.id"
        class="flex items-start gap-3 p-4 bg-flaeche border rounded-xl shadow-lg cursor-pointer"
        :class="META[t.type].cls"
        role="status"
        @click="dismiss(t.id)"
      >
        <component :is="META[t.type].icon" class="w-5 h-5 shrink-0" />
        <p class="text-sm font-bold text-schrift flex-1 min-w-0">{{ t.message }}</p>
        <X class="w-4 h-4 shrink-0 text-slate-300 dark:text-slate-600" />
      </div>
    </TransitionGroup>
  </div>
</template>
