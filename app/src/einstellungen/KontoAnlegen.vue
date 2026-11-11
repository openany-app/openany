<script setup>
/*
 * Ein Konto in der App anlegen (docs/plan-registrierung-app.md, A4).
 *
 * Name → Rätsel (das Gerät rechnet, mit Fortschritt, abbrechbar) →
 * Nutzungsbedingungen → Sicherung anbieten. Das Rätsel läuft in Rust
 * (kontoanlage.rs); die Oberfläche fragt nur nach dem Stand. Android baut
 * die Webansicht beim Zurückkommen neu auf -- deshalb beim Laden und bei
 * Sichtbarkeit nachsehen, ob noch gerechnet wird.
 */
import { ref, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import Nutzungsbedingungen from './Nutzungsbedingungen.vue';

const emit = defineEmits(['lage-geaendert', 'abbrechen', 'fertig']);
const { t, te } = useI18n();

const schritt = ref('name'); // name | rechnet | agb | sicherung
const name = ref('');
const anteil = ref(0);
const kontoname = ref('');
const fehler = ref('');
const arbeitet = ref(false);
let takt = null;

function fehlerText(e) {
    const wort = String(e);
    return te(`app.kontoAnlegen.fehler.${wort}`) ? t(`app.kontoAnlegen.fehler.${wort}`) : wort;
}

async function anlegen() {
    fehler.value = '';
    arbeitet.value = true;
    try {
        await invoke('konto_anlegen_beginnen', { name: name.value });
        schritt.value = 'rechnet';
        anteil.value = 0;
        takt = setInterval(nachfragen, 500);
    } catch (e) {
        fehler.value = fehlerText(e);
    } finally {
        arbeitet.value = false;
    }
}

async function nachfragen() {
    const stand = await invoke('konto_anlegen_stand').catch(() => null);
    if (!stand) return;
    anteil.value = stand.anteil;
    if (stand.laeuft) return;
    clearInterval(takt);
    takt = null;
    if (stand.fertig) {
        kontoname.value = stand.fertig;
        schritt.value = 'agb';
        emit('lage-geaendert');
    } else {
        schritt.value = 'name';
        if (stand.fehler) fehler.value = fehlerText(stand.fehler);
    }
}

async function abbrechen() {
    await invoke('konto_anlegen_abbrechen').catch(() => {});
    clearInterval(takt);
    takt = null;
    schritt.value = 'name';
}

function zurSicherung() {
    document.getElementById('sicherung-kachel')?.scrollIntoView({ behavior: 'smooth', block: 'start' });
    emit('fertig');
}

async function aufnehmen() {
    const stand = await invoke('konto_anlegen_stand').catch(() => null);
    if (stand?.laeuft || stand?.fertig || stand?.fehler) {
        schritt.value = 'rechnet';
        clearInterval(takt);
        takt = setInterval(nachfragen, 500);
        nachfragen();
    }
}

function beiSichtbarkeit() {
    if (document.visibilityState === 'visible') aufnehmen();
}

onMounted(() => {
    aufnehmen();
    document.addEventListener('visibilitychange', beiSichtbarkeit);
});

onUnmounted(() => {
    clearInterval(takt);
    document.removeEventListener('visibilitychange', beiSichtbarkeit);
});
</script>

<template>
  <div class="space-y-3 md:max-w-[520px]">
    <form v-if="schritt === 'name'" class="space-y-2" @submit.prevent="anlegen">
      <label class="block text-sm text-fliess">
        {{ t('app.kontoAnlegen.nameFrage') }}
        <input v-model="name" type="text" autocomplete="username" autocapitalize="off" maxlength="32" class="feld w-full mt-1" required>
      </label>
      <p class="text-xs text-leise">{{ t('app.kontoAnlegen.nameHinweis') }}</p>
      <div class="flex flex-wrap gap-2">
        <button type="submit" class="knopf-wichtig" :disabled="arbeitet || name.trim().length < 3">{{ t('app.kontoAnlegen.anlegen') }}</button>
        <button type="button" class="knopf" @click="emit('abbrechen')">{{ t('common.cancel') }}</button>
      </div>
    </form>

    <div v-else-if="schritt === 'rechnet'" class="space-y-2">
      <p class="text-sm text-fliess">{{ t('app.kontoAnlegen.rechnet') }}</p>
      <div class="h-2 rounded-full bg-auflage overflow-hidden">
        <div class="h-full rounded-full bg-marke transition-all" :style="{ width: Math.round(anteil * 100) + '%' }"></div>
      </div>
      <button class="knopf" @click="abbrechen">{{ t('common.cancel') }}</button>
    </div>

    <div v-else-if="schritt === 'agb'" class="space-y-2">
      <p class="text-sm text-schrift font-bold">{{ t('app.kontoAnlegen.fertig', { name: kontoname }) }}</p>
      <h4 class="text-sm font-extrabold text-schrift">{{ t('app.kontoAnlegen.agbTitel') }}</h4>
      <Nutzungsbedingungen @zugestimmt="schritt = 'sicherung'" />
    </div>

    <div v-else-if="schritt === 'sicherung'" class="space-y-2">
      <h4 class="text-sm font-extrabold text-schrift">{{ t('app.kontoAnlegen.sicherungTitel') }}</h4>
      <p class="text-sm text-fliess">{{ t('app.kontoAnlegen.sicherungHinweis') }}</p>
      <div class="flex flex-wrap gap-2">
        <button class="knopf-wichtig" @click="zurSicherung">{{ t('app.kontoAnlegen.sicherungKnopf') }}</button>
        <button class="knopf" @click="emit('fertig')">{{ t('app.kontoAnlegen.spaeter') }}</button>
      </div>
    </div>

    <p v-if="fehler" class="text-sm text-rose-600 dark:text-rose-400">{{ fehler }}</p>
  </div>
</template>
