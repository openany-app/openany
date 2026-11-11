import { describe, it, expect } from 'vitest';
import { createI18n } from 'vue-i18n';
import calendar from '../i18n/locales/de/calendar.js';
import { regelText } from './regelText';

const { t } = createI18n({ legacy: false, locale: 'de', messages: { de: { calendar } } }).global;
// Wie der Browser die Tage abkürzt (mit oder ohne Punkt), hängt an seiner
// Ausgabe von Intl.
const tag = (n) => new Intl.DateTimeFormat('de', { weekday: 'short' }).format(new Date(2026, 10, 1 + n));
const text = (ev) => regelText({ rrule_until: null, rrule_parts: '', ...ev }, t, 'de');

describe('regelText', () => {
  it('nennt Abstand, Tage und Anzahl', () => {
    expect(text({ rrule: 'WEEKLY', rrule_parts: 'COUNT=3' })).toBe('Wöchentlich, 3 Mal');
    expect(text({ rrule: 'WEEKLY', rrule_parts: 'INTERVAL=2;BYDAY=TU,TH' })).toBe(`Alle 2 Wochen am ${tag(2)}, ${tag(4)}`);
    expect(text({ rrule: 'MONTHLY', rrule_parts: 'BYDAY=1MO,-1FR' })).toBe(`Monatlich am 1. ${tag(1)}, letzten ${tag(5)}`);
    expect(text({ rrule: 'MONTHLY', rrule_parts: 'BYMONTHDAY=15' })).toBe('Monatlich am 15.');
    expect(text({ rrule: 'YEARLY', rrule_parts: 'BYDAY=2SU;BYMONTH=5' })).toBe(`Jährlich am 2. ${tag(0)} im Mai`);
  });

  it('nichts, wenn sich nichts wiederholt', () => {
    expect(text({ rrule: 'NONE' })).toBeNull();
  });
});
