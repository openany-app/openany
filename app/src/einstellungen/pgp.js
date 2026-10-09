import { invoke } from '@tauri-apps/api/core';

/**
 * Ein Fingerabdruck zum Vergleichen: in Vierergruppen, mit einem größeren
 * Abstand in der Mitte -- so, wie Thunderbird und GnuPG ihn zeigen.
 */
export function fingerabdruckLesbar(f) {
    const gruppen = String(f ?? '').toUpperCase().match(/.{1,4}/g) ?? [];
    const haelfte = Math.ceil(gruppen.length / 2);
    return `${gruppen.slice(0, haelfte).join(' ')}  ${gruppen.slice(haelfte).join(' ')}`.trim();
}

/**
 * Text als Datei auf dem Gerät ablegen, AUSSERHALB von „Dateien" (die gehen
 * in den Abgleich). Auf Android über `openanyAblage` in den Ordner
 * „Download" (MainActivity.kt); auf dem Schreibtisch über den
 * Speichern-Dialog (`aufs_geraet`, ablagebefehle.rs). Gibt zurück, wo sie
 * liegt, `null` bei Abbruch, oder wirft den Grund.
 */
export async function aufsGeraet(name, mime, text) {
    if (window.openanyAblage) {
        const fehler = window.openanyAblage.inDownloads(name, mime, text);
        if (fehler) throw new Error(fehler);
        return `Download/${name}`;
    }
    const bytes = new TextEncoder().encode(text);
    let roh = '';
    for (const b of bytes) roh += String.fromCharCode(b);
    return invoke('aufs_geraet', { name, daten: btoa(roh) });
}
