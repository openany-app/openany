import { describe, it, expect } from 'vitest';
import { expandRecurringEvents, vorkommen, kompakt } from './expandEvents';

const serie = (felder) => ({
  id: 's',
  start_date: '2026-11-03T18:00:00',
  end_date: '2026-11-03T19:00:00',
  is_all_day: false,
  rrule: 'WEEKLY',
  rrule_until: null,
  rrule_parts: '',
  exdates: '',
  ...felder,
});

const tage = (ev, bis = '2027-12-31') => vorkommen(ev, bis).map((d) => kompakt(d).split('T')[0]);

describe('vorkommen', () => {
  it('wöchentlich, 3 Mal', () => {
    expect(tage(serie({ rrule_parts: 'COUNT=3' }))).toEqual(['20261103', '20261110', '20261117']);
  });

  it('alle 2 Wochen dienstags und donnerstags, 5 Mal', () => {
    // Der 3.11.2026 ist ein Dienstag.
    expect(tage(serie({ rrule_parts: 'INTERVAL=2;COUNT=5;BYDAY=TU,TH' }))).toEqual([
      '20261103', '20261105', '20261117', '20261119', '20261201',
    ]);
  });

  it('jeden ersten Montag und letzten Freitag im Monat', () => {
    const ev = serie({ rrule: 'MONTHLY', start_date: '2026-11-02T09:00:00', end_date: '2026-11-02T10:00:00', rrule_parts: 'BYDAY=1MO,-1FR' });
    expect(tage(ev, '2027-01-31')).toEqual(['20261102', '20261127', '20261207', '20261225', '20270104', '20270129']);
  });

  it('am 31. nur in Monaten, die einen haben', () => {
    const ev = serie({ rrule: 'MONTHLY', start_date: '2027-01-31T09:00:00', end_date: '2027-01-31T10:00:00' });
    expect(tage(ev, '2027-05-31')).toEqual(['20270131', '20270331', '20270531']);
  });

  it('der letzte Werktag im Monat (BYSETPOS)', () => {
    const ev = serie({ rrule: 'MONTHLY', start_date: '2026-11-30T09:00:00', end_date: '2026-11-30T10:00:00', rrule_parts: 'BYDAY=MO,TU,WE,TH,FR;BYSETPOS=-1' });
    expect(tage(ev, '2027-01-31')).toEqual(['20261130', '20261231', '20270129']);
  });

  it('jährlich am zweiten Sonntag im Mai', () => {
    const ev = serie({ rrule: 'YEARLY', start_date: '2027-05-09T10:00:00', end_date: '2027-05-09T11:00:00', rrule_parts: 'BYDAY=2SU;BYMONTH=5' });
    expect(tage(ev, '2029-12-31')).toEqual(['20270509', '20280514', '20290513']);
  });

  it('werktags, bis zu einem Datum', () => {
    const ev = serie({ rrule: 'DAILY', start_date: '2026-11-05T08:00:00', end_date: '2026-11-05T09:00:00', rrule_parts: 'BYDAY=MO,TU,WE,TH,FR', rrule_until: '2026-11-10T00:00:00' });
    expect(tage(ev)).toEqual(['20261105', '20261106', '20261109', '20261110']);
  });

  it('ohne Regelrest wie bisher', () => {
    expect(tage(serie({}), '2026-11-24')).toEqual(['20261103', '20261110', '20261117', '20261124']);
  });
});

describe('expandRecurringEvents', () => {
  it('lässt Ausnahmen und verschobene Vorkommen aus', () => {
    const s = serie({ rrule_parts: 'COUNT=4', exdates: '20261110T180000' });
    const verschoben = {
      id: 'v',
      start_date: '2026-11-18T20:00:00',
      end_date: '2026-11-18T21:00:00',
      rrule: 'NONE',
      series: 's',
      occurrence: '20261117T180000',
    };
    const aus = expandRecurringEvents([s, verschoben]);
    const beginne = aus.map((e) => kompakt(new Date(e.start_date)));
    // COUNT zählt die Ausnahmen mit (RFC 5545): 3., 10., 17., 24. → 3. und 24.
    expect(beginne).toEqual(['20261103T180000', '20261124T180000', '20261118T200000']);
    expect(aus[0].real_id).toBe('s');
  });

  it('eine Ausnahme in UTC trifft die Ortszeit', () => {
    const s = serie({ rrule_parts: 'COUNT=2', exdates: kompakt(new Date('2026-11-10T18:00:00')).replace(/T.*/, '') });
    expect(expandRecurringEvents([s])).toHaveLength(1);
    const utc = new Date('2026-11-10T18:00:00').toISOString().replace(/[-:]/g, '').split('.')[0] + 'Z';
    expect(expandRecurringEvents([serie({ rrule_parts: 'COUNT=2', exdates: utc })])).toHaveLength(1);
  });
});
