<script setup>
// Zentraler Papierkorb (/papierkorb): EINE gemischte, chronologische Liste
// über alle Inhaltstypen (Notizen, Dateien, Dokumente, Alben, Bilder,
// Projekte, Planungs-Container, Kalender, Termine). Erreichbar über
// Profil-Dropdown bzw. Burgermenü – die Module selbst haben keine eigenen
// Papierkorb-Ansichten mehr.
//
// SEIT DEM 15.09.2026 IM GEMEINSAMEN PAKET, mit einer Datenquelle
// (getTrash, restoreTrashItem, forceDeleteTrashItem). Die Webapp reicht ihren
// api-Service hinein, das Programm seine lokale SQLite – dort mit den Arten,
// die es trägt (Notizen, Kalender, Termine, Kontakte).
import { onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import {
  Trash2, RotateCcw, StickyNote, File, FileText, Images, Image as ImageIcon,
  Briefcase, SquareKanban, Vote, Milestone, MapPin, Calendar as CalendarIcon,
  CalendarClock, Contact as ContactIcon,
} from 'lucide-vue-next';
import { useTrash } from '@oberflaeche/composables/useTrash';
import { formatDate } from '@oberflaeche/shared/date';
import ModulePage from '@oberflaeche/base/ModulePage.vue';

const props = defineProps({
  dataSource: { type: Object, required: true },
});
const api = props.dataSource;

const { t } = useI18n();

const TYPE_ICONS = {
  note: StickyNote,
  file: File,
  document: FileText,
  album: Images,
  album_image: ImageIcon,
  gallery_image: ImageIcon,
  project: Briefcase,
  board: SquareKanban,
  poll: Vote,
  roadmap: Milestone,
  place_group: MapPin,
  calendar: CalendarIcon,
  event: CalendarClock,
  contact: ContactIcon,
};

const {
  items,
  nextPage,
  isLoading,
  loadingMore,
  load,
  loadMore,
  restoreItem,
  forceDeleteItem,
} = useTrash({
  fetchPage: (page) => api.getTrash(page).then((r) => ({ items: r.data.items, next_page: r.data.next_page })),
  restore: (item) => api.restoreTrashItem(item.type, item.id),
  forceDelete: (item) => api.forceDeleteTrashItem(item.type, item.id),
  label: (item) => item.name || t('trash.unnamed'),
});

// Typ-Label + optionaler Kontext (z. B. Projekt-/Album-/Kalendername) in
// einer Zeile: "Kanban-Board · Umzug".
const contextLine = (item) => {
  const type = t(`trash.types.${item.type}`);
  const context = item.type === 'gallery_image' ? t('trash.contextGallery') : item.context;
  return context ? `${type} · ${context}` : type;
};


onMounted(load);
</script>

<template>
  <ModulePage>
    <div class="karte shadow-sm overflow-hidden transition-colors duration-300">
      <div class="flex items-center gap-3 px-3 py-2.5 sm:px-6 sm:py-4 border-b border-linie bg-rose-50/50 dark:bg-rose-900/10">
        <Trash2 class="w-5 h-5 text-rose-500" />
        <div>
          <h3 class="font-extrabold text-schrift">{{ t('common.trash') }}</h3>
          <p class="text-xs text-leise font-medium">{{ t('trash.hint') }}</p>
        </div>
      </div>

      <div v-if="isLoading" class="flex justify-center py-20">
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-marke"></div>
      </div>

      <div v-else-if="items.length > 0" class="divide-y divide-slate-100 dark:divide-slate-800/50">
        <div
          v-for="item in items"
          :key="item.type + '-' + item.id"
          class="flex items-center gap-3 sm:gap-4 px-3 py-2.5 sm:px-6 sm:py-4 hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors"
        >
          <component :is="TYPE_ICONS[item.type] || File" class="w-6 h-6 shrink-0 text-slate-400 dark:text-slate-500" />
          <div class="flex-1 min-w-0">
            <div class="text-sm font-extrabold text-schrift truncate">{{ item.name || t('trash.unnamed') }}</div>
            <div class="text-xs text-leise font-medium truncate">
              {{ contextLine(item) }} · {{ t('trash.deletedOn', { date: formatDate(item.deleted_at) }) }}
            </div>
          </div>
          <div class="flex items-center gap-2 shrink-0">
            <button
              @click="restoreItem(item)"
              class="flex items-center gap-1.5 px-3 py-1.5 text-xs font-bold text-emerald-600 dark:text-emerald-400 bg-emerald-50 dark:bg-emerald-900/20 hover:bg-emerald-100 dark:hover:bg-emerald-900/40 rounded-xl transition-colors cursor-pointer"
              :title="t('common.restore')"
            >
              <RotateCcw class="w-3.5 h-3.5" /> <span class="hidden sm:inline">{{ t('common.restore') }}</span>
            </button>
            <button
              @click="forceDeleteItem(item)"
              class="flex items-center gap-1.5 px-3 py-1.5 text-xs font-bold text-rose-600 dark:text-rose-400 bg-rose-50 dark:bg-rose-900/20 hover:bg-rose-100 dark:hover:bg-rose-900/40 rounded-xl transition-colors cursor-pointer"
              :title="t('common.reallyDeleteTitle')"
            >
              <Trash2 class="w-3.5 h-3.5" /> <span class="hidden sm:inline">{{ t('common.forceDelete') }}</span>
            </button>
          </div>
        </div>
        <div v-if="nextPage" class="p-4 text-center">
          <button
            @click="loadMore"
            :disabled="loadingMore"
            class="px-5 py-2 text-sm font-bold text-marke hover:bg-marke-leise rounded-xl disabled:opacity-50 cursor-pointer"
          >{{ t('common.more') }}</button>
        </div>
      </div>

      <div v-else class="py-20 text-center space-y-4 max-w-sm mx-auto">
        <div class="w-16 h-16 rounded-full bg-slate-50 dark:bg-slate-800 text-slate-400 flex items-center justify-center mx-auto">
          <Trash2 class="w-8 h-8" />
        </div>
        <h4 class="text-lg font-extrabold text-schrift">{{ t('common.trashEmptyTitle') }}</h4>
        <p class="text-sm text-leise">{{ t('trash.emptyHint') }}</p>
      </div>
    </div>
  </ModulePage>
</template>
