<script setup>
// Einheitlicher Seiten-Rahmen der Modul-Seiten: zentrierte Breite, seitliche
// und vertikale Ränder sowie der vertikale Rhythmus zwischen den Blöcken.
// Zentralisiert, damit alle Modul-Seiten denselben Rahmen haben (siehe auch
// ModuleHeader für die Kopf-Kachel).
//
// AUF DEM TELEFON OHNE RAND: Die Inhalte reichen bis an
// den Bildschirmrand, und die Kästen darin verlieren ihre Seitenlinien und
// runden Ecken (`.modul-seite` in stil.css). Ab `md` -- wo die Leiste unten
// der Kopfzeile weicht -- wieder mit Abstand.
</script>

<template>
  <div class="modul-seite max-w-7xl mx-auto md:px-6 lg:px-8 sm:pt-4 sm:pb-8 space-y-1 sm:space-y-6">
    <slot />
  </div>
</template>
