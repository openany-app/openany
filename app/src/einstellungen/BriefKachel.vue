<script setup>
/*
 * Absender und Ort für Briefe aus Notizen (PDF → Brief, DIN 5008). Liegt auf
 * diesem Gerät (shared/briefAbsender.js); der Brief-Dialog liest es vor und
 * merkt sich Änderungen am Absender.
 */
import { ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { Mail } from 'lucide-vue-next';
import { absenderLesen, absenderMerken } from '@oberflaeche/shared/briefAbsender';
import HilfeLink from '@oberflaeche/base/HilfeLink.vue';

const { t } = useI18n();
const werte = ref(absenderLesen());
watch(werte, (w) => absenderMerken(w), { deep: true });

const feldKlasse = 'w-full px-4 py-3 bg-vertieft border border-linie rounded-xl focus:outline-none focus:ring-2 focus:ring-marke text-sm font-semibold text-schrift transition-all';
</script>

<template>
  <div class="karte p-6 shadow-sm space-y-4">
    <h3 class="text-lg font-extrabold text-schrift flex items-center gap-2">
      <Mail class="w-5 h-5 text-marke" /> {{ t('app.brief.title') }}
      <HilfeLink ziel="notes-export" />
    </h3>
    <p class="text-sm text-leise">{{ t('app.brief.intro') }}</p>
    <div class="space-y-1.5">
      <label for="brief-absender-einstellung" class="block text-sm font-bold text-fliess">{{ t('app.brief.absender') }}</label>
      <textarea id="brief-absender-einstellung" v-model="werte.absender" rows="3" :class="feldKlasse" :placeholder="t('app.brief.absenderPlatzhalter')" />
    </div>
    <div class="space-y-1.5">
      <label for="brief-ort-einstellung" class="block text-sm font-bold text-fliess">{{ t('app.brief.ort') }}</label>
      <input id="brief-ort-einstellung" v-model="werte.ort" type="text" :class="feldKlasse" :placeholder="t('app.brief.ortPlatzhalter')" />
    </div>
  </div>
</template>
