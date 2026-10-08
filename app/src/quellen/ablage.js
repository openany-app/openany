/*
 * Etwas von außen in den eigenen Speicher legen -- einsortiert (Tiffy,
 * 30.09.2026): Bilder und Videos in ein Galerie-Album, PDFs und Office-
 * Dateien in eine Akte unter Dokumente, alles andere in einen Ordner unter
 * Dateien. Album und Ordner heißen wie `behaelter` (etwa „Anhänge" oder „Aus
 * Projekten") und entstehen beim ersten Mal.
 *
 * Was hier landet, geht in den Abgleich. Ob das gewollt ist (etwa bei etwas
 * Verschlüsseltem), fragt der Aufrufer vorher.
 */
import { invoke } from '@tauri-apps/api/core';
import { lokaleDateienQuelle } from './dateien';
import { lokaleGalerieQuelle } from './galerie';

const DOKUMENT_ENDUNGEN = ['pdf', 'doc', 'docx', 'xls', 'xlsx', 'ppt', 'pptx', 'odt', 'ods', 'odp', 'rtf'];

export function zielFuer(mime, name) {
    const m = String(mime || '').toLowerCase();
    if (m.startsWith('image/') || m.startsWith('video/')) return 'galerie';
    const endung = String(name || '').toLowerCase().split('.').pop();
    if (m === 'application/pdf' || m.includes('msword') || m.includes('officedocument')
        || m.includes('opendocument') || m.includes('ms-excel') || m.includes('ms-powerpoint')
        || m === 'application/rtf' || DOKUMENT_ENDUNGEN.includes(endung)) return 'documents';
    return 'files';
}

async function ordner(zone, name) {
    const { files } = await invoke('dateien_liste', { zone, ordner: null });
    const da = files.find((d) => d.type === 'folder' && d.name === name);
    if (da) return da.id;
    return (await invoke('datei_ordner_anlegen', { zone, ordner: null, name })).id;
}

async function album(name) {
    const { items } = await invoke('galerie_alben');
    const da = items.find((a) => a.name === name);
    if (da) return da.id;
    return (await invoke('galerie_album_anlegen', { name, beschreibung: '', eltern: null })).id;
}

/** Ablegen und einsortieren. Gibt zurück, wo es jetzt liegt. */
export async function einsortieren(datei, behaelter) {
    const ziel = zielFuer(datei.type, datei.name);
    if (ziel === 'galerie') {
        await lokaleGalerieQuelle().uploadMedia(await album(behaelter), datei);
        return `Galerie › ${behaelter}`;
    }
    await lokaleDateienQuelle(null).uploadFile(await ordner(ziel, behaelter), datei, ziel);
    return `${ziel === 'documents' ? 'Dokumente' : 'Dateien'} › ${behaelter}`;
}

/** Eine Endung zu einem MIME-Typ, wo der Server keinen nennt. */
export function mimeAus(name) {
    const endung = String(name || '').toLowerCase().split('.').pop();
    return {
        pdf: 'application/pdf', jpg: 'image/jpeg', jpeg: 'image/jpeg', png: 'image/png', gif: 'image/gif',
        webp: 'image/webp', heic: 'image/heic', mp4: 'video/mp4', mov: 'video/quicktime', txt: 'text/plain',
        md: 'text/markdown', doc: 'application/msword', odt: 'application/vnd.oasis.opendocument.text',
        docx: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
        xlsx: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
    }[endung] ?? 'application/octet-stream';
}
