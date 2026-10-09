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
          p1: 'Notatki leżą w teczkach, które można dowolnie zagnieżdżać. Notatkom i teczkom możesz zmieniać nazwy, przenosić je i porządkować – tak z czasem rośnie własny zbiór wiedzy. Pole wyszukiwania nad listą przeszukuje wszystkie notatki naraz, tytuł i treść, niezależnie od otwartej teczki.',
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
          p1: 'Do pracy naukowej możesz przypisać teczce plik bibliografii w formacie CSL-JSON – na przykład eksport z Zotero, który wcześniej przesłano na Dysk. Przycisk „Połącz bibliotekę” przy teczce tworzy połączenie; podteczki dziedziczą bibliotekę automatycznie.',
          p2: 'W notatkach tej teczki edytor przy cytowaniu podpowiada pasujące źródła i wstawia krótki przypis do tekstu. Tak przypisy powstają w trakcie pisania, bez przepisywania danych źródła za każdym razem.',
        },
        graph: {
          title: 'Widok grafu',
          p1: 'Widok grafu rysuje twoje notatki jako sieć: każda notatka to punkt, każdy wikilink to połączenie. Punkty są pokolorowane według najwyższej teczki, więc powiązane obszary widać już po kolorze; puste punkty to notatki bez żadnych połączeń, a im więcej powiązań ma notatka, tym większy jej punkt.',
          p2: 'Każda teczka najwyższego poziomu to poziom. Jeśli jest ich kilka, wybierasz jeden lub więcej w legendzie pod grafem – „Wszystkie poziomy” bierze wszystkie, „Żaden” czyści wybór. Zamiast poziomu wejściem może być też tag: pasek poniżej wymienia tagi z ich liczbą, a wybrany tag pokazuje wszystkie notatki, które go mają, razem z tagiem jako szarym rombem. Gdy wybrane są poziomy, pasek wymienia już tylko tagi, które w nich występują. Dopóki niczego nie wybierzesz, graf pozostaje pusty, żeby duże zbiory nie budowały się naraz; „Pokaż wszystkie tagi” dodaje od razu wszystkie tagi z paska.',
          p3: 'Na komputerze powiększasz przyciskami plus/minus w prawym górnym rogu lub kółkiem myszy i przesuwasz widok z wciśniętym przyciskiem myszy; na telefonie powiększasz dwoma palcami i przesuwasz jednym. Kolejny przycisk resetuje widok. Kliknięcie punktu (bez przeciągania) otwiera notatkę. Kliknięcie tagu filtruje według niego listę notatek.',
        },
        uebersicht: {
          title: 'Automatyczny spis',
          p1: 'Obok grafu jest automatyczny spis: wymienia wszystkie tytuły notatek od A do Z, a pod nimi pokazuje rozwijane drzewo słów kluczowych (zagnieżdżone tagi są gałęziami). Kliknięcie tytułu otwiera notatkę, kliknięcie słowa kluczowego filtruje listę – bez potrzeby prowadzenia własnego spisu treści.',
        },
        export: {
          title: 'Eksport do PDF',
          p1: 'Każdą notatkę można pobrać jako PDF – do druku, archiwizacji lub przekazania dalej. Formatowanie z edytora zostaje zachowane.',
        },
        papierkorb: {
          title: 'Kosz',
          p1: 'Usunięte notatki trafiają do kosza (w menu pod twoją nazwą, na telefonie w przycisku menu) i można je tam przywrócić lub usunąć trwale. Przy usuwaniu teczki decydujesz, czy zawarte notatki też zostaną usunięte, czy przeniesione do teczki nadrzędnej.',
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
          p1: 'W karcie „Pliki” tworzysz foldery i przesyłasz pliki dowolnego rodzaju – pojedynczo lub kilka naraz. Folderom i plikom można zmieniać nazwy i je przenosić, pliki ponownie pobierać; foldery możesz też udostępniać w projektach i pobierać jako ZIP.',
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
          p1: 'W karcie „Galeria” są twoje zdjęcia i filmy – w albumach z miniaturami albo, bez albumu, poniżej w „Zdjęcia”. Albumy można zagnieżdżać – np. jeden album na podróż z podalbumem na każdy dzień – i w każdej chwili przenosić. Przeglądarka przewija wszystkie zdjęcia albumu.',
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
          p1: 'Usunięte treści – dokumenty, pliki, zdjęcia, notatki, kontakty, ale też projekty, tablice, kalendarze i terminy – trafiają najpierw do wspólnego kosza i można je tam przywrócić. Po 30 dniach kosz opróżnia się automatycznie; dopiero wtedy miejsce jest ostatecznie zwolnione.',
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
        tagesansicht: {
          title: 'Widok dnia',
          p1: 'Kliknięcie numeru dnia otwiera widok dnia – przegląd zamiast siatki, pomyślany na pytanie z poprzedniego wieczoru: czego dziecko potrzebuje jutro? Na samej górze plan opieki mówi, u kogo jest dziecko („u ciebie” albo u kogo innego). Niżej po kolei lekcje z planu lekcji, przy każdej to, co jest na nią do zrobienia; sprawdziany i inne terminy najbliższych dni pojawiają się już zawczasu z „za … dni”. Strzałkami przechodzisz do poprzedniego lub następnego dnia.',
          p2: 'Przy każdej lekcji szybko dodajesz zadanie przez „Zadanie domowe”; trafia ono jako karta z przedmiotem na tablicę projektu. „Zeszyt” otwiera teczkę przedmiotu. To, co ma termin, ale nie należy do żadnej lekcji, stoi w „Bez lekcji”. Kliknięcie wpisu prowadzi do jego miejsca w projekcie.',
        },
        ausProjekten: {
          title: 'Z projektów',
          p1: 'W „Z projektów” na liście kalendarzy pokazujesz, co twoje projekty wnoszą do kalendarza: plan lekcji, opiekę i terminy – karty i kamienie milowe z terminem. Każde źródło można osobno pokazać i ukryć, a nowe plany pojawiają się same. Wpisy są tylko do odczytu; kliknięcie pokazuje szczegóły, a „Otwórz w projekcie” prowadzi tam, gdzie je zmieniasz.',
        },
        abos: {
          title: 'Subskrypcje kalendarzy',
          p1: 'Zewnętrzne kalendarze – np. święta albo terminarz meczów klubu – dołączasz adresem subskrypcji w formacie ICS. Zasubskrybowane kalendarze aktualizują się automatycznie. Ich terminy są w Openany tylko do odczytu: zmienia się je tam, skąd pochodzi kalendarz, bo każde odświeżenie zastępuje treść z powrotem oryginałem.',
          p2: 'W drugą stronę możesz udostępnić własne kalendarze linkiem subskrypcji: ikona kanału na liście kalendarzy tworzy tajny adres URL, który można zasubskrybować w Google, Apple lub Outlooku – zmiany docierają tam automatycznie. Traktuj link jak hasło; „Odnów link” natychmiast unieważnia stary.',
        },
        importExport: {
          title: 'Import i eksport',
          p1: 'Istniejące kalendarze importujesz jako plik ICS i tak samo eksportujesz swoje – pełna przenośność w obie strony, bez przywiązania do Openany.',
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
          p1: 'Każdy projekt ma wspólny czat w czasie rzeczywistym: wiadomości pojawiają się od razu u wszystkich członków, bez odświeżania strony.',
          p2: 'Ważne wiadomości może przypiąć każdy członek – przypięte wiadomości łatwo znaleźć na pasku nad historią i tam też je odpiąć.',
          p3: 'Dwoma nawiasami kwadratowymi odsyłasz w środku zdania do treści projektu: do notatek, tablic, map drogowych i zbiorów miejsc wraz z ich kartami, kamieniami milowymi i miejscami, a także do udostępnionych folderów, plików, albumów i zdjęć. Podczas pisania lista podpowiedzi pokazuje, co pasuje, i wstawia gotowy odnośnik. To samo robi przycisk z ikoną łańcucha obok przycisku wysyłania – na telefonie zwykle wygodniej niż szukanie nawiasów.',
          p4: 'Kliknięcie odnośnika prowadzi prosto do celu: do notatki, na tablicę z otwartą kartą, do folderu z wyróżnionym plikiem lub do zdjęcia w przeglądarce. Odsyłać można tylko do tego, co i tak jest widoczne w projekcie – czego nikt nie udostępnił, nie ma na liście podpowiedzi i odnośnik do tego nie jest klikalny.',
        },
        planung: {
          title: 'Karta Planowanie',
          p1: 'Na karcie „Planowanie” projekt zbiera narzędzia do planowania. Przyciskiem „Nowe” wybierasz odpowiedni typ: Ustalanie terminu, Ankieta, Grafik dyżurów, Lista „kto co przynosi”, Lista gości, Tablica, Mapa drogowa, Miejsca, Ferie szkolne, Plan lekcji lub Plan opieki. Szablon „Szablon: szkoła” tworzy naraz ferie szkolne, plan lekcji, sprawdziany i tablicę zadań domowych. Przy tym wybierasz swój land, żeby ferie od razu były wpisane, i opcjonalnie rozkład dzwonków swojej szkoły. Tworzyć i edytować może każdy członek; usuwać może twórca lub właściciel projektu.',
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
        boards: {
          title: 'Tablice Kanban',
          p1: 'Na tablicach Kanban porządkujecie zadania w dowolnych kolumnach i kartach – np. „Do zrobienia”, „W toku”, „Gotowe”. Karty przesuwa się przeciąganiem; zmiany widać na żywo u wszystkich członków. Karta może też mieć miejsce z kolekcji miejsc projektu i przedmiot – dzięki temu zadanie domowe należy do właściwej lekcji.',
        },
        roadmap: {
          title: 'Mapa drogowa',
          p1: 'Mapa drogowa pokazuje kamienie milowe z datą i statusem na osi czasu – cały zespół widzi, co ma być gotowe i kiedy. Kamienie milowe można oznaczyć jako osiągnięte, zaległe są wyróżnione. Podobnie jak karty, kamienie milowe mogą mieć miejsce i przedmiot, np. sprawdzian.',
        },
        orte: {
          title: 'Miejsca',
          p1: 'W Miejscach zbieracie punkty na OpenStreetMap – miejsca spotkań, adresy, parkingi. Punkt stawiasz kliknięciem na mapie, opcjonalnie z nazwą i notatką; kliknięcie nazwy kafelka miejsca otwiera punkt bezpośrednio w OpenStreetMap. Raz zebrane miejsca po prostu wybierasz gdzie indziej: przy kartach, kamieniach milowych, przedmiotach i wpisach w planie lekcji lub planie opieki.',
        },
        schuljahr: {
          title: 'Ferie szkolne',
          p1: 'Ferie szkolne opisują rok szkolny: jego okres i dni wolne. Ferie twojego kraju związkowego (Niemcy) można wczytać z pliku ICS, pojedyncze dni wolne i święta dopisujesz ręcznie. Różnica ma znaczenie: w ferie często obowiązuje inna organizacja opieki, a w pojedynczy wolny dzień opieka biegnie w zwykłym rytmie. Import tylko uzupełnia i nie zastępuje niczego, co wpisano ręcznie.',
        },
        wochenplan: {
          title: 'Plan lekcji i plan opieki',
          p1: 'Oba to ten sam element: tygodniowa siatka od poniedziałku do niedzieli, powtarzana tydzień po tygodniu. Plan lekcji zawiera godziny i przedmioty, plan opieki całe dni i osobę – „kto ma dziecko i kiedy”. Jeśli połączysz go z feriami szkolnymi, lekcje w ferie same wypadają; plan opieki można ograniczyć do czasu nauki albo do ferii. Jeśli wasz rytm zmienia się co tydzień, przełączasz na tygodnie A/B: albo co dwa tygodnie od poniedziałku tygodnia A, albo według parzystych i nieparzystych tygodni kalendarzowych, jeśli tak mówi wasze porozumienie – wtedy jednak rytm przeskakuje w latach z 53 tygodniami.',
          p2: 'Pod siatką są okresy. Zastępują ją w swoim przedziale czasu i obejmują to, co nie jest rytmem: wakacje podzielone na pół, zamieniony weekend, wycieczkę klasową. Strefa czasowa należy do planu, a nie do oglądającego – dzięki temu plan lekcji pokazuje obojgu rodzicom tę samą godzinę, nawet jeśli jedno mieszka za granicą.',
        },
        faecher: {
          title: 'Przedmioty',
          p1: 'Przedmioty projektu prowadzisz przez „Przedmioty” w planie lekcji: nazwa, skrót, nauczyciel, kolor i opcjonalnie miejsce. Należą do projektu, a nie do jednego planu, więc przetrwają zmianę semestru. Przedmiot decyduje, co pokazuje lekcja w planie, i łączy karty zadań domowych i sprawdziany z właściwą lekcją. Jeśli usuniesz przedmiot, karty, kamienie milowe i lekcje tracą tylko odwołanie do niego – same zostają.',
        },
        hefte: {
          title: 'Zeszyty do przedmiotów',
          p1: 'Dla każdego przedmiotu „Utwórz i udostępnij teczkę” tworzy teczkę notatek – zeszyt – i od razu udostępnia ją w projekcie. Teczka należy do ciebie, nie do projektu, i liczy się do twojego miejsca; dlatego przy przedmiocie widać, czyja to teczka. Projekt sam nie posiada treści: to, co w nim jest, udostępnili członkowie.',
        },
        planAbo: {
          title: 'Subskrypcja planu lekcji i opieki',
          p1: '„Subskrypcja” tworzy link, którym plan lekcji lub plan opieki można subskrybować w Google, Apple czy Thunderbirdzie – bez otwierania Openany. Link dotyczy typu, nie pojedynczego planu: jeśli opiekę w ferie prowadzicie jako drugi plan, jest w tej samej subskrypcji. Miejsca dołączasz tylko na wyraźne życzenie – link jest publiczny i nie wymaga logowania, a z miejscami kryłyby się za nim wasze adresy. Programy kalendarzowe pobierają subskrypcję tylko co kilka godzin; „Odnów” od razu unieważnia stary link, „Unieważnij” go wyłącza.',
        },
        freigaben: {
          title: 'Udostępnianie treści',
          p1: 'Na karcie „Udostępnienia” jest wszystko, co członkowie oddali do dyspozycji projektu: teczki notatek, albumy oraz foldery i segregatory z Dysku. Udostępnia się tam, gdzie treść jest u siebie – w Notatkach, w Galerii lub na Dysku – z poziomem „Tylko odczyt” albo „Edycja”. Udostępnienie dotyczy wszystkich członków projektu, dziedziczą je podteczki, podalbumy i podfoldery i można je w każdej chwili usunąć. Same treści pozostają u właściciela; gdy udostępnienie się kończy, wszystko mu zostaje.',
          p2: 'Kliknięcie udostępnienia otwiera je w projekcie: notatki w znanym obszarze roboczym z wyszukiwaniem, linkami zwrotnymi i grafem wszystkich udostępnionych notatek projektu, albumy jako siatkę zdjęć z przeglądarką, foldery jako listę plików. Z „Edycją” członkowie mogą też sami coś dodać – pisać notatki, przesyłać zdjęcia, zapisywać pliki i tworzyć podfoldery; „Tylko odczyt” pozwala oglądać i pobierać.',
        },
        projektNotizen: {
          title: 'Wspólne notatki projektu',
          p1: 'W teczce notatek udostępnionej z „Edycją” członkowie mogą bezpośrednio w projekcie tworzyć, edytować i usuwać notatki oraz zakładać podteczki; osadzone obrazy i pliki widzą wszyscy członkowie, a kto może edytować, może też sam je wstawiać. Wszystko, co powstanie – także przesłane obrazy i pliki – należy do właściciela teczki i liczy się do jego miejsca na dysku; usunięte notatki trafiają do jego kosza, więc nic nie przepada na zawsze.',
        },
        ki: {
          title: 'AI w projekcie',
          p1: 'Do projektu można podłączyć AI. Konfiguruje ją właściciel projektu na karcie „Członkowie” pod listą członków: dostawca, model i własny klucz API. Klucz jest zapisywany w postaci zaszyfrowanej i potem nikomu nie jest wyświetlany, także właścicielowi; koszty ponosi konto, do którego należy klucz. W tym samym miejscu wszyscy członkowie widzą, czy AI jest podłączona, do jakiego dostawcy i kto ją skonfigurował.',
          p2: 'Pytasz z poziomu udostępnionych treści: pod notatką przez „Zapytaj AI”, przy dokumentach w udostępnionych segregatorach i folderach (PDF, Word, OpenDocument i pliki tekstowe) oraz – jeśli model rozumie obrazy – przy zdjęciach w przeglądarce udostępnionego albumu. Piszesz polecenie, np. „Streść to w trzech zdaniach”, i dostajesz odpowiedź do skopiowania; kto może edytować notatkę, może też wstawić ją od razu na końcu.',
          p3: 'Ważne: to, o co pytasz, opuszcza Openany. Nad polem wpisu za każdym razem widać, co trafia do jakiego dostawcy – cały tekst notatki, tekst dokumentu lub zdjęcie, pomniejszone i bez miejsca wykonania. Zeskanowane pliki PDF bez warstwy tekstu wymagają najpierw rozpoznawania tekstu, bardzo długie dokumenty są odrzucane.',
        },
        benachrichtigungen: {
          title: 'Powiadamianie członków',
          p1: 'Przy nowych ankietach, ustalaniu terminów lub udostępnieniach możesz na życzenie powiadomić członków wiadomością bezpośrednią – dzięki temu nikt nie przegapi nowości w projekcie.',
        },
        lokal: {
          title: 'Projekty tylko na miejscu',
          p1: 'W aplikacji „+ Projekt” tworzy projekt tylko na tym urządzeniu – bez konta i bez serwera; ma oznaczenie „Tylko na miejscu”. Innych zapraszasz w „Zaproś członka w pobliżu”: na drugim urządzeniu Openany musi być otwarte, w tej samej sieci Wi-Fi lub hotspocie, a oba urządzenia pokazują te same sześć cyfr, które porównujecie i potwierdzacie. Projektem zarządzają urządzenia właściciela; jeśli jest tylko jedno, aplikacja ostrzega – sparuj wcześniej drugie własne urządzenie.',
          p2: 'Członkowie udostępniają w projekcie własne teczki notatek, foldery i albumy przez „Udostępnij coś”, najpierw tylko do odczytu. Teczki notatek można też udostępnić do edycji: wtedy notatkę edytuje naraz tylko jedna osoba, a zapis następuje na urządzeniu osoby, do której należy teczka – jeśli nie ma go w pobliżu, notatka zostaje tylko do odczytu. Czat dociera od razu do wszystkich w pobliżu, do pozostałych przy następnej synchronizacji. Tak samo podróżuje planowanie – tablice, mapy drogowe, miejsca i planowanie szkolne.',
          p3: 'Resztę synchronizujesz przez „Synchronizuj”, gdy tylko ktoś z członków jest w pobliżu. Właściciel może usuwać członków; kto chce odejść, robi to przez „Wystąp”, co działa tylko wtedy, gdy inny członek jest w pobliżu i może się o tym dowiedzieć. To, co udostępnili odchodzący, znika u pozostałych; to, co sami już mieli, zostaje na ich urządzeniach.',
        },
      },
    },
    messages: {
      title: 'Wiadomości',
      subs: {
        direkt: {
          title: 'Wiadomości bezpośrednie',
          p1: 'Do wiadomości dojdziesz na komputerze przez kopertę w nagłówku, na telefonie przez przycisk menu w lewym dolnym rogu. Przyciskiem „Wiadomość” w prawym górnym rogu piszesz bezpośrednio do każdego innego użytkownika – jako odbiorca wystarczy jego nazwa w Openany. Nowe wiadomości pojawiają się od razu, bez odświeżania strony, a linki w nich są klikalne.',
          p2: 'Pod każdą odebraną wiadomością „Odpowiedz” otwiera pole pisania z wpisanym już nadawcą, tą samą drogą, którą przyszła wiadomość. Długie wiadomości są zwinięte; „Czytaj dalej” pokazuje je w całości. Licznik nieprzeczytanych zawsze pokazuje, czy coś nowego czeka – na komputerze przy kopercie, na telefonie przy przycisku menu dolnego paska.',
        },
        suche: {
          title: 'Wyszukiwanie i filtry',
          p1: 'Nad historią jest pole wyszukiwania. Znajduje wiadomości po treści, po nazwie rozmówcy i po identyfikatorach Matrix oraz wyróżnia trafienia. Pod nim przełączniki zawężają listę: do nieprzeczytanych, do wiadomości z załącznikiem i – jeśli używasz więcej niż jednej drogi – do pojedynczych dróg, takich jak Openany czy Matrix. Przełączniki można łączyć.',
          p2: 'Obok pola wyszukiwania przełączasz się między dwoma widokami: „Historia” pokazuje wszystkie wiadomości według czasu, „Według kontaktu” zbiera je w jeden wiersz na rozmówcę, z ostatnią wiadomością i liczbą nieprzeczytanych. Kliknięcie wiersza pokazuje historię właśnie z tą osobą; „Zamknij rozmowę” wraca do wszystkich.',
        },
        anhaenge: {
          title: 'Załączniki',
          p1: 'Przez Matrix wysyłasz też pliki: spinacz w polu pisania dołącza plik do 10 MB, a na telefonie i tablecie przycisk aparatu obok od razu robi zdjęcie. W historii obrazy pojawiają się jako podgląd, inne pliki z nazwą i rozmiarem; kliknięcie otwiera obrazy i PDF-y bezpośrednio w Openany, resztę pobierasz. Drogą Openany idą tylko teksty.',
        },
        matrix: {
          title: 'Wiadomości przez Matrix',
          p1: 'Do osób bez konta Openany dotrzesz przez Matrix. W tym celu w ustawieniach w „Konto Matrix” łączysz istniejące konto na matrix.org lub innym homeserverze – założyć je trzeba tam. Hasło służy tylko do zalogowania i nie jest zapisywane. Openany loguje się przy tym jako osobne urządzenie i od tej chwili może czytać wszystkie nowe wiadomości tego konta, także te pisane w innym programie Matrix; na zewnątrz wszystko pozostaje szyfrowane.',
          p2: 'Gdy konto jest połączone, przy pisaniu wybierasz drogę: Openany lub Matrix, wtedy z identyfikatorem Matrix jako odbiorcą. Odpowiedzi trafiają do tej samej historii i są oznaczone „Matrix”. Jeśli usuniesz wiadomość Matrix, zniknie tylko w Openany; w pokoju zostaje. „Rozłącz” wylogowuje urządzenie; już odebrane wiadomości zostają.',
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
          p1: 'Zdarzenia z twoich projektów – np. nowe ankiety lub udostępnienia – docierają do ciebie jako automatyczna wiadomość bezpośrednia, jeśli nadawca zdecyduje się powiadomić członków. Tutaj znajdziesz też odpowiedź na wiadomość z formularza kontaktowego.',
        },
      },
    },
    contacts: {
      title: 'Książka adresowa',
      subs: {
        adressbuch: {
          title: 'Kontakty i ich kanały',
          p1: 'Książkę adresową otwierasz u góry strony wiadomości; przycisk obok od razu tworzy nowy kontakt. Kontakt zapisuje, jak można się z kimś skontaktować: nazwę, zdjęcie, telefon i e-mail oraz dowolnie wiele innych sposobów – Matrix, Meshtastic, nazwę w Openany, adres, firmę, urodziny lub inne, każdy z własną etykietą, np. prywatny lub praca. Pole wyszukiwania znajduje kontakty po nazwie.',
          p2: 'Sposoby kontaktu są klikalne: numer telefonu dzwoni, adres e-mail otwiera program pocztowy, nazwa w Openany lub identyfikator Matrix otwiera pole pisania z wpisanym już odbiorcą. Kontakt może być powiązany z kontem Openany – to tylko odnośnik i nie daje dostępu do projektów ani treści. Usunięte kontakty trafiają do kosza.',
        },
        adressbuchDatei: {
          title: 'Zapisywanie i przejmowanie książki adresowej',
          p1: 'W ustawieniach, w „Książka adresowa”, pobierasz wszystkie kontakty jako plik vCard lub wczytujesz plik vCard z innego programu. Wczytywanie tylko dodaje: istniejące kontakty są rozpoznawane po identyfikatorze lub nazwie i zachowują to, co mają. Wszystkie dane podróżują w standardowych polach; przepaść mogą co najwyżej własne etykiety, bo wiele książek adresowych zna tylko „prywatny” i „praca”.',
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
          p1: 'Programy na twoim komputerze nie logują się hasłem do konta, tylko osobnym hasłem aplikacji. Tworząc je, nadajesz mu nazwę i wybierasz cel: „Programy” dla WebDAV, Zettlra lub menedżera plików, które czytają i zapisują twoje notatki, albo „Synchronizacja” dla programów, które odzwierciedlają konto i zapisują zmiany z powrotem. Utworzyć je można tylko po zalogowaniu z drugim składnikiem.',
          p2: 'Nowe hasło jest wyświetlane dokładnie raz – skopiuj je od razu. W programie wpisujesz nazwę w Openany jako nazwę użytkownika, adres WebDAV jest we wskazówce pod hasłem. Lista pokazuje, kiedy każde hasło było ostatnio użyte; jeśli któreś unieważnisz, program natychmiast traci dostęp.',
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
          p1: 'Pytania, problemy lub życzenia? Link „Kontakt” w stopce prowadzi do formularza kontaktowego. Twoja wiadomość trafia jako wiadomość bezpośrednia do administratorów, a odpowiedź znajdziesz później w Wiadomościach.',
        },
        bedingungen: {
          title: 'Warunki korzystania',
          p1: 'Warunki korzystania z wersji beta i notę prawną znajdziesz zawsze pod linkami w stopce. W skrócie: w czasie bety Openany jest bezpłatny i bez reklam, a twoje dane leżą na serwerach w Niemczech. Wychodzi tylko to, co sam wysyłasz – np. wiadomości przez Matrix lub zapytania do AI projektu.',
        },
      },
    },
  },
};
