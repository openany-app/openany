export default {
  /**
   * Angemeldet wird bei anyid, und dort stehen die Texte der Anmeldung. Hier
   * stehen nur die Sätze für den Weg dorthin.
   */
  signIn: 'Anmelden',
  signedOut: 'Abgemeldet',
  signInHint: 'Die Anmeldung läuft über anyid.',

  // Die Notbremse hat gegriffen: Die Sitzung hält sich nicht. Der Text nennt
  // die wahrscheinliche Ursache, statt nur „hat nicht geklappt" zu sagen —
  // sonst versucht es jemand fünfmal auf dieselbe Weise.
  handoverFailed: 'Die Anmeldung kam nicht an',
  handoverFailedHint:
    'Die Übergabe von anyid hat diese Sitzung nicht erreicht. Meist liegt es '
    + 'an blockierten Cookies. Ein weiterer Versuch schadet nicht — bleibt es '
    + 'dabei, hilft ein anderer Browser oder ein neues Fenster.',
};
