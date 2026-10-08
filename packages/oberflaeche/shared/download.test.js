import { describe, it, expect, afterEach } from 'vitest';
import { sanitizeFilename, triggerDownload, setDownloadSigner } from '@oberflaeche/shared/download';

describe('triggerDownload', () => {
  afterEach(() => setDownloadSigner(null));

  // Die Tests laufen ohne DOM: ein Dokument, das nur kann, was
  // triggerDownload braucht, und mitschreibt, welcher Link geklickt wurde.
  function mitschneiden() {
    const geklickt = [];
    const alt = globalThis.document;
    globalThis.document = {
      createElement: () => ({
        href: '',
        setAttribute() {},
        click() { geklickt.push(this.href); },
        remove() {},
      }),
      body: { appendChild() {} },
    };
    return { geklickt, zurueck: () => { globalThis.document = alt; } };
  }

  it('nimmt ohne Signierer den Pfad, wie er ist', async () => {
    const m = mitschneiden();
    await triggerDownload('/api/files/1/download', 'a.apk');
    m.zurueck();
    expect(m.geklickt).toEqual(['/api/files/1/download']);
  });

  it('nimmt mit Signierer den signierten Link', async () => {
    setDownloadSigner(async (pfad) => `${pfad}?dl_sig=abc`);
    const m = mitschneiden();
    await triggerDownload('/api/files/1/download', 'a.apk');
    m.zurueck();
    expect(m.geklickt).toEqual(['/api/files/1/download?dl_sig=abc']);
  });

  it('lädt trotzdem, wenn das Signieren scheitert', async () => {
    setDownloadSigner(async () => { throw new Error('offline'); });
    const m = mitschneiden();
    await triggerDownload('/api/files/1/download', 'a.apk');
    m.zurueck();
    expect(m.geklickt).toEqual(['/api/files/1/download']);
  });
});

describe('sanitizeFilename', () => {
  it('lässt harmlose Namen unverändert', () => {
    expect(sanitizeFilename('Rezepte', 'fallback')).toBe('Rezepte');
  });

  it('entfernt Pfad- und Sonderzeichen', () => {
    expect(sanitizeFilename('a/b\\c:d*e?f"g<h>i|j%k', 'fallback')).toBe('abcdefghijk');
  });

  it('trimmt und kollabiert mehrfache Leerzeichen', () => {
    expect(sanitizeFilename('  viel   Platz  ', 'fallback')).toBe('viel Platz');
  });

  it('entfernt Steuerzeichen (z. B. Tabs) ganz', () => {
    expect(sanitizeFilename('a\tb', 'fallback')).toBe('ab');
  });

  it('begrenzt die Länge auf 120 Zeichen', () => {
    expect(sanitizeFilename('x'.repeat(200), 'fallback')).toHaveLength(120);
  });

  it('fällt auf den Fallback zurück, wenn nichts übrig bleibt', () => {
    expect(sanitizeFilename('///', 'notiz-42')).toBe('notiz-42');
    expect(sanitizeFilename('', 'notiz-42')).toBe('notiz-42');
    expect(sanitizeFilename(null, 'notiz-42')).toBe('notiz-42');
  });
});
