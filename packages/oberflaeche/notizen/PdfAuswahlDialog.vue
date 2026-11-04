<script setup>
// PDF einer Notiz: Standard, Clean (nur der Text), Brief oder als Druckfassung
// (nur diese Notiz, mit Titelblatt -- DruckfassungDialog.vue).
//
// Der Standard und die Druckfassung laufen wie bisher (NotesWorkspace,
// DruckfassungDialog); hier wird nur gewählt. Clean und Brief setzt dieser
// Dialog selbst (`api.cleanAlsPdf`, `api.briefAlsPdf`): Beim Brief Empfänger aus dem Adressbuch oder getippt,
// Absender und Ort aus den Einstellungen (shared/briefAbsender.js), Datum
// heute -- alles hier noch änderbar.
import { ref, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import BaseModal from '@oberflaeche/base/BaseModal.vue';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import { useToast } from '@oberflaeche/composables/useToast';
import { sanitizeFilename } from '@oberflaeche/shared/download';
import { pdfAblegen } from '@oberflaeche/shared/ablage';
import { absenderLesen, absenderMerken, anschriftZeilen } from '@oberflaeche/shared/briefAbsender';

const props = defineProps({
  note: { type: Object, required: true },
  api: { type: Object, required: true },
});
const emit = defineEmits(['close', 'standard', 'druckfassung']);

const { t, locale } = useI18n();
const toast = useToast();

const arten = computed(() => [
  'standard',
  ...(props.api.cleanAlsPdf ? ['clean'] : []),
  ...(props.api.briefAlsPdf ? ['brief'] : []),
  ...(props.api.druckfassungAlsPdf ? ['druckfassung'] : []),
]);
const art = ref('standard');

const gemerkt = absenderLesen();
const heute = new Intl.DateTimeFormat(locale.value, { dateStyle: 'long' }).format(new Date());
const brief = ref({
  empfaenger: '',
  absender: gemerkt.absender,
  ortDatum: gemerkt.ort ? `${gemerkt.ort}, ${heute}` : heute,
});

// Postanschriften aus dem Adressbuch: [{ name, adresse }].
const anschriften = ref([]);
onMounted(async () => {
  if (props.api.briefEmpfaenger) anschriften.value = await props.api.briefEmpfaenger().catch(() => []);
});
const anschriftWaehlen = (e) => {
  const a = anschriften.value[Number(e.target.value)];
  if (a) brief.value.empfaenger = anschriftZeilen(a.name, a.adresse);
  e.target.value = '';
};

const arbeitet = ref(false);
async function weiter() {
  if (art.value === 'standard') return emit('standard');
  if (art.value === 'druckfassung') return emit('druckfassung');
  arbeitet.value = true;
  try {
    let res;
    if (art.value === 'clean') {
      res = await props.api.cleanAlsPdf(props.note.id);
    } else {
      absenderMerken({ absender: brief.value.absender, ort: gemerkt.ort });
      res = await props.api.briefAlsPdf(props.note.id, { ...brief.value });
    }
    await pdfAblegen(props.api, {
      akte: 'Notizen',
      name: `${sanitizeFilename(props.note.title, `notiz-${props.note.id}`)} (${t(`notes.pdf.${art.value}`)}).pdf`,
      blob: new Blob([res.data], { type: 'application/pdf' }),
    });
    emit('close');
  } catch (e) {
    console.error('PDF fehlgeschlagen', e);
    toast.error(t('notes.pdfExportFailed'));
  } finally {
    arbeitet.value = false;
  }
}

const feldKlasse = 'w-full px-4 py-3 bg-vertieft border border-linie rounded-xl focus:outline-none focus:ring-2 focus:ring-marke text-sm font-semibold text-schrift transition-all';
</script>

<template>
  <BaseModal :title="t('notes.pdf.titel')" size="md" :persistent="arbeitet" @close="emit('close')">
    <form id="pdf-auswahl-form" class="space-y-4" @submit.prevent="weiter">
      <div class="flex flex-wrap gap-2" role="radiogroup">
        <button v-for="a in arten" :key="a" type="button" role="radio" :aria-checked="art === a"
          class="px-3 py-2 rounded-xl border text-sm"
          :class="art === a ? 'border-marke bg-marke/10 text-marke font-bold' : 'border-linie text-schrift'"
          @click="art = a">
          {{ t(`notes.pdf.${a}`) }}
        </button>
      </div>
      <p class="text-sm text-leise">{{ t(`notes.pdf.hinweis.${art}`) }}</p>

      <template v-if="art === 'brief'">
        <div class="space-y-1.5">
          <label for="brief-empfaenger" class="block text-sm font-bold text-fliess">{{ t('notes.pdf.empfaenger') }}</label>
          <select v-if="anschriften.length" :class="feldKlasse" @change="anschriftWaehlen">
            <option value="">{{ t('notes.pdf.ausAdressbuch') }}</option>
            <option v-for="(a, i) in anschriften" :key="i" :value="i">{{ a.name }} – {{ a.adresse }}</option>
          </select>
          <textarea id="brief-empfaenger" v-model="brief.empfaenger" rows="4" required :class="feldKlasse" />
        </div>
        <div class="space-y-1.5">
          <label for="brief-absender" class="block text-sm font-bold text-fliess">{{ t('notes.pdf.absender') }}</label>
          <textarea id="brief-absender" v-model="brief.absender" rows="3" :class="feldKlasse" />
          <p class="text-xs text-leise">{{ t('notes.pdf.absenderHinweis') }}</p>
        </div>
        <div class="space-y-1.5">
          <label for="brief-datum" class="block text-sm font-bold text-fliess">{{ t('notes.pdf.ortDatum') }}</label>
          <input id="brief-datum" v-model="brief.ortDatum" type="text" :class="feldKlasse" />
        </div>
      </template>
    </form>

    <template #footer>
      <BaseButton variant="secondary" :disabled="arbeitet" @click="emit('close')">{{ t('common.cancel') }}</BaseButton>
      <BaseButton type="submit" form="pdf-auswahl-form" :loading="arbeitet">{{ t('notes.pdf.erstellen') }}</BaseButton>
    </template>
  </BaseModal>
</template>
