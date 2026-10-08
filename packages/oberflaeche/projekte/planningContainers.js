// Was die Planungs-Behälter in der OBERFLÄCHE ausmacht: Icon, Detailansicht,
// Endpunkte. Die Typen selbst stehen in shared/planningTypes.js – der Teil
// muss Vue-frei bleiben, weil chatText.js ihn mitliest und die Vitest-Config
// kein vue-Plugin führt.
//
// Vorher stand jeder dieser Behälter fünfmal in der Planungs-Ansicht: beim Laden,
// in der Liste, im Detail-Dispatch, beim Löschen und beim Anlegen. Drei
// zeichengleiche deleteXY-Funktionen waren das sichtbarste Zeichen davon.
import { defineAsyncComponent } from 'vue';
import {
  LayoutGrid, Flag, MapPin, CalendarRange, CalendarDays, Users,
} from 'lucide-vue-next';
import { api } from './umgebung';
import { PLANNING_TYPES } from './planningTypes';
// Kanban bleibt fest eingebunden wie bisher. Roadmap und Orte kommen erst
// beim Öffnen – so landet Leaflet nur dann im Bündel, wenn eine Orte-Sammlung
// wirklich aufgeht.
import KanbanBoardView from './KanbanBoardView.vue';

// `props` baut die Eigenschaften der Detailansicht. Die drei Ansichten
// erwarten NICHT dasselbe: Das Board braucht die Mitglieder (Zuweisung an
// Karten) und schlägt seinen Einzelpunkt als Modal auf (`openCardId`), die
// beiden anderen brauchen meId/isOwner (Löschrecht) und heben ihn nur hervor
// (`highlightId`). Einfach alles an alle zu geben wäre falsch: Was eine
// Komponente nicht als Prop führt, landet als Attribut am Wurzelelement –
// ein Array stünde dann als „[object Object]" im DOM.
const OBERFLAECHE = {
  board: {
    icon: LayoutGrid,
    view: KanbanBoardView,
    props: ({ members }, offen) => ({ members, openCardId: offen.highlightId }),
    list: (pid) => api.getBoards(pid),
    create: (pid, name, notify) => api.createBoard(pid, name, notify),
    remove: (pid, id) => api.deleteBoard(pid, id),
  },
  roadmap: {
    icon: Flag,
    view: defineAsyncComponent(() => import('./RoadmapView.vue')),
    props: ({ meId, isOwner }, offen) => ({ meId, isOwner, highlightId: offen.highlightId }),
    list: (pid) => api.getRoadmaps(pid),
    create: (pid, name, notify) => api.createRoadmap(pid, name, notify),
    remove: (pid, id) => api.deleteRoadmap(pid, id),
  },
  places: {
    icon: MapPin,
    view: defineAsyncComponent(() => import('./PlaceGroupView.vue')),
    props: ({ meId, isOwner }, offen) => ({ meId, isOwner, highlightId: offen.highlightId }),
    list: (pid) => api.getPlaceGroups(pid),
    create: (pid, name, notify) => api.createPlaceGroup(pid, name, notify),
    remove: (pid, id) => api.deletePlaceGroup(pid, id),
  },
  // Ein Schuljahr braucht beim Anlegen mehr als einen Namen: ohne Zeitraum
  // könnte es die Frage nicht beantworten, für die es da ist („ist der 3.11.
  // ein Schultag?"). `createFields` sagt dem Picker, dass er nach dem Namen
  // noch zwei Datumsfelder zeigen muss – die anderen Behälter haben keine.
  schoolyear: {
    icon: CalendarRange,
    view: defineAsyncComponent(() => import('./SchoolYearView.vue')),
    props: ({ meId, isOwner }, offen) => ({ meId, isOwner, highlightId: offen.highlightId }),
    list: (pid) => api.getSchoolYears(pid),
    create: (pid, name, notify, extra) => api.createSchoolYear(pid, name, notify, extra),
    remove: (pid, id) => api.deleteSchoolYear(pid, id),
    createFields: ['starts_on', 'ends_on'],
  },
  // Ein Behälter, zwei Kacheln. Was die Kachel entscheidet, ist die
  // VORBELEGUNG – `type` und `applies_to` –, nicht der Typ: Papierkorb,
  // Zähler und [[Verweis]] kennen nur `weekplan`. Umgestellt wird beides
  // später in der Ansicht; aus einem Stundenplan kann ein Betreuungsplan
  // werden, ohne dass etwas neu angelegt werden müsste.
  weekplan: {
    icon: CalendarDays,
    view: defineAsyncComponent(() => import('./WeekPlanView.vue')),
    // Der Betreuungsplan weist Tage an Menschen zu – ohne die Mitgliederliste
    // stünden dort Zahlen statt Namen.
    props: ({ meId, isOwner, members }, offen) => ({
      meId, isOwner, members, highlightId: offen.highlightId,
    }),
    list: (pid) => api.getWeekPlans(pid),
    create: (pid, name, notify, extra) => api.createWeekPlan(pid, name, notify, extra),
    remove: (pid, id) => api.deleteWeekPlan(pid, id),
    templateDefaults: {
      // Unterricht gibt es nur außerhalb der Ferien.
      stundenplan: { type: 'timetable', applies_to: 'school_days' },
      // Die Betreuung gilt zunächst immer. Wer sie in den Ferien anders
      // regelt, legt einen zweiten Plan mit `breaks` an – ohne das
      // verschwände die Ferienbetreuung, statt zu gelten.
      betreuungsplan: { type: 'care', applies_to: 'all' },
    },
    templateIcons: { stundenplan: CalendarDays, betreuungsplan: Users },
    // Die Zeile in der Planungsliste soll sagen, WAS dort liegt. Beide
    // Vorlagen tragen denselben Typ, unterscheidbar sind sie nur am Inhalt.
    templateKeyFor: (item) => (item.type === 'care' ? 'betreuungsplan' : 'stundenplan'),
  },
};

/** Die Behälter in der Reihenfolge, in der sie im +Neu-Picker stehen. */
export const PLANNING_CONTAINERS = PLANNING_TYPES.map((typ) => ({ ...typ, ...OBERFLAECHE[typ.kind] }));

export const containerByTemplateKey = (key) =>
  PLANNING_CONTAINERS.find((c) => c.templateKeys.includes(key)) || null;

/**
 * Welche Vorlage EINE Zeile der Liste zeigt.
 *
 * Nur der Wochenplan braucht dafür den Inhalt: Seine beiden Kacheln sind
 * derselbe Typ, unterschieden allein durch `type`. Alle anderen Behälter
 * haben genau eine Vorlage, und die erste ist dann auch die einzige.
 */
export const rowTemplate = (container, item) =>
  container.templateKeyFor?.(item) || container.templateKeys[0];

export const containerByKind = (kind) =>
  PLANNING_CONTAINERS.find((c) => c.kind === kind) || null;

/**
 * Der Behälter, in dem dieser Einzelpunkt liegt (card => Board, …).
 *
 * Die Prüfung auf einen Wert ist nötig, seit es Behälter ohne Einzelpunkt
 * gibt: Das Schuljahr trägt `itemKind: null`, und ohne sie fände ein Aufruf
 * mit null oder undefined genau dieses – die Planungsliste öffnete dann ein
 * Schuljahr, weil ein Verweis keine Art mitbrachte.
 */
export const containerByItemKind = (itemKind) =>
  (itemKind ? PLANNING_CONTAINERS.find((c) => c.itemKind === itemKind) : null) || null;
