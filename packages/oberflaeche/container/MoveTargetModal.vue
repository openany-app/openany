<script setup>
// Verschieben-Dialog für alle Module: oberste Ebene + eingerückter
// Behälter-Baum (options aus folderTreeOptions o. Ä.). Das aktuelle
// Ziel ist deaktiviert; die Auswahl meldet select(idOrNull).
import { Folder, FileText } from 'lucide-vue-next';
import BaseModal from '@oberflaeche/base/BaseModal.vue';

defineProps({
  title: { type: String, required: true },
  options: { type: Array, required: true },  // [{ id, name, depth }]
  currentId: { type: [Number, String], default: null }, // aktueller Behälter (null = oberste Ebene)
  topLabel: { type: String, required: true },
  emptyLabel: { type: String, default: '' },
});

defineEmits(['select', 'close']);
</script>

<template>
  <BaseModal :title="title" @close="$emit('close')">
    <div class="space-y-1">
      <button
        @click="$emit('select', null)"
        :disabled="currentId === null"
        class="w-full text-left px-3 py-2.5 rounded-xl hover:bg-auflage flex items-center gap-2 text-sm font-semibold text-fliess disabled:opacity-40 disabled:pointer-events-none"
      >
        <FileText class="w-4 h-4 text-slate-400" /> {{ topLabel }}
      </button>
      <button
        v-for="f in options"
        :key="f.id"
        @click="$emit('select', f.id)"
        :disabled="currentId === f.id"
        :style="{ paddingLeft: (12 + f.depth * 18) + 'px' }"
        class="w-full text-left px-3 py-2.5 rounded-xl hover:bg-auflage flex items-center gap-2 text-sm font-semibold text-fliess truncate disabled:opacity-40 disabled:pointer-events-none"
      >
        <Folder class="w-4 h-4 text-marke shrink-0" /> {{ f.name }}
      </button>
      <p v-if="options.length === 0 && emptyLabel" class="px-3 py-2 text-xs text-slate-400">{{ emptyLabel }}</p>
    </div>
  </BaseModal>
</template>
