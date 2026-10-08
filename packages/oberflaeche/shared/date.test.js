import { describe, it, expect, afterEach } from 'vitest';
import {
  getLocalDateString,
  toDate,
  formatDate,
  formatDateShort,
  formatDateTime,
  formatWeekdayTime,
  formatDateRangeShort,
  formatDayRangeShort,
} from '@oberflaeche/shared/date';
import { i18n } from '@oberflaeche/i18n';

const sprache = (l) => { i18n.global.locale.value = l; };

describe('getLocalDateString', () => {
  it('formatiert als YYYY-MM-DD mit führenden Nullen', () => {
    expect(getLocalDateString(new Date(2026, 0, 5))).toBe('2026-01-05');
    expect(getLocalDateString(new Date(2026, 11, 31))).toBe('2026-12-31');
  });

  it('nutzt lokale Kalender-Komponenten (kein UTC-Shift wie toISOString)', () => {
    // Lokale Mitternacht muss denselben Tag ergeben, egal in welcher Zone
    // der Test läuft (toISOString würde bei negativem Offset verschieben).
    const d = new Date(2026, 6, 1, 0, 0, 0);
    expect(getLocalDateString(d)).toBe('2026-07-01');
  });
});

describe('toDate', () => {
  it('liest reine Datumsangaben als LOKALE Mitternacht, nicht als UTC', () => {
    // new Date('2026-08-01') wäre UTC-Mitternacht und zeigte westlich von
    // Greenwich den 31.07. – genau der Grund für das angehängte T00:00:00.
    const d = toDate('2026-08-01');
    expect(d.getFullYear()).toBe(2026);
    expect(d.getMonth()).toBe(7);
    expect(d.getDate()).toBe(1);
  });

  it('lässt vollständige Zeitstempel unangetastet', () => {
    expect(toDate('2026-08-01T14:30:00Z').toISOString()).toBe('2026-08-01T14:30:00.000Z');
  });

  it('gibt für Leeres und Unlesbares null zurück', () => {
    expect(toDate(null)).toBeNull();
    expect(toDate('')).toBeNull();
    expect(toDate(undefined)).toBeNull();
    expect(toDate('kein Datum')).toBeNull();
  });

  it('reicht ein Date unverändert durch', () => {
    const d = new Date(2026, 0, 1);
    expect(toDate(d)).toBe(d);
  });
});

describe('Anzeige-Formate', () => {
  afterEach(() => sprache('de'));

  it('folgt der eingestellten Sprache', () => {
    const wert = '2026-08-01T14:30:00';

    sprache('de');
    const deutsch = formatDate(wert);
    sprache('en');
    const englisch = formatDate(wert);

    // 01.08.2026 gegenüber 08/01/2026 – Tag und Monat tauschen die Plätze.
    expect(deutsch).not.toBe(englisch);
    expect(deutsch).toContain('01.08.2026');
    expect(englisch).toContain('08/01/2026');
  });

  it('zeigt für Leeres den Platzhalter, überschreibbar', () => {
    expect(formatDate(null)).toBe('–');
    expect(formatDate('')).toBe('–');
    expect(formatDate(null, '')).toBe('');
  });

  it('unterscheidet die Stile', () => {
    sprache('de');
    const wert = '2026-08-01T14:30:00';

    expect(formatDate(wert)).not.toContain('14:30');
    expect(formatDateTime(wert)).toContain('14:30');
    // Kurzform nennt den Monat als Wort statt als Zahl.
    expect(formatDateShort(wert)).toMatch(/Aug/);
    // Terminvorschläge führen den Wochentag und lassen das Jahr weg.
    expect(formatWeekdayTime(wert)).not.toContain('2026');
  });
});

describe('formatDayRangeShort', () => {
  it('lässt das Jahr weg, formatDateRangeShort nennt es', () => {
    sprache('de');
    const von = new Date(2026, 8, 1);
    const bis = new Date(2026, 8, 7);

    // Der Kalenderkopf auf dem Telefon: vier Bedienelemente in einer Zeile,
    // und die Jahreszahl ist das Erste, was gehen kann.
    expect(formatDayRangeShort(von, bis)).not.toContain('2026');
    expect(formatDayRangeShort(von, bis)).toMatch(/Sept/);
    expect(formatDateRangeShort(von, bis)).toContain('2026');
  });

  it('nennt beide Monate, wenn der Zeitraum über den Monatswechsel geht', () => {
    sprache('de');
    const wert = formatDayRangeShort(new Date(2026, 8, 28), new Date(2026, 9, 4));

    expect(wert).toMatch(/Sept/);
    expect(wert).toMatch(/Okt/);
  });

  it('zeigt für Leeres den Platzhalter', () => {
    expect(formatDayRangeShort(null, new Date(2026, 8, 7))).toBe('–');
    expect(formatDayRangeShort(new Date(2026, 8, 1), '')).toBe('–');
  });
});
