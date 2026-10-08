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
 * „Download" (MainActivity.kt); sonst als gewöhnlicher Download des
 * Browsers. Gibt zurück, wo sie liegt, oder wirft den Grund.
 */
export function aufsGeraet(name, mime, text) {
    if (window.openanyAblage) {
        const fehler = window.openanyAblage.inDownloads(name, mime, text);
        if (fehler) throw new Error(fehler);
        return `Download/${name}`;
    }
    const url = URL.createObjectURL(new Blob([text], { type: mime }));
    const a = document.createElement('a');
    a.href = url;
    a.download = name;
    a.click();
    setTimeout(() => URL.revokeObjectURL(url), 10000);
    return name;
}
