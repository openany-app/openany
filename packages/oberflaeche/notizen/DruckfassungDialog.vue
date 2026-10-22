<script setup>
// Die Druckfassung einer Mappe: Angaben fürs Titelblatt, dann das PDF.
//
// Gesetzt wird von der Quelle (`api.druckfassungAlsPdf`), in der Webapp im
// Browser mit Typst (packages/oberflaeche/druck/). Dieser Dialog weiß davon nichts;
// er sammelt die Angaben, hebt sie an der Mappe auf (`druckfassungSpeichern`)
// und reicht die Datei weiter. Gibt eine Quelle das nicht her, zeigt
// NotesWorkspace den Knopf gar nicht.
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import BaseModal from '@oberflaeche/base/BaseModal.vue';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import { useToast } from '@oberflaeche/composables/useToast';
import { sanitizeFilename } from '@oberflaeche/shared/download';
import { pdfAblegen } from '@oberflaeche/shared/ablage';

const props = defineProps({
  mappe: { type: Object, required: true },
  api: { type: Object, required: true },
});
const emit = defineEmits(['close', 'gespeichert']);

const { t, locale } = useI18n();
const toast = useToast();

const heute = () => new Intl.DateTimeFormat(locale.value, { dateStyle: 'long' }).format(new Date());
const gespeichert = props.mappe.druckfassung ?? {};
const angaben = ref({
  titel: gespeichert.titel ?? props.mappe.name ?? '',
  untertitel: gespeichert.untertitel ?? '',
  name: gespeichert.name ?? '',
  einrichtung: gespeichert.einrichtung ?? '',
  datum: gespeichert.datum ?? heute(),
  zusatz: gespeichert.zusatz ?? '',
  inhaltsverzeichnis: gespeichert.inhaltsverzeichnis ?? true,
  untermappen: gespeichert.untermappen ?? true,
});
const arbeitet = ref(false);

const felder = [
  { key: 'titel', max: 255 },
  { key: 'untertitel', max: 255 },
  { key: 'name', max: 255 },
  { key: 'einrichtung', max: 255, zeilen: 2 },
  { key: 'datum', max: 100 },
  { key: 'zusatz', max: 1000, zeilen: 3 },
];

async function erstellen() {
  arbeitet.value = true;
  const werte = { ...angaben.value };
  try {
    // Erst aufheben: Scheitert danach das Setzen, sind die Angaben nicht weg.
    await props.api.druckfassungSpeichern(props.mappe.id, werte);
    emit('gespeichert', werte);
    const res = await props.api.druckfassungAlsPdf(props.mappe, werte);
    // In die Akte „Druckfassungen" unter Dokumente; jede Fassung bleibt.
    await pdfAblegen(props.api, {
      akte: 'Druckfassungen',
      name: `${sanitizeFilename(werte.titel || props.mappe.name, 'druckfassung')}.pdf`,
      blob: res.data,
    });
    emit('close');
  } catch (e) {
    console.error('Druckfassung fehlgeschlagen', e);
    toast.error(e?.grund === 'leer' ? t('notes.druckfassung.leer') : t('notes.druckfassung.fehler'));
  } finally {
    arbeitet.value = false;
  }
}
</script>

<template>
  <BaseModal :title="t('notes.druckfassung.titel', { name: mappe.name })" size="md" :persistent="arbeitet" @close="emit('close')">
    <form id="druckfassung-form" class="space-y-4" @submit.prevent="erstellen">
      <p class="text-sm text-leise">{{ t('notes.druckfassung.einleitung') }}</p>

      <div v-for="feld in felder" :key="feld.key" class="space-y-1.5">
        <label :for="`druckfassung-${feld.key}`" class="block text-sm font-bold text-fliess">{{ t(`notes.druckfassung.feld.${feld.key}`) }}</label>
        <textarea
          v-if="feld.zeilen"
          :id="`druckfassung-${feld.key}`"
          v-model="angaben[feld.key]"
          :rows="feld.zeilen"
          :maxlength="feld.max"
          class="w-full px-4 py-3 bg-vertieft border border-linie rounded-xl focus:outline-none focus:ring-2 focus:ring-marke text-sm font-semibold text-schrift transition-all"
        />
        <input
          v-else
          :id="`druckfassung-${feld.key}`"
          v-model="angaben[feld.key]"
          type="text"
          :maxlength="feld.max"
          class="w-full px-4 py-3 bg-vertieft border border-linie rounded-xl focus:outline-none focus:ring-2 focus:ring-marke text-sm font-semibold text-schrift transition-all"
        />
      </div>

      <label class="flex items-center gap-3 text-sm font-bold text-fliess cursor-pointer">
        <input id="druckfassung-inhaltsverzeichnis" v-model="angaben.inhaltsverzeichnis" type="checkbox" class="w-4 h-4 accent-marke" />
        {{ t('notes.druckfassung.feld.inhaltsverzeichnis') }}
      </label>
      <label class="flex items-center gap-3 text-sm font-bold text-fliess cursor-pointer">
        <input id="druckfassung-untermappen" v-model="angaben.untermappen" type="checkbox" class="w-4 h-4 accent-marke" />
        {{ t('notes.druckfassung.feld.untermappen') }}
      </label>

      <p class="text-xs text-leise">{{ t('notes.druckfassung.hinweis') }}</p>
    </form>

    <template #footer>
      <BaseButton variant="secondary" :disabled="arbeitet" @click="emit('close')">{{ t('common.cancel') }}</BaseButton>
      <BaseButton type="submit" form="druckfassung-form" :loading="arbeitet">{{ t('notes.druckfassung.erstellen') }}</BaseButton>
    </template>
  </BaseModal>
</template>
