<script setup>
/*
 * Ist DIESES Gerät bei einem Matrix-Konto bestätigt? (docs/plan-matrix-bestaetigen.md)
 *
 * Element zeigt Nachrichten von einem unbestätigten Gerät mit Warnung und will
 * solche Geräte künftig ausschließen. Bestätigt wird hier auf zwei Wegen:
 *   M1  mit dem Wiederherstellungsschlüssel aus Element (das Konto hat eine
 *       Sicherung) -- der Schlüssel wird nicht aufbewahrt;
 *   M2  hat das Konto noch gar keine Identität, legt openany sie an samt
 *       Sicherung, und der neue Schlüssel erscheint genau EINMAL.
 * Gibt es eine Identität ohne Sicherung, geht es nur aus Element.
 */
import { ref, onMounted, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { ShieldCheck, ShieldAlert, Loader2, KeyRound, Copy, AlertTriangle } from 'lucide-vue-next';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import PasswortFeld from '@oberflaeche/base/PasswortFeld.vue';
import { useToast } from '@oberflaeche/composables/useToast';
import { useConfirm } from '@oberflaeche/composables/useConfirm';

const props = defineProps({
  mxid: { type: String, required: true },
  // Erst fragen, wenn das Konto läuft -- vorher antwortet der Homeserver nicht.
  laeuft: { type: Boolean, default: false },
});

const { t } = useI18n();
const toast = useToast();
const { confirmDialog } = useConfirm();

// bestaetigt | mit_schluessel | anlegen | nur_anderswo | null (unbekannt)
const stand = ref(null);
const offen = ref(false);
const schluessel = ref('');
const arbeitet = ref(false);
const fehler = ref('');
// M2: der neue Wiederherstellungsschlüssel -- nur, bis „Fertig“ getippt ist.
const neuerSchluessel = ref('');
const aufgeschrieben = ref(false);

async function lesen() {
  if (!props.laeuft) return;
  try {
    stand.value = await invoke('matrix_bestaetigung', { mxid: props.mxid });
  } catch {
    stand.value = null; // ohne Verbindung: nichts zu sagen
  }
}

async function bestaetigen() {
  arbeitet.value = true;
  fehler.value = '';
  try {
    stand.value = await invoke('matrix_bestaetigen', { mxid: props.mxid, schluessel: schluessel.value });
    schluessel.value = '';
    offen.value = false;
    toast.success(t('app.matrix.bestaetigtMeldung'));
  } catch (e) {
    fehler.value = String(e);
  } finally {
    arbeitet.value = false;
  }
}

async function anlegen() {
  if (!(await confirmDialog(t('app.matrix.anlegenFrage'), { confirmLabel: t('app.matrix.anlegenKnopf') }))) return;
  arbeitet.value = true;
  fehler.value = '';
  try {
    neuerSchluessel.value = await invoke('matrix_sicherung_anlegen', { mxid: props.mxid });
    aufgeschrieben.value = false;
  } catch (e) {
    fehler.value = String(e);
  } finally {
    arbeitet.value = false;
  }
}

async function kopieren() {
  try {
    await navigator.clipboard.writeText(neuerSchluessel.value);
    toast.success(t('app.matrix.kopiert'));
  } catch {
    toast.error(t('app.matrix.nichtKopiert'));
  }
}

async function fertig() {
  neuerSchluessel.value = '';
  await lesen();
}

onMounted(lesen);
watch(() => props.laeuft, lesen);
</script>

<template>
  <div v-if="neuerSchluessel" class="rounded-xl border border-warnung p-3 space-y-2">
    <p class="text-sm font-bold text-schrift flex items-center gap-2"><KeyRound class="w-4 h-4 text-warnung" /> {{ t('app.matrix.neuerSchluesselTitel') }}</p>
    <p class="text-sm text-fliess">{{ t('app.matrix.neuerSchluesselText') }}</p>
    <p class="font-mono text-sm text-schrift bg-vertieft rounded-lg p-3 break-words select-all">{{ neuerSchluessel }}</p>
    <BaseButton variant="secondary" @click="kopieren"><Copy class="w-4 h-4" /> {{ t('app.matrix.kopieren') }}</BaseButton>
    <label class="flex items-start gap-2 text-sm text-fliess">
      <input v-model="aufgeschrieben" type="checkbox" class="mt-0.5" /> {{ t('app.matrix.aufgeschrieben') }}
    </label>
    <BaseButton :disabled="!aufgeschrieben" @click="fertig">{{ t('app.matrix.fertig') }}</BaseButton>
  </div>

  <template v-else-if="stand">
    <p v-if="stand === 'bestaetigt'" class="text-sm text-marke flex items-center gap-2">
      <ShieldCheck class="w-4 h-4 shrink-0" /> {{ t('app.matrix.bestaetigt') }}
    </p>

    <template v-else>
      <p class="text-sm text-warnung flex items-start gap-2">
        <ShieldAlert class="w-4 h-4 shrink-0 mt-0.5" />
        <span>{{ t('app.matrix.nichtBestaetigt') }}
          <template v-if="stand === 'anlegen'"> {{ t('app.matrix.ohneSicherung') }}</template>
          <template v-else-if="stand === 'nur_anderswo'"> {{ t('app.matrix.nurAnderswo') }}</template>
        </span>
      </p>

      <BaseButton v-if="stand === 'mit_schluessel' && !offen" variant="secondary" @click="offen = true">
        <ShieldCheck class="w-4 h-4" /> {{ t('app.matrix.bestaetigenKnopf') }}
      </BaseButton>
      <BaseButton v-if="stand === 'anlegen'" variant="secondary" :disabled="arbeitet" @click="anlegen">
        <Loader2 v-if="arbeitet" class="w-4 h-4 animate-spin" /><KeyRound v-else class="w-4 h-4" /> {{ t('app.matrix.anlegenKnopf') }}
      </BaseButton>

      <form v-if="stand === 'mit_schluessel' && offen" class="space-y-2" @submit.prevent="bestaetigen">
        <label class="block">
          <span class="block text-sm font-bold text-fliess">{{ t('app.matrix.schluesselLabel') }}</span>
          <PasswortFeld v-model="schluessel" required autocomplete="off"
                        class="mt-1 w-full px-3 py-2 rounded-xl border border-linie bg-vertieft text-schrift text-sm font-mono" />
          <span class="mt-1 block text-xs text-leise">{{ t('app.matrix.schluesselHinweis') }}</span>
        </label>
        <div class="flex flex-wrap gap-2">
          <BaseButton type="submit" :disabled="arbeitet || !schluessel.trim()">
            <Loader2 v-if="arbeitet" class="w-4 h-4 animate-spin" /><ShieldCheck v-else class="w-4 h-4" /> {{ t('app.matrix.bestaetigenKnopf') }}
          </BaseButton>
          <BaseButton variant="secondary" :disabled="arbeitet" @click="offen = false; schluessel = ''; fehler = ''">{{ t('common.cancel') }}</BaseButton>
        </div>
      </form>
    </template>
  </template>

  <p v-if="fehler" class="text-sm text-warnung flex items-start gap-2">
    <AlertTriangle class="w-4 h-4 shrink-0 mt-0.5" /><span class="min-w-0 break-words">{{ fehler }}</span>
  </p>
</template>
