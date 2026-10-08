/*
 * Nachrichten — die Wege der App (src-tauri/src/nachrichtenbefehle.rs);
 * „Vor Ort" (`nah`) geht ohne Server direkt zum Gerät (direktbefehle.rs).
 *
 * `matrix` geht von DIESEM Gerät aus, Ende-zu-Ende; `openany` läuft über den
 * Server, wie in der Webapp. Der Weg steht immer dabei und wird nie geraten:
 * Ein openany-Konto darf „@tiffy:matrix.org" heissen.
 *
 * Jeder `invoke` steht auf einer Zeile mit einfachen Schlüsseln: Der Test
 * src-tauri/tests/verdrahtung.rs liest genau diese Zeilen.
 */
import { invoke } from '@tauri-apps/api/core';
import { einsortieren } from './ablage';
import { i18n } from '@oberflaeche/i18n';

/*
 * WARUM DER SATZ HIER ENTSTEHT UND NICHT IN DER FLÄCHE: Dass nur der
 * Matrix-Teil dasteht, gibt es so allein im Programm — drüben ist der Server
 * die einzige Quelle, und fällt er aus, gibt es gar keine Liste. Die Fläche
 * zeigt `serverfehler` nur an; was er bedeutet, weiss dieser Rahmen.
 */
const nurMatrix = (antwort) => ({
    ...antwort,
    serverfehler: antwort.serverfehler ? i18n.global.t('app.nachrichten.nurHier', { fehler: antwort.serverfehler }) : null,
});

/*
 * ANHÄNGE (docs/plan-email-pgp.md, Schritt 1) -- nur über Matrix. Die Datei
 * geht in EINEM Stück als Base64 über die Brücke: Die Grenze liegt bei 10 MB,
 * und ein Uint8Array kam auf Android nicht als Bytes an (quellen/dateien.js).
 */
async function alsBase64(datei) {
    const bytes = new Uint8Array(await datei.arrayBuffer());
    let binaer = '';
    for (let i = 0; i < bytes.length; i += 0x8000) {
        binaer += String.fromCharCode.apply(null, bytes.subarray(i, i + 0x8000));
    }
    return btoa(binaer);
}

async function mitAnhang(ziel, text, datei, von) {
    const daten = await alsBase64(datei);
    return invoke('nachricht_anhang_senden', { ziel, name: datei.name, mime: datei.type || '', daten, text, von });
}

/**
 * Die Bytes eines Anhangs: aus der eigenen Ablage oder vom Homeserver. Der
 * Befehl schickt Base64 (auf Android kämen Bytes sonst als Zahlenliste an,
 * siehe nachrichtenbefehle.rs); hier wird daraus wieder ein Uint8Array.
 */
async function anhangBytes(zeile, index = 0) {
    const text = atob(await invoke('nachricht_anhang', { id: zeile.id, weg: zeile.transport, index }));
    const bytes = new Uint8Array(text.length);
    for (let i = 0; i < text.length; i += 1) bytes[i] = text.charCodeAt(i);
    return bytes;
}

/**
 * „Speichern" im Programm: in Dateien, oberste Ebene. Ein Download-Ordner
 * wäre ausserhalb von openany -- und ohne Abgleich.
 */
/*
 * ANHÄNGE EINSORTIEREN (Tiffy, 30.09.2026). „Speichern" legt einen Anhang
 * dorthin, wo man ihn sucht: Bilder und Videos in das Galerie-Album
 * „Anhänge", PDFs und Office-Dateien nach Dokumente/Anhänge, alles andere
 * nach Dateien/Anhänge. Album und Ordner entstehen beim ersten Mal.
 *
 * NUR AUF WUNSCH, NIE VON SELBST: Was dort liegt, geht in den Abgleich und
 * zählt gegen das Kontingent -- und ein verschlüsselter Anhang läge danach
 * unverschlüsselt auf dem Server. Deshalb fragt die Fläche bei einem
 * verschlüsselten vorher, und „Aufs Gerät" (Download-Ordner, ohne Abgleich)
 * steht daneben.
 */
/** Speichern und einsortieren (quellen/ablage.js). Gibt zurück, wo er jetzt liegt. */
async function anhangSpeichern(zeile, index = 0) {
    const bytes = await anhangBytes(zeile, index);
    const a = zeile.anhaenge?.[index] ?? zeile.anhang;
    return einsortieren(new File([bytes], a.name, { type: a.mime || '' }), 'Anhänge');
}

/** Aufs Gerät: in den Ordner „Download", ohne Abgleich (MainActivity.kt). */
async function anhangAufsGeraet(zeile, index = 0) {
    const a = zeile.anhaenge?.[index] ?? zeile.anhang;
    const b64 = await invoke('nachricht_anhang', { id: zeile.id, weg: zeile.transport, index });
    if (window.openanyAblage?.inDownloadsBase64) {
        const fehler = window.openanyAblage.inDownloadsBase64(a.name, a.mime || 'application/octet-stream', b64);
        if (fehler) throw new Error(fehler);
        return `Download/${a.name}`;
    }
    // Ohne die Brücke (Schreibtisch): ein gewöhnlicher Download.
    const text = atob(b64);
    const bytes = new Uint8Array(text.length);
    for (let i = 0; i < text.length; i += 1) bytes[i] = text.charCodeAt(i);
    const url = URL.createObjectURL(new Blob([bytes], { type: a.mime || '' }));
    const link = document.createElement('a');
    link.href = url;
    link.download = a.name;
    link.click();
    setTimeout(() => URL.revokeObjectURL(url), 60_000);
    return a.name;
}

/*
 * E-MAIL (docs/plan-email-pgp.md, Schritt 2): Das Gerät spricht selbst mit
 * dem Mailserver (src-tauri/src/mailbefehle.rs). Anhänge wie bei Matrix als
 * Base64, aber mehrere und bis 15 MB zusammen.
 */
async function mailSenden(an, text, dateien, extra = {}) {
    const anhaenge = [];
    for (const d of dateien) anhaenge.push({ name: d.name, mime: d.type || '', daten: await alsBase64(d) });
    return invoke('mail_senden', { an, betreff: extra.betreff ?? '', text, anhaenge, antwortAuf: extra.antwortAuf ?? null, von: extra.von ?? null, verschluesseln: Boolean(extra.verschluesseln) });
}

export const mailQuelle = {
    lage: () => invoke('mail_lage'),
    serverFinden: (adresse) => invoke('mail_server_finden', { adresse }),
    verbinden: (konto) => invoke('mail_verbinden', { konto }),
    trennen: (adresse) => invoke('mail_trennen', { adresse }),
    standard: (adresse) => invoke('mail_standard', { adresse }),
    // Ohne Adresse: alle Postfächer.
    abholen: (adresse = null) => invoke('mail_abholen', { adresse }),
    // Spamverdacht: frisch vom Server, nicht im Verlauf.
    spam: (adresse) => invoke('mail_spam', { adresse }),
    keinSpam: (adresse, uid) => invoke('mail_kein_spam', { adresse, uid }),
    spamLoeschen: (adresse, uid) => invoke('mail_spam_loeschen', { adresse, uid }),
    // OpenPGP (Schritt 3a): der eigene Schlüssel je Postfach ...
    pgpErzeugen: (adresse) => invoke('mail_pgp_erzeugen', { adresse }),
    pgpEinlesen: (adresse, daten, passphrase) => invoke('mail_pgp_einlesen', { adresse, daten, passphrase }),
    pgpEntfernen: (adresse) => invoke('mail_pgp_entfernen', { adresse }),
    pgpAusfuhr: (adresse, passphrase) => invoke('mail_pgp_ausfuhr', { adresse, passphrase }),
    pgpOeffentlich: (adresse) => invoke('mail_pgp_oeffentlich', { adresse }),
    // ... und die öffentlichen der Gegenüber, nur auf diesem Gerät.
    pgpListe: () => invoke('mail_pgp_liste'),
    pgpSuchen: (adresse, schluesselserver = false) => invoke('mail_pgp_suchen', { adresse, schluesselserver }),
    pgpHand: (daten) => invoke('mail_pgp_hand', { daten }),
    pgpVergessen: (adresse) => invoke('mail_pgp_vergessen', { adresse }),
    pgpWechselGesehen: (adresse) => invoke('mail_pgp_wechsel_gesehen', { adresse }),
};

export const nachrichtenQuelle = {
    lage: () => invoke('nachrichten_lage'),
    anmelden: (homeserver, benutzer, passwort) => invoke('nachrichten_anmelden', { homeserver, benutzer, passwort }),
    abmelden: (mxid) => invoke('nachrichten_abmelden', { mxid }),
    // Welches Matrix-Konto neue Nachrichten schickt (das erste der Liste).
    standard: (mxid) => invoke('nachrichten_standard', { mxid }),
    // Filter und Suche (Tiffy, 01.10.2026): `{ wege, ungelesen, anhang, q, mit }`.
    liste: async (seite, filter) => nurMatrix(await invoke('nachrichten_liste', { seite, filter })),
    // „Nach Kontakt": je Gegenüber eine Zeile, über alle drei Wege.
    unterhaltungen: async (filter) => nurMatrix(await invoke('nachrichten_unterhaltungen', { filter })),
    senden: (ziel, text, weg, datei, extra) => {
        if (weg === 'email') return mailSenden(ziel, text, datei ? [datei] : [], extra);
        // `von`: das eigene Konto, bei Matrix; ohne das Standard-Konto.
        const von = extra?.von ?? null;
        return datei && weg === 'matrix'
            ? mitAnhang(ziel, text, datei, von)
            : invoke('nachricht_senden', { ziel, text, weg, von });
    },
    gelesen: (id, weg) => invoke('nachricht_gelesen', { id, weg }),
    loeschen: (id, weg, auchServer = false) => invoke('nachricht_loeschen', { id, weg, auchServer }),
    anhangGrenze: (weg) => (weg === 'email' ? Promise.resolve(15 * 1024 * 1024) : invoke('nachrichten_anhang_grenze')),
    // Beim Öffnen des Verlaufs gleich nach neuen Mails sehen.
    abholen: () => invoke('mail_abholen', { adresse: null }),
    // Vor Ort, ohne Server (02.10.2026): wen man anschreiben kann, und blockieren.
    nahZiele: () => invoke('nah_nachricht_ziele'),
    blockieren: (person) => invoke('nah_blockieren', { person }),
    // Anfragen und Bestätigen vor Ort (06.10.2026, direktbefehle.rs).
    kontaktBestaetigen: (ziel) => invoke('nah_kontakt_bestaetigen', { ziel }),
    anfragen: () => invoke('nah_anfragen'),
    anfragenErlauben: (an) => invoke('nah_anfragen_erlauben', { an }),
    // OpenPGP im Schreibfeld: eigener Schlüssel da, Empfänger bekannt?
    pgpStatus: (von, an) => invoke('mail_pgp_status', { von, an }),
    anhang: anhangBytes,
    anhangSpeichern,
    anhangAufsGeraet,
};
