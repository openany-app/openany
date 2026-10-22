/*
 * Sicherheitsnummer (docs/plan-adressbuch-e2e.md, 2c) -- für Nachrichten und
 * Adressbuch. Die Befehle: `sicherheitsnummer*` in src-tauri/src/direktpost.rs.
 *
 * JEDER `invoke` AUF EINER ZEILE mit einfachen Schlüsseln — verdrahtung.rs
 * liest genau diese Zeilen.
 */
import { invoke } from '@tauri-apps/api/core';

export const sicherheitQuelle = {
    sicherheitsnummer: async (name) => await invoke('sicherheitsnummer', { name }),
    sicherheitsnummerBestaetigen: async (name) => await invoke('sicherheitsnummer_bestaetigen', { name }),
    sicherheitsnummerAufheben: async (name) => await invoke('sicherheitsnummer_aufheben', { name }),
    sicherheitsnummerStand: async (namen) => await invoke('sicherheitsnummer_stand', { namen }),
};
