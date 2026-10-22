/*
 * Sucht in einer Supportanfrage nach Mustern, die auf personenbezogene Daten
 * hindeuten (docs/plan-support.md, Tiffy 22.10.2026): E-Mail-Adressen,
 * Telefonnummern, IP-Adressen und Stellen, die einen Namen nennen.
 *
 * Nur ein Netz, keine Gewissheit: Es fängt das Übliche („Mein Name ist …“,
 * ein Gruß mit Namen darunter, eine Nummer mit Vorwahl) und sperrt damit das
 * Senden, bis die Stelle fort ist. Was es nicht kennt, kann durchrutschen --
 * deshalb steht über dem Formular auch der Hinweis.
 *
 * Läuft nur auf dem Gerät. Der Server sieht die Anfrage ohnehin nicht: Sie
 * geht verschlüsselt an die Verwalter.
 */

const GROSS = '[A-ZÄÖÜÀ-ÝĀ-ŽА-ЯІЇЄҐ][\\p{L}\'’-]+';
// Ein bis drei großgeschriebene Wörter -- ein Vor- und Nachname, mehr nicht.
const NAME = `${GROSS}(?:[ \\t]+${GROSS}){0,2}`;

/*
 * Einleitungen, nach denen ein Name folgt -- in den sieben Sprachen der App.
 * „Ich bin …“ steht bewusst NICHT darin: „Ich bin Lehrer“, „ich bin
 * verwirrt“ wären lauter Fehlalarme.
 */
const EINLEITUNGEN = [
    'ich hei(?:ß|ss)e', 'mein name ist', 'mein name lautet', 'hier (?:ist|schreibt)',
    'my name is', "i'm called", 'this is',
    "je m'appelle", 'je m’appelle', 'mon nom est',
    'me llamo', 'mi nombre es', 'soy',
    'chamo-me', 'o meu nome é', 'meu nome é', 'eu sou',
    'nazywam się', 'mam na imię', 'tu (?:pisze|jest)',
    'мене звати', 'моє ім’я', "моє ім'я",
];

// Grußformeln, unter oder hinter denen oft ein Name steht.
const GRUESSE = [
    'viele grüße', 'liebe grüße', 'beste grüße', 'freundliche grüße', 'mit freundlichen grüßen',
    'grüße', 'gruß', 'lg', 'vg', 'mfg',
    'best regards', 'kind regards', 'regards', 'best', 'cheers', 'thanks', 'thank you',
    'cordialement', 'bien à vous', 'merci',
    'saludos', 'un saludo', 'atentamente', 'gracias',
    'cumprimentos', 'atenciosamente', 'obrigad[oa]',
    'pozdrawiam', 'z poważaniem', 'dziękuję',
    'з повагою', 'дякую', 'вітаю',
];

const MUSTER = [
    {
        art: 'email',
        re: /[\p{L}0-9._%+-]+@[\p{L}0-9.-]+\.[\p{L}]{2,}/giu,
    },
    {
        art: 'ip',
        re: /(?<![\d.])(?:(?:25[0-5]|2[0-4]\d|1?\d?\d)\.){3}(?:25[0-5]|2[0-4]\d|1?\d?\d)(?![\d.])/gu,
    },
    {
        // IPv6: Gruppen aus Hex-Ziffern mit Doppelpunkten. Mit `::` oder ab
        // fünf Gruppen -- sonst wäre jede Uhrzeit „10:30:15“ ein Treffer.
        art: 'ip',
        re: /(?<![\p{L}\d:])[0-9a-f]{0,4}(?::[0-9a-f]{0,4}){2,7}(?![\p{L}\d:])/giu,
        nur: (t) => t.includes('::') || t.split(':').length >= 5,
    },
    {
        art: 'telefon',
        // Beginnt mit + , 00 oder 0, dann Ziffern mit Leerzeichen, /, - oder
        // Klammern -- mindestens sieben Ziffern. Punkte zählen nicht als
        // Trenner: „01.02.2026“ ist ein Datum, keine Nummer.
        re: /(?<![\p{L}\d])(?:\+|00|0)[\d][\d ()/-]{5,}\d(?![\p{L}\d])/gu,
        nur: (t) => (t.match(/\d/g) || []).length >= 7,
    },
    {
        art: 'name',
        re: new RegExp(`(?:${EINLEITUNGEN.join('|')})[ \\t:,]+(${NAME})`, 'giu'),
        gruppe: 1,
    },
    {
        art: 'name',
        re: /(?:^|\n)[ \t]*name[ \t]*:[ \t]*(\S[^\n]*)/giu,
        gruppe: 1,
    },
    {
        // Ein Gruß und darunter (oder dahinter) eine Zeile, die nur aus einem
        // bis drei großgeschriebenen Wörtern besteht -- die Unterschrift.
        art: 'name',
        re: new RegExp(`(?:^|\\n)[ \\t]*(?:${GRUESSE.join('|')})[ \\t]*[,!.]?[ \\t]*(?:\\n[ \\t]*)?(${NAME})[ \\t]*[.!]?[ \\t]*(?=\\n|$)`, 'giu'),
        gruppe: 1,
    },
];

/**
 * Die Fundstellen in `text`.
 *
 * @param {string} text
 * @returns {{art: 'email'|'telefon'|'ip'|'name', treffer: string, von: number, bis: number}[]}
 */
export function personenbezug(text) {
    const funde = [];
    if (!text) return funde;

    for (const m of MUSTER) {
        m.re.lastIndex = 0;
        for (const treffer of text.matchAll(m.re)) {
            const teil = m.gruppe ? treffer[m.gruppe] : treffer[0];
            if (!teil || (m.nur && !m.nur(teil))) continue;
            const von = treffer.index + treffer[0].lastIndexOf(teil);
            funde.push({ art: m.art, treffer: teil.trim(), von, bis: von + teil.length });
        }
    }

    // Dieselbe Stelle nur einmal, in Lesereihenfolge.
    funde.sort((a, b) => a.von - b.von);
    return funde.filter((f, i) => i === 0 || f.von >= funde[i - 1].bis);
}
