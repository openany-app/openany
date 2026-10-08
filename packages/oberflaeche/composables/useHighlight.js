import { nextTick, ref, watch } from 'vue';

/**
 * Einen angesprungenen Eintrag hervorheben und ins Bild rollen.
 *
 * Gebraucht überall dort, wo ein [[Verweis]] aus dem Chat nicht in eine
 * eigene Ansicht führt, sondern mitten in eine Liste: eine Karte in einem
 * Board, ein Meilenstein auf der Zeitachse, ein Ort im Raster, eine Datei in
 * einem Ordner. Ohne das Rollen stünde das Gemeinte oft unterhalb des
 * sichtbaren Bereichs, und der Sprung sähe aus wie nichts.
 *
 * Bewusst kein ref für die Elemente: Sie werden nur zum Rollen gebraucht,
 * gerendert wird nichts davon – reaktiv gemacht lösten sie nur Neuzeichnen aus.
 *
 * @param {() => (number|string|null)} getId  hervorzuhebende Id (null = keine)
 * @param {() => number} getCount  Anzahl der Einträge; die Liste steht beim
 *   Aufbau oft noch nicht, deshalb wird auf sie mitgewartet.
 */
export function useHighlight(getId, getCount) {
  const highlighted = ref(null);
  const elemente = {};

  const merkeEintrag = (id, el) => {
    if (el) elemente[id] = el;
    else delete elemente[id];
  };

  watch(
    () => [getId(), getCount()],
    async () => {
      const id = getId();
      highlighted.value = id ?? null;
      if (id == null) return;
      await nextTick();
      elemente[id]?.scrollIntoView({ block: 'center', behavior: 'smooth' });
    },
    { immediate: true },
  );

  return { highlighted, merkeEintrag };
}
