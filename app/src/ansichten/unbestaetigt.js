/*
 * Wie viele eigene, freigegebene Geräte nicht vom Konto gesiegelt sind
 * (docs/plan-geraete-vertrauen.md) -- EINE Quelle für das Schild im Kopf
 * (breit) und den Streifen oben (schmal), damit die Lage nur einmal geholt
 * wird. Sie kostet ein paar Anfragen an den Server: beim Start und dann alle
 * fünf Minuten, solange jemand hinsieht.
 */
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const anzahl = ref(0);
let nutzer = 0;
let takt = null;

async function laden() {
    try {
        anzahl.value = (await invoke('nachrichtenschluessel_lage'))?.unbestaetigt ?? 0;
    } catch {
        anzahl.value = 0; // ohne Verbindung: nichts zu sagen
    }
}

export function unbestaetigteGeraete() {
    return {
        anzahl,
        anmelden() {
            if (nutzer++ === 0) {
                laden();
                takt = setInterval(laden, 5 * 60 * 1000);
            }
        },
        abmelden() {
            if (--nutzer === 0) {
                clearInterval(takt);
                takt = null;
            }
        },
    };
}
