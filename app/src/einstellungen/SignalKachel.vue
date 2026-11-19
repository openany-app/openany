<script setup>
/*
 * Nachrichten-Signal (signal.js): Ton und Vibrieren bei offener App. Bei
 * geschlossener App entscheidet unter Android das System -- dorthin führt
 * der Knopf.
 */
import { useI18n } from 'vue-i18n';
import { BellRing } from 'lucide-vue-next';
import HilfeLink from '@oberflaeche/base/HilfeLink.vue';
import { tonAn, vibrierenAn, kannVibrieren } from '../signal';

const { t } = useI18n();
const kannSystem = Boolean(window.openanyWachdienst?.systemEinstellungen);
const systemOeffnen = () => window.openanyWachdienst.systemEinstellungen();
</script>

<template>
  <div class="karte p-6 shadow-sm space-y-4">
    <h3 class="text-lg font-extrabold text-schrift flex items-center gap-2">
      <BellRing class="w-5 h-5 text-marke" /> {{ t('app.signal.titel') }}
      <HilfeLink ziel="settings-signal" />
    </h3>
    <label class="flex items-center gap-3 text-sm">
      <input v-model="tonAn" type="checkbox" />
      <span class="font-semibold text-schrift">{{ t('app.signal.ton') }}</span>
    </label>
    <label v-if="kannVibrieren" class="flex items-center gap-3 text-sm">
      <input v-model="vibrierenAn" type="checkbox" />
      <span class="font-semibold text-schrift">{{ t('app.signal.vibrieren') }}</span>
    </label>
    <p class="text-sm text-leise">{{ t('app.signal.hinweis') }}</p>
    <button v-if="kannSystem" type="button" class="text-marke font-semibold text-sm"
            @click="systemOeffnen">{{ t('app.signal.systemEinstellungen') }}</button>
  </div>
</template>
