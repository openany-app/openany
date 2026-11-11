import { describe, expect, it } from 'vitest';
import { rechtstextBloecke } from './rechtstext';

describe('Rechtstexte zerlegen', () => {
  it('erkennt Überschriften, Absätze und Listen', () => {
    const b = rechtstextBloecke('# Titel\n\n## Teil\n\nEin **fetter** Satz\n\n- eins\n- zwei\n');
    expect(b.map((x) => x.art)).toEqual(['h1', 'h2', 'p', 'ul']);
    expect(b[2].zeilen[0]).toEqual([{ text: 'Ein ', fett: false }, { text: 'fetter', fett: true }, { text: ' Satz', fett: false }]);
    expect(b[3].zeilen).toHaveLength(2);
  });

  it('hält Zeilen eines Absatzes getrennt, etwa die einer Anschrift', () => {
    expect(rechtstextBloecke('Name  \nStraße  \nOrt')[0].zeilen).toHaveLength(3);
  });

  it('nimmt HTML als Text und übersteht eine leere Antwort', () => {
    expect(rechtstextBloecke('<script>x</script>')[0].zeilen[0][0].text).toBe('<script>x</script>');
    expect(rechtstextBloecke(null)).toEqual([]);
  });
});
