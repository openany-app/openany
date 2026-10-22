import { describe, it, expect } from 'vitest';
import { SitzungsProvider } from './sitzungsProvider';

/*
 * Ein Knoten wie in openany-nahbereich/src/sitzungen.rs, nur im Speicher:
 * fortlaufende Nummern, Ausgangstext genau einmal, Warten bis Neues kommt.
 */
class Knoten {
  constructor(inhalt = '') {
    this.nr = 0;
    this.aenderungen = [];
    this.awareness = new Map();
    this.weg = [];
    this.teilnehmer = new Map();
    this.anleger = null;
    this.inhalt = inhalt;
    this.wartende = [];
    this.aus = false;
  }

  wecken() {
    const w = this.wartende.splice(0);
    w.forEach((f) => f());
  }

  beitreten(name) {
    const kennung = `k${this.teilnehmer.size + 1}-${name}`;
    this.nr += 1;
    this.teilnehmer.set(kennung, { kennung, name, seit: this.nr });
    let anlegen = false;
    if (!this.aenderungen.length && (!this.anleger || !this.teilnehmer.has(this.anleger))) {
      this.anleger = kennung;
      anlegen = true;
    }
    this.wecken();
    return {
      kennung,
      stand: null,
      aenderungen: this.aenderungen.map(([, a]) => a),
      awareness: [...this.awareness].map(([k, [, a]]) => [k, a]),
      teilnehmer: [...this.teilnehmer.values()],
      nr: this.nr,
      anlegen,
      schreiber: this.schreiber(),
      inhalt: this.inhalt,
    };
  }

  schreiber() {
    return [...this.teilnehmer.values()].sort((a, b) => a.seit - b.seit)[0]?.kennung ?? '';
  }

  seit(kennung, ab) {
    return {
      stand: null,
      aenderungen: this.aenderungen.filter(([n]) => n > ab).map(([, a]) => a),
      awareness: [...this.awareness].filter(([k, [n]]) => n > ab && k !== kennung).map(([k, [, a]]) => [k, a]),
      weg: this.weg.filter(([n]) => n > ab).map(([, k]) => k),
      teilnehmer: [...this.teilnehmer.values()],
      nr: this.nr,
      schreiber: this.schreiber(),
      zusammenfassen: false,
      anlegen: false,
    };
  }

  transport(kennung) {
    return {
      senden: async (aenderungen, awareness) => {
        if (this.aus) throw new Error('weg');
        for (const a of aenderungen) {
          this.nr += 1;
          this.aenderungen.push([this.nr, a]);
        }
        if (awareness) {
          this.nr += 1;
          this.awareness.set(kennung, [this.nr, awareness]);
        }
        if (this.aenderungen.length) this.anleger = null;
        this.wecken();
        return this.nr;
      },
      warten: async (ab) => {
        if (this.aus) throw new Error('nicht erreichbar');
        if (this.nr <= ab) await new Promise((r) => this.wartende.push(r));
        if (this.aus) throw new Error('nicht erreichbar');
        return this.seit(kennung, ab);
      },
      zusammenfassen: async () => {},
      verlassen: async () => {
        this.teilnehmer.delete(kennung);
        this.awareness.delete(kennung);
        this.nr += 1;
        this.weg.push([this.nr, kennung]);
        this.wecken();
      },
    };
  }

  abschalten() {
    this.aus = true;
    this.wecken();
  }
}

const text = (p) => p.doc.getText('t').toString();
const ausgangstext = (doc, inhalt) => doc.getText('t').insert(0, inhalt);
const kurz = (ms = 30) => new Promise((r) => setTimeout(r, ms));

function teilnehmen(knoten, name, mehr = {}) {
  const start = knoten.beitreten(name);
  const p = new SitzungsProvider({
    start,
    transport: knoten.transport(start.kennung),
    ausgangstext,
    sammeln: 5,
    versuche: 2,
    ...mehr,
  });
  p.lauf = p.laufen();
  return p;
}

describe('SitzungsProvider', () => {
  it('legt den Ausgangstext genau einmal an', async () => {
    const k = new Knoten('Milch');
    const a = teilnehmen(k, 'Anna');
    const b = teilnehmen(k, 'Ben');
    await kurz(60);
    expect(text(a)).toBe('Milch');
    expect(text(b)).toBe('Milch');
    await a.zerstoeren();
    await b.zerstoeren();
  });

  it('führt gleichzeitiges Schreiben zusammen', async () => {
    const k = new Knoten('Liste');
    const a = teilnehmen(k, 'Anna');
    const b = teilnehmen(k, 'Ben');
    await kurz(60);

    a.doc.getText('t').insert(0, 'A ');
    b.doc.getText('t').insert(text(b).length, ' B');
    await kurz(100);

    expect(text(a)).toBe(text(b));
    expect(text(a)).toContain('A ');
    expect(text(a)).toContain(' B');
    expect(text(a)).toContain('Liste');
    await a.zerstoeren();
    await b.zerstoeren();
  });

  it('zeigt die Anwesenden, und wer geht, verschwindet samt Cursor', async () => {
    const k = new Knoten('x');
    let anwesend = [];
    const a = teilnehmen(k, 'Anna', { onAnwesende: (t) => { anwesend = t; } });
    const b = teilnehmen(k, 'Ben');
    b.setzeIch({ name: 'Ben', color: '#123456' });
    await kurz(80);

    expect(anwesend.map((t) => t.name)).toEqual(['Anna', 'Ben']);
    const fremde = () => [...a.awareness.getStates()].filter(([id]) => id !== a.doc.clientID);
    expect(fremde().map(([, s]) => s.user?.name)).toEqual(['Ben']);

    await b.zerstoeren();
    await kurz(60);
    expect(anwesend.map((t) => t.name)).toEqual(['Anna']);
    expect(fremde()).toEqual([]);
    await a.zerstoeren();
  });

  it('wer zuerst da ist, speichert; geht er, speichert der nächste', async () => {
    const k = new Knoten('x');
    const a = teilnehmen(k, 'Anna');
    let bSpeichert = false;
    const b = teilnehmen(k, 'Ben', { onSchreiber: (ja) => { bSpeichert = ja; } });
    await kurz(40);
    expect(a.ichSpeichere).toBe(true);
    expect(b.ichSpeichere).toBe(false);

    await a.zerstoeren();
    await kurz(60);
    expect(bSpeichert).toBe(true);
    await b.zerstoeren();
  });

  it('ist der Knoten weg, schaltet er auf nur lesen', async () => {
    const k = new Knoten('x');
    let aus = '';
    const a = teilnehmen(k, 'Anna', { onAus: (g) => { aus = g; } });
    await kurz(30);
    k.abschalten();
    await a.lauf;
    expect(aus).toContain('nicht erreichbar');
    expect(a.aus).toBe(true);
  });
});
