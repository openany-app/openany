<script setup>
// Automatische Übersicht („Index") aus den bereits geladenen Daten:
// links alle Titel alphabetisch, rechts ein Tag-Baum (verschachtelte Tags
// #a/b werden aufgefaltet). Klick öffnet die Notiz bzw. filtert nach dem Tag.
// Kein eigener Datenabruf – nutzt titles/tags aus der Notes-Ansicht.
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { FileText, Hash } from 'lucide-vue-next';

const { t } = useI18n();
const props = defineProps({
  titles: { type: Array, default: () => [] }, // [{ id, title }]
  tags: { type: Array, default: () => [] },   // [{ tag, count }]
});
const emit = defineEmits(['open', 'tag']);

const hasTitles = computed(() => props.titles.some((x) => (x.title || '').trim()));
const hasTags = computed(() => props.tags.length > 0);

// Titel A–Z, gruppiert nach Anfangsbuchstabe (Nicht-Buchstaben unter „#").
const titleGroups = computed(() => {
  const sorted = [...props.titles]
    .filter((x) => (x.title || '').trim())
    .sort((a, b) => (a.title || '').localeCompare(b.title || '', 'de', { sensitivity: 'base' }));
  const groups = new Map();
  for (const tt of sorted) {
    const c = (tt.title || '').trim().charAt(0).toUpperCase();
    const key = /\p{L}/u.test(c) ? c : '#';
    if (!groups.has(key)) groups.set(key, []);
    groups.get(key).push(tt);
  }
  return [...groups.entries()]
    .sort((a, b) => a[0].localeCompare(b[0], 'de'))
    .map(([letter, items]) => ({ letter, items }));
});

// Tag-Baum aus verschachtelten Tags bauen, dann flach mit Tiefe rendern.
const tagTree = computed(() => {
  const root = { children: new Map() };
  for (const { tag, count } of props.tags) {
    const parts = String(tag).split('/').filter(Boolean);
    let node = root;
    let path = '';
    parts.forEach((part, i) => {
      path = path ? `${path}/${part}` : part;
      if (!node.children.has(part)) {
        node.children.set(part, { name: part, path, count: 0, children: new Map() });
      }
      node = node.children.get(part);
      if (i === parts.length - 1) node.count = count;
    });
  }
  const out = [];
  const walk = (node, depth) => {
    const kids = [...node.children.values()].sort((a, b) => a.name.localeCompare(b.name, 'de'));
    for (const k of kids) {
      out.push({ name: k.name, path: k.path, count: k.count, depth });
      walk(k, depth + 1);
    }
  };
  walk(root, 0);
  return out;
});
</script>

<template>
  <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
    <!-- Titel-Index A–Z -->
    <div class="karte shadow-sm overflow-hidden">
      <div class="px-5 py-3 border-b border-linie flex items-center gap-2">
        <FileText class="w-4 h-4 text-marke shrink-0" />
        <h3 class="font-extrabold text-sm text-schrift">{{ t('notes.indexTitles') }}</h3>
      </div>
      <div v-if="hasTitles" class="max-h-[60vh] overflow-y-auto divide-y divide-slate-100 dark:divide-slate-800/50">
        <div v-for="g in titleGroups" :key="g.letter">
          <div class="px-5 py-1.5 bg-vertieft text-xs font-extrabold text-slate-400 uppercase tracking-wider sticky top-0 z-10">{{ g.letter }}</div>
          <button v-for="tt in g.items" :key="tt.id" @click="emit('open', tt.id)"
            class="w-full text-left px-5 py-2 text-sm font-semibold text-fliess hover:bg-slate-50 dark:hover:bg-slate-800/50 hover:text-marke truncate transition-colors cursor-pointer">
            {{ tt.title }}
          </button>
        </div>
      </div>
      <p v-else class="py-12 text-center text-sm text-slate-400">{{ t('notes.indexEmptyTitles') }}</p>
    </div>

    <!-- Tag-Baum -->
    <div class="karte shadow-sm overflow-hidden">
      <div class="px-5 py-3 border-b border-linie flex items-center gap-2">
        <Hash class="w-4 h-4 text-marke shrink-0" />
        <h3 class="font-extrabold text-sm text-schrift">{{ t('notes.indexTags') }}</h3>
      </div>
      <div v-if="hasTags" class="max-h-[60vh] overflow-y-auto py-2">
        <button v-for="node in tagTree" :key="node.path" @click="emit('tag', node.path)"
          class="w-full flex items-center gap-2 px-5 py-1.5 text-sm hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors group cursor-pointer"
          :style="`padding-left:${1.25 + node.depth}rem`">
          <Hash class="w-3.5 h-3.5 text-slate-400 shrink-0" />
          <span class="font-semibold text-fliess truncate group-hover:text-marke">{{ node.name }}</span>
          <span v-if="node.count" class="ml-auto text-xs font-bold text-slate-400 shrink-0">{{ node.count }}</span>
        </button>
      </div>
      <p v-else class="py-12 text-center text-sm text-slate-400">{{ t('notes.indexEmptyTags') }}</p>
    </div>
  </div>
</template>
