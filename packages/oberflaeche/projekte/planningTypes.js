// Die Planungs-Typen als Daten. Gegenstück zu App\Support\PlanningTypes im
// Backend: Beide Seiten müssen dieselben Arten kennen, sonst zeigt die eine
// tot an, was die andere auflöst. Ein Test hält die beiden Listen gegeneinander
// (PlanningTypesTest::test_beide_sprachen_kennen_dieselben_arten).
//
// Bewusst frei von Vue, api und DOM. Zwei Gründe: chatText.js liest hier mit
// und soll in der node-Umgebung testbar bleiben (wie folderTree.js und
// date.js) – und die Vitest-Config führt kein vue-Plugin, eine Datei mit
// Komponenten darin wäre also gar nicht prüfbar. Was eine Art in der
// OBERFLÄCHE ausmacht – Icon, Detailansicht, Endpunkte –, steht deshalb in
// components/project/planningContainers.js.
//
//   kind         der Behälter, wie er in einem [[Verweis]] steht
//   itemKind     der Einzelpunkt darin
//   parentField  Feld, unter dem die Auflösung die Id des Behälters
//                mitliefert – ein Einzelpunkt hat keine eigene Ansicht,
//                zeigen lässt er sich nur, indem sein Behälter aufgeht
//   templateKeys Schlüssel im +Neu-Picker; heißt bei den Orten „orte",
//                weil er in den Übersetzungen des Pickers steht. Eine LISTE,
//                weil ein Behälter mehrere Kacheln anbieten kann: Stundenplan
//                und Betreuungsplan sind derselbe Typ mit verschiedener
//                Vorbelegung – wie die fünf Abstimmungs-Vorlagen, die alle
//                auf `polls.type` laufen. Papierkorb, Zähler und Verweis
//                kennen davon nichts.
//   i18n         Präfix; darunter müssen countSingular, countPlural,
//                deleteConfirm, deleteForbidden, deleteFailed und
//                deleteTitle liegen
//   countAttr    Feld der Listenantwort mit der Anzahl der Einzelpunkte
//   signal       `kind` im ProjectPlanningChanged-Ereignis, an dem die Liste
//                erkennt, dass ein Behälter dazukam oder wegfiel. `null` beim
//                Board: Kanban hat mit KanbanBoardChanged sein eigenes
//                Ereignis, weil dort auch Karten und Spalten mitreden.
export const PLANNING_TYPES = [
  {
    kind: 'board',
    itemKind: 'card',
    parentField: 'board_id',
    templateKeys: ['board'],
    i18n: 'projects.boards',
    countAttr: 'cards_count',
    signal: null,
  },
  {
    kind: 'roadmap',
    itemKind: 'milestone',
    parentField: 'roadmap_id',
    templateKeys: ['roadmap'],
    i18n: 'projects.roadmap',
    countAttr: 'milestones_count',
    signal: 'roadmap',
  },
  {
    kind: 'places',
    itemKind: 'place',
    parentField: 'group_id',
    templateKeys: ['orte'],
    i18n: 'projects.places',
    countAttr: 'places_count',
    signal: 'place_group',
  },
  // Der erste Behälter OHNE Einzelpunkt: Man verweist auf „Schuljahr
  // 2026/27", nicht auf „Herbstferien" – die freien Abschnitte darin sind
  // Kleinelemente ohne eigenen Verweis. `itemKind` und `parentField` bleiben
  // deshalb null, und die Ableitungen unten müssen das aushalten.
  {
    kind: 'schoolyear',
    itemKind: null,
    parentField: null,
    templateKeys: ['schuljahr'],
    i18n: 'projects.schoolYear',
    countAttr: 'breaks_count',
    signal: 'school_year',
  },
  // Der erste Behälter mit ZWEI Kacheln im Picker. Stundenplan und
  // Betreuungsplan unterscheiden sich in `week_plans.type` und in der
  // Vorbelegung – aber nicht in Papierkorb, Zähler oder Verweis. Ein zweiter
  // Eintrag hier wäre eine Unterscheidung, die außerhalb der Daten nirgends
  // eine Rolle spielt.
  {
    kind: 'weekplan',
    itemKind: 'slot',
    parentField: 'week_plan_id',
    templateKeys: ['stundenplan', 'betreuungsplan'],
    i18n: 'projects.weekPlan',
    countAttr: 'slots_count',
    signal: 'week_plan',
  },
];

/** Ereignis-Arten, die eine frische Planungsliste bedeuten. */
export const CONTAINER_SIGNALS = PLANNING_TYPES.map((t) => t.signal).filter(Boolean);

/**
 * Alle Arten, die getippt in einem Verweis vorkommen können.
 *
 * `filter(Boolean)` ist nicht Vorsicht, sondern nötig: Ein Behälter ohne
 * Einzelpunkt (Schuljahr) trägt `itemKind: null`, und chatText.js sortiert
 * diese Liste nach Länge – `null.length` wirft, und zwar beim Aufbau des
 * Moduls, also noch bevor irgendetwas zu sehen ist.
 */
export const PLANNING_CHAT_KINDS = PLANNING_TYPES
  .flatMap((a) => [a.kind, a.itemKind])
  .filter(Boolean);

/** Die Schlüssel, unter denen der i18n-Präfix seine Einträge haben muss. */
export const REQUIRED_I18N_KEYS = [
  'countSingular', 'countPlural',
  'deleteConfirm', 'deleteForbidden', 'deleteFailed', 'deleteTitle',
];
