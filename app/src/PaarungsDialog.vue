<script setup>
/*
 * Eine offene Paarung — über jeder Seite, nicht nur in den Einstellungen.
 *
 * ALS DIALOG und nicht als Kasten in einer Liste: Am 15.09.2026 erschien der
 * Kasten auf dem Tablet mitten im Tippen, schob „Suchen" nach unten, und der
 * Tipp traf „Passt" — eine Bestätigung ohne Blick auf den Code. Ein Dialog
 * verschiebt nichts und legt sich über alles, was man gerade treffen wollte.
 */
import BaseModal from '@oberflaeche/base/BaseModal.vue';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import { useNahAbgleich } from './nahAbgleich';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

const { offeneAnfrage, passt, abbrechen } = useNahAbgleich();
</script>

<template>
  <BaseModal v-if="offeneAnfrage" :title="offeneAnfrage.rolle === 'gefragt' ? t('app.dialog.paarungsanfrage') : t('app.dialog.paaren')" persistent
             @close="abbrechen(offeneAnfrage.fingerabdruck)">
    <div class="space-y-3 text-center">
      <p class="text-sm text-fliess">
        <i18n-t :keypath="offeneAnfrage.rolle === 'gefragt' ? 'app.dialog.moechtePaaren' : 'app.dialog.paarenMit'" tag="span" scope="global">
          <template #name><strong>{{ offeneAnfrage.name }}</strong></template>
        </i18n-t>
      </p>
      <p class="text-sm text-leise">{{ t('app.dialog.gleicherCode') }}</p>
      <p class="font-mono text-4xl tracking-[0.2em] text-marke">{{ offeneAnfrage.code.slice(0, 3) }} {{ offeneAnfrage.code.slice(3) }}</p>
      <p v-if="offeneAnfrage.hier_bestaetigt" class="text-sm text-leise">{{ t('app.dialog.wartetAnderes') }}</p>
    </div>
    <template v-if="!offeneAnfrage.hier_bestaetigt" #footer>
      <BaseButton variant="secondary" @click="abbrechen(offeneAnfrage.fingerabdruck)">{{ t('app.dialog.passtNicht') }}</BaseButton>
      <BaseButton @click="passt(offeneAnfrage.fingerabdruck)">{{ t('app.dialog.passt') }}</BaseButton>
    </template>
  </BaseModal>
</template>
