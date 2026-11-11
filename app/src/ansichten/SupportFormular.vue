<script setup>
/*
 * Support am Ende der Hilfe (docs/plan-support.md).
 *
 * Die Anfrage geht als Ende-zu-Ende-verschlüsselte Direktnachricht an jeden
 * Verwalter; die Antwort kommt in den Nachrichten. Vor dem Senden sucht
 * `personenbezug` nach E-Mail-Adressen, Telefonnummern, IP-Adressen und
 * Namen -- solange es etwas findet, bleibt der Knopf gesperrt.
 */
import { ref, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { LifeBuoy, ShieldAlert, Loader2, Send, Info, ChevronRight, Flag } from 'lucide-vue-next';
import { openUrl } from '@tauri-apps/plugin-opener';
import { meldeadresse } from '@oberflaeche/support/meldeadressen';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import { useToast } from '@oberflaeche/composables/useToast';
import { personenbezug } from '@oberflaeche/support/personenbezug';

const emit = defineEmits(['ueber']);
const { t, locale } = useI18n();
const toast = useToast();

const text = ref('');
const sendet = ref(false);
const fehler = ref('');
const verbunden = ref(true);

const funde = computed(() => personenbezug(text.value));

async function senden() {
    if (funde.value.length || !text.value.trim()) return;
    fehler.value = '';
    sendet.value = true;
    try {
        await invoke('support_senden', { text: text.value });
        text.value = '';
        toast.success(t('support.gesendet'));
    } catch (e) {
        const s = String(e);
        fehler.value = s === 'grund:kein_verwalter' ? t('support.keinVerwalter') : s;
    } finally {
        sendet.value = false;
    }
}

onMounted(async () => {
    try {
        verbunden.value = Boolean((await invoke('lage')).gekoppelt);
    } catch {
        verbunden.value = false;
    }
});

async function meldenOeffnen() {
    try {
        await openUrl(meldeadresse(locale.value));
    } catch (e) {
        toast.error(String(e));
    }
}
</script>

<template>
  <div class="space-y-4">
    <div class="flex items-center gap-3">
      <div class="w-9 h-9 rounded-xl bg-marke-leise flex items-center justify-center text-marke">
        <LifeBuoy class="w-4.5 h-4.5" />
      </div>
      <h2 class="text-xl font-extrabold text-schrift tracking-tight">{{ t('support.titel') }}</h2>
    </div>
    <p class="text-sm leading-relaxed text-fliess">{{ t('support.einleitung') }}</p>
    <p class="text-sm leading-relaxed rounded-xl bg-vertieft p-3 text-schrift flex gap-2">
      <ShieldAlert class="w-4 h-4 shrink-0 mt-0.5 text-marke" /> {{ t('support.keineDaten') }}
    </p>

    <p v-if="!verbunden" class="text-sm text-leise">{{ t('support.nichtVerbunden') }}</p>
    <form v-else class="space-y-3" @submit.prevent="senden">
      <textarea v-model="text" rows="6" maxlength="4000" :placeholder="t('support.platzhalter')"
                class="feld w-full" :class="funde.length ? 'ring-2 ring-rose-400' : ''"></textarea>

      <div v-if="funde.length" class="text-sm text-rose-600 dark:text-rose-400 space-y-1">
        <p class="font-bold">{{ t('support.gefunden') }}</p>
        <ul class="list-disc pl-5">
          <li v-for="(f, i) in funde" :key="i">
            {{ t(`support.art.${f.art}`) }}: <span class="font-mono break-all">„{{ f.treffer }}“</span>
          </li>
        </ul>
      </div>

      <BaseButton type="submit" :disabled="sendet || funde.length > 0 || !text.trim()">
        <Loader2 v-if="sendet" class="w-4 h-4 animate-spin" /><Send v-else class="w-4 h-4" />
        {{ t('support.senden') }}
      </BaseButton>
      <p v-if="fehler" class="text-sm text-rose-600 dark:text-rose-400">{{ fehler }}</p>
    </form>

    <button type="button" class="w-full flex items-center gap-3 mt-4 p-3 rounded-xl bg-vertieft text-left cursor-pointer" @click="emit('ueber')">
      <Info class="w-5 h-5 text-marke shrink-0" />
      <span class="flex-1 min-w-0">
        <span class="block text-sm font-bold text-schrift">{{ t('app.ueber.titel') }}</span>
        <span class="block text-xs text-leise">{{ t('app.ueber.untertitel') }}</span>
      </span>
      <ChevronRight class="w-4 h-4 text-slate-400 shrink-0" />
    </button>

    <!--
      Inhalt melden (docs/plan-registrierung-app.md, A5): kein eigener Weg in
      der App, sondern die Seite openany.de/melden im Browser -- dort geht es
      auch ohne Konto, mit Zugangscode für Stand und Antwort. Die Adresse
      folgt der eingestellten Sprache (meldeadressen.js).
    -->
    <div class="pt-4 border-t border-linie space-y-2">
      <h3 class="text-base font-extrabold text-schrift flex items-center gap-2"><Flag class="w-4 h-4 text-marke" /> {{ t('support.meldenTitel') }}</h3>
      <p class="text-sm leading-relaxed text-fliess">{{ t('support.meldenText') }}</p>
      <BaseButton variant="secondary" @click="meldenOeffnen">{{ t('support.meldenKnopf') }}</BaseButton>
    </div>
  </div>
</template>
