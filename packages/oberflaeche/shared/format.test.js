import { describe, it, expect } from 'vitest';
import { formatBytes } from '@oberflaeche/shared/format';

describe('formatBytes', () => {
  it('gibt 0 B für 0', () => {
    expect(formatBytes(0)).toBe('0 B');
  });

  it('skaliert über die 1024er-Einheiten', () => {
    expect(formatBytes(1024)).toBe('1 KB');
    expect(formatBytes(1024 * 1024)).toBe('1 MB');
    expect(formatBytes(1024 * 1024 * 1024)).toBe('1 GB');
  });

  it('rundet auf zwei Nachkommastellen', () => {
    expect(formatBytes(1536)).toBe('1.5 KB');
    expect(formatBytes(1234567)).toBe('1.18 MB');
  });
});
