<script setup>
/*
 * Papierkorb — die Ansicht der Webapp, mit der lokalen SQLite dahinter.
 *
 * Was hier wiederhergestellt oder endgültig gelöscht wird, geht beim nächsten
 * Abgleich mit: als wieder da bzw. als Grabstein, damit die Gegenseite es
 * nicht zurückspielt.
 */
import Papierkorb from '@oberflaeche/papierkorb/PapierkorbAnsicht.vue';
import { lokalePapierkorbQuelle } from '../quellen/papierkorb';

const quelle = lokalePapierkorbQuelle();
</script>

<template>
  <Papierkorb :data-source="quelle" />
</template>
