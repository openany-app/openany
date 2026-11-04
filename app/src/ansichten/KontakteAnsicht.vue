<script setup>
/*
 * Adressbuch — die Arbeitsfläche aus dem gemeinsamen Paket, mit der lokalen
 * SQLite dahinter: beliebig viele Nummern, Adressen und Kennungen je
 * Kontakt, Foto, Suche.
 *
 * DER WEG FÜHRT IN BEIDE RICHTUNGEN, zu den Nachrichten und zurück: Auf dem
 * Tablet gibt es keinen Zurück-Knopf des Browsers, und in der Leiste stehen
 * Kontakte absichtlich nicht.
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

// Wartet dieses Gerät auf Freigabe, liegen beim Server Kontakte, die es noch
// nicht lesen kann -- die Arbeitsfläche sagt das statt „keine Kontakte".
const wartet = ref(false);

onMounted(async () => {
    quelle.wartet().then((w) => { wartet.value = w; }).catch(() => {});
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
    :wartet="wartet"
    @nachrichten="emit('oeffnen', 'messages')"
    @schreiben="emit('oeffnen', 'messages', $event)"
  />
</template>
