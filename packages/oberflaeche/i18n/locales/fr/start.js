// Startseite openany.de (frontend/start.html).
// Achtung vue-i18n: kein rohes @, | oder { in den Texten (außer Platzhaltern).
export default {
  seitentitel: 'openany – privé, chiffré, d’abord sur ton appareil',
  kopf: 'Tes notes, rendez-vous, fichiers et messages – au même endroit.',
  unterzeile: 'Privé, chiffré de bout en bout et d’abord sur ton appareil. Gratuit, sans publicité, sans mot de passe.',
  ladenFuer: 'Télécharger pour {system}',
  alleDownloads: 'Tous les téléchargements',
  iosFolgt: 'openany pour iPhone et iPad arrive bientôt.',
  merkmale: {
    verschluesselt: {
      titel: 'Toi seul peux le lire',
      text: 'Tout est chiffré sur ton appareil avant d’atteindre le serveur. Nous ne pouvons pas lire tes contenus – même si nous le voulions.',
    },
    geraet: {
      titel: 'D’abord sur ton appareil',
      text: 'openany fonctionne sans internet et sans compte. La synchronisation via openany.de est une offre, pas une obligation.',
    },
    alles: {
      titel: 'Tout au même endroit',
      text: 'Notes, agenda, contacts, fichiers, photos, vidéos et messages – dans une seule app au lieu de dix.',
    },
    hubs: {
      titel: 'Ensemble dans des hubs',
      text: 'Partage des dossiers, agendas et albums, dessinez ensemble sur un tableau et discutez dans le chat.',
    },
    sparsam: {
      titel: 'Ni mot de passe, ni adresse e-mail',
      text: 'Ton compte est lié à tes appareils. Nous n’enregistrons ni adresses e-mail ni adresses IP.',
    },
    frei: {
      titel: 'Gratuit et sans publicité',
      text: 'Pas d’abonnement, pas de publicité, aucune vente de données. Chaque compte reçoit au départ 100 Mo de stockage sur openany.de.',
    },
  },
  downloadsTitel: 'Téléchargements',
  version: 'Version {version}',
  empfohlen: 'Pour ton appareil',
  beiGithub: 'Les fichiers sont hébergés sur GitHub.',
  keineListe: 'La liste actuelle ne peut pas être chargée pour le moment.',
  dateien: {
    apk: 'Télécharger l’APK',
    installer: 'Programme d’installation (.exe)',
    msi: 'Paquet MSI',
    dmg: 'Télécharger pour Mac (.dmg)',
    appimage: 'AppImage',
    deb: 'Debian et Ubuntu (.deb)',
  },
  hinweise: {
    android: 'Directement en fichier, pas via le Play Store. La première fois, Android demande si tu autorises les apps de cette source.',
    windows: 'Pour Windows 10 et 11. L’app n’est pas encore signée : si Windows affiche un avertissement, choisis « Informations complémentaires », puis « Exécuter quand même ».',
    macos: 'À partir de macOS 11, pour puce Apple et Intel. L’app n’est pas encore notariée par Apple : au premier lancement, autorise-la dans Réglages système, Confidentialité et sécurité, « Ouvrir quand même ».',
    linux: 'AppImage pour presque toutes les distributions, .deb pour Debian et Ubuntu. 64 bits.',
  },
  fuss: {
    melden: 'Signaler et contact',
    agb: 'Conditions d’utilisation',
    datenschutz: 'Confidentialité',
    impressum: 'Mentions légales',
    github: 'Toutes les versions sur GitHub',
  },
  recht: {
    zurueck: 'Retour à l’accueil',
    nurDeutsch: 'Ce texte ne fait foi que dans sa version allemande.',
    fehler: 'Le texte ne peut pas être chargé pour le moment. Réessaie plus tard.',
  },
};
