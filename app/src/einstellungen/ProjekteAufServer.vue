<script setup>
/*
 * „Projekte auf openany.de" -- der Schalter je Projekt, als Liste unter
 * Abgleich (docs/plan-projekte-eine-art.md).
 *
 * NUR DIE EIGENEN GRÜNDUNGEN: Umschalten darf, wer das Projekt gegründet
 * hat; fremde Projekte stehen hier gar nicht erst.
 *
 * AUSSCHALTEN FRAGT NACH: Auf openany.de gehen dabei Chat, Planung und
 * Freigaben verloren. Vor Ort läuft das Projekt weiter.
 */
import { ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { Cloud } from 'lucide-vue-next';
import { useToast } from '@oberflaeche/composables/useToast';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import { projekteQuelle as quelle } from '../quellen/projekte';

const { t } = useI18n();
const toast = useToast();
const { confirmDelete } = useConfirm();

const projekte = ref(null);
const arbeitet = ref(null);

async function laden() {
    try {
        projekte.value = await quelle.gruendungen();
    } catch (e) {
        projekte.value = [];
        toast.error(String(e));
    }
}
onMounted(laden);

async function umschalten(p, e) {
    const an = Boolean(e.target.checked);
    if (!an && !(await confirmDelete(t('app.abgleich.ausschaltenFrage', { name: p.name }), {
        title: t('app.abgleich.ausschaltenTitel'),
        confirmLabel: t('app.abgleich.ausschaltenKnopf'),
    }))) {
        e.target.checked = true;
        return;
    }
    arbeitet.value = p.id;
    try {
        if (an) {
            const stand = await quelle.einschalten(p.id);
            toast.success(t('app.abgleich.eingeschaltet', { name: p.name })
                + (stand?.nur_vor_ort ? ' ' + t('app.abgleich.ohneKonto', { anzahl: stand.nur_vor_ort }) : ''));
        } else {
            await quelle.ausschalten(p.id);
            toast.success(t('app.abgleich.ausgeschaltet', { name: p.name }));
        }
    } catch (err) {
        toast.error(String(err));
    } finally {
        arbeitet.value = null;
        await laden();
    }
}
</script>

<template>
  <h2 class="mt-6 text-base font-extrabold text-schrift flex items-center gap-2">
    <Cloud class="w-4 h-4 text-marke" /> {{ t('app.abgleich.projekteTitel') }}
  </h2>
  <p class="mt-2 text-sm text-leise md:max-w-[460px]">{{ t('app.abgleich.projekteHinweis') }}</p>

  <p v-if="projekte && !projekte.length" class="mt-2 text-sm text-leise">{{ t('app.abgleich.projekteKeine') }}</p>
  <label v-for="p in projekte || []" :key="p.id" class="mt-3 flex items-center gap-3 text-sm text-fliess md:max-w-[460px]">
    <input type="checkbox" :checked="p.auf_server" :disabled="arbeitet !== null" @change="umschalten(p, $event)">
    <span class="font-bold truncate">{{ p.name }}</span>
    <span v-if="arbeitet === p.id" class="text-leise">{{ t('app.allgemein.laeuft') }}</span>
  </label>
</template>
