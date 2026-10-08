import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { useToast } from '@oberflaeche/composables/useToast';
import { downloadBlob, sanitizeFilename } from '@oberflaeche/shared/download';

/**
 * Notiz als PDF/Markdown und ganze Mappe als ZIP herunterladen.
 *
 * Die laufenden Ids (`exportingPdfId`, `downloadingFolderId`) gehören dazu und
 * nicht in den allgemeinen Zustand der Arbeitsfläche: Sie sind ausschließlich
 * dafür da, den jeweiligen Knopf während des Downloads als beschäftigt zu
 * zeigen.
 *
 * @param {object} o
 * @param {object} o.api  Datenquelle (persönlich oder projekt-scoped)
 */
export function useNoteExport({ api }) {
  const { t } = useI18n();
  const toast = useToast();

  const exportingPdfId = ref(null);
  const downloadingFolderId = ref(null);

  const filename = (title, fallbackId, prefix, ext) =>
    sanitizeFilename(title, `${prefix}-${fallbackId}`) + ext;

  const exportNotePdf = async (note) => {
    exportingPdfId.value = note.id;
    try {
      const res = await api.exportNotePdf(note.id);
      downloadBlob(
        new Blob([res.data], { type: 'application/pdf' }),
        filename(note.title, note.id, 'notiz', '.pdf'),
      );
    } catch {
      toast.error(t('notes.pdfExportFailed'));
    } finally {
      exportingPdfId.value = null;
    }
  };

  const exportNoteMd = async (note) => {
    // Listen-Einträge kommen ohne content – für den Export nachladen.
    let content = note.content;
    if (content === undefined) {
      try {
        content = (await api.getNote(note.id)).data.content;
      } catch {
        toast.error(t('notes.loadFailed'));

        return;
      }
    }

    downloadBlob(
      new Blob([content || ''], { type: 'text/markdown;charset=utf-8' }),
      filename(note.title, note.id, 'notiz', '.md'),
    );
  };

  const downloadFolderZip = async (folder) => {
    downloadingFolderId.value = folder.id;
    try {
      const res = await api.exportNoteFolder(folder.id);
      downloadBlob(new Blob([res.data]), filename(folder.name, folder.id, 'mappe', '.zip'));
    } catch {
      toast.error(t('notes.folderDownloadFailed'));
    } finally {
      downloadingFolderId.value = null;
    }
  };

  return {
    exportingPdfId,
    downloadingFolderId,
    exportNotePdf,
    exportNoteMd,
    downloadFolderZip,
  };
}
