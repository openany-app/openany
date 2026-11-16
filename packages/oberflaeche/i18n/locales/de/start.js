// Startseite openany.de (frontend/start.html).
// Achtung vue-i18n: kein rohes @, | oder { in den Texten (außer Platzhaltern).
export default {
  seitentitel: 'openany – privat, verschlüsselt, zuerst auf deinem Gerät',
  kopf: 'Deine Notizen, Termine, Dateien und Nachrichten – an einem Ort.',
  unterzeile: 'Privat, Ende-zu-Ende verschlüsselt und zuerst auf deinem Gerät. Kostenlos, ohne Werbung, ohne Passwort.',
  ladenFuer: 'Für {system} laden',
  alleDownloads: 'Alle Downloads',
  iosFolgt: 'openany für iPhone und iPad ist in Arbeit.',
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
      text: 'Dein Konto legst du direkt in der App an, und es hängt an deinen Geräten. Wir speichern weder E-Mail-Adressen noch IP-Adressen.',
    },
    frei: {
      titel: 'Kostenlos und ohne Werbung',
      text: 'Kein Abo, keine Anzeigen, kein Verkauf von Daten. Zum Start bekommt jedes Konto 100 MB Speicher auf openany.de.',
    },
  },
  downloadsTitel: 'openany für dein Gerät',
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
    macos: 'Ab macOS 13.3, für Apple-Chip und Intel. Die App ist noch nicht von Apple beglaubigt: Erlaube sie beim ersten Start unter Systemeinstellungen, Datenschutz und Sicherheit, „Trotzdem öffnen“.',
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
  plattformen: 'Für Android, Windows, macOS und Linux. iPhone und iPad sind in Arbeit.',
  kopfAlt: [
    'Startübersicht der App mit Heute, Adressbuch, Notizen, Speicher und Hubs',
    'Album in der Mediathek mit Landschaftsbildern',
  ],
  rundgangKnopf: 'Rundgang ansehen',
  rundgangLabel: 'Rundgang',
  sprung: {
    notizen: 'Notizen',
    kalender: 'Kalender',
    hubs: 'Hubs',
    nachrichten: 'Nachrichten',
    mediathek: 'Mediathek',
    dokumente: 'Dokumente',
    downloads: 'Downloads',
  },
  rundgang: {
    notizen: {
      augenbraue: 'Notizen',
      titel: 'Schreiben und Gedanken verknüpfen',
      abschnitte: [
        {
          titel: 'Schreiben, ordnen, verknüpfen',
          absaetze: [
            'Notizen liegen in Mappen und werden in Markdown geschrieben – mit Überschriften, Listen und Aufgaben zum Abhaken.',
          ],
          punkte: [
            '`[[Wikilinks]]` verbinden Notizen miteinander',
            '`#Tags` sammeln, was zusammengehört',
            'Die Volltextsuche findet jedes Wort',
          ],
          alt: [
            'Notiz mit Liste, Wikilinks, Tags und Aufgaben',
          ],
        },
        {
          titel: 'Sehen, wie alles zusammenhängt',
          absaetze: [
            'Die Ansicht „Verknüpfungen“ zeigt deine Notizen als Netz. Wähle eine Mappe oder einen Tag, und die passenden Notizen erscheinen.',
          ],
          alt: [
            'Notizliste mit Mappen und Notizen, Namen verwischt',
            'Graph der Verknüpfungen zwischen Notizen, Beschriftungen verwischt',
          ],
        },
      ],
    },
    kalender: {
      augenbraue: 'Kalender',
      titel: 'Termine für dich und für alle, mit denen du planst',
      abschnitte: [
        {
          titel: 'Deine Woche im Blick',
          absaetze: [
            'Monat, Woche oder Tag: Termine mit Ort, Notizen und Wiederholung. Ein Tipp auf den Tag genügt.',
          ],
          punkte: [
            'Mehrere Kalender nebeneinander',
            '„Heute“ gleich auf der Startübersicht',
            'Verschlüsselt wie alles andere',
            'Wiederholungen wie „alle 2 Wochen dienstags, 5 Mal“',
          ],
          alt: [
            'Monatsansicht mit Beispielterminen',
          ],
        },
        {
          titel: 'Kalender in Hubs teilen',
          absaetze: [
            'Mit „ansehen“ sehen die Mitglieder deine Termine, mit „bearbeiten“ legen sie selbst welche an. Den Kalender selbst änderst nur du.',
          ],
          alt: [
            'Dialog zum Teilen eines Kalenders im Hub',
            'Hub mit geteilter Mappe, Kalender, Album und Akte',
          ],
        },
        {
          titel: 'Im Kalender und Adressbuch deines Geräts',
          absaetze: [
            'Auf Android erscheinen deine openany-Kalender und Kontakte in den Apps des Telefons – und was du dort änderst, kommt zurück. Am Computer bietet openany sie Kalender- und Adressbuch-Programmen an, nur auf diesem Rechner.',
          ],
          punkte: [
            'Erinnerungen an Termine auf dem Telefon',
            'Am Computer per CalDAV und CardDAV, zum Beispiel für Thunderbird',
          ],
        },
      ],
    },
    hubs: {
      augenbraue: 'Hubs',
      titel: 'Gemeinsam statt nebeneinander',
      abschnitte: [
        {
          titel: 'Ein Ort für eure Gruppe',
          absaetze: [
            'Ein Hub gehört einem Verein, einer WG, einer Familie oder einem Projekt. Mitglieder lädst du über ihren openany-Namen ein. Name und Chat sind Ende-zu-Ende verschlüsselt.',
          ],
          punkte: [
            'Mappen, Kalender, Alben, Ordner und Akten teilen',
            'Je Freigabe: ansehen oder bearbeiten',
          ],
          alt: [
            'Mitgliederliste eines Hubs mit Suche zum Einladen',
          ],
        },
        {
          titel: 'Ein Board für alle',
          absaetze: [
            'Jeder Hub hat ein gemeinsames Board. Alle Mitglieder zeichnen und schreiben darauf – über openany.de oder direkt vor Ort im selben Netz.',
          ],
          punkte: [
            'Formen, Pfeile, Text und Stift',
            'Bilder einfügen',
            'Gleichzeitige Änderungen werden zusammengeführt',
          ],
          alt: [
            'Board mit gezeichnetem Beetplan',
          ],
        },
        {
          titel: 'Absprechen im Hub-Chat',
          absaetze: [
            'Jeder Hub hat seinen eigenen Chat. Mit `[[` verlinkst du Inhalte des Hubs direkt in einer Nachricht.',
          ],
          punkte: [
            'Ende-zu-Ende verschlüsselt',
            'Wichtige Nachrichten anheften',
            'Mitglieder auf Wunsch per Hinweis benachrichtigen',
          ],
          alt: [
            'Hub-Chat mit drei Beispielnachrichten',
          ],
        },
      ],
    },
    nachrichten: {
      augenbraue: 'Nachrichten',
      titel: 'Alle Wege in einem Verlauf',
      abschnitte: [
        {
          titel: 'Direktnachrichten, verschlüsselt',
          absaetze: [
            'Nachrichten zwischen openany-Konten sind Ende-zu-Ende verschlüsselt, Anhänge eingeschlossen. Dazu kommen Matrix und E-Mail – alles in einer Liste.',
          ],
          punkte: [
            'Filter: ungelesen, mit Anhang, openany, vor Ort',
            'Vor Ort auch ganz ohne Server',
            'Nach Verlauf oder nach Kontakt',
          ],
          alt: [
            'Nachrichtenliste mit Suche und Filtern, Inhalte teilweise verwischt',
          ],
        },
        {
          titel: 'Matrix: auch Menschen ohne openany erreichen',
          absaetze: [
            'Verbinde ein Konto bei matrix.org oder einem anderen Homeserver. Dein Gerät meldet sich selbst an, mit eigenen Schlüsseln.',
          ],
          punkte: [
            'openany.de liest nicht mit',
            'Der Verlauf bleibt auf dem Gerät',
            'Das Passwort wird nicht gespeichert',
          ],
          alt: [
            'Einstellungen: Matrix-Konto verbinden',
          ],
        },
        {
          titel: 'E-Mail, auf Wunsch mit PGP',
          absaetze: [
            'Dein vorhandenes Postfach wird zum dritten Weg im Verlauf. Das Gerät spricht selbst mit dem Mailserver.',
            'Öffentliche OpenPGP-Schlüssel deiner Kontakte kommen von selbst aus ihren Mails – damit verschlüsselst du deine Antworten.',
          ],
          alt: [
            'Einstellungen: E-Mail-Postfach verbinden',
            'Einstellungen: Schlüssel der Kontakte',
          ],
        },
      ],
    },
    mediathek: {
      augenbraue: 'Mediathek',
      titel: 'Deine Bilder, Videos und Aufnahmen',
      abschnitte: [
        {
          titel: 'Fotos, Videos und Ton in Alben',
          absaetze: [
            'Alben mit Unteralben, direkt aus der App gefüllt – mit Kamera, Video oder Mikrofon. Alben teilst du in Hubs.',
          ],
          punkte: [
            'Vorschaubilder entstehen auf dem Gerät',
            'Standortdaten bleiben auf Wunsch erhalten',
            'Verschlüsselt gespeichert',
          ],
          alt: [
            'Album in der Mediathek mit Landschaftsbildern',
          ],
        },
        {
          titel: 'Viele Dateien auf einmal',
          absaetze: [
            'Übernimm alle Fotos und Videos deines Telefons auf einmal – als Kopie oder ohne: Dann bleiben sie auf dem Telefon und gehen verschlüsselt zu openany.de.',
          ],
          punkte: [
            'Neue Fotos auf Wunsch automatisch',
            'Aus anderen Apps an openany teilen',
            'Am Computer Dateien und Ordner ins Fenster ziehen oder einen Ordner abgleichen',
          ],
        },
      ],
    },
    dokumente: {
      augenbraue: 'Dokumente',
      titel: 'Papierkram, der sich durchsuchen lässt',
      abschnitte: [
        {
          titel: 'Papierkram in Akten',
          absaetze: [
            'Briefe, Verträge und Unterlagen liegen in Akten. PDFs öffnen direkt in openany: blättern, suchen, Formulare ausfüllen, markieren und kommentieren.',
          ],
          punkte: [
            'Suche auch im Text von PDFs und Office-Dateien',
            'Akten in Hubs teilen',
          ],
          alt: [
            'Dokumente mit einer Liste von Akten, private Namen verwischt',
          ],
        },
        {
          pille: 'Texterkennung',
          titel: 'Vom Foto zum durchsuchbaren PDF',
          absaetze: [
            'Fotografiere einen Brief ab und lege ihn in eine Akte. Daraus wird ein PDF mit unsichtbarer Textebene: durchsuchen, markieren, kopieren.',
            'Die Erkennung läuft auf deinem Gerät. Das Bild geht nirgendwohin.',
          ],
          alt: [
            'Hinweis, dass ein Foto umgewandelt wird',
            'Abfotografierter Brief als PDF, die Suche markiert ein Wort',
          ],
        },
        {
          pille: 'PDF',
          titel: 'Jede Notiz als PDF',
          absaetze: [
            'Ein Tipp macht aus einer Notiz ein PDF und legt es in deine Dokumente. Vier Fassungen stehen zur Wahl:',
          ],
          punkte: [
            'Standard, mit Titel, Stand und Quellen',
            'Clean, nur der Text',
            'Brief nach DIN 5008',
            'Druckfassung mit Titelblatt',
          ],
          alt: [
            'Dialog „PDF erstellen“ mit vier Fassungen',
            'Notiz als fertiges PDF im Betrachter',
          ],
        },
      ],
    },
  },
  versprechen: {
    augenbraue: 'Das Versprechen',
    titel: 'Nur du kannst es lesen.',
    text: 'Alles wird auf deinem Gerät verschlüsselt, bevor es den Server erreicht. Gehostet in Deutschland, ohne Werbung und ohne Tracking.',
  },
  downloadsAugenbraue: 'Downloads',
  ios: {
    titel: 'iPhone und iPad',
    pille: 'In Arbeit',
    text: 'Die iOS-App ist in Arbeit. Sobald sie fertig ist, findest du sie hier.',
  },
  alleVersionen: {
    titel: 'Alle Versionen',
    text: 'Ältere Versionen und alle Dateien liegen bei GitHub.',
    knopf: 'Zu GitHub',
  },
};
