<script setup>
/*
 * Die Kachel „Postfach" -- E-Mail als dritter Weg im Verlauf
 * (docs/plan-email-pgp.md, Schritt 2).
 *
 * VERBINDEN IN ZWEI SCHRITTEN. Erst Adresse und Passwort; die Server sucht
 * das Programm selbst (Autokonfiguration wie in Thunderbird). Findet es
 * keine, oder will jemand andere, klappt „Server von Hand" auf.
 *
 * DAS PASSWORT BLEIBT AUF DIESEM GERÄT, im Tresor (Android-Keystore). Das
 * Gerät spricht selbst mit dem Mailserver; openany.de sieht keine Mail.
 *
 * MEHRERE POSTFÄCHER, EIN VERLAUF. Das erste ist das Standard-Postfach für
 * neue Mails; eine Antwort geht immer von dem, an das die Mail kam.
 */
import { ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { Mail, Loader2, AlertTriangle, ChevronDown, ChevronUp, RefreshCw, Plus, Star } from 'lucide-vue-next';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import { useToast } from '@oberflaeche/composables/useToast';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import { mailQuelle } from '../quellen/nachrichten';
import EigenerSchluessel from './EigenerSchluessel.vue';

const { t } = useI18n();
const toast = useToast();
const { confirmDelete } = useConfirm();

const lage = ref(null);
// Welches Postfach gerade abholt oder getrennt wird ('' = das Formular).
const arbeitet = ref(null);
const hinzufuegen = ref(false);
const fehler = ref('');
const vonHand = ref(false);

const leer = () => ({
    adresse: '',
    anzeigename: '',
    passwort: '',
    benutzer: '',
    imap: { host: '', port: 993, sicherheit: 'ssl' },
    smtp: { host: '', port: 465, sicherheit: 'ssl' },
});
const form = ref(leer());
let gesucht = '';

async function lesen() {
    try { lage.value = await mailQuelle.lage(); } catch (e) { lage.value = { postfaecher: [], fehler: String(e) }; }
}

/** Die Server zur Adresse suchen -- einmal je Adresse. */
async function suchen() {
    const adresse = form.value.adresse.trim();
    if (!adresse.includes('@') || adresse === gesucht) return;
    gesucht = adresse;
    const g = await mailQuelle.serverFinden(adresse).catch(() => null);
    if (g) {
        form.value.imap = { ...g.imap };
        form.value.smtp = { ...g.smtp };
        form.value.benutzer = g.benutzer;
    } else {
        vonHand.value = true;
    }
}

async function verbinden() {
    fehler.value = '';
    arbeitet.value = '';
    try {
        await suchen();
        if (!form.value.imap.host || !form.value.smtp.host) {
            vonHand.value = true;
            throw new Error(t('app.postfach.keineServer'));
        }
        const f = form.value;
        lage.value = await mailQuelle.verbinden({
            adresse: f.adresse.trim(),
            anzeigename: f.anzeigename.trim(),
            benutzer: (f.benutzer || f.adresse).trim(),
            passwort: f.passwort,
            imap: { host: f.imap.host.trim(), port: Number(f.imap.port), sicherheit: f.imap.sicherheit },
            smtp: { host: f.smtp.host.trim(), port: Number(f.smtp.port), sicherheit: f.smtp.sicherheit },
        });
        form.value = leer();
        gesucht = '';
        vonHand.value = false;
        hinzufuegen.value = false;
        toast.success(t('app.postfach.verbunden'));
    } catch (e) {
        fehler.value = String(e?.message ?? e);
    } finally {
        arbeitet.value = null;
    }
}

async function abholen(adresse) {
    arbeitet.value = adresse;
    try {
        const neu = await mailQuelle.abholen(adresse);
        toast.success(neu ? t('app.postfach.neueMails', neu) : t('app.postfach.keineNeuenMails'));
    } catch (e) {
        toast.error(String(e));
    } finally {
        arbeitet.value = null;
        await lesen();
    }
}

async function trennen(adresse) {
    if (!(await confirmDelete(t('app.postfach.trennenFrage', { adresse })))) return;
    arbeitet.value = adresse;
    try {
        await mailQuelle.trennen(adresse);
        toast.success(t('app.postfach.getrennt'));
    } catch (e) {
        toast.error(String(e));
    } finally {
        arbeitet.value = null;
        await lesen();
    }
}

function abbrechen() {
    hinzufuegen.value = false;
    vonHand.value = false;
    fehler.value = '';
    form.value = leer();
    gesucht = '';
}

async function standard(adresse) {
    try { lage.value = await mailQuelle.standard(adresse); } catch (e) { toast.error(String(e)); }
}

onMounted(lesen);

const feld = 'mt-1 w-full px-3 py-2 rounded-xl border border-linie bg-vertieft text-schrift text-sm';
</script>

<template>
  <div class="karte p-6 shadow-sm space-y-4">
    <h3 class="text-lg font-extrabold text-schrift flex items-center gap-2">
      <Mail class="w-5 h-5 text-marke" /> {{ t('app.postfach.titel') }}
    </h3>

    <div v-if="!lage" class="flex justify-center py-4"><Loader2 class="w-5 h-5 animate-spin text-leise" /></div>

    <template v-else>
      <ul v-if="lage.postfaecher.length" class="space-y-3">
        <li v-for="(p, i) in lage.postfaecher" :key="p.adresse" class="rounded-xl border border-linie p-3 space-y-2">
          <div class="flex items-center gap-2 min-w-0">
            <strong class="text-schrift break-all">{{ p.adresse }}</strong>
            <span v-if="i === 0 && lage.postfaecher.length > 1"
                  class="px-2 py-0.5 rounded-full font-extrabold text-[10px] shrink-0 bg-marke-leise text-marke">{{ t('app.allgemein.standard') }}</span>
          </div>
          <p v-if="p.fehler" class="text-sm text-warnung flex items-start gap-2">
            <AlertTriangle class="w-4 h-4 shrink-0 mt-0.5" /><span class="min-w-0 break-words">{{ p.fehler }}</span>
          </p>
          <div class="flex flex-wrap gap-2">
            <BaseButton variant="secondary" :disabled="arbeitet !== null" @click="abholen(p.adresse)">
              <Loader2 v-if="arbeitet === p.adresse" class="w-4 h-4 animate-spin" /><RefreshCw v-else class="w-4 h-4" /> {{ t('app.postfach.jetztAbholen') }}
            </BaseButton>
            <BaseButton v-if="i > 0" variant="secondary" :disabled="arbeitet !== null" @click="standard(p.adresse)">
              <Star class="w-4 h-4" /> {{ t('app.allgemein.alsStandard') }}
            </BaseButton>
            <BaseButton variant="danger" :disabled="arbeitet !== null" @click="trennen(p.adresse)">{{ t('app.postfach.trennen') }}</BaseButton>
          </div>
          <EigenerSchluessel :adresse="p.adresse" :fingerabdruck="p.pgp" @lage="lage = $event" />
        </li>
      </ul>
      <p v-if="lage.postfaecher.length" class="text-sm text-leise">
        {{ t('app.postfach.gemeinsam') }}
        <template v-if="lage.postfaecher.length > 1">{{ t('app.postfach.mehrere') }}</template>
      </p>
      <BaseButton v-if="lage.postfaecher.length && !hinzufuegen" variant="secondary" @click="hinzufuegen = true">
        <Plus class="w-4 h-4" /> {{ t('app.postfach.hinzufuegen') }}
      </BaseButton>

    <form v-if="!lage.postfaecher.length || hinzufuegen" class="space-y-3" @submit.prevent="verbinden">
      <p class="text-sm text-leise">{{ t('app.postfach.einleitung') }}</p>
      <label class="block">
        <span class="block text-sm font-bold text-fliess">{{ t('app.postfach.adresse') }}</span>
        <input v-model="form.adresse" type="email" required autocomplete="email" autocapitalize="off" :class="feld" @blur="suchen" />
      </label>
      <label class="block">
        <span class="block text-sm font-bold text-fliess">{{ t('app.postfach.passwort') }}</span>
        <input v-model="form.passwort" type="password" required autocomplete="current-password" :class="feld" />
        <span class="mt-1 block text-xs text-leise">{{ t('app.postfach.passwortHinweis') }}</span>
      </label>
      <label class="block">
        <span class="block text-sm font-bold text-fliess">{{ t('app.postfach.absendername') }} <span class="font-normal text-leise">{{ t('app.allgemein.optional') }}</span></span>
        <input v-model="form.anzeigename" type="text" autocomplete="name" :class="feld" />
      </label>

      <button type="button" class="text-sm font-bold text-marke flex items-center gap-1 cursor-pointer" @click="vonHand = !vonHand">
        <component :is="vonHand ? ChevronUp : ChevronDown" class="w-4 h-4" /> {{ t('app.postfach.vonHand') }}
      </button>
      <div v-if="vonHand" class="space-y-3 rounded-xl border border-linie p-3">
        <label class="block">
          <span class="block text-sm font-bold text-fliess">{{ t('app.postfach.benutzer') }}</span>
          <input v-model="form.benutzer" type="text" autocapitalize="off" :placeholder="form.adresse" :class="feld" />
        </label>
        <div v-for="art in ['imap', 'smtp']" :key="art" class="grid grid-cols-[1fr_5rem_7rem] gap-2 items-end">
          <label class="block">
            <span class="block text-sm font-bold text-fliess">{{ art === 'imap' ? t('app.postfach.imap') : t('app.postfach.smtp') }}</span>
            <input v-model="form[art].host" type="text" autocapitalize="off" :class="feld" />
          </label>
          <label class="block">
            <span class="block text-sm font-bold text-fliess">{{ t('app.postfach.port') }}</span>
            <input v-model.number="form[art].port" type="number" min="1" max="65535" :class="feld" />
          </label>
          <label class="block">
            <span class="block text-sm font-bold text-fliess">{{ t('app.postfach.sicherheit') }}</span>
            <select v-model="form[art].sicherheit" :class="feld">
              <option value="ssl">SSL/TLS</option>
              <option value="starttls">STARTTLS</option>
            </select>
          </label>
        </div>
      </div>

      <p v-if="fehler" class="text-sm text-warnung flex items-start gap-2">
        <AlertTriangle class="w-4 h-4 shrink-0 mt-0.5" /><span class="min-w-0 break-words">{{ fehler }}</span>
      </p>
      <div class="flex flex-wrap gap-2">
        <BaseButton type="submit" :disabled="arbeitet !== null">
          <Loader2 v-if="arbeitet === ''" class="w-4 h-4 animate-spin" /><Mail v-else class="w-4 h-4" /> {{ t('app.postfach.verbinden') }}
        </BaseButton>
        <BaseButton v-if="hinzufuegen" variant="secondary" :disabled="arbeitet !== null" @click="abbrechen">{{ t('common.cancel') }}</BaseButton>
      </div>
    </form>
    </template>
  </div>
</template>
