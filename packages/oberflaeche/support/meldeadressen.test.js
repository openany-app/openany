import { describe, it, expect } from 'vitest';
import { MELDEPFADE, spracheAusPfad, meldeadresse } from './meldeadressen';

describe('Meldeadressen', () => {
  it('kennt eine eigene Adresse je Sprache', () => {
    expect(new Set(Object.values(MELDEPFADE)).size).toBe(7);
  });

  it('liest die Sprache aus dem Pfad', () => {
    expect(spracheAusPfad('/report')).toBe('en');
    expect(spracheAusPfad('/denuncia/')).toBe('pt');
    expect(spracheAusPfad('/denunciar')).toBe('es');
    expect(spracheAusPfad('/melden.html')).toBe('de');
    expect(spracheAusPfad('/unbekannt')).toBe('de');
  });

  it('baut die Adresse zur App-Sprache', () => {
    expect(meldeadresse('uk')).toBe('https://openany.de/skarga');
    expect(meldeadresse('pt-BR')).toBe('https://openany.de/denuncia');
    expect(meldeadresse('en', 'https://openany.de', 'widerspruch')).toBe('https://openany.de/report?art=widerspruch');
    expect(meldeadresse('xx')).toBe('https://openany.de/melden');
  });
});
