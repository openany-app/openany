import { describe, it, expect } from 'vitest';
import { ref } from 'vue';
import { useCalendarWeek } from '@oberflaeche/composables/useCalendarWeek';

// Mittwoch, 15. Juli 2026 – die Woche geht vom 13. bis zum 19.
const MITTWOCH = new Date(2026, 6, 15);

// Zeitgebundener Termin. Lokale Zeitangabe ohne Zone: Browser und Ansicht
// lesen sie beide lokal, das Ergebnis ist also zeitzonenrobust.
const mkEvent = ({ id, tag = '2026-07-15', von = '09:00', bis = '10:00', calendar_id = 10, ...rest }) => ({
  id,
  calendar_id,
  title: `Termin ${id}`,
  color: '#123456',
  is_all_day: false,
  rrule: 'NONE',
  start_date: `${tag}T${von}:00`,
  end_date: `${tag}T${bis}:00`,
  exdates: null,
  rrule_until: null,
  ...rest,
});

// Overlay-Eintrag, wie ihn /api/calendar/overlay liefert: Wanduhrzeit als
// Zeichenkette, bereits auf einen Tag aufgefaltet.
const mkOverlay = ({ id, date = '2026-07-15', starts_at = '08:00', ends_at = '08:45', source = '1:timetable', ...rest }) => ({
  source,
  project_id: 1,
  date,
  starts_at,
  ends_at,
  is_all_day: false,
  title: `Stunde ${id}`,
  subtitle: null,
  color: '#ec4899',
  place: null,
  link: { kind: 'weekplan_slot', id },
  ...rest,
});

const setup = ({ events = [], calendars = [10], overlay = [], quellen = ['1:timetable'], focused = MITTWOCH } = {}) =>
  useCalendarWeek(
    ref(focused),
    ref(events),
    ref(new Set(calendars)),
    ref(overlay),
    ref(new Set(quellen)),
  );

describe('useCalendarWeek – Wochenraster', () => {
  it('liefert sieben Tage ab Montag, die den fokussierten Tag enthalten', () => {
    const { weekDays } = setup();
    const tage = weekDays.value;

    expect(tage).toHaveLength(7);
    expect(tage[0].date.getDay()).toBe(1); // Montag
    expect(tage[0].dateString).toBe('2026-07-13');
    expect(tage[6].dateString).toBe('2026-07-19');
  });

  it('beginnt auch dann montags, wenn der fokussierte Tag ein Sonntag ist', () => {
    const { weekDays } = setup({ focused: new Date(2026, 6, 19) });
    expect(weekDays.value[0].dateString).toBe('2026-07-13');
  });
});

describe('useCalendarWeek – Stundenachse', () => {
  it('traegt den ganzen Tag, unabhaengig davon, was in der Woche liegt', () => {
    const { hourRange, hours } = setup({ events: [mkEvent({ id: 1 })] });
    expect(hourRange.value).toEqual({ start: 0, end: 24 });
    expect(hours.value[0]).toBe(0);
    expect(hours.value.at(-1)).toBe(23);
    expect(hours.value).toHaveLength(24);
  });

  // Bis zum 08.09.2026 zog sich die Achse auf, wenn ein Eintrag ausserhalb
  // von 7–19 lag. Das Aufziehen ist weg, weil es nichts mehr aufzuziehen
  // gibt -- aber die Sorge dahinter bleibt gueltig: Der Fruehdienst um halb
  // sechs und die Abholung um halb zehn abends gehoeren ins Raster.
  it('verschluckt weder den fruehen Morgen noch den spaeten Abend', () => {
    const { hourRange, timedMap } = setup({
      events: [mkEvent({ id: 1, von: '05:30', bis: '06:15' })],
      overlay: [mkOverlay({ id: 2, starts_at: '21:00', ends_at: '21:30' })],
    });
    expect(hourRange.value).toEqual({ start: 0, end: 24 });

    const [frueh, spaet] = timedMap.value.get('2026-07-15');
    expect(frueh.top).toBeCloseTo((5.5 * 60 / 1440) * 100);
    expect(spaet.top).toBeCloseTo((21 * 60 / 1440) * 100);
    // Beide liegen im Raster, nicht darueber oder darunter hinaus.
    for (const e of [frueh, spaet]) expect(e.top + e.height).toBeLessThanOrEqual(100);
  });
});

describe('useCalendarWeek – Belegung', () => {
  it('legt einen Termin auf seinen Tag und rechnet Lage und Höhe aus', () => {
    const { timedMap } = setup({ events: [mkEvent({ id: 1, von: '10:00', bis: '11:00' })] });
    const [platziert] = timedMap.value.get('2026-07-15');

    // Die Achse traegt den ganzen Tag = 1440 Minuten; 10:00 liegt 600
    // Minuten hinter Mitternacht.
    expect(platziert.top).toBeCloseTo((600 / 1440) * 100);
    expect(platziert.height).toBeCloseTo((60 / 1440) * 100);
    expect(platziert.width).toBe(100);
    expect(timedMap.value.get('2026-07-14')).toHaveLength(0);
  });

  it('blendet Termine unsichtbarer Kalender aus', () => {
    const { timedMap } = setup({ events: [mkEvent({ id: 1 })], calendars: [] });
    expect(timedMap.value.get('2026-07-15')).toHaveLength(0);
  });

  it('blendet Overlay-Einträge ausgeschalteter Quellen aus', () => {
    const { timedMap } = setup({ overlay: [mkOverlay({ id: 1 })], quellen: [] });
    expect(timedMap.value.get('2026-07-15')).toHaveLength(0);
  });

  it('übernimmt die Wanduhrzeit eines Overlay-Eintrags unverändert', () => {
    const { timedMap } = setup({ overlay: [mkOverlay({ id: 1, starts_at: '08:00', ends_at: '08:45' })] });
    const [stunde] = timedMap.value.get('2026-07-15');

    expect(stunde.kind).toBe('overlay');
    expect(stunde.vonMinute).toBe(8 * 60);
    expect(stunde.bisMinute).toBe(8 * 60 + 45);
    expect(stunde.top).toBeCloseTo((8 * 60 / 1440) * 100);
  });

  it('gibt einem Overlay-Eintrag ohne Ende eine Mindestdauer', () => {
    const { timedMap } = setup({ overlay: [mkOverlay({ id: 1, starts_at: '08:00', ends_at: null })] });
    const [stunde] = timedMap.value.get('2026-07-15');
    expect(stunde.bisMinute).toBe(8 * 60 + 30);
  });

  it('stellt Termin und Schulstunde derselben Zeit nebeneinander', () => {
    const { timedMap } = setup({
      events: [mkEvent({ id: 1, von: '08:00', bis: '09:00' })],
      overlay: [mkOverlay({ id: 2, starts_at: '08:00', ends_at: '08:45' })],
    });
    const platziert = timedMap.value.get('2026-07-15');

    expect(platziert).toHaveLength(2);
    expect(platziert.every((e) => e.width === 50)).toBe(true);
    expect(platziert.map((e) => e.left).sort()).toEqual([0, 50]);
  });

  it('teilt die Breite bei drei Überlappenden zu Dritteln', () => {
    const { timedMap } = setup({
      events: [
        mkEvent({ id: 1, von: '09:00', bis: '10:30' }),
        mkEvent({ id: 2, von: '09:15', bis: '10:00' }),
        mkEvent({ id: 3, von: '09:30', bis: '11:00' }),
      ],
    });
    const platziert = timedMap.value.get('2026-07-15');

    expect(platziert).toHaveLength(3);
    expect(platziert.every((e) => Math.round(e.width) === 33)).toBe(true);
    expect(new Set(platziert.map((e) => e.spalte)).size).toBe(3);
  });

  it('lässt einen Eintrag nach dem Ende eines anderen wieder die volle Breite nehmen', () => {
    const { timedMap } = setup({
      events: [
        mkEvent({ id: 1, von: '08:00', bis: '09:00' }),
        mkEvent({ id: 2, von: '09:00', bis: '10:00' }),
      ],
    });
    expect(timedMap.value.get('2026-07-15').every((e) => e.width === 100)).toBe(true);
  });
});

describe('useCalendarWeek – Band über der Achse', () => {
  it('legt ganztägige Termine ins Band, nicht in die Stundenachse', () => {
    const { allDayMap, timedMap } = setup({
      events: [mkEvent({ id: 1, is_all_day: true, von: '00:00', bis: '23:59' })],
    });

    expect(allDayMap.value.get('2026-07-15')).toHaveLength(1);
    expect(timedMap.value.get('2026-07-15')).toHaveLength(0);
  });

  it('führt einen mehrtägigen Termin an jedem berührten Tag im Band', () => {
    const { allDayMap } = setup({
      events: [mkEvent({ id: 1, start_date: '2026-07-14T09:00:00', end_date: '2026-07-16T17:00:00' })],
    });

    expect(allDayMap.value.get('2026-07-13')).toHaveLength(0);
    expect(allDayMap.value.get('2026-07-14')).toHaveLength(1);
    expect(allDayMap.value.get('2026-07-15')).toHaveLength(1);
    expect(allDayMap.value.get('2026-07-16')).toHaveLength(1);
    expect(allDayMap.value.get('2026-07-17')).toHaveLength(0);
  });

  it('führt einen Termin über Mitternacht im Band beider Tage', () => {
    // Über Mitternacht heißt zwei Tage. In der Stundenachse müsste der Balken
    // an jedem Tag neu beschnitten werden – im Band steht er ganz.
    const { allDayMap, timedMap } = setup({
      events: [mkEvent({ id: 1, start_date: '2026-07-15T22:00:00', end_date: '2026-07-16T02:00:00' })],
    });

    expect(allDayMap.value.get('2026-07-15')).toHaveLength(1);
    expect(allDayMap.value.get('2026-07-16')).toHaveLength(1);
    expect(timedMap.value.get('2026-07-15')).toHaveLength(0);
  });

  it('legt eine Fälligkeit aus dem Overlay ins Band', () => {
    const { allDayMap } = setup({
      overlay: [mkOverlay({ id: 1, source: '1:due', is_all_day: true, starts_at: null, ends_at: null })],
      quellen: ['1:due'],
    });
    expect(allDayMap.value.get('2026-07-15')).toHaveLength(1);
  });

  it('faltet wiederkehrende Termine in die Woche auf', () => {
    const { timedMap } = setup({
      events: [mkEvent({ id: 1, tag: '2026-07-01', rrule: 'WEEKLY' })],
    });
    // 1.7. war ein Mittwoch – die Serie trifft den 15.7.
    expect(timedMap.value.get('2026-07-15')).toHaveLength(1);
    expect(timedMap.value.get('2026-07-16')).toHaveLength(0);
  });
});
