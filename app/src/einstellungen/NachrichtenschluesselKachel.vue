<script setup>
/*
 * Die Kachel „Meine Geräte" (bis 21.10.2026 „Nachrichtenschlüssel",
 * docs/plan-direktnachrichten-e2e.md, docs/plan-zero-knowledge.md).
 *
 * Konto-Modell: Ein Schlüssel für alle eigenen Geräte. Ein neues Gerät zeigt
 * 6 Ziffern, ein freigegebenes tippt sie ein (Z4, SPAKE2 -- bis dahin wurden
 * zwei Codes verglichen, und die konnte ein lügender Server fälschen). Ein
 * verlorenes Gerät wird zurückgezogen; dabei wechselt der Schlüssel, und die
 * übrigen Geräte bekommen den neuen (direktpost.rs).
 */
import { ref, watch, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { KeyRound, Loader2, Check, X, RefreshCw, Smartphone } from 'lucide-vue-next';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import { useToast } from '@oberflaeche/composables/useToast';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import FreischaltungNeu from './FreischaltungNeu.vue';
import FreischaltAnfragen from '../ansichten/FreischaltAnfragen.vue';

const { t } = useI18n();
const emit = defineEmits(['lage-geaendert']);
const toast = useToast();
const { confirmDialog } = useConfirm();

const lage = ref(null);
const fehler = ref('');
const arbeitet = ref(false);

async function laden() {
    fehler.value = '';
    try {
        lage.value = await invoke('nachrichtenschluessel_lage');
    } catch (e) {
        fehler.value = String(e);
    }
}

// Jeder Befehl steht wörtlich als Aufruf da: Die Verdrahtungsprüfung
// (src-tauri/tests/verdrahtung.rs) findet ihn sonst nicht.
async function tun(aufruf, erfolg = null) {
    arbeitet.value = true;
    try {
        lage.value = await aufruf();
        if (erfolg) toast.success(erfolg);
    } catch (e) {
        toast.error(String(e));
    } finally {
        arbeitet.value = false;
    }
}

const ablehnen = (g) => tun(() => invoke('nachrichtenschluessel_ablehnen', { geraet: g.geraet }));

async function zurueckziehen(g) {
    if (!(await confirmDialog(t('app.nachrichtenschluessel.zurueckziehenFrage', { name: g.name }), {
        title: t('app.nachrichtenschluessel.zurueckziehen'), confirmLabel: t('app.nachrichtenschluessel.zurueckziehen'),
    }))) return;
    await tun(() => invoke('nachrichtenschluessel_zurueckziehen', { geraet: g.geraet }), t('app.nachrichtenschluessel.gewechselt'));
}

async function wechseln() {
    if (!(await confirmDialog(t('app.nachrichtenschluessel.wechselnFrage'), {
        title: t('app.nachrichtenschluessel.wechseln'), confirmLabel: t('app.nachrichtenschluessel.wechseln'),
    }))) return;
    await tun(() => invoke('nachrichtenschluessel_wechseln'), t('app.nachrichtenschluessel.gewechselt'));
}

/*
 * Solange dieses Gerät eine Kiste erwartet (eben freigeschaltet), alle fünf
 * Sekunden nachsehen; nach dem Freischalten eines anderen ein paar Mal.
 */
let takt = null;
watch(lage, (l) => {
    clearInterval(takt);
    takt = l?.stand === 'wartet' && l?.kiste_erwartet ? setInterval(laden, 5000) : null;
});
onUnmounted(() => clearInterval(takt));

function spaeterLaden() {
    [5000, 15000, 30000, 60000].forEach((ms) => setTimeout(laden, ms));
}

/*
 * KONTO LÖSCHEN (docs/plan-konto-loeschen.md, Tiffy 22.10.2026): unten in
 * „Meine Geräte“, abgesetzt. Bestätigt wird mit dem eigenen Anmeldenamen,
 * gelöscht wird sofort. Vorher müssen Projekte mit weiteren Mitgliedern
 * übergeben sein -- das sagt der Server, und hier steht es in der eigenen
 * Sprache.
 */
const loeschenOffen = ref(false);
const loeschName = ref('');
const loeschFehler = ref('');
const loescht = ref(false);

async function kontoLoeschen() {
    loeschFehler.value = '';
    loescht.value = true;
    try {
        await invoke('konto_loeschen', { name: loeschName.value });
        toast.success(t('settings.konto.geloescht'));
        loeschenOffen.value = false;
        emit('lage-geaendert');
        await laden();
    } catch (e) {
        const text = String(e);
        const grund = text.startsWith('grund:') ? text.slice(6) : null;
        loeschFehler.value = grund && ['name_falsch', 'projekte_uebergeben', 'letzter_verwalter', 'anyid'].includes(grund)
            ? t(`settings.konto.grund.${grund}`)
            : text;
    } finally {
        loescht.value = false;
    }
}

const stand = (g) => (!g.aktiv ? 'zurueckgezogen' : g.freigegeben ? 'freigegeben' : g.wartet ? 'wartet' : 'abgelehnt');

onMounted(laden);
</script>

<template>
  <div class="karte p-6 shadow-sm space-y-4">
    <h3 class="text-lg font-extrabold text-schrift flex items-center gap-2">
      <KeyRound class="w-5 h-5 text-marke" /> {{ t('app.nachrichtenschluessel.titel') }}
    </h3>

    <p v-if="fehler" class="text-sm text-leise">{{ t('app.nachrichtenschluessel.nichtVerbunden') }}</p>
    <p v-else-if="!lage" class="text-sm text-leise flex items-center gap-2"><Loader2 class="w-4 h-4 animate-spin" /> …</p>

    <template v-else>
      <p class="text-sm text-fliess">{{ t('app.nachrichtenschluessel.hinweis') }}</p>

      <div v-if="lage.stand === 'wartet'" class="rounded-xl bg-vertieft p-4 space-y-1">
        <p class="text-sm font-bold text-schrift">{{ t('app.nachrichtenschluessel.wartet') }}</p>
        <p v-if="lage.kiste_erwartet" class="text-sm text-fliess flex items-center gap-2">
          <Loader2 class="w-4 h-4 animate-spin" /> {{ t('app.freischalten.kommtGleich') }}
        </p>
        <template v-else>
          <p class="text-sm text-fliess">{{ t('app.nachrichtenschluessel.wartetHinweis') }}</p>
          <FreischaltungNeu nur-schluessel @lage-geaendert="laden" />
        </template>
      </div>
      <template v-else>
        <p class="text-sm font-bold text-marke flex items-center gap-1.5">
          <Check class="w-4 h-4" /> {{ t('app.nachrichtenschluessel.bereit') }}
        </p>
        <FreischaltAnfragen eingebettet @freigeschaltet="spaeterLaden" />
      </template>

      <ul class="divide-y divide-linie border border-linie rounded-xl overflow-hidden">
        <li v-for="g in lage.geraete" :key="g.geraet" class="px-3 py-2.5 flex flex-wrap items-center gap-2">
          <Smartphone class="w-4 h-4 text-leise shrink-0" />
          <span class="text-sm font-semibold text-schrift flex-1 min-w-0 break-words">
            {{ g.name || t('app.nachrichtenschluessel.ohneName') }}
            <span v-if="g.dieses" class="text-leise font-normal">({{ t('app.nachrichtenschluessel.dieses') }})</span>
          </span>
          <span class="text-xs text-leise">{{ t(`app.nachrichtenschluessel.stand.${stand(g)}`) }}</span>
          <!-- Freigegeben wird seit Z4 nur noch über die 6 Ziffern (oben);
               ablehnen geht weiter. -->
          <BaseButton v-if="g.wartet && lage.stand === 'bereit'" groesse="klein" variant="secondary" :disabled="arbeitet" @click="ablehnen(g)"><X class="w-4 h-4" /> {{ t('app.nachrichtenschluessel.ablehnen') }}</BaseButton>
          <BaseButton v-else-if="!g.dieses && g.aktiv && g.freigegeben && lage.stand === 'bereit'"
                      groesse="klein" variant="ghost" :disabled="arbeitet" @click="zurueckziehen(g)">
            {{ t('app.nachrichtenschluessel.zurueckziehen') }}
          </BaseButton>
        </li>
      </ul>

      <BaseButton v-if="lage.stand === 'bereit'" variant="secondary" :disabled="arbeitet" @click="wechseln">
        <Loader2 v-if="arbeitet" class="w-4 h-4 animate-spin" /><RefreshCw v-else class="w-4 h-4" />
        {{ t('app.nachrichtenschluessel.wechseln') }}
      </BaseButton>
    </template>

    <!-- Konto löschen: abgesetzt, erst auf Knopfdruck offen. -->
    <div v-if="lage && !fehler" class="mt-6 pt-4 border-t border-linie space-y-2">
      <button v-if="!loeschenOffen" class="text-sm text-rose-600 dark:text-rose-400 underline" @click="loeschenOffen = true">
        {{ t('settings.konto.loeschen') }}
      </button>
      <form v-else class="rounded-xl border border-rose-300 dark:border-rose-800 p-4 space-y-2" @submit.prevent="kontoLoeschen">
        <p class="text-sm font-bold text-schrift">{{ t('settings.konto.loeschen') }}</p>
        <p class="text-sm text-fliess">{{ t('settings.konto.hinweis') }}</p>
        <label class="block text-sm text-fliess">
          {{ t('settings.konto.nameFrage') }}
          <input v-model="loeschName" type="text" autocomplete="off" autocapitalize="off" class="feld w-full mt-1">
        </label>
        <div class="flex flex-wrap gap-2">
          <BaseButton type="submit" variant="danger" :disabled="loescht || !loeschName.trim()">
            <Loader2 v-if="loescht" class="w-4 h-4 animate-spin" />
            {{ t('settings.konto.endgueltig') }}
          </BaseButton>
          <BaseButton type="button" variant="secondary" :disabled="loescht" @click="loeschenOffen = false; loeschFehler = ''">
            {{ t('common.cancel') }}
          </BaseButton>
        </div>
        <p v-if="loeschFehler" class="text-sm text-rose-600 dark:text-rose-400">{{ loeschFehler }}</p>
      </form>
    </div>
  </div>
</template>
