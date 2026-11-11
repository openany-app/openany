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
import HilfeLink from '@oberflaeche/base/HilfeLink.vue';

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
    try {
        lage.value = await invoke('speicher_lage');
        if (lage.value.verbunden) {
            resteLesen();
            serverLesen();
        }
    } catch (e) { toast.error(String(e)); }
}

async function regelSetzen(regel) {
    await invoke('speicher_regel_setzen', { regel });
    await lesen();
}

async function nichtMehrBehalten(id) {
    await invoke('behalten_setzen', { id, an: false });
    await lesen();
}

/* Wofür der Platz auf dem Server belegt ist. Ohne Netz bleibt der Abschnitt weg. */
const server = ref(null);
const serverOrte = ['nachrichten', 'notizen', 'kalender', 'kontakte', 'boards', 'galerie', 'dateien', 'dokumente'];
async function serverLesen() {
    server.value = await invoke('server_belegung').catch(() => null);
}

/* Was auf dem Server liegt und keinen Grund mehr dafür hat (aufserver.rs). */
const reste = ref(null);
async function resteLesen() {
    reste.value = await invoke('server_reste').catch(() => null);
}
async function resteNehmen() {
    const ok = await confirmDialog(t('app.speicher.resteFrage'), { title: t('app.speicher.resteTitel'), confirmLabel: t('app.speicher.resteKnopf') });
    if (!ok) return;
    arbeitet.value = true;
    try {
        const r = await invoke('server_reste_nehmen');
        toast.success(t('app.speicher.resteFertig', { entfernt: r.entfernt, groesse: bytes(r.bytes) })
            + (r.einzig_dort ? ` ${t('app.speicherAnsicht.einzigDort', { anzahl: r.einzig_dort })}` : ''));
        await resteLesen();
        await serverLesen();
    } catch (e) {
        toast.error(String(e));
    } finally {
        arbeitet.value = false;
    }
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
      <HilfeLink ziel="settings-speicherGeraet" />
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

      <div v-if="server" class="text-sm text-fliess">
        <p class="font-bold text-schrift">{{ t('app.speicher.serverTitel') }}</p>
        <p class="mt-1 text-xs text-leise">
          {{ server.kontingent === null
            ? t('app.speicher.serverBelegt', { belegt: bytes(server.kontingent_belegt) })
            : t('app.speicher.serverVon', { belegt: bytes(server.kontingent_belegt), grenze: bytes(server.kontingent) }) }}
        </p>
        <dl class="mt-2 space-y-0.5 md:max-w-[320px]">
          <div v-for="ort in serverOrte" :key="ort" class="flex justify-between gap-4">
            <dt>{{ t(`app.speicher.ort.${ort}`) }}</dt>
            <dd class="tabular-nums">{{ bytes(server[ort]) }}</dd>
          </div>
        </dl>
        <p class="mt-1.5 text-xs text-leise">{{ t('app.speicher.serverHinweis') }}</p>
      </div>

      <div v-if="reste?.anzahl" class="text-sm text-fliess">
        <p>{{ t('app.speicher.reste', { anzahl: reste.anzahl, groesse: bytes(reste.bytes) }) }}</p>
        <button class="knopf mt-2" :disabled="arbeitet" @click="resteNehmen">{{ t('app.speicher.resteKnopf') }}</button>
      </div>

      <button class="knopf" :disabled="arbeitet" @click="freigeben">{{ arbeitet ? t('app.speicher.pruefen') : t('app.speicher.freigeben') }}</button>
    </template>
  </div>
</template>
