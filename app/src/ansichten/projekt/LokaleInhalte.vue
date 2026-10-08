<script setup>
/*
 * Freigaben in Projekte ohne Server (01.10.2026): Notiz-Mappen, Ordner und
 * Alben. Fremde liegen im Projekt und kommen beim Abgleich vor Ort -- direkt
 * vom Gerät dessen, der freigegeben hat. Dateien und Originalbilder kommen
 * beim ersten Öffnen und bleiben dann hier.
 *
 * FREMDE NOTIZEN BEARBEITEN (strenge Sperre): Die Sperre kommt vom Gerät
 * dessen, der freigegeben hat; dort wird gespeichert. Gespeichert wird 2 s
 * nach dem letzten Tippen -- das verlängert auch die Sperre (sie läuft nach
 * 10 Minuten ohne Tippen ab). „Fertig" lässt sie los.
 */
import { ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { Plus, Folder, FileText, Image as ImageIcon, NotebookText, Loader2 } from 'lucide-vue-next';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import BaseModal from '@oberflaeche/base/BaseModal.vue';
import PdfBetrachter from '@oberflaeche/speicher/PdfBetrachter.vue';
import { useToast } from '@oberflaeche/composables/useToast';
import { projekteQuelle as quelle } from '../../quellen/projekte';
import { einsortieren, mimeAus } from '../../quellen/ablage';

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
onMounted(laden);

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

// Beim Freigeben von Notiz-Mappen: dürfen Mitglieder bearbeiten?
const mitBearbeiten = ref(false);
async function freigeben(w, ja = true, bearbeiten = mitBearbeiten.value) {
  try {
    await quelle.lokalFreigeben(pid(), w.art, w.schluessel, w.name, ja, w.art === 'note_folder' && bearbeiten);
    waehlen.value = false;
    await laden();
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
async function bearbeitenStarten() {
  const n = notizOffen.value;
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
 * alles andere in den eigenen Speicher übernehmen (unter „Aus Projekten").
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
    const ort = await einsortieren(new File([bytes], name, { type: mime || mimeAus(name) }), 'Aus Projekten');
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
  <div class="space-y-3 md:max-w-[720px]">
    <div v-if="lokaleFreigaben === null" class="flex justify-center py-6"><Loader2 class="w-5 h-5 animate-spin text-leise" /></div>
    <template v-else>
      <p v-if="!lokaleFreigaben.freigaben.length" class="text-sm text-leise">{{ t('app.projekte.keineFreigaben') }}</p>
      <div v-for="f in lokaleFreigaben.freigaben" :key="f.art + f.von + f.schluessel" class="karte shadow-sm">
        <div class="p-3 flex items-center gap-2 border-b border-linie">
          <component :is="ARTEN[f.art]?.icon ?? Folder" class="w-4 h-4 text-marke shrink-0" />
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
        <p v-if="!f.eintraege.length" class="p-3 text-xs text-leise">{{ f.eigen ? t('app.projekte.leer') : t('app.projekte.nochNichts') }}</p>

        <!-- Alben: Vorschaubilder im Raster -->
        <div v-else-if="f.art === 'album'" class="p-2 grid grid-cols-3 sm:grid-cols-4 gap-2">
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
                  class="w-full p-3 flex items-center gap-2 text-left text-sm hover:bg-auflage cursor-pointer border-t border-linie first:border-t-0"
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
      <button type="button" class="knopf flex items-center gap-1.5" @click="waehlen = true">
        <Plus class="w-4 h-4" /> {{ t('app.projekte.etwasFreigeben') }}
      </button>
    </template>

    <!-- Etwas Eigenes freigeben: Notiz-Mappe, Ordner, Album -->
    <BaseModal v-if="waehlen" :title="t('app.projekte.etwasFreigeben')" @close="waehlen = false">
      <div class="space-y-3">
        <p class="text-sm text-leise">{{ t('app.projekte.freigebenHinweis') }}</p>
        <label class="flex items-center gap-2 text-sm text-fliess">
          <input v-model="mitBearbeiten" type="checkbox">
          {{ t('app.projekte.mitBearbeiten') }}
        </label>
        <p v-if="!lokaleFreigaben?.waehlbar?.length" class="text-sm text-fliess">{{ t('app.projekte.nichtsMehr') }}</p>
        <ul v-else class="karte divide-y divide-linie max-h-[55vh] overflow-y-auto">
          <li v-for="w in lokaleFreigaben.waehlbar" :key="w.art + w.schluessel">
            <button type="button" class="w-full p-3 flex items-center gap-2 text-left hover:bg-auflage cursor-pointer" @click="freigeben(w)">
              <component :is="ARTEN[w.art]?.icon ?? Folder" class="w-4 h-4 text-marke shrink-0" />
              <span class="min-w-0">
                <span class="block font-bold text-schrift truncate">{{ w.name }}</span>
                <span class="block text-xs text-leise truncate">{{ w.ort }}</span>
              </span>
            </button>
          </li>
        </ul>
      </div>
    </BaseModal>

    <!-- Ein geteiltes Bild groß -->
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
