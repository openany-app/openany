<script setup>
/*
 * Ein PDF ansehen -- in Dateien und Akten, in der Webapp wie im Programm.
 *
 * Bis zum 27.09.2026 zeigte die Webapp beim Anklicken eines PDFs nur einen
 * Download-Knopf, und das Programm gab es an ein anderes Programm weiter.
 * Jetzt schlägt es pdf.js selbst auf: dieselbe Bibliothek, die Firefox als
 * Betrachter hat und die die Texterkennung ohnehin schon mitbringt.
 *
 * WARUM DIE BAUSTEINE AUS `pdfjs-dist/web` und kein eigenes Zeichnen: Seiten
 * nur nachladen, wenn sie ins Bild kommen, die Textebene zum Markieren, Links,
 * die Suche mit Hervorhebung -- das ist dort fertig und in Firefox erprobt.
 * Selbst gebaut wäre jedes davon ein eigenes Vorhaben.
 *
 * Die Bytes kommen über `laden()`. Die Webapp holt sie vom Server, das
 * Programm aus seiner Ablage (dann auch ohne Netz). Woher, weiß der
 * Betrachter nicht. Knöpfe wie „Herunterladen" kommen über den Slot
 * `aktionen`.
 *
 * FORMULARE AUSFÜLLEN (seit 27.09.2026, docs/plan-pdf-bearbeiten.md, Schritt 1).
 * Gibt die Anwendung `speichern(bytes, { erstes })` mit, zeigt der Betrachter
 * die Felder bedienbar, und sobald etwas geändert ist, einen Knopf
 * „Speichern". pdf.js schreibt die Werte in die Datei (inkrementell, digitale
 * Signaturen bleiben in der Regel gültig). Ohne `speichern` bleibt es Ansicht.
 * Skripte in Formularen laufen NICHT (keine Sandbox) -- Summenfelder rechnen
 * also nicht mit.
 *
 * MARKIEREN, STIFT, TEXT (Schritt 2). Ebenfalls nur mit `speichern`: Der
 * Knopf „Bearbeiten" blendet die Werkzeugleiste ein (PdfWerkzeuge.vue). Die
 * Werkzeuge sind die von pdf.js (HighlightEditor, InkEditor, FreeTextEditor);
 * gespeichert werden sie als echte PDF-Anmerkungen, auf demselben Weg wie
 * die Formularwerte.
 *
 * KOMMENTARE (Schritt 3). pdf.js hat Kommentare an Anmerkungen, aber die
 * Oberfläche dafür (Dialog, Seitenleiste) steckt in Firefox' Betrachter und
 * nicht im Baustein. Es erwartet einen `commentManager` mit ein paar
 * Methoden; `kommentarBruecke` unten ist der schlanke Ersatz: Sie öffnet
 * PdfKommentarDialog und füllt PdfKommentarListe.
 *
 * ALLES VON pdf.js BLEIBT AUSSERHALB VON VUE. `PDFViewer` und Co. arbeiten mit
 * privaten Feldern (#…), und die gehen durch einen reaktiven Proxy kaputt.
 * Deshalb gewöhnliche Variablen, keine refs.
 */
import { ref, computed, nextTick, onMounted, onBeforeUnmount, inject } from 'vue';
import { useI18n } from 'vue-i18n';
import { X, Search, ChevronUp, ChevronDown, ZoomIn, ZoomOut, MoveHorizontal, Loader2, Save, Pencil } from 'lucide-vue-next';
import BaseButton from '../base/BaseButton.vue';
import PdfWerkzeuge from './PdfWerkzeuge.vue';
import PdfKommentarDialog from './PdfKommentarDialog.vue';
import PdfKommentarListe from './PdfKommentarListe.vue';
import { FARBEN, STAERKEN, MARKIER_FARBEN_PDFJS } from './pdfWerkzeuge';
// Das Symbol des Kommentar-Knopfs. pdf.js setzt es im CSS von Firefox'
// Betrachter (`--comment-edit-button-icon`), das im Baustein fehlt.
import kommentarSymbol from 'pdfjs-dist/legacy/web/images/comment-editButton.svg?url';
// In Anführungszeichen: Vite bettet das kleine SVG als data-URL ein, und die
// enthält ' -- ein nacktes url(…) wäre ungültig und würde still verworfen.
const kommentarSymbolCss = `url("${kommentarSymbol}")`;
import { useConfirm } from '../composables/useConfirm';
import { useToast } from '../composables/useToast';
import { holePdfJs, pdfFreigeben } from '../texterkennung/ocr/pdfPages';

const props = defineProps({
  name: { type: String, default: '' },
  // () => Promise<ArrayBuffer | Uint8Array>
  laden: { type: Function, required: true },
  // (bytes: Uint8Array, { erstes: boolean }) => Promise -- optional. `erstes`
  // ist beim ersten Speichern in dieser Sitzung wahr: Nur dann soll die
  // Fassung davor in den Papierkorb, nicht bei jedem Zwischenspeichern.
  speichern: { type: Function, default: null },
});
const emit = defineEmits(['close']);
const { t } = useI18n();

// Im Programm öffnet ein Link im Dokument den Browser des Systems; die
// Webapp braucht das nicht (target=_blank genügt).
const urlOeffnen = inject('oberflaeche:url-oeffnen', null);
const { confirmDialog } = useConfirm();
const toast = useToast();

// Formulare: geändert seit dem letzten Speichern? Und wird gerade gespeichert?
const geaendert = ref(false);
const speichert = ref(false);
let schonGespeichert = false;
// Ein mit Passwort geöffnetes PDF speichern wir nicht: ob pdf.js die
// Verschlüsselung beim Zurückschreiben richtig fortführt, ist nicht erprobt.
const speicherbar = computed(() => Boolean(props.speichern) && !passwort.value);

// Bearbeiten: welche Leiste offen, welches Werkzeug, und je Werkzeug die
// zuletzt gewählte Farbe und Stärke.
const bearbeitenOffen = ref(false);
const werkzeug = ref('lesen');
const farbeJe = ref({ markieren: FARBEN.markieren[0], stift: FARBEN.stift[0], text: FARBEN.text[0] });
const staerkeJe = ref({ stift: STAERKEN.stift[1], text: STAERKEN.text[1] });
const kannZurueck = ref(false);
const kannVor = ref(false);
const auswahlDa = ref(false);
let editoren = null; // der AnnotationEditorUIManager von pdf.js

// Kommentare: der offene Dialog und die Liste im Werkzeug „Kommentare".
// Das Ziel (eine Anmerkung von pdf.js) bleibt außerhalb von Vue.
const kommentarDialog = ref(null); // { text, vorhanden } | null
let kommentarZiel = null;
const kommentarListe = ref(null); // [{ id, seite, text, farbe, datum, rect }] | null

const kommentarText = (ziel) => {
  const k = ziel?.comment;
  if (!k) return '';
  return typeof k === 'string' ? k : (k.deleted ? '' : k.text ?? '');
};
function kommentarOeffnen(ziel) {
  kommentarZiel = ziel;
  const text = kommentarText(ziel);
  kommentarDialog.value = { text, vorhanden: Boolean(text) };
}
function kommentarSetzen(text) {
  if (kommentarZiel) {
    kommentarZiel.comment = text;
    geaendert.value = true;
  }
  kommentarDialog.value = null;
  kommentarZiel = null;
}

const farbeAlsCss = (farbe) => {
  if (!farbe) return '';
  if (typeof farbe === 'string') return farbe;
  const [r, g, b] = Array.from(farbe);
  return `rgb(${r} ${g} ${b})`;
};
const alsEintrag = (k) => ({
  id: k.id,
  seite: (k.pageIndex ?? 0) + 1,
  text: k.contentsObj?.str ?? '',
  farbe: farbeAlsCss(k.color),
  datum: k.modificationDate ?? k.creationDate ?? null,
  rect: k.rect ?? null,
});

// Was pdf.js von einem CommentManager ruft. Popups beim Überfahren und
// Farbanpassungen lassen wir aus; ein Tippen öffnet den Dialog.
const kommentarBruecke = {
  // pdf.js verknüpft seinen Kommentar-Knopf per `ariaControlsElements` mit
  // diesem Element. Mit `null` wirft der Browser -- und pdf.js ließ den Knopf
  // dann still weg. Ein leeres Element genügt; der echte Dialog ist Vue.
  dialogElement: typeof document === 'undefined' ? null : document.createElement('div'),
  setSidebarUiManager() {},
  showDialog(_ui, ziel) { kommentarOeffnen(ziel); },
  toggleCommentPopup(ziel, ausgewaehlt) { if (ausgewaehlt) kommentarOeffnen(ziel); },
  updateComment(daten) {
    if (!kommentarListe.value) return;
    const neu = alsEintrag(daten);
    const rest = kommentarListe.value.filter((e) => e.id !== neu.id);
    kommentarListe.value = neu.text ? [...rest, neu] : rest;
  },
  removeComments(ids) {
    if (kommentarListe.value) kommentarListe.value = kommentarListe.value.filter((e) => !ids.includes(e.id));
  },
  updatePopupColor() {},
  makeCommentColor(farbe) { return farbeAlsCss(farbe) || null; },
  destroyPopup() {},
  showSidebar(alle) {
    kommentarListe.value = alle.map(alsEintrag).filter((e) => e.text)
      .sort((a, b) => a.seite - b.seite || (b.rect?.[3] ?? 0) - (a.rect?.[3] ?? 0));
  },
  hideSidebar() { kommentarListe.value = null; },
  destroy() {},
};

function zumKommentar(e) {
  if (!viewer) return;
  viewer.scrollPageIntoView({
    pageNumber: e.seite,
    destArray: e.rect ? [null, { name: 'XYZ' }, e.rect[0], e.rect[3] + 20, null] : undefined,
  });
}

const behaelter = ref(null);
const zustand = ref('laedt'); // laedt | bereit | passwort | fehler
const fehlertext = ref('');
const passwort = ref('');
const passwortFalsch = ref(false);
const seite = ref(1);
const seiten = ref(0);
const seiteEingabe = ref('1');
const massstab = ref(1);

const sucheOffen = ref(false);
const suchbegriff = ref('');
const treffer = ref({ current: 0, total: 0 });
const nichtGefunden = ref(false);
const suchfeld = ref(null);

let pdfjs = null;
let bausteine = null;
let viewer = null;
let busSeite = null;
let dokument = null;
let bytes = null;
const aufraeumen = new AbortController();

async function bausteineHolen() {
  pdfjs = await holePdfJs();
  // pdf_viewer.mjs liest pdf.js nicht per import, sondern von hier.
  globalThis.pdfjsLib = pdfjs;
  // Die legacy-Fassung wie in holePdfJs: sonst fehlt älteren Browsern
  // `Map.getOrInsertComputed` (Firefox 139, 27.09.2026).
  const [mod] = await Promise.all([
    import('pdfjs-dist/legacy/web/pdf_viewer.mjs'),
    import('pdfjs-dist/legacy/web/pdf_viewer.css'),
  ]);
  bausteine = mod;
}

function viewerAufbauen() {
  const { EventBus, PDFLinkService, PDFFindController, PDFViewer, LinkTarget } = bausteine;
  const bus = new EventBus();
  const links = new PDFLinkService({
    eventBus: bus,
    externalLinkTarget: LinkTarget.BLANK,
    externalLinkRel: 'noopener noreferrer nofollow',
  });
  const finder = new PDFFindController({ eventBus: bus, linkService: links });
  viewer = new PDFViewer({
    container: behaelter.value,
    viewer: behaelter.value.firstElementChild,
    eventBus: bus,
    linkService: links,
    findController: finder,
    // Formularfelder bedienbar nur, wenn sich das Ergebnis auch speichern
    // lässt -- ein Formular, dessen Eingaben beim Schließen verschwinden,
    // wäre eine Falle. Sonst nur Links.
    annotationMode: props.speichern ? pdfjs.AnnotationMode.ENABLE_FORMS : pdfjs.AnnotationMode.ENABLE,
    // Die Werkzeuge ebenso nur mit Speichern; ohne bleibt pdf.js' Editor aus.
    annotationEditorMode: props.speichern ? pdfjs.AnnotationEditorType.NONE : pdfjs.AnnotationEditorType.DISABLE,
    annotationEditorHighlightColors: MARKIER_FARBEN_PDFJS,
    commentManager: props.speichern ? kommentarBruecke : null,
    // Auf Telefonen ist der Speicher knapp; pdf.js zeichnet darüber hinaus
    // gestaffelt nach, statt eine riesige Leinwand anzulegen.
    maxCanvasPixels: 2 ** 24,
  });
  links.setViewer(viewer);

  bus.on('pagesinit', () => {
    // Schmal (Telefon): Seitenbreite. Breit: ganze Seite, höchstens 100 %.
    viewer.currentScaleValue = behaelter.value.clientWidth < 700 ? 'page-width' : 'auto';
  });
  bus.on('pagechanging', ({ pageNumber }) => {
    seite.value = pageNumber;
    seiteEingabe.value = String(pageNumber);
  });
  bus.on('scalechanging', ({ scale }) => { massstab.value = scale; });
  const trefferMelden = ({ matchesCount, state }) => {
    if (matchesCount) treffer.value = matchesCount;
    if (state !== undefined) nichtGefunden.value = state === 1; // FindState.NOT_FOUND
  };
  bus.on('annotationeditoruimanager', ({ uiManager }) => {
    editoren = uiManager;
    // Die Vorgaben einmal setzen, damit der erste Strich schon die Farbe
    // hat, die in der Leiste markiert ist.
    for (const art of ['markieren', 'stift', 'text']) parameterSetzen(art);
  });
  bus.on('editingstateschanged', ({ details }) => {
    kannZurueck.value = Boolean(details.hasSomethingToUndo);
    kannVor.value = Boolean(details.hasSomethingToRedo);
    // Ein Strich landet in der Ablage von pdf.js erst, wenn man das Werkzeug
    // verlässt. „Speichern" soll aber schon nach dem ersten Strich da sein.
    if (details.hasSomethingToUndo) geaendert.value = true;
  });
  bus.on('updatefindmatchescount', trefferMelden);
  bus.on('updatefindcontrolstate', trefferMelden);

  busSeite = { bus, links };
}

async function aufschlagen() {
  zustand.value = 'laedt';
  try {
    // pdf.js übernimmt den Puffer und leert ihn -- für einen zweiten Versuch
    // (Passwort) braucht es das Original noch.
    dokument = await pdfjs.getDocument({
      data: bytes.slice(),
      password: passwort.value || undefined,
      isEvalSupported: false,
      wasmUrl: '/pdfjs/',
      standardFontDataUrl: '/pdfjs/standard_fonts/',
      cMapUrl: '/pdfjs/cmaps/',
      enableXfa: false,
    }).promise;
  } catch (e) {
    if (e?.name === 'PasswordException') {
      passwortFalsch.value = Boolean(passwort.value);
      zustand.value = 'passwort';
      await nextTick();
      document.getElementById('pdf-passwort')?.focus();
      return;
    }
    throw e;
  }
  seiten.value = dokument.numPages;
  dokument.annotationStorage.onSetModified = () => { geaendert.value = true; };
  dokument.annotationStorage.onResetModified = () => { geaendert.value = false; };
  zustand.value = 'bereit';
  viewer.setDocument(dokument);
  busSeite.links.setDocument(dokument, null);
}

onMounted(async () => {
  try {
    const [inhalt] = await Promise.all([props.laden(), bausteineHolen()]);
    bytes = inhalt instanceof Uint8Array ? inhalt : new Uint8Array(inhalt);
    viewerAufbauen();
    gestenAnmelden();
    await aufschlagen();
  } catch (e) {
    console.error('PDF-Betrachter', e);
    // Das Programm meldet Fehler als Text („Der Inhalt liegt nicht auf diesem
    // Gerät."), die Webapp als Error. pdf.js' eigene Ausnahmen sagen dem
    // Menschen nichts und bleiben weg.
    fehlertext.value = typeof e === 'string' ? e
      : e?.message && !/^[A-Z][a-zA-Z]+Exception/.test(e.name ?? '') ? String(e.message) : '';
    zustand.value = 'fehler';
  }
});

/* ---- Zoom ---------------------------------------------------------------- */

function zoomen(schritte) {
  viewer?.updateScale({ steps: schritte, drawingDelay: 400 });
}
function anBreite() {
  if (viewer) viewer.currentScaleValue = 'page-width';
}
const prozent = computed(() => `${Math.round(massstab.value * 100)} %`);

/*
 * Zwei Finger zoomen, Strg+Mausrad auch. Ohne das zöge auf dem Telefon der
 * Browser die ganze Seite groß, samt Leiste -- und die Seiten würden nur
 * aufgeblasen, nicht schärfer gezeichnet.
 */
function gestenAnmelden() {
  const signal = aufraeumen.signal;
  let rest = 1;
  new pdfjs.TouchManager({
    container: behaelter.value,
    signal,
    onPinchStart: () => { rest = 1; },
    onPinching: (ursprung, vorher, jetzt) => {
      // Kleine Schritte gehen beim Runden auf 1 % verloren; gesammelt nicht.
      const faktor = rest * (jetzt / vorher);
      const alt = viewer.currentScale;
      viewer.updateScale({ scaleFactor: faktor, origin: ursprung, drawingDelay: 400 });
      rest = viewer.currentScale === alt ? faktor : 1;
    },
  });
  document.addEventListener('selectionchange', auswahlPruefen, { signal });
  behaelter.value.addEventListener('wheel', (e) => {
    if (!e.ctrlKey && !e.metaKey) return;
    e.preventDefault();
    viewer.updateScale({ steps: e.deltaY < 0 ? 1 : -1, origin: [e.clientX, e.clientY], drawingDelay: 400 });
  }, { passive: false, signal });
  // Links im Dokument: im Programm über den Browser des Systems.
  behaelter.value.addEventListener('click', (e) => {
    const a = e.target.closest?.('a[href]');
    if (!a || !urlOeffnen || !/^https?:/i.test(a.getAttribute('href'))) return;
    e.preventDefault();
    urlOeffnen(a.href);
  }, { capture: true, signal });
}

/* ---- Seiten -------------------------------------------------------------- */

function zuSeite() {
  const n = Number.parseInt(seiteEingabe.value, 10);
  if (viewer && n >= 1 && n <= seiten.value) viewer.currentPageNumber = n;
  else seiteEingabe.value = String(seite.value);
}

/* ---- Suche --------------------------------------------------------------- */

function suchen(art = '', rueckwaerts = false) {
  if (!busSeite) return;
  busSeite.bus.dispatch('find', {
    source: null,
    type: art,
    query: suchbegriff.value,
    caseSensitive: false,
    entireWord: false,
    highlightAll: true,
    findPrevious: rueckwaerts,
    matchDiacritics: false,
  });
}
async function sucheUmschalten() {
  sucheOffen.value = !sucheOffen.value;
  if (sucheOffen.value) {
    await nextTick();
    suchfeld.value?.focus();
  } else {
    suchbegriff.value = '';
    treffer.value = { current: 0, total: 0 };
    nichtGefunden.value = false;
    suchen();
  }
}

/* ---- Bearbeiten ---------------------------------------------------------- */

const MODUS = { lesen: 'NONE', markieren: 'HIGHLIGHT', stift: 'INK', text: 'FREETEXT', kommentare: 'POPUP' };

// Farbe und Stärke eines Werkzeugs an pdf.js geben. Ohne Auswahl gelten sie
// als Vorgabe für das nächste Element; ist eines ausgewählt, ändern sie es.
function parameterSetzen(art) {
  if (!busSeite) return;
  const P = pdfjs.AnnotationEditorParamsType;
  const senden = (type, value) => busSeite.bus.dispatch('switchannotationeditorparams', { source: null, type, value });
  if (art === 'markieren') senden(P.HIGHLIGHT_COLOR, farbeJe.value.markieren);
  if (art === 'stift') {
    senden(P.INK_COLOR, farbeJe.value.stift);
    senden(P.INK_THICKNESS, staerkeJe.value.stift);
  }
  if (art === 'text') {
    senden(P.FREETEXT_COLOR, farbeJe.value.text);
    senden(P.FREETEXT_SIZE, staerkeJe.value.text);
  }
}

// Das Umschalten in pdf.js ist asynchron; fertig ist es mit
// `annotationeditormodechanged`. Gleiches Werkzeug: sofort fertig.
function werkzeugWaehlen(id) {
  if (!viewer || !editoren) return Promise.resolve();
  const modus = pdfjs.AnnotationEditorType[MODUS[id]];
  werkzeug.value = id;
  if (viewer.annotationEditorMode === modus) return Promise.resolve();
  const fertig = new Promise((aufloesen) => {
    const weiter = () => { busSeite?.bus.off('annotationeditormodechanged', weiter); aufloesen(); };
    busSeite.bus.on('annotationeditormodechanged', weiter);
    setTimeout(weiter, 1500);
  });
  viewer.annotationEditorMode = { mode: modus };
  parameterSetzen(id);
  return fertig;
}
function farbeWaehlen(f) {
  farbeJe.value = { ...farbeJe.value, [werkzeug.value]: f };
  parameterSetzen(werkzeug.value);
}
function staerkeWaehlen(n) {
  staerkeJe.value = { ...staerkeJe.value, [werkzeug.value]: n };
  parameterSetzen(werkzeug.value);
}
// Text in diesem Dokument ausgewählt? Für den Knopf „Auswahl markieren".
function auswahlPruefen() {
  const auswahl = document.getSelection();
  const knoten = auswahl && !auswahl.isCollapsed ? auswahl.anchorNode : null;
  const element = knoten?.nodeType === 1 ? knoten : knoten?.parentElement;
  auswahlDa.value = Boolean(element?.closest?.('.textLayer') && behaelter.value?.contains(element));
}
function auswahlMarkieren() {
  editoren?.highlightSelection('main_toolbar');
  auswahlPruefen();
}

function bearbeitenUmschalten() {
  bearbeitenOffen.value = !bearbeitenOffen.value;
  if (!bearbeitenOffen.value && werkzeug.value !== 'lesen') werkzeugWaehlen('lesen');
}

/* ---- Speichern ----------------------------------------------------------- */

async function speichernJetzt() {
  if (!dokument || !speicherbar.value || speichert.value) return false;
  speichert.value = true;
  try {
    // Was gerade entsteht (ein Strich, ein Text im Tippen), erst abschließen:
    // pdf.js übernimmt es beim Verlassen des Werkzeugs. Danach dasselbe
    // Werkzeug wieder aufnehmen.
    const vorher = werkzeug.value;
    document.activeElement?.blur?.();
    if (vorher !== 'lesen') await werkzeugWaehlen('lesen');
    await nextTick();
    const daten = await dokument.saveDocument();
    if (vorher !== 'lesen') werkzeugWaehlen(vorher);
    await props.speichern(daten, { erstes: !schonGespeichert });
    schonGespeichert = true;
    dokument.annotationStorage.resetModified();
    geaendert.value = false;
    toast.success(t('common.pdf.gespeichert'));
    return true;
  } catch (e) {
    console.error('PDF speichern', e);
    toast.error(t('common.pdf.speichernFehler'));
    return false;
  } finally {
    speichert.value = false;
  }
}

/* ---- Schließen: Kreuz, Esc, und im Programm die Zurück-Taste -------------- */

const zurueckSchliesst = inject('oberflaeche:zurueck-schliesst-dialog', false);
const eintrag = `dialog-${Math.random().toString(36).slice(2)}`;

/*
 * Mit ungespeicherten Formulareingaben wird nachgefragt. Kam der Wunsch über
 * die Zurück-Taste, ist unser Verlaufseintrag schon weg -- bleibt der
 * Betrachter offen, legen wir ihn wieder an, sonst ginge das nächste Zurück
 * an der Ansicht vorbei.
 */
async function schliessenWollen({ ausZurueck = false } = {}) {
  if (geaendert.value && speicherbar.value) {
    const verwerfen = await confirmDialog(t('common.pdf.ungespeichertText'), {
      title: t('common.pdf.ungespeichert'),
      confirmLabel: t('common.pdf.verwerfen'),
      cancelLabel: t('common.pdf.weiterBearbeiten'),
      danger: true,
    });
    if (!verwerfen) {
      if (ausZurueck && zurueckSchliesst) history.pushState({ ...history.state, dialog: eintrag }, '');
      return;
    }
  }
  emit('close');
}

const onPopstate = () => { if (history.state?.dialog !== eintrag) schliessenWollen({ ausZurueck: true }); };
const onKeydown = (e) => {
  if ((e.ctrlKey || e.metaKey) && e.key === 's' && speicherbar.value) {
    e.preventDefault();
    speichernJetzt();
  } else if (e.key === 'Escape') {
    // Tippt jemand in einem Textfeld von pdf.js, gehört Esc dorthin.
    if (e.target?.closest?.('.annotationEditorLayer')) return;
    if (werkzeug.value !== 'lesen') werkzeugWaehlen('lesen');
    else if (sucheOffen.value) sucheUmschalten();
    else schliessenWollen();
  } else if ((e.ctrlKey || e.metaKey) && e.key === 'f' && zustand.value === 'bereit') {
    // Die Suche des Browsers fände nur, was gerade gezeichnet ist.
    e.preventDefault();
    if (!sucheOffen.value) sucheUmschalten();
    else suchfeld.value?.focus();
  }
};

onMounted(() => {
  window.addEventListener('keydown', onKeydown);
  if (zurueckSchliesst) {
    history.pushState({ ...history.state, dialog: eintrag }, '');
    window.addEventListener('popstate', onPopstate);
  }
});
onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeydown);
  if (zurueckSchliesst) {
    window.removeEventListener('popstate', onPopstate);
    if (history.state?.dialog === eintrag) history.back();
  }
  aufraeumen.abort();
  viewer?.setDocument(null);
  pdfFreigeben(dokument);
  viewer = dokument = bytes = busSeite = editoren = null;
});

const knopf = 'p-2 rounded-xl text-fliess hover:bg-auflage transition-colors cursor-pointer disabled:opacity-40 disabled:cursor-default';
</script>

<template>
  <!-- z-[95]: unter den Dialogen (z-[100]), damit die Rückfrage beim
       Schließen mit ungespeicherten Eingaben darüber erscheint. -->
  <!-- Die Ränder (Statusleiste, Gestenleiste) wie in der Schale der App:
       ohne sie läge die Kopfzeile im Programm unter der Uhr. -->
  <div
    class="fixed inset-0 z-[95] flex flex-col bg-flaeche"
    :style="{ '--comment-edit-button-icon': kommentarSymbolCss, paddingTop: 'var(--rand-oben)', paddingBottom: 'var(--rand-unten)', paddingLeft: 'var(--rand-links)', paddingRight: 'var(--rand-rechts)' }"
    role="dialog" aria-modal="true" :aria-label="name"
  >
    <!-- Kopf: Name, Suche, Aktionen, Schließen -->
    <div class="flex items-center gap-1 sm:gap-2 px-2 sm:px-4 py-2 border-b border-linie bg-flaeche shrink-0">
      <h3 class="font-extrabold text-schrift truncate min-w-0 flex-1 pl-1">{{ name }}</h3>
      <button v-if="zustand === 'bereit'" :class="knopf" :title="t('common.pdf.suchen')" :aria-label="t('common.pdf.suchen')" :aria-pressed="sucheOffen" @click="sucheUmschalten">
        <Search class="w-5 h-5" />
      </button>
      <button v-if="zustand === 'bereit' && speicherbar" :class="[knopf, bearbeitenOffen ? 'bg-marke-leise text-marke' : '']" :title="t('common.pdf.bearbeiten')" :aria-label="t('common.pdf.bearbeiten')" :aria-pressed="bearbeitenOffen" @click="bearbeitenUmschalten">
        <Pencil class="w-5 h-5" />
      </button>
      <!-- Erscheint erst, wenn etwas geändert wurde (Formular oder Anmerkung). -->
      <BaseButton v-if="speicherbar && (geaendert || speichert)" groesse="klein" :loading="speichert" @click="speichernJetzt">
        <Save class="w-4 h-4" />
        <span class="hidden sm:inline">{{ t('common.pdf.speichern') }}</span>
      </BaseButton>
      <slot name="aktionen" />
      <button :class="knopf" :title="t('common.close')" :aria-label="t('common.close')" @click="schliessenWollen()">
        <X class="w-5 h-5" />
      </button>
    </div>

    <PdfWerkzeuge
      v-if="bearbeitenOffen && zustand === 'bereit'"
      :werkzeug="werkzeug"
      :farbe="farbeJe[werkzeug] ?? ''"
      :staerke="staerkeJe[werkzeug] ?? 0"
      :kann-zurueck="kannZurueck"
      :kann-vor="kannVor"
      :auswahl="auswahlDa"
      @werkzeug="werkzeugWaehlen"
      @farbe="farbeWaehlen"
      @staerke="staerkeWaehlen"
      @zurueck="editoren?.undo()"
      @vor="editoren?.redo()"
      @auswahl-markieren="auswahlMarkieren"
    />

    <!-- Suche -->
    <div v-if="sucheOffen" class="flex items-center gap-1 sm:gap-2 px-2 sm:px-4 py-2 border-b border-linie bg-flaeche shrink-0">
      <input
        ref="suchfeld"
        v-model="suchbegriff"
        type="search"
        enterkeyhint="search"
        class="px-3 py-2 bg-vertieft border border-linie rounded-xl focus:outline-none focus:ring-2 focus:ring-marke text-sm text-schrift flex-1 min-w-0"
        :placeholder="t('common.pdf.suchfeld')"
        :aria-label="t('common.pdf.suchen')"
        @input="suchen()"
        @keydown.enter.prevent="suchen('again', $event.shiftKey)"
      />
      <span class="text-xs text-leise whitespace-nowrap tabular-nums" aria-live="polite">
        <template v-if="suchbegriff && nichtGefunden">{{ t('common.pdf.keineTreffer') }}</template>
        <template v-else-if="suchbegriff && treffer.total">{{ t('common.pdf.treffer', { n: treffer.current, gesamt: treffer.total }) }}</template>
      </span>
      <button :class="knopf" :disabled="!treffer.total" :title="t('common.pdf.vorheriger')" :aria-label="t('common.pdf.vorheriger')" @click="suchen('again', true)">
        <ChevronUp class="w-5 h-5" />
      </button>
      <button :class="knopf" :disabled="!treffer.total" :title="t('common.pdf.naechster')" :aria-label="t('common.pdf.naechster')" @click="suchen('again')">
        <ChevronDown class="w-5 h-5" />
      </button>
    </div>

    <!-- Die Seiten. pdf.js verlangt einen absolut gesetzten Behälter. -->
    <div class="relative flex-1 min-h-0 bg-vertieft">
      <div ref="behaelter" class="pdf-behaelter absolute inset-0 overflow-auto" :class="{ invisible: zustand !== 'bereit' }">
        <div class="pdfViewer"></div>
      </div>

      <div v-if="zustand === 'laedt'" class="absolute inset-0 flex flex-col items-center justify-center gap-3 text-fliess">
        <Loader2 class="w-8 h-8 animate-spin text-marke" />
        <span class="text-sm">{{ t('common.pdf.laedt') }}</span>
      </div>

      <form v-else-if="zustand === 'passwort'" class="absolute inset-0 flex items-center justify-center p-4" @submit.prevent="aufschlagen().catch(() => { zustand = 'fehler'; })">
        <div class="karte w-full max-w-sm p-6 space-y-4">
          <p class="text-sm text-schrift font-bold">{{ t('common.pdf.geschuetzt') }}</p>
          <input id="pdf-passwort" v-model="passwort" type="password" class="px-3 py-2 bg-vertieft border border-linie rounded-xl focus:outline-none focus:ring-2 focus:ring-marke text-sm text-schrift w-full" autocomplete="off" :placeholder="t('common.pdf.passwort')" :aria-label="t('common.pdf.passwort')" />
          <p v-if="passwortFalsch" class="text-sm text-rose-600 font-bold">{{ t('common.pdf.falschesPasswort') }}</p>
          <BaseButton type="submit" block :disabled="!passwort">{{ t('common.pdf.oeffnen') }}</BaseButton>
        </div>
      </form>

      <div v-else-if="zustand === 'fehler'" class="absolute inset-0 flex items-center justify-center p-4">
        <div class="text-center space-y-2 max-w-sm">
          <p class="text-sm font-bold text-schrift">{{ t('common.pdf.fehler') }}</p>
          <p v-if="fehlertext" class="text-xs text-leise break-words">{{ fehlertext }}</p>
        </div>
      </div>

        <PdfKommentarListe v-if="werkzeug === 'kommentare' && kommentarListe" :eintraege="kommentarListe" @springen="zumKommentar" />
      <!-- Auf oberster Ebene: sonst lägen die kleinen Leisten von pdf.js (sehr
           hoher z-index im Seitenbereich) über dem abgedunkelten Hintergrund. -->
      <Teleport to="body">
        <PdfKommentarDialog
          v-if="kommentarDialog"
          :text="kommentarDialog.text"
          :vorhanden="kommentarDialog.vorhanden"
          @speichern="kommentarSetzen"
          @loeschen="kommentarSetzen('')"
          @close="kommentarDialog = null"
        />
      </Teleport>

      <!-- Seite und Zoom, schwebend unten -->
      <div v-if="zustand === 'bereit'" :class="{ 'max-sm:hidden': werkzeug === 'kommentare' && kommentarListe }" class="absolute z-10 bottom-3 left-1/2 -translate-x-1/2 flex items-center gap-1 px-2 py-1 rounded-2xl bg-flaeche shadow-lg border border-linie text-sm">
        <form class="flex items-center gap-1 px-1" @submit.prevent="zuSeite">
          <input
            v-model="seiteEingabe"
            inputmode="numeric"
            class="w-10 text-center bg-transparent text-schrift tabular-nums rounded-lg focus:bg-auflage outline-none"
            :aria-label="t('common.pdf.seite')"
            @blur="zuSeite"
          />
          <span class="text-leise whitespace-nowrap tabular-nums">/ {{ seiten }}</span>
        </form>
        <span class="w-px h-5 bg-linie mx-1"></span>
        <button :class="knopf" :title="t('common.pdf.kleiner')" :aria-label="t('common.pdf.kleiner')" @click="zoomen(-1)">
          <ZoomOut class="w-4 h-4" />
        </button>
        <span class="text-xs text-leise w-11 text-center tabular-nums">{{ prozent }}</span>
        <button :class="knopf" :title="t('common.pdf.groesser')" :aria-label="t('common.pdf.groesser')" @click="zoomen(1)">
          <ZoomIn class="w-4 h-4" />
        </button>
        <button :class="knopf" :title="t('common.pdf.breite')" :aria-label="t('common.pdf.breite')" @click="anBreite">
          <MoveHorizontal class="w-4 h-4" />
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* pdf.js rechnet mit dem Standard des Browsers (content-box). Tailwind stellt
   ALLES auf border-box -- dann schrumpft das Seitenbild um den Seitenrand
   (18 px), die Textebene aber nicht, und je weiter unten auf der Seite, desto
   weiter liegt beides auseinander. Am 27.09.2026 auf dem Tablet: die
   Suchmarkierung eine Zeile unter dem Wort. */
.pdf-behaelter :deep(.pdfViewer),
.pdf-behaelter :deep(.pdfViewer *),
.pdf-behaelter :deep(.pdfViewer *::before),
.pdf-behaelter :deep(.pdfViewer *::after) {
  box-sizing: content-box;
}

/* Die kleinen Leisten, die pdf.js an einer ausgewählten Anmerkung zeigt
   (Farbe, Löschen). Tailwind lässt Knöpfe Schrift und Zeilenhöhe erben -- in
   der Anmerkungsebene sind das über 100 px, und das Symbol rutschte weit unter
   seinen Knopf (27.09.2026). Ohne Tailwind haben Knöpfe die Schrift des
   Systems; genau damit rechnet pdf.js. */
.pdf-behaelter :deep(.editToolbar),
.pdf-behaelter :deep(.editToolbar button),
.pdf-behaelter :deep(.editToolbar input),
.pdf-behaelter :deep(.annotationCommentButton) {
  font-size: 13px;
  line-height: normal;
}

/* Platz unter der letzten Seite, damit die schwebende Leiste sie nicht
   verdeckt. */
.pdf-behaelter :deep(.pdfViewer) {
  padding-bottom: 4.5rem;
}
</style>
