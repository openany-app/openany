/*
 * Projekte — die Boards aus der lokalen SQLite (src-tauri/src/projektbefehle.rs).
 *
 * Der Baum (Board → Spalte → Karte) entsteht in Rust und nicht hier: Die
 * Oberfläche bekäme sonst eine flache Liste und müsste die Zuordnung
 * nachbauen — dieselbe Regel zweimal, und die zweite veraltet.
 *
 * JEDER `invoke` STEHT AUF EINER ZEILE mit einfachen Schlüsseln: Der Test
 * src-tauri/tests/verdrahtung.rs liest genau diese Zeilen.
 */
import { invoke } from '@tauri-apps/api/core';

export const projekteQuelle = {
    liste: () => invoke('projekte_liste'),
    abstimmen: (projekt, option, antwort) => invoke('projekt_abstimmen', { projekt, option, antwort }),
    abgleichen: () => invoke('projekte_abgleichen'),
    // Lokale Projekte (01.10.2026): ohne Konto, mit unterschriebener Mitgliederliste.
    lokalAnlegen: (name) => invoke('projekt_lokal_anlegen', { name }),
    mitglieder: (projekt) => invoke('projekt_mitglieder', { projekt }),
    lokalLoeschen: (projekt) => invoke('projekt_lokal_loeschen', { projekt }),
    mitgliedEntfernen: (projekt, personenId) => invoke('projekt_mitglied_entfernen', { projekt, personenId }),
    austreten: (projekt) => invoke('projekt_austreten', { projekt }),
    // Chat vor Ort.
    chat: (projekt) => invoke('projekt_chat', { projekt }),
    chatSenden: (projekt, text) => invoke('projekt_chat_senden', { projekt, text }),
    chatGelesen: (projekt) => invoke('projekt_chat_gelesen', { projekt }),
    // Freigaben in lokale Projekte und ihr Abgleich vor Ort.
    lokaleFreigaben: (projekt) => invoke('projekt_lokale_freigaben', { projekt }),
    lokalFreigeben: (projekt, art, schluessel, name, freigeben, bearbeiten) => invoke('projekt_lokal_freigeben', { projekt, art, schluessel, name, freigeben, bearbeiten }),
    // Fremde Notizen bearbeiten: Sperre beim Gerät dessen, der freigegeben hat.
    notizSperren: (projekt, id) => invoke('projekt_notiz_sperren', { projekt, id }),
    notizSpeichern: (projekt, id, geraet, titel, inhalt) => invoke('projekt_notiz_speichern', { projekt, id, geraet, titel, inhalt }),
    notizEntsperren: (projekt, id, geraet) => invoke('projekt_notiz_entsperren', { projekt, id, geraet }),
    geteilterInhalt: async (projekt, id, vorschau) => base64Bytes(await invoke('projekt_geteilter_inhalt', { projekt, id, vorschau })),
    geteilteNotiz: (projekt, id) => invoke('projekt_geteilte_notiz', { projekt, id }),
    nahAbgleichen: () => invoke('projekte_nah_abgleichen'),
    // Freigaben (Weg 1): die Liste, der Inhalt einer fremden, ihre Bytes.
    freigaben: (projekt) => invoke('projekt_freigaben', { projekt }),
    freigabeInhalt: (projekt, art, id) => invoke('projekt_freigabe_inhalt', { projekt, art, id }),
    freigabeDatei: async (pfad) => base64Bytes(await invoke('projekt_freigabe_datei', { pfad })),
};

function base64Bytes(b64) {
    const text = atob(b64);
    const bytes = new Uint8Array(text.length);
    for (let i = 0; i < text.length; i += 1) bytes[i] = text.charCodeAt(i);
    return bytes;
}
