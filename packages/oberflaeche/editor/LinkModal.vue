<script setup>
// Link einfügen oder bearbeiten.
//
// Ersetzt ein window.prompt, das drei Probleme hatte: Es war mit 'https://'
// vorbelegt (wer eine volle URL einfügte, bekam sie doppelt), es konnte nur
// die Adresse abfragen und keinen Anzeigetext, und auf dem Handy ist ein
// Browser-Prompt eine unschöne Unterbrechung.
import { ref, computed, onMounted, nextTick } from 'vue';
import { useI18n } from 'vue-i18n';
import BaseModal from '@oberflaeche/base/BaseModal.vue';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import { Link as LinkIcon } from 'lucide-vue-next';
import { normalizeLinkUrl } from '@oberflaeche/shared/linkUrl';

const props = defineProps({
  // Vorhandene Adresse, wenn der Cursor in einem Link steht.
  href: { type: String, default: '' },
  // Markierter Text – wird als Anzeigetext vorgeschlagen.
  text: { type: String, default: '' },
  // Steht der Cursor in einem bestehenden Link? Dann gibt es „Entfernen".
  editing: { type: Boolean, default: false },
});
const emit = defineEmits(['save', 'remove', 'close']);

const { t } = useI18n();

// BEWUSST leer und nicht mit 'https://' vorbelegt: Genau das führte zum
// doppelten Präfix, wenn jemand eine kopierte Adresse einfügte. Fehlt das
// Schema, ergänzt normalize() es beim Speichern.
const url = ref(props.href);
const label = ref(props.text);
const urlEl = ref(null);

onMounted(() => nextTick(() => urlEl.value?.focus()));

const canSave = computed(() => url.value.trim() !== '');

const save = () => {
  if (!canSave.value) return;
  emit('save', { href: normalizeLinkUrl(url.value), text: label.value.trim() });
};
</script>

<template>
  <BaseModal size="sm" @close="emit('close')">
    <template #header>
      <div class="flex items-center gap-2">
        <LinkIcon class="w-5 h-5 text-marke" />
        <h3 class="text-lg font-bold text-schrift">
          {{ editing ? t('editor.linkModal.titleEdit') : t('editor.linkModal.title') }}
        </h3>
      </div>
    </template>

    <div class="space-y-3">
      <div>
        <label for="link-url" class="block text-xs font-bold uppercase tracking-wider text-leise mb-1">
          {{ t('editor.linkModal.url') }}
        </label>
        <input id="link-url" ref="urlEl" v-model="url" type="text" inputmode="url"
          autocapitalize="off" autocorrect="off" spellcheck="false"
          :placeholder="t('editor.linkModal.urlPlaceholder')"
          @keydown.enter.prevent="save"
          class="w-full rounded-lg border border-linie bg-flaeche px-3 py-2 text-sm text-schrift focus:outline-none focus:ring-2 focus:ring-marke" />
      </div>

      <div>
        <label for="link-text" class="block text-xs font-bold uppercase tracking-wider text-leise mb-1">
          {{ t('editor.linkModal.text') }}
        </label>
        <input id="link-text" v-model="label" type="text"
          :placeholder="t('editor.linkModal.textPlaceholder')"
          @keydown.enter.prevent="save"
          class="w-full rounded-lg border border-linie bg-flaeche px-3 py-2 text-sm text-schrift focus:outline-none focus:ring-2 focus:ring-marke" />
        <p class="mt-1 text-xs text-leise">{{ t('editor.linkModal.textHint') }}</p>
      </div>
    </div>

    <div class="flex justify-between gap-2 pt-4">
      <BaseButton v-if="editing" variant="ghost" @click="emit('remove')">
        {{ t('editor.linkModal.remove') }}
      </BaseButton>
      <span v-else></span>
      <div class="flex gap-2">
        <BaseButton variant="ghost" @click="emit('close')">{{ t('common.cancel') }}</BaseButton>
        <BaseButton :disabled="!canSave" @click="save">
          {{ editing ? t('editor.linkModal.apply') : t('editor.linkModal.insert') }}
        </BaseButton>
      </div>
    </div>
  </BaseModal>
</template>
