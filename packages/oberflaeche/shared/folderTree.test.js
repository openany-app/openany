import { describe, it, expect } from 'vitest';
import { childFolders, buildBreadcrumbs, folderTreeOptions } from '@oberflaeche/shared/folderTree';

// Flache Mappen-Liste wie vom Server (id, name, parent_id).
const folders = [
  { id: 1, name: 'Zebra', parent_id: null },
  { id: 2, name: 'Apfel', parent_id: null },
  { id: 3, name: 'Birne', parent_id: 2 },   // unter Apfel
  { id: 4, name: 'Ananas', parent_id: 2 },  // unter Apfel
  { id: 5, name: 'Tief', parent_id: 3 },    // unter Apfel/Birne
];

describe('childFolders', () => {
  it('liefert die Wurzel-Mappen alphabetisch (parentId = null)', () => {
    expect(childFolders(folders, null).map((f) => f.name)).toEqual(['Apfel', 'Zebra']);
  });

  it('liefert die Kinder einer Mappe alphabetisch', () => {
    expect(childFolders(folders, 2).map((f) => f.name)).toEqual(['Ananas', 'Birne']);
  });

  it('gibt leere Liste zurück, wenn es keine Kinder gibt', () => {
    expect(childFolders(folders, 5)).toEqual([]);
  });

  it('behandelt fehlendes parent_id wie oberste Ebene', () => {
    const mixed = [{ id: 9, name: 'Ohne', /* kein parent_id */ }];
    expect(childFolders(mixed, null).map((f) => f.id)).toEqual([9]);
  });
});

describe('buildBreadcrumbs', () => {
  it('gibt in der obersten Ebene nur Home zurück', () => {
    expect(buildBreadcrumbs(folders, null)).toEqual([{ id: null, name: 'Home' }]);
  });

  it('baut den Pfad von Home bis zur aktuellen Mappe', () => {
    expect(buildBreadcrumbs(folders, 5)).toEqual([
      { id: null, name: 'Home' },
      { id: 2, name: 'Apfel' },
      { id: 3, name: 'Birne' },
      { id: 5, name: 'Tief' },
    ]);
  });

  it('bricht bei verwaister Mappe (fehlender Elternteil) sauber ab', () => {
    const orphan = [{ id: 7, name: 'Verwaist', parent_id: 99 }];
    expect(buildBreadcrumbs(orphan, 7)).toEqual([
      { id: null, name: 'Home' },
      { id: 7, name: 'Verwaist' },
    ]);
  });

  it('läuft bei einem Zyklus nicht endlos', () => {
    const cyclic = [
      { id: 1, name: 'A', parent_id: 2 },
      { id: 2, name: 'B', parent_id: 1 },
    ];
    const bc = buildBreadcrumbs(cyclic, 1);
    expect(bc[0]).toEqual({ id: null, name: 'Home' });
    expect(bc).toHaveLength(3); // Home + die zwei Zyklus-Glieder, dann Stopp
  });
});

describe('folderTreeOptions', () => {
  it('liefert eine DFS-Liste mit korrekter Tiefe, pro Ebene alphabetisch', () => {
    expect(folderTreeOptions(folders).map((f) => [f.name, f.depth])).toEqual([
      ['Apfel', 0],
      ['Ananas', 1],
      ['Birne', 1],
      ['Tief', 2],
      ['Zebra', 0],
    ]);
  });

  it('gibt für eine leere Liste ein leeres Ergebnis zurück', () => {
    expect(folderTreeOptions([])).toEqual([]);
  });
});
