// Startseite openany.de (frontend/start.html).
// Achtung vue-i18n: kein rohes @, | oder { in den Texten (außer Platzhaltern).
export default {
  seitentitel: 'openany – prywatnie, szyfrowane, najpierw na twoim urządzeniu',
  kopf: 'Twoje notatki, terminy, pliki i wiadomości – w jednym miejscu.',
  unterzeile: 'Prywatnie, z szyfrowaniem end-to-end i najpierw na twoim urządzeniu. Za darmo, bez reklam, bez hasła.',
  ladenFuer: 'Pobierz na {system}',
  alleDownloads: 'Wszystkie pliki',
  iosFolgt: 'openany na iPhone’a i iPada jest w przygotowaniu.',
  merkmale: {
    verschluesselt: {
      titel: 'Tylko ty możesz to przeczytać',
      text: 'Wszystko jest szyfrowane na twoim urządzeniu, zanim trafi na serwer. Nie możemy czytać twoich treści – nawet gdybyśmy chcieli.',
    },
    geraet: {
      titel: 'Najpierw na twoim urządzeniu',
      text: 'openany działa bez internetu i bez konta. Synchronizacja przez openany.de to możliwość, nie obowiązek.',
    },
    alles: {
      titel: 'Wszystko w jednym miejscu',
      text: 'Notatki, kalendarz, kontakty, pliki, zdjęcia, filmy i wiadomości – w jednej aplikacji zamiast dziesięciu.',
    },
    hubs: {
      titel: 'Razem w hubach',
      text: 'Udostępniaj foldery, kalendarze i albumy innym, rysujcie razem na tablicy i piszcie na czacie.',
    },
    sparsam: {
      titel: 'Bez hasła, bez adresu e-mail',
      text: 'Konto zakładasz bezpośrednio w aplikacji i jest ono związane z twoimi urządzeniami. Nie przechowujemy adresów e-mail ani adresów IP.',
    },
    frei: {
      titel: 'Za darmo i bez reklam',
      text: 'Bez abonamentu, bez reklam, bez sprzedaży danych. Każde konto dostaje na start 100 MB miejsca na openany.de.',
    },
  },
  downloadsTitel: 'openany na twoje urządzenie',
  version: 'Wersja {version}',
  empfohlen: 'Na twoje urządzenie',
  beiGithub: 'Pliki są przechowywane w serwisie GitHub.',
  keineListe: 'Nie można teraz wczytać aktualnej listy.',
  dateien: {
    apk: 'Pobierz APK',
    installer: 'Instalator (.exe)',
    msi: 'Pakiet MSI',
    dmg: 'Pobierz na Maca (.dmg)',
    appimage: 'AppImage',
    deb: 'Debian i Ubuntu (.deb)',
  },
  hinweise: {
    android: 'Bezpośrednio jako plik, nie ze Sklepu Play. Za pierwszym razem Android zapyta, czy zezwolić na aplikacje z tego źródła.',
    windows: 'Dla Windows 10 i 11. Aplikacja nie jest jeszcze podpisana: jeśli Windows ostrzega, wybierz „Więcej informacji”, a potem „Uruchom mimo to”.',
    macos: 'Od macOS 13.3, dla procesorów Apple i Intel. Aplikacja nie jest jeszcze poświadczona przez Apple: przy pierwszym uruchomieniu zezwól na nią w Ustawieniach systemowych, Prywatność i ochrona, „Otwórz mimo to”.',
    linux: 'AppImage dla prawie każdej dystrybucji, .deb dla Debiana i Ubuntu. 64 bity.',
  },
  fuss: {
    melden: 'Zgłoszenia i kontakt',
    agb: 'Warunki korzystania',
    datenschutz: 'Prywatność',
    impressum: 'Nota prawna',
    github: 'Wszystkie wersje w serwisie GitHub',
  },
  recht: {
    zurueck: 'Do strony głównej',
    nurDeutsch: 'Ten tekst obowiązuje tylko w wersji niemieckiej.',
    fehler: 'Nie można teraz wczytać tekstu. Spróbuj później.',
  },
  plattformen: 'Na Androida, Windows, macOS i Linuksa. iPhone i iPad są w przygotowaniu.',
  kopfAlt: [
    'Ekran startowy aplikacji z dzisiejszym dniem, książką adresową, notatkami, pamięcią i hubami',
    'Album w mediatece z krajobrazami',
  ],
  rundgangKnopf: 'Zobacz przewodnik',
  rundgangLabel: 'Przewodnik',
  sprung: {
    notizen: 'Notatki',
    kalender: 'Kalendarz',
    hubs: 'Huby',
    nachrichten: 'Wiadomości',
    mediathek: 'Mediateka',
    dokumente: 'Dokumenty',
    downloads: 'Pobieranie',
  },
  rundgang: {
    notizen: {
      augenbraue: 'Notatki',
      titel: 'Pisz i łącz swoje myśli',
      abschnitte: [
        {
          titel: 'Pisz, porządkuj, łącz',
          absaetze: [
            'Notatki leżą w teczkach i są pisane w Markdownie – z nagłówkami, listami i zadaniami do odhaczenia.',
          ],
          punkte: [
            '`[[Wikilinki]]` łączą notatki ze sobą',
            '`#Tagi` zbierają to, co do siebie pasuje',
            'Wyszukiwanie pełnotekstowe znajdzie każde słowo',
          ],
          alt: [
            'Notatka z listą, wikilinkami, tagami i zadaniami',
          ],
        },
        {
          titel: 'Zobacz, jak wszystko się łączy',
          absaetze: [
            'Widok „Powiązania” pokazuje twoje notatki jako sieć. Wybierz teczkę albo tag, a pojawią się pasujące notatki.',
          ],
          alt: [
            'Lista notatek z teczkami i notatkami, nazwy rozmyte',
            'Graf powiązań między notatkami, etykiety rozmyte',
          ],
        },
      ],
    },
    kalender: {
      augenbraue: 'Kalendarz',
      titel: 'Terminy dla ciebie i dla wszystkich, z którymi planujesz',
      abschnitte: [
        {
          titel: 'Twój tydzień w pigułce',
          absaetze: [
            'Miesiąc, tydzień lub dzień: terminy z miejscem, notatkami i powtarzaniem. Wystarczy dotknąć dnia.',
          ],
          punkte: [
            'Kilka kalendarzy obok siebie',
            '„Dzisiaj” od razu na ekranie startowym',
            'Zaszyfrowane jak wszystko inne',
            'Powtórzenia, np. „co 2 tygodnie we wtorek, 5 razy”',
          ],
          alt: [
            'Widok miesiąca z przykładowymi terminami',
          ],
        },
        {
          titel: 'Udostępniaj kalendarze w hubach',
          absaetze: [
            'Z prawem „podgląd” członkowie widzą twoje terminy, z prawem „edycja” sami je dodają. Sam kalendarz zmieniasz tylko ty.',
          ],
          alt: [
            'Okno udostępniania kalendarza w hubie',
            'Hub z udostępnioną teczką, kalendarzem, albumem i aktami',
          ],
        },
        {
          titel: 'W kalendarzu i kontaktach urządzenia',
          absaetze: [
            'Na Androidzie twoje kalendarze i kontakty z openany pojawiają się w aplikacjach telefonu – a to, co tam zmienisz, wraca. Na komputerze openany udostępnia je programom do kalendarza i kontaktów, tylko na tym komputerze.',
          ],
          punkte: [
            'Przypomnienia o wydarzeniach w telefonie',
            'Na komputerze przez CalDAV i CardDAV, np. dla Thunderbirda',
          ],
        },
      ],
    },
    hubs: {
      augenbraue: 'Huby',
      titel: 'Razem, a nie obok siebie',
      abschnitte: [
        {
          titel: 'Miejsce dla twojej grupy',
          absaetze: [
            'Hub należy do stowarzyszenia, wspólnego mieszkania, rodziny albo projektu. Członków zapraszasz po ich nazwie w openany. Nazwa i czat są szyfrowane end-to-end.',
          ],
          punkte: [
            'Udostępniaj teczki, kalendarze, albumy i akta',
            'Przy każdym udostępnieniu: podgląd lub edycja',
          ],
          alt: [
            'Lista członków hubu z wyszukiwaniem do zapraszania',
          ],
        },
        {
          titel: 'Jeden board dla wszystkich',
          absaetze: [
            'Każdy hub ma wspólny board. Wszyscy członkowie rysują i piszą na nim – przez openany.de albo bezpośrednio w pobliżu, w tej samej sieci.',
          ],
          punkte: [
            'Kształty, strzałki, tekst i pióro',
            'Wstawianie obrazów',
            'Równoczesne zmiany są scalane',
          ],
          alt: [
            'Board z narysowanym planem grządek',
          ],
        },
        {
          titel: 'Ustalenia na czacie hubu',
          absaetze: [
            'Każdy hub ma własny czat. Wpisując `[[`, wstawiasz link do zawartości hubu prosto w wiadomość.',
          ],
          punkte: [
            'Szyfrowany end-to-end',
            'Przypinanie ważnych wiadomości',
            'Powiadamianie członków na życzenie',
          ],
          alt: [
            'Czat hubu z trzema przykładowymi wiadomościami',
          ],
        },
      ],
    },
    nachrichten: {
      augenbraue: 'Wiadomości',
      titel: 'Wszystkie kanały na jednej liście',
      abschnitte: [
        {
          titel: 'Wiadomości bezpośrednie, szyfrowane',
          absaetze: [
            'Wiadomości między kontami openany są szyfrowane end-to-end, razem z załącznikami. Dochodzą do tego Matrix i e-mail – wszystko na jednej liście.',
          ],
          punkte: [
            'Filtry: nieprzeczytane, z załącznikiem, openany, w pobliżu',
            'W pobliżu działa nawet bez serwera',
            'Według rozmowy lub kontaktu',
          ],
          alt: [
            'Lista wiadomości z wyszukiwaniem i filtrami, częściowo rozmyta',
          ],
        },
        {
          titel: 'Matrix: dotrzyj też do osób bez openany',
          absaetze: [
            'Połącz konto na matrix.org lub innym serwerze. Twoje urządzenie loguje się samo, z własnymi kluczami.',
          ],
          punkte: [
            'openany.de niczego nie czyta',
            'Historia zostaje na urządzeniu',
            'Hasło nie jest zapisywane',
          ],
          alt: [
            'Ustawienia: połączenie konta Matrix',
          ],
        },
        {
          titel: 'E-mail, na życzenie z PGP',
          absaetze: [
            'Twoja dotychczasowa skrzynka staje się trzecim kanałem na liście. Urządzenie samo łączy się z serwerem poczty.',
            'Publiczne klucze OpenPGP twoich kontaktów przychodzą same z ich e-mailami – nimi szyfrujesz swoje odpowiedzi.',
          ],
          alt: [
            'Ustawienia: połączenie skrzynki e-mail',
            'Ustawienia: klucze kontaktów',
          ],
        },
      ],
    },
    mediathek: {
      augenbraue: 'Mediateka',
      titel: 'Twoje zdjęcia, filmy i nagrania',
      abschnitte: [
        {
          titel: 'Zdjęcia, filmy i dźwięk w albumach',
          absaetze: [
            'Albumy z podalbumami, uzupełniane prosto z aplikacji – aparatem, kamerą lub mikrofonem. Albumy udostępniasz w hubach.',
          ],
          punkte: [
            'Miniatury powstają na urządzeniu',
            'Dane o lokalizacji zostają, jeśli chcesz',
            'Zapisane w postaci zaszyfrowanej',
          ],
          alt: [
            'Album w mediatece z krajobrazami',
          ],
        },
        {
          titel: 'Wiele plików naraz',
          absaetze: [
            'Dodaj naraz wszystkie zdjęcia i filmy z telefonu – jako kopię albo bez niej: wtedy zostają w telefonie i trafiają zaszyfrowane do openany.de.',
          ],
          punkte: [
            'Nowe zdjęcia automatycznie, jeśli chcesz',
            'Udostępnianie do openany z innych aplikacji',
            'Na komputerze przeciąganie plików i folderów do okna lub synchronizacja folderu',
          ],
        },
      ],
    },
    dokumente: {
      augenbraue: 'Dokumenty',
      titel: 'Papiery, które da się przeszukać',
      abschnitte: [
        {
          titel: 'Papiery w aktach',
          absaetze: [
            'Listy, umowy i dokumenty leżą w aktach. PDF-y otwierają się bezpośrednio w openany: przeglądanie, wyszukiwanie, wypełnianie formularzy, zaznaczanie i komentowanie.',
          ],
          punkte: [
            'Wyszukiwanie także w treści PDF-ów i plików biurowych',
            'Udostępnianie akt w hubach',
          ],
          alt: [
            'Dokumenty z listą akt, prywatne nazwy rozmyte',
          ],
        },
        {
          pille: 'Rozpoznawanie tekstu',
          titel: 'Ze zdjęcia do przeszukiwalnego PDF-a',
          absaetze: [
            'Zrób zdjęcie listu i włóż je do akt. Powstaje PDF z niewidoczną warstwą tekstu: można go przeszukiwać, zaznaczać i kopiować.',
            'Rozpoznawanie działa na twoim urządzeniu. Obraz nigdzie nie trafia.',
          ],
          alt: [
            'Komunikat o konwersji zdjęcia',
            'Sfotografowany list jako PDF, wyszukiwanie zaznacza słowo',
          ],
        },
        {
          pille: 'PDF',
          titel: 'Każda notatka jako PDF',
          absaetze: [
            'Jedno dotknięcie zamienia notatkę w PDF i zapisuje go w twoich dokumentach. Do wyboru są cztery wersje:',
          ],
          punkte: [
            'Standardowa, z tytułem, datą i źródłami',
            'Clean, sam tekst',
            'List według DIN 5008',
            'Wersja do druku ze stroną tytułową',
          ],
          alt: [
            'Okno tworzenia PDF z czterema wersjami',
            'Notatka jako gotowy PDF w przeglądarce',
          ],
        },
      ],
    },
  },
  versprechen: {
    augenbraue: 'Obietnica',
    titel: 'Tylko ty możesz to przeczytać.',
    text: 'Wszystko jest szyfrowane na twoim urządzeniu, zanim trafi na serwer. Hosting w Niemczech, bez reklam i bez śledzenia.',
  },
  downloadsAugenbraue: 'Pobieranie',
  ios: {
    titel: 'iPhone i iPad',
    pille: 'W przygotowaniu',
    text: 'Aplikacja na iOS jest w przygotowaniu. Gdy będzie gotowa, znajdziesz ją tutaj.',
  },
  alleVersionen: {
    titel: 'Wszystkie wersje',
    text: 'Starsze wersje i wszystkie pliki są na GitHubie.',
    knopf: 'Przejdź do GitHuba',
  },
};
