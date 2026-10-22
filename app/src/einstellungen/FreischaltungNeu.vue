<script setup>
/*
 * Das NEUE Gerät bei der Freischaltung durch ein anderes (Z4, 21.10.2026,
 * docs/plan-zero-knowledge.md).
 *
 * Zwei Fälle mit demselben Kasten:
 * - noch nicht verbunden (Kachel „Abgleich"): Anmeldename eintippen, dann
 *   stehen 6 Ziffern da; ein anderes Gerät tippt sie ein und meldet dieses
 *   bei anyid an. Dahinter der alte Weg fürs erste Gerät: Passwort im Browser.
 * - verbunden, aber ohne Schlüssel (`nur-schluessel`, Kachel „Meine Geräte"):
 *   gleich die Ziffern.
 *
 * Die Ziffern gehen nie an den Server -- sie sind das Passwort des PAKE
 * (freischalten.rs).
 */
import { ref, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { Smartphone } from 'lucide-vue-next';

const props = defineProps({ nurSchluessel: { type: Boolean, default: false } });
const emit = defineEmits(['lage-geaendert', 'passwort']);
const { t } = useI18n();

const schritt = ref('start'); // start | name | ziffern
const name = ref('');
const anzeige = ref(null);
const meldung = ref('');
const fehler = ref('');
const arbeitet = ref(false);
let takt = null;

async function beginnen() {
    fehler.value = '';
    meldung.value = '';
    arbeitet.value = true;
    try {
        anzeige.value = await invoke('freischaltung_beginnen', { name: props.nurSchluessel ? null : name.value });
        schritt.value = 'ziffern';
        taktSetzen(anzeige.value.intervall);
        if (!props.nurSchluessel) emit('lage-geaendert');
    } catch (e) {
        fehler.value = String(e);
    } finally {
        arbeitet.value = false;
    }
}

function taktSetzen(sekunden) {
    clearInterval(takt);
    takt = setInterval(nachfragen, Math.max(2, sekunden) * 1000);
}

async function nachfragen() {
    try {
        const stand = await invoke('freischaltung_schritt');
        if (stand.stand === 'wartet') return;

        aufhoeren();
        if (stand.stand === 'gekoppelt') meldung.value = t('app.abgleich.gekoppelt', { name: stand.name });
        else if (stand.stand === 'gesendet') meldung.value = t('app.freischalten.gesendet');
        else if (stand.stand === 'stimmt_nicht') meldung.value = t('app.freischalten.stimmtNicht');
        else meldung.value = t('app.freischalten.abgelaufen');
        emit('lage-geaendert');
    } catch (e) {
        aufhoeren();
        fehler.value = String(e);
    }
}

function aufhoeren() {
    clearInterval(takt);
    takt = null;
    anzeige.value = null;
    schritt.value = 'start';
}

async function abbrechen() {
    await invoke('freischaltung_abbrechen').catch(() => {});
    aufhoeren();
}

/*
 * Wie bei der Kopplung im Browser: Android hält die Webansicht an oder baut
 * sie neu auf, während der Mensch zum anderen Gerät greift. Deshalb an der
 * Sichtbarkeit wieder aufnehmen und sofort einmal fragen.
 */
async function aufnehmen() {
    try {
        const laufend = await invoke('freischaltung_laeuft');
        if (!laufend || laufend.nur_schluessel !== props.nurSchluessel) return;
        anzeige.value = laufend;
        schritt.value = 'ziffern';
        taktSetzen(laufend.intervall);
        nachfragen();
    } catch { /* nichts offen */ }
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
  <div class="space-y-2">
    <div v-if="schritt === 'ziffern' && anzeige" class="mt-2 md:max-w-[460px] text-center bg-flaeche border border-linie p-4 rounded-xl shadow-sm">
      <div class="font-mono text-4xl font-extrabold tracking-[0.12em] text-marke mb-1.5">{{ anzeige.ziffern }}</div>
      <p class="text-sm text-fliess">{{ t('app.freischalten.zifferHinweis') }}</p>
      <div class="flex items-center justify-center gap-2 mt-3">
        <button @click="abbrechen" class="knopf">{{ t('common.cancel') }}</button>
      </div>
    </div>

    <template v-else-if="nurSchluessel">
      <button class="knopf-wichtig" :disabled="arbeitet" @click="beginnen">{{ t('app.freischalten.zifferAnzeigen') }}</button>
    </template>

    <form v-else-if="schritt === 'name'" class="flex flex-wrap items-end gap-2 mt-2" @submit.prevent="beginnen">
      <label class="flex-1 min-w-[12rem] text-sm text-fliess">
        {{ t('app.freischalten.nameFrage') }}
        <input v-model="name" type="text" autocomplete="username" autocapitalize="off" class="feld w-full mt-1" required>
      </label>
      <button type="submit" class="knopf-wichtig" :disabled="arbeitet || !name.trim()">{{ t('app.freischalten.weiter') }}</button>
      <button type="button" class="knopf" @click="schritt = 'start'">{{ t('common.cancel') }}</button>
    </form>

    <template v-else>
      <div class="flex flex-wrap items-center gap-2 mt-2">
        <button class="knopf-wichtig flex items-center gap-2" @click="schritt = 'name'">
          <Smartphone class="w-4 h-4" /> {{ t('app.freischalten.anderesGeraet') }}
        </button>
      </div>
      <button class="text-sm text-leise underline" @click="emit('passwort')">{{ t('app.freischalten.erstesGeraet') }}</button>
    </template>

    <p v-if="meldung" class="text-sm text-fliess">{{ meldung }}</p>
    <p v-if="fehler" class="text-sm text-rose-600 dark:text-rose-400">{{ fehler }}</p>
  </div>
</template>
