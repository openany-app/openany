import { describe, it, expect } from 'vitest';
import { PLANNING_TYPES, PLANNING_CHAT_KINDS, REQUIRED_I18N_KEYS } from './planningTypes';
import de from '@oberflaeche/i18n/locales/de/projects';
import en from '@oberflaeche/i18n/locales/en/projects';
import dePlanning from '@oberflaeche/i18n/locales/de/planning';
import enPlanning from '@oberflaeche/i18n/locales/en/planning';

// Ein Eintrag hier trägt Versprechen an drei Stellen: einen i18n-Präfix, einen
// Vorlagen-Schlüssel im +Neu-Picker und eine Art im [[Verweis]]. Bricht eines
// davon, sieht man es erst in der laufenden Oberfläche – als leerer
// Übersetzungsschlüssel oder als toter Verweis.

const nachPfad = (objekt, pfad) => pfad.split('.').reduce((o, teil) => o?.[teil], objekt);

describe('PLANNING_TYPES', () => {
  it('führt jede Art genau einmal', () => {
    expect(new Set(PLANNING_CHAT_KINDS).size).toBe(PLANNING_CHAT_KINDS.length);
  });

  it.each(PLANNING_TYPES)('$kind hat alle Übersetzungen unter $i18n', (typ) => {
    // Der Präfix beginnt mit „projects." – die Datei IST dieser Namensraum.
    const pfad = typ.i18n.replace(/^projects\./u, '');

    for (const sprache of [de, en]) {
      const block = nachPfad(sprache, pfad);
      expect(block, `${typ.i18n} fehlt`).toBeTypeOf('object');

      for (const schluessel of REQUIRED_I18N_KEYS) {
        expect(block[schluessel], `${typ.i18n}.${schluessel} fehlt`).toBeTypeOf('string');
      }
    }
  });

  // Ein Behälter darf mehrere Kacheln anbieten (Stundenplan und
  // Betreuungsplan sind ein Typ) – dann braucht JEDE davon ihre Texte.
  it.each(PLANNING_TYPES)('$kind hat für jede Kachel eine Vorlage im +Neu-Picker', (typ) => {
    expect(typ.templateKeys.length, `${typ.kind} nennt keine Vorlage`).toBeGreaterThan(0);

    for (const sprache of [dePlanning, enPlanning]) {
      for (const schluessel of typ.templateKeys) {
        const vorlage = sprache.templates[schluessel];
        expect(vorlage?.label, `templates.${schluessel}.label fehlt`).toBeTypeOf('string');
        expect(vorlage?.hint, `templates.${schluessel}.hint fehlt`).toBeTypeOf('string');
        // Der Platzhalter im Anlegen-Formular hängt am Vorlagen-Schlüssel,
        // nicht am Typ: Ohne ihn stünde dort ein leeres Feld ohne Hinweis.
        expect(sprache.titlePlaceholder?.[schluessel], `titlePlaceholder.${schluessel} fehlt`).toBeTypeOf('string');
      }
    }
  });
});
