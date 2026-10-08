export default {
  title: 'Adressbuch',
  empty: 'Noch keine Kontakte.',
  emptyHint: 'Ein Kontakt hält fest, wie jemand erreichbar ist – Nummern, Adressen, Matrix, Meshtastic.',
  search: 'Namen suchen',
  searchPlaceholder: 'Namen suchen',
  noMatch: 'Kein Kontakt heißt „{suche}".',
  add: 'Kontakt',
  // Der Titel am openany-Namen: Er sagt, was ein Klick tut. Ohne ihn sieht
  // die Zeile aus wie ein Link und niemand weiss, wohin.
  writeMessage: 'Nachricht schreiben',
  newTitle: 'Neuer Kontakt',
  editTitle: 'Kontakt bearbeiten',
  displayName: 'Name',
  displayNamePlaceholder: 'Wie die Person im Adressbuch heißt',
  linkedUser: 'Verknüpftes Konto',
  linkedUserNone: 'kein Konto verknüpft',
  // Der Satz steht bewusst im Formular und nicht in der Hilfe: Ein Verweis
  // auf ein Konto sieht nach einer Freigabe aus und ist keine.
  linkedUserHint: 'Nur ein Verweis – er gibt keinen Zugriff auf Projekte oder Inhalte.',
  noIdentifiers: 'noch kein Weg hinterlegt',

  // Nummern, Adressen, Anschriften. „Weg" und nicht „Kanal": Ein Kanal ist im
  // Rest der Anwendung etwas anderes (Chat), und zwei Bedeutungen für ein
  // Wort sind eine Falle für den, der die Hilfe liest.
  // „Weitere", weil Telefon und E-Mail schon oben als offene Felder stehen.
  moreChannels: 'Weitere Wege',
  channelAdd: 'Weg',
  channelsEmpty: 'Noch nichts weiter.',
  channelKind: 'Art des Wegs',
  channelValue: 'Nummer, Adresse oder Kennung',
  channelLabel: 'Beschriftung, z. B. privat oder Arbeit',
  channelRemove: 'Weg entfernen',
  photoChoose: 'Bild wählen',
  photoRemove: 'Bild entfernen',
  photoHint: 'Das Bild wird auf 512×512 gerechnet.',
  photoFailed: 'Das Bild konnte nicht gespeichert werden — der Kontakt schon.',
  advanced: 'Erweitert',
  kinds: {
    phone: 'Telefon',
    email: 'E-Mail',
    openany: 'openany-Name',
    address: 'Anschrift',
    birthday: 'Geburtstag',
    company: 'Betrieb',
    matrix: 'Matrix',
    meshtastic: 'Meshtastic',
    // Der Sammelposten: Ohne ihn trägt der Mensch seine Faxnummer gar nicht
    // ein, bis es eine eigene Art dafür gibt.
    other: 'Sonstiges',
  },
  // Nur für die beiden offenen Felder – die Zeilen unten teilen sich einen
  // gemeinsamen Platzhalter, weil dort die Art daneben steht.
  kindPlaceholder: {
    phone: '+49 170 1234567',
    email: "name{'@'}beispiel.de",
  },
  save: 'Speichern',
  saved: 'Kontakt gespeichert.',
  created: 'Kontakt angelegt.',
  deleted: 'Kontakt in den Papierkorb gelegt.',
  deleteConfirm: 'Kontakt „{name}“ in den Papierkorb legen?',
  loadFailed: 'Adressbuch konnte nicht geladen werden.',
  saveFailed: 'Speichern fehlgeschlagen.',
  deleteFailed: 'Löschen fehlgeschlagen.',
  nameRequired: 'Bitte einen Namen angeben.',
};
