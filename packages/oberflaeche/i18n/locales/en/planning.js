// Der Reiter „Board“ eines Hubs: das eine Board, dazu
// die Texte der Zeichenfläche (whiteboard) und zum Teilen von Kalendern.
export default {
  board: {
    titel: 'Board',
    hinweis: 'The shared board of this hub: all members draw and write on it – via openany.de and nearby.',
    geaendert: 'Last changed: {zeit}',
    oeffnen: 'Open board',
  },
  kalender: {
    teilenHinweis: 'With “view”, members see the events of this calendar; with “edit”, they can also add, change and delete events. Only you can change the calendar itself. Subscribed calendars can only be shared nearby.',
    beenden: 'Stop sharing',
    beendenFrage: 'Stop sharing “{name}” in this hub? The events disappear for all members; your calendar stays.',
  },
  whiteboard: {
    neu: 'New whiteboard',
    nurAnsehen: 'View only – the person who created this whiteboard can save it.',
    nurAnsehenFern: 'View only – you can draw along when a device of {name} is nearby.',
    speichert: 'Saving …',
    gespeichert: 'Saved',
    nichtGespeichert: 'Not saved – will retry with the next change.',
    veraltet: 'Another of your devices saved in the meantime. The whiteboard is being reloaded.',
    offline: 'Offline – you are seeing the last loaded version.',
    ladenFehler: 'The whiteboard could not be opened.',
    confirmDelete: 'Delete whiteboard “{title}”?',
    loeschen: 'Delete whiteboard',
    laedt: 'Loading canvas …',
    einbinden: 'Embed',
    alleFreigaben: 'All shares',
    keineFreigaben: 'Nothing is shared in this hub yet. You can share under “Shared”.',
    leer: 'There is nothing here that can be embedded.',
    nichtMehrGeteilt: 'No longer shared',
    karteNotiz: 'Note',
    kartePdf: 'PDF',
    karteDatei: 'File',
  },
};
