import { describe, it, expect } from 'vitest';
import { useSpeicherStart, speicherStartSpeichernMit } from './useSpeicherStart';

describe('useSpeicherStart', () => {
  it('beginnt mit den Dokumenten und nimmt nur bekannte Reiter', async () => {
    const { speicherStart, setSpeicherStart, uebernehmeVomServer } = useSpeicherStart();
    expect(speicherStart.value).toBe('documents');

    uebernehmeVomServer('papierkorb');
    expect(speicherStart.value).toBe('documents');

    await setSpeicherStart('gallery');
    expect(speicherStart.value).toBe('gallery');
  });

  it('übernimmt den Wert, den der Server bestätigt', async () => {
    speicherStartSpeichernMit(async () => 'files');
    const { speicherStart, setSpeicherStart } = useSpeicherStart();
    await setSpeicherStart('documents');
    expect(speicherStart.value).toBe('files');
  });

  it('fällt zurück, wenn das Speichern scheitert', async () => {
    speicherStartSpeichernMit(async () => { throw new Error('offline'); });
    const { speicherStart, setSpeicherStart } = useSpeicherStart();
    const vorher = speicherStart.value;
    await expect(setSpeicherStart(vorher === 'gallery' ? 'files' : 'gallery')).rejects.toThrow('offline');
    expect(speicherStart.value).toBe(vorher);
    speicherStartSpeichernMit(null);
  });
});
