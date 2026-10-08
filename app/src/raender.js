/*
 * Die Ränder des Bildschirms: Statusleiste oben, Gestenleiste unten.
 *
 * WARUM NICHT EINFACH `env(safe-area-inset-*)`: Auf Android meldet die
 * Webansicht diese Werte nicht verlässlich — unten kam am 06.09.2026 auf dem
 * Tablet 0 an, oben lag am 15.09.2026 der Inhalt unter der Statusleiste.
 * `MainActivity.kt` liest die echten Werte bei Android ab und stellt sie als
 * `window.openanyRaender` bereit.
 *
 * Auf dem Schreibtisch gibt es dieses Objekt nicht; dann bleiben die
 * Vorgaben aus `@oberflaeche/stil.css` stehen, und die greifen auf `env()` zurück.
 */

const SEITEN = ['oben', 'unten', 'links', 'rechts', 'tastatur'];

function anwenden() {
    const r = window.openanyRaender;

    if (!r) return;

    for (const seite of SEITEN) {
        document.documentElement.style.setProperty(`--rand-${seite}`, `${r[seite]()}px`);
    }
}

export function raenderBeobachten() {
    anwenden();
    // Android meldet neue Ränder (Drehen, Wechsel der Gestensteuerung) über
    // dieses Ereignis; `resize` fängt den Fall ab, dass es vor dem Laden kam.
    window.addEventListener('openany-raender', anwenden);
    window.addEventListener('resize', anwenden);
}
