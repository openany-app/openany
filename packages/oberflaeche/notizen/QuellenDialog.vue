<script setup>
// Die Quellenliste einer Mappe: anlegen, ändern, entfernen.
//
// Was hier steht, ist die CSL-JSON-Datei der Mappe (QuellenListe.php) --
// dieselbe, aus der der Editor beim Zitieren vorschlägt und die Druckfassung
// ihr Literaturverzeichnis setzt. Die Formular-Logik steht in quellen.js;
// hier nur Liste, Formular und Speichern.
import { ref, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { Plus, Trash2, ArrowLeft, Copy } from 'lucide-vue-next';
import BaseModal from '@oberflaeche/base/BaseModal.vue';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import { useToast } from '@oberflaeche/composables/useToast';
import {
  ARTEN, felderVon, alsFormular, ausFormular, schluesselVorschlag, schluesselGueltig, kurzanzeige, vollstaendig,
} from './quellen';

const props = defineProps({
  mappe: { type: Object, required: true },
  api: { type: Object, required: true },
});
const emit = defineEmits(['close']);

const { t } = useI18n();
const toast = useToast();

const laedt = ref(true);
const speichert = ref(false);
const eintraege = ref([]);
const info = ref({ datei: null, geerbt: false, bearbeitbar: true });
const geaendert = ref(false);

// Das offene Formular: { index (-1 = neu), werte, schluesselVonHand }
const offen = ref(null);

onMounted(async () => {
  try {
    const { data } = await props.api.quellenLesen(props.mappe.id);
    eintraege.value = data.eintraege ?? [];
    info.value = { datei: data.datei, geerbt: data.geerbt, bearbeitbar: data.bearbeitbar };
  } catch (e) {
    console.error('Quellen nicht geladen', e);
    toast.error(t('notes.quellen.ladenFehler'));
    emit('close');
  } finally {
    laedt.value = false;
  }
});

const sortiert = computed(() => eintraege.value
  .map((e, index) => ({ e, index, text: kurzanzeige(e, t('notes.quellen.und')) }))
  .sort((a, b) => a.text.localeCompare(b.text, undefined, { sensitivity: 'base' })));

const vergebene = (ausser) => new Set(eintraege.value.filter((_, i) => i !== ausser).map((e) => e.id));

function oeffnen(index) {
  const e = index >= 0 ? eintraege.value[index] : {};
  offen.value = { index, werte: alsFormular(e), schluesselVonHand: index >= 0 };
}

// Solange niemand den Kurznamen selbst gesetzt hat, folgt er Verfasser und Jahr.
function feldGeaendert() {
  const o = offen.value;
  if (o && !o.schluesselVonHand) o.werte.schluessel = schluesselVorschlag(o.werte, vergebene(o.index));
}

const fehler = computed(() => {
  const o = offen.value;
  if (!o) return '';
  if (!vollstaendig(o.werte)) return t('notes.quellen.fehlt');
  if (!schluesselGueltig(o.werte.schluessel)) return t('notes.quellen.schluesselUngueltig');
  if (vergebene(o.index).has(o.werte.schluessel.trim())) return t('notes.quellen.schluesselVergeben');
  return '';
});

function uebernehmen() {
  const o = offen.value;
  if (fehler.value) return;
  const vorher = o.index >= 0 ? eintraege.value[o.index] : {};
  const eintrag = ausFormular(o.werte, vorher);
  if (o.index >= 0) eintraege.value.splice(o.index, 1, eintrag);
  else eintraege.value.push(eintrag);
  geaendert.value = true;
  offen.value = null;
}

function entfernen() {
  const o = offen.value;
  if (o.index >= 0) {
    eintraege.value.splice(o.index, 1);
    geaendert.value = true;
  }
  offen.value = null;
}

async function speichern() {
  speichert.value = true;
  try {
    await props.api.quellenSpeichern(props.mappe.id, eintraege.value);
    toast.success(t('notes.quellen.gespeichert'));
    emit('close');
  } catch (e) {
    console.error('Quellen nicht gespeichert', e);
    toast.error(e?.response?.data?.message || t('notes.quellen.speichernFehler'));
  } finally {
    speichert.value = false;
  }
}

async function kopieren(schluessel) {
  try {
    await navigator.clipboard.writeText(`[@${schluessel}]`);
    toast.success(t('notes.quellen.kopiert'));
  } catch {
    // Ohne Zwischenablage (unsicherer Kontext) steht der Kurzname ja da.
  }
}

const feldKlasse = 'w-full px-4 py-3 bg-vertieft border border-linie rounded-xl focus:outline-none focus:ring-2 focus:ring-marke text-sm font-semibold text-schrift transition-all';
const mehrzeilig = new Set(['autoren', 'herausgeber']);
const datumsfeld = new Set(['datum', 'abgerufen']);
// Arten als i18n-Schlüssel: „article-journal" -> „article_journal".
const artSchluessel = (typ) => (ARTEN[typ] ? typ : 'misc').replace(/-/g, '_');
const beschriftung = (feld, typ) => (feld === 'enthalten'
  ? t(`notes.quellen.feld.enthalten_${artSchluessel(typ)}`)
  : t(`notes.quellen.feld.${feld}`));
// Arten im Auswahlfeld: die des Formulars, dazu eine fremde aus einer Datei.
const artenAuswahl = computed(() => {
  const typ = offen.value?.werte.typ;
  const arten = Object.keys(ARTEN);
  return typ && !ARTEN[typ] ? [...arten, typ] : arten;
});
const artName = (typ) => (ARTEN[typ] ? t(`notes.quellen.art.${artSchluessel(typ)}`) : typ);
</script>

<template>
  <BaseModal :title="t('notes.quellen.titel', { name: mappe.name })" size="lg" :persistent="speichert || geaendert" @close="emit('close')">
    <div v-if="laedt" class="py-8 text-center text-sm text-leise">{{ t('common.loading') }}</div>

    <!-- Ein Eintrag -->
    <form v-else-if="offen" id="quelle-form" class="space-y-4" @submit.prevent="uebernehmen">
      <button type="button" class="flex items-center gap-1.5 text-sm font-bold text-leise hover:text-schrift" @click="offen = null">
        <ArrowLeft class="w-4 h-4" /> {{ t('notes.quellen.zurueck') }}
      </button>

      <div class="space-y-1.5">
        <label for="quelle-typ" class="block text-sm font-bold text-fliess">{{ t('notes.quellen.feld.typ') }}</label>
        <select id="quelle-typ" v-model="offen.werte.typ" :class="feldKlasse" @change="feldGeaendert">
          <option v-for="typ in artenAuswahl" :key="typ" :value="typ">{{ artName(typ) }}</option>
        </select>
      </div>

      <div v-for="feld in felderVon(offen.werte.typ)" :key="feld" class="space-y-1.5">
        <label :for="`quelle-${feld}`" class="block text-sm font-bold text-fliess">{{ beschriftung(feld, offen.werte.typ) }}</label>
        <textarea
          v-if="mehrzeilig.has(feld)"
          :id="`quelle-${feld}`"
          v-model="offen.werte[feld]"
          rows="2"
          :placeholder="t('notes.quellen.namenHinweis')"
          :class="feldKlasse"
          @input="feldGeaendert"
        />
        <input
          v-else
          :id="`quelle-${feld}`"
          v-model="offen.werte[feld]"
          :type="datumsfeld.has(feld) ? 'date' : (feld === 'url' ? 'url' : 'text')"
          :inputmode="feld === 'jahr' ? 'numeric' : undefined"
          maxlength="1000"
          :class="feldKlasse"
          @input="feldGeaendert"
        />
      </div>

      <div class="space-y-1.5">
        <label for="quelle-schluessel" class="block text-sm font-bold text-fliess">{{ t('notes.quellen.feld.schluessel') }}</label>
        <input
          id="quelle-schluessel"
          v-model="offen.werte.schluessel"
          type="text"
          maxlength="200"
          autocapitalize="off"
          spellcheck="false"
          :class="[feldKlasse, 'font-mono']"
          @input="offen.schluesselVonHand = true"
        />
        <p class="text-xs text-leise">{{ t('notes.quellen.schluesselHinweis', { beispiel: `[@${offen.werte.schluessel || 'mueller2020'}, S. 12]` }) }}</p>
      </div>

      <p v-if="fehler" class="text-xs font-bold text-red-600 dark:text-red-400">{{ fehler }}</p>
    </form>

    <!-- Die Liste -->
    <div v-else class="space-y-4">
      <p class="text-sm text-leise">{{ t('notes.quellen.einleitung') }}</p>
      <p v-if="info.datei" class="text-xs text-leise">
        {{ t(info.geerbt ? 'notes.quellen.dateiGeerbt' : 'notes.quellen.datei', { name: info.datei.name }) }}
      </p>
      <p v-if="!info.bearbeitbar" class="text-sm font-bold text-fliess">{{ t('notes.quellen.nurLesen') }}</p>

      <ul v-if="sortiert.length" class="divide-y divide-linie border border-linie rounded-xl overflow-hidden">
        <li v-for="z in sortiert" :key="z.e.id" class="flex items-center gap-2 px-3 py-2.5">
          <button
            type="button"
            class="flex-1 min-w-0 text-left disabled:cursor-default"
            :disabled="!info.bearbeitbar"
            @click="oeffnen(z.index)"
          >
            <span class="block text-sm font-semibold text-schrift truncate">{{ z.text }}</span>
            <span class="block text-xs font-mono text-leise truncate">[@{{ z.e.id }}]</span>
          </button>
          <button type="button" class="p-1.5 text-slate-400 hover:text-schrift rounded-lg shrink-0" :title="t('notes.quellen.kopieren')" @click="kopieren(z.e.id)">
            <Copy class="w-4 h-4" />
          </button>
        </li>
      </ul>
      <p v-else class="text-sm text-leise">{{ t('notes.quellen.leer') }}</p>

      <BaseButton v-if="info.bearbeitbar" variant="secondary" @click="oeffnen(-1)">
        <Plus class="w-4 h-4" /> {{ t('notes.quellen.neu') }}
      </BaseButton>

      <p class="text-xs text-leise">{{ t('notes.quellen.zitierHinweis') }}</p>
    </div>

    <template #footer>
      <template v-if="offen">
        <BaseButton v-if="offen.index >= 0" variant="danger" @click="entfernen"><Trash2 class="w-4 h-4" /> {{ t('notes.quellen.entfernen') }}</BaseButton>
        <BaseButton type="submit" form="quelle-form" :disabled="!!fehler">{{ t('notes.quellen.uebernehmen') }}</BaseButton>
      </template>
      <template v-else>
        <BaseButton variant="secondary" :disabled="speichert" @click="emit('close')">{{ t('common.cancel') }}</BaseButton>
        <BaseButton v-if="info.bearbeitbar" :loading="speichert" :disabled="!geaendert || laedt" @click="speichern">{{ t('common.save') }}</BaseButton>
      </template>
    </template>
  </BaseModal>
</template>
