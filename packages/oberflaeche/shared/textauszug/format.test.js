import { describe, it, expect } from 'vitest';
import { formatVon } from './format';

describe('formatVon', () => {
  it('erkennt nach Typ und sonst nach Endung', () => {
    expect(formatVon('x.bin', 'application/pdf')).toBe('pdf');
    expect(formatVon('Notizen.MD', '')).toBe('text');
    expect(formatVon('Brief.docx', 'application/octet-stream')).toBe('docx');
    expect(formatVon('text.txt', 'text/plain; charset=utf-8')).toBe('text');
  });

  it('kennt Bilder und Fremdes nicht', () => {
    expect(formatVon('Foto.jpg', 'image/jpeg')).toBeNull();
    expect(formatVon('Tabelle.xlsx', '')).toBeNull();
    expect(formatVon('ohne Endung', null)).toBeNull();
  });
});
