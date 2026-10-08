/*
 * Die Datenquelle des Papierkorbs — die lokale SQLite statt der API.
 *
 * Dieselben Methodennamen wie der api-Service der Webapp, Antworten als
 * `{ data: … }`. Die Befehle dahinter: `papierkorb_*` in src-tauri/src/lib.rs.
 * Hier liegen Notizen, Kalender, Termine und Kontakte.
 *
 * JEDER `invoke` AUF EINER ZEILE — verdrahtung.rs liest genau diese Zeilen.
 */
import { invoke } from '@tauri-apps/api/core';

export function lokalePapierkorbQuelle() {
    return {
        getTrash: async (page) => ({ data: await invoke('papierkorb_liste', { seite: page }) }),
        restoreTrashItem: async (type, id) => ({ data: await invoke('papierkorb_zurueck', { art: type, id }) }),
        forceDeleteTrashItem: async (type, id) => ({ data: await invoke('papierkorb_endgueltig', { art: type, id }) }),
    };
}
