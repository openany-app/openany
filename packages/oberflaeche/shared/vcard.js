/*
 * Adressbücher tauschen – vCard lesen und schreiben.
 *
 * SEIT OKTOBER 2026 HIER UND NICHT MEHR AUF DEM SERVER (docs/plan-adressbuch-e2e.md):
 * Der Server liest das Adressbuch nicht mehr, also kann er es auch nicht
 * mehr ein- oder ausgeben. Die Regeln sind dieselben wie vorher in
 * `VCard.php`:
 *
 * GESCHRIEBEN WIRD 3.0, GELESEN WIRD 2.1, 3.0 UND 4.0. 3.0 kommt überall an;
 * beim Lesen ist Nachsicht kostenlos – ein Android-Telefon liefert bis heute
 * 2.1 mit Quoted-Printable.
 *
 * SO VIEL STANDARDFELD WIE MÖGLICH: Fremde Programme werfen unbekannte
 * `X-`-Felder beim Speichern weg. Matrix, Meshtastic und der openany-Name
 * reisen deshalb als `IMPP` mit eigenem Schema – ein fremdes Programm kennt
 * das Schema nicht, behält die Zeile aber, weil das Feld bekannt ist.
 *
 * WAS WIR NICHT KENNEN, WIRD DURCHGEREICHT (`vcard:<feld>`) und geht genau so
 * zurück, roh und ohne Entmaskieren: Bei einem fremden Feld weiß niemand, ob
 * ein `;` trennt oder gemeint ist.
 */
import { VCARD_PRAEFIX } from './kontaktwege';

/** Art des Wegs → Feld. Die einzige Stelle; gelesen in beide Richtungen. */
const FELDER = {
  phone: 'TEL',
  email: 'EMAIL',
  address: 'ADR',
  company: 'ORG',
  birthday: 'BDAY',
  other: 'NOTE',
};

/** Art des Wegs → Schema der `IMPP`-URI. */
const IMPP_SCHEMA = { matrix: 'matrix', meshtastic: 'meshtastic', openany: 'openany' };

/** Felder älterer openany-Dateien, die noch gelesen werden. */
const NUR_LESEN = {
  'X-OPENANY-MATRIX': 'matrix',
  'X-OPENANY-MESHTASTIC': 'meshtastic',
  'X-OPENANY-NAME': 'openany',
  'X-OPENANY-OTHER': 'other',
};

/** Gerüst, schon Verarbeitetes und Bilder/Schlüssel – nicht durchgereicht. */
const NICHT_DURCHREICHEN = new Set([
  'BEGIN', 'END', 'VERSION', 'PRODID', 'REV', 'SOURCE', 'KIND', 'XML', 'CLIENTPIDMAP',
  'FN', 'N', 'UID', 'PHOTO',
  'LOGO', 'SOUND', 'KEY',
]);

/** Höchstzahl Kontakte je Einlesen – eine Bremse, keine Aussage. */
export const MAX_KONTAKTE = 5000;

const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

// --- Schreiben --------------------------------------------------------------

/** Die vier Zeichen, die in einem vCard-Wert etwas bedeuten. */
function maskieren(wert) {
  return String(wert ?? '')
    .replace(/\\/g, '\\\\')
    .replace(/\r\n|\n|\r/g, '\\n')
    .replace(/;/g, '\\;')
    .replace(/,/g, '\\,');
}

/** Nur sichern, was die Zeile zerreißen würde. */
function zeilenfest(wert) {
  return String(wert ?? '').replace(/\r\n|\n|\r/g, '\\n');
}

/** Die Beschriftung als `TYPE=` – oder gar nicht. */
function typ(label) {
  const sauber = String(label ?? '').replace(/[:;,\r\n]+/g, ' ').trim();
  return sauber === '' ? '' : `;TYPE=${sauber}`;
}

/** `@name:server` → `matrix:u/name:server`; sonst nur das Schema davor. */
function uri(art, wert) {
  const schema = IMPP_SCHEMA[art];
  const m = art === 'matrix' ? /^@([^:]+):(.+)$/.exec(wert) : null;
  return m ? `${schema}:u/${m[1]}:${m[2]}` : `${schema}:${wert}`;
}

/** Nach 75 Oktetten falten, nie mitten in einem UTF-8-Zeichen. */
function falten(zeile) {
  const kodierer = new TextEncoder();
  if (kodierer.encode(zeile).length <= 75) return zeile;
  const teile = [];
  let stueck = '';
  let laenge = 0;
  for (const zeichen of zeile) {
    const n = kodierer.encode(zeichen).length;
    // Fortsetzungszeilen beginnen mit einem Leerzeichen – es zählt mit.
    const grenze = teile.length === 0 ? 75 : 74;
    if (laenge + n > grenze) {
      teile.push(stueck);
      stueck = '';
      laenge = 0;
    }
    stueck += zeichen;
    laenge += n;
  }
  teile.push(stueck);
  return teile.join('\r\n ');
}

function wegZeile(weg) {
  const marke = typ(weg.label);
  if (String(weg.kind).startsWith(VCARD_PRAEFIX)) {
    return `${weg.kind.slice(VCARD_PRAEFIX.length).toUpperCase()}${marke}:${zeilenfest(weg.value)}`;
  }
  if (IMPP_SCHEMA[weg.kind]) {
    const dienst = weg.kind.charAt(0).toUpperCase() + weg.kind.slice(1);
    return `IMPP${marke};X-SERVICE-TYPE=${dienst}:${maskieren(uri(weg.kind, weg.value))}`;
  }
  const feld = FELDER[weg.kind] ?? FELDER.other;
  // ADR hat sieben Teile; die eine Zeile liegt in „Straße".
  const wert = feld === 'ADR' ? `;;${maskieren(weg.value)};;;;` : maskieren(weg.value);
  return `${feld}${marke}:${wert}`;
}

/**
 * Kontakte als vCard 3.0.
 *
 * @param {Array<{id: string, display_name: string, channels: Array, foto?: string|null}>} kontakte
 *   `foto`: JPEG/PNG als Base64 (ohne `data:`-Vorspann) oder nichts.
 */
export function vcardSchreiben(kontakte) {
  return kontakte.map((k) => {
    const zeilen = ['BEGIN:VCARD', 'VERSION:3.0'];
    zeilen.push(`FN:${maskieren(k.display_name)}`);
    // N will fünf Teile; der Name wird NICHT auseinandergeraten.
    zeilen.push(`N:${maskieren(k.display_name)};;;;`);
    // Über die Kennung findet ein späteres Einlesen denselben Kontakt.
    zeilen.push(`UID:urn:uuid:${k.id}`);
    if (k.foto) zeilen.push(`PHOTO;ENCODING=b;TYPE=JPEG:${k.foto}`);
    for (const weg of k.channels ?? []) zeilen.push(wegZeile(weg));
    zeilen.push('END:VCARD');
    return zeilen.map(falten).join('\r\n') + '\r\n';
  }).join('');
}

// --- Lesen ------------------------------------------------------------------

function entmaskieren(wert) {
  return String(wert).replace(/\\([nN;,\\])/g, (_, z) => (z === 'n' || z === 'N' ? '\n' : z));
}

/** Quoted-Printable als UTF-8. */
function qpDekodieren(wert) {
  const bytes = [];
  for (let i = 0; i < wert.length; i += 1) {
    if (wert[i] === '=' && /^[0-9A-Fa-f]{2}$/.test(wert.slice(i + 1, i + 3))) {
      bytes.push(parseInt(wert.slice(i + 1, i + 3), 16));
      i += 2;
    } else {
      bytes.push(...new TextEncoder().encode(wert[i]));
    }
  }
  return new TextDecoder().decode(new Uint8Array(bytes));
}

/** Gefaltete und Quoted-Printable-weich umbrochene Zeilen zusammenführen. */
function entfalten(inhalt) {
  const zeilen = [];
  for (const zeile of String(inhalt).replace(/^\uFEFF/, '').split(/\r\n|\r|\n/)) {
    const letzte = zeilen[zeilen.length - 1];
    if (letzte !== undefined && letzte.endsWith('=') && /QUOTED-PRINTABLE/i.test(letzte)) {
      zeilen[zeilen.length - 1] = letzte.slice(0, -1) + zeile;
    } else if (letzte !== undefined && (zeile.startsWith(' ') || zeile.startsWith('\t'))) {
      zeilen[zeilen.length - 1] = letzte + zeile.slice(1);
    } else {
      zeilen.push(zeile);
    }
  }
  return zeilen;
}

function parameterLesen(teile) {
  const p = {};
  for (const teil of teile) {
    const gleich = teil.indexOf('=');
    // vCard 2.1 schreibt den Typ nackt hin: `TEL;CELL:…`.
    if (gleich < 0) p.TYPE = teil.trim().toUpperCase();
    else p[teil.slice(0, gleich).trim().toUpperCase()] = teil.slice(gleich + 1).trim().replace(/^"|"$/g, '');
  }
  return p;
}

/** `PREF`, `INTERNET`, `VOICE` sind Vermerke, keine Beschriftungen. */
function beschriftung(p) {
  const marke = String(p['X-ABLABEL'] ?? p.TYPE ?? '').trim();
  if (marke === '' || ['PREF', 'INTERNET', 'VOICE'].includes(marke.toUpperCase())) return null;
  return marke.slice(0, 255);
}

/** Welcher Dienst steckt in dieser `IMPP`-Zeile? Fremdes bleibt IMPP. */
function dienst(wert, p) {
  const schema = String(wert).trim().split(':')[0].toLowerCase();
  const ueberSchema = Object.keys(IMPP_SCHEMA).find((a) => IMPP_SCHEMA[a] === schema);
  if (ueberSchema) return ueberSchema;
  const marke = String(p['X-SERVICE-TYPE'] ?? '').trim().toLowerCase();
  return Object.keys(IMPP_SCHEMA).find((a) => IMPP_SCHEMA[a] === marke) ?? `${VCARD_PRAEFIX}impp`;
}

function ausUri(art, wert) {
  const w = wert.trim();
  if (!IMPP_SCHEMA[art]) return w;
  const ohne = w.replace(new RegExp(`^${IMPP_SCHEMA[art]}:`, 'i'), '');
  const m = art === 'matrix' ? /^u\/([^:]+):(.+)$/.exec(ohne) : null;
  return m ? `@${m[1]}:${m[2]}` : ohne;
}

/** Das Bild als Base64 – nur eingebettet; eine Adresse wird nicht geholt. */
function bildAus(wert, p) {
  let w = wert.trim();
  if (w.toLowerCase().startsWith('data:')) {
    const komma = w.indexOf(',');
    w = komma < 0 ? '' : w.slice(komma + 1);
  } else if (!['b', 'base64'].includes(String(p.ENCODING ?? '').toLowerCase())) {
    return null;
  }
  w = w.replace(/\s+/g, '');
  return /^[A-Za-z0-9+/]+={0,2}$/.test(w) ? w : null;
}

/** An `;` teilen, die trennen – nicht an maskierten (`\;`). */
function teilen(wert) {
  return String(wert).split(/(?<!\\);/);
}

function zeileLesen(laufend, zeile) {
  const doppel = zeile.indexOf(':');
  const teile = zeile.slice(0, doppel).split(';');
  let wert = zeile.slice(doppel + 1);
  let name = teile.shift().trim().toUpperCase();
  // Apples Gruppierung `item1.TEL:…` – das Präfix interessiert nicht.
  if (name.includes('.')) name = name.slice(name.lastIndexOf('.') + 1);
  const p = parameterLesen(teile);
  if (String(p.ENCODING ?? '').toUpperCase() === 'QUOTED-PRINTABLE') wert = qpDekodieren(wert);

  if (name === 'PHOTO') { laufend.foto = bildAus(wert, p); return; }
  if (name === 'UID') {
    const roh = wert.trim().replace(/^urn:uuid:/i, '');
    laufend.uuid = UUID.test(roh) ? roh.toLowerCase() : null;
    return;
  }
  if (name === 'FN') { laufend.display_name = entmaskieren(wert).trim(); return; }
  if (name === 'N') {
    // Nachname zuerst; zusammengesetzt, sonst hieße jemand nur „Beispiel".
    const s = teilen(wert).map((x) => entmaskieren(x).trim());
    laufend.nachname = [s[1] ?? '', s[0] ?? ''].filter(Boolean).join(' ').trim();
    return;
  }

  const art = name === 'IMPP'
    ? dienst(wert, p)
    : (Object.keys(FELDER).find((a) => FELDER[a] === name) ?? NUR_LESEN[name] ?? null);

  if (art === null) {
    if (NICHT_DURCHREICHEN.has(name) || !/^[A-Z0-9][A-Z0-9-]{0,50}$/.test(name)) return;
    laufend.channels.push({ kind: VCARD_PRAEFIX + name.toLowerCase(), label: beschriftung(p), value: wert.trim() });
    return;
  }

  if (name === 'IMPP') wert = ausUri(art, entmaskieren(wert));
  const fertig = art === 'address'
    ? teilen(wert).map((x) => entmaskieren(x).trim()).filter(Boolean).join(', ')
    : entmaskieren(wert).trim();
  if (fertig === '') return;
  laufend.channels.push({ kind: art, label: beschriftung(p), value: fertig });
}

/**
 * Eine vCard-Datei in Kontakte zerlegen.
 *
 * @returns {Array<{uuid: string|null, display_name: string, foto: string|null, channels: Array}>}
 */
export function vcardLesen(inhalt) {
  const kontakte = [];
  let laufend = null;
  for (const zeile of entfalten(inhalt)) {
    const oben = zeile.trim().toUpperCase();
    if (oben === 'BEGIN:VCARD') {
      laufend = { uuid: null, display_name: '', nachname: '', foto: null, channels: [] };
      continue;
    }
    if (oben === 'END:VCARD') {
      if (laufend) {
        // FN fehlt in mancher 2.1-Datei; dann trägt N den Namen.
        if (laufend.display_name === '') laufend.display_name = laufend.nachname;
        delete laufend.nachname;
        kontakte.push(laufend);
      }
      laufend = null;
      continue;
    }
    if (laufend && zeile.includes(':')) zeileLesen(laufend, zeile);
  }
  return kontakte.filter((k) => k.display_name !== '').slice(0, MAX_KONTAKTE);
}

/**
 * Gelesene Kontakte mit dem Adressbuch zusammenführen – ERGÄNZT, ERSETZT
 * NICHT. Derselbe Mensch: erst die Kennung, dann der Name. Ein Weg kommt
 * dazu, wenn Art UND Wert neu sind; ein Foto nur, wo noch keines ist.
 *
 * Zurück kommt, was zu tun ist: `neu` (anlegen) und `ergaenzt` (ändern), je
 * mit `foto`, falls eines zu setzen ist.
 */
export function zusammenfuehren(vorhanden, gelesen) {
  const nachId = new Map(vorhanden.map((k) => [k.id, k]));
  const nachName = new Map();
  for (const k of vorhanden) if (!nachName.has(k.display_name)) nachName.set(k.display_name, k);

  const neu = [];
  const ergaenzt = [];
  const bearbeitet = new Map();
  for (const g of gelesen) {
    const da = (g.uuid && nachId.get(g.uuid)) || nachName.get(g.display_name);
    if (!da) {
      const k = { id: g.uuid && !nachId.has(g.uuid) ? g.uuid : null, display_name: g.display_name, channels: g.channels, foto: g.foto };
      neu.push(k);
      // Steht derselbe Name zweimal in der Datei, ergänzt der zweite den ersten.
      nachName.set(g.display_name, { ...k, neu: k });
      continue;
    }
    if (da.neu) {
      const bekannt = new Set(da.neu.channels.map((w) => `${w.kind}|${w.value}`));
      for (const w of g.channels) if (!bekannt.has(`${w.kind}|${w.value}`)) da.neu.channels.push(w);
      if (!da.neu.foto) da.neu.foto = g.foto;
      continue;
    }
    const ziel = bearbeitet.get(da.id) ?? { id: da.id, display_name: da.display_name, channels: [...(da.channels ?? [])], foto: null, hatFoto: Boolean(da.photo), getan: false };
    const bekannt = new Set(ziel.channels.map((w) => `${w.kind}|${w.value}`));
    for (const w of g.channels) {
      if (bekannt.has(`${w.kind}|${w.value}`)) continue;
      ziel.channels.push(w);
      bekannt.add(`${w.kind}|${w.value}`);
      ziel.getan = true;
    }
    if (!ziel.hatFoto && !ziel.foto && g.foto) { ziel.foto = g.foto; ziel.getan = true; }
    bearbeitet.set(da.id, ziel);
  }
  for (const z of bearbeitet.values()) if (z.getan) ergaenzt.push(z);
  return { neu, ergaenzt };
}
