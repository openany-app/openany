export default {
  // Zwischenspeicher, kein Bestand: Ohne Netz lässt sich lesen, was zuletzt
  // offen war – schreiben nicht. Der Nutzer muss erkennen können, ob er den
  // aktuellen Stand sieht.
  readOnly: 'Kein Netz – du siehst zuletzt Geladenes, Bearbeiten geht gerade nicht.',
  asOf: 'Stand {when}',
  justNow: 'von gerade eben',
  minutesAgo: 'von vor {count} Minuten',
  hoursAgo: 'von vor {count} Stunden | von vor {count} Stunden',
  daysAgo: 'von vor {count} Tagen',
  updateReady: 'Eine neue Fassung ist bereit.',
  updateNow: 'Neu laden',
};
