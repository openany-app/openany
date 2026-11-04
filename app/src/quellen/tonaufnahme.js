/*
 * Ton aufnehmen für die Mediathek -- je Plattform anders, nach außen gleich.
 *
 * ANDROID nimmt im Dienst auf (Tonaufnahme.kt), damit es bei ausgeschaltetem
 * Bildschirm weiterläuft und auch Systemaudio geht. Die Seite liest nur den
 * Stand; ist sie weg, läuft die Aufnahme trotzdem.
 *
 * WINDOWS UND MACOS nehmen in der Webansicht auf (`MediaRecorder`). Systemaudio
 * nur auf Windows: Dort bietet die Bildschirmfreigabe den Ton mit an, die
 * Webansicht auf macOS nicht.
 *
 * LINUX gar nicht: WebKitGTK gibt das Mikrofon der Seite nicht frei.
 *
 * `tonAufnehmer` gibt `null` zurück, wo es nicht geht -- dann fehlt der Knopf.
 *
 * JEDER `invoke` AUF EINER ZEILE — verdrahtung.rs liest genau diese Zeilen.
 */
import { invoke } from '@tauri-apps/api/core';

const warte = (ms) => new Promise((weiter) => setTimeout(weiter, ms));

function androidAufnehmer(bruecke) {
    const stand = () => JSON.parse(bruecke.stand());
    return {
        kannSystem: bruecke.kannSystem(),
        stand,
        starten: async (art) => bruecke.starten(art),
        pausieren: (an) => bruecke.pausieren(an),
        verwerfen: async () => bruecke.verwerfen(),
        async speichern(album, name) {
            bruecke.beenden();
            let s = stand();
            while (s.laeuft) {
                await warte(100);
                s = stand();
            }
            if (!s.fertig) throw new Error(s.fehler || 'technik');
            return invoke('galerie_aufnahme_ablegen', { album, name: `${name}.m4a`, dauer: s.dauer });
        },
    };
}

/** Der erste Typ, den diese Webansicht aufnehmen kann, samt Dateiendung. */
function aufnahmeFormat() {
    const typen = [['audio/mp4', 'm4a'], ['audio/webm;codecs=opus', 'webm'], ['audio/webm', 'webm']];
    return typen.find(([typ]) => MediaRecorder.isTypeSupported(typ)) ?? null;
}

function webansichtAufnehmer(quelle, kannSystem) {
    let aktiv = null;
    let fehler = null;

    const zeit = () => (aktiv ? aktiv.bisher + (aktiv.seit ? performance.now() - aktiv.seit : 0) : 0);
    const aufraeumen = () => {
        aktiv?.strom.getTracks().forEach((spur) => spur.stop());
        aktiv?.kontext.close().catch(() => {});
        aktiv = null;
    };

    async function strom(art) {
        if (art !== 'system') return navigator.mediaDevices.getUserMedia({ audio: true });
        const freigabe = await navigator.mediaDevices.getDisplayMedia({ video: true, audio: true });
        const ton = freigabe.getAudioTracks();
        freigabe.getVideoTracks().forEach((spur) => spur.stop());
        if (!ton.length) throw Object.assign(new Error('keinTon'), { name: 'KeinTon' });
        return new MediaStream(ton);
    }

    return {
        kannSystem,
        stand: () => {
            let pegel = 0;
            if (aktiv) {
                const werte = new Uint8Array(aktiv.pegelmesser.fftSize);
                aktiv.pegelmesser.getByteTimeDomainData(werte);
                pegel = Math.max(...werte.map((w) => Math.abs(w - 128))) / 128;
            }
            return { laeuft: Boolean(aktiv), pausiert: aktiv?.recorder.state === 'paused', art: aktiv?.art ?? 'mikrofon', ms: zeit(), pegel, fehler, fertig: false, dauer: null };
        },
        async starten(art) {
            fehler = null;
            try {
                const format = aufnahmeFormat();
                if (!format) throw new Error('technik');
                const s = await strom(art);
                const kontext = new AudioContext();
                const pegelmesser = kontext.createAnalyser();
                kontext.createMediaStreamSource(s).connect(pegelmesser);
                const recorder = new MediaRecorder(s, { mimeType: format[0] });
                const stuecke = [];
                recorder.ondataavailable = (e) => { if (e.data.size) stuecke.push(e.data); };
                recorder.start(1000);
                aktiv = { art, recorder, stuecke, strom: s, kontext, pegelmesser, endung: format[1], typ: format[0].split(';')[0], bisher: 0, seit: performance.now() };
            } catch (e) {
                if (e?.name === 'KeinTon') fehler = 'keinTon';
                else if (e?.name === 'NotAllowedError') fehler = art === 'system' ? 'abgebrochen' : 'recht';
                else fehler = 'technik';
            }
        },
        pausieren(an) {
            if (!aktiv || aktiv.recorder.state === 'inactive') return;
            if (an) {
                aktiv.recorder.pause();
                aktiv.bisher = zeit();
                aktiv.seit = 0;
            } else {
                aktiv.recorder.resume();
                aktiv.seit = performance.now();
            }
        },
        async verwerfen() {
            if (aktiv && aktiv.recorder.state !== 'inactive') aktiv.recorder.stop();
            aufraeumen();
        },
        async speichern(album, name) {
            const a = aktiv;
            const dauer = zeit() / 1000;
            if (a.recorder.state !== 'inactive') {
                await new Promise((fertig) => {
                    a.recorder.onstop = fertig;
                    a.recorder.stop();
                });
            }
            aufraeumen();
            const datei = new File(a.stuecke, `${name}.${a.endung}`, { type: a.typ });
            const { data } = await quelle.uploadMedia(album, datei);
            return invoke('galerie_ton_dauer', { id: data.id, dauer });
        },
    };
}

export function tonAufnehmer(quelle) {
    if (typeof window === 'undefined') return null;
    if (window.openanyTon) return androidAufnehmer(window.openanyTon);
    const ua = navigator.userAgent;
    if (!/Windows|Macintosh/.test(ua) || !navigator.mediaDevices?.getUserMedia || typeof MediaRecorder === 'undefined') return null;
    return webansichtAufnehmer(quelle, /Windows/.test(ua) && Boolean(navigator.mediaDevices.getDisplayMedia));
}
