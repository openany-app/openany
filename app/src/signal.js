/*
 * NACHRICHTEN-SIGNAL: ein kurzer Ton und, unter Android, ein Vibrieren,
 * wenn der Ungelesen-Zähler steigt, solange die App offen ist (App.vue).
 * Bei geschlossener App meldet sich statt dessen die Mitteilung des Systems
 * (wachdienst.rs, `fenster_sichtbar`).
 *
 * Die Schalter gelten je Gerät und liegen deshalb im Browser-Speicher.
 */
import { ref, watch } from 'vue';

const SCHLUESSEL = 'openany.signal';

function lesen() {
    try {
        return JSON.parse(localStorage.getItem(SCHLUESSEL)) ?? {};
    } catch {
        return {};
    }
}

const gespeichert = lesen();
export const tonAn = ref(gespeichert.ton ?? true);
export const vibrierenAn = ref(gespeichert.vibrieren ?? false);
export const kannVibrieren = Boolean(window.openanyWachdienst?.vibrieren);

watch([tonAn, vibrierenAn], () => {
    try {
        localStorage.setItem(SCHLUESSEL, JSON.stringify({ ton: tonAn.value, vibrieren: vibrierenAn.value }));
    } catch {
        // dann eben nur bis zum nächsten Start
    }
});

let klang = null;
// Webansichten lassen Ton erst nach einer Berührung zu: Beim ersten Tippen
// wird der Klang schon bereitgestellt.
document.addEventListener('pointerdown', () => {
    try {
        klang ??= new AudioContext();
        klang.resume();
    } catch {
        // ohne Ton
    }
}, { once: true });
function tonSpielen() {
    try {
        klang ??= new AudioContext();
        if (klang.state === 'suspended') klang.resume();
        // Zwei kurze, leise Töne aufwärts.
        [[880, 0], [1320, 0.12]].forEach(([hz, ab]) => {
            const osz = klang.createOscillator();
            const laut = klang.createGain();
            const t0 = klang.currentTime + ab;
            osz.type = 'sine';
            osz.frequency.value = hz;
            laut.gain.setValueAtTime(0, t0);
            laut.gain.linearRampToValueAtTime(0.15, t0 + 0.01);
            laut.gain.exponentialRampToValueAtTime(0.001, t0 + 0.25);
            osz.connect(laut).connect(klang.destination);
            osz.start(t0);
            osz.stop(t0 + 0.26);
        });
    } catch {
        // ohne Ton
    }
}

// Mehrere Nachrichten kurz hintereinander geben ein Signal.
let zuletzt = 0;
export function signalGeben() {
    const jetzt = Date.now();
    if (jetzt - zuletzt < 2000) return;
    zuletzt = jetzt;
    if (tonAn.value) tonSpielen();
    if (vibrierenAn.value && kannVibrieren) {
        try {
            window.openanyWachdienst.vibrieren();
        } catch {
            // ohne Vibrieren
        }
    }
}
