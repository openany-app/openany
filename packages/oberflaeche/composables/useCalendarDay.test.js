import { describe, it, expect } from 'vitest';
import { ref } from 'vue';
import { useCalendarDay } from '@oberflaeche/composables/useCalendarDay';

const mkStunde = ({ title, starts_at = '08:00', subject_id = null, items = [] }) => ({
  date: '2026-08-10',
  starts_at,
  ends_at: '08:45',
  is_all_day: false,
  subject_id,
  title,
  subtitle: null,
  color: null,
  place: null,
  subject: subject_id ? { id: subject_id, name: title, note_folder: null } : null,
  link: { kind: 'slot', id: 1 },
  items,
});

const mkKarte = ({ id, title, subject_id = null }) => ({
  kind: 'card', id, title, subject_id, due_date: '2026-08-10', days_left: 0, is_preview: false,
  link: { kind: 'card', id, board_id: 1 },
});

const mkArbeit = ({ id, title, subject_id = null, days_left = 5 }) => ({
  kind: 'milestone', id, title, subject_id, due_date: '2026-08-15', days_left, is_preview: days_left > 0,
  link: { kind: 'milestone', id, roadmap_id: 1 },
});

const setup = (projects, me = 7) => useCalendarDay(ref({ date: '2026-08-10', projects }), ref(me));

const mkProjekt = (extra = {}) => ({
  project_id: 1,
  project_name: 'Schule – Lena',
  care: [],
  lessons: [],
  unbound: [],
  homework_target: { board_id: 1, column_id: 2 },
  ...extra,
});

describe('useCalendarDay – Stunden', () => {
  it('behält die Reihenfolge des Servers bei', () => {
    // Nicht neu sortiert: Die Reihenfolge trägt dort eine Regel („die
    // Hausaufgabe hängt an der ERSTEN Stunde ihres Fachs").
    const { bloecke } = setup([mkProjekt({
      lessons: [
        mkStunde({ title: 'Deutsch', starts_at: '08:00' }),
        mkStunde({ title: 'Mathe', starts_at: '08:50' }),
        mkStunde({ title: 'Sport', starts_at: '09:45' }),
      ],
    })]);

    expect(bloecke.value[0].stunden.map((s) => s.title)).toEqual(['Deutsch', 'Mathe', 'Sport']);
  });

  it('hängt einen Einzelpunkt an die Stunde seines Fachs', () => {
    const { bloecke } = setup([mkProjekt({
      lessons: [
        mkStunde({ title: 'Deutsch', subject_id: 1 }),
        mkStunde({ title: 'Mathe', starts_at: '08:50', subject_id: 2, items: [mkKarte({ id: 9, title: 'S. 42 Nr. 3–7', subject_id: 2 })] }),
      ],
    })]);

    expect(bloecke.value[0].stunden[0].items).toHaveLength(0);
    expect(bloecke.value[0].stunden[1].items[0].title).toBe('S. 42 Nr. 3–7');
  });

  it('stellt in einer Stunde das heute Fällige vor den Vorausblick', () => {
    const { bloecke } = setup([mkProjekt({
      lessons: [mkStunde({
        title: 'Mathe',
        subject_id: 2,
        items: [
          mkArbeit({ id: 3, title: 'Test in zwölf Tagen', subject_id: 2, days_left: 12 }),
          mkArbeit({ id: 2, title: 'Test in fünf Tagen', subject_id: 2, days_left: 5 }),
          mkKarte({ id: 1, title: 'Heute abzugeben', subject_id: 2 }),
        ],
      })],
    })]);

    expect(bloecke.value[0].stunden[0].items.map((i) => i.title))
      .toEqual(['Heute abzugeben', 'Test in fünf Tagen', 'Test in zwölf Tagen']);
  });
});

describe('useCalendarDay – was an keine Stunde passt', () => {
  it('steht unten statt nirgends', () => {
    const { bloecke } = setup([mkProjekt({
      lessons: [mkStunde({ title: 'Mathe', subject_id: 2 })],
      unbound: [mkKarte({ id: 5, title: 'Sportsachen einpacken' })],
    })]);

    expect(bloecke.value[0].stunden[0].items).toHaveLength(0);
    expect(bloecke.value[0].ungebunden.map((i) => i.title)).toEqual(['Sportsachen einpacken']);
  });

  it('kommt auch ohne eine einzige Stunde durch', () => {
    const { bloecke, istLeer } = setup([mkProjekt({
      unbound: [mkKarte({ id: 5, title: 'Sportsachen einpacken' })],
    })]);

    expect(istLeer.value).toBe(false);
    expect(bloecke.value[0].ungebunden).toHaveLength(1);
  });
});

describe('useCalendarDay – die Betreuungszeile', () => {
  it('erkennt, dass das Kind bei mir ist', () => {
    const { bloecke } = setup([mkProjekt({
      care: [{ assigned_to: 7, assigned_name: 'Mama', title: 'bei Mama', color: '#6366f1', starts_at: null, ends_at: null }],
    })], 7);

    expect(bloecke.value[0].betreuung[0].beiMir).toBe(true);
  });

  it('nennt sonst den anderen Menschen beim Namen', () => {
    const { bloecke } = setup([mkProjekt({
      care: [{ assigned_to: 8, assigned_name: 'Papa', title: 'bei Papa', color: null, starts_at: null, ends_at: null }],
    })], 7);

    expect(bloecke.value[0].betreuung[0].beiMir).toBe(false);
    expect(bloecke.value[0].betreuung[0].name).toBe('Papa');
  });

  it('bleibt still, wenn es keinen Betreuungsplan gibt', () => {
    const { bloecke } = setup([mkProjekt({ lessons: [mkStunde({ title: 'Mathe' })] })]);
    expect(bloecke.value[0].betreuung).toEqual([]);
  });
});

describe('useCalendarDay – leerer Tag', () => {
  it('meldet einen Tag ohne Projekte als leer', () => {
    const { istLeer } = setup([]);
    expect(istLeer.value).toBe(true);
  });

  it('kommt mit einer noch nicht geladenen Antwort zurecht', () => {
    const { bloecke, istLeer } = useCalendarDay(ref(null), ref(1));
    expect(bloecke.value).toEqual([]);
    expect(istLeer.value).toBe(true);
  });
});
