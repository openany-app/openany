import { FileText, Calendar as CalendarIcon, Image as ImageIcon, Folder as FolderIcon, Briefcase, Contact as ContactIcon, Mail as MailIcon } from 'lucide-vue-next';

// Zentrale Modul-Registry. Neue Module brauchen nur einen Eintrag hier
// plus ihre Routen (mit meta: { module: '<id>' }) im Router.
//
// Alle Module stehen immer im Menü und stehen jedem Konto offen. Die
// Auswahl in den Einstellungen steuert nur, welche Module auf der
// Startseite als Kachel erscheinen – Reihenfolge hier = Menü- und
// Startseiten-Reihenfolge.
//
// Nachrichten und Adressbuch haben keinen Punkt in der Leiste (sie hängen am
// Briefumschlag der Kopfzeile), sind seit dem 13.09.2026 aber als Kachel für
// die Startseite wählbar.
// label/description sind i18n-Keys (keine Texte) – modules.js ist reines JS
// ohne Component-Context und kann vue-i18n nicht selbst aufrufen. Verbraucher
// lösen sie über t(mod.label) / t(mod.description) auf (siehe shell.modules.*
// in i18n/locales/{de,en}/shell.js).
export const MODULES = [
  {
    id: 'notes',
    label: 'shell.modules.notes.label',
    icon: FileText,
    route: '/notes',
    description: 'shell.modules.notes.description',
  },
  {
    id: 'calendar',
    label: 'shell.modules.calendar.label',
    icon: CalendarIcon,
    route: '/calendar',
    description: 'shell.modules.calendar.description',
  },
  {
    // Technisch eigenständiges Modul (eigene Rollen-Deckelung und Routen),
    // aber ohne Menüpunkt und nicht separat für die Startseite wählbar –
    // die Galerie wird auf der Dateien-Seite unterhalb der Dateien
    // ausgegeben (Files.vue bettet Albums.vue ein).
    id: 'albums',
    label: 'shell.modules.albums.label',
    icon: ImageIcon,
    route: null,
    homeSelectable: false,
    description: 'shell.modules.albums.description',
  },
  {
    // Heißt in der Oberfläche "Speicher"; Id und Pfad bleiben aus
    // Kompatibilitätsgründen 'files' bzw. /files.
    id: 'files',
    label: 'shell.modules.files.label',
    icon: FolderIcon,
    route: '/files',
    description: 'shell.modules.files.description',
  },
  {
    // Kein Ort in der Leiste, sondern der Briefumschlag in der Kopfzeile –
    // hier steht der Eintrag nur, damit die Startseite eine Kachel dafür
    // anbieten kann. `navPunkt: false` hält ihn aus NAV_MODULES heraus.
    id: 'messages',
    label: 'shell.modules.messages.label',
    icon: MailIcon,
    route: '/messages',
    navPunkt: false,
    description: 'shell.modules.messages.description',
  },
  {
    // Das Adressbuch: wer, und wie erreichbar (Matrix, Meshtastic). Es ist
    // die Bedingung dafür, dass ein Programm überhaupt einen Weg wählen
    // kann – ohne Kennungen gibt es nichts zu funken.
    //
    // NICHT IN DER LEISTE UND SEIT DEM 10.09.2026 AUCH NICHT MEHR IM MENÜ,
    // sondern in den NACHRICHTEN. Ein Kontakt ist kein Ort, an den man zum
    // Arbeiten geht, sondern das Adressbuch der Kommunikation – und dort
    // gehört es hin, neben die Nachrichten selbst. Das Menü sammelt, was zum
    // Konto gehört (Einstellungen, Hilfe, Papierkorb); ein Adressbuch dazwischen
    // war ein Fremdkörper, den man nur fand, wenn man ihn schon kannte.
    // Sobald Chat dazukommt, gehört er ohnehin an dieselbe Stelle.
    //
    // Die Route bleibt: `/contacts` ist ein eigener Ort, nur erreicht man ihn
    // jetzt über die Kopfzeile der Nachrichten (Messages.vue).
    //
    // Am 06.09.2026 herausgenommen: Beim Bau des Adressbuchs stand hier nur
    // `route`, und `NAV_MODULES` nimmt automatisch alles auf, was eine hat –
    // so entstand ein fünfter Reiter, den niemand bestellt hatte.
    id: 'contacts',
    label: 'shell.modules.contacts.label',
    icon: ContactIcon,
    route: '/contacts',
    navPunkt: false,
    // Bis zum 13.09.2026 stand hier `homeSelectable: false`; seitdem gibt
    // es eine Adressbuch-Kachel für die Startseite.
    description: 'shell.modules.contacts.description',
  },
  {
    id: 'projects',
    label: 'shell.modules.projects.label',
    icon: Briefcase,
    route: '/projects',
    description: 'shell.modules.projects.description',
  },
];

export const ALL_MODULE_IDS = MODULES.map((m) => m.id);

// Module mit eigenem Punkt in der Leiste.
//
// Ausgenommen sind die Galerie (sie wird auf der Dateien-Seite mit
// ausgegeben, hat also keine Route) und ausdrücklich das Adressbuch: Die
// Leiste trägt ORTE, an die man zum Arbeiten geht. Kontakte und Nachrichten
// sind Werkzeuge und stehen im Menü – anders wäre die Leiste voll, bevor der
// Chat überhaupt gebaut ist.
export const NAV_MODULES = MODULES.filter((m) => m.route && m.navPunkt !== false);

// Hier stand bis zum 10.09.2026 `MENUE_MODULES` – Module mit Route, aber ohne
// Platz in der Leiste. Sein einziges Mitglied war das Adressbuch, und das ist
// zu den Nachrichten gewandert. Ein Filter, der immer eine leere Liste
// liefert, sieht aus wie eine Fähigkeit und ist keine; wer wieder ein Modul
// ins Menü legen will, schreibt die drei Zeilen dann neu – mit einem
// Mitglied, an dem sich prüfen lässt, ob sie stimmen.
