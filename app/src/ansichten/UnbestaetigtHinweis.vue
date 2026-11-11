<script setup>
/*
 * Ein eigenes, freigegebenes Gerät ist nicht vom Konto gesiegelt
 * (docs/plan-geraete-vertrauen.md). Was zu tun ist, steht in den
 * Einstellungen → Meine Geräte.
 *
 * `art`: `knopf` -- dezentes Schild im Kopf (breite Bildschirme); `streifen`
 * -- schmaler Streifen oben wie „offline“, wo es keinen Kopf gibt (Handy,
 * Tablet hochkant). Ein Punkt über dem Inhalt verdeckte dort Knöpfe der
 * Ansicht.
 */
import { computed, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { ShieldAlert } from 'lucide-vue-next';
import { unbestaetigteGeraete } from './unbestaetigt';

const props = defineProps({ art: { type: String, default: 'knopf' } });
const emit = defineEmits(['oeffnen']);
const { t } = useI18n();
const { anzahl, anmelden, abmelden } = unbestaetigteGeraete();

const text = computed(() => t('app.nachrichtenschluessel.unbestaetigtKopf', { anzahl: anzahl.value }, anzahl.value));
const kurz = computed(() => t('app.nachrichtenschluessel.unbestaetigtStreifen', { anzahl: anzahl.value }, anzahl.value));

onMounted(anmelden);
onUnmounted(abmelden);
</script>

<template>
  <button
    v-if="anzahl > 0 && props.art === 'knopf'"
    type="button"
    class="pille flex items-center justify-center w-9 h-9 cursor-pointer text-warnung"
    :title="text"
    :aria-label="text"
    @click="emit('oeffnen')"
  >
    <ShieldAlert class="w-4 h-4" />
  </button>
  <button
    v-else-if="anzahl > 0"
    type="button"
    class="w-full flex items-center justify-center gap-2 px-4 py-1.5 bg-flaeche text-warnung text-xs font-bold cursor-pointer"
    :aria-label="text"
    @click="emit('oeffnen')"
  >
    <ShieldAlert class="w-3.5 h-3.5 shrink-0" />
    <span>{{ kurz }}</span>
  </button>
</template>
