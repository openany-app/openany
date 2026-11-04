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
          p1: 'Konten bei Openany entstehen ausschließlich auf persönliche Einladung – es gibt keine offene Registrierung. Openany kennt keine E-Mail-Adresse von dir. Deshalb gibt es auch kein Zurücksetzen per E-Mail: Bewahre dein Passwort gut auf.',
          p2: 'Angemeldet wird mit Nutzername und Passwort über anyid, den gemeinsamen Anmeldedienst. Dieselbe Anmeldung gilt für Openany, anyitem und anytail; Passwort und zweiter Faktor werden deshalb dort verwaltet und nicht in Openany selbst (siehe „Sicherheit" unter den Einstellungen).',
        },
        ohneKonto: {
          title: 'Die App ohne Konto',
          p1: 'Die App ist ein vollständiges Programm, das auch ganz ohne Konto und ohne Server auskommt: Notizen, Kalender, Speicher, Adressbuch und Nachrichten liegen dann nur auf diesem Gerät. Was hier nicht eingerichtet ist oder nicht geht, blendet die App aus, statt es leer anzuzeigen – der Weg Openany bei den Nachrichten etwa erscheint erst, wenn das Gerät mit openany.de verbunden ist.',
          p2: 'Willst du deine Daten mit Openany im Netz abgleichen, tippst du in den Einstellungen unter „Abgleich" auf „Mit openany.de verbinden"; mit deinen anderen Geräten paarst du dich unter „Geräte in der Nähe". Danach steht in der Kopfzeile und im Menü ein Knopf „Abgleichen", der sich dreht, solange abgeglichen wird.',
        },
        startseite: {
          title: 'Die Startseite',
          p1: 'Die Startseite zeigt kompakte Kacheln der Module, die du dafür ausgewählt hast: die Termine von heute, das Adressbuch mit Suche, die drei zuletzt bearbeiteten Notizen, die drei letzten Dateien samt belegtem Speicher und deine Hubs. Jede Kachel führt in ihr Modul; welche erscheinen, legst du in den Einstellungen unter „Module auf der Startseite" fest.',
          p2: 'Zu Beginn sind alle Kacheln gewählt. Wählst du alle ab, erscheint stattdessen die Willkommens-Übersicht: Sie stellt alle Funktionen vor und lässt dich die Kacheln direkt zusammenstellen. Die Auswahl blendet nichts aus – alle Bereiche bleiben über die Leiste und das Menü erreichbar.',
        },
        navigation: {
          title: 'Menü und Aufbau',
          p1: 'Am Computer liegt oben eine Kopfzeile: Links führt das runde Logo zur Startseite, daneben stehen Notizen, Kalender, Speicher und Hubs. Rechts folgen der Hell-/Dunkel-Umschalter, der Briefumschlag zu den Nachrichten mit Ungelesen-Zähler und das Feld mit deinem Namen – es öffnet Einstellungen, Hilfe, Papierkorb und Abmelden.',
          p2: 'Auf dem Handy rückt alles nach unten: Eine feste Leiste zeigt die Module als Symbole, ganz links davon sitzt der Menü-Knopf mit dem Zähler für ungelesene Nachrichten. Er öffnet ein Feld mit Nachrichten, Einstellungen, Hilfe, Papierkorb, dem Hell-/Dunkel-Umschalter und dem Abmelden. Der Umschalter wechselt zwischen hell, dunkel und automatisch – automatisch folgt der Einstellung deines Geräts. Ein Tipp auf das gerade geöffnete Modul führt zu seinem Startpunkt.',
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
          p1: 'Notizen liegen in Mappen, die sich beliebig tief verschachteln lassen. Notizen und Mappen kannst du umbenennen, verschieben und neu anordnen – so wächst mit der Zeit eine eigene Wissenssammlung. „Finden“ oben (Lupe) öffnet ein Suchfeld, das alle Notizen auf einmal durchsucht, Titel und Inhalt; öffnest du sie in einer Mappe, lässt sich die Suche mit „nur in dieser Mappe“ darauf beschränken. Was sich mit der geöffneten Mappe tun lässt, steht rechts in der Pfadleiste im Menü „…“.',
          p2: 'Nach oben kommst du über den Pfad oder mit der Zurück-Taste, die eine Ebene höher geht; eine offene Notiz schließt das Kreuz in der Pfadleiste. Neben jeder Mappe steht, wann sich zuletzt eine Notiz darin geändert hat, auch in ihren Untermappen. Eine Mappe ohne Notizen zeigt kein Datum.',
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
          p1: 'Über den Büroklammer-Knopf in der Werkzeugleiste fügst du eine Datei von deinem Gerät in eine Notiz ein. Bilder erscheinen als platzsparende Vorschau direkt in der Notiz (die volle Auflösung liegt in der Mediathek), andere Dateien als Link zum Öffnen.',
          p2: 'Eingefügte Inhalte werden automatisch einsortiert: Bilder in ein Mediathek-Album, Text- und Office-Dateien in eine Dokumenten-Akte, alles Übrige in einen Datei-Ordner – jeweils in einem eigenen Bereich für Notizen.',
        },
        zitate: {
          title: 'Zitate und Literatur',
          p1: 'Für wissenschaftliches Arbeiten führt jede Mappe eine Quellenliste: In der geöffneten Mappe öffnest du sie im Menü „…“ unter „Quellen“ und trägst Bücher, Aufsätze, Webseiten, Urteile oder Gesetze ein. Jede Quelle bekommt einen Kurznamen. Die Liste ist eine CSL-JSON-Datei unter Dateien, also auch mit Zotero und Zettlr verwendbar.',
          p2: 'In Notizen dieser Mappe schlägt der Editor beim Zitieren passende Quellen vor und setzt den Beleg in den Text. So entstehen Belege beim Schreiben, ohne die Quellenangaben jedes Mal abzutippen. Wie sie im PDF erscheinen, steht unter „Druckfassung“.',
        },
        graph: {
          title: 'Graph-Ansicht',
          p1: 'Die Graph-Ansicht zeichnet deine Notizen als Netz: Jede Notiz ist ein Punkt, jeder Wikilink eine Verbindung. Die Punkte sind nach oberster Mappe eingefärbt, sodass zusammengehörige Bereiche schon an der Farbe erkennbar sind; hohle Punkte stehen für Notizen ohne jede Verbindung, und je mehr Verknüpfungen eine Notiz hat, desto größer ihr Punkt.',
          p2: 'Jede oberste Mappe ist eine Ebene. Gibt es mehrere, wählst du in der Legende unter dem Graphen eine oder mehrere aus – „Alle Ebenen" nimmt alle, „Keine" leert die Auswahl. Statt einer Ebene kann auch ein Tag der Einstieg sein: Die Leiste darunter nennt die Tags mit ihrer Anzahl, und ein gewählter Tag zeigt alle Notizen, die ihn tragen, samt dem Tag als grauer Raute. Sind Ebenen gewählt, stehen in der Leiste nur noch die Tags, die darin vorkommen. Bis du etwas wählst, bleibt der Graph leer, damit große Sammlungen nicht auf einmal aufgebaut werden; „Alle Tags anzeigen" holt alle Tags der Leiste auf einmal hinein.',
          p3: 'Am Computer zoomst du mit den Plus-/Minus-Knöpfen oben rechts oder dem Mausrad und verschiebst die Ansicht mit gedrückter Maustaste; auf dem Handy zoomst du mit zwei Fingern und verschiebst mit einem. Ein weiterer Knopf setzt die Ansicht zurück. Ein Klick auf einen Punkt (ohne Ziehen) öffnet die Notiz, ein Klick auf einen Tag filtert die Notizliste danach.',
        },
        export: {
          title: 'PDF-Export',
          p1: 'Der PDF-Knopf in der offenen Notiz bietet vier Fassungen: „Standard“ ist die Notiz mit Titel, Stand und Literaturverzeichnis. „Clean“ ist nur der Text, ohne Titel, Stand, Seitenzahlen und Literaturverzeichnis; Zitate bleiben, wie du sie geschrieben hast. „Brief“ setzt die Notiz nach DIN 5008 für den Fensterumschlag, mit dem Titel als Betreff. „Druckfassung“ gibt nur diese Notiz mit Titelblatt aus. Das PDF landet in der Akte „Notizen“ unter Speicher → Dokumente (frühere Fassungen bleiben daneben erhalten), die Druckfassung in der Akte „Druckfassungen“; „Öffnen“ in der Meldung zeigt es gleich an.',
        },
        druckfassung: {
          title: 'Druckfassung',
          p1: 'Aus einer Mappe wird ein förmliches Dokument zum Abgeben oder Drucken: In der geöffneten Mappe wählst du im Menü „…“ „Druckfassung“, trägst Titel, Untertitel, Name, Einrichtung, Datum und Zusatzangaben fürs Titelblatt ein und entscheidest, ob ein Inhaltsverzeichnis dazukommt. Jede Notiz der Mappe wird ein Kapitel, alphabetisch nach Titel – mit Vorsilben wie „01 Einleitung“ legst du die Reihenfolge fest; im Dokument erscheint dann nur „Einleitung“. Untermappen kommen mit, wenn der Haken „Untermappen einbeziehen“ gesetzt ist: Eine Untermappe wird eine Überschrift, ihre Notizen rücken darunter („2 Grundlagen“, „2.1 Begriffe“); eine Notiz, die heißt wie ihre Untermappe, steht als Text direkt unter deren Überschrift.',
          p2: 'Gesetzt wird in Times-artiger Schrift mit serifenlosen Überschriften, 2,5 cm Rand, eineinhalbzeilig, im Blocksatz mit Silbentrennung; die Seitenzahlen beginnen nach dem Titelblatt. Die Angaben bleiben an der Mappe gespeichert. Das PDF entsteht auf deinem Gerät – beim ersten Mal dauert es einige Sekunden – und landet in der Akte „Druckfassungen“ unter Speicher → Dokumente; jede Fassung bleibt erhalten.',
          p3: "Zitate aus der Quellenliste der Mappe: Im Text schreibst du [{'@'}kurzname] oder [{'@'}kurzname, S. 12], für eine Fußnote ^[Text]. Im PDF – Notiz wie Druckfassung – entstehen daraus Harvard-Zitate wie „(Müller, 2020, S. 12)“ und am Ende das Literaturverzeichnis. Für Fußnoten gibt es in der Werkzeugleiste des Editors einen eigenen Knopf (hochgestellte Zahl): Er setzt die Klammern, markierter Text wird zur Fußnote.",
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
          p1: 'In der App auf dem Computer kannst du eine eigene Mappe mit einem Ordner verbinden: Mappe öffnen, im Menü „…“ „Mit Ordner verbinden …“ wählen und den Ordner aussuchen. Jede Notiz liegt dort als Datei „Id Titel.md“, Untermappen als Unterordner – so, wie Zettlr oder Obsidian es erwarten. Was du dort änderst, umbenennst, verschiebst oder neu anlegst, übernimmt die App, solange sie läuft; haben sich beide Seiten geändert, entsteht eine Konfliktkopie. Löschst du eine Datei, landet die Notiz im Papierkorb. Der Ordner liegt unverschlüsselt auf deinem Computer; „Ordner trennen“ lässt die Dateien liegen. Bilder in Notizen liegen neben der Notiz, unter dem Namen aus dem Text; ein neues Bild dort, das eine Notiz nennt, kommt in die App (Mediathek, Album „Notizen: …“).',
        },
      },
    },
    files: {
      title: 'Speicher',
      subs: {
        aufbau: {
          title: 'Mediathek, Dateien, Dokumente',
          p1: 'Speicher teilt sich in drei Reiter, die oben in der Kopfzeile stehen: die Mediathek für Fotos, Videos und Ton, Dateien für alles Mögliche und Dokumente für Schriftstücke in Akten. Wechselst du den Reiter, bleibt der andere Bereich, wie du ihn verlassen hast – geöffnete Ordner und geladene Listen gehen nicht verloren.',
          p2: 'Welcher Reiter zuerst aufgeht, wenn du Speicher öffnest, legst du in den Einstellungen unter „Start unter Speicher" fest; solange du nichts wählst, sind es die Dokumente.',
        },
        dokumente: {
          title: 'Dokumentenablage',
          p1: 'Im Reiter „Dokumente" organisierst du PDF-, Office- und Textdateien in Akten. Hochladen geht per Knopf oder indem du Dateien einfach in die geöffnete Akte ziehst. Auf Handy und Tablet steht daneben ein Kamera-Knopf: Er fotografiert ein Schriftstück und legt es gleich als Dokument in die Akte.',
          p2: 'Akten lassen sich beliebig tief verschachteln. Akten wie einzelne Dokumente kannst du umbenennen und verschieben; ganze Akten außerdem in einem Hub unter „Freigaben“ teilen. Neben jeder Akte steht, wann sich zuletzt etwas darin geändert hat, auch in ihren Unterakten.',
        },
        texterkennung: {
          title: 'Durchsuchbare Dokumente',
          p1: 'Fotografierst du ein Schriftstück ab und legst es in eine Akte – mit dem Kamera-Knopf oder als Foto aus deiner Bildersammlung –, wird daraus automatisch ein PDF mit einer unsichtbaren Textebene. Das Dokument sieht aus wie dein Foto, lässt sich aber durchsuchen, markieren, kopieren und vorlesen. Die Erkennung läuft auf deinem eigenen Gerät – das Bild wird dafür nirgendwo hingeschickt.',
          p2: 'Legst du mehrere Fotos auf einmal ab, wirst du gefragt, ob daraus ein mehrseitiges Dokument werden soll – etwa die Seiten eines Briefes – oder einzelne. Das Foto selbst wird nicht zusätzlich gespeichert; wer das Bild behalten möchte, legt es in der Mediathek ab. Beim ersten Mal dauert die Umwandlung länger, weil die Texterkennung einmalig geladen und danach gespeichert wird.',
          p3: 'Liegt schon ein eingescanntes PDF in einer Akte, kannst du die Erkennung nachträglich darauf anwenden: „Text erkennen" liest das Dokument Seite für Seite. Ist das Papier sauber weiß, bleiben die Seiten unverändert und es kommt nur der Text hinzu. Sind sie grau oder ungleichmäßig ausgeleuchtet – wie bei einem abfotografierten Blatt –, werden sie zusätzlich aufgehellt; die Meldung am Ende sagt dir, was passiert ist. War das PDF bereits durchsuchbar, bleibt es unangetastet.',
        },
        dateien: {
          title: 'Dateien verwalten',
          p1: 'Im Reiter „Dateien" legst du Ordner an und lädst Dateien jeder Art hoch – einzeln oder mehrere auf einmal. Ein Tipp auf eine Datei öffnet sie mit einem passenden Programm deines Geräts. Das Symbol zum Herunterladen legt eine Kopie in den Download-Ordner, am Computer an einen Ort deiner Wahl; bei PDFs steht es im Betrachter. Ordner wie Dateien lassen sich umbenennen und verschieben; Ordner kannst du außerdem in einem Hub unter „Freigaben“ teilen. Was sich mit einem geöffneten Ordner oder einer Akte tun lässt, steht rechts in der Pfadleiste im Menü „…“. Nach oben kommst du über den Pfad oder mit der Zurück-Taste, die eine Ebene höher geht.',
          p2: 'Wie viel du belegst, siehst du auf der Startseiten-Kachel. Wie viel davon auf diesem Gerät liegt, legst du in den Einstellungen unter „Speicher auf diesem Gerät“ fest.',
        },
        suche: {
          title: 'Suchen',
          p1: 'Die Lupe in Dateien und Dokumenten öffnet ein Suchfeld, das alle Ordner beziehungsweise Akten auf einmal durchsucht, nicht nur den gerade geöffneten. Ein Klick auf einen Treffer öffnet die Datei; daneben steht, wo sie liegt, und ein Klick auf diese Angabe bringt dich in den Ordner.',
          p2: 'Gefunden wird nicht nur nach dem Namen, sondern auch im Text: in PDFs, Word- und OpenDocument-Texten sowie Text-, CSV- und Markdown-Dateien bis 50 MB. Openany liest diesen Text nach und nach im Hintergrund, solange Speicher geöffnet ist; bis alles gelesen ist, sagt dir die Suche, wie viele Dateien vorerst nur nach Namen gefunden werden. Bei einem Treffer im Text zeigt die Liste die Fundstelle mit dem gesuchten Wort hervorgehoben. Eingescannte PDFs ohne Textebene findet die Suche erst, nachdem du „Text erkennen" darauf angewendet hast.',
        },
        pdf: {
          title: 'PDFs ansehen und bearbeiten',
          p1: 'Ein Klick auf ein PDF öffnet es direkt in Openany. Du blätterst durch die Seiten, vergrößerst oder verkleinerst, passt die Ansicht an die Breite an und durchsuchst das Dokument mit der Lupe – die Treffer werden auf der Seite hervorgehoben. „Mit anderem Programm öffnen“ gibt es an ein anderes Programm weiter. Ist ein PDF mit Passwort geschützt, fragt Openany zuerst danach.',
          p2: 'Formulare im PDF füllst du direkt aus. Über „Bearbeiten" erscheint außerdem eine Werkzeugleiste: „Markieren" hebt Text farbig hervor, mit dem „Stift" zeichnest du frei auf die Seite, und mit „Text" setzt du eigene Zeilen aufs Blatt – Farbe, Strichstärke und Schriftgröße wählst du selbst, und jeder Schritt lässt sich rückgängig machen. Rechenfelder in Formularen rechnen allerdings nicht mit, weil Openany keine Skripte aus fremden Dateien ausführt.',
          p3: 'An Markierungen und Striche hängst du Kommentare: Wähle „Kommentare" und tippe die Stelle an; dieselbe Ansicht listet alle Kommentare des Dokuments auf. Gespeichert wird alles als gewöhnliche PDF-Anmerkung, die auch andere Programme anzeigen. Beim ersten Speichern legt Openany die Fassung davor in den Papierkorb, sodass du zum Original zurückkannst; verlässt du das PDF mit ungespeicherten Änderungen, fragt es vorher nach.',
        },
        galerie: {
          title: 'Mediathek und Alben',
          p1: 'Im Reiter „Mediathek" liegen deine Fotos, Videos und Tondateien – in Alben mit Vorschau-Kacheln oder, ohne Album, darunter unter „Medien". Alben lassen sich verschachteln, etwa ein Album je Reise mit einem Unteralbum je Tag, und jederzeit verschieben. Die Lightbox blättert durch alle Bilder eines Albums. Ein Foto verschiebst du in der großen Ansicht; hältst du es im Raster gedrückt, wählst du mehrere aus und verschiebst oder löschst sie zusammen. Was sich mit dem geöffneten Album tun lässt, steht rechts in der Pfadleiste im Menü „…“.',
          p2: 'Die neueste Aufnahme steht oben, gegliedert nach Monaten mit einer Überschrift je Monat. Maßgeblich ist das Aufnahmedatum aus den Fotodaten; fehlt es – etwa bei Bildschirmfotos oder bearbeiteten Bildern –, zählt der Tag des Hochladens.',
          p3: 'In der großen Ansicht lädst du ein Foto oder Video als Originaldatei herunter: auf dem Handy in den Download-Ordner, am Computer an einen Ort deiner Wahl. Alben lassen sich in einem Hub unter „Freigaben“ teilen. Die Originale bleiben dabei bei dir. Über „Aktionen“ im Album lädst du ein eigenes Titelbild hoch; es zählt nicht als Eintrag.',
        },
        videos: {
          title: 'Videos und Kamera',
          p1: 'Neben Fotos nimmt die Mediathek auch Videos auf. Sie erscheinen als Kachel mit einem Standbild; ein Tipp öffnet sie mit dem Video-Programm deines Geräts.',
          p2: 'Auf Handy und Tablet stehen in der Mediathek und in jedem Album zwei Kamera-Knöpfe: einer für ein Foto, einer für ein Video. Die Aufnahme landet ohne Umweg an der Stelle, die gerade geöffnet ist. Am Computer gibt es die Knöpfe nicht, dort lädst du wie gewohnt hoch.',
        },
        ton: {
          title: 'Tonaufnahmen und Musik',
          p1: 'Die Mediathek nimmt auch Tondateien auf, etwa MP3 oder M4A. Sie erscheinen als Kachel mit Name und Dauer; ein Tipp spielt sie direkt in der App ab. Tondateien, die von außen kommen, sortiert die App ebenfalls hier ein.',
          p2: 'Über den Mikrofon-Knopf nimmst du direkt auf, mit Pause, Verwerfen und Speichern. Die Aufnahme landet im geöffneten Album, sonst im Album „Aufnahmen". Auf Android läuft sie auch bei ausgeschaltetem Bildschirm weiter, die Benachrichtigung zeigt das an. Auf Linux gibt es den Knopf nicht.',
          p3: 'Auf Android (ab Version 10) und Windows kannst du statt des Mikrofons das Systemaudio wählen. Das System fragt dann jedes Mal nach deiner Einwilligung. Manche Apps sperren das Mitschneiden, Telefonate gehen nie. Gespräche darfst du nur aufnehmen, wenn alle Beteiligten einverstanden sind.',
        },
        karte: {
          title: 'Aufnahmedatum und Ort',
          p1: 'Zu jedem Bild zeigt die Lightbox das Aufnahmedatum und – sofern das Foto ihn enthält – den Aufnahmeort; „Auf Karte zeigen" öffnet die Stelle auf OpenStreetMap. Lädst du Fotos vom Android-Handy hoch, wähle sie über die Dateien-App aus und nicht über die Galerie: Sonst entfernt Android den Standort, bevor das Bild bei Openany ankommt.',
        },
        papierkorbSpeicher: {
          title: 'Papierkorb',
          p1: 'Gelöschte Inhalte – Dokumente, Dateien, Fotos, Notizen, Kontakte, aber auch Hubs, Kalender und Termine – wandern zunächst in den gemeinsamen Papierkorb und lassen sich dort wiederherstellen. Nach 30 Tagen leert sich der Papierkorb automatisch; erst dann ist der Speicherplatz endgültig frei.',
        },
        dateienVerschluesselt: {
          title: 'Ende-zu-Ende verschlüsselt',
          p1: 'Deine Dateien, Akten, Fotos und Alben liegen auf openany.de nur verschlüsselt: Namen, Ordner, Alben, Aufnahmedaten und Ort stehen in einem verschlüsselten Kopf, die Inhalte selbst sind mit einem eigenen Schlüssel je Datei verschlüsselt. Lesen können sie nur deine freigegebenen Geräte.',
        },
      },
    },
    calendar: {
      title: 'Kalender',
      subs: {
        verwalten: {
          title: 'Mehrere Kalender',
          p1: 'Du kannst beliebig viele Kalender nebeneinander führen – etwa privat, Familie, Verein –, jeder mit eigener Farbe. Einzelne Kalender blendest du per Klick ein und aus. Oben wechselst du zwischen Monat, Woche und Tag; Woche und Tag zeigen die Termine nach Uhrzeit, und vergangene Termine erscheinen dezent ausgegraut. Auf Handy und Tablet blätterst du durch Wischen: im Monat nach oben und unten, in Woche und Tag zur Seite. Am Computer gibt es dafür Pfeile.',
        },
        termine: {
          title: 'Termine anlegen',
          p1: 'Termine erstellst du per Klick auf einen Tag: mit Titel, Beschreibung, Uhrzeit oder als ganztägiges beziehungsweise mehrtägiges Ereignis. Auch Wiederholungen sind möglich – täglich, wöchentlich, monatlich oder jährlich. Ein Klick auf einen Termin zeigt die Details, dort bearbeitest oder löschst du ihn.',
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
      title: 'Hubs',
      subs: {
        grundlagen: {
          title: 'Hubs und Mitglieder',
          p1: 'Ein Hub ist ein gemeinsamer Arbeitsbereich: Du lädst andere Nutzer als Mitglieder ein, die Eingeladenen nehmen die Einladung an oder lehnen sie ab. Der Hub-Eigentümer verwaltet die Mitglieder und kann den Hub auch an ein anderes Mitglied übergeben.',
          p2: 'Jeder Hub hat vier Reiter: Freigaben, Board, Mitglieder und Chat. Er öffnet bei den Freigaben. Die Zahlen an den Reitern zeigen ungelesene Chat-Nachrichten beziehungsweise, wie viel darin liegt.',
        },
        chat: {
          title: 'Hub-Chat',
          p1: 'Jeder Hub hat einen gemeinsamen Chat in Echtzeit: Nachrichten erscheinen sofort bei allen Mitgliedern, ohne die Seite neu zu laden. Der Chat ist Ende-zu-Ende verschlüsselt: Lesen können ihn nur die Mitglieder auf ihren Geräten, openany.de nicht. Tritt jemand aus, wechselt der Schlüssel von selbst – Neues liest er dann nicht mehr.',
          p2: 'Wichtige Nachrichten kann jedes Mitglied anheften – angeheftete Nachrichten sind für alle in der Pin-Leiste über dem Verlauf schnell wiederzufinden und lassen sich dort auch wieder lösen.',
          p3: 'Mit zwei eckigen Klammern verweist du mitten im Satz auf Inhalte des Hubs: auf Notizen sowie auf freigegebene Ordner, Dateien, Alben und Bilder. Eine Vorschlagsliste zeigt beim Tippen, was in Frage kommt, und setzt den Verweis fertig ein. Denselben Dienst tut der Knopf mit dem Ketten-Symbol neben dem Senden-Knopf – auf dem Handy meist bequemer, als zwei Klammern zu suchen.',
          p4: 'Ein Klick auf einen Verweis führt direkt zum Ziel: in die Notiz, in den Ordner mit hervorgehobener Datei oder zum Bild in der Lightbox. Verweisen kannst du nur auf das, was im Hub ohnehin sichtbar ist – was niemand freigegeben hat, taucht weder in der Vorschlagsliste auf, noch wird ein Verweis darauf anklickbar.',
        },
        board: {
          title: 'Board',
          p1: 'Jeder Hub hat genau ein Board – eine gemeinsame Zeichenfläche, die mit dem Hub entsteht. Unter „Board“ öffnest du es mit „Board öffnen“. Alle Mitglieder zeichnen, schreiben und ordnen darauf; gespeichert wird von selbst, kurz nach jeder Änderung. Über „Einbinden“ legst du geteilte Bilder, Notizen und Dateien des Hubs als Karte aufs Board.',
          p2: 'Zeichnen mehrere zur selben Zeit, führt die App ihre Änderungen zusammen: Jedes Element nimmt die jüngste Fassung an, Gelöschtes bleibt gelöscht. Sind Mitglieder in der Nähe, zeichnet ihr live zusammen und seht die Zeiger der anderen; sonst kommen die Änderungen beim nächsten Abgleich an. Das Board ist Ende-zu-Ende verschlüsselt.',
        },
        geteilteKalender: {
          title: 'Kalender teilen',
          p1: 'Unter „Freigaben“ teilst du mit „Teilen“ (vor Ort: „Etwas freigeben“) auch einen deiner Kalender mit dem Hub. Er steht dann in der Liste unter „Kalender“; ein Tipp darauf öffnet ihn im Kalender – bei den Mitgliedern unter „Aus Hubs“. Mit „ansehen“ lesen sie nur, mit „bearbeiten“ legen sie Termine an, ändern und löschen sie; den Kalender selbst ändert nur du. Haben zwei denselben Termin zugleich geändert, steht die zweite Fassung als Kopie daneben, mit „Konflikt“, Datum und Uhrzeit im Titel. Alles ist mit einem eigenen Schlüssel verschlüsselt. Über das X neben dem Kalender beendest du das Teilen; dein Kalender bleibt. Ohne openany.de bekommen die Mitglieder ihn, sobald sie in der Nähe abgleichen – dort nur zum Lesen; Abo-Kalender teilst du nur so.',
        },
        freigaben: {
          title: 'Inhalte freigeben',
          p1: 'Im Reiter „Freigaben“ teilst du mit „Teilen“ eine Notiz-Mappe, ein Album oder einen Ordner (auch eine Akte) mit dem Hub: erst die Art wählen, dann was. Alles bleibt Ende-zu-Ende verschlüsselt – mit einem eigenen Schlüssel je Freigabe, der wechselt, wenn du das Teilen beendest oder jemand den Hub verlässt. Die Inhalte bleiben bei dir; endet das Teilen, bleibt dir alles erhalten.',
          p2: 'Alben und Ordner sind nur zum Ansehen geteilt: Die Mitglieder öffnen sie im Hub, sehen Unteralben und Unterordner, öffnen Bilder und PDFs und legen mit „Übernehmen“ eine Kopie unter „Aus Hubs“ in ihren eigenen Speicher. Ändern kannst nur du.',
          p3: 'Notiz-Mappen teilst du mit der Stufe „ansehen“ oder „bearbeiten“. Die Mitglieder öffnen sie im gewohnten Notiz-Editor und sehen auch die Bilder darin; mit „bearbeiten“ ändern sie Notizen und legen neue an. Bilder einfügen kann vorerst nur, wem die Mappe gehört.',
        },
        lokal: {
          title: 'Auf openany.de oder nur vor Ort',
          p1: 'Mit „+ Hub" legst du einen Hub auf diesem Gerät an. Ist das Gerät mit openany.de verbunden, kommt er gleich auch dorthin – in der Liste trägt er dann eine kleine Wolke. Wer einen Hub gegründet hat, schaltet das unter Einstellungen → Abgleich → „Hubs auf openany.de" an und aus. Ausgeschaltet gleicht er nur noch vor Ort ab; auf openany.de werden Hub, Chat, Planung und Freigaben gelöscht. Mit Wolke lädst du Mitglieder über ihren openany-Namen ein (nur Eigentümer, und nur wen, der die App schon einmal mit openany.de abgeglichen hat). Ohne Wolke lädst du unter „Mitglied in der Nähe einladen" ein: Auf dem anderen Gerät muss Openany offen sein, im selben WLAN oder Hotspot, und beide Geräte zeigen dieselben sechs Ziffern, die ihr vergleicht und bestätigt. Verwaltet wird der Hub von den Geräten des Eigentümers; gibt es davon nur eines, warnt die App – paare dann vorher ein zweites eigenes Gerät.',
          p2: 'Mitglieder geben eigene Notiz-Mappen, Ordner und Alben unter „Etwas freigeben" in den Hub, zunächst nur zum Lesen. Notiz-Mappen lassen sich auch zum Bearbeiten freigeben: Dann bearbeitet immer nur eine Person eine Notiz, und gespeichert wird auf dem Gerät der Person, der die Mappe gehört – ist es nicht in der Nähe, bleibt die Notiz nur lesbar. Der Chat erreicht sofort alle, die gerade in der Nähe sind, die übrigen beim nächsten Abgleich. Auch das Board reist so mit.',
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
          p1: 'In der Webapp stehen unter Nachrichten nur Hinweise: Benachrichtigungen aus deinen Hubs, die Meldung, dass dich jemand zu einem Hub hinzugefügt hat, Antworten auf das Kontaktformular und Meldungen zu deinen Geräten. Die Suche über der Liste findet Hinweise nach Text und Absender.',
          p2: 'Direktnachrichten – auch über Matrix, E-Mail und vor Ort – gibt es nur in der App. Sie sind Ende-zu-Ende verschlüsselt, und die Schlüssel liegen auf deinen Geräten, nicht auf openany.de. Deshalb kann die Webapp sie weder zeigen noch schreiben.',
        },
        suche: {
          title: 'Suchen und filtern',
          p1: 'Über dem Verlauf steht ein Suchfeld. Es findet Nachrichten nach ihrem Text, nach dem Namen des Gegenübers und nach Matrix-Kennungen und hebt die Fundstellen hervor. Darunter schränken Schalter die Liste ein: auf ungelesene Nachrichten, auf solche mit Anhang und – wenn du mehr als einen Weg nutzt – auf einzelne Wege wie Openany oder Matrix. Die Schalter lassen sich kombinieren.',
          p2: 'Neben dem Suchfeld wechselst du zwischen zwei Ansichten: „Verlauf" zeigt alle Nachrichten der Zeit nach, „Nach Kontakt" fasst sie zu einer Zeile je Gegenüber zusammen, mit der letzten Nachricht und der Zahl der ungelesenen. Ein Klick auf eine Zeile zeigt den Verlauf mit genau dieser Person; „Unterhaltung schließen" führt zurück zu allen.',
        },
        anhaenge: {
          title: 'Anhänge',
          p1: 'Über Matrix schickst du auch Dateien: Die Büroklammer im Schreibfeld hängt eine Datei bis 10 MB an, auf Handy und Tablet nimmt der Kamera-Knopf daneben gleich ein Foto auf. Im Verlauf erscheinen Bilder als Vorschau, andere Dateien mit Name und Größe; ein Klick öffnet Bilder und PDFs direkt in Openany, alles andere lädst du herunter. Über den Weg Openany gehen Dateien bis 10 MB, Ende-zu-Ende verschlüsselt wie der Text. Sie zählen zu deinem Speicher und zu dem des Empfängers, bis jeder die Nachricht bei sich löscht.',
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
          p3: 'Per E-Mail gehen Anhänge bis 20 MB. Empfangene Anhänge legst du mit „Speichern" in deine Dateien oder mit „Aufs Gerät" in den Download-Ordner. Kam ein Anhang verschlüsselt, fragt die App vorher nach: In Dateien liegt er unverschlüsselt und wird mit openany.de abgeglichen.',
        },
        pgp: {
          title: 'Verschlüsselte E-Mail mit OpenPGP',
          p1: 'Mails kannst du mit OpenPGP Ende-zu-Ende verschlüsseln. Dafür braucht jedes Postfach einen eigenen Schlüssel: In den Einstellungen am Postfach erzeugst du ihn mit „Schlüssel erzeugen" – oder, wenn du das Postfach schon mit PGP nutzt, etwa in Thunderbird, liest du den vorhandenen mit „Schlüssel einlesen" ein, damit beide Programme dieselben Mails lesen. Der geheime Schlüssel liegt nur auf diesem Gerät. „Sicherungskopie speichern" legt ihn, mit einer Passphrase verschlossen, im Download-Ordner ab; „Öffentlichen Schlüssel speichern" gibt dir die Datei, die du anderen schickst.',
          p2: 'Die öffentlichen Schlüssel deiner Kontakte sammelt die App unter „Schlüssel der Kontakte" von selbst aus Mails, die sie mitschicken. Fehlt einer, suchst du ihn nach der Adresse – auf Wunsch auch bei keys.openpgp.org, das dann erfährt, nach wem du fragst – oder liest ihn aus einer Datei ein. Vergleiche den Fingerabdruck am besten einmal mit dem Gegenüber; ändert sich ein Schlüssel, weist die App darauf hin.',
          p3: 'Beim Schreiben schaltest du „Verschlüsselt senden" ein; darunter steht, ob der Schlüssel des Empfängers bekannt ist. Antwortest du auf eine verschlüsselte Mail, ist der Schalter schon an. Im Verlauf tragen Mails die Marke „verschlüsselt" und, wenn sie unterschrieben sind, „signiert" – oder eine Warnung, wenn die Unterschrift nicht stimmt.',
        },
        vorOrt: {
          title: 'Nachrichten vor Ort',
          p1: 'Mit dem Weg „Vor Ort" schreibst du Geräten in der Nähe direkt, ganz ohne Server. Auf dem anderen Gerät muss Openany geöffnet sein, und beide müssen im selben WLAN sein. Schreiben könnt ihr einander, sobald ihr euch kennt: eigene Geräte, Mitglieder eines gemeinsamen Hubs – oder Personen, die ihr vor Ort bestätigt habt. Dafür wählst du „Vor Ort bestätigen", beide Geräte zeigen dieselben 6 Ziffern, und beide tippen „Passt". Ist das andere Gerät gerade nicht da, wartet eine Nachricht und geht beim nächsten Treffen hinüber.',
          p2: 'Von Unbekannten kommt zunächst nichts an. Schaltest du in den Einstellungen unter „Geräte in der Nähe" „Anfragen von Geräten in der Nähe erlauben" ein, können sie dir bis zu 3 Nachrichten mit je 500 Zeichen schicken; diese stehen unter „Anfragen" über dem Verlauf, nicht im Verlauf selbst. Antworten kannst du erst nach der Bestätigung per 6 Ziffern. Wer dir nichts mehr schicken soll, den sperrst du mit „Blockieren". Wurde eine eigene Nachricht abgelehnt, steht an ihr „nicht zugestellt", und der Grund erscheint, wenn du darauf zeigst.',
        },
        systemnachrichten: {
          title: 'Benachrichtigungen und Antworten',
          p1: 'Ereignisse aus deinen Hubs – etwa neue Freigaben – erreichen dich als Hinweis, sofern der Absender die Mitglieder informieren lässt; ebenso die Meldung, dass dich jemand zu einem Hub hinzugefügt hat. Hinweise sind nicht verschlüsselt und stehen in App und Webapp. Auch die Antwort auf eine Nachricht über das Kontaktformular findest du hier.',
        },
      },
    },
    contacts: {
      title: 'Adressbuch',
      subs: {
        adressbuch: {
          title: 'Kontakte und ihre Wege',
          p1: 'Das Adressbuch öffnest du über seine Kachel auf der Startseite oder oben auf der Nachrichten-Seite; der Knopf daneben legt direkt einen neuen Kontakt an. Ein Kontakt hält fest, wie jemand erreichbar ist: Name, Bild, Telefon und E-Mail sowie beliebig viele weitere Wege – Matrix, Meshtastic, Openany-Name, Anschrift, Betrieb, Geburtstag oder Sonstiges, jeweils mit eigener Beschriftung wie privat oder Arbeit. Das Suchfeld findet Kontakte nach Namen.',
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
          p1: 'Dein Openany-Name steht fest und lässt sich nicht ändern. Die Zeitzone stellst du hier ein – ein Knopf übernimmt die Zone des Geräts. Darunter zeigt „Speicherplatz", wie viel du belegst.',
        },
        module: {
          title: 'Module auf der Startseite',
          p1: 'Hier wählst du, welche Module als Kachel auf der Startseite erscheinen – Notizen, Kalender, Speicher, Hubs und Adressbuch. Die Auswahl blendet nichts aus; alle Module bleiben erreichbar. Ohne Auswahl zeigt die Startseite die Willkommens-Übersicht.',
        },
        sicherheit: {
          title: 'Sicherheit: Passwort und zweiter Faktor',
          p1: 'Passwort, zweiter Faktor und deine Geräte liegen bei anyid, nicht bei Openany. Die Karte „Konto & Sicherheit" in den Einstellungen führt mit drei Links direkt dorthin: „Passwort ändern", „Zweiter Faktor" und „Deine Geräte", wo du gekoppelte Geräte ansiehst und wieder hinauswirfst. Änderungen gelten für Openany, anyitem und anytail zugleich.',
          p2: 'Der zweite Faktor ist ein Code aus einer Authenticator-App, der nach dem Passwort abgefragt wird; ohne ihn lässt sich kein App-Passwort anlegen. Es gibt weder Wiederherstellungscodes noch ein Zurücksetzen per E-Mail. Verlierst du Passwort oder Authenticator-App, kann dir auch niemand sonst wieder hereinhelfen.',
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
          p1: 'Unter „Abgleich" verbindest du die App mit „Mit openany.de verbinden" mit deinem Konto. Hast du schon ein verbundenes Gerät, schaltest du das neue darauf mit 6 Ziffern frei (siehe „Meine Geräte“). Beim ersten Gerät zeigt die App einen Code, den du auf der Seite von anyid im Browser eintippst und bestätigst. Diese Bestätigung im Browser ist kein Umweg, sondern der Nachweis, dass wirklich du das Gerät hinzufügst. Verbinden ist freiwillig: Ohne Verbindung bleibt alles auf diesem Gerät, und „Verbindung trennen" macht es jederzeit rückgängig, ohne dass hier Daten verloren gehen.',
          p2: 'Danach gleicht „Abgleichen" in beide Richtungen ab und sagt hinterher, was geholt, geschoben oder übersprungen wurde. „Alles neu holen" lädt den ganzen Bestand noch einmal statt nur des Neuen; gelöscht wird dabei nichts.',
          p3: 'Mit „Im Hintergrund auffrischen" gleicht die App etwa stündlich von selbst ab – nur mit Netz und nicht bei leerem Akku. Im Mobilfunk kommt dabei nur die Liste, Dateien und Bilder erst im WLAN. Im WLAN versucht sie es dabei auch mit gepaarten Geräten in der Nähe, wenn Openany dort läuft. Auf dem Computer gleicht Openany stündlich ab, solange es läuft; mit „Im Hintergrund weiterlaufen" startet es beim Anmelden und bleibt beim Schließen des Fensters als Symbol in der Menüleiste.',
        },
        geraeteNah: {
          title: 'Geräte in der Nähe',
          p1: 'Deine eigenen Geräte gleichen sich auch direkt ab, ganz ohne Server – solange beide im selben WLAN oder Hotspot sind und auf beiden Openany läuft. Unter „Geräte in der Nähe" findest du sie mit „Suchen" und paarst sie mit „Paaren": Beide Geräte zeigen denselben Code, und auf beiden bestätigst du, dass er passt. Danach gleichen sie sich von selbst ab, solange Openany auf beiden läuft, und bei jedem „Abgleichen"; „Vergessen" löst die Paarung wieder.',
          p2: 'Unter „Dein Name" legst du fest, wie du anderen vor Ort erscheinst. Unter diesem Namen lädt man dich in Hubs ein – dich als Person, nicht ein einzelnes Gerät; er gilt für alle deine gepaarten Geräte.',
        },
        meineGeraete: {
          title: 'Meine Geräte',
          p1: 'Was du Ende-zu-Ende verschlüsselst, können nur deine Geräte lesen; sie teilen dafür einen Schlüssel. Ein neues Gerät zeigt 6 Ziffern, und du tippst sie hier auf einem schon eingerichteten Gerät ein. Geht ein Gerät verloren, ziehst du es hier zurück; dabei wechselt der Schlüssel. Der Schlüssel steckt auch in deiner Sicherungsdatei.',
        },
        speicherGeraet: {
          title: 'Speicher auf diesem Gerät',
          p1: 'Unter „Speicher auf diesem Gerät“ entscheidest du, was hier liegt. „Alles behalten“ holt alles aus deinen Alben, Akten und Ordnern, solange Platz ist; das ist für Rechner und Tablet gedacht. Bei „Ausgewählte Alben, Akten und Ordner“ liegt hier nur, was du im Speicher mit „Auf diesem Gerät behalten“ wählst; von allem anderen siehst du nur den Namen. Neue Kamerafotos kommen in das Album „Kamera“. Was in keinem Album, keiner Akte und keinem Ordner liegt, bleibt auf dem Gerät und kommt in keinen Abgleich.',
          p2: 'Auf den Server gehen Notizen, Kalender, Kontakte, Nachrichten und die Namen von Alben, Akten und Ordnern. Fotos und Dateien gehen nur dorthin, wenn sie in einem Hub geteilt sind, an einer Notiz hängen oder du „Auf den Server legen“ wählst, einzeln oder für ein ganzes Album, eine Akte oder einen Ordner. Alles andere gleichen deine Geräte direkt untereinander ab. „Vom Server nehmen“ löscht nichts: Deine Geräte behalten ihre Kopien, und die letzte Kopie bleibt immer erhalten.',
          p3: 'Unter „Speicher auf diesem Gerät“ siehst du auch, wie viel auf dem Server liegt, aufgeteilt nach Nachrichten, Notizen, Mediathek, Dateien und Dokumenten.',
        },
        sofort: {
          title: 'Sofort benachrichtigen',
          p1: 'Mit „Sofort benachrichtigen" hält die App sparsame Verbindungen zu Openany und zu deinen E-Mail-Postfächern, damit neue Nachrichten und Mails sofort ankommen – ganz ohne Google. Für die Postfächer braucht es kein Openany. Android zeigt dafür einen dauerhaften Hinweis an, den du in den Systemeinstellungen ausblenden kannst. Auf dem Computer wacht Openany, solange es läuft, und meldet sich mit einer Mitteilung des Systems.',
        },
        sicherung: {
          title: 'Sicherung in einer Datei',
          p1: 'Unter „Sicherung" legst du mit „Sicherung anlegen" eine verschlüsselte Datei mit allem an, was Openany auf diesem Gerät hat: Notizen, Kalender, Kontakte, Dateien, Mediathek, Hubs, Nachrichten und deine Postfächer samt Passwort und eigenem OpenPGP-Schlüssel. Die App schlägt eine Passphrase aus sechs Wörtern vor; schreib sie auf und bewahre sie getrennt von der Datei auf. Ohne sie lässt sich die Datei nicht öffnen – auch nicht von uns.',
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
