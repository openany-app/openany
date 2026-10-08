/*
 * Die Umgebung der Projekt-Ansichten (seit 01.10.2026 im Paket).
 *
 * WARUM ES DIESE DATEI GIBT. Die Projekt-Ansichten kamen aus der Webapp und
 * riefen dort `api` direkt auf -- über hundert verschiedene Methoden. Statt
 * daraus ein neues Interface zu schneiden, bekommen sie ein Objekt mit
 * DENSELBEN Methodennamen: Die Webapp setzt ihr echtes `api` ein, die App
 * ihr eigenes (lokaler Speicher, Abgleich, vor Ort). So bleibt die Webapp
 * Zeile für Zeile, wie sie war.
 *
 * Gesetzt wird sie EINMAL beim Start des Rahmens (`projektUmgebungSetzen`).
 * Beide Rahmen sind eigene Bündel; ein Modulzustand reicht.
 *
 * Was hier hindurchgeht:
 *   api                      die Server-Aufrufe (bzw. ihr Ersatz in der App)
 *   initEcho / onEchoReconnect / leaveEchoChannel
 *                            der Live-Kanal; ohne ihn (App) gibt initEcho null
 *   KiFrage, textAuslesen, istTextAuszugFehler
 *                            die KI-Bausteine der Webapp (fehlen = kein KI-Knopf)
 *   createProjectNotesSource, NOTIZ_ERWEITERUNGEN
 *                            die Notizen hinter einer Projekt-Freigabe
 */
import { defineComponent, h } from 'vue';

let umgebung = null;

export function projektUmgebungSetzen(neu) {
  umgebung = neu;
}

function von(name) {
  if (!umgebung) throw new Error(`Projekt-Umgebung nicht gesetzt (${name}).`);
  return umgebung[name];
}

/** Die Server-Aufrufe -- erst beim Aufruf nachgeschlagen. */
export const api = new Proxy({}, {
  get: (_, methode) => von('api')[methode],
});

/*
 * WAS DIESER RAHMEN KANN (local-first, 02.10.2026): Ein Knopf, der nur
 * „geht nur in der Webapp" meldet, gehört nicht auf den Schirm. Die App nennt
 * in `kann` die Fähigkeiten, die ihr fehlen; was nicht genannt ist, gilt.
 * Die Webapp nennt nichts -- bei ihr bleibt alles, wie es war.
 *
 *   abstimmungenVerwalten  anlegen, schließen, kopieren, löschen, Einträge
 *   schulPaket             „Preset: Schule"
 *   planAbo                Abo-Link eines Wochenplans
 *   ferienImport           Ferien aus ICS oder nach Bundesland
 *   heft                   Heft zu einem Fach anlegen
 *   zuweisen               Karte, Stunde, Zeitraum einem Konto zuweisen
 */
export const kann = (was) => umgebung?.kann?.[was] ?? true;

export const initEcho = () => umgebung?.initEcho?.() ?? null;
export const onEchoReconnect = (fn) => umgebung?.onEchoReconnect?.(fn) ?? (() => {});
export const leaveEchoChannel = (...a) => umgebung?.leaveEchoChannel?.(...a);

/** Die KI-Frage der Webapp -- ohne sie (App) rendert hier nichts. */
export const KiFrage = defineComponent({
  name: 'KiFrage',
  inheritAttrs: false,
  setup(_, { attrs }) {
    return () => (umgebung?.KiFrage ? h(umgebung.KiFrage, attrs) : null);
  },
});

export const textAuslesen = (...a) => von('textAuslesen')(...a);
export const istTextAuszugFehler = (e) => Boolean(umgebung?.istTextAuszugFehler?.(e));

export const createProjectNotesSource = (...a) => von('createProjectNotesSource')(...a);
export const notizErweiterungen = () => umgebung?.NOTIZ_ERWEITERUNGEN ?? {};
