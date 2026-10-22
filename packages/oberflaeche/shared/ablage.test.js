import { describe, it, expect } from 'vitest';
import { freierName } from './ablage';

describe('freierName', () => {
  it('behält jede Fassung: Titel.pdf, Titel (2).pdf, Titel (3).pdf', () => {
    expect(freierName([], 'Titel.pdf')).toBe('Titel.pdf');
    expect(freierName(['Titel.pdf'], 'Titel.pdf')).toBe('Titel (2).pdf');
    expect(freierName(['titel.PDF', 'Titel (2).pdf'], 'Titel.pdf')).toBe('Titel (3).pdf');
    expect(freierName(['Ohne'], 'Ohne')).toBe('Ohne (2)');
  });
});
