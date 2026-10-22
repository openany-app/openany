/*
 * Gemeinsam schreiben (docs/plan-gemeinsam-schreiben.md, Schritt G2).
 *
 * Verbindet ein Yjs-Dokument mit einer Sitzung beim Knoten -- dem Gerät,
 * das die Notiz führt. Der Weg dorthin ist ein „Transport“ (in der App:
 * Tauri-Befehle, app/src/sitzung.js), damit sich dieser Teil ohne App
 * prüfen lässt:
 *
 *   transport.senden(aenderungen: string[], awareness: string|null) → nr
 *   transport.warten(ab: number) → Neues
 *   transport.zusammenfassen(stand: string, bis: number)
 *   transport.verlassen()
 *
 * Alle Bytes reisen als Base64.
 *
 * WAS ER TUT:
 * - Beim Start Stand und Änderungen anwenden; ist die Sitzung leer und
 *   dieser Teilnehmer dafür eingeteilt (`anlegen`), den Ausgangstext
 *   einsetzen -- über `ausgangstext(doc, inhalt)`, weil nur der Editor
 *   weiß, wie aus Markdown ein Dokument wird.
 * - Eigene Änderungen und den eigenen Cursor kurz sammeln und schicken.
 * - In einer Schleife warten und anwenden, was kommt.
 * - Wer geht, verschwindet samt Cursor.
 * - Ist er der Schreiber und bittet der Knoten darum, den Stand
 *   zusammenfassen.
 * - Antwortet der Knoten nicht mehr: `onAus()` -- der Editor wird
 *   schreibgeschützt.
 */
import * as Y from 'yjs';
import {
  Awareness,
  applyAwarenessUpdate,
  encodeAwarenessUpdate,
  removeAwarenessStates,
} from 'y-protocols/awareness';

const LEITUNG = 'leitung';

export function zuBase64(bytes) {
  let s = '';
  for (let i = 0; i < bytes.length; i += 0x8000) {
    s += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  }
  return btoa(s);
}

export function ausBase64(text) {
  const s = atob(text);
  const b = new Uint8Array(s.length);
  for (let i = 0; i < s.length; i++) b[i] = s.charCodeAt(i);
  return b;
}

export class SitzungsProvider {
  /**
   * @param {object} o
   * @param {object} o.start   Antwort von `sitzung_beitreten`
   * @param {object} o.transport
   * @param {(doc: Y.Doc, inhalt: string) => void} o.ausgangstext
   * @param {Y.Doc} [o.doc]
   * @param {(teilnehmer: object[]) => void} [o.onAnwesende]
   * @param {(ichSpeichere: boolean) => void} [o.onSchreiber]
   * @param {(grund: string) => void} [o.onAus]
   * @param {number} [o.sammeln]   ms, die Änderungen gesammelt werden
   * @param {number} [o.versuche]  Fehlschläge in Folge, bis `onAus` kommt
   */
  constructor(o) {
    this.start = o.start;
    this.kennung = o.start.kennung;
    this.transport = o.transport;
    this.ausgangstext = o.ausgangstext;
    this.onAnwesende = o.onAnwesende ?? (() => {});
    this.onSchreiber = o.onSchreiber ?? (() => {});
    this.onAus = o.onAus ?? (() => {});
    this.sammeln = o.sammeln ?? 40;
    this.versuche = o.versuche ?? 3;

    this.doc = o.doc ?? new Y.Doc();
    this.awareness = new Awareness(this.doc);
    this.ab = o.start.nr;
    this.schreiber = o.start.schreiber;
    this.beendet = false;
    this.aus = false;
    this.ausstehend = [];
    this.awarenessAusstehend = false;
    this.zeitgeber = null;
    // Welche Yjs-Clients zu welcher Kennung gehören -- damit beim Gehen der
    // richtige Cursor verschwindet.
    this.clientsVon = new Map();

    this.beiAenderung = (update, herkunft) => {
      if (herkunft === LEITUNG || this.beendet) return;
      this.ausstehend.push(update);
      this.planen();
    };
    this.beiAwareness = ({ added, updated, removed }, herkunft) => {
      if (herkunft === 'local') {
        this.awarenessAusstehend = true;
        this.planen();
        return;
      }
      if (typeof herkunft === 'string' && herkunft.startsWith('k:')) {
        const k = herkunft.slice(2);
        const menge = this.clientsVon.get(k) ?? new Set();
        [...added, ...updated].forEach((c) => menge.add(c));
        removed.forEach((c) => menge.delete(c));
        this.clientsVon.set(k, menge);
      }
    };
    this.doc.on('update', this.beiAenderung);
    this.awareness.on('update', this.beiAwareness);

    this.anwenden({
      stand: o.start.stand,
      aenderungen: o.start.aenderungen,
      awareness: o.start.awareness,
      weg: [],
      teilnehmer: o.start.teilnehmer,
      schreiber: o.start.schreiber,
      anlegen: o.start.anlegen,
    });
    this.inhalt = o.start.inhalt ?? '';
  }

  get ichSpeichere() {
    return this.schreiber === this.kennung;
  }

  /** Eigenen Zustand für die Cursor setzen (Name, Farbe). */
  setzeIch(felder) {
    this.awareness.setLocalStateField('user', felder);
  }

  /** Die Schleife starten. Kehrt zurück, wenn die Sitzung endet. */
  async laufen() {
    let fehlschlaege = 0;
    while (!this.beendet) {
      try {
        const neu = await this.transport.warten(this.ab);
        if (this.beendet) return;
        fehlschlaege = 0;
        this.anwenden(neu);
        if (neu.zusammenfassen && this.ichSpeichere) {
          const bis = this.ab;
          await this.transport.zusammenfassen(zuBase64(Y.encodeStateAsUpdate(this.doc)), bis);
        }
      } catch (e) {
        fehlschlaege += 1;
        if (fehlschlaege >= this.versuche) {
          this.ausschalten(String(e));
          return;
        }
        await new Promise((r) => setTimeout(r, 200 * fehlschlaege));
      }
    }
  }

  anwenden(neu) {
    Y.transact(this.doc, () => {
      if (neu.stand) Y.applyUpdate(this.doc, ausBase64(neu.stand), LEITUNG);
      for (const a of neu.aenderungen ?? []) Y.applyUpdate(this.doc, ausBase64(a), LEITUNG);
    }, LEITUNG);
    for (const [k, a] of neu.awareness ?? []) {
      if (k === this.kennung) continue;
      applyAwarenessUpdate(this.awareness, ausBase64(a), `k:${k}`);
    }
    for (const k of neu.weg ?? []) {
      const clients = this.clientsVon.get(k);
      if (clients?.size) removeAwarenessStates(this.awareness, [...clients], LEITUNG);
      this.clientsVon.delete(k);
    }
    if (neu.nr !== undefined) this.ab = Math.max(this.ab, neu.nr);
    if (neu.teilnehmer) this.onAnwesende(neu.teilnehmer);
    if (neu.schreiber !== undefined && neu.schreiber !== this.schreiber) {
      this.schreiber = neu.schreiber;
      this.onSchreiber(this.ichSpeichere);
    }
    if (neu.anlegen && this.leer()) {
      this.ausgangstext(this.doc, this.inhalt ?? this.start.inhalt ?? '');
    }
  }

  leer() {
    // Ein Dokument ohne einen einzigen Eintrag in irgendeinem Typ.
    return Y.encodeStateVector(this.doc).length <= 1;
  }

  planen() {
    if (this.zeitgeber || this.beendet) return;
    this.zeitgeber = setTimeout(() => {
      this.zeitgeber = null;
      this.abschicken();
    }, this.sammeln);
  }

  async abschicken() {
    if (this.aus) return;
    const updates = this.ausstehend.splice(0);
    const mitAwareness = this.awarenessAusstehend;
    this.awarenessAusstehend = false;
    if (!updates.length && !mitAwareness) return;
    const aenderungen = updates.length ? [zuBase64(Y.mergeUpdates(updates))] : [];
    const awareness = mitAwareness
      ? zuBase64(encodeAwarenessUpdate(this.awareness, [this.doc.clientID]))
      : null;
    try {
      await this.transport.senden(aenderungen, awareness);
    } catch {
      // Nicht verlieren: beim nächsten Mal wieder mit.
      this.ausstehend.unshift(...updates);
      if (mitAwareness) this.awarenessAusstehend = true;
      if (!this.beendet) setTimeout(() => this.planen(), 300);
    }
  }

  ausschalten(grund) {
    if (this.aus) return;
    this.aus = true;
    this.beendet = true;
    clearTimeout(this.zeitgeber);
    this.onAus(grund);
  }

  /** Gehen: Ausstehendes schicken, dann verlassen. */
  async zerstoeren() {
    if (this.beendet && this.aus) return;
    clearTimeout(this.zeitgeber);
    this.zeitgeber = null;
    await this.abschicken();
    this.beendet = true;
    this.doc.off('update', this.beiAenderung);
    this.awareness.off('update', this.beiAwareness);
    this.awareness.destroy();
    try {
      await this.transport.verlassen();
    } catch {
      /* der Knoten räumt nach einer Minute selbst auf */
    }
  }
}
