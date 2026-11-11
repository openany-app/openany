<script setup>
/*
 * Die Aktionen einer Listenzeile auf schmalen Bildschirmen: ein Knopf, der sie
 * als Liste öffnet. Dort steht in der Spalte das Datum, für mehrere Knöpfe
 * daneben ist kein Platz. Eine Datei ohne eigene Ansicht bliebe sonst ohne Weg
 * zu Verschieben und Löschen.
 *
 * Aktionen wie bei ActionButtons. Klicks bleiben hier: Sonst öffnete der Tipp
 * zugleich die Zeile.
 */
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { MoreVertical } from 'lucide-vue-next';
import BaseModal from '../base/BaseModal.vue';

defineProps({
  actions: { type: Array, required: true },
  titel: { type: String, default: '' },
});

const { t } = useI18n();
const offen = ref(false);

function waehlen(aktion) {
  offen.value = false;
  aktion.onClick();
}
</script>

<template>
  <div @click.stop>
    <button
      type="button"
      class="p-1.5 -mr-1 text-slate-400 dark:text-slate-500 rounded-lg cursor-pointer"
      :title="t('common.aktionen')"
      :aria-label="t('common.aktionen')"
      aria-haspopup="dialog"
      @click="offen = true"
    >
      <MoreVertical class="w-4 h-4" />
    </button>
    <BaseModal v-if="offen" :title="titel" size="sm" @close="offen = false">
      <div class="-mx-2 -my-2">
        <button
          v-for="a in actions"
          :key="a.key"
          type="button"
          :disabled="a.disabled || a.loading"
          class="w-full text-left px-3 py-3 rounded-xl flex items-center gap-3 text-sm font-semibold disabled:opacity-50 cursor-pointer"
          :class="a.danger ? 'text-rose-600' : 'text-fliess'"
          @click="waehlen(a)"
        >
          <component :is="a.icon" class="w-5 h-5 shrink-0" :class="a.danger ? '' : 'text-slate-400'" />
          <span>{{ a.title }}</span>
        </button>
      </div>
    </BaseModal>
  </div>
</template>
