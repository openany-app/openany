import { describe, it, expect } from 'vitest';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { FREMDE_PAKETE, NUR_TESTS } from './fremde-pakete.js';

const WURZEL = dirname(fileURLToPath(import.meta.url));

function dateien(ordner) {
  return readdirSync(ordner).flatMap((name) => {
    const pfad = join(ordner, name);
    if (statSync(pfad).isDirectory()) return pfad.includes('/i18n/locales') ? [] : dateien(pfad);
    return /\.(js|vue)$/.test(name) ? [pfad] : [];
  });
}

describe('fremde Pakete', () => {
  it('jeder Import von außen steht in FREMDE_PAKETE', () => {
    const erlaubt = new Set([...FREMDE_PAKETE, ...NUR_TESTS]);
    const fehlt = new Set();

    for (const datei of dateien(WURZEL)) {
      const text = readFileSync(datei, 'utf8');
      for (const [, name] of text.matchAll(/(?:from|import)\s+'([^'.][^']*)'/g)) {
        if (name.startsWith('@oberflaeche') || name.startsWith('node:')) continue;
        // '@scope/paket/unterpfad' und 'paket/unterpfad' auf den Paketnamen kürzen
        const teile = name.split('/');
        const paket = name.startsWith('@') ? teile.slice(0, 2).join('/') : teile[0];
        if (!erlaubt.has(paket)) fehlt.add(`${paket} (${datei.slice(WURZEL.length + 1)})`);
      }
    }

    expect([...fehlt]).toEqual([]);
  });
});
