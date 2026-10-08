import { describe, it, expect, vi, beforeEach } from 'vitest';

// Die vier Nachbarn werden ersetzt: Canvas, WASM und HTTP haben in der
// node-Umgebung nichts verloren. Geprüft wird der Ablauf – wer wann
// hochgeladen wird, und was beim Abbruch passiert.
// Die Datenquelle wird übergeben (Webapp-API oder lokale Ablage im Programm).
const uploadFile = vi.fn().mockResolvedValue({ data: {} });
const quelle = { uploadFile: (...a) => uploadFile(...a) };

const prepareForOcr = vi.fn();
vi.mock('./ocr/prepareImage', () => ({
  prepareForOcr: (...a) => prepareForOcr(...a),
  freigeben: vi.fn(),
}));

const recognizeWords = vi.fn();
const releaseOcr = vi.fn().mockResolvedValue(undefined);
vi.mock('./ocr/recognize', () => ({
  recognizeWords: (...a) => recognizeWords(...a),
  ocrMoeglich: () => true,
  releaseOcr: (...a) => releaseOcr(...a),
}));

const buildPdf = vi.fn(async (seiten) => new Blob([`pdf:${seiten.length}`]));
vi.mock('./ocr/buildPdf', () => ({ buildPdf: (...a) => buildPdf(...a) }));

const { useDocumentIntake } = await import('./useDocumentIntake');

const foto = (name) => ({ name, type: 'image/jpeg' });

beforeEach(() => {
  vi.clearAllMocks();
  // Arbeitsbild groesser als die sichtbare Ebene – so sieht es echt aus.
  prepareForOcr.mockResolvedValue({
    grau: { width: 3000, height: 4200 },
    farbe: { width: 2000, height: 2800 },
    wordsWidth: 3000,
    wordsHeight: 4200,
  });
  recognizeWords.mockResolvedValue({ words: [{ text: 'Rechnung', x0: 1, y0: 1, x1: 9, y1: 9 }], text: 'Rechnung' });
});

describe('istBild', () => {
  it('erkennt Fotos und lässt andere Dokumente in Ruhe', () => {
    const { istBild } = useDocumentIntake(quelle);
    expect(istBild({ type: 'image/jpeg' })).toBe(true);
    expect(istBild({ type: 'image/heic' })).toBe(true);
    expect(istBild({ type: 'application/pdf' })).toBe(false);
    expect(istBild({})).toBe(false);
  });
});

describe('verarbeite', () => {
  it('macht aus einem Foto ein PDF mit passendem Namen', async () => {
    const { verarbeite } = useDocumentIntake(quelle);
    const ergebnis = await verarbeite([foto('rechnung.jpg')], 7, {});

    expect(ergebnis).toEqual({ abgelegt: 1, ohneText: 0 });
    expect(uploadFile).toHaveBeenCalledTimes(1);
    const [parentId, datei, zone] = uploadFile.mock.calls[0];
    expect(parentId).toBe(7);
    expect(datei.name).toBe('rechnung.pdf');
    expect(datei.type).toBe('application/pdf');
    expect(zone).toBe('documents');
  });

  it('legt einzeln je Foto ein eigenes Dokument ab', async () => {
    const { verarbeite } = useDocumentIntake(quelle);
    const ergebnis = await verarbeite([foto('a.jpg'), foto('b.jpg'), foto('c.jpg')], null, { zusammen: false });

    expect(ergebnis.abgelegt).toBe(3);
    expect(uploadFile.mock.calls.map((c) => c[1].name)).toEqual(['a.pdf', 'b.pdf', 'c.pdf']);
  });

  it('bindet mehrere Fotos zu einem Dokument unter dem gewählten Namen', async () => {
    const { verarbeite } = useDocumentIntake(quelle);
    const ergebnis = await verarbeite([foto('a.jpg'), foto('b.jpg')], null, { zusammen: true, name: 'Kündigung' });

    expect(ergebnis.abgelegt).toBe(1);
    expect(uploadFile).toHaveBeenCalledTimes(1);
    expect(uploadFile.mock.calls[0][1].name).toBe('Kündigung.pdf');
  });

  // Der Kern der Sache: Die Texterkennung darf ausfallen, ohne dass der
  // Nutzer sein Dokument verliert – aber er muss es erfahren.
  it('legt das PDF auch ab, wenn die Texterkennung ausfällt', async () => {
    recognizeWords.mockResolvedValue(null);
    const { verarbeite } = useDocumentIntake(quelle);
    const ergebnis = await verarbeite([foto('rechnung.jpg')], null, {});

    expect(ergebnis).toEqual({ abgelegt: 1, ohneText: 1 });
    expect(uploadFile).toHaveBeenCalledTimes(1);
  });

  it('meldet auch beim mehrseitigen Dokument, wenn kein Text drin ist', async () => {
    recognizeWords.mockResolvedValue(null);
    const { verarbeite } = useDocumentIntake(quelle);
    const ergebnis = await verarbeite([foto('a.jpg'), foto('b.jpg')], null, { zusammen: true, name: 'Brief' });

    expect(ergebnis).toEqual({ abgelegt: 1, ohneText: 1 });
  });

  // Die Wortkaesten zaehlen in Pixeln des Arbeitsbildes, das absichtlich
  // groesser ist als die sichtbare Ebene. Geht diese Angabe unterwegs
  // verloren, liegt die Textebene im PDF daneben – und zwar unsichtbar.
  it('reicht den Bezugsrahmen der Wortkästen an buildPdf durch', async () => {
    const { verarbeite } = useDocumentIntake(quelle);
    await verarbeite([foto('a.jpg')], null, {});

    const seiten = buildPdf.mock.calls[0][0];
    expect(seiten[0].wordsWidth).toBe(3000);
    expect(seiten[0].wordsHeight).toBe(4200);
    expect(seiten[0].canvas.width).toBe(2000); // sichtbar bleibt kleiner
  });

  it('räumt den Fortschritt am Ende ab', async () => {
    const { verarbeite, stand } = useDocumentIntake(quelle);
    await verarbeite([foto('a.jpg')], null, {});
    expect(stand.value).toBeNull();
  });

  it('reicht einen Upload-Fehler (z. B. Speicherlimit) nach oben durch', async () => {
    uploadFile.mockRejectedValueOnce({ response: { data: { message: 'Speicherlimit überschritten.' } } });
    const { verarbeite, stand } = useDocumentIntake(quelle);

    await expect(verarbeite([foto('a.jpg')], null, {})).rejects.toMatchObject({
      response: { data: { message: 'Speicherlimit überschritten.' } },
    });
    // Auch im Fehlerfall darf kein Dialog stehen bleiben.
    expect(stand.value).toBeNull();
  });
});

describe('abbrechen', () => {
  // Der Abbruch beendet den Worker. recognizeWords meldet das – seiner
  // Aufgabe entsprechend – als "Erkennung ausgefallen", nicht als Fehler.
  // Ohne die Abbruch-Prüfung danach entstünde daraus ein stillschweigend
  // abgelegtes PDF ohne Textebene: genau das, was niemand bestellt hat.
  it('lädt nichts hoch, wenn während der Erkennung abgebrochen wird', async () => {
    const { verarbeite, abbrechen } = useDocumentIntake(quelle);

    recognizeWords.mockImplementation(async () => {
      await abbrechen();
      return null; // wie nach einem terminate()
    });

    const ergebnis = await verarbeite([foto('a.jpg')], null, {});

    expect(releaseOcr).toHaveBeenCalled();
    expect(uploadFile).not.toHaveBeenCalled();
    expect(ergebnis.abgelegt).toBe(0);
  });

  it('behält beim Einzel-Weg, was schon fertig abgelegt war', async () => {
    const { verarbeite, abbrechen } = useDocumentIntake(quelle);
    let durchlauf = 0;

    recognizeWords.mockImplementation(async () => {
      durchlauf += 1;
      if (durchlauf === 2) { await abbrechen(); return null; }
      return { words: [{ text: 'x', x0: 1, y0: 1, x1: 9, y1: 9 }], text: 'x' };
    });

    const ergebnis = await verarbeite([foto('a.jpg'), foto('b.jpg'), foto('c.jpg')], null, { zusammen: false });

    expect(ergebnis.abgelegt).toBe(1);
    expect(uploadFile.mock.calls.map((c) => c[1].name)).toEqual(['a.pdf']);
  });
});
