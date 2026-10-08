// Nachtraegliche Texterkennung auf einem bereits abgelegten PDF.
//
// Der Anlass: Ein Scanner (oder eine Scanner-App) liefert ein PDF, in dem jede
// Seite nur ein Bild ist. Es sieht aus wie ein Dokument, laesst sich aber
// nicht durchsuchen, nicht markieren, nicht vorlesen. Genau die Luecke, die
// der Foto-Weg (useDocumentIntake) beim Hochladen schliesst – nur eben fuer
// alles, was schon liegt.
//
// Der wesentliche Unterschied zum Foto-Weg, und der Grund, warum das hier
// eigene Bausteine hat: Beim Foto entsteht ein NEUES PDF aus einem Bild. Hier
// gibt es schon eins. Es neu zu rastern, um Text darueberzulegen, hiesse einen
// 300-dpi-Scan gegen ein neu gerechnetes JPEG einzutauschen – sichtbar
// schlechter, und das Original waere weg. Stattdessen:
//
//   1. Das PDF holen und aufschlagen (pdfPages)
//   2. Prüfen, ob es schon Text hat – wenn ja, hier aufhoeren
//   3. Jede Seite rendern, nur damit Tesseract etwas zu lesen hat
//   4. Die Wortkaesten in PDF-Punkte umrechnen
//   5. Die Textebene in das UNVERAENDERTE Original schreiben (textLayer)
//   6. Den Inhalt des Knotens ersetzen – gleiche ID, gleicher Platz
//
// Was der Scan zeigt, aendert sich dabei um kein Pixel. Was dazukommt, sind
// ein paar Kilobyte Text, den man nicht sieht.
//
// ---------------------------------------------------------------------------
// Der zweite Weg: aufbereiten wie beim Foto
// ---------------------------------------------------------------------------
//
// Manche „Scans" sind gar keine, sondern abfotografierte Blaetter aus einer
// Handy-App. Die haben genau die Maengel, gegen die der Fotoweg gebaut ist:
// graues Papier, Schatten quer ueber die Seite. Dort waere es falsch, das
// Original zu schonen – da IST nichts zu schonen.
//
// Deshalb entscheidet nicht die Herkunft der Datei, sondern eine Messung an
// der ersten Seite (beurteileSeite):
//
//   sauber        -> Weg 1: Textebene ins Original, kein Pixel angefasst
//   grau/schattig -> Weg 2: Seiten aufhellen und neu aufbauen, wie beim Foto
//
// Das ist dieselbe Zurueckhaltung, die der Fotoweg schon kennt: detectCropRect
// schneidet nur, wenn es einen Rand findet. Gemessen statt geraten – denn der
// Preis von Weg 2 ist echt. An einem tatsaechlichen Flachbett-Scan
// nachgerechnet: 213 ppi Farb-JPEG, Papier schon bei reinweiss 255,
// Ungleichmaessigkeit 1 Stufe. Neu gerastert waeren daraus 150 dpi geworden –
// ein Drittel Aufloesung weg, fuer 10 % weniger Bytes und kein einziges
// sichtbar weisseres Pixel.

import { ref } from 'vue';
import { recognizeWords, ocrMoeglich, releaseOcr } from './ocr/recognize';
import {
  oeffnePdf, hatSchonText, renderSeite, platziereWoerter, gibSeiteFrei,
  beurteileSeite, anzeigeFassung, seitenFormat, pdfFreigeben,
} from './ocr/pdfPages';
import { schreibeTextebene } from './ocr/textLayer';
import { buildPdf } from './ocr/buildPdf';
import { glaetten, freigeben } from './ocr/prepareImage';

/**
 * Obergrenze. Jede Seite kostet auf einem Mittelklasse-Geraet fuenf bis
 * fuenfzehn Sekunden; ein 200-Seiten-Konvolut liefe eine Stunde und wuerde
 * dabei den Browser-Tab belegen. Eine ehrliche Absage ist besser als ein
 * Vorgang, den niemand zu Ende wartet.
 */
export const MAX_SEITEN = 50;

/**
 * @param {{ downloadFileContent: (id) => Promise<{data: ArrayBuffer}>,
 *           replaceFileContent: (id, datei) => Promise }} quelle
 *        Wie bei useDocumentIntake: Webapp-API oder lokale Ablage.
 * @param {{ sprachen?: () => string[] }} optionen
 */
export function useDocumentOcr(quelle, { sprachen } = {}) {
  // null = kein Vorgang. Sonst der Stand fuer das Fortschrittsfenster.
  const stand = ref(null);
  let abgebrochen = false;

  /** Nur PDFs – auf ein Bild oder eine Tabelle gaebe es nichts anzuwenden. */
  const istPdf = (item) => item?.type === 'pdf';

  function setze(teil) {
    if (stand.value) stand.value = { ...stand.value, ...teil };
  }

  /**
   * @returns {Promise<{status: string, woerter?: number, seiten?: number}>}
   *   'fertig'      – Textebene eingefuegt, Datei ersetzt
   *   'aufbereitet' – dasselbe, aber die Seiten wurden zusaetzlich aufgehellt
   *                   und neu aufgebaut (das Bild hat sich also geaendert)
   *   'schon-text'  – das PDF war bereits durchsuchbar, nichts geaendert
   *   'kein-text'   – nichts erkannt, nichts geaendert
   *   'zu-viele'    – ueber MAX_SEITEN, nichts geaendert
   *   'abgebrochen' – vom Nutzer beendet, nichts geaendert
   *
   * Vier von fünf Ausgängen lassen das Dokument unangetastet. Das ist
   * Absicht: Ersetzt wird nur, wenn wirklich etwas dazugekommen ist.
   */
  async function erkenne(item) {
    abgebrochen = false;
    stand.value = { schritt: 'fetch', nummer: 0, gesamt: 0, prozent: 0 };

    let pdf = null;
    try {
      const antwort = await quelle.downloadFileContent(item.id);
      // Zwei Kopien mit Absicht: pdf.js uebernimmt den Puffer und leert ihn
      // dabei. Das Original brauchen wir danach noch fuer die Textebene.
      const original = new Uint8Array(antwort.data);
      pdf = await oeffnePdf(original.slice().buffer);

      if (abgebrochen) return { status: 'abgebrochen' };

      if (pdf.numPages > MAX_SEITEN) {
        return { status: 'zu-viele', seiten: pdf.numPages };
      }

      setze({ schritt: 'render', gesamt: pdf.numPages });
      if (await hatSchonText(pdf)) {
        return { status: 'schon-text' };
      }

      // Weg 1 oder Weg 2? Entschieden wird EINMAL, an der ersten Seite, und
      // dann fuer das ganze Dokument. Je Seite neu zu entscheiden waere
      // genauer, ergaebe aber ein PDF, in dem manche Seiten neu gerastert sind
      // und andere nicht – sichtbar unterschiedlich, ohne dass jemand
      // erklaeren koennte, warum.
      setze({ schritt: 'render', nummer: 1 });
      const ersteSeite = await renderSeite(pdf, 1);
      const urteil = beurteileSeite(ersteSeite.canvas);
      const aufbereiten = urteil.lohnt;

      const textSeiten = [];   // Weg 1: Platzierungen in PDF-Punkten
      const bildSeiten = [];   // Weg 2: fertige Seiten fuer buildPdf
      const offen = [];        // alles, was noch Speicher haelt

      try {
        for (let n = 1; n <= pdf.numPages; n += 1) {
          if (abgebrochen) return { status: 'abgebrochen' };

          setze({ schritt: 'render', nummer: n, prozent: 0 });
          // Die erste Seite ist schon gerendert – dafuer noch einmal durch
          // pdf.js zu gehen waere reine Wartezeit.
          const { canvas, viewport } = n === 1 ? ersteSeite : await renderSeite(pdf, n);

          let woerter = null;
          if (ocrMoeglich()) {
            woerter = (await recognizeWords(canvas, (m) => {
              const schritt = m.status === 'recognizing text' ? 'recognize' : 'load';
              setze({ schritt, nummer: n, prozent: Math.round((m.progress ?? 0) * 100) });
            }, sprachen?.()))?.words ?? null;
          }

          // Wie im Foto-Weg: Ein Abbruch beendet den Worker, und recognizeWords
          // meldet das – seiner Aufgabe entsprechend – als „Erkennung
          // ausgefallen" statt als Fehler. Ohne diese Prüfung wuerde aus dem
          // Abbruch ein stillschweigend ersetztes Dokument ohne Textebene.
          if (abgebrochen) return { status: 'abgebrochen' };

          if (aufbereiten) {
            // Die sichtbare Ebene: kleiner als das Arbeitsbild und aufgehellt.
            // Genau die Reihenfolge des Fotowegs – erst verkleinern, dann
            // glaetten, damit die Kachelrechnung auf dem Bild laeuft, das
            // hinterher auch im PDF steht.
            const anzeige = glaetten(anzeigeFassung(canvas));
            offen.push(anzeige);
            bildSeiten.push({
              canvas: anzeige,
              words: woerter,
              wordsWidth: canvas.width,
              wordsHeight: canvas.height,
              format: seitenFormat(viewport),
            });
          } else if (woerter?.length) {
            textSeiten.push({ nummer: n, woerter: platziereWoerter(viewport, woerter) });
          }

          // Das Arbeitsbild hat seinen Zweck erfuellt. Bei Weg 2 haengt die
          // Anzeige-Fassung nicht daran – drawImage hat kopiert.
          if (canvas !== bildSeiten.at(-1)?.canvas) gibSeiteFrei(canvas);
        }

        if (abgebrochen) return { status: 'abgebrochen' };

        let blob;
        let woerter;

        if (aufbereiten) {
          if (!bildSeiten.some((seite) => seite.words?.length)) return { status: 'kein-text' };
          setze({ schritt: 'write', prozent: 100 });
          blob = await buildPdf(bildSeiten);
          woerter = bildSeiten.reduce((summe, seite) => summe + (seite.words?.length ?? 0), 0);
        } else {
          if (!textSeiten.length) return { status: 'kein-text' };
          setze({ schritt: 'write', prozent: 100 });
          ({ blob, woerter } = await schreibeTextebene(original, textSeiten));
          if (!woerter) return { status: 'kein-text' };
        }

        if (abgebrochen) return { status: 'abgebrochen' };

        setze({ schritt: 'save' });
        const datei = new File([blob], item.name, { type: 'application/pdf' });
        await quelle.replaceFileContent(item.id, datei);

        return {
          status: aufbereiten ? 'aufbereitet' : 'fertig',
          woerter,
          seiten: pdf.numPages,
        };
      } finally {
        offen.forEach(freigeben);
        gibSeiteFrei(ersteSeite.canvas);
      }
    } finally {
      // pdf.js haelt je Dokument einen Worker und die entpackten Bilddaten.
      try { await pdfFreigeben(pdf); } catch { /* schon zu */ }
      stand.value = null;
    }
  }

  /** Abbrechen: Der Tesseract-Worker wird abgeraeumt, nicht nur ignoriert. */
  async function abbrechen() {
    abgebrochen = true;
    await releaseOcr();
    stand.value = null;
  }

  return { stand, istPdf, erkenne, abbrechen };
}
