<script setup>
// Ein geöffnetes Whiteboard (docs/plan-whiteboard.md, W1): die ganze Fläche
// für Excalidraw, oben eine schmale Leiste mit Zurück, Titel und Stand.
//
// Wer es angelegt hat, zeichnet; gespeichert wird kurz nach jeder Änderung
// und beim Verlassen.
//
// Gemeinsam zeichnen (W3b): Die Ansicht tritt der Sitzung beim Gerät des
// Besitzers bei (umgebung `whiteboardSitzung`, Yjs: yBindung.js). Dann
// zeichnen alle Mitglieder vor Ort mit, ihre Zeiger erscheinen, und
// gespeichert wird vom Schreiber der Sitzung über den Knoten. Ist das
// Besitzergerät nicht in der Nähe, sehen Mitglieder nur an.
//
// Einbinden (W2): geteilte Bilder erscheinen als Bild, Notizen und Dateien
// als Karte. Gespeichert wird nur der Verweis; die Bilder holt die Ansicht
// beim Öffnen aus der Freigabe, eine Karte öffnet ihren Inhalt darüber
// (umgebung `whiteboardVerweis`).
//
// Excalidraw samt React wird erst hier geladen (excalidraw.js).
import { ref, shallowRef, computed, watch, onMounted, onBeforeUnmount } from 'vue';
import { useI18n } from 'vue-i18n';
import { ArrowLeft, Eye, Loader2, CloudOff, Check, ImagePlus } from 'lucide-vue-next';
import { api, whiteboardVerweis, whiteboardSitzung } from '../umgebung';
import Anwesende from '@oberflaeche/editor/Anwesende.vue';
import WhiteboardEinbinden from './WhiteboardEinbinden.vue';
import { useToast } from '@oberflaeche/composables/useToast';
import {
  zumSpeichern, fassung, excalidrawSprache, alsLink, istVerweisLink, bildVerweise, dateiId, platzhalter, eingepasst,
} from './zeichnung';
import { ELEMENTE, LOKAL, eintragen, anlegen, zusammenfuehren, elementeVon, mitzeichner } from './yBindung';

const props = defineProps({
  projectId: { type: [Number, String], required: true },
  // Antwort von api.getWhiteboard: uuid, titel, eigen, stand, inhalt, offline
  start: { type: Object, required: true },
});
const emit = defineEmits(['zurueck']);

const { t, locale } = useI18n();
const toast = useToast();

const wb = ref(props.start);
const feld = ref(null);
const laedt = ref(true);
const status = ref(''); // '' | 'speichert' | 'gespeichert' | 'fehler'
const sitzung = shallowRef(null); // die Sitzung vor Ort, sonst null
const darfSpeichern = computed(() => wb.value.eigen && !wb.value.offline);
// Mitzeichnen: in einer laufenden Sitzung alle, sonst nur der Besitzer.
const darfZeichnen = computed(() => (sitzung.value ? !sitzung.value.aus.value : darfSpeichern.value));
// Speichern: in der Sitzung ihr Schreiber, sonst der Besitzer.
const speichertHier = computed(() => (sitzung.value ? sitzung.value.ichSpeichere.value && !sitzung.value.aus.value : darfSpeichern.value));
const einbindenOffen = ref(false);
const verweisOffen = ref(null); // die Karte, deren Inhalt gerade darüber offen ist
const VerweisAnsicht = shallowRef(whiteboardVerweis());

const SPEICHERN_NACH = 1500;
let flaeche = null;
let excalidraw = null;
let letzte = fassung(props.start.inhalt?.elements);
let ausstehend = null;
let zeitgeber = null;
let speichertGerade = null;
let weg = false;

const dunkel = () => document.documentElement.classList.contains('dark');
const helligkeit = new MutationObserver(() => flaeche?.setzen({ theme: dunkel() ? 'dark' : 'light' }));

function beiAenderung(elemente) {
  const s = sitzung.value;
  if (s && !s.aus.value && excalidraw) eintragen(s.editor.doc, excalidraw.getSceneElementsIncludingDeleted());
  if (!speichertHier.value || fassung(elemente) === letzte) return;
  ausstehend = elemente;
  clearTimeout(zeitgeber);
  zeitgeber = setTimeout(speichern, SPEICHERN_NACH);
}

async function speichern() {
  clearTimeout(zeitgeber);
  zeitgeber = null;
  if (speichertGerade) return speichertGerade;
  if (!ausstehend) return null;
  const elemente = ausstehend;
  ausstehend = null;
  status.value = 'speichert';
  speichertGerade = (async () => {
    try {
      if (sitzung.value) {
        // Über den Knoten: Er gehört dem Besitzer und speichert beim Server.
        await sitzung.value.speichern(wb.value.titel, JSON.stringify(zumSpeichern(elemente)));
      } else {
        const { data: stand } = await api.saveWhiteboard(props.projectId, wb.value.uuid, {
          titel: wb.value.titel,
          inhalt: zumSpeichern(elemente),
          stand: wb.value.stand,
        });
        wb.value = { ...wb.value, stand };
      }
      letzte = fassung(elemente);
      status.value = 'gespeichert';
    } catch (e) {
      if (String(e).includes('whiteboard-veraltet')) {
        toast.info(t('planning.whiteboard.veraltet'));
        await neuLaden();
      } else {
        // Der nächste Strich versucht es wieder, mit der ganzen Zeichnung.
        status.value = 'fehler';
      }
    } finally {
      speichertGerade = null;
    }
    if (ausstehend && !weg) zeitgeber = setTimeout(speichern, SPEICHERN_NACH);
  })();
  return speichertGerade;
}

async function neuLaden() {
  try {
    const { data } = await api.getWhiteboard(props.projectId, wb.value.uuid);
    wb.value = data;
    letzte = fassung(data.inhalt?.elements);
    ausstehend = null;
    status.value = '';
    excalidraw?.updateScene({ elements: data.inhalt?.elements ?? [] });
    bilderLaden(data.inhalt?.elements);
    flaeche?.setzen({ viewModeEnabled: !darfSpeichern.value });
  } catch {
    toast.error(t('planning.whiteboard.ladenFehler'));
  }
}

// --- Einbinden ---

const KARTE = { breite: 280, hoehe: 76 };

// Die Mitte des sichtbaren Ausschnitts, in Koordinaten der Zeichnung.
function mitte() {
  const a = excalidraw.getAppState();
  const z = a.zoom?.value ?? 1;
  return { x: a.width / 2 / z - a.scrollX, y: a.height / 2 / z - a.scrollY };
}

const groesseVon = (dataURL) => new Promise((gut) => {
  const img = new Image();
  img.onload = () => gut(eingepasst(img.naturalWidth, img.naturalHeight));
  img.onerror = () => gut(eingepasst(0, 0));
  img.src = dataURL;
});

async function einfuegen({ verweis, bild }) {
  einbindenOffen.value = false;
  if (!excalidraw || !flaeche) return;
  const m = mitte();
  let gerippe;
  if (verweis.art === 'bild') {
    try {
      const b = bild ?? (await api.getWhiteboardBild(props.projectId, verweis.freigabe, verweis.uuid)).data;
      const fileId = dateiId(verweis);
      excalidraw.addFiles([{ id: fileId, dataURL: b.dataURL, mimeType: b.mimeType, created: Date.now() }]);
      const g = await groesseVon(b.dataURL);
      gerippe = { type: 'image', fileId, x: m.x - g.width / 2, y: m.y - g.height / 2, width: g.width, height: g.height, customData: { verweis } };
    } catch {
      toast.error(t('planning.whiteboard.nichtMehrGeteilt'));
      return;
    }
  } else {
    const art = verweis.art === 'notiz' ? t('planning.whiteboard.karteNotiz')
      : verweis.mime === 'application/pdf' ? t('planning.whiteboard.kartePdf') : t('planning.whiteboard.karteDatei');
    gerippe = {
      type: 'rectangle',
      x: m.x - KARTE.breite / 2,
      y: m.y - KARTE.hoehe / 2,
      width: KARTE.breite,
      height: KARTE.hoehe,
      backgroundColor: verweis.art === 'notiz' ? '#e7f5ff' : '#fff5f5',
      fillStyle: 'solid',
      roundness: { type: 3 },
      link: alsLink(verweis),
      customData: { verweis },
      label: { text: `${verweis.name}\n${art}`, fontSize: 18 },
    };
  }
  const neu = flaeche.bauen([gerippe]);
  excalidraw.updateScene({ elements: [...excalidraw.getSceneElementsIncludingDeleted(), ...neu] });
}

// Beim Öffnen: die Bilder aus ihren Freigaben holen; was nicht mehr geteilt
// ist, bekommt einen Platzhalter. Jedes Bild einmal -- in einer Sitzung
// kommen neue von den anderen dazu.
const geladen = new Set();
async function bilderLaden(elemente) {
  for (const { fileId, verweis } of bildVerweise(elemente)) {
    if (weg) return;
    if (geladen.has(fileId)) continue;
    geladen.add(fileId);
    let datei;
    try {
      const b = (await api.getWhiteboardBild(props.projectId, verweis.freigabe, verweis.uuid)).data;
      datei = { id: fileId, dataURL: b.dataURL, mimeType: b.mimeType, created: Date.now() };
    } catch {
      datei = { id: fileId, dataURL: platzhalter(t('planning.whiteboard.nichtMehrGeteilt')), mimeType: 'image/svg+xml', created: Date.now() };
    }
    excalidraw?.addFiles([datei]);
  }
}

// Eine Karte antippen: ihren Inhalt über dem Whiteboard öffnen.
function beiLink(element, ereignis) {
  const verweis = element?.customData?.verweis;
  if (!istVerweisLink(element?.link) || !verweis) return;
  ereignis?.preventDefault?.();
  if (VerweisAnsicht.value) verweisOffen.value = verweis;
}

// --- Gemeinsam zeichnen (W3b) ---

// Ausgangsstand einer leeren Sitzung: die gespeicherte Zeichnung.
const sitzungsAnfang = (doc, inhalt) => {
  let elemente = [];
  try { elemente = JSON.parse(inhalt || '{}').elements ?? []; } catch { /* leer anfangen */ }
  anlegen(doc, elemente);
};

// Fremde Änderungen: einmal je Bild in die Szene (requestAnimationFrame
// fasst viele kleine zusammen).
let szeneGeplant = false;
function fremdesZeigen() {
  if (szeneGeplant) return;
  szeneGeplant = true;
  requestAnimationFrame(() => {
    szeneGeplant = false;
    const s = sitzung.value;
    if (!s || !excalidraw || weg) return;
    const elemente = zusammenfuehren(excalidraw.getSceneElementsIncludingDeleted(), s.editor.doc);
    excalidraw.updateScene({ elements: elemente });
    bilderLaden(elemente);
  });
}

const zeigerZeigen = () => {
  const s = sitzung.value;
  if (s && excalidraw) excalidraw.updateScene({ collaborators: mitzeichner(s.editor.awareness, s.editor.doc.clientID) });
};

function beiZeiger({ pointer, button }) {
  const s = sitzung.value;
  if (!s || s.aus.value || !pointer) return;
  s.editor.awareness.setLocalStateField('zeiger', { x: pointer.x, y: pointer.y, knopf: button });
}

async function sitzungStarten() {
  const oeffnen = whiteboardSitzung();
  if (!oeffnen || wb.value.offline) return;
  try {
    const s = await oeffnen({ projekt: String(props.projectId), uuid: wb.value.uuid, ausgangstext: sitzungsAnfang });
    if (weg) { s.schliessen(); return; }
    s.editor.awareness.setLocalStateField('user', s.editor.ich);
    s.editor.doc.getMap(ELEMENTE).observe((_e, tr) => { if (tr.origin !== LOKAL) fremdesZeigen(); });
    s.editor.awareness.on('change', zeigerZeigen);
    sitzung.value = s;
  } catch {
    // Kein Knoten in der Nähe: wie W1 -- der Besitzer zeichnet allein, die
    // anderen sehen an.
  }
}

let geschlossen = false;
async function schliessen() {
  if (geschlossen) return;
  geschlossen = true;
  // Was seit dem letzten Speichern gezeichnet wurde, noch mitnehmen.
  if (speichertHier.value && excalidraw) {
    const elemente = excalidraw.getSceneElements();
    if (fassung(elemente) !== letzte) ausstehend = elemente;
  }
  if (speichertGerade) await speichertGerade;
  await speichern();
  await sitzung.value?.schliessen().catch(() => {});
  emit('zurueck');
}

// Zurück (Android-Taste) schließt das Whiteboard statt die Seite -- wie
// BaseModal, mit eigenem Verlaufseintrag.
const eintrag = `whiteboard-${Math.random().toString(36).slice(2)}`;
const onPopstate = () => {
  if (history.state?.dialog === eintrag) return;
  // Ist eine Karte offen, schließt Zurück erst sie -- das Whiteboard bleibt.
  if (verweisOffen.value) {
    verweisOffen.value = null;
    history.pushState({ ...history.state, dialog: eintrag }, '');
    return;
  }
  schliessen();
};

onMounted(async () => {
  history.pushState({ ...history.state, dialog: eintrag }, '');
  window.addEventListener('popstate', onPopstate);
  helligkeit.observe(document.documentElement, { attributes: true, attributeFilter: ['class'] });

  window.EXCALIDRAW_ASSET_PATH ??= '/excalidraw/';
  try {
    const [{ anbringen }] = await Promise.all([import('./excalidraw.js'), sitzungStarten()]);
    if (weg) return;
    const ausSitzung = sitzung.value ? elementeVon(sitzung.value.editor.doc) : [];
    const anfang = ausSitzung.length ? ausSitzung : (wb.value.inhalt?.elements ?? []);
    flaeche = anbringen(feld.value, {
      initialData: { elements: anfang, scrollToContent: true },
      viewModeEnabled: !darfZeichnen.value,
      theme: dunkel() ? 'dark' : 'light',
      langCode: excalidrawSprache(locale.value),
      UIOptions: {
        tools: { image: false },
        canvasActions: { loadScene: false, saveToActiveFile: false, export: false, toggleTheme: false },
      },
      excalidrawAPI: (a) => {
        excalidraw = a;
        bilderLaden(anfang);
      },
      onChange: beiAenderung,
      onLinkOpen: beiLink,
      onPointerUpdate: beiZeiger,
    });
  } catch {
    toast.error(t('planning.whiteboard.ladenFehler'));
    emit('zurueck');
  } finally {
    laedt.value = false;
  }
});

// Fällt der Knoten weg, wird nur noch angesehen.
watch(darfZeichnen, (ja) => flaeche?.setzen({ viewModeEnabled: !ja }));

onBeforeUnmount(() => {
  weg = true;
  sitzung.value?.schliessen().catch(() => {});
  clearTimeout(zeitgeber);
  helligkeit.disconnect();
  window.removeEventListener('popstate', onPopstate);
  if (history.state?.dialog === eintrag) history.back();
  flaeche?.abbauen();
});
</script>

<template>
  <div class="fixed inset-0 z-[90] flex flex-col bg-boden"
    :style="{ paddingTop: 'var(--rand-oben, 0px)', paddingBottom: 'var(--rand-unten, 0px)' }">
    <div class="flex items-center gap-2 px-2 py-1.5 border-b border-linie bg-flaeche shrink-0 min-w-0">
      <button @click="schliessen" class="p-2 text-leise hover:bg-auflage rounded-xl" :title="t('common.back')" :aria-label="t('common.back')">
        <ArrowLeft class="w-5 h-5" />
      </button>
      <h3 class="font-extrabold text-schrift truncate min-w-0">{{ wb.titel }}</h3>
      <span v-if="wb.offline" class="flex items-center gap-1 text-xs font-bold text-amber-600 dark:text-amber-400 shrink-0" :title="t('planning.whiteboard.offline')">
        <CloudOff class="w-4 h-4" /> <span class="hidden sm:inline">{{ t('planning.whiteboard.offline') }}</span>
      </span>
      <span v-else-if="!darfZeichnen && !laedt && !sitzung" class="flex items-center gap-1 text-xs font-bold text-leise shrink-0 min-w-0" :title="t('planning.whiteboard.nurAnsehenFern', { name: wb.von_name })">
        <Eye class="w-4 h-4 shrink-0" /> <span class="hidden sm:inline truncate">{{ t('planning.whiteboard.nurAnsehenFern', { name: wb.von_name }) }}</span>
      </span>
      <button v-if="darfZeichnen && !laedt" type="button" @click="einbindenOffen = true"
        class="ml-auto flex items-center gap-1.5 px-2.5 py-1.5 text-sm font-bold text-marke hover:bg-marke-leise rounded-xl shrink-0"
        :title="t('planning.whiteboard.einbinden')">
        <ImagePlus class="w-4 h-4" /> <span class="hidden sm:inline">{{ t('planning.whiteboard.einbinden') }}</span>
      </button>
      <span class="text-xs font-medium shrink-0 flex items-center gap-1"
        :class="[darfZeichnen && !laedt ? '' : 'ml-auto', status === 'fehler' ? 'text-rose-500' : 'text-leise']">
        <template v-if="status === 'speichert'"><Loader2 class="w-3.5 h-3.5 animate-spin" /> {{ t('planning.whiteboard.speichert') }}</template>
        <template v-else-if="status === 'gespeichert'"><Check class="w-3.5 h-3.5" /> {{ t('planning.whiteboard.gespeichert') }}</template>
        <template v-else-if="status === 'fehler'">{{ t('planning.whiteboard.nichtGespeichert') }}</template>
      </span>
    </div>
    <!-- Wer mitzeichnet (W3b) -- allein und mit Knoten: nichts. -->
    <div v-if="sitzung && (sitzung.aus.value || sitzung.anwesende.value.length > 1)" class="px-3 py-1.5 border-b border-linie bg-flaeche shrink-0">
      <Anwesende :teilnehmer="sitzung.anwesende.value" :ich="sitzung.kennung" :aus="sitzung.aus.value" />
    </div>
    <div class="relative flex-1 min-h-0">
      <div ref="feld" class="absolute inset-0"></div>
      <div v-if="laedt" class="absolute inset-0 flex flex-col items-center justify-center gap-2 text-leise text-sm">
        <Loader2 class="w-6 h-6 animate-spin" />
        {{ t('planning.whiteboard.laedt') }}
      </div>
    </div>

    <WhiteboardEinbinden v-if="einbindenOffen" :project-id="projectId" @einfuegen="einfuegen" @close="einbindenOffen = false" />
    <component :is="VerweisAnsicht" v-if="verweisOffen && VerweisAnsicht" :projekt="String(projectId)" :verweis="verweisOffen" @close="verweisOffen = null" />
  </div>
</template>
