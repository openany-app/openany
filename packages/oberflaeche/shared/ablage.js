// PDFs aus Notizen in eine Akte unter Dokumente.
//
// Notiz-PDF -> Akte „Notizen", Druckfassung -> Akte „Druckfassungen". Jede
// Fassung bleibt: „Titel.pdf", „Titel (2).pdf" … Heruntergeladen wird nicht
// -- die Akte ist der Ort.
//
// Die Quelle bringt mit:
//   dokumentAblegen(akte, name, blob) -> {id, name, parent_id}
//   dokumentOeffnen(datei)            springt zur Akte und öffnet die Datei
//   ablageMitDownload                 true: zusätzlich herunterladen
// Fehlt `dokumentAblegen`, wird heruntergeladen.
import { downloadBlob } from './download';
import { useToast } from '../composables/useToast';
import { i18n } from '../i18n';

/**
 * @param {object} quelle  Notiz-Quelle (siehe oben)
 * @param {{akte: string, name: string, blob: Blob}} pdf
 */
export async function pdfAblegen(quelle, { akte, name, blob }) {
  if (!quelle.dokumentAblegen) {
    await downloadBlob(blob, name);
    return null;
  }
  const datei = await quelle.dokumentAblegen(akte, name, blob);
  const { t } = i18n.global;
  useToast().success(
    t('notes.ablage.gespeichert', { akte }),
    quelle.dokumentOeffnen ? { label: t('notes.ablage.oeffnen'), onClick: () => quelle.dokumentOeffnen(datei) } : null,
  );
  if (quelle.ablageMitDownload) await downloadBlob(blob, datei?.name ?? name);
  return datei;
}
