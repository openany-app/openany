import { computed } from 'vue';
import { getLocalDateString } from '@oberflaeche/shared/date';
import { expandRecurringEvents } from '@oberflaeche/shared/expandEvents';

// Rechnet aus dem fokussierten Monat, den (evtl. wiederkehrenden) Terminen
// und den sichtbaren Kalendern das Monatsraster samt Termin-Belegung.
// Reine Ableitung – alle Eingaben sind refs, die Ausgaben sind computed.
export function useCalendarGrid(focusedDate, events, visibleCalendarIds) {
  // Tageszellen des Rasters: führende Tage des Vormonats, der ganze Monat,
  // auffüllende Tage des Folgemonats bis zur vollen Woche (4–6 Wochen).
  const calendarGrid = computed(() => {
    const year = focusedDate.value.getFullYear();
    const month = focusedDate.value.getMonth();

    const firstDay = new Date(year, month, 1);
    const lastDay = new Date(year, month + 1, 0);

    let startDayOfWeek = firstDay.getDay() - 1; // Montag = 0
    if (startDayOfWeek === -1) startDayOfWeek = 6;

    const days = [];
    const prevMonthLastDay = new Date(year, month, 0).getDate();
    for (let i = startDayOfWeek - 1; i >= 0; i--) {
      const d = new Date(year, month - 1, prevMonthLastDay - i);
      days.push({ date: d, dateString: getLocalDateString(d), isCurrentMonth: false });
    }

    for (let i = 1; i <= lastDay.getDate(); i++) {
      const d = new Date(year, month, i);
      days.push({ date: d, dateString: getLocalDateString(d), isCurrentMonth: true });
    }

    let nextMonthDay = 1;
    while (days.length % 7 !== 0) {
      const d = new Date(year, month + 1, nextMonthDay++);
      days.push({ date: d, dateString: getLocalDateString(d), isCurrentMonth: false });
    }

    return days;
  });

  // Auffaltung wiederkehrender Termine: gemeinsame Logik in
  // shared/expandEvents.js (auch vom Kalender-Widget der Startseite genutzt).
  const expandedEvents = computed(() => expandRecurringEvents(events.value));

  // Belegung pro Tageszelle: je Woche werden überlappende Termine in feste
  // Zeilen gepackt (längste zuerst), damit mehrtägige Balken über die Tage
  // hinweg in derselben Zeile bleiben. null = leerer Platzhalter-Slot.
  const gridEventsMap = computed(() => {
    const result = new Map();
    const weekCount = calendarGrid.value.length / 7;

    for (let w = 0; w < weekCount; w++) {
      const weekCells = calendarGrid.value.slice(w * 7, (w + 1) * 7);
      const weekStartStr = weekCells[0].dateString;
      const weekEndStr = weekCells[6].dateString;

      const overlappingEvents = expandedEvents.value.filter(e => {
        if (!visibleCalendarIds.value.has(e.calendar_id)) return false;
        const startStr = getLocalDateString(new Date(e.start_date));
        const endStr = getLocalDateString(new Date(e.end_date));
        return startStr <= weekEndStr && endStr >= weekStartStr;
      }).sort((a, b) => {
        const aLen = new Date(a.end_date) - new Date(a.start_date);
        const bLen = new Date(b.end_date) - new Date(b.start_date);
        if (aLen !== bLen) return bLen - aLen;
        return a.start_date.localeCompare(b.start_date);
      });

      const rows = [];
      const eventRowAssignments = new Map();

      for (const e of overlappingEvents) {
        const startStr = getLocalDateString(new Date(e.start_date));
        const endStr = getLocalDateString(new Date(e.end_date));
        const startIdx = Math.max(0, weekCells.findIndex(c => c.dateString >= startStr));
        let lastMatchIdx = -1;
        for (let i = weekCells.length - 1; i >= 0; i--) {
          if (weekCells[i].dateString <= endStr) {
            lastMatchIdx = i;
            break;
          }
        }
        const endIdx = Math.min(6, lastMatchIdx !== -1 ? lastMatchIdx : 6);

        let rowIndex = 0;
        while (true) {
          if (!rows[rowIndex]) rows[rowIndex] = Array(7).fill(false);
          let canFit = true;
          for (let i = startIdx; i <= endIdx; i++) {
            if (rows[rowIndex][i]) { canFit = false; break; }
          }
          if (canFit) {
            for (let i = startIdx; i <= endIdx; i++) {
              rows[rowIndex][i] = true;
            }
            eventRowAssignments.set(e.id, rowIndex);
            break;
          }
          rowIndex++;
        }
      }

      for (const cell of weekCells) {
        const dayEventsArr = overlappingEvents.filter(e => {
          const startStr = getLocalDateString(new Date(e.start_date));
          const endStr = getLocalDateString(new Date(e.end_date));
          return cell.dateString >= startStr && cell.dateString <= endStr;
        });

        const slots = [];
        for (const e of dayEventsArr) {
          const r = eventRowAssignments.get(e.id);
          slots[r] = e;
        }
        for (let i = 0; i < slots.length; i++) {
          if (slots[i] === undefined) slots[i] = null;
        }
        result.set(cell.dateString, slots);
      }
    }
    return result;
  });

  return { calendarGrid, gridEventsMap };
}
