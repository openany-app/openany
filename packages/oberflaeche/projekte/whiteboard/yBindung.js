/*
 * Gemeinsam zeichnen (docs/plan-whiteboard.md, W3b): Excalidraw ↔ Yjs.
 *
 * Ein `Y.Map` hält die Elemente nach ihrer Kennung. Excalidraw zählt an
 * jedem Element selbst mit (`version`, bei Gleichstand entscheidet die
 * kleinere `versionNonce` -- dieselbe Regel wie Excalidraws eigener
 * Abgleich). Darum stören sich zwei nicht, die verschiedene Elemente
 * bewegen, und bei demselben gewinnt überall dieselbe Fassung.
 *
 * Gelöschte Elemente bleiben als `isDeleted` in der Map -- sonst brächte ein
 * Gerät, das die Löschung noch nicht kennt, sie zurück.
 */

export const ELEMENTE = 'elemente';
export const LOKAL = 'whiteboard-lokal';

/** Ist `a` neuer als `b`? */
export function neuer(a, b) {
  if (!b) return true;
  if (!a) return false;
  if ((a.version ?? 0) !== (b.version ?? 0)) return (a.version ?? 0) > (b.version ?? 0);
  return (a.versionNonce ?? 0) < (b.versionNonce ?? 0);
}

// Excalidraw friert seine Elemente ein; Yjs bekommt eine schlichte Kopie.
const kopie = (e) => JSON.parse(JSON.stringify(e));

/** Eigene Änderungen in die Map: jedes Element, das neuer ist als dort. */
export function eintragen(doc, elemente) {
  const karte = doc.getMap(ELEMENTE);
  const neu = (elemente ?? []).filter((e) => neuer(e, karte.get(e.id)));
  if (!neu.length) return 0;
  doc.transact(() => {
    for (const e of neu) karte.set(e.id, kopie(e));
  }, LOKAL);
  return neu.length;
}

/** Die Ausgangszeichnung in eine leere Map (genau einmal je Sitzung). */
export function anlegen(doc, elemente) {
  const karte = doc.getMap(ELEMENTE);
  doc.transact(() => {
    for (const e of elemente ?? []) karte.set(e.id, kopie(e));
  }, LOKAL);
}

/**
 * Die Szene, wie sie nach fremden Änderungen aussehen soll: je Kennung die
 * neuere Fassung aus Szene und Map; die Reihenfolge der Szene bleibt, neue
 * Elemente kommen dahinter.
 */
export function zusammenfuehren(szene, doc) {
  const karte = doc.getMap(ELEMENTE);
  const aus = [];
  const gesehen = new Set();
  for (const e of szene ?? []) {
    gesehen.add(e.id);
    const dort = karte.get(e.id);
    aus.push(dort && neuer(dort, e) ? dort : e);
  }
  for (const [id, e] of karte.entries()) {
    if (!gesehen.has(id)) aus.push(e);
  }
  return aus;
}

/** Die Elemente der Map -- etwa als Ausgangspunkt beim Öffnen. */
export const elementeVon = (doc) => [...doc.getMap(ELEMENTE).values()];

/**
 * Die Zeiger der anderen für Excalidraws `collaborators`: aus der Awareness
 * (`user`: Name und Farbe, `zeiger`: x, y, Knopf).
 */
export function mitzeichner(awareness, eigeneId) {
  const aus = new Map();
  for (const [id, s] of awareness.getStates()) {
    if (id === eigeneId || !s?.zeiger) continue;
    const farbe = s.user?.color ?? '#1971c2';
    aus.set(String(id), {
      pointer: { x: s.zeiger.x, y: s.zeiger.y, tool: 'pointer' },
      button: s.zeiger.knopf === 'down' ? 'down' : 'up',
      username: s.user?.name ?? '',
      color: { background: farbe, stroke: farbe },
    });
  }
  return aus;
}
