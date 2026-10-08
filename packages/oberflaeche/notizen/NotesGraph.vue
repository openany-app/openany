<script setup>
// Graph-Ansicht der Notizen. Eigene Mini-Force-Simulation auf SVG
// (gebündelt, keine externe Lib). Darstellung bewusst reduziert:
//  - Notizen = Kreis in Indigo; MIT [[Wikilink]] gefüllt, OHNE hohl (Waise).
//  - Tags = graue Raute, nur wenn der Tag-Layer eingeschaltet ist.
//  - Notiz->Notiz-Kanten durchgezogen (Markenfarbe), Notiz->Tag gestrichelt (grau).
// Klick auf eine Notiz öffnet sie, Klick auf einen Tag filtert danach.
import { ref, shallowRef, computed, onMounted, onBeforeUnmount } from 'vue';
import { useI18n } from 'vue-i18n';
import { gezeigteNotizen, leistenTags, nachEbenen } from './graphAuswahl';
import { Loader2, Waypoints, Hash, Plus, Minus, Maximize2 } from 'lucide-vue-next';

const { t } = useI18n();
const emit = defineEmits(['open', 'tag']);
const props = defineProps({
  // Loader: liefert eine Promise mit { data: { nodes, edges, tags, tagEdges } }.
  // Die Webapp fragt den Server (eigener, Mappen- oder Projekt-Graph), das
  // Programm den eigenen Speicher (`notizbuch_graph`). Seit 30.09.2026 im
  // gemeinsamen Paket, damit beide dieselbe Ansicht zeichnen.
  loadGraph: { type: Function, required: true },
});

const isLoading = ref(true);
const errored = ref(false);
// Einzeln ausgewählte Tags: nur diese erscheinen als Knoten im Graphen.
const selectedTags = ref(new Set());

// FLACH, NICHT TIEF REAKTIV (30.09.2026). Die Simulation liest und schreibt
// x/y/vx/vy tausendfach je Schritt; über Vues Proxys kostete das auf dem
// Tablet so viel, dass nur zwei, drei Bilder je Sekunde kamen und der Graph
// über eine Minute zappelte. Gezeichnet wird, wenn das Feld neu zugewiesen
// wird (einmal je Bild in simulate()).
const noteNodes = shallowRef([]);  // { id, kind:'note', title, x, y, vx, vy, linked, degree }
const tagNodes = shallowRef([]);   // { id, kind:'tag', tag, count, x, y, vx, vy }
const noteEdges = shallowRef([]);  // { from, to } (Referenzen auf Note-Nodes)
const tagEdges = shallowRef([]);   // { from: Note-Node, to: Tag-Node }
// --- Ebenen-Filter: Klick auf einen Legenden-Punkt (oberste Mappe) blendet
// diese Gruppe ein oder aus; mehrere kombinierbar.
//
// LEERE AUSWAHL = KEINE EBENE (Tiffy, 30.09.2026). Vorher hiess leer „alle"
// -- und wer den Graphen öffnete, bekam sofort jede Notiz gezeichnet und
// eingependelt, bei vielen Notizen ein Knäuel, das dauert. Jetzt wählt man
// erst, was man sehen will. Gibt es nur eine Ebene, gibt es nichts zu wählen:
// Dann steht sie gleich da.
const selectedGroups = ref(new Set());
const ebenenImSpiel = computed(() => nachEbenen(selectedGroups.value, groupList.value.length));
const shownNoteNodes = computed(() => gezeigteNotizen(noteNodes.value, selectedGroups.value, groupList.value.length, tagEdges.value, selectedTags.value));
const shownNoteSet = computed(() => new Set(shownNoteNodes.value));
const shownNoteEdges = computed(() => noteEdges.value.filter((e) => shownNoteSet.value.has(e.from) && shownNoteSet.value.has(e.to)));

// --- Tags (Tiffy, 30.09.2026): Ohne gewählte Ebene nennt die Leiste alle
// Tags, und ein gewählter Tag bringt seine Notizen mit. Mit gewählten
// Ebenen nennt sie nur die Tags, die in deren Notizen vorkommen. Gezeichnet
// werden die ausgewählten, die in der Leiste stehen; ihre Kanten nur zu
// gezeigten Notizen. ---
const leiste = computed(() => leistenTags(tagNodes.value, tagEdges.value, shownNoteSet.value, ebenenImSpiel.value));
const hasTags = computed(() => leiste.value.size > 0);
// Tags für die Auswahl-Leiste unter dem Graphen (alphabetisch, mit Anzahl).
const sortedTags = computed(() => [...leiste.value.entries()]
  .map(([tag, count]) => ({ tag, count }))
  .sort((a, b) => a.tag.localeCompare(b.tag, 'de')));
const shownTagNodes = computed(() => tagNodes.value.filter((n) => selectedTags.value.has(n.tag) && leiste.value.has(n.tag)));
const shownTagEdges = computed(() => tagEdges.value.filter((e) => selectedTags.value.has(e.to.tag)
  && leiste.value.has(e.to.tag) && shownNoteSet.value.has(e.from)));
const allTagsShown = computed(() => hasTags.value && [...leiste.value.keys()].every((tg) => selectedTags.value.has(tg)));

const W = 900;
const H = 560;
let raf = null;

// Cluster-Farben je oberster Mappe (group). Erste Farbe = Indigo, damit
// Graphen ohne echte Gruppierung wie bisher aussehen. Notizen ohne Mappe
// (group null) bleiben neutral (slate).
const GROUP_COLORS = ['#6366f1', '#0ea5e9', '#10b981', '#f59e0b', '#ef4444', '#8b5cf6', '#ec4899', '#14b8a6', '#f97316', '#84cc16'];
const NEUTRAL_COLOR = '#94a3b8';

const scatter = (i, total) => {
  const angle = (i / Math.max(1, total)) * Math.PI * 2;
  return {
    x: W / 2 + Math.cos(angle) * 180 + (Math.random() - 0.5) * 40,
    y: H / 2 + Math.sin(angle) * 180 + (Math.random() - 0.5) * 40,
    vx: 0, vy: 0,
  };
};

const load = async () => {
  isLoading.value = true;
  errored.value = false;
  try {
    const res = await props.loadGraph();
    const total = res.data.nodes.length;
    const byId = new Map();
    noteNodes.value = res.data.nodes.map((n, i) => {
      const node = { ...n, kind: 'note', linked: false, ...scatter(i, total) };
      byId.set(n.id, node);
      return node;
    });
    noteEdges.value = res.data.edges
      .map((e) => ({ from: byId.get(e.from), to: byId.get(e.to) }))
      .filter((e) => e.from && e.to);
    // "linked" = an mindestens einer Notiz-zu-Notiz-Kante beteiligt.
    noteEdges.value.forEach((e) => {
      e.from.linked = true; e.to.linked = true;
      e.from.degree = (e.from.degree || 0) + 1; e.to.degree = (e.to.degree || 0) + 1;
    });

    const tags = res.data.tags || [];
    const byTag = new Map();
    tagNodes.value = tags.map((tg, i) => {
      const node = { id: 'tag:' + tg.tag, kind: 'tag', tag: tg.tag, count: tg.count, ...scatter(i, tags.length) };
      byTag.set(tg.tag, node);
      return node;
    });
    tagEdges.value = (res.data.tagEdges || [])
      .map((e) => ({ from: byId.get(e.note), to: byTag.get(e.tag) }))
      .filter((e) => e.from && e.to);

    simulate();
  } catch (e) {
    errored.value = true;
  } finally {
    isLoading.value = false;
  }
};

// Aktive Mengen für die Simulation (nur die ausgewählten Tags).
const simNodes = () => [...shownNoteNodes.value, ...shownTagNodes.value];
const simEdges = () => [...shownNoteEdges.value, ...shownTagEdges.value];

// Force-Layout: Abstoßung aller Knoten, Federkraft je Kante, leichte
// Zentrierung. Läuft eine begrenzte Zahl Ticks und kommt dann zur Ruhe.
//
// MEHRERE SCHRITTE JE BILD (30.09.2026). Gezeichnet wurde nach jedem Schritt
// -- bei 80 Notizen mit Hunderten Kanten schaffte das Tablet nur wenige Bilder
// in der Sekunde, und die 220 Schritte zogen sich über fast eine Minute, in
// der der Graph zappelte. Jetzt rechnet jedes Bild so viele Schritte, wie in
// etwa 12 ms passen, und zeichnet einmal.
const TICKS = 220;
const simulate = () => {
  if (raf) cancelAnimationFrame(raf);
  let ticks = 0;
  const tick = () => {
    const ns = simNodes();
    const es = simEdges();
    for (let i = 0; i < ns.length; i++) {
      for (let j = i + 1; j < ns.length; j++) {
        const a = ns[i]; const b = ns[j];
        let dx = a.x - b.x; let dy = a.y - b.y;
        let d2 = dx * dx + dy * dy;
        if (d2 < 1) { dx = Math.random() - 0.5; dy = Math.random() - 0.5; d2 = 1; }
        const f = 11000 / d2; // Abstoßung (kräftig -> viel Luft zwischen Knoten)
        const d = Math.sqrt(d2);
        a.vx += (dx / d) * f; a.vy += (dy / d) * f;
        b.vx -= (dx / d) * f; b.vy -= (dy / d) * f;
      }
    }
    for (const e of es) {
      const dx = e.to.x - e.from.x; const dy = e.to.y - e.from.y;
      const d = Math.max(1, Math.sqrt(dx * dx + dy * dy));
      // Kanten innerhalb derselben Mappe (group) etwas stärker anziehen ->
      // die Cluster rücken räumlich zusammen.
      const sameGroup = e.from.kind === 'note' && e.to.kind === 'note'
        && e.from.group != null && e.from.group === e.to.group;
      const f = (d - 230) * (sameGroup ? 0.03 : 0.02); // Feder Richtung Wunschlänge (deutlich länger)
      e.from.vx += (dx / d) * f; e.from.vy += (dy / d) * f;
      e.to.vx -= (dx / d) * f; e.to.vy -= (dy / d) * f;
    }
    for (const n of ns) {
      // Sanfte Zentrierung (schwach, damit die Abstoßung greift). Waisen
      // ohne Kante hält sonst nichts -- sie trieben weit hinaus und machten
      // den ganzen Graphen klein. Sie ziehen fester zur Mitte.
      const zug = n.kind === 'note' && !n.linked ? 0.008 : 0.0012;
      n.vx += (W / 2 - n.x) * zug;
      n.vy += (H / 2 - n.y) * zug;
      n.vx *= 0.82; n.vy *= 0.82;    // Dämpfung
      // Kein Rand: Die viewBox folgt den Knoten ohnehin. Ein harter Rand
      // stapelte bei vielen Kanten Knoten übereinander in den Ecken.
      n.x += n.vx;
      n.y += n.vy;
    }
  };
  const step = () => {
    const beginn = performance.now();
    do { tick(); ticks++; } while (ticks < TICKS && performance.now() - beginn < 12);
    // Reaktivität anstoßen (Objekte wurden in-place mutiert) -- einmal je Bild.
    noteNodes.value = [...noteNodes.value];
    if (selectedTags.value.size) tagNodes.value = [...tagNodes.value];
    if (ticks < TICKS) raf = requestAnimationFrame(step);
  };
  raf = requestAnimationFrame(step);
};

// Einzelnen Tag an-/abwählen (Leiste unter dem Graphen).
const toggleTag = (tag) => {
  const s = new Set(selectedTags.value);
  s.has(tag) ? s.delete(tag) : s.add(tag);
  selectedTags.value = s;
  simulate(); // mit geändertem Knotenset neu einpendeln
};
// „Alle Tags anzeigen": alle in der Leiste einschalten – oder, wenn schon
// alle an sind, aus.
const toggleAllTags = () => {
  selectedTags.value = allTagsShown.value ? new Set() : new Set(leiste.value.keys());
  simulate();
};

// Legenden-Punkt (oberste Mappe/„Ebene") an-/abwählen; ausgewählte Ebenen
// werden exklusiv gezeigt. Ohne Auswahl bleibt der ganze Graph sichtbar.
const groupKeyOf = (g) => (g.id == null ? 'root' : String(g.id));
const isGroupSelected = (g) => selectedGroups.value.has(groupKeyOf(g));
const toggleGroup = (g) => {
  const s = new Set(selectedGroups.value);
  const k = groupKeyOf(g);
  s.has(k) ? s.delete(k) : s.add(k);
  selectedGroups.value = s;
  simulate(); // mit geändertem Knotenset neu einpendeln
};
const resetGroups = () => { selectedGroups.value = new Set(); simulate(); };
const allGroups = () => { selectedGroups.value = new Set(groupList.value.map(groupKeyOf)); simulate(); };
const alleEbenenGewaehlt = computed(() => groupList.value.every((g) => selectedGroups.value.has(groupKeyOf(g))));

// Notiz-Radius nach Grad; Tag-Raute nach Häufigkeit (dezent). Bewusst klein
// gehalten, damit die Knoten auch bei vielen Notizen auseinander stehen.
const noteRadius = (n) => Math.min(10, 4 + (n.degree || 0) * 0.9);
const tagRadius = (n) => Math.min(10, 5 + Math.log2(1 + n.count) * 1.4);
const diamond = (n) => {
  const r = tagRadius(n);
  return `${n.x},${n.y - r} ${n.x + r},${n.y} ${n.x},${n.y + r} ${n.x - r},${n.y}`;
};

// --- Hover-Fokus: über einem Knoten werden nur er, seine Nachbarn und die
// verbindenden Kanten hervorgehoben; alles andere blasst ab. ---
const hoverNode = shallowRef(null); // aktuell überfahrener Knoten (Notiz oder Tag)
// Direkte Nachbarn (über beide Kantenarten) des überfahrenen Knotens.
const hoverNeighbors = computed(() => {
  const set = new Set();
  const h = hoverNode.value;
  if (!h) return set;
  set.add(h);
  for (const e of shownNoteEdges.value) {
    if (e.from === h) set.add(e.to);
    if (e.to === h) set.add(e.from);
  }
  for (const e of shownTagEdges.value) {
    if (e.from === h) set.add(e.to);
    if (e.to === h) set.add(e.from);
  }
  return set;
});
// Sichtbarkeit im Fokus-Modus: ohne Hover ist alles voll sichtbar.
const nodeActive = (n) => !hoverNode.value || hoverNeighbors.value.has(n);
// Ist dieser Knoten gerade fokussiert (überfahrener Knoten oder ein Nachbar)?
// Dann wird sein Label vollständig ausgeschrieben statt gekürzt.
const nodeFocused = (n) => !!hoverNode.value && hoverNeighbors.value.has(n);
// Label-Text: im Fokus voller Titel/Tag, sonst gekürzt.
const noteLabel = (n) => nodeFocused(n) ? (n.title || '…') : (n.title || '…').slice(0, 22);
const tagLabel = (n) => nodeFocused(n) ? n.tag : n.tag.slice(0, 20);
const edgeActive = (e) => !hoverNode.value || (hoverNeighbors.value.has(e.from) && hoverNeighbors.value.has(e.to)
  && (e.from === hoverNode.value || e.to === hoverNode.value));

// Cluster-Gruppen (oberste Mappe je Notiz) für Einfärbung + Legende. In
// Reihenfolge des Vorkommens; 'root' (ohne Mappe) bleibt neutral.
const groupList = computed(() => {
  const seen = new Map();
  for (const n of noteNodes.value) {
    const key = n.group == null ? 'root' : String(n.group);
    if (!seen.has(key)) {
      seen.set(key, { id: n.group ?? null, name: n.group == null ? t('notes.graphRootGroup') : (n.groupName || '…') });
    }
  }
  let ci = 0;
  return [...seen.values()].map((g) => ({
    ...g,
    color: g.id == null ? NEUTRAL_COLOR : GROUP_COLORS[ci++ % GROUP_COLORS.length],
  }));
});
const colorByGroup = computed(() => {
  const m = new Map();
  for (const g of groupList.value) m.set(g.id == null ? 'root' : String(g.id), g.color);
  return m;
});
// Ohne echte Gruppierung (0/1 Gruppe) bleibt es beim Standard-Indigo.
const nodeColor = (n) => (groupList.value.length <= 1
  ? GROUP_COLORS[0]
  : (colorByGroup.value.get(n.group == null ? 'root' : String(n.group)) || NEUTRAL_COLOR));

// ViewBox folgt den tatsächlichen Knoten-Grenzen (statt fixem Rahmen), damit
// der Graph immer zentriert und vollständig im Fenster sitzt – egal, wohin
// die Simulation ihn schiebt. Unten etwas mehr Platz für die Beschriftung.
const viewBox = computed(() => {
  const ns = [...shownNoteNodes.value, ...shownTagNodes.value];
  if (!ns.length) return `0 0 ${W} ${H}`;
  let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
  for (const n of ns) {
    const r = n.kind === 'tag' ? tagRadius(n) : noteRadius(n);
    minX = Math.min(minX, n.x - r);
    maxX = Math.max(maxX, n.x + r);
    minY = Math.min(minY, n.y - r);
    maxY = Math.max(maxY, n.y + r + 20); // Label sitzt unter dem Knoten
  }
  const padX = 40, padY = 30;
  minX -= padX; maxX += padX; minY -= padY; maxY += padY;
  return `${minX} ${minY} ${maxX - minX} ${maxY - minY}`;
});

// --- Zoom & Pan ---
// Über die automatisch zentrierte viewBox legt sich ein Nutzer-Transform
// (translate + scale). +/- (und Mausrad) am Desktop, Pinch am Handy –
// jeweils um den Fokuspunkt herum, damit dort nichts wegspringt.
const svgEl = ref(null);
const INITIAL_ZOOM = 1;
const zoom = ref(INITIAL_ZOOM);
const panX = ref(0);
const panY = ref(0);
const MIN_Z = 0.3, MAX_Z = 4;

// Screen-Koordinaten -> viewBox-Koordinaten (berücksichtigt viewBox +
// preserveAspectRatio über die aktuelle CTM der SVG).
const toLocal = (clientX, clientY) => {
  const ctm = svgEl.value?.getScreenCTM();
  if (!ctm) return { x: 0, y: 0 };
  const p = new DOMPoint(clientX, clientY).matrixTransform(ctm.inverse());
  return { x: p.x, y: p.y };
};

// Auf newZoom skalieren, dabei den Fokuspunkt (fx,fy in viewBox-Koordinaten)
// fixieren.
const applyZoom = (newZoom, fx, fy) => {
  const z0 = zoom.value;
  const z = Math.min(MAX_Z, Math.max(MIN_Z, newZoom));
  panX.value = fx - (z / z0) * (fx - panX.value);
  panY.value = fy - (z / z0) * (fy - panY.value);
  zoom.value = z;
};

// +/- : um die Mitte des sichtbaren Bereichs zoomen.
const zoomBy = (mult) => {
  const [minX, minY, w, h] = viewBox.value.split(' ').map(Number);
  applyZoom(zoom.value * mult, minX + w / 2, minY + h / 2);
};
const resetZoom = () => { zoom.value = INITIAL_ZOOM; panX.value = 0; panY.value = 0; };

const onWheel = (e) => {
  e.preventDefault();
  const { x, y } = toLocal(e.clientX, e.clientY);
  applyZoom(zoom.value * (e.deltaY < 0 ? 1.12 : 1 / 1.12), x, y);
};

// Klick vs. Ziehen: hat der Zeiger sich über den Schwellwert bewegt, gilt es
// als Verschieben – der anschließende Klick öffnet dann KEINE Notiz.
let dragMoved = false;
// Kam die letzte Interaktion per Touch? Dann gilt am Handy (kein Hover):
// erstes Tippen isoliert (setzt hoverNode), zweites Tippen auf denselben
// Knoten öffnet. Am Desktop (Maus) öffnet ein Klick weiterhin sofort.
let lastWasTouch = false;
let nodeTapConsumed = false; // wurde der Tap von einem Knoten verarbeitet?
const onNodeOpen = (n) => {
  if (dragMoved) return;
  nodeTapConsumed = true;
  if (lastWasTouch && hoverNode.value !== n) { hoverNode.value = n; return; }
  emit('open', n);
};
const onTagClick = (tag, n) => {
  if (dragMoved) return;
  nodeTapConsumed = true;
  if (lastWasTouch && hoverNode.value !== n) { hoverNode.value = n; return; }
  emit('tag', tag);
};
// Nach einem Touch schickt der Browser ein synthetisches mouseenter VOR dem
// click. Das setzte hoverNode schon -- und der erste Tipp öffnete sofort,
// statt zu isolieren. Kurz nach einem Touch zählt Hover deshalb nicht.
const onEnter = (n) => { if (Date.now() - lastTouchAt > 700) hoverNode.value = n; };
const onLeave = () => { if (Date.now() - lastTouchAt > 700) hoverNode.value = null; };
// Tap auf leeren Hintergrund (kein Knoten) hebt am Handy die Isolation auf.
const onSvgClick = () => {
  if (lastWasTouch && !dragMoved && !nodeTapConsumed) hoverNode.value = null;
  nodeTapConsumed = false;
};

// Verschieben um ein Screen-Delta (in viewBox-Koordinaten umgerechnet).
const panByScreen = (fromX, fromY, toX, toY) => {
  const a = toLocal(fromX, fromY);
  const b = toLocal(toX, toY);
  panX.value += b.x - a.x;
  panY.value += b.y - a.y;
};

// --- Maus-Pan (Desktop) ---
let mouse = null;
const onMouseMove = (e) => {
  if (!mouse) return;
  panByScreen(mouse.x, mouse.y, e.clientX, e.clientY);
  if (Math.hypot(e.clientX - mouse.x, e.clientY - mouse.y) > 4) dragMoved = true;
  mouse = { x: e.clientX, y: e.clientY };
};
const onMouseUp = () => {
  mouse = null;
  window.removeEventListener('mousemove', onMouseMove);
  window.removeEventListener('mouseup', onMouseUp);
};
const onMouseDown = (e) => {
  if (e.button !== 0) return;
  // Nach einem Touch feuert der Browser synthetische Maus-Events; die dürfen
  // den Touch-Modus nicht zurücksetzen (sonst öffnet der erste Tap sofort).
  if (Date.now() - lastTouchAt > 700) lastWasTouch = false;
  mouse = { x: e.clientX, y: e.clientY };
  dragMoved = false;
  window.addEventListener('mousemove', onMouseMove);
  window.addEventListener('mouseup', onMouseUp);
};

// --- Touch: 1 Finger = verschieben, 2 Finger = zoomen ---
let pinch = null;
let touchPan = null;
const touchDist = (t) => Math.hypot(t[0].clientX - t[1].clientX, t[0].clientY - t[1].clientY);
let lastTouchAt = 0;
const onTouchStart = (e) => {
  dragMoved = false;
  lastWasTouch = true;
  lastTouchAt = Date.now();
  if (e.touches.length === 2) {
    touchPan = null;
    const mid = toLocal((e.touches[0].clientX + e.touches[1].clientX) / 2, (e.touches[0].clientY + e.touches[1].clientY) / 2);
    pinch = { d0: touchDist(e.touches), z0: zoom.value, px0: panX.value, py0: panY.value, fx: mid.x, fy: mid.y };
  } else if (e.touches.length === 1) {
    touchPan = { x: e.touches[0].clientX, y: e.touches[0].clientY };
  }
};
const onTouchMove = (e) => {
  if (pinch && e.touches.length === 2) {
    e.preventDefault();
    const z = Math.min(MAX_Z, Math.max(MIN_Z, pinch.z0 * (touchDist(e.touches) / pinch.d0)));
    panX.value = pinch.fx - (z / pinch.z0) * (pinch.fx - pinch.px0);
    panY.value = pinch.fy - (z / pinch.z0) * (pinch.fy - pinch.py0);
    zoom.value = z;
    return;
  }
  if (touchPan && e.touches.length === 1) {
    e.preventDefault();
    const t = e.touches[0];
    panByScreen(touchPan.x, touchPan.y, t.clientX, t.clientY);
    if (Math.hypot(t.clientX - touchPan.x, t.clientY - touchPan.y) > 10) dragMoved = true;
    touchPan = { x: t.clientX, y: t.clientY };
  }
};
const onTouchEnd = (e) => {
  if (e.touches.length < 2) pinch = null;
  if (e.touches.length === 0) touchPan = null;
};

onMounted(load);
onBeforeUnmount(() => { if (raf) cancelAnimationFrame(raf); });
</script>

<template>
  <div class="karte shadow-sm overflow-hidden transition-colors duration-300">
    <div class="flex items-center justify-between gap-3 px-6 py-4 border-b border-linie">
      <div class="flex items-center gap-3">
        <!-- Icon nur am Desktop; am Handy Platz sparen. -->
        <Waypoints class="hidden sm:block w-5 h-5 text-leise" />
        <h3 class="font-extrabold text-schrift">{{ t('notes.graphTitle') }}</h3>
      </div>
      <!-- Alle Tags anzeigen (bzw. wieder ausblenden) -->
      <button
        v-if="hasTags"
        @click="toggleAllTags"
        :title="t('notes.graphToggleTags')"
        class="flex items-center gap-2 px-4 py-2 text-sm font-bold rounded-xl transition-all cursor-pointer shrink-0"
        :class="allTagsShown
          ? 'bg-slate-700 dark:bg-slate-300 text-white dark:text-slate-900'
          : 'bg-slate-100 hover:bg-slate-200 dark:bg-slate-800 dark:hover:bg-slate-700 text-fliess'"
      >
        <Hash class="w-4 h-4" />
        <span>{{ t('notes.graphToggleTags') }}</span>
      </button>
    </div>

    <div v-if="isLoading" class="flex justify-center py-20">
      <Loader2 class="w-8 h-8 animate-spin text-slate-400" />
    </div>
    <div v-else-if="errored" class="py-16 text-center text-sm font-bold text-rose-500">{{ t('notes.loadFailed') }}</div>
    <div v-else-if="!noteNodes.length" class="py-20 text-center space-y-2 max-w-md mx-auto px-6">
      <Waypoints class="w-10 h-10 text-slate-300 dark:text-slate-600 mx-auto" />
      <p class="text-sm font-medium text-leise">{{ t('notes.graphEmpty') }}</p>
    </div>
    <div v-else class="relative">
     <!-- Noch keine Ebene gewählt: ein Satz statt eines leeren Rahmens. -->
     <div v-if="!shownNoteNodes.length" class="py-16 text-center space-y-2 max-w-md mx-auto px-6">
      <Waypoints class="w-10 h-10 text-slate-300 dark:text-slate-600 mx-auto" />
      <p class="text-sm font-medium text-leise">{{ t('notes.graphChooseGroup') }}</p>
     </div>
     <div v-else class="overflow-x-auto">
      <svg ref="svgEl" :viewBox="viewBox" preserveAspectRatio="xMidYMid meet"
        class="w-full md:min-w-[640px] select-none cursor-grab active:cursor-grabbing" style="max-height: 80vh; touch-action: none;"
        @wheel="onWheel" @mousedown="onMouseDown" @click="onSvgClick"
        @touchstart="onTouchStart" @touchmove="onTouchMove" @touchend="onTouchEnd">
       <g :transform="`translate(${panX} ${panY}) scale(${zoom})`">
        <!-- Ebene 1: Kanten. Im Hover-Fokus blassen nicht beteiligte Kanten ab. -->
        <!-- Notiz->Tag-Kanten (gestrichelt, grau) – nur ausgewählte Tags -->
        <line v-for="(e, i) in shownTagEdges" :key="'te' + i"
          :x1="e.from.x" :y1="e.from.y" :x2="e.to.x" :y2="e.to.y"
          class="stroke-slate-300 dark:stroke-slate-600 transition-opacity" stroke-width="0.7" stroke-dasharray="3 3"
          :style="{ opacity: edgeActive(e) ? 1 : 0.06 }" />
        <!-- Notiz->Notiz-Kanten (durchgezogen, Markenfarbe) -->
        <line v-for="(e, i) in shownNoteEdges" :key="'e' + i"
          :x1="e.from.x" :y1="e.from.y" :x2="e.to.x" :y2="e.to.y"
          class="stroke-marke transition-opacity" stroke-width="0.8"
          :style="{ opacity: edgeActive(e) ? 1 : 0.06 }" />

        <!-- Ebene 2: Knoten-Formen (ohne Beschriftung). Nicht fokussierte blassen ab. -->
        <!-- Tag-Knoten (graue Raute) – nur ausgewählte Tags -->
        <polygon v-for="n in shownTagNodes" :key="'ts' + n.id" :points="diamond(n)"
          @click="onTagClick(n.tag, n)" @mouseenter="onEnter(n)" @mouseleave="onLeave"
          class="fill-slate-400 dark:fill-slate-500 hover:fill-slate-600 dark:hover:fill-slate-300 transition-opacity cursor-pointer"
          :style="{ opacity: nodeActive(n) ? 1 : 0.12 }" />

        <!-- Notiz-Knoten (Farbe nach oberster Mappe; gefüllt = verlinkt, hohl = Waise) -->
        <circle v-for="n in shownNoteNodes" :key="'ns' + n.id" :cx="n.x" :cy="n.y" :r="noteRadius(n)"
          @click="onNodeOpen(n)" @mouseenter="onEnter(n)" @mouseleave="onLeave"
          class="transition-opacity cursor-pointer hover:opacity-80"
          :class="n.linked ? '' : 'fill-white dark:fill-slate-900'"
          :style="[n.linked ? `fill:${nodeColor(n)}` : `stroke:${nodeColor(n)}`, { opacity: nodeActive(n) ? 1 : 0.12 }]"
          :stroke-width="n.linked ? 0 : 1.5" />

        <!-- Ebene 3: Beschriftungen zuletzt -> liegen immer VOR allen Knoten,
             werden also nie von einem anderen Kreis verdeckt. -->
        <text v-for="n in shownTagNodes" :key="'tl' + n.id" :x="n.x" :y="n.y + tagRadius(n) + 12" text-anchor="middle"
          class="graph-label font-semibold select-none pointer-events-none transition-opacity"
          style="font-size: 13px;" :style="{ opacity: nodeActive(n) ? 1 : 0.12 }">#{{ tagLabel(n) }}</text>
        <text v-for="n in shownNoteNodes" :key="'nl' + n.id" :x="n.x" :y="n.y + noteRadius(n) + 12" text-anchor="middle"
          class="graph-label font-semibold select-none pointer-events-none transition-opacity"
          style="font-size: 13px;" :style="{ opacity: nodeActive(n) ? 1 : 0.12 }">{{ noteLabel(n) }}</text>
       </g>
      </svg>
     </div>

      <!-- Legende: oberste Mappe -> Cluster-Farbe (nur bei mehreren Gruppen).
           Klick auf einen Punkt zeigt diese Ebene exklusiv; mehrere kombinierbar. -->
      <div v-if="groupList.length > 1" class="border-t border-linie px-4 py-2.5 flex items-center gap-x-2 gap-y-1.5 flex-wrap">
        <button v-for="g in groupList" :key="g.id ?? 'root'" @click="toggleGroup(g)"
          :title="t('notes.graphGroupFilter')"
          class="flex items-center gap-1.5 text-xs font-semibold rounded-xl px-2 py-1 transition-all cursor-pointer"
          :class="isGroupSelected(g)
            ? 'bg-slate-700 dark:bg-slate-300 text-white dark:text-slate-900'
            : 'text-leise hover:bg-auflage'">
          <span class="w-3 h-3 rounded-full shrink-0" :style="`background:${g.color}`"></span>
          {{ g.name.slice(0, 24) }}
        </button>
        <button v-if="!alleEbenenGewaehlt" @click="allGroups"
          class="text-xs font-bold text-leise hover:text-slate-700 dark:hover:text-slate-200 underline underline-offset-2 cursor-pointer ml-1">
          {{ t('notes.graphGroupReset') }}
        </button>
        <button v-if="selectedGroups.size" @click="resetGroups"
          class="text-xs font-bold text-leise hover:text-slate-700 dark:hover:text-slate-200 underline underline-offset-2 cursor-pointer ml-1">
          {{ t('notes.graphGroupNone') }}
        </button>
      </div>

      <!-- Zoom-Steuerung: +/- (und Mausrad) am Desktop, Pinch am Handy -->
      <div class="absolute top-3 right-3 flex flex-col gap-1.5">
        <button @click="zoomBy(1.25)" :title="t('notes.graphZoomIn')" :aria-label="t('notes.graphZoomIn')"
          class="w-9 h-9 flex items-center justify-center bg-white/90 dark:bg-slate-800/90 backdrop-blur border border-linie text-fliess rounded-xl shadow-sm hover:bg-slate-100 dark:hover:bg-slate-700 transition-colors cursor-pointer">
          <Plus class="w-4 h-4" />
        </button>
        <button @click="zoomBy(0.8)" :title="t('notes.graphZoomOut')" :aria-label="t('notes.graphZoomOut')"
          class="w-9 h-9 flex items-center justify-center bg-white/90 dark:bg-slate-800/90 backdrop-blur border border-linie text-fliess rounded-xl shadow-sm hover:bg-slate-100 dark:hover:bg-slate-700 transition-colors cursor-pointer">
          <Minus class="w-4 h-4" />
        </button>
        <button @click="resetZoom" :title="t('notes.graphZoomReset')" :aria-label="t('notes.graphZoomReset')"
          class="w-9 h-9 flex items-center justify-center bg-white/90 dark:bg-slate-800/90 backdrop-blur border border-linie text-fliess rounded-xl shadow-sm hover:bg-slate-100 dark:hover:bg-slate-700 transition-colors cursor-pointer">
          <Maximize2 class="w-4 h-4" />
        </button>
      </div>

      <!-- Tag-Leiste unter dem Graphen: Schlagworte einzeln an-/abwählen,
           ausgewählte erscheinen als Knoten im Graphen. -->
      <div v-if="hasTags" class="border-t border-linie px-4 py-3 flex items-center gap-2 flex-wrap">
        <Hash class="w-4 h-4 text-slate-400 shrink-0" />
        <button v-for="tg in sortedTags" :key="tg.tag" @click="toggleTag(tg.tag)"
          class="flex items-center gap-1 px-2.5 py-1 text-xs font-bold rounded-xl transition-colors cursor-pointer"
          :class="selectedTags.has(tg.tag)
            ? 'bg-slate-700 dark:bg-slate-300 text-white dark:text-slate-900'
            : 'bg-auflage text-fliess hover:bg-slate-200 dark:hover:bg-slate-700'">
          #{{ tg.tag }} <span class="opacity-60">{{ tg.count }}</span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Beschriftung: fast schwarze Schrift mit hellem Halo – in BEIDEN Themes gut
   lesbar (der helle Rand gibt auch im Dark Mode den Kontrast). Überschreibt
   bewusst die fill-slate-*-Klassen aus dem Template. */
.graph-label {
  fill: #0f172a; /* slate-900, fast schwarz */
  stroke: #ffffff;
  stroke-width: 3px;
  paint-order: stroke;
  stroke-linejoin: round;
}
</style>
