import { useI18n } from 'vue-i18n';
import { useToast } from './useToast';
import { formatBytes } from '../shared/format';

/*
 * Viele Dateien übernehmen (docs/plan-viele-dateien.md, D2): wählen lassen
 * und einreihen. Die Quelle (`dataSource.uebernahme`) gibt es nur, wo die
 * Anwendung eine Schlange hat — sonst bleibt das Auswahlfenster der
 * Oberfläche, und `uebernehmen` meldet `false`.
 *
 * Den Lauf zeigt `UebernahmeZeile.vue`, das Ablegen `AblageRahmen.vue`.
 */

/** Ein Grund aus der Schlange (Kürzel) als Satz; Unbekanntes wie es kam. */
export function grundText(t, te, grund) {
  const schluessel = `common.uebernahme.gruende.${grund}`;
  return te(schluessel) ? t(schluessel) : grund;
}

export function useUebernahme(uebernahme) {
  const { t } = useI18n();
  const toast = useToast();

  async function auftrag(geben) {
    try {
      const e = await geben();
      if (e?.bekannt) toast.info(t('common.uebernahme.bekannt', { n: e.bekannt }, e.bekannt));
    } catch (err) {
      const text = typeof err === 'string' ? err : String(err?.message ?? err);
      const platz = /^kein_platz:(\d+):(\d+)$/.exec(text);
      toast.error(platz
        ? t('common.uebernahme.keinPlatzVorab', { gebraucht: formatBytes(Number(platz[1])), frei: formatBytes(Number(platz[2])) })
        : text);
    }
  }

  /** `true`, wenn die Schlange den Auftrag hat (oder gewählt wurde, nichts). */
  async function uebernehmen(zone, ziel = null) {
    if (!uebernahme?.waehlen) return false;
    await auftrag(() => uebernahme.waehlen(zone, ziel));
    return true;
  }

  /** Das eben Abgelegte (AblageRahmen.vue) nach `zone`/`ziel`. */
  const ablegen = (zone, ziel = null) => auftrag(() => uebernahme.ablegen(zone, ziel));

  return { uebernehmen, ablegen, auftrag };
}
