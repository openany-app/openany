// `new Function(…)` ohne eval -- für den Satz-Kern von typst.ts.
//
// ANLASS (13.10.2026): Der Rust-Teil von typst.ts lässt sich über
// wasm-bindgen beim Start kleine Funktionen aus Text bauen. Die CSP von
// openany.de verbietet das (kein 'unsafe-eval'), und das Notiz-PDF brach in
// Firefox am Mac und am Telefon mit „Ausführung eines JavaScript-Evals
// blockiert" ab. Die CSP für die ganze Webapp aufzuweichen kam nicht in Frage.
//
// WAS DER KERN VERLANGT, sind ausschließlich Platzhalter -- unter
// nachgespielter CSP vollständig mitgeschnitten, bis ein PDF stand:
//
//   Function("return 0")                     Konstanten
//   Function("return true")
//   Function("path", "return path")          gibt sein Argument zurück
//   Function("throw new Error('Dummy …')")   wirft einen festen Fehler
//
// Genau diese Formen werden hier OHNE eval nachgebildet. Alles andere wirft
// einen EvalError -- dasselbe, was die CSP täte. Das Vite-Plugin in
// vite.config.js setzt diese Funktion beim Bauen an die Stelle von
// `new Function(` im Verbindungscode des Kerns.

const KONSTANTEN = { true: true, false: false, null: null, undefined };
const BEZEICHNER = /^[A-Za-z_$][\w$]*$/;

/** Ein Funktionsrumpf -> Nachbildung, oder null. `param`: der eine Parametername, falls es einen gibt. */
function rumpfNachbilden(rumpf, param = null) {
  const wurf = /^\s*throw\s+new\s+Error\((['"])([^'"\\]*)\1\)\s*;?\s*$/.exec(rumpf);
  if (wurf) {
    const text = wurf[2];
    return () => { throw new Error(text); };
  }

  const rueckgabe = /^\s*return\s+(-?\d+(?:\.\d+)?|true|false|null|undefined|this|[A-Za-z_$][\w$]*)\s*;?\s*$/.exec(rumpf);
  if (!rueckgabe) return null;
  const wert = rueckgabe[1];

  if (wert === 'this') return function () { return globalThis; };
  if (wert in KONSTANTEN) return () => KONSTANTEN[wert];
  if (/^-?\d/.test(wert)) {
    const zahl = Number(wert);
    return () => zahl;
  }
  // Ein Bezeichner: nur der eigene Parameter (Identität), sonst nicht.
  return param !== null && wert === param ? (x) => x : null;
}

/**
 * Ersatz für `new Function(...args)`.
 * @throws {EvalError} für alles, was keine der bekannten Formen ist
 */
export function funktionOhneEval(...args) {
  let f = null;
  if (args.length === 1) {
    f = rumpfNachbilden(String(args[0]));
  } else if (args.length === 2) {
    const param = String(args[0]).trim();
    if (BEZEICHNER.test(param)) f = rumpfNachbilden(String(args[1]), param);
  }
  if (f) return f;
  throw new EvalError(`Ohne eval nicht nachzubilden (CSP): Function(${args.map((a) => JSON.stringify(String(a))).join(', ')})`);
}
