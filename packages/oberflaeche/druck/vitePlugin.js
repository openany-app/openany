// Vite-Plugin: der Satz-Kern von typst.ts ohne eval (siehe ohneEval.js).
//
// Sein Verbindungscode baut über wasm-bindgen kleine Funktionen mit
// `new Function(…)`; die CSP von openany.de verbietet das, die der App
// ebenso (nur 'wasm-unsafe-eval'). Hier wird das beim Bauen gegen eine
// Nachbildung ohne eval getauscht. GENAU ZWEI STELLEN werden erwartet
// (typst.ts 0.7.0): Ändert ein Update das, bricht der Build ab, statt die
// Absicherung still zu verlieren.
//
// Webapp (frontend/) und App (app/) binden es beide ein, seit der Druck-Teil
// im Paket liegt (15.10.2026). Hier liegt es als reines Node-Modul: keine
// Abhängigkeit außer `path`.
import path from 'node:path';

export const typstOhneEval = {
  name: 'openany-typst-ohne-eval',
  enforce: 'pre',
  transform(code, id) {
    if (!id.replace(/\\/g, '/').includes('@myriaddreamin/typst-ts-web-compiler/pkg/typst_ts_web_compiler.mjs')) return null;
    const stellen = code.match(/new Function\(/g)?.length ?? 0;
    if (stellen !== 2) {
      this.error(`typst-ts-web-compiler: ${stellen} statt 2 Stellen mit \`new Function(\` -- ohneEval.js und dieses Plugin nachprüfen`);
    }
    const ersatz = JSON.stringify(path.resolve(import.meta.dirname, 'ohneEval.js'));
    return { code: `import { funktionOhneEval as __ohneEval } from ${ersatz};\n${code.replace(/new Function\(/g, '__ohneEval(')}`, map: null };
  },
};

// Der Satz-Kern darf nicht vorgebündelt werden: Sonst liefe der
// Entwicklungsserver an typstOhneEval vorbei.
export const TYPST_NICHT_VORBUENDELN = ['@myriaddreamin/typst.ts', '@myriaddreamin/typst-ts-web-compiler'];
