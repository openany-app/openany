<script setup>
/*
 * Kalender — die Arbeitsfläche der Webapp, mit der lokalen SQLite dahinter.
 *
 * Bis zum 15.09.2026 stand hier eine eigene Wochenliste ohne Serien. Jetzt ist
 * es dieselbe Komponente wie drüben: Monat und Woche, Serientermine,
 * Kalender anlegen, einfärben, ausblenden.
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
