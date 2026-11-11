/*
 * Ist diese App zu alt für openany.de (B37)? Dann die Mindestversion, sonst ''.
 * Einmal je Start gefragt -- die Antwort steht neben der Versionsnummer in
 * Hilfe und Über-Ansicht als „Bitte aktualisieren“.
 */
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const mindestversion = ref('');
let gefragt = false;

export function useMindestversion() {
    if (!gefragt) {
        gefragt = true;
        invoke('app_mindestversion')
            .then((v) => { mindestversion.value = v ?? ''; })
            .catch(() => {});
    }
    return mindestversion;
}
