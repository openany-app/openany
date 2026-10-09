<script setup>
/*
 * Die Kachel „Sicherung" (08.10.2026): alles von openany auf diesem Gerät in
 * einer verschlüsselten Datei -- und eine solche Datei wieder einspielen.
 * Sie ersetzt den alten Stick (sicherungsbefehle.rs).
 *
 * WOHIN UND WOHER: Auf Android wählt man den Ort in der Auswahl des Systems
 * (auch ein Stick); die Antwort kommt als Ereignis `openany-sicherung`
 * (MainActivity.kt). Auf dem Schreibtisch öffnen sich die Dialoge des
 * Systems (ablagebefehle.rs).
 *
 * EINSPIELEN ERSETZT. Erst öffnen (Passphrase prüfen, Kopf zeigen), dann
 * nachfragen, dann neu starten -- getauscht wird beim Start.
 */
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { Archive, Download, Upload, RefreshCw, Loader2, AlertTriangle } from 'lucide-vue-next';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import { useToast } from '@oberflaeche/composables/useToast';

const { t, locale } = useI18n();
const toast = useToast();

const android = typeof window !== 'undefined' && !!window.openanyAblage?.sicherungWaehlen;

// '' | 'anlegen' | 'oeffnen' | 'bestaetigen'
const schritt = ref('');
const arbeitet = ref(false);
const fehler = ref('');
const regel = ref('');

// Anlegen
const vorschlag = ref('');
const eigene = ref(false);
const eigeneText = ref('');
const aufgeschrieben = ref(false);

// Einspielen
const quelle = ref('');
const passphrase = ref('');
const kopf = ref(null);

const MINDESTENS = 12;
const glatt = (p) => p.trim().split(/\s+/).join(' ').toLowerCase();
const gewaehlt = computed(() => (eigene.value ? glatt(eigeneText.value) : vorschlag.value));
const kannAnlegen = computed(() => gewaehlt.value.length >= MINDESTENS && aufgeschrieben.value && !arbeitet.value);

function bytes(n) {
    const einheiten = ['B', 'KB', 'MB', 'GB'];
    let i = 0;
    while (n >= 1024 && i < einheiten.length - 1) { n /= 1024; i++; }
    return `${n.toFixed(i ? 1 : 0)} ${einheiten[i]}`;
}

function zuruecksetzen() {
    schritt.value = '';
    fehler.value = '';
    eigene.value = false;
    eigeneText.value = '';
    aufgeschrieben.value = false;
    passphrase.value = '';
    quelle.value = '';
    kopf.value = null;
}

/* ── Anlegen ── */

async function neuerVorschlag() {
    vorschlag.value = await invoke('sicherung_vorschlag');
}

async function anlegenBeginnen() {
    zuruecksetzen();
    await neuerVorschlag();
    schritt.value = 'anlegen';
}

let groesse = 0;
async function anlegen() {
    fehler.value = '';
    arbeitet.value = true;
    try {
        const a = await invoke('sicherung_anlegen', { passphrase: gewaehlt.value });
        groesse = a.groesse;
        if (android) {
            // Weiter geht es im Ereignis „abgelegt".
            if (!window.openanyAblage.sicherungAblegen(a.pfad, a.name)) throw new Error(t('app.sicherung.nichtAbgelegt'));
            return;
        }
        // Schreibtisch: Speichern-Dialog. Abgebrochen (`null`): Die
        // Passphrase steht noch -- einfach noch einmal.
        const ort = await invoke('sicherung_ablegen', { pfad: a.pfad });
        arbeitet.value = false;
        if (!ort) return;
        toast.success(t('app.sicherung.gesichert', { ort, groesse: bytes(groesse) }));
        zuruecksetzen();
    } catch (e) {
        fehler.value = String(e?.message ?? e);
        arbeitet.value = false;
    }
}

/* ── Einspielen ── */

async function waehlen() {
    zuruecksetzen();
    if (android) {
        // Weiter geht es im Ereignis „gewaehlt".
        window.openanyAblage.sicherungWaehlen();
        return;
    }
    try {
        const pfad = await invoke('sicherung_waehlen');
        if (!pfad) return;
        quelle.value = pfad;
        schritt.value = 'oeffnen';
    } catch (e) {
        toast.error(String(e));
    }
}

async function oeffnen() {
    fehler.value = '';
    arbeitet.value = true;
    try {
        kopf.value = await invoke('sicherung_oeffnen', { pfad: quelle.value, passphrase: passphrase.value });
        passphrase.value = '';
        schritt.value = 'bestaetigen';
    } catch (e) {
        fehler.value = String(e?.message ?? e);
    } finally {
        arbeitet.value = false;
    }
}

async function verwerfen() {
    try { await invoke('sicherung_verwerfen'); } catch { /* nichts zu tun */ }
    zuruecksetzen();
}

async function einspielen() {
    arbeitet.value = true;
    try {
        await invoke('sicherung_einspielen');
        // Auf dem Schreibtisch startet die Schale selbst neu.
        window.openanyAblage?.neuStarten?.();
    } catch (e) {
        fehler.value = String(e?.message ?? e);
        arbeitet.value = false;
    }
}

const erstellt = computed(() => kopf.value
    ? new Date(kopf.value.erstellt).toLocaleString(locale.value, { dateStyle: 'medium', timeStyle: 'short' })
    : '');

/* ── Antworten von Android ── */

function antwort(ev) {
    const d = ev.detail ?? {};
    if (d.art === 'abgelegt') {
        arbeitet.value = false;
        if (d.ok) { toast.success(t('app.sicherung.gesichertOhneOrt', { groesse: bytes(groesse) })); zuruecksetzen(); }
        else if (d.fehler) fehler.value = d.fehler;
        // Abgebrochen: Die Datei ist fort (MainActivity löscht sie), die
        // Passphrase steht noch -- einfach noch einmal.
    } else if (d.art === 'gewaehlt') {
        if (d.pfad) { quelle.value = d.pfad; schritt.value = 'oeffnen'; }
        else if (d.fehler) toast.error(d.fehler);
    }
}

onMounted(async () => {
    window.addEventListener('openany-sicherung', antwort);
    try { regel.value = (await invoke('speicher_lage')).regel; } catch { /* nur der Hinweis fehlt */ }
});
onBeforeUnmount(() => window.removeEventListener('openany-sicherung', antwort));

const feld = 'mt-1 w-full px-3 py-2 rounded-xl border border-linie bg-vertieft text-schrift text-sm';
</script>

<template>
  <div class="karte p-6 shadow-sm space-y-4">
    <h3 class="text-lg font-extrabold text-schrift flex items-center gap-2">
      <Archive class="w-5 h-5 text-marke" /> {{ t('app.sicherung.titel') }}
    </h3>

    <template v-if="schritt === ''">
      <p class="text-sm text-fliess">{{ t('app.sicherung.hinweis') }}</p>
      <p v-if="regel && regel !== 'alles'" class="text-xs text-leise">{{ t('app.sicherung.beiBedarf') }}</p>
      <div class="flex flex-wrap gap-2">
        <BaseButton @click="anlegenBeginnen"><Download class="w-4 h-4" /> {{ t('app.sicherung.anlegen') }}</BaseButton>
        <BaseButton variant="secondary" @click="waehlen"><Upload class="w-4 h-4" /> {{ t('app.sicherung.einspielen') }}</BaseButton>
      </div>
    </template>

    <form v-else-if="schritt === 'anlegen'" class="space-y-3" @submit.prevent="anlegen">
      <p class="text-sm text-fliess">{{ t('app.sicherung.passphraseHinweis') }}</p>
      <template v-if="!eigene">
        <p class="font-mono text-base font-bold text-schrift bg-vertieft rounded-xl px-3 py-2 break-words select-all">{{ vorschlag }}</p>
        <div class="flex flex-wrap gap-x-4 gap-y-2 text-sm font-bold">
          <button type="button" class="text-marke flex items-center gap-1 cursor-pointer" :disabled="arbeitet" @click="neuerVorschlag">
            <RefreshCw class="w-4 h-4" /> {{ t('app.sicherung.andererVorschlag') }}
          </button>
          <button type="button" class="text-marke cursor-pointer" :disabled="arbeitet" @click="eigene = true">{{ t('app.sicherung.eigeneWaehlen') }}</button>
        </div>
      </template>
      <template v-else>
        <label class="block">
          <span class="block text-sm font-bold text-fliess">{{ t('app.sicherung.eigene', { anzahl: MINDESTENS }) }}</span>
          <input v-model="eigeneText" type="text" autocomplete="off" autocapitalize="none" spellcheck="false" :class="feld" />
        </label>
        <button type="button" class="text-sm font-bold text-marke cursor-pointer" :disabled="arbeitet" @click="eigene = false">{{ t('app.sicherung.dochVorschlag') }}</button>
      </template>
      <label class="flex items-start gap-2 text-sm text-fliess">
        <input v-model="aufgeschrieben" type="checkbox" class="mt-1" />
        <span>{{ t('app.sicherung.aufgeschrieben') }}</span>
      </label>
      <div class="flex flex-wrap gap-2">
        <BaseButton type="submit" :disabled="!kannAnlegen">
          <Loader2 v-if="arbeitet" class="w-4 h-4 animate-spin" /><Download v-else class="w-4 h-4" /> {{ t('app.sicherung.anlegenKnopf') }}
        </BaseButton>
        <BaseButton variant="secondary" :disabled="arbeitet" @click="zuruecksetzen">{{ t('common.cancel') }}</BaseButton>
      </div>
    </form>

    <form v-else-if="schritt === 'oeffnen'" class="space-y-3" @submit.prevent="oeffnen">
      <p v-if="!android" class="text-sm text-fliess">
        <span class="font-bold">{{ t('app.sicherung.datei') }}:</span> <span class="break-all">{{ quelle.split(/[\\/]/).pop() }}</span>
      </p>
      <label class="block">
        <span class="block text-sm font-bold text-fliess">{{ t('app.sicherung.passphrase') }}</span>
        <input v-model="passphrase" type="password" autocomplete="off" :class="feld" />
      </label>
      <div class="flex flex-wrap gap-2">
        <BaseButton type="submit" :disabled="arbeitet || !quelle || !passphrase">
          <Loader2 v-if="arbeitet" class="w-4 h-4 animate-spin" /><Upload v-else class="w-4 h-4" /> {{ t('app.sicherung.oeffnen') }}
        </BaseButton>
        <BaseButton variant="secondary" :disabled="arbeitet" @click="verwerfen">{{ t('common.cancel') }}</BaseButton>
      </div>
    </form>

    <div v-else-if="schritt === 'bestaetigen'" class="space-y-3">
      <p class="text-sm text-schrift font-bold">{{ t('app.sicherung.vom', { datum: erstellt, geraet: kopf.geraet }) }}</p>
      <p class="text-sm text-warnung flex items-start gap-2">
        <AlertTriangle class="w-4 h-4 shrink-0 mt-0.5" /><span>{{ t('app.sicherung.ersetzt') }}</span>
      </p>
      <div class="flex flex-wrap gap-2">
        <BaseButton variant="danger" :disabled="arbeitet" @click="einspielen">
          <Loader2 v-if="arbeitet" class="w-4 h-4 animate-spin" /> {{ t('app.sicherung.ersetzenKnopf') }}
        </BaseButton>
        <BaseButton variant="secondary" :disabled="arbeitet" @click="verwerfen">{{ t('common.cancel') }}</BaseButton>
      </div>
    </div>

    <p v-if="fehler" class="text-sm text-warnung flex items-start gap-2">
      <AlertTriangle class="w-4 h-4 shrink-0 mt-0.5" /><span class="min-w-0 break-words">{{ fehler }}</span>
    </p>
  </div>
</template>
