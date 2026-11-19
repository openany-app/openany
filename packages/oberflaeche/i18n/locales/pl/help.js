// Uwaga vue-i18n: w tekstach bez surowych @, | ani {.
export default {
  title: 'Pomoc',
  intro: 'Tutaj znajdziesz opisy i instrukcje do wszystkich modułów i funkcji Openany. Spis treści po lewej prowadzi prosto do właściwej sekcji.',
  version: 'Wersja {version}',
  aktualisieren: 'Zaktualizuj aplikację',
  aktualisierenTitel: 'openany.de wymaga co najmniej wersji {version}.',
  toc: 'Spis treści',
  sections: {
    start: {
      title: 'Pierwsze kroki',
      subs: {
        ohneKonto: {
          title: 'Aplikacja bez konta',
          p1: 'Aplikacja to pełny program, który działa także bez konta i bez serwera: notatki, kalendarz, dysk, książka adresowa i wiadomości są wtedy tylko na tym urządzeniu. To, czego tu nie skonfigurowano albo co nie działa, aplikacja ukrywa, zamiast pokazywać puste – droga Openany w wiadomościach pojawia się na przykład dopiero wtedy, gdy urządzenie jest połączone z openany.de.',
          p2: 'Jeśli chcesz synchronizować dane z openany.de, załóż konto w ustawieniach w „Synchronizacja” albo połącz się z istniejącym; z innymi urządzeniami parujesz się w „Urządzenia w pobliżu”. Potem w nagłówku i w menu pojawia się przycisk „Synchronizuj”, który się obraca podczas synchronizacji.',
        },
        kontoAnlegen: {
          title: 'Zakładanie konta',
          p1: 'Konto w openany.de zakładasz bezpośrednio w aplikacji: w ustawieniach w „Synchronizacja” stuknij „Utwórz nowe konto” i wybierz nazwę. Potem urządzenie rozwiązuje małe zadanie obliczeniowe, które chroni przed masowym zakładaniem kont; zwykle trwa to krócej niż minutę. Nie potrzebujesz adresu e-mail ani hasła.',
          p2: 'Następnie akceptujesz warunki korzystania – dopiero wtedy masz miejsce w openany.de, na start 100 MB. Kolejne urządzenia łączysz przez już połączone urządzenie za pomocą 6 cyfr (zobacz „Moje urządzenia”).',
          p3: 'Ponieważ twoje konto nie ma hasła, plik kopii zapasowej jest jedyną drogą powrotu, jeśli zgubisz jedyne urządzenie. Aplikacja proponuje go zaraz po założeniu konta; utwórz go i przechowuj bezpiecznie razem z hasłem (zobacz „Kopia zapasowa w jednym pliku”).',
        },
        startseite: {
          title: 'Strona startowa',
          p1: 'Strona startowa pokazuje zwięzłe kafelki wybranych modułów: dzisiejsze terminy, książkę adresową z wyszukiwaniem, trzy ostatnio edytowane notatki, trzy ostatnie pliki z zajętym miejscem i twoje huby. Każdy kafelek prowadzi do swojego modułu; które się pojawiają, ustalasz w ustawieniach w „Moduły na stronie startowej”.',
          p2: 'Na początku wybrane są wszystkie kafelki. Jeśli odznaczysz wszystkie, pojawi się ekran powitalny: przedstawia wszystkie funkcje i pozwala od razu złożyć kafelki. Wybór niczego nie ukrywa – wszystkie obszary pozostają dostępne przez pasek i menu.',
        },
        navigation: {
          title: 'Menu i układ',
          p1: 'Na komputerze u góry jest nagłówek: po lewej okrągłe logo prowadzi do strony startowej, obok są Notatki, Kalendarz, Dysk i Huby. Po prawej znajdują się przełącznik jasny/ciemny, koperta wiadomości z licznikiem nieprzeczytanych i pole z twoją nazwą – otwiera Ustawienia, Pomoc, Kosz i Wyloguj.',
          p2: 'Na telefonie wszystko przesuwa się na dół: stały pasek pokazuje moduły jako ikony, a skrajnie po lewej jest przycisk menu z licznikiem nieprzeczytanych wiadomości. Otwiera panel z Wiadomościami, Ustawieniami, Pomocą, Koszem, przełącznikiem jasny/ciemny i wylogowaniem. Przełącznik zmienia tryb między jasnym, ciemnym i automatycznym – automatyczny podąża za ustawieniem urządzenia. Dotknięcie już otwartego modułu prowadzi do jego punktu startowego.',
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
          p1: 'Notatki leżą w teczkach, które można dowolnie zagnieżdżać. Notatkom i teczkom możesz zmieniać nazwy, przenosić je i porządkować – tak z czasem rośnie własny zbiór wiedzy. Lupa u góry otwiera pole wyszukiwania, które przeszukuje wszystkie notatki naraz, tytuł i treść; otwarte w teczce, „tylko w tej teczce” ogranicza je do niej. Co można zrobić z otwartą teczką, znajdziesz w „…” po prawej stronie paska ścieżki.',
          p2: 'W górę przechodzisz przez ścieżkę albo przyciskiem Wstecz, który cofa o jeden poziom; krzyżyk na pasku ścieżki zamyka otwartą notatkę. Obok każdej teczki widać, kiedy ostatnio zmieniła się w niej notatka, także w podteczkach. Teczka bez notatek nie pokazuje daty.',
        },
        wikilinks: {
          title: 'Wikilinki i linki zwrotne',
          p1: 'Dwoma nawiasami kwadratowymi łączysz notatki ze sobą: wpisz nawiasy i tytuł notatki docelowej, a link powstanie automatycznie – jeśli notatka jeszcze nie istnieje, link zadziała, gdy tylko się pojawi. Jeśli w tekście ma stać coś innego niż tytuł, podajesz własny tekst po pionowej kresce; link i tak prowadzi do właściwej notatki.',
          p2: 'Każda notatka pokazuje swoje linki zwrotne: listę wszystkich notatek, które do niej prowadzą. Tak odnajdziesz powiązania bez prowadzenia własnych zapisków.',
        },
        tags: {
          title: 'Tagi',
          p1: 'Słowa kluczowe nadajesz bezpośrednio w tekście krzyżykiem przed słowem; przy pisaniu pomaga lista już użytych tagów. Ukośnikiem zagnieżdżasz słowa kluczowe – np. Hub/Podtemat; filtr na temat główny znajdzie wtedy także wszystkie podtematy. W widoku grafu słowa kluczowe można pokazać jako osobne węzły, a kliknięcie słowa kluczowego filtruje tam listę notatek.',
        },
        einfuegen: {
          title: 'Wstawianie obrazów i plików',
          p1: 'Przycisk spinacza na pasku narzędzi wstawia do notatki plik z twojego urządzenia. Obrazy pojawiają się jako zwięzły podgląd w samej notatce (pełna rozdzielczość jest w Mediatece), a inne pliki jako link do otwarcia.',
          p2: 'Wstawione treści są automatycznie porządkowane: obrazy do albumu w Mediatece, pliki tekstowe i Office do segregatora w Dokumentach, cała reszta do folderu plików – zawsze w osobnym obszarze dla notatek.',
        },
        zitate: {
          title: 'Cytowania i bibliografia',
          p1: 'Do pracy naukowej każda teczka ma listę źródeł: w otwartej teczce wybierz „Źródła” w „…” i wpisz książki, artykuły, strony, orzeczenia lub ustawy. Każde źródło dostaje skrót. Lista to plik CSL-JSON w Plikach, działa więc także z Zotero i Zettlr.',
          p2: 'W notatkach tej teczki edytor podpowiada przy cytowaniu pasujące źródła i wstawia cytat do tekstu. Dzięki temu cytaty powstają w trakcie pisania, bez każdorazowego przepisywania danych. O ich wyglądzie w PDF przeczytasz w „Wersja do druku”.',
        },
        graph: {
          title: 'Widok grafu',
          p1: 'Widok grafu rysuje twoje notatki jako sieć: każda notatka to punkt, każdy wikilink to połączenie. Punkty są pokolorowane według najwyższej teczki, więc powiązane obszary widać już po kolorze; puste punkty to notatki bez żadnych połączeń, a im więcej powiązań ma notatka, tym większy jej punkt.',
          p2: 'Każda teczka najwyższego poziomu to poziom. Jeśli jest ich kilka, wybierasz jeden lub więcej w legendzie pod grafem – „Wszystkie poziomy” bierze wszystkie, „Żaden” czyści wybór. Zamiast poziomu wejściem może być też tag: pasek poniżej wymienia tagi z ich liczbą, a wybrany tag pokazuje wszystkie notatki, które go mają, razem z tagiem jako szarym rombem. Gdy wybrane są poziomy, pasek wymienia już tylko tagi, które w nich występują. Dopóki niczego nie wybierzesz, graf pozostaje pusty, żeby duże zbiory nie budowały się naraz; „Pokaż wszystkie tagi” dodaje od razu wszystkie tagi z paska.',
          p3: 'Na komputerze powiększasz przyciskami plus/minus w prawym górnym rogu lub kółkiem myszy i przesuwasz widok z wciśniętym przyciskiem myszy; na telefonie powiększasz dwoma palcami i przesuwasz jednym. Kolejny przycisk resetuje widok. Kliknięcie punktu (bez przeciągania) otwiera notatkę. Kliknięcie tagu filtruje według niego listę notatek.',
        },
        export: {
          title: 'Eksport do PDF',
          p1: 'Przycisk PDF w otwartej notatce oferuje cztery wersje: „Standardowy” to notatka z tytułem, datą i bibliografią. „Clean” to sam tekst, bez tytułu, daty, numerów stron i bibliografii; cytaty zostają tak, jak je napisano. „List” składa notatkę według DIN 5008 do koperty z okienkiem, z tytułem jako tematem. „Wersja do druku” tworzy tylko tę notatkę ze stroną tytułową. PDF trafia do teczki „Notizen” w Pamięć → Dokumenty (wcześniejsze wersje zostają obok), a wersja do druku do teczki „Druckfassungen”; „Otwórz” w komunikacie od razu go pokazuje.',
        },
        druckfassung: {
          title: 'Wersja do druku',
          p1: 'Folder staje się formalnym dokumentem do oddania lub wydruku: w otwartym folderze wybierz w „…” pozycję „Wersja do druku”, wpisz tytuł, podtytuł, imię i nazwisko, instytucję, datę i dodatkowe informacje na stronę tytułową oraz zdecyduj, czy dodać spis treści. Każda notatka z folderu staje się rozdziałem, alfabetycznie według tytułu; przedrostkami, np. „01 Wstęp”, ustalasz kolejność, a w dokumencie pojawia się tylko „Wstęp”. Podfoldery są uwzględniane, gdy zaznaczono „Uwzględnij podfoldery”: podfolder staje się nagłówkiem, a jego notatki trafiają pod niego („2 Podstawy”, „2.1 Pojęcia”); notatka o tej samej nazwie co podfolder pojawia się jako tekst bezpośrednio pod tym nagłówkiem.',
          p2: 'Skład wykorzystuje krój podobny do Times z bezszeryfowymi nagłówkami, marginesy 2,5 cm, interlinię 1,5, wyjustowany tekst z dzieleniem wyrazów; numeracja stron zaczyna się po stronie tytułowej. Dane pozostają zapisane przy folderze. PDF powstaje na twoim urządzeniu – za pierwszym razem trwa to kilka sekund – i trafia do teczki „Druckfassungen” w Pamięć → Dokumenty; każda wersja zostaje zachowana.',
          p3: "Cytaty z listy źródeł teczki: w tekście piszesz [{'@'}skrot] lub [{'@'}skrot, s. 12], a ^[Tekst] dla przypisu. W PDF – notatki lub wersji do druku – powstają z tego cytaty harwardzkie, np. „(Müller, 2020, s. 12)”, a na końcu bibliografia.Pasek narzędzi edytora ma przycisk przypisu (cyfra w indeksie górnym): wstawia nawiasy, a zaznaczony tekst staje się przypisem.",
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
          p1: 'W aplikacji na komputerze możesz połączyć swój segregator z folderem: otwórz segregator, wybierz w menu „…” pozycję „Połącz z folderem …” i wskaż folder. Każda notatka leży tam jako plik „Id Tytuł.md”, podsegregatory jako podfoldery – tak, jak oczekują tego Zettlr czy Obsidian. To, co tam zmienisz, przemianujesz, przeniesiesz lub utworzysz, aplikacja przejmuje, dopóki działa; jeśli zmieniły się obie strony, powstaje kopia konfliktu. Gdy usuniesz plik, notatka trafia do kosza. Folder leży na komputerze niezaszyfrowany; „Odłącz folder” zostawia pliki na miejscu. Obrazy z notatek leżą obok notatki, pod nazwą z tekstu; nowy obraz, do którego odsyła notatka, trafia do aplikacji (mediateka, album „Notizen: …”).',
        },
      },
    },
    files: {
      title: 'Dysk',
      subs: {
        aufbau: {
          title: 'Mediateka, pliki, dokumenty',
          p1: 'Dysk dzieli się na trzy karty u góry strony: Mediatekę na zdjęcia i filmy, Pliki na wszystko inne i Dokumenty na pisma w segregatorach. Gdy zmieniasz kartę, drugi obszar zostaje taki, jak go zostawiłeś – otwarte foldery i wczytane listy nie znikają.',
          p2: 'Która karta otwiera się jako pierwsza, gdy wchodzisz na Dysk, ustawiasz w ustawieniach w „Start dysku”; dopóki nic nie wybierzesz, są to Dokumenty.',
        },
        dokumente: {
          title: 'Archiwum dokumentów',
          p1: 'W karcie „Dokumenty” porządkujesz pliki PDF, Office i tekstowe w segregatorach. Przesyłasz przyciskiem lub po prostu przeciągając pliki do otwartego segregatora. Na telefonie i tablecie obok jest przycisk aparatu: fotografuje pismo i od razu zapisuje je jako dokument w segregatorze.',
          p2: 'Segregatory można dowolnie zagnieżdżać. Segregatorom i dokumentom możesz zmieniać nazwy i przenosić je; całe segregatory można też udostępnić hubowi w „Udostępnienia”. Obok każdego segregatora widać, kiedy ostatnio coś się w nim zmieniło, także w podsegregatorach.',
        },
        texterkennung: {
          title: 'Przeszukiwalne dokumenty',
          p1: 'Jeśli sfotografujesz pismo i zapiszesz je w segregatorze – przyciskiem aparatu albo jako zdjęcie ze swojej kolekcji – automatycznie powstanie PDF z niewidoczną warstwą tekstu. Dokument wygląda jak twoje zdjęcie, ale da się go przeszukiwać, zaznaczać, kopiować i odczytywać na głos. Rozpoznawanie działa na twoim urządzeniu – obraz nigdzie nie jest wysyłany.',
          p2: 'Jeśli zapisujesz kilka zdjęć naraz, pojawi się pytanie, czy mają tworzyć jeden dokument wielostronicowy – np. strony listu – czy osobne. Samo zdjęcie nie jest dodatkowo zapisywane; kto chce zachować obraz, zapisuje go w Mediatece. Za pierwszym razem konwersja trwa dłużej, bo rozpoznawanie tekstu wczytuje się jednorazowo i zostaje zapisane.',
          p3: 'Jeśli w segregatorze jest już zeskanowany PDF, możesz zastosować rozpoznawanie później: „Rozpoznaj tekst” czyta dokument strona po stronie. Jeśli papier jest czysto biały, strony pozostają bez zmian i dochodzi tylko tekst. Jeśli są szare lub nierówno oświetlone – jak sfotografowana kartka – zostaną dodatkowo rozjaśnione; komunikat na końcu powie, co się stało. Jeśli PDF był już przeszukiwalny, pozostaje nietknięty.',
        },
        dateien: {
          title: 'Zarządzanie plikami',
          p1: 'W karcie „Pliki” tworzysz foldery i przesyłasz pliki dowolnego rodzaju – pojedynczo lub po kilka naraz. Dotknięcie pliku otwiera go w odpowiedniej aplikacji na urządzeniu. Ikona pobierania zapisuje kopię w folderze Download, a na komputerze w wybranym miejscu; przy PDF-ach jest w przeglądarce. Foldery i pliki można przenosić i zmieniać im nazwy; foldery można też udostępnić hubowi w „Udostępnienia”. Co można zrobić z otwartym folderem lub segregatorem, znajdziesz w „…” po prawej stronie paska ścieżki. W górę przechodzisz przez ścieżkę albo przyciskiem Wstecz, który cofa o jeden poziom.',
          p2: 'Ile zajmujesz, widać na kafelku strony startowej. Ile z tego zostaje na tym urządzeniu, ustawiasz w ustawieniach w „Pamięć na tym urządzeniu”.',
        },
        suche: {
          title: 'Wyszukiwanie',
          p1: 'Lupa w Plikach i Dokumentach otwiera pole wyszukiwania, które przeszukuje wszystkie foldery lub segregatory naraz, a nie tylko ten otwarty. Kliknięcie wyniku otwiera plik; obok widać, gdzie leży, a kliknięcie tej ścieżki przenosi cię do folderu.',
          p2: 'Wyszukiwanie znajduje nie tylko po nazwie, ale też w treści: w PDF-ach, tekstach Word i OpenDocument oraz plikach tekstowych, CSV i Markdown do 50 MB. Openany czyta ten tekst stopniowo w tle, dopóki Dysk jest otwarty; zanim przeczyta wszystko, wyszukiwanie mówi, ile plików na razie znajdzie tylko po nazwie. Przy trafieniu w treści lista pokazuje fragment z wyróżnionym szukanym słowem. Zeskanowane PDF-y bez warstwy tekstu pojawią się dopiero po zastosowaniu „Rozpoznaj tekst”.',
        },
        pdf: {
          title: 'Przeglądanie i edycja PDF',
          p1: 'Kliknięcie PDF-a otwiera go bezpośrednio w Openany. Przewracasz strony, powiększasz lub pomniejszasz, dopasowujesz widok do szerokości i przeszukujesz dokument lupą – trafienia są wyróżnione na stronie. „Otwórz w innej aplikacji” przekazuje go do innej aplikacji. Jeśli PDF jest chroniony hasłem, Openany najpierw o nie pyta.',
          p2: 'Formularze w PDF-ie wypełniasz bezpośrednio. Przycisk „Edytuj” pokazuje ponadto pasek narzędzi: „Zakreślacz” wyróżnia tekst kolorem, „Pióro” pozwala swobodnie rysować na stronie, a „Tekst” umieszcza na kartce własne linijki – kolor, grubość linii i rozmiar czcionki wybierasz sam, a każdy krok można cofnąć. Pola obliczeniowe w formularzach jednak nie liczą, bo Openany nie uruchamia skryptów z cudzych plików.',
          p3: 'Do zakreśleń i linii możesz dodawać komentarze: wybierz „Komentarze” i dotknij miejsca; ten sam widok wyświetla wszystkie komentarze dokumentu. Wszystko zapisuje się jako zwykłe adnotacje PDF, które pokazują też inne programy. Przy pierwszym zapisie Openany przenosi poprzednią wersję do kosza, więc możesz wrócić do oryginału; jeśli opuszczasz PDF z niezapisanymi zmianami, najpierw zapyta.',
        },
        galerie: {
          title: 'Mediateka i albumy',
          p1: 'W karcie „Mediateka” są twoje zdjęcia i filmy – w albumach z miniaturami albo, bez albumu, poniżej w „Multimedia”. Albumy można zagnieżdżać – np. jeden album na podróż z podalbumem na każdy dzień – i w każdej chwili przenosić. Przeglądarka przewija wszystkie zdjęcia albumu. Zdjęcie przenosisz w dużym widoku; przytrzymanie w siatce pozwala wybrać kilka i razem je przenieść lub usunąć. Co można zrobić z otwartym albumem, znajdziesz w „…” po prawej stronie paska ścieżki.',
          p2: 'Najnowsze ujęcie jest na górze, pogrupowane według miesięcy z nagłówkiem dla każdego miesiąca. Liczy się data wykonania z danych zdjęcia; jeśli jej brak – np. przy zrzutach ekranu lub edytowanych obrazach – liczy się dzień przesłania.',
          p3: 'W dużym widoku pobierasz zdjęcie lub film jako oryginalny plik: na telefonie do folderu Download, na komputerze w wybrane miejsce. Albumy można udostępnić hubowi w „Udostępnienia”. Oryginały zostają u ciebie. W „Akcjach” albumu prześlesz własną okładkę; nie liczy się jako element.',
        },
        fotosVomGeraet: {
          title: "Zdjęcia z urządzenia",
          p1: "Na Androidzie w mediatece „Dodaj” → „Dodaj zdjęcia z urządzenia” przejmuje zdjęcia i filmy, które już są na urządzeniu: wszystkie, tylko wybrane albumy urządzenia albo od wybranej daty. Albumy urządzenia stają się albumami o tej samej nazwie w openany; wcześniej widać, czy starczy miejsca, a to, co już przejęto, zostaje pominięte. Jeśli urządzenie jest połączone z openany.de, raz wybierasz, jak trafiają zdjęcia: na openany.de – wtedy na urządzeniu zostaje tylko podgląd i nic nie zajmuje miejsca podwójnie – albo jako kopia na tym urządzeniu.",
          p2: "Dzięki „Dodawaj nowe zdjęcia automatycznie” – w mediatece pod „Dodaj” albo w Ustawienia → Pamięć i kopia zapasowa – to, co aparat zrobi od teraz, samo trafia do mediateki, nawet gdy openany nie jest otwarte: z wybranych albumów urządzenia, od chwili włączenia. Na życzenie tylko przez Wi-Fi albo tylko podczas ładowania. Android ogranicza pracę w tle, więc zdjęcia przychodzą z pewnym opóźnieniem. Na openany.de liczą się do twojej pamięci. Jeśli używasz Zdjęć Google, wyłącz tam kopię zapasową – inaczej obie aplikacje wysyłają.",
        },
        videos: {
          title: 'Filmy i aparat',
          p1: 'Oprócz zdjęć Mediateka przyjmuje też filmy. Pojawiają się jako kafelek z kadrem; dotknięcie otwiera je w aplikacji wideo na urządzeniu.',
          p2: 'Na telefonie i tablecie Mediateka i każdy album mają dwa przyciski aparatu: jeden do zdjęcia, drugi do filmu. Nagranie trafia od razu tam, gdzie jesteś. Na komputerze tych przycisków nie ma; przesyłasz jak zwykle.',
        },
        ton: {
          title: 'Nagrania dźwięku i muzyka',
          p1: 'Mediateka przechowuje też pliki dźwiękowe, np. MP3 lub M4A. Pojawiają się jako kafelek z nazwą i czasem trwania; dotknięcie odtwarza je w aplikacji. Pliki dźwiękowe z zewnątrz też tu trafiają.',
          p2: 'Przycisk mikrofonu nagrywa bezpośrednio, z pauzą, odrzuceniem i zapisem. Nagranie trafia do otwartego albumu, a w innym wypadku do albumu „Nagrania”. Na Androidzie działa też przy wyłączonym ekranie, co pokazuje powiadomienie. Na Linuksie tego przycisku nie ma.',
          p3: 'Na Androidzie (od wersji 10) i Windows możesz zamiast mikrofonu wybrać dźwięk systemowy. System za każdym razem prosi o zgodę. Niektóre aplikacje to blokują, a rozmów telefonicznych nie da się nagrać. Rozmowy nagrywaj tylko za zgodą wszystkich uczestników.',
        },
        karte: {
          title: 'Data i miejsce wykonania',
          p1: 'Przy każdym zdjęciu przeglądarka pokazuje datę wykonania i – jeśli zdjęcie ją zawiera – lokalizację; „Pokaż na mapie” otwiera to miejsce w OpenStreetMap. Gdy przesyłasz zdjęcia z telefonu z Androidem, wybieraj je przez aplikację Pliki, a nie przez galerię: inaczej Android usunie lokalizację, zanim zdjęcie dotrze do Openany.',
        },
        papierkorbSpeicher: {
          title: 'Kosz',
          p1: 'Usunięte treści – dokumenty, pliki, zdjęcia, notatki, kontakty, ale też huby, kalendarze i terminy – trafiają najpierw do wspólnego kosza i można je tam przywrócić. Po 30 dniach kosz opróżnia się automatycznie; dopiero wtedy miejsce jest ostatecznie zwolnione.',
        },
        dateienVerschluesselt: {
          title: 'Szyfrowanie end-to-end',
          p1: 'Twoje pliki, akta, zdjęcia i albumy są przechowywane na openany.de wyłącznie w postaci zaszyfrowanej: nazwy, foldery, albumy, daty wykonania i miejsce są w zaszyfrowanym nagłówku, a same treści są szyfrowane osobnym kluczem dla każdego pliku. Odczytać je mogą tylko twoje zatwierdzone urządzenia.',
        },
      },
    },
    calendar: {
      title: 'Kalendarz',
      subs: {
        verwalten: {
          title: 'Kilka kalendarzy',
          p1: 'Możesz prowadzić obok siebie dowolnie wiele kalendarzy – np. prywatny, rodzinny, klubowy – każdy z własnym kolorem. Kliknięcie pokazuje lub ukrywa kalendarz. U góry przełączasz między miesiącem, tygodniem i dniem; tydzień i dzień pokazują terminy według godzin, a minione są dyskretnie wyszarzone. Na telefonie i tablecie przewracasz strony przesunięciem palca: w miesiącu w górę i w dół, w tygodniu i dniu na boki. Na komputerze służą do tego strzałki.',
        },
        termine: {
          title: 'Dodawanie terminów',
          p1: 'Termin tworzysz kliknięciem w dzień: z tytułem, opisem, godziną albo jako wydarzenie całodniowe lub wielodniowe. Możliwe są też powtórzenia – codzienne, cotygodniowe, comiesięczne lub coroczne. Kliknięcie terminu pokazuje szczegóły, gdzie go edytujesz lub usuwasz.',
          p2: "Przypomnienie odzywa się o wybranym czasie przed początkiem. Jeśli termin przychodzi z innego programu z dokładniejszym powtarzaniem – co dwa tygodnie, tylko trzy razy, we wtorki i czwartki, w każdy pierwszy poniedziałek – zostaje ono zachowane i opisane słowami przy terminie. Jeśli sam wybierzesz inne powtarzanie, zastąpi ono stare. Termin serii przesunięty osobno pozostaje osobnym terminem, a seria zostawia jego dawne miejsce wolne.",
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
        imTelefon: {
          title: "W kalendarzu telefonu",
          p1: "Na Androidzie Ustawienia → Kalendarz → „Pokazuj w kalendarzu telefonu” pokazuje twoje kalendarze w każdej aplikacji kalendarza w telefonie, na koncie „openany” – także udostępnione w hubach, a subskrypcje, jeśli je dodasz. To, co tam zmienisz, utworzysz lub usuniesz, wraca do openany. Przypomnienia pokazuje wtedy sam telefon.",
          p2: "Kalendarz Google pokazuje inne konta dopiero po jednym kroku z twojej strony: tam Ustawienia → Zarządzaj kontami → włącz „openany”. Terminy są odszyfrowane tylko na tym urządzeniu. Wyłączenie usuwa je z kalendarza telefonu; w openany wszystko zostaje.",
        },
        andereProgramme: {
          title: "Dla innych programów na komputerze",
          p1: "Na komputerze Ustawienia → Kalendarz → „Udostępnij innym programom na tym komputerze” udostępniają twoje kalendarze i książkę adresową przez CalDAV i CardDAV – dla Thunderbirda, Kalendarza i Kontaktów Apple i innych. Kafelek pokazuje adres, użytkownika i hasło do skopiowania; z nimi zakładasz konto w programie. W Thunderbirdzie: Nowy kalendarz → W sieci albo Nowa książka CardDAV. U Apple: Inne konto CalDAV lub CardDAV → Ręcznie. Jeśli z komputera korzysta kilka kont użytkowników, włączaj to tylko wtedy, gdy tego potrzebujesz: gdy Openany nie działa, inne konto mogłoby odpowiedzieć zamiast niego i przejąć hasło wysyłane przez twój program.",
          p2: "Usługa jest dostępna tylko na tym komputerze i tylko wtedy, gdy openany działa; gdy aplikacja jest zamknięta, program nadrabia później. Zmiany wprowadzone tam wracają do openany; kalendarze udostępnione w hubach są tam tylko do odczytu. „Nowe hasło” blokuje skonfigurowane programy, dopóki nie dostaną nowego.",
        },
      },
    },
    projects: {
      title: 'Huby',
      subs: {
        grundlagen: {
          title: 'Huby i członkowie',
          p1: 'Hub to wspólna przestrzeń robocza: zapraszasz innych użytkowników jako członków, a zaproszeni przyjmują lub odrzucają zaproszenie. Właściciel hubu zarządza członkami i może też przekazać hub innemu członkowi.',
          p2: 'Każdy hub ma cztery karty: Udostępnienia, Board, Członkowie i Czat. Otwiera się na Udostępnieniach. Liczby na kartach pokazują nieprzeczytane wiadomości czatu lub ile w nich jest.',
        },
        chat: {
          title: 'Czat hubu',
          p1: 'Każdy hub ma wspólny czat w czasie rzeczywistym: wiadomości pojawiają się od razu u wszystkich członków, bez odświeżania strony. Czat jest szyfrowany end-to-end: czytają go tylko członkowie na swoich urządzeniach, openany.de nie. Gdy ktoś odchodzi, klucz zmienia się sam – nie czyta już niczego nowego.',
          p2: 'Ważne wiadomości może przypiąć każdy członek – przypięte wiadomości łatwo znaleźć na pasku nad historią i tam też je odpiąć.',
          p3: 'Dwoma nawiasami kwadratowymi odsyłasz w środku zdania do zawartości hubu: do notatek oraz do udostępnionych folderów, plików, albumów i zdjęć. Podczas pisania lista podpowiedzi pokazuje, co pasuje, i wstawia gotowy odnośnik. To samo robi przycisk z symbolem łańcucha obok przycisku wysyłania – na telefonie zwykle wygodniejszy niż szukanie dwóch nawiasów.',
          p4: 'Kliknięcie odnośnika prowadzi prosto do celu: do notatki, do folderu z wyróżnionym plikiem albo do zdjęcia w podglądzie. Odsyłać możesz tylko do tego, co i tak jest widoczne w hubie – czego nikt nie udostępnił, nie ma ani na liście podpowiedzi, ani jako klikalnego odnośnika.',
        },
        board: {
          title: 'Board',
          p1: 'Każdy hub ma dokładnie jeden board – wspólną powierzchnię do rysowania, która powstaje razem z hubem. Dotknięcie „Board” go otwiera; „Wstecz” wraca do poprzedniej karty. Wszyscy członkowie rysują, piszą i układają na nim; zapis dzieje się sam, chwilę po każdej zmianie. Przez „Wstaw” kładziesz na boardzie udostępnione w hubie obrazy, notatki i pliki jako kartę.',
          p2: 'Gdy kilka osób rysuje jednocześnie, aplikacja łączy ich zmiany: każdy element przyjmuje najnowszą wersję, a usunięte pozostaje usunięte. Gdy członkowie są w pobliżu, rysujecie razem na żywo i widzicie wskaźniki innych; w przeciwnym razie zmiany docierają przy następnej synchronizacji. Board jest szyfrowany end-to-end.',
        },
        geteilteKalender: {
          title: 'Udostępnianie kalendarzy',
          p1: 'W „Udostępnieniach” przyciskiem „Udostępnij” (w pobliżu: „Udostępnij coś”) udostępniasz hubowi także jeden ze swoich kalendarzy. Pojawia się wtedy na liście w sekcji „Kalendarze”; dotknięcie otwiera go w kalendarzu – u członków w sekcji „Z hubów”. Z opcją „podgląd” tylko go czytają, z opcją „edycja” dodają, zmieniają i usuwają wydarzenia. Sam kalendarz zmieniasz tylko ty. Jeśli dwie osoby zmienią to samo wydarzenie jednocześnie, druga wersja zostaje obok jako kopia, z „Konflikt”, datą i godziną w tytule. Wszystko jest zaszyfrowane własnym kluczem. Krzyżyk obok kalendarza kończy udostępnianie; twój kalendarz zostaje. Bez openany.de członkowie dostają go, gdy tylko zsynchronizują się w pobliżu – tam tylko do odczytu; kalendarze subskrybowane udostępnisz tylko w ten sposób.',
        },
        freigaben: {
          title: 'Udostępnianie treści',
          p1: 'W karcie „Udostępnienia” przyciskiem „Udostępnij” dzielisz się z hubem teczką notatek, albumem albo folderem (także aktami): najpierw wybierasz rodzaj, potem co. Wszystko pozostaje szyfrowane end-to-end – z osobnym kluczem dla każdego udostępnienia, który zmienia się, gdy kończysz udostępnianie albo ktoś opuszcza hub. Treści zostają u ciebie; gdy udostępnianie się kończy, wszystko zachowujesz.',
          p2: 'Albumy i foldery są udostępniane tylko do podglądu: członkowie otwierają je w hubie, widzą podalbumy i podfoldery, otwierają zdjęcia i PDF-y, a przyciskiem „Przejmij” zapisują kopię we własnym magazynie w „Aus Hubs”. Zmieniać możesz tylko ty.',
          p3: 'Teczki notatek udostępniasz z poziomem „podgląd” albo „edycja”. Członkowie otwierają je w zwykłym edytorze notatek i widzą też zdjęcia; z „edycją” zmieniają notatki i dodają nowe. Na razie zdjęcia może wstawiać tylko właściciel teczki.',
        },
        lokal: {
          title: 'Na openany.de albo tylko na miejscu',
          p1: 'Przez „+ Hub” tworzysz hub na tym urządzeniu. Jeśli urządzenie jest połączone z openany.de, hub od razu trafia też tam – na liście ma wtedy małą chmurkę. Kto założył hub, włącza i wyłącza to w Ustawienia → Synchronizacja → „Huby na openany.de”. Wyłączony synchronizuje się tylko na miejscu; na openany.de hub, czat, planowanie i udostępnienia zostają usunięte. Z chmurką zapraszasz członków po ich nazwie w openany (tylko właściciele i tylko osoby, które choć raz zsynchronizowały aplikację z openany.de). Bez chmurki zapraszasz przez „Zaproś członka w pobliżu”: na drugim urządzeniu musi być otwarte Openany, w tej samej sieci Wi-Fi lub hotspocie, a oba urządzenia pokazują te same sześć cyfr, które porównujecie i potwierdzacie. Hubem zarządzają urządzenia właściciela; jeśli jest tylko jedno, aplikacja ostrzega – sparuj wcześniej drugie własne urządzenie. Gdy usuwasz hub, najpierw zostaje to podpisane na jego liście członków; dopiero po tym urządzenia pozostałych wiedzą, że go już nie ma. Jeśli openany.de zgłasza hub jako usunięty bez tego zapisu, zostaje on na twoim urządzeniu i dalej synchronizuje się w pobliżu.',
          p2: 'Członkowie udostępniają w hubie własne teczki notatek, foldery i albumy przez „Udostępnij coś”, najpierw tylko do odczytu. Teczki notatek można też udostępnić do edycji: wtedy notatkę edytuje naraz tylko jedna osoba, a zapis następuje na urządzeniu osoby, do której należy teczka – jeśli nie ma go w pobliżu, notatka zostaje tylko do odczytu. Czat dociera od razu do wszystkich w pobliżu, do pozostałych przy następnej synchronizacji. Tak samo podróżuje board.',
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
        suche: {
          title: 'Wyszukiwanie i filtry',
          p1: 'Nad historią jest pole wyszukiwania. Znajduje wiadomości po treści, po nazwie rozmówcy i po identyfikatorach Matrix oraz wyróżnia trafienia. Pod nim przełączniki zawężają listę: do nieprzeczytanych, do wiadomości z załącznikiem i – jeśli używasz więcej niż jednej drogi – do pojedynczych dróg, takich jak Openany czy Matrix. Przełączniki można łączyć.',
          p2: 'Obok pola wyszukiwania przełączasz się między dwoma widokami: „Historia” pokazuje wszystkie wiadomości według czasu, „Według kontaktu” zbiera je w jeden wiersz na rozmówcę, z ostatnią wiadomością i liczbą nieprzeczytanych. Kliknięcie wiersza pokazuje historię właśnie z tą osobą; „Zamknij rozmowę” wraca do wszystkich.',
        },
        anhaenge: {
          title: 'Załączniki',
          p1: 'Przez Matrix wysyłasz też pliki: spinacz w polu pisania dołącza plik do 10 MB, a na telefonie i tablecie przycisk aparatu obok od razu robi zdjęcie. W historii obrazy pojawiają się jako podgląd, inne pliki z nazwą i rozmiarem; kliknięcie otwiera obrazy i PDF-y bezpośrednio w Openany, resztę pobierasz. Drogą Openany idą pliki do 10 MB, szyfrowane end-to-end jak tekst. Liczą się do twojej pamięci i do pamięci odbiorcy, dopóki każde z was nie usunie wiadomości.',
        },
        matrixApp: {
          title: 'Wiadomości przez Matrix',
          p1: 'W aplikacji twoje urządzenie samo jest urządzeniem Matrix. Logujesz się istniejącym kontem w ustawieniach w „Konta Matrix”, a twoje wiadomości są szyfrowane end-to-end między tym urządzeniem a rozmówcą – openany.de ich nie czyta. Aby inne aplikacje Matrix, takie jak Element, ufały temu urządzeniu, zweryfikuj je tam, przy koncie: kluczem odzyskiwania z Element lub – jeśli konto nie ma jeszcze kopii zapasowej – openany ją utworzy i raz pokaże nowy klucz.',
          p2: 'Możesz połączyć kilka kont Matrix. Ich wiadomości mają wspólną historię; nowe wychodzą z konta domyślnego, odpowiedzi z konta, na które przyszła wiadomość, a „Ustaw jako domyślne” ustawia inne konto jako domyślne. Jeśli usuniesz wiadomość Matrix, zniknie tylko na tym urządzeniu; w pokoju zostaje.',
        },
        email: {
          title: 'E-mail przez twoją skrzynkę',
          p1: 'W aplikacji dochodzi e-mail jako kolejna droga – nie osobny program pocztowy, lecz twoja istniejąca skrzynka, np. w Posteo, mailbox.org czy GMX. W ustawieniach w „Skrzynki e-mail” wpisujesz adres i hasło; zwykle jest to hasło aplikacji, które najpierw tworzysz u dostawcy. Serwery aplikacja wyszukuje sama; jeśli żadnych nie znajdzie, wpisujesz je w „Serwery ręcznie”. Hasło pozostaje zamknięte na tym urządzeniu, a urządzenie samo rozmawia z serwerem poczty – openany.de nie widzi żadnej wiadomości.',
          p2: 'Maile pojawiają się z tematem we wspólnej historii w Wiadomościach; „Pobierz teraz” od razu pobiera nowe. Jeśli połączysz kilka skrzynek, nowe maile wychodzą ze skrzynki domyślnej, a odpowiedzi z tej, na którą przyszedł mail; przy pisaniu wybierasz w „Od”. Przy usuwaniu decydujesz, czy mail zniknie tylko tutaj, czy trafi też do folderu kosza na serwerze. Jeśli dostawca uznał coś za spam, nad historią pojawia się informacja; tam „To nie spam” przenosi mail z powrotem do skrzynki odbiorczej.',
          p3: 'E-mailem załączniki mogą mieć do 20 MB. Odebrane załączniki zapisujesz w swoich plikach przez „Zapisz” albo w folderze pobranych przez „Na urządzenie”. Jeśli załącznik przyszedł zaszyfrowany, aplikacja najpierw pyta: w Plikach leży on niezaszyfrowany i jest synchronizowany z openany.de.',
        },
        pgp: {
          title: 'Szyfrowany e-mail z OpenPGP',
          p1: 'Maile możesz szyfrować end-to-end za pomocą OpenPGP. Każda skrzynka potrzebuje do tego własnego klucza: w ustawieniach przy skrzynce tworzysz go przyciskiem „Utwórz klucz” – albo, jeśli już używasz tej skrzynki z PGP, np. w Thunderbirdzie, wczytujesz istniejący przez „Wczytaj klucz”, aby oba programy czytały te same maile. Klucz prywatny jest tylko na tym urządzeniu. „Zapisz kopię zapasową” zapisuje go, zamkniętego hasłem, w folderze pobranych; „Zapisz klucz publiczny” daje ci plik, który wysyłasz innym.',
          p2: 'Klucze publiczne twoich kontaktów aplikacja sama zbiera w „Klucze kontaktów” z maili, które je dołączają. Jeśli jakiegoś brak, szukasz go po adresie – na życzenie także w keys.openpgp.org, który dowie się wtedy, o kogo pytasz – albo wczytujesz z pliku. Najlepiej raz porównaj odcisk palca z rozmówcą; jeśli klucz się zmieni, aplikacja to zaznaczy.',
          p3: 'Przy pisaniu włączasz „Wyślij zaszyfrowane”; pod spodem widać, czy klucz odbiorcy jest znany. Jeśli odpowiadasz na zaszyfrowany mail, przełącznik jest już włączony. W historii maile mają oznaczenie „zaszyfrowane”, a jeśli są podpisane, „podpisane” – albo ostrzeżenie, gdy podpis się nie zgadza.',
        },
        vorOrt: {
          title: 'Wiadomości w pobliżu',
          p1: 'Drogą „W pobliżu” piszesz bezpośrednio do urządzeń w pobliżu, zupełnie bez serwera. Na drugim urządzeniu Openany musi być otwarte, a oba muszą być w tej samej sieci Wi-Fi. Pisać do siebie możecie, gdy się znacie: własne urządzenia, członkowie wspólnego hubu – albo osoby, które potwierdziliście na miejscu. W tym celu wybierasz „Potwierdź na miejscu”, oba urządzenia pokazują te same 6 cyfr i oboje stukacie „Pasuje”. Jeśli drugiego urządzenia akurat nie ma, wiadomość czeka i przejdzie przy następnym spotkaniu.',
          p2: 'Od nieznajomych najpierw nic nie dociera. Jeśli w ustawieniach w „Urządzenia w pobliżu” włączysz „Zezwalaj na prośby od urządzeń w pobliżu”, mogą wysłać ci do 3 wiadomości po 500 znaków; stoją one w „Prośby” nad historią, a nie w samej historii. Odpowiedzieć możesz dopiero po potwierdzeniu 6 cyframi. Kto ma ci już nic nie wysyłać, tego zatrzymujesz przez „Zablokuj”. Jeśli twoja wiadomość została odrzucona, ma oznaczenie „nie dostarczono”, a powód pojawia się po wskazaniu.',
        },
        systemnachrichten: {
          title: 'Powiadomienia i odpowiedzi',
          p1: 'Zdarzenia z twoich hubów – na przykład nowe udostępnienia – docierają do ciebie jako powiadomienie, jeśli nadawca informuje członków; tak samo wiadomość, że ktoś dodał cię do hubu, oraz wiadomości od openany.de o twoim koncie, np. prośba o jego potwierdzenie. Powiadomienia nie są szyfrowane end-to-end. Tu znajdziesz też odpowiedź na wiadomość wysłaną przez formularz pomocy.',
        },
      },
    },
    contacts: {
      title: 'Książka adresowa',
      subs: {
        adressbuch: {
          title: 'Kontakty i ich kanały',
          p1: 'Książkę adresową otwierasz jej kafelkiem na stronie startowej albo u góry strony wiadomości; przycisk obok od razu tworzy nowy kontakt. Kontakt zapisuje, jak można się z kimś skontaktować: nazwę, zdjęcie, telefon i e-mail oraz dowolnie wiele innych sposobów – Matrix, Meshtastic, nazwę w Openany, adres, firmę, urodziny lub inne, każdy z własną etykietą, np. prywatny lub praca. Pole wyszukiwania znajduje kontakty po nazwie.',
          p2: 'Sposoby kontaktu są klikalne: numer telefonu dzwoni, adres e-mail otwiera program pocztowy, nazwa w Openany lub identyfikator Matrix otwiera pole pisania z wpisanym już odbiorcą. Usunięte kontakty trafiają do kosza.',
        },
        verschluesselt: {
          title: 'Szyfrowanie end-to-end',
          p1: 'Twoja książka adresowa jest szyfrowana end-to-end tym samym kluczem co twoje wiadomości bezpośrednie. Czytać ją mogą tylko twoje zatwierdzone urządzenia; openany.de ją przechowuje i synchronizuje, ale nie widzi ani nazw, ani numerów, ani zdjęć. Nowe urządzenie pokazuje książkę adresową dopiero po zatwierdzeniu na innym urządzeniu.',
        },
        vcard: {
          title: 'Zapis i wczytywanie vCard',
          p1: 'W Ustawieniach → Książka adresowa zapiszesz wszystkie kontakty jako plik vCard albo wczytasz plik z innego programu. Wczytywanie tylko dodaje: istniejące kontakty są rozpoznawane po identyfikatorze lub nazwie i zachowują to, co mają. Sam plik nie jest szyfrowany – przechowuj go odpowiednio.',
        },
        imTelefon: {
          title: "W kontaktach telefonu",
          p1: "Na Androidzie Ustawienia → Książka adresowa → „Pokazuj w kontaktach telefonu” pokazuje twoje kontakty w książce adresowej telefonu, na koncie „openany” – gdy ktoś dzwoni, pojawia się jego imię. Imię, telefon, e-mail, adres, firma, urodziny, strona, pseudonim i zdjęcie przechodzą do telefonu; to, co tam zmienisz, utworzysz lub usuniesz, wraca. Telefon nie zna identyfikatorów Matrix, Meshtastic ani Openany – zostają tylko w openany. Zdjęcie zmienione tam openany przywraca. Wyłączenie usuwa stamtąd kontakty.",
        },
        andereProgramme: {
          title: "Dla innych programów na komputerze",
          p1: "Na komputerze na przykład Thunderbird lub Kontakty Apple czytają i zapisują twoją książkę adresową przez CardDAV – włączane w Ustawienia → Kalendarz, z tymi samymi danymi co dla kalendarzy (zobacz tam). Przenoszone są wszystkie drogi kontaktu, także Matrix, Meshtastic i nazwa Openany.",
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
        module: {
          title: 'Moduły na stronie startowej',
          p1: 'Tutaj wybierasz, które moduły pojawią się jako kafelek na stronie startowej – Notatki, Kalendarz, Dysk, Huby i Książka adresowa. Wybór niczego nie ukrywa; wszystkie moduły pozostają dostępne. Bez wyboru strona startowa pokazuje ekran powitalny.',
        },
        spracheDesign: {
          title: 'Wygląd i język',
          p1: 'W „Wygląd” wybierasz jeden z dwóch wyglądów: „Klarowny” w morskim odcieniu z ostrzejszymi rogami i spokojnym tłem albo „Liliowy” z zaokrąglonymi rogami i tłem we wzór. Oba są w wersji jasnej i ciemnej – to przełączasz niezależnie przełącznikiem w nagłówku lub w menu na telefonie. W „Język” ustawiasz, w jakim języku Openany do ciebie mówi: niemieckim, angielskim, hiszpańskim, francuskim, portugalskim, polskim lub ukraińskim. Dopóki nie wybierzesz, podąża za językiem urządzenia lub przeglądarki; jeśli go tu nie ma, Openany mówi po angielsku.',
        },
        abgleich: {
          title: 'Synchronizacja z Openany',
          p1: 'W „Synchronizacja” łączysz aplikację z openany.de. Jeśli nie masz jeszcze konta, załóż je przez „Utwórz nowe konto” (zobacz „Zakładanie konta”). Jeśli masz już połączone urządzenie, zatwierdzasz na nim nowe za pomocą 6 cyfr (zobacz „Moje urządzenia”). Starsze konto z hasłem łączysz przez „Połącz z openany.de”: aplikacja pokazuje kod, który wpisujesz i potwierdzasz na stronie anyid w przeglądarce. Łączenie jest dobrowolne: bez połączenia wszystko zostaje na tym urządzeniu, a „Rozłącz” w każdej chwili to cofa bez utraty danych tutaj.',
          p2: 'Potem „Synchronizuj” synchronizuje w obie strony i informuje, co zostało pobrane, wysłane lub pominięte. „Pobierz wszystko ponownie” pobiera ponownie cały zasób zamiast samych nowości; nic przy tym nie jest usuwane.',
          p3: 'Z „Odświeżaj w tle” aplikacja synchronizuje się sama mniej więcej co godzinę – tylko z połączeniem i nie przy słabej baterii. W sieci komórkowej przychodzi tylko lista, pliki i zdjęcia dopiero przez Wi-Fi. Przez Wi-Fi próbuje też ze sparowanymi urządzeniami w pobliżu, jeśli działa na nich Openany. Na komputerze Openany synchronizuje się co godzinę, dopóki działa; z „Działaj dalej w tle” uruchamia się po zalogowaniu i po zamknięciu okna zostaje jako ikona na pasku menu.',
        },
        geraeteNah: {
          title: 'Urządzenia w pobliżu',
          p1: 'Twoje własne urządzenia synchronizują się też bezpośrednio, zupełnie bez serwera – o ile są w tej samej sieci Wi-Fi lub hotspocie, a na obu jest otwarte Openany. W „Urządzenia w pobliżu” znajdujesz je przez „Szukaj” i łączysz przez „Sparuj”: oba urządzenia pokazują ten sam kod, a na obu potwierdzasz, że się zgadza. Odtąd synchronizują się same, dopóki Openany działa na obu, oraz przy każdym „Synchronizuj”; „Zapomnij” cofa parowanie.',
          p2: 'W „Twoje imię” ustalasz, jak widzą cię inni na miejscu. Pod tym imieniem zaprasza się cię do hubów – ciebie jako osobę, a nie pojedyncze urządzenie; obowiązuje ono na wszystkich twoich sparowanych urządzeniach.',
        },
        meineGeraete: {
          title: 'Moje urządzenia',
          p1: 'To, co szyfrujesz end-to-end, mogą odczytać tylko twoje urządzenia; współdzielą w tym celu klucz. Nowe urządzenie pokazuje 6 cyfr, a ty wpisujesz je tutaj na już skonfigurowanym urządzeniu. Jeśli urządzenie zaginie, cofasz je tutaj; klucz się wtedy zmienia. Klucz jest też w twoim pliku kopii zapasowej.',
          p2: 'Na samym dole „Moje urządzenia” sam usuwasz swoje konto w openany.de – od razu i ze wszystkim, co tam jest. Dla bezpieczeństwa wpisujesz swoją nazwę użytkownika. Huby, w których są jeszcze inni członkowie, najpierw komuś przekazujesz.',
        },
        fristen: {
          title: 'Potwierdzanie konta i aktywność',
          p1: 'Aby nie gromadziły się nieużywane konta, openany.de prosi cię dwa dni po założeniu konta o ponowne potwierdzenie; u góry pojawia się pasek z „Zachowaj konto”. Jeśli nie potwierdzisz w ciągu 14 dni, konto zostanie usunięte.',
          p2: 'Jeśli przez cztery miesiące nie otworzysz aplikacji, ta sama prośba przyjdzie ponownie, a dwa tygodnie przed usunięciem ostatnie przypomnienie. Wystarczy stuknąć „Zachowaj konto”. Gdy konto zostanie usunięte, znikają tylko treści w openany.de; to, co jest na twoich urządzeniach, zostaje.',
        },
        speicherGeraet: {
          title: 'Pamięć na tym urządzeniu',
          p1: 'W „Pamięć na tym urządzeniu” decydujesz, co tu leży. „Zachowaj wszystko” pobiera wszystko z Twoich albumów, segregatorów i folderów, dopóki jest miejsce; to opcja dla komputerów i tabletów. Przy „Wybrane albumy, segregatory i foldery” leży tu tylko to, co wybierzesz w Pamięci przez „Zachowaj na tym urządzeniu”; z całej reszty widzisz tylko nazwę. Nowe zdjęcia z aparatu trafiają do albumu „Aparat”. To, co nie leży w albumie, segregatorze ani folderze, zostaje na swoim urządzeniu i nigdy nie jest synchronizowane.',
          p2: 'Na serwer trafiają notatki, kalendarze, kontakty, wiadomości oraz nazwy albumów, segregatorów i folderów. Zdjęcia i pliki trafiają tam tylko wtedy, gdy są udostępnione w hubie, dołączone do notatki lub gdy wybierzesz „Umieść na serwerze”, pojedynczo albo dla całego albumu, segregatora czy folderu. Całą resztę Twoje urządzenia synchronizują bezpośrednio między sobą. „Usuń z serwera” niczego nie kasuje: Twoje urządzenia zachowują swoje kopie, a ostatnia kopia zawsze zostaje.',
          p3: 'W „Pamięć na tym urządzeniu” widzisz też, ile leży na serwerze, w podziale na wiadomości, notatki, mediatekę, pliki i dokumenty.',
        },
        sofort: {
          title: 'Powiadamiaj od razu',
          p1: 'Z „Powiadamiaj od razu” aplikacja utrzymuje oszczędne połączenia z Openany i twoimi skrzynkami pocztowymi, aby nowe wiadomości i maile przychodziły od razu – bez Google. Skrzynki nie potrzebują do tego Openany. Android pokazuje przy tym stałe powiadomienie, które możesz ukryć w ustawieniach systemu. Na komputerze Openany czuwa, dopóki działa, i daje znać powiadomieniem systemowym.',
        },
        signal: {
          title: 'Sygnał wiadomości',
          p1: 'Gdy przyjdzie nowa wiadomość, a openany jest otwarte, aplikacja odtwarza krótki dźwięk – a na życzenie urządzenie wibruje. Jeśli jesteś już w Wiadomościach, jest cicho, bo i tak ją tam widzisz. Dźwięk i wibrację włączasz w Ustawienia → Wiadomości → „Sygnał wiadomości”; dotyczy to tylko tego urządzenia.',
          p2: 'Gdy openany jest zamknięte lub w tle, a „Powiadamiaj od razu” jest włączone, pojawia się zamiast tego powiadomienie systemowe. Jego głośność i wibrację ustawiasz w ustawieniach systemu Android; przycisk na kafelku tam prowadzi.',
        },
        sicherung: {
          title: 'Kopia zapasowa w jednym pliku',
          p1: 'W „Kopia zapasowa” przyciskiem „Utwórz kopię” tworzysz zaszyfrowany plik ze wszystkim, co Openany ma na tym urządzeniu: notatkami, kalendarzem, kontaktami, plikami, mediateką, hubami, wiadomościami i skrzynkami pocztowymi razem z hasłem i własnym kluczem OpenPGP. Aplikacja proponuje hasło z sześciu słów; zapisz je i przechowuj osobno od pliku. Bez niego pliku nie da się otworzyć – nawet nam.',
          p2: 'Przyciskiem „Przywróć kopię” otwierasz taki plik na innym urządzeniu. Zastępuje wszystko, co tam było; potem połączenie z openany.de jest usunięte, a urządzenia w pobliżu parujesz od nowa. To, co identyfikuje urządzenie, nie jest dołączane.',
        },
        tresor: {
          title: 'Poświadczenia i klucze w sejfie',
          p1: 'To, czym to urządzenie przedstawia się Openany i innym urządzeniom, jest przechowywane zamknięte na urządzeniu; klucz do tego zostaje w pęku kluczy systemu – na Androidzie w Keystore, a na komputerze w pęku kluczy macOS, menedżerze poświadczeń Windows lub Secret Service w Linuksie. Tam są też hasła do twoich skrzynek pocztowych i twoje tajne klucze OpenPGP. „Rozłącz” usuwa poświadczenia tylko tutaj – urządzenie odwołujesz na stałe w „Moje urządzenia”.',
        },
      },
    },
    contact: {
      title: 'Zgłoszenia, zasady i kontakt',
      subs: {
        kontakt: {
          title: 'Kontakt z nami',
          p1: 'Pytania, problemy lub życzenia wysyłasz przez formularz pomocy na samym dole tej pomocy; trafia on do administratorów zaszyfrowany end-to-end, a odpowiedź znajdziesz w Wiadomościach. Bez konta skontaktujesz się z nami przez formularz na openany.de/zglos w „Pytanie lub sprawa”. Odpowiadamy po niemiecku lub angielsku.',
        },
        bedingungen: {
          title: 'Warunki korzystania i prywatność',
          p1: 'Warunki korzystania i politykę prywatności znajdziesz w „O openany” pod formularzem pomocy, a notę prawną na samym końcu tej pomocy; wszystkie trzy są też na openany.de. W skrócie: openany jest bezpłatne i bez reklam, twoje treści są szyfrowane na twoich urządzeniach, a serwery stoją w Niemczech.',
        },
        melden: {
          title: 'Zgłaszanie treści',
          p1: 'Treści lub konta naruszające prawo albo warunki korzystania zgłaszasz przez formularz na openany.de/zglos – także bez konta. „Otwórz formularz” nad formularzem pomocy prowadzi tam w języku aplikacji. Zgłoszenie jest szyfrowane w przeglądarce; mogą je przeczytać tylko administratorzy.',
          p2: 'Po wysłaniu dostajesz kod dostępu. Dzięki niemu widzisz na tej samej stronie stan i naszą odpowiedź oraz możesz dopytać. Przechowuj go dobrze – bez niego nie dotrzesz już do zgłoszenia. Zostaje ono usunięte trzy miesiące po ostatniej wiadomości.',
        },
        entscheidungen: {
          title: 'Gdy zablokujemy lub usuniemy twoje konto',
          p1: 'Jeśli konto narusza warunki korzystania, administratorzy mogą je zablokować lub usunąć. Gdy twoje konto jest zablokowane, u góry pojawia się czerwony pasek; stuknięcie pokazuje, od kiedy, na czym opiera się decyzja i dlaczego. Synchronizacja z openany.de jest wtedy wstrzymana; to, co jest na twoich urządzeniach, zostaje. Po zniesieniu blokady wszystko działa jak wcześniej.',
          p2: 'Jeśli twoje konto zostanie usunięte, aplikacja przy następnej synchronizacji raz pokaże powód, a potem się rozłączy; aplikacja działa dalej bez openany.de. Jeśli nie zgadzasz się z decyzją, stuknij „Odwołaj się”: otworzy się formularz na openany.de/zglos z „Odwołanie od decyzji”, a my ponownie rozpatrzymy decyzję.',
        },
      },
    },
  },
};
