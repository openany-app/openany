/*
 * Projekte aus der lokalen SQLite (src-tauri/src/projektbefehle.rs).
 *
 * JEDER `invoke` STEHT AUF EINER ZEILE mit einfachen Schlüsseln: Der Test
 * src-tauri/tests/verdrahtung.rs liest genau diese Zeilen.
 */
import { invoke } from '@tauri-apps/api/core';

export const projekteQuelle = {
    liste: () => invoke('projekte_liste'),
    abgleichen: () => invoke('projekte_abgleichen'),
    // Eine Art Projekt (docs/plan-projekte-eine-art.md, E3): Es entsteht auf
    // dem Gerät, mit Kopplung gleich auch auf openany.de. Antwort: { id, fehler }.
    anlegen: (name) => invoke('projekt_anlegen', { name }),
    // Der Schalter „auf openany.de" unter Einstellungen → Abgleich.
    gruendungen: () => invoke('projekte_gruendungen'),
    einschalten: (projekt) => invoke('projekt_einschalten', { projekt }),
    ausschalten: (projekt) => invoke('projekt_ausschalten', { projekt }),
    mitglieder: (projekt) => invoke('projekt_mitglieder', { projekt }),
    lokalLoeschen: (projekt) => invoke('projekt_lokal_loeschen', { projekt }),
    mitgliedEntfernen: (projekt, personenId) => invoke('projekt_mitglied_entfernen', { projekt, personenId }),
    austreten: (projekt) => invoke('projekt_austreten', { projekt }),
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
    // Geteilte Notiz-Mappen, Ende-zu-Ende (docs/plan-notizen-e2e.md, 6c).
    mappen: (projekt) => invoke('projekt_mappen', { projekt }),
    mappeTeilen: (projekt, mappe, stufe) => invoke('projekt_mappe_teilen', { projekt, mappe, stufe }),
    mappeBeenden: (projekt, freigabe) => invoke('projekt_mappe_beenden', { projekt, freigabe }),
    // Geteilte Alben und Ordner, Ende-zu-Ende (docs/plan-dateien-e2e.md, 7c).
    sacheTeilen: (projekt, art, ziel) => invoke('projekt_sache_teilen', { projekt, art, ziel }),
    freigabeSachen: (projekt, freigabe) => invoke('projekt_freigabe_sachen', { projekt, freigabe }),
    freigabeHolen: (projekt, freigabe, uuid, vorschau) => invoke('projekt_freigabe_holen', { projekt, freigabe, uuid, vorschau }),
    freigabeBytes: async (projekt, freigabe, uuid) => base64Bytes(await invoke('projekt_freigabe_bytes', { projekt, freigabe, uuid })),
};

function base64Bytes(b64) {
    const text = atob(b64);
    const bytes = new Uint8Array(text.length);
    for (let i = 0; i < text.length; i += 1) bytes[i] = text.charCodeAt(i);
    return bytes;
}
