// Hilfe-Seite (/hilfe): Handbuch zu allen Modulen und Funktionen.
// Die Gliederung (Abschnitte, Unterabschnitte, Absatz-Anzahl) steht in
// shared/helpOutline.js und wird gegen beide Sprachen geprüft – hier stehen
// nur die Texte.
// Achtung vue-i18n: kein rohes @, | oder { in den Texten.
export default {
  title: 'Hilfe',
  intro: 'Hier findest du Beschreibungen und Anleitungen zu allen Modulen und Funktionen von Openany. Über das Inhaltsverzeichnis links springst du direkt zum passenden Abschnitt.',
  version: 'Version {version}',
  toc: 'Inhalt',
  sections: {
    start: {
      title: 'Erste Schritte',
      subs: {
        konto: {
          title: 'Konto und Anmeldung',
          p1: 'Konten bei Openany entstehen ausschließlich auf persönliche Einladung – es gibt keine offene Registrierung. Eine E-Mail-Adresse ist freiwillig: Openany funktioniert auch ganz ohne, mit E-Mail kannst du ein vergessenes Passwort aber selbst zurücksetzen.',
          p2: 'Angemeldet wird mit Nutzername und Passwort über anyid, den gemeinsamen Anmeldedienst. Dieselbe Anmeldung gilt für Openany, anyitem und anytail; Passwort und zweiter Faktor werden deshalb dort verwaltet und nicht in Openany selbst (siehe „Sicherheit" unter den Einstellungen).',
        },
        ohneKonto: {
          title: 'Die App ohne Konto',
          p1: 'Die App ist ein vollständiges Programm, das auch ganz ohne Konto und ohne Server auskommt: Notizen, Kalender, Speicher, Adressbuch und Nachrichten liegen dann nur auf diesem Gerät. Was hier nicht eingerichtet ist oder nicht geht, blendet die App aus, statt es leer anzuzeigen – der Weg Openany bei den Nachrichten etwa erscheint erst, wenn das Gerät mit openany.de verbunden ist.',
          p2: 'Willst du deine Daten mit Openany im Netz abgleichen, tippst du in den Einstellungen unter „Abgleich" auf „Mit openany.de verbinden"; mit deinen anderen Geräten paarst du dich unter „Geräte in der Nähe". Danach steht in der Kopfzeile und im Menü ein Knopf „Abgleichen", der sich dreht, solange abgeglichen wird.',
        },
        startseite: {
          title: 'Die Startseite',
          p1: 'Die Startseite zeigt kompakte Kacheln der Module, die du dafür ausgewählt hast: die nächsten Termine, die zuletzt bearbeiteten Notizen, die letzten Dateien samt Speicher-Balken, deine Projekte mit Ungelesen-Zähler, die neuesten Nachrichten im Posteingang und das Adressbuch. Jede Kachel führt in ihr Modul; welche erscheinen, legst du in den Einstellungen unter „Module auf der Startseite" fest.',
          p2: 'Hast du keine Module für die Startseite ausgewählt, erscheint stattdessen die Willkommens-Übersicht: Sie stellt alle Funktionen vor und lässt dich die Kacheln direkt zusammenstellen. Die Auswahl blendet nichts aus – alle Bereiche bleiben über die Leiste und das Menü erreichbar.',
        },
        navigation: {
          title: 'Menü und Aufbau',
          p1: 'Am Computer liegt oben eine Kopfzeile: Links führt das runde Logo zur Startseite, daneben stehen Notizen, Kalender, Speicher und Projekte. Rechts folgen der Hell-/Dunkel-Umschalter, der Briefumschlag zu den Nachrichten mit Ungelesen-Zähler und das Feld mit deinem Namen – es öffnet Einstellungen, Hilfe, Papierkorb und Abmelden.',
          p2: 'Auf dem Handy rückt alles nach unten: Eine feste Leiste zeigt die Module als Symbole, ganz links davon sitzt der Menü-Knopf mit dem Zähler für ungelesene Nachrichten. Er öffnet ein Feld mit Nachrichten, Einstellungen, Hilfe, Papierkorb, dem Hell-/Dunkel-Umschalter und dem Abmelden. Der Umschalter wechselt zwischen hell, dunkel und automatisch – automatisch folgt der Einstellung deines Geräts.',
        },
      },
    },
    notes: {
      title: 'Notizen',
      subs: {
        editor: {
          title: 'Schreiben im Editor',
          p1: 'Der Notiz-Editor arbeitet wie eine moderne Textverarbeitung: Überschriften, Listen, Zitate, Codeblöcke, Fett- und Kursivschrift wählst du über die Werkzeugleiste. Alternativ tippst du Markdown-Kürzel – zum Beispiel eine Raute mit Leerzeichen für eine Überschrift oder einen Spiegelstrich für eine Liste – und sie verwandeln sich beim Schreiben in Formatierung.',
          p2: 'Gespeichert wird automatisch: Kurz nachdem du aufhörst zu tippen, sichert Openany die Notiz und zeigt den Speicherstatus in der Werkzeugleiste an. Unter jeder Notiz stehen außerdem das Erstell- und das letzte Änderungsdatum. Auf dem Handy erscheint die Werkzeugleiste einzeilig direkt über der Tastatur, sobald du in den Text tippst, und lässt sich seitlich durchscrollen – so sind alle Werkzeuge griffbereit, stören aber beim Lesen nicht.',
        },
        mappen: {
          title: 'Mappen und Struktur',
          p1: 'Notizen liegen in Mappen, die sich beliebig tief verschachteln lassen. Notizen und Mappen kannst du umbenennen, verschieben und neu anordnen – so wächst mit der Zeit eine eigene Wissenssammlung. „Finden“ oben (Lupe) öffnet ein Suchfeld, das alle Notizen auf einmal durchsucht, Titel und Inhalt; öffnest du sie in einer Mappe, lässt sich die Suche mit „nur in dieser Mappe“ darauf beschränken. Was sich mit der geöffneten Mappe tun lässt, steht rechts in der Pfadleiste im Menü „Mappe“.',
          p2: 'Eine ganze Mappe samt Untermappen lädst du als ZIP-Archiv herunter; die Notizen liegen darin als Markdown-Dateien und bleiben damit überall lesbar.',
        },
        wikilinks: {
          title: 'Wikilinks und Rückverweise',
          p1: 'Mit zwei eckigen Klammern verlinkst du Notizen untereinander: Tippe die Klammern und den Titel der Ziel-Notiz, und der Link entsteht automatisch – existiert die Notiz noch nicht, wird der Link aktiv, sobald es sie gibt. Soll im Text etwas anderes stehen als der Titel, gibst du hinter einem senkrechten Strich einen eigenen Anzeigetext an; verlinkt bleibt trotzdem die richtige Notiz.',
          p2: 'Jede Notiz zeigt ihre Rückverweise: eine Liste aller Notizen, die auf sie verlinken. So findest du Zusammenhänge wieder, ohne selbst Buch führen zu müssen.',
        },
        tags: {
          title: 'Tags',
          p1: 'Schlagworte vergibst du direkt im Text mit einer Raute vor dem Wort; beim Tippen hilft eine Vorschlagsliste bereits vergebener Tags. Mit einem Schrägstrich verschachtelst du Schlagworte – etwa Projekt/Unterthema; ein Filter auf das Oberthema findet dann auch alle Unterthemen. In der Graph-Ansicht lassen sich die Schlagworte als eigene Knoten einblenden, und ein Klick auf ein Schlagwort dort filtert die Notizliste darauf.',
        },
        einfuegen: {
          title: 'Bilder und Dateien einfügen',
          p1: 'Über den Büroklammer-Knopf in der Werkzeugleiste fügst du Inhalte in eine Notiz ein – entweder frisch hochgeladen oder als Verweis auf bereits in Openany gespeicherte Inhalte. Bilder erscheinen als platzsparende Vorschau direkt in der Notiz (die volle Auflösung liegt in der Galerie), andere Dateien als anklickbarer Link zum Herunterladen.',
          p2: 'Eingefügte Inhalte werden automatisch einsortiert: Bilder in ein Galerie-Album, Text- und Office-Dateien in eine Dokumenten-Akte, alles Übrige in einen Datei-Ordner – jeweils in einem eigenen Bereich für Notizen. Solange der Verweis in mindestens einer Notiz steht, lässt sich die Datei in Speicher oder Galerie nicht in den Papierkorb legen; entfernst du den Verweis aus dem Text, wird sie wieder frei löschbar.',
        },
        zitate: {
          title: 'Zitate und Literatur',
          p1: 'Für wissenschaftliches Arbeiten kannst du einer Mappe eine Literaturdatei im CSL-JSON-Format zuweisen – etwa einen Export aus Zotero, den du zuvor in den Speicher hochgeladen hast. In der geöffneten Mappe stellt das Menü „Mappe“ → „Bibliothek verknüpfen" die Verbindung her; Untermappen erben die Bibliothek automatisch.',
          p2: 'In Notizen dieser Mappe schlägt der Editor beim Zitieren passende Quellen vor und setzt einen Kurzbeleg in den Text. So entstehen Belege beim Schreiben, ohne die Quellenangaben jedes Mal abzutippen.',
        },
        graph: {
          title: 'Graph-Ansicht',
          p1: 'Die Graph-Ansicht zeichnet deine Notizen als Netz: Jede Notiz ist ein Punkt, jeder Wikilink eine Verbindung. Die Punkte sind nach oberster Mappe eingefärbt, sodass zusammengehörige Bereiche schon an der Farbe erkennbar sind; hohle Punkte stehen für Notizen ohne jede Verbindung, und je mehr Verknüpfungen eine Notiz hat, desto größer ihr Punkt.',
          p2: 'Jede oberste Mappe ist eine Ebene. Gibt es mehrere, wählst du in der Legende unter dem Graphen eine oder mehrere aus – „Alle Ebenen" nimmt alle, „Keine" leert die Auswahl. Statt einer Ebene kann auch ein Tag der Einstieg sein: Die Leiste darunter nennt die Tags mit ihrer Anzahl, und ein gewählter Tag zeigt alle Notizen, die ihn tragen, samt dem Tag als grauer Raute. Sind Ebenen gewählt, stehen in der Leiste nur noch die Tags, die darin vorkommen. Bis du etwas wählst, bleibt der Graph leer, damit große Sammlungen nicht auf einmal aufgebaut werden; „Alle Tags anzeigen" holt alle Tags der Leiste auf einmal hinein.',
          p3: 'Am Computer zoomst du mit den Plus-/Minus-Knöpfen oben rechts oder dem Mausrad und verschiebst die Ansicht mit gedrückter Maustaste; auf dem Handy zoomst du mit zwei Fingern und verschiebst mit einem. Ein weiterer Knopf setzt die Ansicht zurück. Ein Klick auf einen Punkt (ohne Ziehen) öffnet die Notiz, ein Klick auf einen Tag filtert die Notizliste danach.',
        },
        export: {
          title: 'PDF-Export',
          p1: 'Jede Notiz lässt sich als PDF ausgeben – zum Drucken, Archivieren oder Weitergeben. Die Formatierung aus dem Editor bleibt dabei erhalten. Das PDF landet in der Akte „Notizen“ unter Speicher → Dokumente (frühere Fassungen bleiben daneben erhalten); „Öffnen“ in der Meldung zeigt es gleich an.',
        },
        druckfassung: {
          title: 'Druckfassung',
          p1: 'Aus einer Mappe wird ein förmliches Dokument zum Abgeben oder Drucken: In der geöffneten Mappe wählst du im Menü „Mappe“ „Druckfassung“, trägst Titel, Untertitel, Name, Einrichtung, Datum und Zusatzangaben fürs Titelblatt ein und entscheidest, ob ein Inhaltsverzeichnis dazukommt. Jede Notiz der Mappe wird ein Kapitel, alphabetisch nach Titel – mit Vorsilben wie „01 Einleitung“ legst du die Reihenfolge fest; im Dokument erscheint dann nur „Einleitung“. Untermappen kommen mit, wenn der Haken „Untermappen einbeziehen“ gesetzt ist: Eine Untermappe wird eine Überschrift, ihre Notizen rücken darunter („2 Grundlagen“, „2.1 Begriffe“); eine Notiz, die heißt wie ihre Untermappe, steht als Text direkt unter deren Überschrift.',
          p2: 'Gesetzt wird in Times-artiger Schrift mit serifenlosen Überschriften, 2,5 cm Rand, eineinhalbzeilig, im Blocksatz mit Silbentrennung; die Seitenzahlen beginnen nach dem Titelblatt. Die Angaben bleiben an der Mappe gespeichert. Das PDF entsteht auf deinem Gerät – beim ersten Mal dauert es einige Sekunden – und landet in der Akte „Druckfassungen“ unter Speicher → Dokumente; jede Fassung bleibt erhalten.',
          p3: "Zitate: In der geöffneten Mappe öffnest du im Menü „Mappe“ → „Quellen“ die Quellenliste und trägst Bücher, Aufsätze, Webseiten, Urteile oder Gesetze ein; jede bekommt einen Kurznamen. Im Text schreibst du [{'@'}kurzname] oder [{'@'}kurzname, S. 12], für eine Fußnote ^[Text]. Im PDF – Notiz wie Druckfassung – entstehen daraus Harvard-Zitate wie „(Müller, 2020, S. 12)“ und am Ende das Literaturverzeichnis. Die Liste ist eine CSL-JSON-Datei unter Dateien, also auch mit Zotero und Zettlr verwendbar. Für Fußnoten gibt es in der Werkzeugleiste des Editors einen eigenen Knopf (hochgestellte Zahl): Er setzt die Klammern, markierter Text wird zur Fußnote.",
        },
        papierkorb: {
          title: 'Papierkorb',
          p1: 'Gelöschte Notizen landen im Papierkorb (im Menü unter deinem Namen, auf dem Handy im Menü-Knopf) und lassen sich dort wiederherstellen oder endgültig löschen. Beim Löschen einer Mappe entscheidest du, ob die enthaltenen Notizen mitgelöscht oder in die übergeordnete Mappe verschoben werden.',
        },
        verschluesselt: {
          title: 'Ende-zu-Ende verschlüsselt',
          p1: 'Deine Notizen, Mappen, Druckfassungs-Angaben und Quellenlisten liegen auf openany.de nur verschlüsselt: Lesen können sie nur deine freigegebenen Geräte. Auch die Namen deiner Mappen sieht der Server nicht. Suche, Rückverweise, Schlagwörter und Graph rechnet dein Gerät.',
        },
        ordner: {
          title: 'Notizen als Dateien (Zettlr)',
          p1: 'In der App auf dem Computer kannst du eine eigene Mappe mit einem Ordner verbinden: Mappe öffnen, im Menü „…“ „Mit Ordner verbinden …“ wählen und den Ordner aussuchen. Jede Notiz liegt dort als Datei „Id Titel.md“, Untermappen als Unterordner – so, wie Zettlr oder Obsidian es erwarten. Was du dort änderst, umbenennst, verschiebst oder neu anlegst, übernimmt die App, solange sie läuft; haben sich beide Seiten geändert, entsteht eine Konfliktkopie. Löschst du eine Datei, landet die Notiz im Papierkorb. Der Ordner liegt unverschlüsselt auf deinem Computer; „Ordner trennen“ lässt die Dateien liegen. Bilder in Notizen liegen neben der Notiz, unter dem Namen aus dem Text; ein neues Bild dort, das eine Notiz nennt, kommt in die App (Galerie, Album „Notizen: …“).',
        },
      },
    },
    files: {
      title: 'Speicher',
      subs: {
        aufbau: {
          title: 'Galerie, Dateien, Dokumente',
          p1: 'Speicher teilt sich in drei Reiter, die oben in der Kopfzeile stehen: die Galerie für Fotos und Videos, Dateien für alles Mögliche und Dokumente für Schriftstücke in Akten. Wechselst du den Reiter, bleibt der andere Bereich, wie du ihn verlassen hast – geöffnete Ordner und geladene Listen gehen nicht verloren.',
          p2: 'Welcher Reiter zuerst aufgeht, wenn du Speicher öffnest, legst du in den Einstellungen unter „Start unter Speicher" fest; solange du nichts wählst, sind es die Dokumente.',
        },
        dokumente: {
          title: 'Dokumentenablage',
          p1: 'Im Reiter „Dokumente" organisierst du PDF-, Office- und Textdateien in Akten. Hochladen geht per Knopf oder indem du Dateien einfach in die geöffnete Akte ziehst. Auf Handy und Tablet steht daneben ein Kamera-Knopf: Er fotografiert ein Schriftstück und legt es gleich als Dokument in die Akte.',
          p2: 'Akten lassen sich beliebig tief verschachteln. Akten wie einzelne Dokumente kannst du umbenennen und verschieben; ganze Akten außerdem in Projekte freigeben und als ZIP herunterladen.',
        },
        texterkennung: {
          title: 'Durchsuchbare Dokumente',
          p1: 'Fotografierst du ein Schriftstück ab und legst es in eine Akte – mit dem Kamera-Knopf oder als Foto aus deiner Bildersammlung –, wird daraus automatisch ein PDF mit einer unsichtbaren Textebene. Das Dokument sieht aus wie dein Foto, lässt sich aber durchsuchen, markieren, kopieren und vorlesen. Die Erkennung läuft auf deinem eigenen Gerät – das Bild wird dafür nirgendwo hingeschickt.',
          p2: 'Legst du mehrere Fotos auf einmal ab, wirst du gefragt, ob daraus ein mehrseitiges Dokument werden soll – etwa die Seiten eines Briefes – oder einzelne. Das Foto selbst wird nicht zusätzlich gespeichert; wer das Bild behalten möchte, legt es in der Galerie ab. Beim ersten Mal dauert die Umwandlung länger, weil die Texterkennung einmalig geladen und danach gespeichert wird.',
          p3: 'Liegt schon ein eingescanntes PDF in einer Akte, kannst du die Erkennung nachträglich darauf anwenden: „Text erkennen" liest das Dokument Seite für Seite. Ist das Papier sauber weiß, bleiben die Seiten unverändert und es kommt nur der Text hinzu. Sind sie grau oder ungleichmäßig ausgeleuchtet – wie bei einem abfotografierten Blatt –, werden sie zusätzlich aufgehellt; die Meldung am Ende sagt dir, was passiert ist. War das PDF bereits durchsuchbar, bleibt es unangetastet.',
        },
        dateien: {
          title: 'Dateien verwalten',
          p1: 'Im Reiter „Dateien" legst du Ordner an und lädst Dateien jeder Art hoch – einzeln oder mehrere auf einmal. Ordner wie Dateien lassen sich umbenennen und verschieben, Dateien wieder herunterladen; Ordner kannst du außerdem in Projekte freigeben und als ZIP herunterladen. Was sich mit einem Ordner oder einer Akte tun lässt – herunterladen, freigeben, umbenennen, verschieben, löschen –, steht im geöffneten Ordner rechts in der Pfadleiste im Menü „Ordner“ (in einer Akte heißt es „Akte“).',
          p2: 'Jedes Konto hat ein Speicher-Kontingent; die aktuelle Belegung siehst du auf der Startseiten-Kachel und in den Einstellungen unter „Speicherplatz".',
        },
        suche: {
          title: 'Suchen',
          p1: 'Die Lupe in Dateien und Dokumenten öffnet ein Suchfeld, das alle Ordner beziehungsweise Akten auf einmal durchsucht, nicht nur den gerade geöffneten. Ein Klick auf einen Treffer öffnet die Datei; daneben steht, wo sie liegt, und ein Klick auf diese Angabe bringt dich in den Ordner.',
          p2: 'Gefunden wird nicht nur nach dem Namen, sondern auch im Text: in PDFs, Word- und OpenDocument-Texten sowie Text-, CSV- und Markdown-Dateien bis 50 MB. Openany liest diesen Text nach und nach im Hintergrund, solange Speicher geöffnet ist; bis alles gelesen ist, sagt dir die Suche, wie viele Dateien vorerst nur nach Namen gefunden werden. Bei einem Treffer im Text zeigt die Liste die Fundstelle mit dem gesuchten Wort hervorgehoben. Eingescannte PDFs ohne Textebene findet die Suche erst, nachdem du „Text erkennen" darauf angewendet hast.',
        },
        pdf: {
          title: 'PDFs ansehen und bearbeiten',
          p1: 'Ein Klick auf ein PDF öffnet es direkt in Openany, ohne Herunterladen. Du blätterst durch die Seiten, vergrößerst oder verkleinerst, passt die Ansicht an die Breite an und durchsuchst das Dokument mit der Lupe – die Treffer werden auf der Seite hervorgehoben. Ist ein PDF mit Passwort geschützt, fragt Openany zuerst danach.',
          p2: 'Formulare im PDF füllst du direkt aus. Über „Bearbeiten" erscheint außerdem eine Werkzeugleiste: „Markieren" hebt Text farbig hervor, mit dem „Stift" zeichnest du frei auf die Seite, und mit „Text" setzt du eigene Zeilen aufs Blatt – Farbe, Strichstärke und Schriftgröße wählst du selbst, und jeder Schritt lässt sich rückgängig machen. Rechenfelder in Formularen rechnen allerdings nicht mit, weil Openany keine Skripte aus fremden Dateien ausführt.',
          p3: 'An Markierungen und Striche hängst du Kommentare: Wähle „Kommentare" und tippe die Stelle an; dieselbe Ansicht listet alle Kommentare des Dokuments auf. Gespeichert wird alles als gewöhnliche PDF-Anmerkung, die auch andere Programme anzeigen. Beim ersten Speichern legt Openany die Fassung davor in den Papierkorb, sodass du zum Original zurückkannst; verlässt du das PDF mit ungespeicherten Änderungen, fragt es vorher nach.',
        },
        galerie: {
          title: 'Fotogalerie und Alben',
          p1: 'Im Reiter „Galerie" liegen deine Fotos und Videos – in Alben mit Vorschau-Kacheln oder, ohne Album, darunter unter „Bilder". Alben lassen sich verschachteln, etwa ein Album je Reise mit einem Unteralbum je Tag, und jederzeit verschieben. Die Lightbox blättert durch alle Bilder eines Albums. Herunterladen, Freigeben, Verschieben und Löschen eines Albums findest du im geöffneten Album rechts in der Pfadleiste im Menü „Album“.',
          p2: 'Die neueste Aufnahme steht oben, gegliedert nach Monaten mit einer Überschrift je Monat. Maßgeblich ist das Aufnahmedatum aus den Fotodaten; fehlt es – etwa bei Bildschirmfotos oder bearbeiteten Bildern –, zählt der Tag des Hochladens.',
          p3: 'Ganze Alben lädst du als ZIP herunter, einzelne Bilder als Originaldatei. Alben lassen sich ebenfalls in Projekte freigeben.',
        },
        videos: {
          title: 'Videos und Kamera',
          p1: 'Neben Fotos nimmt die Galerie auch Videos auf. Sie erscheinen als Kachel mit einem Standbild und spielen in der Lightbox ab. Manche Handys nehmen im Format HEVC auf, das nicht jeder Browser abspielen kann; dann lädst du das Video herunter und öffnest es mit einem anderen Programm.',
          p2: 'Auf Handy und Tablet stehen in der Galerie und in jedem Album zwei Kamera-Knöpfe: einer für ein Foto, einer für ein Video. Die Aufnahme landet ohne Umweg an der Stelle, die gerade geöffnet ist. Am Computer gibt es die Knöpfe nicht, dort lädst du wie gewohnt hoch.',
        },
        karte: {
          title: 'Aufnahmedatum und Ort',
          p1: 'Zu jedem Bild zeigt die Lightbox das Aufnahmedatum und – sofern das Foto ihn enthält – den Aufnahmeort; „Auf Karte zeigen" öffnet die Stelle auf OpenStreetMap. Lädst du Fotos vom Android-Handy hoch, wähle sie über die Dateien-App aus und nicht über die Galerie: Sonst entfernt Android den Standort, bevor das Bild bei Openany ankommt.',
        },
        papierkorbSpeicher: {
          title: 'Papierkorb',
          p1: 'Gelöschte Inhalte – Dokumente, Dateien, Fotos, Notizen, Kontakte, aber auch Projekte, Kalender und Termine – wandern zunächst in den gemeinsamen Papierkorb und lassen sich dort wiederherstellen. Nach 30 Tagen leert sich der Papierkorb automatisch; erst dann ist der Speicherplatz endgültig frei.',
        },
        dateienVerschluesselt: {
          title: 'Ende-zu-Ende verschlüsselt',
          p1: 'Deine Dateien, Akten, Fotos und Alben liegen auf openany.de nur verschlüsselt: Namen, Ordner, Alben, Aufnahmedaten und Ort stehen in einem verschlüsselten Kopf, die Inhalte selbst sind mit einem eigenen Schlüssel je Datei verschlüsselt. Lesen können sie nur deine freigegebenen Geräte. Beim ersten Abgleich mit dieser Fassung holt die App alle Originale, verschlüsselt sie und lädt sie neu hoch – das dauert eine Weile und läuft am besten im WLAN. Was zu dem Zeitpunkt im Papierkorb liegt, wird dabei endgültig gelöscht, und Freigaben von Alben und Ordnern in Projekten enden.',
        },
      },
    },
    calendar: {
      title: 'Kalender',
      subs: {
        verwalten: {
          title: 'Mehrere Kalender',
          p1: 'Du kannst beliebig viele Kalender nebeneinander führen – etwa privat, Familie, Verein –, jeder mit eigener Farbe. Einzelne Kalender blendest du per Klick ein und aus. Oben wechselst du zwischen Monats- und Wochenansicht; die Woche zeigt die Termine nach Uhrzeit, und vergangene Termine erscheinen dezent ausgegraut.',
        },
        termine: {
          title: 'Termine anlegen',
          p1: 'Termine erstellst du per Klick auf einen Tag: mit Titel, Beschreibung, Uhrzeit oder als ganztägiges beziehungsweise mehrtägiges Ereignis. Auch Wiederholungen sind möglich – täglich, wöchentlich, monatlich oder jährlich. Bestehende Termine bearbeitest oder löschst du direkt aus der Ansicht. Die Uhrzeiten gelten in der Zeitzone, die in deinem Profil eingestellt ist.',
        },
        abos: {
          title: 'Kalender-Abos',
          p1: 'Externe Kalender – etwa Feiertage oder der Spielplan des Vereins – bindest du per Abo-URL im ICS-Format ein. Abonnierte Kalender aktualisieren sich automatisch. Ihre Termine sind in Openany nur zu lesen: Geändert wird dort, wo der Kalender herkommt, denn jeder Abruf ersetzt den Inhalt wieder durch das Original.',
        },
        verschluesselt: {
          title: 'Ende-zu-Ende verschlüsselt',
          p1: 'Deine Kalender und Termine liegen auf openany.de nur verschlüsselt: Lesen können sie nur deine freigegebenen Geräte. Auch die Adresse eines Abos ist verschlüsselt; abgerufen wird es von jedem Gerät selbst, openany.de ruft nichts ab.',
        },
        importExport: {
          title: 'Import und Export',
          p1: 'Über „Importieren“ liest du eine ICS-Datei in einen deiner Kalender ein. Liest du dieselbe Datei noch einmal ein, werden die Termine aktualisiert statt verdoppelt. Das Teilen-Symbol am Kalender speichert ihn als ICS-Datei – zum Mitnehmen in jedes andere Kalenderprogramm.',
        },
      },
    },
    projects: {
      title: 'Projekte',
      subs: {
        grundlagen: {
          title: 'Projekte und Mitglieder',
          p1: 'Ein Projekt ist ein gemeinsamer Arbeitsbereich: Du lädst andere Nutzer als Mitglieder ein, die Eingeladenen nehmen die Einladung an oder lehnen sie ab. Der Projekt-Eigentümer verwaltet die Mitglieder und kann das Projekt auch an ein anderes Mitglied übergeben.',
          p2: 'Jedes Projekt hat vier Reiter: Chat, Planung, Mitglieder und Freigaben. Die Zahlen an den Reitern zeigen ungelesene Chat-Nachrichten beziehungsweise, wie viel darin liegt.',
        },
        chat: {
          title: 'Projekt-Chat',
          p1: 'Jedes Projekt hat einen gemeinsamen Chat in Echtzeit: Nachrichten erscheinen sofort bei allen Mitgliedern, ohne die Seite neu zu laden. Der Chat ist Ende-zu-Ende verschlüsselt: Lesen können ihn nur die Mitglieder auf ihren Geräten, openany.de nicht. Tritt jemand aus, wechselt der Schlüssel von selbst – Neues liest er dann nicht mehr.',
          p2: 'Wichtige Nachrichten kann jedes Mitglied anheften – angeheftete Nachrichten sind für alle in der Pin-Leiste über dem Verlauf schnell wiederzufinden und lassen sich dort auch wieder lösen.',
          p3: 'Mit zwei eckigen Klammern verweist du mitten im Satz auf Inhalte des Projekts: auf Notizen sowie auf freigegebene Ordner, Dateien, Alben und Bilder. Eine Vorschlagsliste zeigt beim Tippen, was in Frage kommt, und setzt den Verweis fertig ein. Denselben Dienst tut der Knopf mit dem Ketten-Symbol neben dem Senden-Knopf – auf dem Handy meist bequemer, als zwei Klammern zu suchen.',
          p4: 'Ein Klick auf einen Verweis führt direkt zum Ziel: in die Notiz, in den Ordner mit hervorgehobener Datei oder zum Bild in der Lightbox. Verweisen kannst du nur auf das, was im Projekt ohnehin sichtbar ist – was niemand freigegeben hat, taucht weder in der Vorschlagsliste auf, noch wird ein Verweis darauf anklickbar.',
        },
        planung: {
          title: 'Der Planung-Tab',
          p1: 'Im Reiter „Planung" sammelt das Projekt seine Abstimmungen und Listen. Über den „Neu"-Knopf wählst du den passenden Typ aus: Terminfindung, Umfrage, Schichtplan, Mitbringliste oder Einladungsliste. Erstellen und abstimmen darf jedes Mitglied; löschen darf der jeweilige Ersteller oder der Projekt-Eigentümer.',
          p2: 'Abstimmungen sind Ende-zu-Ende verschlüsselt – auch die Stimmen. Die Geräte der Mitglieder zählen selbst und achten auf die Regeln: Bei begrenzten Plätzen zählt, wer sich zuerst eingetragen hat. Abschließen kann nur, wer die Abstimmung angelegt hat; bei einer Terminfindung trägt dein Gerät den gewählten Termin auf Wunsch in deinen Kalender ein.',
        },
        terminfindung: {
          title: 'Terminfindung',
          p1: 'Bei der Terminfindung schlägst du mehrere Terminoptionen vor und alle stimmen mit Ja, Nein oder Vielleicht ab – so vereinbart ihr ein Treffen ohne langes Hin und Her.',
        },
        umfragen: {
          title: 'Umfragen',
          p1: 'Eine Umfrage stellt eine Frage mit festen Antworten – Ja/Nein oder eigene, auf Wunsch anonym. Mit abhakbaren Optionen dient sie auch als gemeinsame Aufgabenliste; Abstimmungen lassen sich duplizieren und schließen.',
        },
        listen: {
          title: 'Schichtplan, Mitbringliste, Einladungsliste',
          p1: 'Drei Listen-Typen nehmen Orga-Arbeit ab: Beim Schichtplan legst du Schichten mit Plätzen an und die Mitglieder tragen sich ein. Die Mitbringliste klärt, wer was mitbringt – Einträge lassen sich übernehmen und ergänzen. Die Einladungsliste behält Gäste im Blick (eingeladen, zugesagt, abgesagt), auch solche ohne Openany-Konto.',
        },
        geteilteKalender: {
          title: 'Kalender teilen',
          p1: 'Unter „Planung“ teilst du einen deiner Kalender mit dem Projekt: Die Mitglieder sehen dort die nächsten Termine und im eigenen Kalender unter „Aus Projekten“ alle – nur zum Lesen. Ändern kannst nur du; Änderungen kommen beim nächsten Abgleich an. Alles ist mit dem Projektschlüssel verschlüsselt. Über den Papierkorb neben dem Kalender beendest du das Teilen; dein Kalender bleibt.',
        },
        freigaben: {
          title: 'Inhalte freigeben',
          p1: 'Im Reiter „Freigaben“ teilst du mit „Teilen“ eine Notiz-Mappe, ein Album oder einen Ordner (auch eine Akte) mit dem Projekt: erst die Art wählen, dann was. Alles bleibt Ende-zu-Ende verschlüsselt – mit einem eigenen Schlüssel je Freigabe, der wechselt, wenn du das Teilen beendest oder jemand das Projekt verlässt. Die Inhalte bleiben bei dir; endet das Teilen, bleibt dir alles erhalten.',
          p2: 'Alben und Ordner sind nur zum Ansehen geteilt: Die Mitglieder öffnen sie im Projekt, sehen Unteralben und Unterordner, öffnen Bilder und PDFs und legen mit „Übernehmen“ eine Kopie unter „Aus Projekten“ in ihren eigenen Speicher. Ändern kannst nur du.',
          p3: 'Notiz-Mappen teilst du mit der Stufe „ansehen“ oder „bearbeiten“. Die Mitglieder öffnen sie im gewohnten Notiz-Editor und sehen auch die Bilder darin; mit „bearbeiten“ ändern sie Notizen und legen neue an. Bilder einfügen kann vorerst nur, wem die Mappe gehört.',
        },
        lokal: {
          title: 'Projekte nur vor Ort',
          p1: 'In der App legst du mit „+ Projekt" ein Projekt nur auf diesem Gerät an – ohne Konto und ohne Server; es trägt die Marke „Nur vor Ort". Andere lädst du unter „Mitglied in der Nähe einladen" ein: Auf dem anderen Gerät muss Openany offen sein, im selben WLAN oder Hotspot, und beide Geräte zeigen dieselben sechs Ziffern, die ihr vergleicht und bestätigt. Verwaltet wird das Projekt von den Geräten des Eigentümers; gibt es davon nur eines, warnt die App – paare dann vorher ein zweites eigenes Gerät.',
          p2: 'Mitglieder geben eigene Notiz-Mappen, Ordner und Alben unter „Etwas freigeben" ins Projekt, zunächst nur zum Lesen. Notiz-Mappen lassen sich auch zum Bearbeiten freigeben: Dann bearbeitet immer nur eine Person eine Notiz, und gespeichert wird auf dem Gerät der Person, der die Mappe gehört – ist es nicht in der Nähe, bleibt die Notiz nur lesbar. Der Chat erreicht sofort alle, die gerade in der Nähe sind, die übrigen beim nächsten Abgleich. Auch die Planung – Abstimmungen und Listen – reist so mit.',
          p3: 'Den Rest gleichst du mit „Abgleichen" ab, sobald ein Mitglied in der Nähe ist. Der Eigentümer kann Mitglieder entfernen; wer selbst gehen will, tritt mit „Austreten" aus, und das geht nur, wenn ein anderes Mitglied in der Nähe ist, das davon erfährt. Was Ausgeschiedene geteilt hatten, verschwindet bei den anderen; was sie selbst schon hatten, bleibt auf ihren Geräten.',
        },
      },
    },
    messages: {
      title: 'Nachrichten',
      subs: {
        direkt: {
          title: 'Direktnachrichten',
          p1: 'Die Nachrichten öffnest du über das Menü. Mit „Nachricht" oben rechts schreibst du jedem anderen Nutzer direkt – als Empfänger genügt sein Openany-Name. Links in Nachrichten sind anklickbar.',
          p2: 'Unter jeder empfangenen Nachricht öffnet „Antworten" das Schreibfeld mit dem Absender schon eingetragen, auf demselben Weg, auf dem die Nachricht kam. Lange Nachrichten sind eingeklappt; „Weiterlesen" zeigt sie ganz. Der Ungelesen-Zähler am Menü-Knopf zeigt, ob etwas Neues wartet.',
          p3: 'Direktnachrichten sind Ende-zu-Ende verschlüsselt: openany.de speichert nur Unlesbares, lesen können sie nur deine Geräte. Alle deine Geräte teilen dafür einen Schlüssel. Ein neues Gerät zeigt 6 Ziffern; du tippst sie auf einem schon eingerichteten ein, und es ist angemeldet und hat den Schlüssel. Der Schlüssel steckt auch in deiner Sicherungsdatei. Geht ein Gerät verloren, ziehst du es in den Einstellungen unter „Meine Geräte" zurück; dabei wechselt der Schlüssel. Schreiben kannst du jedem, der die App mindestens einmal geöffnet hat.',
        },
        hinweise: {
          title: 'Hinweise',
          p1: 'In der Webapp stehen unter Nachrichten nur Hinweise: Benachrichtigungen aus deinen Projekten, die Meldung, dass dich jemand zu einem Projekt hinzugefügt hat, Antworten auf das Kontaktformular und Meldungen zu deinen Geräten. Die Suche über der Liste findet Hinweise nach Text und Absender.',
          p2: 'Direktnachrichten – auch über Matrix, E-Mail und vor Ort – gibt es nur in der App. Sie sind Ende-zu-Ende verschlüsselt, und die Schlüssel liegen auf deinen Geräten, nicht auf openany.de. Deshalb kann die Webapp sie weder zeigen noch schreiben.',
        },
        suche: {
          title: 'Suchen und filtern',
          p1: 'Über dem Verlauf steht ein Suchfeld. Es findet Nachrichten nach ihrem Text, nach dem Namen des Gegenübers und nach Matrix-Kennungen und hebt die Fundstellen hervor. Darunter schränken Schalter die Liste ein: auf ungelesene Nachrichten, auf solche mit Anhang und – wenn du mehr als einen Weg nutzt – auf einzelne Wege wie Openany oder Matrix. Die Schalter lassen sich kombinieren.',
          p2: 'Neben dem Suchfeld wechselst du zwischen zwei Ansichten: „Verlauf" zeigt alle Nachrichten der Zeit nach, „Nach Kontakt" fasst sie zu einer Zeile je Gegenüber zusammen, mit der letzten Nachricht und der Zahl der ungelesenen. Ein Klick auf eine Zeile zeigt den Verlauf mit genau dieser Person; „Unterhaltung schließen" führt zurück zu allen.',
        },
        anhaenge: {
          title: 'Anhänge',
          p1: 'Über Matrix schickst du auch Dateien: Die Büroklammer im Schreibfeld hängt eine Datei bis 10 MB an, auf Handy und Tablet nimmt der Kamera-Knopf daneben gleich ein Foto auf. Im Verlauf erscheinen Bilder als Vorschau, andere Dateien mit Name und Größe; ein Klick öffnet Bilder und PDFs direkt in Openany, alles andere lädst du herunter. Über den Weg Openany gehen Dateien bis 15 MB, Ende-zu-Ende verschlüsselt wie der Text. Sie zählen zu deinem Speicher und zu dem des Empfängers, bis jeder die Nachricht bei sich löscht.',
        },
        matrixApp: {
          title: 'Nachrichten über Matrix',
          p1: 'In der App ist dein Gerät selbst das Matrix-Gerät. Du meldest dich in den Einstellungen unter „Matrix-Konten" mit einem bestehenden Konto an, und deine Nachrichten sind Ende-zu-Ende verschlüsselt zwischen diesem Gerät und dem Gegenüber – openany.de liest nicht mit. Die Geräte-Bestätigung per Emoji-Vergleich kann die App noch nicht; in anderen Matrix-Programmen steht sie deshalb als nicht bestätigtes Gerät, verschlüsselt ist trotzdem.',
          p2: 'Du kannst mehrere Matrix-Konten verbinden. Ihre Nachrichten stehen gemeinsam im Verlauf; neue gehen vom Standard-Konto, Antworten von dem Konto, bei dem die Nachricht ankam, und „Als Standard" macht ein anderes Konto zum Standard. Löschst du eine Matrix-Nachricht, verschwindet sie nur auf diesem Gerät; im Raum bleibt sie stehen.',
        },
        email: {
          title: 'E-Mail über dein Postfach',
          p1: 'In der App kommt E-Mail als weiterer Weg dazu – kein eigenes Mailprogramm, sondern dein vorhandenes Postfach, etwa bei Posteo, mailbox.org oder GMX. In den Einstellungen unter „E-Mail-Postfächer" trägst du Adresse und Passwort ein; meist ist das ein App-Passwort, das du vorher beim Anbieter anlegst. Die Server sucht die App selbst heraus; findet sie keine, trägst du sie unter „Server von Hand" ein. Das Passwort bleibt verschlossen auf diesem Gerät, und das Gerät spricht selbst mit dem Mailserver – openany.de sieht keine Mail.',
          p2: 'Die Mails stehen mit Betreff im gemeinsamen Verlauf unter Nachrichten; „Jetzt abholen" holt neue sofort. Verbindest du mehrere Postfächer, gehen neue Mails vom Standard-Postfach und Antworten von dem, an das die Mail kam; beim Schreiben wählst du unter „Von". Beim Löschen entscheidest du, ob eine Mail nur hier verschwindet oder auch auf dem Mailserver in den Papierkorb-Ordner wandert. Hat dein Anbieter etwas als Spam einsortiert, erscheint über dem Verlauf ein Hinweis; dort holst du eine Mail mit „Kein Spam" zurück in den Posteingang.',
          p3: 'Per E-Mail gehen Anhänge bis 15 MB. Empfangene Anhänge legst du mit „Speichern" in deine Dateien oder mit „Aufs Gerät" in den Download-Ordner. Kam ein Anhang verschlüsselt, fragt die App vorher nach: In Dateien liegt er unverschlüsselt und wird mit openany.de abgeglichen.',
        },
        pgp: {
          title: 'Verschlüsselte E-Mail mit OpenPGP',
          p1: 'Mails kannst du mit OpenPGP Ende-zu-Ende verschlüsseln. Dafür braucht jedes Postfach einen eigenen Schlüssel: In den Einstellungen am Postfach erzeugst du ihn mit „Schlüssel erzeugen" – oder, wenn du das Postfach schon mit PGP nutzt, etwa in Thunderbird, liest du den vorhandenen mit „Schlüssel einlesen" ein, damit beide Programme dieselben Mails lesen. Der geheime Schlüssel liegt nur auf diesem Gerät. „Sicherungskopie speichern" legt ihn, mit einer Passphrase verschlossen, im Download-Ordner ab; „Öffentlichen Schlüssel speichern" gibt dir die Datei, die du anderen schickst.',
          p2: 'Die öffentlichen Schlüssel deiner Kontakte sammelt die App unter „Schlüssel der Kontakte" von selbst aus Mails, die sie mitschicken. Fehlt einer, suchst du ihn nach der Adresse – auf Wunsch auch bei keys.openpgp.org, das dann erfährt, nach wem du fragst – oder liest ihn aus einer Datei ein. Vergleiche den Fingerabdruck am besten einmal mit dem Gegenüber; ändert sich ein Schlüssel, weist die App darauf hin.',
          p3: 'Beim Schreiben schaltest du „Verschlüsselt senden" ein; darunter steht, ob der Schlüssel des Empfängers bekannt ist. Antwortest du auf eine verschlüsselte Mail, ist der Schalter schon an. Im Verlauf tragen Mails die Marke „verschlüsselt" und, wenn sie unterschrieben sind, „signiert" – oder eine Warnung, wenn die Unterschrift nicht stimmt.',
        },
        vorOrt: {
          title: 'Nachrichten vor Ort',
          p1: 'Mit dem Weg „Vor Ort" schreibst du Geräten in der Nähe direkt, ganz ohne Server. Auf dem anderen Gerät muss Openany geöffnet sein, und beide müssen im selben WLAN sein. Schreiben könnt ihr einander, sobald ihr euch kennt: eigene Geräte, Mitglieder eines gemeinsamen Projekts – oder Personen, die ihr vor Ort bestätigt habt. Dafür wählst du „Vor Ort bestätigen", beide Geräte zeigen dieselben 6 Ziffern, und beide tippen „Passt". Ist das andere Gerät gerade nicht da, wartet eine Nachricht und geht beim nächsten Treffen hinüber.',
          p2: 'Von Unbekannten kommt zunächst nichts an. Schaltest du in den Einstellungen unter „Geräte in der Nähe" „Anfragen von Geräten in der Nähe erlauben" ein, können sie dir bis zu 3 Nachrichten mit je 500 Zeichen schicken; diese stehen unter „Anfragen" über dem Verlauf, nicht im Verlauf selbst. Antworten kannst du erst nach der Bestätigung per 6 Ziffern. Wer dir nichts mehr schicken soll, den sperrst du mit „Blockieren". Wurde eine eigene Nachricht abgelehnt, steht an ihr „nicht zugestellt", und der Grund erscheint, wenn du darauf zeigst.',
        },
        systemnachrichten: {
          title: 'Benachrichtigungen und Antworten',
          p1: 'Ereignisse aus deinen Projekten – etwa neue Umfragen oder Freigaben – erreichen dich als Hinweis, sofern der Absender die Mitglieder informieren lässt; ebenso die Meldung, dass dich jemand zu einem Projekt hinzugefügt hat. Hinweise sind nicht verschlüsselt und stehen in App und Webapp. Auch die Antwort auf eine Nachricht über das Kontaktformular findest du hier.',
        },
      },
    },
    contacts: {
      title: 'Adressbuch',
      subs: {
        adressbuch: {
          title: 'Kontakte und ihre Wege',
          p1: 'Das Adressbuch öffnest du oben auf der Nachrichten-Seite; der Knopf daneben legt direkt einen neuen Kontakt an. Ein Kontakt hält fest, wie jemand erreichbar ist: Name, Bild, Telefon und E-Mail sowie beliebig viele weitere Wege – Matrix, Meshtastic, Openany-Name, Anschrift, Betrieb, Geburtstag oder Sonstiges, jeweils mit eigener Beschriftung wie privat oder Arbeit. Das Suchfeld findet Kontakte nach Namen.',
          p2: 'Die Wege sind anklickbar: Eine Telefonnummer ruft an, eine E-Mail-Adresse öffnet das Mailprogramm, ein Openany-Name oder eine Matrix-Kennung öffnet das Schreibfeld mit dem Empfänger schon eingetragen. Gelöschte Kontakte landen im Papierkorb.',
        },
        verschluesselt: {
          title: 'Ende-zu-Ende verschlüsselt',
          p1: 'Dein Adressbuch ist Ende-zu-Ende verschlüsselt, mit demselben Schlüssel wie deine Direktnachrichten. Lesen können es nur deine freigegebenen Geräte; openany.de bewahrt es auf und gleicht es ab, sieht aber weder Namen noch Nummern oder Bilder. Ein neues Gerät zeigt das Adressbuch erst, wenn du es auf einem anderen Gerät freigegeben hast. Die Webapp hat deshalb kein Adressbuch mehr.',
        },
        vcard: {
          title: 'Als vCard sichern und einlesen',
          p1: 'Unter Einstellungen → Adressbuch speicherst du alle Kontakte als vCard-Datei oder liest eine aus einem anderen Programm ein. Einlesen legt nur dazu: Wer schon da ist, wird über seine Kennung oder seinen Namen wiedererkannt und behält, was er hat. Die Datei selbst ist nicht verschlüsselt – bewahre sie entsprechend auf.',
        },
        sicherheitsnummer: {
          title: 'Sicherheitsnummer',
          p1: 'Mit der Sicherheitsnummer prüfst du, dass niemand dazwischen mitliest – auch nicht openany.de. Öffne sie im Kontakt unter „Sicherheit“ oder in den Nachrichten, wenn du eine Unterhaltung aus „Nach Kontakt“ geöffnet hast. Vergleicht die zwölf Zahlengruppen auf euren beiden Geräten; stimmen sie überein, tippe „Stimmt überein“. Nachrichten dieses Menschen tragen dann ein Häkchen am Schloss. Ändert sich sein Schlüssel später, etwa nach einem verlorenen Gerät, verschwindet das Häkchen, und ein Hinweis bittet euch, neu zu vergleichen.',
        },
      },
    },
    settings: {
      title: 'Einstellungen und Konto',
      subs: {
        profil: {
          title: 'Profil, Zeitzone und Speicherplatz',
          p1: 'Dein Openany-Name steht fest und lässt sich nicht ändern. Die E-Mail-Adresse ist freiwillig und dient nur Konto-Zwecken wie dem Passwort-Zurücksetzen, niemals Werbung; ändern kannst du sie nur mit deinem Passwort. Die Zeitzone legt fest, wie die Uhrzeiten deiner Termine gemeint sind – ein Knopf übernimmt die Zone des Geräts. Darunter zeigt „Speicherplatz", wie viel du belegst.',
        },
        module: {
          title: 'Module auf der Startseite',
          p1: 'Hier wählst du, welche Module als Kachel auf der Startseite erscheinen – Notizen, Kalender, Speicher, Projekte, Nachrichten und Adressbuch. Die Auswahl blendet nichts aus; alle Module bleiben erreichbar. Ohne Auswahl zeigt die Startseite die Willkommens-Übersicht.',
        },
        sicherheit: {
          title: 'Sicherheit: Passwort und zweiter Faktor',
          p1: 'Passwort, zweiter Faktor und deine Geräte liegen bei anyid, nicht bei Openany. Die Karte „Konto & Sicherheit" in den Einstellungen führt mit drei Links direkt dorthin: „Passwort ändern", „Zweiter Faktor" und „Deine Geräte", wo du gekoppelte Geräte ansiehst und wieder hinauswirfst. Änderungen gelten für Openany, anyitem und anytail zugleich.',
          p2: 'Der zweite Faktor ist ein Code aus einer Authenticator-App, der nach dem Passwort abgefragt wird; ohne ihn lässt sich kein App-Passwort anlegen. Beim Einrichten bekommst du Wiederherstellungscodes; hebe sie auf – ohne sie und ohne die App kommst du nicht mehr herein. Hast du eine E-Mail-Adresse hinterlegt, kannst du ein vergessenes Passwort selbst zurücksetzen.',
        },
        spracheDesign: {
          title: 'Aussehen und Sprache',
          p1: 'Unter „Aussehen" wählst du zwischen zwei Designs: „Klar" in Petrol mit kantigeren Ecken und ruhiger Fläche oder „Flieder" mit runden Ecken und gemustertem Hintergrund. Beide gibt es hell und dunkel – das stellst du unabhängig davon über den Umschalter in der Kopfzeile oder im Handy-Menü um. Unter „Sprache" stellst du ein, in welcher Sprache Openany mit dir spricht: Deutsch, Englisch, Spanisch, Französisch, Portugiesisch, Polnisch oder Ukrainisch. Bis du selbst wählst, richtet es sich nach der Sprache deines Geräts oder Browsers; gibt es die hier nicht, spricht Openany Englisch.',
        },
        zugriff: {
          title: 'Zugriff von außen: App-Passwörter',
          p1: 'Programme auf deinem Rechner melden sich nicht mit deinem Kontopasswort an, sondern mit einem eigenen App-Passwort. Beim Anlegen gibst du ihm einen Namen und wählst den Zweck „Abgleich“ – für Programme, die dein Konto spiegeln und zurückschreiben. Notizen gibt es auf dem Server nur noch verschlüsselt, deshalb gibt es keinen Zugang per WebDAV mehr. Anlegen geht nur, wenn du dich mit zweitem Faktor angemeldet hast.',
          p2: 'Das neue Passwort wird genau einmal angezeigt – kopiere es sofort. Im Programm trägst du deinen Openany-Namen als Benutzernamen ein. Die Liste zeigt, wann jedes Passwort zuletzt benutzt wurde; widerrufst du eines, verliert das Programm sofort den Zugang.',
        },
        abgleich: {
          title: 'Abgleich mit Openany',
          p1: 'Unter „Abgleich" verbindest du die App mit „Mit openany.de verbinden" mit deinem Konto. Die App zeigt einen Code, den du auf der Seite von anyid im Browser eintippst und bestätigst. Diese Bestätigung im Browser ist kein Umweg, sondern der Nachweis, dass wirklich du das Gerät hinzufügst. Verbinden ist freiwillig: Ohne Verbindung bleibt alles auf diesem Gerät, und „Verbindung trennen" macht es jederzeit rückgängig, ohne dass hier Daten verloren gehen.',
          p2: 'Danach gleicht „Abgleichen" in beide Richtungen ab und sagt hinterher, was geholt, geschoben oder übersprungen wurde. „Alles neu holen" lädt den ganzen Bestand noch einmal statt nur des Neuen; gelöscht wird dabei nichts.',
          p3: 'Mit „Im Hintergrund auffrischen" gleicht die App etwa stündlich von selbst ab – nur mit Netz und nicht bei leerem Akku. Im Mobilfunk kommt dabei nur die Liste, Dateien und Bilder erst im WLAN. Geräte in der Nähe gehören nicht dazu; für sie muss die App offen sein. Auf dem Computer gleicht Openany stündlich ab, solange es läuft.',
        },
        geraeteNah: {
          title: 'Geräte in der Nähe',
          p1: 'Deine eigenen Geräte gleichen sich auch direkt ab, ganz ohne Server – solange beide im selben WLAN oder Hotspot sind und auf beiden Openany offen ist. Unter „Geräte in der Nähe" findest du sie mit „Suchen" und paarst sie mit „Paaren": Beide Geräte zeigen denselben Code, und auf beiden bestätigst du, dass er passt. Danach gleichen sie sich bei jedem „Abgleichen" untereinander ab; „Vergessen" löst die Paarung wieder.',
          p2: 'Unter „Dein Name" legst du fest, wie du anderen vor Ort erscheinst. Unter diesem Namen lädt man dich in Projekte ein – dich als Person, nicht ein einzelnes Gerät; er gilt für alle deine gepaarten Geräte.',
        },
        speicherGeraet: {
          title: 'Speicher auf diesem Gerät',
          p1: 'Unter „Speicher auf diesem Gerät" entscheidest du, wie viel die App von deinen Dateien und Bildern vorhält. „Bei Bedarf" zeigt alle Dateien in der Liste, holt den Inhalt aber erst beim Öffnen. Bei „Ausgewählte Ordner und Alben" bleibt das immer auf dem Gerät, was du im Speicher mit „Auf diesem Gerät behalten" markiert hast, samt Unterordnern. „Alles behalten" holt nach jedem Abgleich, was fehlt, solange Platz ist.',
        },
        sofort: {
          title: 'Sofort benachrichtigen',
          p1: 'Mit „Sofort benachrichtigen" hält die App sparsame Verbindungen zu Openany und zu deinen E-Mail-Postfächern, damit neue Nachrichten und Mails sofort ankommen – ganz ohne Google. Für die Postfächer braucht es kein Openany. Android zeigt dafür einen dauerhaften Hinweis an, den du in den Systemeinstellungen ausblenden kannst. Auf dem Computer wacht Openany, solange es läuft, und meldet sich mit einer Mitteilung des Systems.',
        },
        sicherung: {
          title: 'Sicherung in einer Datei',
          p1: 'Unter „Sicherung" legst du mit „Sicherung anlegen" eine verschlüsselte Datei mit allem an, was Openany auf diesem Gerät hat: Notizen, Kalender, Kontakte, Dateien, Galerie, Projekte, Nachrichten und deine Postfächer samt Passwort und eigenem OpenPGP-Schlüssel. Die App schlägt eine Passphrase aus sechs Wörtern vor; schreib sie auf und bewahre sie getrennt von der Datei auf. Ohne sie lässt sich die Datei nicht öffnen – auch nicht von uns.',
          p2: 'Mit „Sicherung einspielen" öffnest du eine solche Datei auf einem anderen Gerät. Sie ersetzt alles, was dort war; danach ist die Verbindung zu openany.de getrennt, und Geräte in der Nähe paarst du neu. Was ein Gerät ausweist, kommt nicht mit.',
        },
        tresor: {
          title: 'Ausweise und Schlüssel im Tresor',
          p1: 'Womit sich dieses Gerät bei Openany und anderen Geräten ausweist, liegt verschlossen auf dem Gerät; der Schlüssel dazu bleibt im Schlüsselbund des Systems – unter Android im Keystore, auf dem Computer in der Schlüsselbundverwaltung von macOS, den Windows-Anmeldeinformationen oder dem Secret Service unter Linux. Dort liegen auch die Passwörter deiner Postfächer und deine geheimen OpenPGP-Schlüssel. „Verbindung trennen" entfernt die Ausweise nur hier – endgültig widerrufen wird in der Geräteliste bei anyid und unter den App-Passwörtern von Openany.',
        },
      },
    },
    contact: {
      title: 'Hilfe und Kontakt',
      subs: {
        kontakt: {
          title: 'Kontakt zu den Betreibern',
          p1: 'Fragen, Probleme oder Wünsche? Über den Link „Kontakt" in der Fußzeile erreichst du das Kontaktformular. Deine Nachricht geht als Hinweis an die Admins, und die Antwort findest du später unter Nachrichten.',
        },
        bedingungen: {
          title: 'Nutzungsbedingungen',
          p1: 'Die Nutzungsbedingungen der Beta und das Impressum findest du jederzeit über die Links in der Fußzeile. Kurz gefasst: Openany ist während der Beta kostenlos und werbefrei, und deine Daten liegen auf Servern in Deutschland. Hinaus geht nur, was du selbst hinausschickst – etwa Nachrichten über Matrix oder E-Mail.',
        },
      },
    },
  },
};
