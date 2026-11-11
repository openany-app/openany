<script setup>
/*
 * Kalender im Kalender des Telefons (docs/plan-kalender-kontakte-im-system.md,
 * K1, Entscheidung 4): ein Schalter, wie DAVx5. Die Termine erscheinen unter
 * dem Konto „openany“ in jeder Kalender-App -- entschlüsselt, nur auf diesem
 * Gerät. Abos nur auf Wunsch (Entscheidung 2).
 *
 * Unter Android: Die Rechte erfragt die Schale (`openanySystemkalender`,
 * MainActivity.kt), eingetragen wird von Rust aus (systemkalender.rs).
 *
 * Am Desktop (K5): ein CalDAV-Dienst auf 127.0.0.1 für Thunderbird, Apple
 * Kalender & Co. (caldav.rs) -- hier Schalter und Zugangsdaten.
 */
import { ref, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { CalendarDays } from 'lucide-vue-next';
import { useToast } from '@oberflaeche/composables/useToast';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import HilfeLink from '@oberflaeche/base/HilfeLink.vue';

const { t } = useI18n();
const toast = useToast();
const { confirmDialog } = useConfirm();
const bruecke = typeof window !== 'undefined' ? window.openanySystemkalender : null;

const an = ref(false);
const abos = ref(false);
const arbeitet = ref(false);
let warten = null;

async function laden() {
    const s = await invoke('systemkalender_stand');
    an.value = s.an;
    abos.value = s.abos;
}

async function setzen(neuAn, neuAbos) {
    arbeitet.value = true;
    try {
        await invoke('systemkalender_setzen', { an: neuAn, abos: neuAbos });
        an.value = neuAn;
        abos.value = neuAbos;
        if (neuAn) toast.success(t('app.systemkalender.eingeschaltet'));
    } catch (e) {
        toast.error(typeof e === 'string' ? e : t('app.systemkalender.fehler'));
        await laden();
    } finally {
        arbeitet.value = false;
    }
}

function umschalten(ereignis) {
    const neu = ereignis.target.checked;
    if (neu && !bruecke.erlaubt()) {
        // Erst die Rechte; weiter, wenn die Antwort da ist.
        ereignis.target.checked = false;
        warten = () => setzen(true, abos.value);
        bruecke.erlauben();
        return;
    }
    setzen(neu, abos.value);
}

function erlaubnis(e) {
    if (!warten) return;
    const weiter = warten;
    warten = null;
    if (e.detail) weiter();
    else toast.error(t('app.systemkalender.keinRecht'));
}

// Am Desktop: der Dienst.
const dienst = ref(null);

async function dienstLaden() {
    dienst.value = await invoke('kalenderdienst_stand');
}

async function dienstSetzen(ereignis) {
    arbeitet.value = true;
    try {
        await invoke('kalenderdienst_setzen', { an: ereignis.target.checked });
    } catch (e) {
        toast.error(typeof e === 'string' ? e : t('app.systemkalender.fehler'));
    } finally {
        arbeitet.value = false;
        await dienstLaden();
    }
}

async function neuesKennwort() {
    if (!(await confirmDialog(t('app.systemkalender.neuesKennwortFrage'), { confirmLabel: t('app.systemkalender.neuesKennwort') }))) return;
    await invoke('kalenderdienst_neues_kennwort');
    await dienstLaden();
}

async function kopieren(text) {
    try {
        await navigator.clipboard.writeText(text);
        toast.success(t('app.systemkalender.kopiert'));
    } catch {
        /* ohne Zwischenablage bleibt der Text zum Abschreiben */
    }
}

onMounted(() => {
    if (!bruecke) {
        dienstLaden();
        return;
    }
    window.addEventListener('openany-systemkalender', erlaubnis);
    laden();
});
onUnmounted(() => window.removeEventListener('openany-systemkalender', erlaubnis));
</script>

<template>
  <div v-if="bruecke" class="karte p-6 shadow-sm space-y-4">
    <h3 class="text-lg font-extrabold text-schrift flex items-center gap-2">
      <CalendarDays class="w-5 h-5 text-marke" /> {{ t('app.systemkalender.titel') }}
      <HilfeLink ziel="calendar-imTelefon" />
    </h3>
    <label class="flex items-center gap-3 text-sm">
      <input type="checkbox" :checked="an" :disabled="arbeitet" @change="umschalten" />
      <span class="font-semibold text-schrift">{{ t('app.systemkalender.zeigen') }}</span>
    </label>
    <label v-if="an" class="flex items-center gap-3 text-sm ml-7">
      <input type="checkbox" :checked="abos" :disabled="arbeitet" @change="(e) => setzen(true, e.target.checked)" />
      <span class="text-schrift">{{ t('app.systemkalender.abos') }}</span>
    </label>
    <p class="text-sm text-leise">{{ t('app.systemkalender.hinweis') }}</p>
    <p v-if="an" class="text-sm text-fliess">{{ t('app.systemkalender.google') }}</p>
    <p v-if="an" class="text-xs text-amber-600 dark:text-amber-500">{{ t('app.systemkalender.zurueck') }}</p>
    <p class="text-xs text-leise">{{ t('app.systemkalender.verschluesselt') }}</p>
  </div>
  <div v-else-if="dienst?.moeglich" class="karte p-6 shadow-sm space-y-4">
    <h3 class="text-lg font-extrabold text-schrift flex items-center gap-2">
      <CalendarDays class="w-5 h-5 text-marke" /> {{ t('app.systemkalender.titel') }}
      <HilfeLink ziel="calendar-andereProgramme" />
    </h3>
    <label class="flex items-center gap-3 text-sm">
      <input type="checkbox" :checked="dienst.an" :disabled="arbeitet" @change="dienstSetzen" />
      <span class="font-semibold text-schrift">{{ t('app.systemkalender.anbieten') }}</span>
    </label>
    <p class="text-sm text-leise">{{ t('app.systemkalender.dienstHinweis') }}</p>
    <template v-if="dienst.an">
      <p v-if="!dienst.laeuft" class="text-sm text-amber-600 dark:text-amber-500">{{ t('app.systemkalender.laeuftNicht') }}</p>
      <dl class="text-sm space-y-2">
        <div v-for="[schluessel, wert] in [['adresse', dienst.adresse], ['benutzer', dienst.benutzer], ['kennwort', dienst.kennwort]]" :key="schluessel" class="flex items-center gap-3">
          <dt class="w-24 shrink-0 text-leise">{{ t('app.systemkalender.' + schluessel) }}</dt>
          <dd class="flex-1 min-w-0 font-mono text-schrift truncate select-all">{{ wert }}</dd>
          <button type="button" class="text-marke text-xs font-semibold" @click="kopieren(wert)">{{ t('app.systemkalender.kopieren') }}</button>
        </div>
      </dl>
      <button type="button" class="text-xs font-semibold text-leise hover:text-schrift" :disabled="arbeitet" @click="neuesKennwort">
        {{ t('app.systemkalender.neuesKennwort') }}
      </button>
      <p class="text-sm text-fliess">{{ t('app.systemkalender.thunderbird') }}</p>
      <p class="text-xs text-amber-600 dark:text-amber-500">{{ t('app.systemkalender.dienstZurueck') }}</p>
    </template>
    <p class="text-xs text-leise">{{ t('app.systemkalender.verschluesselt') }}</p>
  </div>
</template>
