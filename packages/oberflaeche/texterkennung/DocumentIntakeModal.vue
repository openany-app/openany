<script setup>
// Zwei Dialoge in einem, weil sie zum selben Vorgang gehoeren:
//
//   1. die Nachfrage bei mehreren Fotos – ein mehrseitiges Dokument oder
//      einzelne? (Wer einen dreiseitigen Brief abfotografiert, will eins;
//      wer drei Belege ablegt, will drei.)
//   2. der Fortschritt waehrend der Umwandlung.
//
// Der Fortschritt ist hier kein Zierrat: Der erste Lauf laedt einmalig ein paar
// Megabyte Sprachdaten, danach dauert jedes Foto einige Sekunden. Ohne sichtbare
// Bewegung sieht das aus, als haenge die App.
import { ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { Loader2 } from 'lucide-vue-next';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import BaseModal from '@oberflaeche/base/BaseModal.vue';

const props = defineProps({
  // Nachfrage-Modus: Anzahl Fotos, sonst 0.
  frage: { type: Number, default: 0 },
  // Vorschlag fuer den Dokumentnamen (Name des ersten Fotos, ohne Endung).
  vorschlag: { type: String, default: '' },
  // Fortschritt: { schritt, nummer, gesamt, prozent } oder null.
  stand: { type: Object, default: null },
  // 'foto' = Umwandlung beim Hochladen, 'pdf' = nachträgliche Erkennung auf
  // einem schon abgelegten Scan. Derselbe Fortschritt, andere Schritte und
  // andere Zählweise (Fotos gegen Seiten).
  modus: { type: String, default: 'foto' },
});

const emit = defineEmits(['einzeln', 'zusammen', 'abbrechen']);
const { t } = useI18n();
const L = (key, params) => t(`documents.${key}`, params ?? {});

const name = ref(props.vorschlag);

const schritte = {
  foto: {
    prepare: 'convertStepPrepare',
    load: 'convertStepLoad',
    recognize: 'convertStepRecognize',
    build: 'convertStepBuild',
    save: 'convertStepSave',
  },
  pdf: {
    fetch: 'ocrStepFetch',
    render: 'ocrStepRender',
    load: 'convertStepLoad',
    recognize: 'convertStepRecognize',
    write: 'ocrStepWrite',
    save: 'ocrStepSave',
  },
};

const istPdf = computed(() => props.modus === 'pdf');

const schrittText = computed(() => {
  const karte = schritte[props.modus] ?? schritte.foto;
  return L(karte[props.stand?.schritt] ?? (istPdf.value ? 'ocrStepFetch' : 'convertStepPrepare'));
});

// „Foto 2 von 5" gegen „Seite 2 von 12" – beim Scan sind es Seiten EINES
// Dokuments, nicht mehrere Dokumente.
const fortschrittText = computed(() => L(
  istPdf.value ? 'ocrPageProgress' : 'convertProgress',
  { nummer: props.stand?.nummer ?? 1, gesamt: props.stand?.gesamt ?? 1 },
));

// Der Balken bewegt sich nur in den beiden Phasen, die wirklich Prozente
// melden. Beim Bauen und Ablegen laeuft er als unbestimmte Bewegung weiter,
// statt auf einem Wert stehen zu bleiben.
const bestimmt = computed(() => ['load', 'recognize'].includes(props.stand?.schritt));
</script>

<template>
  <BaseModal
    :title="frage ? L('convertMultiTitle') : (istPdf ? L('ocrTitle') : L('convertTitle'))"
    persistent
    :fest="!frage"
    @close="emit('abbrechen')"
  >
    <!-- 1. Nachfrage bei mehreren Fotos -->
    <div v-if="frage" class="space-y-4">
      <p class="text-sm font-semibold text-fliess">
        {{ L('convertMultiHint', { count: frage }) }}
      </p>

      <div class="space-y-2">
        <label class="block text-sm font-bold text-fliess">{{ L('convertMultiName') }}</label>
        <input
          v-model="name"
          type="text"
          class="w-full px-4 py-3 bg-vertieft border border-linie rounded-xl focus:outline-none focus:ring-2 focus:ring-marke text-sm font-semibold text-schrift transition-all"
        />
      </div>
    </div>

    <!-- 2. Fortschritt -->
    <div v-else class="space-y-3">
      <div class="flex items-center gap-3 text-sm font-bold text-fliess">
        <Loader2 class="animate-spin h-5 w-5 text-marke shrink-0" />
        <span>{{ schrittText }}</span>
      </div>

      <p v-if="(stand?.gesamt ?? 1) > 1 && (stand?.nummer ?? 0) > 0" class="text-xs font-semibold text-leise">
        {{ fortschrittText }}
      </p>

      <div class="h-2 w-full bg-auflage rounded-full overflow-hidden">
        <div
          class="h-full bg-marke rounded-full transition-all duration-300"
          :class="bestimmt ? '' : 'animate-pulse w-1/3'"
          :style="bestimmt ? { width: `${stand?.prozent ?? 0}%` } : null"
        ></div>
      </div>

      <p class="text-xs font-semibold text-leise">{{ L('convertFirstRunHint') }}</p>
    </div>

    <template #footer>
      <template v-if="frage">
        <BaseButton variant="secondary" @click="emit('einzeln')">{{ L('convertMultiSingle') }}</BaseButton>
        <BaseButton @click="emit('zusammen', name.trim())">{{ L('convertMultiCombined') }}</BaseButton>
      </template>
      <BaseButton v-else variant="secondary" @click="emit('abbrechen')">{{ L('convertCancel') }}</BaseButton>
    </template>
  </BaseModal>
</template>
