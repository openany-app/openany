/*
 * Geräte in der Nähe — ein Zustand für das ganze Programm.
 *
 * Bis zum 15.09.2026 stand das alles in der Einstellungskachel: Nur wer dort
 * war, sah eine Paarungsanfrage, und nur dort ließ sich abgleichen. Jetzt
 * fragt die Schale alle 3 s nach (`nah_lage`), zeigt Anfragen überall und
 * trägt „Abgleichen" in der Kopfzeile bzw. im Menü.
 *
 * `stand` wächst, wenn sich hier durch einen Abgleich etwas geändert hat —
 * ob dieses Gerät getippt hat oder das andere. Die Schale baut dann die
 * offene Ansicht neu auf, damit sie liest, was angekommen ist.
 */
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const nah = ref(null);
const fehler = ref('');
const laeuft = ref('');
const berichte = ref({});
const stand = ref(0);

let takt = null;
let angenommenZuletzt = null;

async function lesen() {
    try {
        nah.value = await invoke('nah_lage');
    } catch (e) {
        return; // bleibt beim letzten Stand
    }
    const a = nah.value?.angenommen ?? 0;
    if (angenommenZuletzt !== null && a > angenommenZuletzt) stand.value++;
    angenommenZuletzt = a;
}

function starten() {
    if (takt) return;
    lesen();
    takt = setInterval(lesen, 3000);
}

async function suchen() {
    await invoke('nah_suchen');
    setTimeout(lesen, 1500);
}

// Jeder Befehl mit festem Namen: verdrahtung.rs liest die Namen aus dem Text,
// ein `invoke(befehl, …)` mit Variable sähe er nicht (am 15.09.2026 so
// aufgefallen).
async function aufruf(tun) {
    fehler.value = '';
    try {
        await tun();
    } catch (e) {
        fehler.value = String(e);
    }
    await lesen();
}
const paaren = (fingerabdruck) => aufruf(() => invoke('nah_paaren', { fingerabdruck }));
const passt = (fingerabdruck) => aufruf(() => invoke('nah_bestaetigen', { fingerabdruck }));
const abbrechen = (fingerabdruck) => aufruf(() => invoke('nah_abbrechen', { fingerabdruck }));
const vergessen = (fingerabdruck) => aufruf(() => invoke('nah_vergessen', { fingerabdruck }));

/*
 * EINLADUNG VOR ORT in ein lokales Projekt (01.10.2026). Läuft wie das
 * Paaren — Code auf beiden Geräten, beide bestätigen —, sieht aber nie so aus
 * (EinladungsDialog.vue). Den Abschluss übernimmt `nah_lage`, sobald das
 * andere Gerät bestätigt hat.
 */
async function einladen(projekt, fingerabdruck) {
    fehler.value = '';
    try {
        await invoke('nah_einladen', { projekt, fingerabdruck });
    } finally {
        await lesen();
    }
}
const einladungPasst = (fingerabdruck) => aufruf(() => invoke('nah_einladung_bestaetigen', { fingerabdruck }));
const einladungAbbrechen = (fingerabdruck) => aufruf(() => invoke('nah_einladung_abbrechen', { fingerabdruck }));

/** Mit einem gepaarten Gerät abgleichen. Gibt den Bericht zurück (oder wirft). */
async function abgleichen(fingerabdruck) {
    laeuft.value = fingerabdruck;
    try {
        const bericht = await invoke('nah_abgleichen', { fingerabdruck });
        berichte.value = { ...berichte.value, [fingerabdruck]: bericht };
        if (bericht.gezogen || bericht.konflikte) stand.value++;
        return bericht;
    } finally {
        laeuft.value = '';
        await lesen();
    }
}

/**
 * Gepaarte Geräte, mit denen ein Abgleich versucht wird — ALLE, nicht nur die
 * gerade als „in der Nähe" gelisteten. Die Liste hinkt hinterher (am
 * 15.09.2026: Telefon offen im selben Hotspot, aber seit zwei Minuten nicht
 * nachgesucht); ob ein Gerät erreichbar ist, sagt der Versuch zuverlässiger.
 */
const erreichbar = computed(() => nah.value?.gepaarte ?? []);

/**
 * Mit allen gepaarten Geräten nacheinander abgleichen.
 * Gibt je Gerät { name, bericht | fehler } zurück.
 */
async function abgleichenAlle() {
    const ergebnisse = [];
    for (const g of erreichbar.value) {
        try {
            ergebnisse.push({ name: g.name, bericht: await abgleichen(g.fingerabdruck) });
        } catch (e) {
            ergebnisse.push({ name: g.name, fehler: String(e) });
        }
    }
    return ergebnisse;
}

export function useNahAbgleich() {
    return {
        nah, fehler, laeuft, berichte, stand, erreichbar,
        starten, lesen, suchen, paaren, passt, abbrechen, vergessen, abgleichen, abgleichenAlle,
        // Immer nur EINE Anfrage auf einmal im Dialog.
        offeneAnfrage: computed(() => nah.value?.offen?.[0] ?? null),
        offenMit: (fp) => nah.value?.offen?.find((o) => o.fingerabdruck === fp),
        einladen, einladungPasst, einladungAbbrechen,
        offeneEinladung: computed(() => nah.value?.einladungen?.[0] ?? null),
    };
}
