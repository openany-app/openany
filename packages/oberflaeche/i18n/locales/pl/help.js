// Uwaga vue-i18n: w tekstach bez surowych @, | ani {.
export default {
  title: 'Pomoc',
  intro: 'Tutaj znajdziesz opisy i instrukcje do wszystkich modułów i funkcji Openany. Spis treści po lewej prowadzi prosto do właściwej sekcji.',
  version: 'Wersja {version}',
  toc: 'Spis treści',
  sections: {
    start: {
      title: 'Pierwsze kroki',
      subs: {
        konto: {
          title: 'Konto i logowanie',
          p1: 'Konta w Openany powstają wyłącznie na osobiste zaproszenie – nie ma otwartej rejestracji. Adres e-mail jest dobrowolny: Openany działa także bez niego, ale z e-mailem możesz sam zresetować zapomniane hasło.',
          p2: 'Logujesz się nazwą użytkownika i hasłem przez anyid, wspólną usługę logowania. To samo logowanie działa w Openany, anyitem i anytail; dlatego hasło i drugi składnik są zarządzane tam, a nie w samym Openany (zob. „Bezpieczeństwo” w ustawieniach).',
        },
        ohneKonto: {
          title: 'Aplikacja bez konta',
          p1: 'Aplikacja to pełny program, który działa także bez konta i bez serwera: notatki, kalendarz, dysk, książka adresowa i wiadomości są wtedy tylko na tym urządzeniu. To, czego tu nie skonfigurowano albo co nie działa, aplikacja ukrywa, zamiast pokazywać puste – droga Openany w wiadomościach pojawia się na przykład dopiero wtedy, gdy urządzenie jest połączone z openany.de.',
          p2: 'Jeśli chcesz synchronizować dane z Openany w sieci, stuknij „Połącz z openany.de” w ustawieniach w „Synchronizacja”; z innymi swoimi urządzeniami parujesz się w „Urządzenia w pobliżu”. Potem w nagłówku i w menu pojawia się przycisk „Synchronizuj”, który obraca się podczas synchronizacji.',
        },
        startseite: {
          title: 'Strona startowa',
          p1: 'Strona startowa pokazuje zwięzłe kafelki wybranych modułów: najbliższe terminy, ostatnio edytowane notatki, ostatnie pliki z paskiem zajętości dysku, twoje projekty z licznikiem nieprzeczytanych, najnowsze wiadomości w skrzynce i książkę adresową. Każdy kafelek prowadzi do swojego modułu; które się pojawiają, ustalasz w ustawieniach w „Moduły na stronie startowej”.',
          p2: 'Jeśli nie wybrano modułów na stronę startową, pojawia się ekran powitalny: przedstawia wszystkie funkcje i pozwala od razu złożyć kafelki. Wybór niczego nie ukrywa – wszystkie obszary pozostają dostępne przez pasek i menu.',
        },
        navigation: {
          title: 'Menu i układ',
          p1: 'Na komputerze u góry jest nagłówek: po lewej okrągłe logo prowadzi do strony startowej, obok są Notatki, Kalendarz, Dysk i Projekty. Po prawej znajdują się przełącznik jasny/ciemny, koperta wiadomości z licznikiem nieprzeczytanych i pole z twoją nazwą – otwiera Ustawienia, Pomoc, Kosz i Wyloguj.',
          p2: 'Na telefonie wszystko przesuwa się na dół: stały pasek pokazuje moduły jako ikony, a skrajnie po lewej jest przycisk menu z licznikiem nieprzeczytanych wiadomości. Otwiera panel z Wiadomościami, Ustawieniami, Pomocą, Koszem, przełącznikiem jasny/ciemny i wylogowaniem. Przełącznik zmienia tryb między jasnym, ciemnym i automatycznym – automatyczny podąża za ustawieniem urządzenia.',
        },
      },
    },
    notes: {
      title: 'Notatki',
      subs: {
        editor: {
          title: 'Pisanie w edytorze',
          p1: 'Edytor notatek działa jak nowoczesny edytor tekstu: nagłówki, listy, cytaty, bloki kodu, pogrubienie i kursywę wybierasz z paska narzędzi. Możesz też wpisywać skróty Markdown – na przykład krzyżyk ze spacją dla nagłówka albo myślnik dla listy – a zmienią się w formatowanie podczas pisania.',
          p2: 'Zapis jest automatyczny: chwilę po tym, jak przestaniesz pisać, Openany zapisuje notatkę i pokazuje stan zapisu na pasku narzędzi. Pod każdą notatką są też data utworzenia i ostatniej zmiany. Na telefonie pasek narzędzi pojawia się w jednej linii tuż nad klawiaturą, gdy dotkniesz tekstu, i można go przewijać w bok – wszystkie narzędzia są pod ręką, ale nie przeszkadzają w czytaniu.',
        },
        mappen: {
          title: 'Teczki i struktura',
          p1: 'Notatki leżą w teczkach, które można dowolnie zagnieżdżać. Notatkom i teczkom możesz zmieniać nazwy, przenosić je i porządkować – tak z czasem rośnie własny zbiór wiedzy. Lupa u góry otwiera pole wyszukiwania, które przeszukuje wszystkie notatki naraz, tytuł i treść; otwarte w teczce, „tylko w tej teczce” ogranicza je do niej. Co można zrobić z otwartą teczką, znajdziesz w „Teczka” po prawej stronie paska ścieżki.',
          p2: 'Całą teczkę z podteczkami pobierzesz jako archiwum ZIP; notatki są w nim plikami Markdown i pozostają czytelne wszędzie.',
        },
        wikilinks: {
          title: 'Wikilinki i linki zwrotne',
          p1: 'Dwoma nawiasami kwadratowymi łączysz notatki ze sobą: wpisz nawiasy i tytuł notatki docelowej, a link powstanie automatycznie – jeśli notatka jeszcze nie istnieje, link zadziała, gdy tylko się pojawi. Jeśli w tekście ma stać coś innego niż tytuł, podajesz własny tekst po pionowej kresce; link i tak prowadzi do właściwej notatki.',
          p2: 'Każda notatka pokazuje swoje linki zwrotne: listę wszystkich notatek, które do niej prowadzą. Tak odnajdziesz powiązania bez prowadzenia własnych zapisków.',
        },
        tags: {
          title: 'Tagi',
          p1: 'Słowa kluczowe nadajesz bezpośrednio w tekście krzyżykiem przed słowem; przy pisaniu pomaga lista już użytych tagów. Ukośnikiem zagnieżdżasz słowa kluczowe – np. Projekt/Podtemat; filtr na temat główny znajdzie wtedy także wszystkie podtematy. W widoku grafu słowa kluczowe można pokazać jako osobne węzły, a kliknięcie słowa kluczowego filtruje tam listę notatek.',
        },
        einfuegen: {
          title: 'Wstawianie obrazów i plików',
          p1: 'Przyciskiem spinacza na pasku narzędzi wstawiasz treści do notatki – świeżo przesłane albo jako odnośnik do treści już zapisanych w Openany. Obrazy pojawiają się jako oszczędny podgląd bezpośrednio w notatce (pełna rozdzielczość leży w Galerii), inne pliki jako klikalny link do pobrania.',
          p2: 'Wstawione treści są automatycznie porządkowane: obrazy do albumu w Galerii, pliki tekstowe i Office do segregatora w Dokumentach, cała reszta do folderu plików – zawsze w osobnym obszarze dla notatek. Dopóki odnośnik jest w co najmniej jednej notatce, pliku nie da się przenieść do kosza z Dysku ani Galerii; gdy usuniesz odnośnik z tekstu, znów można go usunąć.',
        },
        zitate: {
          title: 'Cytowania i bibliografia',
          p1: 'Do pracy naukowej możesz przypisać teczce plik bibliografii w formacie CSL-JSON – na przykład eksport z Zotero, który wcześniej przesłano na Dysk. W otwartej teczce połączenie tworzy „Teczka” → „Połącz bibliotekę”; podteczki dziedziczą bibliotekę automatycznie.',
          p2: 'W notatkach tej teczki edytor przy cytowaniu podpowiada pasujące źródła i wstawia krótki przypis do tekstu. Tak przypisy powstają w trakcie pisania, bez przepisywania danych źródła za każdym razem.',
        },
        graph: {
          title: 'Widok grafu',
          p1: 'Widok grafu rysuje twoje notatki jako sieć: każda notatka to punkt, każdy wikilink to połączenie. Punkty są pokolorowane według najwyższej teczki, więc powiązane obszary widać już po kolorze; puste punkty to notatki bez żadnych połączeń, a im więcej powiązań ma notatka, tym większy jej punkt.',
          p2: 'Każda teczka najwyższego poziomu to poziom. Jeśli jest ich kilka, wybierasz jeden lub więcej w legendzie pod grafem – „Wszystkie poziomy” bierze wszystkie, „Żaden” czyści wybór. Zamiast poziomu wejściem może być też tag: pasek poniżej wymienia tagi z ich liczbą, a wybrany tag pokazuje wszystkie notatki, które go mają, razem z tagiem jako szarym rombem. Gdy wybrane są poziomy, pasek wymienia już tylko tagi, które w nich występują. Dopóki niczego nie wybierzesz, graf pozostaje pusty, żeby duże zbiory nie budowały się naraz; „Pokaż wszystkie tagi” dodaje od razu wszystkie tagi z paska.',
          p3: 'Na komputerze powiększasz przyciskami plus/minus w prawym górnym rogu lub kółkiem myszy i przesuwasz widok z wciśniętym przyciskiem myszy; na telefonie powiększasz dwoma palcami i przesuwasz jednym. Kolejny przycisk resetuje widok. Kliknięcie punktu (bez przeciągania) otwiera notatkę. Kliknięcie tagu filtruje według niego listę notatek.',
        },
        export: {
          title: 'Eksport do PDF',
          p1: 'Każdą notatkę można wyeksportować jako PDF – do druku, archiwizacji lub przekazania dalej. Formatowanie z edytora zostaje zachowane. PDF trafia do teczki „Notizen” w Pamięć → Dokumenty (wcześniejsze wersje zostają obok); „Otwórz” w komunikacie od razu go pokazuje.',
        },
        druckfassung: {
          title: 'Wersja do druku',
          p1: 'Folder staje się formalnym dokumentem do oddania lub wydruku: w otwartym folderze wybierz w „Teczka” pozycję „Wersja do druku”, wpisz tytuł, podtytuł, imię i nazwisko, instytucję, datę i dodatkowe informacje na stronę tytułową oraz zdecyduj, czy dodać spis treści. Każda notatka z folderu staje się rozdziałem, alfabetycznie według tytułu; przedrostkami, np. „01 Wstęp”, ustalasz kolejność, a w dokumencie pojawia się tylko „Wstęp”. Podfoldery są uwzględniane, gdy zaznaczono „Uwzględnij podfoldery”: podfolder staje się nagłówkiem, a jego notatki trafiają pod niego („2 Podstawy”, „2.1 Pojęcia”); notatka o tej samej nazwie co podfolder pojawia się jako tekst bezpośrednio pod tym nagłówkiem.',
          p2: 'Skład wykorzystuje krój podobny do Times z bezszeryfowymi nagłówkami, marginesy 2,5 cm, interlinię 1,5, wyjustowany tekst z dzieleniem wyrazów; numeracja stron zaczyna się po stronie tytułowej. Dane pozostają zapisane przy folderze. PDF powstaje na twoim urządzeniu – za pierwszym razem trwa to kilka sekund – i trafia do teczki „Druckfassungen” w Pamięć → Dokumenty; każda wersja zostaje zachowana.',
          p3: "Cytaty: w otwartym folderze otwierasz „Teczka” → „Źródła” i wpisujesz książki, artykuły, strony, orzeczenia lub ustawy; każde dostaje skrót. W tekście piszesz [{'@'}skrot] lub [{'@'}skrot, s. 12], a ^[Tekst] dla przypisu. W PDF – notatki lub wersji do druku – powstają z tego cytaty harwardzkie, np. „(Müller, 2020, s. 12)”, a na końcu bibliografia. Lista to plik CSL-JSON w Plikach, działa więc także z Zotero i Zettlr. Pasek narzędzi edytora ma przycisk przypisu (cyfra w indeksie górnym): wstawia nawiasy, a zaznaczony tekst staje się przypisem.",
        },
        papierkorb: {
          title: 'Kosz',
          p1: 'Usunięte notatki trafiają do kosza (w menu pod twoją nazwą, na telefonie w przycisku menu) i można je tam przywrócić lub usunąć trwale. Przy usuwaniu teczki decydujesz, czy zawarte notatki też zostaną usunięte, czy przeniesione do teczki nadrzędnej.',
        },
        verschluesselt: {
          title: 'Szyfrowanie end-to-end',
          p1: 'Twoje notatki, foldery, dane wersji do druku i listy źródeł są przechowywane na openany.de wyłącznie w postaci zaszyfrowanej: odczytać je mogą tylko twoje zatwierdzone urządzenia. Serwer nie widzi też nazw twoich folderów. Wyszukiwanie, odwołania zwrotne, tagi i graf oblicza twoje urządzenie.',
        },
        ordner: {
          title: 'Notatki jako pliki (Zettlr)',
          p1: 'W aplikacji na komputerze możesz połączyć swój segregator z folderem: otwórz segregator, wybierz w menu „…” pozycję „Połącz z folderem …” i wskaż folder. Każda notatka leży tam jako plik „Id Tytuł.md”, podsegregatory jako podfoldery – tak, jak oczekują tego Zettlr czy Obsidian. To, co tam zmienisz, przemianujesz, przeniesiesz lub utworzysz, aplikacja przejmuje, dopóki działa; jeśli zmieniły się obie strony, powstaje kopia konfliktu. Gdy usuniesz plik, notatka trafia do kosza. Folder leży na komputerze niezaszyfrowany; „Odłącz folder” zostawia pliki na miejscu. Obrazy z notatek leżą obok notatki, pod nazwą z tekstu; nowy obraz, do którego odsyła notatka, trafia do aplikacji (galeria, album „Notizen: …”).',
        },
      },
    },
    files: {
      title: 'Dysk',
      subs: {
        aufbau: {
          title: 'Galeria, pliki, dokumenty',
          p1: 'Dysk dzieli się na trzy karty u góry strony: Galerię na zdjęcia i filmy, Pliki na wszystko inne i Dokumenty na pisma w segregatorach. Gdy zmieniasz kartę, drugi obszar zostaje taki, jak go zostawiłeś – otwarte foldery i wczytane listy nie znikają.',
          p2: 'Która karta otwiera się jako pierwsza, gdy wchodzisz na Dysk, ustawiasz w ustawieniach w „Start dysku”; dopóki nic nie wybierzesz, są to Dokumenty.',
        },
        dokumente: {
          title: 'Archiwum dokumentów',
          p1: 'W karcie „Dokumenty” porządkujesz pliki PDF, Office i tekstowe w segregatorach. Przesyłasz przyciskiem lub po prostu przeciągając pliki do otwartego segregatora. Na telefonie i tablecie obok jest przycisk aparatu: fotografuje pismo i od razu zapisuje je jako dokument w segregatorze.',
          p2: 'Segregatory można dowolnie zagnieżdżać. Segregatorom i pojedynczym dokumentom możesz zmieniać nazwy i je przenosić; całe segregatory możesz też udostępniać w projektach i pobierać jako ZIP.',
        },
        texterkennung: {
          title: 'Przeszukiwalne dokumenty',
          p1: 'Jeśli sfotografujesz pismo i zapiszesz je w segregatorze – przyciskiem aparatu albo jako zdjęcie ze swojej kolekcji – automatycznie powstanie PDF z niewidoczną warstwą tekstu. Dokument wygląda jak twoje zdjęcie, ale da się go przeszukiwać, zaznaczać, kopiować i odczytywać na głos. Rozpoznawanie działa na twoim urządzeniu – obraz nigdzie nie jest wysyłany.',
          p2: 'Jeśli zapisujesz kilka zdjęć naraz, pojawi się pytanie, czy mają tworzyć jeden dokument wielostronicowy – np. strony listu – czy osobne. Samo zdjęcie nie jest dodatkowo zapisywane; kto chce zachować obraz, zapisuje go w Galerii. Za pierwszym razem konwersja trwa dłużej, bo rozpoznawanie tekstu wczytuje się jednorazowo i zostaje zapisane.',
          p3: 'Jeśli w segregatorze jest już zeskanowany PDF, możesz zastosować rozpoznawanie później: „Rozpoznaj tekst” czyta dokument strona po stronie. Jeśli papier jest czysto biały, strony pozostają bez zmian i dochodzi tylko tekst. Jeśli są szare lub nierówno oświetlone – jak sfotografowana kartka – zostaną dodatkowo rozjaśnione; komunikat na końcu powie, co się stało. Jeśli PDF był już przeszukiwalny, pozostaje nietknięty.',
        },
        dateien: {
          title: 'Zarządzanie plikami',
          p1: 'W karcie „Pliki” tworzysz foldery i przesyłasz pliki dowolnego rodzaju – pojedynczo lub kilka naraz. Folderom i plikom można zmieniać nazwy i je przenosić, pliki ponownie pobierać; foldery możesz też udostępniać w projektach i pobierać jako ZIP. Co można zrobić z folderem lub teczką – pobrać, udostępnić, zmienić nazwę, przenieść, usunąć – znajdziesz w otwartym folderze po prawej stronie paska ścieżki w „Folder” (w segregatorze: „Segregator”).',
          p2: 'Każde konto ma limit miejsca; aktualną zajętość widzisz na kafelku strony startowej i w ustawieniach w „Miejsce na dysku”.',
        },
        suche: {
          title: 'Wyszukiwanie',
          p1: 'Lupa w Plikach i Dokumentach otwiera pole wyszukiwania, które przeszukuje wszystkie foldery lub segregatory naraz, a nie tylko ten otwarty. Kliknięcie wyniku otwiera plik; obok widać, gdzie leży, a kliknięcie tej ścieżki przenosi cię do folderu.',
          p2: 'Wyszukiwanie znajduje nie tylko po nazwie, ale też w treści: w PDF-ach, tekstach Word i OpenDocument oraz plikach tekstowych, CSV i Markdown do 50 MB. Openany czyta ten tekst stopniowo w tle, dopóki Dysk jest otwarty; zanim przeczyta wszystko, wyszukiwanie mówi, ile plików na razie znajdzie tylko po nazwie. Przy trafieniu w treści lista pokazuje fragment z wyróżnionym szukanym słowem. Zeskanowane PDF-y bez warstwy tekstu pojawią się dopiero po zastosowaniu „Rozpoznaj tekst”.',
        },
        pdf: {
          title: 'Przeglądanie i edycja PDF',
          p1: 'Kliknięcie PDF-a otwiera go bezpośrednio w Openany, bez pobierania. Przewracasz strony, powiększasz lub pomniejszasz, dopasowujesz widok do szerokości i przeszukujesz dokument lupą – trafienia są wyróżnione na stronie. Jeśli PDF jest chroniony hasłem, Openany najpierw o nie pyta.',
          p2: 'Formularze w PDF-ie wypełniasz bezpośrednio. Przycisk „Edytuj” pokazuje ponadto pasek narzędzi: „Zakreślacz” wyróżnia tekst kolorem, „Pióro” pozwala swobodnie rysować na stronie, a „Tekst” umieszcza na kartce własne linijki – kolor, grubość linii i rozmiar czcionki wybierasz sam, a każdy krok można cofnąć. Pola obliczeniowe w formularzach jednak nie liczą, bo Openany nie uruchamia skryptów z cudzych plików.',
          p3: 'Do zakreśleń i linii możesz dodawać komentarze: wybierz „Komentarze” i dotknij miejsca; ten sam widok wyświetla wszystkie komentarze dokumentu. Wszystko zapisuje się jako zwykłe adnotacje PDF, które pokazują też inne programy. Przy pierwszym zapisie Openany przenosi poprzednią wersję do kosza, więc możesz wrócić do oryginału; jeśli opuszczasz PDF z niezapisanymi zmianami, najpierw zapyta.',
        },
        galerie: {
          title: 'Galeria zdjęć i albumy',
          p1: 'W karcie „Galeria” są twoje zdjęcia i filmy – w albumach z miniaturami albo, bez albumu, poniżej w „Zdjęcia”. Albumy można zagnieżdżać – np. jeden album na podróż z podalbumem na każdy dzień – i w każdej chwili przenosić. Przeglądarka przewija wszystkie zdjęcia albumu. Aby pobrać, udostępnić, przenieść lub usunąć album, otwórz go i użyj „Album” po prawej stronie paska ścieżki.',
          p2: 'Najnowsze ujęcie jest na górze, pogrupowane według miesięcy z nagłówkiem dla każdego miesiąca. Liczy się data wykonania z danych zdjęcia; jeśli jej brak – np. przy zrzutach ekranu lub edytowanych obrazach – liczy się dzień przesłania.',
          p3: 'Całe albumy pobierasz jako ZIP, pojedyncze zdjęcia jako oryginalny plik. Albumy również można udostępniać w projektach.',
        },
        videos: {
          title: 'Filmy i aparat',
          p1: 'Oprócz zdjęć Galeria przyjmuje też filmy. Pojawiają się jako kafelek z klatką podglądu i odtwarzają w przeglądarce zdjęć. Niektóre telefony nagrywają w formacie HEVC, którego nie każda przeglądarka potrafi odtworzyć; wtedy pobierz film i otwórz go innym programem.',
          p2: 'Na telefonie i tablecie Galeria i każdy album mają dwa przyciski aparatu: jeden do zdjęcia, drugi do filmu. Nagranie trafia od razu tam, gdzie jesteś. Na komputerze tych przycisków nie ma; przesyłasz jak zwykle.',
        },
        karte: {
          title: 'Data i miejsce wykonania',
          p1: 'Przy każdym zdjęciu przeglądarka pokazuje datę wykonania i – jeśli zdjęcie ją zawiera – lokalizację; „Pokaż na mapie” otwiera to miejsce w OpenStreetMap. Gdy przesyłasz zdjęcia z telefonu z Androidem, wybieraj je przez aplikację Pliki, a nie przez galerię: inaczej Android usunie lokalizację, zanim zdjęcie dotrze do Openany.',
        },
        papierkorbSpeicher: {
          title: 'Kosz',
          p1: 'Usunięte treści – dokumenty, pliki, zdjęcia, notatki, kontakty, ale też projekty, kalendarze i terminy – trafiają najpierw do wspólnego kosza i można je tam przywrócić. Po 30 dniach kosz opróżnia się automatycznie; dopiero wtedy miejsce jest ostatecznie zwolnione.',
        },
        dateienVerschluesselt: {
          title: 'Szyfrowanie end-to-end',
          p1: 'Twoje pliki, akta, zdjęcia i albumy są przechowywane na openany.de wyłącznie w postaci zaszyfrowanej: nazwy, foldery, albumy, daty wykonania i miejsce są w zaszyfrowanym nagłówku, a same treści są szyfrowane osobnym kluczem dla każdego pliku. Odczytać je mogą tylko twoje zatwierdzone urządzenia. Przy pierwszej synchronizacji w tej wersji aplikacja pobiera wszystkie oryginały, szyfruje je i przesyła ponownie – to chwilę trwa i najlepiej robić to przez Wi-Fi. To, co jest wtedy w koszu, zostaje usunięte na stałe, a udostępnienia albumów i folderów w projektach się kończą.',
        },
      },
    },
    calendar: {
      title: 'Kalendarz',
      subs: {
        verwalten: {
          title: 'Kilka kalendarzy',
          p1: 'Możesz prowadzić dowolnie wiele kalendarzy obok siebie – np. prywatny, rodzinny, klubowy – każdy w swoim kolorze. Pojedyncze kalendarze pokazujesz i ukrywasz kliknięciem. U góry przełączasz widok miesiąca i tygodnia; tydzień pokazuje terminy według godziny, a minione terminy są dyskretnie wyszarzone.',
        },
        termine: {
          title: 'Dodawanie terminów',
          p1: 'Termin tworzysz kliknięciem w dzień: z tytułem, opisem, godziną albo jako wydarzenie całodniowe lub wielodniowe. Możliwe są też powtórzenia – codziennie, co tydzień, co miesiąc lub co rok. Istniejące terminy edytujesz lub usuwasz bezpośrednio z widoku. Godziny obowiązują w strefie czasowej ustawionej w twoim profilu.',
        },
        abos: {
          title: 'Subskrypcje kalendarzy',
          p1: 'Zewnętrzne kalendarze – np. święta albo terminarz meczów klubu – dołączasz adresem subskrypcji w formacie ICS. Zasubskrybowane kalendarze aktualizują się automatycznie. Ich terminy są w Openany tylko do odczytu: zmienia się je tam, skąd pochodzi kalendarz, bo każde odświeżenie zastępuje treść z powrotem oryginałem.',
        },
        verschluesselt: {
          title: 'Szyfrowanie end-to-end',
          p1: 'Twoje kalendarze i wydarzenia są przechowywane na openany.de wyłącznie w postaci zaszyfrowanej: odczytać je mogą tylko Twoje zatwierdzone urządzenia. Adres subskrypcji też jest zaszyfrowany; każde urządzenie pobiera ją samo, openany.de niczego nie pobiera.',
        },
        importExport: {
          title: 'Import i eksport',
          p1: 'Przez „Importuj” wczytujesz plik ICS do jednego ze swoich kalendarzy. Jeśli wczytasz ten sam plik ponownie, wydarzenia zostaną zaktualizowane zamiast zdublowane. Ikona udostępniania przy kalendarzu zapisuje go jako plik ICS – do przeniesienia do dowolnego innego programu kalendarza.',
        },
      },
    },
    projects: {
      title: 'Projekty',
      subs: {
        grundlagen: {
          title: 'Projekty i członkowie',
          p1: 'Projekt to wspólna przestrzeń robocza: zapraszasz innych użytkowników jako członków, a zaproszeni przyjmują lub odrzucają zaproszenie. Właściciel projektu zarządza członkami i może też przekazać projekt innemu członkowi.',
          p2: 'Każdy projekt ma cztery karty: Czat, Planowanie, Członkowie i Udostępnienia. Liczby na kartach pokazują nieprzeczytane wiadomości czatu lub ile w nich jest.',
        },
        chat: {
          title: 'Czat projektu',
          p1: 'Każdy projekt ma wspólny czat w czasie rzeczywistym: wiadomości pojawiają się od razu u wszystkich członków, bez odświeżania strony. Czat jest szyfrowany end-to-end: czytają go tylko członkowie na swoich urządzeniach, openany.de nie. Gdy ktoś odchodzi, klucz zmienia się sam – nie czyta już niczego nowego.',
          p2: 'Ważne wiadomości może przypiąć każdy członek – przypięte wiadomości łatwo znaleźć na pasku nad historią i tam też je odpiąć.',
          p3: 'Dwoma nawiasami kwadratowymi odsyłasz w środku zdania do zawartości projektu: do notatek oraz do udostępnionych folderów, plików, albumów i zdjęć. Podczas pisania lista podpowiedzi pokazuje, co pasuje, i wstawia gotowy odnośnik. To samo robi przycisk z symbolem łańcucha obok przycisku wysyłania – na telefonie zwykle wygodniejszy niż szukanie dwóch nawiasów.',
          p4: 'Kliknięcie odnośnika prowadzi prosto do celu: do notatki, do folderu z wyróżnionym plikiem albo do zdjęcia w podglądzie. Odsyłać możesz tylko do tego, co i tak jest widoczne w projekcie – czego nikt nie udostępnił, nie ma ani na liście podpowiedzi, ani jako klikalnego odnośnika.',
        },
        planung: {
          title: 'Karta Planowanie',
          p1: 'W zakładce „Planowanie" projekt zbiera swoje głosowania i listy. Przyciskiem „Nowe" wybierasz odpowiedni typ: ustalanie terminu, ankieta, grafik dyżurów, lista „kto co przynosi" lub lista gości. Tworzyć i głosować może każdy członek; usuwać może twórca lub właściciel projektu.',
          p2: 'Głosowania są szyfrowane end-to-end – także głosy. Urządzenia członków liczą same i pilnują zasad: przy ograniczonej liczbie miejsc liczy się, kto zapisał się pierwszy. Zamknąć głosowanie może tylko osoba, która je utworzyła; przy ustalaniu terminu twoje urządzenie może dodać wybrany termin do twojego kalendarza.',
        },
        terminfindung: {
          title: 'Ustalanie terminu',
          p1: 'Przy ustalaniu terminu proponujesz kilka opcji, a wszyscy głosują tak, nie lub może – tak umawiacie spotkanie bez długiego przerzucania się wiadomościami.',
        },
        umfragen: {
          title: 'Ankiety',
          p1: 'Ankieta zadaje pytanie ze stałymi odpowiedziami – tak/nie lub własnymi, na życzenie anonimowo. Z opcjami do odhaczania służy też jako wspólna lista zadań; głosowania można duplikować i zamykać.',
        },
        listen: {
          title: 'Grafik dyżurów, lista „kto co przynosi”, lista gości',
          p1: 'Trzy rodzaje list odciążają organizację: w grafiku dyżurów tworzysz dyżury z miejscami, a członkowie się zapisują. Lista „kto co przynosi” wyjaśnia, kto co przyniesie – wpisy można przejmować i uzupełniać. Lista gości trzyma w ryzach gości (zaproszeni, przyjęli, odmówili), także tych bez konta Openany.',
        },
        geteilteKalender: {
          title: 'Udostępnianie kalendarzy',
          p1: 'W „Planowaniu” udostępniasz projektowi jeden ze swoich kalendarzy: członkowie widzą tam najbliższe wydarzenia, a wszystkie we własnym kalendarzu w sekcji „Z projektów” – tylko do odczytu. Zmieniać je możesz tylko ty; zmiany docierają przy następnej synchronizacji. Wszystko jest zaszyfrowane kluczem projektu. Kosz obok kalendarza kończy udostępnianie; twój kalendarz zostaje.',
        },
        freigaben: {
          title: 'Udostępnianie treści',
          p1: 'W karcie „Udostępnienia” przyciskiem „Udostępnij” dzielisz się z projektem teczką notatek, albumem albo folderem (także aktami): najpierw wybierasz rodzaj, potem co. Wszystko pozostaje szyfrowane end-to-end – z osobnym kluczem dla każdego udostępnienia, który zmienia się, gdy kończysz udostępnianie albo ktoś opuszcza projekt. Treści zostają u ciebie; gdy udostępnianie się kończy, wszystko zachowujesz.',
          p2: 'Albumy i foldery są udostępniane tylko do podglądu: członkowie otwierają je w projekcie, widzą podalbumy i podfoldery, otwierają zdjęcia i PDF-y, a przyciskiem „Przejmij” zapisują kopię we własnym magazynie w „Aus Projekten”. Zmieniać możesz tylko ty.',
          p3: 'Teczki notatek udostępniasz z poziomem „podgląd” albo „edycja”. Członkowie otwierają je w zwykłym edytorze notatek i widzą też zdjęcia; z „edycją” zmieniają notatki i dodają nowe. Na razie zdjęcia może wstawiać tylko właściciel teczki.',
        },
        lokal: {
          title: 'Projekty tylko na miejscu',
          p1: 'W aplikacji „+ Projekt” tworzy projekt tylko na tym urządzeniu – bez konta i bez serwera; ma oznaczenie „Tylko na miejscu”. Innych zapraszasz w „Zaproś członka w pobliżu”: na drugim urządzeniu Openany musi być otwarte, w tej samej sieci Wi-Fi lub hotspocie, a oba urządzenia pokazują te same sześć cyfr, które porównujecie i potwierdzacie. Projektem zarządzają urządzenia właściciela; jeśli jest tylko jedno, aplikacja ostrzega – sparuj wcześniej drugie własne urządzenie.',
          p2: 'Członkowie udostępniają w projekcie własne teczki notatek, foldery i albumy przez „Udostępnij coś”, najpierw tylko do odczytu. Teczki notatek można też udostępnić do edycji: wtedy notatkę edytuje naraz tylko jedna osoba, a zapis następuje na urządzeniu osoby, do której należy teczka – jeśli nie ma go w pobliżu, notatka zostaje tylko do odczytu. Czat dociera od razu do wszystkich w pobliżu, do pozostałych przy następnej synchronizacji. Tak samo podróżuje planowanie – głosowania i listy.',
          p3: 'Resztę synchronizujesz przez „Synchronizuj”, gdy tylko ktoś z członków jest w pobliżu. Właściciel może usuwać członków; kto chce odejść, robi to przez „Wystąp”, co działa tylko wtedy, gdy inny członek jest w pobliżu i może się o tym dowiedzieć. To, co udostępnili odchodzący, znika u pozostałych; to, co sami już mieli, zostaje na ich urządzeniach.',
        },
      },
    },
    messages: {
      title: 'Wiadomości',
      subs: {
        direkt: {
          title: 'Wiadomości bezpośrednie',
          p1: 'Wiadomości otwierasz z menu. Przyciskiem „Wiadomość" w prawym górnym rogu piszesz bezpośrednio do każdego innego użytkownika – wystarczy jego nazwa w Openany. Linki w wiadomościach są klikalne.',
          p2: 'Pod każdą odebraną wiadomością „Odpowiedz" otwiera edytor z wpisanym już nadawcą, tym samym kanałem, którym przyszła wiadomość. Długie wiadomości są zwinięte; „Czytaj dalej" pokazuje je w całości. Licznik nieprzeczytanych na przycisku menu pokazuje, czy czeka coś nowego.',
          p3: 'Wiadomości bezpośrednie są szyfrowane end-to-end: openany.de przechowuje tylko nieczytelne dane, a czytać je mogą tylko twoje urządzenia. Wszystkie twoje urządzenia dzielą jeden klucz. Nowe urządzenie pokazuje 6 cyfr; wpisujesz je na już skonfigurowanym i jest zalogowane oraz ma klucz. Klucz jest też w twoim pliku kopii zapasowej. Jeśli zgubisz urządzenie, wycofaj je w ustawieniach w sekcji „Moje urządzenia"; klucz się wtedy zmienia. Możesz pisać do każdego, kto choć raz otworzył aplikację.',
        },
        hinweise: {
          title: 'Powiadomienia',
          p1: 'W aplikacji webowej sekcja Wiadomości pokazuje tylko powiadomienia: z twoich projektów, informację, że ktoś dodał cię do projektu, odpowiedzi na formularz kontaktowy oraz komunikaty o twoich urządzeniach. Wyszukiwarka nad listą znajduje powiadomienia po treści i nadawcy.',
          p2: 'Wiadomości bezpośrednie – również przez Matrix, e-mail i w pobliżu – są dostępne tylko w aplikacji. Są szyfrowane end-to-end, a klucze są na twoich urządzeniach, nie na openany.de. Dlatego aplikacja webowa nie może ich ani pokazać, ani napisać.',
        },
        suche: {
          title: 'Wyszukiwanie i filtry',
          p1: 'Nad historią jest pole wyszukiwania. Znajduje wiadomości po treści, po nazwie rozmówcy i po identyfikatorach Matrix oraz wyróżnia trafienia. Pod nim przełączniki zawężają listę: do nieprzeczytanych, do wiadomości z załącznikiem i – jeśli używasz więcej niż jednej drogi – do pojedynczych dróg, takich jak Openany czy Matrix. Przełączniki można łączyć.',
          p2: 'Obok pola wyszukiwania przełączasz się między dwoma widokami: „Historia” pokazuje wszystkie wiadomości według czasu, „Według kontaktu” zbiera je w jeden wiersz na rozmówcę, z ostatnią wiadomością i liczbą nieprzeczytanych. Kliknięcie wiersza pokazuje historię właśnie z tą osobą; „Zamknij rozmowę” wraca do wszystkich.',
        },
        anhaenge: {
          title: 'Załączniki',
          p1: 'Przez Matrix wysyłasz też pliki: spinacz w polu pisania dołącza plik do 10 MB, a na telefonie i tablecie przycisk aparatu obok od razu robi zdjęcie. W historii obrazy pojawiają się jako podgląd, inne pliki z nazwą i rozmiarem; kliknięcie otwiera obrazy i PDF-y bezpośrednio w Openany, resztę pobierasz. Drogą Openany idą pliki do 15 MB, szyfrowane end-to-end jak tekst. Liczą się do twojej pamięci i do pamięci odbiorcy, dopóki każde z was nie usunie wiadomości.',
        },
        matrixApp: {
          title: 'Wiadomości przez Matrix',
          p1: 'W aplikacji twoje urządzenie samo jest urządzeniem Matrix. Logujesz się istniejącym kontem w ustawieniach w „Konta Matrix”, a twoje wiadomości są szyfrowane end-to-end między tym urządzeniem a rozmówcą – openany.de ich nie czyta. Weryfikacji urządzenia przez porównanie emoji aplikacja jeszcze nie potrafi; inne programy Matrix pokazują ją więc jako niezweryfikowane urządzenie, ale szyfrowanie i tak działa.',
          p2: 'Możesz połączyć kilka kont Matrix. Ich wiadomości mają wspólną historię; nowe wychodzą z konta domyślnego, odpowiedzi z konta, na które przyszła wiadomość, a „Ustaw jako domyślne” ustawia inne konto jako domyślne. Jeśli usuniesz wiadomość Matrix, zniknie tylko na tym urządzeniu; w pokoju zostaje.',
        },
        email: {
          title: 'E-mail przez twoją skrzynkę',
          p1: 'W aplikacji dochodzi e-mail jako kolejna droga – nie osobny program pocztowy, lecz twoja istniejąca skrzynka, np. w Posteo, mailbox.org czy GMX. W ustawieniach w „Skrzynki e-mail” wpisujesz adres i hasło; zwykle jest to hasło aplikacji, które najpierw tworzysz u dostawcy. Serwery aplikacja wyszukuje sama; jeśli żadnych nie znajdzie, wpisujesz je w „Serwery ręcznie”. Hasło pozostaje zamknięte na tym urządzeniu, a urządzenie samo rozmawia z serwerem poczty – openany.de nie widzi żadnej wiadomości.',
          p2: 'Maile pojawiają się z tematem we wspólnej historii w Wiadomościach; „Pobierz teraz” od razu pobiera nowe. Jeśli połączysz kilka skrzynek, nowe maile wychodzą ze skrzynki domyślnej, a odpowiedzi z tej, na którą przyszedł mail; przy pisaniu wybierasz w „Od”. Przy usuwaniu decydujesz, czy mail zniknie tylko tutaj, czy trafi też do folderu kosza na serwerze. Jeśli dostawca uznał coś za spam, nad historią pojawia się informacja; tam „To nie spam” przenosi mail z powrotem do skrzynki odbiorczej.',
          p3: 'E-mailem załączniki mogą mieć do 15 MB. Odebrane załączniki zapisujesz w swoich plikach przez „Zapisz” albo w folderze pobranych przez „Na urządzenie”. Jeśli załącznik przyszedł zaszyfrowany, aplikacja najpierw pyta: w Plikach leży on niezaszyfrowany i jest synchronizowany z openany.de.',
        },
        pgp: {
          title: 'Szyfrowany e-mail z OpenPGP',
          p1: 'Maile możesz szyfrować end-to-end za pomocą OpenPGP. Każda skrzynka potrzebuje do tego własnego klucza: w ustawieniach przy skrzynce tworzysz go przyciskiem „Utwórz klucz” – albo, jeśli już używasz tej skrzynki z PGP, np. w Thunderbirdzie, wczytujesz istniejący przez „Wczytaj klucz”, aby oba programy czytały te same maile. Klucz prywatny jest tylko na tym urządzeniu. „Zapisz kopię zapasową” zapisuje go, zamkniętego hasłem, w folderze pobranych; „Zapisz klucz publiczny” daje ci plik, który wysyłasz innym.',
          p2: 'Klucze publiczne twoich kontaktów aplikacja sama zbiera w „Klucze kontaktów” z maili, które je dołączają. Jeśli jakiegoś brak, szukasz go po adresie – na życzenie także w keys.openpgp.org, który dowie się wtedy, o kogo pytasz – albo wczytujesz z pliku. Najlepiej raz porównaj odcisk palca z rozmówcą; jeśli klucz się zmieni, aplikacja to zaznaczy.',
          p3: 'Przy pisaniu włączasz „Wyślij zaszyfrowane”; pod spodem widać, czy klucz odbiorcy jest znany. Jeśli odpowiadasz na zaszyfrowany mail, przełącznik jest już włączony. W historii maile mają oznaczenie „zaszyfrowane”, a jeśli są podpisane, „podpisane” – albo ostrzeżenie, gdy podpis się nie zgadza.',
        },
        vorOrt: {
          title: 'Wiadomości w pobliżu',
          p1: 'Drogą „W pobliżu” piszesz bezpośrednio do urządzeń w pobliżu, zupełnie bez serwera. Na drugim urządzeniu Openany musi być otwarte, a oba muszą być w tej samej sieci Wi-Fi. Pisać do siebie możecie, gdy się znacie: własne urządzenia, członkowie wspólnego projektu – albo osoby, które potwierdziliście na miejscu. W tym celu wybierasz „Potwierdź na miejscu”, oba urządzenia pokazują te same 6 cyfr i oboje stukacie „Pasuje”. Jeśli drugiego urządzenia akurat nie ma, wiadomość czeka i przejdzie przy następnym spotkaniu.',
          p2: 'Od nieznajomych najpierw nic nie dociera. Jeśli w ustawieniach w „Urządzenia w pobliżu” włączysz „Zezwalaj na prośby od urządzeń w pobliżu”, mogą wysłać ci do 3 wiadomości po 500 znaków; stoją one w „Prośby” nad historią, a nie w samej historii. Odpowiedzieć możesz dopiero po potwierdzeniu 6 cyframi. Kto ma ci już nic nie wysyłać, tego zatrzymujesz przez „Zablokuj”. Jeśli twoja wiadomość została odrzucona, ma oznaczenie „nie dostarczono”, a powód pojawia się po wskazaniu.',
        },
        systemnachrichten: {
          title: 'Powiadomienia i odpowiedzi',
          p1: 'Zdarzenia z twoich projektów – np. nowe ankiety lub udostępnienia – docierają do ciebie jako powiadomienie, jeśli nadawca zdecyduje się poinformować członków; podobnie informacja, że ktoś dodał cię do projektu. Powiadomienia nie są szyfrowane i widać je w aplikacji i w aplikacji webowej. Tutaj znajdziesz też odpowiedź na wiadomość wysłaną przez formularz kontaktowy.',
        },
      },
    },
    contacts: {
      title: 'Książka adresowa',
      subs: {
        adressbuch: {
          title: 'Kontakty i ich kanały',
          p1: 'Książkę adresową otwierasz u góry strony wiadomości; przycisk obok od razu tworzy nowy kontakt. Kontakt zapisuje, jak można się z kimś skontaktować: nazwę, zdjęcie, telefon i e-mail oraz dowolnie wiele innych sposobów – Matrix, Meshtastic, nazwę w Openany, adres, firmę, urodziny lub inne, każdy z własną etykietą, np. prywatny lub praca. Pole wyszukiwania znajduje kontakty po nazwie.',
          p2: 'Sposoby kontaktu są klikalne: numer telefonu dzwoni, adres e-mail otwiera program pocztowy, nazwa w Openany lub identyfikator Matrix otwiera pole pisania z wpisanym już odbiorcą. Usunięte kontakty trafiają do kosza.',
        },
        verschluesselt: {
          title: 'Szyfrowanie end-to-end',
          p1: 'Twoja książka adresowa jest szyfrowana end-to-end tym samym kluczem co wiadomości bezpośrednie. Czytać ją mogą tylko Twoje zatwierdzone urządzenia; openany.de ją przechowuje i synchronizuje, ale nie widzi nazw, numerów ani zdjęć. Nowe urządzenie pokazuje książkę dopiero po zatwierdzeniu go na innym urządzeniu. Dlatego aplikacja webowa nie ma już książki adresowej.',
        },
        vcard: {
          title: 'Zapis i wczytywanie vCard',
          p1: 'W Ustawieniach → Książka adresowa zapiszesz wszystkie kontakty jako plik vCard albo wczytasz plik z innego programu. Wczytywanie tylko dodaje: istniejące kontakty są rozpoznawane po identyfikatorze lub nazwie i zachowują to, co mają. Sam plik nie jest szyfrowany – przechowuj go odpowiednio.',
        },
        sicherheitsnummer: {
          title: 'Numer bezpieczeństwa',
          p1: 'Numerem bezpieczeństwa sprawdzasz, że nikt nie czyta po drodze – nawet openany.de. Otwórz go w kontakcie w sekcji „Bezpieczeństwo” albo w wiadomościach po otwarciu rozmowy z widoku „Według kontaktu”. Porównajcie dwanaście grup cyfr na obu urządzeniach; jeśli się zgadzają, stuknij „Zgadza się”. Wiadomości tej osoby mają wtedy znaczek przy kłódce. Jeśli jej klucz później się zmieni, np. po utracie urządzenia, znaczek znika, a komunikat prosi o ponowne porównanie.',
        },
      },
    },
    settings: {
      title: 'Ustawienia i konto',
      subs: {
        profil: {
          title: 'Profil, strefa czasowa i miejsce na dysku',
          p1: 'Twoja nazwa w Openany jest stała i nie można jej zmienić. Adres e-mail jest dobrowolny i służy tylko celom konta, jak resetowanie hasła, nigdy reklamie; zmienić go możesz tylko z hasłem. Strefa czasowa określa, jak rozumiane są godziny twoich terminów – przycisk przejmuje strefę urządzenia. Poniżej „Miejsce na dysku” pokazuje, ile zajmujesz.',
        },
        module: {
          title: 'Moduły na stronie startowej',
          p1: 'Tutaj wybierasz, które moduły pojawią się jako kafelek na stronie startowej – Notatki, Kalendarz, Dysk, Projekty, Wiadomości i Książka adresowa. Wybór niczego nie ukrywa; wszystkie moduły pozostają dostępne. Bez wyboru strona startowa pokazuje ekran powitalny.',
        },
        sicherheit: {
          title: 'Bezpieczeństwo: hasło i drugi składnik',
          p1: 'Hasło, drugi składnik i twoje urządzenia są w anyid, a nie w Openany. Karta „Konto i bezpieczeństwo” w ustawieniach prowadzi tam bezpośrednio trzema linkami: „Zmień hasło”, „Drugi składnik” i „Twoje urządzenia”, gdzie widzisz powiązane urządzenia i możesz je usunąć. Zmiany obowiązują jednocześnie w Openany, anyitem i anytail.',
          p2: 'Drugi składnik to kod z aplikacji uwierzytelniającej, o który system pyta po haśle; bez niego nie da się utworzyć hasła aplikacji. Przy konfiguracji dostajesz kody odzyskiwania; zachowaj je – bez nich i bez aplikacji nie wejdziesz już na konto. Jeśli masz zapisany adres e-mail, zapomniane hasło możesz zresetować samodzielnie.',
        },
        spracheDesign: {
          title: 'Wygląd i język',
          p1: 'W „Wygląd” wybierasz jeden z dwóch wyglądów: „Klarowny” w morskim odcieniu z ostrzejszymi rogami i spokojnym tłem albo „Liliowy” z zaokrąglonymi rogami i tłem we wzór. Oba są w wersji jasnej i ciemnej – to przełączasz niezależnie przełącznikiem w nagłówku lub w menu na telefonie. W „Język” ustawiasz, w jakim języku Openany do ciebie mówi: niemieckim, angielskim, hiszpańskim, francuskim, portugalskim, polskim lub ukraińskim. Dopóki nie wybierzesz, podąża za językiem urządzenia lub przeglądarki; jeśli go tu nie ma, Openany mówi po angielsku.',
        },
        zugriff: {
          title: 'Dostęp z zewnątrz: hasła aplikacji',
          p1: 'Programy na twoim komputerze nie logują się hasłem do konta, tylko osobnym hasłem aplikacji. Tworząc je, nadajesz mu nazwę i wybierasz cel „Synchronizacja” – dla programów, które odzwierciedlają twoje konto i zapisują zmiany z powrotem. Notatki są na serwerze przechowywane wyłącznie w postaci zaszyfrowanej, dlatego nie ma już dostępu przez WebDAV. Utworzyć hasło można tylko po zalogowaniu z drugim składnikiem.',
          p2: 'Nowe hasło jest wyświetlane dokładnie raz – skopiuj je od razu. W programie wpisujesz nazwę w Openany jako nazwę użytkownika. Lista pokazuje, kiedy każde hasło było ostatnio używane; jeśli je odwołasz, program natychmiast traci dostęp.',
        },
        abgleich: {
          title: 'Synchronizacja z Openany',
          p1: 'W „Synchronizacja” łączysz aplikację ze swoim kontem przez „Połącz z openany.de”. Aplikacja pokazuje kod, który wpisujesz i potwierdzasz na stronie anyid w przeglądarce. To potwierdzenie w przeglądarce nie jest objazdem, lecz dowodem, że to naprawdę ty dodajesz urządzenie. Połączenie jest dobrowolne: bez niego wszystko zostaje na tym urządzeniu, a „Rozłącz” cofa je w każdej chwili, bez utraty danych tutaj.',
          p2: 'Potem „Synchronizuj” synchronizuje w obie strony i informuje, co zostało pobrane, wysłane lub pominięte. „Pobierz wszystko ponownie” pobiera ponownie cały zasób zamiast samych nowości; nic przy tym nie jest usuwane.',
          p3: 'Z „Odświeżaj w tle” aplikacja synchronizuje się sama mniej więcej co godzinę – tylko z połączeniem i nie przy słabej baterii. W sieci komórkowej przychodzi tylko lista, pliki i zdjęcia dopiero przez Wi-Fi. Urządzenia w pobliżu nie są tu uwzględnione; dla nich aplikacja musi być otwarta. Na komputerze Openany synchronizuje się co godzinę, dopóki działa.',
        },
        geraeteNah: {
          title: 'Urządzenia w pobliżu',
          p1: 'Twoje własne urządzenia synchronizują się też bezpośrednio, zupełnie bez serwera – o ile są w tej samej sieci Wi-Fi lub hotspocie, a na obu jest otwarte Openany. W „Urządzenia w pobliżu” znajdujesz je przez „Szukaj” i łączysz przez „Sparuj”: oba urządzenia pokazują ten sam kod, a na obu potwierdzasz, że się zgadza. Odtąd synchronizują się między sobą przy każdym „Synchronizuj”; „Zapomnij” cofa parowanie.',
          p2: 'W „Twoje imię” ustalasz, jak widzą cię inni na miejscu. Pod tym imieniem zaprasza się cię do projektów – ciebie jako osobę, a nie pojedyncze urządzenie; obowiązuje ono na wszystkich twoich sparowanych urządzeniach.',
        },
        speicherGeraet: {
          title: 'Pamięć na tym urządzeniu',
          p1: 'W „Pamięć na tym urządzeniu” decydujesz, ile plików i zdjęć aplikacja trzyma pod ręką. „Na żądanie” pokazuje wszystkie pliki na liście, ale treść pobiera dopiero przy otwarciu. Przy „Wybrane foldery i albumy” na urządzeniu zawsze zostaje to, co oznaczyłeś na Dysku jako „Zachowaj na tym urządzeniu”, razem z podfolderami. „Zachowaj wszystko” po każdej synchronizacji pobiera brakujące rzeczy, dopóki jest miejsce.',
        },
        sofort: {
          title: 'Powiadamiaj od razu',
          p1: 'Z „Powiadamiaj od razu” aplikacja utrzymuje oszczędne połączenia z Openany i twoimi skrzynkami pocztowymi, aby nowe wiadomości i maile przychodziły od razu – bez Google. Skrzynki nie potrzebują do tego Openany. Android pokazuje przy tym stałe powiadomienie, które możesz ukryć w ustawieniach systemu. Na komputerze Openany czuwa, dopóki działa, i daje znać powiadomieniem systemowym.',
        },
        sicherung: {
          title: 'Kopia zapasowa w jednym pliku',
          p1: 'W „Kopia zapasowa” przyciskiem „Utwórz kopię” tworzysz zaszyfrowany plik ze wszystkim, co Openany ma na tym urządzeniu: notatkami, kalendarzem, kontaktami, plikami, galerią, projektami, wiadomościami i skrzynkami pocztowymi razem z hasłem i własnym kluczem OpenPGP. Aplikacja proponuje hasło z sześciu słów; zapisz je i przechowuj osobno od pliku. Bez niego pliku nie da się otworzyć – nawet nam.',
          p2: 'Przyciskiem „Przywróć kopię” otwierasz taki plik na innym urządzeniu. Zastępuje wszystko, co tam było; potem połączenie z openany.de jest usunięte, a urządzenia w pobliżu parujesz od nowa. To, co identyfikuje urządzenie, nie jest dołączane.',
        },
        tresor: {
          title: 'Poświadczenia i klucze w sejfie',
          p1: 'To, czym to urządzenie legitymuje się wobec Openany i innych urządzeń, leży zamknięte na urządzeniu; klucz do tego zostaje w pęku kluczy systemu – na Androidzie w Keystore, na komputerze w pęku kluczy macOS, Menedżerze poświadczeń Windows lub Secret Service w Linuksie. Są tam też hasła twoich skrzynek i twoje tajne klucze OpenPGP. „Rozłącz” usuwa poświadczenia tylko tutaj – ostatecznie odwołuje się je na liście urządzeń w anyid i w hasłach aplikacji Openany.',
        },
      },
    },
    contact: {
      title: 'Pomoc i kontakt',
      subs: {
        kontakt: {
          title: 'Kontakt z operatorami',
          p1: 'Pytania, problemy lub pomysły? Link „Kontakt" w stopce prowadzi do formularza kontaktowego. Twoja wiadomość trafia do administratorów jako powiadomienie, a odpowiedź znajdziesz później w Wiadomościach.',
        },
        bedingungen: {
          title: 'Warunki korzystania',
          p1: 'Warunki korzystania z wersji beta i notę prawną znajdziesz zawsze pod linkami w stopce. W skrócie: w czasie bety Openany jest bezpłatny i bez reklam, a twoje dane leżą na serwerach w Niemczech. Wychodzi tylko to, co sam wysyłasz – np. wiadomości przez Matrix lub e-mail.',
        },
      },
    },
  },
};
