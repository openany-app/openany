<script setup>
/*
 * Gewechselte Sicherheitsnummern (docs/plan-geraete-vertrauen.md, D).
 *
 * Erscheint nur, wenn ein Kontakt, dessen Nummer du verglichen hast, seither
 * einen anderen Konto-Schlüssel hat -- ein neues Konto, oder jemand, der sich
 * dazwischen gestellt hat. Ein Tipp öffnet denselben Dialog wie im Adressbuch;
 * die Markierungen dort und in den Nachrichten bleiben.
 */
import { ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { ShieldAlert, ChevronRight } from 'lucide-vue-next';
import SicherheitsnummerDialog from '@oberflaeche/sicherheit/SicherheitsnummerDialog.vue';
import { sicherheitQuelle } from '../quellen/sicherheit';

const { t } = useI18n();
const gewechselt = ref([]);
const offen = ref(null);

async function laden() {
    gewechselt.value = await sicherheitQuelle.sicherheitsnummernGeaendert().catch(() => []);
}

onMounted(laden);
</script>

<template>
  <section v-if="gewechselt.length" class="karte p-5 space-y-3">
    <h3 class="text-base font-bold text-schrift flex items-center gap-2">
      <ShieldAlert class="w-5 h-5 text-warnung" /> {{ t('app.sicherheitsnummern.titel') }}
    </h3>
    <p class="text-sm text-fliess">{{ t('app.sicherheitsnummern.hinweis') }}</p>
    <ul class="divide-y divide-linie border border-linie rounded-xl overflow-hidden">
      <li v-for="g in gewechselt" :key="g.name">
        <button type="button" class="w-full px-3 py-2.5 flex items-center gap-2 text-left cursor-pointer" @click="offen = g.name">
          <span class="text-sm font-semibold text-schrift flex-1 min-w-0 break-words">{{ g.name }}</span>
          <span class="text-xs text-leise">{{ t('app.sicherheitsnummern.vergleichen') }}</span>
          <ChevronRight class="w-4 h-4 text-leise" />
        </button>
      </li>
    </ul>
    <SicherheitsnummerDialog v-if="offen" :name="offen" :quelle="sicherheitQuelle"
      @close="offen = null" @geaendert="laden" />
  </section>
</template>
