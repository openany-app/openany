<script setup>
/*
 * Ein Passwortfeld mit Augen-Knopf: Wer tippt, kann nachsehen, was er getippt
 * hat (docs/plan-matrix-bestaetigen.md, P). Verborgen ist die Vorgabe; jedes
 * neue Einhängen des Felds beginnt wieder verborgen.
 *
 * Alles außer `modelValue` geht an das <input> (Klasse, autocomplete, id,
 * required …) -- das Feld sieht damit aus wie die übrigen der Ansicht.
 */
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { Eye, EyeOff } from 'lucide-vue-next';

defineOptions({ inheritAttrs: false });
const model = defineModel({ type: String, default: '' });

const { t } = useI18n();
const sichtbar = ref(false);
</script>

<template>
  <div class="relative">
    <input
      v-model="model"
      v-bind="$attrs"
      :type="sichtbar ? 'text' : 'password'"
      autocapitalize="off"
      autocorrect="off"
      spellcheck="false"
      class="pr-11"
    />
    <button
      type="button"
      class="absolute right-1 bottom-0 top-1 px-2.5 flex items-center text-leise hover:text-schrift cursor-pointer"
      :aria-label="sichtbar ? t('common.passwortVerbergen') : t('common.passwortZeigen')"
      :aria-pressed="sichtbar"
      :title="sichtbar ? t('common.passwortVerbergen') : t('common.passwortZeigen')"
      @click="sichtbar = !sichtbar"
    >
      <EyeOff v-if="sichtbar" class="w-4 h-4" />
      <Eye v-else class="w-4 h-4" />
    </button>
  </div>
</template>
