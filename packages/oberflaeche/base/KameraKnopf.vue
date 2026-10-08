<script setup>
/*
 * Foto oder Video aufnehmen -- über die Kamera-App des Systems.
 *
 * `<input capture>` öffnet auf Telefon und Tablet direkt die Kamera, in der
 * Webapp wie im Programm (Tauri reicht es an `ACTION_IMAGE_CAPTURE` bzw.
 * `ACTION_VIDEO_CAPTURE` weiter). Das Ergebnis kommt als gewöhnliche Datei
 * zurück und geht denselben Weg wie jede hochgeladene: in einer Akte durch
 * die Texterkennung, in der Galerie mit Standbild.
 *
 * ZWEI FELDER, NICHT EINES. Ein Feld mit `accept="image/*,video/*"` öffnet im
 * Programm nur die Videoaufnahme (Tauri entscheidet sich bei beidem für
 * Video). Deshalb je Art ein Feld und ein Knopf.
 *
 * NUR AUF TOUCH-GERÄTEN. Am Schreibtisch ignoriert der Browser `capture` und
 * öffnet die Dateiauswahl -- ein Knopf „Foto aufnehmen", der eine
 * Dateiauswahl zeigt, wäre irreführend. `pointer: coarse` ist dieselbe
 * Unterscheidung, mit der die Oberfläche sonst Telefon und Rechner trennt.
 */
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { Camera, Video } from 'lucide-vue-next';

const props = defineProps({
  // 'foto' (Akten) oder 'foto-video' (Galerie)
  arten: { type: String, default: 'foto' },
  disabled: { type: Boolean, default: false },
});
const emit = defineEmits(['aufgenommen']);
const { t } = useI18n();

const beruehrbar = typeof window !== 'undefined'
  && typeof window.matchMedia === 'function'
  && window.matchMedia('(pointer: coarse)').matches;

const fotoFeld = ref(null);
const videoFeld = ref(null);

function genommen(event) {
  const dateien = Array.from(event.target.files || []);
  // Sofort zurücksetzen: sonst löst dieselbe Aufnahme kein zweites Mal aus.
  event.target.value = null;
  if (dateien.length) emit('aufgenommen', dateien);
}

const knopf = 'flex items-center justify-center h-9 w-9 bg-slate-100 hover:bg-slate-200 dark:bg-slate-800 dark:hover:bg-slate-700 text-fliess rounded-xl transition-all cursor-pointer disabled:opacity-50';
</script>

<template>
  <template v-if="beruehrbar">
    <button type="button" :class="knopf" :disabled="disabled" :title="t('common.kameraFoto')" :aria-label="t('common.kameraFoto')" @click="fotoFeld.click()">
      <Camera class="w-4 h-4" />
    </button>
    <input ref="fotoFeld" type="file" accept="image/*" capture="environment" class="hidden" @change="genommen" />

    <template v-if="props.arten === 'foto-video'">
      <button type="button" :class="knopf" :disabled="disabled" :title="t('common.kameraVideo')" :aria-label="t('common.kameraVideo')" @click="videoFeld.click()">
        <Video class="w-4 h-4" />
      </button>
      <input ref="videoFeld" type="file" accept="video/*" capture="environment" class="hidden" @change="genommen" />
    </template>
  </template>
</template>
