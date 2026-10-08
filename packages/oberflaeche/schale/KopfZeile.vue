<script setup>
// Die Kopfzeile am Schreibtisch (ab `md`, 768 px): links das Zeichen, dann
// die Orte, rechts, was die Anwendung dort hinstellt.
//
// DIE GRENZE IST `md` – für ALLE Stellen zusammen, sonst stehen Leiste unten
// und Kopf oben gleichzeitig da. Ein Tablet hochkant ist rund 600 CSS-Pixel
// breit (Leiste), quer rund 1000 (Kopf).
//
// Ohne Router: Orte mit `to` zeichnet die Link-Komponente aus
// `oberflaeche:link`, sonst ein Knopf mit `waehlen`.
import { inject } from 'vue';
import Marke from './Marke.vue';

defineProps({
  orte: { type: Array, required: true }, // [{ id, label, icon, to? }]
  aktiv: { type: String, default: null },
  verbunden: { type: Boolean, default: true },
  startLabel: { type: String, default: '' },
  startZiel: { type: String, default: null },
});
defineEmits(['start', 'waehlen']);

const Link = inject('oberflaeche:link', null);
const AKTIV = 'bg-marke text-white shadow-sm shadow-marke/30';
</script>

<template>
  <header class="hidden md:block bg-flaeche border-b border-linie sticky z-50 transition-colors duration-300"
          :style="{ top: 'var(--rand-oben)' }">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 h-16 flex items-center justify-between">
      <component
        :is="startZiel && Link ? Link : 'button'"
        :to="startZiel && Link ? startZiel : undefined"
        :title="startLabel"
        class="flex hover:opacity-90 transition-opacity cursor-pointer"
        @click="$emit('start')"
      >
        <Marke :verbunden="verbunden" />
      </component>

      <nav class="flex items-center gap-2 mx-4 min-w-0">
        <component
          v-for="ort in orte"
          :key="ort.id"
          :is="ort.to && Link ? Link : 'button'"
          :to="ort.to && Link ? ort.to : undefined"
          class="group px-2.5 py-2 rounded-xl text-sm font-bold transition-all flex items-center gap-1.5 cursor-pointer"
          :class="aktiv === ort.id ? AKTIV : 'text-marke hover:bg-marke-leise'"
          @click="$emit('waehlen', ort.id)"
        >
          <component :is="ort.icon" class="w-4.5 h-4.5" :class="aktiv === ort.id ? 'text-white' : 'text-marke'" />
          <span>{{ ort.label }}</span>
        </component>
      </nav>

      <div class="flex items-center gap-3">
        <slot name="rechts" />
      </div>
    </div>
  </header>
</template>
