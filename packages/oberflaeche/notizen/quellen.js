// Die Quellenliste einer Mappe: zwischen CSL-JSON und dem Formular.
//
// Gespeichert wird CSL-JSON (QuellenListe.php) -- das Format von Zotero und
// Zettlr. Das Formular kennt nur eine Handvoll Arten mit deutschen Feldern;
// alles, was es nicht kennt (Felder aus einem Zotero-Export, fremde Arten),
// bleibt beim Speichern unangetastet erhalten.

/** Formularfeld -> CSL-Feld (Text). */
const CSL = {
  titel: 'title',
  verlag: 'publisher',
  ort: 'publisher-place',
  auflage: 'edition',
  enthalten: 'container-title',
  jahrgang: 'volume',
  heft: 'issue',
  seiten: 'page',
  url: 'URL',
  gericht: 'authority',
  art: 'genre',
  aktenzeichen: 'number',
};

/**
 * Die Arten des Formulars und ihre Felder, in Reihenfolge. `enthalten` heißt
 * je nach Art anders (Zeitschrift, Sammelband, Fundstelle) -- die Beschriftung
 * kommt aus `notes.quellen.feld.enthalten_<art>`.
 */
export const ARTEN = {
  book: ['autoren', 'jahr', 'titel', 'auflage', 'ort', 'verlag'],
  'article-journal': ['autoren', 'jahr', 'titel', 'enthalten', 'jahrgang', 'heft', 'seiten'],
  chapter: ['autoren', 'jahr', 'titel', 'herausgeber', 'enthalten', 'ort', 'verlag', 'seiten'],
  webpage: ['autoren', 'jahr', 'titel', 'url', 'abgerufen'],
  legal_case: ['gericht', 'art', 'datum', 'aktenzeichen', 'enthalten'],
  legislation: ['titel', 'jahr', 'enthalten'],
  // Auch für Arten, die nur aus einer fremden Datei kommen (report, thesis …).
  misc: ['autoren', 'jahr', 'titel', 'enthalten', 'url'],
};

/** Welche Felder eine Art im Formular zeigt. */
export const felderVon = (typ) => ARTEN[typ] ?? ARTEN.misc;

const zeilen = (text) => String(text ?? '').split(/\r?\n/).map((z) => z.trim()).filter(Boolean);

/** „Nachname, Vorname" je Zeile; ohne Komma eine Einrichtung (literal). */
function namenAusText(text) {
  return zeilen(text).map((z) => {
    const komma = z.indexOf(',');
    if (komma < 0) return { literal: z };
    const family = z.slice(0, komma).trim();
    const given = z.slice(komma + 1).trim();
    return given ? { family, given } : { family };
  });
}

function namenAlsText(liste) {
  return (Array.isArray(liste) ? liste : [])
    .map((n) => n?.literal ?? [n?.family, n?.given].filter(Boolean).join(', '))
    .filter(Boolean)
    .join('\n');
}

const teile = (d) => (Array.isArray(d?.['date-parts']?.[0]) ? d['date-parts'][0] : []);

/** 2020-03-12 aus [2020, 3, 12] -- für <input type="date">. */
function tagAlsText(d) {
  const [j, m, t] = teile(d);
  return j && m && t ? `${j}-${String(m).padStart(2, '0')}-${String(t).padStart(2, '0')}` : '';
}

function tagAusText(text) {
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(String(text ?? ''));
  return m ? { 'date-parts': [[Number(m[1]), Number(m[2]), Number(m[3])]] } : null;
}

/** Ein CSL-Eintrag -> Formularwerte (alles Text). */
export function alsFormular(eintrag = {}) {
  const f = { typ: eintrag.type || 'book', schluessel: eintrag.id ?? '' };
  for (const [feld, csl] of Object.entries(CSL)) f[feld] = eintrag[csl] != null ? String(eintrag[csl]) : '';
  f.autoren = namenAlsText(eintrag.author);
  f.herausgeber = namenAlsText(eintrag.editor);
  f.jahr = teile(eintrag.issued)[0] ? String(teile(eintrag.issued)[0]) : '';
  f.datum = tagAlsText(eintrag.issued);
  f.abgerufen = tagAlsText(eintrag.accessed);
  return f;
}

/**
 * Formularwerte -> CSL-Eintrag. Felder, die das Formular nicht kennt,
 * bleiben aus `vorher` erhalten. Wechselt die Art, fällt weg, was die neue
 * nicht zeigt -- sonst stünde ein Verlag unsichtbar unter einem Aufsatz.
 */
export function ausFormular(f, vorher = {}) {
  const e = { ...vorher, id: f.schluessel.trim(), type: f.typ };
  const felder = felderVon(f.typ);
  const artGewechselt = vorher.type && vorher.type !== f.typ;

  const setzen = (csl, wert) => {
    if (wert == null || wert === '' || (Array.isArray(wert) && !wert.length)) delete e[csl];
    else e[csl] = wert;
  };

  for (const [feld, csl] of Object.entries(CSL)) {
    if (felder.includes(feld)) setzen(csl, String(f[feld] ?? '').trim());
    else if (artGewechselt) delete e[csl];
  }
  if (felder.includes('autoren')) setzen('author', namenAusText(f.autoren));
  else if (artGewechselt) delete e.author;
  if (felder.includes('herausgeber')) setzen('editor', namenAusText(f.herausgeber));
  else if (artGewechselt) delete e.editor;
  if (felder.includes('abgerufen')) setzen('accessed', tagAusText(f.abgerufen));
  else if (artGewechselt) delete e.accessed;

  if (felder.includes('datum')) {
    setzen('issued', tagAusText(f.datum));
  } else if (felder.includes('jahr')) {
    const jahr = Number.parseInt(f.jahr, 10);
    // Ein Monat/Tag aus einer fremden Datei bleibt, solange das Jahr stimmt.
    if (Number.isInteger(jahr) && teile(vorher.issued)[0] === jahr) e.issued = vorher.issued;
    else setzen('issued', Number.isInteger(jahr) ? { 'date-parts': [[jahr]] } : null);
  }
  return e;
}

// Wie QuellenListe.php: was sich mit [@…] zitieren lässt.
const SCHLUESSEL = /^[^\s[\];,@]+$/u;

/** Kurzname gültig? (Leer, mit Leerzeichen oder Klammern: nein.) */
export const schluesselGueltig = (s) => SCHLUESSEL.test(String(s ?? '').trim());

/** „Müller" -> „muller": nur Kleinbuchstaben und Ziffern, Umlaute ohne Punkte. */
function falten(text) {
  return String(text ?? '')
    .normalize('NFD')
    .replace(/[̀-ͯ]/g, '')
    .replace(/ß/g, 'ss')
    .toLowerCase()
    .replace(/[^a-z0-9]/g, '');
}

/**
 * Ein Kurzname aus Verfasser und Jahr: mueller2020, bei Gleichstand
 * mueller2020a, mueller2020b -- wie in Harvard-Verzeichnissen üblich.
 * @param {object} f  Formularwerte
 * @param {Set<string>} vergeben  Kurznamen der ANDEREN Einträge
 */
export function schluesselVorschlag(f, vergeben = new Set()) {
  const erster = namenAusText(f.autoren)[0];
  const wort = (erster?.family ?? erster?.literal ?? f.gericht ?? '') || f.titel || '';
  const name = falten(String(wort).trim().split(/\s+/)[0]) || 'quelle';
  const jahr = (f.datum || '').slice(0, 4) || String(f.jahr ?? '').trim().replace(/\D/g, '');
  const grund = `${name}${jahr}`;
  if (!vergeben.has(grund)) return grund;
  for (const z of 'abcdefghijklmnopqrstuvwxyz') if (!vergeben.has(grund + z)) return grund + z;
  return `${grund}-${Date.now()}`;
}

/** Eine Zeile für die Liste: „Müller und Weber (2020): Titel". */
export function kurzanzeige(e, und = 'und') {
  const namen = (e.author ?? []).map((n) => n?.family ?? n?.literal).filter(Boolean);
  const wer = namen.length > 2 ? `${namen[0]} et al.` : namen.join(` ${und} `) || e.authority || '';
  const jahr = teile(e.issued)[0];
  const titel = e.title || [e.genre, e.number].filter(Boolean).join(' ');
  return [wer && (jahr ? `${wer} (${jahr})` : wer), titel].filter(Boolean).join(': ') || e.id;
}

/** Hat der Eintrag genug, um ihn zu erkennen? (Wie quellenZuBib: Verfasser oder Titel.) */
export function vollstaendig(f) {
  return Boolean(zeilen(f.autoren).length || String(f.titel ?? '').trim() || String(f.gericht ?? '').trim());
}
