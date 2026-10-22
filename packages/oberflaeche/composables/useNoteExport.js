import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { useToast } from '@oberflaeche/composables/useToast';
import { downloadBlob, sanitizeFilename } from '@oberflaeche/shared/download';
import { pdfAblegen } from '@oberflaeche/shared/ablage';

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
      // In die Akte „Notizen" unter Dokumente (shared/ablage.js); die
      // Webapp lädt zusätzlich herunter.
      await pdfAblegen(api, {
        akte: 'Notizen',
        name: filename(note.title, note.id, 'notiz', '.pdf'),
        blob: new Blob([res.data], { type: 'application/pdf' }),
      });
    } catch (e) {
      // Der Grund in die Konsole: Die Meldung allein sagt nicht, woran es lag
      // (am 13.10.2026 war es die CSP, und das stand nur dort).
      console.error('PDF-Export fehlgeschlagen', e);
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

    try {
      await downloadBlob(
        new Blob([content || ''], { type: 'text/markdown;charset=utf-8' }),
        filename(note.title, note.id, 'notiz', '.md'),
      );
    } catch (e) {
      // In der App legt die Ablage die Datei selbst ab -- und kann scheitern.
      console.error('Markdown-Export fehlgeschlagen', e);
      toast.error(t('notes.loadFailed'));
    }
  };

  const downloadFolderZip = async (folder) => {
    downloadingFolderId.value = folder.id;
    try {
      // Das ZIP baut ein Auftrag auf dem Server; die Quelle lädt herunter,
      // sobald es fertig ist. Bei großen Mappen kann das dauern.
      toast.info(t('common.zip.vorbereiten'));
      await api.exportNoteFolder(folder.id, filename(folder.name, folder.id, 'mappe', '.zip'));
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
