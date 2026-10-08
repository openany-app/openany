import { ref } from 'vue';
import { useToast } from '@oberflaeche/composables/useToast';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import { i18n } from '@oberflaeche/i18n';

const t = i18n.global.t;

/**
 * Papierkorb-Explorer: hält die paginierte Liste der soft-gelöschten
 * Elemente und kapselt Wiederherstellen sowie endgültiges Löschen (inkl.
 * Bestätigung und Fehler-Toasts). Genutzt vom zentralen Papierkorb
 * (Trash.vue) mit der gemischten Liste aller Inhaltstypen.
 *
 * @param {object}   cfg
 * @param {(page:number)=>Promise<{items:Array, next_page:number|null}>} cfg.fetchPage
 *        Lädt eine Papierkorb-Seite und normalisiert die Antwort.
 * @param {(item:object)=>Promise<any>} cfg.restore     Element wiederherstellen.
 * @param {(item:object)=>Promise<any>} cfg.forceDelete Element endgültig löschen.
 * @param {(item:object)=>string}     cfg.label         Anzeigename (für die Abfrage).
 * @param {(item:object)=>void}      [cfg.onRemoved]    Nach restore/forceDelete
 *        (z. B. einen offenen Editor schließen).
 */
export function useTrash({ fetchPage, restore, forceDelete, label, onRemoved }) {
  const toast = useToast();
  const { confirmDelete } = useConfirm();

  const items = ref([]);
  const nextPage = ref(null);
  const isLoading = ref(false);
  const loadingMore = ref(false);

  const load = async ({ reset = true } = {}) => {
    const page = reset ? 1 : nextPage.value;
    if (page == null) return;
    if (reset) isLoading.value = true;
    try {
      const { items: pageItems, next_page } = await fetchPage(page);
      items.value = reset ? pageItems : [...items.value, ...pageItems];
      nextPage.value = next_page;
    } catch (e) {
      console.error('Failed to load trash', e);
    } finally {
      isLoading.value = false;
    }
  };

  const loadMore = async () => {
    loadingMore.value = true;
    try { await load({ reset: false }); } finally { loadingMore.value = false; }
  };

  // Typ + Id vergleichen: in der gemischten Liste kollidieren Ids
  // verschiedener Tabellen (Notiz #7 vs. Projekt #7).
  const drop = (item) => {
    items.value = items.value.filter((i) => !(i.id === item.id && i.type === item.type));
    onRemoved?.(item);
  };

  const restoreItem = async (item) => {
    try {
      await restore(item);
      drop(item);
    } catch (err) {
      toast.error(err.response?.data?.message || t('common.restoreFailed'));
    }
  };

  const forceDeleteItem = async (item) => {
    if (!(await confirmDelete(t('common.confirmForceDelete', { name: label(item) })))) return;
    try {
      await forceDelete(item);
      drop(item);
    } catch (err) {
      toast.error(t('common.deleteFailed'));
    }
  };

  return { items, nextPage, isLoading, loadingMore, load, loadMore, restoreItem, forceDeleteItem };
}
