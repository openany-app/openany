<script setup>
// Modul-Auswahl für die Startseite. Die Auswahl steuert nur die Startseite
// und darf leer sein – dann zeigt die Startseite die Willkommens-Übersicht.
//
// WOHIN DIE AUSWAHL GEHT, SAGT DIE ANWENDUNG (`speichern`): die Webapp ins
// Konto, das Programm auf das Gerät. Und WELCHE Module zur Wahl stehen
// (`auswahl`): das Programm bietet nur an, was es trägt.
import { ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { Settings as SettingsIcon } from 'lucide-vue-next';
import { MODULES } from '@oberflaeche/module';
import { useToast } from '@oberflaeche/composables/useToast';

const { t } = useI18n();
const toast = useToast();

const props = defineProps({
  modules: { type: Array, required: true },        // aktivierte Module
  // (ids) => Promise<ids> – liefert die verbindliche Liste zurück.
  speichern: { type: Function, required: true },
  // Ids, die zur Wahl stehen; ohne Angabe alle mit eigener Kachel.
  auswahl: { type: Array, default: null },
});
// Bei Fehlern soll der Eltern-View die Einstellungen komplett neu laden.
const emit = defineEmits(['reload']);

const selected = ref([...props.modules]);
watch(() => props.modules, (v) => { selected.value = [...v]; });

// Alle Module mit eigener Kachel. Bis zum 30.08.2026 filterte hier
// zusätzlich die Rollen-Obergrenze mit; sie ist gefallen.
const visibleModules = MODULES.filter((m) => m.homeSelectable !== false
  && (!props.auswahl || props.auswahl.includes(m.id)));

const toggleModule = async (modId) => {
  const index = selected.value.indexOf(modId);
  if (index > -1) {
    selected.value.splice(index, 1);
  } else {
    selected.value.push(modId);
  }
  try {
    selected.value = (await props.speichern([...selected.value])) ?? selected.value;
    if (!selected.value.length) {
      toast.success(t('settings.modules.noneSelected'));
    }
  } catch (err) {
    toast.error(t('settings.modules.saveFailed'));
    emit('reload');
  }
};
</script>

<template>
  <div class="karte p-6 shadow-sm space-y-6">
    <h3 class="text-lg font-extrabold text-schrift flex items-center gap-2"><SettingsIcon class="w-5 h-5 text-marke" /> {{ t('settings.modules.title') }}</h3>
    <p class="text-sm text-leise font-medium -mt-3">
      {{ t('settings.modules.description') }}
    </p>
    <div class="space-y-3">
      <label
        v-for="mod in visibleModules"
        :key="mod.id"
        class="flex items-center justify-between gap-4 p-4 border rounded-xl transition-all duration-300"
        :class="[
          selected.includes(mod.id)
            ? 'bg-marke-leise border-marke shadow-sm cursor-pointer'
            : 'border-linie hover:bg-slate-50 dark:hover:bg-slate-800/50 cursor-pointer',
        ]"
      >
        <div class="min-w-0">
          <span
            class="font-bold transition-colors flex items-center gap-2"
            :class="selected.includes(mod.id) ? 'text-marke' : 'text-fliess'"
          >
            <component :is="mod.icon" class="w-4 h-4 text-marke shrink-0" />
            {{ t(mod.label) }}
          </span>
          <span class="block text-xs font-medium text-slate-400 dark:text-slate-500 mt-0.5">
            {{ t(mod.description) }}
          </span>
        </div>
        <div
          class="relative w-11 h-6 rounded-full transition-colors shrink-0"
          :class="selected.includes(mod.id) ? 'bg-marke' : 'bg-slate-200 dark:bg-slate-700'"
          role="switch"
          :aria-checked="selected.includes(mod.id)"
          :aria-label="t(mod.label)"
          @click="toggleModule(mod.id)"
        >
          <div class="absolute top-[2px] left-[2px] bg-white border-slate-300 border rounded-full h-5 w-5 transition-all shadow-sm" :class="selected.includes(mod.id) ? 'translate-x-full border-white' : ''"></div>
        </div>
      </label>
    </div>
  </div>
</template>
