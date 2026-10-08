<script setup>
/*
 * „Schlüssel der Kontakte" -- die öffentlichen OpenPGP-Schlüssel der
 * Gegenüber (docs/plan-email-pgp.md, Schritt 3a).
 *
 * NUR AUF DIESEM GERÄT, nicht im Abgleich (Tiffy, 29.09.2026): Sonst sähe
 * openany.de, mit wem verschlüsselt geschrieben wird.
 *
 * WOHER SIE KOMMEN: von selbst aus Mails mit Autocrypt-Kopfzeile, und beim
 * Suchen beim Mailanbieter des Gegenübers (WKD). keys.openpgp.org nur, wenn
 * das Häkchen gesetzt ist -- der Schlüsselserver erführe sonst nebenbei,
 * wem man schreibt. Und von Hand, als Datei.
 */
import { ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { KeyRound, Loader2, AlertTriangle, Search, Upload, Trash2, Check } from 'lucide-vue-next';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import { useToast } from '@oberflaeche/composables/useToast';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import { mailQuelle } from '../quellen/nachrichten';
import { fingerabdruckLesbar } from './pgp';

const { t } = useI18n();
const toast = useToast();
const { confirmDelete } = useConfirm();

const liste = ref(null);
const suche = ref('');
const schluesselserver = ref(false);
const sucht = ref(false);
const fehler = ref('');

const QUELLEN = {
    autocrypt: 'autocrypt',
    wkd: 'wkd',
    'keys.openpgp.org': 'keyserver',
    hand: 'hand',
};
const quelleText = (q) => (QUELLEN[q] ? t(`app.pgp.quelle.${QUELLEN[q]}`) : q);

async function laden() {
    try { liste.value = await mailQuelle.pgpListe(); } catch (e) { liste.value = []; fehler.value = String(e); }
}

async function suchen() {
    const adresse = suche.value.trim();
    if (!adresse) return;
    sucht.value = true;
    fehler.value = '';
    try {
        const k = await mailQuelle.pgpSuchen(adresse, schluesselserver.value);
        if (k) {
            toast.success(t('app.pgp.gefunden', { adresse: k.adresse }));
            suche.value = '';
        } else {
            fehler.value = schluesselserver.value
                ? t('app.pgp.nirgends', { adresse })
                : t('app.pgp.nichtBeimAnbieter', { adresse });
        }
        await laden();
    } catch (e) {
        fehler.value = String(e);
    } finally {
        sucht.value = false;
    }
}

async function datei(ereignis) {
    const f = ereignis.target.files?.[0];
    ereignis.target.value = '';
    if (!f) return;
    fehler.value = '';
    try {
        const zeilen = await mailQuelle.pgpHand(await f.text());
        toast.success(t('app.pgp.eingelesen', { adressen: zeilen.map((z) => z.adresse).join(', ') }));
        await laden();
    } catch (e) {
        fehler.value = String(e);
    }
}

async function vergessen(k) {
    if (!(await confirmDelete(t('app.pgp.loeschenFrage', { adresse: k.adresse })))) return;
    await mailQuelle.pgpVergessen(k.adresse);
    await laden();
}

async function gesehen(k) {
    await mailQuelle.pgpWechselGesehen(k.adresse);
    await laden();
}

onMounted(laden);

const feld = 'w-full px-3 py-2 rounded-xl border border-linie bg-vertieft text-schrift text-sm';
</script>

<template>
  <div class="karte p-6 shadow-sm space-y-4">
    <h3 class="text-lg font-extrabold text-schrift flex items-center gap-2">
      <KeyRound class="w-5 h-5 text-marke" /> {{ t('app.pgp.titel') }}
    </h3>
    <p class="text-sm text-leise">{{ t('app.pgp.hinweis') }}</p>

    <div v-if="!liste" class="flex justify-center py-4"><Loader2 class="w-5 h-5 animate-spin text-leise" /></div>
    <p v-else-if="!liste.length" class="text-sm text-fliess">{{ t('app.pgp.keine') }}</p>
    <ul v-else class="space-y-3">
      <li v-for="k in liste" :key="k.adresse" class="rounded-xl border border-linie p-3 space-y-1">
        <div class="flex items-center justify-between gap-2">
          <strong class="text-sm text-schrift break-all">{{ k.adresse }}</strong>
          <button type="button" class="text-rose-600 dark:text-rose-400 cursor-pointer shrink-0" :aria-label="t('app.pgp.loeschenVon', { adresse: k.adresse })" @click="vergessen(k)">
            <Trash2 class="w-4 h-4" />
          </button>
        </div>
        <p class="font-mono text-xs text-fliess break-all">{{ fingerabdruckLesbar(k.fingerabdruck) }}</p>
        <p class="text-xs text-leise">{{ quelleText(k.quelle) }}</p>
        <div v-if="k.vorher" class="text-xs text-warnung flex flex-wrap items-start gap-2">
          <AlertTriangle class="w-4 h-4 shrink-0" />
          <span class="min-w-0 flex-1 break-words">{{ t('app.pgp.geaendert', { vorher: fingerabdruckLesbar(k.vorher) }) }}</span>
          <button type="button" class="font-bold text-marke flex items-center gap-1 cursor-pointer" @click="gesehen(k)">
            <Check class="w-3.5 h-3.5" /> {{ t('app.pgp.gesehen') }}
          </button>
        </div>
      </li>
    </ul>

    <form class="space-y-2" @submit.prevent="suchen">
      <span class="block text-sm font-bold text-fliess">{{ t('app.pgp.suchen') }}</span>
      <div class="flex gap-2">
        <input v-model="suche" type="email" autocapitalize="off" :placeholder="t('app.pgp.suchenPlatzhalter')" :class="feld" />
        <BaseButton type="submit" variant="secondary" :disabled="sucht || !suche.trim()">
          <Loader2 v-if="sucht" class="w-4 h-4 animate-spin" /><Search v-else class="w-4 h-4" />
        </BaseButton>
      </div>
      <label class="flex items-start gap-2 text-xs text-leise">
        <input v-model="schluesselserver" type="checkbox" class="mt-0.5" />
        <span>{{ t('app.pgp.keyserver') }}</span>
      </label>
    </form>

    <label class="inline-flex items-center gap-2 text-sm font-bold text-marke cursor-pointer">
      <Upload class="w-4 h-4" /> {{ t('app.pgp.ausDatei') }}
      <input type="file" accept=".asc,.gpg,.key,.pgp,text/plain,application/pgp-keys" class="hidden" @change="datei" />
    </label>

    <p v-if="fehler" class="text-sm text-warnung flex items-start gap-2">
      <AlertTriangle class="w-4 h-4 shrink-0 mt-0.5" /><span class="min-w-0 break-words">{{ fehler }}</span>
    </p>
  </div>
</template>
