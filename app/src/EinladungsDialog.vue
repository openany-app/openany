<script setup>
/*
 * Eine offene Einladung in ein lokales Projekt — über jeder Seite, wie die
 * Paarungsanfrage, aber nie mit ihr zu verwechseln
 * (docs/konzept-lokale-mitgliedschaften.md, §4): Ein „Passt" hier gibt EIN
 * Projekt frei, beim Paaren das ganze Gerät. Deshalb ein anderer Titel, ein
 * Satz, der Projekt und Einladende nennt, und ein Symbol dazu.
 *
 * KONTAKT BESTÄTIGEN (06.10.2026) läuft durch denselben Dialog, aber mit
 * eigenem Titel und Satz: Statt eines Projekts steht `openany-kontakt` im
 * Feld (einladen.rs, `KONTAKT`). Ein „Passt" macht die Person bekannt --
 * dann dürfen beide einander vor Ort schreiben.
 */
import { computed } from 'vue';
import { Users, UserCheck } from 'lucide-vue-next';
import BaseModal from '@oberflaeche/base/BaseModal.vue';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import { useNahAbgleich } from './nahAbgleich';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

const { offeneEinladung: e, einladungPasst, einladungAbbrechen } = useNahAbgleich();
const kontakt = computed(() => e.value?.projekt === 'openany-kontakt');
</script>

<template>
  <BaseModal v-if="e" :title="kontakt ? t('app.dialog.kontaktTitel') : e.rolle === 'gefragt' ? t('app.dialog.einladungTitel') : t('app.dialog.einladenTitel')" persistent
             @close="einladungAbbrechen(e.fingerabdruck)">
    <div class="space-y-3 text-center">
      <UserCheck v-if="kontakt" class="w-8 h-8 mx-auto text-marke" />
      <Users v-else class="w-8 h-8 mx-auto text-marke" />
      <p v-if="kontakt" class="text-sm text-fliess">
        <i18n-t :keypath="e.rolle === 'gefragt' ? 'app.dialog.kontaktGefragt' : 'app.dialog.kontaktAnfragend'" tag="span" scope="global">
          <template #name><strong>{{ e.rolle === 'gefragt' ? (e.von || e.name) : e.name }}</strong></template>
        </i18n-t>
      </p>
      <p v-else class="text-sm text-fliess">
        <i18n-t :keypath="e.rolle === 'gefragt' ? 'app.dialog.laedtDichEin' : 'app.dialog.duLaedstEin'" tag="span" scope="global">
          <template #name><strong>{{ e.rolle === 'gefragt' ? (e.von || e.name) : e.name }}</strong></template>
          <template #projekt><strong>{{ t('app.dialog.projektName', { name: e.projekt_name }) }}</strong></template>
        </i18n-t>
      </p>
      <p class="text-sm text-leise">{{ t('app.dialog.gleicherCode') }}</p>
      <p class="font-mono text-4xl tracking-[0.2em] text-marke">{{ e.code.slice(0, 3) }} {{ e.code.slice(3) }}</p>
      <p v-if="e.hier_bestaetigt" class="text-sm text-leise">{{ t('app.dialog.wartetAuf', { name: e.name }) }}</p>
    </div>
    <template v-if="!e.hier_bestaetigt" #footer>
      <BaseButton variant="secondary" @click="einladungAbbrechen(e.fingerabdruck)">{{ t('app.dialog.passtNicht') }}</BaseButton>
      <BaseButton @click="einladungPasst(e.fingerabdruck)">{{ t('app.dialog.passt') }}</BaseButton>
    </template>
  </BaseModal>
</template>
