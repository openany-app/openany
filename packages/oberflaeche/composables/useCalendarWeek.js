import { computed } from 'vue';
import { getLocalDateString } from '@oberflaeche/shared/date';
import { expandRecurringEvents } from '@oberflaeche/shared/expandEvents';

// Rechnet aus dem fokussierten Tag, den Terminen und der Overlay-Schicht die
// Wochenansicht: sieben Spalten Mo–So, eine Stundenachse und je Tag die
// Einträge mit ihrer Lage darin. Reine Ableitung wie useCalendarGrid – alle
// Eingaben sind refs, die Ausgaben sind computed.
//
// Zwei Quellen, zwei Zeitherkünfte, und der Unterschied ist der Grund, warum
// hier nichts umgerechnet wird:
//
//   Ein eigener Termin trägt einen Zeitstempel (`start_date`); seine Uhrzeit
//     liest der Browser lokal aus – so, wie das Monatsraster es auch tut.
//   Ein Overlay-Eintrag trägt WANDUHRZEIT ('08:00') in der Zeitzone seines
//     Wochenplans, serverseitig bereits aufgefaltet. Diese Zeichenkette wird
//     unverändert übernommen: Die Schulstunde beginnt um acht, unabhängig
//     davon, in welcher Zone der zusehende Elternteil gerade sitzt. Sie in
//     die Betrachterzone zu drehen wäre genau der Fehler, den `035a7fc` eine
//     Ebene tiefer bereits beseitigt hat.
const TAG_MINUTEN = 24 * 60;

// Untergrenze für die Höhe eines Eintrags, in Minuten. Eine Fünf-Minuten-
// Erinnerung wäre sonst ein Strich, den man nicht treffen kann.
const MINDESTDAUER = 30;

// Die Achse trägt den GANZEN Tag, 0 bis 24 Uhr. Sichtbar ist davon ein
// Ausschnitt; die Ansicht startet bei 7 Uhr und lässt scrollen (siehe
// CalendarWeekGrid).
//
// Bis zum 08.09.2026 stand hier ein Fenster von 7 bis 19, das sich bei Bedarf
// aufzog: Lag ein Termin früher oder später, wuchs die Achse mit. Das löste
// das Problem („der Frühdienst um sechs wird verschluckt"), erzeugte aber ein
// zweites: Die Achse war in jeder Woche anders hoch, und dieselbe Uhrzeit lag
// je nach Woche woanders auf dem Schirm. Ein voller Tag ist ruhiger und
// braucht keine Fallunterscheidung.
//
// Die alte Vorgabe steht noch als Anfang der Ansicht in CalendarWeekGrid.
const TAG = { start: 0, end: 24 };


const minutenAusUhrzeit = (hhmm) => {
  const [h, m] = String(hhmm).split(':');
  return Number(h) * 60 + Number(m);
};

const minutenAusDatum = (d) => d.getHours() * 60 + d.getMinutes();

const montagVon = (datum) => {
  const d = new Date(datum.getFullYear(), datum.getMonth(), datum.getDate());
  const versatz = (d.getDay() + 6) % 7; // Montag = 0
  d.setDate(d.getDate() - versatz);
  return d;
};

// Eigene Termine in die gemeinsame Eintragsform bringen.
function eintraegeAusTerminen(events, sichtbareKalender) {
  const liste = [];

  for (const e of expandRecurringEvents(events)) {
    if (!sichtbareKalender.has(e.calendar_id)) continue;

    const start = new Date(e.start_date);
    const ende = new Date(e.end_date);
    const startTag = getLocalDateString(start);
    const endTag = getLocalDateString(ende);

    // Mehrtägiges kommt ins Band oben, auch wenn es Uhrzeiten trägt. Ein
    // Balken, der über Mitternacht durch die Stundenachse läuft, müsste an
    // jedem Tag neu beschnitten werden – und beantwortet doch nur „läuft
    // noch", wofür das Band die richtige Form ist.
    const ganztaegig = Boolean(e.is_all_day) || startTag !== endTag;

    liste.push({
      id: `event-${e.id}`,
      kind: 'event',
      title: e.title,
      color: e.color,
      ganztaegig,
      startTag,
      endTag,
      vonMinute: ganztaegig ? 0 : minutenAusDatum(start),
      bisMinute: ganztaegig ? TAG_MINUTEN : minutenAusDatum(ende),
      zeitQuelle: e.start_date,
      eintrag: e,
    });
  }

  return liste;
}

// Overlay-Einträge (Stundenplan, Betreuung, Fälligkeiten) in dieselbe Form.
// Sie sind serverseitig auf je einen Tag aufgefaltet – kein Zeitraum, keine
// RRULE.
function eintraegeAusOverlay(items, sichtbareQuellen) {
  const liste = [];

  for (const item of items) {
    if (!sichtbareQuellen.has(item.source)) continue;

    const ganztaegig = Boolean(item.is_all_day) || !item.starts_at;
    const von = ganztaegig ? 0 : minutenAusUhrzeit(item.starts_at);
    const bis = ganztaegig
      ? TAG_MINUTEN
      : (item.ends_at ? minutenAusUhrzeit(item.ends_at) : von + MINDESTDAUER);

    liste.push({
      id: `overlay-${item.source}-${item.link.kind}-${item.link.id}-${item.date}`,
      kind: 'overlay',
      title: item.title,
      color: item.color,
      ganztaegig,
      startTag: item.date,
      endTag: item.date,
      vonMinute: von,
      bisMinute: Math.max(bis, von + MINDESTDAUER),
      zeitQuelle: `${item.date}T${item.starts_at || '00:00'}`,
      eintrag: item,
    });
  }

  return liste;
}

// Überlappende Einträge eines Tages nebeneinanderstellen: Wer sich zeitlich
// berührt, teilt sich die Breite; wer allein steht, bekommt sie ganz. Die
// Spaltenzahl gilt für den ganzen Block überlappender Einträge, nicht je
// Eintrag – sonst wären zwei Termine derselben Stunde verschieden breit.
function verteileAufSpalten(eintraege) {
  const sortiert = [...eintraege].sort(
    (a, b) => a.vonMinute - b.vonMinute || b.bisMinute - a.bisMinute,
  );

  const ergebnis = [];
  let block = [];      // die gerade offene Gruppe sich berührender Einträge
  let spaltenEnde = []; // Endminute je Spalte innerhalb des Blocks
  let blockEnde = -1;

  const blockAbschliessen = () => {
    for (const e of block) e.spaltenZahl = spaltenEnde.length;
    block = [];
    spaltenEnde = [];
    blockEnde = -1;
  };

  for (const e of sortiert) {
    if (block.length && e.vonMinute >= blockEnde) blockAbschliessen();

    let spalte = spaltenEnde.findIndex((ende) => ende <= e.vonMinute);
    if (spalte === -1) {
      spaltenEnde.push(e.bisMinute);
      spalte = spaltenEnde.length - 1;
    } else {
      spaltenEnde[spalte] = e.bisMinute;
    }

    const platziert = { ...e, spalte, spaltenZahl: 1 };
    block.push(platziert);
    ergebnis.push(platziert);
    blockEnde = Math.max(blockEnde, e.bisMinute);
  }

  if (block.length) blockAbschliessen();

  return ergebnis;
}

/**
 * @param focusedDate       ref<Date>   – irgendein Tag der gezeigten Woche
 * @param events            ref<Array>  – eigene Termine (roh, mit RRULE)
 * @param visibleCalendarIds ref<Set>   – sichtbare eigene Kalender
 * @param overlayItems      ref<Array>  – Einträge aus /api/calendar/overlay
 * @param visibleOverlayKeys ref<Set>   – sichtbare Overlay-Quellen
 */
export function useCalendarWeek(
  focusedDate,
  events,
  visibleCalendarIds,
  overlayItems,
  visibleOverlayKeys,
) {
  // Die sieben Tage der Woche, beginnend am Montag.
  const weekDays = computed(() => {
    const montag = montagVon(focusedDate.value);
    return Array.from({ length: 7 }, (_, i) => {
      const d = new Date(montag.getFullYear(), montag.getMonth(), montag.getDate() + i);
      return { date: d, dateString: getLocalDateString(d) };
    });
  });

  const tagesSchluessel = computed(() => weekDays.value.map((c) => c.dateString));

  // Alle Einträge beider Quellen, die diese Woche berühren.
  const eintraege = computed(() => {
    const von = tagesSchluessel.value[0];
    const bis = tagesSchluessel.value[6];

    return [
      ...eintraegeAusTerminen(events.value, visibleCalendarIds.value),
      ...eintraegeAusOverlay(overlayItems.value ?? [], visibleOverlayKeys.value ?? new Set()),
    ].filter((e) => e.startTag <= bis && e.endTag >= von);
  });

  const zeitgebunden = computed(() => eintraege.value.filter((e) => !e.ganztaegig));

  // Fest: der ganze Tag. Nichts wird mehr verschluckt, und nichts muss
  // aufgezogen werden – die Ansicht scrollt stattdessen.
  const hourRange = computed(() => TAG);

  const hours = computed(() => {
    const { start, end } = hourRange.value;
    return Array.from({ length: end - start }, (_, i) => start + i);
  });

  // dateString -> ganztägige/mehrtägige Einträge (Band über der Achse).
  const allDayMap = computed(() => {
    const map = new Map();
    for (const tag of tagesSchluessel.value) map.set(tag, []);

    for (const e of eintraege.value) {
      if (!e.ganztaegig) continue;
      for (const tag of tagesSchluessel.value) {
        if (tag >= e.startTag && tag <= e.endTag) map.get(tag).push(e);
      }
    }

    return map;
  });

  // dateString -> Einträge mit Lage im Fenster, in Prozent der Spaltenhöhe
  // bzw. -breite. Prozent statt Pixel: Die Zeilenhöhe gehört der Ansicht,
  // die Lage darin gehört hierher.
  const timedMap = computed(() => {
    const { start, end } = hourRange.value;
    const fensterVon = start * 60;
    const fensterMinuten = (end - start) * 60;
    const map = new Map();

    for (const tag of tagesSchluessel.value) {
      const desTages = zeitgebunden.value.filter((e) => e.startTag === tag);

      map.set(tag, verteileAufSpalten(desTages).map((e) => {
        const dauer = Math.max(e.bisMinute - e.vonMinute, MINDESTDAUER);
        return {
          ...e,
          top: ((e.vonMinute - fensterVon) / fensterMinuten) * 100,
          height: (dauer / fensterMinuten) * 100,
          left: (e.spalte / e.spaltenZahl) * 100,
          width: (1 / e.spaltenZahl) * 100,
        };
      }));
    }

    return map;
  });

  return { weekDays, hours, hourRange, allDayMap, timedMap };
}
