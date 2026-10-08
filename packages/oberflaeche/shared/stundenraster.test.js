import { describe, it, expect } from 'vitest';
import { raster, STUNDENRASTER, stundeZuZeit } from '@oberflaeche/shared/stundenraster';

describe('stundenraster – rechnen', () => {
  it('reiht Stunden mit der Vorgabe-Pause aneinander', () => {
    const r = raster({ start: '08:00', dauer: 45, pause: 5, anzahl: 3 });

    expect(r).toEqual([
      { from: '08:00', to: '08:45' },
      { from: '08:50', to: '09:35' },
      { from: '09:40', to: '10:25' },
    ]);
  });

  it('nimmt für einzelne Stellen eine längere Pause', () => {
    // Nach der ZWEITEN Stunde (Index 1) die große Pause.
    const r = raster({ start: '08:00', dauer: 45, pause: 5, pausen: { 1: 20 }, anzahl: 3 });

    expect(r[2]).toEqual({ from: '09:55', to: '10:40' });
  });

  it('führt über die volle Stunde hinweg richtig weiter', () => {
    const r = raster({ start: '11:50', dauer: 45, pause: 5, anzahl: 2 });

    expect(r).toEqual([
      { from: '11:50', to: '12:35' },
      { from: '12:40', to: '13:25' },
    ]);
  });
});

describe('stundenraster – Vorlagen', () => {
  it('liefert das klassische Raster mit acht Stunden ab acht', () => {
    const r = STUNDENRASTER.klassisch();

    expect(r).toHaveLength(8);
    expect(r[0]).toEqual({ from: '08:00', to: '08:45' });
    // Große Pause nach der zweiten Stunde.
    expect(r[2].from).toBe('09:55');
  });

  it('liefert Doppelstunden zu 90 Minuten', () => {
    const r = STUNDENRASTER.doppel();

    expect(r).toHaveLength(4);
    expect(r[0]).toEqual({ from: '08:00', to: '09:30' });
  });

  it('lässt keine Stunde rückwärts laufen', () => {
    for (const bauen of Object.values(STUNDENRASTER)) {
      const r = bauen();
      for (const [i, p] of r.entries()) {
        expect(p.from < p.to).toBe(true);
        if (i > 0) expect(r[i - 1].to <= p.from).toBe(true);
      }
    }
  });
});

describe('stundenraster – Zuordnung', () => {
  const periods = [{ from: '08:00', to: '08:45' }, { from: '08:50', to: '09:35' }];

  it('findet die Stunde zu einer Anfangszeit', () => {
    expect(stundeZuZeit(periods, '08:50')).toBe(1);
    // Der Server liefert Zeiten auch als HH:MM:SS.
    expect(stundeZuZeit(periods, '08:00:00')).toBe(0);
  });

  it('meldet null für eine Zeit außerhalb des Rasters', () => {
    expect(stundeZuZeit(periods, '10:15')).toBe(null);
    expect(stundeZuZeit(periods, null)).toBe(null);
    expect(stundeZuZeit(null, '08:00')).toBe(null);
  });
});
