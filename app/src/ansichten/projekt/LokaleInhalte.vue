<script setup>
/*
 * Freigaben in Projekte ohne Server: Notiz-Mappen, Ordner und
 * Alben. Fremde liegen im Projekt und kommen beim Abgleich vor Ort -- direkt
 * vom Gerät dessen, der freigegeben hat. Dateien und Originalbilder kommen
 * beim ersten Öffnen und bleiben dann hier.
 *
 * FREMDE NOTIZEN BEARBEITEN (strenge Sperre): Die Sperre kommt vom Gerät
 * dessen, der freigegeben hat; dort wird gespeichert. Gespeichert wird 2 s
 * nach dem letzten Tippen -- das verlängert auch die Sperre (sie läuft nach
 * 10 Minuten ohne Tippen ab). „Fertig" lässt sie los.
 */
import { ref, shallowRef, computed, onMounted, onBeforeUnmount } from 'vue';
import { useI18n } from 'vue-i18n';
import { Plus, Folder, FolderArchive, FileText, Image as ImageIcon, NotebookText, CalendarDays, Loader2, CircleX } from 'lucide-vue-next';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import BaseModal from '@oberflaeche/base/BaseModal.vue';
import PdfBetrachter from '@oberflaeche/speicher/PdfBetrachter.vue';
import MarkdownEditor from '@oberflaeche/editor/MarkdownEditor.vue';
import Anwesende from '@oberflaeche/editor/Anwesende.vue';
import { sitzungOeffnen } from '../../sitzung';
import { useToast } from '@oberflaeche/composables/useToast';
import { projekteQuelle as quelle } from '../../quellen/projekte';
import { einsortieren, mimeAus } from '../../quellen/ablage';
import { teilbareKalender, kalenderTeilen, GETEILT, geteilteKalender, kalenderBeenden } from '../../quellen/kalenderTeilen';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import KalenderRahmen from './KalenderRahmen.vue';
import ZurueckSchliesst from '@oberflaeche/base/ZurueckSchliesst.vue';

const props = defineProps({ project: { type: Object, required: true } });
const toast = useToast();
const { t } = useI18n();
const pid = () => props.project.id;

const lokaleFreigaben = ref(null);
const waehlen = ref(false);
const notizOffen = ref(null);
const geteiltesBild = ref(null); // { name, url, bytes, mime }
const geteiltesPdf = ref(null); // { name, id }
const holt = ref(null); // id, waehrend Bytes kommen
const vorschauen = ref({}); // id -> Blob-URL | false

const ARTEN = {
  note_folder: { ort: 'notizen', icon: NotebookText },
  file_folder: { ort: 'dateien', icon: Folder },
  album: { ort: 'galerie', icon: ImageIcon },
  calendar: { ort: 'kalender', icon: CalendarDays },
};
const ortText = (f) => {
  const ort = f.art === 'file_folder' && f.zone === 'documents' ? 'dokumente' : ARTEN[f.art]?.ort;
  return ort ? t(`app.projekte.ort.${ort}`) : '';
};
const vonText = (f) => (f.eigen ? t('app.projekte.vonDir') : f.von ? t('app.projekte.von', { name: f.von }) : t('app.projekte.vonMitglied'));
const groesseText = (b) => (b >= 1048576 ? `${(b / 1048576).toFixed(1)} MB` : `${Math.max(1, Math.round(b / 1024))} KB`);

async function laden() {
  try {
    lokaleFreigaben.value = await quelle.lokaleFreigaben(pid());
    vorschauenLaden();
  } catch (e) {
    lokaleFreigaben.value = { freigaben: [], waehlbar: [] };
    toast.error(String(e));
  }
}
// Geteilte Kalender stehen in der Liste wie alles andere.
const kalender = ref([]);
const kalenderOffen = ref(null);
const kalenderLaden = async () => { kalender.value = await geteilteKalender(pid()); };
onMounted(() => {
  laden();
  kalenderLaden();
  window.addEventListener(GETEILT, kalenderLaden);
});
onBeforeUnmount(() => window.removeEventListener(GETEILT, kalenderLaden));

// Die Liste nach Arten, in der Reihenfolge der Überschriften: Mappen,
// Kalender, Alben, Ordner, Akten.
const GRUPPEN = ['mappe', 'kalender', 'album', 'ordner', 'akte'];
// Das Symbol steht an der Überschrift, nicht an jedem Eintrag.
const GRUPPEN_ICON = { mappe: NotebookText, kalender: CalendarDays, album: ImageIcon, ordner: Folder, akte: FolderArchive };
const gruppeVon = (f) => {
  if (f.art === 'kalender') return 'kalender';
  if (f.art === 'note_folder') return 'mappe';
  if (f.art === 'album') return 'album';
  return f.zone === 'documents' ? 'akte' : 'ordner';
};
const sortiert = computed(() => [...(lokaleFreigaben.value?.freigaben ?? []), ...kalender.value]
  .map((f) => ({ f, g: gruppeVon(f) }))
  .sort((a, b) => GRUPPEN.indexOf(a.g) - GRUPPEN.indexOf(b.g)));

// Im Dialog dieselben Arten mit Überschrift.
const wahlSortiert = computed(() => [...(lokaleFreigaben.value?.waehlbar ?? []), ...kalenderWahl.value]
  .map((w) => ({ w, g: w.art === 'calendar' ? 'kalender' : gruppeVon(w) }))
  .sort((a, b) => GRUPPEN.indexOf(a.g) - GRUPPEN.indexOf(b.g)));

const { confirmDialog } = useConfirm();
async function kalenderZurueck(k) {
  try {
    await kalenderBeenden(pid(), k, confirmDialog, t);
  } catch (e) {
    toast.error(String(e));
  }
}

// Vorschaubilder der Alben -- die fremden kamen beim Abgleich mit.
async function vorschauenLaden() {
  for (const f of lokaleFreigaben.value?.freigaben ?? []) {
    if (f.art !== 'album') continue;
    for (const e of f.eintraege) {
      if (e.id in vorschauen.value || !e.vorschau_da) continue;
      try {
        const bytes = await quelle.geteilterInhalt(pid(), e.id, true);
        vorschauen.value = { ...vorschauen.value, [e.id]: URL.createObjectURL(new Blob([bytes], { type: 'image/jpeg' })) };
      } catch {
        vorschauen.value = { ...vorschauen.value, [e.id]: false };
      }
    }
  }
}

// Zur Wahl unter „Etwas freigeben": was die App nennt, dazu die eigenen
// Kalender, die hier noch nicht geteilt sind.
const kalenderWahl = ref([]);
async function waehlenOeffnen() {
  waehlen.value = true;
  kalenderWahl.value = (await teilbareKalender(pid()).catch(() => []))
    .map((k) => ({ art: 'calendar', schluessel: k.id, name: k.name, ort: t('app.projekte.ort.kalender') }));
}

// Beim Freigeben von Notiz-Mappen: dürfen Mitglieder bearbeiten?
const mitBearbeiten = ref(false);
async function freigeben(w, ja = true, bearbeiten = mitBearbeiten.value) {
  try {
    if (w.art === 'calendar') {
      await kalenderTeilen(pid(), w.schluessel);
      waehlen.value = false;
      return;
    }
    await quelle.lokalFreigeben(pid(), w.art, w.schluessel, w.name, ja, w.art === 'note_folder' && bearbeiten);
    waehlen.value = false;
    await laden();
    window.dispatchEvent(new Event(GETEILT));
  } catch (e) {
    toast.error(String(e));
  }
}

async function notizLesen(id, f) {
  try {
    notizOffen.value = { ...(await quelle.geteilteNotiz(pid(), id)), id, bearbeitbar: Boolean(f && !f.eigen && f.stufe === 'edit') };
  } catch (e) {
    toast.error(String(e));
  }
}

const bearbeitung = ref(null); // { id, geraet, titel, inhalt, stand }
let speicherUhr = null;
/*
 * GEMEINSAM SCHREIBEN (docs/plan-gemeinsam-schreiben.md): Erst der Sitzung
 * beim Gerät dessen beitreten, der freigegeben hat -- dann schreiben alle
 * zugleich. Kann es das noch nicht (ältere App), wie bisher mit Sperre.
 */
const gemeinsam = shallowRef(null); // { s, id, titel, inhalt, stand }
let gemeinsamUhr = null;
async function gemeinsamSpeichern() {
  const g = gemeinsam.value;
  if (!g || !g.s.ichSpeichere.value) return;
  clearTimeout(gemeinsamUhr);
  try {
    await g.s.speichern(g.titel, g.inhalt);
    if (gemeinsam.value === g) gemeinsam.value = { ...g, stand: 'gespeichert' };
  } catch (e) {
    if (gemeinsam.value === g) gemeinsam.value = { ...g, stand: 'fehler' };
    toast.error(String(e));
  }
}
function gemeinsamGetippt(md) {
  const g = gemeinsam.value;
  if (!g || md === g.inhalt) return;
  gemeinsam.value = { ...g, inhalt: md, stand: 'geaendert' };
  clearTimeout(gemeinsamUhr);
  gemeinsamUhr = setTimeout(gemeinsamSpeichern, 2000);
}
async function gemeinsamFertig() {
  const g = gemeinsam.value;
  if (!g) return;
  if (g.stand === 'geaendert') await gemeinsamSpeichern();
  gemeinsam.value = null;
  await g.s.schliessen().catch(() => {});
  await laden();
}
onBeforeUnmount(() => gemeinsam.value?.s.schliessen().catch(() => {}));

async function bearbeitenStarten() {
  const n = notizOffen.value;
  try {
    const s = await sitzungOeffnen({ projekt: pid(), id: n.id });
    gemeinsam.value = { s, id: n.id, titel: s.start.titel, inhalt: s.start.inhalt, stand: 'gesperrt' };
    notizOffen.value = null;
    return;
  } catch (e) {
    if (String(e).includes('older app') || String(e).includes('Read only')) {
      toast.error(String(e));
      return;
    }
    // Sonst: das Gerät drüben kann noch keine Sitzungen -- wie bisher.
  }
  try {
    const g = await quelle.notizSperren(pid(), n.id);
    bearbeitung.value = { id: n.id, geraet: g.geraet, titel: g.titel, inhalt: g.inhalt, stand: 'gesperrt' };
    notizOffen.value = null;
  } catch (e) {
    toast.error(String(e));
  }
}
async function bearbeitungSpeichern() {
  const b = bearbeitung.value;
  if (!b) return;
  clearTimeout(speicherUhr);
  b.stand = 'speichert';
  try {
    await quelle.notizSpeichern(pid(), b.id, b.geraet, b.titel, b.inhalt);
    if (bearbeitung.value === b) b.stand = 'gespeichert';
  } catch (e) {
    if (bearbeitung.value === b) b.stand = 'fehler';
    toast.error(String(e));
  }
}
function bearbeitungGetippt() {
  if (!bearbeitung.value) return;
  bearbeitung.value.stand = 'geaendert';
  clearTimeout(speicherUhr);
  speicherUhr = setTimeout(bearbeitungSpeichern, 2000);
}
async function bearbeitungFertig() {
  const b = bearbeitung.value;
  if (!b) return;
  if (b.stand === 'geaendert') await bearbeitungSpeichern();
  try {
    await quelle.notizEntsperren(pid(), b.id, b.geraet);
  } catch {
    // Läuft spätestens nach 10 Minuten von selbst ab.
  }
  bearbeitung.value = null;
  await laden();
}

/*
 * Eine geteilte Datei oder ein Bild öffnen: PDF im Betrachter, Bilder groß,
 * alles andere in den eigenen Speicher übernehmen (unter „Aus Hubs").
 */
async function oeffnen(f, e) {
  if (f.art === 'note_folder') return notizLesen(e.id, f);
  const mime = e.mime || mimeAus(e.name);
  if (mime.includes('pdf')) {
    geteiltesPdf.value = { name: e.name, id: e.id };
    return;
  }
  holt.value = e.id;
  try {
    const bytes = await quelle.geteilterInhalt(pid(), e.id, false);
    if (mime.startsWith('image/')) {
      geteiltesBild.value = { name: e.name, url: URL.createObjectURL(new Blob([bytes], { type: mime })), bytes, mime };
    } else {
      await uebernehmen(e.name, bytes, mime);
    }
    laden();
  } catch (err) {
    toast.error(String(err));
  } finally {
    holt.value = null;
  }
}
async function uebernehmen(name, bytes, mime) {
  try {
    const ort = await einsortieren(new File([bytes], name, { type: mime || mimeAus(name) }), 'Aus Hubs');
    toast.success(t('app.projekte.uebernommen', { ort }));
  } catch (e) {
    toast.error(String(e));
  }
}
function bildSchliessen() {
  if (geteiltesBild.value?.url) URL.revokeObjectURL(geteiltesBild.value.url);
  geteiltesBild.value = null;
}
</script>

<template>
  <!-- Texte seitlich so weit eingerückt wie die Reiter darüber (eine
       Flucht); die Freigaben selbst laufen als Linien von Rand zu Rand. -->
  <div class="space-y-3 md:max-w-[720px]">
    <div v-if="lokaleFreigaben === null" class="flex justify-center py-6"><Loader2 class="w-5 h-5 animate-spin text-leise" /></div>
    <template v-else>
      <p v-if="!sortiert.length" class="text-sm text-leise px-2 sm:px-3">{{ t('app.projekte.keineFreigaben') }}</p>
      <template v-for="({ f, g }, i) in sortiert" :key="(f.uuid ?? '') + f.art + (f.von ?? '') + (f.schluessel ?? '')">
      <h4 v-if="i === 0 || sortiert[i - 1].g !== g" class="flex items-center gap-1.5 text-xs font-bold uppercase tracking-wider text-leise pt-2 px-2 sm:px-3">
        <component :is="GRUPPEN_ICON[g]" class="w-4 h-4 text-marke" />{{ t(`app.teilen.gruppe.${g}`) }}
      </h4>
      <!-- Ein geteilter Kalender: antippen öffnet ihn im Kalender. -->
      <div v-if="g === 'kalender'" class="bg-flaeche border-y border-linie flex items-center gap-1">
        <button type="button" class="flex-1 py-3 pl-[1.875rem] sm:pl-[2.125rem] pr-2 sm:pr-3 flex items-center gap-2 text-left hover:bg-auflage cursor-pointer min-w-0" @click="kalenderOffen = f">
          <span class="min-w-0">
            <span class="block font-bold text-schrift truncate">{{ f.name }}</span>
            <span class="block text-xs text-leise truncate">{{ t('app.projekte.ort.kalender') }} · {{ f.eigen ? t('app.projekte.vonDir') : t('app.projekte.von', { name: f.von_name }) }}</span>
          </span>
        </button>
        <button v-if="f.eigen" type="button" class="p-2 mr-1 text-leise hover:text-rose-500 rounded-xl" :title="t('planning.kalender.beenden')" @click="kalenderZurueck(f)"><CircleX class="w-4 h-4" /></button>
      </div>
      <div v-else class="bg-flaeche border-y border-linie">
        <!-- Eingerückt bis unter das Wort der Überschrift. -->
        <div class="py-3 pl-[1.875rem] sm:pl-[2.125rem] pr-2 sm:pr-3 flex items-center gap-2 border-b border-linie">
          <span class="min-w-0">
            <span class="block font-bold text-schrift truncate">{{ f.name }}</span>
            <span class="block text-xs text-leise truncate">{{ ortText(f) }} · {{ vonText(f) }}<template v-if="f.art === 'note_folder'"> · {{ f.stufe === 'edit' ? t('app.projekte.stufeEdit') : t('app.projekte.stufeLesen') }}</template></span>
          </span>
          <span v-if="f.eigen" class="ml-auto flex flex-col items-end gap-1 shrink-0">
            <button type="button" class="text-xs font-bold text-rose-600 dark:text-rose-400 cursor-pointer" @click="freigeben(f, false)">{{ t('app.projekte.zuruecknehmen') }}</button>
            <button v-if="f.art === 'note_folder'" type="button" class="text-xs font-bold text-marke cursor-pointer"
                    @click="freigeben(f, true, f.stufe !== 'edit')">{{ f.stufe === 'edit' ? t('app.projekte.nurLesen') : t('app.projekte.bearbeitenErlauben') }}</button>
          </span>
        </div>
        <p v-if="!f.eintraege.length" class="py-3 pl-[1.875rem] sm:pl-[2.125rem] pr-2 sm:pr-3 text-xs text-leise">{{ f.eigen ? t('app.projekte.leer') : t('app.projekte.nochNichts') }}</p>

        <!-- Alben: Vorschaubilder im Raster -->
        <div v-else-if="f.art === 'album'" class="py-2 pl-[1.875rem] sm:pl-[2.125rem] pr-2 sm:pr-3 grid grid-cols-3 sm:grid-cols-4 gap-2">
          <button v-for="e in f.eintraege" :key="e.id" type="button" :title="e.pfad ? `${e.pfad}/${e.name}` : e.name"
                  class="aspect-square rounded-lg overflow-hidden bg-auflage flex items-center justify-center cursor-pointer"
                  @click="oeffnen(f, e)">
            <Loader2 v-if="holt === e.id" class="w-4 h-4 animate-spin text-leise" />
            <img v-else-if="vorschauen[e.id]" :src="vorschauen[e.id]" :alt="e.name" class="w-full h-full object-cover" />
            <ImageIcon v-else class="w-6 h-6 text-leise" />
          </button>
        </div>

        <!-- Notizen und Dateien: eine Zeile je Eintrag -->
        <template v-else>
          <button v-for="e in f.eintraege" :key="e.id" type="button"
                  class="w-full py-3 pl-[1.875rem] sm:pl-[2.125rem] pr-2 sm:pr-3 flex items-center gap-2 text-left text-sm hover:bg-auflage cursor-pointer border-t border-linie first:border-t-0"
                  @click="oeffnen(f, e)">
            <NotebookText v-if="f.art === 'note_folder'" class="w-4 h-4 text-leise shrink-0" />
            <ImageIcon v-else-if="(e.mime || '').startsWith('image/')" class="w-4 h-4 text-sky-500 shrink-0" />
            <FileText v-else class="w-4 h-4 text-rose-500 shrink-0" />
            <span class="min-w-0 flex-1">
              <span class="block truncate text-schrift">{{ e.name || t('app.projekte.ohneTitel') }}</span>
              <span v-if="e.pfad || f.art !== 'note_folder'" class="block text-xs text-leise truncate">
                <template v-if="e.pfad">{{ e.pfad }} · </template><template v-if="f.art !== 'note_folder'">{{ groesseText(e.groesse) }}{{ e.da ? '' : ` · ${t('app.projekte.nochNichtHier')}` }}</template>
              </span>
            </span>
            <Loader2 v-if="holt === e.id" class="w-4 h-4 animate-spin text-leise shrink-0" />
          </button>
        </template>
      </div>
      </template>
      <button type="button" class="knopf flex items-center gap-1.5 mx-2 sm:mx-3" @click="waehlenOeffnen">
        <Plus class="w-4 h-4" /> {{ t('app.projekte.etwasFreigeben') }}
      </button>
    </template>

    <KalenderRahmen v-if="kalenderOffen" :kalender="kalenderOffen" @close="kalenderOffen = null" />

    <!-- Etwas Eigenes freigeben: Notiz-Mappe, Ordner, Album, Kalender -->
    <BaseModal v-if="waehlen" :title="t('app.projekte.etwasFreigeben')" @close="waehlen = false">
      <div class="space-y-3">
        <p class="text-sm text-leise">{{ t('app.projekte.freigebenHinweis') }}</p>
        <label class="flex items-center gap-2 text-sm text-fliess">
          <input v-model="mitBearbeiten" type="checkbox">
          {{ t('app.projekte.mitBearbeiten') }}
        </label>
        <p v-if="!wahlSortiert.length" class="text-sm text-fliess">{{ t('app.projekte.nichtsMehr') }}</p>
        <ul v-else class="karte divide-y divide-linie max-h-[55vh] overflow-y-auto">
          <template v-for="({ w, g }, i) in wahlSortiert" :key="w.art + w.schluessel">
          <li v-if="i === 0 || wahlSortiert[i - 1].g !== g" class="flex items-center gap-1.5 px-3 pt-3 pb-1 text-xs font-bold uppercase tracking-wider text-leise">
            <component :is="GRUPPEN_ICON[g]" class="w-4 h-4 text-marke" />{{ t(`app.teilen.gruppe.${g}`) }}
          </li>
          <li>
            <button type="button" class="w-full p-3 flex items-center gap-2 text-left hover:bg-auflage cursor-pointer" @click="freigeben(w)">
              <span class="min-w-0">
                <span class="block font-bold text-schrift truncate">{{ w.name }}</span>
                <span class="block text-xs text-leise truncate">{{ w.ort }}</span>
              </span>
            </button>
          </li>
          </template>
        </ul>
      </div>
    </BaseModal>

    <!-- Ein geteiltes Bild groß; Zurück schließt es. -->
    <ZurueckSchliesst v-if="geteiltesBild" @schliessen="bildSchliessen" />
    <div v-if="geteiltesBild" class="fixed inset-0 z-[110] bg-black/90 flex flex-col" @click.self="bildSchliessen">
      <div class="flex items-center justify-between gap-2 p-3 pt-[max(0.75rem,env(safe-area-inset-top))] mt-6 text-white">
        <span class="text-sm truncate">{{ geteiltesBild.name }}</span>
        <div class="flex gap-4 text-sm font-bold">
          <button type="button" class="cursor-pointer" @click="uebernehmen(geteiltesBild.name, geteiltesBild.bytes, geteiltesBild.mime)">{{ t('app.projekte.uebernehmen') }}</button>
          <button type="button" class="cursor-pointer" @click="bildSchliessen">{{ t('common.close') }}</button>
        </div>
      </div>
      <img :src="geteiltesBild.url" :alt="geteiltesBild.name" class="flex-1 min-h-0 object-contain" />
    </div>

    <!-- Ein geteiltes PDF -->
    <PdfBetrachter v-if="geteiltesPdf" :name="geteiltesPdf.name"
                   :laden="() => quelle.geteilterInhalt(project.id, geteiltesPdf.id, false)" @close="geteiltesPdf = null" />

    <!-- Eine geteilte Notiz lesen -->
    <BaseModal v-if="notizOffen" :title="notizOffen.titel || t('app.projekte.ohneTitel')" size="lg" @close="notizOffen = null">
      <div class="mb-2 flex items-center gap-2">
        <p v-if="notizOffen.von" class="text-xs text-leise">{{ t('app.projekte.von', { name: notizOffen.von }) }}{{ notizOffen.bearbeitbar ? '' : ` · ${t('app.projekte.stufeLesen')}` }}</p>
        <button v-if="notizOffen.bearbeitbar" type="button" class="ml-auto knopf-wichtig text-sm" @click="bearbeitenStarten">{{ t('app.projekte.bearbeiten') }}</button>
      </div>
      <div class="text-sm text-schrift whitespace-pre-wrap break-words">{{ notizOffen.inhalt }}</div>
    </BaseModal>

    <!-- Eine fremde Notiz gemeinsam bearbeiten (Sitzung beim Gerät dessen, der sie freigegeben hat) -->
    <BaseModal v-if="gemeinsam" :title="gemeinsam.titel || t('app.projekte.notizBearbeiten')" size="lg" persistent @close="gemeinsamFertig">
      <div class="space-y-2">
        <Anwesende :teilnehmer="gemeinsam.s.anwesende.value" :ich="gemeinsam.s.kennung" :aus="gemeinsam.s.aus.value" />
        <MarkdownEditor
          :model-value="gemeinsam.inhalt"
          :sitzung="gemeinsam.s.editor"
          :read-only="Boolean(gemeinsam.s.aus.value)"
          :einfuegbar="false"
          class="min-h-[45vh]"
          @update:model-value="gemeinsamGetippt"
        />
        <p v-if="gemeinsam.s.ichSpeichere.value" class="text-xs text-leise">
          {{ t(`app.projekte.stand.${gemeinsam.stand}`) }}
        </p>
      </div>
      <template #footer>
        <BaseButton @click="gemeinsamFertig">{{ t('app.projekte.fertig') }}</BaseButton>
      </template>
    </BaseModal>

    <!-- Eine fremde Notiz bearbeiten (gesperrt beim Gerät dessen, der sie freigegeben hat) -->
    <BaseModal v-if="bearbeitung" :title="t('app.projekte.notizBearbeiten')" size="lg" persistent @close="bearbeitungFertig">
      <div class="space-y-2">
        <input v-model="bearbeitung.titel" class="feld w-full font-bold" :placeholder="t('app.projekte.titel')" @input="bearbeitungGetippt">
        <textarea v-model="bearbeitung.inhalt" class="feld w-full min-h-[45vh] font-mono text-sm" @input="bearbeitungGetippt" />
        <p class="text-xs text-leise">
          {{ t(`app.projekte.stand.${bearbeitung.stand}`) }}
        </p>
      </div>
      <template #footer>
        <BaseButton @click="bearbeitungFertig">{{ t('app.projekte.fertig') }}</BaseButton>
      </template>
    </BaseModal>
  </div>
</template>
