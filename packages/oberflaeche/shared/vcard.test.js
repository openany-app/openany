import { describe, it, expect } from 'vitest';
import { vcardLesen, vcardSchreiben, zusammenfuehren } from './vcard';

const karte = (...zeilen) => ['BEGIN:VCARD', 'VERSION:3.0', ...zeilen, 'END:VCARD', ''].join('\r\n');

describe('vCard lesen', () => {
  it('macht aus einer fremden Karte einen Kontakt', () => {
    const [k] = vcardLesen(karte(
      'FN:Beispiel Person',
      'TEL;TYPE=CELL:+49 170 0000000',
      'EMAIL;TYPE=INTERNET:person@beispiel.test',
      'ORG:Beispiel gGmbH',
      'ADR;TYPE=HOME:;;Musterweg 1;Musterstadt;;12345;',
    ));
    expect(k.display_name).toBe('Beispiel Person');
    const art = (a) => k.channels.find((w) => w.kind === a);
    expect(art('phone')).toEqual({ kind: 'phone', label: 'CELL', value: '+49 170 0000000' });
    expect(art('email').label).toBeNull();
    expect(art('company').value).toBe('Beispiel gGmbH');
    expect(art('address').value).toBe('Musterweg 1, Musterstadt, 12345');
  });

  it('nimmt ohne FN den Namen aus N', () => {
    expect(vcardLesen('BEGIN:VCARD\r\nVERSION:2.1\r\nN:Person;Beispiel;;;\r\nEND:VCARD\r\n')[0].display_name).toBe('Beispiel Person');
  });

  it('liest Quoted-Printable mit weichem Umbruch und gefaltete Zeilen', () => {
    const [k] = vcardLesen(['BEGIN:VCARD', 'VERSION:2.1',
      'FN;CHARSET=UTF-8;ENCODING=QUOTED-PRINTABLE:Gr=C3=BC=', 'n',
      'X-OPENANY-OTHER:eine lange Angabe, umgebro', ' chen',
      'END:VCARD', ''].join('\r\n'));
    expect(k.display_name).toBe('Grün');
    expect(k.channels[0]).toMatchObject({ kind: 'other', value: 'eine lange Angabe, umgebrochen' });
  });

  it('erkennt IMPP am Schema und reicht Fremdes durch', () => {
    const [k] = vcardLesen(karte('FN:B',
      'IMPP;X-SERVICE-TYPE=Matrix:matrix:u/beispiel:matrix.test',
      'IMPP:meshtastic:!a1b2c3d4',
      'IMPP;X-SERVICE-TYPE=Skype:skype:beispiel',
      'URL:https://beispiel.test',
      'CATEGORIES:a\\,b;c'));
    expect(k.channels).toEqual([
      { kind: 'matrix', label: null, value: '@beispiel:matrix.test' },
      { kind: 'meshtastic', label: null, value: '!a1b2c3d4' },
      { kind: 'vcard:impp', label: null, value: 'skype:beispiel' },
      { kind: 'vcard:url', label: null, value: 'https://beispiel.test' },
      { kind: 'vcard:categories', label: null, value: 'a\\,b;c' },
    ]);
  });

  it('nimmt ein eingebettetes Bild, aber keine Adresse', () => {
    expect(vcardLesen(karte('FN:B', 'PHOTO;ENCODING=b;TYPE=JPEG:QUJD'))[0].foto).toBe('QUJD');
    expect(vcardLesen(karte('FN:B', 'PHOTO;VALUE=uri:https://beispiel.test/b.jpg'))[0].foto).toBeNull();
  });
});

describe('vCard schreiben', () => {
  it('übersteht für jede Art eine Rundreise', () => {
    const wege = [
      { kind: 'phone', label: 'Arbeit', value: '+49 30 000' },
      { kind: 'email', label: null, value: 'a@beispiel.test' },
      { kind: 'address', label: null, value: 'Musterweg 1; Hof' },
      { kind: 'company', label: null, value: 'Beispiel, AG' },
      { kind: 'birthday', label: null, value: '1990-01-01' },
      { kind: 'other', label: null, value: 'Termin: 10 Uhr' },
      { kind: 'matrix', label: null, value: '@b:matrix.test' },
      { kind: 'meshtastic', label: null, value: '!a1' },
      { kind: 'openany', label: null, value: 'beispiel' },
      { kind: 'vcard:impp', label: null, value: 'xmpp:b@beispiel.test' },
      { kind: 'vcard:categories', label: null, value: 'a\\,b;c' },
    ];
    const id = '11111111-2222-4333-8444-555555555555';
    const text = vcardSchreiben([{ id, display_name: 'Beispiel; Person', channels: wege, foto: 'QUJD' }]);
    expect(text).toContain('UID:urn:uuid:' + id);
    const [k] = vcardLesen(text);
    expect(k.uuid).toBe(id);
    expect(k.display_name).toBe('Beispiel; Person');
    expect(k.foto).toBe('QUJD');
    expect(k.channels).toEqual(wege);
  });

  it('faltet lange Zeilen nach 75 Oktetten, ohne ein Zeichen zu zerschneiden', () => {
    const text = vcardSchreiben([{ id: 'x', display_name: 'ü'.repeat(60), channels: [] }]);
    for (const z of text.split('\r\n')) expect(new TextEncoder().encode(z).length).toBeLessThanOrEqual(75);
    expect(vcardLesen(text)[0].display_name).toBe('ü'.repeat(60));
  });
});

describe('zusammenführen', () => {
  const da = [{ id: 'k-1', display_name: 'Beispiel', channels: [{ kind: 'phone', label: null, value: '1' }], photo: null }];

  it('ergänzt über Kennung oder Namen und legt nur Fehlendes dazu', () => {
    const { neu, ergaenzt } = zusammenfuehren(da, [
      { uuid: null, display_name: 'Beispiel', foto: 'QUJD', channels: [{ kind: 'phone', label: 'x', value: '1' }, { kind: 'email', label: null, value: 'b@beispiel.test' }] },
      { uuid: '11111111-2222-4333-8444-555555555555', display_name: 'Neu', foto: null, channels: [] },
      { uuid: null, display_name: 'Neu', foto: null, channels: [{ kind: 'phone', label: null, value: '2' }] },
    ]);
    expect(ergaenzt).toHaveLength(1);
    expect(ergaenzt[0].channels.map((w) => w.value)).toEqual(['1', 'b@beispiel.test']);
    expect(ergaenzt[0].foto).toBe('QUJD');
    expect(neu).toHaveLength(1);
    expect(neu[0].id).toBe('11111111-2222-4333-8444-555555555555');
    expect(neu[0].channels).toEqual([{ kind: 'phone', label: null, value: '2' }]);
  });

  it('ersetzt kein vorhandenes Foto und meldet Unverändertes nicht', () => {
    const { neu, ergaenzt } = zusammenfuehren([{ ...da[0], photo: { url: 'data:' } }], [
      { uuid: 'k-1', display_name: 'Umbenannt', foto: 'QUJD', channels: [{ kind: 'phone', label: null, value: '1' }] },
    ]);
    expect(neu).toEqual([]);
    expect(ergaenzt).toEqual([]);
  });
});
