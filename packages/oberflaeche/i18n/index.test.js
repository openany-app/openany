import { describe, it, expect } from 'vitest';
import { createI18n } from 'vue-i18n';
import { SUPPORTED_LOCALES } from './index';

// Hier ALLE Sprachen fest geladen – in der App kommen sie erst bei Bedarf.
const module = import.meta.glob('./locales/*/index.js', { eager: true });
const sprachen = Object.fromEntries(
  SUPPORTED_LOCALES.map((l) => [l, module[`./locales/${l}/index.js`]?.default]),
);
const { de } = sprachen;

/*
 * JEDE ZEILE MUSS SICH UEBERSETZEN LASSEN.
 *
 * Der Fehler, den dieser Test faengt, ist am 07.09.2026 in Produktion
 * gelandet und hat einen ganzen Dialog erschlagen: Ein Platzhalter
 * `name@beispiel.de` sieht wie Text aus, ist fuer vue-i18n aber der Anfang
 * einer VERKNUEPFTEN NACHRICHT (`@:andere.zeile`). Beim ersten Aufruf wirft
 * der Uebersetzer einen SyntaxError -- nicht beim Bauen, nicht beim Start,
 * sondern erst, wenn jemand die Stelle oeffnet, an der die Zeile vorkommt.
 *
 * Genauso empfindlich: `|` (trennt Mehrzahlformen) und `{`/`}` (Platzhalter).
 * Wer so ein Zeichen woertlich meint, schreibt es als `{'@'}`.
 *
 * Der Test ruft deshalb JEDE Zeile einmal auf, denn erst der Aufruf
 * uebersetzt sie.
 */
const i18n = createI18n({ legacy: false, locale: 'de', messages: sprachen });

const pfade = (objekt, praefix = '') =>
  Object.entries(objekt).flatMap(([schluessel, wert]) => {
    const pfad = praefix ? `${praefix}.${schluessel}` : schluessel;

    if (typeof wert === 'string') return [pfad];
    if (wert && typeof wert === 'object') return pfade(wert, pfad);

    return [];
  });

describe.each(Object.entries(sprachen))('%s', (sprache, nachrichten) => {
  const alle = pfade(nachrichten ?? {});

  it('hat ueberhaupt Zeilen', () => {
    expect(alle.length).toBeGreaterThan(500);
  });

  it('uebersetzt jede Zeile ohne Syntaxfehler', () => {
    i18n.global.locale.value = sprache;
    const kaputt = [];

    alle.forEach((pfad) => {
      try {
        // Ein Platzhalter-Objekt, damit `{name}` nicht als fehlend auffaellt;
        // geprueft wird hier die FORM der Zeile, nicht ihr Inhalt.
        i18n.global.t(pfad, { count: 1 }, { plural: 1 });
      } catch (e) {
        kaputt.push(`${pfad}: ${e.message}`);
      }
    });

    expect(kaputt).toEqual([]);
  });
});

/*
 * JEDE SPRACHE HAT GENAU DIE ZEILEN DES DEUTSCHEN.
 *
 * Fehlt eine, springt vue-i18n still auf Deutsch zurück – mitten in einem
 * französischen Dialog steht dann ein deutscher Satz, und niemand merkt es,
 * der nicht gerade Französisch eingestellt hat. Überzählige Zeilen sind
 * Reste umbenannter Schlüssel.
 *
 * Und dieselben Platzhalter: Wer `{name}` beim Übersetzen zu `{nom}` macht,
 * bekommt keinen Fehler, sondern eine leere Stelle im Satz.
 */
const blaetter = (objekt, praefix = '') =>
  Object.entries(objekt).flatMap(([schluessel, wert]) => {
    const pfad = praefix ? `${praefix}.${schluessel}` : schluessel;

    if (typeof wert === 'string') return [[pfad, wert]];
    if (wert && typeof wert === 'object') return blaetter(wert, pfad);

    return [];
  });

const platzhalter = (text) => [...new Set(text.match(/\{\s*\w+\s*\}/g) ?? [])].sort();

describe.each(SUPPORTED_LOCALES.filter((l) => l !== 'de'))('%s gegen Deutsch', (sprache) => {
  const soll = new Map(blaetter(de));
  const ist = new Map(blaetter(sprachen[sprache] ?? {}));

  it('ist in SUPPORTED_LOCALES und hat eine Datei', () => {
    expect(sprachen[sprache]).toBeTypeOf('object');
  });

  it('hat keine fehlenden Zeilen', () => {
    expect([...soll.keys()].filter((k) => !ist.has(k))).toEqual([]);
  });

  it('hat keine überzähligen Zeilen', () => {
    expect([...ist.keys()].filter((k) => !soll.has(k))).toEqual([]);
  });

  it('hat dieselben Platzhalter', () => {
    const abweichend = [...soll]
      .filter(([k]) => ist.has(k))
      .filter(([k, v]) => platzhalter(v).join() !== platzhalter(ist.get(k)).join())
      .map(([k, v]) => `${k}: ${platzhalter(v)} ≠ ${platzhalter(ist.get(k))}`);

    expect(abweichend).toEqual([]);
  });
});

/*
 * DREI MEHRZAHLFORMEN IM POLNISCHEN UND UKRAINISCHEN.
 *
 * Die Regel steht in i18n/index.js und nicht in vue-i18n selbst; ohne sie
 * wählt vue-i18n bei drei Formen nach seiner eigenen Zählung und schreibt
 * „5 kontakty". Die 21 trennt beide Sprachen: ukrainisch Einzahl, polnisch
 * nicht.
 */
describe('Mehrzahl', () => {
  it.each([
    ['pl', 1, '1 kontakt'],
    ['pl', 3, '3 kontakty'],
    ['pl', 5, '5 kontaktów'],
    ['pl', 13, '13 kontaktów'],
    ['pl', 21, '21 kontaktów'],
    ['pl', 22, '22 kontakty'],
    ['uk', 1, '1 контакт'],
    ['uk', 4, '4 контакти'],
    ['uk', 11, '11 контактів'],
    ['uk', 21, '21 контакт'],
    ['uk', 25, '25 контактів'],
  ])('%s: %i', async (sprache, anzahl, erwartet) => {
    const { i18n: app, setLocale } = await import('./index');
    await setLocale(sprache);

    expect(app.global.t('home.dashboard.contactsTotal', anzahl)).toBe(erwartet);
  });

  // Dieselbe Regel für die Zeilen der App (app.js, seit 03.10.2026).
  it.each([
    ['pl', 1, 'Jeden nowy mail.'],
    ['pl', 3, '3 nowe maile.'],
    ['pl', 5, '5 nowych maili.'],
    ['uk', 1, '1 новий лист.'],
    ['uk', 21, '21 новий лист.'],
    ['uk', 3, '3 нові листи.'],
    ['uk', 11, '11 нових листів.'],
  ])('App, %s: %i', async (sprache, anzahl, erwartet) => {
    const { i18n: app, setLocale } = await import('./index');
    await setLocale(sprache);

    expect(app.global.t('app.postfach.neueMails', anzahl)).toBe(erwartet);
  });
});

describe('Sprache beim Start', () => {
  it.each([
    [null, ['de-DE'], 'de'],
    [null, ['it-IT', 'de-DE'], 'de'],
    [null, ['it-IT'], 'en'],
    [null, ['tr'], 'en'],
    [null, [], 'en'],
    [null, ['UK-ua'], 'uk'],
    ['fr', ['de-DE'], 'fr'],
    ['xx', ['pl-PL'], 'pl'],
  ])('gewählt %s, Gerät %j → %s', async (gewaehlt, geraet, erwartet) => {
    const { waehleSprache } = await import('./index');
    expect(waehleSprache(gewaehlt, geraet)).toBe(erwartet);
  });
});
