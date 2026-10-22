import { describe, it, expect } from 'vitest';
import * as Y from 'yjs';
import { Awareness } from 'y-protocols/awareness';
import { neuer, eintragen, anlegen, zusammenfuehren, elementeVon, mitzeichner, ELEMENTE, LOKAL } from './yBindung';

const el = (id, version, mehr = {}) => ({ id, type: 'rectangle', version, versionNonce: 5, ...mehr });

// Zwei Dokumente, die sich alles schicken -- wie zwei Geräte in einer Sitzung.
function paar() {
  const a = new Y.Doc();
  const b = new Y.Doc();
  a.on('update', (u, h) => { if (h !== 'b') Y.applyUpdate(b, u, 'a'); });
  b.on('update', (u, h) => { if (h !== 'a') Y.applyUpdate(a, u, 'b'); });
  return [a, b];
}

describe('Whiteboard ↔ Yjs', () => {
  it('neuer: Version, bei Gleichstand die kleinere Nonce', () => {
    expect(neuer(el('a', 2), el('a', 1))).toBe(true);
    expect(neuer(el('a', 1), el('a', 2))).toBe(false);
    expect(neuer(el('a', 1, { versionNonce: 3 }), el('a', 1, { versionNonce: 9 }))).toBe(true);
    expect(neuer(el('a', 1), undefined)).toBe(true);
  });

  it('trägt nur ein, was neuer ist -- als lokale Änderung', () => {
    const doc = new Y.Doc();
    let herkunft = null;
    doc.on('update', (_u, h) => { herkunft = h; });
    expect(eintragen(doc, [el('a', 1)])).toBe(1);
    expect(herkunft).toBe(LOKAL);
    expect(eintragen(doc, [el('a', 1)])).toBe(0);
    expect(eintragen(doc, [el('a', 2)])).toBe(1);
    expect(doc.getMap(ELEMENTE).get('a').version).toBe(2);
  });

  it('zwei zeichnen verschiedene Elemente: beide sehen beides', () => {
    const [a, b] = paar();
    anlegen(a, [el('x', 1)]);
    eintragen(a, [el('x', 2)]);
    eintragen(b, [el('y', 1)]);
    const szeneA = zusammenfuehren([el('x', 2)], a);
    const szeneB = zusammenfuehren([el('y', 1)], b);
    expect(szeneA.map((e) => e.id).sort()).toEqual(['x', 'y']);
    expect(szeneB.map((e) => e.id).sort()).toEqual(['x', 'y']);
    expect(szeneB.find((e) => e.id === 'x').version).toBe(2);
  });

  it('dasselbe Element: überall gewinnt dieselbe Fassung, Löschen bleibt gelöscht', () => {
    const [a, b] = paar();
    anlegen(a, [el('x', 1)]);
    eintragen(a, [el('x', 3, { versionNonce: 1 })]);
    eintragen(b, [el('x', 3, { versionNonce: 7 })]); // verliert: größere Nonce
    expect(a.getMap(ELEMENTE).get('x').versionNonce).toBe(1);
    expect(b.getMap(ELEMENTE).get('x').versionNonce).toBe(1);

    eintragen(b, [el('x', 4, { isDeleted: true })]);
    const szene = zusammenfuehren([el('x', 3, { versionNonce: 1 })], a);
    expect(szene[0].isDeleted).toBe(true);
    expect(elementeVon(a)).toHaveLength(1);
  });

  it('zeigt die Zeiger der anderen, nicht den eigenen', () => {
    const doc = new Y.Doc();
    const aw = new Awareness(doc);
    aw.setLocalStateField('zeiger', { x: 1, y: 2 });
    aw.states.set(99, { user: { name: 'Ben', color: '#123456' }, zeiger: { x: 5, y: 6, knopf: 'down' } });
    const m = mitzeichner(aw, doc.clientID);
    expect([...m.keys()]).toEqual(['99']);
    expect(m.get('99')).toMatchObject({ pointer: { x: 5, y: 6 }, button: 'down', username: 'Ben' });
    aw.destroy();
  });
});
