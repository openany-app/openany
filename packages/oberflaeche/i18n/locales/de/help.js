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
          p1: 'Notizen liegen in Mappen, die sich beliebig tief verschachteln lassen. Notizen und Mappen kannst du umbenennen, verschieben und neu anordnen – so wächst mit der Zeit eine eigene Wissenssammlung. Das Suchfeld über der Liste durchsucht alle Notizen auf einmal, Titel und Inhalt, unabhängig von der gerade geöffneten Mappe.',
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
          p1: 'Für wissenschaftliches Arbeiten kannst du einer Mappe eine Literaturdatei im CSL-JSON-Format zuweisen – etwa einen Export aus Zotero, den du zuvor in den Speicher hochgeladen hast. Der Knopf „Bibliothek verknüpfen" an der Mappe stellt die Verbindung her; Untermappen erben die Bibliothek automatisch.',
          p2: 'In Notizen dieser Mappe schlägt der Editor beim Zitieren passende Quellen vor und setzt einen Kurzbeleg in den Text. So entstehen Belege beim Schreiben, ohne die Quellenangaben jedes Mal abzutippen.',
        },
        graph: {
          title: 'Graph-Ansicht',
          p1: 'Die Graph-Ansicht zeichnet deine Notizen als Netz: Jede Notiz ist ein Punkt, jeder Wikilink eine Verbindung. Die Punkte sind nach oberster Mappe eingefärbt, sodass zusammengehörige Bereiche schon an der Farbe erkennbar sind; hohle Punkte stehen für Notizen ohne jede Verbindung, und je mehr Verknüpfungen eine Notiz hat, desto größer ihr Punkt.',
          p2: 'Jede oberste Mappe ist eine Ebene. Gibt es mehrere, wählst du in der Legende unter dem Graphen eine oder mehrere aus – „Alle Ebenen" nimmt alle, „Keine" leert die Auswahl. Statt einer Ebene kann auch ein Tag der Einstieg sein: Die Leiste darunter nennt die Tags mit ihrer Anzahl, und ein gewählter Tag zeigt alle Notizen, die ihn tragen, samt dem Tag als grauer Raute. Sind Ebenen gewählt, stehen in der Leiste nur noch die Tags, die darin vorkommen. Bis du etwas wählst, bleibt der Graph leer, damit große Sammlungen nicht auf einmal aufgebaut werden; „Alle Tags anzeigen" holt alle Tags der Leiste auf einmal hinein.',
          p3: 'Am Computer zoomst du mit den Plus-/Minus-Knöpfen oben rechts oder dem Mausrad und verschiebst die Ansicht mit gedrückter Maustaste; auf dem Handy zoomst du mit zwei Fingern und verschiebst mit einem. Ein weiterer Knopf setzt die Ansicht zurück. Ein Klick auf einen Punkt (ohne Ziehen) öffnet die Notiz, ein Klick auf einen Tag filtert die Notizliste danach.',
        },
        uebersicht: {
          title: 'Automatische Übersicht',
          p1: 'Neben dem Graphen gibt es eine automatische Übersicht: Sie listet alle Notiztitel von A bis Z und zeigt darunter einen aufklappbaren Baum deiner Schlagworte (verschachtelte Tags erscheinen als Zweige). Ein Klick auf einen Titel öffnet die Notiz, ein Klick auf ein Schlagwort filtert die Liste – ganz ohne dass du selbst ein Inhaltsverzeichnis pflegen musst.',
        },
        export: {
          title: 'PDF-Export',
          p1: 'Jede Notiz lässt sich als PDF herunterladen – zum Drucken, Archivieren oder Weitergeben. Die Formatierung aus dem Editor bleibt dabei erhalten.',
        },
        papierkorb: {
          title: 'Papierkorb',
          p1: 'Gelöschte Notizen landen im Papierkorb (im Menü unter deinem Namen, auf dem Handy im Menü-Knopf) und lassen sich dort wiederherstellen oder endgültig löschen. Beim Löschen einer Mappe entscheidest du, ob die enthaltenen Notizen mitgelöscht oder in die übergeordnete Mappe verschoben werden.',
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
          p1: 'Im Reiter „Dateien" legst du Ordner an und lädst Dateien jeder Art hoch – einzeln oder mehrere auf einmal. Ordner wie Dateien lassen sich umbenennen und verschieben, Dateien wieder herunterladen; Ordner kannst du außerdem in Projekte freigeben und als ZIP herunterladen.',
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
          p1: 'Im Reiter „Galerie" liegen deine Fotos und Videos – in Alben mit Vorschau-Kacheln oder, ohne Album, darunter unter „Bilder". Alben lassen sich verschachteln, etwa ein Album je Reise mit einem Unteralbum je Tag, und jederzeit verschieben. Die Lightbox blättert durch alle Bilder eines Albums.',
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
          p1: 'Gelöschte Inhalte – Dokumente, Dateien, Fotos, Notizen, Kontakte, aber auch Projekte, Boards, Kalender und Termine – wandern zunächst in den gemeinsamen Papierkorb und lassen sich dort wiederherstellen. Nach 30 Tagen leert sich der Papierkorb automatisch; erst dann ist der Speicherplatz endgültig frei.',
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
        tagesansicht: {
          title: 'Tagesansicht',
          p1: 'Ein Klick auf die Zahl eines Tages öffnet die Tagesansicht – eine Durchsicht statt eines Rasters, gedacht für die Frage am Vorabend: Was braucht das Kind morgen? Ganz oben steht aus dem Betreuungsplan, bei wem es ist („bei dir" oder bei wem sonst). Darunter folgen die Stunden aus dem Stundenplan der Reihe nach, an jeder das, was dafür fällig ist; Klassenarbeiten und andere Fristen der nächsten Tage erscheinen schon vorab mit „in … Tagen". Mit den Pfeilen blätterst du zum Vortag oder zum nächsten Tag.',
          p2: 'An jeder Stunde trägst du mit „Hausaufgabe" schnell eine ein; sie landet als Karte mit dem Fach im Board des Projekts. „Heft" öffnet die Mappe zum Fach. Was fällig ist, aber zu keiner Stunde gehört, steht unter „Ohne Stunde". Ein Eintrag führt mit einem Klick an seine Stelle im Projekt.',
        },
        ausProjekten: {
          title: 'Aus Projekten',
          p1: 'Unter „Aus Projekten" in der Kalenderliste blendest du ein, was deine Projekte zum Kalender beitragen: Stundenplan, Betreuung und Fälligkeiten – Karten und Meilensteine mit einer Frist. Jede Quelle lässt sich einzeln ein- und ausblenden, und neue Pläne erscheinen von selbst. Die Einträge sind nur zu lesen; ein Klick zeigt die Einzelheiten, und „Im Projekt öffnen" führt dorthin, wo du sie änderst.',
        },
        abos: {
          title: 'Kalender-Abos',
          p1: 'Externe Kalender – etwa Feiertage oder der Spielplan des Vereins – bindest du per Abo-URL im ICS-Format ein. Abonnierte Kalender aktualisieren sich automatisch. Ihre Termine sind in Openany nur zu lesen: Geändert wird dort, wo der Kalender herkommt, denn jeder Abruf ersetzt den Inhalt wieder durch das Original.',
          p2: 'Umgekehrt kannst du eigene Kalender per Abo-Link freigeben: Das Feed-Symbol in der Kalender-Liste erzeugt eine geheime URL, die sich in Google, Apple oder Outlook abonnieren lässt – Änderungen kommen dort automatisch an. Behandle den Link wie ein Passwort; über „Link erneuern" wird der alte sofort ungültig.',
        },
        importExport: {
          title: 'Import und Export',
          p1: 'Bestehende Kalender importierst du als ICS-Datei, und ebenso exportierst du deine Kalender wieder – volle Datenmitnahme in beide Richtungen, ohne Bindung an Openany.',
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
          p1: 'Jedes Projekt hat einen gemeinsamen Chat in Echtzeit: Nachrichten erscheinen sofort bei allen Mitgliedern, ohne die Seite neu zu laden.',
          p2: 'Wichtige Nachrichten kann jedes Mitglied anheften – angeheftete Nachrichten sind für alle in der Pin-Leiste über dem Verlauf schnell wiederzufinden und lassen sich dort auch wieder lösen.',
          p3: 'Mit zwei eckigen Klammern verweist du mitten im Satz auf Inhalte des Projekts: auf Notizen, auf Boards, Roadmaps und Orte-Sammlungen samt der einzelnen Karten, Meilensteine und Orte darin sowie auf freigegebene Ordner, Dateien, Alben und Bilder. Eine Vorschlagsliste zeigt beim Tippen, was in Frage kommt, und setzt den Verweis fertig ein. Denselben Dienst tut der Knopf mit dem Ketten-Symbol neben dem Senden-Knopf – auf dem Handy meist bequemer, als zwei Klammern zu suchen.',
          p4: 'Ein Klick auf einen Verweis führt direkt zum Ziel: in die Notiz, auf das Board mit der aufgeschlagenen Karte, in den Ordner mit hervorgehobener Datei oder zum Bild in der Lightbox. Verweisen kannst du nur auf das, was im Projekt ohnehin sichtbar ist – was niemand freigegeben hat, taucht weder in der Vorschlagsliste auf, noch wird ein Verweis darauf anklickbar.',
        },
        planung: {
          title: 'Der Planung-Tab',
          p1: 'Im Reiter „Planung" sammelt das Projekt seine Planungswerkzeuge. Über den „Neu"-Knopf wählst du den passenden Typ aus: Terminfindung, Umfrage, Schichtplan, Mitbringliste, Einladungsliste, Board, Roadmap, Orte, Schulferien, Stundenplan oder Betreuungsplan. Die Vorlage „Preset: Schule" legt Schulferien, Stundenplan, Klassenarbeiten und ein Hausaufgaben-Board auf einmal an; dabei wählst du dein Bundesland, damit die Ferien gleich drinstehen, und auf Wunsch das Stundenraster deiner Schule. Erstellen und bearbeiten darf jedes Mitglied; löschen darf der jeweilige Ersteller oder der Projekt-Eigentümer.',
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
        boards: {
          title: 'Kanban-Boards',
          p1: 'Auf Kanban-Boards organisiert ihr Aufgaben in frei gestaltbaren Spalten und Karten – zum Beispiel „Offen", „In Arbeit", „Fertig". Karten lassen sich per Ziehen verschieben; Änderungen erscheinen live bei allen Mitgliedern. Eine Karte kann außerdem einen Ort aus den Orte-Sammlungen des Projekts und ein Fach tragen – so gehört eine Hausaufgabe zur passenden Stunde.',
        },
        roadmap: {
          title: 'Roadmap',
          p1: 'Die Roadmap zeigt Meilensteine mit Datum und Status auf einer Zeitachse – so sieht das ganze Team, was bis wann fertig sein soll. Meilensteine lassen sich als erreicht markieren, überfällige sind gekennzeichnet. Wie Karten können auch Meilensteine einen Ort und ein Fach tragen, etwa eine Klassenarbeit.',
        },
        orte: {
          title: 'Orte',
          p1: 'Unter Orte sammelt ihr Kartenpunkte auf OpenStreetMap – Treffpunkte, Adressen, Parkplätze. Den Punkt setzt du per Klick auf die Karte, dazu optional Name und Notiz; ein Klick auf den Namen einer Ortskachel öffnet den Punkt direkt auf OpenStreetMap. Einmal gesammelt, wählst du die Orte an anderer Stelle einfach aus: an Karten, Meilensteinen, Fächern und Einträgen im Stunden- oder Betreuungsplan.',
        },
        schuljahr: {
          title: 'Schulferien',
          p1: 'Schulferien halten ein Schuljahr fest: den Zeitraum und die freien Tage darin. Die Ferien deines Bundeslandes lassen sich als ICS-Datei einlesen, einzelne Studien- und Feiertage trägst du von Hand nach. Der Unterschied zählt: In den Ferien gilt oft eine andere Betreuungsregelung, an einem einzelnen freien Tag läuft sie im gewohnten Rhythmus weiter. Der Import ergänzt nur und ersetzt nichts, was du selbst eingetragen hast.',
        },
        wochenplan: {
          title: 'Stundenplan und Betreuungsplan',
          p1: 'Beide sind derselbe Baustein: ein Wochenraster von Montag bis Sonntag, das sich Woche für Woche wiederholt. Der Stundenplan trägt Uhrzeiten und Fächer, der Betreuungsplan ganze Tage und eine Person – „wer hat das Kind wann". Verknüpfst du Schulferien damit, fällt der Unterricht in den Ferien von selbst aus; ein Betreuungsplan lässt sich wahlweise auf die Schulzeit oder auf die Ferien begrenzen. Wechselt euer Rhythmus wöchentlich, stellst du auf A/B-Wochen um: entweder alle zwei Wochen ab dem Montag der A-Woche oder nach geraden und ungeraden Kalenderwochen, wenn eure Vereinbarung es so festhält – dann springt der Rhythmus allerdings in Jahren mit 53 Kalenderwochen.',
          p2: 'Unter dem Raster stehen die Abschnitte. Sie überschreiben es in ihrem Zeitraum und decken das ab, was kein Rhythmus ist: die Sommerferien hälftig geteilt, ein getauschtes Wochenende, eine Klassenfahrt. Die Zeitzone gehört dem Plan und nicht dem Betrachter – so steht der Stundenplan für beide Eltern zur selben Uhrzeit da, auch wenn einer im Ausland lebt.',
        },
        faecher: {
          title: 'Fächer',
          p1: 'Die Fächer eines Projekts pflegst du über „Fächer" im Stundenplan: Name, Kürzel, Lehrkraft, Farbe und auf Wunsch ein Ort. Sie gehören dem Projekt und nicht einem einzelnen Plan, überstehen also den Halbjahreswechsel. Ein Fach legt fest, was eine Stunde im Stundenplan zeigt, und verbindet Hausaufgaben-Karten und Klassenarbeiten mit der passenden Stunde. Löschst du ein Fach, verlieren Karten, Meilensteine und Stunden nur den Verweis darauf – sie selbst bleiben.',
        },
        hefte: {
          title: 'Hefte zu den Fächern',
          p1: 'Zu jedem Fach legt „Mappe anlegen und freigeben" eine Notiz-Mappe an – das Heft – und gibt sie gleich ins Projekt frei. Die Mappe gehört dir, nicht dem Projekt, und zählt auf deinen Speicher; am Fach steht deshalb, wessen Mappe es ist. Ein Projekt besitzt selbst keine Inhalte: Was darin liegt, haben Mitglieder freigegeben.',
        },
        planAbo: {
          title: 'Stundenplan und Betreuung abonnieren',
          p1: 'Über „Abo" erzeugst du einen Link, mit dem sich der Stundenplan oder die Betreuung in Google, Apple oder Thunderbird abonnieren lässt – ohne Openany zu öffnen. Der Link gilt für den Typ, nicht für einen einzelnen Plan: Führt ihr die Ferienbetreuung als zweiten Plan, steht sie im selben Abo. Orte gibst du nur auf ausdrücklichen Wunsch mit – der Link ist öffentlich und braucht keine Anmeldung, und mit Orten stünden eure Adressen dahinter. Kalender-Programme holen das Abo nur alle paar Stunden ab; „Erneuern" macht den alten Link sofort ungültig, „Widerrufen" schaltet ihn ab.',
        },
        freigaben: {
          title: 'Inhalte freigeben',
          p1: 'Im Reiter „Freigaben" liegt alles, was Mitglieder dem Projekt zur Verfügung gestellt haben: Notiz-Mappen, Alben sowie Ordner und Akten aus dem Speicher. Freigegeben wird dort, wo der Inhalt zu Hause ist – in den Notizen, in der Galerie oder im Speicher –, wahlweise mit der Stufe „Nur Lesen" oder „Bearbeiten". Eine Freigabe gilt für alle Projekt-Mitglieder, vererbt sich auf Untermappen, Unteralben und Unterordner und lässt sich jederzeit wieder entfernen. Die Inhalte selbst bleiben beim Eigentümer; endet die Freigabe, bleibt ihm alles erhalten.',
          p2: 'Ein Klick auf eine Freigabe öffnet sie im Projekt: Notizen in der gewohnten Arbeitsfläche samt Suche, Rückverweisen und einem Graph über alle freigegebenen Notizen des Projekts, Alben als Fotoraster mit Lightbox, Ordner als Dateiliste. Mit „Bearbeiten" dürfen Mitglieder auch selbst beitragen – Notizen schreiben, Fotos hochladen, Dateien ablegen und Unterordner anlegen; „Nur Lesen" erlaubt Ansehen und Herunterladen.',
        },
        projektNotizen: {
          title: 'Gemeinsame Projekt-Notizen',
          p1: 'In einer mit „Bearbeiten" freigegebenen Notiz-Mappe können Mitglieder direkt im Projekt Notizen anlegen, bearbeiten und löschen sowie Unter-Mappen erstellen; eingebettete Bilder und Dateien sehen alle Mitglieder, und wer bearbeiten darf, kann auch selbst welche einfügen. Alles Angelegte – auch hochgeladene Bilder und Dateien – gehört dem Eigentümer der Mappe und zählt auf dessen Speicher; gelöschte Notizen landen in seinem Papierkorb, sodass nichts endgültig verloren geht.',
        },
        ki: {
          title: 'KI im Projekt',
          p1: 'Ein Projekt kann eine KI angebunden bekommen. Das richtet der Projekt-Eigentümer im Reiter „Mitglieder" unter der Mitgliederliste ein: Anbieter, Modell und ein eigener API-Schlüssel. Der Schlüssel wird verschlüsselt gespeichert und danach niemandem mehr angezeigt, auch dem Eigentümer nicht; die Kosten trägt das Konto, dem der Schlüssel gehört. Alle Mitglieder sehen an derselben Stelle, ob eine KI angebunden ist, an welchen Anbieter und wer sie eingerichtet hat.',
          p2: 'Gefragt wird aus freigegebenen Inhalten heraus: unter einer Notiz über „KI fragen", bei Dokumenten in freigegebenen Akten und Ordnern (PDF, Word, OpenDocument und Textdateien) und – sofern das Modell Bilder versteht – bei Fotos in der Lightbox eines freigegebenen Albums. Du schreibst einen Auftrag, etwa „Fasse es in drei Sätzen zusammen", und bekommst die Antwort zum Kopieren; wer die Notiz bearbeiten darf, kann sie auch direkt am Ende einfügen.',
          p3: 'Wichtig: Was du fragst, verlässt Openany. Über dem Eingabefeld steht jedes Mal, was an welchen Anbieter geht – der ganze Notiztext, der Text des Dokuments oder das Foto, verkleinert und ohne Aufnahmeort. Eingescannte PDFs ohne Textebene brauchen vorher die Texterkennung, sehr lange Dokumente werden abgelehnt.',
        },
        benachrichtigungen: {
          title: 'Mitglieder informieren',
          p1: 'Bei neuen Umfragen, Terminfindungen oder Freigaben kannst du die Mitglieder auf Wunsch per Direktnachricht informieren – so verpasst niemand Neuigkeiten im Projekt.',
        },
        lokal: {
          title: 'Projekte nur vor Ort',
          p1: 'In der App legst du mit „+ Projekt" ein Projekt nur auf diesem Gerät an – ohne Konto und ohne Server; es trägt die Marke „Nur vor Ort". Andere lädst du unter „Mitglied in der Nähe einladen" ein: Auf dem anderen Gerät muss Openany offen sein, im selben WLAN oder Hotspot, und beide Geräte zeigen dieselben sechs Ziffern, die ihr vergleicht und bestätigt. Verwaltet wird das Projekt von den Geräten des Eigentümers; gibt es davon nur eines, warnt die App – paare dann vorher ein zweites eigenes Gerät.',
          p2: 'Mitglieder geben eigene Notiz-Mappen, Ordner und Alben unter „Etwas freigeben" ins Projekt, zunächst nur zum Lesen. Notiz-Mappen lassen sich auch zum Bearbeiten freigeben: Dann bearbeitet immer nur eine Person eine Notiz, und gespeichert wird auf dem Gerät der Person, der die Mappe gehört – ist es nicht in der Nähe, bleibt die Notiz nur lesbar. Der Chat erreicht sofort alle, die gerade in der Nähe sind, die übrigen beim nächsten Abgleich. Auch die Planung – Boards, Roadmaps, Orte und die Schulplanung – reist so mit.',
          p3: 'Den Rest gleichst du mit „Abgleichen" ab, sobald ein Mitglied in der Nähe ist. Der Eigentümer kann Mitglieder entfernen; wer selbst gehen will, tritt mit „Austreten" aus, und das geht nur, wenn ein anderes Mitglied in der Nähe ist, das davon erfährt. Was Ausgeschiedene geteilt hatten, verschwindet bei den anderen; was sie selbst schon hatten, bleibt auf ihren Geräten.',
        },
      },
    },
    messages: {
      title: 'Nachrichten',
      subs: {
        direkt: {
          title: 'Direktnachrichten',
          p1: 'Die Nachrichten erreichst du am Computer über den Briefumschlag in der Kopfzeile, auf dem Handy über den Menü-Knopf unten links. Mit „Nachricht" oben rechts schreibst du jedem anderen Nutzer direkt – als Empfänger genügt sein Openany-Name. Neue Nachrichten erscheinen sofort, ohne Neuladen der Seite, und Links darin sind anklickbar.',
          p2: 'Unter jeder empfangenen Nachricht öffnet „Antworten" das Schreibfeld mit dem Absender schon eingetragen, auf demselben Weg, auf dem die Nachricht kam. Lange Nachrichten sind eingeklappt; „Weiterlesen" zeigt sie ganz. Der Ungelesen-Zähler zeigt jederzeit, ob etwas Neues wartet – am Computer am Briefumschlag, auf dem Handy am Menü-Knopf der unteren Leiste.',
        },
        suche: {
          title: 'Suchen und filtern',
          p1: 'Über dem Verlauf steht ein Suchfeld. Es findet Nachrichten nach ihrem Text, nach dem Namen des Gegenübers und nach Matrix-Kennungen und hebt die Fundstellen hervor. Darunter schränken Schalter die Liste ein: auf ungelesene Nachrichten, auf solche mit Anhang und – wenn du mehr als einen Weg nutzt – auf einzelne Wege wie Openany oder Matrix. Die Schalter lassen sich kombinieren.',
          p2: 'Neben dem Suchfeld wechselst du zwischen zwei Ansichten: „Verlauf" zeigt alle Nachrichten der Zeit nach, „Nach Kontakt" fasst sie zu einer Zeile je Gegenüber zusammen, mit der letzten Nachricht und der Zahl der ungelesenen. Ein Klick auf eine Zeile zeigt den Verlauf mit genau dieser Person; „Unterhaltung schließen" führt zurück zu allen.',
        },
        anhaenge: {
          title: 'Anhänge',
          p1: 'Über Matrix schickst du auch Dateien: Die Büroklammer im Schreibfeld hängt eine Datei bis 10 MB an, auf Handy und Tablet nimmt der Kamera-Knopf daneben gleich ein Foto auf. Im Verlauf erscheinen Bilder als Vorschau, andere Dateien mit Name und Größe; ein Klick öffnet Bilder und PDFs direkt in Openany, alles andere lädst du herunter. Über den Weg Openany gehen nur Texte.',
        },
        matrix: {
          title: 'Nachrichten über Matrix',
          p1: 'Menschen ohne Openany-Konto erreichst du über Matrix. Dafür verbindest du in den Einstellungen unter „Matrix-Konto" ein bestehendes Konto bei matrix.org oder einem anderen Homeserver – anlegen musst du es dort. Das Passwort wird nur zum Anmelden benutzt und nicht gespeichert. Openany meldet sich dabei als eigenes Gerät an und kann ab dann alle neuen Nachrichten dieses Kontos mitlesen, auch die, die du in einem anderen Matrix-Programm schreibst; nach außen bleibt alles verschlüsselt.',
          p2: 'Ist ein Konto verbunden, wählst du beim Schreiben den Weg: Openany oder Matrix, dann mit der Matrix-Kennung als Empfänger. Antworten kommen in denselben Verlauf und sind mit „Matrix" gekennzeichnet. Löschst du eine Matrix-Nachricht, verschwindet sie nur in Openany; im Raum bleibt sie stehen. Über „Verbindung trennen" meldest du das Gerät wieder ab; bereits empfangene Nachrichten bleiben erhalten.',
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
          p1: 'Ereignisse aus deinen Projekten – etwa neue Umfragen oder Freigaben – erreichen dich als automatische Direktnachricht, sofern der Absender die Mitglieder informieren lässt. Auch die Antwort auf eine Nachricht über das Kontaktformular findest du hier.',
        },
      },
    },
    contacts: {
      title: 'Adressbuch',
      subs: {
        adressbuch: {
          title: 'Kontakte und ihre Wege',
          p1: 'Das Adressbuch öffnest du oben auf der Nachrichten-Seite; der Knopf daneben legt direkt einen neuen Kontakt an. Ein Kontakt hält fest, wie jemand erreichbar ist: Name, Bild, Telefon und E-Mail sowie beliebig viele weitere Wege – Matrix, Meshtastic, Openany-Name, Anschrift, Betrieb, Geburtstag oder Sonstiges, jeweils mit eigener Beschriftung wie privat oder Arbeit. Das Suchfeld findet Kontakte nach Namen.',
          p2: 'Die Wege sind anklickbar: Eine Telefonnummer ruft an, eine E-Mail-Adresse öffnet das Mailprogramm, ein Openany-Name oder eine Matrix-Kennung öffnet das Schreibfeld mit dem Empfänger schon eingetragen. Ein Kontakt kann mit einem Openany-Konto verknüpft sein – das ist nur ein Verweis und gibt keinen Zugriff auf Projekte oder Inhalte. Gelöschte Kontakte landen im Papierkorb.',
        },
        adressbuchDatei: {
          title: 'Adressbuch sichern und übernehmen',
          p1: 'In den Einstellungen unter „Adressbuch" lädst du alle Kontakte als vCard-Datei herunter oder liest eine vCard-Datei aus einem anderen Programm ein. Einlesen legt nur dazu: Vorhandene Kontakte werden über ihre Kennung oder ihren Namen wiedererkannt und behalten, was sie haben. Alle Angaben reisen in Standardfeldern; verlorengehen können höchstens eigene Beschriftungen, weil viele Adressbücher nur „privat" und „Arbeit" kennen.',
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
          p1: 'Programme auf deinem Rechner melden sich nicht mit deinem Kontopasswort an, sondern mit einem eigenen App-Passwort. Beim Anlegen gibst du ihm einen Namen und wählst den Zweck: „Programme" für WebDAV, Zettlr oder einen Dateimanager, die deine Notizen lesen und schreiben, oder „Abgleich" für Programme, die dein Konto spiegeln und zurückschreiben. Anlegen geht nur, wenn du dich mit zweitem Faktor angemeldet hast.',
          p2: 'Das neue Passwort wird genau einmal angezeigt – kopiere es sofort. Im Programm trägst du deinen Openany-Namen als Benutzernamen ein, die WebDAV-Adresse steht im Hinweis unter dem Passwort. Die Liste zeigt, wann jedes Passwort zuletzt benutzt wurde; widerrufst du eines, verliert das Programm sofort den Zugang.',
        },
        abgleich: {
          title: 'Abgleich mit Openany',
          p1: 'Unter „Abgleich" verbindest du die App mit „Mit openany.de verbinden" mit deinem Konto. Die App zeigt einen Code, den du auf der Seite von anyid im Browser eintippst und bestätigst. Diese Bestätigung im Browser ist kein Umweg, sondern der Nachweis, dass wirklich du das Gerät hinzufügst. Verbinden ist freiwillig: Ohne Verbindung bleibt alles auf diesem Gerät, und „Verbindung trennen" macht es jederzeit rückgängig, ohne dass hier Daten verloren gehen.',
          p2: 'Danach gleicht „Abgleichen" in beide Richtungen ab und sagt hinterher, was geholt, geschoben oder übersprungen wurde. „Alles neu holen" lädt den ganzen Bestand noch einmal statt nur des Neuen; gelöscht wird dabei nichts.',
          p3: 'Mit „Im Hintergrund auffrischen" gleicht die App etwa stündlich von selbst ab – nur mit Netz und nicht bei leerem Akku. Im Mobilfunk kommt dabei nur die Liste, Dateien und Bilder erst im WLAN. Geräte in der Nähe gehören nicht dazu; für sie muss die App offen sein.',
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
          p1: 'Mit „Sofort benachrichtigen" hält die App sparsame Verbindungen zu Openany und zu deinen E-Mail-Postfächern, damit neue Nachrichten und Mails sofort ankommen – ganz ohne Google. Für die Postfächer braucht es kein Openany. Android zeigt dafür einen dauerhaften Hinweis an, den du in den Systemeinstellungen ausblenden kannst.',
        },
        sicherung: {
          title: 'Sicherung in einer Datei',
          p1: 'Unter „Sicherung" legst du mit „Sicherung anlegen" eine verschlüsselte Datei mit allem an, was Openany auf diesem Gerät hat: Notizen, Kalender, Kontakte, Dateien, Galerie, Projekte, Nachrichten und deine Postfächer samt Passwort und eigenem OpenPGP-Schlüssel. Die App schlägt eine Passphrase aus sechs Wörtern vor; schreib sie auf und bewahre sie getrennt von der Datei auf. Ohne sie lässt sich die Datei nicht öffnen – auch nicht von uns.',
          p2: 'Mit „Sicherung einspielen" öffnest du eine solche Datei auf einem anderen Gerät. Sie ersetzt alles, was dort war; danach ist die Verbindung zu openany.de getrennt, und Geräte in der Nähe paarst du neu. Was ein Gerät ausweist, kommt nicht mit.',
        },
        tresor: {
          title: 'Ausweise und Schlüssel im Tresor',
          p1: 'Womit sich dieses Gerät bei Openany und anderen Geräten ausweist, liegt verschlossen auf dem Gerät; der Schlüssel dazu bleibt im Android-Keystore. Dort liegen auch die Passwörter deiner Postfächer und deine geheimen OpenPGP-Schlüssel. „Verbindung trennen" entfernt die Ausweise nur hier – endgültig widerrufen wird in der Geräteliste bei anyid und unter den App-Passwörtern von Openany.',
        },
      },
    },
    contact: {
      title: 'Hilfe und Kontakt',
      subs: {
        kontakt: {
          title: 'Kontakt zu den Betreibern',
          p1: 'Fragen, Probleme oder Wünsche? Über den Link „Kontakt" in der Fußzeile erreichst du das Kontaktformular. Deine Nachricht geht als Direktnachricht an die Admins, und die Antwort findest du später unter Nachrichten.',
        },
        bedingungen: {
          title: 'Nutzungsbedingungen',
          p1: 'Die Nutzungsbedingungen der Beta und das Impressum findest du jederzeit über die Links in der Fußzeile. Kurz gefasst: Openany ist während der Beta kostenlos und werbefrei, und deine Daten liegen auf Servern in Deutschland. Hinaus geht nur, was du selbst hinausschickst – etwa Nachrichten über Matrix oder Anfragen an die KI eines Projekts.',
        },
      },
    },
  },
};
