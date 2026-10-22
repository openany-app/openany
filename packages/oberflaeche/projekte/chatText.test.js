import { describe, it, expect } from 'vitest';
import { parseChatText, linkKey, linkMarkup, SHARED_KINDS } from './chatText';

const typen = (body) => parseChatText(body).map((s) => s.type);

describe('parseChatText – Text', () => {
  it('gibt reinen Text als ein Stück zurück', () => {
    expect(parseChatText('Bis Samstag!')).toEqual([{ type: 'text', value: 'Bis Samstag!' }]);
  });

  it('kommt mit leer und Unsinn klar', () => {
    expect(parseChatText('')).toEqual([]);
    expect(parseChatText(null)).toEqual([]);
    expect(parseChatText(undefined)).toEqual([]);
  });

  it('behält Zeilenumbrüche im Text', () => {
    expect(parseChatText('a\nb')).toEqual([{ type: 'text', value: 'a\nb' }]);
  });
});

describe('parseChatText – Wikilinks', () => {
  it('erkennt einen Verweis mitten im Satz und behält die Reihenfolge', () => {
    expect(parseChatText('Mengen in [[Getränkeliste]] nachsehen')).toEqual([
      { type: 'text', value: 'Mengen in ' },
      { type: 'wikilink', target: 'Getränkeliste', label: 'Getränkeliste', kind: null, id: null },
      { type: 'text', value: ' nachsehen' },
    ]);
  });

  it('nimmt den Teil vor dem Strich als Ziel, den dahinter als Anzeige', () => {
    expect(parseChatText('[[Getränkeliste|die Liste]]')).toEqual([
      { type: 'wikilink', target: 'Getränkeliste', label: 'die Liste', kind: null, id: null },
    ]);
  });

  it('blendet die Kennung aus der Anzeige aus, behält sie aber als Ziel', () => {
    const [s] = parseChatText('[[20260801182245 Getränkeliste]]');
    expect(s.target).toBe('20260801182245 Getränkeliste');
    expect(s.label).toBe('Getränkeliste');
  });

  it('lässt eine reine Kennung als Anzeige stehen, wenn nichts übrig bliebe', () => {
    const [s] = parseChatText('[[20260801182245]]');
    expect(s.label).toBe('20260801182245');
  });

  it('erkennt mehrere Verweise in einer Nachricht', () => {
    expect(typen('[[A]] und [[B]]')).toEqual(['wikilink', 'text', 'wikilink']);
  });

  it('ignoriert unvollständige und leere Klammern', () => {
    expect(typen('offen [[ohne Ende')).toEqual(['text']);
    expect(typen('[[]]')).toEqual(['text']);
    expect(typen('[[|nur Anzeige]]')).toEqual(['text']);
  });

  it('läuft nicht über Zeilenumbrüche hinweg', () => {
    expect(typen('[[A\nB]]')).toEqual(['text']);
  });
});

describe('parseChatText – Web-Adressen', () => {
  it('erkennt http und https', () => {
    expect(typen('http://a.de')).toEqual(['url']);
    expect(typen('https://a.de')).toEqual(['url']);
  });

  it('lässt den Schlusspunkt beim Satz, nicht bei der Adresse', () => {
    expect(parseChatText('Karte: https://kaffeeliebe.de/karte.')).toEqual([
      { type: 'text', value: 'Karte: ' },
      { type: 'url', href: 'https://kaffeeliebe.de/karte', label: 'https://kaffeeliebe.de/karte' },
      { type: 'text', value: '.' },
    ]);
  });

  it('gibt eine schließende Klammer zurück, die nicht zur Adresse gehört', () => {
    const s = parseChatText('(siehe https://a.de/x)');
    expect(s[1]).toEqual({ type: 'url', href: 'https://a.de/x', label: 'https://a.de/x' });
    expect(s[2]).toEqual({ type: 'text', value: ')' });
  });

  it('behält Klammern, die zur Adresse gehören', () => {
    const [s] = parseChatText('https://de.wikipedia.org/wiki/Kubb_(Spiel)');
    expect(s.href).toBe('https://de.wikipedia.org/wiki/Kubb_(Spiel)');
  });

  it('macht aus javascript: und data: keinen Link', () => {
    expect(typen('javascript:alert(1)')).toEqual(['text']);
    expect(typen('data:text/html,<script>')).toEqual(['text']);
  });

  it('ignoriert Adressen ohne Schema', () => {
    expect(typen('kaffeeliebe.de/karte')).toEqual(['text']);
  });
});

describe('parseChatText – gemischt', () => {
  it('erkennt Verweis und Adresse in einer Nachricht', () => {
    expect(typen('[[Getränkeliste]] und https://a.de dazu')).toEqual(['wikilink', 'text', 'url', 'text']);
  });

  it('ist bei mehrfachem Aufruf stabil (kein hängender lastIndex)', () => {
    const eingabe = '[[A]] https://a.de';
    expect(parseChatText(eingabe)).toEqual(parseChatText(eingabe));
  });
});

describe('parseChatText – ohne Wikilinks (Direktnachrichten)', () => {
  const ohne = (s) => parseChatText(s, { wikilinks: false });

  it('lässt [[…]] unverändert im Text stehen', () => {
    expect(ohne('siehe [[Getränkeliste]] dort')).toEqual([
      { type: 'text', value: 'siehe [[Getränkeliste]] dort' },
    ]);
  });

  it('behält die Klammern auch bei kanonischer Form mit Kennung', () => {
    const stuecke = ohne('[[20260801182244 Getränkeliste]]');
    expect(stuecke).toHaveLength(1);
    expect(stuecke[0].value).toBe('[[20260801182244 Getränkeliste]]');
  });

  it('verlinkt Adressen weiterhin', () => {
    expect(typen('a https://a.de b')).toEqual(['text', 'url', 'text']);
    expect(ohne('[[A]] https://a.de').map((s) => s.type)).toEqual(['text', 'url']);
  });
});

describe('parseChatText – getippte Ziele ([[art:id]])', () => {
  const eins = (s) => parseChatText(s)[0];

  it('zerlegt Art und Id', () => {
    expect(eins('[[folder:12]]')).toEqual({
      type: 'wikilink', target: 'folder:12', label: '', kind: 'folder', id: 12,
    });
  });

  it('kennt alle Arten', () => {
    // Muss mit ChatLinks::TYPED_TARGET im Backend übereinstimmen: Kennt die
    // eine Seite eine Art nicht, zeigt sie tot an, was die andere auflöst.
    const arten = ['note', 'folder', 'file', 'album', 'photo'];
    for (const art of arten) {
      expect(eins(`[[${art}:7]]`).kind).toBe(art);
    }
  });

  it('lässt die Anzeige leer, wenn keine dabeisteht', () => {
    // „folder:12" als Anzeigetext wäre schlicht falsch – den Namen kennt nur
    // der Server, und über ihn zeigt die Anzeige stets den aktuellen.
    expect(eins('[[album:3]]').label).toBe('');
  });

  it('nimmt einen mitgegebenen Anzeigetext', () => {
    expect(eins('[[folder:12|Einkauf]]')).toEqual({
      type: 'wikilink', target: 'folder:12', label: 'Einkauf', kind: 'folder', id: 12,
    });
  });

  it('behandelt unbekannte Arten wie einen gewöhnlichen Titel', () => {
    // Sonst verschwände der Text spurlos, wenn eine ältere Fassung des
    // Frontends eine neu hinzugekommene Art noch nicht kennt.
    const s = eins('[[gadget:12]]');
    expect(s.kind).toBeNull();
    expect(s.label).toBe('gadget:12');
  });

  it('ist kein getipptes Ziel ohne Id oder mit Zusatz', () => {
    expect(eins('[[folder:]]').kind).toBeNull();
    expect(eins('[[folder:12x]]').kind).toBeNull();
    expect(eins('[[Folder 12]]').kind).toBeNull();
  });

  it('erkennt die Art unabhängig von der Schreibung', () => {
    // Das Backend führt seine Karte kleingeschrieben (linkKey). Erkennte die
    // Anzeige [[Folder:12]] nicht als getippt, hätte sie ein aufgelöstes Ziel
    // vor sich und zeigte trotzdem toten Text.
    const s = eins('[[Folder:12]]');
    expect(s.kind).toBe('folder');
    expect(linkKey(s.target)).toBe('folder:12');
  });
});

describe('SHARED_KINDS', () => {
  it('nennt genau die Arten, die an einer Freigabe hängen', () => {
    // Die Trennung ist keine Kosmetik: Sie entscheidet, welcher Hinweis an
    // einem toten Verweis steht („nicht freigegeben" vs. „gibt es nicht") –
    // und in ProjectDetail, in welchen Reiter ein Klick führt.
    expect([...SHARED_KINDS].sort()).toEqual(['album', 'file', 'folder', 'note', 'photo']);
  });

  it('kennt keine Planungs-Arten mehr (Rückbau 16.10.2026)', () => {
    for (const art of ['board', 'roadmap', 'places', 'card', 'milestone', 'place']) {
      expect(parseChatText(`[[${art}:7]]`)[0].kind).toBeNull();
    }
  });
});

describe('linkKey', () => {
  it('schreibt klein – so führt das Backend seine Karte', () => {
    expect(linkKey('Getränkeliste')).toBe('getränkeliste');
    expect(linkKey('20260801182245 Getränke')).toBe('20260801182245 getränke');
  });

  it('kommt mit leer klar', () => {
    expect(linkKey(undefined)).toBe('');
  });
});

describe('linkMarkup', () => {
  it('setzt die getippte Form mit lesbarem Namen ein', () => {
    expect(linkMarkup({ kind: 'folder', id: 12, title: 'Einkauf' })).toBe('[[folder:12|Einkauf]]');
  });

  it('entschärft Klammern und Strich im Namen', () => {
    // Ein Ordner „Einkauf | Getränke" ergäbe sonst [[folder:12|Einkauf | Getränke]],
    // und der Zerleger schnitte am ERSTEN Strich – der Rest bliebe toter Text.
    expect(linkMarkup({ kind: 'folder', id: 12, title: 'Einkauf | Getränke' }))
      .toBe('[[folder:12|Einkauf Getränke]]');
    // „]]" beendete den Verweis sogar mitten im Namen.
    expect(linkMarkup({ kind: 'note', id: 3, title: 'Kosten [[alt]]' }))
      .toBe('[[note:3|Kosten alt]]');
  });

  it('lässt die Anzeige weg, wenn vom Namen nichts übrig bleibt', () => {
    expect(linkMarkup({ kind: 'folder', id: 12, title: '||' })).toBe('[[folder:12]]');
    expect(linkMarkup({ kind: 'folder', id: 12, title: null })).toBe('[[folder:12]]');
  });

  it('ergibt einen Verweis, den der Zerleger wieder auseinandernimmt', () => {
    // Die eigentliche Zusicherung: Was der Picker einsetzt, muss die Anzeige
    // als getipptes Ziel wiedererkennen – sonst stünde es tot in der Nachricht.
    const [s] = parseChatText(linkMarkup({ kind: 'album', id: 8, title: 'Umbau 2026' }));
    expect(s).toEqual({
      type: 'wikilink', target: 'album:8', label: 'Umbau 2026', kind: 'album', id: 8,
    });
  });
});
