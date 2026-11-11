/*
 * Viele Dateien auf einmal übernehmen (docs/plan-viele-dateien.md, D2) — die
 * Schlange läuft in Rust (uebernahme.rs), hier nur Auftrag und Stand.
 *
 * Gewählt wird im Dialog des Systems, nicht in der Webansicht: Rust liest die
 * Dateien direkt von der Platte, auch ein Video mit Gigabytes. Wählen und
 * Ablegen deshalb nur auf dem Schreibtisch; unter Android bleibt dafür das
 * Auswahlfenster der Oberfläche, und Fotos des Geräts kommen über
 * `fotosVomGeraetQuelle`.
 *
 * Ändert sich der Stand, meldet die Schale `openany-uebernahme` am Fenster;
 * beim Ziehen von Dateien ins Fenster `openany-ziehen` mit der Stelle
 * (ablegen.rs). Die Pfade bleiben in Rust — `ablegen` nennt nur das Ziel.
 *
 * JEDER `invoke` AUF EINER ZEILE — verdrahtung.rs liest genau diese Zeilen.
 */
import { invoke } from '@tauri-apps/api/core';

const SCHREIBTISCH = typeof window !== 'undefined' && !window.openanyAblage;

export function uebernahmeQuelle() {
    const quelle = {
        stand: () => invoke('uebernahme_stand'),
        pausieren: (pausiert) => invoke('uebernahme_pausieren', { pausiert }),
        nichtUebernommen: () => invoke('uebernahme_nicht_uebernommen'),
        abschliessen: () => invoke('uebernahme_abschliessen'),
        abbrechen: () => invoke('uebernahme_abbrechen'),
        /** „Liste der Übernommenen vergessen" (Entscheidung 9). */
        vergessen: () => invoke('uebernahme_vergessen'),
        beobachten: (rueckruf) => {
            window.addEventListener('openany-uebernahme', rueckruf);
            return () => window.removeEventListener('openany-uebernahme', rueckruf);
        },
    };
    if (!SCHREIBTISCH) return quelle;
    return {
        ...quelle,
        /** Dateien wählen und einreihen → `{ neu, bekannt }`, `null` bei Abbruch. */
        waehlen: (zone, ziel) => invoke('uebernahme_waehlen', { zone, ziel: ziel ?? null }),
        ablegen: (zone, ziel) => invoke('uebernahme_ablegen', { zone, ziel: ziel ?? null }),
        beimZiehen: (rueckruf) => {
            const hoeren = (e) => rueckruf(e.detail);
            window.addEventListener('openany-ziehen', hoeren);
            return () => window.removeEventListener('openany-ziehen', hoeren);
        },
    };
}

/*
 * Fotos vom Gerät übernehmen (docs/plan-viele-dateien.md, D3) — nur unter
 * Android. Die Rechte erfragt die Schale (`openanyMedien`, MainActivity.kt),
 * was es gibt und was übernommen wird, weiß Rust (medien.rs). Die Seite nennt
 * nur die Auswahl, nie eine Adresse.
 */
export function fotosVomGeraetQuelle() {
    const bruecke = typeof window !== 'undefined' ? window.openanyMedien : null;
    if (!bruecke) return null;
    return {
        /** `alle`, `auswahl` oder `keine`. */
        erlaubt: () => bruecke.stand(),
        erlauben: () => bruecke.erlauben(),
        beimErlauben: (rueckruf) => {
            const hoeren = (e) => rueckruf(e.detail);
            window.addEventListener('openany-medien', hoeren);
            return () => window.removeEventListener('openany-medien', hoeren);
        },
        uebersicht: () => invoke('medien_uebersicht'),
        /** `{ alben: [Name, …] (leer = alle), ab: ms | null }` */
        uebernehmen: (auswahl) => invoke('medien_uebernehmen', { auswahl }),
        /** Neue Fotos automatisch (D5, fotowaechter.rs). */
        automatischStand: () => invoke('fotos_automatisch_stand'),
        automatischSetzen: (w) => invoke('fotos_automatisch_setzen', { an: w.an, alben: w.alben, nurWlan: w.nurWlan, nurLaden: w.nurLaden, ohneKopie: w.ohneKopie }),
    };
}

/*
 * openany als Teilen-Ziel (Skizze C) — nur unter Android. Was geteilt wurde,
 * hat die Schale schon kopiert (MainActivity.kt); Rust weiß, wo (teilen.rs).
 * Die Seite fragt nur, ob etwas bereitliegt, und nennt das Album.
 */
export function geteiltQuelle() {
    const bruecke = typeof window !== 'undefined' ? window.openanyTeilen : null;
    if (!bruecke) return null;
    return {
        /** `{ medien, dokumente, sonst }` */
        stand: () => invoke('geteilt_stand'),
        uebernehmen: (album, neuesAlbum) => invoke('geteilt_uebernehmen', { album: album ?? null, neuesAlbum: neuesAlbum ?? null }),
        verwerfen: () => invoke('geteilt_verwerfen'),
        /** Zurück zu der App, aus der geteilt wurde. */
        fertig: () => bruecke.fertig(),
        beobachten: (rueckruf) => {
            window.addEventListener('openany-teilen', rueckruf);
            return () => window.removeEventListener('openany-teilen', rueckruf);
        },
    };
}
