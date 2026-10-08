import { describe, it, expect } from 'vitest';
import { normalizeLinkUrl } from './linkUrl';

describe('normalizeLinkUrl', () => {
  it('ergaenzt https:// bei einer nackten Domain', () => {
    expect(normalizeLinkUrl('example.de')).toBe('https://example.de');
    expect(normalizeLinkUrl('example.de/seite?a=1')).toBe('https://example.de/seite?a=1');
  });

  it('laesst eine vollstaendige Adresse in Ruhe', () => {
    // Der eigentliche Fehler: Das Eingabefeld war mit 'https://' vorbelegt,
    // eine eingefuegte Adresse wurde dadurch zu 'https://https://…'.
    expect(normalizeLinkUrl('https://example.de')).toBe('https://example.de');
    expect(normalizeLinkUrl('http://example.de')).toBe('http://example.de');
  });

  it('erzeugt niemals ein doppeltes Schema', () => {
    for (const eingabe of ['https://example.de', 'http://x.de', 'mailto:a@b.de']) {
      expect(normalizeLinkUrl(eingabe).match(/https?:\/\//g)?.length ?? 0).toBeLessThan(2);
    }
  });

  it('respektiert andere Schemata', () => {
    expect(normalizeLinkUrl('mailto:tiffy@example.de')).toBe('mailto:tiffy@example.de');
    expect(normalizeLinkUrl('tel:+4930123456')).toBe('tel:+4930123456');
  });

  it('laesst interne Pfade und Anker unangetastet', () => {
    // So liegen eingefuegte Notiz-Anhaenge vor.
    expect(normalizeLinkUrl('/api/notes/1/assets/skizze.pdf')).toBe('/api/notes/1/assets/skizze.pdf');
    expect(normalizeLinkUrl('#abschnitt')).toBe('#abschnitt');
  });

  it('schneidet Leerraum ab und behandelt Leeres als leer', () => {
    expect(normalizeLinkUrl('  example.de  ')).toBe('https://example.de');
    expect(normalizeLinkUrl('   ')).toBe('');
    expect(normalizeLinkUrl('')).toBe('');
    expect(normalizeLinkUrl(null)).toBe('');
    expect(normalizeLinkUrl(undefined)).toBe('');
  });
});
