/*
 * Die Datenquelle der Galerie — die lokale SQLite und die Inhaltsablage statt
 * der API. Dieselben Methodennamen wie der api-Service der Webapp, dazu die
 * Adressen der Bilder (siehe GalerieArbeitsflaeche.vue).
 *
 * Bilder kommen als Pfad in der Ablage; `convertFileSrc` macht daraus eine
 * Adresse des Asset-Protokolls (freigegeben nur für `inhalte/`).
 *
 * JEDER `invoke` AUF EINER ZEILE — verdrahtung.rs liest genau diese Zeilen.
 */
import { invoke, convertFileSrc } from '@tauri-apps/api/core';
import { originalAblegen } from '../aufsGeraet';
import { alsBytes, stueckeSchicken } from './dateien';
import { istTon, tonDauer, tonMime, WELLE_HOECHSTENS } from '@oberflaeche/galerie/ton';
import { tonAufnehmer } from './tonaufnahme';
import { istVideo, videoStandbild, videoStandbildAusAdresse } from '@oberflaeche/galerie/videoStandbild';

const adresse = (pfad) => (pfad ? convertFileSrc(pfad) : '');

/*
 * Ein Video bekommt sein Standbild in `galerie_bild_fertig` vom Betriebssystem
 * (standbild.rs, auf Android `MediaMetadataRetriever`) -- nicht aus der
 * Webansicht: Dort spielen Videos aus der Ablage nicht zuverlässig, also
 * ließe sich auch kein Bild ziehen.
 */
async function bildSpeichern(album, datei, fortschritt) {
    const kennzeichen = await invoke('datei_hochladen_beginnen', { zone: 'galerie', ordner: album, name: datei.name, mime: datei.type || tonMime(datei.name) });
    await stueckeSchicken(kennzeichen, datei, fortschritt);
    const bild = await invoke('galerie_bild_fertig', { kennzeichen });
    if (SCHREIBTISCH && istVideo(datei) && !bild.hat_vorschau) {
        await standbildSetzen(bild.id, await videoStandbild(datei).catch(() => null));
    }
    if (istTon(datei)) {
        const dauer = await tonDauer(datei);
        if (dauer) return invoke('galerie_ton_dauer', { id: bild.id, dauer });
    }
    return bild;
}

/** Bis hierher spielt der Player der App, darüber der des Systems (galerie_inhalt). */
const TON_HOECHSTENS = 64 * 1024 * 1024;

/*
 * AUF DEM SCHREIBTISCH zieht die Webansicht das Standbild
 * (docs/plan-desktop.md 1d): Dort gibt es keinen MediaMetadataRetriever, aber
 * die Webansicht spielt Videos zuverlässig -- anders als auf Android, wo
 * genau das der Grund für den Weg über das System war. Android erkennt man
 * an der Brücke der Schale.
 */
const SCHREIBTISCH = typeof window !== 'undefined' && !window.openanyAblage;

async function standbildSetzen(id, ergebnis) {
    if (!ergebnis?.bild) return false;
    const bytes = new Uint8Array(await ergebnis.bild.arrayBuffer());
    let roh = '';
    for (let i = 0; i < bytes.length; i += 0x8000) roh += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
    await invoke('galerie_standbild_setzen', { id, jpeg: btoa(roh), dauer: ergebnis.dauer ?? null });
    return true;
}

/*
 * Videos, die schon hier liegen, aber noch kein Standbild haben (vom Abgleich
 * gekommen): nacheinander nachziehen, je Sitzung höchstens einmal je Video.
 * Läuft neben der Anzeige her; das Bild erscheint beim nächsten Laden.
 */
const nachgezogen = new Set();
let warteschlange = Promise.resolve();
function standbilderNachziehen(bilder) {
    if (!SCHREIBTISCH) return;
    for (const b of bilder ?? []) {
        if (!istVideo(b) || b.hat_vorschau || !b.vorhanden || !b.pfad || nachgezogen.has(b.id)) continue;
        nachgezogen.add(b.id);
        warteschlange = warteschlange
            .then(() => videoStandbildAusAdresse(adresse(b.pfad)))
            .then((e) => standbildSetzen(b.id, e))
            .catch(() => {});
    }
}

const mitNachziehen = (liste) => { standbilderNachziehen(Array.isArray(liste) ? liste : liste?.media ?? liste?.bilder); return liste; };

export function lokaleGalerieQuelle() {
    const quelle = {
        getAlbums: async () => ({ data: await invoke('galerie_alben') }),
        getAlbum: async (id) => ({ data: mitNachziehen(await invoke('galerie_album', { id })) }),
        createAlbum: async (daten) => ({ data: await invoke('galerie_album_anlegen', { name: daten.name, beschreibung: daten.description || '', eltern: daten.parent_id ?? null }) }),
        updateAlbum: async (id, daten) => ({ data: await invoke('galerie_album_verschieben', { id, eltern: daten.parent_id ?? null }) }),
        getAlbumTree: async () => ({ data: await invoke('galerie_albenbaum') }),
        deleteAlbum: async (id) => ({ data: await invoke('galerie_album_papierkorb', { id }) }),
        kameraAlbum: (name) => invoke('galerie_kamera_album', { name }),
        uploadMedia: async (album, datei, optionen) => ({ data: await bildSpeichern(album, datei, optionen?.fortschritt) }),
        deleteMedia: async (_album, id) => ({ data: await invoke('galerie_bild_papierkorb', { id }) }),
        moveMedia: async (id, album) => ({ data: await invoke('galerie_bild_verschieben', { id, album }) }),
        getGalleryMedia: async () => ({ data: mitNachziehen(await invoke('galerie_bilder')) }),
        uploadGalleryMedia: async (datei, optionen) => ({ data: await bildSpeichern(null, datei, optionen?.fortschritt) }),
        deleteGalleryMedia: async (id) => ({ data: await invoke('galerie_bild_papierkorb', { id }) }),

        bildUrl: (bild) => adresse(bild.pfad || bild.vorschau_pfad),
        // Ohne Vorschau fällt ein BILD aufs Original zurück, ein Video nicht:
        // Die Kachel soll ihr Film-Symbol zeigen, statt ein Video in ein <img>
        // zu laden (siehe bild_anzeige in galeriebefehle.rs).
        vorschauUrl: (bild) => adresse(bild.vorschau_pfad || (istVideo(bild) ? '' : bild.pfad)),
        titelbildSetzen: async (album, datei) => {
            const bild = await bildSpeichern(album, datei);
            await invoke('galerie_titelbild_setzen', { album, id: bild.id });
        },
        titelbildEntfernen: (album) => invoke('galerie_titelbild_entfernen', { album }),
        albumTitelbildUrl: (album) => (album.titelbild_pfad ? adresse(album.titelbild_pfad) : null),

        /*
         * VIDEOS SPIELT DER PLAYER DES SYSTEMS (galerie_oeffnen). Liegt der
         * Inhalt noch nicht hier, erst holen. Auf Android kommt ein Pfad
         * zurück, den MainActivity.kt über den FileProvider weitergibt.
         */
        videoOeffnen: async (bild) => {
            if (!bild.vorhanden) {
                bild.pfad = await invoke('galerie_bild_holen', { id: bild.id });
                bild.vorhanden = true;
            }
            const pfad = await invoke('galerie_oeffnen', { id: bild.id });
            if (pfad) window.openanyOeffnen?.oeffnen(pfad, bild.mime_type || 'video/*');
        },

        /*
         * TON SPIELT DIE APP SELBST. Auf Android als Blob, weil Medien aus der
         * Ablage dort nicht zuverlässig spielen (siehe oben); am Schreibtisch
         * direkt. Der Blob dient auch der Wellenform. `null` heißt: Die Datei
         * ist zu groß und läuft schon im Player des Systems.
         */
        async tonLaden(bild) {
            await this.bildVorbereiten(bild);
            if (SCHREIBTISCH) {
                const url = adresse(bild.pfad);
                const blob = bild.size <= WELLE_HOECHSTENS ? await fetch(url).then((a) => a.blob()).catch(() => null) : null;
                return { url, blob };
            }
            if (bild.size > TON_HOECHSTENS) {
                await this.videoOeffnen(bild);
                return null;
            }
            const bytes = alsBytes(await invoke('galerie_inhalt', { id: bild.id }));
            const blob = new Blob([bytes], { type: bild.mime_type || 'audio/mp4' });
            return { url: URL.createObjectURL(blob), blob };
        },

        /** Das Original aufs Gerät: auf dem Handy nach „Download", am Schreibtisch mit Dialog. */
        bildHerunterladen: (bild) => originalAblegen(() => invoke('galerie_bild_aufs_geraet', { id: bild.id }), bild.name, bild.mime_type),

        /** Liegt nur die Vorschau hier, das Original von einem anderen Gerät holen. */
        bildVorbereiten: async (bild) => {
            if (bild.vorhanden) return;
            bild.pfad = await invoke('galerie_bild_holen', { id: bild.id });
            bild.vorhanden = true;
        },
    };
    quelle.tonAufnehmer = tonAufnehmer(quelle);
    return quelle;
}
