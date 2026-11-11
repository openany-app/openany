// Startseite openany.de (frontend/start.html).
// Achtung vue-i18n: kein rohes @, | oder { in den Texten (außer Platzhaltern).
export default {
  seitentitel: 'openany – privat, verschlüsselt, zuerst auf deinem Gerät',
  kopf: 'Deine Notizen, Termine, Dateien und Nachrichten – an einem Ort.',
  unterzeile: 'Privat, Ende-zu-Ende verschlüsselt und zuerst auf deinem Gerät. Kostenlos, ohne Werbung, ohne Passwort.',
  ladenFuer: 'Für {system} laden',
  alleDownloads: 'Alle Downloads',
  iosFolgt: 'Für iPhone und iPad folgt openany bald.',
  merkmale: {
    verschluesselt: {
      titel: 'Nur du kannst es lesen',
      text: 'Alles wird auf deinem Gerät verschlüsselt, bevor es den Server erreicht. Wir können deine Inhalte nicht lesen – auch nicht, wenn wir wollten.',
    },
    geraet: {
      titel: 'Zuerst auf deinem Gerät',
      text: 'openany funktioniert ohne Internet und ohne Konto. Der Abgleich über openany.de ist ein Angebot, keine Pflicht.',
    },
    alles: {
      titel: 'Alles an einem Ort',
      text: 'Notizen, Kalender, Kontakte, Dateien, Fotos, Videos und Nachrichten – in einer App statt in zehn.',
    },
    hubs: {
      titel: 'Gemeinsam in Hubs',
      text: 'Teile Mappen, Kalender und Alben mit anderen, zeichnet zusammen auf einem Board und schreibt euch im Chat.',
    },
    sparsam: {
      titel: 'Kein Passwort, keine E-Mail-Adresse',
      text: 'Dein Konto hängt an deinen Geräten. Wir speichern weder E-Mail-Adressen noch IP-Adressen.',
    },
    frei: {
      titel: 'Kostenlos und ohne Werbung',
      text: 'Kein Abo, keine Anzeigen, kein Verkauf von Daten. Zum Start bekommt jedes Konto 100 MB Speicher auf openany.de.',
    },
  },
  downloadsTitel: 'Downloads',
  version: 'Version {version}',
  empfohlen: 'Für dein Gerät',
  beiGithub: 'Die Dateien liegen bei GitHub.',
  keineListe: 'Die aktuelle Liste lässt sich gerade nicht laden.',
  dateien: {
    apk: 'APK laden',
    installer: 'Installer (.exe)',
    msi: 'MSI-Paket',
    dmg: 'Für Mac laden (.dmg)',
    appimage: 'AppImage',
    deb: 'Debian und Ubuntu (.deb)',
  },
  hinweise: {
    android: 'Direkt als Datei, nicht im Play Store. Beim ersten Mal fragt Android, ob du Apps aus dieser Quelle erlauben möchtest.',
    windows: 'Für Windows 10 und 11. Die App ist noch nicht signiert: Warnt Windows, wähle „Weitere Informationen“ und dann „Trotzdem ausführen“.',
    macos: 'Ab macOS 11, für Apple-Chip und Intel. Die App ist noch nicht von Apple beglaubigt: Erlaube sie beim ersten Start unter Systemeinstellungen, Datenschutz und Sicherheit, „Trotzdem öffnen“.',
    linux: 'AppImage für fast jede Verteilung, .deb für Debian und Ubuntu. 64 Bit.',
  },
  fuss: {
    melden: 'Melden und Kontakt',
    agb: 'Nutzungsbedingungen',
    datenschutz: 'Datenschutz',
    impressum: 'Impressum',
    github: 'Alle Versionen auf GitHub',
  },
  recht: {
    zurueck: 'Zur Startseite',
    nurDeutsch: 'Dieser Text gilt nur in der deutschen Fassung.',
    fehler: 'Der Text lässt sich gerade nicht laden. Versuch es später noch einmal.',
  },
};
