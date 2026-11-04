import { describe, expect, it } from 'vitest';
import { istTon, spitzen, tonMime } from './ton';

describe('Ton erkennen', () => {
  it('nach dem Typ', () => {
    expect(istTon({ type: 'audio/mpeg', name: 'a.mp3' })).toBe(true);
    expect(istTon({ mime_type: 'audio/mp4' })).toBe(true);
    expect(istTon({ type: 'video/mp4', name: 'a.m4a' })).toBe(false);
  });

  it('nach der Endung, wenn kein Typ gemeldet wird', () => {
    expect(istTon({ type: '', name: 'Gespräch.OPUS' })).toBe(true);
    expect(istTon({ type: '', name: 'bericht.pdf' })).toBe(false);
    expect(tonMime('a.m4a')).toBe('audio/mp4');
    expect(tonMime('a.pdf')).toBe('');
  });
});

describe('Wellenform', () => {
  it('nimmt je Abschnitt die lauteste Stelle, bezogen auf die ganze Datei', () => {
    const kanal = Float32Array.from([0.1, -0.5, 0, 0.25]);
    expect(spitzen([kanal], 2)).toEqual([1, 0.5]);
  });

  it('bleibt bei Stille und ohne Daten flach', () => {
    expect(spitzen([new Float32Array(4)], 2)).toEqual([0, 0]);
    expect(spitzen([], 2)).toEqual([0, 0]);
  });
});
