<script setup>
// Die Leiste unten auf Telefon und Tablet: links das Menü, dann die Orte.
//
// DIE LEISTE TRÄGT ORTE, DAS MENÜ-BLATT TRÄGT WERKZEUGE. Damit ist sie
// gedeckelt und wächst nicht mit jeder neuen Funktion.
//
// Nicht-aktive Punkte tragen die Marke wie am Schreibtisch; aktiv ist eine
// quadratische Kachel. Ohne Innenabstand: Die volle Kachelbreite reicht in
// allen Sprachen – mit `px-1` wurden „Calendario" und „Календар" abgeschnitten.
//
// Ohne Router: Punkte mit `to` zeichnet die Link-Komponente aus
// `oberflaeche:link`, sonst ein Knopf mit `waehlen`.
//
// UNTEN DER RAND DES GERÄTS (`--rand-unten`, siehe stil.css): Die Leiste
// wächst um ihn, statt ihn von ihren 4rem abzuziehen.
import { inject } from 'vue';
import { Menu, X } from 'lucide-vue-next';

defineProps({
  // [{ id, label, icon, to? }] – label schon übersetzt
  orte: { type: Array, required: true },
  aktiv: { type: String, default: null },
  menueOffen: { type: Boolean, default: false },
  menueLabel: { type: String, default: '' },
  zaehler: { type: Number, default: 0 },
});
defineEmits(['menue', 'waehlen']);

const Link = inject('oberflaeche:link', null);
const AKTIV = 'bg-marke text-white shadow-sm shadow-marke/30';
</script>

<template>
  <nav class="md:hidden fixed bottom-0 inset-x-0 z-50 bg-flaeche border-t border-linie transition-colors duration-300"
       :style="{ paddingBottom: 'var(--rand-unten)' }">
    <div class="flex items-stretch justify-around h-16">
      <button
        @click="$emit('menue')"
        :aria-label="menueLabel"
        :title="menueLabel"
        class="relative flex items-center justify-center py-2 pl-4 pr-6 shrink-0 border-r border-linie text-marke transition-colors cursor-pointer"
      >
        <component :is="menueOffen ? X : Menu" class="w-9 h-9" />
        <span v-if="zaehler > 0 && !menueOffen" class="absolute top-1 right-3 min-w-[1.1rem] h-[1.1rem] px-1 flex items-center justify-center text-[10px] font-extrabold bg-rose-500 text-white rounded-full leading-none border-2 border-white dark:border-slate-900">
          {{ zaehler }}
        </span>
      </button>
      <component
        v-for="ort in orte"
        :key="ort.id"
        :is="ort.to && Link ? Link : 'button'"
        :to="ort.to && Link ? ort.to : undefined"
        class="flex items-center justify-center min-w-0 flex-1 text-[10px] font-bold cursor-pointer"
        @click="$emit('waehlen', ort.id)"
      >
        <span class="flex flex-col items-center justify-center gap-0.5 w-14 h-14 rounded-xl transition-colors"
              :class="aktiv === ort.id && !menueOffen ? AKTIV : 'text-marke'">
          <component :is="ort.icon" class="w-5 h-5 shrink-0" />
          <span class="truncate max-w-full">{{ ort.label }}</span>
        </span>
      </component>
    </div>
  </nav>
</template>
