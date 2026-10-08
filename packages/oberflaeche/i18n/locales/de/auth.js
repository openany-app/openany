export default {
  /**
   * AM 28.08.2026 VON 90 AUF WENIGE ZEILEN GESCHRUMPFT.
   *
   * Hier standen die Texte der eigenen Anmeldemaske: Benutzername, Passwort,
   * falsche Zugangsdaten, zweiter Faktor per Post, Passkey-Aufforderung,
   * Registrierung, Nutzungsbedingungen. Nichts davon zeigt openany noch —
   * angemeldet wird bei anyid, und dort stehen diese Texte auch.
   *
   * Was bleibt, sind die zwei Sätze für den Weg dorthin.
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
