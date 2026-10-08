// Texterkennung im Browser. Tesseract laeuft als WebAssembly in einem Worker;
// hier steht nur, wie er geholt, am Leben gehalten und wieder losgelassen wird.
//
// Drei Dinge sind hier wichtiger als sie aussehen:
//
// 1. ALLES wird lazy geladen. Der Import steht in der Funktion, nicht oben –
//    wer nie ein Foto in eine Akte legt, laedt keine drei Megabyte WASM.
//
// 2. NICHTS kommt von einem CDN. tesseract.js wuerde Worker, Core und
//    Sprachdaten sonst von jsdelivr ziehen: ein externer Abruf pro Nutzer, und
//    unsere CSP (default-src 'self') verbietet ihn ohnehin. Die Dateien liegen
//    unter /tesseract/ und kommen von scripts/fetch-tesseract-assets.sh.
//
// 3. Ein Fehler hier darf NIE einen Upload verhindern. Die Texterkennung ist
//    eine Zugabe: geht sie schief, entsteht das PDF trotzdem, nur ohne
//    durchsuchbaren Text. Deshalb gibt recognizeWords im Zweifel `null` zurueck
//    und wirft nicht.

/**
 * Die Sprachen der Oberfläche und ihre Tesseract-Namen. Mitgeliefert werden
 * alle sieben (scripts/fetch-tesseract-assets.sh); GELADEN werden je Lauf nur
 * zwei -- die der Oberfläche und Englisch.
 *
 * NICHT ALLE SIEBEN AUF EINMAL. Tesseract prüft jedes Wort gegen jedes
 * geladene Modell: sieben Modelle sind siebenmal so viel Speicher, spürbar
 * langsamer, und die Trefferquote SINKT, weil ähnliche Schriften (Polnisch,
 * Deutsch) einander Wörter wegschnappen. Englisch kommt immer dazu, weil in
 * fast jedem Schreiben englische Wörter, Marken oder Adressen stehen.
 */
const TESSERACT_NAMEN = { de: 'deu', en: 'eng', es: 'spa', fr: 'fra', pl: 'pol', pt: 'por', uk: 'ukr' };

/** Welche Modelle für eine Oberflächensprache (`de`, `pt-BR` …) geladen werden. */
export function sprachenFuer(locale) {
  const eigene = TESSERACT_NAMEN[String(locale ?? '').slice(0, 2).toLowerCase()] ?? 'deu';
  return eigene === 'eng' ? ['eng'] : [eigene, 'eng'];
}

/** Ohne Angabe wie bisher: Deutsch und Englisch. */
const STANDARD = ['deu', 'eng'];

/**
 * Woerter unterhalb dieser Sicherheit fliegen raus. Tesseract vergibt auf
 * Bildrauschen und Papierkanten gern Treffer wie „|" oder „,,"; die im PDF zu
 * haben ist schlechter als sie wegzulassen, weil sie beim Markieren mitspringen.
 */
const MIN_SICHERHEIT = 30;

/**
 * Der Worker ueberlebt mehrere Fotos – die Sprachdaten laedt man einmal.
 * Wechselt die Sprache (jemand stellt die Oberflaeche um), entsteht ein neuer.
 */
let workerPromise = null;
let workerSprachen = '';
let meldeFortschritt = () => {};

async function holeWorker(sprachen) {
  const schluessel = sprachen.join('+');
  if (workerPromise && workerSprachen === schluessel) return workerPromise;
  if (workerPromise) await releaseOcr();
  workerSprachen = schluessel;

  workerPromise = (async () => {
    const { createWorker } = await import('tesseract.js');
    return createWorker(sprachen, 1 /* OEM.LSTM_ONLY */, {
      // Direkt statt ueber eine blob:-URL: so genuegt worker-src 'self'.
      workerBlobURL: false,
      workerPath: '/tesseract/worker.min.js',
      // Verzeichnis, kein einzelnes File: tesseract.js sucht sich die Variante
      // passend zur SIMD-Faehigkeit des Geraets selbst aus.
      corePath: '/tesseract/',
      langPath: '/tesseract/lang',
      // Wir legen die .traineddata unkomprimiert ab. Sonst schickt Caddy sein
      // eigenes Content-Encoding, der Browser packt bereits aus, und
      // tesseract.js versucht ein zweites Mal zu entpacken.
      gzip: false,
      logger: (m) => meldeFortschritt(m),
    });
  })().catch((e) => {
    // Beim naechsten Foto neu versuchen statt dauerhaft kaputt zu bleiben.
    workerPromise = null;
    throw e;
  });

  return workerPromise;
}

/**
 * Steht die Texterkennung auf diesem Geraet ueberhaupt zur Verfuegung?
 * Ohne WebAssembly und Worker hat es keinen Zweck, 3 MB zu laden, um dann zu
 * scheitern – dann geht es direkt in den Weg ohne Textebene.
 */
export function ocrMoeglich() {
  return typeof WebAssembly === 'object' && typeof Worker === 'function';
}

/**
 * Erkennt Text auf einem vorbereiteten (grauen, geradegerueckten) Canvas.
 *
 * @returns {Promise<{words: Array, text: string} | null>}
 *          `null`, wenn die Erkennung nicht moeglich war oder schiefging –
 *          der Aufrufer baut dann ein PDF ohne Textebene.
 */
export async function recognizeWords(canvas, onProgress = () => {}, sprachen = STANDARD) {
  if (!ocrMoeglich()) return null;

  try {
    meldeFortschritt = onProgress;
    const worker = await holeWorker(sprachen?.length ? sprachen : STANDARD);

    // `blocks` ist per Voreinstellung aus; ohne die Anforderung bekaemen wir
    // nur den Fliesstext ohne Koordinaten – und ohne Koordinaten gibt es
    // keine Textebene, die auf den Woertern liegt.
    const { data } = await worker.recognize(canvas, {}, { text: true, blocks: true });

    return { words: sammleWoerter(data), text: data.text ?? '' };
  } catch (e) {
    console.warn('Texterkennung fehlgeschlagen – PDF entsteht ohne Textebene.', e);
    return null;
  } finally {
    meldeFortschritt = () => {};
  }
}

/**
 * Die Wortliste steckt drei Ebenen tief (Block > Absatz > Zeile > Wort).
 * Uns interessiert nur das Wort mit seinem Kasten; die Gliederung darueber
 * traegt fuer eine unsichtbare Textebene nichts bei.
 */
function sammleWoerter(page) {
  const woerter = [];
  for (const block of page?.blocks ?? []) {
    for (const absatz of block?.paragraphs ?? []) {
      for (const zeile of absatz?.lines ?? []) {
        for (const wort of zeile?.words ?? []) {
          const text = (wort?.text ?? '').trim();
          if (!text || (wort.confidence ?? 0) < MIN_SICHERHEIT) continue;
          const { x0, y0, x1, y1 } = wort.bbox ?? {};
          if (![x0, y0, x1, y1].every(Number.isFinite) || x1 <= x0 || y1 <= y0) continue;
          woerter.push({ text, x0, y0, x1, y1 });
        }
      }
    }
  }
  return woerter;
}

/**
 * Worker abbauen. Wird beim Abbrechen und beim Verlassen der Seite gerufen –
 * ein WASM-Worker mit geladenen Sprachdaten belegt sonst dauerhaft Speicher.
 */
export async function releaseOcr() {
  const laufend = workerPromise;
  workerPromise = null;
  if (!laufend) return;
  try {
    (await laufend).terminate();
  } catch {
    // Schon tot oder nie fertig geworden – beides in Ordnung.
  }
}
