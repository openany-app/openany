#!/usr/bin/env node
/*
 * Die Lizenzen aller Pakete, die in der App stecken: public/lizenzen.json.
 *
 * Läuft bei jedem Bau (`npm run build`), damit die Liste zum gebauten Stand
 * passt, und liegt danach in der App, also ohne Netz lesbar. Die Datei ist nicht
 * eingecheckt.
 *
 * Rust aus `cargo metadata`, ohne Pakete, die nur zum Bauen oder Testen dienen.
 * JavaScript aus der package-lock.json: was von `dependencies` aus erreichbar ist,
 * ohne Bauwerkzeuge und ohne native Bausteine für einzelne Rechner. Die Kennzeichen
 * `dev` der Lockdatei reichen dafür nicht: Über `@tailwindcss/vite` verliert Vite
 * sie samt allem, was daran hängt. Gleiche Lizenztexte stehen nur einmal in der Datei.
 *
 * WAS DARÜBER HINAUS GILT:
 * - Liefert ein Paket keinen Lizenztext mit, kommt der Standardtext seiner Lizenz
 *   aus `lizenztexte/` (SPDX) und das Paket ist mit `standard` gekennzeichnet.
 *   Seine Urheber stehen dann in `urheber`, soweit das Paket sie nennt.
 * - `lizenzen-nachtrag.json` nennt, was kein Paketverzeichnis kennt: fehlende
 *   Angaben und Bestandteile wie die Android-Bibliotheken oder gebündelte Schriften.
 *   Ihre Texte liegen wörtlich aus den Quellen in `lizenztexte/nachtrag/`.
 * - Lizenzdateien unter `public/` erscheinen als eigene Einträge.
 *
 * Bleibt ein Paket ohne Lizenz und ohne Text, bricht der Bau ab: Die Liste soll
 * nicht still unvollständig werden.
 */
import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const APP = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const NACHTRAG = JSON.parse(readFileSync(join(APP, 'scripts', 'lizenzen-nachtrag.json'), 'utf8'));
// Gehört zu openany selbst, liegt aber in einem eigenen Repository.
const EIGENE_PAKETE = new Set(['anyid-client']);
// Steht in `dependencies`, läuft aber nur beim Bauen.
const NUR_ZUM_BAUEN = new Set(['@tailwindcss/vite']);
const LIZENZDATEI = /^(licen[cs]e|copying|notice|unlicense)/i;

const texte = [];
const textIndex = new Map();
const eintragen = (text) => {
  const sauber = text.replace(/\r\n/g, '\n').trim();
  if (!textIndex.has(sauber)) {
    textIndex.set(sauber, texte.length);
    texte.push(sauber);
  }
  return textIndex.get(sauber);
};

/** Der Standardtext einer Lizenz. Bei „oder" reicht die erste, die wir haben, bei „und" braucht es alle. */
function standardtexte(ausdruck) {
  const namen = String(ausdruck).match(/[A-Za-z0-9.+-]+/g)?.filter((w) => !['OR', 'AND', 'WITH'].includes(w)) ?? [];
  const vorhanden = namen.filter((n) => existsSync(join(APP, 'scripts', 'lizenztexte', `${n}.txt`)));
  const gewaehlt = /\bAND\b/.test(ausdruck) ? vorhanden : vorhanden.slice(0, 1);
  return gewaehlt.map((n) => eintragen(readFileSync(join(APP, 'scripts', 'lizenztexte', `${n}.txt`), 'utf8')));
}

function lizenztexte(ordner) {
  // Gebaute Pakete legen ihre Hinweise zu mitgelieferten Dateien oft in dist/.
  return [ordner, join(ordner, 'dist')].flatMap((o) => {
    if (!existsSync(o) || !statSync(o).isDirectory()) return [];
    return readdirSync(o)
      .filter((n) => LIZENZDATEI.test(n) && statSync(join(o, n)).isFile())
      .sort()
      .map((n) => eintragen(readFileSync(join(o, n), 'utf8')));
  });
}

/** Die Adresse des Quellcodes, wie sie das Paket nennt. */
const quelle = (repository) => String((typeof repository === 'string' ? repository : repository?.url) ?? '')
  .replace(/^git\+/, '').replace(/\.git$/, '');

/** Bei Paketen aus einem Git-Repository liegt die Lizenz oft ganz oben darin. */
function rustTexte(paket) {
  let ordner = dirname(paket.manifest_path);
  for (let tiefe = 0; tiefe < 4; tiefe++) {
    const gefunden = lizenztexte(ordner);
    if (gefunden.length || !paket.source.startsWith('git+')) return gefunden;
    ordner = dirname(ordner);
  }
  return [];
}

function rustPakete() {
  const roh = execFileSync('cargo', ['metadata', '--format-version', '1', '--locked'], { cwd: APP, maxBuffer: 1 << 30, encoding: 'utf8' });
  const { packages, resolve: aufloesung, workspace_members: eigene } = JSON.parse(roh);
  const nachId = new Map(packages.map((p) => [p.id, p]));
  const knoten = new Map(aufloesung.nodes.map((n) => [n.id, n]));
  const gesehen = new Set();
  const offen = [...eigene];
  while (offen.length) {
    const id = offen.pop();
    if (gesehen.has(id)) continue;
    gesehen.add(id);
    for (const dep of knoten.get(id)?.deps ?? []) {
      if (dep.dep_kinds.some((k) => k.kind === null)) offen.push(dep.pkg);
    }
  }
  return [...gesehen]
    .map((id) => nachId.get(id))
    // Ohne `source` liegt das Paket bei uns: Es ist Teil von openany selbst.
    .filter((p) => p.source && !EIGENE_PAKETE.has(p.name))
    .map((p) => ({ name: p.name, version: p.version, lizenz: p.license ?? '', art: 'rust', urheber: (p.authors ?? []).join(', '), quelle: quelle(p.repository), texte: rustTexte(p) }));
}

/** Wo node_modules ein Paket für den Eintrag `von` findet: von innen nach außen. */
function aufloesen(eintraege, name, von) {
  let pfad = von;
  while (true) {
    const treffer = `${pfad ? `${pfad}/` : ''}node_modules/${name}`;
    if (eintraege[treffer]) return treffer;
    if (!pfad) return null;
    const schnitt = pfad.lastIndexOf('/node_modules/');
    pfad = schnitt >= 0 ? pfad.slice(0, schnitt) : '';
  }
}

function erreichbar(eintraege) {
  const gesehen = new Set();
  const offen = Object.keys(eintraege[''].dependencies ?? {})
    .filter((n) => !NUR_ZUM_BAUEN.has(n))
    .map((name) => [name, '']);
  while (offen.length) {
    const [name, von] = offen.pop();
    const pfad = aufloesen(eintraege, name, von);
    if (!pfad || gesehen.has(pfad)) continue;
    const paket = eintraege[pfad];
    if (paket.os || paket.cpu) continue;
    gesehen.add(pfad);
    const pflichtPartner = Object.keys(paket.peerDependencies ?? {})
      .filter((n) => !paket.peerDependenciesMeta?.[n]?.optional);
    const namen = [...Object.keys(paket.dependencies ?? {}), ...Object.keys(paket.optionalDependencies ?? {}), ...pflichtPartner];
    offen.push(...namen.map((n) => [n, pfad]));
  }
  return gesehen;
}

function jsPakete() {
  const { packages: eintraege } = JSON.parse(readFileSync(join(APP, 'package-lock.json'), 'utf8'));
  return [...erreichbar(eintraege)]
    .map((pfad) => {
      const p = eintraege[pfad];
      const name = p.name ?? pfad.slice(pfad.lastIndexOf('node_modules/') + 13);
      const ordner = join(APP, pfad);
      if (!existsSync(ordner)) return null;
      const angaben = JSON.parse(readFileSync(join(ordner, 'package.json'), 'utf8'));
      const urheber = typeof angaben.author === 'string' ? angaben.author : angaben.author?.name ?? '';
      return { name, version: p.version, lizenz: p.license ?? angaben.license ?? '', art: 'js', urheber, quelle: quelle(angaben.repository), texte: lizenztexte(ordner) };
    })
    .filter(Boolean);
}

/** Lizenzdateien, die mit Dateien unter public/ in die App kommen. */
function beigelegte(ordner = join(APP, 'public')) {
  if (!existsSync(ordner)) return [];
  return readdirSync(ordner, { withFileTypes: true }).flatMap((e) => {
    const pfad = join(ordner, e.name);
    if (e.isDirectory()) return beigelegte(pfad);
    if (!/^licen[cs]e/i.test(e.name)) return [];
    return [{ name: relative(join(APP, 'public'), pfad).replace(/\\/g, '/'), version: '', lizenz: '', art: 'datei', urheber: '', texte: [eintragen(readFileSync(pfad, 'utf8'))] }];
  });
}

const pakete = [
  ...rustPakete(),
  ...jsPakete(),
  ...beigelegte(),
  ...NACHTRAG.pakete.map(({ datei, ...p }) => ({
    version: '',
    art: 'nachtrag',
    ...p,
    texte: [datei].flat().map((n) => eintragen(readFileSync(join(APP, 'scripts', 'lizenztexte', 'nachtrag', `${n}.txt`), 'utf8'))),
  })),
]
  .map((p) => ({ ...p, lizenz: p.lizenz || NACHTRAG.angaben[p.name] || '' }))
  .map((p) => {
    if (p.texte.length) return p;
    const texte = standardtexte(p.lizenz);
    return texte.length ? { ...p, texte, standard: true } : p;
  })
  .sort((a, b) => a.name.localeCompare(b.name) || a.version.localeCompare(b.version));

const ziel = join(APP, 'public', 'lizenzen.json');
mkdirSync(dirname(ziel), { recursive: true });
writeFileSync(ziel, JSON.stringify({ pakete, texte }));

console.log(`Lizenzen: ${pakete.length} Einträge, ${texte.length} Texte, ${Math.round(statSync(ziel).size / 1024)} KB, ${pakete.filter((p) => p.standard).length} mit Standardtext`);
const offen = pakete.filter((p) => (!p.lizenz && p.art !== 'datei') || !p.texte.length).map((p) => `${p.name} (${p.lizenz || 'ohne Angabe'})`);
if (offen.length) {
  console.error(`Lizenzen unvollständig: ${offen.join(', ')}`);
  process.exit(1);
}
