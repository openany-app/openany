// Vite-Plugin: die Schriften von Excalidraw liegen in der App, nichts kommt
// aus dem Netz (docs/plan-whiteboard.md).
//
// Excalidraw lädt seine Schriften von `window.EXCALIDRAW_ASSET_PATH` und
// sonst von esm.sh -- das verbietet die CSP der App ohnehin. Hier werden sie
// aus node_modules unter EXCALIDRAW_PFAD bereitgestellt: beim Entwickeln
// durchgereicht, beim Bauen ins Bündel gelegt.
//
// Ohne Xiaolai (chinesische Handschrift, 12 MB): Wer Chinesisch schreibt,
// bekommt die Ersatzschrift des Systems.
import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs';
import path from 'node:path';

export const EXCALIDRAW_PFAD = '/excalidraw/';
const WEGLASSEN = ['Xiaolai'];

function dateien(ordner, vor = '') {
  return readdirSync(ordner).flatMap((name) => {
    const rel = vor ? `${vor}/${name}` : name;
    if (statSync(path.join(ordner, name)).isDirectory()) {
      return WEGLASSEN.includes(rel) ? [] : dateien(path.join(ordner, name), rel);
    }
    return [rel];
  });
}

export function excalidrawSchriften() {
  let quelle = null;
  return {
    name: 'openany-excalidraw-schriften',
    configResolved(config) {
      quelle = path.join(config.root, 'node_modules/@excalidraw/excalidraw/dist/prod/fonts');
    },
    configureServer(server) {
      server.middlewares.use(`${EXCALIDRAW_PFAD}fonts`, (req, res, next) => {
        const rel = decodeURIComponent((req.url ?? '').split('?')[0]).replace(/^\/+/, '');
        const datei = path.join(quelle, rel);
        if (rel.includes('..') || !existsSync(datei) || statSync(datei).isDirectory()) return next();
        res.setHeader('Content-Type', 'font/woff2');
        res.end(readFileSync(datei));
      });
    },
    generateBundle() {
      if (!existsSync(quelle)) this.error(`Excalidraw-Schriften fehlen: ${quelle}`);
      for (const rel of dateien(quelle)) {
        this.emitFile({ type: 'asset', fileName: `excalidraw/fonts/${rel}`, source: readFileSync(path.join(quelle, rel)) });
      }
    },
  };
}

// „Bibliothek" heißt im Board „Elemente" -- wie der Reiter im Hub. Excalidraw
// bringt den Text in seinen Sprachdateien mit und lässt ihn sich nicht
// übergeben; hier wird beim Bauen genau dieser eine Schlüssel ersetzt.
// Englisch steckt nicht in einer Sprachdatei, sondern in einem Chunk.
const ELEMENTE = {
  Library: 'Items',
  Bibliothek: 'Elemente',
  Biblioteca: 'Elementos',
  'Biblioth\\xE8que': '\\xC9l\\xE9ments',
  Biblioteka: 'Elementy',
  // Ukrainisch steht in der Sprachdatei als \\u-Folge.
  '\\u0411\\u0456\\u0431\\u043B\\u0456\\u043E\\u0442\\u0435\\u043A\\u0430': '\\u0415\\u043B\\u0435\\u043C\\u0435\\u043D\\u0442\\u0438',
};

export function excalidrawElemente() {
  return {
    name: 'openany-excalidraw-elemente',
    transform(code, id) {
      if (!/@excalidraw[\\/]excalidraw[\\/]dist[\\/]/.test(id) || !code.includes('library:"')) return null;
      const neu = code.replace(/([,{]library:)"([^"]*)"/, (ganz, vor, alt) => (alt in ELEMENTE ? `${vor}"${ELEMENTE[alt]}"` : ganz));
      return neu === code ? null : { code: neu, map: null };
    },
  };
}
