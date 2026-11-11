/*
 * Rechtstexte von openany.de holen. Die zuletzt geholte Fassung bleibt für die
 * Zeit ohne Netz liegen. Das ist nur eine Bequemlichkeit: Fehlt sie, sagt die
 * Anzeige, dass der Text gerade nicht erreichbar ist.
 *
 * JEDER `invoke` AUF EINER ZEILE — verdrahtung.rs liest genau diese Zeilen.
 */
import { invoke } from '@tauri-apps/api/core';

const merkName = (art) => `openany_rechtstext_${art}`;

export function gemerkterRechtstext(art) {
    try {
        return localStorage.getItem(merkName(art)) ?? '';
    } catch {
        return '';
    }
}

/** Der aktuelle Text. Wirft, wenn der Server nicht erreichbar ist. */
export async function rechtstextHolen(art) {
    const text = await invoke('rechtstext_holen', { art });
    try {
        localStorage.setItem(merkName(art), text);
    } catch {
        // Dann eben beim nächsten Mal wieder aus dem Netz.
    }
    return text;
}
