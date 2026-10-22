<script setup>
/*
 * Das ALTE Gerät bei der Freischaltung (Z4, 21.10.2026,
 * docs/plan-zero-knowledge.md): „‚Telefon' möchte sich anmelden" -- die 6
 * Ziffern vom anderen Bildschirm eintippen, freischalten.
 *
 * Steht oben in der App (Tiffy: Hinweis oben + Kachel) und in der Kachel
 * „Meine Geräte" (`eingebettet`). Gefragt wird nur, solange das Fenster
 * sichtbar ist -- eine Anfrage gilt ohnehin nur zehn Minuten.
 *
 * Nach dem Freischalten kommt die Kiste mit dem Kontoschlüssel von selbst,
 * sobald sich das neue Gerät gemeldet hat (direktpost.rs,
 * `freigeschaltete_bedienen`). Damit das nicht erst beim nächsten Blick
 * passiert, fragt diese Ansicht danach eine Weile nach dem Stand.
 */
import { ref, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { KeyRound, Check, X, Loader2 } from 'lucide-vue-next';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import { useToast } from '@oberflaeche/composables/useToast';

defineProps({ eingebettet: { type: Boolean, default: false } });
const emit = defineEmits(['freigeschaltet']);
const { t } = useI18n();
const toast = useToast();

const anfragen = ref([]);
const ziffern = ref({});
const arbeitet = ref(null);
const fehler = ref({});
let takt = null;
let nachlauf = null;

async function laden() {
    try {
        anfragen.value = await invoke('freischaltanfragen');
    } catch {
        anfragen.value = [];
    }
}

async function freischalten(a) {
    fehler.value = { ...fehler.value, [a.anfrage]: '' };
    arbeitet.value = a.anfrage;
    try {
        await invoke('anfrage_freischalten', { anfrage: a.anfrage, ziffern: ziffern.value[a.anfrage] ?? '' });
        toast.success(t('app.freischalten.erledigt', { name: a.geraet }));
        emit('freigeschaltet');
        nachlaufen();
    } catch (e) {
        fehler.value = { ...fehler.value, [a.anfrage]: String(e) };
    } finally {
        arbeitet.value = null;
        await laden();
    }
}

async function ablehnen(a) {
    arbeitet.value = a.anfrage;
    try {
        await invoke('anfrage_ablehnen', { anfrage: a.anfrage });
    } catch (e) {
        toast.error(String(e));
    } finally {
        arbeitet.value = null;
        await laden();
    }
}

/*
 * Zwei Minuten lang alle fünf Sekunden auf den Schlüsselstand schauen: Dabei
 * legt die Schale die Kiste, sobald das neue Gerät im Verzeichnis steht.
 */
function nachlaufen() {
    clearInterval(nachlauf);
    let runden = 24;
    nachlauf = setInterval(async () => {
        runden -= 1;
        try { await invoke('nachrichtenschluessel_lage'); } catch { /* später */ }
        if (runden <= 0) { clearInterval(nachlauf); nachlauf = null; }
    }, 5000);
}

function taktSetzen() {
    clearInterval(takt);
    takt = document.visibilityState === 'visible' ? setInterval(laden, 15000) : null;
}

function beiSichtbarkeit() {
    taktSetzen();
    if (document.visibilityState === 'visible') laden();
}

onMounted(() => {
    laden();
    taktSetzen();
    document.addEventListener('visibilitychange', beiSichtbarkeit);
});

onUnmounted(() => {
    clearInterval(takt);
    document.removeEventListener('visibilitychange', beiSichtbarkeit);
});
</script>

<template>
  <div v-if="anfragen.length" :class="eingebettet ? 'space-y-3' : 'mb-4 karte p-4 space-y-3 border-marke'">
    <form v-for="a in anfragen" :key="a.anfrage" class="space-y-2" @submit.prevent="freischalten(a)">
      <p class="text-sm font-bold text-schrift flex items-center gap-2">
        <KeyRound class="w-4 h-4 text-marke shrink-0" />
        {{ a.art === 'schluessel' ? t('app.freischalten.anfrageSchluessel', { name: a.geraet }) : t('app.freischalten.anfrageAnmelden', { name: a.geraet }) }}
      </p>
      <p class="text-sm text-fliess">{{ t('app.freischalten.eintippen', { name: a.geraet }) }}</p>
      <div class="flex flex-wrap items-center gap-2">
        <input v-model="ziffern[a.anfrage]" type="text" inputmode="numeric" autocomplete="one-time-code" maxlength="7"
               placeholder="000 000" class="feld w-32 font-mono text-lg tracking-widest text-center">
        <BaseButton type="submit" groesse="klein" :disabled="arbeitet !== null">
          <Loader2 v-if="arbeitet === a.anfrage" class="w-4 h-4 animate-spin" /><Check v-else class="w-4 h-4" />
          {{ arbeitet === a.anfrage ? t('app.freischalten.laeuft') : t('app.freischalten.freischalten') }}
        </BaseButton>
        <BaseButton type="button" groesse="klein" variant="secondary" :disabled="arbeitet !== null" @click="ablehnen(a)">
          <X class="w-4 h-4" /> {{ t('app.freischalten.nein') }}
        </BaseButton>
      </div>
      <p v-if="fehler[a.anfrage]" class="text-sm text-rose-600 dark:text-rose-400">{{ fehler[a.anfrage] }}</p>
    </form>
  </div>
</template>
