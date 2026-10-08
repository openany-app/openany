import { describe, it, expect } from 'vitest';
import { wegZiel, WEG_ARTEN, OFFENE_FELDER, VCARD_FELDER, vcardName, istDurchgereicht, nachrichtenName, nachrichtenWeg } from './kontaktwege';

describe('kontaktwege', () => {
  it('macht aus einer lesbaren Nummer eine waehlbare', () => {
    expect(wegZiel({ kind: 'phone', value: '+49 (170) 123-4567' })).toBe('tel:+491701234567');
  });

  it('laesst eine Mailadresse, wie sie ist', () => {
    expect(wegZiel({ kind: 'email', value: 'hannah@example.org' })).toBe('mailto:hannah@example.org');
  });

  // Ein toter Link ist schlimmer als keiner: Er sieht klickbar aus, und wer
  // ihn antippt, bekommt nichts und weiss nicht, ob es an ihm lag.
  it('gibt fuer alles, womit ein Betriebssystem nichts anfangen kann, kein Ziel', () => {
    expect(wegZiel({ kind: 'address', value: 'Musterweg 1' })).toBeNull();
    expect(wegZiel({ kind: 'company', value: 'Beispiel gGmbH' })).toBeNull();
    expect(wegZiel({ kind: 'openany', value: 'hannah' })).toBeNull();
    expect(wegZiel({ kind: 'matrix', value: '@hannah:matrix.org' })).toBeNull();
    expect(wegZiel({ kind: 'other', value: 'Fax 030 111' })).toBeNull();
  });

  it('gibt fuer eine leere Zeile kein Ziel', () => {
    expect(wegZiel({ kind: 'phone', value: '   ' })).toBeNull();
    expect(wegZiel(null)).toBeNull();
  });

  it('kennt dieselben Arten wie der Server, in der Reihenfolge des Auswahlfelds', () => {
    expect(WEG_ARTEN).toEqual([
      'matrix', 'meshtastic', 'openany', 'address', 'company', 'birthday',
      'other', 'phone', 'email',
    ]);
  });

  // Die offenen Felder sind eine ABKUERZUNG, kein zweiter Mechanismus: Jede
  // dieser Arten muss auch im Auswahlfeld stehen, sonst laesst sich keine
  // zweite Nummer eintragen.
  // Eine Auswahl, keine Grenze: Was ein fremdes Adressbuch mitbringt, kommt
  // auch dann an, wenn es hier nicht steht.
  it('zeigt die vCard-Felder mit Praefix und erkennt sie wieder', () => {
    expect(VCARD_FELDER).toContain('vcard:nickname');
    expect(VCARD_FELDER.every(istDurchgereicht)).toBe(true);
    expect(WEG_ARTEN.some(istDurchgereicht)).toBe(false);
  });

  it('zeigt ein durchgereichtes Feld unter seinem eigenen Namen', () => {
    expect(vcardName('vcard:x-abrelatednames')).toBe('X-ABRELATEDNAMES');
    expect(vcardName('vcard:nickname')).toBe('NICKNAME');
  });

  it('haelt jede Art mit offenem Feld auch im Auswahlfeld', () => {
    expect(OFFENE_FELDER).toEqual(['phone', 'email']);
    OFFENE_FELDER.forEach((art) => expect(WEG_ARTEN).toContain(art));
  });
  /*
   * DER openany-NAME IST DER EINZIGE WEG NACH INNEN.
   *
   * Ein Klick darauf oeffnet die Nachrichten mit vorgemerktem Empfaenger --
   * kein `tel:`, kein `mailto:`, sondern jemand mit einem Konto hier. Was
   * diese Funktion zurueckgibt, ist deshalb ein NAME und keine Adresse: Die
   * Webapp baut daraus eine Route, `openany-app` einen Bildschirm.
   */
  it('nennt den Empfaenger nur beim openany-Namen', () => {
    expect(nachrichtenName({ kind: 'openany', value: 'tiffy' })).toBe('tiffy');
    expect(nachrichtenName({ kind: 'email', value: 'tiffy@beispiel.de' })).toBe(null);
    expect(nachrichtenName({ kind: 'matrix', value: '@tiffy:example.org' })).toBe(null);
    expect(nachrichtenName({ kind: 'vcard:nickname', value: 'tiffy' })).toBe(null);
  });

  /*
   * MATRIX IST DER ZWEITE WEG, DER EINE NACHRICHT TRAGEN KANN.
   *
   * `nachrichtenName` sagt dazu weiter nein, und das ist richtig: Die Frage
   * dort lautet „ist das jemand von hier?". Wer wissen will, ob ueberhaupt
   * etwas geht, fragt `nachrichtenWeg`.
   */
  it('kennt beide Wege, auf denen eine Nachricht reisen kann', () => {
    expect(nachrichtenWeg({ kind: 'openany', value: 'tiffy' })).toEqual({ kanal: 'openany', kennung: 'tiffy' });
    expect(nachrichtenWeg({ kind: 'matrix', value: '@tiffy:matrix.org' })).toEqual({ kanal: 'matrix', kennung: '@tiffy:matrix.org' });
  });

  it('traegt auf keinem anderen Weg eine Nachricht', () => {
    expect(nachrichtenWeg({ kind: 'email', value: 'tiffy@beispiel.de' })).toBeNull();
    expect(nachrichtenWeg({ kind: 'meshtastic', value: '!a1b2c3d4' })).toBeNull();
    expect(nachrichtenWeg({ kind: 'vcard:nickname', value: 'tiffy' })).toBeNull();
    expect(nachrichtenWeg(null)).toBeNull();
  });

  // Dieselbe Falle wie bei `nachrichtenName`: Eine Kennung mit Leerzeichen
  // drumherum ist dieselbe Kennung, und eine aus Leerzeichen ist keine.
  it('macht auch bei Matrix aus Leere kein Ziel', () => {
    expect(nachrichtenWeg({ kind: 'matrix', value: '   ' })).toBeNull();
    expect(nachrichtenWeg({ kind: 'matrix', value: '  @tiffy:matrix.org  ' }))
      .toEqual({ kanal: 'matrix', kennung: '@tiffy:matrix.org' });
  });

  // Leer heisst leer -- und ein Name mit Leerzeichen drumherum ist derselbe
  // Name. Ohne das Trimmen entstuende eine Route `?an=%20tiffy%20`, und der
  // Empfaenger waere jemand, den es nicht gibt.
  it('macht aus Leere kein Ziel', () => {
    expect(nachrichtenName({ kind: 'openany', value: '   ' })).toBe(null);
    expect(nachrichtenName({ kind: 'openany', value: '' })).toBe(null);
    expect(nachrichtenName({ kind: 'openany' })).toBe(null);
    expect(nachrichtenName({ kind: 'openany', value: '  tiffy  ' })).toBe('tiffy');
    expect(nachrichtenName(null)).toBe(null);
  });
});
