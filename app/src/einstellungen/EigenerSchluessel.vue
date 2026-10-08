<script setup>
/*
 * Der eigene OpenPGP-Schlüssel eines Postfachs (docs/plan-email-pgp.md,
 * Schritt 3a) -- erzeugen, einlesen, entfernen.
 *
 * DER GEHEIME SCHLÜSSEL BLEIBT AUF DIESEM GERÄT, im Tresor. Eingelesen wird
 * er als Datei (etwa der Export aus Thunderbird) samt Passphrase; hier
 * entsperrt, denn der Tresor ist selbst verschlossen.
 *
 * Mit Schlüssel trägt jede Mail aus diesem Postfach ihn als Autocrypt-
 * Kopfzeile -- so findet das Gegenüber ihn von selbst.
 */
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { KeyRound, Loader2, Upload, AlertTriangle, Download, Share2 } from 'lucide-vue-next';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import { useToast } from '@oberflaeche/composables/useToast';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import { mailQuelle } from '../quellen/nachrichten';
import { fingerabdruckLesbar, aufsGeraet } from './pgp';

const props = defineProps({
    adresse: { type: String, required: true },
    fingerabdruck: { type: String, default: null },
});
const emit = defineEmits(['lage']);

const { t } = useI18n();
const toast = useToast();
const { confirmDelete } = useConfirm();
const arbeitet = ref(false);
const einlesenOffen = ref(false);
const datei = ref(null);
const passphrase = ref('');
const fehler = ref('');
const sicherungOffen = ref(false);
const sicherung = ref({ eins: '', zwei: '' });

async function erzeugen() {
    arbeitet.value = true;
    fehler.value = '';
    try {
        emit('lage', await mailQuelle.pgpErzeugen(props.adresse));
        toast.success(t('app.schluessel.erzeugt'));
    } catch (e) {
        fehler.value = String(e);
    } finally {
        arbeitet.value = false;
    }
}

async function einlesen() {
    if (!datei.value) return;
    arbeitet.value = true;
    fehler.value = '';
    try {
        const text = await datei.value.text();
        emit('lage', await mailQuelle.pgpEinlesen(props.adresse, text, passphrase.value));
        passphrase.value = '';
        datei.value = null;
        einlesenOffen.value = false;
        toast.success(t('app.schluessel.eingelesen'));
    } catch (e) {
        fehler.value = String(e);
    } finally {
        arbeitet.value = false;
    }
}

async function entfernen() {
    if (!(await confirmDelete(t('app.schluessel.entfernenFrage')))) return;
    try {
        emit('lage', await mailQuelle.pgpEntfernen(props.adresse));
    } catch (e) {
        toast.error(String(e));
    }
}

/*
 * DIE SICHERUNGSKOPIE geht in „Download" auf dem Gerät, nicht in „Dateien":
 * Die gehen in den Abgleich, und der geheime Schlüssel gehört nicht auf den
 * Server -- auch nicht mit Passphrase.
 */
async function sichern() {
    fehler.value = '';
    if (sicherung.value.eins.length < 8) { fehler.value = t('app.schluessel.zuKurz'); return; }
    if (sicherung.value.eins !== sicherung.value.zwei) { fehler.value = t('app.schluessel.ungleich'); return; }
    arbeitet.value = true;
    try {
        const text = await mailQuelle.pgpAusfuhr(props.adresse, sicherung.value.eins);
        const ort = aufsGeraet(`openany-${props.adresse}-geheimer-schluessel.asc`, 'application/pgp-keys', text);
        sicherung.value = { eins: '', zwei: '' };
        sicherungOffen.value = false;
        toast.success(t('app.schluessel.gesichert', { ort }));
    } catch (e) {
        fehler.value = String(e?.message ?? e);
    } finally {
        arbeitet.value = false;
    }
}

async function oeffentlichSpeichern() {
    fehler.value = '';
    try {
        const text = await mailQuelle.pgpOeffentlich(props.adresse);
        const ort = aufsGeraet(`openany-${props.adresse}-oeffentlicher-schluessel.asc`, 'application/pgp-keys', text);
        toast.success(t('app.schluessel.oeffentlichGesichert', { ort }));
    } catch (e) {
        fehler.value = String(e?.message ?? e);
    }
}

const feld = 'mt-1 w-full px-3 py-2 rounded-xl border border-linie bg-vertieft text-schrift text-sm';
</script>

<template>
  <div class="pt-2 border-t border-linie space-y-2">
    <p class="text-sm font-bold text-fliess flex items-center gap-1.5">
      <KeyRound class="w-4 h-4 text-marke" /> OpenPGP
    </p>

    <template v-if="fingerabdruck">
      <p class="text-xs text-leise">{{ t('app.schluessel.fingerabdruck') }}</p>
      <p class="font-mono text-xs text-schrift break-all">{{ fingerabdruckLesbar(fingerabdruck) }}</p>
      <div v-if="!sicherungOffen" class="flex flex-wrap gap-x-4 gap-y-2 text-sm font-bold">
        <button type="button" class="text-marke flex items-center gap-1 cursor-pointer" @click="sicherungOffen = true">
          <Download class="w-4 h-4" /> {{ t('app.schluessel.sicherung') }}
        </button>
        <button type="button" class="text-marke flex items-center gap-1 cursor-pointer" @click="oeffentlichSpeichern">
          <Share2 class="w-4 h-4" /> {{ t('app.schluessel.oeffentlich') }}
        </button>
        <button type="button" class="text-rose-600 dark:text-rose-400 cursor-pointer" @click="entfernen">
          {{ t('app.schluessel.entfernen') }}
        </button>
      </div>
      <form v-else class="space-y-2" @submit.prevent="sichern">
        <p class="text-xs text-leise">{{ t('app.schluessel.sicherungHinweis') }}</p>
        <label class="block">
          <span class="block text-sm font-bold text-fliess">{{ t('app.schluessel.passphraseNeu') }}</span>
          <input v-model="sicherung.eins" type="password" autocomplete="new-password" :class="feld" />
        </label>
        <label class="block">
          <span class="block text-sm font-bold text-fliess">{{ t('app.schluessel.nochEinmal') }}</span>
          <input v-model="sicherung.zwei" type="password" autocomplete="new-password" :class="feld" />
        </label>
        <div class="flex flex-wrap gap-2">
          <BaseButton type="submit" :disabled="arbeitet">
            <Loader2 v-if="arbeitet" class="w-4 h-4 animate-spin" /><Download v-else class="w-4 h-4" /> {{ t('common.save') }}
          </BaseButton>
          <BaseButton variant="secondary" :disabled="arbeitet" @click="sicherungOffen = false; fehler = ''; sicherung = { eins: '', zwei: '' }">{{ t('common.cancel') }}</BaseButton>
        </div>
      </form>
    </template>

    <template v-else>
      <p class="text-xs text-leise">{{ t('app.schluessel.keiner') }}</p>
      <div v-if="!einlesenOffen" class="flex flex-wrap gap-2">
        <BaseButton variant="secondary" :disabled="arbeitet" @click="erzeugen">
          <Loader2 v-if="arbeitet" class="w-4 h-4 animate-spin" /><KeyRound v-else class="w-4 h-4" /> {{ t('app.schluessel.erzeugen') }}
        </BaseButton>
        <BaseButton variant="secondary" :disabled="arbeitet" @click="einlesenOffen = true">
          <Upload class="w-4 h-4" /> {{ t('app.schluessel.einlesen') }}
        </BaseButton>
      </div>
      <form v-else class="space-y-2" @submit.prevent="einlesen">
        <label class="block">
          <span class="block text-sm font-bold text-fliess">{{ t('app.schluessel.datei') }}</span>
          <input type="file" accept=".asc,.gpg,.key,.pgp,text/plain,application/pgp-keys" class="mt-1 block w-full text-sm text-fliess"
                 @change="datei = $event.target.files?.[0] ?? null" />
        </label>
        <label class="block">
          <span class="block text-sm font-bold text-fliess">{{ t('app.schluessel.passphrase') }} <span class="font-normal text-leise">{{ t('app.schluessel.passphraseLeer') }}</span></span>
          <input v-model="passphrase" type="password" autocomplete="off" :class="feld" />
        </label>
        <div class="flex flex-wrap gap-2">
          <BaseButton type="submit" :disabled="arbeitet || !datei">
            <Loader2 v-if="arbeitet" class="w-4 h-4 animate-spin" /><Upload v-else class="w-4 h-4" /> {{ t('app.schluessel.einlesenKnopf') }}
          </BaseButton>
          <BaseButton variant="secondary" :disabled="arbeitet" @click="einlesenOffen = false; fehler = ''">{{ t('common.cancel') }}</BaseButton>
        </div>
      </form>
    </template>

    <p v-if="fehler" class="text-sm text-warnung flex items-start gap-2">
      <AlertTriangle class="w-4 h-4 shrink-0 mt-0.5" /><span class="min-w-0 break-words">{{ fehler }}</span>
    </p>
  </div>
</template>
