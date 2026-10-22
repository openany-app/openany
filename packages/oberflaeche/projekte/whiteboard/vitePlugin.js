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
