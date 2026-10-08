<script setup>
// Gemeinsame Kopf-Kachel der Modul-Seiten (Notizen, Projekte, Kalender,
// Speicher …): links die Icon-Kachel, rechts ein Slot für die modul-eigenen
// Bedienelemente. Zentralisiert Größe, Abstände und Icon-Stil, damit die
// Kopf-Kacheln überall gleich aussehen.
//
// Einen Modul-TITEL nimmt die Kachel bewusst nicht entgegen: Welches Modul
// offen ist, zeigt die Navigation. Es gab hier einmal einen title-Prop, den
// alle sechs Aufrufstellen brav befüllten und den das Template nie
// ausgegeben hat.
defineProps({
  icon: { type: [Object, Function], required: true }, // lucide-Icon-Komponente
  // Mobil oben festkleben. Für gestapelte Köpfe (z. B. Speicher: Reiter +
  // Bereichs-Aktionen) nur den obersten sticky lassen, den Rest static.
  sticky: { type: Boolean, default: true },
});
</script>

<template>
  <!-- Mobil sticky am oberen Rand (kein Desktop-Header da); ab md wieder im
       Fluss (dort ist der App-Header bereits sticky). -->
  <!-- sticky=true (oberste Kachel): mobil eine randlose Leiste wie die
       Bottom-Nav – bis an die Ränder (negative Margins heben ModulePage's
       Padding auf), ohne runde Ecken/Schatten, nur unten eine Trennlinie.
       Ab md wieder normale Karte im Fluss. sticky=false: immer Karte. -->
  <div
    class="flex flex-row items-center justify-between gap-2 sm:gap-4 bg-flaeche border-linie p-3 sm:p-4 transition-colors duration-300"
    :class="sticky
      ? 'sticky top-0 z-20 -mx-1 sm:-mx-6 -mt-1 sm:-mt-4 rounded-none border-b shadow-none md:static md:z-auto md:mx-0 md:mt-0 md:rounded-xl md:border md:shadow-sm'
      : 'border rounded-xl shadow-sm'"
  >
    <!-- Nur die Icon-Kachel; der Modul-Titel entfällt (Menü zeigt das Modul). -->
    <div class="w-9 h-9 rounded-xl bg-marke text-white flex items-center justify-center shadow-lg shadow-marke/30 shrink-0">
      <component :is="icon" class="w-5 h-5" />
    </div>
    <!-- Modul-Bedienelemente: EIN Wurzel-Element (Card ist justify-between) -->
    <slot />
  </div>
</template>
