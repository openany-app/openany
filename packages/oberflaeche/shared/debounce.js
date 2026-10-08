// Verzögert den Aufruf, bis `delay` ms lang kein neuer kam
// (z. B. Autosave, Suche-während-des-Tippens).
//
// `cancel()` verwirft einen ausstehenden Aufruf – nötig, wenn stattdessen
// sofort ausgeführt wird (Autosave beim Verlassen des Editors), damit der
// Timer nicht kurz darauf ein zweites Mal feuert.
export function debounce(fn, delay) {
  let t;
  const debounced = function (...args) {
    clearTimeout(t);
    t = setTimeout(() => fn.apply(this, args), delay);
  };
  debounced.cancel = () => clearTimeout(t);
  return debounced;
}
