<script setup>
/*
 * Der ehrliche Platzhalter.
 *
 * Ein Reiter, der eine leere Liste zeigt, behauptet „du hast nichts". Bei
 * Speicher und Projekten ist das falsch — es gibt dort etwas, dieses Programm
 * holt es nur nicht. Der Unterschied ist für den Menschen kein feiner: Im
 * einen Fall sucht er seine Dateien, im anderen weiß er, woran er ist.
 */
import { useI18n } from 'vue-i18n';

const { t } = useI18n();
defineProps({ titel: String, grund: String });
</script>

<template>
  <h1 class="text-2xl font-extrabold text-schrift mb-4">{{ titel }}</h1>

  <div class="max-w-xl bg-flaeche border border-linie p-6 rounded-xl shadow-sm space-y-3">
    <p class="text-sm text-fliess font-medium">{{ grund }}</p>
    <p class="text-sm text-leise">{{ t('app.speicherAnsicht.nochNicht') }}</p>
  </div>
</template>
