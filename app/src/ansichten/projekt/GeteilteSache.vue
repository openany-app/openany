<script setup>
/*
 * Ein geteiltes Album oder ein geteilter Ordner im Projekt, Ende-zu-Ende
 * (docs/plan-dateien-e2e.md, 7c). NUR ANSEHEN: Die Köpfe kommen verschlüsselt
 * vom Server und gehen auf diesem Gerät auf (dateiteilen.rs); Vorschauen und
 * Originale holt die App erst, wenn sie gebraucht werden, und legt sie in
 * ihre Ablage. „Übernehmen" legt eine eigene Kopie unter „Aus Projekten" ab.
 */
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';
import { useI18n } from 'vue-i18n';
import { convertFileSrc } from '@tauri-apps/api/core';
import { Loader2, AlertTriangle, Folder, FileText, Image as ImageIcon, ChevronLeft, Download, Lock } from 'lucide-vue-next';
import BaseModal from '@oberflaeche/base/BaseModal.vue';
import PdfBetrachter from '@oberflaeche/speicher/PdfBetrachter.vue';
import { useToast } from '@oberflaeche/composables/useToast';
import { projekteQuelle as quelle } from '../../quellen/projekte';
import { einsortieren, mimeAus } from '../../quellen/ablage';

const { t } = useI18n();
const props = defineProps({
  projekt: { type: String, required: true }, // uuid
  sache: { type: Object, required: true }, // { uuid, art, name, von_name, eigen }
});
const emit = defineEmits(['close']);
const toast = useToast();

const stand = ref(null); // Antwort von projekt_freigabe_sachen
const fehler = ref('');
const weg = ref([]); // [{ uuid, name }], die Wurzel zuerst
const hier = computed(() => weg.value[weg.value.length - 1]);
const vorschauen = ref({}); // uuid -> Adresse, oder false
const bild = ref(null); // { name, url, uuid, mime }
const pdf = ref(null); // { name, uuid }
const arbeitet = ref(null);

const istAlbum = computed(() => props.sache.art === 'album');
const mime = (d) => d.mime || mimeAus(d.name);
const groesse = (n) => (n == null ? '' : n < 1024 * 1024 ? `${Math.max(1, Math.round(n / 1024))} KB` : `${(n / 1024 / 1024).toFixed(1)} MB`);

async function laden() {
  fehler.value = '';
  try {
    stand.value = await quelle.freigabeSachen(props.projekt, props.sache.uuid);
    weg.value = [{ uuid: stand.value.wurzel, name: stand.value.name }];
    vorschauenLaden();
  } catch (e) {
    fehler.value = String(e);
  }
}

// Was in der aktuellen Ebene liegt: Unteralben bzw. Unterordner zuerst.
const ebenen = computed(() => {
  if (!stand.value || !hier.value) return [];
  const liste = istAlbum.value ? stand.value.alben : stand.value.dateien.filter((d) => d.ordner);
  return liste.filter((x) => x.eltern === hier.value.uuid).sort((a, b) => String(a.name).localeCompare(String(b.name), 'de'));
});
const bilder = computed(() => (istAlbum.value && stand.value ? stand.value.bilder.filter((b) => b.album === hier.value?.uuid) : []));
const dateien = computed(() => (!istAlbum.value && stand.value
  ? stand.value.dateien.filter((d) => !d.ordner && d.eltern === hier.value?.uuid).sort((a, b) => String(a.name).localeCompare(String(b.name), 'de'))
  : []));

async function vorschauenLaden() {
  for (const b of bilder.value) {
    if (vorschauen.value[b.uuid] !== undefined) continue;
    if (b.vorschau_pfad) {
      vorschauen.value = { ...vorschauen.value, [b.uuid]: convertFileSrc(b.vorschau_pfad) };
      continue;
    }
    if (!b.hat_vorschau) {
      vorschauen.value = { ...vorschauen.value, [b.uuid]: false };
      continue;
    }
    try {
      const pfad = await quelle.freigabeHolen(props.projekt, props.sache.uuid, b.uuid, true);
      vorschauen.value = { ...vorschauen.value, [b.uuid]: convertFileSrc(pfad) };
    } catch {
      vorschauen.value = { ...vorschauen.value, [b.uuid]: false };
    }
  }
}

function hinein(x) {
  weg.value = [...weg.value, { uuid: x.uuid, name: x.name }];
  vorschauenLaden();
}

function zurueck() {
  weg.value = weg.value.slice(0, -1);
  vorschauenLaden();
}

async function bildOeffnen(b) {
  arbeitet.value = b.uuid;
  try {
    const pfad = await quelle.freigabeHolen(props.projekt, props.sache.uuid, b.uuid, false);
    bild.value = { name: b.name, url: convertFileSrc(pfad), uuid: b.uuid, mime: mime(b) };
  } catch (e) {
    toast.error(String(e));
  } finally {
    arbeitet.value = null;
  }
}

async function oeffnen(d) {
  const m = mime(d);
  if (m === 'application/pdf') {
    pdf.value = { name: d.name, uuid: d.uuid };
  } else if (m.startsWith('image/')) {
    await bildOeffnen(d);
  } else {
    await uebernehmen(d);
  }
}

async function uebernehmen(d) {
  arbeitet.value = d.uuid;
  try {
    const bytes = await quelle.freigabeBytes(props.projekt, props.sache.uuid, d.uuid);
    const ort = await einsortieren(new File([bytes], d.name, { type: mime(d) }), 'Aus Projekten');
    toast.success(t('app.teilen.uebernommen', { ort }));
  } catch (e) {
    toast.error(String(e));
  } finally {
    arbeitet.value = null;
  }
}

onMounted(laden);
onBeforeUnmount(() => { bild.value = null; });
</script>

<template>
  <BaseModal :title="hier?.name ?? sache.name ?? '…'" size="lg" @close="emit('close')">
    <div class="space-y-3">
      <p class="text-xs text-leise flex items-center gap-1">
        <Lock class="w-3 h-3" />
        {{ sache.eigen ? t('app.projekte.vonDir') : t('app.projekte.von', { name: sache.von_name }) }} · {{ t('app.teilen.nurAnsehen') }}
      </p>
      <button v-if="weg.length > 1" type="button" class="flex items-center gap-1 text-sm font-bold text-marke cursor-pointer" @click="zurueck">
        <ChevronLeft class="w-4 h-4" /> {{ weg[weg.length - 2].name }}
      </button>

      <div v-if="!stand && !fehler" class="flex justify-center py-6"><Loader2 class="w-5 h-5 animate-spin text-leise" /></div>
      <p v-else-if="fehler" class="text-sm text-warnung flex items-start gap-2">
        <AlertTriangle class="w-4 h-4 shrink-0 mt-0.5" />
        <span class="min-w-0 break-words">{{ t('app.freigabe.nichtGeladen', { fehler }) }}</span>
      </p>

      <template v-else>
        <ul v-if="ebenen.length" class="divide-y divide-linie rounded-xl border border-linie">
          <li v-for="x in ebenen" :key="x.uuid">
            <button type="button" class="w-full flex items-center gap-3 p-3 text-left cursor-pointer" @click="hinein(x)">
              <component :is="istAlbum ? ImageIcon : Folder" class="w-5 h-5 shrink-0 text-marke" />
              <span class="text-sm font-medium text-schrift truncate">{{ x.name }}</span>
            </button>
          </li>
        </ul>

        <!-- Album: Bilder als Raster -->
        <div v-if="bilder.length" class="grid grid-cols-3 sm:grid-cols-4 gap-2">
          <button v-for="b in bilder" :key="b.uuid" type="button"
                  class="aspect-square rounded-lg overflow-hidden bg-auflage flex items-center justify-center cursor-pointer"
                  :title="b.name" @click="bildOeffnen(b)">
            <Loader2 v-if="arbeitet === b.uuid" class="w-4 h-4 animate-spin text-leise" />
            <img v-else-if="vorschauen[b.uuid]" :src="vorschauen[b.uuid]" :alt="b.name" class="w-full h-full object-cover" />
            <ImageIcon v-else-if="vorschauen[b.uuid] === false" class="w-6 h-6 text-leise" />
            <Loader2 v-else class="w-4 h-4 animate-spin text-leise" />
          </button>
        </div>

        <!-- Ordner: Dateien als Liste -->
        <ul v-if="dateien.length" class="divide-y divide-linie rounded-xl border border-linie">
          <li v-for="d in dateien" :key="d.uuid" class="flex items-center gap-3 p-3">
            <button type="button" class="flex-1 min-w-0 flex items-center gap-3 text-left cursor-pointer" @click="oeffnen(d)">
              <ImageIcon v-if="mime(d).startsWith('image/')" class="w-5 h-5 shrink-0 text-sky-500" />
              <FileText v-else class="w-5 h-5 shrink-0 text-rose-500" />
              <span class="min-w-0">
                <span class="block text-sm font-medium text-schrift truncate">{{ d.name }}</span>
                <span class="block text-xs text-leise">{{ groesse(d.groesse) }}</span>
              </span>
            </button>
            <Loader2 v-if="arbeitet === d.uuid" class="w-4 h-4 animate-spin text-leise" />
            <button v-else type="button" class="text-leise hover:text-marke cursor-pointer"
                    :title="t('app.freigabe.uebernehmenTitel')" :aria-label="t('app.freigabe.uebernehmenTitel')"
                    @click="uebernehmen(d)">
              <Download class="w-4 h-4" />
            </button>
          </li>
        </ul>

        <p v-if="!ebenen.length && !bilder.length && !dateien.length" class="text-sm text-fliess">{{ t('app.teilen.leer') }}</p>
      </template>
    </div>

    <!-- Ein Bild groß -->
    <div v-if="bild" class="fixed inset-0 z-[110] bg-black/90 flex flex-col" @click.self="bild = null">
      <!-- Unter die Statusleiste rutscht die Zeile sonst (edge-to-edge). -->
      <div class="flex items-center justify-between gap-2 p-3 pt-[max(0.75rem,env(safe-area-inset-top))] mt-6 text-white">
        <span class="text-sm truncate">{{ bild.name }}</span>
        <div class="flex gap-4 text-sm font-bold">
          <button type="button" class="cursor-pointer" @click="uebernehmen(bild)">{{ t('app.freigabe.uebernehmen') }}</button>
          <button type="button" class="cursor-pointer" @click="bild = null">{{ t('common.close') }}</button>
        </div>
      </div>
      <img :src="bild.url" :alt="bild.name" class="flex-1 min-h-0 object-contain" />
    </div>

    <PdfBetrachter v-if="pdf" :name="pdf.name" :laden="() => quelle.freigabeBytes(projekt, sache.uuid, pdf.uuid)" @close="pdf = null" />
  </BaseModal>
</template>
