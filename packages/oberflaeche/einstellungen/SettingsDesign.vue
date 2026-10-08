<script setup>
// Auswahl des Aussehens. Zwei Kacheln, sofortiges Speichern – wie bei der
// Modul-Auswahl nebenan. Ein Design-Wechsel ist folgenlos und umkehrbar;
// ein Formular mit Speichern-Knopf waere hier zu viel Zeremonie.
//
// Nicht zu verwechseln mit dem Hell/Dunkel-Schalter im Kopf der Seite: Das
// ist eine zweite, davon unabhaengige Achse. Jedes Design gibt es hell und
// dunkel.
import { useI18n } from 'vue-i18n';
import { Palette, Check } from 'lucide-vue-next';
// Aus dem Paket: Wo die Wahl zusätzlich gespeichert wird (Webapp: im Konto),
// hängt die Anwendung selbst an (designSpeichernMit).
import { useDesign } from '@oberflaeche/composables/useDesign';
import { useToast } from '@oberflaeche/composables/useToast';

const { t } = useI18n();
const toast = useToast();
const { design, setDesign } = useDesign();

// Ein Farbtupfer je Design, damit man sieht, worum es geht, bevor man klickt.
// Bis zum 08.09.2026 standen hier ZWEI Punkte je Design – der helle und der
// dunkle Ton. Seit jedes Design nur noch EINE Farbe hat (style.css, `--marke`),
// zeigten beide Punkte dasselbe.
//
// Fest verdrahtet und nicht aus der Palette gezogen: Die Palette trägt immer
// nur das gerade aktive Design, hier sollen aber beide nebeneinander stehen.
// FOLGT DAMIT NICHT `--marke`; wer dort die Marke ändert, ändert das hier mit.
const DESIGNS = [
  { id: 'klar', ton: '#2a8d97' },
  { id: 'flieder', ton: '#885c8f' },
];

const waehlen = async (id) => {
  try {
    await setDesign(id);
  } catch (e) {
    toast.error(t('settings.design.saveFailed'));
  }
};
</script>

<template>
  <div class="karte p-6 shadow-sm space-y-6">
    <h3 class="text-lg font-extrabold text-schrift flex items-center gap-2">
      <Palette class="w-5 h-5 text-marke" /> {{ t('settings.design.title') }}
    </h3>
    <p class="text-sm text-leise font-medium -mt-3">
      {{ t('settings.design.description') }}
    </p>

    <div class="space-y-3">
      <button
        v-for="d in DESIGNS"
        :key="d.id"
        type="button"
        role="radio"
        :aria-checked="design === d.id"
        @click="waehlen(d.id)"
        class="w-full flex items-center justify-between gap-4 p-4 border rounded-xl text-left transition-all duration-300 cursor-pointer"
        :class="design === d.id
          ? 'bg-marke-leise border-marke shadow-sm'
          : 'border-linie hover:bg-slate-50 dark:hover:bg-slate-800/50'"
      >
        <div class="flex items-center gap-3 min-w-0">
          <!-- Der Ton dieses Designs; er gilt in heller wie dunkler Ansicht. -->
          <span class="w-5 h-5 rounded-full border-2 border-white dark:border-slate-900 shrink-0"
                aria-hidden="true" :style="{ backgroundColor: d.ton }"></span>
          <span class="min-w-0">
            <span
              class="block font-bold transition-colors"
              :class="design === d.id ? 'text-marke' : 'text-fliess'"
            >
              {{ t(`settings.design.${d.id}.label`) }}
            </span>
            <span class="block text-xs font-medium text-slate-400 dark:text-slate-500 mt-0.5">
              {{ t(`settings.design.${d.id}.hint`) }}
            </span>
          </span>
        </div>
        <Check v-if="design === d.id" class="w-5 h-5 text-marke shrink-0" />
      </button>
    </div>
  </div>
</template>
