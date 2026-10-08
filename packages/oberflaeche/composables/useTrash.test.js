import { describe, it, expect, vi } from 'vitest';
import { useTrash } from '@oberflaeche/composables/useTrash';
import { useConfirm } from '@oberflaeche/composables/useConfirm';

// Standard-Konfiguration mit steuerbaren Fakes; einzelne Tests überschreiben
// gezielt Teile davon.
function makeTrash(overrides = {}) {
  const cfg = {
    fetchPage: vi.fn((page) =>
      Promise.resolve({
        items: page === 1 ? [{ id: 1 }, { id: 2 }] : [{ id: 3 }],
        next_page: page === 1 ? 2 : null,
      })
    ),
    restore: vi.fn(() => Promise.resolve()),
    forceDelete: vi.fn(() => Promise.resolve()),
    label: (item) => `#${item.id}`,
    ...overrides,
  };
  return { cfg, trash: useTrash(cfg) };
}

// Bestätigungsdialog (Singleton) aus dem Test heraus beantworten: die
// forceDeleteItem-Promise hängt an confirmDelete, bis respond() aufgerufen wird.
async function answerConfirm(promise, value) {
  await Promise.resolve();
  useConfirm().respond(value);
  return promise;
}

describe('useTrash', () => {
  it('load() ersetzt die Liste und übernimmt next_page', async () => {
    const { cfg, trash } = makeTrash();

    await trash.load();

    expect(cfg.fetchPage).toHaveBeenCalledWith(1);
    expect(trash.items.value).toEqual([{ id: 1 }, { id: 2 }]);
    expect(trash.nextPage.value).toBe(2);
    expect(trash.isLoading.value).toBe(false);
  });

  it('loadMore() hängt die nächste Seite an und toggelt loadingMore', async () => {
    const { trash } = makeTrash();
    await trash.load();

    await trash.loadMore();

    expect(trash.items.value).toEqual([{ id: 1 }, { id: 2 }, { id: 3 }]);
    expect(trash.nextPage.value).toBeNull();
    expect(trash.loadingMore.value).toBe(false);
  });

  it('loadMore() ohne next_page lädt nicht nach', async () => {
    const { cfg, trash } = makeTrash();
    await trash.load();
    await trash.loadMore(); // next_page wird null
    cfg.fetchPage.mockClear();

    await trash.loadMore();

    expect(cfg.fetchPage).not.toHaveBeenCalled();
  });

  it('restoreItem() entfernt das Element und ruft onRemoved', async () => {
    const onRemoved = vi.fn();
    const { cfg, trash } = makeTrash({ onRemoved });
    await trash.load();

    await trash.restoreItem({ id: 1 });

    expect(cfg.restore).toHaveBeenCalledWith({ id: 1 });
    expect(trash.items.value).toEqual([{ id: 2 }]);
    expect(onRemoved).toHaveBeenCalledWith({ id: 1 });
  });

  it('restoreItem() lässt das Element bei Fehler in der Liste', async () => {
    const { trash } = makeTrash({ restore: vi.fn(() => Promise.reject(new Error('boom'))) });
    await trash.load();

    await trash.restoreItem({ id: 1 });

    expect(trash.items.value).toEqual([{ id: 1 }, { id: 2 }]);
  });

  it('forceDeleteItem() löscht nach Bestätigung', async () => {
    const { cfg, trash } = makeTrash();
    await trash.load();

    await answerConfirm(trash.forceDeleteItem({ id: 2 }), true);

    expect(cfg.forceDelete).toHaveBeenCalledWith({ id: 2 });
    expect(trash.items.value).toEqual([{ id: 1 }]);
  });

  it('forceDeleteItem() tut nichts, wenn die Bestätigung abgelehnt wird', async () => {
    const { cfg, trash } = makeTrash();
    await trash.load();

    await answerConfirm(trash.forceDeleteItem({ id: 2 }), false);

    expect(cfg.forceDelete).not.toHaveBeenCalled();
    expect(trash.items.value).toEqual([{ id: 1 }, { id: 2 }]);
  });
  it('drop() vergleicht Typ UND Id (gemischte Liste)', async () => {
    const { trash } = makeTrash({
      fetchPage: () => Promise.resolve({
        items: [{ type: 'note', id: 7 }, { type: 'project', id: 7 }],
        next_page: null,
      }),
    });
    await trash.load();

    await trash.restoreItem({ type: 'note', id: 7 });

    // Das gleichnamige Projekt #7 bleibt stehen.
    expect(trash.items.value).toEqual([{ type: 'project', id: 7 }]);
  });
});
