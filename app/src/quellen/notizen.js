/*
 * Die Datenquelle der Notizen-Arbeitsfläche — die lokale SQLite statt der API.
 *
 * DIESELBE FORM WIE DIE WEBAPP-QUELLE (frontend/src/components/notes/
 * notesSources.js): gleiche Methodennamen, Antworten als `{ data: … }` mit
 * denselben Feldern. Die Arbeitsfläche weiß nicht, woher sie ihre Notizen hat,
 * und soll es auch nicht wissen. Die Befehle dahinter stehen in
 * src-tauri/src/lib.rs (`notizbuch_*`).
 *
 * WAS HIER FEHLT, FEHLT MIT ABSICHT, und die Arbeitsfläche blendet dann den
 * Knopf aus: PDF (`exportNotePdf`), ZIP (`exportNoteFolder`), Graph,
 * Bibliothek, KI. Anhänge werden gelesen (seit 16.09.2026) UND angelegt
 * (seit 17.09.2026, `notizbuch_anhang_*`): Ein Bild geht in die Galerie, ein
 * Dokument in die Akte „Notizen", und der Abgleich bringt beides hinüber.
 *
 * JEDER `invoke` STEHT AUF EINER ZEILE mit einfachen Schlüsseln: Der Test
 * src-tauri/tests/verdrahtung.rs liest genau diese Zeilen und prüft Namen und
 * Argumente gegen die Befehle.
 */
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { stueckeSchicken } from './dateien';
import { i18n } from '@oberflaeche/i18n';

const antwort = (data) => ({ data });

/*
 * Ein Fehler aus Rust ist ein blanker Text. Die Arbeitsfläche erwartet eine
 * HTTP-artige Antwort und zeigt bei 422 die Meldung im Editor an — ohne diese
 * Hülle stünde dort nur „Fehler", ohne Grund.
 */
const alsFehler = (e) => Object.assign(new Error(String(e)), {
    response: { status: 422, data: { message: String(e), errors: { content: [String(e)] } } },
});

async function rufe(befehl) {
    try {
        return await befehl();
    } catch (e) {
        throw alsFehler(e);
    }
}

export function lokaleNotizenQuelle() {
    return {
        supportsLibrary: false,
        ai: null,

        // Kurz nach dem letzten Tastendruck speichern statt nach zehn Sekunden
        // wie in der Webapp: Hier ist Speichern eine Zeile in der SQLite, kein
        // Dateischreiben auf einem Server. Mehrere Speicherungen derselben
        // Notiz fasst der Abgleich ohnehin zu einer zusammen.
        speicherTaktMs: 1500,

        // Notizen
        getNotes: (folderId, page, tag, q) => rufe(async () => antwort(
            await invoke('notizbuch_liste', { mappe: folderId ?? null, seite: page, tag: tag ?? null, suche: q || null }))),
        getNote: (id) => rufe(async () => antwort(await invoke('notizbuch_notiz', { id }))),
        createNote: (daten) => rufe(async () => antwort(
            await invoke('notizbuch_anlegen', { titel: daten.title ?? '', inhalt: daten.content ?? '', mappe: daten.note_folder_id ?? null }))),
        updateNote: (id, daten) => rufe(async () => antwort(
            await invoke('notizbuch_aendern', { id, titel: daten.title ?? null, inhalt: daten.content ?? null, mappeSetzen: 'note_folder_id' in daten, mappe: daten.note_folder_id ?? null }))),
        deleteNote: (id) => rufe(async () => antwort(await invoke('notiz_papierkorb', { zkId: id }))),

        // Mappen
        getNoteFolders: () => rufe(async () => antwort({ folders: await invoke('notizbuch_mappen') })),
        createNoteFolder: (name, parentId) => rufe(async () => antwort(
            { id: await invoke('notizbuch_mappe_anlegen', { name, eltern: parentId ?? null }) })),
        updateNoteFolder: (id, daten) => rufe(async () => antwort(
            { id: await invoke('notizbuch_mappe_umbenennen', { id, name: daten.name }) })),
        deleteNoteFolder: (id, mode) => rufe(async () => antwort(await invoke('notizbuch_mappe_loeschen', { id, modus: mode }))),

        // Titel, Tags, Rückverweise
        getNoteTitles: () => rufe(async () => antwort({ items: await invoke('notizbuch_titel') })),
        resolveTitle: (title) => rufe(async () => antwort({ note: await invoke('notizbuch_aufloesen', { ziel: title }) })),
        getNoteTags: () => rufe(async () => antwort({ items: await invoke('notizbuch_tags') })),
        getNoteBacklinks: (id) => rufe(async () => antwort({ items: await invoke('notizbuch_rueckverweise', { id }) })),
        // Der Graph (seit 30.09.2026): dieselbe Form wie drüben /notes/graph.
        getGraph: () => rufe(async () => antwort(await invoke('notizbuch_graph'))),

        /*
         * ANHÄNGE.
         *
         * Ein Anhang ist eine Zuordnung — „dieser Pfad im Text meint dieses
         * Bild". Sie kommt über den Abgleich (`Art::Notizanhang`); die Bytes
         * gehören dem Ziel und reisen über dessen eigenen Eintrag.
         *
         * Bis zum 16.09.2026 stand hier „noch keine", und Bilder in Notizen
         * blieben als roher Markdown-Text stehen. Der Abgleich zählte sie
         * derweil als „übersprungen" — eine Zahl, die niemand deuten konnte.
         * Bis zum 17.09.2026 hieß es darunter „kann dieses Programm noch
         * nicht speichern".
         *
         * `assetUrl` ist SYNCHRON (die Arbeitsfläche ruft es beim Zeichnen),
         * deshalb bringt die Liste den lokalen Pfad schon mit — und die
         * Antwort auf ein Speichern ebenso, damit das Bild sofort dasteht.
         */
        getNoteAssets: async (mappe) => antwort({ items: await invoke('notizbuch_anhaenge', { mappe: mappe ?? null }) }),
        assetUrl: (anhang) => (anhang?.pfad ? convertFileSrc(anhang.pfad) : null),

        // Zitate brauchen die Bibliothek (BibTeX) -- die gibt es hier nicht.
        getNoteCitations: async () => antwort({ items: [] }),

        // Speichern: in Stücken über dieselbe Brücke wie eine Datei
        // (Base64, siehe dateien.js). Am Ende entstehen Sache und Zuordnung.
        uploadNoteAsset: (folderId, datei) => rufe(async () => {
            const kennzeichen = await invoke('notizbuch_anhang_beginnen', { mappe: folderId ?? null, name: datei.name, mime: datei.type || '' });
            await stueckeSchicken(kennzeichen, datei);
            return antwort(await invoke('notizbuch_anhang_fertig', { kennzeichen }));
        }),

        /*
         * ÖFFNEN STATT HERUNTERLADEN. Die Webapp lädt einen Anhang als Blob
         * und stößt einen Browser-Download an; in der Webansicht auf Android
         * läuft das ins Leere. Hier legt die Schale die Datei bereit, und
         * `openanyOeffnen` (MainActivity) reicht sie an ein anderes Programm
         * weiter — wie bei Dateien. Auf dem Schreibtisch öffnet die Schale
         * selbst und gibt nichts zurück.
         */
        openNoteAsset: (anhang, folderId) => rufe(async () => {
            const offen = await invoke('notizbuch_anhang_oeffnen', { mappe: folderId ?? null, path: anhang.path });
            if (offen && !window.openanyOeffnen?.oeffnen(offen.pfad, offen.mime)) {
                throw new Error(i18n.global.t('app.fehler.anhangOeffnen'));
            }
        }),
    };
}
