// Die Quellenliste einer Mappe (CSL-JSON) -> BibLaTeX für Typst.
//
// WARUM ÜBERSETZT: Gespeichert wird CSL-JSON -- das schreibt Zotero, das
// liest Zettlr, und das zeigt der Editor beim Zitieren an. Typst liest aber
// nur BibLaTeX (.bib) oder sein eigenes YAML. BibLaTeX ist das verbreitetere,
// und Typst setzt daraus mit „harvard-cite-them-right" Zitat und Verzeichnis.
//
// WELCHES FELD WOHIN, ist am gesetzten PDF ausprobiert (14.10.2026), nicht
// nach der BibLaTeX-Anleitung: Typst (Hayagriva) übergeht bei @misc `note`
// und `howpublished` stillschweigend, `publisher` aber zeigt es. Deshalb
// landen Fundstellen von Urteilen und Gesetzen dort.
import harvardCsl from './stile/harvard-cite-them-right.csl?raw';

/** Ein Wert in geschweiften Klammern -- ohne Zeichen, die BibLaTeX umdeutet. */
function wert(text) {
  return String(text)
    .replace(/[{}\\]/g, '')
    .replace(/[&%$#_]/g, (z) => `\\${z}`)
    .replace(/\s+/g, ' ')
    .trim();
}

/** Adressen bleiben wie sie sind; nur Klammern dürfen nicht hinein. */
function adresse(text) {
  return String(text).replace(/[{}\s]/g, '');
}

/** CSL-Namen -> „Nachname, Vorname and …"; Körperschaften doppelt geklammert. */
function namen(liste) {
  if (!Array.isArray(liste)) return '';
  return liste
    .map((n) => {
      if (!n || typeof n !== 'object') return '';
      if (n.literal) return `{${wert(n.literal)}}`;
      const nach = wert(n.family ?? '');
      const vor = wert(n.given ?? '');
      return nach && vor ? `${nach}, ${vor}` : nach || vor;
    })
    .filter(Boolean)
    .join(' and ');
}

/** CSL-Datum -> 2020, 2020-03 oder 2020-03-12. */
export function datum(d) {
  const teile = d?.['date-parts']?.[0];
  if (Array.isArray(teile) && teile.length && Number.isInteger(Number(teile[0]))) {
    return teile.slice(0, 3).map((x, i) => String(Number(x)).padStart(i === 0 ? 4 : 2, '0')).join('-');
  }
  // Freie Angaben („Frühjahr 2020") taugen nicht; das Jahr darin schon.
  const jahr = /\b(\d{4})\b/.exec(String(d?.raw ?? d?.literal ?? ''));
  return jahr ? jahr[1] : '';
}

/** 12.03.2020 -- für die Bezeichnung eines Urteils, sprachneutral. */
function tagesdatum(d) {
  const [j, m, t] = d?.['date-parts']?.[0] ?? [];
  return j && m && t ? `${String(t).padStart(2, '0')}.${String(m).padStart(2, '0')}.${j}` : '';
}

// Ein Schlüssel, den BibLaTeX und Typst ohne Umstände lesen. Die Liste in
// openany lässt schon nur zitierbare zu (kein Leerzeichen, keine Klammern);
// was aus einer fremden Datei kommt, kann mehr enthalten.
const SCHLUESSEL = /^[^\s{}(),=\\#%"'~[\];@]+$/u;

/** Ist dieser Schlüssel in einer .bib-Datei verwendbar? */
export function brauchbarerSchluessel(id) {
  return typeof id === 'string' && SCHLUESSEL.test(id);
}

/** Ein Eintrag -> [BibLaTeX-Typ, {feld: wert}] */
function abbilden(e) {
  const f = {
    author: namen(e.author),
    editor: namen(e.editor),
    title: e.title ? wert(e.title) : '',
    date: datum(e.issued),
    url: e.URL ? adresse(e.URL) : '',
    doi: e.DOI ? adresse(e.DOI) : '',
  };
  const verlag = () => {
    f.publisher = e.publisher ? wert(e.publisher) : '';
    f.location = e['publisher-place'] ? wert(e['publisher-place']) : '';
  };
  const fundstelle = () => {
    const quelle = e['container-title'] ?? e.publisher;
    f.publisher = quelle ? wert(quelle) : '';
  };

  switch (e.type) {
    case 'book':
      verlag();
      f.edition = e.edition ? wert(e.edition) : '';
      f.isbn = e.ISBN ? wert(e.ISBN) : '';
      return ['book', f];
    case 'article-journal':
    case 'article-magazine':
    case 'article-newspaper':
    case 'article':
      f.journaltitle = e['container-title'] ? wert(e['container-title']) : '';
      f.volume = e.volume ? wert(e.volume) : '';
      f.number = e.issue ? wert(e.issue) : '';
      f.pages = e.page ? wert(e.page).replace(/\s*[-–]\s*/g, '--') : '';
      return ['article', f];
    case 'chapter':
    case 'paper-conference':
      verlag();
      f.booktitle = e['container-title'] ? wert(e['container-title']) : '';
      f.pages = e.page ? wert(e.page).replace(/\s*[-–]\s*/g, '--') : '';
      return [e.type === 'chapter' ? 'incollection' : 'inproceedings', f];
    case 'webpage':
    case 'post':
    case 'post-weblog':
      f.urldate = datum(e.accessed);
      return ['online', f];
    case 'thesis':
    case 'report':
      f.publisher = e.publisher ? wert(e.publisher) : '';
      f.type = e.genre ? wert(e.genre) : '';
      return [e.type, f];
    case 'legal_case':
      // Das Gericht steht, wo sonst die Verfasser stehen: „(BAG, 2020)".
      f.author = e.authority ? `{${wert(e.authority)}}` : f.author;
      f.title = f.title || wert([e.genre, tagesdatum(e.issued), e.number].filter(Boolean).join(', '));
      fundstelle();
      return ['misc', f];
    case 'legislation':
    case 'bill':
      // Als Buch: Typst setzt den Namen dann kursiv statt in Anführungszeichen.
      fundstelle();
      return ['book', f];
    default:
      fundstelle();
      return ['misc', f];
  }
}

/**
 * @param {Array<object>} eintraege  CSL-JSON
 * @returns {{bib: string, schluessel: Set<string>}} die Datei und welche
 *          Schlüssel darin stehen (nur die darf ein Zitat verwenden -- Typst
 *          bricht bei einem unbekannten ab)
 */
export function quellenZuBib(eintraege) {
  const schluessel = new Set();
  const teile = [];
  for (const e of Array.isArray(eintraege) ? eintraege : []) {
    if (!e || typeof e !== 'object' || !brauchbarerSchluessel(e.id) || schluessel.has(e.id)) continue;
    const [typ, felder] = abbilden(e);
    // Ohne Verfasser UND ohne Titel weiß niemand, was gemeint ist.
    if (!felder.author && !felder.title) continue;
    schluessel.add(e.id);
    const zeilen = Object.entries(felder)
      .filter(([, v]) => v)
      .map(([k, v]) => `  ${k} = {${v}},`);
    teile.push(`@${typ}{${e.id},\n${zeilen.join('\n')}\n}`);
  }
  return { bib: teile.join('\n\n') + (teile.length ? '\n' : ''), schluessel };
}

/** Unter diesen Namen liegen Liste und Stil im Typst-Projekt. */
export const BIB_DATEI = 'quellen.bib';
export const STIL_DATEI = 'quellen.csl';

// DER STIL ALS EIGENE DATEI (15.10.2026), nicht Typsts eingebauter
// „harvard-cite-them-right": Dessen deutsche Datumsform lässt den Punkt nach
// dem Tag weg („Zugegriffen: 14 Oktober 2026"). Die Datei ist die offizielle
// aus dem CSL-Verzeichnis (stile/, CC BY-SA 3.0); am gesetzten PDF verglichen
// unterscheidet sie sich vom eingebauten nur in diesem Punkt. Für Deutsch
// kommt die Datumsform mit Punkt dazu, die übrigen Sprachen setzen den Stil
// unverändert.
const DEUTSCHES_DATUM = `  <locale xml:lang="de">
    <date form="text">
      <date-part name="day" suffix=". "/>
      <date-part name="month" suffix=" "/>
      <date-part name="year"/>
    </date>
  </locale>
`;

/** Die Stil-Datei für eine Sprache. */
export function stilFuer(sprache) {
  if (String(sprache || '').slice(0, 2).toLowerCase() !== 'de') return harvardCsl;
  return harvardCsl
    .replace('  <locale xml:lang="en-GB">', `${DEUTSCHES_DATUM}  <locale xml:lang="en-GB">`)
    // Der Erscheinungstag bei Zeitungen, Blogs, Vorträgen: „12. März".
    .replace('<date-part name="day" suffix=" "/>', '<date-part name="day" suffix=". "/>');
}

// Typst sagt auf Deutsch „Bibliographie"; in Hausarbeiten heißt es
// „Literaturverzeichnis". Für andere Sprachen das dort Übliche.
const UEBERSCHRIFT = {
  de: 'Literaturverzeichnis',
  en: 'References',
  fr: 'Bibliographie',
  es: 'Bibliografía',
  pl: 'Bibliografia',
  pt: 'Referências',
  uk: 'Список літератури',
};

/** Der Typst-Aufruf fürs Verzeichnis -- Harvard, wie mit Tiffy festgelegt. */
export function literaturverzeichnis(sprache) {
  const titel = UEBERSCHRIFT[String(sprache || '').slice(0, 2).toLowerCase()];
  return `#bibliography("${BIB_DATEI}", ${titel ? `title: [${titel}], ` : ''}style: "${STIL_DATEI}")`;
}

/**
 * Die Quellen einer Mappe, fertig für den Satz. Scheitert das Laden (keine
 * Liste, kein Netz, fremde Quelle ohne Liste), wird eben ohne gesetzt:
 * `[@…]` bleibt dann als Text stehen, das PDF entsteht trotzdem.
 *
 * @param {object} quelle  eine Notiz-Quelle (notesSources.js)
 * @param {?number} mappeId
 */
export async function quellenHolen(quelle, mappeId) {
  if (!mappeId || !quelle.quellenLesen) return quellenZuBib([]);
  try {
    const { data } = await quelle.quellenLesen(mappeId);
    return quellenZuBib(data?.eintraege ?? []);
  } catch (e) {
    console.warn('Quellen nicht geladen -- ohne Zitate gesetzt', e);
    return quellenZuBib([]);
  }
}

/** Liste und Stil zu den übrigen Dateien legen -- nur, wenn auch zitiert wird. */
export function bibBeilegen(dateien, bib, zitiert, sprache = 'de') {
  if (!zitiert.size) return false;
  dateien.set(BIB_DATEI, new TextEncoder().encode(bib));
  dateien.set(STIL_DATEI, new TextEncoder().encode(stilFuer(sprache)));
  return true;
}
