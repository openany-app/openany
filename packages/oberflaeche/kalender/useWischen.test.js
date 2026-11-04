import { describe, expect, it, vi, beforeEach } from 'vitest';
import { ref } from 'vue';
import { useBlaettern } from './useWischen';

function flaeche() {
  const laufend = [];
  return {
    laufend,
    getAnimations: () => laufend,
    animate: vi.fn((schritte) => {
      let abbrechen;
      const finished = new Promise((fertig, fehler) => {
        abbrechen = () => fehler(new Error('abgebrochen'));
        queueMicrotask(() => setTimeout(fertig));
      });
      const a = { schritte, finished, cancel: () => abbrechen() };
      laufend.push(a);
      return a;
    }),
  };
}

describe('useBlaettern', () => {
  beforeEach(() => {
    vi.stubGlobal('window', { matchMedia: () => ({ matches: false }) });
  });

  it('blättert erst, wenn die alte Seite draußen ist, und kommt von der Gegenseite', async () => {
    const el = flaeche();
    const { blaettern, laeuft } = useBlaettern(ref(el));
    const aendern = vi.fn(() => expect(el.animate).toHaveBeenCalledTimes(1));

    await blaettern('links', aendern);

    expect(aendern).toHaveBeenCalledOnce();
    expect(el.animate.mock.calls[0][0][1].transform).toBe('translate(-25%, 0)');
    expect(el.animate.mock.calls[1][0][0].transform).toBe('translate(25%, 0)');
    expect(laeuft.value).toBe(false);
  });

  it('verliert bei einem zweiten Wisch mitten im ersten keinen Schritt', async () => {
    const el = flaeche();
    const { blaettern, laeuft } = useBlaettern(ref(el));
    const aendern = vi.fn();

    const erster = blaettern('hoch', aendern);
    const zweiter = blaettern('hoch', aendern);
    await Promise.all([erster, zweiter]);

    expect(aendern).toHaveBeenCalledTimes(2);
    expect(laeuft.value).toBe(false);
  });

  it('blättert ohne Bewegung, wenn das Gerät keine möchte', async () => {
    vi.stubGlobal('window', { matchMedia: () => ({ matches: true }) });
    const el = flaeche();
    const { blaettern } = useBlaettern(ref(el));
    const aendern = vi.fn();

    await blaettern('rechts', aendern);

    expect(aendern).toHaveBeenCalledOnce();
    expect(el.animate).not.toHaveBeenCalled();
  });
});
