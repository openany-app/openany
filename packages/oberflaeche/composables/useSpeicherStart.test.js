import { describe, it, expect } from 'vitest';
import { useSpeicherStart } from './useSpeicherStart';

describe('useSpeicherStart', () => {
  it('beginnt mit den Dokumenten und nimmt nur bekannte Reiter', () => {
    const { speicherStart, setSpeicherStart } = useSpeicherStart();
    expect(speicherStart.value).toBe('documents');

    setSpeicherStart('papierkorb');
    expect(speicherStart.value).toBe('documents');

    setSpeicherStart('gallery');
    expect(speicherStart.value).toBe('gallery');
  });
});
