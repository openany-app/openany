<script setup>
/*
 * Ein geteilter Kalender, geöffnet über den Freigaben eines Hubs: ein
 * eigener Rahmen über dem Hub wie bei Mappen, Alben und
 * Ordnern. Er zeigt nur diesen einen Kalender; „Zurück" (auch die
 * Android-Taste) schließt ihn, und man steht wieder in den Freigaben.
 */
import { useI18n } from 'vue-i18n';
import { ArrowLeft } from 'lucide-vue-next';
import KalenderArbeitsflaeche from '@oberflaeche/kalender/KalenderArbeitsflaeche.vue';
import { useZurueckSchliesst } from '@oberflaeche/composables/useZurueckSchliesst';
import { lokaleKalenderQuelle } from '../../quellen/kalender';

// kalender: { name, kalender (eigene uuid), uuid (Freigabe), eigen }
const props = defineProps({ kalender: { type: Object, required: true } });
const emit = defineEmits(['close']);
const { t } = useI18n();

// Ein eigener heißt im Kalender wie bei dir, ein fremder `p:<freigabe>`.
const id = props.kalender.eigen ? props.kalender.kalender : `p:${props.kalender.uuid}`;

// Dieselbe Quelle wie im Kalender -- nur auf diesen einen beschränkt.
const alle = lokaleKalenderQuelle();
const quelle = {
  ...alle,
  getCalendars: async () => ({ data: (await alle.getCalendars()).data.filter((k) => k.id === id) }),
  getEvents: async (von, bis) => ({ data: (await alle.getEvents(von, bis)).data.filter((e) => e.calendar_id === id) }),
};

useZurueckSchliesst(() => emit('close'));
</script>

<template>
  <div class="rahmen-ueber-leiste z-[90] bg-boden flex flex-col"
    :style="{ paddingTop: 'var(--rand-oben)', paddingLeft: 'var(--rand-links)', paddingRight: 'var(--rand-rechts)' }">
    <div class="flex items-center gap-2 px-4 py-3 border-b border-linie shrink-0">
      <button type="button" class="p-2 rounded-xl hover:bg-auflage" :title="t('common.back')" @click="emit('close')"><ArrowLeft class="w-5 h-5" /></button>
      <span class="font-bold text-schrift truncate">{{ kalender.name }}</span>
    </div>
    <!-- Die Arbeitsfläche klebt sonst unter dem Rand oben fest -- hier ist
         die Leiste darüber der Rand. -->
    <div class="flex-1 min-h-0 overflow-y-auto" style="--rand-oben: 0px">
      <KalenderArbeitsflaeche :data-source="quelle" :fokus-kalender="id" />
    </div>
  </div>
</template>
