<script setup>
// EIN Menü-Eintrag, zwei Orte: das Profil-Menü am Schreibtisch und das
// Menü-Blatt auf dem Telefon. Die Liste dahinter ist dieselbe (App.vue,
// `menuEintraege`) – bis zum 08.09.2026 stand sie zweimal im Markup, mit
// unterschiedlichen Klassen für dieselben Einträge. Wer einen Punkt ergänzte,
// ergänzte ihn an einer Stelle und wunderte sich auf dem anderen Gerät.
//
// Die beiden Orte sehen NICHT gleich aus, und das sollen sie auch nicht: Das
// Blatt hat Daumengröße (py-3), das Dropdown ist kompakt (py-2.5). Der
// Unterschied steckt in `variante` und sonst nirgends.
//
// OHNE ROUTER-ZWANG (seit 15.09.2026, gemeinsames Paket): Einträge mit `to`
// werden mit der Link-Komponente gezeichnet, die die Anwendung unter
// `oberflaeche:link` bereitstellt (die Webapp gibt dort ihren Router-Link).
// Das Programm hat keinen Router und stellt nichts bereit – dann ist jeder
// Eintrag ein Knopf, und die Anwendung reagiert auf `click`.
import { inject } from 'vue';

const Link = inject('oberflaeche:link', null);

defineProps({
  eintrag: { type: Object, required: true }, // { id, label, icon, to?, ton? }
  variante: { type: String, default: 'blatt' }, // 'blatt' (mobil) | 'menue' (Desktop)
});

const GROESSE = {
  blatt: 'gap-3 px-4 py-3 rounded-xl',
  menue: 'gap-2.5 px-4 py-2.5',
};
const ICON = { blatt: 'w-4.5 h-4.5', menue: 'w-4 h-4' };
// `ton: 'warnung'` färbt Abmelden rot – der einzige Eintrag, der sich
// abhebt. Alles andere trägt den Fließtext-Ton.
const TON = {
  normal: 'text-fliess hover:bg-marke-leise',
  warnung: 'text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/30',
};
</script>

<template>
  <component
    :is="eintrag.to && Link ? Link : 'button'"
    :to="eintrag.to && Link ? eintrag.to : undefined"
    class="w-full flex items-center text-sm font-bold transition-colors"
    :class="[GROESSE[variante], TON[eintrag.ton || 'normal'], eintrag.to && Link ? '' : 'cursor-pointer']"
  >
    <component :is="eintrag.icon" :class="[ICON[variante], eintrag.ton === 'warnung' ? '' : 'text-slate-400']" />
    <span>{{ eintrag.label }}</span>
    <!-- Ungelesen-Zähler; heute nur an den Nachrichten. -->
    <span
      v-if="eintrag.zaehler"
      class="ml-auto min-w-[1.25rem] h-5 px-1.5 flex items-center justify-center text-[11px] font-extrabold bg-rose-500 text-white rounded-full"
    >{{ eintrag.zaehler }}</span>
  </component>
</template>
