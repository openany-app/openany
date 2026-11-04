import { describe, it, expect, beforeEach } from 'vitest';
import { eintragen, nochOffen, austragen, rahmenOffen, lebt } from '@oberflaeche/composables/verlaufsrahmen';

// Ein Verlauf wie im Browser: ein Stapel von Zuständen.
function verlauf() {
  const zustaende = [null];
  globalThis.history = {
    get state() { return zustaende.at(-1); },
    pushState: (s) => { zustaende.push(s); },
    back: () => { zustaende.pop(); },
  };
}

describe('verlaufsrahmen', () => {
  beforeEach(verlauf);

  it('ein verwaister Eintrag darüber schließt den Rahmen darunter nicht', async () => {
    eintragen('rahmen');
    eintragen('ansicht');
    // Wechsel im selben Rendern: Die Ansicht schließt vor dem `onMounted` des
    // Bearbeiten-Dialogs. Sie liegt danach nicht oben und räumt nichts ab.
    austragen('ansicht');
    eintragen('bearbeiten');
    await Promise.resolve();
    expect(history.state.dialog).toBe('bearbeiten');

    // Der Bearbeiten-Dialog schließt: Der Verlauf landet auf dem
    // verwaisten Eintrag -- der Rahmen steht noch im Stapel.
    austragen('bearbeiten');
    await Promise.resolve();
    expect(history.state.dialog).toBe('ansicht');
    expect(nochOffen('rahmen')).toBe(true);
    expect(nochOffen('bearbeiten')).toBe(false);
    austragen('rahmen');
  });

  it('räumt den eigenen Eintrag ab, wenn nichts Neues darüberliegt', async () => {
    eintragen('dialog');
    austragen('dialog');
    await Promise.resolve();
    expect(history.state).toBeNull();
  });

  it('ohne Eintrag ist nichts offen', () => {
    expect(nochOffen('rahmen')).toBe(false);
  });

  it('weiß, ob überhaupt ein Rahmen offen ist', () => {
    expect(rahmenOffen()).toBe(false);
    eintragen('rahmen');
    expect(rahmenOffen()).toBe(true);
    austragen('rahmen');
    expect(rahmenOffen()).toBe(false);
  });

  it('eine Ordnerebene zählt nicht als offener Rahmen, lebt aber', () => {
    eintragen('ebene', { ebene: true });
    expect(rahmenOffen()).toBe(false);
    expect(lebt('ebene')).toBe(true);
    austragen('ebene');
    expect(lebt('ebene')).toBe(false);
  });

  it('kennt die lebenden Einträge', () => {
    eintragen('rahmen');
    eintragen('dialog');
    austragen('dialog');
    // Zurück landet auf dem Rahmen: Er lebt, niemand darf darüber springen.
    expect(lebt('rahmen')).toBe(true);
    expect(lebt('dialog')).toBe(false);
    austragen('rahmen');
  });
});
