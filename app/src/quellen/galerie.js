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
import { stueckeSchicken } from './dateien';
import { istVideo } from '@oberflaeche/galerie/videoStandbild';

const adresse = (pfad) => (pfad ? convertFileSrc(pfad) : '');

/*
 * Ein Video bekommt sein Standbild in `galerie_bild_fertig` vom Betriebssystem
 * (standbild.rs, auf Android `MediaMetadataRetriever`) -- nicht mehr aus der
 * Webansicht. Seit 27.09.2026: In der Webansicht spielten Videos aus der
 * Ablage nicht zuverlässig, also ließ sich dort auch kein Bild ziehen.
 */
async function bildSpeichern(album, datei, fortschritt) {
    const kennzeichen = await invoke('datei_hochladen_beginnen', { zone: 'galerie', ordner: album, name: datei.name, mime: datei.type || '' });
    await stueckeSchicken(kennzeichen, datei, fortschritt);
    return await invoke('galerie_bild_fertig', { kennzeichen });
}

export function lokaleGalerieQuelle() {
    return {
        getAlbums: async () => ({ data: await invoke('galerie_alben') }),
        getAlbum: async (id) => ({ data: await invoke('galerie_album', { id }) }),
        createAlbum: async (daten) => ({ data: await invoke('galerie_album_anlegen', { name: daten.name, beschreibung: daten.description || '', eltern: daten.parent_id ?? null }) }),
        updateAlbum: async (id, daten) => ({ data: await invoke('galerie_album_verschieben', { id, eltern: daten.parent_id ?? null }) }),
        getAlbumTree: async () => ({ data: await invoke('galerie_albenbaum') }),
        deleteAlbum: async (id) => ({ data: await invoke('galerie_album_papierkorb', { id }) }),
        uploadMedia: async (album, datei, optionen) => ({ data: await bildSpeichern(album, datei, optionen?.fortschritt) }),
        deleteMedia: async (_album, id) => ({ data: await invoke('galerie_bild_papierkorb', { id }) }),
        getGalleryMedia: async () => ({ data: await invoke('galerie_bilder') }),
        uploadGalleryMedia: async (datei, optionen) => ({ data: await bildSpeichern(null, datei, optionen?.fortschritt) }),
        deleteGalleryMedia: async (id) => ({ data: await invoke('galerie_bild_papierkorb', { id }) }),

        bildUrl: (bild) => adresse(bild.pfad || bild.vorschau_pfad),
        // Ohne Vorschau fällt ein BILD aufs Original zurück, ein Video nicht:
        // Die Kachel soll ihr Film-Symbol zeigen, statt ein Video in ein <img>
        // zu laden (siehe bild_anzeige in galeriebefehle.rs).
        vorschauUrl: (bild) => adresse(bild.vorschau_pfad || (istVideo(bild) ? '' : bild.pfad)),
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

        /** Liegt nur die Vorschau hier, das Original von einem anderen Gerät holen. */
        bildVorbereiten: async (bild) => {
            if (bild.vorhanden) return;
            bild.pfad = await invoke('galerie_bild_holen', { id: bild.id });
            bild.vorhanden = true;
        },
    };
}
