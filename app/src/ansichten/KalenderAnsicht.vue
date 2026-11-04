<script setup>
/*
 * Kalender — die Arbeitsfläche aus dem gemeinsamen Paket, mit der lokalen
 * SQLite dahinter: Monat und Woche, Serientermine, Kalender anlegen,
 * einfärben, ausblenden.
 *
 * Ohne `erweiterungen`: Abo-Link und Projekt-Einblendungen brauchen einen
 * Server und fehlen hier — mit ihren Knöpfen.
 */
import KalenderArbeitsflaeche from '@oberflaeche/kalender/KalenderArbeitsflaeche.vue';
import { lokaleKalenderQuelle } from '../quellen/kalender';

const quelle = lokaleKalenderQuelle();
</script>

<template>
  <KalenderArbeitsflaeche :data-source="quelle" />
</template>
