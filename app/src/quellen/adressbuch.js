/*
 * Die Datenquelle der Adressbuch-Arbeitsfläche — die lokale SQLite statt der API.
 *
 * Dieselben Methodennamen wie der api-Service der Webapp (getContacts,
 * createContact …), Antworten als `{ data: … }` mit denselben Feldern. Die
 * Befehle dahinter: `adressbuch_*` in src-tauri/src/lib.rs.
 *
 * Fotos kommen als fertige `photo.url` (data:) — es gibt keinen Server, der
 * sie unter einer Adresse ausliefert. Deshalb fehlt `contactPhotoUrl`.
 *
 * JEDER `invoke` AUF EINER ZEILE mit einfachen Schlüsseln — verdrahtung.rs
 * liest genau diese Zeilen.
 */
import { invoke } from '@tauri-apps/api/core';

const antwort = (data) => ({ data });

/*
 * DAS BILD WIRD HIER VERKLEINERT, bevor es gespeichert wird.
 *
 * Ein Foto aus der Kamera hat vier bis acht Megabyte. Es ginge sonst so in die
 * Datei, beim Abgleich als Base64 über die Leitung, und der Server verkleinert
 * es dort ohnehin. 512 Pixel an der längeren Seite reichen für einen Kreis von
 * 64 Pixeln auch auf einem hochauflösenden Bildschirm.
 */
const KANTE = 512;

async function verkleinert(datei) {
    const bild = await createImageBitmap(datei);
    const faktor = Math.min(1, KANTE / Math.max(bild.width, bild.height));
    const leinwand = document.createElement('canvas');
    leinwand.width = Math.round(bild.width * faktor);
    leinwand.height = Math.round(bild.height * faktor);
    leinwand.getContext('2d').drawImage(bild, 0, 0, leinwand.width, leinwand.height);

    const blob = await new Promise((fertig) => leinwand.toBlob(fertig, 'image/jpeg', 0.85));
    const bytes = new Uint8Array(await blob.arrayBuffer());
    let roh = '';
    for (let i = 0; i < bytes.length; i += 0x8000) roh += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
    return btoa(roh);
}

export function lokaleAdressbuchQuelle() {
    return {
        getContacts: async () => antwort(await invoke('adressbuch_kontakte')),
        createContact: async (daten) => antwort(await invoke('adressbuch_speichern', { id: null, name: daten.display_name, wege: daten.channels })),
        updateContact: async (id, daten) => antwort(await invoke('adressbuch_speichern', { id, name: daten.display_name, wege: daten.channels })),
        deleteContact: async (id) => antwort(await invoke('adressbuch_loeschen', { id })),
        uploadContactPhoto: async (id, datei) => {
            const bild = await verkleinert(datei);
            return antwort(await invoke('adressbuch_foto_setzen', { id, bild }));
        },
        deleteContactPhoto: async (id) => antwort(await invoke('adressbuch_foto_entfernen', { id })),
    };
}
