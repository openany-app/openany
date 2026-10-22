import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import tailwindcss from '@tailwindcss/vite';
import path from 'path';
import { FREMDE_PAKETE } from '../packages/oberflaeche/fremde-pakete.js';
// Notiz-PDF und Druckfassung (seit 15.10.2026 auch hier): Satz-Kern ohne eval.
import { typstOhneEval, TYPST_NICHT_VORBUENDELN } from '../packages/oberflaeche/druck/vitePlugin.js';
// Whiteboards: die Schriften von Excalidraw aus dem Bündel, nicht aus dem Netz.
import { excalidrawSchriften } from '../packages/oberflaeche/projekte/whiteboard/vitePlugin.js';

// Der feste Port ist Absicht: tauri.conf.json nennt ihn, und ein Vite, das
// sich bei Belegung stillschweigend einen anderen sucht, ergäbe ein leeres
// Fenster ohne erkennbaren Grund. 5181, damit anytail-app (5180) daneben
// laufen kann.
//
// `@oberflaeche` ist dasselbe Paket wie in der Webapp (frontend/vite.config.js):
// Alias statt npm-Paket, `dedupe`, damit das Paket Vue von HIER bekommt, und
// `fs.allow` für den Entwicklungsserver. Siehe packages/oberflaeche/README.md.
export default defineConfig({
    plugins: [vue(), tailwindcss(), typstOhneEval, excalidrawSchriften()],
    clearScreen: false,
    optimizeDeps: {
        exclude: TYPST_NICHT_VORBUENDELN,
    },
    resolve: {
        alias: {
            '@oberflaeche': path.resolve(import.meta.dirname, '../packages/oberflaeche'),
        },
        dedupe: FREMDE_PAKETE,
    },
    server: {
        port: 5181,
        strictPort: true,
        fs: { allow: ['.', '../packages/oberflaeche'] },
    },
});
