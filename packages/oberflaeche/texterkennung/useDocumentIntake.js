// Fotos, die in eine Akte gelegt werden, kommen dort als durchsuchbares PDF an
// – nie als Bild. Diese Datei haelt den Ablauf dafuer zusammen.
//
// Warum ueberhaupt im Browser und nicht auf dem Server: Das Foto ist hier schon
// im Speicher, die Rechenzeit kostet niemanden ausser dem eigenen Geraet, und
// das Bild muss fuer die Erkennung nirgends hingeschickt werden.
//
// Der Ablauf je Foto:
//
//   1. laden (EXIF-Drehung), Schiefwinkel messen
//   2. zweimal neu zeichnen – grau fuers Erkennen, farbig fuers PDF
//   3. Tesseract liefert Woerter samt Kaesten (darf ausfallen)
//   4. jsPDF baut Bild + unsichtbare Textebene
//   5. das PDF geht durch den ganz normalen Upload
//
// Punkt 3 ist ausdruecklich der einzige Schritt, der scheitern DARF. Faellt er
// aus – altes Geraet, kein WASM, Abbruch –, entsteht dasselbe PDF ohne
// Textebene. Der Nutzer bekommt sein Dokument, es ist nur nicht durchsuchbar,
// und der Hinweis darauf sagt das ausdruecklich. Was nie passiert: dass ein
// Foto stillschweigend gar nicht abgelegt wird.

import { ref } from 'vue';
import { prepareForOcr, freigeben } from './ocr/prepareImage';
import { recognizeWords, ocrMoeglich, releaseOcr } from './ocr/recognize';
import { buildPdf } from './ocr/buildPdf';

/** Endung weg – aus „rechnung.jpg" wird „rechnung.pdf". */
function ohneEndung(name) {
  return name.replace(/\.[^.]+$/, '') || name;
}

/**
 * @param {{ uploadFile: (ordner, datei, zone) => Promise }} quelle
 *        Die Datenquelle der Dateifläche -- in der Webapp die API, im
 *        Programm die lokale Ablage (app/src/quellen/dateien.js).
 * @param {{ sprachen?: () => string[] }} optionen
 *        Die Sprachen für die Erkennung, als Funktion: Sie folgen der Sprache
 *        der Oberfläche, und die kann sich zwischen zwei Fotos ändern.
 */
export function useDocumentIntake(quelle, { sprachen } = {}) {
  // null = kein Vorgang. Sonst der Stand, den das Modal anzeigt.
  const stand = ref(null);
  let abgebrochen = false;

  const istBild = (file) => (file?.type || '').startsWith('image/');

  function setze(teil) {
    if (stand.value) stand.value = { ...stand.value, ...teil };
  }

  /**
   * Ein Foto zu einer PDF-Seite verarbeiten.
   * Wirft nur, wenn schon das Bild nicht zu lesen war – Erkennungsfehler
   * kommen als `words: null` zurueck.
   */
  async function seiteAus(file, nummer, gesamt) {
    setze({ schritt: 'prepare', nummer, gesamt, prozent: 0 });
    const { grau, farbe, wordsWidth, wordsHeight } = await prepareForOcr(file);

    let words = null;
    if (ocrMoeglich()) {
      words = (await recognizeWords(grau, (m) => {
        // tesseract.js meldet mehrere Phasen; fuer den Nutzer sind nur zwei
        // unterscheidbar – „laedt noch" und „liest gerade".
        const schritt = m.status === 'recognizing text' ? 'recognize' : 'load';
        setze({ schritt, prozent: Math.round((m.progress ?? 0) * 100) });
      }, sprachen?.()))?.words ?? null;
    }

    // Das graue Arbeitsbild hat seinen Zweck erfuellt. Das farbige wird noch
    // gebraucht und erst nach dem PDF freigegeben.
    freigeben(grau);
    // wordsWidth/-Height muessen mit: Die Kaesten zaehlen in Pixeln des grauen
    // Arbeitsbildes, `farbe` ist kleiner (siehe prepareImage).
    return { canvas: farbe, words, wordsWidth, wordsHeight };
  }

  /** Seiten zu einem PDF binden und ablegen. Gibt zurueck, ob Text drin ist. */
  async function ablegen(seiten, name, parentId) {
    setze({ schritt: 'build', prozent: 100 });
    const blob = await buildPdf(seiten);

    setze({ schritt: 'save' });
    const datei = new File([blob], `${name}.pdf`, { type: 'application/pdf' });
    await quelle.uploadFile(parentId, datei, 'documents');

    return seiten.some((s) => s.words?.length);
  }

  /**
   * @param {File[]} bilder      Die Fotos aus der Auswahl, in Anzeigereihenfolge
   * @param {number|null} parentId  Zielakte
   * @param {boolean} zusammen   true = ein mehrseitiges Dokument
   * @param {string} name        Name des Dokuments (nur bei zusammen)
   * @returns {Promise<{abgelegt: number, ohneText: number}>}
   */
  async function verarbeite(bilder, parentId, { zusammen = false, name = '' } = {}) {
    abgebrochen = false;
    stand.value = { schritt: 'prepare', nummer: 1, gesamt: bilder.length, prozent: 0 };

    let abgelegt = 0;
    let ohneText = 0;
    const offen = [];

    try {
      const seiten = [];

      for (let i = 0; i < bilder.length; i += 1) {
        if (abgebrochen) break;
        const seite = await seiteAus(bilder[i], i + 1, bilder.length);
        offen.push(seite.canvas);

        // Zweite Prüfung, und die ist die wichtigere: Ein Abbruch beendet den
        // Worker, und recognizeWords meldet das – seiner Aufgabe entsprechend –
        // als „Erkennung ausgefallen" statt als Fehler. Ohne diese Zeile würde
        // aus dem Abbruch ein stillschweigend abgelegtes PDF ohne Textebene.
        if (abgebrochen) break;

        if (zusammen) {
          seiten.push(seite);
          continue;
        }

        // Einzeln: jedes Foto sofort ablegen. Bricht der Nutzer danach ab,
        // bleibt das Fertige liegen, statt mit verworfen zu werden.
        const mitText = await ablegen([seite], ohneEndung(bilder[i].name), parentId);
        abgelegt += 1;
        if (!mitText) ohneText += 1;
        freigeben(seite.canvas);
      }

      if (zusammen && seiten.length && !abgebrochen) {
        const mitText = await ablegen(seiten, name || ohneEndung(bilder[0].name), parentId);
        abgelegt = 1;
        if (!mitText) ohneText = 1;
      }
    } finally {
      offen.forEach(freigeben);
      stand.value = null;
    }

    return { abgelegt, ohneText };
  }

  /**
   * Abbrechen. Der Worker wird abgeraeumt statt nur ignoriert – ein
   * WASM-Worker mit geladenen Sprachdaten haelt sonst dauerhaft Speicher.
   */
  async function abbrechen() {
    abgebrochen = true;
    await releaseOcr();
    stand.value = null;
  }

  return { stand, istBild, verarbeite, abbrechen, ohneEndung };
}
