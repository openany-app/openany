// Typst im Browser: Quelltext + Dateien -> PDF.
//
// WARUM HIER UND NICHT AUF DEM SERVER (seit Oktober 2026): dompdf rechnete
// jedes Notiz-PDF in einem PHP-Thread, ohne Silbentrennung und ohne die
// Bilder der Notiz. Typst setzt im Browser in einer Zehntelsekunde, mit
// deutscher Silbentrennung, Fußnoten und Tabellen -- und ist die Grundlage
// der geplanten Druckfassung.
//
// WARUM typst.ts UND NICHT typst-wasm: typst-wasm braucht gemeinsamen
// Speicher (SharedArrayBuffer, also eine „cross-origin isolated" Webapp)
// oder JSPI. Firefox auf Android hat beides nicht (am 13.10.2026 am Telefon
// gemessen), Safari kein JSPI. typst.ts läuft ohne.
//
// DER SATZ-KERN IST GROSS: 28 MB, gepackt etwa 11 MB. Er wird erst beim
// ersten PDF geladen -- dieses Modul kommt nur über einen dynamischen Import
// herein -- und liegt danach im Cache des Browsers (Vite hängt einen
// Inhalts-Hash an den Namen, Caddy liefert /assets/* `immutable`).
// Gemessen: erstes Laden am Telefon 4,8 s, jedes PDF danach 0,1 s.
//
// ALLES VON openany.de: Satz-Kern und Schriften liefert die Webapp selbst
// aus, ohne CDN.
//
// KEIN `loadFonts` VON typst.ts. Dessen Schriften-Lader baut in seiner
// ersten Zeile `Function('m', 'return import(m)')` -- ein Kniff, um unter
// Node Pakete nachzuladen, der aber auch im Browser läuft. Die CSP von
// openany.de verbietet das (kein 'unsafe-eval'), und der Start brach ab:
// am 13.10.2026 in Firefox am Mac und am Telefon, auf der Testseite ohne CSP
// nicht. Auch `disableDefaultFontAssets()` läuft über diesen Lader. Die CSP
// aufzuweichen hieße, die ganze Webapp schlechter zu schützen; deshalb steht
// hier ein eigener Lader, der die Schriften direkt übergibt.
import kernUrl from '@myriaddreamin/typst-ts-web-compiler/pkg/typst_ts_web_compiler_bg.wasm?url';
import serifRegular from '@typst-wasm/fonts/LibertinusSerif-Regular.otf?url';
import serifBold from '@typst-wasm/fonts/LibertinusSerif-Bold.otf?url';
import serifItalic from '@typst-wasm/fonts/LibertinusSerif-Italic.otf?url';
import serifBoldItalic from '@typst-wasm/fonts/LibertinusSerif-BoldItalic.otf?url';
import monoRegular from '@typst-wasm/fonts/DejaVuSansMono.ttf?url';
import monoBold from '@typst-wasm/fonts/DejaVuSansMono-Bold.ttf?url';
// Für die Druckfassung: TeX Gyre Termes (Fließtext, Times-artig, wie es
// viele Vorgaben verlangen) und TeX Gyre Heros (Überschriften, Helvetica-
// artig). Von CTAN, GUST-Font-Lizenz (LPPL 1.3c) -- liegt in schriften/.
import termesRegular from './schriften/texgyretermes-regular.otf?url';
import termesBold from './schriften/texgyretermes-bold.otf?url';
import termesItalic from './schriften/texgyretermes-italic.otf?url';
import termesBoldItalic from './schriften/texgyretermes-bolditalic.otf?url';
import herosRegular from './schriften/texgyreheros-regular.otf?url';
import herosBold from './schriften/texgyreheros-bold.otf?url';

const SCHRIFTEN = [
  serifRegular, serifBold, serifItalic, serifBoldItalic, monoRegular, monoBold,
  termesRegular, termesBold, termesItalic, termesBoldItalic, herosRegular, herosBold,
];

/**
 * Ein Schriften-Lader ohne `Function(…)` (siehe oben).
 *
 * DIE BEIDEN MARKEN SIND typst.ts-INTERNA (dist/esm/compiler.mjs, 0.7.0):
 * Ohne `_kind: 'fontLoader'` bricht `init` ab („no font loader found"),
 * ohne `_preloadRemoteFontOptions.assets === false` hängt es selbst einen
 * Lader für Standard-Schriften an -- und der liefe wieder über `Function`.
 * Deshalb steht typst.ts in package.json auf genau 0.7.0: Ein Update muss
 * diese Stelle bewusst nachprüfen.
 */
function eigeneSchriften(schriften) {
  const lader = async (_, { builder }) => {
    for (const schrift of schriften) await builder.add_raw_font(schrift);
  };
  lader._kind = 'fontLoader';
  lader._preloadRemoteFontOptions = { assets: false };
  return lader;
}

let bereit = null;
let reihe = Promise.resolve();

async function compilerHolen() {
  if (!bereit) {
    bereit = (async () => {
      const [{ createTypstCompiler }, ...schriften] = await Promise.all([
        import('@myriaddreamin/typst.ts'),
        ...SCHRIFTEN.map(async (url) => new Uint8Array(await (await fetch(url)).arrayBuffer())),
      ]);
      const compiler = createTypstCompiler();
      await compiler.init({
        getModule: () => kernUrl,
        beforeBuild: [eigeneSchriften(schriften)],
      });
      return compiler;
    })();
    // Scheitert das Laden (Netz weg), soll der nächste Versuch von vorn
    // beginnen und nicht ewig dasselbe gescheiterte Versprechen bekommen.
    bereit.catch(() => { bereit = null; });
  }
  return bereit;
}

/**
 * Typst-Quelltext setzen.
 *
 * @param {string} quelltext  der Inhalt von main.typ
 * @param {Map<string, Uint8Array>} [dateien]  weitere Dateien (Bilder) nach Namen
 * @returns {Promise<Uint8Array>} das PDF
 */
export function setzen(quelltext, dateien = new Map()) {
  // EINS NACH DEM ANDEREN: Der Compiler hat EIN virtuelles Projekt. Zwei
  // gleichzeitige Aufträge schrieben sich sonst gegenseitig main.typ um.
  const lauf = reihe.then(async () => {
    const compiler = await compilerHolen();
    for (const [name, bytes] of dateien) compiler.mapShadow(`/${name}`, bytes);
    compiler.addSource('/main.typ', quelltext);
    try {
      const ergebnis = await compiler.compile({ mainFilePath: '/main.typ', format: 1 /* pdf */ });
      if (!ergebnis.result) {
        const fehler = (ergebnis.diagnostics ?? []).map((d) => d.message ?? String(d)).join('\n');
        throw new Error(`Typst: ${fehler || 'unbekannter Fehler'}`);
      }
      return ergebnis.result;
    } finally {
      for (const name of dateien.keys()) compiler.unmapShadow(`/${name}`);
    }
  });
  reihe = lauf.catch(() => {});
  return lauf;
}
