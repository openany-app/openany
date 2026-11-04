// Der Reiter „Board“ eines Hubs: das eine Board, dazu
// die Texte der Zeichenfläche (whiteboard) und zum Teilen von Kalendern.
export default {
  board: {
    titel: 'Board',
    hinweis: 'Wspólny board tego hubu: wszyscy członkowie rysują i piszą na nim – przez openany.de i w pobliżu.',
    geaendert: 'Ostatnia zmiana: {zeit}',
    oeffnen: 'Otwórz board',
  },
  kalender: {
    teilenHinweis: 'Z opcją „podgląd” członkowie widzą wydarzenia z tego kalendarza, z opcją „edycja” mogą je też dodawać, zmieniać i usuwać. Sam kalendarz zmieniasz tylko ty. Kalendarze subskrybowane udostępnisz tylko w pobliżu.',
    beenden: 'Zakończ udostępnianie',
    beendenFrage: 'Przestać udostępniać „{name}” w tym hubie? Wydarzenia znikną u wszystkich członków; twój kalendarz zostaje.',
  },
  whiteboard: {
    neu: 'Nowa tablica',
    nurAnsehen: 'Tylko podgląd – zapisać może osoba, która utworzyła tablicę.',
    nurAnsehenFern: 'Tylko podgląd – rysować razem można, gdy urządzenie {name} jest w pobliżu.',
    speichert: 'Zapisywanie …',
    gespeichert: 'Zapisano',
    nichtGespeichert: 'Nie zapisano – kolejna próba przy następnej zmianie.',
    veraltet: 'Inne z twoich urządzeń zapisało w międzyczasie. Tablica zostanie wczytana ponownie.',
    offline: 'Brak połączenia – widzisz ostatnio wczytaną wersję.',
    ladenFehler: 'Nie udało się otworzyć tablicy.',
    confirmDelete: 'Usunąć tablicę „{title}”?',
    loeschen: 'Usuń tablicę',
    laedt: 'Wczytywanie obszaru rysowania …',
    einbinden: 'Wstaw',
    alleFreigaben: 'Wszystkie udostępnienia',
    keineFreigaben: 'W tym hubie nic jeszcze nie udostępniono. Udostępnić możesz w zakładce „Udostępnienia”.',
    leer: 'Nie ma tu nic, co można wstawić.',
    nichtMehrGeteilt: 'Już nieudostępnione',
    karteNotiz: 'Notatka',
    kartePdf: 'PDF',
    karteDatei: 'Plik',
  },
};
