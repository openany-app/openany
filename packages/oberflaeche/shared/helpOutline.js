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
// `p` ist die Anzahl der Absätze p1..pN. `nur: 'app'` an einem Abschnitt
// oder Unterabschnitt: steht nur in der Hilfe des Programms (Tiffy,
// 03.10.2026: Was nur die App kann, gehört nicht in die Webapp-Hilfe).
// `nur: 'web'`: nur in der Webapp -- für das, was im Programm anders geht
// (Matrix: seit Oktober 2026 nur im Programm; die Webapp sagt, wo es ist).
export const HELP_OUTLINE = [
  { id: 'start', subs: [
    { id: 'konto', p: 2, nur: 'web' }, { id: 'ohneKonto', p: 2, nur: 'app' },
    { id: 'startseite', p: 2 }, { id: 'navigation', p: 2 },
  ] },
  // Notizen gibt es seit Schritt 6 nur in der App (docs/plan-notizen-e2e.md).
  { id: 'notes', nur: 'app', subs: [
    { id: 'editor', p: 2 }, { id: 'mappen', p: 2 }, { id: 'wikilinks', p: 2 },
    { id: 'tags', p: 1 }, { id: 'einfuegen', p: 2 }, { id: 'zitate', p: 2 },
    { id: 'graph', p: 3 }, { id: 'export', p: 1 },
    { id: 'druckfassung', p: 3 }, { id: 'papierkorb', p: 1 },
    { id: 'verschluesselt', p: 1 }, { id: 'ordner', p: 1 },
  ] },
  // Dateien, Akten und Galerie seit Schritt 7 nur in der App (docs/plan-dateien-e2e.md).
  { id: 'files', nur: 'app', subs: [
    { id: 'aufbau', p: 2 }, { id: 'dokumente', p: 2 }, { id: 'texterkennung', p: 3 },
    { id: 'dateien', p: 2 }, { id: 'suche', p: 2 }, { id: 'pdf', p: 3 },
    { id: 'galerie', p: 3 }, { id: 'videos', p: 2 },
    { id: 'karte', p: 1 }, { id: 'papierkorbSpeicher', p: 1 },
    { id: 'dateienVerschluesselt', p: 1 },
  ] },
  // Den Kalender gibt es seit Schritt 4 nur in der App (docs/plan-kalender-e2e.md).
  { id: 'calendar', nur: 'app', subs: [
    { id: 'verwalten', p: 1 }, { id: 'termine', p: 1 }, { id: 'abos', p: 1 },
    { id: 'verschluesselt', p: 1 }, { id: 'importExport', p: 1 },
  ] },
  // Projekte gibt es seit Schritt 3 nur in der App (docs/plan-projektchat-e2e.md).
  { id: 'projects', nur: 'app', subs: [
    { id: 'grundlagen', p: 2 }, { id: 'chat', p: 4 }, { id: 'planung', p: 2 },
    { id: 'terminfindung', p: 1 }, { id: 'umfragen', p: 1 }, { id: 'listen', p: 1 },
    { id: 'geteilteKalender', p: 1 },
    { id: 'freigaben', p: 3 },
    { id: 'lokal', p: 3, nur: 'app' },
  ] },
  { id: 'messages', subs: [
    // Direktnachrichten gibt es seit Schritt 3 nur in der App; die Webapp
    // zeigt Hinweise (docs/plan-direktnachrichten-e2e.md).
    { id: 'direkt', p: 3, nur: 'app' }, { id: 'hinweise', p: 2, nur: 'web' },
    { id: 'suche', p: 2, nur: 'app' }, { id: 'anhaenge', p: 1, nur: 'app' },
    { id: 'matrixApp', p: 2, nur: 'app' },
    { id: 'email', p: 3, nur: 'app' }, { id: 'pgp', p: 3, nur: 'app' },
    { id: 'vorOrt', p: 2, nur: 'app' }, { id: 'systemnachrichten', p: 1 },
  ] },
  // Das Adressbuch gibt es seit Schritt 2 nur in der App (docs/plan-adressbuch-e2e.md).
  { id: 'contacts', nur: 'app', subs: [
    { id: 'adressbuch', p: 2 }, { id: 'verschluesselt', p: 1 }, { id: 'vcard', p: 1 },
    { id: 'sicherheitsnummer', p: 1 },
  ] },
  { id: 'settings', subs: [
    { id: 'profil', p: 1, nur: 'web' }, { id: 'module', p: 1 },
    { id: 'sicherheit', p: 2, nur: 'web' },
    { id: 'spracheDesign', p: 1 },
    { id: 'zugriff', p: 2, nur: 'web' },
    { id: 'abgleich', p: 3, nur: 'app' }, { id: 'geraeteNah', p: 2, nur: 'app' },
    { id: 'speicherGeraet', p: 1, nur: 'app' }, { id: 'sicherung', p: 2, nur: 'app' },
    { id: 'sofort', p: 1, nur: 'app' },
    { id: 'tresor', p: 1, nur: 'app' },
  ] },
  { id: 'contact', nur: 'web', subs: [
    { id: 'kontakt', p: 1 }, { id: 'bedingungen', p: 1 },
  ] },
];

/** Die Gliederung, wie sie die Hilfe zeigt: im Programm ohne `nur: 'web'`, sonst ohne `nur: 'app'`. */
export function hilfeGliederung({ programm = false } = {}) {
  const sichtbar = (eintrag) => eintrag.nur !== (programm ? 'web' : 'app');

  return HELP_OUTLINE.filter(sichtbar)
    .map((s) => ({ ...s, subs: s.subs.filter(sichtbar) }))
    .filter((s) => s.subs.length > 0);
}
