<script setup>
/*
 * Die Kachel „Geräte in der Nähe" — finden, paaren und abgleichen im selben
 * WLAN oder Hotspot, ganz ohne Server. Dazu der Name dieses Geräts und der
 * eigene Name, unter dem man vor Ort eingeladen wird.
 */
import { ref, watch, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { Radar } from 'lucide-vue-next';
import { useNahAbgleich } from '../nahAbgleich';
import HilfeLink from '@oberflaeche/base/HilfeLink.vue';

const props = defineProps({ lage: Object });
const emit = defineEmits(['lage-geaendert']);
const { t } = useI18n();

const form = ref({ geraetename: '' });
const fehler = ref('');

/*
 * Geraete in der Naehe (APK 0.1): Wer ist im selben WLAN oder Hotspot?
 * Finden, paaren, abgleichen — ohne Server. Der Zustand gehört der Schale
 * (`nahAbgleich.js`); der Paarungsdialog steht dort auch, damit eine Anfrage
 * auf jeder Seite erscheint.
 */
const {
    nah, fehler: nahFehler, laeuft: nahLaeuft, berichte: nahBerichte,
    lesen: nahLesen, suchen: nahSuchen, paaren, vergessen, abgleichen: mitGeraetAbgleichen, offenMit,
} = useNahAbgleich();

/*
 * DU ALS PERSON (lokale Mitgliedschaften). Hinter
 * jedem Gerät steht eine Person; eigene, gepaarte Geräte sind dieselbe. Wer
 * dich später vor Ort in ein Projekt einlädt, lädt die Person ein -- dann
 * sind alle deine Geräte dabei. Der Name gilt für alle, sobald sie sich
 * abgleichen.
 */
/*
 * ANFRAGEN VON UNBEKANNTEN: Vorgabe aus. Dann kommt von
 * Geräten, deren Person hier weder bekannt noch per 6 Ziffern bestätigt ist,
 * nichts an. An: höchstens drei Nachrichten zu je 500 Zeichen, im
 * Anfragen-Ordner der Nachrichten (direktbefehle.rs).
 */
const anfragenErlaubt = ref(false);
async function anfragenLesen() {
    try { anfragenErlaubt.value = Boolean((await invoke('nah_anfragen')).erlaubt); } catch { /* bleibt aus */ }
}
async function anfragenSetzen(e) {
    const an = Boolean(e.target.checked);
    try {
        await invoke('nah_anfragen_erlauben', { an });
        anfragenErlaubt.value = an;
    } catch (err) {
        nahFehler.value = String(err);
        e.target.checked = anfragenErlaubt.value;
    }
}

const personenname = ref('');
let personennameBearbeitet = false;
watch(() => nah.value?.person?.name, (name) => {
    if (!personennameBearbeitet) personenname.value = name ?? '';
}, { immediate: true });
async function personennameSpeichern() {
    nahFehler.value = '';
    try {
        await invoke('nah_person_name', { name: personenname.value });
        personennameBearbeitet = false;
        await nahLesen();
    } catch (e) {
        nahFehler.value = String(e);
    }
}

async function nahAbgleichen(fingerabdruck) {
    nahFehler.value = '';
    try {
        await mitGeraetAbgleichen(fingerabdruck);
    } catch (e) {
        nahFehler.value = String(e);
    }
}
function zeitpunkt(iso) {
    if (!iso) return '';
    const d = new Date(iso);
    return Number.isNaN(d.getTime()) ? '' : d.toLocaleString([], { dateStyle: 'short', timeStyle: 'short' });
}

/** Den Namen dieses Geräts speichern (Abschnitt „Geräte in der Nähe"). */
async function speichern() {
    fehler.value = '';

    try {
        await invoke('einstellungen_speichern', { geraetename: form.value.geraetename });
        emit('lage-geaendert');
    } catch (e) {
        fehler.value = String(e);
    }
}

function ausLage() {
    if (props.lage) form.value = { geraetename: props.lage.geraetename };
}

onMounted(() => {
    ausLage();
    nahLesen();
    anfragenLesen();
});
</script>

<template>
  <div class="karte p-6 shadow-sm space-y-4 lg:col-span-2">
    <h3 class="text-lg font-extrabold text-schrift flex items-center gap-2">
      <Radar class="w-5 h-5 text-marke" /> {{ t('app.nah.titel') }}
      <HilfeLink ziel="settings-geraeteNah" />
    </h3>
    <p class="text-sm text-leise">{{ t('app.nah.hinweis') }}</p>
    <label class="flex items-start gap-3 text-sm text-fliess md:max-w-[460px]">
      <input type="checkbox" class="mt-0.5" :checked="anfragenErlaubt" @change="anfragenSetzen">
      <span>
        <span class="font-bold">{{ t('app.nah.anfragenErlauben') }}</span> — {{ t('app.nah.anfragenHinweis') }}
      </span>
    </label>
    <!-- Der Name gilt auch ohne Server, und hier sieht man ihn gleich in der
         Liste der anderen. Ein neuer Name gilt sofort, ohne Neustart. -->
    <label class="block md:max-w-[460px]">
      <span class="block text-xs font-bold text-leise mb-1">{{ t('app.nah.geraetename') }}</span>
      <span class="flex items-center gap-2">
        <input v-model="form.geraetename" class="feld" :placeholder="t('app.nah.geraetenamePlatzhalter')" autocorrect="off">
        <button class="knopf-wichtig shrink-0" @click="speichern">{{ t('common.save') }}</button>
      </span>
    </label>
    <p v-if="nah?.fingerabdruck" class="text-xs text-leise">
      {{ t('app.nah.fingerabdruck') }} <span class="font-mono">{{ nah.fingerabdruck }}</span>
    </p>

    <template v-if="nah?.person">
      <label class="block md:max-w-[460px] mt-2">
        <span class="block text-xs font-bold text-leise mb-1">{{ t('app.nah.deinName') }}</span>
        <span class="flex items-center gap-2">
          <input v-model="personenname" class="feld" :placeholder="t('app.nah.deinNamePlatzhalter')" autocorrect="off"
                 @input="personennameBearbeitet = true">
          <button class="knopf-wichtig shrink-0" @click="personennameSpeichern">{{ t('common.save') }}</button>
        </span>
      </label>
      <p class="text-xs text-leise">{{ t('app.nah.deinNameHinweis') }}</p>
      <p class="text-xs text-leise">
        {{ t('app.nah.deineGeraete') }} <span class="font-bold text-fliess">{{ nah.person.geraete.map((g) => g.name || '?').join(' · ') }}</span>
        · {{ t('app.nah.personenId') }} <span class="font-mono">{{ nah.person.personen_id }}</span>
      </p>
    </template>
    <p v-if="nah?.fehler" class="text-sm text-rose-600 dark:text-rose-400">{{ nah.fehler }}</p>
    <p v-else-if="nah && !nah.laeuft" class="text-sm text-leise">{{ t('app.nah.startet') }}</p>
    <p v-if="nahFehler" class="text-sm text-rose-600 dark:text-rose-400">{{ nahFehler }}</p>

    <ul v-if="nah?.geraete?.length" class="karte divide-y divide-linie">
      <li v-for="g in nah.geraete" :key="g.fingerabdruck" class="px-4 py-3 flex items-center gap-3">
        <span class="w-2 h-2 rounded-full shrink-0" :class="g.openany ? 'bg-marke' : 'bg-slate-400'"></span>
        <span class="min-w-0 flex-1">
          <span class="block font-bold text-schrift truncate">{{ g.name }}</span>
          <span class="block text-xs text-leise truncate">
            {{ g.openany ? 'openany' : (g.modell || 'LocalSend') }} · {{ g.adresse }} · <span class="font-mono">{{ g.fingerabdruck.slice(0, 12) }}</span>
          </span>
        </span>
        <span v-if="g.gepaart" class="text-xs font-bold text-marke shrink-0">{{ t('app.nah.gepaart') }}</span>
        <button v-else-if="g.openany && !offenMit(g.fingerabdruck)" class="knopf shrink-0" @click="paaren(g.fingerabdruck)">{{ t('app.nah.paaren') }}</button>
      </li>
    </ul>
    <p v-else-if="nah?.laeuft" class="text-sm text-leise">{{ t('app.nah.niemand') }}</p>
    <div class="flex items-center gap-2">
      <button class="knopf" :disabled="!nah?.laeuft" @click="nahSuchen">{{ t('app.nah.suchen') }}</button>
    </div>

    <template v-if="nah?.gepaarte?.length">
      <h3 class="mt-4 text-sm font-extrabold text-schrift">{{ t('app.nah.gepaarteTitel') }}</h3>
      <ul class="karte divide-y divide-linie md:max-w-[460px]">
        <li v-for="g in nah.gepaarte" :key="'gepaart-' + g.fingerabdruck" class="px-4 py-3">
          <div class="flex items-center gap-3">
            <span class="w-2 h-2 rounded-full shrink-0" :class="g.da ? 'bg-emerald-500' : 'bg-slate-400'"></span>
            <span class="min-w-0 flex-1">
              <span class="block font-bold text-schrift truncate">{{ g.name }}</span>
              <span class="block text-xs text-leise truncate">{{ g.da ? t('app.nah.inDerNaehe') : t('app.nah.nichtInDerNaehe') }} · <span class="font-mono">{{ g.fingerabdruck.slice(0, 12) }}</span></span>
              <span v-if="g.letzter_lauf" class="block text-xs text-leise truncate">{{ t('app.nah.zuletzt', { zeit: zeitpunkt(g.letzter_lauf) }) }}</span>
            </span>
          </div>
          <div class="flex flex-wrap items-center gap-2 mt-2 pl-5">
            <button class="knopf-wichtig" :disabled="!!nahLaeuft" @click="nahAbgleichen(g.fingerabdruck)">
              {{ nahLaeuft === g.fingerabdruck ? t('app.allgemein.laeuft') : t('app.allgemein.jetztAbgleichen') }}
            </button>
            <button class="knopf" :disabled="!!nahLaeuft" @click="vergessen(g.fingerabdruck)">{{ t('app.nah.vergessen') }}</button>
          </div>
          <div v-if="nahBerichte[g.fingerabdruck]" class="mt-2 ml-5 text-sm text-fliess">
            {{ t('app.nah.bericht', { geholt: nahBerichte[g.fingerabdruck].gezogen, geschickt: nahBerichte[g.fingerabdruck].geschoben }) }}<template v-if="nahBerichte[g.fingerabdruck].konflikte"> · {{ t('app.nah.berichtKonflikte', { count: nahBerichte[g.fingerabdruck].konflikte, name: g.name }, nahBerichte[g.fingerabdruck].konflikte) }}</template>
            <p v-for="(anzahl, grund) in nahBerichte[g.fingerabdruck].uebersprungen_weil ?? {}" :key="grund" class="mt-1 text-leise">
              {{ t('app.nah.uebersprungen', { grund, anzahl }) }}
            </p>
            <p v-for="f in nahBerichte[g.fingerabdruck].fehler" :key="f" class="mt-1 text-rose-600 dark:text-rose-400">{{ f }}</p>
          </div>
          <p v-else-if="g.letzter_fehler" class="mt-2 ml-5 text-sm text-rose-600 dark:text-rose-400">{{ g.letzter_fehler }}</p>
        </li>
      </ul>
    </template>
    <p v-if="fehler" class="text-sm text-rose-600 dark:text-rose-400">{{ fehler }}</p>
  </div>
</template>
