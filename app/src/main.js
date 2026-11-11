import { createApp, defineAsyncComponent } from 'vue';
import App from './App.vue';
import './stil.css';
import '@oberflaeche/formen.css';
import { i18n, setLocale, detectInitialLocale } from '@oberflaeche/i18n';
import { openUrl } from '@tauri-apps/plugin-opener';
import { raenderBeobachten } from './raender';
import { projektUmgebungSetzen } from '@oberflaeche/projekte/umgebung';
import { projektApi } from './quellen/projektApi';
import { downloadsAufsGeraet } from './aufsGeraet';

// Die Projekt-Ansichten der Webapp (packages/oberflaeche/projekte) laufen
// auch hier -- ihr `api` beantwortet die App selbst aus dem lokalen Speicher
// (quellen/projektApi.js). Kein Live-Kanal, keine KI: Das gibt es nur am
// Server.
projektUmgebungSetzen({
    api: projektApi,
    // Eine Karte im Whiteboard öffnet ihren Inhalt über dem Whiteboard. Erst
    // beim Öffnen geladen: Sie bringt Notiz-Editor und PDF-Betrachter mit.
    WHITEBOARD_VERWEIS: defineAsyncComponent(() => import('./ansichten/projekt/WhiteboardVerweis.vue')),
    // Gemeinsam zeichnen vor Ort: die Sitzung eines Whiteboards (Yjs, erst
    // beim ersten Board geladen).
    whiteboardSitzung: async ({ projekt, uuid, ausgangstext }) => {
        const { sitzungOeffnen } = await import('./sitzung');
        return sitzungOeffnen({ projekt, id: `whiteboard:${uuid}`, ausgangstext });
    },
});

raenderBeobachten();
// Downloads der Oberfläche (PDF, Markdown) aufs Gerät statt ins Leere.
downloadsAufsGeraet();

const app = createApp(App);
app.use(i18n);
// Ohne Router gehört der Verlauf dem Programm: Zurück darf einen offenen
// Dialog schließen (BaseModal im Paket).
app.provide('oberflaeche:zurueck-schliesst-dialog', true);
// Links aus einem Dokument (PDF-Betrachter) gehen an den Browser des Systems;
// die Webansicht selbst soll keine fremden Seiten laden.
app.provide('oberflaeche:url-oeffnen', (url) => openUrl(url).catch(() => {}));

// Wie in der Webapp: erst die Sprache, dann aufhängen — sonst blitzt beim
// Start kurz Deutsch auf. Die Sprachdateien liegen im Bündel des Programms,
// nachgeladen wird also nichts über das Netz.
setLocale(detectInitialLocale(), { persist: false }).finally(() => app.mount('#app'));
