<script setup>
/*
 * Freigaben eines Server-Projekts: Notiz-Mappen, Alben und Ordner, alles
 * Ende-zu-Ende (docs/plan-notizen-e2e.md 6c, docs/plan-dateien-e2e.md 7c).
 * Die alten, lesbaren Freigaben gibt es seit 7d nicht mehr.
 */
import { ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { Folder, Image as ImageIcon, NotebookText, Plus, Trash2, Lock } from 'lucide-vue-next';
import BaseModal from '@oberflaeche/base/BaseModal.vue';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import { lokaleNotizenQuelle } from '../../quellen/notizen';
import MappeBetrachter from './MappeBetrachter.vue';
import GeteilteSache from './GeteilteSache.vue';
import { invoke } from '@tauri-apps/api/core';
import { useToast } from '@oberflaeche/composables/useToast';
import { projekteQuelle as quelle } from '../../quellen/projekte';

const props = defineProps({ project: { type: Object, required: true } });
const toast = useToast();
const { t } = useI18n();

const { confirmDialog } = useConfirm();

/*
 * GETEILT, Ende-zu-Ende: Notiz-Mappen (docs/plan-notizen-e2e.md, 6c), Alben
 * und Ordner (docs/plan-dateien-e2e.md, 7c). Sie kommen über einen eigenen
 * Weg (mappenteilen.rs, dateiteilen.rs) und öffnen sich hier im Projekt.
 * Geteilt wird hier: ein „+ Teilen", erst die Art, dann was (Tiffy, 20.10.2026).
 */
const mappen = ref([]);
const mappeOffen = ref(null);
const sacheOffen = ref(null);
const teilenOffen = ref(false);
const art = ref('mappe');
const auswahl = ref({ mappe: [], album: [], ordner: [] }); // je Art: [{ id, pfad }]
const gewaehlt = ref(null);
const stufe = ref('view');
const teilt = ref(false);

async function mappenLaden() {
  try {
    mappen.value = (await quelle.mappen(props.project.id)).mappen ?? [];
  } catch {
    mappen.value = [];
  }
}

onMounted(mappenLaden);

// Eigenes mit ganzem Pfad, zum Auswählen: [{ id, name, parent_id }] -> [{ id, pfad }].
function mitPfad(liste, vorne = '') {
  const nachId = new Map(liste.map((m) => [m.id, m]));
  const pfad = (m) => (m.parent_id && nachId.has(m.parent_id) ? `${pfad(nachId.get(m.parent_id))} / ${m.name}` : m.name);
  return liste.map((m) => ({ id: m.id, pfad: vorne + pfad(m) })).sort((a, b) => a.pfad.localeCompare(b.pfad, 'de'));
}
const ohneFehler = (p, leer) => p.catch(() => leer);

async function teilenOeffnen() {
  const [mappenListe, alben, dateien, akten] = await Promise.all([
    ohneFehler(lokaleNotizenQuelle().getNoteFolders().then((r) => r.data.folders ?? []), []),
    ohneFehler(invoke('galerie_albenbaum').then((r) => r.albums ?? []), []),
    ohneFehler(invoke('dateien_baum', { zone: 'files' }).then((r) => r.folders ?? []), []),
    ohneFehler(invoke('dateien_baum', { zone: 'documents' }).then((r) => r.folders ?? []), []),
  ]);
  auswahl.value = {
    mappe: mitPfad(mappenListe),
    album: mitPfad(alben),
    ordner: [...mitPfad(dateien, `${t('app.teilen.dateienOrt')} / `), ...mitPfad(akten, `${t('app.teilen.aktenOrt')} / `)],
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
    else await quelle.sacheTeilen(props.project.id, art.value, gewaehlt.value);
    teilenOffen.value = false;
    await mappenLaden();
  } catch (e) {
    toast.error(t('app.teilen.fehler', { grund: String(e) }));
  } finally {
    teilt.value = false;
  }
}

const artIcon = (m) => ({ album: ImageIcon, ordner: Folder }[m.art] ?? NotebookText);
const anzahlText = (m) => (m.art === 'album' ? t('app.teilen.bilder', m.anzahl ?? 0) : m.art === 'ordner' ? t('app.teilen.dateien', m.anzahl ?? 0) : t('app.mappe.anzahl', m.anzahl ?? 0));
const stufeText = (m) => (m.art === 'mappe' && m.stufe === 'edit' ? t('app.mappe.bearbeiten') : t('app.mappe.ansehen'));

function geteiltOeffnen(m) {
  if (m.art === 'album' || m.art === 'ordner') sacheOffen.value = m;
  else mappeOffen.value = m;
}

async function beenden(m) {
  const frage = m.art === 'mappe' ? t('app.mappe.beendenFrage', { name: m.name ?? '…' }) : t('app.teilen.beendenFrage', { name: m.name ?? '…' });
  if (!(await confirmDialog(frage, { title: t('app.mappe.beenden'), confirmLabel: t('app.mappe.beenden') }))) return;
  try {
    await quelle.mappeBeenden(props.project.id, m.uuid);
    await mappenLaden();
  } catch (e) {
    toast.error(String(e));
  }
}

</script>

<template>
  <div class="md:max-w-[720px]">
    <!-- Geteilt (Ende-zu-Ende): Notiz-Mappen, Alben, Ordner -->
    <div class="flex items-center gap-2 mb-2">
      <h3 class="text-sm font-bold text-schrift">{{ t('app.teilen.titel') }}</h3>
      <BaseButton class="ml-auto" groesse="kopf" @click="teilenOeffnen"><Plus class="w-4 h-4" /> {{ t('app.teilen.knopf') }}</BaseButton>
    </div>
    <p v-if="!mappen.length" class="text-sm text-leise mb-6">{{ t('app.teilen.keine') }}</p>
    <div v-else class="karte shadow-sm divide-y divide-linie mb-6">
      <div v-for="m in mappen" :key="m.uuid" class="flex items-center gap-1">
        <button type="button" class="flex-1 flex items-start gap-3 p-3 text-left hover:bg-auflage transition-colors min-w-0" :disabled="!m.name" @click="geteiltOeffnen(m)">
          <component :is="artIcon(m)" class="w-5 h-5 mt-0.5 shrink-0 text-marke" />
          <span class="flex-1 min-w-0">
            <span class="block text-sm font-medium text-schrift truncate">{{ m.name ?? (m.art === 'mappe' ? t('app.mappe.verschlossen') : t('app.teilen.verschlossen')) }}</span>
            <span class="block text-xs text-leise">
              {{ m.eigen ? t('app.projekte.vonDir') : t('app.projekte.von', { name: m.von_name }) }} · {{ stufeText(m) }} · {{ anzahlText(m) }}
            </span>
          </span>
          <Lock class="w-3.5 h-3.5 mt-1 shrink-0 text-leise" :title="t('app.mappe.verschluesselt')" />
        </button>
        <button v-if="m.eigen" type="button" class="p-2 mr-1 text-leise hover:text-rose-500 rounded-xl" :title="t('app.mappe.beenden')" @click="beenden(m)"><Trash2 class="w-4 h-4" /></button>
      </div>
    </div>

    <BaseModal v-if="teilenOffen" :title="t('app.teilen.dialogTitel')" size="sm" @close="teilenOffen = false">
      <div class="flex flex-wrap gap-2 mb-3" role="radiogroup">
        <button
          v-for="a in ['mappe', 'album', 'ordner']" :key="a" type="button" role="radio" :aria-checked="art === a"
          class="flex items-center gap-1.5 rounded-xl border px-3 py-1.5 text-sm cursor-pointer"
          :class="art === a ? 'border-marke bg-marke/10 text-marke font-bold' : 'border-linie text-schrift'"
          @click="art = a; artWaehlen()"
        >
          <component :is="artIcon({ art: a })" class="w-4 h-4" />
          {{ t({ mappe: 'app.teilen.artMappe', album: 'app.teilen.artAlbum', ordner: 'app.teilen.artOrdner' }[a]) }}
        </button>
      </div>
      <p class="text-sm text-leise mb-3">{{ t({ mappe: 'app.mappe.teilenHinweis', album: 'app.teilen.hinweisAlbum', ordner: 'app.teilen.hinweisOrdner' }[art]) }}</p>
      <template v-if="auswahl[art].length">
        <select v-model="gewaehlt" class="w-full rounded-xl border border-linie p-2 mb-3 bg-vertieft text-schrift">
          <option v-for="m in auswahl[art]" :key="m.id" :value="m.id">{{ m.pfad }}</option>
        </select>
        <select v-if="art === 'mappe'" v-model="stufe" class="w-full rounded-xl border border-linie p-2 bg-vertieft text-schrift">
          <option value="view">{{ t('app.mappe.ansehen') }}</option>
          <option value="edit">{{ t('app.mappe.bearbeiten') }}</option>
        </select>
      </template>
      <p v-else class="text-sm text-leise">{{ t({ mappe: 'app.mappe.keineEigenen', album: 'app.teilen.keineAlben', ordner: 'app.teilen.keineOrdner' }[art]) }}</p>
      <template #footer>
        <BaseButton variant="ghost" @click="teilenOffen = false">{{ t('common.cancel') }}</BaseButton>
        <BaseButton :loading="teilt" :disabled="gewaehlt == null" @click="teilen">{{ t('app.teilen.knopf') }}</BaseButton>
      </template>
    </BaseModal>
    <MappeBetrachter v-if="mappeOffen" :projekt="project.id" :mappe="mappeOffen" @close="mappeOffen = null; mappenLaden()" />
    <GeteilteSache v-if="sacheOffen" :projekt="project.id" :sache="sacheOffen" @close="sacheOffen = null" />
  </div>
</template>
