import { describe, it, expect } from 'vitest';
import { vorlage } from './notizVorlage';

describe('vorlage', () => {
  it('maskiert den Titel und setzt Sprache und Silbentrennung', () => {
    const t = vorlage({ titel: 'Frist #1 [wichtig]', stand: '13.10.2026, 18:00', rumpf: 'Text', sprache: 'de-DE' });
    expect(t).toContain('lang: "de"');
    expect(t).toContain('hyphenate: true');
    expect(t).toContain('Frist \\#1 \\[wichtig\\]');
    expect(t).toContain('#set document(title: "Frist #1 [wichtig]")');
    expect(t.trim().endsWith('Text')).toBe(true);
  });
});
