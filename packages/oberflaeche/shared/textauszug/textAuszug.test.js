import { describe, it, expect } from 'vitest';
import { docxText, odtText, textAuslesen, TextAuszugFehler, MAX_ZEICHEN } from './textAuszug';
import { zipEintrag, ZipFehler } from './zipLesen';

// Ein minimales ZIP-Archiv bauen – gespeichert oder mit echter Deflate-
// Kompression, damit beide Wege des Lesers durchlaufen werden.
async function baueZip(eintraege, { komprimieren = false } = {}) {
  const enc = new TextEncoder();
  const teile = [];
  const verzeichnis = [];
  let offset = 0;

  for (const [name, inhalt] of Object.entries(eintraege)) {
    const nameB = enc.encode(name);
    const roh = enc.encode(inhalt);
    const daten = komprimieren
      ? new Uint8Array(await new Response(new Blob([roh]).stream().pipeThrough(new CompressionStream('deflate-raw'))).arrayBuffer())
      : roh;
    const methode = komprimieren ? 8 : 0;

    const kopf = new DataView(new ArrayBuffer(30));
    kopf.setUint32(0, 0x04034b50, true);
    kopf.setUint16(8, methode, true);
    kopf.setUint32(18, daten.length, true);
    kopf.setUint32(22, roh.length, true);
    kopf.setUint16(26, nameB.length, true);
    teile.push(new Uint8Array(kopf.buffer), nameB, daten);

    const v = new DataView(new ArrayBuffer(46));
    v.setUint32(0, 0x02014b50, true);
    v.setUint16(10, methode, true);
    v.setUint32(20, daten.length, true);
    v.setUint32(24, roh.length, true);
    v.setUint16(28, nameB.length, true);
    v.setUint32(42, offset, true);
    verzeichnis.push(new Uint8Array(v.buffer), nameB);

    offset += 30 + nameB.length + daten.length;
  }

  const anzahl = Object.keys(eintraege).length;
  const vLaenge = verzeichnis.reduce((s, b) => s + b.length, 0);
  const ende = new DataView(new ArrayBuffer(22));
  ende.setUint32(0, 0x06054b50, true);
  ende.setUint16(8, anzahl, true);
  ende.setUint16(10, anzahl, true);
  ende.setUint32(12, vLaenge, true);
  ende.setUint32(16, offset, true);

  const alle = [...teile, ...verzeichnis, new Uint8Array(ende.buffer)];
  const gesamt = new Uint8Array(alle.reduce((s, b) => s + b.length, 0));
  let pos = 0;
  for (const b of alle) { gesamt.set(b, pos); pos += b.length; }
  return gesamt.buffer;
}

const WORD = `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>
<w:p><w:pPr><w:pStyle w:val="berschrift1"/></w:pPr><w:r><w:t>Kostentheorie</w:t></w:r></w:p>
<w:p><w:r><w:t xml:space="preserve">Fixkosten fallen </w:t></w:r><w:r><w:rPr><w:b/></w:rPr><w:t>unabhängig</w:t></w:r><w:r><w:t xml:space="preserve"> von der Menge an &amp; bleiben.</w:t></w:r></w:p>
<w:p/>
<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/></w:numPr></w:pPr><w:r><w:t>Miete</w:t></w:r></w:p>
<w:p><w:r><w:t>Zeile eins</w:t><w:br/><w:t>Zeile zwei</w:t></w:r></w:p>
<w:tbl><w:tr><w:tc><w:p><w:r><w:t>Art</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>Betrag</w:t></w:r></w:p></w:tc></w:tr>
<w:tr><w:tc><w:p><w:r><w:t>Miete</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>800 €</w:t></w:r></w:p></w:tc></w:tr></w:tbl>
<w:p><w:r><w:delText>gelöscht</w:delText><w:t>Ende</w:t></w:r></w:p>
</w:body></w:document>`;

describe('docxText', () => {
  it('liest Überschriften, Absätze, Listen, Umbrüche und Tabellen', () => {
    expect(docxText(WORD)).toBe([
      '# Kostentheorie',
      'Fixkosten fallen unabhängig von der Menge an & bleiben.',
      '- Miete',
      'Zeile eins\nZeile zwei',
      '| Art | Betrag |\n| Miete | 800 € |',
      'Ende',
    ].join('\n\n'));
  });

  it('erkennt englische Überschriften-Stile', () => {
    const xml = '<w:body><w:p><w:pPr><w:pStyle w:val="Heading2"/></w:pPr><w:r><w:t>Kapitel</w:t></w:r></w:p></w:body>';
    expect(docxText(xml)).toBe('## Kapitel');
  });

  it('übernimmt gelöschten Text aus der Änderungsverfolgung nicht', () => {
    expect(docxText(WORD)).not.toContain('gelöscht');
  });
});

describe('odtText', () => {
  it('liest Überschriften, Absätze mit Leerzeichen, Listen und Tabellen', () => {
    const xml = `<office:document-content><office:body><office:text>
<text:h text:outline-level="2">Kosten</text:h>
<text:p>Fix<text:s text:c="2"/>und<text:tab/>variabel</text:p>
<text:list><text:list-item><text:p>Miete</text:p></text:list-item></text:list>
<table:table><table:table-row><table:table-cell><text:p>A</text:p></table:table-cell><table:table-cell><text:p>1</text:p></table:table-cell></table:table-row></table:table>
</office:text></office:body></office:document-content>`;

    expect(odtText(xml)).toBe('## Kosten\n\nFix  und\tvariabel\n\n- Miete\n\n| A | 1 |');
  });
});

describe('zipEintrag', () => {
  it('liest gespeicherte und komprimierte Einträge', async () => {
    const gespeichert = await baueZip({ 'a.txt': 'eins', 'b/c.xml': '<x>zwei</x>' });
    const komprimiert = await baueZip({ 'a.txt': 'eins', 'b/c.xml': '<x>zwei</x>' }, { komprimieren: true });

    for (const archiv of [gespeichert, komprimiert]) {
      expect(new TextDecoder().decode(await zipEintrag(archiv, 'b/c.xml'))).toBe('<x>zwei</x>');
    }
  });

  it('liefert null für einen fehlenden Eintrag', async () => {
    expect(await zipEintrag(await baueZip({ 'a.txt': 'x' }), 'fehlt.xml')).toBeNull();
  });

  it('erkennt, wenn gar kein ZIP vorliegt', async () => {
    await expect(zipEintrag(new TextEncoder().encode('Hallo, ich bin Text').buffer, 'x')).rejects.toBeInstanceOf(ZipFehler);
  });
});

describe('textAuslesen', () => {
  it('liest eine komprimierte Word-Datei von vorn bis hinten', async () => {
    const docx = await baueZip({ '[Content_Types].xml': '<Types/>', 'word/document.xml': WORD }, { komprimieren: true });
    expect(await textAuslesen('docx', docx)).toContain('| Miete | 800 € |');
  });

  it('meldet eine Word-Datei ohne Dokumentteil als unlesbar', async () => {
    const kaputt = await baueZip({ 'irgendwas.xml': '<x/>' });
    await expect(textAuslesen('docx', kaputt)).rejects.toMatchObject({ grund: 'unlesbar' });
  });

  it('lehnt zu langen Text ab, statt ihn abzuschneiden', async () => {
    const lang = new TextEncoder().encode('x'.repeat(MAX_ZEICHEN + 1)).buffer;
    await expect(textAuslesen('text', lang)).rejects.toMatchObject({ grund: 'zuLang' });
  });

  it('meldet eine leere Datei als „kein Text"', async () => {
    const leer = new TextEncoder().encode('  \n ').buffer;
    const fehler = await textAuslesen('text', leer).catch((e) => e);
    expect(fehler).toBeInstanceOf(TextAuszugFehler);
    expect(fehler.grund).toBe('keinText');
  });
});
