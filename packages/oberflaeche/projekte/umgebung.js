/*
 * Die Umgebung der Projekt-Ansichten.
 *
 * WARUM ES DIESE DATEI GIBT. Die Projekt-Ansichten rufen `api` auf; die App
 * setzt dafür ein Objekt mit DENSELBEN Methodennamen ein (lokaler Speicher,
 * Abgleich, vor Ort), statt ein eigenes Interface zu schneiden.
 *
 * Gesetzt wird sie EINMAL beim Start des Rahmens (`projektUmgebungSetzen`).
 * Beide Rahmen sind eigene Bündel; ein Modulzustand reicht.
 *
 * Was hier hindurchgeht:
 *   api                      die Server-Aufrufe (bzw. ihr Ersatz in der App)
 *   initEcho / onEchoReconnect / leaveEchoChannel
 *                            der Live-Kanal; ohne ihn (App) gibt initEcho null
 *   WHITEBOARD_VERWEIS       was eine Karte im Whiteboard öffnet
 *   whiteboardSitzung        gemeinsam zeichnen vor Ort
 */

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

export const initEcho = () => umgebung?.initEcho?.() ?? null;
export const onEchoReconnect = (fn) => umgebung?.onEchoReconnect?.(fn) ?? (() => {});
export const leaveEchoChannel = (...a) => umgebung?.leaveEchoChannel?.(...a);

// Die Ansicht, die eine Karte im Whiteboard öffnet (Props projekt, verweis;
// Ereignis close) -- oder nichts, dann bleibt die Karte stumm.
export const whiteboardVerweis = () => umgebung?.WHITEBOARD_VERWEIS ?? null;
// Gemeinsam zeichnen (W3b): ({ projekt, uuid, ausgangstext }) → Sitzung wie
// in app/src/sitzung.js -- oder nichts, dann zeichnet nur der Besitzer.
export const whiteboardSitzung = () => umgebung?.whiteboardSitzung ?? null;
