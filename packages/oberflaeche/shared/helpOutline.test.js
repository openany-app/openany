import { describe, it, expect } from 'vitest';
import { HELP_OUTLINE, hilfeGliederung } from '@oberflaeche/shared/helpOutline';
import de from '@oberflaeche/i18n/locales/de/help';
import en from '@oberflaeche/i18n/locales/en/help';
import es from '@oberflaeche/i18n/locales/es/help';
import fr from '@oberflaeche/i18n/locales/fr/help';
import pl from '@oberflaeche/i18n/locales/pl/help';
import pt from '@oberflaeche/i18n/locales/pt/help';
import uk from '@oberflaeche/i18n/locales/uk/help';

/**
 * Die Hilfeseite baut ihre Schlüssel aus der Gliederung zusammen. Fehlt einer,
 * gibt es keinen Fehler – vue-i18n zeigt dann die rohe Schlüsselzeile
 * („help.sections.projects.subs.chat.p3") mitten im Handbuch. Genau das kann
 * passieren, weil ein neuer Absatz drei Dateien braucht: die Gliederung und
 * beide Sprachen.
 */
const sprachen = { de, en, es, fr, pl, pt, uk };

/** Alle Schlüssel, die die Seite anfragen wird. */
const erwarteteSchluessel = () => {
  const keys = [];
  for (const s of HELP_OUTLINE) {
    keys.push(`sections.${s.id}.title`);
    for (const sub of s.subs) {
      keys.push(`sections.${s.id}.subs.${sub.id}.title`);
      for (let i = 1; i <= sub.p; i++) keys.push(`sections.${s.id}.subs.${sub.id}.p${i}`);
    }
  }

  return keys;
};

const hole = (obj, pfad) => pfad.split('.').reduce((o, teil) => (o == null ? undefined : o[teil]), obj);

/** Alle Blatt-Schlüssel, die in einer Sprachdatei tatsächlich stehen. */
const vorhandeneSchluessel = (obj, praefix = '') => {
  const keys = [];
  for (const [k, v] of Object.entries(obj)) {
    const pfad = praefix ? `${praefix}.${k}` : k;
    if (v && typeof v === 'object') keys.push(...vorhandeneSchluessel(v, pfad));
    else keys.push(pfad);
  }

  return keys;
};

describe('Handbuch-Gliederung', () => {
  for (const [name, texte] of Object.entries(sprachen)) {
    it(`hat für jeden angekündigten Absatz einen Text (${name})`, () => {
      const fehlend = erwarteteSchluessel().filter((k) => typeof hole(texte, k) !== 'string' || ! hole(texte, k).trim());

      expect(fehlend).toEqual([]);
    });

    it(`hat keine Texte, die die Seite nie zeigt (${name})`, () => {
      // Übrig gebliebene Absätze sind tote Übersetzung: Sie stehen in der
      // Datei, erscheinen aber nirgends – und verdecken beim nächsten Lesen,
      // dass ein Abschnitt gekürzt wurde.
      const erwartet = new Set([...erwarteteSchluessel(), 'title', 'intro', 'toc', 'version']);
      const uebrig = vorhandeneSchluessel(texte).filter((k) => ! erwartet.has(k));

      expect(uebrig).toEqual([]);
    });
  }

  it('beschreibt in allen Sprachen dasselbe Handbuch', () => {
    for (const texte of Object.values(sprachen)) {
      expect(vorhandeneSchluessel(texte).sort()).toEqual(vorhandeneSchluessel(de).sort());
    }
  });
});

describe('Hilfe je Rahmen', () => {
  const ids = (gliederung) => gliederung.flatMap((s) => s.subs.map((sub) => `${s.id}.${sub.id}`));
  const nur = (wo) => HELP_OUTLINE.flatMap((s) => s.subs
    .filter((sub) => s.nur === wo || sub.nur === wo)
    .map((sub) => `${s.id}.${sub.id}`));

  it('zeigt im Programm alles außer dem, was nur die Webapp betrifft', () => {
    expect(ids(hilfeGliederung({ programm: true })))
      .toEqual(ids(HELP_OUTLINE).filter((id) => !nur('web').includes(id)));
  });

  it('zeigt in der Webapp alles außer dem, was nur das Programm kann', () => {
    expect(ids(hilfeGliederung()))
      .toEqual(ids(HELP_OUTLINE).filter((id) => !nur('app').includes(id)));
  });

  it('kennt nur die Marken app und web', () => {
    const marken = HELP_OUTLINE.flatMap((s) => [s.nur, ...s.subs.map((sub) => sub.nur)]).filter(Boolean);
    expect(marken.filter((m) => !['app', 'web'].includes(m))).toEqual([]);
  });
});
