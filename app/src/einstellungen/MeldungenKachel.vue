<script setup>
/*
 * Meldungen ohne Konto, für Verwalter (docs/plan-registrierung-app.md, A5).
 *
 * Die Kachel erscheint nur, wenn der Server diesem Konto Meldungen zeigt.
 * Entschlüsselt wird mit dem Konto-Schlüssel dieses Geräts
 * (meldungsbefehle.rs); Antworten gehen an die meldende Person und an alle
 * Verwalter. Den Stand sieht die meldende Person auf openany.de/melden.
 */
import { ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { Flag } from 'lucide-vue-next';
import { useToast } from '@oberflaeche/composables/useToast';

const { t, locale } = useI18n();
const toast = useToast();
const meldungen = ref(null);
const offen = ref(null);
const antwort = ref('');
const arbeitet = ref(false);
const STAENDE = ['eingegangen', 'in_bearbeitung', 'beantwortet', 'abgeschlossen'];

async function lesen() {
    meldungen.value = await invoke('meldungen_lesen').catch(() => null);
}

function ersetzen(neu) {
    meldungen.value = meldungen.value.map((m) => (m.kennung === neu.kennung ? neu : m));
}

async function antworten(m) {
    if (!antwort.value.trim()) return;
    arbeitet.value = true;
    try {
        ersetzen(await invoke('meldung_antworten', { kennung: m.kennung, antwort: antwort.value }));
        antwort.value = '';
    } catch (e) {
        toast.error(String(e));
    } finally {
        arbeitet.value = false;
    }
}

async function standSetzen(m, stand) {
    try {
        ersetzen(await invoke('meldung_stand_setzen', { kennung: m.kennung, stand }));
    } catch (e) {
        toast.error(String(e));
    }
}

function datum(iso) {
    return iso ? new Date(iso).toLocaleString(locale.value, { dateStyle: 'medium', timeStyle: 'short' }) : '';
}

onMounted(lesen);
</script>

<template>
  <div v-if="meldungen" class="karte p-6 shadow-sm space-y-4 lg:col-span-2">
    <h3 class="text-lg font-extrabold text-schrift flex items-center gap-2">
      <Flag class="w-5 h-5 text-marke" /> {{ t('app.meldungen.titel') }}
    </h3>
    <p v-if="!meldungen.length" class="text-sm text-leise">{{ t('app.meldungen.keine') }}</p>
    <ul class="space-y-2">
      <li v-for="m in meldungen" :key="m.kennung" class="rounded-xl border border-linie">
        <button type="button" class="w-full flex flex-wrap items-center justify-between gap-2 p-3 text-left" @click="offen = offen === m.kennung ? null : m.kennung">
          <span class="text-sm font-bold text-schrift">{{ m.nachrichten[0]?.inhalt?.art ? t(`melden.arten.${m.nachrichten[0].inhalt.art}`) : t('app.meldungen.unlesbar') }}</span>
          <span class="text-xs text-leise">{{ t(`melden.stand.${m.stand}`) }} · {{ datum(m.letzte_nachricht_at) }}</span>
        </button>
        <div v-if="offen === m.kennung" class="px-3 pb-3 space-y-3">
          <div v-for="(n, i) in m.nachrichten" :key="i" class="rounded-lg p-2" :class="n.von === 'verwaltung' ? 'bg-vertieft' : ''">
            <p class="text-xs font-bold text-leise">{{ t(`app.meldungen.von.${n.von}`) }} · {{ datum(n.at) }}</p>
            <template v-if="n.inhalt">
              <p v-if="n.inhalt.wo" class="text-sm text-fliess"><span class="font-bold">{{ n.inhalt.art === 'widerspruch' ? t('melden.woWiderspruch') : t('melden.wo') }}</span> {{ n.inhalt.wo }}</p>
              <p class="text-sm text-schrift whitespace-pre-wrap">{{ n.inhalt.text }}</p>
              <p v-if="n.inhalt.name || n.inhalt.kontakt" class="text-xs text-leise">{{ [n.inhalt.name, n.inhalt.kontakt].filter(Boolean).join(' · ') }}</p>
              <p v-if="n.inhalt.sprache" class="text-xs text-leise">{{ t('app.meldungen.sprache', { sprache: n.inhalt.sprache }) }}</p>
            </template>
            <p v-else class="text-sm text-leise">{{ t('app.meldungen.unlesbar') }}</p>
          </div>
          <label class="block text-sm font-bold text-schrift">{{ t('app.meldungen.stand') }}
            <select class="feld mt-1" :value="m.stand" @change="standSetzen(m, $event.target.value)">
              <option v-for="s in STAENDE" :key="s" :value="s">{{ t(`melden.stand.${s}`) }}</option>
            </select>
          </label>
          <label class="block text-sm font-bold text-schrift">{{ t('app.meldungen.antwort') }}
            <textarea v-model="antwort" rows="3" class="feld w-full mt-1"></textarea>
          </label>
          <p class="text-xs text-leise">{{ t('app.meldungen.antwortHinweis') }}</p>
          <button class="knopf-wichtig" :disabled="arbeitet || !antwort.trim()" @click="antworten(m)">{{ t('app.meldungen.senden') }}</button>
        </div>
      </li>
    </ul>
  </div>
</template>
