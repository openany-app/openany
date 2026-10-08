import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { useToast } from '@oberflaeche/composables/useToast';
import { downloadBlob } from '@oberflaeche/shared/download';

/**
 * In Notizen eingefügte Bilder/PDFs und die Zitier-Bibliothek der Mappe.
 *
 * Der Notiztext enthält nur relative Pfade (`![](x.png)`); erst diese Karte
 * macht daraus Anzeige-URLs. Sie hängt an der MAPPE, nicht an der Notiz –
 * deshalb wird sie beim Mappenwechsel neu geladen und nicht bei jedem Öffnen.
 *
 * @param {object} o
 * @param {object} o.api        Datenquelle; baut die URLs je Kontext selbst
 *                              (persönlich vs. projekt-autorisierte Route)
 * @param {import('vue').Ref} o.editorRef  Editor, in den eingefügt wird
 * @param {() => Promise<void>} o.reloadAll  Neuaufbau nach Bibliotheks-Wechsel
 */
export function useNoteAssets({ api, editorRef, reloadAll }) {
  const { t } = useI18n();
  const toast = useToast();

  const noteAssets = ref(new Map()); // path -> { id, target_type }
  const noteCitations = ref([]);
  const noteContextFolderId = ref(null);
  const insertDialogOpen = ref(false);
  const libraryModal = ref(null); // { folder, candidates }

  const loadNoteContext = async (folderId) => {
    noteContextFolderId.value = folderId;
    noteAssets.value = new Map();
    noteCitations.value = [];
    try {
      const [assetsRes, citesRes] = await Promise.all([
        api.getNoteAssets(folderId), api.getNoteCitations(folderId),
      ]);
      noteAssets.value = new Map(assetsRes.data.items.map((a) => [a.path, a]));
      noteCitations.value = citesRes.data.items;
    } catch (e) {
      // Kontext ist optional – Editor funktioniert auch ohne (Pfade bleiben Text).
      console.error('Notiz-Kontext konnte nicht geladen werden', e);
    }
  };

  // Relativer Pfad -> authentifizierte Anzeige-URL (oder null = unaufgelöst).
  // Bilder werden als Galerie-Thumbnail eingebettet (?thumb=1): kleiner UND
  // robuste Auslieferung mit explizitem image/jpeg – die Original-Auslieferung
  // setzt keinen Content-Type, weshalb PNG teils nicht angezeigt wurde.
  const assetResolver = (path) => {
    const asset = noteAssets.value.get(path);
    if (!asset) return null;

    return api.assetUrl(asset, noteContextFolderId.value);
  };

  // Map neu setzen statt nur zu mutieren: Eine Map ist für Vue kein reaktiver
  // Container, ein set() allein löste kein Neuzeichnen aus.
  const remember = (path, asset) => {
    noteAssets.value.set(path, asset);
    noteAssets.value = new Map(noteAssets.value);
  };

  const uploadAsset = async (file) => {
    const res = await api.uploadNoteAsset(noteContextFolderId.value, file);
    remember(res.data.path, res.data);

    return res.data;
  };

  /**
   * Ergebnis aus dem Einfüge-Dialog (verknüpft oder hochgeladen) übernehmen.
   *
   * Der ganze Eintrag wird gemerkt, nicht nur drei Felder: Die App-Quelle
   * bringt den lokalen Pfad mit (`pfad`), aus dem `assetUrl` dort synchron
   * die Anzeige-Adresse baut. Ohne ihn stünde das eben eingefügte Bild bis
   * zum nächsten Mappenwechsel leer da.
   */
  const applyInsert = (asset) => {
    const { path, target_type, name } = asset;
    remember(path, asset);
    if (target_type === 'media') editorRef.value?.insertImageAsset(path);
    else editorRef.value?.insertFileAsset(path, name);
    insertDialogOpen.value = false;
  };

  const onPasteImage = async (file) => {
    try {
      const asset = await uploadAsset(file);
      editorRef.value?.insertImageAsset(asset.path);
    } catch (e) {
      toast.error(e.response?.data?.message || t('notes.insertFailed'));
    }
  };

  /**
   * Klick auf einen eingefügten PDF-Link: Anhang herunterladen/öffnen.
   *
   * Eine Quelle, die den Anhang selbst öffnen kann (die App: mit einem
   * anderen Programm auf dem Gerät), sagt das über `openNoteAsset`. Ein
   * Browser-Download liefe in der Webansicht auf Android ins Leere.
   */
  const onAssetOpen = async (path) => {
    const asset = noteAssets.value.get(path);
    if (!asset) return;
    try {
      if (api.openNoteAsset) {
        await api.openNoteAsset(asset, noteContextFolderId.value);
        return;
      }
      const res = await api.downloadNoteAsset(asset.id, noteContextFolderId.value);
      downloadBlob(new Blob([res.data]), path);
    } catch {
      toast.error(t('notes.loadFailed'));
    }
  };

  // --- Bibliothek (CSL-JSON) mit einer Mappe verknüpfen ---

  const openLibraryModal = async (folder) => {
    try {
      const res = await api.getNoteLibraryCandidates();
      libraryModal.value = { folder, candidates: res.data.items };
    } catch {
      toast.error(t('notes.libraryFailed'));
    }
  };

  const setLibrary = async (fileNodeId) => {
    try {
      await api.updateNoteFolder(libraryModal.value.folder.id, { library_file_node_id: fileNodeId });
      libraryModal.value = null;
      await reloadAll();
    } catch (e) {
      toast.error(e.response?.data?.message || t('notes.libraryFailed'));
    }
  };

  return {
    noteAssets,
    noteCitations,
    noteContextFolderId,
    insertDialogOpen,
    libraryModal,
    loadNoteContext,
    assetResolver,
    uploadAsset,
    applyInsert,
    onPasteImage,
    onAssetOpen,
    openLibraryModal,
    setLibrary,
  };
}
