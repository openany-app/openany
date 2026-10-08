<script setup>
// Einheitlicher Seiten-Rahmen der Modul-Seiten: zentrierte Breite, seitliche
// und vertikale Ränder sowie der vertikale Rhythmus zwischen den Blöcken.
// Zentralisiert, damit alle Modul-Seiten denselben Rahmen haben (siehe auch
// ModuleHeader für die Kopf-Kachel).
</script>

<template>
  <div class="max-w-7xl mx-auto px-1 sm:px-6 lg:px-8 pt-1 sm:pt-4 pb-1 sm:pb-8 space-y-1 sm:space-y-6">
    <slot />
  </div>
</template>
