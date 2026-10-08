import { computed } from 'vue';

// Ordnet die Antwort von /api/calendar/day zu dem an, was die Tagesansicht
// zeichnet: je Projekt ein Block mit der Betreuungszeile, den Stunden in
// ihrer Reihenfolge und darunter dem, was an keine Stunde passte.
//
// Was hier NICHT passiert: das Zuordnen selbst. Welche Hausaufgabe an welcher
// Stunde hängt und welche Klassenarbeit als Vorausblick erscheint, entscheidet
// der Server (`CalendarDayController`) – er hat das Fach am Slot, die
// Freigabe-Prüfung der Mappe und dieselbe Auffaltung, die auch den ICS-Feed
// speist. Diese Regeln ein zweites Mal in JavaScript zu führen wäre genau die
// Doppelung, die der Plan an anderer Stelle vermeidet.
//
// Die REIHENFOLGE der Stunden kommt ebenfalls vom Server und wird hier nicht
// noch einmal sortiert: Sie ist dort keine Zierde, sondern trägt eine Regel
// („die Hausaufgabe hängt an der ERSTEN Stunde ihres Fachs"). Zwei
// Sortierungen könnten auseinanderlaufen, und dann hinge die Karte sichtbar
// an der falschen Zeile.

// Innerhalb einer Stunde zuerst, was heute fällig ist, dann der Vorausblick
// nach Nähe. „Heute abzugeben" und „in zwölf Tagen" sind zwei verschiedene
// Dringlichkeiten; die nähere gehört nach oben.
const nachDringlichkeit = (a, b) => (a.is_preview === b.is_preview
  ? (a.days_left ?? 0) - (b.days_left ?? 0)
  : (a.is_preview ? 1 : -1));

/**
 * @param daten ref<Object|null> – die rohe Antwort von /api/calendar/day
 * @param meId  ref<number|null> – der angemeldete Mensch (für „bei dir")
 */
export function useCalendarDay(daten, meId) {
  const bloecke = computed(() => (daten.value?.projects ?? []).map((p) => ({
    projectId: p.project_id,
    projectName: p.project_name,

    // Die erste Frage des Abends ist nicht, was ansteht, sondern ob das Kind
    // morgen überhaupt da ist. Deshalb steht sie im Kopf und nicht in der
    // Liste – und deshalb gehört diese Ansicht in den Kalender: Nur dort
    // liegen Stundenplan und Betreuung übereinander.
    betreuung: (p.care ?? []).map((c) => ({
      beiMir: c.assigned_to != null && c.assigned_to === meId.value,
      name: c.assigned_name,
      title: c.title,
      color: c.color,
      starts_at: c.starts_at,
      ends_at: c.ends_at,
    })),

    stunden: (p.lessons ?? []).map((s) => ({
      ...s,
      items: [...(s.items ?? [])].sort(nachDringlichkeit),
    })),

    // Kein passender Slot heißt nicht „verschwunden". Dieselbe Stelle für
    // beide Fälle: Karte ohne Fach und Karte, deren Fach heute keine Stunde
    // hat.
    ungebunden: [...(p.unbound ?? [])].sort(nachDringlichkeit),

    homeworkTarget: p.homework_target ?? null,
  })));

  const istLeer = computed(() => bloecke.value.length === 0);

  return { bloecke, istLeer };
}
