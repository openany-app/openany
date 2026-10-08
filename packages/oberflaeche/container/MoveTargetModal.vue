<script setup>
// Verschieben-Dialog für alle Module: oberste Ebene + eingerückter
// Behälter-Baum (options aus folderTreeOptions o. Ä.). Das aktuelle
// Ziel ist deaktiviert; die Auswahl meldet select(idOrNull).
import { X, Folder, FolderInput, FileText } from 'lucide-vue-next';

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
  <div class="fixed inset-0 bg-slate-900/60 backdrop-blur-sm flex items-center justify-center p-4 z-[100]" @click.self="$emit('close')">
    <div class="bg-flaeche w-full max-w-md rounded-xl shadow-2xl p-6 space-y-4 border border-linie">
      <div class="flex items-center justify-between border-b border-linie pb-4">
        <div class="flex items-center gap-3">
          <FolderInput class="w-6 h-6 text-marke" />
          <h3 class="font-extrabold text-xl text-schrift truncate">{{ title }}</h3>
        </div>
        <button @click="$emit('close')" class="p-1.5 text-slate-400 hover:bg-auflage rounded-xl transition-colors shrink-0">
          <X class="w-5 h-5" />
        </button>
      </div>
      <div class="max-h-72 overflow-y-auto space-y-1">
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
    </div>
  </div>
</template>
