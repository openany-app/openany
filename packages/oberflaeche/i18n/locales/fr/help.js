// Attention vue-i18n : pas de @, | ou { bruts dans les textes.
export default {
  title: 'Aide',
  intro: 'Tu trouveras ici des descriptions et des guides pour tous les modules et fonctions d’Openany. Le sommaire à gauche te mène directement à la bonne section.',
  version: 'Version {version}',
  aktualisieren: 'Mets à jour l’app',
  aktualisierenTitel: 'openany.de nécessite au moins la version {version}.',
  toc: 'Sommaire',
  sections: {
    start: {
      title: 'Premiers pas',
      subs: {
        ohneKonto: {
          title: 'L’application sans compte',
          p1: 'L’application est un programme complet qui fonctionne aussi sans compte ni serveur : notes, agenda, stockage, carnet d’adresses et messages restent alors uniquement sur cet appareil. Ce qui n’est pas configuré ici ou ne fonctionne pas, l’application le masque au lieu de l’afficher vide – la voie Openany des messages, par exemple, n’apparaît qu’une fois l’appareil connecté à openany.de.',
          p2: 'Si tu veux synchroniser tes données avec openany.de, crée un compte dans les réglages sous « Synchronisation » ou connecte-toi à un compte existant ; tu associes tes autres appareils sous « Appareils à proximité ». Ensuite, l’en-tête et le menu affichent un bouton « Synchroniser » qui tourne pendant la synchronisation.',
        },
        kontoAnlegen: {
          title: 'Créer un compte',
          p1: 'Tu crées un compte sur openany.de directement dans l’app : dans les réglages, sous « Synchronisation », appuie sur « Créer un compte » et choisis un nom. Ton appareil résout ensuite un petit calcul qui protège contre la création de comptes en masse ; cela prend généralement moins d’une minute. Tu n’as besoin ni d’adresse e-mail ni de mot de passe.',
          p2: 'Ensuite, tu acceptes les conditions d’utilisation – ce n’est qu’alors que tu disposes d’espace sur openany.de, 100 Mo au départ. Tu connectes d’autres appareils via un appareil déjà connecté, avec 6 chiffres (voir « Mes appareils »).',
          p3: 'Comme ton compte n’a pas de mot de passe, un fichier de sauvegarde est le seul moyen de revenir si tu perds ton unique appareil. L’app te le propose juste après la création ; crée-le et conserve-le en lieu sûr avec sa phrase secrète (voir « Sauvegarde dans un fichier »).',
        },
        startseite: {
          title: 'L’accueil',
          p1: 'L’accueil affiche des tuiles compactes des modules que tu as choisis : les rendez-vous du jour, le carnet d’adresses avec recherche, les trois notes modifiées le plus récemment, les trois derniers fichiers avec l’espace occupé et tes hubs. Chaque tuile mène à son module ; tu choisis lesquelles apparaissent dans les paramètres, sous « Modules sur l’accueil ».',
          p2: 'Au début, toutes les tuiles sont choisies. Si tu les retires toutes, la page de bienvenue apparaît à la place : elle présente toutes les fonctions et te permet de composer directement les tuiles. La sélection ne masque rien – tous les espaces restent accessibles via la barre et le menu.',
        },
        navigation: {
          title: 'Menu et structure',
          p1: 'Sur ordinateur, un en-tête se trouve en haut : à gauche, le logo rond mène à l’accueil, à côté se trouvent Notes, Agenda, Stockage et Hubs. À droite suivent le bouton clair/sombre, l’enveloppe des messages avec compteur de non-lus et le champ avec ton nom – il ouvre Paramètres, Aide, Corbeille et Déconnexion.',
          p2: 'Sur mobile, tout passe en bas : une barre fixe affiche les modules sous forme d’icônes, et tout à gauche se trouve le bouton de menu avec le compteur de messages non lus. Il ouvre un panneau avec Messages, Paramètres, Aide, Corbeille, le bouton clair/sombre et la déconnexion. Le bouton alterne entre clair, sombre et automatique – automatique suit le réglage de ton appareil. Toucher le module déjà ouvert te ramène à son point de départ.',
        },
      },
    },
    notes: {
      title: 'Notes',
      subs: {
        editor: {
          title: 'Écrire dans l’éditeur',
          p1: 'L’éditeur de notes fonctionne comme un traitement de texte moderne : titres, listes, citations, blocs de code, gras et italique se choisissent dans la barre d’outils. Tu peux aussi taper des raccourcis Markdown – par exemple un dièse suivi d’un espace pour un titre ou un tiret pour une liste – qui se transforment en mise en forme pendant la frappe.',
          p2: 'L’enregistrement est automatique : peu après que tu as cessé de taper, Openany sauvegarde la note et affiche l’état dans la barre d’outils. Sous chaque note figurent aussi la date de création et celle de la dernière modification. Sur mobile, la barre d’outils apparaît sur une seule ligne juste au-dessus du clavier dès que tu touches le texte, et défile latéralement – tous les outils sont à portée de main sans gêner la lecture.',
        },
        mappen: {
          title: 'Classeurs et structure',
          p1: 'Les notes sont rangées dans des classeurs imbricables à volonté. Tu peux renommer, déplacer et réorganiser notes et classeurs – ainsi se construit peu à peu ta propre base de connaissances. La loupe en haut ouvre un champ de recherche qui parcourt toutes les notes à la fois, titre et contenu ; ouverte dans un classeur, « uniquement dans ce classeur » la limite à celui-ci. Ce que tu peux faire avec le classeur ouvert se trouve sous « … », à droite de la barre de chemin.',
          p2: 'Tu remontes par le chemin ou avec la touche Retour, qui monte d’un niveau ; la croix dans la barre de chemin ferme une note ouverte. À côté de chaque classeur, tu vois quand une note qu’il contient a changé pour la dernière fois, sous-classeurs compris. Un classeur sans notes n’affiche pas de date.',
        },
        wikilinks: {
          title: 'Wikiliens et rétroliens',
          p1: 'Avec deux crochets, tu relies des notes entre elles : tape les crochets et le titre de la note cible, et le lien se crée automatiquement – si la note n’existe pas encore, le lien devient actif dès qu’elle existe. Si le texte doit afficher autre chose que le titre, tu indiques un texte affiché personnalisé après une barre verticale ; le lien pointe toujours vers la bonne note.',
          p2: 'Chaque note affiche ses rétroliens : la liste de toutes les notes qui pointent vers elle. Tu retrouves ainsi les liens entre les idées sans devoir tenir de registre.',
        },
        tags: {
          title: 'Tags',
          p1: 'Les mots-clés s’attribuent directement dans le texte avec un dièse devant le mot ; pendant la frappe, une liste propose les tags déjà utilisés. Avec une barre oblique, tu imbriques les mots-clés – par exemple Hub/Sous-thème ; un filtre sur le thème principal trouve alors aussi tous les sous-thèmes. Dans la vue graphe, les mots-clés peuvent s’afficher comme nœuds distincts, et un clic sur un mot-clé y filtre la liste des notes.',
        },
        einfuegen: {
          title: 'Insérer des images et des fichiers',
          p1: 'Le bouton trombone de la barre d’outils insère dans une note un fichier de ton appareil. Les images apparaissent sous forme d’aperçu compact dans la note (la pleine résolution se trouve dans la Médiathèque), les autres fichiers sous forme de lien à ouvrir.',
          p2: 'Les contenus insérés sont rangés automatiquement : les images dans un album de la Médiathèque, les fichiers texte et Office dans un classeur de Documents, tout le reste dans un dossier de fichiers – chaque fois dans un espace dédié aux notes.',
        },
        zitate: {
          title: 'Citations et bibliographie',
          p1: 'Pour le travail universitaire, chaque classeur tient une liste de sources : dans le classeur ouvert, choisis « Sources » sous « … » et saisis livres, articles, sites web, décisions de justice ou lois. Chaque source reçoit un nom court. La liste est un fichier CSL-JSON dans Fichiers, utilisable aussi avec Zotero et Zettlr.',
          p2: 'Dans les notes de ce classeur, l’éditeur propose des sources adaptées quand tu cites et insère la citation dans le texte. Les citations naissent ainsi pendant l’écriture, sans ressaisir les références à chaque fois. Leur apparence dans le PDF est décrite sous « Version imprimable ».',
        },
        graph: {
          title: 'Vue graphe',
          p1: 'La vue graphe dessine tes notes comme un réseau : chaque note est un point, chaque wikilien une connexion. Les points sont colorés selon le classeur de niveau supérieur, si bien que les ensembles liés se reconnaissent à la couleur ; les points creux représentent des notes sans aucun lien, et plus une note a de liens, plus son point est gros.',
          p2: 'Chaque classeur de premier niveau est un niveau. S’il y en a plusieurs, tu en choisis un ou plusieurs dans la légende sous le graphe – « Tous les niveaux » les prend tous, « Aucun » vide la sélection. Au lieu d’un niveau, un tag peut aussi servir d’entrée : la barre en dessous liste les tags avec leur nombre, et un tag choisi montre toutes les notes qui le portent, avec le tag en losange gris. Dès que des niveaux sont choisis, la barre ne liste plus que les tags qui y figurent. Tant que tu ne choisis rien, le graphe reste vide, afin que les grandes collections ne soient pas construites d’un coup ; « Afficher tous les tags » ajoute d’un coup tous les tags de la barre.',
          p3: 'Sur ordinateur, tu zoomes avec les boutons plus/moins en haut à droite ou la molette, et tu déplaces la vue en maintenant le bouton de la souris ; sur mobile, tu zoomes à deux doigts et déplaces avec un seul. Un autre bouton réinitialise la vue. Un clic sur un point (sans glisser) ouvre la note. Un clic sur un tag filtre la liste des notes en conséquence.',
        },
        export: {
          title: 'Export PDF',
          p1: 'Le bouton PDF de la note ouverte propose quatre versions : « Standard » est la note avec titre, date et bibliographie. « Clean » n’est que le texte, sans titre, date, numéros de page ni bibliographie ; les citations restent telles que tu les as écrites. « Lettre » met la note en page selon DIN 5008 pour enveloppe à fenêtre, avec le titre comme objet. « Version imprimable » produit cette seule note avec une page de titre. Le PDF va dans le classeur « Notizen » sous Stockage → Documents (les versions précédentes sont conservées), la version imprimable dans le classeur « Druckfassungen » ; « Ouvrir » dans le message l’affiche aussitôt.',
        },
        druckfassung: {
          title: 'Version imprimable',
          p1: 'Un dossier devient un document formel à rendre ou à imprimer : dans le dossier ouvert, choisis « Version imprimable » sous « … », saisis titre, sous-titre, nom, établissement, date et informations complémentaires pour la page de titre, et décide d’ajouter ou non une table des matières. Chaque note du dossier devient un chapitre, par ordre alphabétique du titre ; des préfixes comme « 01 Introduction » fixent l’ordre, et le document n’affiche alors que « Introduction ». Les sous-classeurs sont inclus si « Inclure les sous-classeurs » est coché : un sous-classeur devient un titre et ses notes passent en dessous (« 2 Bases », « 2.1 Notions ») ; une note portant le nom de son sous-classeur apparaît comme texte juste sous ce titre.',
          p2: 'La composition utilise une police de type Times avec des titres sans empattement, des marges de 2,5 cm, un interligne de 1,5, un texte justifié avec césure ; la numérotation commence après la page de titre. Les informations restent enregistrées avec le dossier. Le PDF est créé sur ton appareil – la première fois prend quelques secondes – et va dans le dossier « Druckfassungen » sous Stockage → Documents ; chaque version est conservée.',
          p3: "Citations de la liste de sources du classeur : dans le texte, écris [{'@'}nomcourt] ou [{'@'}nomcourt, p. 12], et ^[Texte] pour une note de bas de page. Dans le PDF – note ou version imprimable – cela devient des citations Harvard comme « (Müller, 2020, p. 12) » avec la bibliographie à la fin.La barre d’outils de l’éditeur a un bouton pour les notes de bas de page (chiffre en exposant) : il insère les crochets, et le texte sélectionné devient la note.",
        },
        papierkorb: {
          title: 'Corbeille',
          p1: 'Les notes supprimées vont dans la corbeille (dans le menu sous ton nom, sur mobile dans le bouton de menu) et peuvent y être restaurées ou supprimées définitivement. Lorsque tu supprimes un classeur, tu décides si les notes qu’il contient sont supprimées aussi ou déplacées dans le classeur parent.',
        },
        verschluesselt: {
          title: 'Chiffré de bout en bout',
          p1: 'Tes notes, dossiers, indications de la version imprimée et listes de sources ne sont stockés sur openany.de que chiffrés : seuls tes appareils autorisés peuvent les lire. Le serveur ne voit pas non plus les noms de tes dossiers. La recherche, les rétroliens, les tags et le graphe sont calculés sur ton appareil.',
        },
        ordner: {
          title: 'Notes en fichiers (Zettlr)',
          p1: 'Dans l’app sur ton ordinateur, tu peux relier un de tes classeurs à un dossier : ouvre le classeur, choisis « Relier à un dossier … » dans le menu « … » et sélectionne le dossier. Chaque note y est un fichier « Id Titre.md », les sous-classeurs des sous-dossiers – comme Zettlr ou Obsidian l’attendent. Ce que tu y modifies, renommes, déplaces ou crées est repris par l’app tant qu’elle tourne ; si les deux côtés ont changé, une copie de conflit est créée. Si tu supprimes un fichier, la note va à la corbeille. Le dossier est stocké sans chiffrement sur ton ordinateur ; « Délier le dossier » laisse les fichiers en place. Les images des notes se trouvent à côté de la note, sous le nom utilisé dans le texte ; une nouvelle image placée là et citée par une note est reprise dans l’app (médiathèque, album « Notizen: … »).',
        },
      },
    },
    files: {
      title: 'Stockage',
      subs: {
        aufbau: {
          title: 'Médiathèque, fichiers, documents',
          p1: 'Le stockage se divise en trois onglets en haut de la page : la Médiathèque pour les photos et vidéos, Fichiers pour tout le reste et Documents pour les papiers rangés en classeurs. Quand tu changes d’onglet, l’autre espace reste tel que tu l’as laissé – dossiers ouverts et listes chargées ne sont pas perdus.',
          p2: 'L’onglet qui s’ouvre en premier quand tu ouvres le stockage se règle dans les paramètres, sous « Démarrage du stockage » ; tant que tu n’as rien choisi, ce sont les Documents.',
        },
        dokumente: {
          title: 'Archivage de documents',
          p1: 'Dans l’onglet « Documents », tu ranges fichiers PDF, Office et texte dans des classeurs. Le téléversement se fait par bouton ou en glissant simplement les fichiers dans le classeur ouvert. Sur téléphone et tablette, un bouton appareil photo se trouve à côté : il photographie un papier et le range aussitôt comme document dans le classeur.',
          p2: 'Les classeurs s’imbriquent à volonté. Tu peux renommer et déplacer classeurs et documents ; des classeurs entiers peuvent aussi être partagés avec un hub sous « Partages ». À côté de chaque classeur, tu vois quand quelque chose y a changé pour la dernière fois, sous-classeurs compris.',
        },
        texterkennung: {
          title: 'Documents interrogeables',
          p1: 'Si tu photographies un document papier et le ranges dans un classeur – avec le bouton appareil photo ou comme photo de ta collection d’images –, il devient automatiquement un PDF doté d’une couche de texte invisible. Le document ressemble à ta photo, mais on peut y rechercher, surligner, copier et le faire lire à voix haute. La reconnaissance s’exécute sur ton propre appareil – l’image n’est envoyée nulle part.',
          p2: 'Si tu ranges plusieurs photos à la fois, on te demande s’il faut en faire un document de plusieurs pages – comme les pages d’une lettre – ou des documents séparés. La photo elle-même n’est pas conservée en plus ; pour garder l’image, range-la dans la Médiathèque. La première conversion est plus longue, car la reconnaissance de texte est chargée une seule fois puis conservée.',
          p3: 'Si un PDF scanné se trouve déjà dans un classeur, tu peux lui appliquer la reconnaissance après coup : « Reconnaître le texte » lit le document page par page. Si le papier est bien blanc, les pages restent inchangées et seul le texte est ajouté. Si elles sont grises ou éclairées de façon inégale – comme une feuille photographiée –, elles sont en plus éclaircies ; le message final t’indique ce qui s’est passé. Si le PDF était déjà interrogeable, il n’est pas modifié.',
        },
        dateien: {
          title: 'Gérer les fichiers',
          p1: 'Dans l’onglet « Fichiers », tu crées des dossiers et téléverses des fichiers de tout type – un par un ou plusieurs à la fois. Toucher un fichier l’ouvre avec une application adaptée de ton appareil. L’icône de téléchargement enregistre une copie dans le dossier Download ou, sur ordinateur, à l’endroit de ton choix ; pour les PDF, elle se trouve dans la visionneuse. Dossiers et fichiers se renomment et se déplacent ; les dossiers peuvent aussi être partagés avec un hub sous « Partages ». Ce que tu peux faire avec un dossier ou un classeur ouvert se trouve sous « … », à droite de la barre de chemin. Tu remontes par le chemin ou avec la touche Retour, qui monte d’un niveau.',
          p2: 'Tu vois combien tu occupes sur la tuile de l’accueil. Ce qui reste sur cet appareil se règle dans les paramètres sous « Stockage sur cet appareil ».',
        },
        suche: {
          title: 'Rechercher',
          p1: 'La loupe dans Fichiers et Documents ouvre un champ de recherche qui parcourt tous les dossiers ou classeurs à la fois, pas seulement celui qui est ouvert. Un clic sur un résultat ouvre le fichier ; à côté figure son emplacement, et un clic dessus t’emmène dans le dossier.',
          p2: 'La recherche ne trouve pas seulement par nom, mais aussi dans le texte : dans les PDF, les textes Word et OpenDocument ainsi que les fichiers texte, CSV et Markdown jusqu’à 50 Mo. Openany lit ce texte petit à petit en arrière-plan tant que le stockage est ouvert ; tant que tout n’est pas lu, la recherche t’indique combien de fichiers ne se trouvent pour l’instant que par leur nom. Pour un résultat dans le texte, la liste montre le passage avec le mot cherché mis en évidence. Les PDF scannés sans couche de texte n’apparaissent qu’après leur avoir appliqué « Reconnaître le texte ».',
        },
        pdf: {
          title: 'Afficher et modifier des PDF',
          p1: 'Un clic sur un PDF l’ouvre directement dans Openany. Tu feuillettes les pages, agrandis ou réduis, ajustes l’affichage à la largeur et recherches dans le document avec la loupe – les résultats sont mis en évidence sur la page. « Ouvrir avec une autre application » le transmet à une autre application. Si un PDF est protégé par mot de passe, Openany te le demande d’abord.',
          p2: 'Tu remplis directement les formulaires du PDF. « Modifier » fait en plus apparaître une barre d’outils : « Surligner » met du texte en couleur, le « Stylo » permet de dessiner librement sur la page et « Texte » pose tes propres lignes sur la feuille – couleur, épaisseur du trait et taille de police sont à ton choix, et chaque étape peut être annulée. Les champs de calcul des formulaires ne calculent toutefois pas, car Openany n’exécute pas de scripts provenant de fichiers tiers.',
          p3: 'Tu peux ajouter des commentaires aux surlignages et aux traits : choisis « Commentaires » et touche l’endroit ; la même vue liste tous les commentaires du document. Tout est enregistré sous forme d’annotations PDF ordinaires, que d’autres programmes affichent aussi. Au premier enregistrement, Openany place la version précédente dans la corbeille, pour que tu puisses revenir à l’original ; si tu quittes le PDF avec des modifications non enregistrées, il te le demande d’abord.',
        },
        galerie: {
          title: 'Médiathèque et albums',
          p1: 'L’onglet « Médiathèque » contient tes photos et vidéos – dans des albums avec vignettes ou, sans album, en dessous sous « Médias ». Les albums s’imbriquent – par exemple un album par voyage avec un sous-album par jour – et se déplacent à tout moment. La visionneuse fait défiler toutes les images d’un album. Tu déplaces une photo depuis la grande vue ; un appui long dans la grille en sélectionne plusieurs, à déplacer ou supprimer ensemble. Ce que tu peux faire avec l’album ouvert se trouve sous « … », à droite de la barre de chemin.',
          p2: 'La prise de vue la plus récente est en haut, regroupée par mois avec un titre pour chaque mois. C’est la date de prise de vue des données de la photo qui compte ; si elle manque – captures d’écran ou images retouchées, par exemple –, c’est le jour du téléversement.',
          p3: 'Dans la grande vue, tu télécharges une photo ou une vidéo en fichier original : dans le dossier Download sur mobile, à l’endroit de ton choix sur ordinateur. Les albums peuvent être partagés avec un hub sous « Partages ». Les originaux restent chez toi. Via « Actions » dans l’album, tu importes ta propre image de couverture ; elle ne compte pas comme élément.',
        },
        fotosVomGeraet: {
          title: "Photos de l’appareil",
          p1: "Sous Android, dans la médiathèque, « Ajouter » → « Ajouter les photos de l’appareil » reprend les photos et vidéos déjà présentes sur l’appareil : toutes, seulement certains albums de l’appareil, ou à partir d’une date. Les albums de l’appareil deviennent des albums du même nom dans openany ; avant, tu vois si la place suffit, et ce qui est déjà repris est ignoré. Si l’appareil est relié à openany.de, tu choisis une fois comment les photos arrivent : sur openany.de, et seul l’aperçu reste sur l’appareil, sans place prise deux fois – ou en copie sur cet appareil.",
          p2: "Avec « Ajouter automatiquement les nouvelles photos » – dans la médiathèque sous « Ajouter » ou dans Paramètres → Stockage et sauvegarde –, ce que l’appareil photo prend désormais arrive tout seul dans la médiathèque, même quand openany n’est pas ouvert : depuis les albums de l’appareil que tu choisis, à partir de l’activation. Au choix, seulement en Wi-Fi ou seulement pendant la charge. Android limite le travail en arrière-plan, les photos arrivent donc avec un peu de retard. Sur openany.de, elles comptent dans ton stockage. Si tu utilises Google Photos, désactive sa sauvegarde – sinon les deux envoient.",
        },
        videos: {
          title: 'Vidéos et appareil photo',
          p1: 'En plus des photos, la Médiathèque accepte aussi les vidéos. Elles apparaissent sous forme de tuile avec une image fixe ; les toucher les ouvre avec l’application vidéo de ton appareil.',
          p2: 'Sur téléphone et tablette, la Médiathèque et chaque album ont deux boutons appareil photo : l’un pour une photo, l’autre pour une vidéo. L’enregistrement arrive directement là où tu te trouves. Sur ordinateur, ces boutons n’existent pas ; tu téléverses comme d’habitude.',
        },
        ton: {
          title: 'Enregistrements audio et musique',
          p1: 'La médiathèque accueille aussi les fichiers audio, comme MP3 ou M4A. Ils apparaissent sous forme de tuile avec nom et durée ; un appui les lit dans l’app. Les fichiers audio venus de l’extérieur sont aussi rangés ici.',
          p2: 'Le bouton micro enregistre directement, avec pause, abandon et sauvegarde. L’enregistrement va dans l’album ouvert, sinon dans l’album « Enregistrements ». Sur Android, il continue écran éteint, comme l’indique la notification. Ce bouton n’existe pas sous Linux.',
          p3: 'Sur Android (version 10 ou plus) et Windows, tu peux choisir le son du système plutôt que le micro. Le système demande ton accord à chaque fois. Certaines applis l’interdisent et les appels ne sont jamais possibles. N’enregistre une conversation que si tout le monde est d’accord.',
        },
        karte: {
          title: 'Date et lieu de prise de vue',
          p1: 'Pour chaque image, la visionneuse affiche la date de prise de vue et – si la photo le contient – le lieu ; « Voir sur la carte » ouvre l’endroit sur OpenStreetMap. Si tu téléverses des photos depuis un téléphone Android, choisis-les via l’application Fichiers plutôt que via la galerie : sinon Android supprime la position avant que l’image n’arrive dans Openany.',
        },
        papierkorbSpeicher: {
          title: 'Corbeille',
          p1: 'Les contenus supprimés – documents, fichiers, photos, notes, contacts, mais aussi hubs, agendas et rendez-vous – passent d’abord dans la corbeille commune et peuvent y être restaurés. Au bout de 30 jours, la corbeille se vide automatiquement ; ce n’est qu’alors que l’espace est définitivement libéré.',
        },
        dateienVerschluesselt: {
          title: 'Chiffré de bout en bout',
          p1: 'Tes fichiers, classeurs, photos et albums ne sont stockés sur openany.de que chiffrés : noms, dossiers, albums, dates de prise de vue et lieu figurent dans un en-tête chiffré, et les contenus eux-mêmes sont chiffrés avec une clé propre à chaque fichier. Seuls tes appareils autorisés peuvent les lire.',
        },
      },
    },
    calendar: {
      title: 'Agenda',
      subs: {
        verwalten: {
          title: 'Plusieurs agendas',
          p1: 'Tu peux tenir autant d’agendas que tu veux côte à côte – par exemple perso, famille, club –, chacun avec sa couleur. Un clic affiche ou masque un agenda. En haut, tu passes du mois à la semaine et au jour ; semaine et jour affichent les rendez-vous par heure, et les rendez-vous passés apparaissent discrètement grisés. Sur téléphone et tablette, tu changes de page en balayant : vers le haut et le bas dans le mois, sur le côté dans la semaine et le jour. Sur ordinateur, il y a des flèches pour cela.',
        },
        termine: {
          title: 'Créer des rendez-vous',
          p1: 'Tu crées un rendez-vous en cliquant sur un jour : avec titre, description, heure, ou comme événement d’une journée entière ou de plusieurs jours. Les répétitions sont possibles aussi – quotidiennes, hebdomadaires, mensuelles ou annuelles. Un clic sur un rendez-vous affiche ses détails, où tu le modifies ou le supprimes.',
          p2: "Un rappel se manifeste à l’heure choisie avant le début. Si un rendez-vous vient d’un autre programme avec une répétition plus précise – toutes les deux semaines, seulement trois fois, le mardi et le jeudi, chaque premier lundi –, elle est conservée et décrite en toutes lettres sur le rendez-vous. Si tu choisis toi-même une autre répétition, elle remplace l’ancienne. Un rendez-vous déplacé isolément dans une série reste un rendez-vous à part, et la série laisse sa place d’origine libre.",
        },
        abos: {
          title: 'Abonnements d’agenda',
          p1: 'Les agendas externes – par exemple les jours fériés ou le calendrier des matchs du club – s’intègrent via une URL d’abonnement au format ICS. Les agendas abonnés se mettent à jour automatiquement. Leurs rendez-vous sont en lecture seule dans Openany : on les modifie là d’où vient l’agenda, car chaque mise à jour remplace de nouveau le contenu par l’original.',
        },
        verschluesselt: {
          title: 'Chiffré de bout en bout',
          p1: 'Tes agendas et rendez-vous ne sont stockés sur openany.de que sous forme chiffrée : seuls tes appareils autorisés peuvent les lire. L’adresse d’un abonnement est chiffrée elle aussi ; chaque appareil la récupère lui-même, openany.de ne récupère rien.',
        },
        importExport: {
          title: 'Importer et exporter',
          p1: 'Avec « Importer », tu charges un fichier ICS dans l’un de tes agendas. Si tu importes à nouveau le même fichier, les rendez-vous sont mis à jour au lieu d’être dupliqués. L’icône de partage à côté d’un agenda l’enregistre en fichier ICS – à emporter dans n’importe quelle autre application d’agenda.',
        },
        imTelefon: {
          title: "Dans le calendrier du téléphone",
          p1: "Sous Android, Paramètres → Calendrier → « Afficher dans le calendrier du téléphone » montre tes agendas dans toutes les applis d’agenda du téléphone, sous le compte « openany » – y compris ceux partagés dans des hubs, et les abonnements si tu les ajoutes. Ce que tu y modifies, crées ou supprimes revient dans openany. Les rappels sont alors affichés par le téléphone lui-même.",
          p2: "Google Agenda n’affiche les autres comptes qu’après une étape de ta part : là-bas, Paramètres → Gérer les comptes → activer « openany ». Les rendez-vous ne sont déchiffrés que sur cet appareil. Désactiver les retire du calendrier du téléphone ; tout reste dans openany.",
        },
        andereProgramme: {
          title: "Pour d’autres programmes sur l’ordinateur",
          p1: "Sur ordinateur, Paramètres → Calendrier → « Proposer aux autres programmes de cet ordinateur » propose tes agendas et ton carnet d’adresses via CalDAV et CardDAV – pour Thunderbird, Calendrier et Contacts d’Apple et d’autres. La tuile affiche adresse, utilisateur et mot de passe à copier ; avec eux, tu configures un compte dans le programme. Dans Thunderbird : Nouvel agenda → Sur le réseau, ou Nouveau carnet CardDAV. Chez Apple : Autre compte CalDAV ou CardDAV → Manuel. Si plusieurs comptes utilisateur partagent l’ordinateur, active-le seulement quand tu en as besoin : quand Openany ne tourne pas, un autre compte pourrait répondre à sa place et recevoir le mot de passe envoyé par ton programme.",
          p2: "Le service n’est accessible que sur cet ordinateur et seulement tant qu’openany est ouvert ; si l’appli est fermée, le programme rattrape plus tard. Les modifications faites là-bas reviennent dans openany ; les agendas partagés dans des hubs y sont en lecture seule. « Nouveau mot de passe » bloque les programmes déjà configurés jusqu’à ce qu’ils reçoivent le nouveau.",
        },
      },
    },
    projects: {
      title: 'Hubs',
      subs: {
        grundlagen: {
          title: 'Hubs et membres',
          p1: 'Un hub est un espace de travail commun : tu invites d’autres utilisateurs comme membres, qui acceptent ou refusent l’invitation. Le propriétaire du hub gère les membres et peut aussi transmettre le hub à un autre membre.',
          p2: 'Chaque hub a quatre onglets : Partages, Board, Membres et Chat. Il s’ouvre sur Partages. Les chiffres sur les onglets indiquent les messages de chat non lus ou le nombre d’éléments qu’ils contiennent.',
        },
        chat: {
          title: 'Chat du hub',
          p1: 'Chaque hub dispose d’un chat commun en temps réel : les messages apparaissent immédiatement chez tous les membres, sans recharger la page. Le chat est chiffré de bout en bout : seuls les membres peuvent le lire sur leurs appareils, pas openany.de. Si quelqu’un part, la clé change d’elle-même – il ne lit plus rien de nouveau.',
          p2: 'Chaque membre peut épingler des messages importants – les messages épinglés se retrouvent vite dans la barre d’épingles au-dessus de l’historique et peuvent y être détachés.',
          p3: 'Avec deux crochets, tu renvoies en pleine phrase à des contenus du hub : aux notes ainsi qu’aux dossiers, fichiers, albums et images partagés. Pendant la saisie, une liste de suggestions montre ce qui convient et insère le lien terminé. Le bouton avec le symbole de chaîne à côté du bouton d’envoi fait la même chose – sur téléphone, souvent plus pratique que de chercher deux crochets.',
          p4: 'Un clic sur un lien mène directement à la cible : dans la note, dans le dossier avec le fichier mis en évidence ou vers l’image dans la visionneuse. Tu ne peux renvoyer qu’à ce qui est déjà visible dans le hub – ce que personne n’a partagé n’apparaît ni dans la liste de suggestions ni comme lien cliquable.',
        },
        board: {
          title: 'Board',
          p1: 'Chaque hub a exactement un board – une surface de dessin commune, créée avec le hub. Un appui sur « Board » l’ouvre ; « Retour » ramène à l’onglet d’où tu venais. Tous les membres y dessinent, écrivent et organisent ; l’enregistrement se fait tout seul, peu après chaque modification. Avec « Intégrer », tu poses sur le board des images, notes et fichiers partagés du hub sous forme de carte.',
          p2: 'Si plusieurs personnes dessinent en même temps, l’app fusionne leurs modifications : chaque élément prend sa version la plus récente, et ce qui est supprimé le reste. Avec des membres à proximité, vous dessinez ensemble en direct et voyez les pointeurs des autres ; sinon, les modifications arrivent à la prochaine synchronisation. Le board est chiffré de bout en bout.',
        },
        geteilteKalender: {
          title: 'Partager des agendas',
          p1: 'Sous « Partages », « Partager » (à proximité : « Partager quelque chose ») te permet aussi de partager l’un de tes agendas avec le hub. Il apparaît alors dans la liste sous « Agendas » ; un appui l’ouvre dans l’agenda – chez les membres sous « Des hubs ». Avec « voir », ils le lisent seulement ; avec « modifier », ils ajoutent, changent et suppriment des rendez-vous. Toi seul modifies l’agenda lui-même. Si deux personnes modifient le même rendez-vous en même temps, la seconde version reste à côté en copie, avec « Conflit », la date et l’heure dans le titre. Tout est chiffré avec une clé propre. Le X à côté de l’agenda arrête le partage ; ton agenda reste. Sans openany.de, les membres le reçoivent dès qu’ils se synchronisent à proximité – en lecture seule ; les agendas abonnés ne se partagent que de cette façon.',
        },
        freigaben: {
          title: 'Partager des contenus',
          p1: 'Dans l’onglet « Partages », « Partager » partage avec le hub un dossier de notes, un album ou un dossier (ou classeur) : choisis d’abord le type, puis l’élément. Tout reste chiffré de bout en bout – avec une clé propre à chaque partage, qui change quand tu arrêtes de partager ou que quelqu’un quitte le hub. Les contenus restent chez toi ; quand le partage prend fin, tu gardes tout.',
          p2: 'Les albums et dossiers sont partagés en consultation seulement : les membres les ouvrent dans le hub, voient sous-albums et sous-dossiers, ouvrent images et PDF et, avec « Reprendre », en mettent une copie dans leur propre stockage sous « Des hubs ». Toi seul peux les modifier.',
          p3: 'Les dossiers de notes se partagent avec le niveau « consulter » ou « modifier ». Les membres les ouvrent dans l’éditeur de notes habituel et voient aussi les images qu’ils contiennent ; avec « modifier », ils changent les notes et en créent. Pour l’instant, seul le propriétaire du dossier peut insérer des images.',
        },
        lokal: {
          title: 'Sur openany.de ou uniquement sur place',
          p1: '« + Hub » crée un hub sur cet appareil. Si l’appareil est connecté à openany.de, il y va aussitôt aussi – il porte alors un petit nuage dans la liste. Qui a fondé un hub l’active et le désactive sous Réglages → Synchronisation → « Hubs sur openany.de ». Désactivé, il ne se synchronise plus que sur place ; sur openany.de, le hub, le chat, la planification et les partages sont supprimés. Avec le nuage, tu invites les membres par leur nom openany (propriétaires seulement, et seulement des personnes qui ont déjà synchronisé l’app avec openany.de). Sans nuage, tu invites sous « Inviter un membre à proximité » : Openany doit être ouvert sur l’autre appareil, sur le même Wi-Fi ou point d’accès, et les deux appareils affichent les mêmes six chiffres, que vous comparez et confirmez. Le hub est géré par les appareils du propriétaire ; s’il n’y en a qu’un, l’app prévient – associe d’abord un deuxième appareil à toi. Quand tu supprimes un hub, cela est d’abord signé dans sa liste des membres ; c’est seulement ainsi que les appareils des autres savent qu’il n’existe plus. Si openany.de signale un hub comme supprimé sans cette trace, il reste sur ton appareil et continue de se synchroniser à proximité.',
          p2: 'Les membres partagent leurs propres classeurs de notes, dossiers et albums dans le hub sous « Partager quelque chose », d’abord en lecture seule. Les classeurs de notes peuvent aussi être partagés en modification : une seule personne modifie alors une note à la fois, et l’enregistrement se fait sur l’appareil de la personne à qui appartient le classeur – s’il n’est pas à proximité, la note reste en lecture seule. Le chat atteint aussitôt toutes les personnes à proximité, les autres à la prochaine synchronisation. Le board voyage de la même manière.',
          p3: 'Tu synchronises le reste avec « Synchroniser » dès qu’un membre est à proximité. Le propriétaire peut retirer des membres ; qui veut partir le fait avec « Quitter », ce qui ne marche que si un autre membre est à proximité pour en être informé. Ce que les partants avaient partagé disparaît chez les autres ; ce qu’ils avaient déjà reste sur leurs appareils.',
        },
      },
    },
    messages: {
      title: 'Messages',
      subs: {
        direkt: {
          title: 'Messages directs',
          p1: 'Tu ouvres tes messages depuis le menu. Avec « Message » en haut à droite, tu écris directement à n\'importe quel autre utilisateur – son nom Openany suffit. Les liens dans les messages sont cliquables.',
          p2: 'Sous chaque message reçu, « Répondre » ouvre l\'éditeur avec l\'expéditeur déjà renseigné, par le même canal que le message. Les longs messages sont repliés ; « Lire la suite » les affiche en entier. Le compteur de non-lus sur le bouton de menu indique si quelque chose de nouveau t\'attend.',
          p3: 'Les messages directs sont chiffrés de bout en bout : openany.de ne stocke que des données illisibles, seuls tes appareils peuvent les lire. Tous tes appareils partagent une clé. Un nouvel appareil affiche 6 chiffres ; tu les saisis sur un appareil déjà configuré, et il est connecté et reçoit la clé. La clé se trouve aussi dans ton fichier de sauvegarde. Si tu perds un appareil, retire-le dans les réglages sous « Mes appareils » ; la clé change alors. Tu peux écrire à toute personne qui a ouvert l\'application au moins une fois.',
        },
        suche: {
          title: 'Rechercher et filtrer',
          p1: 'Au-dessus du fil se trouve un champ de recherche. Il trouve les messages par leur texte, par le nom de l’interlocuteur et par identifiant Matrix, et met les résultats en évidence. En dessous, des boutons restreignent la liste : aux messages non lus, à ceux avec pièce jointe et – si tu utilises plus d’une voie – à certaines voies comme Openany ou Matrix. Ces boutons se combinent.',
          p2: 'À côté du champ de recherche, tu passes d’une vue à l’autre : « Fil » montre tous les messages dans l’ordre chronologique, « Par contact » les regroupe en une ligne par interlocuteur, avec le dernier message et le nombre de non-lus. Un clic sur une ligne affiche le fil avec cette personne précise ; « Fermer la conversation » te ramène à tous.',
        },
        anhaenge: {
          title: 'Pièces jointes',
          p1: 'Par Matrix, tu envoies aussi des fichiers : le trombone du champ de saisie joint un fichier jusqu’à 10 Mo, et sur téléphone et tablette le bouton appareil photo à côté prend directement une photo. Dans le fil, les images apparaissent en aperçu, les autres fichiers avec nom et taille ; un clic ouvre images et PDF directement dans Openany, le reste se télécharge. Par la voie Openany passent des fichiers jusqu’à 10 Mo, chiffrés de bout en bout comme le texte. Ils comptent dans ton stockage et dans celui du destinataire jusqu’à ce que chacun supprime le message.',
        },
        matrixApp: {
          title: 'Messages via Matrix',
          p1: 'Dans l’application, ton appareil est lui-même l’appareil Matrix. Tu te connectes avec un compte existant dans les paramètres, sous « Comptes Matrix », et tes messages sont chiffrés de bout en bout entre cet appareil et ton interlocuteur – openany.de ne lit pas. Pour que d’autres applications Matrix comme Element fassent confiance à cet appareil, vérifie-le là, au compte : avec la clé de récupération d’Element ou – si ton compte n’a pas encore de sauvegarde – openany en crée une et te montre la nouvelle clé une fois.',
          p2: 'Tu peux connecter plusieurs comptes Matrix. Leurs messages partagent un même fil ; les nouveaux partent du compte par défaut, les réponses du compte sur lequel le message est arrivé, et « Définir par défaut » fait d’un autre compte le compte par défaut. Si tu supprimes un message Matrix, il disparaît seulement sur cet appareil ; il reste dans le salon.',
        },
        email: {
          title: 'E-mail via ta boîte',
          p1: 'Dans l’application, l’e-mail s’ajoute comme voie supplémentaire – pas un logiciel de messagerie à part, mais ta boîte existante, par exemple chez Posteo, mailbox.org ou GMX. Dans les paramètres, sous « Boîtes mail », tu saisis adresse et mot de passe ; le plus souvent un mot de passe d’application créé au préalable chez ton fournisseur. L’application trouve les serveurs elle-même ; si elle n’en trouve pas, tu les saisis sous « Serveurs à la main ». Le mot de passe reste sous clé sur cet appareil, et l’appareil parle lui-même au serveur de messagerie – openany.de ne voit aucun e-mail.',
          p2: 'Les e-mails apparaissent avec leur objet dans le fil commun des Messages ; « Relever maintenant » récupère aussitôt les nouveaux. Si tu connectes plusieurs boîtes, les nouveaux e-mails partent de la boîte par défaut et les réponses de celle qui a reçu l’e-mail ; en écrivant, tu choisis sous « De ». À la suppression, tu décides si un e-mail disparaît seulement ici ou passe aussi dans le dossier corbeille du serveur. Si ton fournisseur a classé quelque chose en spam, un avis apparaît au-dessus du fil ; là, « Pas un spam » renvoie l’e-mail dans la boîte de réception.',
          p3: 'Par e-mail, les pièces jointes peuvent aller jusqu’à 20 Mo. Les pièces jointes reçues vont dans tes fichiers avec « Enregistrer » ou dans le dossier de téléchargements avec « Sur l’appareil ». Si une pièce jointe est arrivée chiffrée, l’application demande d’abord : dans Fichiers, elle est stockée en clair et synchronisée avec openany.de.',
        },
        pgp: {
          title: 'E-mail chiffré avec OpenPGP',
          p1: 'Tu peux chiffrer tes e-mails de bout en bout avec OpenPGP. Pour cela, chaque boîte a besoin de sa propre clé : dans les paramètres, à la boîte, tu la crées avec « Créer une clé » – ou, si tu utilises déjà la boîte avec PGP, par exemple dans Thunderbird, tu importes la clé existante avec « Importer une clé », afin que les deux programmes lisent les mêmes e-mails. La clé secrète se trouve uniquement sur cet appareil. « Enregistrer une copie de sauvegarde » la dépose, verrouillée par une phrase secrète, dans le dossier de téléchargements ; « Enregistrer la clé publique » te donne le fichier à envoyer aux autres.',
          p2: 'Les clés publiques de tes contacts, l’application les rassemble d’elle-même sous « Clés des contacts », à partir des e-mails qui les joignent. S’il en manque une, tu la cherches par adresse – si tu le souhaites aussi sur keys.openpgp.org, qui apprend alors qui tu cherches – ou tu l’importes depuis un fichier. Compare de préférence une fois l’empreinte avec ton interlocuteur ; si une clé change, l’application te le signale.',
          p3: 'En écrivant, tu actives « Envoyer chiffré » ; en dessous, tu vois si la clé du destinataire est connue. Si tu réponds à un e-mail chiffré, l’option est déjà activée. Dans le fil, les e-mails portent la marque « chiffré » et, s’ils sont signés, « signé » – ou un avertissement si la signature ne correspond pas.',
        },
        vorOrt: {
          title: 'Messages à proximité',
          p1: 'Avec la voie « À proximité », tu écris directement aux appareils proches, sans aucun serveur. Openany doit être ouvert sur l’autre appareil, et les deux doivent être sur le même Wi-Fi. Vous pouvez vous écrire dès que vous vous connaissez : tes propres appareils, les membres d’un hub commun – ou les personnes que vous avez confirmées sur place. Pour cela, tu choisis « Confirmer sur place », les deux appareils affichent les mêmes 6 chiffres, et vous touchez tous deux « Correspond ». Si l’autre appareil n’est pas là, le message attend et part à la prochaine rencontre.',
          p2: 'Au début, rien n’arrive des inconnus. Si tu actives « Autoriser les demandes des appareils à proximité » dans les paramètres, sous « Appareils à proximité », ils peuvent t’envoyer jusqu’à 3 messages de 500 caractères chacun ; ceux-ci apparaissent sous « Demandes » au-dessus du fil, pas dans le fil lui-même. Tu ne peux répondre qu’après la confirmation avec les 6 chiffres. Qui ne doit plus rien t’envoyer, tu l’arrêtes avec « Bloquer ». Si l’un de tes messages a été refusé, il porte « non distribué », et la raison s’affiche quand tu le survoles.',
        },
        systemnachrichten: {
          title: 'Notifications et réponses',
          p1: 'Les événements de tes hubs – par exemple de nouveaux partages – t’arrivent sous forme d’avis si l’expéditeur en informe les membres ; de même l’avis que quelqu’un t’a ajouté à un hub, et les messages d’openany.de concernant ton compte, comme la demande de le confirmer. Les avis ne sont pas chiffrés de bout en bout. Tu trouves aussi ici la réponse à un message envoyé via le formulaire d’assistance.',
        },
      },
    },
    contacts: {
      title: 'Carnet d’adresses',
      subs: {
        adressbuch: {
          title: 'Contacts et leurs moyens',
          p1: 'Le carnet d’adresses s’ouvre depuis sa tuile sur l’accueil ou en haut de la page Messages ; le bouton à côté crée directement un nouveau contact. Un contact indique comment joindre quelqu’un : nom, image, téléphone et e-mail ainsi que d’autres moyens à volonté – Matrix, Meshtastic, nom Openany, adresse postale, entreprise, anniversaire ou autre, chacun avec son libellé comme perso ou travail. Le champ de recherche trouve les contacts par nom.',
          p2: 'Les moyens sont cliquables : un numéro de téléphone appelle, une adresse e-mail ouvre le programme de messagerie, un nom Openany ou un identifiant Matrix ouvre le champ d’écriture avec le destinataire déjà rempli. Les contacts supprimés vont dans la corbeille.',
        },
        verschluesselt: {
          title: 'Chiffré de bout en bout',
          p1: 'Ton carnet d’adresses est chiffré de bout en bout, avec la même clé que tes messages directs. Seuls tes appareils autorisés peuvent le lire ; openany.de le conserve et le synchronise, mais ne voit ni noms, ni numéros, ni images. Un nouvel appareil n’affiche le carnet d’adresses qu’une fois autorisé sur un autre appareil.',
        },
        vcard: {
          title: 'Enregistrer et importer des vCard',
          p1: 'Dans Paramètres → Carnet d’adresses, tu enregistres tous les contacts dans un fichier vCard ou tu en importes un d’un autre programme. L’import ne fait qu’ajouter : les contacts existants sont reconnus par leur identifiant ou leur nom et gardent ce qu’ils ont. Le fichier lui-même n’est pas chiffré – conserve-le en conséquence.',
        },
        imTelefon: {
          title: "Dans les contacts du téléphone",
          p1: "Sous Android, Paramètres → Carnet d’adresses → « Afficher dans les contacts du téléphone » montre tes contacts dans le carnet du téléphone, sous le compte « openany » – quand quelqu’un appelle, son nom s’affiche. Nom, téléphone, e-mail, adresse, entreprise, anniversaire, site web, surnom et photo passent ; ce que tu y modifies, crées ou supprimes revient. Le téléphone ne connaît pas les identifiants Matrix, Meshtastic ou Openany : ils restent seulement dans openany. Une photo modifiée là-bas est rétablie par openany. Désactiver retire les contacts de là-bas.",
        },
        andereProgramme: {
          title: "Pour d’autres programmes sur l’ordinateur",
          p1: "Sur ordinateur, Thunderbird ou Contacts d’Apple, par exemple, lisent et écrivent ton carnet d’adresses via CardDAV – activé sous Paramètres → Calendrier, avec les mêmes informations que pour les agendas (voir là). Tous les moyens de contact voyagent avec, y compris Matrix, Meshtastic et le nom Openany.",
        },
        sicherheitsnummer: {
          title: 'Numéro de sécurité',
          p1: 'Le numéro de sécurité te permet de vérifier que personne ne lit entre vous – pas même openany.de. Ouvre-le dans le contact, sous « Sécurité », ou dans les messages après avoir ouvert une conversation depuis « Par contact ». Comparez les douze groupes de chiffres sur vos deux appareils ; s’ils correspondent, touche « Ça correspond ». Les messages de cette personne portent alors une coche à côté du cadenas. Si sa clé change plus tard, par exemple après la perte d’un appareil, la coche disparaît et un avis vous invite à comparer de nouveau.',
        },
      },
    },
    settings: {
      title: 'Paramètres et compte',
      subs: {
        module: {
          title: 'Modules sur l’accueil',
          p1: 'Ici, tu choisis quels modules apparaissent en tuile sur l’accueil – Notes, Agenda, Stockage, Hubs et Carnet d’adresses. La sélection ne masque rien ; tous les modules restent accessibles. Sans sélection, l’accueil affiche la page de bienvenue.',
        },
        spracheDesign: {
          title: 'Apparence et langue',
          p1: 'Sous « Apparence », tu choisis entre deux designs : « Net », bleu pétrole avec des angles plus marqués et une surface calme, ou « Lilas », avec des angles arrondis et un fond à motif. Les deux existent en clair et en sombre – ce que tu règles indépendamment avec le bouton de l’en-tête ou du menu mobile. Sous « Langue », tu choisis la langue dans laquelle Openany te parle : allemand, anglais, espagnol, français, portugais, polonais ou ukrainien. Tant que tu n’as pas choisi, il suit la langue de ton appareil ou de ton navigateur ; si elle n’est pas disponible, Openany parle anglais.',
        },
        abgleich: {
          title: 'Synchroniser avec Openany',
          p1: 'Sous « Synchronisation », tu connectes l’app à openany.de. Si tu n’as pas encore de compte, crée-en un avec « Créer un compte » (voir « Créer un compte »). Si tu as déjà un appareil connecté, tu y autorises le nouveau avec 6 chiffres (voir « Mes appareils »). Un ancien compte avec mot de passe se connecte avec « Se connecter à openany.de » : l’app affiche un code que tu saisis et confirmes sur la page d’anyid dans le navigateur. La connexion est facultative : sans connexion, tout reste sur cet appareil, et « Se déconnecter » l’annule à tout moment sans perte de données ici.',
          p2: 'Ensuite, « Synchroniser » synchronise dans les deux sens et indique ce qui a été récupéré, envoyé ou ignoré. « Tout récupérer à nouveau » recharge tout le contenu au lieu du seul nouveau ; rien n’est supprimé.',
          p3: 'Avec « Actualiser en arrière-plan », l’application se synchronise d’elle-même environ toutes les heures – seulement avec une connexion et pas avec une batterie faible. En données mobiles, seule la liste arrive ; fichiers et images seulement en Wi-Fi. En Wi-Fi, elle essaie aussi avec les appareils à proximité associés si Openany y est ouvert. Sur ordinateur, Openany se synchronise toutes les heures tant qu’il est ouvert ; avec « Continuer en arrière-plan », il démarre à l’ouverture de session et reste sous forme d’icône dans la barre de menus quand tu fermes la fenêtre.',
        },
        geraeteNah: {
          title: 'Appareils à proximité',
          p1: 'Tes propres appareils se synchronisent aussi directement entre eux, sans serveur – tant qu’ils sont sur le même Wi-Fi ou point d’accès et qu’Openany est ouvert sur les deux. Sous « Appareils à proximité », tu les trouves avec « Rechercher » et les relies avec « Associer » : les deux appareils affichent le même code, et sur chacun tu confirmes qu’il correspond. Dès lors, ils se synchronisent d’eux-mêmes tant qu’Openany est ouvert sur les deux, et à chaque « Synchroniser » ; « Oublier » annule l’association.',
          p2: 'Sous « Ton nom », tu choisis comment tu apparais aux autres sur place. C’est sous ce nom qu’on t’invite dans des hubs – toi en tant que personne, pas un appareil précis ; il vaut pour tous tes appareils associés.',
        },
        meineGeraete: {
          title: 'Mes appareils',
          p1: 'Ce que tu chiffres de bout en bout ne peut être lu que par tes appareils ; ils partagent pour cela une clé. Un nouvel appareil affiche 6 chiffres, et tu les saisis ici sur un appareil déjà configuré. Si un appareil est perdu, tu le retires ici ; la clé change alors. La clé se trouve aussi dans ton fichier de sauvegarde.',
          p2: 'Tout en bas de « Mes appareils », tu supprimes toi-même ton compte sur openany.de – immédiatement et avec tout ce qui s’y trouve. Par sécurité, tu saisis ton nom d’utilisateur. Les hubs qui ont encore d’autres membres, tu les transmets d’abord à quelqu’un.',
        },
        fristen: {
          title: 'Confirmer ton compte et rester actif',
          p1: 'Pour que des comptes inutilisés ne s’accumulent pas, openany.de te demande deux jours après la création de confirmer ton compte encore une fois ; une bande avec « Garder le compte » apparaît en haut. Si tu ne confirmes pas dans les 14 jours, le compte est supprimé.',
          p2: 'Si tu n’ouvres pas l’app pendant quatre mois, la même demande revient, avec un dernier rappel deux semaines avant la suppression. Un appui sur « Garder le compte » suffit. Quand un compte est supprimé, seuls les contenus sur openany.de disparaissent ; ce qui se trouve sur tes appareils reste.',
        },
        speicherGeraet: {
          title: 'Stockage sur cet appareil',
          p1: 'Sous « Stockage sur cet appareil », tu décides de ce qui reste ici. « Tout garder » récupère tout ce qui se trouve dans tes albums, dossiers et classeurs tant qu’il y a de la place ; c’est prévu pour les ordinateurs et les tablettes. Avec « Albums, dossiers et classeurs sélectionnés », seul ce que tu choisis dans Stockage avec « Garder sur cet appareil » reste ici ; du reste, tu ne vois que le nom. Les nouvelles photos de l’appareil photo vont dans l’album « Appareil photo ». Ce qui ne se trouve ni dans un album, ni dans un dossier ou un classeur reste sur son appareil et n’est jamais synchronisé.',
          p2: 'Vont sur le serveur les notes, les calendriers, les contacts, les messages et les noms des albums, dossiers et classeurs. Les photos et les fichiers n’y vont que s’ils sont partagés dans un hub, joints à une note, ou si tu choisis « Mettre sur le serveur », un par un ou pour tout un album, dossier ou classeur. Tout le reste se synchronise directement entre tes appareils. « Retirer du serveur » ne supprime rien : tes appareils gardent leurs copies, et la dernière copie reste toujours.',
          p3: 'Sous « Stockage sur cet appareil », tu vois aussi ce qui se trouve sur le serveur, réparti entre messages, notes, médiathèque, fichiers et documents.',
        },
        sofort: {
          title: 'Notifier immédiatement',
          p1: 'Avec « Notifier immédiatement », l’application maintient des connexions économes vers Openany et vers tes boîtes mail, pour que les nouveaux messages et e-mails arrivent aussitôt – sans Google. Les boîtes mail n’ont pas besoin d’Openany pour cela. Android affiche pour cela une notification permanente, que tu peux masquer dans les paramètres du système. Sur ordinateur, Openany veille tant qu’il est ouvert et prévient par une notification du système.',
        },
        signal: {
          title: 'Signal de message',
          p1: 'Quand un nouveau message arrive alors qu’openany est ouvert, l’application joue un bref son – et, si tu le souhaites, l’appareil vibre. Si tu es déjà dans Messages, rien ne sonne, puisque tu le vois de toute façon. Son et vibration se règlent sous Paramètres → Messages → « Signal de message » ; cela vaut pour cet appareil seulement.',
          p2: 'Si openany est fermé ou en arrière-plan et que « Notifier immédiatement » est activé, une notification du système apparaît à la place. Son volume et sa vibration se règlent dans les paramètres système d’Android ; le bouton de la tuile y mène.',
        },
        sicherung: {
          title: 'Sauvegarde dans un fichier',
          p1: 'Sous « Sauvegarde », « Créer une sauvegarde » produit un fichier chiffré avec tout ce qu’Openany a sur cet appareil : notes, calendrier, contacts, fichiers, médiathèque, hubs, messages et tes boîtes mail avec mot de passe et ta propre clé OpenPGP. L’application propose une phrase secrète de six mots ; note-la et garde-la séparée du fichier. Sans elle, le fichier ne peut pas être ouvert – pas même par nous.',
          p2: '« Restaurer une sauvegarde » ouvre un tel fichier sur un autre appareil. Elle remplace tout ce qui s’y trouvait ; ensuite la connexion à openany.de est supprimée et tu associes de nouveau les appareils à proximité. Ce qui identifie un appareil n’est pas inclus.',
        },
        tresor: {
          title: 'Identifiants et clés dans le coffre',
          p1: 'Ce qui permet à cet appareil de s’identifier auprès d’Openany et d’autres appareils est conservé verrouillé sur l’appareil ; la clé reste dans le trousseau du système – le Keystore sous Android, et sur l’ordinateur le trousseau de macOS, le gestionnaire d’identification de Windows ou le Secret Service sous Linux. Les mots de passe de tes boîtes mail et tes clés OpenPGP secrètes s’y trouvent aussi. « Se déconnecter » ne retire les identifiants qu’ici – un appareil est révoqué définitivement sous « Mes appareils ».',
        },
      },
    },
    contact: {
      title: 'Signaler, règles et contact',
      subs: {
        kontakt: {
          title: 'Nous contacter',
          p1: 'Envoie tes questions, problèmes ou souhaits via le formulaire d’assistance tout en bas de cette aide ; il est chiffré de bout en bout jusqu’aux administrateurs, et tu trouves la réponse dans les messages. Sans compte, tu nous joins via le formulaire sur openany.de/signaler sous « Question ou demande ». Nous répondons en allemand ou en anglais.',
        },
        bedingungen: {
          title: 'Conditions d’utilisation et confidentialité',
          p1: 'Tu trouves les conditions d’utilisation et la politique de confidentialité sous « À propos d’openany » sous le formulaire d’assistance, et les mentions légales tout à la fin de cette aide ; les trois figurent aussi sur openany.de. En bref : openany est gratuit et sans publicité, tes contenus sont chiffrés sur tes appareils, et les serveurs sont en Allemagne.',
        },
        melden: {
          title: 'Signaler un contenu',
          p1: 'Tu signales des contenus ou des comptes qui enfreignent la loi ou les conditions d’utilisation via le formulaire sur openany.de/signaler – même sans compte. « Ouvrir le formulaire » au-dessus du formulaire d’assistance t’y mène dans la langue de l’app. Le signalement est chiffré dans ton navigateur ; seuls les administrateurs peuvent le lire.',
          p2: 'Après l’envoi, tu reçois un code d’accès. Avec lui, tu vois sur la même page l’état et notre réponse et tu peux poser des questions. Conserve-le bien – sans lui, tu ne peux plus accéder à ton signalement. Il est supprimé trois mois après le dernier message.',
        },
        entscheidungen: {
          title: 'Si nous suspendons ou supprimons ton compte',
          p1: 'Si un compte enfreint les conditions d’utilisation, les administrateurs peuvent le suspendre ou le supprimer. Si ton compte est suspendu, une bande rouge apparaît en haut ; un appui dessus indique depuis quand, sur quoi repose la décision et pourquoi. La synchronisation avec openany.de est alors suspendue ; ce qui se trouve sur tes appareils reste. Quand la suspension est levée, tout reprend comme avant.',
          p2: 'Si ton compte est supprimé, l’app affiche une fois la raison lors de la synchronisation suivante, puis se déconnecte ; l’app continue de fonctionner sans openany.de. Si tu n’es pas d’accord avec une décision, appuie sur « Contester » : cela ouvre le formulaire sur openany.de/signaler avec « Contestation d’une décision », et nous réexaminons la décision.',
        },
      },
    },
  },
};
