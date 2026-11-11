// Startseite openany.de (frontend/start.html).
// Achtung vue-i18n: kein rohes @, | oder { in den Texten (außer Platzhaltern).
export default {
  seitentitel: 'openany – private, encrypted, on your device first',
  kopf: 'Your notes, appointments, files and messages – in one place.',
  unterzeile: 'Private, end-to-end encrypted and on your device first. Free, without ads, without a password.',
  ladenFuer: 'Download for {system}',
  alleDownloads: 'All downloads',
  iosFolgt: 'openany for iPhone and iPad is coming soon.',
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
      text: 'Your account is tied to your devices. We store neither email addresses nor IP addresses.',
    },
    frei: {
      titel: 'Free and without ads',
      text: 'No subscription, no ads, no selling of data. Every account starts with 100 MB of storage on openany.de.',
    },
  },
  downloadsTitel: 'Downloads',
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
    macos: 'macOS 11 or later, for Apple silicon and Intel. The app is not notarised by Apple yet: on first launch, allow it under System Settings, Privacy and Security, “Open Anyway”.',
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
};
