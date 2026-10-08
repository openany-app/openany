import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import tailwindcss from '@tailwindcss/vite';
import path from 'path';
import { FREMDE_PAKETE } from '../packages/oberflaeche/fremde-pakete.js';

// Der feste Port ist Absicht: tauri.conf.json nennt ihn, und ein Vite, das
// sich bei Belegung stillschweigend einen anderen sucht, ergäbe ein leeres
// Fenster ohne erkennbaren Grund. 5181, damit anytail-app (5180) daneben
// laufen kann.
//
// `@oberflaeche` ist dasselbe Paket wie in der Webapp (frontend/vite.config.js):
// Alias statt npm-Paket, `dedupe`, damit das Paket Vue von HIER bekommt, und
// `fs.allow` für den Entwicklungsserver. Siehe packages/oberflaeche/README.md.
export default defineConfig({
    plugins: [vue(), tailwindcss()],
    clearScreen: false,
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
