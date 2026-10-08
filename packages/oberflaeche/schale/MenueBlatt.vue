<script setup>
// Das Menü-Blatt auf Telefon und Tablet: von unten, über der Leiste.
// Oben das Zeichen und Schließen, darunter die Einträge, abgesetzt Hell/Dunkel
// und – wo es eines gibt – Abmelden.
//
// „Über Openany" steht NICHT hier. Es wird einmal gelesen und dann nie wieder,
// und ein Menü mit sieben Einträgen macht die sechs wichtigen langsamer.
import { X } from 'lucide-vue-next';
import Marke from './Marke.vue';
import MenuEintrag from './MenuEintrag.vue';

defineProps({
  eintraege: { type: Array, required: true },
  helligkeit: { type: Object, required: true }, // { id, label, icon }
  abmelden: { type: Object, default: null },
  verbunden: { type: Boolean, default: true },
});
defineEmits(['schliessen', 'marke', 'eintrag', 'helligkeit', 'abmelden']);
</script>

<template>
  <div class="md:hidden fixed inset-0 z-[60]" @click.self="$emit('schliessen')">
    <div class="absolute inset-0 bg-slate-900/50 backdrop-blur-sm" @click="$emit('schliessen')"></div>
    <div class="absolute inset-x-0 px-3" :style="{ bottom: 'calc(4rem + var(--rand-unten))' }">
      <div class="karte shadow-2xl p-3 space-y-1">
        <div class="flex items-center justify-between px-2 pt-1 pb-2">
          <slot name="marke">
            <button class="flex cursor-pointer" @click="$emit('marke')">
              <Marke groesse="blatt" :verbunden="verbunden" />
            </button>
          </slot>
          <button @click="$emit('schliessen')" class="p-2 text-slate-400 hover:bg-auflage rounded-xl cursor-pointer">
            <X class="w-5 h-5" />
          </button>
        </div>

        <MenuEintrag v-for="e in eintraege" :key="e.id" :eintrag="e" @click="$emit('eintrag', e.id)" />
        <div class="border-t border-linie my-1"></div>
        <MenuEintrag :eintrag="helligkeit" @click="$emit('helligkeit')" />
        <MenuEintrag v-if="abmelden" :eintrag="abmelden" @click="$emit('abmelden')" />
      </div>
    </div>
  </div>
</template>
