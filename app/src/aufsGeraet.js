/*
 * Downloads der Oberfläche aufs Gerät (15.10.2026).
 *
 * Die Arbeitsflächen aus dem Paket rufen `downloadBlob` (Notiz als PDF oder
 * Markdown, Druckfassung). In der Webansicht führt der Browser-Download ins
 * Leere; hier geht die Datei denselben Weg wie ein Nachrichten-Anhang
 * (quellen/nachrichten.js): auf Android nach „Download" (MainActivity.kt),
 * auf dem Schreibtisch in einen Speichern-Dialog (ablagebefehle.rs).
 */
import { invoke } from '@tauri-apps/api/core';
import { setBlobAblage } from '@oberflaeche/shared/download';
import { useToast } from '@oberflaeche/composables/useToast';
import { i18n } from '@oberflaeche/i18n';

/** Blob -> Base64 ohne „data:…;base64,"-Vorspann. */
function alsBase64(blob) {
    return new Promise((fertig, fehler) => {
        const leser = new FileReader();
        leser.onload = () => fertig(String(leser.result).replace(/^data:[^,]*,/, ''));
        leser.onerror = () => fehler(leser.error);
        leser.readAsDataURL(blob);
    });
}

async function ablegen(blob, name) {
    const b64 = await alsBase64(blob);
    let ort;
    if (window.openanyAblage?.inDownloadsBase64) {
        const fehler = window.openanyAblage.inDownloadsBase64(name, blob.type || 'application/octet-stream', b64);
        if (fehler) throw new Error(fehler);
        ort = `Download/${name}`;
    } else {
        // Schreibtisch: `null` = abgebrochen, dann keine Meldung.
        ort = await invoke('aufs_geraet', { name, daten: b64 });
        if (!ort) return null;
    }
    useToast().success(i18n.global.t('app.download.gespeichert', { ort }));
    return ort;
}

export function downloadsAufsGeraet() {
    setBlobAblage(ablegen);
}
