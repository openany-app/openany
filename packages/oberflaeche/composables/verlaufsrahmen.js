/*
 * Verlaufseinträge für Dialoge und Rahmen (BaseModal, useZurueckSchliesst):
 * Zurück schließt den obersten, nicht die Seite.
 *
 * JEDER EINTRAG TRÄGT DEN GANZEN STAPEL der offenen Rahmen. Wechselt ein
 * Dialog in einen anderen (Termin ansehen → bearbeiten), schließt der erste,
 * während der zweite schon seinen Eintrag gelegt hat -- der Eintrag des
 * ersten bleibt liegen. Schließt später der zweite, landet der Verlauf auf
 * diesem verwaisten Eintrag. Wer nur „bin ich der oberste?" fragte, hielte
 * das für Zurück und schlösse mit; wer im Stapel nachsieht, bleibt offen.
 *
 * WELCHE EINTRÄGE NOCH LEBEN, steht in `offene`: Wer Zurück auf einem
 * Eintrag landet, dessen Rahmen schon zu ist, springt darüber hinweg
 * (App.vue) -- aber nur dann. Ein Rahmen ist nicht immer ein Element mit
 * `role="dialog"`; daran gemessen, hielt die App den Kalender-Rahmen eines
 * Hubs für verwaist und schloss ihn mit.
 */

const offene = new Set();
// Einträge für Ordnerebenen: Zurück geht durch sie hindurch, sie zählen aber
// nicht als offener Rahmen -- der Neuaufbau nach einem Abgleich wartet nicht
// auf sie.
const ebenen = new Set();

/** Einen Eintrag für diesen Rahmen anlegen. */
export function eintragen(eintrag, { ebene = false } = {}) {
  offene.add(eintrag);
  if (ebene) ebenen.add(eintrag);
  const stapel = [...(history.state?.stapel ?? []), eintrag];
  history.pushState({ ...history.state, dialog: eintrag, stapel }, '');
}

/** Steht dieser Rahmen nach einem Zurück noch im Verlauf? */
export function nochOffen(eintrag) {
  return (history.state?.stapel ?? []).includes(eintrag);
}

/** Ist gerade ein Dialog oder Rahmen offen? */
export function rahmenOffen() {
  return offene.size > ebenen.size;
}

/** Ist der Rahmen zu diesem Eintrag noch offen? */
export function lebt(eintrag) {
  return offene.has(eintrag);
}

/** Abmelden, ohne den Verlauf anzufassen: Wer ihn abräumt, tut es selbst. */
export function abmelden(eintrag) {
  offene.delete(eintrag);
  ebenen.delete(eintrag);
}

/**
 * Beim Schließen: abmelden und den eigenen Eintrag abräumen, wenn er oben liegt.
 *
 * ERST NACH DEM LAUFENDEN RENDERN NACHSEHEN: Wechselt ein Dialog in einen
 * anderen, schließt der alte vor dem `onMounted` des neuen. Ein sofortiges
 * `history.back()` käme erst an, wenn der neue seinen Eintrag gelegt hat, und
 * schlösse ihn gleich wieder.
 */
export function austragen(eintrag) {
  abmelden(eintrag);
  queueMicrotask(() => {
    if (history.state?.dialog === eintrag) history.back();
  });
}
