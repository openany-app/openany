import { describe, it, expect } from 'vitest';
import { aufnahmezeit, nachAufnahme, nachMonaten } from './aufnahme';

const exif = (id, date, created_at = '2026-10-02T10:00:00Z') => ({ id, created_at, custom_properties: { exif: { date } } });

describe('Galerie nach Aufnahmedatum', () => {
  it('nimmt das EXIF-Datum, sonst das Hochladen', () => {
    expect(new Date(aufnahmezeit(exif(1, '2026:08:15 14:03:00'))).getMonth()).toBe(7);
    expect(aufnahmezeit({ id: 2, created_at: '2026-10-02T10:00:00Z' })).toBe(Date.parse('2026-10-02T10:00:00Z'));
    // Kaputtes EXIF („0000:00:00") zählt nicht.
    expect(aufnahmezeit(exif(3, '0000:00:00 00:00:00'))).toBe(Date.parse('2026-10-02T10:00:00Z'));
    expect(aufnahmezeit({})).toBe(0);
  });

  it('alte Fotos, heute hochgeladen, stehen bei ihrem Monat', () => {
    const urlaub = exif('urlaub', '2025:07:20 12:00:00');
    const screenshot = { id: 'neu', created_at: '2026-10-01T09:00:00Z' };
    const fest = exif('fest', '2026:08:15 18:00:00');
    expect(nachAufnahme([urlaub, screenshot, fest]).map((b) => b.id)).toEqual(['neu', 'fest', 'urlaub']);

    const monate = nachMonaten([urlaub, screenshot, fest, exif('fest2', '2026:08:02 10:00:00')], 'de');
    expect(monate.map((m) => m.titel)).toEqual(['Oktober 2026', 'August 2026', 'Juli 2025']);
    expect(monate[1].bilder.map((b) => b.id)).toEqual(['fest', 'fest2']);
  });
});
