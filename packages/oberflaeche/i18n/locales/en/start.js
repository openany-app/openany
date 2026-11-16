// Startseite openany.de (frontend/start.html).
// Achtung vue-i18n: kein rohes @, | oder { in den Texten (außer Platzhaltern).
export default {
  seitentitel: 'openany – private, encrypted, on your device first',
  kopf: 'Your notes, appointments, files and messages – in one place.',
  unterzeile: 'Private, end-to-end encrypted and on your device first. Free, without ads, without a password.',
  ladenFuer: 'Download for {system}',
  alleDownloads: 'All downloads',
  iosFolgt: 'openany for iPhone and iPad is in the works.',
  merkmale: {
    verschluesselt: {
      titel: 'Only you can read it',
      text: 'Everything is encrypted on your device before it reaches the server. We cannot read your content – not even if we wanted to.',
    },
    geraet: {
      titel: 'On your device first',
      text: 'openany works without internet and without an account. Syncing via openany.de is an offer, not a requirement.',
    },
    alles: {
      titel: 'Everything in one place',
      text: 'Notes, calendar, contacts, files, photos, videos and messages – in one app instead of ten.',
    },
    hubs: {
      titel: 'Together in hubs',
      text: 'Share folders, calendars and albums with others, draw together on a board and chat.',
    },
    sparsam: {
      titel: 'No password, no email address',
      text: 'You create your account right in the app, and it is tied to your devices. We store neither email addresses nor IP addresses.',
    },
    frei: {
      titel: 'Free and without ads',
      text: 'No subscription, no ads, no selling of data. Every account starts with 100 MB of storage on openany.de.',
    },
  },
  downloadsTitel: 'openany for your device',
  version: 'Version {version}',
  empfohlen: 'For your device',
  beiGithub: 'The files are hosted on GitHub.',
  keineListe: 'The current list cannot be loaded right now.',
  dateien: {
    apk: 'Download APK',
    installer: 'Installer (.exe)',
    msi: 'MSI package',
    dmg: 'Download for Mac (.dmg)',
    appimage: 'AppImage',
    deb: 'Debian and Ubuntu (.deb)',
  },
  hinweise: {
    android: 'Directly as a file, not from the Play Store. The first time, Android asks whether you want to allow apps from this source.',
    windows: 'For Windows 10 and 11. The app is not signed yet: if Windows warns you, choose “More info” and then “Run anyway”.',
    macos: 'macOS 13.3 or later, for Apple silicon and Intel. The app is not notarised by Apple yet: on first launch, allow it under System Settings, Privacy and Security, “Open Anyway”.',
    linux: 'AppImage for almost any distribution, .deb for Debian and Ubuntu. 64-bit.',
  },
  fuss: {
    melden: 'Report and contact',
    agb: 'Terms of use',
    datenschutz: 'Privacy',
    impressum: 'Legal notice',
    github: 'All versions on GitHub',
  },
  recht: {
    zurueck: 'Back to the start page',
    nurDeutsch: 'This text is only valid in its German version.',
    fehler: 'The text cannot be loaded right now. Please try again later.',
  },
  plattformen: 'For Android, Windows, macOS and Linux. iPhone and iPad are in the works.',
  kopfAlt: [
    'Home screen of the app with today, address book, notes, storage and hubs',
    'Album in the media library with landscape pictures',
  ],
  rundgangKnopf: 'Take the tour',
  rundgangLabel: 'Tour',
  sprung: {
    notizen: 'Notes',
    kalender: 'Calendar',
    hubs: 'Hubs',
    nachrichten: 'Messages',
    mediathek: 'Media library',
    dokumente: 'Documents',
    downloads: 'Downloads',
  },
  rundgang: {
    notizen: {
      augenbraue: 'Notes',
      titel: 'Write and connect your thoughts',
      abschnitte: [
        {
          titel: 'Write, organise, connect',
          absaetze: [
            'Notes live in folders and are written in Markdown – with headings, lists and tasks to tick off.',
          ],
          punkte: [
            '`[[Wikilinks]]` connect notes to each other',
            '`#Tags` gather what belongs together',
            'Full-text search finds every word',
          ],
          alt: [
            'Note with a list, wikilinks, tags and tasks',
          ],
        },
        {
          titel: 'See how everything connects',
          absaetze: [
            'The “Links” view shows your notes as a network. Pick a folder or a tag, and the matching notes appear.',
          ],
          alt: [
            'Note list with folders and notes, names blurred',
            'Graph of links between notes, labels blurred',
          ],
        },
      ],
    },
    kalender: {
      augenbraue: 'Calendar',
      titel: 'Events for you and everyone you plan with',
      abschnitte: [
        {
          titel: 'Your week at a glance',
          absaetze: [
            'Month, week or day: events with location, notes and repetition. One tap on the day is enough.',
          ],
          punkte: [
            'Several calendars side by side',
            '“Today” right on the home screen',
            'Encrypted like everything else',
            'Repeats such as “every 2 weeks on Tuesday, 5 times”',
          ],
          alt: [
            'Month view with example events',
          ],
        },
        {
          titel: 'Share calendars in hubs',
          absaetze: [
            'With “view”, members see your events; with “edit”, they add their own. Only you change the calendar itself.',
          ],
          alt: [
            'Dialog for sharing a calendar in a hub',
            'Hub with a shared folder, calendar, album and record',
          ],
        },
        {
          titel: 'In your device’s calendar and contacts',
          absaetze: [
            'On Android, your openany calendars and contacts appear in the phone’s apps – and what you change there comes back. On a computer, openany offers them to calendar and contacts programs, only on that computer.',
          ],
          punkte: [
            'Event reminders on your phone',
            'On a computer via CalDAV and CardDAV, for example for Thunderbird',
          ],
        },
      ],
    },
    hubs: {
      augenbraue: 'Hubs',
      titel: 'Together, not side by side',
      abschnitte: [
        {
          titel: 'A place for your group',
          absaetze: [
            'A hub belongs to a club, a shared flat, a family or a project. You invite members by their openany name. Name and chat are end-to-end encrypted.',
          ],
          punkte: [
            'Share folders, calendars, albums and records',
            'Per share: view or edit',
          ],
          alt: [
            'Member list of a hub with search for inviting',
          ],
        },
        {
          titel: 'One board for everyone',
          absaetze: [
            'Every hub has a shared board. All members draw and write on it – via openany.de or directly nearby on the same network.',
          ],
          punkte: [
            'Shapes, arrows, text and pen',
            'Insert images',
            'Simultaneous changes are merged',
          ],
          alt: [
            'Board with a drawn garden bed plan',
          ],
        },
        {
          titel: 'Coordinate in the hub chat',
          absaetze: [
            'Every hub has its own chat. Type `[[` to link the hub’s content right in a message.',
          ],
          punkte: [
            'End-to-end encrypted',
            'Pin important messages',
            'Notify members if you like',
          ],
          alt: [
            'Hub chat with three example messages',
          ],
        },
      ],
    },
    nachrichten: {
      augenbraue: 'Messages',
      titel: 'Every channel in one conversation list',
      abschnitte: [
        {
          titel: 'Direct messages, encrypted',
          absaetze: [
            'Messages between openany accounts are end-to-end encrypted, attachments included. Matrix and email join them – all in one list.',
          ],
          punkte: [
            'Filters: unread, with attachment, openany, nearby',
            'Nearby works without any server',
            'By conversation or by contact',
          ],
          alt: [
            'Message list with search and filters, partly blurred',
          ],
        },
        {
          titel: 'Matrix: reach people without openany',
          absaetze: [
            'Connect an account on matrix.org or another homeserver. Your device signs in by itself, with its own keys.',
          ],
          punkte: [
            'openany.de cannot read along',
            'The history stays on the device',
            'The password is not stored',
          ],
          alt: [
            'Settings: connect a Matrix account',
          ],
        },
        {
          titel: 'Email, with PGP if you want',
          absaetze: [
            'Your existing mailbox becomes the third channel in the list. The device talks to the mail server itself.',
            'Your contacts’ public OpenPGP keys arrive with their emails – use them to encrypt your replies.',
          ],
          alt: [
            'Settings: connect an email mailbox',
            'Settings: contacts’ keys',
          ],
        },
      ],
    },
    mediathek: {
      augenbraue: 'Media library',
      titel: 'Your pictures, videos and recordings',
      abschnitte: [
        {
          titel: 'Photos, videos and audio in albums',
          absaetze: [
            'Albums with sub-albums, filled right from the app – with camera, video or microphone. Share albums in hubs.',
          ],
          punkte: [
            'Thumbnails are made on your device',
            'Location data kept if you want',
            'Stored encrypted',
          ],
          alt: [
            'Album in the media library with landscape pictures',
          ],
        },
        {
          titel: 'Many files at once',
          absaetze: [
            'Add all photos and videos from your phone at once – as a copy or without one: then they stay on the phone and go to openany.de encrypted.',
          ],
          punkte: [
            'New photos automatically, if you like',
            'Share from other apps to openany',
            'On a computer, drag files and folders onto the window or keep a folder in sync',
          ],
        },
      ],
    },
    dokumente: {
      augenbraue: 'Documents',
      titel: 'Paperwork you can search',
      abschnitte: [
        {
          titel: 'Paperwork in records',
          absaetze: [
            'Letters, contracts and documents live in records. PDFs open right in openany: page through, search, fill in forms, highlight and comment.',
          ],
          punkte: [
            'Search inside PDFs and office files',
            'Share records in hubs',
          ],
          alt: [
            'Documents with a list of records, private names blurred',
          ],
        },
        {
          pille: 'Text recognition',
          titel: 'From photo to searchable PDF',
          absaetze: [
            'Take a photo of a letter and put it in a record. It becomes a PDF with an invisible text layer: search, highlight, copy.',
            'Recognition runs on your device. The image goes nowhere.',
          ],
          alt: [
            'Notice that a photo is being converted',
            'Photographed letter as a PDF, search highlights a word',
          ],
        },
        {
          pille: 'PDF',
          titel: 'Every note as a PDF',
          absaetze: [
            'One tap turns a note into a PDF and puts it in your documents. Choose from four versions:',
          ],
          punkte: [
            'Standard, with title, date and sources',
            'Clean, just the text',
            'Letter, DIN 5008 layout',
            'Print version with a title page',
          ],
          alt: [
            'Dialog for creating a PDF with four versions',
            'Note as a finished PDF in the viewer',
          ],
        },
      ],
    },
  },
  versprechen: {
    augenbraue: 'The promise',
    titel: 'Only you can read it.',
    text: 'Everything is encrypted on your device before it reaches the server. Hosted in Germany, without ads and without tracking.',
  },
  downloadsAugenbraue: 'Downloads',
  ios: {
    titel: 'iPhone and iPad',
    pille: 'In the works',
    text: 'The iOS app is in the works. As soon as it is ready, you will find it here.',
  },
  alleVersionen: {
    titel: 'All versions',
    text: 'Older versions and all files are on GitHub.',
    knopf: 'Go to GitHub',
  },
};
