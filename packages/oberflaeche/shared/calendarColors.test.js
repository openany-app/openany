import { describe, it, expect } from 'vitest';
import { PRESET_COLORS, DEFAULT_CALENDAR_COLOR } from './calendarColors';

describe('calendarColors', () => {
  it('leitet die Vorgabefarbe aus der Liste ab', () => {
    // DEFAULT_CALENDAR_COLOR wird per .find(…).value gebildet und wuerde
    // beim Umbenennen des Eintrags eine TypeError werfen. Dieser Test faengt
    // das ab, bevor der Anlege-Dialog beim Oeffnen kaputtgeht.
    expect(DEFAULT_CALENDAR_COLOR).toBeTruthy();
    expect(PRESET_COLORS.map((c) => c.value)).toContain(DEFAULT_CALENDAR_COLOR);
  });

  it('haelt die Auswahl frei von Doppelungen', () => {
    const werte = PRESET_COLORS.map((c) => c.value);
    expect(new Set(werte).size).toBe(werte.length);
  });

  it('fuehrt nur vollstaendige Hex-Werte', () => {
    // Die Werte landen direkt in :style-Bindungen und in der Datenbank.
    for (const c of PRESET_COLORS) {
      expect(c.value).toMatch(/^#[0-9a-f]{6}$/);
      expect(c.label.trim()).not.toBe('');
    }
  });
});
