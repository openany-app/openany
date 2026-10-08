<script setup>
/*
 * Adressbuch — die Arbeitsfläche der Webapp, mit der lokalen SQLite dahinter.
 *
 * Bis zum 15.09.2026 stand hier eine eigene Tabelle mit zwei Kennungen. Jetzt
 * ist es dieselbe Komponente wie drüben: beliebig viele Nummern, Adressen und
 * Kennungen je Kontakt, Foto, Suche.
 *
 * SEIT DEM 24.09.2026 FÜHRT DER WEG IN BEIDE RICHTUNGEN. Hier stand, ein Weg
 * zu den Nachrichten gebe es erst, „solange das Programm keinen Chat hat" —
 * den hat es jetzt. Ohne ihn war das Adressbuch eine Sackgasse: Die Webapp
 * hat den Zurück-Knopf des Browsers, auf dem Tablet gibt es den nicht, und in
 * der Leiste stehen Kontakte absichtlich nicht.
 *
 * Der Klick auf eine Kennung meldet `schreiben`; wohin das führt, weiss die
 * Schale. Einen Router hat dieses Programm nicht.
 */
import { ref, onMounted } from 'vue';
import AdressbuchArbeitsflaeche from '@oberflaeche/adressbuch/AdressbuchArbeitsflaeche.vue';
import { lokaleAdressbuchQuelle } from '../quellen/adressbuch';
import { nachrichtenQuelle } from '../quellen/nachrichten';

const emit = defineEmits(['oeffnen']);

const quelle = lokaleAdressbuchQuelle();

/*
 * OB EIN MATRIX-KONTO VERBUNDEN IST, entscheidet, ob eine Matrix-Kennung
 * anklickbar ist. Ohne Konto gäbe es drüben im Schreibfeld den Matrix-Weg
 * gar nicht — der Verweis führte in ein Formular, das ihn nicht anbietet.
 */
const matrixVerbunden = ref(false);

onMounted(async () => {
    try {
        matrixVerbunden.value = Boolean((await nachrichtenQuelle.lage())?.mxid);
    } catch {
        // Kein Matrix-Konto zu kennen ist kein Fehler: Dann bleibt die
        // Kennung schlichter Text, wie vorher auch.
    }
});
</script>

<template>
  <AdressbuchArbeitsflaeche
    :data-source="quelle"
    nachrichten
    :matrix-verbunden="matrixVerbunden"
    @nachrichten="emit('oeffnen', 'messages')"
    @schreiben="emit('oeffnen', 'messages', $event)"
  />
</template>
