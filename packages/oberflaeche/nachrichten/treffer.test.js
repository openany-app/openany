import { describe, it, expect } from 'vitest';
import { trefferTeile } from './treffer';

describe('trefferTeile', () => {
  it('ohne Suche ein Stück', () => {
    expect(trefferTeile('Hallo', '  ')).toEqual([{ t: 'Hallo', treffer: false }]);
  });
  it('markiert jeden Treffer, Groß/klein egal, Schreibweise bleibt', () => {
    expect(trefferTeile('Kanal am kanal', 'KANAL')).toEqual([
      { t: 'Kanal', treffer: true },
      { t: ' am ', treffer: false },
      { t: 'kanal', treffer: true },
    ]);
  });
  it('Treffer in der Mitte, und nichts für null', () => {
    expect(trefferTeile('Treffen um drei', 'um').map((x) => x.treffer)).toEqual([false, true, false]);
    expect(trefferTeile(null, 'x')).toEqual([]);
  });
});
