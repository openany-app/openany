import { Extension, InputRule } from '@tiptap/core';
import { anfuehrungszeichenFuer } from '@oberflaeche/shared/anfuehrungszeichen';

/**
 * Typografische Anführungszeichen beim Tippen – nur auf Deutsch.
 *
 * Aus " wird „ am Zitatanfang und “ am Zitatende. Was von beidem gemeint ist,
 * entscheidet shared/anfuehrungszeichen.js anhand des Zeichens links vom
 * Cursor; hier steht nur die Anbindung an Tiptap.
 *
 * Bewusst KEINE Typography-Erweiterung von Tiptap: die brächte ein Dutzend
 * weiterer Ersetzungen mit (-- zu –, ... zu …, (c) zu ©), die still in die
 * .md-Dateien schriebe, wonach niemand gefragt hat.
 *
 * Zwei Dinge muss diese Erweiterung NICHT selbst prüfen, weil der Kern das
 * schon tut (siehe @tiptap/core, InputRule.ts): In Codeblöcken und in
 * `code`-Auszeichnungen laufen Eingaberegeln gar nicht erst an – dort bleibt
 * das gerade " stehen, wo es hingehört. Ebenso pausiert er während einer
 * IME-Eingabe (view.composing).
 */
export default Extension.create({
  name: 'deutscheAnfuehrungszeichen',

  addOptions() {
    return {
      /**
       * Wird bei JEDEM getippten " gefragt, nicht einmal beim Bauen – die
       * Sprache lässt sich zur Laufzeit umstellen (i18n/index.js: setLocale),
       * ohne dass der Editor dafür neu gebaut wird.
       */
      istDeutsch: () => false,
    };
  },

  addInputRules() {
    return [
      new InputRule({
        find: /"$/,
        handler: ({ state, range, chain }) => {
          // null = "nicht zuständig". Der Kern lässt das getippte Zeichen dann
          // unverändert durch, statt die Eingabe zu verschlucken.
          if (!this.options.istDeutsch()) return null;

          const $von = state.doc.resolve(range.from);
          const davor = $von.parentOffset > 0
            // Platzhalter für Knoten (Bild, Wikilink): ein Zitat, das direkt
            // hinter einem solchen beginnt, soll schließen, nicht öffnen.
            ? $von.parent.textBetween($von.parentOffset - 1, $von.parentOffset, null, '￼')
            : '';

          chain().insertContentAt(range, anfuehrungszeichenFuer(davor)).run();
          return undefined;
        },
      }),
    ];
  },
});
