<script setup>
/*
 * Neue Fotos automatisch übernehmen (docs/plan-viele-dateien.md, D5) -- auch
 * hier in den Einstellungen, nicht nur in der Mediathek. Nur unter Android.
 *
 * Ein Schalter: Ein öffnet denselben Dialog wie in der Mediathek, schon auf
 * „an“ -- die Gerätealben will gewählt sein. Aus wirkt sofort.
 */
import { ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { RefreshCw } from 'lucide-vue-next';
import { useToast } from '@oberflaeche/composables/useToast';
import NeueFotosAutomatisch from '@oberflaeche/galerie/NeueFotosAutomatisch.vue';
import { fotosVomGeraetQuelle } from '../quellen/uebernahme';
import HilfeLink from '@oberflaeche/base/HilfeLink.vue';

const { t } = useI18n();
const toast = useToast();
const geraet = fotosVomGeraetQuelle();
const quelle = { fotosVomGeraet: geraet };

const lage = ref(null);
const offen = ref(false);
const einschalten = ref(false);
const schaltet = ref(false);

async function laden() {
    try {
        lage.value = await geraet.automatischStand();
    } catch {
        lage.value = null;
    }
}

function oeffnen(an) {
    einschalten.value = an;
    offen.value = true;
}

async function umschalten(ereignis) {
    if (ereignis.target.checked) {
        // Erst im Dialog wählen; bis dahin bleibt der Schalter, wie er war.
        ereignis.target.checked = false;
        oeffnen(true);
        return;
    }
    schaltet.value = true;
    try {
        const l = lage.value;
        await geraet.automatischSetzen({
            an: false,
            alben: l.alben,
            nurWlan: l.nur_wlan,
            nurLaden: l.nur_laden,
            ohneKopie: l.ohne_kopie ?? true,
        });
        toast.success(t('albums.automatisch.ausgeschaltet'));
    } catch (e) {
        toast.error(typeof e === 'string' ? e : t('albums.vomGeraet.fehler'));
    } finally {
        schaltet.value = false;
        await laden();
    }
}

async function geschlossen() {
    offen.value = false;
    await laden();
}

onMounted(() => {
    if (geraet) laden();
});
</script>

<template>
  <div v-if="geraet" class="karte p-6 shadow-sm space-y-4">
    <h3 class="text-lg font-extrabold text-schrift flex items-center gap-2">
      <RefreshCw class="w-5 h-5 text-marke" /> {{ t('albums.automatisch.titel') }}
      <HilfeLink ziel="files-fotosVomGeraet" />
    </h3>
    <label class="flex items-center gap-3 text-sm">
      <input type="checkbox" :checked="lage?.an" :disabled="!lage || schaltet" @change="umschalten" />
      <span class="font-semibold text-schrift">{{ t('albums.automatisch.an') }}</span>
    </label>
    <p class="text-sm text-leise">{{ t('app.fotosAutomatisch.hinweis') }}</p>
    <div v-if="lage?.an" class="flex items-center justify-between gap-3 text-sm">
      <span class="text-fliess break-words">{{ t('app.fotosAutomatisch.aus', { alben: lage.alben.join(', ') }) }}</span>
      <button type="button" class="text-marke font-semibold shrink-0" @click="oeffnen(false)">{{ t('app.fotosAutomatisch.aendern') }}</button>
    </div>
    <NeueFotosAutomatisch v-if="offen" :quelle="quelle" :einschalten="einschalten" @close="geschlossen" />
  </div>
</template>
