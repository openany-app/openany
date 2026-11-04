// Der Reiter „Board“ eines Hubs: das eine Board, dazu
// die Texte der Zeichenfläche (whiteboard) und zum Teilen von Kalendern.
export default {
  board: {
    titel: 'Board',
    hinweis: 'Le board commun de ce hub : tous les membres y dessinent et y écrivent – via openany.de et à proximité.',
    geaendert: 'Dernière modification : {zeit}',
    oeffnen: 'Ouvrir le board',
  },
  kalender: {
    teilenHinweis: 'Avec « voir », les membres voient les rendez-vous de cet agenda ; avec « modifier », ils peuvent aussi en ajouter, les changer et les supprimer. Toi seul modifies l’agenda lui-même. Les agendas abonnés ne se partagent qu’à proximité.',
    beenden: 'Arrêter le partage',
    beendenFrage: 'Ne plus partager « {name} » dans ce hub ? Les rendez-vous disparaissent pour tous les membres ; ton agenda reste.',
  },
  whiteboard: {
    neu: 'Nouveau tableau blanc',
    nurAnsehen: 'Lecture seule – seule la personne qui a créé ce tableau peut l’enregistrer.',
    nurAnsehenFern: 'Lecture seule – tu peux dessiner avec quand un appareil de {name} est à proximité.',
    speichert: 'Enregistrement …',
    gespeichert: 'Enregistré',
    nichtGespeichert: 'Non enregistré – nouvel essai à la prochaine modification.',
    veraltet: 'Un autre de tes appareils a enregistré entre-temps. Le tableau est rechargé.',
    offline: 'Hors ligne – tu vois la dernière version chargée.',
    ladenFehler: 'Le tableau blanc n’a pas pu être ouvert.',
    confirmDelete: 'Supprimer le tableau blanc « {title} » ?',
    loeschen: 'Supprimer le tableau blanc',
    laedt: 'Chargement de la zone de dessin …',
    einbinden: 'Intégrer',
    alleFreigaben: 'Tous les partages',
    keineFreigaben: 'Rien n’est encore partagé dans ce hub. Le partage se fait sous « Partages ».',
    leer: 'Il n’y a rien ici qui puisse être intégré.',
    nichtMehrGeteilt: 'Plus partagé',
    karteNotiz: 'Note',
    kartePdf: 'PDF',
    karteDatei: 'Fichier',
  },
};
