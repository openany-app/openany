<script setup>
/*
 * Die Kachel „Speicher auf diesem Gerät" (Phase 3, Stufe B).
 *
 * WAS HIER EINGESTELLT WIRD, REIST NICHT MIT. Das Tablet mit wenig Platz
 * holt Inhalte erst beim Öffnen, der PC behält alles — beide sehen trotzdem
 * dieselben Ordner und Dateien.
 *
 * „Platz freigeben" löscht nur, was ein erreichbares gepaartes Gerät
 * nachweislich hat. Die letzte Kopie bleibt immer.
 */
import { ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { HardDrive } from 'lucide-vue-next';
import { useToast } from '@oberflaeche/composables/useToast';
import { useConfirm } from '@oberflaeche/composables/useConfirm';

const { t } = useI18n();
const toast = useToast();
const { confirmDialog } = useConfirm();
const lage = ref(null);
const arbeitet = ref(false);

function bytes(n) {
    const einheiten = ['B', 'KB', 'MB', 'GB', 'TB'];
    let i = 0;
    while (n >= 1024 && i < einheiten.length - 1) { n /= 1024; i++; }
    return `${n.toFixed(i ? 1 : 0)} ${einheiten[i]}`;
}

async function lesen() {
    try { lage.value = await invoke('speicher_lage'); } catch (e) { toast.error(String(e)); }
}

async function regelSetzen(regel) {
    await invoke('speicher_regel_setzen', { regel });
    await lesen();
}

async function nichtMehrBehalten(id) {
    await invoke('behalten_setzen', { id, an: false });
    await lesen();
}

async function freigeben() {
    const warnung = lage.value?.regel === 'alles' ? ` ${t('app.speicher.freigebenAlles')}` : '';
    const ok = await confirmDialog(
        `${t('app.speicher.freigebenFrage')}${warnung}`,
        { title: t('app.speicher.freigeben'), confirmLabel: t('app.speicher.freigebenKnopf') },
    );
    if (!ok) return;
    arbeitet.value = true;
    try {
        const r = await invoke('platz_freigeben');
        // Die Meldung nennt, was fort ist und was bleibt.
        toast.success(t('app.speicher.freigegeben', { entfernt: r.entfernt, groesse: bytes(r.bytes) })
            + (r.einzig_hier ? ` ${t('app.speicher.nurHier', { anzahl: r.einzig_hier })}` : ''));
    } catch (e) {
        toast.error(String(e));
    } finally {
        arbeitet.value = false;
        await lesen();
    }
}

onMounted(lesen);
</script>

<template>
  <div class="karte p-6 shadow-sm space-y-4">
    <h3 class="text-lg font-extrabold text-schrift flex items-center gap-2">
      <HardDrive class="w-5 h-5 text-marke" /> {{ t('app.speicher.titel') }}
    </h3>

    <template v-if="lage">
      <div>
        <div class="flex items-center justify-between text-xs font-bold text-leise mb-1.5">
          <span>{{ t('app.speicher.belegen', { groesse: bytes(lage.belegt) }) }}</span>
          <span>{{ t('app.speicher.freiVon', { frei: bytes(lage.frei), gesamt: bytes(lage.gesamt) }) }}</span>
        </div>
        <div class="h-2 rounded-full bg-auflage overflow-hidden">
          <div class="h-full rounded-full bg-marke" :style="{ width: Math.min(100, Math.round(((lage.gesamt - lage.frei) / Math.max(1, lage.gesamt)) * 100)) + '%' }"></div>
        </div>
        <p class="mt-1.5 text-xs text-leise">{{ t('app.speicher.reserve', { groesse: bytes(lage.reserve) }) }}</p>
      </div>

      <div class="space-y-2">
        <label class="flex items-start gap-3 cursor-pointer">
          <input type="radio" name="inhalte-regel" class="mt-1" :checked="lage.regel === 'bei_bedarf'" @change="regelSetzen('bei_bedarf')">
          <span>
            <span class="block text-sm font-bold text-schrift">{{ t('app.speicher.beiBedarf') }}</span>
            <span class="block text-xs text-leise">{{ t('app.speicher.beiBedarfHinweis') }}</span>
          </span>
        </label>
        <label class="flex items-start gap-3 cursor-pointer">
          <input type="radio" name="inhalte-regel" class="mt-1" :checked="lage.regel === 'ausgewaehlt'" @change="regelSetzen('ausgewaehlt')">
          <span>
            <span class="block text-sm font-bold text-schrift">{{ t('app.speicher.ausgewaehlt') }}</span>
            <span class="block text-xs text-leise">{{ t('app.speicher.ausgewaehltHinweis') }}</span>
          </span>
        </label>
        <ul v-if="lage.regel === 'ausgewaehlt'" class="ml-7 space-y-1">
          <li v-if="!lage.behalten.length" class="text-xs text-leise">{{ t('app.speicher.nichtsMarkiert') }}</li>
          <li v-for="b in lage.behalten" :key="b.id" class="flex items-center gap-2 text-sm text-fliess">
            <span class="flex-1 min-w-0 truncate">{{ t(b.art === 'album' ? 'app.speicher.album' : 'app.speicher.ordner', { name: b.name }) }}</span>
            <button class="knopf shrink-0" @click="nichtMehrBehalten(b.id)">{{ t('app.allgemein.entfernen') }}</button>
          </li>
        </ul>
        <label class="flex items-start gap-3 cursor-pointer">
          <input type="radio" name="inhalte-regel" class="mt-1" :checked="lage.regel === 'alles'" @change="regelSetzen('alles')">
          <span>
            <span class="block text-sm font-bold text-schrift">{{ t('app.speicher.alles') }}</span>
            <span class="block text-xs text-leise">{{ t('app.speicher.allesHinweis') }}</span>
          </span>
        </label>
      </div>

      <p v-if="lage.fehlend" class="text-sm text-fliess">
        {{ t('app.speicher.fehlend', { count: lage.fehlend, groesse: bytes(lage.fehlend_bytes) }, lage.fehlend) }}
      </p>

      <button class="knopf" :disabled="arbeitet" @click="freigeben">{{ arbeitet ? t('app.speicher.pruefen') : t('app.speicher.freigeben') }}</button>
    </template>
  </div>
</template>
