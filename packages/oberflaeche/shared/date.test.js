import { describe, it, expect, afterEach } from 'vitest';
import {
  getLocalDateString,
  isoWoche,
  toDate,
  formatDate,
  formatDateShort,
  formatDateTime,
  formatDateRangeShort,
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
  });
});

describe('formatDateRangeShort', () => {
  it('zieht den Zeitraum zusammen und nennt das Jahr', () => {
    sprache('de');
    const wert = formatDateRangeShort(new Date(2026, 8, 1), new Date(2026, 8, 7));
    expect(wert).toContain('2026');
    expect(wert).toMatch(/Sept/);
  });
});

describe('isoWoche', () => {
  it('zählt nach ISO 8601', () => {
    expect(isoWoche(new Date(2026, 9, 26))).toBe(44); // Mo 26.10.2026
    expect(isoWoche(new Date(2026, 10, 1))).toBe(44); // So 01.11.2026
    expect(isoWoche(new Date(2027, 0, 1))).toBe(53); // Fr 01.01.2027 gehört zur letzten Woche 2026
    expect(isoWoche(new Date(2025, 11, 29))).toBe(1); // Mo 29.12.2025 = KW 1/2026
  });
});
