/*
 * Die Datenquelle der Kalender-Arbeitsfläche — die lokale SQLite statt der API.
 *
 * Dieselben Methodennamen wie der api-Service der Webapp (getCalendars,
 * getEvents, createEvent …), Antworten als `{ data: … }` mit denselben
 * Feldern. Die Befehle dahinter: `kalenderbuch_*` in src-tauri/src/lib.rs.
 *
 * `syncCalendar` HOLT HIER SELBST. Drüben fragt die Webapp den Server, der
 * den Feed abruft; hier ruft das Gerät ihn ab. Für alles darüber — den
 * Knopf, das Adressfeld, den Schreibschutz auf Abo-Kalendern — ist das
 * dasselbe: Die Arbeitsfläche prüft nur, OB es die Methode gibt.
 *
 * Das ist der Grund für die ganze Naht. Wer einen Kalender von FamilyWall
 * eingebunden hat und kein openany-Konto besitzt, sähe den Termin, den dort
 * jemand einträgt, sonst nie.
 *
 * NICHT DA, mit Absicht — die Arbeitsfläche blendet die Knöpfe aus:
 * importICS, exportICS (Datei-Dialoge), getCalendarOverlay und
 * getCalendarDay (kommen aus Projekten auf dem Server).
 *
 * JEDER `invoke` AUF EINER ZEILE mit einfachen Schlüsseln — verdrahtung.rs
 * liest genau diese Zeilen.
 */
import { invoke } from '@tauri-apps/api/core';

const antwort = (data) => ({ data });

function terminDaten(payload) {
    return {
        kalender: payload.calendar_id,
        titel: payload.title ?? '',
        beschreibung: payload.description ?? null,
        ort: payload.location ?? null,
        beginn: payload.start_date,
        ende: payload.end_date,
        ganztags: Boolean(payload.is_all_day),
        rrule: payload.rrule ?? 'NONE',
    };
}

/*
 * Die Abo-Adresse beim Ändern — und der Unterschied, der hier zählt.
 *
 * `null` heißt für die Schale „unverändert", der leere Text heißt „Abo
 * aufgeben". Wer beides gleich behandelte, verlöre bei jedem Farbwechsel das
 * Abo: Die Seitenleiste schickt zum Umfärben nur `{ color }`.
 */
function aboAus(daten) {
    if (!('sync_url' in daten)) return null;
    return daten.sync_url ?? '';
}

export function lokaleKalenderQuelle() {
    return {
        getCalendars: async () => antwort(await invoke('kalenderbuch_kalender')),
        createCalendar: async (form) => antwort(await invoke('kalenderbuch_kalender_anlegen', { name: form.name, farbe: form.color ?? null, aboUrl: form.sync_url ?? null })),
        updateCalendar: async (id, daten) => antwort(await invoke('kalenderbuch_kalender_aendern', { id, name: daten.name ?? null, farbe: daten.color ?? null, aboUrl: aboAus(daten) })),
        deleteCalendar: async (id) => antwort(await invoke('kalenderbuch_kalender_loeschen', { id })),
        syncCalendar: async (id) => antwort(await invoke('kalenderbuch_abo_auffrischen', { id })),

        getEvents: async (from, to) => antwort(await invoke('kalenderbuch_termine', { von: from, bis: to })),
        createEvent: async (payload) => {
            const d = terminDaten(payload);
            return antwort(await invoke('kalenderbuch_termin_speichern', { id: null, kalender: d.kalender, titel: d.titel, beschreibung: d.beschreibung, ort: d.ort, beginn: d.beginn, ende: d.ende, ganztags: d.ganztags, rrule: d.rrule }));
        },
        updateEvent: async (id, payload) => {
            const d = terminDaten(payload);
            return antwort(await invoke('kalenderbuch_termin_speichern', { id, kalender: d.kalender, titel: d.titel, beschreibung: d.beschreibung, ort: d.ort, beginn: d.beginn, ende: d.ende, ganztags: d.ganztags, rrule: d.rrule }));
        },
        deleteEvent: async (id) => antwort(await invoke('termin_papierkorb', { uuid: id })),
    };
}
