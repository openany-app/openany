// Gliederung des Handbuchs (/hilfe): welche Abschnitte, welche
// Unterabschnitte, und wie viele Absätze jeder Unterabschnitt hat.
//
// Bewusst getrennt von HelpPage.vue: Die Gliederung ist Inhalt, kein
// Aussehen. Sie steckte in der Komponente und war damit ungeprüft – wer
// einen Absatz ergänzt, muss die Gliederung und alle sieben help.js
// anfassen, und vergisst er eine, steht auf der Seite die rohe
// Schlüsselzeile „help.sections.projects.subs.chat.p3". Als eigenes Modul
// lässt sich die Gliederung gegen alle Sprachen prüfen
// (shared/helpOutline.test.js).
//
// `p` ist die Anzahl der Absätze p1..pN. Die Hilfe steht nur noch im
// Programm; eine eigene Fassung für die Webapp gibt es nicht mehr.
export const HELP_OUTLINE = [
  { id: 'start', subs: [
    { id: 'ohneKonto', p: 2 }, { id: 'kontoAnlegen', p: 3 },
    { id: 'startseite', p: 2 }, { id: 'navigation', p: 2 },
  ] },
  // Notizen gibt es nur in der App (docs/plan-notizen-e2e.md).
  { id: 'notes', subs: [
    { id: 'editor', p: 2 }, { id: 'mappen', p: 2 }, { id: 'wikilinks', p: 2 },
    { id: 'tags', p: 1 }, { id: 'einfuegen', p: 2 }, { id: 'zitate', p: 2 },
    { id: 'graph', p: 3 }, { id: 'export', p: 1 },
    { id: 'druckfassung', p: 3 }, { id: 'papierkorb', p: 1 },
    { id: 'verschluesselt', p: 1 }, { id: 'ordner', p: 1 },
  ] },
  // Dateien, Akten und Galerie nur in der App (docs/plan-dateien-e2e.md).
  { id: 'files', subs: [
    { id: 'aufbau', p: 2 }, { id: 'dokumente', p: 2 }, { id: 'texterkennung', p: 3 },
    { id: 'dateien', p: 2 }, { id: 'suche', p: 2 }, { id: 'pdf', p: 3 },
    { id: 'galerie', p: 3 }, { id: 'fotosVomGeraet', p: 2 }, { id: 'videos', p: 2 }, { id: 'ton', p: 3 },
    { id: 'karte', p: 1 }, { id: 'papierkorbSpeicher', p: 1 },
    { id: 'dateienVerschluesselt', p: 1 },
  ] },
  // Den Kalender gibt es nur in der App (docs/plan-kalender-e2e.md).
  { id: 'calendar', subs: [
    { id: 'verwalten', p: 1 }, { id: 'termine', p: 2 }, { id: 'abos', p: 1 },
    { id: 'verschluesselt', p: 1 }, { id: 'importExport', p: 1 },
    // Im Kalender des Telefons (Android) und für andere Programme (Desktop).
    { id: 'imTelefon', p: 2 }, { id: 'andereProgramme', p: 2 },
  ] },
  // Projekte gibt es nur in der App (docs/plan-projektchat-e2e.md).
  { id: 'projects', subs: [
    // In der Reihenfolge der Reiter: Freigaben, Board, Mitglieder, Chat.
    { id: 'grundlagen', p: 2 }, { id: 'freigaben', p: 3 }, { id: 'geteilteKalender', p: 1 },
    { id: 'board', p: 2 }, { id: 'chat', p: 4 },
    { id: 'lokal', p: 3 },
  ] },
  { id: 'messages', subs: [
    { id: 'direkt', p: 3 }, { id: 'suche', p: 2 }, { id: 'anhaenge', p: 1 },
    { id: 'matrixApp', p: 2 }, { id: 'email', p: 3 }, { id: 'pgp', p: 3 },
    { id: 'vorOrt', p: 2 }, { id: 'systemnachrichten', p: 1 },
  ] },
  // Das Adressbuch gibt es nur in der App (docs/plan-adressbuch-e2e.md).
  { id: 'contacts', subs: [
    { id: 'adressbuch', p: 2 }, { id: 'verschluesselt', p: 1 }, { id: 'vcard', p: 1 },
    { id: 'imTelefon', p: 1 }, { id: 'andereProgramme', p: 1 },
    { id: 'sicherheitsnummer', p: 1 },
  ] },
  { id: 'settings', subs: [
    { id: 'module', p: 1 }, { id: 'spracheDesign', p: 1 },
    { id: 'abgleich', p: 3 }, { id: 'geraeteNah', p: 2 },
    { id: 'meineGeraete', p: 2 }, { id: 'fristen', p: 2 },
    { id: 'speicherGeraet', p: 3 }, { id: 'sicherung', p: 2 },
    { id: 'sofort', p: 1 }, { id: 'signal', p: 2 }, { id: 'tresor', p: 1 },
  ] },
  // Kontakt, Meldeweg, Entscheidungen der Verwaltung, Rechtstexte.
  { id: 'contact', subs: [
    { id: 'kontakt', p: 1 }, { id: 'melden', p: 2 }, { id: 'entscheidungen', p: 2 },
    { id: 'bedingungen', p: 1 },
  ] },
];

/** Die Gliederung, wie sie die Hilfe zeigt. */
export function hilfeGliederung() {
  return HELP_OUTLINE;
}
