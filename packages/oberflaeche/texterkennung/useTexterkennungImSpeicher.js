/*
 * Die Texterkennung in der Dateifläche -- für Webapp UND Programm.
 *
 * BIS ZUM 26.09.2026 STAND DAS NUR IN DER WEBAPP (`FileExplorer.vue`). Die
 * Dateifläche selbst war seit dem 15.09. gemeinsam, ihre Erweiterungen nicht:
 * Im Programm gab es deshalb weder „Text erkennen" noch Fotos, die in einer
 * Akte als durchsuchbares PDF ankommen. Niemand hatte es entfernt -- es war
 * nur nie mitgewandert.
 *
 * Zwei Wege, beide nur in Akten (zone `documents`):
 *
 *   1. Foto → durchsuchbares PDF beim Hochladen (useDocumentIntake)
 *   2. „Text erkennen" auf einem schon abgelegten PDF (useDocumentOcr)
 *
 * Die Hülle hängt `erweiterung` in ihre `erweiterungen` und zeigt
 * `<TexterkennungDialoge :texterkennung="…" />`. Was je Rahmen verschieden
 * ist, kommt herein: die Datenquelle (API oder lokale Ablage) und die
 * Sprache der Oberfläche.
 */
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { ScanText } from 'lucide-vue-next';
import { useToast } from '@oberflaeche/composables/useToast';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import { ocrMoeglich, sprachenFuer } from './ocr/recognize';
import { useDocumentIntake } from './useDocumentIntake';
import { useDocumentOcr, MAX_SEITEN } from './useDocumentOcr';

/**
 * @param {object} quelle  Die Datenquelle der Dateifläche. Gebraucht werden
 *   `uploadFile`, `downloadFileContent` und `replaceFileContent`.
 * @param {string} zone    'files' | 'documents' -- nur in Akten wird erkannt.
 */
export function useTexterkennungImSpeicher(quelle, zone) {
  const { t, locale } = useI18n();
  const toast = useToast();
  const { confirmDialog } = useConfirm();

  const inAkten = zone === 'documents';
  const L = (key, params) => t(`${inAkten ? 'documents' : 'files'}.${key}`, params ?? {});
  const sprachen = () => sprachenFuer(locale.value);

  // --- Foto → durchsuchbares PDF ---
  const intake = useDocumentIntake(quelle, { sprachen });
  const frage = ref(0); // Anzahl Fotos in der Nachfrage, 0 = keine
  const vorschlag = ref('');
  const wartendeBilder = ref([]);
  let kontextJetzt = null;
  let fotosDanach = [];

  /**
   * Kann dieses Gerät überhaupt umwandeln? Ohne Canvas und createImageBitmap
   * gibt es keinen Weg -- dann ist eine ehrliche Absage besser als ein Foto,
   * das anschließend abgelehnt wird.
   */
  const umwandlungMoeglich = () => typeof createImageBitmap === 'function'
    && typeof HTMLCanvasElement !== 'undefined';

  async function umwandeln(bilder, { zusammen, name = '' }) {
    frage.value = 0;
    wartendeBilder.value = [];
    const kontext = kontextJetzt;
    try {
      const { abgelegt, ohneText } = await intake.verarbeite(bilder, kontext.ordnerId, { zusammen, name });
      if (!abgelegt) return; // abgebrochen
      kontext.neuLaden();
      if (ohneText) toast.error(L('convertDoneNoText'));
      else toast.success(L('convertDone'));
    } catch (err) {
      console.error('Foto-Umwandlung fehlgeschlagen:', err);
      kontext.zeigeFehler(err?.response?.data?.message || L('convertFailed'));
    }
  }

  function umwandlungAbbrechen() {
    frage.value = 0;
    wartendeBilder.value = [];
    intake.abbrechen();
  }

  // --- „Text erkennen" auf einem PDF ---
  const ocr = useDocumentOcr(quelle, { sprachen });

  async function erkennen(item, kontext) {
    if (!ocrMoeglich()) {
      toast.error(L('ocrUnsupported'));
      return;
    }
    const bestaetigt = await confirmDialog(L('ocrConfirm', { name: item.name }), {
      title: L('ocrConfirmTitle'),
      confirmLabel: L('ocrStart'),
    });
    if (!bestaetigt) return;

    try {
      const { status, woerter, seiten } = await ocr.erkenne(item);
      if (status === 'fertig' || status === 'aufbereitet') {
        kontext.neuLaden();
        toast.success(L(status === 'aufbereitet' ? 'ocrDoneEnhanced' : 'ocrDone', { woerter }));
      } else if (status === 'schon-text') {
        toast.error(L('ocrAlreadyText'));
      } else if (status === 'kein-text') {
        toast.error(L('ocrNoText'));
      } else if (status === 'zu-viele') {
        toast.error(L('ocrTooManyPages', { seiten, max: MAX_SEITEN }));
      }
    } catch (err) {
      console.error('Texterkennung fehlgeschlagen:', err);
      toast.error(err?.response?.data?.message || String(err?.message ?? '') || L('ocrFailed'));
    }
  }

  /** Die Stücke für `erweiterungen` der Dateifläche. */
  const erweiterung = {
    /** In einer Akte gehen Fotos einen eigenen Weg; alles andere unverändert. */
    hochladenVorbereiten(files) {
      const bilder = inAkten ? files.filter(intake.istBild) : [];
      if (!bilder.length) return files;
      // Erst der Rest, dann die Fotos (nachHochladen).
      fotosDanach = bilder;
      return files.filter((f) => !bilder.includes(f));
    },

    async nachHochladen(kontext) {
      const bilder = fotosDanach;
      fotosDanach = [];
      if (!bilder.length) return;

      kontextJetzt = kontext;
      if (!umwandlungMoeglich()) {
        kontext.zeigeFehler(L('convertUnsupported'));
        return;
      }
      if (bilder.length === 1) {
        await umwandeln(bilder, { zusammen: false });
        return;
      }
      // Mehrere Fotos: erst fragen, ob daraus ein mehrseitiges Dokument
      // werden soll. Ein dreiseitiger Brief und drei Belege sehen von hier
      // aus gleich aus.
      wartendeBilder.value = bilder;
      vorschlag.value = intake.ohneEndung(bilder[0].name);
      frage.value = bilder.length;
    },

    /** Die Zeilen-Aktion „Text erkennen" -- oder keine. */
    aktionen(item, kontext) {
      if (!inAkten || !ocr.istPdf(item)) return [];
      return [{ key: 'ocr', icon: ScanText, title: L('ocrAction'), onClick: () => erkennen(item, kontext) }];
    },
  };

  return {
    erweiterung,
    // Für TexterkennungDialoge.vue
    frage,
    vorschlag,
    intakeStand: intake.stand,
    ocrStand: ocr.stand,
    einzeln: () => umwandeln(wartendeBilder.value, { zusammen: false }),
    zusammen: (name) => umwandeln(wartendeBilder.value, { zusammen: true, name }),
    umwandlungAbbrechen,
    ocrAbbrechen: ocr.abbrechen,
  };
}
