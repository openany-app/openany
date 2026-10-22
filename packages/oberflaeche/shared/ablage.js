// PDFs aus Notizen in eine Akte unter Dokumente (Tiffy, 15.10.2026).
//
// Notiz-PDF -> Akte „Notizen", Druckfassung -> Akte „Druckfassungen". Jede
// Fassung bleibt: „Titel.pdf", „Titel (2).pdf" … Heruntergeladen wird nicht
// mehr, weder in der Webapp noch in der App -- die Akte ist der Ort (Tiffy).
//
// Die Quelle bringt mit:
//   dokumentAblegen(akte, name, blob) -> {id, name, parent_id}
//   dokumentOeffnen(datei)            springt zur Akte und öffnet die Datei
//   ablageMitDownload                 true: zusätzlich herunterladen
// Fehlt `dokumentAblegen`, bleibt es beim Herunterladen wie bisher.
import { downloadBlob } from './download';
import { useToast } from '../composables/useToast';
import { i18n } from '../i18n';

/** „Titel.pdf" -> der erste freie Name unter `vorhandene` („Titel (2).pdf" …). */
export function freierName(vorhandene, name) {
  const belegt = new Set([...vorhandene].map((n) => String(n).toLowerCase()));
  if (!belegt.has(name.toLowerCase())) return name;
  const punkt = name.lastIndexOf('.');
  const [stamm, endung] = punkt > 0 ? [name.slice(0, punkt), name.slice(punkt)] : [name, ''];
  for (let n = 2; n < 1000; n++) {
    const versuch = `${stamm} (${n})${endung}`;
    if (!belegt.has(versuch.toLowerCase())) return versuch;
  }
  return `${stamm} (${Date.now()})${endung}`;
}

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
