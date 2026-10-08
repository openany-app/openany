import { describe, it, expect } from 'vitest';
import { gezeigteNotizen, leistenTags, nachEbenen } from './graphAuswahl';

const a1 = { id: 1, group: 10 };
const a2 = { id: 2, group: 10 };
const b1 = { id: 3, group: 20 };
const lose = { id: 4, group: null };
const alle = [a1, a2, b1, lose];
const bwl = { tag: 'bwl' };
const party = { tag: 'party' };
const kanten = [
  { from: a1, to: bwl },
  { from: a2, to: bwl },
  { from: b1, to: party },
  { from: b1, to: bwl },
];
const tags = [{ tag: 'bwl', count: 3 }, { tag: 'party', count: 1 }];

describe('Graphenansicht: Auswahl', () => {
  it('zeigt ohne Ebene und ohne Tag nichts', () => {
    expect(gezeigteNotizen(alle, new Set(), 3, kanten, new Set())).toEqual([]);
  });

  it('zeigt ohne Ebene die Notizen der gewählten Tags', () => {
    expect(gezeigteNotizen(alle, new Set(), 3, kanten, new Set(['party']))).toEqual([b1]);
  });

  it('zeigt nur die gewählten Ebenen, auch die oberste', () => {
    expect(gezeigteNotizen(alle, new Set(['10', 'root']), 3, kanten, new Set(['party']))).toEqual([a1, a2, lose]);
  });

  it('zeigt bei nur einer Ebene alles, ohne Wahl', () => {
    expect(gezeigteNotizen([a1, a2], new Set(), 1)).toEqual([a1, a2]);
  });

  it('nennt ohne Ebene alle Tags mit ihrer ganzen Zahl', () => {
    const leiste = leistenTags(tags, kanten, new Set([b1]), nachEbenen(new Set(), 3));
    expect([...leiste.entries()]).toEqual([['bwl', 3], ['party', 1]]);
  });

  it('nennt mit Ebenen nur Tags der gezeigten Notizen, mit deren Zahl', () => {
    const leiste = leistenTags(tags, kanten, new Set([a1, a2]), nachEbenen(new Set(['10']), 3));
    expect([...leiste.entries()]).toEqual([['bwl', 2]]);
  });
});
