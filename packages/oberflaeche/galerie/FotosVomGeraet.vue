<script setup>
/*
 * Fotos vom Gerät übernehmen (docs/plan-viele-dateien.md, D3, Skizze B):
 *
 *   ( ) Alle Fotos und Videos              4.812 · 31,2 GB
 *   (•) Nach Album
 *       [x] Kamera                         3.904 · 27,0 GB
 *   ( ) Ab Datum  [ 01.01.2026 ]
 *   Zusammen 4.515 · 28,1 GB – passt (frei: 40 GB)
 *   Bereits Übernommenes wird übersprungen.
 *
 * Beim ersten Mal (mit openany.de verbunden) die Frage aus Frage 6: auf
 * openany.de, hier nur die Vorschau -- oder als Kopie auf diesem Gerät. Die
 * Antwort merkt sich die Anwendung; „Ändern“ fragt neu.
 *
 * `quelle` = die Quelle der Galerie: `fotosVomGeraet` (erlaubt, erlauben,
 * beimErlauben, uebersicht, uebernehmen) und `uebernahme` (vergessen).
 * Gerätealben werden gleichnamige Alben; den Lauf zeigt die UebernahmeZeile.
 */
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { Loader2 } from 'lucide-vue-next';
import BaseModal from '../base/BaseModal.vue';
import BaseButton from '../base/BaseButton.vue';
import { useToast } from '../composables/useToast';
import { useConfirm } from '../composables/useConfirm';
import { useUebernahme } from '../composables/useUebernahme';
import { formatBytes } from '../shared/format';

const props = defineProps({
  quelle: { type: Object, required: true },
});
const emit = defineEmits(['close']);

const { t, locale } = useI18n();
const toast = useToast();
const { confirmDialog } = useConfirm();
const geraet = props.quelle.fotosVomGeraet;
const { auftrag } = useUebernahme(props.quelle.uebernahme);

const erlaubt = ref(geraet.erlaubt());
const lage = ref(null);
const laedt = ref(false);
const art = ref('alben');
const gewaehlt = ref(new Set());
const ab = ref('');
const schickt = ref(false);
// Frage 6: true = ohne Kopie (auf openany.de), false = Kopie, null = fragen.
const wahl = ref(null);
const nurWlan = ref(false);

const zahl = (n) => Number(n).toLocaleString(locale.value);
const menge = (anzahl, bytes) => `${zahl(anzahl)} · ${formatBytes(bytes)}`;

async function laden() {
  erlaubt.value = geraet.erlaubt();
  if (erlaubt.value === 'keine') return;
  laedt.value = true;
  try {
    lage.value = await geraet.uebersicht();
    wahl.value = lage.value.verbunden ? (lage.value.ohne_kopie ?? null) : false;
    // Vorgewählt: alle Gerätealben, ausser Bildschirmfotos.
    gewaehlt.value = new Set(lage.value.alben.map((a) => a.name).filter((n) => !/screenshot/i.test(n)));
  } catch (e) {
    toast.error(typeof e === 'string' ? e : t('albums.vomGeraet.fehler'));
  } finally {
    laedt.value = false;
  }
}

const zusammen = computed(() => {
  const l = lage.value;
  if (!l) return null;
  if (art.value === 'alle') return { anzahl: l.anzahl, bytes: l.bytes };
  if (art.value === 'alben') {
    const alben = l.alben.filter((a) => gewaehlt.value.has(a.name));
    return { anzahl: alben.reduce((s, a) => s + a.anzahl, 0), bytes: alben.reduce((s, a) => s + a.bytes, 0) };
  }
  return null; // Ab Datum: gezählt wird beim Übernehmen.
});
// Ohne Kopie zählt der Platz im Konto (wenn bekannt), sonst der hier.
const frei = computed(() => (wahl.value ? lage.value?.server_frei ?? null : lage.value?.frei ?? null));
const passt = computed(() => !zusammen.value || frei.value === null || zusammen.value.bytes <= frei.value);
const bereit = computed(() =>
  lage.value && wahl.value !== null && !schickt.value && passt.value
  && (art.value !== 'alben' || gewaehlt.value.size > 0)
  && (art.value !== 'ab' || ab.value));

function umschalten(name) {
  const s = new Set(gewaehlt.value);
  if (s.has(name)) s.delete(name); else s.add(name);
  gewaehlt.value = s;
}

async function uebernehmen() {
  schickt.value = true;
  const auswahl = {
    alben: art.value === 'alben' ? [...gewaehlt.value] : [],
    ab: art.value === 'ab' ? new Date(ab.value).getTime() : null,
    ohne_kopie: wahl.value === true,
    nur_wlan: wahl.value === true && nurWlan.value,
  };
  await auftrag(() => geraet.uebernehmen(auswahl));
  schickt.value = false;
  emit('close');
}

async function vergessen() {
  const ok = await confirmDialog(t('albums.vomGeraet.vergessenText'), {
    title: t('albums.vomGeraet.vergessenTitel'),
    confirmLabel: t('albums.vomGeraet.vergessen'),
  });
  if (!ok) return;
  const n = await props.quelle.uebernahme.vergessen();
  toast.info(t('albums.vomGeraet.vergessenFertig', { n: zahl(n) }));
}

let abmelden = null;
onMounted(() => {
  abmelden = geraet.beimErlauben(() => laden());
  laden();
});
onUnmounted(() => abmelden?.());
</script>

<template>
  <BaseModal :title="t('albums.vomGeraet.titel')" size="lg" @close="emit('close')">
    <div v-if="erlaubt === 'keine'" class="space-y-4 text-sm text-fliess">
      <p>{{ t('albums.vomGeraet.zugriffText') }}</p>
      <BaseButton @click="geraet.erlauben()">{{ t('albums.vomGeraet.zugriff') }}</BaseButton>
    </div>

    <div v-else-if="laedt || !lage" class="flex justify-center py-10">
      <Loader2 class="w-8 h-8 text-marke animate-spin" />
    </div>

    <div v-else-if="wahl === null" class="space-y-3 text-sm">
      <p class="text-fliess">{{ t('albums.vomGeraet.wieFrage') }}</p>
      <button type="button" class="w-full text-left p-3 rounded-xl border border-slate-200 dark:border-slate-700 hover:bg-auflage" @click="wahl = true">
        <span class="block font-semibold text-schrift">{{ t('albums.vomGeraet.mitServer') }}</span>
        <span class="block text-fliess">{{ t('albums.vomGeraet.mitServerText') }}</span>
      </button>
      <button type="button" class="w-full text-left p-3 rounded-xl border border-slate-200 dark:border-slate-700 hover:bg-auflage" @click="wahl = false">
        <span class="block font-semibold text-schrift">{{ t('albums.vomGeraet.nurHier') }}</span>
        <span class="block text-fliess">{{ t('albums.vomGeraet.nurHierText') }}</span>
      </button>
      <p class="text-leise">{{ t('albums.vomGeraet.wieGemerkt') }}</p>
    </div>

    <div v-else class="space-y-3 text-sm">
      <p v-if="erlaubt === 'auswahl'" class="p-3 rounded-xl bg-auflage text-fliess">
        {{ t('albums.vomGeraet.nurAuswahl') }}
        <button type="button" class="font-semibold text-marke hover:underline" @click="geraet.erlauben()">{{ t('albums.vomGeraet.auswahlAendern') }}</button>
      </p>

      <label class="flex items-center gap-3">
        <input v-model="art" type="radio" value="alle" />
        <span class="flex-1 font-semibold text-schrift">{{ t('albums.vomGeraet.alle') }}</span>
        <span class="text-leise tabular-nums">{{ menge(lage.anzahl, lage.bytes) }}</span>
      </label>

      <label class="flex items-center gap-3">
        <input v-model="art" type="radio" value="alben" />
        <span class="flex-1 font-semibold text-schrift">{{ t('albums.vomGeraet.nachAlbum') }}</span>
      </label>
      <ul v-if="art === 'alben'" class="ml-7 space-y-2 max-h-[40vh] overflow-y-auto">
        <li v-for="a in lage.alben" :key="a.name">
          <label class="flex items-center gap-3">
            <input type="checkbox" :checked="gewaehlt.has(a.name)" @change="umschalten(a.name)" />
            <span class="flex-1 text-schrift break-all">{{ a.name || t('albums.vomGeraet.ohneAlbum') }}</span>
            <span class="text-leise tabular-nums">{{ menge(a.anzahl, a.bytes) }}</span>
          </label>
        </li>
      </ul>

      <label class="flex items-center gap-3">
        <input v-model="art" type="radio" value="ab" />
        <span class="font-semibold text-schrift">{{ t('albums.vomGeraet.abDatum') }}</span>
        <input v-model="ab" type="date" class="feld py-1" :disabled="art !== 'ab'" />
      </label>

      <div class="pt-3 border-t border-slate-100 dark:border-slate-800 space-y-1">
        <p class="text-fliess">
          {{ t(wahl ? 'albums.vomGeraet.wahlServer' : 'albums.vomGeraet.wahlKopie') }}
          <button v-if="lage.verbunden" type="button" class="font-semibold text-marke hover:underline" @click="wahl = null">{{ t('albums.vomGeraet.aendern') }}</button>
        </p>
        <label v-if="wahl" class="flex items-center gap-3">
          <input v-model="nurWlan" type="checkbox" />
          <span class="text-schrift">{{ t('albums.vomGeraet.nurWlan') }}</span>
        </label>
        <p v-if="zusammen && frei !== null" :class="passt ? 'text-fliess' : 'text-rose-600 font-semibold'">
          {{ t((passt ? 'albums.vomGeraet.passt' : 'albums.vomGeraet.passtNicht') + (wahl ? 'Konto' : ''), { menge: menge(zusammen.anzahl, zusammen.bytes), frei: formatBytes(frei) }) }}
        </p>
        <p class="text-leise">
          {{ t('albums.vomGeraet.uebersprungen') }}
          <button type="button" class="font-semibold text-marke hover:underline" @click="vergessen">{{ t('albums.vomGeraet.vergessen') }}</button>
        </p>
      </div>
    </div>

    <template #footer>
      <BaseButton variant="secondary" @click="emit('close')">{{ t('common.cancel') }}</BaseButton>
      <BaseButton v-if="erlaubt !== 'keine'" :disabled="!bereit" :loading="schickt" @click="uebernehmen">{{ t('albums.vomGeraet.uebernehmen') }}</BaseButton>
    </template>
  </BaseModal>
</template>
