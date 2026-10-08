import { describe, it, expect } from 'vitest';
import { mapSaveError, AUTOSAVE_MS } from '@oberflaeche/composables/useNoteAutosave';

// Die Deutung eines fehlgeschlagenen Speicherversuchs ist der einzige Teil des
// Autosave, in dem stilles Weitermachen Daten kostet — deshalb steht sie als
// eigene Funktion da und wird hier festgenagelt.
describe('mapSaveError', () => {
  const err = (status, data) => ({ response: { status, data } });

  it('erkennt die Fremdaenderung (409) und meldet sie als solche', () => {
    // Ohne dieses Flag laesst der naechste Tastendruck die fremde Fassung
    // ueberschreiben — der Grund, warum 409 nicht wie ein Fehler unter vielen
    // behandelt werden darf.
    expect(mapSaveError(err(409, { message: 'Datei wurde extern geaendert' }))).toEqual({
      external: true,
      message: 'Datei wurde extern geaendert',
    });
  });

  it('nimmt bei 409 auch eine fehlende Meldung hin', () => {
    expect(mapSaveError(err(409, {}))).toEqual({ external: true, message: '' });
  });

  it('zeigt den Validierungsfehler aus errors.content (422, z. B. 1-MB-Limit)', () => {
    const e = err(422, { errors: { content: ['Notiz zu gross'] }, message: 'Ungueltig' });

    expect(mapSaveError(e)).toEqual({ external: false, message: 'Notiz zu gross' });
  });

  it('faellt bei 422 ohne Feldfehler auf die allgemeine Meldung zurueck', () => {
    expect(mapSaveError(err(422, { message: 'Ungueltig' }))).toEqual({
      external: false,
      message: 'Ungueltig',
    });
  });

  it('laesst andere Fehler ohne Zusatztext (der Status genuegt der Anzeige)', () => {
    expect(mapSaveError(err(500, { message: 'Serverfehler' }))).toEqual({
      external: false,
      message: '',
    });
  });

  it('kommt mit einem Fehler ohne response klar (Netzwerkabbruch)', () => {
    expect(mapSaveError(new Error('Network Error'))).toEqual({ external: false, message: '' });
    expect(mapSaveError(undefined)).toEqual({ external: false, message: '' });
  });
});

describe('AUTOSAVE_MS', () => {
  it('bleibt bei 10 Sekunden', () => {
    // Bewusst traeger als die urspruenglichen 2 Sekunden: Seit Notizen als
    // Dateien liegen, schreibt jede Ausloesung die ganze Datei samt fsync.
    // Gegen Verlust bei laengerem Takt sichern die Sofort-Speicherungen beim
    // Tab-Wechsel und beim Verlassen der Seite.
    expect(AUTOSAVE_MS).toBe(10000);
  });
});
