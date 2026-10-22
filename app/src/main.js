import { createApp } from 'vue';
import App from './App.vue';
import './stil.css';
import './formen.css';
import { i18n, setLocale, detectInitialLocale } from '@oberflaeche/i18n';
import { openUrl } from '@tauri-apps/plugin-opener';
import { raenderBeobachten } from './raender';
import { projektUmgebungSetzen } from '@oberflaeche/projekte/umgebung';
import { projektApi } from './quellen/projektApi';
import WhiteboardVerweis from './ansichten/projekt/WhiteboardVerweis.vue';
import { sitzungOeffnen } from './sitzung';
import { downloadsAufsGeraet } from './aufsGeraet';

// Die Projekt-Ansichten der Webapp (packages/oberflaeche/projekte) laufen
// auch hier -- ihr `api` beantwortet die App selbst aus dem lokalen Speicher
// (quellen/projektApi.js). Kein Live-Kanal, keine KI: Das gibt es nur am
// Server.
projektUmgebungSetzen({
    api: projektApi,
    // Was die App (noch) nicht kann, zeigt sie nicht an (local-first).
    kann: { abstimmungenVerwalten: true, schulPaket: false, planAbo: false, ferienImport: false, heft: false, zuweisen: false },
    // Eine Karte im Whiteboard öffnet ihren Inhalt über dem Whiteboard.
    WHITEBOARD_VERWEIS: WhiteboardVerweis,
    // Gemeinsam zeichnen vor Ort (W3b): die Sitzung eines Whiteboards.
    whiteboardSitzung: ({ projekt, uuid, ausgangstext }) => sitzungOeffnen({ projekt, id: `whiteboard:${uuid}`, ausgangstext }),
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
