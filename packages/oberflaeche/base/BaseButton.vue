<script setup>
// Einheitlicher Button: eine Stelle für Farben/Radien/States statt
// duplizierter Tailwind-Ketten. Varianten decken die im UI etablierten
// Stile ab; `loading` zeigt einen Spinner und deaktiviert den Button.
//
// VIER GROESSEN, weil der Bestand vier Rollen hatte. Als hier nur eine stand,
// schrieben 47 Knöpfe ihre Kette lieber selbst aus – in 32 verschiedenen
// Fassungen (px-2.5 bis px-6, py-1 bis py-3.5, text-xs bis text-base).
// Wer eine vierte Größe braucht, braucht sie meistens nicht.
import { computed } from 'vue';
import { Loader2 } from 'lucide-vue-next';

const props = defineProps({
  variant: { type: String, default: 'primary' }, // primary | secondary | danger | ghost
  groesse: { type: String, default: 'normal' },  // klein | normal | gross | kopf
  type: { type: String, default: 'button' },
  loading: { type: Boolean, default: false },
  disabled: { type: Boolean, default: false },
  block: { type: Boolean, default: false }, // volle Breite
});

const VARIANTS = {
  primary: 'bg-marke hover:bg-marke-satt text-white shadow-md shadow-marke/20',
  secondary: 'bg-auflage hover:bg-slate-200 dark:hover:bg-slate-700 text-fliess',
  danger: 'bg-rose-600 hover:bg-rose-700 text-white shadow-md shadow-rose-500/20',
  ghost: 'text-fliess hover:bg-auflage',
};

// Die Icon-Größe wandert mit: ein 4×4-Symbol neben `text-base` wirkt verloren.
const GROESSEN = {
  klein:  'px-3 py-1.5 text-xs gap-1.5',
  normal: 'px-4 py-2.5 text-sm gap-2',
  gross:  'px-6 py-3 text-base gap-2',
  // `kopf` ist der Knopf in der Kopf-Kachel eines Moduls (ModuleHeader):
  // feste Höhe, damit er neben den anderen Bedienelementen auf einer Linie
  // steht, und auf dem Telefon schmaler, wo die Beschriftung ohnehin
  // ausgeblendet wird. Kam 13 Mal vor, in 5 leicht verschiedenen Fassungen.
  kopf:   'h-9 px-2.5 sm:px-4 py-2 text-sm gap-1.5',
};

const classes = computed(() => [
  'inline-flex items-center justify-center rounded-xl font-bold transition-all cursor-pointer',
  'disabled:opacity-50 disabled:cursor-not-allowed',
  GROESSEN[props.groesse] ?? GROESSEN.normal,
  VARIANTS[props.variant] ?? VARIANTS.primary,
  props.block ? 'w-full' : '',
]);

const spinner = computed(() => (props.groesse === 'gross' ? 'w-5 h-5' : 'w-4 h-4'));
</script>

<template>
  <button :type="type" :disabled="disabled || loading" :class="classes">
    <Loader2 v-if="loading" :class="[spinner, 'animate-spin']" />
    <slot />
  </button>
</template>
