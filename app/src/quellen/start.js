/*
 * Die Datenquelle der Startseiten-Kacheln — die lokale SQLite statt der API.
 * Dieselbe Antwortform wie /api/dashboard, nur mit den Teilen, die das
 * Programm trägt (Termine, Notizen, Kontakte). Befehl: `start_uebersicht`.
 */
import { invoke } from '@tauri-apps/api/core';

export function lokaleStartQuelle() {
    return {
        getDashboard: async () => ({ data: await invoke('start_uebersicht') }),
    };
}
