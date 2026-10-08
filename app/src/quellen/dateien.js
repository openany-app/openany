/*
 * Die Datenquelle für Dateien und Dokumente — die lokale SQLite und die
 * Inhaltsablage statt der API.
 *
 * Dieselben Methodennamen wie der api-Service der Webapp, Antworten als
 * `{ data: … }`. Die Befehle dahinter: src-tauri/src/dateibefehle.rs.
 *
 * SPEICHERN IN STÜCKEN VON 2 MB: `file.slice` liest nur das Stück, das gerade
 * gebraucht wird. Eine 300-MB-Datei liegt so nie im Ganzen im Speicher der
 * Webansicht.
 *
 * JEDER `invoke` AUF EINER ZEILE — verdrahtung.rs liest genau diese Zeilen.
 */
import { invoke } from '@tauri-apps/api/core';

const STUECK = 2 * 1024 * 1024;

/*
 * BASE64 UND NICHT ROH. Am Schreibtisch kämen rohe Bytes an; auf Android
 * geht jeder Aufruf als JSON über die Brücke der Webansicht, und ein
 * `Uint8Array` war dort am 15.09.2026 kein Stück mehr, sondern eine
 * Fehlermeldung. Ein Weg für alle Geräte ist verlässlicher als zwei.
 */
function base64(bytes) {
    let binaer = '';
    for (let i = 0; i < bytes.length; i += 0x8000) {
        binaer += String.fromCharCode.apply(null, bytes.subarray(i, i + 0x8000));
    }
    return btoa(binaer);
}

/** Die Stücke einer angemeldeten Ladung schicken; bei Fehler abbrechen. */
export async function stueckeSchicken(kennzeichen, datei, fortschritt) {
    try {
        for (let von = 0; von < datei.size; von += STUECK) {
            const stueck = base64(new Uint8Array(await datei.slice(von, von + STUECK).arrayBuffer()));
            await invoke('datei_hochladen_stueck', { kennzeichen, stueck });
            fortschritt?.(Math.min(1, (von + STUECK) / datei.size));
        }
    } catch (e) {
        await invoke('datei_hochladen_abbrechen', { kennzeichen }).catch(() => {});
        throw e;
    }
}

async function hochladen(ordner, datei, zone, fortschritt) {
    const kennzeichen = await invoke('datei_hochladen_beginnen', { zone, ordner, name: datei.name, mime: datei.type || '' });
    await stueckeSchicken(kennzeichen, datei, fortschritt);
    return await invoke('datei_hochladen_fertig', { kennzeichen });
}

/**
 * Die Bytes einer Datei -- und liegt der Inhalt auf einem anderen Gerät
 * („bei Bedarf"), erst holen. Die Texterkennung kennt nur die Kennung, nicht
 * die Zeile mit `vorhanden`.
 */
async function inhaltLesen(id) {
    try {
        return alsBytes(await invoke('datei_inhalt', { id }));
    } catch (e) {
        // Der Wortlaut kommt aus dateibefehle.rs („The content is not on this
        // device."). Ändert er sich dort, muss er hier mit.
        if (!String(e).includes('not on this device')) throw e;
        await invoke('datei_holen', { id });
        return alsBytes(await invoke('datei_inhalt', { id }));
    }
}

/**
 * Den Inhalt ersetzen: dieselben Stücke wie beim Hochladen, aber am Ende
 * `datei_ersetzen_fertig` statt einer neuen Datei.
 *
 * optionen.zone      'documents' (Vorgabe, die Texterkennung gibt es nur in
 *                    Akten) oder 'files' -- der PDF-Betrachter speichert in
 *                    beiden. Die Zone muss zu der der Datei passen.
 * optionen.vorfassung Zusatz für die alte Fassung im Papierkorb
 *                    („vor Bearbeitung"); ohne ihn wird sie weggeräumt.
 */
async function inhaltErsetzen(id, datei, optionen = {}) {
    const zone = optionen.zone || 'documents';
    const vorfassung = optionen.vorfassung || null;
    const kennzeichen = await invoke('datei_hochladen_beginnen', { zone, ordner: null, name: datei.name, mime: datei.type || '' });
    await stueckeSchicken(kennzeichen, datei);
    return await invoke('datei_ersetzen_fertig', { kennzeichen, id, vorfassung });
}

/**
 * @param {(datei) => Promise<void>} oeffnen  wie die Ansicht eine Datei zeigt
 */
export function lokaleDateienQuelle(oeffnen) {
    return {
        getFiles: async (ordner, _seite, zone) => ({ data: await invoke('dateien_liste', { zone, ordner }) }),
        createFolder: async (ordner, name, zone) => ({ data: await invoke('datei_ordner_anlegen', { zone, ordner, name }) }),
        renameFileNode: async (id, name) => ({ data: await invoke('datei_umbenennen', { id, name }) }),
        uploadFile: async (ordner, datei, zone, optionen) => ({ data: await hochladen(ordner, datei, zone, optionen?.fortschritt) }),
        deleteFileOrFolder: async (id) => ({ data: await invoke('datei_papierkorb', { id }) }),
        getFileTree: async (zone) => ({ data: await invoke('dateien_baum', { zone }) }),
        // Suche nach Namen über alle Ordner -- im eigenen Speicher, auch ohne Netz.
        sucheDateien: async (q, zone) => ({ data: await invoke('dateien_suchen', { zone, q }) }),
        // Der Inhaltsleser (Stufe 2): liest, was hier liegt, und legt den Text hier ab.
        textOffen: (zone, version) => invoke('dateien_text_offen', { zone, version }),
        textSetzen: (id, abdruck, stand, text, version) => invoke('datei_text_setzen', { id, abdruck, stand, text, version }),
        moveFileNode: async (id, ordner) => ({ data: await invoke('datei_verschieben', { id, ordner }) }),
        // Für die Texterkennung (@oberflaeche/texterkennung): das PDF lesen
        // und seinen Inhalt mit Textebene zurückschreiben.
        downloadFileContent: async (id) => ({ data: await inhaltLesen(id) }),
        replaceFileContent: async (id, datei, optionen) => ({ data: await inhaltErsetzen(id, datei, optionen) }),
        oeffnen,
        // Im PDF-Betrachter: „Mit anderem Programm öffnen".
        externOeffnen: dateiExternOeffnen,
    };
}

/*
 * BYTES AUS `ipc::Response` -- auf Android kommen sie als JSON-Zahlenliste
 * an, am Schreibtisch als ArrayBuffer (29.09.2026 am Tablet gesehen). Eine
 * Zahlenliste in einen Blob zu stecken, ergibt „37,80,68,…" statt der Datei.
 */
export function alsBytes(antwort) {
    if (antwort instanceof Uint8Array) return antwort;
    if (antwort instanceof ArrayBuffer) return new Uint8Array(antwort);
    if (Array.isArray(antwort)) return Uint8Array.from(antwort);
    return new Uint8Array(antwort ?? []);
}

/** Die Bytes einer Datei als Blob — für den Betrachter. */
export async function dateiAlsBlob(datei) {
    const bytes = alsBytes(await invoke('datei_inhalt', { id: datei.id }));
    return new Blob([bytes], { type: datei.mime || 'application/octet-stream' });
}

/**
 * Mit einem anderen Programm öffnen. Am Schreibtisch öffnet die Schale selbst;
 * auf Android kommt ein Pfad zurück, den MainActivity.kt weitergibt.
 * Gibt `false` zurück, wenn es nicht ging.
 */
export async function dateiExternOeffnen(datei) {
    const pfad = await invoke('datei_oeffnen', { id: datei.id });
    if (!pfad) return true;
    return Boolean(window.openanyOeffnen?.oeffnen(pfad, datei.mime || ''));
}

/** Den Inhalt von einem gepaarten Gerät holen, wenn er hier fehlt. */
export async function dateiHolen(datei) {
    await invoke('datei_holen', { id: datei.id });
}

/** Einen Ordner oder ein Album auf diesem Gerät behalten (an) oder nicht mehr. */
export async function behaltenSetzen(id, an) {
    await invoke('behalten_setzen', { id, an });
}
