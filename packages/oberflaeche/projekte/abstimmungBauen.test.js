import { describe, expect, it } from 'vitest';
import { abstimmungBauen } from './abstimmungBauen';

const ANNA = 1;
const BEN = 2;
const CARL = 3;
const mitglieder = [
  { user_id: ANNA, name: 'Anna', rolle: 'owner' },
  { user_id: BEN, name: 'Ben', rolle: 'member' },
  { user_id: CARL, name: 'Carl', rolle: 'member' },
];

let nr = 0;
const e = (von, inhalt, extra = {}) => ({
  nr: ++nr, uuid: `e${nr}`, von, von_name: String(von), zeit: '2026-10-19T10:00:00Z', geaendert: '2026-10-19T10:00:00Z', inhalt, ...extra,
});
const kopf = (typ, extra = {}) => e(ANNA, { art: 'kopf', typ, titel: 'Beispiel', einstellungen: {}, ...extra });
const option = (von, extra = {}) => e(von, { art: 'option', label: 'Beispiel', ...extra });
const stimme = (von, opt, antwort = 'yes', extra = {}) => e(von, { art: 'stimme', option: opt.uuid, antwort }, extra);
const bauen = (eintraege, ich = BEN, jetzt = new Date('2026-10-19T12:00:00Z')) => abstimmungBauen(
  { uuid: 'u1', created_by: ANNA, created_at: '2026-10-19T09:00:00Z', ich, mitglieder, eintraege }, null, jetzt,
);

describe('abstimmungBauen', () => {
  it('ohne Kopf der Erstellerin gibt es keine Abstimmung', () => {
    expect(bauen([e(BEN, { art: 'kopf', typ: 'survey', titel: 'Fremd' })])).toBeNull();
  });

  it('Kapazität: es zählen die ersten nach der Nummer des Servers', () => {
    const o = option(ANNA, { kapazitaet: 2 });
    const p = bauen([kopf('signup'), o, stimme(CARL, o), stimme(BEN, o), stimme(ANNA, o)]);
    expect(p.options[0].claims.map((c) => c.name)).toEqual(['Carl', 'Ben']);
    // Die eigene Person trägt meId (null).
    expect(p.options[0].claims[1].user_id).toBeNull();
  });

  it('Optionen von Mitgliedern nur bei „Mitglieder dürfen ergänzen"', () => {
    const fremd = option(BEN);
    expect(bauen([kopf('signup'), option(ANNA), fremd]).options).toHaveLength(1);
    expect(bauen([kopf('signup', { einstellungen: { member_options: true } }), option(ANNA), fremd]).options).toHaveLength(2);
  });

  it('Umfrage ohne Mehrfachwahl: die zuerst abgegebene Stimme zählt', () => {
    const a = option(ANNA);
    const b = option(ANNA);
    const p = bauen([kopf('survey'), a, b, stimme(BEN, a), stimme(BEN, b), stimme(CARL, b)]);
    expect(p.options.map((o) => o.count)).toEqual([1, 1]);
    expect(p.my_option_ids).toEqual([a.uuid]);
    expect(p.members.find((m) => m.name === 'Carl').voted).toBe(true);
  });

  it('anonyme Umfrage zeigt keine Namen', () => {
    const a = option(ANNA);
    const p = bauen([kopf('survey', { einstellungen: { anonymous: true } }), a, stimme(BEN, a)]);
    expect(p.votes).toBeUndefined();
    expect(p.voted_count).toBe(1);
  });

  it('Terminfindung zählt ja/nein/vielleicht und kennt die beste Option', () => {
    const a = option(ANNA, { beginn: '2026-11-02T10:00:00' });
    const b = option(ANNA, { beginn: '2026-11-03T10:00:00' });
    const p = bauen([kopf('schedule'), a, b, stimme(BEN, a, 'no'), stimme(BEN, b, 'yes'), stimme(CARL, b, 'maybe')]);
    expect(p.options[1].tally).toEqual({ yes: 1, no: 0, maybe: 1 });
    expect(p.best_option_id).toBe(b.uuid);
    expect(p.votes).toHaveLength(3);
  });

  it('Einladung: nur die eingeladene Person sagt zu oder ab', () => {
    const ben = option(ANNA, { label: 'Ben', person: BEN });
    const gast = option(ANNA, { label: 'Gast ohne Konto' });
    const p = bauen([kopf('invite'), ben, gast,
      e(CARL, { art: 'stimme', option: ben.uuid, antwort: 'no' }),
      e(BEN, { art: 'stimme', option: ben.uuid, antwort: 'yes' }),
      e(BEN, { art: 'gast', option: gast.uuid, status: 'yes' }),
      e(ANNA, { art: 'gast', option: gast.uuid, status: 'no' }),
    ]);
    expect(p.options.map((o) => o.invite_status)).toEqual(['yes', 'no']);
    expect(p.counts).toEqual({ total: 2, yes: 1, no: 1, invited: 0 });
  });

  it('nach Ablauf zählt nichts Neues mehr', () => {
    const a = option(ANNA);
    const p = bauen([
      kopf('signup', { bis: '2026-10-19T11:00:00Z' }), a,
      stimme(BEN, a),
      stimme(CARL, a, 'yes', { zeit: '2026-10-19T11:30:00Z', geaendert: '2026-10-19T11:30:00Z' }),
    ]);
    expect(p.status).toBe('closed');
    expect(p.options[0].claims).toHaveLength(1);
  });

  it('abhaken dürfen Erstellerin und wer eingetragen ist, sonst niemand', () => {
    const a = option(ANNA);
    const nurCarl = bauen([kopf('signup'), a, e(CARL, { art: 'erledigt', option: a.uuid, erledigt: true })]);
    expect(nurCarl.options[0].done).toBe(false);
    const ben = bauen([kopf('signup'), a, stimme(BEN, a), e(BEN, { art: 'erledigt', option: a.uuid, erledigt: true })]);
    expect(ben.options[0].done).toBe(true);
  });
});
