<script setup>
// Einheitliche Modal-Hülle: Overlay, Karte, Kopfzeile mit Schließen-Kreuz,
// Esc/Overlay-Klick schließen. Inhalt über den Default-Slot, Aktionen über
// den footer-Slot. Sichtbarkeit steuert der Aufrufer per v-if.
import { onMounted, onBeforeUnmount, inject } from 'vue';
import { useI18n } from 'vue-i18n';
import { X } from 'lucide-vue-next';

const { t } = useI18n();

defineProps({
  title: { type: String, default: '' },
  size: { type: String, default: 'md' }, // sm | md | lg
  // z. B. für Gefahren-Dialoge: Overlay-Klick schließt nicht.
  persistent: { type: Boolean, default: false },
});
const emit = defineEmits(['close']);

const SIZES = { sm: 'max-w-sm', md: 'max-w-md', lg: 'max-w-lg' };

const onKeydown = (e) => { if (e.key === 'Escape') emit('close'); };

/*
 * ZURÜCK SCHLIESST DEN DIALOG – im Programm.
 *
 * Auf Android ist die Zurück-Taste der Weg, einen Dialog loszuwerden. Ohne
 * eigenen Verlaufseintrag verließ sie am 15.09.2026 auf dem Tablet gleich die
 * ganze Seite. Deshalb legt ein offener Dialog einen Eintrag an; Zurück nimmt
 * ihn weg und schließt. Wird der Dialog anders geschlossen (Kreuz, Speichern),
 * räumt er seinen Eintrag selbst wieder ab.
 *
 * NUR, WO DIE ANWENDUNG ES WILL (`oberflaeche:zurueck-schliesst-dialog`). Die
 * Webapp hat einen Router, der den Verlauf für sich beansprucht; ein fremder
 * Eintrag darin wäre ein Navigationsschritt, den er nicht kennt.
 */
const zurueckSchliesst = inject('oberflaeche:zurueck-schliesst-dialog', false);
const eintrag = `dialog-${Math.random().toString(36).slice(2)}`;
const onPopstate = () => { if (history.state?.dialog !== eintrag) emit('close'); };

// Ein Feld bekommt den Fokus, die Tastatur fährt hoch: es danach in den
// sichtbaren Teil holen. Ohne Verzögerung stünde es dort, wo vorher Platz war.
const insBild = (e) => {
  const feld = e.target;
  if (!feld?.matches?.('input, textarea, select')) return;
  setTimeout(() => feld.scrollIntoView?.({ block: 'nearest' }), 350);
};

onMounted(() => {
  window.addEventListener('keydown', onKeydown);
  if (zurueckSchliesst) {
    history.pushState({ ...history.state, dialog: eintrag }, '');
    window.addEventListener('popstate', onPopstate);
  }
});
onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeydown);
  if (zurueckSchliesst) {
    window.removeEventListener('popstate', onPopstate);
    if (history.state?.dialog === eintrag) history.back();
  }
});
</script>

<template>
  <!-- `ueber-tastatur` (stil.css): In der App endet der Dialog über der
       Bildschirmtastatur. Kopf und Fuß stehen, nur der Inhalt scrollt --
       sonst verschwand „Übernehmen" hinter der Tastatur (15.10.2026, Tablet,
       Quellen-Dialog). Im Browser ist die Tastaturhöhe 0. -->
  <div class="fixed inset-0 bg-slate-900/60 backdrop-blur-sm flex items-center justify-center p-4 z-[100] ueber-tastatur"
    @click.self="persistent ? null : emit('close')">
    <div class="karte w-full shadow-2xl max-h-[90vh] flex flex-col overflow-hidden"
      :class="SIZES[size] ?? SIZES.md" role="dialog" aria-modal="true" @focusin="insBild">
      <div v-if="title || $slots.header" class="flex items-center justify-between px-6 py-4 border-b border-linie shrink-0">
        <slot name="header">
          <h3 class="font-extrabold text-schrift">{{ title }}</h3>
        </slot>
        <button @click="emit('close')" class="p-1.5 text-slate-400 hover:bg-auflage rounded-xl cursor-pointer" :aria-label="t('common.close')">
          <X class="w-5 h-5" />
        </button>
      </div>
      <div class="p-6 overflow-y-auto min-h-0 flex-1">
        <slot />
      </div>
      <div v-if="$slots.footer" class="flex justify-end gap-2 px-6 py-4 border-t border-linie shrink-0">
        <slot name="footer" />
      </div>
    </div>
  </div>
</template>
