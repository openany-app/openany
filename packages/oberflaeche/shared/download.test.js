import { describe, it, expect } from 'vitest';
import { sanitizeFilename } from '@oberflaeche/shared/download';

describe('sanitizeFilename', () => {
  it('lässt harmlose Namen unverändert', () => {
    expect(sanitizeFilename('Rezepte', 'fallback')).toBe('Rezepte');
  });

  it('entfernt Pfad- und Sonderzeichen', () => {
    expect(sanitizeFilename('a/b\\c:d*e?f"g<h>i|j%k', 'fallback')).toBe('abcdefghijk');
  });

  it('trimmt und kollabiert mehrfache Leerzeichen', () => {
    expect(sanitizeFilename('  viel   Platz  ', 'fallback')).toBe('viel Platz');
  });

  it('entfernt Steuerzeichen (z. B. Tabs) ganz', () => {
    expect(sanitizeFilename('a\tb', 'fallback')).toBe('ab');
  });

  it('begrenzt die Länge auf 120 Zeichen', () => {
    expect(sanitizeFilename('x'.repeat(200), 'fallback')).toHaveLength(120);
  });

  it('fällt auf den Fallback zurück, wenn nichts übrig bleibt', () => {
    expect(sanitizeFilename('///', 'notiz-42')).toBe('notiz-42');
    expect(sanitizeFilename('', 'notiz-42')).toBe('notiz-42');
    expect(sanitizeFilename(null, 'notiz-42')).toBe('notiz-42');
  });
});
