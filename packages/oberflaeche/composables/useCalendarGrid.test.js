import { describe, it, expect } from 'vitest';
import { ref } from 'vue';
import { useCalendarGrid } from '@oberflaeche/composables/useCalendarGrid';

// Ganztägiger Termin für Juli 2026 (lokale Mitternacht → zeitzonenrobust,
// weil Instanz-Tag und Zell-Tag beide über getLocalDateString laufen).
const mkEvent = ({ id, start, end, rrule = 'NONE', calendar_id = 10 }) => ({
  id,
  calendar_id,
  color: '#123456',
  is_all_day: true,
  rrule,
  start_date: `${start}T00:00:00`,
  end_date: `${end}T23:59:59`,
  exdates: null,
  rrule_until: null,
});

// Liegt an dateStr ein Slot mit dieser (Original-)Event-Id?
const eventAt = (map, dateStr, realId) =>
  (map.get(dateStr) || []).some((e) => e && (e.real_id ?? e.id) === realId);

const setup = (events, visibleIds, focused = new Date(2026, 6, 15)) =>
  useCalendarGrid(ref(focused), ref(events), ref(new Set(visibleIds)));

describe('useCalendarGrid – Raster', () => {
  it('füllt volle Wochen, beginnt montags, deckt den ganzen Monat ab', () => {
    const { calendarGrid } = setup([], []);
    const grid = calendarGrid.value;

    expect(grid.length % 7).toBe(0);
    expect(grid[0].date.getDay()).toBe(1); // Montag
    expect(grid.filter((c) => c.isCurrentMonth).length).toBe(31); // Juli
  });
});

describe('useCalendarGrid – Belegung', () => {
  it('legt einen Einzeltermin in die Zelle seines Tages', () => {
    const { gridEventsMap } = setup([mkEvent({ id: 1, start: '2026-07-15', end: '2026-07-15' })], [10]);
    expect(eventAt(gridEventsMap.value, '2026-07-15', 1)).toBe(true);
  });

  it('blendet Termine unsichtbarer Kalender aus', () => {
    const { gridEventsMap } = setup([mkEvent({ id: 1, start: '2026-07-15', end: '2026-07-15' })], []);
    const slots = gridEventsMap.value.get('2026-07-15') || [];
    expect(slots.every((e) => !e)).toBe(true);
  });

  it('belegt bei mehrtägigen Terminen jede berührte Zelle, aber nicht danach', () => {
    const { gridEventsMap } = setup([mkEvent({ id: 3, start: '2026-07-14', end: '2026-07-16' })], [10]);
    const map = gridEventsMap.value;
    expect(eventAt(map, '2026-07-14', 3)).toBe(true);
    expect(eventAt(map, '2026-07-15', 3)).toBe(true);
    expect(eventAt(map, '2026-07-16', 3)).toBe(true);
    expect(eventAt(map, '2026-07-17', 3)).toBe(false);
  });

  it('faltet wöchentliche Termine in mehrere Instanzen auf', () => {
    const { gridEventsMap } = setup([mkEvent({ id: 2, start: '2026-07-01', end: '2026-07-01', rrule: 'WEEKLY' })], [10]);
    const map = gridEventsMap.value;
    expect(eventAt(map, '2026-07-01', 2)).toBe(true);
    expect(eventAt(map, '2026-07-08', 2)).toBe(true);
    expect(eventAt(map, '2026-07-15', 2)).toBe(true);
    // Dazwischen (kein Wochentag der Serie) nicht.
    expect(eventAt(map, '2026-07-02', 2)).toBe(false);
  });

  it('packt überlappende Termine in getrennte Zeilen (Slots)', () => {
    const { gridEventsMap } = setup([
      mkEvent({ id: 4, start: '2026-07-14', end: '2026-07-14' }),
      mkEvent({ id: 5, start: '2026-07-14', end: '2026-07-14' }),
    ], [10]);
    const slots = gridEventsMap.value.get('2026-07-14');
    const ids = slots.filter(Boolean).map((e) => e.id).sort();
    expect(ids).toEqual([4, 5]);
    // Zwei Termine → zwei belegte Slots (unterschiedliche Zeilen).
    expect(slots.filter(Boolean).length).toBe(2);
  });
});
