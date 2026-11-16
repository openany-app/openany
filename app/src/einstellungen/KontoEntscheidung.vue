<script setup>
/*
 * Eine Entscheidung der Verwaltung über dieses Konto: gesperrt oder
 * gelöscht, mit Grundlage und Begründung (Digital Services Act, Art. 17).
 * Die Begründung steht wörtlich, wie die Verwaltung sie geschrieben hat;
 * alles drumherum in der Sprache der App.
 *
 * `art`: `sperre` (daten = Kontofristen.sperre) oder `abschied`
 * (daten = Abschied, das Konto ist fort).
 */
import { useI18n } from 'vue-i18n';
import { openUrl } from '@tauri-apps/plugin-opener';
import { meldeadresse } from '@oberflaeche/support/meldeadressen';
import { useToast } from '@oberflaeche/composables/useToast';

defineProps({
    art: { type: String, required: true },
    daten: { type: Object, required: true },
});
const emit = defineEmits(['verstanden']);
const { t, locale } = useI18n();
const toast = useToast();

function datum(iso) {
    return iso ? new Date(iso).toLocaleDateString(locale.value, { dateStyle: 'long' }) : '';
}

async function widersprechen() {
    try {
        await openUrl(meldeadresse(locale.value, 'https://openany.de', 'widerspruch'));
    } catch (e) {
        toast.error(String(e));
    }
}
</script>

<template>
  <div class="space-y-4">
    <p class="text-sm font-bold text-schrift">
      {{ art === 'sperre' ? t('app.entscheidung.gesperrtSeit', { datum: datum(daten.seit) }) : t('app.entscheidung.geloeschtAm', { datum: datum(daten.am) }) }}
    </p>
    <div class="rounded-xl bg-vertieft p-3 space-y-2">
      <p class="text-xs font-bold text-leise">{{ t('app.entscheidung.grundlage') }}</p>
      <p class="text-sm text-schrift">{{ t(`app.entscheidung.grundlagen.${daten.grundlage}`) }}</p>
      <p class="text-xs font-bold text-leise">{{ t('app.entscheidung.begruendung') }}</p>
      <p class="text-sm text-schrift whitespace-pre-wrap">{{ daten.begruendung }}</p>
    </div>
    <p class="text-sm text-fliess">{{ t('app.entscheidung.mensch') }}</p>
    <p class="text-sm text-fliess">{{ art === 'sperre' ? t('app.entscheidung.folgeSperre') : t('app.entscheidung.folgeLoeschung') }}</p>
    <p class="text-sm text-fliess">{{ t('app.entscheidung.widerspruchHinweis') }}</p>
    <div class="flex flex-wrap gap-2">
      <button type="button" class="knopf" @click="widersprechen">{{ t('app.entscheidung.widersprechen') }}</button>
      <button type="button" class="knopf-wichtig" @click="emit('verstanden')">{{ t('app.entscheidung.verstanden') }}</button>
    </div>
  </div>
</template>
