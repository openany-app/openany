import { describe, it, expect } from 'vitest';
import { zitateUndFussnotenFreilegen as frei } from './markdownFreilegen';

describe('zitateUndFussnotenFreilegen', () => {
  it('legt Zitate frei, wie Pandoc sie liest', () => {
    expect(frei('Text \\[@mueller2020, S. 12\\] und \\[@a; @b\\].')).toBe('Text [@mueller2020, S. 12] und [@a; @b].');
    expect(frei('\\[@mein\\_schluessel\\]')).toBe('[@mein\\_schluessel]');
  });

  it('legt Fußnoten frei, auch mit Formatierung, Zitat und Wikilink darin', () => {
    expect(frei('Satz.^\\[Eine **fette** Anmerkung \\[@a\\] zu [[Notiz]].\\] Weiter.'))
      .toBe('Satz.^[Eine **fette** Anmerkung [@a] zu [[Notiz]].] Weiter.');
    expect(frei('^\\[mit \\[Klammer\\] darin\\]')).toBe('^[mit \\[Klammer\\] darin]');
  });

  it('lässt sonstige Klammern und Code in Ruhe', () => {
    expect(frei('\\[nur Text\\] und \\[ @kein\\]')).toBe('\\[nur Text\\] und \\[ @kein\\]');
    expect(frei('`\\[@a\\]` und `^\\[x\\]`')).toBe('`\\[@a\\]` und `^\\[x\\]`');
    expect(frei('```\n\\[@a\\]\n```\n\\[@b\\]')).toBe('```\n\\[@a\\]\n```\n[@b]');
  });

  it('eine nie geschlossene Fußnote bleibt, wie sie ist, auch über Absätze', () => {
    expect(frei('^\\[offen\n\nweiter\\]')).toBe('^\\[offen\n\nweiter\\]');
  });

  it('ist beständig: zweimal freigelegt ändert nichts mehr', () => {
    const einmal = frei('A \\[@a\\] B^\\[c\\]');
    expect(frei(einmal)).toBe(einmal);
  });
});
