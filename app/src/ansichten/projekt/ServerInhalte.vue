<script setup>
/*
 * Freigaben eines Server-Projekts: Notiz-Mappen, Alben und Ordner, alles
 * Ende-zu-Ende (docs/plan-notizen-e2e.md 6c, docs/plan-dateien-e2e.md 7c).
 */
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';
import { useI18n } from 'vue-i18n';
import { Folder, FolderArchive, Image as ImageIcon, NotebookText, CalendarDays, Plus, CircleX } from 'lucide-vue-next';
import BaseModal from '@oberflaeche/base/BaseModal.vue';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import { lokaleNotizenQuelle } from '../../quellen/notizen';
import MappeBetrachter from './MappeBetrachter.vue';
import GeteilteSache from './GeteilteSache.vue';
import KalenderRahmen from './KalenderRahmen.vue';
import { invoke } from '@tauri-apps/api/core';
import { useToast } from '@oberflaeche/composables/useToast';
import { projekteQuelle as quelle } from '../../quellen/projekte';
import { teilbareKalender, kalenderTeilen, GETEILT, geteilteKalender, kalenderBeenden } from '../../quellen/kalenderTeilen';

const props = defineProps({ project: { type: Object, required: true } });
const toast = useToast();
const { t } = useI18n();

const { confirmDialog } = useConfirm();

/*
 * GETEILT, Ende-zu-Ende: Notiz-Mappen (docs/plan-notizen-e2e.md, 6c), Alben
 * und Ordner (docs/plan-dateien-e2e.md, 7c). Sie kommen über einen eigenen
 * Weg (mappenteilen.rs, dateiteilen.rs) und öffnen sich hier im Projekt.
 * Geteilt wird hier: ein „+ Teilen", erst die Art, dann was -- auch
 * Kalender. Die Liste steht nach Arten: Mappen, Kalender, Alben, Ordner,
 * Akten.
 */
const mappen = ref([]);
const mappeOffen = ref(null);
const sacheOffen = ref(null);
const kalenderOffen = ref(null);
const teilenOffen = ref(false);
const art = ref('mappe');
const auswahl = ref({ mappe: [], kalender: [], album: [], ordner: [], akte: [] }); // je Art: [{ id, pfad }]
const gewaehlt = ref(null);
const stufe = ref('view');
const teilt = ref(false);

const kalender = ref([]);

async function mappenLaden() {
  const [m, k] = await Promise.all([
    quelle.mappen(props.project.id).then((r) => r.mappen ?? []).catch(() => []),
    geteilteKalender(props.project.id),
  ]);
  mappen.value = m;
  kalender.value = k;
}

// Die Arten in der Reihenfolge der Überschriften. Ein Ordner ohne Zone
// gilt als Ordner.
const ARTEN = ['mappe', 'kalender', 'album', 'ordner', 'akte'];
const artVon = (m) => (m.art === 'ordner' && m.zone === 'documents' ? 'akte' : m.art);
const gruppen = computed(() => ARTEN
  .map((a) => ({ art: a, eintraege: a === 'kalender' ? kalender.value : mappen.value.filter((m) => artVon(m) === a) }))
  .filter((g) => g.eintraege.length));

onMounted(() => {
  mappenLaden();
  window.addEventListener(GETEILT, mappenLaden);
});
onBeforeUnmount(() => window.removeEventListener(GETEILT, mappenLaden));

// Eigenes mit ganzem Pfad, zum Auswählen: [{ id, name, parent_id }] -> [{ id, pfad }].
function mitPfad(liste, vorne = '') {
  const nachId = new Map(liste.map((m) => [m.id, m]));
  const pfad = (m) => (m.parent_id && nachId.has(m.parent_id) ? `${pfad(nachId.get(m.parent_id))} / ${m.name}` : m.name);
  return liste.map((m) => ({ id: m.id, pfad: vorne + pfad(m) })).sort((a, b) => a.pfad.localeCompare(b.pfad, 'de'));
}
const ohneFehler = (p, leer) => p.catch(() => leer);

async function teilenOeffnen() {
  const [mappenListe, alben, dateien, akten, kalender] = await Promise.all([
    ohneFehler(lokaleNotizenQuelle().getNoteFolders().then((r) => r.data.folders ?? []), []),
    ohneFehler(invoke('galerie_albenbaum').then((r) => r.albums ?? []), []),
    ohneFehler(invoke('dateien_baum', { zone: 'files' }).then((r) => r.folders ?? []), []),
    ohneFehler(invoke('dateien_baum', { zone: 'documents' }).then((r) => r.folders ?? []), []),
    ohneFehler(teilbareKalender(props.project.id, { ohneAbos: true }), []),
  ]);
  auswahl.value = {
    mappe: mitPfad(mappenListe),
    album: mitPfad(alben),
    ordner: mitPfad(dateien),
    akte: mitPfad(akten),
    kalender: kalender.map((k) => ({ id: k.id, pfad: k.name })),
  };
  art.value = 'mappe';
  artWaehlen();
  teilenOffen.value = true;
}

function artWaehlen() {
  gewaehlt.value = auswahl.value[art.value][0]?.id ?? null;
  stufe.value = 'view';
}

async function teilen() {
  if (gewaehlt.value == null) return;
  teilt.value = true;
  try {
    if (art.value === 'mappe') await quelle.mappeTeilen(props.project.id, gewaehlt.value, stufe.value);
    else if (art.value === 'kalender') await kalenderTeilen(props.project.id, gewaehlt.value, stufe.value);
    else await quelle.sacheTeilen(props.project.id, art.value === 'akte' ? 'ordner' : art.value, gewaehlt.value);
    teilenOffen.value = false;
    await mappenLaden();
    window.dispatchEvent(new Event(GETEILT));
  } catch (e) {
    toast.error(t('app.teilen.fehler', { grund: String(e) }));
  } finally {
    teilt.value = false;
  }
}

// Das Symbol steht an der Überschrift der Art, nicht an jedem Eintrag.
const artIcon = (m) => ({ album: ImageIcon, ordner: Folder, akte: FolderArchive, kalender: CalendarDays }[m.art] ?? NotebookText);
const anzahlText = (m) => (m.art === 'album' ? t('app.teilen.bilder', m.anzahl ?? 0) : m.art === 'ordner' ? t('app.teilen.dateien', m.anzahl ?? 0) : t('app.mappe.anzahl', m.anzahl ?? 0));
const stufeText = (m) => (['mappe', 'kalender'].includes(m.art) && m.stufe === 'edit' ? t('app.mappe.bearbeiten') : t('app.mappe.ansehen'));

function geteiltOeffnen(m) {
  if (m.art === 'kalender') kalenderOffen.value = m;
  else if (m.art === 'album' || m.art === 'ordner') sacheOffen.value = m;
  else mappeOffen.value = m;
}

async function beenden(m) {
  if (m.art === 'kalender') {
    try {
      if (await kalenderBeenden(props.project.id, m, confirmDialog, t)) await mappenLaden();
    } catch (e) {
      toast.error(String(e));
    }
    return;
  }
  const frage = m.art === 'mappe' ? t('app.mappe.beendenFrage', { name: m.name ?? '…' }) : t('app.teilen.beendenFrage', { name: m.name ?? '…' });
  if (!(await confirmDialog(frage, { title: t('app.mappe.beenden'), confirmLabel: t('app.mappe.beenden') }))) return;
  try {
    await quelle.mappeBeenden(props.project.id, m.uuid);
    await mappenLaden();
    window.dispatchEvent(new Event(GETEILT));
  } catch (e) {
    toast.error(String(e));
  }
}

</script>

<template>
  <!-- Texte seitlich so weit eingerückt wie die Reiter darüber (eine
       Flucht); die Listen selbst laufen als Linien von Rand zu Rand. -->
  <div class="md:max-w-[720px]">
    <!-- Geteilt (Ende-zu-Ende): Notiz-Mappen, Alben, Ordner -->
    <div class="flex items-center gap-2 mb-2 px-2 sm:px-3">
      <h3 class="text-sm font-bold text-schrift">{{ t('app.teilen.titel') }}</h3>
      <BaseButton class="ml-auto" groesse="kopf" @click="teilenOeffnen"><Plus class="w-4 h-4" /> {{ t('app.teilen.knopf') }}</BaseButton>
    </div>
    <p v-if="!gruppen.length" class="text-sm text-leise mb-6 px-2 sm:px-3">{{ t('app.teilen.keine') }}</p>
    <section v-for="g in gruppen" :key="g.art" class="mb-5">
      <h4 class="flex items-center gap-1.5 text-xs font-bold uppercase tracking-wider text-leise mb-1.5 px-2 sm:px-3">
        <component :is="artIcon({ art: g.art })" class="w-4 h-4 text-marke" />{{ t(`app.teilen.gruppe.${g.art}`) }}
      </h4>
      <div class="bg-flaeche border-y border-linie divide-y divide-linie">
        <div v-for="m in g.eintraege" :key="m.uuid" class="flex items-center gap-1">
          <!-- Eingerückt bis unter das Wort der Überschrift. -->
          <button type="button" class="flex-1 flex items-start gap-3 py-3 pl-[1.875rem] sm:pl-[2.125rem] pr-2 sm:pr-3 text-left hover:bg-auflage transition-colors min-w-0" :disabled="!m.name" @click="geteiltOeffnen(m)">
            <span class="flex-1 min-w-0">
              <span class="block text-sm font-medium text-schrift truncate">{{ m.name ?? (m.art === 'mappe' ? t('app.mappe.verschlossen') : t('app.teilen.verschlossen')) }}</span>
              <span class="block text-xs text-leise">
                {{ m.eigen ? t('app.projekte.vonDir') : t('app.projekte.von', { name: m.von_name }) }} · {{ stufeText(m) }}<template v-if="m.art !== 'kalender'"> · {{ anzahlText(m) }}</template>
              </span>
            </span>
          </button>
          <button v-if="m.eigen" type="button" class="p-2 mr-1 text-leise hover:text-rose-500 rounded-xl" :title="t('app.mappe.beenden')" @click="beenden(m)"><CircleX class="w-4 h-4" /></button>
        </div>
      </div>
    </section>

    <BaseModal v-if="teilenOffen" :title="t('app.teilen.dialogTitel')" size="sm" @close="teilenOffen = false">
      <div class="flex flex-wrap gap-2 mb-3" role="radiogroup">
        <button
          v-for="a in ARTEN" :key="a" type="button" role="radio" :aria-checked="art === a"
          class="flex items-center gap-1.5 rounded-xl border px-3 py-1.5 text-sm cursor-pointer"
          :class="art === a ? 'border-marke bg-marke/10 text-marke font-bold' : 'border-linie text-schrift'"
          @click="art = a; artWaehlen()"
        >
          <component :is="artIcon({ art: a })" class="w-4 h-4" />
          {{ t({ mappe: 'app.teilen.artMappe', kalender: 'app.teilen.artKalender', album: 'app.teilen.artAlbum', ordner: 'app.teilen.artOrdner', akte: 'app.teilen.artAkte' }[a]) }}
        </button>
      </div>
      <p class="text-sm text-leise mb-3">{{ t({ mappe: 'app.mappe.teilenHinweis', album: 'app.teilen.hinweisAlbum', ordner: 'app.teilen.hinweisOrdner', akte: 'app.teilen.hinweisOrdner', kalender: 'planning.kalender.teilenHinweis' }[art]) }}</p>
      <template v-if="auswahl[art].length">
        <select v-model="gewaehlt" class="w-full rounded-xl border border-linie p-2 mb-3 bg-vertieft text-schrift">
          <option v-for="m in auswahl[art]" :key="m.id" :value="m.id">{{ m.pfad }}</option>
        </select>
        <select v-if="art === 'mappe' || art === 'kalender'" v-model="stufe" class="w-full rounded-xl border border-linie p-2 bg-vertieft text-schrift">
          <option value="view">{{ t('app.mappe.ansehen') }}</option>
          <option value="edit">{{ t('app.mappe.bearbeiten') }}</option>
        </select>
      </template>
      <p v-else class="text-sm text-leise">{{ t({ mappe: 'app.mappe.keineEigenen', album: 'app.teilen.keineAlben', ordner: 'app.teilen.keineOrdner', akte: 'app.teilen.keineAkten', kalender: 'app.teilen.keineKalender' }[art]) }}</p>
      <template #footer>
        <BaseButton variant="ghost" @click="teilenOffen = false">{{ t('common.cancel') }}</BaseButton>
        <BaseButton :loading="teilt" :disabled="gewaehlt == null" @click="teilen">{{ t('app.teilen.knopf') }}</BaseButton>
      </template>
    </BaseModal>
    <MappeBetrachter v-if="mappeOffen" :projekt="project.id" :mappe="mappeOffen" @close="mappeOffen = null; mappenLaden()" />
    <GeteilteSache v-if="sacheOffen" :projekt="project.id" :sache="sacheOffen" @close="sacheOffen = null" />
    <KalenderRahmen v-if="kalenderOffen" :kalender="kalenderOffen" @close="kalenderOffen = null" />
  </div>
</template>
