<script setup>
// Die KI des Projekts, im Mitglieder-Tab.
//
// WARUM HIER: Die Anbindung ist eine Frage danach, wer außer den Menschen in
// der Liste darüber noch mitliest. Sie steht deshalb dort, wo man nachsieht,
// wer beteiligt ist – für ALLE Mitglieder sichtbar, einrichten darf nur ein
// Owner.
//
// Der Schlüssel geht beim Speichern einmal hindurch und kommt nie zurück,
// auch nicht zum Owner. Ändern heißt: Modell wechseln (Schlüssel bleibt)
// oder einen neuen Schlüssel eintragen.
import { ref, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { Sparkles, Loader2, AlertTriangle, Unplug, Pencil } from 'lucide-vue-next';
import { api } from './umgebung';
import { useToast } from '@oberflaeche/composables/useToast';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import { formatDateTime } from '@oberflaeche/shared/date';
import BaseButton from '@oberflaeche/base/BaseButton.vue';

const props = defineProps({
  projectId: { type: [Number, String], required: true },
});

const { t } = useI18n();
const toast = useToast();
const { confirmDelete } = useConfirm();

const laedt = ref(true);
const zustand = ref(null);
const bearbeiten = ref(false);
const speichert = ref(false);
const feldfehler = ref({});
const allgemein = ref('');

const leereForm = () => ({ provider: 'openrouter', model: '', apiKey: '' });
const form = ref(leereForm());

const anbindung = computed(() => zustand.value?.anbindung ?? null);
const darfVerwalten = computed(() => !!zustand.value?.darf_verwalten);
const formSichtbar = computed(() => darfVerwalten.value && (bearbeiten.value || !anbindung.value));

async function laden() {
  laedt.value = true;
  try {
    const { data } = await api.getProjectAi(props.projectId);
    zustand.value = data;
  } finally {
    laedt.value = false;
  }
}

function aendern() {
  form.value = {
    provider: anbindung.value.provider,
    model: anbindung.value.model,
    apiKey: '',
  };
  feldfehler.value = {};
  allgemein.value = '';
  bearbeiten.value = true;
}

async function speichern() {
  speichert.value = true;
  feldfehler.value = {};
  allgemein.value = '';
  try {
    const { data } = await api.saveProjectAi(props.projectId, form.value);
    zustand.value = data;
    bearbeiten.value = false;
    // Sofort vergessen – der Schlüssel soll nicht im Speicher des Browsers
    // stehen bleiben, bis jemand die Seite wechselt.
    form.value = leereForm();
    toast.success(t('projects.ai.saved'));
  } catch (e) {
    const status = e?.response?.status;
    if (status === 422 && e.response.data?.errors) {
      feldfehler.value = Object.fromEntries(
        Object.entries(e.response.data.errors).map(([feld, meldungen]) => [feld, meldungen[0]]),
      );
    } else {
      allgemein.value = e?.response?.data?.message || t('projects.ai.saveFailed');
    }
  } finally {
    speichert.value = false;
  }
}

async function entfernen() {
  if (!(await confirmDelete(t('projects.ai.removeConfirm')))) return;
  await api.removeProjectAi(props.projectId);
  zustand.value = { ...zustand.value, anbindung: null };
  bearbeiten.value = false;
  toast.success(t('projects.ai.removed'));
}

onMounted(laden);
</script>

<template>
  <div class="karte p-5 shadow-sm space-y-3">
    <h3 class="text-sm font-bold text-fliess flex items-center gap-2">
      <Sparkles class="w-4 h-4 text-marke" /> {{ t('projects.ai.title') }}
    </h3>

    <div v-if="laedt" class="py-4 flex justify-center">
      <Loader2 class="w-5 h-5 animate-spin text-leise" />
    </div>

    <template v-else>
      <!-- Angebunden: für alle sichtbar, was hinausgeht und wohin. -->
      <template v-if="anbindung && !bearbeiten">
        <p class="text-sm text-fliess">
          {{ t('projects.ai.connected', { anbieter: anbindung.provider_name }) }}
          <span class="font-bold break-all">{{ anbindung.model }}</span>
        </p>
        <p class="text-xs text-leise">
          {{ t('projects.ai.setUpBy', {
            wer: anbindung.eingerichtet_von || t('projects.ai.unknownPerson'),
            wann: formatDateTime(anbindung.eingerichtet_am),
          }) }}
        </p>
        <p class="text-sm text-fliess rounded-xl border border-linie bg-auflage p-3 flex items-start gap-2">
          <AlertTriangle class="w-4 h-4 shrink-0 mt-0.5 text-marke" />
          <span>{{ t('projects.ai.whatHappens', { anbieter: anbindung.provider_name }) }}</span>
        </p>
        <div v-if="darfVerwalten" class="flex flex-wrap gap-2">
          <BaseButton type="button" variant="secondary" groesse="klein" @click="aendern">
            <Pencil class="w-3.5 h-3.5" /> {{ t('projects.ai.change') }}
          </BaseButton>
          <BaseButton type="button" variant="danger" groesse="klein" @click="entfernen">
            <Unplug class="w-3.5 h-3.5" /> {{ t('projects.ai.remove') }}
          </BaseButton>
        </div>
      </template>

      <p v-else-if="!darfVerwalten" class="text-sm text-leise">{{ t('projects.ai.none') }}</p>

      <!-- Einrichten / ändern: nur Owner. -->
      <form v-if="formSichtbar" class="space-y-3 max-w-md" @submit.prevent="speichern">
        <p v-if="!anbindung" class="text-sm text-leise">{{ t('projects.ai.intro') }}</p>

        <div>
          <label class="block text-sm font-bold text-fliess" for="ki-anbieter">{{ t('projects.ai.provider') }}</label>
          <select id="ki-anbieter" v-model="form.provider" required
            class="mt-1 w-full px-3 py-2 rounded-xl border border-linie bg-grund text-schrift">
            <option v-for="a in zustand.anbieter" :key="a.id" :value="a.id">{{ a.name }}</option>
          </select>
        </div>

        <div>
          <label class="block text-sm font-bold text-fliess" for="ki-modell">{{ t('projects.ai.model') }}</label>
          <input id="ki-modell" v-model="form.model" type="text" required maxlength="255"
            :placeholder="t('projects.ai.modelPlaceholder')" autocomplete="off" spellcheck="false"
            class="mt-1 w-full px-3 py-2 rounded-xl border border-linie bg-grund text-schrift" />
          <p v-if="feldfehler.model" class="mt-1 text-sm text-warnung">{{ feldfehler.model }}</p>
        </div>

        <div>
          <label class="block text-sm font-bold text-fliess" for="ki-schluessel">{{ t('projects.ai.key') }}</label>
          <input id="ki-schluessel" v-model="form.apiKey" type="password" :required="!anbindung" maxlength="512"
            autocomplete="off" :placeholder="anbindung ? t('projects.ai.keyKeep') : ''"
            class="mt-1 w-full px-3 py-2 rounded-xl border border-linie bg-grund text-schrift" />
          <p class="mt-1 text-xs text-leise">{{ t('projects.ai.keyHint') }}</p>
          <p v-if="feldfehler.api_key" class="mt-1 text-sm text-warnung">{{ feldfehler.api_key }}</p>
        </div>

        <p v-if="allgemein" class="text-sm text-warnung flex items-start gap-2">
          <AlertTriangle class="w-4 h-4 shrink-0 mt-0.5" /> {{ allgemein }}
        </p>

        <div class="flex flex-wrap gap-2">
          <BaseButton type="submit" groesse="normal" :disabled="speichert">
            <Loader2 v-if="speichert" class="w-4 h-4 animate-spin" />
            <Sparkles v-else class="w-4 h-4" />
            {{ t('projects.ai.save') }}
          </BaseButton>
          <BaseButton v-if="anbindung" type="button" variant="ghost" groesse="normal" @click="bearbeiten = false">
            {{ t('projects.ai.cancel') }}
          </BaseButton>
        </div>
      </form>
    </template>
  </div>
</template>
