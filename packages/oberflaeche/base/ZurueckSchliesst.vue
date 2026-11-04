<script setup>
// Für Rahmen ohne eigene Komponente (etwa ein Bild groß): solange dies
// eingehängt ist, schließt Zurück den Rahmen (useZurueckSchliesst).
import { useZurueckSchliesst } from '@oberflaeche/composables/useZurueckSchliesst';

const emit = defineEmits(['schliessen']);
useZurueckSchliesst(() => emit('schliessen'));
</script>

<template>
  <span hidden></span>
</template>
