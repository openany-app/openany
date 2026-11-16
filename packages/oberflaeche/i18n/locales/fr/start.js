// Startseite openany.de (frontend/start.html).
// Achtung vue-i18n: kein rohes @, | oder { in den Texten (außer Platzhaltern).
export default {
  seitentitel: 'openany – privé, chiffré, d’abord sur ton appareil',
  kopf: 'Tes notes, rendez-vous, fichiers et messages – au même endroit.',
  unterzeile: 'Privé, chiffré de bout en bout et d’abord sur ton appareil. Gratuit, sans publicité, sans mot de passe.',
  ladenFuer: 'Télécharger pour {system}',
  alleDownloads: 'Tous les téléchargements',
  iosFolgt: 'openany pour iPhone et iPad est en préparation.',
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
      text: 'Tu crées ton compte directement dans l’app, et il est lié à tes appareils. Nous n’enregistrons ni adresses e-mail ni adresses IP.',
    },
    frei: {
      titel: 'Gratuit et sans publicité',
      text: 'Pas d’abonnement, pas de publicité, aucune vente de données. Chaque compte reçoit au départ 100 Mo de stockage sur openany.de.',
    },
  },
  downloadsTitel: 'openany pour ton appareil',
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
    macos: 'À partir de macOS 13.3, pour puce Apple et Intel. L’app n’est pas encore notariée par Apple : au premier lancement, autorise-la dans Réglages système, Confidentialité et sécurité, « Ouvrir quand même ».',
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
  plattformen: 'Pour Android, Windows, macOS et Linux. iPhone et iPad sont en préparation.',
  kopfAlt: [
    'Écran d’accueil de l’app avec aujourd’hui, carnet d’adresses, notes, stockage et hubs',
    'Album de la médiathèque avec des paysages',
  ],
  rundgangKnopf: 'Faire la visite',
  rundgangLabel: 'Visite',
  sprung: {
    notizen: 'Notes',
    kalender: 'Agenda',
    hubs: 'Hubs',
    nachrichten: 'Messages',
    mediathek: 'Médiathèque',
    dokumente: 'Documents',
    downloads: 'Téléchargements',
  },
  rundgang: {
    notizen: {
      augenbraue: 'Notes',
      titel: 'Écrire et relier tes idées',
      abschnitte: [
        {
          titel: 'Écrire, ranger, relier',
          absaetze: [
            'Les notes sont rangées dans des classeurs et écrites en Markdown – avec titres, listes et tâches à cocher.',
          ],
          punkte: [
            'Les `[[wikiliens]]` relient les notes entre elles',
            'Les `#tags` regroupent ce qui va ensemble',
            'La recherche plein texte trouve chaque mot',
          ],
          alt: [
            'Note avec liste, wikiliens, tags et tâches',
          ],
        },
        {
          titel: 'Voir comment tout est lié',
          absaetze: [
            'La vue « Liens » montre tes notes sous forme de réseau. Choisis un classeur ou un tag, et les notes correspondantes apparaissent.',
          ],
          alt: [
            'Liste des notes avec classeurs et notes, noms floutés',
            'Graphe des liens entre les notes, étiquettes floutées',
          ],
        },
      ],
    },
    kalender: {
      augenbraue: 'Agenda',
      titel: 'Des rendez-vous pour toi et pour ceux avec qui tu t’organises',
      abschnitte: [
        {
          titel: 'Ta semaine d’un coup d’œil',
          absaetze: [
            'Mois, semaine ou jour : des rendez-vous avec lieu, notes et répétition. Un appui sur le jour suffit.',
          ],
          punkte: [
            'Plusieurs agendas côte à côte',
            '« Aujourd’hui » dès l’écran d’accueil',
            'Chiffré comme tout le reste',
            'Répétitions comme « toutes les 2 semaines le mardi, 5 fois »',
          ],
          alt: [
            'Vue mensuelle avec des rendez-vous d’exemple',
          ],
        },
        {
          titel: 'Partager des agendas dans les hubs',
          absaetze: [
            'Avec « consulter », les membres voient tes rendez-vous ; avec « modifier », ils en ajoutent eux-mêmes. Toi seul modifies l’agenda lui-même.',
          ],
          alt: [
            'Boîte de dialogue pour partager un agenda dans un hub',
            'Hub avec un classeur, un agenda, un album et un dossier partagés',
          ],
        },
        {
          titel: 'Dans le calendrier et les contacts de ton appareil',
          absaetze: [
            'Sur Android, tes calendriers et contacts openany apparaissent dans les apps du téléphone – et ce que tu y modifies revient. Sur ordinateur, openany les propose aux logiciels d’agenda et de contacts, uniquement sur cet ordinateur.',
          ],
          punkte: [
            'Rappels d’événements sur le téléphone',
            'Sur ordinateur via CalDAV et CardDAV, par exemple pour Thunderbird',
          ],
        },
      ],
    },
    hubs: {
      augenbraue: 'Hubs',
      titel: 'Ensemble plutôt que côte à côte',
      abschnitte: [
        {
          titel: 'Un lieu pour ton groupe',
          absaetze: [
            'Un hub appartient à une association, une colocation, une famille ou un projet. Tu invites les membres par leur nom openany. Le nom et le chat sont chiffrés de bout en bout.',
          ],
          punkte: [
            'Partager classeurs, agendas, albums et dossiers',
            'Pour chaque partage : consulter ou modifier',
          ],
          alt: [
            'Liste des membres d’un hub avec recherche pour inviter',
          ],
        },
        {
          titel: 'Un board pour tous',
          absaetze: [
            'Chaque hub a un board commun. Tous les membres y dessinent et écrivent – via openany.de ou directement à proximité, sur le même réseau.',
          ],
          punkte: [
            'Formes, flèches, texte et stylo',
            'Insérer des images',
            'Les modifications simultanées sont fusionnées',
          ],
          alt: [
            'Board avec un plan de parcelles dessiné',
          ],
        },
        {
          titel: 'S’organiser dans le chat du hub',
          absaetze: [
            'Chaque hub a son propre chat. Avec `[[`, tu relies le contenu du hub directement dans un message.',
          ],
          punkte: [
            'Chiffré de bout en bout',
            'Épingler les messages importants',
            'Prévenir les membres si tu le souhaites',
          ],
          alt: [
            'Chat du hub avec trois messages d’exemple',
          ],
        },
      ],
    },
    nachrichten: {
      augenbraue: 'Messages',
      titel: 'Tous les canaux dans une seule liste',
      abschnitte: [
        {
          titel: 'Messages directs, chiffrés',
          absaetze: [
            'Les messages entre comptes openany sont chiffrés de bout en bout, pièces jointes comprises. Matrix et l’e-mail s’y ajoutent – tout dans une seule liste.',
          ],
          punkte: [
            'Filtres : non lus, avec pièce jointe, openany, à proximité',
            'À proximité, même sans aucun serveur',
            'Par conversation ou par contact',
          ],
          alt: [
            'Liste des messages avec recherche et filtres, en partie floutée',
          ],
        },
        {
          titel: 'Matrix : joindre aussi ceux qui n’ont pas openany',
          absaetze: [
            'Connecte un compte sur matrix.org ou un autre serveur. Ton appareil se connecte lui-même, avec ses propres clés.',
          ],
          punkte: [
            'openany.de ne lit rien',
            'L’historique reste sur l’appareil',
            'Le mot de passe n’est pas enregistré',
          ],
          alt: [
            'Paramètres : connecter un compte Matrix',
          ],
        },
        {
          titel: 'L’e-mail, avec PGP si tu veux',
          absaetze: [
            'Ta boîte mail existante devient le troisième canal de la liste. L’appareil parle lui-même au serveur de messagerie.',
            'Les clés OpenPGP publiques de tes contacts arrivent avec leurs e-mails – elles servent à chiffrer tes réponses.',
          ],
          alt: [
            'Paramètres : connecter une boîte mail',
            'Paramètres : clés des contacts',
          ],
        },
      ],
    },
    mediathek: {
      augenbraue: 'Médiathèque',
      titel: 'Tes images, vidéos et enregistrements',
      abschnitte: [
        {
          titel: 'Photos, vidéos et son en albums',
          absaetze: [
            'Des albums et sous-albums, remplis directement depuis l’app – avec l’appareil photo, la vidéo ou le micro. Tu partages les albums dans les hubs.',
          ],
          punkte: [
            'Les miniatures sont créées sur l’appareil',
            'Les données de lieu conservées si tu veux',
            'Enregistré chiffré',
          ],
          alt: [
            'Album de la médiathèque avec des paysages',
          ],
        },
        {
          titel: 'Beaucoup de fichiers d’un coup',
          absaetze: [
            'Reprends d’un coup toutes les photos et vidéos de ton téléphone – en copie ou sans : elles restent alors sur le téléphone et partent chiffrées vers openany.de.',
          ],
          punkte: [
            'Nouvelles photos automatiquement, si tu veux',
            'Partager vers openany depuis d’autres apps',
            'Sur ordinateur, glisser fichiers et dossiers dans la fenêtre ou synchroniser un dossier',
          ],
        },
      ],
    },
    dokumente: {
      augenbraue: 'Documents',
      titel: 'Des papiers où l’on peut chercher',
      abschnitte: [
        {
          titel: 'Les papiers dans des classeurs',
          absaetze: [
            'Lettres, contrats et documents sont rangés dans des classeurs. Les PDF s’ouvrent directement dans openany : feuilleter, chercher, remplir des formulaires, surligner et commenter.',
          ],
          punkte: [
            'Recherche aussi dans le texte des PDF et fichiers bureautiques',
            'Partager des classeurs dans les hubs',
          ],
          alt: [
            'Documents avec une liste de classeurs, noms privés floutés',
          ],
        },
        {
          pille: 'Reconnaissance de texte',
          titel: 'De la photo au PDF consultable',
          absaetze: [
            'Photographie une lettre et range-la dans un classeur. Elle devient un PDF avec une couche de texte invisible : chercher, surligner, copier.',
            'La reconnaissance se fait sur ton appareil. L’image ne va nulle part.',
          ],
          alt: [
            'Message indiquant la conversion d’une photo',
            'Lettre photographiée en PDF, la recherche surligne un mot',
          ],
        },
        {
          pille: 'PDF',
          titel: 'Chaque note en PDF',
          absaetze: [
            'Un appui transforme une note en PDF et la range dans tes documents. Quatre versions au choix :',
          ],
          punkte: [
            'Standard, avec titre, date et sources',
            'Clean, juste le texte',
            'Lettre au format DIN 5008',
            'Version imprimable avec page de titre',
          ],
          alt: [
            'Boîte de dialogue de création de PDF avec quatre versions',
            'Note sous forme de PDF dans le lecteur',
          ],
        },
      ],
    },
  },
  versprechen: {
    augenbraue: 'La promesse',
    titel: 'Toi seul peux le lire.',
    text: 'Tout est chiffré sur ton appareil avant d’atteindre le serveur. Hébergé en Allemagne, sans publicité et sans pistage.',
  },
  downloadsAugenbraue: 'Téléchargements',
  ios: {
    titel: 'iPhone et iPad',
    pille: 'En préparation',
    text: 'L’app iOS est en préparation. Dès qu’elle sera prête, tu la trouveras ici.',
  },
  alleVersionen: {
    titel: 'Toutes les versions',
    text: 'Les anciennes versions et tous les fichiers sont sur GitHub.',
    knopf: 'Vers GitHub',
  },
};
