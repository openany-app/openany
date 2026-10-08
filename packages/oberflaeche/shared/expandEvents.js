import { getLocalDateString } from '@oberflaeche/shared/date';

// Wiederkehrende Termine in einzelne Instanzen auffalten (bis 2 Jahre in
// die Zukunft bzw. rrule_until, mit EXDATE-Ausnahmen). Instanzen tragen
// real_id + original_event für Bearbeiten/Löschen. Gemeinsame Basis für
// das Monatsraster (useCalendarGrid) und das Kalender-Widget der
// Startseite.
export function expandRecurringEvents(events) {
  const expanded = [];
  const limitDate = new Date();
  limitDate.setFullYear(limitDate.getFullYear() + 2);
  const limitStr = getLocalDateString(limitDate);

  for (const ev of events) {
    if (!ev.rrule || ev.rrule === 'NONE') {
      expanded.push(ev);
      continue;
    }

    const exdates = ev.exdates ? ev.exdates.split(',') : [];
    const rruleUntilStr = ev.rrule_until ? getLocalDateString(new Date(ev.rrule_until)) : null;

    let currentStart = new Date(ev.start_date);
    let currentEnd = new Date(ev.end_date);
    let maxInstances = 730;
    let instancesCount = 1;

    while (instancesCount < maxInstances) {
      const startStr = getLocalDateString(currentStart);
      if (startStr > limitStr) break;
      if (rruleUntilStr && startStr > rruleUntilStr) break;

      const compactStart = currentStart.toISOString().replace(/[-:]/g, '').split('.')[0];
      const isExcluded = exdates.some(ex => ex.startsWith(compactStart) || ex.startsWith(compactStart.split('T')[0]));

      if (!isExcluded) {
        expanded.push({
          ...ev,
          id: ev.id + '-' + instancesCount,
          real_id: ev.id,
          original_event: ev,
          start_date: currentStart.toISOString(),
          end_date: currentEnd.toISOString(),
        });
      }

      if (ev.rrule === 'DAILY') {
        currentStart.setDate(currentStart.getDate() + 1);
        currentEnd.setDate(currentEnd.getDate() + 1);
      } else if (ev.rrule === 'WEEKLY') {
        currentStart.setDate(currentStart.getDate() + 7);
        currentEnd.setDate(currentEnd.getDate() + 7);
      } else if (ev.rrule === 'MONTHLY') {
        currentStart.setMonth(currentStart.getMonth() + 1);
        currentEnd.setMonth(currentEnd.getMonth() + 1);
      } else if (ev.rrule === 'YEARLY') {
        currentStart.setFullYear(currentStart.getFullYear() + 1);
        currentEnd.setFullYear(currentEnd.getFullYear() + 1);
      }
      instancesCount++;
    }
  }
  return expanded;
}
