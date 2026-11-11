import { regelTeile } from '@oberflaeche/shared/expandEvents';
import { formatDate } from '@oberflaeche/shared/date';

// Eine Wiederholung in Worten: „Alle 2 Wochen am Di, Do, 6 Mal“. Wochentage
// und Monate nennt der Browser in der Sprache der Oberfläche (Intl).

const TAGE = ['SU', 'MO', 'TU', 'WE', 'TH', 'FR', 'SA'];
const FREQ = { DAILY: 'daily', WEEKLY: 'weekly', MONTHLY: 'monthly', YEARLY: 'yearly' };

function tagName(locale, tag) {
  // Der 1.11.2026 ist ein Sonntag.
  return new Intl.DateTimeFormat(locale, { weekday: 'short' }).format(new Date(2026, 10, 1 + tag));
}

function monatName(locale, monat) {
  return new Intl.DateTimeFormat(locale, { month: 'long' }).format(new Date(2026, monat - 1, 1));
}

function ordnung(t, n) {
  if (n === -1) return t('calendar.recurrence.last');
  if (n === -2) return t('calendar.recurrence.secondLast');
  if (n >= 1 && n <= 5) return t(`calendar.recurrence.nth${n}`);
  return String(n);
}

/** `null`, wenn der Termin sich nicht wiederholt. */
export function regelText(ev, t, locale) {
  const freq = FREQ[ev?.rrule];
  if (!freq) return null;
  const teile = regelTeile(ev.rrule_parts);
  const n = Math.max(1, parseInt(teile.INTERVAL, 10) || 1);
  let text = t(`calendar.recurrence.${freq}`, { n }, n);

  const tage = (teile.BYDAY || '').split(',').filter(Boolean).map((w) => {
    const m = /^([+-]?\d{1,2})?([A-Z]{2})$/.exec(w);
    if (!m || TAGE.indexOf(m[2]) < 0) return w;
    const name = tagName(locale, TAGE.indexOf(m[2]));
    return m[1] ? t('calendar.recurrence.nthDay', { nth: ordnung(t, parseInt(m[1], 10)), tag: name }) : name;
  });
  if (tage.length) text += ' ' + t('calendar.recurrence.onDays', { tage: tage.join(', ') });

  const monatstage = (teile.BYMONTHDAY || '').split(',').filter(Boolean);
  if (monatstage.length) {
    const liste = monatstage.map((d) => (d.startsWith('-') ? ordnung(t, parseInt(d, 10)) : t('calendar.recurrence.monthDay', { d })));
    text += ' ' + t('calendar.recurrence.onMonthDays', { tage: liste.join(', ') });
  }

  const monate = (teile.BYMONTH || '').split(',').filter(Boolean).map((m) => monatName(locale, parseInt(m, 10)));
  if (monate.length) text += ' ' + t('calendar.recurrence.inMonths', { monate: monate.join(', ') });

  const pos = (teile.BYSETPOS || '').split(',').filter(Boolean);
  if (pos.length) text += ', ' + t('calendar.recurrence.setPos', { pos: pos.map((p) => ordnung(t, parseInt(p, 10))).join(', ') });

  const anzahl = parseInt(teile.COUNT, 10);
  if (anzahl) text += ', ' + t('calendar.recurrence.times', { n: anzahl }, anzahl);
  else if (ev.rrule_until) text += ', ' + t('calendar.recurrence.until', { datum: formatDate(ev.rrule_until) });
  return text;
}
