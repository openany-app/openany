import { describe, it, expect } from 'vitest';
import { normal, passt } from './textsuche';

describe('textsuche', () => {
  it('findet Mueller, wenn jemand muller tippt', () => {
    expect(passt('Hannah Müller', 'muller')).toBe(true);
    expect(passt('Hannah Müller', 'Müller')).toBe(true);
  });

  it('nimmt Gross- und Kleinschreibung nicht ernst', () => {
    expect(passt('Hannah Müller', 'HANNAH')).toBe(true);
  });

  // Wer zwei Woerter tippt, will einschraenken -- nicht eine Wortfolge suchen.
  it('verlangt alle Woerter, aber in beliebiger Reihenfolge', () => {
    expect(passt('Hannah Müller', 'muller hannah')).toBe(true);
    expect(passt('Hannah Müller', 'hannah schmidt')).toBe(false);
  });

  // Sonst verschwaende die ganze Liste, sobald jemand ins Feld klickt und
  // wieder herausgeht.
  it('laesst bei leerer Suche alles durch', () => {
    expect(passt('Hannah Müller', '')).toBe(true);
    expect(passt('Hannah Müller', '   ')).toBe(true);
  });

  it('kommt mit fehlendem Text zurecht', () => {
    expect(passt(null, 'hannah')).toBe(false);
    expect(passt(undefined, '')).toBe(true);
  });

  // Æ ist ein eigener Buchstabe und keine Betonung -- die Grenze des
  // Verfahrens, und sie ist gewollt.
  it('nimmt nur Betonungszeichen weg, keine eigenen Buchstaben', () => {
    expect(normal('Müller')).toBe('muller');
    expect(normal('Ærø')).toBe('ærø');
  });
});
