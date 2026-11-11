<script setup>
/*
 * Das Impressum am Ende der Hilfe.
 *
 * Geholt wird erst, wenn der Abschnitt in Sicht kommt: Wer die Hilfe auf einem
 * Gerät ohne Konto nur öffnet, soll deshalb nicht openany.de anrufen.
 */
import { onBeforeUnmount, onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { Landmark } from 'lucide-vue-next';
import RechtstextAnzeige from '@oberflaeche/shared/RechtstextAnzeige.vue';
import { gemerkterRechtstext, rechtstextHolen } from '../quellen/rechtstext';

const { t } = useI18n();

const wurzel = ref(null);
const text = ref(gemerkterRechtstext('impressum'));
const veraltet = ref(false);
const fehlt = ref(false);

let beobachter = null;
async function laden() {
  try {
    text.value = await rechtstextHolen('impressum');
    veraltet.value = false;
    fehlt.value = false;
  } catch {
    veraltet.value = Boolean(text.value);
    fehlt.value = !text.value;
  }
}

onMounted(() => {
  beobachter = new IntersectionObserver((eintraege) => {
    if (!eintraege.some((e) => e.isIntersecting)) return;
    beobachter.disconnect();
    laden();
  });
  beobachter.observe(wurzel.value);
});
onBeforeUnmount(() => beobachter?.disconnect());
</script>

<template>
  <div ref="wurzel" class="space-y-4">
    <div class="flex items-center gap-3">
      <div class="w-9 h-9 rounded-xl bg-marke-leise flex items-center justify-center text-marke">
        <Landmark class="w-4.5 h-4.5" />
      </div>
      <h2 class="text-xl font-extrabold text-schrift tracking-tight">{{ t('shell.footer.imprint') }}</h2>
    </div>
    <p v-if="veraltet" class="rounded-xl bg-vertieft p-3 text-sm text-schrift">{{ t('app.ueber.veraltet') }}</p>
    <p v-if="fehlt" class="text-sm text-leise">{{ t('app.ueber.nichtErreichbar') }}</p>
    <RechtstextAnzeige v-else-if="text" :text="text" ohne-titel />
  </div>
</template>
